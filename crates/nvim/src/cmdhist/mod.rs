//! Command-line history: the five rings (cmd, search, expr, input, debug)
//! behind `:history`, the `histadd()`/`histget()`/`histdel()`/`histnr()`
//! functions, cmdline up/down navigation, and shada persistence.
//!
//! The core is [`Ring`], a fixed-capacity ring buffer sized from the
//! 'history' option. Raw ring indexes are part of the public contract:
//! cmdline navigation in ex_getln keeps an index into the ring across
//! keystrokes, so slot positions (including vacant slots left by resizes
//! and deletions) must behave exactly like the C arrays did.
//!
//! Shada interaction: [`hist_shada_view`] lends entries out for writing,
//! [`hist_shada_take`] moves them out for the read-merge as C allocations
//! (the Rust global allocator is malloc-backed, so either side may free),
//! and [`hist_shada_replace`] takes the merged result back in, owned.

#![forbid(unsafe_code)]

mod ring;

use crate::cstr;
use crate::types::Candidate;
use ring::EMPTY_RING;
pub use ring::{HistEntry, Ring};
use std::ffi::CString;
use std::rc::Rc;

use crate::charset::strsize;
use crate::eval::typval::{NumBuf, tv_get_number, tv_get_number_chk};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::cmdmod_has;
use crate::ex_getln::{get_cmdline_firstc, get_list_range};
use crate::getchar::state::{got_int, maptick};
use crate::global_cell::GlobalCell;
use crate::memory::ThinCString;
use crate::message::{
    cut_middle, message_filtered, msg, msg_display_bytes, msg_ext_set_kind, msg_putchar, msg_title,
};
use crate::option::vars::p_hi;
use crate::os::cshim::gettext;
use crate::os::time::os_time;
use crate::regexp::{OwnedMatch, RE_MAGIC, RE_STRING, vim_regexec};
use crate::shada::ShadaExtra;
use crate::types::{
    CmdModFlags, EvalFuncData, ExArg, Expand, Failed, HistoryType, IOSIZE, OptInt, Timestamp,
    TypVal, VAR_NUMBER, VarNumber,
};
use crate::ui::state::Columns;
use core::ffi::{CStr, c_int};

pub const HIST_DEFAULT: HistoryType = -2;
pub const HIST_INVALID: HistoryType = -1;
pub const HIST_CMD: HistoryType = 0;
pub const HIST_SEARCH: HistoryType = 1;
pub const HIST_EXPR: HistoryType = 2;
pub const HIST_INPUT: HistoryType = 3;
pub const HIST_DEBUG: HistoryType = 4;
pub const HIST_COUNT: usize = 5;

/// Names accepted by `:history` and `histget()` etc., indexed by history
/// type. NUL-terminated because [`get_history_arg`] hands them to C-string
/// consumers.
const HISTORY_NAMES: [&[u8]; HIST_COUNT] =
    [b"cmd\0", b"search\0", b"expr\0", b"input\0", b"debug\0"];
/// One-character history names: `:` `=` `@` `>` and the search separators.
const SHORT_NAMES: &[u8] = b":=@>?/";

/// The five history rings. All the same length; resized together by
/// [`init_history`].
static HISTORY: GlobalCell<[Ring; HIST_COUNT]> = GlobalCell::new([EMPTY_RING; HIST_COUNT]);

/// `maptick` value at the last search entry added from a mapping, or -1.
/// Consecutive searches from one mapping replace each other.
static LAST_MAPTICK: GlobalCell<c_int> = GlobalCell::new(-1);

fn valid_histype(histype: c_int) -> bool {
    (0..HIST_COUNT as c_int).contains(&histype)
}

/// Current ring capacity (the 'history' option value at the last
/// [`init_history`] call; 0 before the first).
pub fn get_hislen() -> c_int {
    HISTORY.with(|h| h[0].len() as c_int)
}

/// Raw slot index of the newest entry of `histype`, or -1.
pub fn get_hisidx(histype: c_int) -> c_int {
    if !valid_histype(histype) {
        return -1;
    }
    HISTORY.with(|h| h[histype as usize].newest_idx())
}

/// A copy of the entry at raw ring index `idx`: what a caller keeps while it
/// runs code that may add to or delete from the history.
pub struct HistEntryText {
    /// The entry's text.
    pub text: CString,
    /// Separator character (search history only; NUL elsewhere).
    pub sep: u8,
}

/// The entry at raw slot `idx` of history `histype`, if occupied, copied.
pub fn hist_entry_text(histype: c_int, idx: c_int) -> Option<HistEntryText> {
    if !valid_histype(histype) {
        return None;
    }
    HISTORY.with(|h| {
        h[histype as usize].get(idx).map(|e| HistEntryText {
            text: CString::from(&*e.text),
            sep: e.sep,
        })
    })
}

/// Translate a cmdline first-character into a history type.
pub fn hist_char2type(c: c_int) -> HistoryType {
    if c == ':' as c_int {
        HIST_CMD
    } else if c == '=' as c_int {
        HIST_EXPR
    } else if c == '@' as c_int {
        HIST_INPUT
    } else if c == '>' as c_int {
        HIST_DEBUG
    } else if c == 0 || c == '/' as c_int || c == '?' as c_int {
        HIST_SEARCH
    } else {
        HIST_INVALID
    }
}

/// Translate a (possibly abbreviated, case-insensitive) history name into
/// a type. An empty name means the current cmdline's history, or
/// [`HIST_DEFAULT`] when `return_default` is set.
fn get_histtype(name: &[u8], return_default: bool) -> HistoryType {
    if name.is_empty() {
        if return_default {
            return HIST_DEFAULT;
        }
        // SAFETY: reads the cmdline state global; main thread only.
        return hist_char2type(get_cmdline_firstc());
    }
    for (i, hist_name) in HISTORY_NAMES.iter().enumerate() {
        let hist_name = &hist_name[..hist_name.len() - 1]; // drop the NUL
        if name.len() <= hist_name.len() && hist_name[..name.len()].eq_ignore_ascii_case(name) {
            return i as HistoryType;
        }
    }
    if name.len() == 1 && SHORT_NAMES.contains(&name[0]) {
        return hist_char2type(c_int::from(name[0]));
    }
    HIST_INVALID
}

/// Sync the rings to the 'history' option, keeping the newest entries when
/// shrinking.
pub fn init_history() {
    let history_opt = p_hi();
    assert!(
        (0..=OptInt::from(c_int::MAX)).contains(&history_opt),
        "'history' out of range"
    );
    let newlen = history_opt as usize;
    HISTORY.with_mut(|h| {
        if h[0].len() != newlen {
            for ring in h {
                ring.resize(newlen);
            }
        }
    });
}

/// Add `new_entry` to history `histype`, deduplicating against existing
/// entries. `in_map` marks searches issued from a mapping (consecutive
/// ones replace each other); `sep` is the search separator to remember.
pub fn add_to_history(histype: c_int, new_entry: &[u8], in_map: bool, sep: u8) {
    if get_hislen() == 0 || histype == HIST_INVALID {
        return;
    }
    debug_assert!(histype != HIST_DEFAULT);
    if cmdmod_has(CmdModFlags::KEEPPATTERNS) && histype == HIST_SEARCH {
        return;
    }
    let now = os_time();
    HISTORY.with_mut(|h| {
        if histype == HIST_SEARCH && in_map {
            if maptick.get() == LAST_MAPTICK.get() && !h[HIST_SEARCH as usize].is_empty() {
                // Consecutive searches from one mapping: only the last one
                // is kept.
                h[HIST_SEARCH as usize].drop_newest();
            }
            LAST_MAPTICK.set(-1);
        }
        let ring = &mut h[histype as usize];
        let sep_match = if histype == HIST_SEARCH {
            Some(sep)
        } else {
            None
        };
        if ring.move_to_front(new_entry, sep_match, now) {
            return;
        }
        ring.add(new_entry, sep, now);
        if histype == HIST_SEARCH && in_map {
            LAST_MAPTICK.set(maptick.get());
        }
    });
}

/// Sequence number of the newest entry of `histype`, or -1.
fn get_history_idx(histype: c_int) -> c_int {
    if get_hislen() == 0 || !valid_histype(histype) {
        return -1;
    }
    HISTORY.with(|h| h[histype as usize].newest_number())
}

/// Map a history number to a raw slot index (see [`Ring::calc_idx`]).
fn calc_hist_idx(histype: c_int, num: c_int) -> c_int {
    if !valid_histype(histype) {
        return -1;
    }
    HISTORY.with(|h| h[histype as usize].calc_idx(num))
}

/// Clear history `histype`. Returns `Ok`/`Err`.
fn clr_history(histype: c_int) -> Result<(), Failed> {
    if get_hislen() != 0 && valid_histype(histype) {
        HISTORY.with_mut(|h| h[histype as usize].clear());
        return Ok(());
    }
    Err(Failed)
}

/// Delete all entries of `histype` matching the vim regex `pat`.
fn del_history_entry(histype: c_int, pat: &CStr) -> bool {
    if get_hislen() == 0 || !valid_histype(histype) || pat.is_empty() || get_hisidx(histype) < 0 {
        return false;
    }
    let Some(mut regmatch) = OwnedMatch::compile(pat, RE_MAGIC + RE_STRING, false) else {
        return false;
    };
    HISTORY.with_mut(|h| {
        h[histype as usize].delete_matching(|e| vim_regexec(&mut regmatch, e.cstr(), 0))
    })
}

/// Delete the entry of `histype` with history number `num`.
fn del_history_idx(histype: c_int, num: c_int) -> bool {
    let i = calc_hist_idx(histype, num);
    if i < 0 {
        return false;
    }
    HISTORY.with_mut(|h| {
        let ring = &mut h[histype as usize];
        if histype == HIST_SEARCH && maptick.get() == LAST_MAPTICK.get() && i == ring.newest_idx() {
            LAST_MAPTICK.set(-1);
        }
        ring.delete_at(i);
    });
    true
}

/// The history type named by a string argument, [`HIST_INVALID`] when the
/// typval is not a string (which is where `tv_get_string_chk` reports its
/// own error).
fn arg_histtype(arg: &TypVal) -> HistoryType {
    let mut numbuf = NumBuf::new();
    numbuf
        .bytes_chk(arg)
        .map_or(HIST_INVALID, |name| get_histtype(name, false))
}

/// "histadd()" function
pub fn f_histadd(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract; the result starts out 0.
    result.write_number(0);
    if check_secure() {
        return;
    }
    // SAFETY: eval-function contract.
    let histype = arg_histtype(&args[0]);
    if histype == HIST_INVALID {
        return;
    }
    let mut buf = NumBuf::new();
    let entry = buf.bytes(&args[1]);
    let added = !entry.is_empty() && {
        init_history();
        add_to_history(histype, entry, false, 0);
        true
    };
    if added {
        // SAFETY: eval-function contract.
        result.write_number(1);
    }
}

/// "histdel()" function
pub fn f_histdel(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let n = {
        let name = numbuf.bytes_chk(&args[0]);
        if let Some(name) = name {
            let histype = get_histtype(name, false);
            let Some(arg) = args.get(1) else {
                // Only one argument: clear the whole history.
                return result.write_number(VarNumber::from(clr_history(histype).is_ok() as c_int));
            };
            if arg.v_type() == VAR_NUMBER {
                // Delete by history number.
                del_history_idx(histype, tv_get_number(arg) as c_int) as c_int
            } else {
                // Delete by regex.
                let mut buf = NumBuf::new();
                c_int::from(del_history_entry(histype, buf.string(arg)))
            }
        } else {
            0
        }
    };
    // SAFETY: eval-function contract.
    result.write_number(VarNumber::from(n));
}

/// "histget()" function
pub fn f_histget(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let text = numbuf.bytes_chk(&args[0]).map(|name| {
        let histype = get_histtype(name, false);
        let num = if args.len() <= 1 {
            get_history_idx(histype)
        } else {
            tv_get_number_chk(&args[1]).unwrap_or(-1) as c_int
        };
        let idx = calc_hist_idx(histype, num);
        match hist_entry_text(histype, idx) {
            None => ThinCString::empty(),
            Some(e) => ThinCString::from_cstr(&e.text),
        }
    });
    result.write_string(text);
}

/// "histnr()" function
pub fn f_histnr(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract.
    let histype = arg_histtype(&args[0]);
    let n = if histype == HIST_INVALID {
        HIST_INVALID
    } else {
        get_history_idx(histype)
    };
    // SAFETY: eval-function contract.
    result.write_number(VarNumber::from(n));
}

/// ":history" command: list history entries, optionally filtered by
/// history name ("cmd", ":", "all", ...) and a number range.
pub fn ex_history(excmd: &mut ExArg) {
    msg_ext_set_kind(c"list_cmd");
    if get_hislen() == 0 {
        msg(gettext(c"'history' option is zero"), 0);
        return;
    }
    let Some((histypes, first, last)) = parse_history_arg(excmd.line.arg()) else {
        return;
    };
    for histype in histypes {
        if got_int.get() {
            break;
        }
        list_one_history(histype, first, last);
    }
}

/// Parse `:history`'s argument: an optional history name (or an abbreviation
/// of "all") followed by an optional `[first][,last]` range. Reports its own
/// errors and answers `None` when it did.
fn parse_history_arg(
    bytes: &[u8],
) -> Option<(core::ops::RangeInclusive<HistoryType>, c_int, c_int)> {
    // A leading digit, '-' or ',' starts the range: the history is the
    // default one and there is no name to read.
    let named = !bytes
        .first()
        .is_some_and(|&b| b.is_ascii_digit() || b == b'-' || b == b',');
    let name_len = if named {
        bytes
            .iter()
            .take_while(|&&b| b.is_ascii_alphabetic() || SHORT_NAMES.contains(&b))
            .count()
    } else {
        0
    };
    let histypes = if !named {
        HIST_CMD..=HIST_CMD
    } else {
        let name = &bytes[..name_len];
        match get_histtype(name, false) {
            HIST_INVALID => {
                let all = b"all";
                if name.len() > all.len() || !all[..name.len()].eq_ignore_ascii_case(name) {
                    let arg = String::from_utf8_lossy(bytes);
                    crate::semsg!("E488: Trailing characters: {arg}");
                    return None;
                }
                0..=(HIST_COUNT as HistoryType - 1)
            }
            histype => histype..=histype,
        }
    };
    let mut end = name_len;
    let mut first: c_int = 1;
    let mut last: c_int = -1;
    let parsed = get_list_range(bytes, &mut end, &mut first, &mut last);
    let rest = bytes.get(end..).unwrap_or_default();
    if parsed.is_ok() && rest.is_empty() {
        return Some((histypes, first, last));
    }
    if rest.is_empty() {
        let arg = String::from_utf8_lossy(bytes);
        crate::semsg!("E1510: Value too large: {arg}");
    } else {
        let rest = String::from_utf8_lossy(rest);
        crate::semsg!("E488: Trailing characters: {rest}");
    }
    None
}

/// Print one history's title, then every entry whose sequence number falls
/// in `[first, last]` — negative bounds counting back from the newest entry.
fn list_one_history(histype: HistoryType, first: c_int, last: c_int) {
    let name = HISTORY_NAMES[histype as usize];
    let name = String::from_utf8_lossy(&name[..name.len() - 1]);
    let title = format!("\n      #  {name} history\0");
    msg_title(cstr::in_bytes(title.as_bytes()));
    let hislen = get_hislen();
    let idx = get_hisidx(histype);
    let number_at = |i: c_int| HISTORY.with(|h| h[histype as usize].number_at(i));
    let resolve = |bound: c_int| {
        if bound >= 0 {
            bound
        } else if -i64::from(bound) > i64::from(hislen) {
            0
        } else {
            number_at((hislen + bound + idx + 1) % hislen)
        }
    };
    let (first, last) = (resolve(first), resolve(last));
    if idx < 0 || first > last {
        return;
    }
    // List from the oldest slot forward, ending at the newest.
    let mut i = idx + 1;
    while !got_int.get() {
        if i == hislen {
            i = 0;
        }
        if let Some(entry) = hist_entry_text(histype, i) {
            let num = number_at(i);
            if num >= first && num <= last && !message_filtered(&entry.text) {
                print_history_entry(entry.text.to_bytes(), num, i == idx);
            }
        }
        if i == idx {
            break;
        }
        i += 1;
    }
}

/// One `:history` row: `>` on the newest entry, the sequence number, and the
/// text truncated to the window width.
fn print_history_entry(text: &[u8], num: c_int, newest: bool) {
    msg_putchar('\n' as c_int);
    // Upstream builds the row in `IObuff`: the marker, the number, and the
    // text in what is left.
    let mut row = format!("{}{num:6}  ", if newest { '>' } else { ' ' }).into_bytes();
    let left = IOSIZE as usize - row.len();
    if strsize(text) > Columns.get() - 10 {
        row.extend_from_slice(&cut_middle(text, Columns.get() - 10, left));
    } else {
        row.extend_from_slice(&text[..text.len().min(left - 1)]);
    }
    msg_display_bytes(&row, 0, false);
}

/// Completion source for `:history` arguments: the one-character names,
/// the long names, then "all".
pub fn get_history_arg(_expand: &Expand, idx: usize) -> Option<Candidate> {
    if let Some(&short) = SHORT_NAMES.get(idx) {
        return Some(Candidate::Owned(
            CString::new([short]).expect("a history name is not NUL"),
        ));
    }
    let i = idx - SHORT_NAMES.len();
    if let Some(name) = HISTORY_NAMES.get(i) {
        return Some(Candidate::Borrowed(
            CStr::from_bytes_with_nul(name).expect("a NUL-terminated literal"),
        ));
    }
    (i == HIST_COUNT).then_some(Candidate::Borrowed(c"all"))
}

/// One history entry copied for, or taken by, the shada writer and merger.
pub(crate) struct HistShadaEntry {
    pub text: Rc<CStr>,
    /// Separator character (search history only; NUL elsewhere).
    pub sep: u8,
    pub timestamp: Timestamp,
    pub extra: Option<ShadaExtra>,
}

/// One entry of the merged history the shada reader hands back, owned.
pub(crate) struct HistRestored {
    pub text: Rc<CStr>,
    /// Separator character (search history only; NUL elsewhere).
    pub sep: u8,
    pub timestamp: Timestamp,
    pub extra: Option<ShadaExtra>,
}

fn shada_sep(histype: c_int, sep: u8) -> u8 {
    if histype == HIST_SEARCH { sep } else { 0 }
}

/// Copies of the entries of `histype` for writing shada, oldest first.
pub(crate) fn hist_shada_view(histype: c_int) -> Vec<HistShadaEntry> {
    HISTORY.with(|h| {
        let ring = &h[histype as usize];
        ring.oldest_first_indices()
            .into_iter()
            .map(|i| {
                let e = ring.get(i).expect("index of occupied slot");
                HistShadaEntry {
                    text: e.text.clone(),
                    sep: shada_sep(histype, e.sep),
                    timestamp: e.timestamp,
                    extra: e.extra.clone(),
                }
            })
            .collect()
    })
}

/// Move the entries of `histype` out for the shada read-merge, oldest
/// first, leaving the ring empty.
pub(crate) fn hist_shada_take(histype: c_int) -> Vec<HistShadaEntry> {
    HISTORY.with_mut(|h| {
        let ring = &mut h[histype as usize];
        let taken: Vec<HistShadaEntry> = ring
            .oldest_first_indices()
            .into_iter()
            .map(|i| {
                let e = ring.entries[i as usize].take().expect("occupied slot");
                HistShadaEntry {
                    text: e.text,
                    sep: shada_sep(histype, e.sep),
                    timestamp: e.timestamp,
                    extra: e.extra,
                }
            })
            .collect();
        ring.clear();
        taken
    })
}

/// Replace the contents of `histype` with `entries` (oldest first, as
/// produced by the shada merge), renumbering from 1. Entries beyond the
/// ring capacity are discarded oldest-first.
pub(crate) fn hist_shada_replace(histype: c_int, entries: Vec<HistRestored>) {
    init_history();
    HISTORY.with_mut(|h| {
        let ring = &mut h[histype as usize];
        ring.clear();
        let skip = entries.len().saturating_sub(ring.len());
        let mut n: c_int = 0;
        for entry in entries.into_iter().skip(skip) {
            n += 1;
            ring.entries[(n - 1) as usize] = Some(HistEntry {
                number: n,
                text: entry.text,
                sep: entry.sep,
                timestamp: entry.timestamp,
                extra: entry.extra,
            });
        }
        ring.num = n;
        ring.idx = n - 1;
    });
}
