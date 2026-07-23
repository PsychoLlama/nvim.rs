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

use crate::src::nvim::charset::vim_strsize;
use crate::src::nvim::eval::typval::{
    tv_get_number, tv_get_number_chk, tv_get_string_buf, tv_get_string_chk,
};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_getln::{get_cmdline_firstc, get_list_range};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    cmdmod, e_trailing_arg, e_val_too_large, got_int, maptick, p_hi, Columns, IObuff,
};
use crate::src::nvim::memory::{xfree, xstrlcpy};
use crate::src::nvim::message::{
    message_filtered, msg, msg_ext_set_kind, msg_outtrans, msg_putchar, msg_puts_title, semsg,
    trunc_string,
};
use crate::src::nvim::os::libc::{gettext, snprintf};
use crate::src::nvim::os::time::os_time;
use crate::src::nvim::regexp::{regmatch_T, vim_regcomp, vim_regexec, vim_regfree};
use crate::src::nvim::strings::xstrnsave;
use crate::src::nvim::types::{
    exarg_T, expand_T, size_t, typval_T, varnumber_T, AdditionalData, EvalFuncData, HistoryType,
    OptInt, Timestamp, VarType,
};
use core::ffi::{c_char, c_int, c_void, CStr};
use std::ffi::CString;

pub const HIST_DEFAULT: HistoryType = -2;
pub const HIST_INVALID: HistoryType = -1;
pub const HIST_CMD: HistoryType = 0;
pub const HIST_SEARCH: HistoryType = 1;
pub const HIST_EXPR: HistoryType = 2;
pub const HIST_INPUT: HistoryType = 3;
pub const HIST_DEBUG: HistoryType = 4;
pub const HIST_COUNT: usize = 5;

const OK: c_int = 1;
const FAIL: c_int = 0;
/// `cmdmod_T.cmod_flags` bit for `:keeppatterns`.
const CMOD_KEEPPATTERNS: c_int = 4096;
const RE_MAGIC: c_int = 1;
const RE_STRING: c_int = 2;
const IOSIZE: c_int = 1024 + 1;

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

/// One history entry.
pub struct HistEntry {
    number: c_int,
    text: CString,
    sep: u8,
    timestamp: Timestamp,
    extra: ExtraData,
}

impl HistEntry {
    /// Sequence number shown by `:history` and returned by `histnr()`.
    pub fn number(&self) -> c_int {
        self.number
    }

    /// Entry text without terminator.
    pub fn text(&self) -> &[u8] {
        self.text.as_bytes()
    }

    /// Separator character (search history only; NUL elsewhere).
    pub fn sep(&self) -> u8 {
        self.sep
    }

    pub fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    fn c_ptr(&self) -> *const c_char {
        self.text.as_ptr()
    }
}

/// Truncate at the first NUL; history entries are C strings and can never
/// contain one.
fn to_cstring(bytes: &[u8]) -> CString {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    CString::new(&bytes[..end]).expect("no interior NUL before `end`")
}

/// A fixed-capacity history ring. Pure data structure: all editor-state
/// coupling (the 'history' option, maptick, timestamps) lives in the
/// module-level functions.
///
/// `idx` is the raw slot of the newest entry, `-1` when empty. Entries sit
/// contiguously behind `idx` (wrapping); vacant slots elsewhere are normal
/// after resizes and deletions.
pub struct Ring {
    entries: Vec<Option<HistEntry>>,
    idx: c_int,
    num: c_int,
}

const EMPTY_RING: Ring = Ring {
    entries: Vec::new(),
    idx: -1,
    num: 0,
};

impl Ring {
    pub fn new(len: usize) -> Ring {
        Ring {
            entries: (0..len).map(|_| None).collect(),
            idx: -1,
            num: 0,
        }
    }

    /// Ring capacity (the `hislen` of the C implementation).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.idx < 0
    }

    /// Raw slot index of the newest entry, `-1` when empty.
    pub fn newest_idx(&self) -> c_int {
        self.idx
    }

    /// Sequence number of the newest entry; `-1` when the ring is empty,
    /// `0` when `idx` points at a vacant slot (possible after deleting the
    /// newest entry — a C quirk `histnr()` exposes).
    pub fn newest_number(&self) -> c_int {
        if self.idx < 0 {
            return -1;
        }
        self.number_at(self.idx)
    }

    /// Entry at raw slot `idx`, if occupied.
    pub fn get(&self, idx: c_int) -> Option<&HistEntry> {
        let idx = usize::try_from(idx).ok()?;
        self.entries.get(idx)?.as_ref()
    }

    /// Sequence number at a raw slot; vacant slots read as 0, exactly like
    /// the zero-filled C array.
    fn number_at(&self, idx: c_int) -> c_int {
        self.get(idx).map_or(0, |e| e.number)
    }

    /// Resize to `newlen` slots, keeping the newest entries. Mirrors the C
    /// `init_history` layout: kept entries are compacted to the front in
    /// age order (any leading vacancy from a not-full ring included), and
    /// `idx` lands on `min(newlen, oldlen) - 1`.
    pub fn resize(&mut self, newlen: usize) {
        let oldlen = self.entries.len() as c_int;
        let newlen_i = newlen as c_int;
        if newlen_i == oldlen {
            return;
        }
        let mut temp: Vec<Option<HistEntry>> = (0..newlen).map(|_| None).collect();
        let j = self.idx;
        if j >= 0 {
            // l1: slots [0..=j] to keep; l2: kept slots wrapped at the end.
            let l1 = (j + 1).min(newlen_i);
            let l2 = newlen_i.min(oldlen) - l1;
            let i1 = j + 1 - l1;
            let i2 = l1.max(oldlen - newlen_i + l1);
            for k in 0..l2 {
                temp[k as usize] = self.entries[(i2 + k) as usize].take();
            }
            for k in 0..l1 {
                temp[(l2 + k) as usize] = self.entries[(i1 + k) as usize].take();
            }
        }
        self.idx = if j < 0 { -1 } else { newlen_i.min(oldlen) - 1 };
        self.entries = temp;
    }

    /// Append an entry, overwriting the oldest slot when full.
    pub fn add(&mut self, text: &[u8], sep: u8, now: Timestamp) {
        self.idx += 1;
        if self.idx == self.entries.len() as c_int {
            self.idx = 0;
        }
        self.num += 1;
        self.entries[self.idx as usize] = Some(HistEntry {
            number: self.num,
            text: to_cstring(text),
            sep,
            timestamp: now,
            extra: ExtraData::NONE,
        });
    }

    /// If `text` is already in the ring (searching newest to oldest,
    /// stopping at the first vacant slot), move it to the front with a
    /// fresh number and timestamp and report `true`. `sep` must also match
    /// when given (search history distinguishes `/` from `?` entries).
    pub fn move_to_front(&mut self, text: &[u8], sep: Option<u8>, now: Timestamp) -> bool {
        if self.idx < 0 {
            return false;
        }
        let hislen = self.entries.len() as c_int;
        let start = self.idx;
        let mut i = start;
        let found = loop {
            match self.entries[i as usize].as_ref() {
                None => return false,
                Some(e) => {
                    if e.text.as_bytes() == text && sep.map_or(true, |s| s == e.sep) {
                        break i;
                    }
                }
            }
            i -= 1;
            if i < 0 {
                i = hislen - 1;
            }
            if i == start {
                return false;
            }
        };
        let mut entry = self.entries[found as usize].take().expect("slot occupied");
        let mut i = found;
        while i != start {
            let next = (i + 1) % hislen;
            self.entries[i as usize] = self.entries[next as usize].take();
            i = next;
        }
        self.num += 1;
        entry.number = self.num;
        entry.timestamp = now;
        entry.extra = ExtraData::NONE;
        self.entries[start as usize] = Some(entry);
        true
    }

    /// Remove the newest entry, stepping `idx` back (used when a search
    /// from a mapping replaces the previous search from the same mapping).
    pub fn drop_newest(&mut self) {
        self.entries[self.idx as usize] = None;
        self.num -= 1;
        self.idx -= 1;
        if self.idx < 0 {
            self.idx = self.entries.len() as c_int - 1;
        }
    }

    /// Map a history number to a raw slot index: positive `num` finds the
    /// entry with that sequence number, negative counts back from the
    /// newest (-1 = newest). Returns -1 if there is no such entry.
    pub fn calc_idx(&self, num: c_int) -> c_int {
        let hislen = self.entries.len() as c_int;
        let mut i = self.idx;
        if hislen == 0 || i < 0 || num == 0 {
            return -1;
        }
        if num > 0 {
            let mut wrapped = false;
            while self.number_at(i) > num {
                i -= 1;
                if i >= 0 {
                    continue;
                }
                if wrapped {
                    break;
                }
                i += hislen;
                wrapped = true;
            }
            if i >= 0 && self.number_at(i) == num && self.get(i).is_some() {
                return i;
            }
        } else if -i64::from(num) <= i64::from(hislen) {
            i += num + 1;
            if i < 0 {
                i += hislen;
            }
            if self.get(i).is_some() {
                return i;
            }
        }
        -1
    }

    /// Drop every entry and reset numbering.
    pub fn clear(&mut self) {
        for slot in &mut self.entries {
            *slot = None;
        }
        self.idx = -1;
        self.num = 0;
    }

    /// Delete every entry `matches` accepts, compacting survivors toward
    /// the newest slot (entries keep their numbers). Returns whether
    /// anything matched.
    pub fn delete_matching(&mut self, mut matches: impl FnMut(&HistEntry) -> bool) -> bool {
        if self.idx < 0 {
            return false;
        }
        let hislen = self.entries.len() as c_int;
        let idx = self.idx;
        let mut found = false;
        let mut i = idx;
        let mut last = idx;
        loop {
            let matched = match self.entries[i as usize].as_ref() {
                None => break,
                Some(e) => matches(e),
            };
            if matched {
                found = true;
                self.entries[i as usize] = None;
            } else {
                if i != last {
                    self.entries[last as usize] = self.entries[i as usize].take();
                }
                last -= 1;
                if last < 0 {
                    last += hislen;
                }
            }
            i -= 1;
            if i < 0 {
                i += hislen;
            }
            if i == idx {
                break;
            }
        }
        if self.entries[idx as usize].is_none() {
            self.idx = -1;
        }
        found
    }

    /// Delete the entry at raw slot `i`, shifting newer entries down and
    /// stepping `idx` back one slot (which may leave it on a vacant slot —
    /// the C behavior when the newest entry is deleted).
    pub fn delete_at(&mut self, mut i: c_int) {
        let hislen = self.entries.len() as c_int;
        let idx = self.idx;
        self.entries[i as usize] = None;
        while i != idx {
            let j = (i + 1) % hislen;
            self.entries[i as usize] = self.entries[j as usize].take();
            i = j;
        }
        self.idx = if idx > 0 { idx - 1 } else { idx - 1 + hislen };
    }

    /// Raw slot indexes of the live entries, oldest first: the first
    /// occupied slot after `idx`, forward (wrapping) until a vacant slot
    /// or until `idx` itself has been yielded.
    fn oldest_first_indices(&self) -> Vec<c_int> {
        let mut out = Vec::new();
        if self.idx < 0 {
            return out;
        }
        let hislen = self.entries.len() as c_int;
        let idx = self.idx;
        let mut p = idx;
        loop {
            p = (p + 1) % hislen;
            if self.entries[p as usize].is_some() {
                break;
            }
            if p == idx {
                return out;
            }
        }
        loop {
            if self.entries[p as usize].is_none() {
                break;
            }
            out.push(p);
            if p == idx {
                break;
            }
            p = (p + 1) % hislen;
        }
        out
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
        return hist_char2type(unsafe { get_cmdline_firstc() });
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
    assert!(histype != HIST_DEFAULT);
    if cmdmod.with(|m| m.cmod_flags as c_int & CMOD_KEEPPATTERNS != 0) && histype == HIST_SEARCH {
        return;
    }
    // SAFETY: wall-clock read, no editor state involved.
    let now = unsafe { os_time() };
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
fn clr_history(histype: c_int) -> c_int {
    if get_hislen() != 0 && valid_histype(histype) {
        HISTORY.with_mut(|h| h[histype as usize].clear());
        return OK;
    }
    FAIL
}

/// Delete all entries of `histype` matching the vim regex `pat`.
///
/// # Safety
///
/// `pat` must be a valid NUL-terminated string.
unsafe fn del_history_entry(histype: c_int, pat: *const c_char) -> bool {
    if get_hislen() == 0 || !valid_histype(histype) || *pat == 0 || get_hisidx(histype) < 0 {
        return false;
    }
    let mut regmatch = regmatch_T {
        regprog: vim_regcomp(pat, RE_MAGIC + RE_STRING),
        startp: [core::ptr::null_mut(); 10],
        endp: [core::ptr::null_mut(); 10],
        rm_matchcol: 0,
        rm_ic: false,
    };
    if regmatch.regprog.is_null() {
        return false;
    }
    let found = HISTORY.with_mut(|h| {
        h[histype as usize].delete_matching(|e| {
            // SAFETY: entry text is NUL-terminated and outlives the call.
            unsafe { vim_regexec(&raw mut regmatch, e.c_ptr(), 0) }
        })
    });
    vim_regfree(regmatch.regprog);
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

/// "histadd()" function
pub unsafe extern "C" fn f_histadd(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = 0;
    if check_secure() {
        return;
    }
    let name = tv_get_string_chk(argvars);
    let histype = if name.is_null() {
        HIST_INVALID
    } else {
        get_histtype(CStr::from_ptr(name).to_bytes(), false)
    };
    if histype == HIST_INVALID {
        return;
    }
    let mut buf = [0 as c_char; 65];
    let entry = tv_get_string_buf(argvars.offset(1), buf.as_mut_ptr());
    if *entry == 0 {
        return;
    }
    init_history();
    add_to_history(histype, CStr::from_ptr(entry).to_bytes(), false, 0);
    (*rettv).vval.v_number = 1;
}

/// "histdel()" function
pub unsafe extern "C" fn f_histdel(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    const VAR_UNKNOWN: VarType = 0;
    const VAR_NUMBER: VarType = 1;
    let name = tv_get_string_chk(argvars);
    let n = if name.is_null() {
        0
    } else {
        let histype = get_histtype(CStr::from_ptr(name).to_bytes(), false);
        let arg = argvars.offset(1);
        if (*arg).v_type == VAR_UNKNOWN {
            // Only one argument: clear the whole history.
            clr_history(histype)
        } else if (*arg).v_type == VAR_NUMBER {
            // Delete by history number.
            del_history_idx(histype, tv_get_number(arg) as c_int) as c_int
        } else {
            // Delete by regex.
            let mut buf = [0 as c_char; 65];
            del_history_entry(histype, tv_get_string_buf(arg, buf.as_mut_ptr())) as c_int
        }
    };
    (*rettv).vval.v_number = varnumber_T::from(n);
}

/// "histget()" function
pub unsafe extern "C" fn f_histget(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    const VAR_UNKNOWN: VarType = 0;
    const VAR_STRING: VarType = 2;
    let name = tv_get_string_chk(argvars);
    if name.is_null() {
        (*rettv).vval.v_string = core::ptr::null_mut();
    } else {
        let histype = get_histtype(CStr::from_ptr(name).to_bytes(), false);
        let num = if (*argvars.offset(1)).v_type == VAR_UNKNOWN {
            get_history_idx(histype)
        } else {
            tv_get_number_chk(argvars.offset(1), core::ptr::null_mut()) as c_int
        };
        let idx = calc_hist_idx(histype, num);
        (*rettv).vval.v_string = match hist_entry_ref(histype, idx) {
            None => xstrnsave(b"\0".as_ptr() as *const c_char, 0),
            Some(e) => xstrnsave(e.text, e.len),
        };
    }
    (*rettv).v_type = VAR_STRING;
}

/// "histnr()" function
pub unsafe extern "C" fn f_histnr(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let name = tv_get_string_chk(argvars);
    let histype = if name.is_null() {
        HIST_INVALID
    } else {
        get_histtype(CStr::from_ptr(name).to_bytes(), false)
    };
    (*rettv).vval.v_number = varnumber_T::from(if histype == HIST_INVALID {
        HIST_INVALID
    } else {
        get_history_idx(histype)
    });
}

/// ":history" command: list history entries, optionally filtered by
/// history name ("cmd", ":", "all", ...) and a number range.
pub unsafe extern "C" fn ex_history(eap: *mut exarg_T) {
    let mut histype1 = HIST_CMD;
    let mut histype2 = HIST_CMD;
    let mut hisidx1: c_int = 1;
    let mut hisidx2: c_int = -1;
    let arg: *mut c_char = (*eap).arg;
    let mut end: *mut c_char;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const c_char);
    if get_hislen() == 0 {
        msg(
            gettext(b"'history' option is zero\0".as_ptr() as *const c_char),
            0,
        );
        return;
    }
    let first = *arg as u8;
    if !(first.is_ascii_digit() || first == b'-' || first == b',') {
        end = arg;
        while {
            let b = *end as u8;
            b.is_ascii_alphabetic() || SHORT_NAMES.contains(&b)
        } {
            end = end.add(1);
        }
        let name = core::slice::from_raw_parts(arg as *const u8, end.offset_from(arg) as usize);
        histype1 = get_histtype(name, false);
        if histype1 == HIST_INVALID {
            let all = b"all";
            if name.len() <= all.len() && all[..name.len()].eq_ignore_ascii_case(name) {
                histype1 = 0;
                histype2 = HIST_COUNT as c_int - 1;
            } else {
                semsg(gettext(&raw const e_trailing_arg as *const c_char), arg);
                return;
            }
        } else {
            histype2 = histype1;
        }
    } else {
        end = arg;
    }
    if get_list_range(&raw mut end, &raw mut hisidx1, &raw mut hisidx2) == FAIL || *end != 0 {
        if *end != 0 {
            semsg(gettext(&raw const e_trailing_arg as *const c_char), end);
        } else {
            semsg(gettext(&raw const e_val_too_large as *const c_char), arg);
        }
        return;
    }
    while !got_int.get() && histype1 <= histype2 {
        let name = HISTORY_NAMES[histype1 as usize];
        let name = String::from_utf8_lossy(&name[..name.len() - 1]);
        let title = format!("\n      #  {name} history\0");
        msg_puts_title(title.as_ptr() as *const c_char);
        let hislen = get_hislen();
        let idx = get_hisidx(histype1);
        let number_at = |i: c_int| HISTORY.with(|h| h[histype1 as usize].number_at(i));
        // Negative range bounds count back from the newest entry.
        let resolve = |bound: c_int| {
            if bound >= 0 {
                bound
            } else if -i64::from(bound) > i64::from(hislen) {
                0
            } else {
                number_at((hislen + bound + idx + 1) % hislen)
            }
        };
        let j = resolve(hisidx1);
        let k = resolve(hisidx2);
        if idx >= 0 && j <= k {
            // List from the oldest slot forward, ending at the newest.
            let mut i = idx + 1;
            while !got_int.get() {
                if i == hislen {
                    i = 0;
                }
                if let Some(entry) = hist_entry_ref(histype1, i) {
                    let num = number_at(i);
                    if num >= j && num <= k && !message_filtered(entry.text) {
                        msg_putchar('\n' as c_int);
                        let len = snprintf(
                            IObuff.ptr() as *mut c_char,
                            IOSIZE as size_t,
                            b"%c%6d  \0".as_ptr() as *const c_char,
                            if i == idx { '>' as c_int } else { ' ' as c_int },
                            num,
                        );
                        if vim_strsize(entry.text) > Columns.get() - 10 {
                            trunc_string(
                                entry.text,
                                (IObuff.ptr() as *mut c_char).offset(len as isize),
                                Columns.get() - 10,
                                IOSIZE - len,
                            );
                        } else {
                            xstrlcpy(
                                (IObuff.ptr() as *mut c_char).offset(len as isize),
                                entry.text,
                                (IOSIZE - len) as size_t,
                            );
                        }
                        msg_outtrans(IObuff.ptr() as *mut c_char, 0, false);
                    }
                }
                if i == idx {
                    break;
                }
                i += 1;
            }
        }
        histype1 += 1;
    }
}

/// Completion source for `:history` arguments: the one-character names,
/// the long names, then "all".
pub unsafe extern "C" fn get_history_arg(xp: *mut expand_T, idx: c_int) -> *mut c_char {
    let short_count = SHORT_NAMES.len() as c_int;
    if (0..short_count).contains(&idx) {
        (*xp).xp_buf[0] = SHORT_NAMES[idx as usize] as c_char;
        (*xp).xp_buf[1] = 0;
        return (*xp).xp_buf.as_mut_ptr();
    }
    let i = (idx - short_count) as usize;
    if i < HIST_COUNT {
        return HISTORY_NAMES[i].as_ptr() as *mut c_char;
    }
    if i == HIST_COUNT {
        return b"all\0".as_ptr() as *const c_char as *mut c_char;
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
                xfree(se.text.cast::<c_void>());
                xfree(se.additional_data.cast::<c_void>());
                continue;
            }
            let text = to_cstring(CStr::from_ptr(se.text).to_bytes());
            xfree(se.text.cast::<c_void>());
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
