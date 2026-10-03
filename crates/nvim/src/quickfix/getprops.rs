//! Reading a list from Vimscript.
//!
//! [`qf_get_properties`] is `getqflist({what})`: [`qf_getprop_keys2flags`]
//! turns the requested keys into a flag set and one `qf_getprop_*` helper
//! answers each. [`get_errorlist`] is the plain, no-argument form, whose
//! entries [`get_qfline_items`] builds.
//!
//! Building the answer allocates Vimscript values and runs no user code, so
//! the list is read through a borrow for the whole of it.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::list::string_tv;
use crate::option::vars::P_EFM;
use crate::types::{QfId, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, kListLenMayKnow};
use core::ffi::{c_int, c_uint};

/// Add a number under `key`.
fn add_nr(dict: &mut Dict, key: &str, value: VarNumber) -> Result<(), KeyTaken> {
    Ok(dict.add_number(key.as_bytes(), value)?)
}

/// Add a copy of `value` under `key`, or the empty string for `None` — which
/// is what every caller here wants for a field it never set.
fn add_str(dict: &mut Dict, key: &str, value: Option<&[u8]>) -> Result<(), KeyTaken> {
    Ok(dict.add_tv(key.as_bytes(), &string_tv(value.unwrap_or(b"")))?)
}

/// Add a list under `key`, which takes over the reference.
fn add_list(dict: &mut Dict, key: &str, list: Option<ListRef>) -> Result<(), KeyTaken> {
    Ok(dict.add_list(key.as_bytes(), list)?)
}

/// Add a copy of `tv` under `key`.
fn add_tv(dict: &mut Dict, key: &str, tv: &TypVal) -> Result<(), KeyTaken> {
    Ok(dict.add_tv(key.as_bytes(), tv)?)
}

/// Whether `what` names `key` at all — its value is never looked at, since
/// asking for a key is the whole request.
fn asked_for(what: &Dict, key: &str) -> bool {
    what.find(key.as_bytes()).is_some()
}

/// Append one entry to `list`, as the dictionary `getqflist()` reports.
///
/// Cannot fail. The dictionary is fresh and each key is written once, so the
/// only way one of the writes could be refused is a null item, which cannot
/// happen — upstream `abort()`s there rather than reporting anything, and so
/// does this.
fn get_qfline_items(entry: &QfEntry, list: &mut List) {
    // Handle entries with a non-existing buffer number.
    let mut bufnum = entry.fnum;
    if bufnum != 0 && find_buf(bufnum).is_none() {
        bufnum = 0;
    }

    let mut dict = tv_dict_alloc();
    // The type is one character, or nothing for "none".
    let kind = [entry.kind.cast_unsigned()];
    let kind: &[u8] = if entry.kind == 0 { b"" } else { &kind };

    let d = &mut *dict;
    let added = add_nr(d, "bufnr", VarNumber::from(bufnum))
        .and_then(|()| add_nr(d, "lnum", VarNumber::from(entry.lnum)))
        .and_then(|()| add_nr(d, "end_lnum", VarNumber::from(entry.end_lnum)))
        .and_then(|()| add_nr(d, "col", VarNumber::from(entry.col)))
        .and_then(|()| add_nr(d, "end_col", VarNumber::from(entry.end_col)))
        .and_then(|()| add_nr(d, "vcol", VarNumber::from(entry.viscol)))
        .and_then(|()| add_nr(d, "nr", VarNumber::from(entry.nr)))
        .and_then(|()| add_str(d, "module", entry.module.as_deref()))
        .and_then(|()| add_str(d, "pattern", entry.pattern.as_deref()))
        .and_then(|()| add_str(d, "text", Some(&entry.text)))
        .and_then(|()| add_str(d, "type", Some(kind)))
        .and_then(|()| {
            if entry.user_data.v_type() == VAR_UNKNOWN {
                Ok(())
            } else {
                add_tv(d, "user_data", &entry.user_data)
            }
        })
        .and_then(|()| add_nr(d, "valid", VarNumber::from(entry.valid)));
    // Only a NULL dict_item would cause this, which cannot happen.
    added.expect("a fresh dictionary takes every key");
    list.push_dict(Some(dict));
}

/// Fill `list` with the entries of list `qf_idx` — the current one for
/// `INVALID_QFIDX` — or with just entry `eidx` when that is positive. A
/// negative `eidx` asks for nothing at all.
pub(crate) fn get_errorlist(
    qi: Qi,
    mut qf_idx: c_int,
    eidx: c_int,
    list: &mut List,
) -> Result<(), QfError> {
    if eidx < 0 {
        return Ok(());
    }
    if qf_idx == INVALID_QFIDX {
        qf_idx = qi.current;
    }
    if qf_idx >= qi.list_count {
        return Err(QfError::NoSuchList);
    }
    let qfl = qi.list(qf_idx);
    if qfl.is_empty() {
        return Err(QfError::NoSuchList);
    }
    if eidx > 0 {
        if let Some(entry) = qfl.nth(eidx) {
            get_qfline_items(entry, list);
        }
        return Ok(());
    }
    for entry in &qfl.entries {
        if got_int.get() {
            break;
        }
        get_qfline_items(entry, list);
    }
    Ok(())
}

/// `getqflist({'lines': […]})`: parse the given lines with `'errorformat'`
/// — the `'efm'` key overrides it — into a throwaway list and answer the
/// entries, without touching any real list.
fn qf_get_list_from_lines(what: &Dict, lines: &TypVal, retdict: &mut Dict) -> Result<(), QfError> {
    if lines.v_type() != VAR_LIST || lines.list_ref().is_none() {
        return Err(QfError::BadValue);
    }

    let errorformat = match what.find(b"efm") {
        None => P_EFM.get(),
        Some(efm_di) => {
            if efm_di.di_tv.v_type() != VAR_STRING || efm_di.di_tv.string_or_null().is_null() {
                return Err(QfError::BadValue);
            }
            crate::memory::XString::from_bytes(crate::eval::list::string_bytes(&efm_di.di_tv))
        }
    };

    // Only a List value is supported.
    let mut l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    let id = new_location_stack(QFLT_INTERNAL, 1);
    let qi = id.stack();
    // A copy of the value: parsing can run autocommands, which could change
    // the dictionary it came from.
    let lines = lines.clone();
    let parsed = qf_init_ext(
        qi,
        0,
        Input::Value(&lines),
        None,
        errorformat.as_cstr(),
        // Only reached with a value, which never reads the buffer's own.
        true,
        false,
        None,
        None,
    ) > 0;
    if parsed {
        // Whether the throwaway list had entries is not this answer:
        // parsing nothing out of the lines is still a successful read.
        let _ = get_errorlist(qi, 0, 0, &mut l);
    }
    free_unreferenced_stack(id);

    add_list(retdict, "items", Some(l))?;
    Ok(())
}

/// The window id of the quickfix window showing this stack, or 0.
fn qf_winid(qi: Option<Qi>) -> c_int {
    qi.and_then(qf_find_win).map_or(0, |win| win.handle)
}

/// The number of the buffer holding the quickfix window's contents, or 0
/// when there is no such buffer any more.
fn qf_getprop_qfbufnr(qi: Option<Qi>, retdict: &mut Dict) -> Result<(), KeyTaken> {
    let bufnum = qi
        .map(|qi| qi.bufnr)
        .filter(|&bufnr| find_buf(bufnr).is_some())
        .unwrap_or(0);
    add_nr(retdict, "qfbufnr", VarNumber::from(bufnum))
}

/// The `what` keys, in the order the flag set numbers them. `filewinid` is
/// the odd one out: it is answered for a location list only, both when it
/// is asked for by name and when `all` asks for everything.
const GETLIST_KEYS: [(&str, GetListProps); 12] = [
    ("title", GetListProps::TITLE),
    ("items", GetListProps::ITEMS),
    ("nr", GetListProps::NR),
    ("winid", GetListProps::WINID),
    ("context", GetListProps::CONTEXT),
    ("id", GetListProps::ID),
    ("idx", GetListProps::IDX),
    ("size", GetListProps::SIZE),
    ("changedtick", GetListProps::TICK),
    ("filewinid", GetListProps::FILEWINID),
    ("qfbufnr", GetListProps::QFBUFNR),
    ("quickfixtextfunc", GetListProps::QFTF),
];

/// Which properties `what` asks for.
fn qf_getprop_keys2flags(what: &Dict, loclist: bool) -> GetListProps {
    let mut flags = GetListProps::NONE;
    if asked_for(what, "all") {
        flags |= GetListProps::ALL;
        if !loclist {
            flags.clear(GetListProps::FILEWINID);
        }
    }
    for (key, flag) in GETLIST_KEYS {
        // `filewinid` belongs to a location list only.
        if flag == GetListProps::FILEWINID && !loclist {
            continue;
        }
        if asked_for(what, key) {
            flags |= flag;
        }
    }
    flags
}

/// Which list `what` names, through its `nr` or `id` key, or the current one
/// when it names neither. Answers `None` for a list that is not on the stack,
/// and for a `nr`/`id` of the wrong type.
fn qf_getprop_qfidx(qi: Qi, what: &Dict) -> Option<c_int> {
    let mut qf_idx = Some(qi.current);

    // Use the specified list, or the last list, or the current one.
    if let Some(di) = what.find(b"nr") {
        if di.di_tv.v_type() == VAR_NUMBER {
            // For zero, use the current list.
            let nr = di.di_tv.number_or_zero();
            if nr != 0 {
                qf_idx = c_int::try_from(nr - 1)
                    .ok()
                    .filter(|&idx| idx >= 0 && idx < qi.list_count);
            }
        } else if di.di_tv.v_type() == VAR_STRING
            && crate::eval::list::string_bytes(&di.di_tv) == b"$"
        {
            // Get the last list.
            qf_idx = Some(qi.list_count - 1);
        } else {
            qf_idx = None;
        }
    }

    // An id overrides the number.
    if let Some(di) = what.find(b"id") {
        if di.di_tv.v_type() == VAR_NUMBER {
            // For zero, use the current list.
            let id = di.di_tv.number_or_zero();
            if id != 0 {
                qf_idx = c_uint::try_from(id).ok().and_then(|id| qi.find_list(id));
            }
        } else {
            qf_idx = None;
        }
    }

    qf_idx
}

/// What `getqflist({what})` answers when there is no list to read: the
/// requested keys with empty values.
fn qf_getprop_defaults(
    qi: Option<Qi>,
    flags: GetListProps,
    locstack: bool,
    retdict: &mut Dict,
) -> Result<(), KeyTaken> {
    let wanted = |flag: GetListProps| flags.has(flag);

    // `?` is upstream's `if (status == OK && ...)` ladder: the first key
    // the dictionary refuses stops the rest from being written.
    if wanted(GetListProps::TITLE) {
        add_str(retdict, "title", None)?;
    }
    if wanted(GetListProps::ITEMS) {
        let l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
        add_list(retdict, "items", Some(l))?;
    }
    if wanted(GetListProps::NR) {
        add_nr(retdict, "nr", 0)?;
    }
    if wanted(GetListProps::WINID) {
        add_nr(retdict, "winid", VarNumber::from(qf_winid(qi)))?;
    }
    if wanted(GetListProps::CONTEXT) {
        add_str(retdict, "context", None)?;
    }
    if wanted(GetListProps::ID) {
        add_nr(retdict, "id", 0)?;
    }
    if wanted(GetListProps::IDX) {
        add_nr(retdict, "idx", 0)?;
    }
    if wanted(GetListProps::SIZE) {
        add_nr(retdict, "size", 0)?;
    }
    if wanted(GetListProps::TICK) {
        add_nr(retdict, "changedtick", 0)?;
    }
    if locstack && wanted(GetListProps::FILEWINID) {
        add_nr(retdict, "filewinid", 0)?;
    }
    if wanted(GetListProps::QFBUFNR) {
        qf_getprop_qfbufnr(qi, retdict)?;
    }
    if wanted(GetListProps::QFTF) {
        add_str(retdict, "quickfixtextfunc", None)?;
    }
    Ok(())
}

/// The id of the window the location list belongs to, which only a location
/// list window has.
fn qf_getprop_filewinid(window: Option<Win>, id: QfId, retdict: &mut Dict) -> Result<(), KeyTaken> {
    let winid = window
        .filter(|&wp| wp.is_location_list_window())
        .and_then(|_| qf_find_win_with_loclist(id))
        .map_or(0, |ll_wp| ll_wp.handle);
    add_nr(retdict, "filewinid", VarNumber::from(winid))
}

/// The entries of the list, or of just entry `eidx`.
///
/// An empty list is a perfectly good answer, so neither the walk's refusal
/// nor the write's is passed on — upstream discarded both here too.
fn qf_getprop_items(qi: Qi, qf_idx: c_int, eidx: c_int, retdict: &mut Dict) {
    let mut l = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
    let _ = get_errorlist(qi, qf_idx, eidx, &mut l);
    let _ = add_list(retdict, "items", Some(l));
}

/// The arbitrary value `setqflist()` attached to the list, or the empty
/// string when it has none.
fn qf_getprop_ctx(qfl: &QfList, retdict: &mut Dict) -> Result<(), KeyTaken> {
    match &qfl.context {
        None => add_str(retdict, "context", None),
        Some(ctx) => add_tv(retdict, "context", ctx),
    }
}

/// The index of the current entry, or of the entry `eidx` names.
fn qf_getprop_idx(qfl: &QfList, mut eidx: c_int, retdict: &mut Dict) -> Result<(), KeyTaken> {
    if eidx == 0 {
        eidx = if qfl.is_empty() { 0 } else { qfl.index };
    }
    add_nr(retdict, "idx", VarNumber::from(eidx))
}

/// The list's `'quickfixtextfunc'` callback, or the empty string.
fn qf_getprop_qftf(qfl: &QfList, retdict: &mut Dict) -> Result<(), KeyTaken> {
    if !qfl.text_func.is_set() {
        return add_str(retdict, "quickfixtextfunc", None);
    }
    let mut tv = TV_INITIAL_VALUE;
    qfl.text_func.put(&mut tv);
    add_tv(retdict, "quickfixtextfunc", &tv)
}

/// `getqflist({what})` and `getloclist(nr, {what})`: fill `retdict` with the
/// properties `what` names.
pub(crate) fn qf_get_properties(
    window: Option<Win>,
    what: &Dict,
    retdict: &mut Dict,
) -> Result<(), QfError> {
    // A 'lines' key asks about lines the caller supplies, not about a
    // list at all.
    if let Some(lines) = what.find(b"lines") {
        return qf_get_list_from_lines(what, &lines.di_tv, retdict);
    }

    let qi = match window {
        Some(wp) => wp.location_list().map(QfId::stack),
        None => Some(Qi::global()),
    };

    let flags = qf_getprop_keys2flags(what, window.is_some());

    let named = qi
        .filter(|qi| !qi.is_empty())
        .and_then(|qi| Some((qi, qf_getprop_qfidx(qi, what)?)));
    let Some((qi, qf_idx)) = named else {
        // `?` here is the `From<KeyTaken>` conversion: the defaults can
        // only fail the way the dictionary layer fails.
        qf_getprop_defaults(qi, flags, window.is_some(), retdict)?;
        return Ok(());
    };

    // An 'idx' key asks about one entry rather than the whole list.
    let mut eidx = 0;
    if let Some(di) = what.find(b"idx") {
        if di.di_tv.v_type() != VAR_NUMBER {
            return Err(QfError::BadValue);
        }
        eidx = c_int::try_from(di.di_tv.number_or_zero()).unwrap_or(-1);
    }

    let wanted = |flag: GetListProps| flags.has(flag);
    let qfl = qi.slot(qf_idx);

    // As in `qf_getprop_defaults`: the first refused key stops the rest.
    if wanted(GetListProps::TITLE) {
        add_str(retdict, "title", qfl.title.as_deref())?;
    }
    if wanted(GetListProps::NR) {
        add_nr(retdict, "nr", VarNumber::from(qf_idx + 1))?;
    }
    if wanted(GetListProps::WINID) {
        add_nr(retdict, "winid", VarNumber::from(qf_winid(Some(qi))))?;
    }
    if wanted(GetListProps::ITEMS) {
        qf_getprop_items(qi, qf_idx, eidx, retdict);
    }
    if wanted(GetListProps::CONTEXT) {
        qf_getprop_ctx(&qfl, retdict)?;
    }
    if wanted(GetListProps::ID) {
        add_nr(retdict, "id", VarNumber::from(qfl.id))?;
    }
    if wanted(GetListProps::IDX) {
        qf_getprop_idx(&qfl, eidx, retdict)?;
    }
    if wanted(GetListProps::SIZE) {
        add_nr(retdict, "size", VarNumber::from(qfl.count()))?;
    }
    if wanted(GetListProps::TICK) {
        add_nr(retdict, "changedtick", VarNumber::from(qfl.changedtick))?;
    }
    if window.is_some() && wanted(GetListProps::FILEWINID) {
        qf_getprop_filewinid(window, qi.id(), retdict)?;
    }
    if wanted(GetListProps::QFBUFNR) {
        qf_getprop_qfbufnr(Some(qi), retdict)?;
    }
    if wanted(GetListProps::QFTF) {
        qf_getprop_qftf(&qfl, retdict)?;
    }
    Ok(())
}

/// The quickfix stack's `getqflist()` with no argument, or a window's
/// `getloclist()`: every entry of the current list, into `list`.
pub(crate) fn get_errorlist_of(window: Option<Win>, list: &mut List) -> Result<(), QfError> {
    let qi = match window {
        Some(wp) => wp.location_list().ok_or(QfError::NoSuchList)?.stack(),
        None => Qi::global(),
    };
    get_errorlist(qi, INVALID_QFIDX, 0, list)
}
