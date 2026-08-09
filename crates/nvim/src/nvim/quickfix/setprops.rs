//! Writing a list from Vimscript.
//!
//! [`set_errorlist`] is `setqflist()`: a list of dictionaries goes through
//! [`qf_add_entries`] and [`qf_add_entry_from_dict`], and a `what`
//! dictionary through [`qf_set_properties`] and the `qf_setprop_*`
//! helpers.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
#[allow(unused_imports)]
use crate::semsg_c;
use crate::src::nvim::types::{
    VAR_DICT, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED,
};
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
/// `qfl` must be a live list and `di` a live entry.
unsafe fn qf_setprop_qftf(qfl: *mut qf_list_T, di: *mut dictitem_T) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        if check_secure() {
            return FAIL;
        }
        callback_free(&raw mut (*qfl).qf_qftf_cb);
        let mut cb = Callback {
            data: Callback_data {
                funcref: ptr::null_mut(),
            },
            type_0: kCallbackNone,
        };
        // A value that is not a callable leaves the list without one.
        if callback_from_typval(&raw mut cb, &raw mut (*di).di_tv) {
            (*qfl).qf_qftf_cb = cb;
        }
        OK
    }
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
    static DID_BUFNR_EMSG: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: forwarded from the caller.
    unsafe {
        if first_entry {
            DID_BUFNR_EMSG.set(false);
        }

        let filename = tv_dict_get_string(d, c"filename".as_ptr(), true);
        let module = tv_dict_get_string(d, c"module".as_ptr(), true);
        let mut bufnum = tv_dict_get_number(d, c"bufnr".as_ptr()) as c_int;
        let lnum = tv_dict_get_number(d, c"lnum".as_ptr()) as linenr_T;
        let end_lnum = tv_dict_get_number(d, c"end_lnum".as_ptr()) as linenr_T;
        let col = tv_dict_get_number(d, c"col".as_ptr()) as c_int;
        let end_col = tv_dict_get_number(d, c"end_col".as_ptr()) as c_int;
        // Not narrowed to a bool: `setqflist({'vcol': 5})` stores the 5 and
        // `getqflist()` reports it back.
        let vcol = tv_dict_get_number(d, c"vcol".as_ptr()) as c_char;
        let nr = tv_dict_get_number(d, c"nr".as_ptr()) as c_int;
        let kind = tv_dict_get_string(d, c"type".as_ptr(), false);
        let pattern = tv_dict_get_string(d, c"pattern".as_ptr(), true);
        let mut text = tv_dict_get_string(d, c"text".as_ptr(), true);
        if text.is_null() {
            text = xcalloc(1, 1).cast();
        }
        let mut user_data = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        tv_dict_get_tv(d, c"user_data".as_ptr(), &raw mut user_data);

        // An entry that names neither a file nor a position cannot be
        // jumped to.
        let mut valid = !(filename.is_null() && bufnum == 0 || lnum == 0 && pattern.is_null());

        if bufnum != 0 && buflist_findnr(bufnum).is_null() {
            // Ignore the buffer number, and report it once per call.
            if !DID_BUFNR_EMSG.get() {
                DID_BUFNR_EMSG.set(true);
                semsg_c!(gettext(c"E92: Buffer %d not found".as_ptr()), bufnum);
            }
            valid = false;
            bufnum = 0;
        }

        // An explicit "valid" overrides all of that.
        if !find(d, "valid").is_null() {
            valid = tv_dict_get_bool(d, c"valid".as_ptr(), false as c_int) != 0;
        }

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
        );

        xfree(filename.cast());
        xfree(module.cast());
        xfree(pattern.cast());
        xfree(text.cast());
        tv_clear(&raw mut user_data);

        if valid {
            *valid_entry = true;
        }
    }
}

/// Whether `entry` is a better match for the position the list was on than
/// `other_entry` is: the same file beats another file, then the nearer line,
/// then the nearer column. A target of zero at any level ends the
/// comparison, which is how `setqflist(…, 'u')` keeps the cursor put when
/// there is nothing to compare against.
///
/// # Safety
///
/// Both entries must be live.
unsafe fn entry_is_closer_to_target(
    entry: *mut qfline_T,
    other_entry: *mut qfline_T,
    target_fnum: c_int,
    target_lnum: c_int,
    target_col: c_int,
) -> bool {
    // SAFETY: forwarded from the caller.
    unsafe {
        if target_fnum == 0 {
            return false;
        }
        let is_target_file = (*entry).qf_fnum != 0 && (*entry).qf_fnum == target_fnum;
        let other_is_target_file =
            (*other_entry).qf_fnum != 0 && (*other_entry).qf_fnum == target_fnum;
        if is_target_file != other_is_target_file {
            return is_target_file;
        }

        if target_lnum == 0 {
            return false;
        }
        // An entry without a line number is infinitely far away.
        let distance = |qfp: *mut qfline_T| {
            if (*qfp).qf_lnum != 0 {
                abs((*qfp).qf_lnum as c_int - target_lnum)
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
        let distance = |qfp: *mut qfline_T| {
            if (*qfp).qf_col != 0 {
                abs((*qfp).qf_col - target_col)
            } else {
                INT_MAX
            }
        };
        let (column_distance, other_column_distance) = (distance(entry), distance(other_entry));
        column_distance < other_column_distance
    }
}

/// Add every dictionary in `list` to list `qf_idx`, as `action` says: `' '`
/// starts a new list, `'a'` appends, `'r'` replaces the entries and `'u'`
/// replaces them while keeping the cursor on the nearest entry.
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
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut qfl = qf_get_list(qi, qf_idx);
        let mut old_last: *mut qfline_T = ptr::null_mut();

        // Where the list was, so that 'u' can find the nearest entry again.
        let (mut prev_fnum, mut prev_lnum, mut prev_col) = (0, 0, 0);
        if !(*qfl).qf_ptr.is_null() {
            prev_fnum = (*(*qfl).qf_ptr).qf_fnum;
            prev_lnum = (*(*qfl).qf_ptr).qf_lnum as c_int;
            prev_col = (*(*qfl).qf_ptr).qf_col;
        }

        let mut select_first_entry = false;
        let mut select_nearest_entry = false;
        if action == ' ' as c_int || qf_idx == (*qi).qf_listcount {
            // Make a new list.
            select_first_entry = true;
            qf_new_list(qi, title);
            qf_idx = (*qi).qf_curlist;
            qfl = qf_get_list(qi, qf_idx);
        } else if action == 'a' as c_int {
            if qf_list_empty(qfl) {
                // Appending to an empty list is starting one.
                select_first_entry = true;
            } else {
                // Adding to an existing list, so use the last entry.
                old_last = (*qfl).qf_last;
            }
        } else if action == 'r' as c_int {
            select_first_entry = true;
            qf_free_items(qfl);
            qf_store_title(qfl, title);
        } else if action == 'u' as c_int {
            select_nearest_entry = true;
            qf_free_items(qfl);
            qf_store_title(qfl, title);
        }

        let mut valid_entry = false;
        let mut entry_to_select: *mut qfline_T = ptr::null_mut();
        let mut entry_to_select_index = 0;
        if !list.is_null() {
            let first = tv_list_first(list);
            let mut li = (*list).lv_first;
            while !li.is_null() {
                if (*li).li_tv.v_type == VAR_DICT && !(*li).li_tv.vval.v_dict.is_null() {
                    let d = (*li).li_tv.vval.v_dict;
                    qf_add_entry_from_dict(qfl, d, ptr::eq(li, first), &mut valid_entry);

                    let entry = (*qfl).qf_last;
                    let wanted = select_first_entry && entry_to_select.is_null()
                        || select_nearest_entry
                            && (entry_to_select.is_null()
                                || entry_is_closer_to_target(
                                    entry,
                                    entry_to_select,
                                    prev_fnum,
                                    prev_lnum,
                                    prev_col,
                                ));
                    if wanted {
                        entry_to_select = entry;
                        entry_to_select_index = (*qfl).qf_count;
                    }
                }
                li = (*li).li_next;
            }
        }

        if valid_entry {
            (*qfl).qf_nonevalid = false;
        } else if (*qfl).qf_index == 0 {
            (*qfl).qf_nonevalid = true;
        }
        if !entry_to_select.is_null() {
            (*qfl).qf_ptr = entry_to_select;
            (*qfl).qf_index = entry_to_select_index;
        }

        // Don't update the cursor in quickfix window when appending entries.
        qf_update_buffer(qi, old_last);
        OK
    }
}

/// Which list a `setqflist()` `what` names, through its `nr` or `id` key.
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
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut qf_idx = (*qi).qf_curlist;

        let di = find(what, "nr");
        if !di.is_null() {
            if (*di).di_tv.v_type == VAR_NUMBER {
                // For zero use the current list.
                if (*di).di_tv.vval.v_number != 0 {
                    qf_idx = (*di).di_tv.vval.v_number as c_int - 1;
                }
                if (action == ' ' as c_int || action == 'a' as c_int)
                    && qf_idx == (*qi).qf_listcount
                {
                    // Create a new list.
                    *newlist = true;
                    qf_idx = if qf_stack_empty(qi) {
                        0
                    } else {
                        (*qi).qf_listcount - 1
                    };
                } else if qf_idx < 0 || qf_idx >= (*qi).qf_listcount {
                    return INVALID_QFIDX;
                } else if action != ' ' as c_int {
                    *newlist = false;
                }
            } else if (*di).di_tv.v_type == VAR_STRING
                && strequal((*di).di_tv.vval.v_string, c"$".as_ptr())
            {
                if !qf_stack_empty(qi) {
                    qf_idx = (*qi).qf_listcount - 1;
                } else if *newlist {
                    qf_idx = 0;
                } else {
                    return INVALID_QFIDX;
                }
            } else {
                return INVALID_QFIDX;
            }
        }

        // An id names a list outright, but only when a new one is not being
        // started.
        if !*newlist {
            let di = find(what, "id");
            if !di.is_null() {
                if (*di).di_tv.v_type != VAR_NUMBER {
                    return INVALID_QFIDX;
                }
                return qf_id2nr(qi, (*di).di_tv.vval.v_number as c_uint);
            }
        }
        qf_idx
    }
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
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        if (*di).di_tv.v_type != VAR_STRING {
            return FAIL;
        }
        let qfl = qf_get_list(qi, qf_idx);
        xfree((*qfl).qf_title.cast());
        (*qfl).qf_title = tv_dict_get_string(what, c"title".as_ptr(), true);
        if qf_idx == (*qi).qf_curlist {
            qf_update_win_titlevar(qi);
        }
        OK
    }
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
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        if (*di).di_tv.v_type != VAR_LIST {
            return FAIL;
        }
        // The title survives the entries being replaced, so it has to be
        // copied out before `qf_add_entries` frees them.
        let title_save = xstrdup((*qf_get_list(qi, qf_idx)).qf_title);
        let action = if action == ' ' as c_int {
            'a' as c_int
        } else {
            action
        };
        let retval = qf_add_entries(qi, qf_idx, (*di).di_tv.vval.v_list, title_save, action);
        xfree(title_save.cast());
        retval
    }
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
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut errorformat = p_efm.get();
        let efm_di = find(what, "efm");
        if !efm_di.is_null() {
            if (*efm_di).di_tv.v_type != VAR_STRING || (*efm_di).di_tv.vval.v_string.is_null() {
                return FAIL;
            }
            errorformat = (*efm_di).di_tv.vval.v_string;
        }

        // Only a List value is supported.
        if (*di).di_tv.v_type != VAR_LIST || (*di).di_tv.vval.v_list.is_null() {
            return FAIL;
        }

        if action == 'r' as c_int || action == 'u' as c_int {
            qf_free_items(qf_get_list(qi, qf_idx));
        }
        let parsed = qf_init_ext(
            qi,
            qf_idx,
            ptr::null(),
            ptr::null_mut(),
            &raw mut (*di).di_tv,
            errorformat,
            false,
            0,
            0,
            ptr::null(),
            ptr::null_mut(),
        ) >= 0;
        if parsed { OK } else { FAIL }
    }
}

/// Attach an arbitrary value to the list, which `getqflist({'context': 1})`
/// hands back.
///
/// # Safety
///
/// `qfl` must be a live list and `di` a live entry.
unsafe fn qf_setprop_context(qfl: *mut qf_list_T, di: *mut dictitem_T) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        tv_free((*qfl).qf_ctx);
        let ctx: *mut typval_T = xcalloc(1, size_of::<typval_T>()).cast();
        tv_copy(&raw mut (*di).di_tv, ctx);
        (*qfl).qf_ctx = ctx;
        OK
    }
}

/// Move the list's cursor to entry `di`, or to the last entry for `"$"`.
///
/// # Safety
///
/// `qi` must be a live stack, `qfl` a live list and `di` a live entry.
unsafe fn qf_setprop_curidx(
    qi: *mut qf_info_T,
    qfl: *mut qf_list_T,
    di: *const dictitem_T,
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut newidx = if (*di).di_tv.v_type == VAR_STRING
            && !(*di).di_tv.vval.v_string.is_null()
            && strcmp((*di).di_tv.vval.v_string, c"$".as_ptr()) == 0
        {
            // Select the last entry in the list.
            (*qfl).qf_count
        } else {
            let mut not_a_number = false;
            let idx = tv_get_number_chk(&raw const (*di).di_tv, &raw mut not_a_number) as c_int;
            if not_a_number {
                return FAIL;
            }
            idx
        };

        if newidx < 1 {
            return FAIL;
        }
        newidx = newidx.min((*qfl).qf_count);

        let old_qfidx = (*qfl).qf_index;
        let qf_ptr = get_nth_entry(qfl, newidx, &mut newidx);
        if qf_ptr.is_null() {
            return FAIL;
        }
        (*qfl).qf_ptr = qf_ptr;
        (*qfl).qf_index = newidx;

        // Update the displayed quickfix list.
        if (*qf_get_curlist(qi)).qf_id == (*qfl).qf_id {
            qf_win_pos_update(qi, old_qfidx);
        }
        OK
    }
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
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let mut newlist = action == ' ' as c_int || qf_stack_empty(qi);
        let mut qf_idx = qf_setprop_get_qfidx(qi, what, action, &mut newlist);
        if qf_idx == INVALID_QFIDX {
            return FAIL;
        }

        if newlist {
            (*qi).qf_curlist = qf_idx;
            qf_new_list(qi, title);
            qf_idx = (*qi).qf_curlist;
        }
        let qfl = qf_get_list(qi, qf_idx);

        // Each key that is present overwrites the answer, so what is
        // reported is the last one's result, not the worst.
        let mut retval = FAIL;
        let di = find(what, "title");
        if !di.is_null() {
            retval = qf_setprop_title(qi, qf_idx, what, di);
        }
        let di = find(what, "items");
        if !di.is_null() {
            retval = qf_setprop_items(qi, qf_idx, di, action);
        }
        let di = find(what, "lines");
        if !di.is_null() {
            retval = qf_setprop_items_from_lines(qi, qf_idx, what, di, action);
        }
        let di = find(what, "context");
        if !di.is_null() {
            retval = qf_setprop_context(qfl, di);
        }
        let di = find(what, "idx");
        if !di.is_null() {
            retval = qf_setprop_curidx(qi, qfl, di);
        }
        let di = find(what, "quickfixtextfunc");
        if !di.is_null() {
            retval = qf_setprop_qftf(qfl, di);
        }

        if newlist || retval == OK {
            qf_list_changed(qfl);
        }
        if newlist {
            qf_update_buffer(qi, ptr::null_mut());
        }
        retval
    }
}

/// `setqflist()` and `setloclist()`. A null `wp` means the quickfix stack.
/// An `action` of `'f'` frees the whole stack; otherwise either `list` or
/// `what` says what to write, never both.
///
/// # Safety
///
/// `wp` must be null or a live window; `list`, `title` and `what` null or
/// live.
pub unsafe fn set_errorlist(
    wp: *mut win_T,
    list: *mut list_T,
    action: c_int,
    title: *mut c_char,
    what: *mut dict_T,
) -> c_int {
    // SAFETY: forwarded from the caller.
    unsafe {
        let qi = if wp.is_null() {
            ql_info.get()
        } else {
            ll_get_or_alloc_list(wp)
        };
        debug_assert!(!qi.is_null());

        if action == 'f' as c_int {
            // Free the entire quickfix or location list stack.
            qf_free_stack(wp, qi);
            return OK;
        }

        if !list.is_null() && tv_list_len(list) != 0 && !what.is_null() {
            semsg_c!(
                gettext(&raw const e_invarg2 as *const c_char),
                gettext(c"cannot have both a list and a \"what\" argument".as_ptr()),
            );
            return FAIL;
        }

        incr_quickfix_busy();
        let retval = if what.is_null() {
            let retval = qf_add_entries(qi, (*qi).qf_curlist, list, title, action);
            if retval == OK {
                qf_list_changed(qf_get_curlist(qi));
            }
            retval
        } else {
            qf_set_properties(qi, what, action, title)
        };
        decr_quickfix_busy();
        retval
    }
}
