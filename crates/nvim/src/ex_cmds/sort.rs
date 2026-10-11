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

#![forbid(unsafe_code)]

use super::LineCopy;
use super::say;
use super::{MAXLNUM, RE_MAGIC, e_interr, e_invarg, e_noprevre, kExtmarkNOOP, kExtmarkUndo};
use crate::ascii::ascii_iswhite;
use crate::change::changed_lines;
use crate::charset::Str2NrBases;
use crate::charset::{skip, str2nr_scan};
use crate::edit::{BeginlineOpts, beginline};
use crate::extmark::extmark_splice;
use crate::getchar::state::got_int;
use crate::mark::mark_adjust;
use crate::memline::{Lines, ml_append_cstr, ml_delete};
use crate::message::emsg;
use crate::message_fmt::{msg_bytes, msg_cstr};
use crate::option::vars::p_ic;
use crate::os::cshim::{cstr_ncasecmp, gettext, strcoll, strtod_prefix};
use crate::os::input::fast_breakcheck;
use crate::regexp::{OwnedMatch, skip_regexp_at, vim_regexec};
use crate::search::last_pattern_owned;
use crate::semsg;
use crate::types::{BCount, ColNr, ExArg, ExtmarkOp, Float, LineNr, NUL, VarNumber, size_t};
use crate::undo::u_save;
use crate::winlayer::Buf;
use crate::winlayer::Win;
use core::cmp::Ordering;
use core::ffi::{CStr, c_char, c_int};

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

    /// Compare two keys: `strcoll`, `strcasecmp` or `strcmp`.
    fn compare(self, s1: &CStr, s2: &CStr) -> Ordering {
        if self.locale {
            strcoll(s1, s2).cmp(&0)
        } else if self.ignore_case {
            // `strncasecmp` with no limit is `strcasecmp`.
            cstr_ncasecmp(s1, s2, usize::MAX).cmp(&0)
        } else {
            s1.to_bytes().cmp(s2.to_bytes())
        }
    }

    /// [`StringOrder::compare`] for two keys that carry their own NUL.
    fn compare_keys(self, k1: &Key, k2: &Key) -> Ordering {
        self.compare(k1.as_cstr(), k2.as_cstr())
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

    fn as_cstr(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0).expect("a key ends in a NUL")
    }
}

/// What one line is sorted on.  Every line of a range gets the same variant:
/// the flags decide which once, before the range is walked.
enum SortKey {
    /// The bytes of the line the comparison looks at.
    Text(Key),
    /// The integer parsed out of that range.  A line without one sorts before
    /// every line with one, which is what `Option`'s order says.
    Number(Option<VarNumber>),
    /// The float parsed out of that range.
    Float(Float),
}

/// One line of the range as `:sort` sees it.
struct SortLine {
    lnum: LineNr,
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

/// The `/pat/` argument both commands accept in place of a flag letter,
/// its opening delimiter at `delim_at` in the command line.
///
/// Returns where the closing delimiter is, which upstream overwrites with a
/// NUL so the pattern is its own string -- the scan resumes one byte past
/// it.  `None` means it failed, with the reason already reported.
fn compile_sort_pattern(
    args: &mut ExArg,
    delim_at: usize,
    regmatch: &mut Option<OwnedMatch>,
) -> Option<usize> {
    // The delimiter is handed on as a `char`, sign-extended -- and the skip
    // compares the pattern's bytes the same way, so a high byte still ends it.
    let delim = c_int::from(args.line.byte_at(delim_at) as c_char);
    let start = delim_at + 1;
    let end = start + skip_regexp_at(args.line.tail(start), delim, 1);
    if c_int::from(args.line.byte_at(end) as c_char) != delim {
        let startp = msg_bytes(args.line.rest_of(start));
        semsg!("E654: Missing delimiter after search pattern: {startp}");
        return None;
    }
    args.line.terminate_at(end);

    // Use the last search pattern if the sort pattern is empty.
    let compiled = if end == start {
        let Some(last) = last_pattern_owned() else {
            emsg(gettext(e_noprevre));
            return None;
        };
        OwnedMatch::compile(last.as_cstr(), RE_MAGIC, p_ic())
    } else {
        OwnedMatch::compile(args.line.cstr_from(start), RE_MAGIC, p_ic())
    };
    *regmatch = Some(compiled?);
    Some(end)
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
/// `at` indexes a byte of the command's argument.
fn flag_fallback(
    args: &mut ExArg,
    at: usize,
    byte: u8,
    regmatch: &mut Option<OwnedMatch>,
    keep_nextcmd: bool,
) -> Option<Fallback> {
    let base = args.line.arg;
    if let Some(next) = args.line.check_next(base + at)
        && !(keep_nextcmd && args.line.next.is_some())
    {
        args.line.next = Some(next);
        return Some(Fallback::NextCmd);
    }
    if is_alpha(byte) || regmatch.is_some() {
        let arg0 = msg_cstr(args.line.cstr_from(base + at));
        semsg!("E475: Invalid argument: {arg0}");
        return None;
    }
    compile_sort_pattern(args, base + at, regmatch).map(|end| Fallback::Pattern(end - base))
}

/// Read `:sort`'s flags and its optional pattern.  `None` means the command
/// was rejected, with the reason already reported.
fn parse_sort_flags(
    args: &mut ExArg,
    spec: &mut SortSpec,
    regmatch: &mut Option<OwnedMatch>,
) -> Option<StringOrder> {
    let mut order = StringOrder::BYTES;
    // Only one of 'n', 'b', 'o', 'f' and 'x' is allowed.
    let mut formats = 0;
    let mut at = 0;

    loop {
        // The pattern arm rewrites the argument as it goes, so each byte is
        // read from the buffer rather than from a slice taken up front.
        let byte = args.line.byte_at(args.line.arg + at);
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
                Flag::Other => match flag_fallback(args, at, byte, regmatch, false)? {
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
fn parse_uniq_flags(
    args: &mut ExArg,
    mode: &mut UniqMode,
    use_match: &mut bool,
    regmatch: &mut Option<OwnedMatch>,
) -> Option<StringOrder> {
    let mut order = StringOrder::BYTES;
    let mut at = 0;

    loop {
        let byte = args.line.byte_at(args.line.arg + at);
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
                Flag::Other => match flag_fallback(args, at, byte, regmatch, true)? {
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
/// The line is read through the cache rather than handed in as bytes: the
/// match runs over a C string, and the memline's terminator is the only
/// thing that says where a line's bytes end.
fn match_range(
    regmatch: &mut Option<OwnedMatch>,
    lines: &mut Lines,
    lnum: LineNr,
    use_match: bool,
) -> (ColNr, ColNr) {
    let len = lines.line_len(lnum);
    let Some(regmatch) = regmatch.as_mut() else {
        return (0, len);
    };
    if !vim_regexec(regmatch, lines.line_cstr(lnum, 0), 0) {
        return (0, 0);
    }
    let (start, end) = match regmatch.group(0) {
        Some(span) => (span.start as ColNr, span.end as ColNr),
        None => (0, 0),
    };
    if use_match { (start, end) } else { (end, len) }
}

/// The number or float in `line[start..end]`, read so that no digit past
/// `end` counts -- what upstream terminates the line in place for.
fn number_key(line: &[u8], start: ColNr, end: ColNr, spec: &SortSpec) -> SortKey {
    let from = &line[start as usize..end as usize];

    if spec.float {
        let mut at = skip::white(from);
        if from.get(at) == Some(&b'+') {
            at += 1 + skip::white(&from[at + 1..]);
        }
        // An empty key sorts before any number.
        SortKey::Float(if at == from.len() {
            -Float::MAX
        } else {
            strtod_prefix(&from[at..]).0
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
            let parsed = str2nr_scan(&from[at..], 0, spec.radix, false, true);
            SortKey::Number(Some(parsed.value))
        }
    }
}

/// Build one [`SortLine`] per line of the range.
///
/// `None` means the user interrupted the scan.  Doing the pattern match, the
/// number conversion and the text key's one copy here is what keeps them out
/// of the comparison.
fn collect_sort_keys(
    line1: LineNr,
    line2: LineNr,
    spec: &SortSpec,
    regmatch: &mut Option<OwnedMatch>,
) -> Option<Vec<SortLine>> {
    let mut sorted = Vec::with_capacity((line2 - line1 + 1) as usize);
    let mut lines = Lines::current();

    for lnum in line1..=line2 {
        let (start, end) = match_range(regmatch, &mut lines, lnum, spec.use_match);
        let text = lines.line(lnum);
        let key = if spec.numeric || spec.float {
            // `match_range` answers offsets into `text`.
            number_key(text, start, end, spec)
        } else {
            SortKey::Text(Key::new(&text[start as usize..end as usize]))
        };
        sorted.push(SortLine { lnum, key });

        if regmatch.is_some() {
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
    lnum: LineNr,
    /// What was read and what was written, for the extmark splice.
    old_bytes: BCount,
    new_bytes: BCount,
    /// A line ended up somewhere other than where it started.
    moved: bool,
    /// The user interrupted the walk.
    interrupted: bool,
}

/// Append the range's lines below it in the sorted order.
fn append_sorted(
    sorted: &[SortLine],
    order: StringOrder,
    unique: bool,
    reverse: bool,
    line2: LineNr,
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
    let (mut previous, mut current) = (LineCopy::new(), LineCopy::new());

    while placed.done < count {
        let from = if reverse {
            count - placed.done - 1
        } else {
            placed.done
        };
        let get_lnum = sorted[from].lnum;

        // If the original line number of the line being placed is not the
        // same as "lnum" (accounting for offset), the buffer changed.
        if get_lnum + (count as LineNr - 1) != placed.lnum {
            placed.moved = true;
        }

        // Copy the line out of the memline: `ml_append` may invalidate it,
        // and "unique" needs it next time round.
        let bytelen = current.fill_line(get_lnum) as BCount + 1;
        placed.old_bytes += bytelen;

        let duplicate = unique
            && placed.done > 0
            && order.compare(current.as_cstr(), previous.as_cstr()) == Ordering::Equal;
        if !duplicate {
            let failed = ml_append_cstr(placed.lnum, current.as_cstr()).is_err();
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
pub fn ex_sort(excmd: &mut ExArg) {
    sort_range(excmd);
}

/// `:sort`, with the command's argument block borrowed.
fn sort_range(args: &mut ExArg) {
    let (forceit, line1, line2) = (args.forceit, args.line1, args.line2);

    // Sorting one line is really quick!
    if line2 - line1 < 1 {
        return;
    }
    if u_save(line1 - 1, line2 + 1).is_err() {
        return;
    }

    let mut regmatch = None;
    let mut spec = SortSpec::new();
    let mut count = (line2 - line1) as size_t + 1;

    'sortend: {
        let Some(order) = parse_sort_flags(args, &mut spec, &mut regmatch) else {
            break 'sortend;
        };
        let Some(mut sorted) = collect_sort_keys(line1, line2, &spec, &mut regmatch) else {
            break 'sortend;
        };

        // Note: can't be interrupted!
        sorted.sort_by(|l1, l2| compare_lines(order, l1, l2));

        // Insert the lines in the sorted order below the last one.
        let placed = append_sorted(&sorted, order, spec.unique, forceit, line2);
        if placed.interrupted {
            break 'sortend;
        }

        // Delete the original lines if appending worked.
        if placed.done == count {
            for _ in 0..count {
                let _ = ml_delete(line1);
            }
        } else {
            count = 0;
        }

        finish_sort(line1, line2, count, &placed);
    }

    drop(regmatch);
    if got_int.get() {
        emsg(gettext(e_interr));
    }
}

/// Adjust marks and extmarks for the lines `:sort` moved, and put the cursor
/// on the first of them.
fn finish_sort(line1: LineNr, line2: LineNr, count: size_t, placed: &Placed) {
    let lnum = placed.lnum;
    let deleted = count as LineNr - (lnum - line2);
    if deleted > 0 {
        mark_adjust(line2 - deleted, line2, MAXLNUM, -deleted, kExtmarkNOOP);
        say::more(-deleted);
    } else if deleted < 0 {
        mark_adjust(line2, MAXLNUM, -deleted, 0, kExtmarkNOOP);
    }

    if placed.moved || deleted != 0 {
        extmark_splice(
            Buf::current(),
            line1 - 1,
            0,
            count as c_int,
            0,
            placed.old_bytes,
            lnum - line2,
            0,
            placed.new_bytes,
            kExtmarkUndo,
        );
        changed_lines(Buf::current(), line1, 0, line2 + 1, -deleted, true);
    }

    Win::current().w_cursor.lnum = line1;
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
    done_lnum: LineNr,
}

impl UniqScan {
    /// Decide what one line means: the line to delete (zero for none), and
    /// whether its key becomes the one the next line is compared against.
    fn step(
        &mut self,
        mode: UniqMode,
        i: LineNr,
        count: LineNr,
        lnum: LineNr,
        is_match: bool,
    ) -> (LineNr, bool) {
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
pub fn ex_uniq(excmd: &mut ExArg) {
    uniq_range(excmd);
}

/// `:uniq`, with the command's argument block borrowed.
fn uniq_range(args: &mut ExArg) {
    let (forceit, line1, line2) = (args.forceit, args.line1, args.line2);
    let mut count = line2 - line1 + 1;

    // Uniq one line is really quick!
    if count <= 1 {
        return;
    }
    if u_save(line1 - 1, line2 + 1).is_err() {
        return;
    }

    let mut regmatch = None;
    let mut mode = if forceit {
        UniqMode::OnlyRepeated
    } else {
        UniqMode::Dedup
    };
    let mut use_match = false;
    let mut change_occurred = false;
    let mut deleted = 0;

    'uniqend: {
        let Some(order) = parse_uniq_flags(args, &mut mode, &mut use_match, &mut regmatch) else {
            break 'uniqend;
        };

        let mut scan = UniqScan {
            in_run: false,
            force_unmatch: false,
            done_lnum: line1 - 1,
        };
        // The key of the line the next one is compared against.
        let (mut previous, mut current) = (LineCopy::new(), LineCopy::new());
        let mut i = 0;
        while i < count {
            let get_lnum = line1 + i;
            // Read the key out of the line and compare it with the one
            // before: the delete below invalidates the memline's copy.
            {
                let mut lines = Lines::current();
                let (start, end) = match_range(&mut regmatch, &mut lines, get_lnum, use_match);
                let text = lines.line(get_lnum);
                // A line the pattern missed compares its whole text, which is
                // what upstream's "terminate at `end` only when there is an
                // end to terminate at" comes to.
                let stop = if end > 0 { end as usize } else { text.len() };
                current.fill(&text[start as usize..stop]);
            }

            let is_match =
                i > 0 && order.compare(current.as_cstr(), previous.as_cstr()) == Ordering::Equal;
            let (delete_lnum, keep) = scan.step(mode, i, count, get_lnum, is_match);
            if keep {
                core::mem::swap(&mut previous, &mut current);
            }

            if delete_lnum > 0 {
                let _ = ml_delete(delete_lnum);
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
        mark_adjust(
            line2 - deleted,
            line2,
            MAXLNUM,
            -deleted,
            if change_occurred {
                kExtmarkUndo
            } else {
                kExtmarkNOOP
            } as ExtmarkOp,
        );
        say::more(-deleted);
        if change_occurred {
            changed_lines(Buf::current(), line1, 0, line2 + 1, -deleted, true);
        }
        Win::current().w_cursor.lnum = line1;
        beginline(BeginlineOpts::WHITE | BeginlineOpts::FIX);
    }

    drop(regmatch);
    if got_int.get() {
        emsg(gettext(e_interr));
    }
}
