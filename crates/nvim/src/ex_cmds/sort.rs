//! `:sort` and `:uniq`.
//!
//! Both look at one *key* per line -- the whole line, or the part a `/pat/`
//! argument picked out of it, or the number or float parsed from that part --
//! and then either reorder the range by the key ([`ex_sort`]) or delete the
//! lines whose key repeats ([`ex_uniq`]).
//!
//! `:sort` builds the whole key array first, sorts it and appends the lines
//! back in the new order below the range before deleting the original.  The
//! line *text* stays in the memline -- only one line at a time is copied back
//! out of it -- while a text key, which the comparison needs over and over,
//! is copied once as the array is built.  `:uniq` never reorders, so it walks
//! the range once and deletes as it goes.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::cur_win;
use super::say;
use super::{MAXLNUM, RE_MAGIC, e_interr, e_invarg, e_noprevre, kExtmarkNOOP, kExtmarkUndo};
use crate::ascii::ascii_iswhite;
use crate::change::changed_lines;
use crate::charset::Str2NrBases;
use crate::charset::{skip, vim_str2nr};
use crate::cstr;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_docmd::check_nextcmd;
use crate::extmark::extmark_splice;
use crate::main::{curbuf, got_int, p_ic};
use crate::mark::mark_adjust;
use crate::memline::{Lines, ml_append, ml_delete};
use crate::message::emsg;
use crate::message_fmt::c_str;
use crate::os::cshim::gettext;
use crate::os::input::fast_breakcheck;
use crate::regexp::{skip_regexp_err, vim_regcomp, vim_regexec, vim_regfree};
use crate::search::last_search_pat;
use crate::semsg;
use crate::types::{
    ExtmarkOp, NUL, bcount_t, colnr_T, exarg_T, float_T, linenr_T, regmatch_T, size_t, varnumber_T,
};
use crate::undo::u_save;
use crate::winlayer::Buf;
use ::libc::{strcasecmp, strcoll, strtod};
use core::cmp::Ordering;
use core::ffi::{c_char, c_int};
use core::ptr;

/// How two keys are compared as *text*: `l` asks the locale, `i` folds case,
/// and `l` wins when both are given because upstream tests it first.  It is a
/// file static upstream only because `qsort`'s comparator cannot be handed
/// anything; here it is a value the flag parse answers with.
#[derive(Clone, Copy)]
struct StringOrder {
    locale: bool,
    ignore_case: bool,
}

impl StringOrder {
    /// The plain byte order, before any flag is read.
    const BYTES: StringOrder = StringOrder {
        locale: false,
        ignore_case: false,
    };

    /// Compare two NUL-terminated keys.
    ///
    /// # Safety
    /// Both must be NUL-terminated.
    unsafe fn compare(self, s1: *const c_char, s2: *const c_char) -> Ordering {
        // SAFETY: caller's contract.
        let sign = if self.locale {
            unsafe { strcoll(s1, s2) }
        } else if self.ignore_case {
            unsafe { strcasecmp(s1.cast_mut(), s2.cast_mut()) }
        } else {
            unsafe { cstr::cmp(s1, s2) as c_int }
        };
        sign.cmp(&0)
    }

    /// [`StringOrder::compare`] for two keys that carry their own NUL.
    fn compare_keys(self, k1: &Key, k2: &Key) -> Ordering {
        // SAFETY: a `Key` is NUL-terminated by construction.
        unsafe { self.compare(k1.as_ptr(), k2.as_ptr()) }
    }
}

/// One line's text key: the compared bytes plus the NUL `strcoll` and
/// `strcasecmp` need.  Built once per line, not once per comparison.
struct Key(Box<[u8]>);

impl Key {
    fn new(bytes: &[u8]) -> Key {
        let mut owned = Vec::with_capacity(bytes.len() + 1);
        owned.extend_from_slice(bytes);
        owned.push(NUL as u8);
        Key(owned.into_boxed_slice())
    }

    fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr().cast()
    }
}

/// A NUL-terminated scratch key, refilled once per line.
///
/// `:uniq` and `:sort u` both compare one line with the one before it, which
/// upstream does with two `xmalloc`ed buffers of the range's longest line and
/// a `strcpy`.  Two `Vec`s that swap say the same thing, keep their capacity,
/// and need no maximum measured up front.
#[derive(Default)]
struct Scratch(Vec<u8>);

impl Scratch {
    /// The empty string, so a comparison against a key that has not been
    /// filled in yet still reads one.
    fn empty() -> Scratch {
        Scratch(vec![NUL as u8])
    }

    fn fill(&mut self, bytes: &[u8]) {
        self.0.clear();
        self.0.extend_from_slice(bytes);
        self.0.push(NUL as u8);
    }

    fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr().cast()
    }
}

/// What one line is sorted on.  Every line of a range gets the same variant:
/// the flags decide which once, before the range is walked.
enum SortKey {
    /// The bytes of the line the comparison looks at.
    Text(Key),
    /// The integer parsed out of that range.  A line without one sorts before
    /// every line with one, which is what `Option`'s order says.
    Number(Option<varnumber_T>),
    /// The float parsed out of that range.
    Float(float_T),
}

/// One line of the range as `:sort` sees it.
struct SortLine {
    lnum: linenr_T,
    key: SortKey,
}

/// The order two lines sort in: a strict total order, which is what
/// [`slice::sort_by`] asks for and what upstream's `qsort` comparator already
/// was -- equal keys fall back on the line numbers, and no two lines share
/// one, so the sort's *stability* never comes into it.
fn compare_lines(order: StringOrder, l1: &SortLine, l2: &SortLine) -> Ordering {
    let keys = match (&l1.key, &l2.key) {
        (SortKey::Number(a), SortKey::Number(b)) => a.cmp(b),
        // Not `partial_cmp`: upstream tests `==` then `>`, so a NaN sorts
        // before everything, including itself.
        (SortKey::Float(a), SortKey::Float(b)) => {
            if a == b {
                Ordering::Equal
            } else if a > b {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        }
        (SortKey::Text(a), SortKey::Text(b)) => order.compare_keys(a, b),
        // Unreachable: every key has the same shape.  Falling back on the
        // line order keeps the comparison a total one either way.
        _ => Ordering::Equal,
    };
    // If two lines have the same value, preserve the original order.
    keys.then(l1.lnum.cmp(&l2.lnum))
}

/// A zeroed `regmatch_T`; only `regprog` is meaningful before a match.
fn no_regmatch() -> regmatch_T {
    regmatch_T {
        regprog: ptr::null_mut(),
        startp: [ptr::null_mut(); 10],
        endp: [ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    }
}

/// The `/pat/` argument both commands accept in place of a flag letter.
///
/// Returns the index of the closing delimiter, which upstream overwrites with
/// a NUL so the pattern is its own string -- the scan resumes one byte past
/// it.  `None` means it failed, with the reason already reported.
///
/// # Safety
/// `arg` must be the command's NUL-terminated argument and `at` must index
/// the opening delimiter.
unsafe fn compile_sort_pattern(
    arg: *mut c_char,
    at: usize,
    regmatch: &mut regmatch_T,
) -> Option<usize> {
    // SAFETY: caller's contract.
    let delim = unsafe { arg.add(at) };
    let end = unsafe { skip_regexp_err(delim.add(1), *delim as c_int, 1) };
    if end.is_null() {
        return None;
    }
    unsafe { *end = NUL as c_char };

    // Use the last search pattern if the sort pattern is empty.
    regmatch.regprog = if end == unsafe { delim.add(1) } {
        if last_search_pat().is_null() {
            emsg(gettext(e_noprevre));
            return None;
        }
        unsafe { vim_regcomp(last_search_pat(), RE_MAGIC) }
    } else {
        unsafe { vim_regcomp(delim.add(1), RE_MAGIC) }
    };
    if regmatch.regprog.is_null() {
        return None;
    }
    regmatch.rm_ic = p_ic.get() != 0;
    Some(unsafe { end.offset_from(arg) } as usize)
}

/// True for the letters that can only ever be flags, so that anything else is
/// a pattern delimiter.
fn is_alpha(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}

/// The flags `:sort` was given.
struct SortSpec {
    /// `u`: drop a line equal to the one before it.
    unique: bool,
    /// `r`: compare what the pattern matched, not what follows it.
    use_match: bool,
    /// The base `b`/`o`/`x` forced, or nothing for `n`'s decimal.
    radix: Str2NrBases,
    /// `n`/`b`/`o`/`x`: compare as integers.
    numeric: bool,
    /// `f`: compare as floats.
    float: bool,
}

impl SortSpec {
    /// Nothing asked for: a plain text sort of whole lines.
    fn new() -> SortSpec {
        SortSpec {
            unique: false,
            use_match: false,
            radix: Str2NrBases::NONE,
            numeric: false,
            float: false,
        }
    }
}

/// The NUL that ends the command's argument.
const NUL_BYTE: u8 = NUL as u8;

/// What one byte of a `:sort`/`:uniq` argument turned out to be.  Both
/// commands share the text-order letters and the `r` flag; the rest is each
/// command's own.
enum Flag {
    /// A letter both commands understand, already applied.
    Shared,
    /// The end of the argument: a NUL or a `"` comment.
    End,
    /// Neither: `at` is where the pattern, the next command, or the error is.
    Other,
}

/// What [`flag_fallback`] made of a byte that was no flag letter.
enum Fallback {
    /// A following command, which ends the argument.
    NextCmd,
    /// A `/pat/` whose closing delimiter is at this index.
    Pattern(usize),
}

/// Apply the flag letters `:sort` and `:uniq` have in common.
fn shared_flag(byte: u8, order: &mut StringOrder, use_match: &mut bool) -> Flag {
    match byte {
        NUL_BYTE | b'"' => Flag::End,
        b'i' => {
            order.ignore_case = true;
            Flag::Shared
        }
        b'l' => {
            order.locale = true;
            Flag::Shared
        }
        b'r' => {
            *use_match = true;
            Flag::Shared
        }
        _ if ascii_iswhite(byte as c_int) => Flag::Shared,
        _ => Flag::Other,
    }
}

/// What a byte that is not a flag letter means: the start of a `/pat/`, a
/// following command, or an error already reported (`None`).  `:uniq` keeps
/// the *first* following command it sees where `:sort` overwrites -- the one
/// thing the two scans do not share.
///
/// # Safety
/// `arg` must be the command's NUL-terminated argument and `at` must index
/// one of its bytes.
unsafe fn flag_fallback(
    eap: &mut exarg_T,
    at: usize,
    byte: u8,
    regmatch: &mut regmatch_T,
    keep_nextcmd: bool,
) -> Option<Fallback> {
    let arg = eap.arg;
    // SAFETY: caller's contract.
    let next = unsafe { check_nextcmd(arg.add(at)) };
    if !next.is_null() && !(keep_nextcmd && !eap.nextcmd.is_null()) {
        eap.nextcmd = next;
        return Some(Fallback::NextCmd);
    }
    if is_alpha(byte) || !regmatch.regprog.is_null() {
        // SAFETY: as above.
        let arg0 = unsafe { c_str(arg.add(at)) };
        semsg!("E475: Invalid argument: {arg0}");
        return None;
    }
    // SAFETY: as above.
    unsafe { compile_sort_pattern(arg, at, regmatch) }.map(Fallback::Pattern)
}

/// Read `:sort`'s flags and its optional pattern.  `None` means the command
/// was rejected, with the reason already reported.
///
/// # Safety
/// `eap.arg` must be the command's NUL-terminated argument.
unsafe fn parse_sort_flags(
    eap: &mut exarg_T,
    spec: &mut SortSpec,
    regmatch: &mut regmatch_T,
) -> Option<StringOrder> {
    let arg = eap.arg;
    let mut order = StringOrder::BYTES;
    // Only one of 'n', 'b', 'o', 'f' and 'x' is allowed.
    let mut formats = 0;
    let mut at = 0;

    loop {
        // The pattern arm rewrites the argument as it goes, so each byte is
        // read from the buffer rather than from a slice taken up front.
        // SAFETY: `at` has not passed the argument's NUL.
        let byte = unsafe { *arg.add(at) } as u8;
        let mut radix = |base| {
            spec.radix = base;
            formats += 1;
        };
        match byte {
            b'n' => {
                spec.numeric = true;
                formats += 1;
            }
            b'f' => {
                spec.float = true;
                formats += 1;
            }
            b'b' => radix(Str2NrBases::BIN | Str2NrBases::FORCE),
            b'o' => radix(Str2NrBases::OCT | Str2NrBases::FORCE),
            b'x' => radix(Str2NrBases::HEX | Str2NrBases::FORCE),
            b'u' => spec.unique = true,
            _ => match shared_flag(byte, &mut order, &mut spec.use_match) {
                Flag::Shared => {}
                Flag::End => break,
                // SAFETY: `at` indexes the argument's own bytes.
                Flag::Other => match unsafe { flag_fallback(eap, at, byte, regmatch, false) }? {
                    Fallback::NextCmd => break,
                    Fallback::Pattern(end) => at = end,
                },
            },
        }
        at += 1;
    }

    if formats > 1 {
        emsg(gettext(e_invarg));
        return None;
    }
    // From here on "numeric" covers every integer format.
    spec.numeric |= !spec.radix.is_empty();
    Some(order)
}

/// Read `:uniq`'s flags and its optional pattern.  `None` means the command
/// was rejected, with the reason already reported.
///
/// # Safety
/// `eap.arg` must be the command's NUL-terminated argument.
unsafe fn parse_uniq_flags(
    eap: &mut exarg_T,
    mode: &mut UniqMode,
    use_match: &mut bool,
    regmatch: &mut regmatch_T,
) -> Option<StringOrder> {
    let arg = eap.arg;
    let mut order = StringOrder::BYTES;
    let mut at = 0;

    loop {
        // SAFETY: `at` has not passed the argument's NUL.
        let byte = unsafe { *arg.add(at) } as u8;
        match byte {
            // 'u' is only valid when '!' is not given.
            b'u' => {
                if *mode != UniqMode::OnlyRepeated {
                    *mode = UniqMode::OnlyUnique;
                }
            }
            _ => match shared_flag(byte, &mut order, use_match) {
                Flag::Shared => {}
                Flag::End => break,
                // SAFETY: `at` indexes the argument's own bytes.
                Flag::Other => match unsafe { flag_fallback(eap, at, byte, regmatch, true) }? {
                    Fallback::NextCmd => break,
                    Fallback::Pattern(end) => at = end,
                },
            },
        }
        at += 1;
    }

    Some(order)
}

/// The part of `line` the comparison should look at: the whole line when
/// there is no pattern, what the pattern matched under `r`, what follows the
/// match otherwise -- and nothing at all for a line the pattern misses.
///
/// # Safety
/// `line` must be a buffer line, so that the byte past its last is a NUL:
/// `vim_regexec` reads a string, not a slice.
unsafe fn match_range(
    regmatch: &mut regmatch_T,
    line: &mut [u8],
    use_match: bool,
) -> (colnr_T, colnr_T) {
    let len = line.len() as colnr_T;
    if regmatch.regprog.is_null() {
        return (0, len);
    }
    let base = line.as_mut_ptr().cast::<c_char>();
    // SAFETY: caller's contract.
    if !unsafe { vim_regexec(regmatch, base, 0) } {
        return (0, 0);
    }
    // SAFETY: both are positions inside the line just matched.
    let start = unsafe { regmatch.startp[0].offset_from(base) } as colnr_T;
    let end = unsafe { regmatch.endp[0].offset_from(base) } as colnr_T;
    if use_match { (start, end) } else { (end, len) }
}

/// The number or float in `line[start..end]`.
///
/// The line is temporarily terminated at `end` so that `vim_str2nr` and
/// `strtod` cannot read digits past the match -- unless the range already
/// ends at the line's own NUL, where there is nothing to terminate.
///
/// # Safety
/// `line` must be a buffer line and `start <= end <= line.len()`.
unsafe fn number_key(line: &mut [u8], start: colnr_T, end: colnr_T, spec: &SortSpec) -> SortKey {
    let (start, end) = (start as usize, end as usize);
    // Terminate the key so the C parsers stop there.  When the range already
    // reaches the line's last byte, the NUL past it is the line's own.
    let saved = line.get(end).copied();
    if let Some(byte) = line.get_mut(end) {
        *byte = NUL_BYTE;
    }
    // The slice stops where the NUL does: a skip over the *slice* would walk
    // straight past a terminator that only a pointer walk can see.
    let from = &line[start..end];

    let key = if spec.float {
        let mut at = skip::white(from);
        if from.get(at) == Some(&b'+') {
            at += 1 + skip::white(&from[at + 1..]);
        }
        // An empty key sorts before any number.
        SortKey::Float(if at == from.len() {
            -float_T::MAX
        } else {
            // SAFETY: `from` is followed by a NUL, written above or the
            // line's own.
            unsafe { strtod(from[at..].as_ptr().cast(), ptr::null_mut()) }
        })
    } else {
        let mut at = if spec.radix.has(Str2NrBases::HEX) {
            skip::to_hex(from)
        } else if spec.radix.has(Str2NrBases::BIN) {
            skip::to_bin(from)
        } else {
            skip::to_digit(from)
        };
        // Include a preceding negative sign -- even when no digit was found
        // at all, which is how upstream reads a bare `-` as zero.
        if at > 0 && from[at - 1] == b'-' {
            at -= 1;
        }
        if at == from.len() {
            // A line without a number sorts before any number.
            SortKey::Number(None)
        } else {
            let mut value: varnumber_T = 0;
            // SAFETY: as above.
            unsafe {
                vim_str2nr(
                    from[at..].as_ptr().cast(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    spec.radix,
                    &mut value,
                    ptr::null_mut(),
                    0,
                    false,
                    ptr::null_mut(),
                )
            };
            SortKey::Number(Some(value))
        }
    };

    if let (Some(byte), Some(saved)) = (line.get_mut(end), saved) {
        *byte = saved;
    }
    key
}

/// Build one [`SortLine`] per line of the range.
///
/// `None` means the user interrupted the scan.  Doing the pattern match, the
/// number conversion and the text key's one copy here is what keeps them out
/// of the comparison.
///
/// # Safety
/// The range must be lines of the current buffer, and nothing may change the
/// buffer while the scan runs.
unsafe fn collect_sort_keys(
    line1: linenr_T,
    line2: linenr_T,
    spec: &SortSpec,
    regmatch: &mut regmatch_T,
) -> Option<Vec<SortLine>> {
    let mut sorted = Vec::with_capacity((line2 - line1 + 1) as usize);
    // SAFETY: caller's contract -- the scan reads one line at a time and
    // changes nothing.
    let mut lines = unsafe { Lines::current() };

    for lnum in line1..=line2 {
        let text = lines.line_mut(lnum);
        // SAFETY: a buffer line is NUL-terminated past its last byte.
        let (start, end) = unsafe { match_range(regmatch, text, spec.use_match) };
        let key = if spec.numeric || spec.float {
            // SAFETY: as above; `match_range` answers offsets into `text`.
            unsafe { number_key(text, start, end, spec) }
        } else {
            SortKey::Text(Key::new(&text[start as usize..end as usize]))
        };
        sorted.push(SortLine { lnum, key });

        if !regmatch.regprog.is_null() {
            fast_breakcheck();
        }
        if got_int.get() {
            return None;
        }
    }
    Some(sorted)
}

/// How much of the range `:sort` put back, and what it cost.
struct Placed {
    /// How many of the range's lines the walk got through; short of the
    /// range's length only when an append failed.
    done: size_t,
    /// The line below the last one appended.
    lnum: linenr_T,
    /// What was read and what was written, for the extmark splice.
    old_bytes: bcount_t,
    new_bytes: bcount_t,
    /// A line ended up somewhere other than where it started.
    moved: bool,
    /// The user interrupted the walk.
    interrupted: bool,
}

/// Append the range's lines below it in the sorted order.
///
/// # Safety
/// Every `lnum` in `sorted` must still be a line of the current buffer.
unsafe fn append_sorted(
    sorted: &[SortLine],
    order: StringOrder,
    unique: bool,
    reverse: bool,
    line2: linenr_T,
) -> Placed {
    let count = sorted.len() as size_t;
    let mut placed = Placed {
        done: 0,
        lnum: line2,
        old_bytes: 0,
        new_bytes: 0,
        moved: false,
        interrupted: false,
    };
    // The previously appended line, which `u` compares the next one with.
    let (mut previous, mut current) = (Scratch::empty(), Scratch::default());

    while placed.done < count {
        let from = if reverse {
            count - placed.done - 1
        } else {
            placed.done
        };
        let get_lnum = sorted[from].lnum;

        // If the original line number of the line being placed is not the
        // same as "lnum" (accounting for offset), the buffer changed.
        if get_lnum + (count as linenr_T - 1) != placed.lnum {
            placed.moved = true;
        }

        // Copy the line out of the memline: `ml_append` may invalidate it,
        // and "unique" needs it next time round.
        // SAFETY: caller's contract, and the handle dies before the append.
        let bytelen = {
            let mut lines = unsafe { Lines::current() };
            let text = lines.line(get_lnum);
            current.fill(text);
            // Include the EOL in the byte length.
            (text.len() + 1) as bcount_t
        };
        placed.old_bytes += bytelen;

        // SAFETY: both scratch buffers are NUL-terminated.
        let duplicate = unique
            && placed.done > 0
            && unsafe { order.compare(current.as_ptr(), previous.as_ptr()) } == Ordering::Equal;
        if !duplicate {
            // SAFETY: the text is this call's own NUL-terminated copy.
            let failed =
                unsafe { ml_append(placed.lnum, current.as_ptr().cast_mut(), 0, false) }.is_err();
            placed.lnum += 1;
            if failed {
                return placed;
            }
            placed.new_bytes += bytelen;
            core::mem::swap(&mut previous, &mut current);
        }

        fast_breakcheck();
        if got_int.get() {
            placed.interrupted = true;
            return placed;
        }
        placed.done += 1;
    }
    placed
}

/// `:sort`.
///
/// # Safety
/// `eap` must be a live Ex command whose range is inside the current buffer.
pub unsafe fn ex_sort(eap: *mut exarg_T) {
    // SAFETY: caller's contract.  The dispatcher's `exarg_T` outlives the
    // command and is reached through no other pointer while it runs.
    unsafe { sort_range(&mut *eap) };
}

/// `:sort`, with the command's argument block borrowed.
///
/// # Safety
/// `eap`'s range must be inside the current buffer.
unsafe fn sort_range(eap: &mut exarg_T) {
    let (forceit, line1, line2) = (eap.forceit, eap.line1, eap.line2);

    // Sorting one line is really quick!
    if line2 - line1 < 1 {
        return;
    }
    // SAFETY: the range is inside the current buffer.
    if u_save(line1 - 1, line2 + 1).is_err() {
        return;
    }

    let mut regmatch = no_regmatch();
    let mut spec = SortSpec::new();
    let mut count = (line2 - line1) as size_t + 1;

    'sortend: {
        // SAFETY: `eap.arg` is the command's own argument.
        let Some(order) = (unsafe { parse_sort_flags(eap, &mut spec, &mut regmatch) }) else {
            break 'sortend;
        };
        // SAFETY: the range is inside the current buffer and the scan
        // changes nothing.
        let Some(mut sorted) = (unsafe { collect_sort_keys(line1, line2, &spec, &mut regmatch) })
        else {
            break 'sortend;
        };

        // Note: can't be interrupted!
        sorted.sort_by(|l1, l2| compare_lines(order, l1, l2));

        // Insert the lines in the sorted order below the last one.
        // SAFETY: the range is still there, above where the copies go.
        let placed = unsafe { append_sorted(&sorted, order, spec.unique, forceit != 0, line2) };
        if placed.interrupted {
            break 'sortend;
        }

        // Delete the original lines if appending worked.
        if placed.done == count {
            for _ in 0..count {
                // SAFETY: the range is still there, above the new lines.
                let _ = unsafe { ml_delete(line1) };
            }
        } else {
            count = 0;
        }

        // SAFETY: the range is the one just rewritten.
        unsafe { finish_sort(line1, line2, count, &placed) };
    }

    // SAFETY: the program is this command's own.
    unsafe { vim_regfree(regmatch.regprog) };
    if got_int.get() {
        emsg(gettext(e_interr));
    }
}

/// Adjust marks and extmarks for the lines `:sort` moved, and put the cursor
/// on the first of them.
///
/// # Safety
/// The range must be the one just rewritten.
unsafe fn finish_sort(line1: linenr_T, line2: linenr_T, count: size_t, placed: &Placed) {
    let lnum = placed.lnum;
    let deleted = count as linenr_T - (lnum - line2);
    // SAFETY: caller's contract.
    if deleted > 0 {
        unsafe {
            mark_adjust(
                line2 - deleted,
                line2,
                MAXLNUM as linenr_T,
                -deleted,
                kExtmarkNOOP,
            )
        };
        say::more(-deleted);
    } else if deleted < 0 {
        unsafe { mark_adjust(line2, MAXLNUM as linenr_T, -deleted, 0, kExtmarkNOOP) };
    }

    if placed.moved || deleted != 0 {
        // SAFETY: as above.
        unsafe {
            extmark_splice(
                curbuf.get(),
                line1 - 1,
                0,
                count as c_int,
                0,
                placed.old_bytes,
                lnum - line2,
                0,
                placed.new_bytes,
                kExtmarkUndo,
            )
        };
        unsafe { changed_lines(Buf::new(curbuf.get()), line1, 0, line2 + 1, -deleted, true) };
    }

    cur_win().w_cursor.lnum = line1;
    beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
}

/// Which lines `:uniq` keeps.
#[derive(Clone, Copy, PartialEq)]
enum UniqMode {
    /// The plain form: collapse each run of equal lines to its first.
    Dedup,
    /// `:uniq!`: keep only the lines that do repeat.
    OnlyRepeated,
    /// `:uniq u`: keep only the lines that do not.
    OnlyUnique,
}

/// The running state of `:uniq`'s single pass over the range.
struct UniqScan {
    /// The previous line was part of a run of equal lines.
    in_run: bool,
    /// The next line must be treated as *not* matching, because this step
    /// already deleted the line it would have been compared against.
    force_unmatch: bool,
    /// The last line `:uniq!` has already decided about.
    done_lnum: linenr_T,
}

impl UniqScan {
    /// Decide what one line means: the line to delete (zero for none), and
    /// whether its key becomes the one the next line is compared against.
    fn step(
        &mut self,
        mode: UniqMode,
        i: linenr_T,
        count: linenr_T,
        lnum: linenr_T,
        is_match: bool,
    ) -> (linenr_T, bool) {
        // The flag is cleared whether or not it had anything to override;
        // `&&` would short-circuit past that when `is_match` is already false.
        let forced = core::mem::replace(&mut self.force_unmatch, false);
        let is_match = is_match && !forced;
        match mode {
            UniqMode::Dedup if is_match => (lnum, false),
            UniqMode::Dedup => (0, true),

            UniqMode::OnlyRepeated if is_match => {
                self.done_lnum = lnum - 1;
                self.in_run = true;
                (lnum, false)
            }
            UniqMode::OnlyRepeated => {
                // The line before this one ended a run of one, so it is the
                // one to drop -- and this line has then lost the line it was
                // compared against.
                let delete = if i > 0 && !self.in_run && lnum - 1 > self.done_lnum {
                    self.force_unmatch = true;
                    lnum - 1
                } else if i >= count - 1 {
                    lnum
                } else {
                    0
                };
                self.in_run = false;
                (delete, true)
            }

            UniqMode::OnlyUnique if is_match => {
                let delete = if self.in_run { lnum } else { lnum - 1 };
                self.in_run = true;
                (delete, false)
            }
            UniqMode::OnlyUnique => {
                // Only reachable once the previous step deleted this line's
                // predecessor and reset the index to zero.
                let delete = if i == 0 && self.in_run { lnum } else { 0 };
                self.in_run = false;
                (delete, true)
            }
        }
    }
}

/// `:uniq`.
///
/// # Safety
/// `eap` must be a live Ex command whose range is inside the current buffer.
pub unsafe fn ex_uniq(eap: *mut exarg_T) {
    // SAFETY: caller's contract, as [`ex_sort`].
    unsafe { uniq_range(&mut *eap) };
}

/// `:uniq`, with the command's argument block borrowed.
///
/// # Safety
/// `eap`'s range must be inside the current buffer.
unsafe fn uniq_range(eap: &mut exarg_T) {
    let (forceit, line1, line2) = (eap.forceit, eap.line1, eap.line2);
    let mut count = line2 - line1 + 1;

    // Uniq one line is really quick!
    if count <= 1 {
        return;
    }
    // SAFETY: the range is inside the current buffer.
    if u_save(line1 - 1, line2 + 1).is_err() {
        return;
    }

    let mut regmatch = no_regmatch();
    let mut mode = if forceit != 0 {
        UniqMode::OnlyRepeated
    } else {
        UniqMode::Dedup
    };
    let mut use_match = false;
    let mut change_occurred = false;
    let mut deleted = 0;

    'uniqend: {
        // SAFETY: `eap.arg` is the command's own argument.
        let Some(order) =
            (unsafe { parse_uniq_flags(eap, &mut mode, &mut use_match, &mut regmatch) })
        else {
            break 'uniqend;
        };

        let mut scan = UniqScan {
            in_run: false,
            force_unmatch: false,
            done_lnum: line1 - 1,
        };
        // The key of the line the next one is compared against.
        let (mut previous, mut current) = (Scratch::empty(), Scratch::default());
        let mut i = 0;
        while i < count {
            let get_lnum = line1 + i;
            // Read the key out of the line and compare it with the one
            // before: the delete below invalidates the memline's copy.
            // SAFETY: `get_lnum` is a line of the current buffer, and the
            // handle dies before anything else touches the memline.
            {
                let mut lines = unsafe { Lines::current() };
                let text = lines.line_mut(get_lnum);
                // SAFETY: a buffer line is NUL-terminated past its last byte.
                let (start, end) = unsafe { match_range(&mut regmatch, text, use_match) };
                // A line the pattern missed compares its whole text, which is
                // what upstream's "terminate at `end` only when there is an
                // end to terminate at" comes to.
                let stop = if end > 0 { end as usize } else { text.len() };
                current.fill(&text[start as usize..stop]);
            }

            // SAFETY: both scratch buffers are NUL-terminated.
            let is_match = i > 0
                && unsafe { order.compare(current.as_ptr(), previous.as_ptr()) } == Ordering::Equal;
            let (delete_lnum, keep) = scan.step(mode, i, count, get_lnum, is_match);
            if keep {
                core::mem::swap(&mut previous, &mut current);
            }

            if delete_lnum > 0 {
                // SAFETY: it is a line of the range.
                let _ = unsafe { ml_delete(delete_lnum) };
                i -= get_lnum - delete_lnum + 1;
                count -= 1;
                deleted += 1;
                change_occurred = true;
            }

            fast_breakcheck();
            if got_int.get() {
                break 'uniqend;
            }
            i += 1;
        }

        // Adjust marks for deleted lines and prepare for displaying.
        // SAFETY: the range is the one just rewritten.
        unsafe {
            mark_adjust(
                line2 - deleted,
                line2,
                MAXLNUM as linenr_T,
                -deleted,
                if change_occurred {
                    kExtmarkUndo
                } else {
                    kExtmarkNOOP
                } as ExtmarkOp,
            )
        };
        say::more(-deleted);
        if change_occurred {
            // SAFETY: as above.
            unsafe { changed_lines(Buf::new(curbuf.get()), line1, 0, line2 + 1, -deleted, true) };
        }
        cur_win().w_cursor.lnum = line1;
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }

    // SAFETY: the program is this command's own.
    unsafe { vim_regfree(regmatch.regprog) };
    if got_int.get() {
        emsg(gettext(e_interr));
    }
}
