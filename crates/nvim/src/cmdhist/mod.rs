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
//! [`hist_shada_take`]/[`hist_shada_replace`] move ownership out and back
//! in for the read-merge. Strings crossing that boundary are C allocations
//! (the Rust global allocator is malloc-backed, so either side may free).

#![deny(unsafe_op_in_unsafe_fn)]

mod ring;

use ring::{EMPTY_RING, to_cstring};
pub use ring::{HistEntry, Ring};

use crate::charset::vim_strsize;
use crate::eval::typval::{NumBuf, tv_get_number, tv_get_number_chk, tv_get_string_buf};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::cmdmod_has;
use crate::ex_getln::{get_cmdline_firstc, get_list_range};
use crate::global_cell::GlobalCell;
use crate::main::{Columns, got_int, maptick, p_hi};
use crate::memory::{xfree, xstrlcpy};
use crate::message::{
    message_filtered, msg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts_title,
    trunc_string,
};
use crate::os::cshim::{gettext, snprintf};
use crate::os::time::os_time;
use crate::regexp::{RE_MAGIC, RE_STRING, vim_regcomp, vim_regexec, vim_regfree};
use crate::strings::xstrnsave;
use crate::types::{
    AdditionalData, CmdModFlags, EvalFuncData, Failed, HistoryType, IOSIZE, OptInt, Timestamp,
    VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, exarg_T, expand_T, regmatch_T, size_t, typval_T,
    varnumber_T,
};
use core::ffi::{CStr, c_char, c_int, c_void};

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

/// Owned `*mut AdditionalData` (opaque extra shada payload on an entry);
/// freed on drop. Null means none.
struct ExtraData(*mut AdditionalData);

impl ExtraData {
    const NONE: ExtraData = ExtraData(core::ptr::null_mut());

    /// Move the pointer out, leaving none behind.
    fn take(&mut self) -> *mut AdditionalData {
        core::mem::replace(&mut self.0, core::ptr::null_mut())
    }
}

impl Drop for ExtraData {
    fn drop(&mut self) {
        // SAFETY: the pointer is either null or a live malloc-family
        // allocation this entry owns.
        unsafe { xfree(self.0.cast::<c_void>()) };
    }
}

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

/// Borrowed view of the entry at raw ring index `idx`.
#[derive(Copy, Clone)]
pub struct HistEntryRef {
    /// NUL-terminated entry text; valid until the entry is removed or
    /// overwritten.
    pub text: *const c_char,
    /// Text length in bytes, excluding the terminator.
    pub len: usize,
    /// Separator character (search history only; NUL elsewhere).
    pub sep: u8,
}

/// The entry at raw slot `idx` of history `histype`, if occupied.
pub fn hist_entry_ref(histype: c_int, idx: c_int) -> Option<HistEntryRef> {
    if !valid_histype(histype) {
        return None;
    }
    HISTORY.with(|h| {
        h[histype as usize].get(idx).map(|e| HistEntryRef {
            text: e.c_ptr(),
            len: e.text().len(),
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
    let history_opt = p_hi.get();
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

/// Clear history `histype`. Returns OK/FAIL.
fn clr_history(histype: c_int) -> Result<(), Failed> {
    if get_hislen() != 0 && valid_histype(histype) {
        HISTORY.with_mut(|h| h[histype as usize].clear());
        return Ok(());
    }
    Err(Failed)
}

/// Delete all entries of `histype` matching the vim regex `pat`.
///
/// # Safety
///
/// `pat` must be a valid NUL-terminated string.
unsafe fn del_history_entry(histype: c_int, pat: *const c_char) -> bool {
    // SAFETY: caller contract; `pat` points at at least its terminator.
    let empty = unsafe { *pat } == 0;
    if get_hislen() == 0 || !valid_histype(histype) || empty || get_hisidx(histype) < 0 {
        return false;
    }
    // SAFETY: caller contract; the compiled program is ours to free below.
    let regprog = unsafe { vim_regcomp(pat, RE_MAGIC + RE_STRING) };
    if regprog.is_null() {
        return false;
    }
    let mut regmatch = regmatch_T {
        regprog,
        startp: [core::ptr::null_mut(); 10],
        endp: [core::ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    let found = HISTORY.with_mut(|h| {
        h[histype as usize].delete_matching(|e| {
            // SAFETY: entry text is NUL-terminated and outlives the call.
            unsafe { vim_regexec(&raw mut regmatch, e.c_ptr(), 0) }
        })
    });
    // SAFETY: the program was compiled above and nothing references it now.
    unsafe { vim_regfree(regmatch.regprog) };
    found
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
///
/// # Safety
///
/// `arg` must be a valid typval.
unsafe fn arg_histtype(arg: *const typval_T) -> HistoryType {
    let mut numbuf = NumBuf::new();
    // SAFETY: caller contract; a non-null result is a NUL-terminated string
    // owned by the typval, which outlives the lookup.
    unsafe {
        let name = numbuf.string_chk(arg);
        if name.is_null() {
            HIST_INVALID
        } else {
            get_histtype(CStr::from_ptr(name).to_bytes(), false)
        }
    }
}

/// "histadd()" function
pub unsafe fn f_histadd(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract; the result starts out 0.
    unsafe { (*rettv).vval.v_number = 0 };
    // SAFETY: reads the 'secure'/sandbox globals.
    if check_secure() {
        return;
    }
    // SAFETY: eval-function contract.
    let histype = unsafe { arg_histtype(argvars) };
    if histype == HIST_INVALID {
        return;
    }
    let mut buf = [0 as c_char; 65];
    // SAFETY: `histadd()` takes two arguments; the entry is NUL-terminated
    // and lives in the typval or in `buf`, both of which outlive the add.
    let added = unsafe {
        let entry = tv_get_string_buf(argvars.offset(1), buf.as_mut_ptr());
        *entry != 0 && {
            init_history();
            add_to_history(histype, CStr::from_ptr(entry).to_bytes(), false, 0);
            true
        }
    };
    if added {
        // SAFETY: eval-function contract.
        unsafe { (*rettv).vval.v_number = 1 };
    }
}

/// "histdel()" function
pub unsafe fn f_histdel(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: eval-function contract; a non-null name is NUL-terminated, and
    // the second argument is only read once its type says it is present.
    let n = unsafe {
        let name = numbuf.string_chk(argvars);
        if name.is_null() {
            0
        } else {
            let histype = get_histtype(CStr::from_ptr(name).to_bytes(), false);
            let arg = argvars.offset(1);
            if (*arg).v_type == VAR_UNKNOWN {
                // Only one argument: clear the whole history.
                clr_history(histype).is_ok() as c_int
            } else if (*arg).v_type == VAR_NUMBER {
                // Delete by history number.
                del_history_idx(histype, tv_get_number(arg) as c_int) as c_int
            } else {
                // Delete by regex.
                let mut buf = [0 as c_char; 65];
                del_history_entry(histype, tv_get_string_buf(arg, buf.as_mut_ptr())) as c_int
            }
        }
    };
    // SAFETY: eval-function contract.
    unsafe { (*rettv).vval.v_number = varnumber_T::from(n) };
}

/// "histget()" function
pub unsafe fn f_histget(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    // SAFETY: eval-function contract.
    let name = unsafe { numbuf.string_chk(argvars) };
    let text = if name.is_null() {
        core::ptr::null_mut()
    } else {
        // SAFETY: a non-null name is NUL-terminated; the optional second
        // argument is only read once its type says it is present, and
        // `xstrnsave` copies the entry text before returning.
        unsafe {
            let histype = get_histtype(CStr::from_ptr(name).to_bytes(), false);
            let num = if (*argvars.offset(1)).v_type == VAR_UNKNOWN {
                get_history_idx(histype)
            } else {
                tv_get_number_chk(argvars.offset(1), core::ptr::null_mut()) as c_int
            };
            let idx = calc_hist_idx(histype, num);
            match hist_entry_ref(histype, idx) {
                None => xstrnsave(c"".as_ptr(), 0),
                Some(e) => xstrnsave(e.text, e.len),
            }
        }
    };
    // SAFETY: eval-function contract.
    unsafe {
        (*rettv).vval.v_string = text;
        (*rettv).v_type = VAR_STRING;
    }
}

/// "histnr()" function
pub unsafe fn f_histnr(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: eval-function contract.
    let histype = unsafe { arg_histtype(argvars) };
    let n = if histype == HIST_INVALID {
        HIST_INVALID
    } else {
        get_history_idx(histype)
    };
    // SAFETY: eval-function contract.
    unsafe { (*rettv).vval.v_number = varnumber_T::from(n) };
}

/// ":history" command: list history entries, optionally filtered by
/// history name ("cmd", ":", "all", ...) and a number range.
pub unsafe fn ex_history(eap: *mut exarg_T) {
    // SAFETY: caller contract; the message kind is a static string.
    let arg = unsafe {
        msg_ext_set_kind(c"list_cmd".as_ptr());
        (*eap).arg
    };
    if get_hislen() == 0 {
        msg(gettext(c"'history' option is zero"), 0);
        return;
    }
    // SAFETY: an ex-command argument is NUL-terminated.
    let Some((histypes, first, last)) = (unsafe { parse_history_arg(arg) }) else {
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
///
/// # Safety
///
/// `arg` must be a NUL-terminated ex-command argument.
unsafe fn parse_history_arg(
    arg: *mut c_char,
) -> Option<(core::ops::RangeInclusive<HistoryType>, c_int, c_int)> {
    // SAFETY: caller contract; the argument outlives this call.
    let bytes = unsafe { CStr::from_ptr(arg).to_bytes() };
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
    let mut end = unsafe { arg.add(name_len) };
    let mut first: c_int = 1;
    let mut last: c_int = -1;
    // SAFETY: `end` points into the argument, and `get_list_range` advances
    // it over what it parsed, never past the terminator.
    let (parsed, rest) = unsafe {
        let parsed = get_list_range(&raw mut end, &raw mut first, &raw mut last);
        (parsed, CStr::from_ptr(end).to_bytes())
    };
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
    // SAFETY: `title` is NUL-terminated and outlives the call, which copies
    // what it keeps.
    unsafe { msg_puts_title(title.as_ptr() as *const c_char) };
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
        if let Some(entry) = hist_entry_ref(histype, i) {
            let num = number_at(i);
            // SAFETY: the entry text is NUL-terminated and stays valid while
            // it is printed.
            if num >= first && num <= last && !unsafe { message_filtered(entry.text) } {
                print_history_entry(entry, num, i == idx);
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
fn print_history_entry(entry: HistEntryRef, num: c_int, newest: bool) {
    let mut row = [0 as c_char; IOSIZE as usize];
    // SAFETY: `entry.text` is NUL-terminated and outlives the call; `row` is
    // `IOSIZE` bytes, and the number prefix `snprintf` reports is the offset
    // both writers continue from, each with the remaining room.
    unsafe {
        msg_putchar('\n' as c_int);
        let buf = row.as_mut_ptr();
        let marker = if newest { '>' } else { ' ' } as c_int;
        let len = snprintf(buf, IOSIZE as size_t, c"%c%6d  ".as_ptr(), marker, num);
        let text = buf.offset(len as isize);
        if vim_strsize(entry.text) > Columns.get() - 10 {
            trunc_string(entry.text, text, Columns.get() - 10, IOSIZE - len);
        } else {
            xstrlcpy(text, entry.text, (IOSIZE - len) as size_t);
        }
        msg_outtrans(buf, 0, false);
    }
}

/// Completion source for `:history` arguments: the one-character names,
/// the long names, then "all".
pub unsafe fn get_history_arg(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    let short_count = SHORT_NAMES.len() as c_int;
    if (0..short_count).contains(&idx) {
        // SAFETY: caller contract; `xp_buf` is the completion scratch buffer,
        // far longer than the character and terminator written here.
        return unsafe {
            (*xp).xp_buf[0] = SHORT_NAMES[idx as usize] as c_char;
            (*xp).xp_buf[1] = 0;
            (*xp).xp_buf.as_mut_ptr()
        };
    }
    let i = (idx - short_count) as usize;
    if i < HIST_COUNT {
        return HISTORY_NAMES[i].as_ptr() as *mut c_char;
    }
    if i == HIST_COUNT {
        return c"all".as_ptr() as *mut c_char;
    }
    core::ptr::null_mut()
}

/// One history entry crossing the shada boundary.
#[derive(Copy, Clone)]
pub struct HistShadaEntry {
    /// NUL-terminated text. Borrowed from the ring for
    /// [`hist_shada_view`]; a malloc-family allocation owned by the holder
    /// for [`hist_shada_take`]/[`hist_shada_replace`].
    pub text: *mut c_char,
    /// Separator character (search history only; NUL elsewhere).
    pub sep: c_char,
    pub timestamp: Timestamp,
    /// Opaque extra payload; ownership follows `text`.
    pub additional_data: *mut AdditionalData,
}

fn shada_sep(histype: c_int, sep: u8) -> c_char {
    if histype == HIST_SEARCH {
        sep as c_char
    } else {
        0
    }
}

/// Borrow the entries of `histype` for writing shada, oldest first. The
/// text pointers stay valid until the ring is next mutated.
pub fn hist_shada_view(histype: c_int) -> Vec<HistShadaEntry> {
    HISTORY.with(|h| {
        let ring = &h[histype as usize];
        ring.oldest_first_indices()
            .into_iter()
            .map(|i| {
                let e = ring.get(i).expect("index of occupied slot");
                HistShadaEntry {
                    text: e.c_ptr() as *mut c_char,
                    sep: shada_sep(histype, e.sep),
                    timestamp: e.timestamp,
                    additional_data: e.extra.0,
                }
            })
            .collect()
    })
}

/// Move the entries of `histype` out for the shada read-merge, oldest
/// first, leaving the ring empty. The caller owns the returned text and
/// additional-data allocations.
pub fn hist_shada_take(histype: c_int) -> Vec<HistShadaEntry> {
    HISTORY.with_mut(|h| {
        let ring = &mut h[histype as usize];
        let taken: Vec<HistShadaEntry> = ring
            .oldest_first_indices()
            .into_iter()
            .map(|i| {
                let mut e = ring.entries[i as usize].take().expect("occupied slot");
                // Same allocation shape the C code kept: text, NUL, then
                // the separator byte.
                let bytes = e.text.as_bytes();
                let mut buf = Vec::with_capacity(bytes.len() + 2);
                buf.extend_from_slice(bytes);
                buf.push(0);
                buf.push(e.sep);
                HistShadaEntry {
                    text: Box::into_raw(buf.into_boxed_slice()) as *mut c_char,
                    sep: shada_sep(histype, e.sep),
                    timestamp: e.timestamp,
                    additional_data: e.extra.take(),
                }
            })
            .collect();
        ring.clear();
        taken
    })
}

/// Replace the contents of `histype` with `entries` (oldest first, as
/// produced by the shada merge), renumbering from 1. Takes ownership of
/// each entry's text (freed after copying) and additional data. Entries
/// beyond the ring capacity are discarded oldest-first.
///
/// # Safety
///
/// Every `text` must be a valid NUL-terminated malloc-family allocation
/// and every `additional_data` null or owned, as [`HistShadaEntry`]
/// documents; the caller must not use them afterwards.
pub unsafe fn hist_shada_replace(histype: c_int, entries: Vec<HistShadaEntry>) {
    init_history();
    HISTORY.with_mut(|h| {
        let ring = &mut h[histype as usize];
        ring.clear();
        let skip = entries.len().saturating_sub(ring.len());
        let mut n: c_int = 0;
        for (k, se) in entries.into_iter().enumerate() {
            if k < skip {
                // SAFETY: caller contract; both allocations are owned here.
                unsafe {
                    xfree(se.text.cast::<c_void>());
                    xfree(se.additional_data.cast::<c_void>());
                }
                continue;
            }
            // SAFETY: caller contract; `text` is a NUL-terminated allocation
            // owned here, copied out before it is freed.
            let text = unsafe {
                let text = to_cstring(CStr::from_ptr(se.text).to_bytes());
                xfree(se.text.cast::<c_void>());
                text
            };
            n += 1;
            ring.entries[(n - 1) as usize] = Some(HistEntry {
                number: n,
                text,
                sep: se.sep as u8,
                timestamp: se.timestamp,
                extra: ExtraData(se.additional_data),
            });
        }
        ring.num = n;
        ring.idx = n - 1;
    });
}
