//! Writing a list from Vimscript.
//!
//! [`set_errorlist`] is `setqflist()`: a list of dictionaries goes through
//! [`qf_add_entries`] and [`qf_add_entry_from_dict`], and a `what`
//! dictionary through [`qf_set_properties`] and the `qf_setprop_*`
//! helpers.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::eval::typval::NumBuf;
use crate::semsg_c;
use crate::types::{VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VarLock};
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

/// The `what` entry under `key`, or null. `tv_dict_find` copies exactly the
/// length it is given, so a Rust `&str` is the key type.
///
/// # Safety
///
/// `what` must be null or a live dictionary.
unsafe fn find(what: *const dict_T, key: &str) -> *mut dictitem_T {
    // SAFETY: the caller's dictionary; the key is `key.len()` bytes long.
    unsafe { tv_dict_find(what, key.as_ptr().cast(), key.len() as ptrdiff_t) }
}

/// Set the list's `'quickfixtextfunc'` callback from `di`.
///
/// # Safety
///
/// `di` must be a live entry.
unsafe fn qf_setprop_qftf(mut qfl: Qfl, di: *mut dictitem_T) -> Result<(), QfError> {
    if check_secure() {
        return Err(QfError::Forbidden);
    }
    let mut cb = Callback {
        data: Callback_data {
            funcref: ptr::null_mut(),
        },
        type_0: kCallbackNone,
    };
    // SAFETY: the list's own callback slot, and the caller's entry.
    unsafe { callback_free(&raw mut qfl.qf_qftf_cb) };
    // A value that is not a callable leaves the list without one.
    if unsafe { callback_from_typval(&raw mut cb, &raw mut (*di).di_tv) } {
        qfl.qf_qftf_cb = cb;
    }
    Ok(())
}

/// Add one entry described by a `setqflist()` dictionary. `first_entry`
/// resets the "already complained about a bad buffer number" flag, so that
/// each call to `setqflist()` reports E92 once rather than once per entry.
/// `valid_entry` is set when the entry names a real position.
///
/// # Safety
///
/// `qfl` must be a live list and `d` a live dictionary.
unsafe fn qf_add_entry_from_dict(
    qfl: *mut qf_list_T,
    d: *mut dict_T,
    first_entry: bool,
    valid_entry: &mut bool,
) {
    let mut numbuf = NumBuf::new();
    static DID_BUFNR_EMSG: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: forwarded from the caller.
    if first_entry {
        DID_BUFNR_EMSG.set(false);
    }

    let filename = unsafe { tv_dict_get_string_alloc(d, c"filename".as_ptr()) };
    let module = unsafe { tv_dict_get_string_alloc(d, c"module".as_ptr()) };
    let mut bufnum = unsafe { tv_dict_get_number(d, c"bufnr".as_ptr()) } as c_int;
    let lnum = unsafe { tv_dict_get_number(d, c"lnum".as_ptr()) } as linenr_T;
    let end_lnum = unsafe { tv_dict_get_number(d, c"end_lnum".as_ptr()) } as linenr_T;
    let col = unsafe { tv_dict_get_number(d, c"col".as_ptr()) } as c_int;
    let end_col = unsafe { tv_dict_get_number(d, c"end_col".as_ptr()) } as c_int;
    // Not narrowed to a bool: `setqflist({'vcol': 5})` stores the 5 and
    // `getqflist()` reports it back.
    let vcol = unsafe { tv_dict_get_number(d, c"vcol".as_ptr()) } as c_char;
    let nr = unsafe { tv_dict_get_number(d, c"nr".as_ptr()) } as c_int;
    let kind = unsafe { numbuf.dict_string(d, c"type".as_ptr()) };
    let pattern = unsafe { tv_dict_get_string_alloc(d, c"pattern".as_ptr()) };
    let mut text = unsafe { tv_dict_get_string_alloc(d, c"text".as_ptr()) };
    if text.is_null() {
        text = unsafe { xcalloc(1, 1) }.cast();
    }
    let mut user_data = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    };
    unsafe { tv_dict_get_tv(d, c"user_data".as_ptr(), &raw mut user_data) };

    // An entry that names neither a file nor a position cannot be
    // jumped to.
    let mut valid = !(filename.is_null() && bufnum == 0 || lnum == 0 && pattern.is_null());

    if bufnum != 0 && find_buf(bufnum).is_none() {
        // Ignore the buffer number, and report it once per call.
        if !DID_BUFNR_EMSG.get() {
            DID_BUFNR_EMSG.set(true);
            unsafe { semsg_c!(gettext(c"E92: Buffer %d not found"), bufnum) };
        }
        valid = false;
        bufnum = 0;
    }

    // An explicit "valid" overrides all of that.
    if !unsafe { find(d, "valid") }.is_null() {
        valid = unsafe { tv_dict_get_bool(d, c"valid".as_ptr(), false as c_int) } != 0;
    }

    unsafe {
        qf_add_entry(
            qfl,
            &NewEntry {
                fname: filename,
                module,
                bufnum,
                lnum,
                end_lnum,
                col,
                end_col,
                vis_col: vcol,
                pattern,
                nr,
                kind: if kind.is_null() { 0 } else { *kind },
                user_data: &raw mut user_data,
                valid,
                ..NewEntry::new(text)
            },
        )
    };

    unsafe { xfree(filename.cast()) };
    unsafe { xfree(module.cast()) };
    unsafe { xfree(pattern.cast()) };
    unsafe { xfree(text.cast()) };
    unsafe { tv_clear(&raw mut user_data) };

    if valid {
        *valid_entry = true;
    }
}

/// Whether `entry` is a better match for the position the list was on than
/// `other_entry` is: the same file beats another file, then the nearer line,
/// then the nearer column. A target of zero at any level ends the
/// comparison, which is how `setqflist(…, 'u')` keeps the cursor put when
/// there is nothing to compare against.
///
/// C's `abs()` on an `int`, which is pure arithmetic: it reads no memory,
/// so the one region it needs is here rather than at each use.
fn abs_c(n: c_int) -> c_int {
    // SAFETY: arithmetic on a value; no pointer is involved.
    unsafe { abs(n) }
}

fn entry_is_closer_to_target(
    entry: Qfe,
    other_entry: Qfe,
    target_fnum: c_int,
    target_lnum: c_int,
    target_col: c_int,
) -> bool {
    if target_fnum == 0 {
        return false;
    }
    let is_target_file = entry.qf_fnum != 0 && entry.qf_fnum == target_fnum;
    let other_is_target_file = other_entry.qf_fnum != 0 && other_entry.qf_fnum == target_fnum;
    if is_target_file != other_is_target_file {
        return is_target_file;
    }

    if target_lnum == 0 {
        return false;
    }
    // An entry without a line number is infinitely far away.
    let distance = |qfp: Qfe| {
        if qfp.qf_lnum != 0 {
            abs_c(qfp.qf_lnum as c_int - target_lnum)
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
    let distance = |qfp: Qfe| {
        if qfp.qf_col != 0 {
            abs_c(qfp.qf_col - target_col)
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
/// refused, and every other decision here is unconditional. Upstream declared
/// it `int` and answered `OK` on every path, which made `setqflist()` look as
/// though adding entries could be rejected.
///
/// # Safety
///
/// `qi` must be a live stack, `list` null or a live list, and `title`
/// NUL-terminated.
unsafe fn qf_add_entries(
    qi: *mut qf_info_T,
    mut qf_idx: c_int,
    list: *mut list_T,
    title: *mut c_char,
    action: c_int,
) {
    // SAFETY: forwarded from the caller.
    let mut qfl = unsafe { qf_get_list(qi, qf_idx) };
    let mut old_last: *mut qfline_T = ptr::null_mut();

    // Where the list was, so that 'u' can find the nearest entry again.
    let (mut prev_fnum, mut prev_lnum, mut prev_col) = (0, 0, 0);
    if !unsafe { (*qfl).qf_ptr }.is_null() {
        prev_fnum = unsafe { (*(*qfl).qf_ptr).qf_fnum };
        prev_lnum = unsafe { (*(*qfl).qf_ptr).qf_lnum } as c_int;
        prev_col = unsafe { (*(*qfl).qf_ptr).qf_col };
    }

    let mut select_first_entry = false;
    let mut select_nearest_entry = false;
    if action == ' ' as c_int || qf_idx == unsafe { (*qi).qf_listcount } {
        // Make a new list.
        select_first_entry = true;
        unsafe { qf_new_list(qi, title) };
        qf_idx = unsafe { (*qi).qf_curlist };
        qfl = unsafe { qf_get_list(qi, qf_idx) };
    } else if action == 'a' as c_int {
        if unsafe { qf_list_empty(qfl) } {
            // Appending to an empty list is starting one.
            select_first_entry = true;
        } else {
            // Adding to an existing list, so use the last entry.
            old_last = unsafe { (*qfl).qf_last };
        }
    } else if action == 'r' as c_int {
        select_first_entry = true;
        unsafe { qf_free_items(qfl) };
        unsafe { qf_store_title(qfl, title) };
    } else if action == 'u' as c_int {
        select_nearest_entry = true;
        unsafe { qf_free_items(qfl) };
        unsafe { qf_store_title(qfl, title) };
    }

    let mut valid_entry = false;
    let mut entry_to_select: Option<Qfe> = None;
    let mut entry_to_select_index = 0;
    if !list.is_null() {
        let first = unsafe { tv_list_first(list) };
        let mut li = unsafe { (*list).lv_first };
        while !li.is_null() {
            if unsafe { (*li).li_tv.v_type } == VAR_DICT
                && !unsafe { (*li).li_tv.vval.v_dict }.is_null()
            {
                let d = unsafe { (*li).li_tv.vval.v_dict };
                unsafe { qf_add_entry_from_dict(qfl, d, ptr::eq(li, first), &mut valid_entry) };

                let entry = unsafe { Qfe::new((*qfl).qf_last) };
                let wanted = select_first_entry && entry_to_select.is_none()
                    || select_nearest_entry
                        && entry_to_select.is_none_or(|chosen| {
                            entry_is_closer_to_target(entry, chosen, prev_fnum, prev_lnum, prev_col)
                        });
                if wanted {
                    entry_to_select = Some(entry);
                    entry_to_select_index = unsafe { (*qfl).qf_count };
                }
            }
            li = unsafe { (*li).li_next };
        }
    }

    if valid_entry {
        unsafe { (*qfl).qf_nonevalid = false };
    } else if unsafe { (*qfl).qf_index } == 0 {
        unsafe { (*qfl).qf_nonevalid = true };
    }
    if let Some(entry_to_select) = entry_to_select {
        unsafe { (*qfl).qf_ptr = entry_to_select.raw() };
        unsafe { (*qfl).qf_index = entry_to_select_index };
    }

    // Don't update the cursor in quickfix window when appending entries.
    unsafe { qf_update_buffer(qi, old_last) };
}

/// Which list a `setqflist()` `what` names, through its `nr` or `id` key, or
/// `None` for one that is not on the stack — upstream's `INVALID_QFIDX`
/// sentinel.
///
/// `newlist` is both an input — whether a new list is being started — and an
/// output, since an `nr` one past the end asks for one.
///
/// # Safety
///
/// `qi` must be a live stack and `what` null or a live dictionary.
unsafe fn qf_setprop_get_qfidx(
    qi: *const qf_info_T,
    what: *const dict_T,
    action: c_int,
    newlist: &mut bool,
) -> Option<c_int> {
    // SAFETY: forwarded from the caller.
    let mut qf_idx = unsafe { (*qi).qf_curlist };

    let di = unsafe { find(what, "nr") };
    if !di.is_null() {
        if unsafe { (*di).di_tv.v_type } == VAR_NUMBER {
            // For zero use the current list.
            if unsafe { (*di).di_tv.vval.v_number } != 0 {
                qf_idx = unsafe { (*di).di_tv.vval.v_number } as c_int - 1;
            }
            if (action == ' ' as c_int || action == 'a' as c_int)
                && qf_idx == unsafe { (*qi).qf_listcount }
            {
                // Create a new list.
                *newlist = true;
                qf_idx = if unsafe { qf_stack_empty(qi) } {
                    0
                } else {
                    unsafe { (*qi).qf_listcount - 1 }
                };
            } else if qf_idx < 0 || qf_idx >= unsafe { (*qi).qf_listcount } {
                return None;
            } else if action != ' ' as c_int {
                *newlist = false;
            }
        } else if unsafe { (*di).di_tv.v_type } == VAR_STRING
            && unsafe { strequal((*di).di_tv.vval.v_string, c"$".as_ptr()) }
        {
            if !unsafe { qf_stack_empty(qi) } {
                qf_idx = unsafe { (*qi).qf_listcount } - 1;
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
    if !*newlist {
        let di = unsafe { find(what, "id") };
        if !di.is_null() {
            if unsafe { (*di).di_tv.v_type } != VAR_NUMBER {
                return None;
            }
            let by_id = unsafe { qf_id2nr(qi, (*di).di_tv.vval.v_number as c_uint) };
            return (by_id != INVALID_QFIDX).then_some(by_id);
        }
    }
    Some(qf_idx)
}

/// Set the list's title.
///
/// # Safety
///
/// `qi` must be a live stack, `what` and `di` live.
unsafe fn qf_setprop_title(
    qi: *mut qf_info_T,
    qf_idx: c_int,
    what: *const dict_T,
    di: *const dictitem_T,
) -> Result<(), QfError> {
    // SAFETY: forwarded from the caller.
    if unsafe { (*di).di_tv.v_type } != VAR_STRING {
        return Err(QfError::BadValue);
    }
    let qfl = unsafe { qf_get_list(qi, qf_idx) };
    unsafe { xfree((*qfl).qf_title.cast()) };
    unsafe { (*qfl).qf_title = tv_dict_get_string_alloc(what, c"title".as_ptr()) };
    if qf_idx == unsafe { (*qi).qf_curlist } {
        qf_update_win_titlevar(unsafe { Qi::new(qi) });
    }
    Ok(())
}

/// Replace the list's entries with the dictionaries in `di`.
///
/// # Safety
///
/// `qi` must be a live stack and `di` a live entry.
unsafe fn qf_setprop_items(
    qi: *mut qf_info_T,
    qf_idx: c_int,
    di: *mut dictitem_T,
    action: c_int,
) -> Result<(), QfError> {
    // SAFETY: forwarded from the caller.
    if unsafe { (*di).di_tv.v_type } != VAR_LIST {
        return Err(QfError::BadValue);
    }
    // The title survives the entries being replaced, so it has to be
    // copied out before `qf_add_entries` frees them.
    let title_save = unsafe { xstrdup((*qf_get_list(qi, qf_idx)).qf_title) };
    let action = if action == ' ' as c_int {
        'a' as c_int
    } else {
        action
    };
    unsafe { qf_add_entries(qi, qf_idx, (*di).di_tv.vval.v_list, title_save, action) };
    unsafe { xfree(title_save.cast()) };
    Ok(())
}

/// Replace the list's entries with the result of parsing the lines in `di`
/// with `'errorformat'` — or with the `what` dictionary's `efm`.
///
/// # Safety
///
/// `qi` must be a live stack, and `what` and `di` live.
unsafe fn qf_setprop_items_from_lines(
    qi: *mut qf_info_T,
    qf_idx: c_int,
    what: *const dict_T,
    di: *mut dictitem_T,
    action: c_int,
) -> Result<(), QfError> {
    // SAFETY: forwarded from the caller.
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
    if unsafe { (*di).di_tv.v_type } != VAR_LIST || unsafe { (*di).di_tv.vval.v_list }.is_null() {
        return Err(QfError::BadValue);
    }

    if action == 'r' as c_int || action == 'u' as c_int {
        unsafe { qf_free_items(qf_get_list(qi, qf_idx)) };
    }
    let parsed = unsafe {
        qf_init_ext(
            qi,
            qf_idx,
            ptr::null(),
            None,
            &raw mut (*di).di_tv,
            errorformat,
            false,
            0,
            0,
            ptr::null(),
            ptr::null_mut(),
        )
    } >= 0;
    parsed.then_some(()).ok_or(QfError::Unparsable)
}

/// Attach an arbitrary value to the list, which `getqflist({'context': 1})`
/// hands back.
///
/// Cannot fail: any value at all is a legal context. Upstream declared it
/// `int` and answered `OK`.
///
/// # Safety
///
/// `di` must be a live entry.
unsafe fn qf_setprop_context(mut qfl: Qfl, di: *mut dictitem_T) {
    // SAFETY: the list's own context slot, and the caller's entry.
    let ctx: *mut typval_T = unsafe {
        tv_free(qfl.qf_ctx);
        let ctx = xcalloc(1, size_of::<typval_T>()).cast();
        tv_copy(&raw mut (*di).di_tv, ctx);
        ctx
    };
    qfl.qf_ctx = ctx;
}

/// Move the list's cursor to entry `di`, or to the last entry for `"$"`.
///
/// # Safety
///
/// `di` must be a live entry.
unsafe fn qf_setprop_curidx(qi: Qi, mut qfl: Qfl, di: *const dictitem_T) -> Result<(), QfError> {
    // SAFETY: forwarded from the caller -- a live dictionary entry.
    let mut newidx = unsafe {
        if (*di).di_tv.v_type == VAR_STRING
            && !(*di).di_tv.vval.v_string.is_null()
            && strcmp((*di).di_tv.vval.v_string, c"$".as_ptr()) == 0
        {
            // Select the last entry in the list.
            qfl.qf_count
        } else {
            let mut not_a_number = false;
            let idx = tv_get_number_chk(&raw const (*di).di_tv, &raw mut not_a_number) as c_int;
            if not_a_number {
                return Err(QfError::BadValue);
            }
            idx
        }
    };

    if newidx < 1 {
        return Err(QfError::BadValue);
    }
    newidx = newidx.min(qfl.qf_count);

    let old_qfidx = qfl.qf_index;
    // SAFETY: a live list and an index inside it.
    let Some(qf_ptr) = get_nth_entry(qfl, newidx, &mut newidx) else {
        return Err(QfError::BadValue);
    };
    qfl.qf_ptr = qf_ptr.raw();
    qfl.qf_index = newidx;

    // Update the displayed quickfix list.
    // SAFETY: a live stack always has a current list.
    if unsafe { (*qf_get_curlist(qi.raw())).qf_id } == qfl.qf_id {
        qf_win_pos_update(qi, old_qfidx);
    }
    Ok(())
}

/// `setqflist(…, {what})`: apply each property `what` names.
///
/// # Safety
///
/// `qi` must be a live stack, `what` live and `title` NUL-terminated.
unsafe fn qf_set_properties(
    qi: *mut qf_info_T,
    what: *const dict_T,
    action: c_int,
    title: *mut c_char,
) -> Result<(), QfError> {
    // SAFETY: forwarded from the caller.
    let mut newlist = action == ' ' as c_int || unsafe { qf_stack_empty(qi) };
    let found = unsafe { qf_setprop_get_qfidx(qi, what, action, &mut newlist) };
    let Some(mut qf_idx) = found else {
        return Err(QfError::NoSuchList);
    };

    if newlist {
        unsafe { (*qi).qf_curlist = qf_idx };
        unsafe { qf_new_list(qi, title) };
        qf_idx = unsafe { (*qi).qf_curlist };
    }
    let qfl = unsafe { Qfl::new(qf_get_list(qi, qf_idx)) };

    // Each key that is present overwrites the answer, so what is
    // reported is the last one's result, not the worst. A `what` that
    // named none of them is `NothingToSet` — upstream's initial `FAIL`,
    // which is why `setqflist([], 'r', {})` answers -1.
    let mut retval = Err(QfError::NothingToSet);
    let di = unsafe { find(what, "title") };
    if !di.is_null() {
        retval = unsafe { qf_setprop_title(qi, qf_idx, what, di) };
    }
    let di = unsafe { find(what, "items") };
    if !di.is_null() {
        retval = unsafe { qf_setprop_items(qi, qf_idx, di, action) };
    }
    let di = unsafe { find(what, "lines") };
    if !di.is_null() {
        retval = unsafe { qf_setprop_items_from_lines(qi, qf_idx, what, di, action) };
    }
    let di = unsafe { find(what, "context") };
    if !di.is_null() {
        unsafe { qf_setprop_context(qfl, di) };
        retval = Ok(());
    }
    let di = unsafe { find(what, "idx") };
    if !di.is_null() {
        retval = unsafe { qf_setprop_curidx(Qi::new(qi), qfl, di) };
    }
    let di = unsafe { find(what, "quickfixtextfunc") };
    if !di.is_null() {
        retval = unsafe { qf_setprop_qftf(qfl, di) };
    }

    if newlist || retval.is_ok() {
        unsafe { qf_list_changed(qfl.raw()) };
    }
    if newlist {
        unsafe { qf_update_buffer(qi, ptr::null_mut()) };
    }
    retval
}

/// `setqflist()` and `setloclist()`. A null `wp` means the quickfix stack.
/// An `action` of `'f'` frees the whole stack; otherwise either `list` or
/// `what` says what to write, never both.
///
/// # Safety
///
/// `list`, `title` and `what` must be null or live.
pub unsafe fn set_errorlist(
    wp: Option<Win>,
    list: *mut list_T,
    action: c_int,
    title: *mut c_char,
    what: *mut dict_T,
) -> Result<(), QfError> {
    // SAFETY: forwarded from the caller.
    let qi = match wp {
        Some(wp) => ll_get_or_alloc_list(wp),
        None => QfStack::Global.raw(),
    };
    debug_assert!(!qi.is_null());

    if action == 'f' as c_int {
        // Free the entire quickfix or location list stack.
        qf_free_stack(wp, unsafe { Qi::new(qi) });
        return Ok(());
    }

    if !list.is_null() && unsafe { tv_list_len(list) } != 0 && !what.is_null() {
        unsafe {
            semsg_c!(
                gettext(e_invarg2),
                gettext(c"cannot have both a list and a \"what\" argument"),
            )
        };
        return Err(QfError::BadValue);
    }

    incr_quickfix_busy();
    let retval = if what.is_null() {
        unsafe { qf_add_entries(qi, (*qi).qf_curlist, list, title, action) };
        unsafe { qf_list_changed(qf_get_curlist(qi)) };
        Ok(())
    } else {
        unsafe { qf_set_properties(qi, what, action, title) }
    };
    unsafe { decr_quickfix_busy() };
    retval
}
