//! `:sort` and `:uniq`.
//!
//! Both look at one *key* per line -- the whole line, or the part a `/pat/`
//! argument picked out of it, or the number or float parsed from that part --
//! and then either reorder the range by the key ([`ex_sort`]) or delete the
//! lines whose key repeats ([`ex_uniq`]).
//!
//! `:sort` builds the whole key array first, hands it to `qsort` and appends
//! the lines back in the new order below the range before deleting the
//! original.  That keeps the line *text* in the memline: only the keys, and
//! one line at a time, are ever copied.  Its comparator cannot report an
//! error, so an interrupt is signalled through [`SORT_ABORT`], which makes
//! every further comparison answer "equal" and `qsort` finish quickly.
//!
//! `:uniq` never reorders, so it walks the range once and deletes as it goes.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{MAXLNUM, RE_MAGIC, e_interr, e_invarg, e_noprevre, kExtmarkNOOP, kExtmarkUndo};
use crate::ascii::ascii_iswhite;
use crate::change::changed_lines;
use crate::charset::Str2NrBases;
use crate::charset::{skiptobin, skiptodigit, skiptohex, skipwhite, vim_str2nr};
use crate::cstr;
use crate::edit::{BeginlineOpts, beginline};
use crate::ex_docmd::check_nextcmd;
use crate::extmark::extmark_splice;
use crate::global_cell::GlobalCell;
use crate::main::{curbuf, got_int, p_ic};
use crate::mark::mark_adjust;
use crate::memline::{ml_append, ml_delete, ml_get, ml_get_len};
use crate::memory::{xfree, xmalloc};
use crate::message::{emsg, msgmore};
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
use crate::winlayer::{Buf, Win};
use ::libc::{qsort, strcasecmp, strcoll, strcpy, strtod};
use core::cmp::Ordering;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// How two keys are compared as *text*: `l` asks the locale, `i` folds case,
/// and `l` wins when both are given because upstream tests it first.
#[derive(Clone, Copy)]
struct StringOrder {
    locale: bool,
    ignore_case: bool,
}

/// The comparison mode, which `sort_compare` reads and cannot be handed.
static SORT_ORDER: GlobalCell<StringOrder> = GlobalCell::new(StringOrder {
    locale: false,
    ignore_case: false,
});

/// Set once the user interrupts a sort.  There is no way to stop `qsort`
/// immediately, but a comparator that always answers "equal" makes it decide
/// it is done; the half-sorted array is then thrown away.
static SORT_ABORT: GlobalCell<bool> = GlobalCell::new(false);

/// A buffer big enough for the longest line of the range, used to hold one
/// line's key across a call that may invalidate the memline's own copy.
/// `:sort`'s comparator needs a second one for the same reason.
static SORTBUF1: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());
static SORTBUF2: GlobalCell<*mut c_char> = GlobalCell::new(ptr::null_mut());

/// What one line is sorted on.  Every line of a range gets the same variant:
/// the flags decide which once, before the range is walked.
#[derive(Clone, Copy)]
enum SortKey {
    /// The byte range of the line the comparison looks at.
    Text { start: colnr_T, end: colnr_T },
    /// The integer parsed out of that range.  A line without one sorts
    /// before every line with one, which is what `Option`'s order says.
    Number(Option<varnumber_T>),
    /// The float parsed out of that range.
    Float(float_T),
}

/// One line of the range, as `:sort` sees it.
#[derive(Clone, Copy)]
struct SortLine {
    lnum: linenr_T,
    key: SortKey,
}

/// Compare two NUL-terminated keys as text.
///
/// # Safety
/// Both must be NUL-terminated.
unsafe fn string_compare(s1: *const c_char, s2: *const c_char) -> c_int {
    let order = SORT_ORDER.get();
    // SAFETY: caller's contract.
    if order.locale {
        unsafe { strcoll(s1, s2) }
    } else if order.ignore_case {
        unsafe { strcasecmp(s1.cast_mut(), s2.cast_mut()) }
    } else {
        unsafe { cstr::cmp(s1, s2) as c_int }
    }
}

/// `qsort`'s comparator over the `SortLine` array.
///
/// # Safety
/// `s1` and `s2` must point at elements of that array.
unsafe extern "C" fn sort_compare(s1: *const c_void, s2: *const c_void) -> c_int {
    // SAFETY: caller's contract.
    let (l1, l2) = unsafe { (*s1.cast::<SortLine>(), *s2.cast::<SortLine>()) };

    if SORT_ABORT.get() {
        return 0;
    }
    // The only way out of a long sort.
    fast_breakcheck();
    if got_int.get() {
        SORT_ABORT.set(true);
    }

    let order = match (l1.key, l2.key) {
        (SortKey::Number(a), SortKey::Number(b)) => a.cmp(&b),
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
        (SortKey::Text { .. }, SortKey::Text { .. }) => {
            // SAFETY: both lines are still in the buffer, and the two
            // buffers hold the longest line of the range.
            unsafe { copy_key(SORTBUF1.get(), l1) };
            unsafe { copy_key(SORTBUF2.get(), l2) };
            match unsafe { string_compare(SORTBUF1.get(), SORTBUF2.get()) } {
                0 => Ordering::Equal,
                n if n < 0 => Ordering::Less,
                _ => Ordering::Greater,
            }
        }
        // Unreachable: every key has the same shape.  Falling back on the
        // line order keeps the comparator a total one either way.
        _ => Ordering::Equal,
    };

    match order {
        // If two lines have the same value, preserve the original order.
        Ordering::Equal => l1.lnum - l2.lnum,
        Ordering::Less => -1,
        Ordering::Greater => 1,
    }
}

/// Copy the compared part of `line`'s text into `buf`, NUL-terminated.
///
/// # Safety
/// `buf` must have room for the range and its NUL; `line.lnum` must be a line
/// of the current buffer and `line.key` a [`SortKey::Text`].
unsafe fn copy_key(buf: *mut c_char, line: SortLine) {
    let SortKey::Text { start, end } = line.key else {
        return;
    };
    let len = (end - start) as usize;
    // SAFETY: caller's contract.  Upstream copies the byte past the range as
    // well and then overwrites it with the NUL.
    let into = buf.cast::<u8>();
    unsafe {
        into.copy_from_nonoverlapping(
            (ml_get(line.lnum).add(start as usize)).cast(),
            len as size_t,
        )
    };
    unsafe { *buf.add(len) = NUL as c_char };
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
    /// The base `b`/`o`/`x` forced, or nothing for `n`'s plain decimal.
    radix: Str2NrBases,
    /// `n`/`b`/`o`/`x`: compare as integers.
    numeric: bool,
    /// `f`: compare as floats.
    float: bool,
}

/// Read `:sort`'s flags and its optional pattern.  `false` means the command
/// was rejected, with the reason already reported.
///
/// # Safety
/// `eap` must be a live Ex command.
unsafe fn parse_sort_flags(
    eap: *mut exarg_T,
    spec: &mut SortSpec,
    regmatch: &mut regmatch_T,
) -> bool {
    // SAFETY: caller's contract.
    let arg = unsafe { (*eap).arg };
    let mut order = StringOrder {
        locale: false,
        ignore_case: false,
    };
    // Only one of 'n', 'b', 'o', 'f' and 'x' is allowed.
    let mut formats = 0;
    let mut at = 0;

    loop {
        // The pattern arm rewrites the argument as it goes, so each byte is
        // read from the buffer rather than from a slice taken up front.
        // SAFETY: `at` has not passed the argument's NUL.
        let byte = unsafe { *arg.add(at) } as u8;
        match byte {
            0 => break,
            b'i' => order.ignore_case = true,
            b'l' => order.locale = true,
            b'r' => spec.use_match = true,
            b'n' => {
                spec.numeric = true;
                formats += 1;
            }
            b'f' => {
                spec.float = true;
                formats += 1;
            }
            b'b' => {
                spec.radix = Str2NrBases::BIN | Str2NrBases::FORCE;
                formats += 1;
            }
            b'o' => {
                spec.radix = Str2NrBases::OCT | Str2NrBases::FORCE;
                formats += 1;
            }
            b'x' => {
                spec.radix = Str2NrBases::HEX | Str2NrBases::FORCE;
                formats += 1;
            }
            b'u' => spec.unique = true,
            // A comment starts here.
            b'"' => break,
            _ if ascii_iswhite(byte as c_int) => {}
            _ => {
                // SAFETY: `at` indexes the argument's own bytes.
                let next = unsafe { check_nextcmd(arg.add(at)) };
                if !next.is_null() {
                    // SAFETY: caller's contract.
                    unsafe { (*eap).nextcmd = next };
                    break;
                }
                if is_alpha(byte) || !regmatch.regprog.is_null() {
                    // SAFETY: as above.
                    let arg0 = unsafe { c_str(arg.add(at)) };
                    semsg!("E475: Invalid argument: {arg0}");
                    return false;
                }
                // SAFETY: as above.
                match unsafe { compile_sort_pattern(arg, at, regmatch) } {
                    Some(end) => at = end,
                    None => return false,
                }
            }
        }
        at += 1;
    }

    SORT_ORDER.set(order);
    if formats > 1 {
        emsg(gettext(e_invarg));
        return false;
    }
    // From here on "numeric" covers every integer format.
    spec.numeric |= !spec.radix.is_empty();
    true
}

/// The part of `line` the comparison should look at: the whole line when
/// there is no pattern, what the pattern matched under `r`, what follows the
/// match otherwise -- and nothing at all for a line the pattern misses.
///
/// # Safety
/// `line` must be a live buffer line of `len` bytes.
unsafe fn match_range(
    regmatch: &mut regmatch_T,
    line: *mut c_char,
    len: c_int,
    use_match: bool,
) -> (colnr_T, colnr_T) {
    if regmatch.regprog.is_null() {
        return (0, len);
    }
    // SAFETY: caller's contract.
    if !unsafe { vim_regexec(regmatch, line, 0) } {
        return (0, 0);
    }
    let start = unsafe { regmatch.startp[0].offset_from(line) } as colnr_T;
    let end = unsafe { regmatch.endp[0].offset_from(line) } as colnr_T;
    if use_match { (start, end) } else { (end, len) }
}

/// The number or float in `line[start..end]`.
///
/// The line is temporarily terminated at `end` so that `vim_str2nr` and
/// `strtod` cannot read digits past the match.
///
/// # Safety
/// `line` must be a live buffer line and `start`/`end` byte offsets into it.
unsafe fn number_key(line: *mut c_char, start: colnr_T, end: colnr_T, spec: &SortSpec) -> SortKey {
    // SAFETY: caller's contract.
    let stop = unsafe { line.add(end as usize) };
    let saved = unsafe { *stop };
    unsafe { *stop = NUL as c_char };
    let from = unsafe { line.add(start as usize) };

    let key = if spec.float {
        let mut s = unsafe { skipwhite(from) };
        if unsafe { *s } == '+' as c_char {
            s = unsafe { skipwhite(s.add(1)) };
        }
        // An empty line sorts before any number.
        SortKey::Float(if unsafe { *s } == NUL as c_char {
            -float_T::MAX
        } else {
            unsafe { strtod(s, ptr::null_mut()) }
        })
    } else {
        let mut s = if spec.radix.has(Str2NrBases::HEX) {
            unsafe { skiptohex(from) }
        } else if spec.radix.has(Str2NrBases::BIN) {
            unsafe { skiptobin(from) }.cast_mut()
        } else {
            unsafe { skiptodigit(from) }
        };
        // Include a preceding negative sign.
        if s > from && unsafe { *s.sub(1) } == '-' as c_char {
            s = unsafe { s.sub(1) };
        }
        if unsafe { *s } == NUL as c_char {
            // A line without a number sorts before any number.
            SortKey::Number(None)
        } else {
            let mut value: varnumber_T = 0;
            unsafe {
                vim_str2nr(
                    s,
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

    unsafe { *stop = saved };
    key
}

/// Build one [`SortLine`] per line of the range, and report the longest line.
///
/// `None` means the user interrupted the scan.  Doing the pattern match and
/// the number conversion once per line here is what keeps them out of the
/// comparator.
///
/// # Safety
/// The range must be lines of the current buffer.
unsafe fn collect_sort_keys(
    line1: linenr_T,
    line2: linenr_T,
    spec: &SortSpec,
    regmatch: &mut regmatch_T,
) -> Option<(Vec<SortLine>, c_int)> {
    let mut lines = Vec::with_capacity((line2 - line1 + 1) as usize);
    let mut maxlen = 0;

    for lnum in line1..=line2 {
        // SAFETY: `lnum` is a line of the current buffer.
        let (text, len) = (ml_get(lnum), ml_get_len(lnum));
        maxlen = maxlen.max(len);

        // SAFETY: as above.
        let (start, end) = unsafe { match_range(regmatch, text, len, spec.use_match) };
        let key = if spec.numeric || spec.float {
            // SAFETY: as above.
            unsafe { number_key(text, start, end, spec) }
        } else {
            SortKey::Text { start, end }
        };
        lines.push(SortLine { lnum, key });

        if !regmatch.regprog.is_null() {
            fast_breakcheck();
        }
        if got_int.get() {
            return None;
        }
    }
    Some((lines, maxlen))
}

/// `:sort`.
///
/// # Safety
/// `eap` must be a live Ex command whose range is inside the current buffer.
pub unsafe fn ex_sort(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let (forceit, line1, line2) = unsafe { ((*eap).forceit, (*eap).line1, (*eap).line2) };
    let mut count = (line2 - line1) as size_t + 1;

    // Sorting one line is really quick!
    if count <= 1 {
        return;
    }
    // SAFETY: the range is inside the current buffer.
    if u_save(line1 - 1, line2 + 1).is_err() {
        return;
    }

    SORTBUF1.set(ptr::null_mut());
    SORTBUF2.set(ptr::null_mut());
    SORT_ABORT.set(false);
    let mut regmatch = no_regmatch();
    let mut spec = SortSpec {
        unique: false,
        use_match: false,
        radix: Str2NrBases::NONE,
        numeric: false,
        float: false,
    };
    // Buffer contents changed.
    let mut change_occurred = false;
    let mut lnum = line2;

    'sortend: {
        // SAFETY: caller's contract.
        if !unsafe { parse_sort_flags(eap, &mut spec, &mut regmatch) } {
            break 'sortend;
        }
        // SAFETY: the range is inside the current buffer.
        let Some((mut lines, maxlen)) =
            (unsafe { collect_sort_keys(line1, line2, &spec, &mut regmatch) })
        else {
            break 'sortend;
        };

        // Allocate the two buffers that can hold the longest line, then sort
        // the array of line numbers.  Note: can't be interrupted!
        // SAFETY: both are freed below, on every path.
        SORTBUF1.set(unsafe { xmalloc(maxlen as size_t + 1) }.cast());
        SORTBUF2.set(unsafe { xmalloc(maxlen as size_t + 1) }.cast());
        unsafe {
            qsort(
                lines.as_mut_ptr().cast(),
                count,
                size_of::<SortLine>(),
                Some(sort_compare),
            )
        };
        if SORT_ABORT.get() {
            break 'sortend;
        }

        // Insert the lines in the sorted order below the last one.
        let mut old_count: bcount_t = 0;
        let mut new_count: bcount_t = 0;
        let mut placed = 0;
        while placed < count {
            let from = if forceit != 0 {
                count - placed - 1
            } else {
                placed
            };
            let get_lnum = lines[from].lnum;

            // If the original line number of the line being placed is not
            // the same as "lnum" (accounting for offset), the buffer changed.
            if get_lnum + (count as linenr_T - 1) != lnum {
                change_occurred = true;
            }

            // SAFETY: `get_lnum` is still a line of the current buffer.
            let stop = unsafe {
                let text = ml_get(get_lnum);
                // Include the EOL in the byte length.
                let bytelen = (ml_get_len(get_lnum) + 1) as bcount_t;
                old_count += bytelen;
                if spec.unique && placed > 0 && string_compare(text, SORTBUF1.get()) == 0 {
                    false
                } else {
                    // Copy the line into a buffer: it may become invalid in
                    // `ml_append`, and "unique" needs it next time round.
                    strcpy(SORTBUF1.get(), text);
                    let failed = ml_append(lnum, SORTBUF1.get(), 0, false).is_err();
                    lnum += 1;
                    if !failed {
                        new_count += bytelen;
                    }
                    failed
                }
            };
            if stop {
                break;
            }

            fast_breakcheck();
            if got_int.get() {
                break 'sortend;
            }
            placed += 1;
        }

        // Delete the original lines if appending worked.
        if placed == count {
            for _ in 0..count {
                // SAFETY: the range is still there, below the new lines.
                let _ = unsafe { ml_delete(line1) };
            }
        } else {
            count = 0;
        }

        // Adjust marks for deleted (or added) lines and prepare for display.
        let deleted = count as linenr_T - (lnum - line2);
        // SAFETY: the range is the one just rewritten.
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
            unsafe { msgmore(-deleted) };
        } else if deleted < 0 {
            unsafe { mark_adjust(line2, MAXLNUM as linenr_T, -deleted, 0, kExtmarkNOOP) };
        }

        if change_occurred || deleted != 0 {
            unsafe {
                extmark_splice(
                    curbuf.get(),
                    line1 - 1,
                    0,
                    count as c_int,
                    0,
                    old_count,
                    lnum - line2,
                    0,
                    new_count,
                    kExtmarkUndo,
                )
            };
            unsafe { changed_lines(Buf::new(curbuf.get()), line1, 0, line2 + 1, -deleted, true) };
        }

        cur_win().w_cursor.lnum = line1;
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }

    // SAFETY: each is either null or ours.
    unsafe { xfree(SORTBUF1.get().cast()) };
    unsafe { xfree(SORTBUF2.get().cast()) };
    unsafe { vim_regfree(regmatch.regprog) };
    if got_int.get() {
        emsg(gettext(e_interr));
    }
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

/// Read `:uniq`'s flags and its optional pattern.  `false` means the command
/// was rejected, with the reason already reported.
///
/// # Safety
/// `eap` must be a live Ex command.
unsafe fn parse_uniq_flags(
    eap: *mut exarg_T,
    mode: &mut UniqMode,
    use_match: &mut bool,
    regmatch: &mut regmatch_T,
) -> bool {
    // SAFETY: caller's contract.
    let arg = unsafe { (*eap).arg };
    let mut order = StringOrder {
        locale: false,
        ignore_case: false,
    };
    let mut at = 0;

    loop {
        // SAFETY: `at` has not passed the argument's NUL.
        let byte = unsafe { *arg.add(at) } as u8;
        match byte {
            0 => break,
            b'i' => order.ignore_case = true,
            b'l' => order.locale = true,
            b'r' => *use_match = true,
            // 'u' is only valid when '!' is not given.
            b'u' => {
                if *mode != UniqMode::OnlyRepeated {
                    *mode = UniqMode::OnlyUnique;
                }
            }
            // A comment starts here.
            b'"' => break,
            _ if ascii_iswhite(byte as c_int) => {}
            _ => {
                // SAFETY: `at` indexes the argument's own bytes.
                let next = unsafe { check_nextcmd(arg.add(at)) };
                // SAFETY: caller's contract.
                if !next.is_null() && unsafe { (*eap).nextcmd.is_null() } {
                    // SAFETY: as above.
                    unsafe { (*eap).nextcmd = next };
                    break;
                }
                if is_alpha(byte) || !regmatch.regprog.is_null() {
                    // SAFETY: as above.
                    let arg0 = unsafe { c_str(arg.add(at)) };
                    semsg!("E475: Invalid argument: {arg0}");
                    return false;
                }
                // SAFETY: as above.
                match unsafe { compile_sort_pattern(arg, at, regmatch) } {
                    Some(end) => at = end,
                    None => return false,
                }
            }
        }
        at += 1;
    }

    SORT_ORDER.set(order);
    true
}

/// `:uniq`.
///
/// # Safety
/// `eap` must be a live Ex command whose range is inside the current buffer.
pub unsafe fn ex_uniq(eap: *mut exarg_T) {
    // SAFETY: caller's contract.
    let (forceit, line1, line2) = unsafe { ((*eap).forceit, (*eap).line1, (*eap).line2) };
    let mut count = line2 - line1 + 1;

    // Uniq one line is really quick!
    if count <= 1 {
        return;
    }
    // SAFETY: the range is inside the current buffer.
    if u_save(line1 - 1, line2 + 1).is_err() {
        return;
    }

    SORTBUF1.set(ptr::null_mut());
    SORT_ABORT.set(false);
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
        // SAFETY: caller's contract.
        if !unsafe { parse_uniq_flags(eap, &mut mode, &mut use_match, &mut regmatch) } {
            break 'uniqend;
        }

        // Find the length of the longest line.
        let mut maxlen = 0;
        for lnum in line1..=line2 {
            // SAFETY: `lnum` is a line of the current buffer.
            maxlen = maxlen.max(ml_get_len(lnum));
            if got_int.get() {
                break 'uniqend;
            }
        }
        // SAFETY: freed below, on every path.
        SORTBUF1.set(unsafe { xmalloc(maxlen as size_t + 1) }.cast());

        let mut scan = UniqScan {
            in_run: false,
            force_unmatch: false,
            done_lnum: line1 - 1,
        };
        let mut i = 0;
        while i < count {
            let get_lnum = line1 + i;
            // SAFETY: `get_lnum` is a line of the current buffer.
            let (text, len) = (ml_get(get_lnum), ml_get_len(get_lnum));
            // SAFETY: as above.
            let (start, end) = unsafe { match_range(&mut regmatch, text, len, use_match) };

            // Terminate the line at the end of the key, compare it with the
            // one before, and put the byte back.
            // SAFETY: `start` and `end` are offsets into this line.
            let (delete_lnum, _) = unsafe {
                let saved = if end > 0 {
                    Some(core::mem::replace(
                        &mut *text.add(end as usize),
                        NUL as c_char,
                    ))
                } else {
                    None
                };
                let is_match =
                    i > 0 && string_compare(text.add(start as usize), SORTBUF1.get()) == 0;
                let step = scan.step(mode, i, count, get_lnum, is_match);
                if step.1 {
                    strcpy(SORTBUF1.get(), text.add(start as usize));
                }
                if let Some(saved) = saved {
                    *text.add(end as usize) = saved;
                }
                step
            };

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
        unsafe { msgmore(-deleted) };
        if change_occurred {
            unsafe { changed_lines(Buf::new(curbuf.get()), line1, 0, line2 + 1, -deleted, true) };
        }
        cur_win().w_cursor.lnum = line1;
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }

    // SAFETY: it is either null or ours.
    unsafe { xfree(SORTBUF1.get().cast()) };
    unsafe { vim_regfree(regmatch.regprog) };
    if got_int.get() {
        emsg(gettext(e_interr));
    }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
