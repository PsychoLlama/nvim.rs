//! Reading a list from Vimscript.
//!
//! [`qf_get_properties`] is `getqflist({what})`: [`qf_getprop_keys2flags`]
//! turns the requested keys into a flag set and one `qf_getprop_*` helper
//! answers each. [`get_errorlist`] is the plain, no-argument form, whose
//! entries [`get_qfline_items`] builds.
//!
//! Every key in the answer is written by one of the four `add_*` helpers.
//! `tv_dict_add_*` take a key pointer *and* its length and copy exactly that
//! many bytes, so a Rust `&str` is the key type here — the transpile spelled
//! each one out as a C literal plus a `size_of` of its array type.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::types::{VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VarLock, kListLenMayKnow};
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

/// Add a number under `key`.
///
/// # Safety
///
/// `dict` must be a live dictionary.
unsafe fn add_nr(dict: *mut dict_T, key: &str, value: varnumber_T) -> Result<(), KeyTaken> {
    // SAFETY: the caller's dictionary; the key is `key.len()` bytes long.
    Ok(unsafe { tv_dict_add_nr(dict, key.as_ptr().cast(), key.len(), value) }?)
}

/// Add a string under `key`. A null `value` is stored as the empty string,
/// which is what every caller here wants for a field it never set.
///
/// # Safety
///
/// `dict` must be a live dictionary and `value` null or NUL-terminated.
unsafe fn add_str(dict: *mut dict_T, key: &str, value: *const c_char) -> Result<(), KeyTaken> {
    let value = if value.is_null() { c"".as_ptr() } else { value };
    // SAFETY: the caller's dictionary and string.
    Ok(unsafe { tv_dict_add_str(dict, key.as_ptr().cast(), key.len(), value) }?)
}

/// Add a list under `key`, which takes over the reference.
///
/// # Safety
///
/// `dict` and `list` must be live.
unsafe fn add_list(dict: *mut dict_T, key: &str, list: *mut list_T) -> Result<(), KeyTaken> {
    // SAFETY: the caller's dictionary and list.
    Ok(unsafe { tv_dict_add_list(dict, key.as_ptr().cast(), key.len(), list) }?)
}

/// Add a copy of `tv` under `key`.
///
/// # Safety
///
/// `dict` must be live and `tv` a live value.
unsafe fn add_tv(dict: *mut dict_T, key: &str, tv: *mut typval_T) -> Result<(), KeyTaken> {
    // SAFETY: the caller's dictionary and value.
    Ok(unsafe { tv_dict_add_tv(dict, key.as_ptr().cast(), key.len(), tv) }?)
}

/// The entry of `what` under `key`, or null.
///
/// # Safety
///
/// `what` must be null or a live dictionary.
unsafe fn find(what: *const dict_T, key: &str) -> *mut dictitem_T {
    // SAFETY: the caller's dictionary; the key is `key.len()` bytes long.
    unsafe { tv_dict_find(what, key.as_ptr().cast(), key.len() as ptrdiff_t) }
}

/// Whether `what` names `key` at all — its value is never looked at, since
/// asking for a key is the whole request.
///
/// # Safety
///
/// `what` must be null or a live dictionary.
unsafe fn asked_for(what: *const dict_T, key: &str) -> bool {
    // SAFETY: forwarded from the caller.
    !unsafe { find(what, key) }.is_null()
}

/// Append one entry to `list`, as the dictionary `getqflist()` reports.
///
/// Cannot fail. The dictionary is fresh and each key is written once, so the
/// only way one of the writes could be refused is a null item, which cannot
/// happen — upstream `abort()`s there rather than reporting anything, and so
/// does this. Answering `OK`/`FAIL` made two callers check for a failure that
/// is not reachable.
///
/// # Safety
///
/// `qfp` must be a live entry and `list` a live list.
unsafe fn get_qfline_items(qfp: *mut qfline_T, list: *mut list_T) {
    // SAFETY: forwarded from the caller.
    // Handle entries with a non-existing buffer number.
    let mut bufnum = unsafe { (*qfp).qf_fnum };
    if bufnum != 0 && find_buf(bufnum).is_none() {
        bufnum = 0;
    }

    let dict = unsafe { tv_dict_alloc() };
    unsafe { tv_list_append_dict(list, dict) };

    // The type is one character, or NUL for "none".
    let kind = [unsafe { (*qfp).qf_type }, 0];

    if unsafe { add_nr(dict, "bufnr", bufnum as varnumber_T) }.is_err()
        || unsafe { add_nr(dict, "lnum", (*qfp).qf_lnum as varnumber_T) }.is_err()
        || unsafe { add_nr(dict, "end_lnum", (*qfp).qf_end_lnum as varnumber_T) }.is_err()
        || unsafe { add_nr(dict, "col", (*qfp).qf_col as varnumber_T) }.is_err()
        || unsafe { add_nr(dict, "end_col", (*qfp).qf_end_col as varnumber_T) }.is_err()
        || unsafe { add_nr(dict, "vcol", (*qfp).qf_viscol as varnumber_T) }.is_err()
        || unsafe { add_nr(dict, "nr", (*qfp).qf_nr as varnumber_T) }.is_err()
        || unsafe { add_str(dict, "module", (*qfp).qf_module) }.is_err()
        || unsafe { add_str(dict, "pattern", (*qfp).qf_pattern) }.is_err()
        || unsafe { add_str(dict, "text", (*qfp).qf_text) }.is_err()
        || unsafe { add_str(dict, "type", kind.as_ptr()) }.is_err()
        || (unsafe { (*qfp).qf_user_data.v_type } != VAR_UNKNOWN
            && unsafe { add_tv(dict, "user_data", &raw mut (*qfp).qf_user_data) }.is_err())
        || unsafe { add_nr(dict, "valid", (*qfp).qf_valid as varnumber_T) }.is_err()
    {
        // Only a NULL dict_item would cause this, which cannot happen.
        unsafe { abort() };
    }
}

/// Fill `list` with the entries of list `qf_idx`, or with just entry `eidx`
/// when that is positive. A negative `eidx` asks for nothing at all.
///
/// # Safety
///
/// `qi` must be null or a live stack and `list` a live list.
pub(crate) unsafe fn get_errorlist(
    qi_arg: *mut qf_info_T,
    wp: Option<Win>,
    mut qf_idx: c_int,
    eidx: c_int,
    list: *mut list_T,
) -> Result<(), QfError> {
    // SAFETY: forwarded from the caller.
    let mut qi = qi_arg;
    if qi.is_null() {
        qi = match wp {
            Some(wp) => win_loclist(wp),
            None => QfStack::Global.raw(),
        };
        if qi.is_null() {
            return Err(QfError::NoSuchList);
        }
    }

    if eidx < 0 {
        return Ok(());
    }
    if qf_idx == INVALID_QFIDX {
        qf_idx = unsafe { (*qi).qf_curlist };
    }
    if qf_idx >= unsafe { (*qi).qf_listcount } {
        return Err(QfError::NoSuchList);
    }

    let qfl = unsafe { qf_get_list(qi, qf_idx) };
    if unsafe { qf_list_empty(qfl) } {
        return Err(QfError::NoSuchList);
    }

    let mut qfp = unsafe { (*qfl).qf_start };
    let mut i = 1;
    while !got_int.get() && i <= unsafe { (*qfl).qf_count } && !qfp.is_null() {
        if eidx > 0 {
            if eidx == i {
                unsafe { get_qfline_items(qfp, list) };
                return Ok(());
            }
        } else {
            unsafe { get_qfline_items(qfp, list) };
        }
        i += 1;
        qfp = unsafe { (*qfp).qf_next };
    }
    Ok(())
}

/// `getqflist({'lines': […]})`: parse the given lines with `'errorformat'`
/// — the `'efm'` key overrides it — into a throwaway list and answer the
/// entries, without touching any real list.
///
/// # Safety
///
/// `what`, `di` and `retdict` must be live.
unsafe fn qf_get_list_from_lines(
    what: *mut dict_T,
    di: *mut dictitem_T,
    retdict: *mut dict_T,
) -> Result<(), QfError> {
    // SAFETY: forwarded from the caller.
    if unsafe { (*di).di_tv.v_type } != VAR_LIST || unsafe { (*di).di_tv.vval.v_list }.is_null() {
        return Err(QfError::BadValue);
    }

    let mut errorformat = p_efm.get();
    let efm_di = unsafe { find(what, "efm") };
    if !efm_di.is_null() {
        if unsafe { (*efm_di).di_tv.v_type } != VAR_STRING
            || unsafe { (*efm_di).di_tv.vval.v_string }.is_null()
        {
            return Err(QfError::BadValue);
        }
        errorformat = unsafe { (*efm_di).di_tv.vval.v_string };
    }

    // Only a List value is supported.
    let l = unsafe { tv_list_alloc(kListLenMayKnow as ptrdiff_t) };
    let qi = qf_alloc_stack(QFLT_INTERNAL, 1);
    let parsed = unsafe {
        qf_init_ext(
            qi,
            0,
            ptr::null(),
            None,
            &raw mut (*di).di_tv,
            errorformat,
            true,
            0,
            0,
            ptr::null(),
            ptr::null_mut(),
        )
    } > 0;
    if parsed {
        // Whether the throwaway list had entries is not this answer:
        // parsing nothing out of the lines is still a successful read.
        let _ = unsafe { get_errorlist(qi, None, 0, 0, l) };
        unsafe { qf_free(qf_get_list(qi, 0)) };
    }
    unsafe { qf_free_lists(qi) };

    unsafe { add_list(retdict, "items", l) }?;
    Ok(())
}

/// The window id of the quickfix window showing this stack, or 0.
///
/// # Safety
///
/// `qi` must be null or a live stack.
unsafe fn qf_winid(qi: *mut qf_info_T) -> c_int {
    // SAFETY: forwarded from the caller.
    if qi.is_null() {
        return 0;
    }
    // SAFETY: the caller's promise -- a live stack, checked non-null.
    let Some(win) = qf_find_win(unsafe { Qi::new(qi) }) else {
        return 0;
    };
    win.handle as c_int
}

/// The number of the buffer holding the quickfix window's contents, or 0
/// when there is no such buffer any more.
///
/// # Safety
///
/// `qi` must be null or a live stack, and `retdict` live.
unsafe fn qf_getprop_qfbufnr(qi: *const qf_info_T, retdict: *mut dict_T) -> Result<(), KeyTaken> {
    // SAFETY: forwarded from the caller.
    let mut bufnum = 0;
    if !qi.is_null() && find_buf(unsafe { (*qi).qf_bufnr }).is_some() {
        bufnum = unsafe { (*qi).qf_bufnr };
    }
    unsafe { add_nr(retdict, "qfbufnr", bufnum as varnumber_T) }
}

/// The `what` keys, in the order the flag set numbers them. `filewinid` is
/// the odd one out: it is answered for a location list only, both when it
/// is asked for by name and when `all` asks for everything.
const GETLIST_KEYS: [(&str, c_int); 12] = [
    ("title", QF_GETLIST_TITLE as c_int),
    ("items", QF_GETLIST_ITEMS as c_int),
    ("nr", QF_GETLIST_NR as c_int),
    ("winid", QF_GETLIST_WINID as c_int),
    ("context", QF_GETLIST_CONTEXT as c_int),
    ("id", QF_GETLIST_ID as c_int),
    ("idx", QF_GETLIST_IDX as c_int),
    ("size", QF_GETLIST_SIZE as c_int),
    ("changedtick", QF_GETLIST_TICK as c_int),
    ("filewinid", QF_GETLIST_FILEWINID as c_int),
    ("qfbufnr", QF_GETLIST_QFBUFNR as c_int),
    ("quickfixtextfunc", QF_GETLIST_QFTF as c_int),
];

/// Which properties `what` asks for.
///
/// # Safety
///
/// `what` must be null or a live dictionary.
unsafe fn qf_getprop_keys2flags(what: *const dict_T, loclist: bool) -> c_int {
    // SAFETY: forwarded from the caller.
    let mut flags = QF_GETLIST_NONE as c_int;
    if unsafe { asked_for(what, "all") } {
        flags |= QF_GETLIST_ALL as c_int;
        if !loclist {
            flags &= !(QF_GETLIST_FILEWINID as c_int);
        }
    }
    for (key, flag) in GETLIST_KEYS {
        // `filewinid` belongs to a location list only.
        if flag == QF_GETLIST_FILEWINID as c_int && !loclist {
            continue;
        }
        if unsafe { asked_for(what, key) } {
            flags |= flag;
        }
    }
    flags
}

/// Which list `what` names, through its `nr` or `id` key, or the current one
/// when it names neither. Answers `None` for a list that is not on the stack,
/// and for a `nr`/`id` of the wrong type — upstream's `INVALID_QFIDX`
/// sentinel, which every caller had to know about by name.
///
/// # Safety
///
/// `qi` must be a live stack and `what` null or a live dictionary.
unsafe fn qf_getprop_qfidx(qi: *mut qf_info_T, what: *mut dict_T) -> Option<c_int> {
    // SAFETY: forwarded from the caller.
    let mut qf_idx = unsafe { (*qi).qf_curlist };

    // Use the specified list, or the last list, or the current one.
    let di = unsafe { find(what, "nr") };
    if !di.is_null() {
        if unsafe { (*di).di_tv.v_type } == VAR_NUMBER {
            // For zero, use the current list.
            if unsafe { (*di).di_tv.vval.v_number } != 0 {
                qf_idx = unsafe { (*di).di_tv.vval.v_number } as c_int - 1;
                if qf_idx < 0 || qf_idx >= unsafe { (*qi).qf_listcount } {
                    qf_idx = INVALID_QFIDX;
                }
            }
        } else if unsafe { (*di).di_tv.v_type } == VAR_STRING
            && unsafe { strequal((*di).di_tv.vval.v_string, c"$".as_ptr()) }
        {
            // Get the last list.
            qf_idx = unsafe { (*qi).qf_listcount } - 1;
        } else {
            qf_idx = INVALID_QFIDX;
        }
    }

    // An id overrides the number.
    let di = unsafe { find(what, "id") };
    if !di.is_null() {
        if unsafe { (*di).di_tv.v_type } == VAR_NUMBER {
            // For zero, use the current list.
            if unsafe { (*di).di_tv.vval.v_number } != 0 {
                qf_idx = unsafe { qf_id2nr(qi, (*di).di_tv.vval.v_number as c_uint) };
            }
        } else {
            qf_idx = INVALID_QFIDX;
        }
    }

    (qf_idx != INVALID_QFIDX).then_some(qf_idx)
}

/// What `getqflist({what})` answers when there is no list to read: the
/// requested keys with empty values.
///
/// # Safety
///
/// `qi` must be null or a live stack, and `retdict` live.
unsafe fn qf_getprop_defaults(
    qi: *mut qf_info_T,
    flags: c_int,
    locstack: bool,
    retdict: *mut dict_T,
) -> Result<(), KeyTaken> {
    // SAFETY: forwarded from the caller.
    let wanted = |flag: c_uint| flags & flag as c_int != 0;

    // `?` is upstream's `if (status == OK && ...)` ladder: the first key
    // the dictionary refuses stops the rest from being written.
    if wanted(QF_GETLIST_TITLE) {
        unsafe { add_str(retdict, "title", ptr::null()) }?;
    }
    if wanted(QF_GETLIST_ITEMS) {
        let l = unsafe { tv_list_alloc(kListLenMayKnow as ptrdiff_t) };
        unsafe { add_list(retdict, "items", l) }?;
    }
    if wanted(QF_GETLIST_NR) {
        unsafe { add_nr(retdict, "nr", 0) }?;
    }
    if wanted(QF_GETLIST_WINID) {
        unsafe { add_nr(retdict, "winid", qf_winid(qi) as varnumber_T) }?;
    }
    if wanted(QF_GETLIST_CONTEXT) {
        unsafe { add_str(retdict, "context", ptr::null()) }?;
    }
    if wanted(QF_GETLIST_ID) {
        unsafe { add_nr(retdict, "id", 0) }?;
    }
    if wanted(QF_GETLIST_IDX) {
        unsafe { add_nr(retdict, "idx", 0) }?;
    }
    if wanted(QF_GETLIST_SIZE) {
        unsafe { add_nr(retdict, "size", 0) }?;
    }
    if wanted(QF_GETLIST_TICK) {
        unsafe { add_nr(retdict, "changedtick", 0) }?;
    }
    if locstack && wanted(QF_GETLIST_FILEWINID) {
        unsafe { add_nr(retdict, "filewinid", 0) }?;
    }
    if wanted(QF_GETLIST_QFBUFNR) {
        unsafe { qf_getprop_qfbufnr(qi, retdict) }?;
    }
    if wanted(QF_GETLIST_QFTF) {
        unsafe { add_str(retdict, "quickfixtextfunc", ptr::null()) }?;
    }
    Ok(())
}

/// The id of the window the location list belongs to, which only a location
/// list window has.
///
/// # Safety
///
/// `qi` must be a live stack and `retdict` live.
unsafe fn qf_getprop_filewinid(
    wp: Option<Win>,
    qi: *const qf_info_T,
    retdict: *mut dict_T,
) -> Result<(), KeyTaken> {
    let winid = wp
        .filter(|&wp| is_ll_window(wp))
        .and_then(|_| qf_find_win_with_loclist(qi))
        .map_or(0, |ll_wp| ll_wp.handle);
    // SAFETY: forwarded from the caller -- a live dictionary.
    unsafe { add_nr(retdict, "filewinid", winid as varnumber_T) }
}

/// The entries of the list, or of just entry `eidx`.
///
/// An empty list is a perfectly good answer, so neither the walk's refusal
/// nor the write's is passed on — upstream discarded both here too.
///
/// # Safety
///
/// `qi` must be a live stack and `retdict` live.
unsafe fn qf_getprop_items(qi: *mut qf_info_T, qf_idx: c_int, eidx: c_int, retdict: *mut dict_T) {
    // SAFETY: forwarded from the caller.
    let l = unsafe { tv_list_alloc(kListLenMayKnow as ptrdiff_t) };
    let _ = unsafe { get_errorlist(qi, None, qf_idx, eidx, l) };
    let _ = unsafe { add_list(retdict, "items", l) };
}

/// The arbitrary value `setqflist()` attached to the list, or the empty
/// string when it has none.
///
/// # Safety
///
/// `qfl` must be a live list and `retdict` live.
unsafe fn qf_getprop_ctx(qfl: *mut qf_list_T, retdict: *mut dict_T) -> Result<(), KeyTaken> {
    // SAFETY: forwarded from the caller.
    if unsafe { (*qfl).qf_ctx }.is_null() {
        return unsafe { add_str(retdict, "context", ptr::null()) };
    }
    let di = unsafe { tv_dict_item_alloc_len(c"context".as_ptr(), "context".len()) };
    unsafe { tv_copy((*qfl).qf_ctx, &raw mut (*di).di_tv) };
    let status = unsafe { tv_dict_add(retdict, di) };
    if status.is_err() {
        // A refused item is still ours to free.
        unsafe { tv_dict_item_free(di) };
    }
    Ok(status?)
}

/// The index of the current entry, or of the entry `eidx` names.
///
/// # Safety
///
/// `qfl` must be a live list and `retdict` live.
unsafe fn qf_getprop_idx(
    qfl: *mut qf_list_T,
    mut eidx: c_int,
    retdict: *mut dict_T,
) -> Result<(), KeyTaken> {
    // SAFETY: forwarded from the caller.
    if eidx == 0 {
        eidx = unsafe { (*qfl).qf_index };
        if unsafe { qf_list_empty(qfl) } {
            eidx = 0;
        }
    }
    unsafe { add_nr(retdict, "idx", eidx as varnumber_T) }
}

/// The list's `'quickfixtextfunc'` callback, or the empty string.
///
/// # Safety
///
/// `qfl` must be a live list and `retdict` live.
unsafe fn qf_getprop_qftf(qfl: *mut qf_list_T, retdict: *mut dict_T) -> Result<(), KeyTaken> {
    // SAFETY: forwarded from the caller.
    if !unsafe { &(*qfl).qf_qftf_cb }.is_set() {
        return unsafe { add_str(retdict, "quickfixtextfunc", ptr::null()) };
    }
    let mut tv = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    };
    unsafe { callback_put(&raw mut (*qfl).qf_qftf_cb, &raw mut tv) };
    let status = unsafe { add_tv(retdict, "quickfixtextfunc", &raw mut tv) };
    unsafe { tv_clear(&raw mut tv) };
    status
}

/// `getqflist({what})` and `getloclist(nr, {what})`: fill `retdict` with the
/// properties `what` names.
///
/// # Safety
///
/// `what` and `retdict` must be live.
pub(crate) unsafe fn qf_get_properties(
    wp: Option<Win>,
    what: *mut dict_T,
    retdict: *mut dict_T,
) -> Result<(), QfError> {
    // SAFETY: forwarded from the caller.
    let mut qi = QfStack::Global.raw();

    // A 'lines' key asks about lines the caller supplies, not about a
    // list at all.
    let lines = unsafe { find(what, "lines") };
    if !lines.is_null() {
        return unsafe { qf_get_list_from_lines(what, lines, retdict) };
    }

    if let Some(wp) = wp {
        qi = win_loclist(wp);
    }

    let flags = unsafe { qf_getprop_keys2flags(what, wp.is_some()) };

    let named = if unsafe { qf_stack_empty(qi) } {
        None
    } else {
        unsafe { qf_getprop_qfidx(qi, what) }
    };
    let Some(qf_idx) = named else {
        // `?` here is the `From<KeyTaken>` conversion: the defaults can
        // only fail the way the dictionary layer fails.
        unsafe { qf_getprop_defaults(qi, flags, wp.is_some(), retdict) }?;
        return Ok(());
    };

    let qfl = unsafe { qf_get_list(qi, qf_idx) };

    // An 'idx' key asks about one entry rather than the whole list.
    let mut eidx = 0;
    let di = unsafe { find(what, "idx") };
    if !di.is_null() {
        if unsafe { (*di).di_tv.v_type } != VAR_NUMBER {
            return Err(QfError::BadValue);
        }
        eidx = unsafe { (*di).di_tv.vval.v_number } as c_int;
    }

    let wanted = |flag: c_uint| flags & flag as c_int != 0;

    // As in `qf_getprop_defaults`: the first refused key stops the rest.
    if wanted(QF_GETLIST_TITLE) {
        unsafe { add_str(retdict, "title", (*qfl).qf_title) }?;
    }
    if wanted(QF_GETLIST_NR) {
        unsafe { add_nr(retdict, "nr", (qf_idx + 1) as varnumber_T) }?;
    }
    if wanted(QF_GETLIST_WINID) {
        unsafe { add_nr(retdict, "winid", qf_winid(qi) as varnumber_T) }?;
    }
    if wanted(QF_GETLIST_ITEMS) {
        unsafe { qf_getprop_items(qi, qf_idx, eidx, retdict) };
    }
    if wanted(QF_GETLIST_CONTEXT) {
        unsafe { qf_getprop_ctx(qfl, retdict) }?;
    }
    if wanted(QF_GETLIST_ID) {
        unsafe { add_nr(retdict, "id", (*qfl).qf_id as varnumber_T) }?;
    }
    if wanted(QF_GETLIST_IDX) {
        unsafe { qf_getprop_idx(qfl, eidx, retdict) }?;
    }
    if wanted(QF_GETLIST_SIZE) {
        unsafe { add_nr(retdict, "size", (*qfl).qf_count as varnumber_T) }?;
    }
    if wanted(QF_GETLIST_TICK) {
        unsafe { add_nr(retdict, "changedtick", (*qfl).qf_changedtick as varnumber_T) }?;
    }
    if wp.is_some() && wanted(QF_GETLIST_FILEWINID) {
        unsafe { qf_getprop_filewinid(wp, qi, retdict) }?;
    }
    if wanted(QF_GETLIST_QFBUFNR) {
        unsafe { qf_getprop_qfbufnr(qi, retdict) }?;
    }
    if wanted(QF_GETLIST_QFTF) {
        unsafe { qf_getprop_qftf(qfl, retdict) }?;
    }
    Ok(())
}
