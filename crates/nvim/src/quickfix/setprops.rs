//! Writing a list from Vimscript.
//!
//! [`set_errorlist`] is `setqflist()`: a list of dictionaries goes through
//! [`qf_add_entries`] and [`qf_add_entry_from_dict`], and a `what`
//! dictionary through [`qf_set_properties`] and the `qf_setprop_*`
//! helpers.
//!
//! Adding an entry that names a file lists a buffer, which fires `BufNew`,
//! and the caller's list and dictionaries are Vimscript values an
//! autocommand can change. So the `what` dictionary is copied into a
//! [`What`] before anything is set, the entries are walked by index and
//! each one copied out of its dictionary before it is added, and the list
//! is held as a view.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::eval::list::{cstr_of, string_bytes};
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{NumBuf, dict_has_key, list_items};
use crate::memory::XString;
use crate::message_fmt::msg_bytes;
use crate::option::vars::P_EFM;
use crate::semsg;
use crate::types::{ListRef, VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING};
use core::ffi::{CStr, c_char, c_int, c_uint};

/// A reference of the caller's own to the list `tv` holds, if it holds one.
pub(crate) fn list_of(tv: &TypVal) -> Option<ListRef> {
    match tv {
        TypVal::List(list) => (**list).clone(),
        _ => None,
    }
}

/// `tv` as a string the caller owns.
fn string_of(tv: &TypVal) -> XString {
    let mut numbuf = NumBuf::new();
    XString::from_cstr(cstr_of(tv, &mut numbuf))
}

/// `d[key]` as a string the caller owns, or `None` for a missing key.
fn dict_string(d: &Dict, key: &[u8]) -> Option<XString> {
    d.find(key).map(|di| string_of(&di.di_tv))
}

/// `d[key]` as a number, wrapped to a C `int` the way upstream's cast does.
fn dict_int(d: &Dict, key: &[u8]) -> c_int {
    #[allow(clippy::cast_possible_truncation, reason = "C's `(int)` narrowing")]
    let n = dict_get_number(Some(d), key) as c_int;
    n
}

/// One entry described by a `setqflist()` dictionary, copied out of it.
struct DictEntry {
    filename: Option<XString>,
    module: Option<XString>,
    bufnum: c_int,
    lnum: LineNr,
    end_lnum: LineNr,
    col: c_int,
    end_col: c_int,
    vcol: c_char,
    nr: c_int,
    kind: c_char,
    pattern: Option<XString>,
    text: XString,
    user_data: TypVal,
    valid: Option<bool>,
}

impl DictEntry {
    fn of(d: &Dict) -> DictEntry {
        let kind = dict_string(d, b"type").map_or(0, |kind| kind.first().copied().unwrap_or(0));
        let mut user_data = TV_INITIAL_VALUE;
        let _ = dict_get_tv(Some(d), b"user_data", &mut user_data);
        DictEntry {
            filename: dict_string(d, b"filename"),
            module: dict_string(d, b"module"),
            bufnum: dict_int(d, b"bufnr"),
            lnum: dict_int(d, b"lnum"),
            end_lnum: dict_int(d, b"end_lnum"),
            col: dict_int(d, b"col"),
            end_col: dict_int(d, b"end_col"),
            // Not narrowed to a bool: `setqflist({'vcol': 5})` stores the 5
            // and `getqflist()` reports it back. The low byte, as C's
            // `(char)` keeps.
            vcol: c_char::from_le_bytes([dict_get_number(Some(d), b"vcol").to_le_bytes()[0]]),
            nr: dict_int(d, b"nr"),
            kind: kind.cast_signed(),
            pattern: dict_string(d, b"pattern"),
            text: dict_string(d, b"text").unwrap_or_default(),
            user_data,
            valid: dict_has_key(Some(d), b"valid")
                .then(|| dict_get_bool(Some(d), b"valid", c_int::from(false)) != 0),
        }
    }
}

/// Add one entry described by a `setqflist()` dictionary, already copied out
/// of it. `first_entry` resets the "already complained about a bad buffer
/// number" flag, so that each call to `setqflist()` reports E92 once rather
/// than once per entry. Answers whether the entry names a real position.
fn qf_add_entry_from_dict(qfl: Qfl, mut from: DictEntry, first_entry: bool) -> bool {
    static DID_BUFNR_EMSG: GlobalCell<bool> = GlobalCell::new(false);

    if first_entry {
        DID_BUFNR_EMSG.set(false);
    }

    // An entry that names neither a file nor a position cannot be
    // jumped to.
    let mut valid =
        !(from.filename.is_none() && from.bufnum == 0 || from.lnum == 0 && from.pattern.is_none());

    if from.bufnum != 0 && find_buf(from.bufnum).is_none() {
        // Ignore the buffer number, and report it once per call.
        if !DID_BUFNR_EMSG.get() {
            DID_BUFNR_EMSG.set(true);
            semsg!("E92: Buffer {} not found", from.bufnum);
        }
        valid = false;
        from.bufnum = 0;
    }

    // An explicit "valid" overrides all of that.
    if let Some(explicit) = from.valid {
        valid = explicit;
    }

    qf_add_entry(
        qfl,
        &NewEntry {
            fname: from.filename.as_ref().map(XString::as_cstr),
            module: from.module.as_ref().map(XString::as_cstr),
            bufnum: from.bufnum,
            lnum: from.lnum,
            end_lnum: from.end_lnum,
            col: from.col,
            end_col: from.end_col,
            vis_col: from.vcol,
            pattern: from.pattern.as_ref().map(XString::as_cstr),
            nr: from.nr,
            kind: from.kind,
            user_data: Some(&from.user_data),
            valid,
            ..NewEntry::new(from.text.as_cstr())
        },
    );
    valid
}

/// Whether `entry` is a better match for the position the list was on than
/// `other_entry` is: the same file beats another file, then the nearer line,
/// then the nearer column. A target of zero at any level ends the
/// comparison, which is how `setqflist(…, 'u')` keeps the cursor put when
/// there is nothing to compare against.
fn entry_is_closer_to_target(
    entry: &QfEntry,
    other_entry: &QfEntry,
    target_fnum: c_int,
    target_lnum: LineNr,
    target_col: c_int,
) -> bool {
    if target_fnum == 0 {
        return false;
    }
    let is_target_file = entry.fnum != 0 && entry.fnum == target_fnum;
    let other_is_target_file = other_entry.fnum != 0 && other_entry.fnum == target_fnum;
    if is_target_file != other_is_target_file {
        return is_target_file;
    }

    if target_lnum == 0 {
        return false;
    }
    // An entry without a line number is infinitely far away.
    let distance = |entry: &QfEntry| {
        if entry.lnum != 0 {
            (entry.lnum - target_lnum).abs()
        } else {
            INT_MAX
        }
    };
    let (line_distance, other_line_distance) = (distance(entry), distance(other_entry));
    if line_distance != other_line_distance {
        return line_distance < other_line_distance;
    }

    if target_col == 0 {
        return false;
    }
    let distance = |entry: &QfEntry| {
        if entry.col != 0 {
            (entry.col - target_col).abs()
        } else {
            INT_MAX
        }
    };
    let (column_distance, other_column_distance) = (distance(entry), distance(other_entry));
    column_distance < other_column_distance
}

/// Add every dictionary in `list` to list `qf_idx`, as `action` says: `' '`
/// starts a new list, `'a'` appends, `'r'` replaces the entries and `'u'`
/// replaces them while keeping the cursor on the nearest entry.
///
/// Cannot fail: a member of `list` that is not a dictionary is skipped, not
/// refused, and every other decision here is unconditional.
fn qf_add_entries(
    qi: Qi,
    mut qf_idx: c_int,
    list: Option<ListRef>,
    title: Option<&CStr>,
    action: u8,
) {
    let mut qfl = qi.slot(qf_idx);
    let mut old_last = None;

    // Where the list was, so that 'u' can find the nearest entry again.
    let (prev_fnum, prev_lnum, prev_col) = qfl
        .current()
        .map_or((0, 0, 0), |entry| (entry.fnum, entry.lnum, entry.col));

    let mut select_first_entry = false;
    let mut select_nearest_entry = false;
    if action == b' ' || qf_idx == qi.list_count {
        // Make a new list.
        select_first_entry = true;
        qf_new_list(qi, title);
        qf_idx = qi.current;
        qfl = qi.slot(qf_idx);
    } else if action == b'a' {
        if qfl.is_empty() {
            // Appending to an empty list is starting one.
            select_first_entry = true;
        } else {
            // Adding to an existing list, so use the last entry.
            old_last = Some(qfl.entries.len() - 1);
        }
    } else if action == b'r' || action == b'u' {
        select_first_entry = action == b'r';
        select_nearest_entry = action == b'u';
        qf_free_items(&mut qfl);
        qfl.title = title.map(XString::from_cstr);
    }

    let mut valid_entry = false;
    // The chosen entry, by position.
    let mut entry_to_select: Option<usize> = None;
    if let Some(list) = list {
        // An index, and the list asked again each time round: adding an
        // entry can run an autocommand, which can change the list. Each
        // entry is copied out of its dictionary before it is added, for the
        // same reason.
        let mut at = 0;
        while let Some(item) = list_items(Some(&list)).get(at) {
            let from = (item.li_tv.v_type() == VAR_DICT)
                .then(|| item.li_tv.dict_ref().map(DictEntry::of))
                .flatten();
            if let Some(from) = from {
                if qf_add_entry_from_dict(qfl, from, at == 0) {
                    valid_entry = true;
                }
                let added = qfl.entries.len() - 1;
                let wanted = select_first_entry && entry_to_select.is_none()
                    || select_nearest_entry
                        && entry_to_select.is_none_or(|chosen| {
                            entry_is_closer_to_target(
                                &qfl.entries[added],
                                &qfl.entries[chosen],
                                prev_fnum,
                                prev_lnum,
                                prev_col,
                            )
                        });
                if wanted {
                    entry_to_select = Some(added);
                }
            }
            at += 1;
        }
    }

    if valid_entry {
        qfl.no_valid = false;
    } else if qfl.index == 0 {
        qfl.no_valid = true;
    }
    if let Some(chosen) = entry_to_select.filter(|&chosen| chosen < qfl.entries.len()) {
        qfl.cursor = chosen;
        qfl.index = c_int::try_from(chosen + 1).unwrap_or(c_int::MAX);
    }

    // Don't update the cursor in quickfix window when appending entries.
    qf_update_buffer(qi, old_last);
}

/// What a `setqflist()` `what` dictionary holds under each key it knows,
/// copied out of it: setting a list runs autocommands, and those can change
/// the dictionary.
struct What {
    nr: Option<TypVal>,
    id: Option<TypVal>,
    title: Option<TypVal>,
    items: Option<TypVal>,
    lines: Option<TypVal>,
    efm: Option<TypVal>,
    context: Option<TypVal>,
    idx: Option<TypVal>,
    text_func: Option<TypVal>,
}

impl What {
    fn of(what: &Dict) -> What {
        let get = |key: &[u8]| what.find(key).map(|di| di.di_tv.clone());
        What {
            nr: get(b"nr"),
            id: get(b"id"),
            title: get(b"title"),
            items: get(b"items"),
            lines: get(b"lines"),
            efm: get(b"efm"),
            context: get(b"context"),
            idx: get(b"idx"),
            text_func: get(b"quickfixtextfunc"),
        }
    }
}

/// Which list a `setqflist()` `what` names, through its `nr` or `id` key, or
/// `None` for one that is not on the stack.
///
/// `newlist` is both an input — whether a new list is being started — and an
/// output, since an `nr` one past the end asks for one.
fn qf_setprop_get_qfidx(qi: Qi, what: &What, action: u8, newlist: &mut bool) -> Option<c_int> {
    let mut qf_idx = qi.current;

    if let Some(nr) = &what.nr {
        if nr.v_type() == VAR_NUMBER {
            // For zero use the current list.
            let nr = nr.number_or_zero();
            if nr != 0 {
                qf_idx = c_int::try_from(nr - 1).ok()?;
            }
            if (action == b' ' || action == b'a') && qf_idx == qi.list_count {
                // Create a new list.
                *newlist = true;
                qf_idx = if qi.is_empty() { 0 } else { qi.list_count - 1 };
            } else if qf_idx < 0 || qf_idx >= qi.list_count {
                return None;
            } else if action != b' ' {
                *newlist = false;
            }
        } else if nr.v_type() == VAR_STRING && string_bytes(nr) == b"$" {
            if !qi.is_empty() {
                qf_idx = qi.list_count - 1;
            } else if *newlist {
                qf_idx = 0;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    // An id names a list outright, but only when a new one is not being
    // started.
    if !*newlist && let Some(id) = &what.id {
        if id.v_type() != VAR_NUMBER {
            return None;
        }
        return c_uint::try_from(id.number_or_zero())
            .ok()
            .and_then(|id| qi.find_list(id));
    }
    Some(qf_idx)
}

/// Set the list's title.
fn qf_setprop_title(qi: Qi, qf_idx: c_int, title: &TypVal) -> Result<(), QfError> {
    if title.v_type() != VAR_STRING {
        return Err(QfError::BadValue);
    }
    qi.slot(qf_idx).title = Some(string_of(title));
    if qf_idx == qi.current {
        qf_update_win_titlevar(qi);
    }
    Ok(())
}

/// Replace the list's entries with the dictionaries in `items`.
fn qf_setprop_items(qi: Qi, qf_idx: c_int, items: &TypVal, action: u8) -> Result<(), QfError> {
    if items.v_type() != VAR_LIST {
        return Err(QfError::BadValue);
    }
    // The title survives the entries being replaced, so it is copied out
    // before `qf_add_entries` frees them.
    let title_save = qi.slot(qf_idx).title.clone();
    let action = if action == b' ' { b'a' } else { action };
    let list = list_of(items);
    qf_add_entries(
        qi,
        qf_idx,
        list,
        title_save.as_ref().map(XString::as_cstr),
        action,
    );
    Ok(())
}

/// Replace the list's entries with the result of parsing `lines` with
/// `'errorformat'` — or with the `what` dictionary's `efm`.
fn qf_setprop_items_from_lines(
    qi: Qi,
    qf_idx: c_int,
    what: &What,
    lines: &TypVal,
    action: u8,
) -> Result<(), QfError> {
    let errorformat = match &what.efm {
        None => P_EFM.get(),
        Some(efm) => {
            if efm.v_type() != VAR_STRING || efm.string_or_null().is_null() {
                return Err(QfError::BadValue);
            }
            XString::from_bytes(string_bytes(efm))
        }
    };

    // Only a List value is supported.
    if lines.v_type() != VAR_LIST || lines.list_ref().is_none() {
        return Err(QfError::BadValue);
    }

    if action == b'r' || action == b'u' {
        qf_free_items(&mut qi.slot(qf_idx));
    }
    let parsed = qf_init_ext(
        qi,
        qf_idx,
        Input::Value(lines),
        None,
        errorformat.as_cstr(),
        // Only reached with a value, which never reads the buffer's own.
        true,
        false,
        None,
        None,
    ) >= 0;
    parsed.then_some(()).ok_or(QfError::Unparsable)
}

/// Attach an arbitrary value to the list, which `getqflist({'context': 1})`
/// hands back.
fn qf_setprop_context(mut qfl: Qfl, context: &TypVal) {
    let mut ctx = Box::new(TypVal::Unknown);
    tv_copy(context, &mut ctx);
    let old = qfl.context.replace(ctx);
    drop(old);
}

/// Move the list's cursor to entry `idx`, or to the last entry for `"$"`.
fn qf_setprop_curidx(qi: Qi, mut qfl: Qfl, idx: &TypVal) -> Result<(), QfError> {
    let mut newidx = if idx.v_type() == VAR_STRING
        && !idx.string_or_null().is_null()
        && string_bytes(idx) == b"$"
    {
        // Select the last entry in the list.
        qfl.count()
    } else {
        let Ok(n) = tv_get_number_chk(idx) else {
            return Err(QfError::BadValue);
        };
        c_int::try_from(n).unwrap_or(if n < 0 { c_int::MIN } else { c_int::MAX })
    };

    if newidx < 1 {
        return Err(QfError::BadValue);
    }
    newidx = newidx.min(qfl.count());

    let old_qfidx = qfl.index;
    let Ok(at) = get_nth_entry(&qfl, newidx, &mut newidx) else {
        return Err(QfError::BadValue);
    };
    qfl.cursor = at;
    qfl.index = newidx;

    // Update the displayed quickfix list.
    if qi.current_list().id == qfl.id {
        qf_win_pos_update(qi, old_qfidx);
    }
    Ok(())
}

/// Set the list's `'quickfixtextfunc'` callback from `value`.
fn qf_setprop_qftf(mut qfl: Qfl, value: &TypVal) -> Result<(), QfError> {
    if check_secure() {
        return Err(QfError::Forbidden);
    }
    // Made before the list's is let go: making one can evaluate a name.
    let cb = Callback::from_typval(value);
    let mut old = core::mem::replace(&mut qfl.text_func, Callback::None);
    old.clear();
    // A value that is not a callable leaves the list without one.
    if let Some(cb) = cb {
        qfl.text_func = cb;
    }
    Ok(())
}

/// `setqflist(…, {what})`: apply each property `what` names.
fn qf_set_properties(mut qi: Qi, what: &What, action: u8, title: &CStr) -> Result<(), QfError> {
    let mut newlist = action == b' ' || qi.is_empty();
    let found = qf_setprop_get_qfidx(qi, what, action, &mut newlist);
    let Some(mut qf_idx) = found else {
        return Err(QfError::NoSuchList);
    };

    if newlist {
        qi.current = qf_idx;
        qf_new_list(qi, Some(title));
        qf_idx = qi.current;
    }
    let mut qfl = qi.slot(qf_idx);

    // Each key that is present overwrites the answer, so what is
    // reported is the last one's result, not the worst. A `what` that
    // named none of them is `NothingToSet` — upstream's initial `FAIL`,
    // which is why `setqflist([], 'r', {})` answers -1.
    let mut retval = Err(QfError::NothingToSet);
    if let Some(title) = &what.title {
        retval = qf_setprop_title(qi, qf_idx, title);
    }
    if let Some(items) = &what.items {
        retval = qf_setprop_items(qi, qf_idx, items, action);
    }
    if let Some(lines) = &what.lines {
        retval = qf_setprop_items_from_lines(qi, qf_idx, what, lines, action);
    }
    if let Some(context) = &what.context {
        qf_setprop_context(qfl, context);
        retval = Ok(());
    }
    if let Some(idx) = &what.idx {
        retval = qf_setprop_curidx(qi, qfl, idx);
    }
    if let Some(text_func) = &what.text_func {
        retval = qf_setprop_qftf(qfl, text_func);
    }

    if newlist || retval.is_ok() {
        qfl.changed();
    }
    if newlist {
        qf_update_buffer(qi, None);
    }
    retval
}

/// `setqflist()` and `setloclist()`. `None` means the quickfix stack. An
/// `action` of `'f'` frees the whole stack; otherwise either `list` or
/// `what` says what to write, never both.
pub fn set_errorlist(
    window: Option<Win>,
    list: Option<ListRef>,
    action: u8,
    title: &CStr,
    what: Option<&Dict>,
) -> Result<(), QfError> {
    let qi = match window {
        Some(wp) => wp.location_list_or_new(),
        None => Qi::global(),
    };

    if action == b'f' {
        // Free the entire quickfix or location list stack.
        qf_free_stack(window, qi);
        return Ok(());
    }

    if list.as_ref().is_some_and(|list| list_len(Some(list)) != 0) && what.is_some() {
        let arg0 =
            msg_bytes(gettext(c"cannot have both a list and a \"what\" argument").to_bytes());
        semsg!("E475: Invalid argument: {arg0}");
        return Err(QfError::BadValue);
    }

    // Copies, made before anything runs: the keys of `what`, the title. The
    // list is the caller's reference of its own.
    let what = what.map(What::of);
    let title = title.to_owned();

    let busy = QuickfixBusy::hold();
    let retval = match &what {
        None => {
            qf_add_entries(qi, qi.current, list, Some(&title), action);
            qi.current_slot().changed();
            Ok(())
        }
        Some(what) => qf_set_properties(qi, what, action, &title),
    };
    drop(busy);
    retval
}
