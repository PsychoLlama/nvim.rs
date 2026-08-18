//! The `match*()` Vimscript functions.
//!
//! `matchadd()`/`matchaddpos()`/`matchdelete()`/`clearmatches()` are thin
//! wrappers over the list operations in the parent; `getmatches()` and
//! `setmatches()` are the dictionary round trip that lets a match list be
//! saved and restored, including the `pos1`..`pos8` keys a position match
//! is described by.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{MB_MAXCHAR, VAR_DICT, VAR_LIST, VAR_UNKNOWN, kListLenMayKnow};

/// How many `posN` keys a saved position match can carry.
///
/// `matchaddpos()` itself takes any number of positions, but only the first
/// eight survive a `getmatches()`/`setmatches()` round trip — the reader
/// stops at `pos8`, so this is upstream's limit and not an arbitrary one.
const MAX_SAVED_POS: c_int = 8;

/// `tv_dict_add_str` with a Rust key: the dictionary copies exactly the
/// length it is given, so the transpile's `b"group\0"` plus
/// `size_of::<[c_char; 6]>() - 1` collapses to a `&str`.
///
/// # Safety
/// `d` must be live and `val` null or NUL-terminated.
unsafe fn put_str(d: *mut dict_T, key: &str, val: *const c_char) {
    // SAFETY: the caller's dictionary and value.
    unsafe {
        tv_dict_add_str(d, key.as_ptr().cast(), key.len(), val);
    }
}

/// `tv_dict_add_nr` with a Rust key; see [`put_str`].
///
/// # Safety
/// `d` must be live.
unsafe fn put_nr(d: *mut dict_T, key: &str, nr: varnumber_T) {
    // SAFETY: the caller's dictionary.
    unsafe {
        tv_dict_add_nr(d, key.as_ptr().cast(), key.len(), nr);
    }
}

/// `tv_dict_find` with a Rust key; null when absent.
///
/// # Safety
/// `d` must be null or live.
unsafe fn find(d: *const dict_T, key: &str) -> *mut dictitem_T {
    // SAFETY: the caller's dictionary.
    unsafe { tv_dict_find(d, key.as_ptr().cast(), key.len() as ptrdiff_t) }
}

/// Reads `matchadd()`'s and `matchaddpos()`' optional fifth argument, the
/// `{'conceal': c, 'window': w}` dictionary.
///
/// # Safety
/// `tv` must be live; `conceal_char` and `win` must be writable.
unsafe fn matchadd_dict_arg(
    tv: *mut typval_T,
    conceal_char: *mut *const c_char,
    win: *mut *mut win_T,
) -> c_int {
    // SAFETY: the caller's typval and out-parameters.
    unsafe {
        if (*tv).v_type != VAR_DICT {
            emsg(gettext(&raw const e_dictreq as *const c_char));
            return FAIL;
        }
        let dict = (*tv).vval.v_dict;

        let di = find(dict, "conceal");
        if !di.is_null() {
            *conceal_char = tv_get_string(&raw mut (*di).di_tv);
        }

        let di = find(dict, "window");
        if di.is_null() {
            return OK;
        }
        *win = find_win_by_nr_or_id(&raw mut (*di).di_tv);
        if (*win).is_null() {
            emsg(gettext(&raw const e_invalwindow as *const c_char));
            return FAIL;
        }
        OK
    }
}

/// `clearmatches([win])`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe fn f_clearmatches(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let win = get_optional_window(argvars, 0);
        if !win.is_null() {
            clear_matches(win);
        }
    }
}

/// `getmatches([win])`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe fn f_getmatches(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let win = get_optional_window(argvars, 0);
        let l = tv_list_alloc_ret(rettv, kListLenMayKnow as ptrdiff_t);
        if win.is_null() {
            return;
        }

        let mut cur = (*win).w_match_head;
        while !cur.is_null() {
            let dict = tv_dict_alloc();
            if (*cur).mit_match.regprog.is_null() {
                // Added with matchaddpos(): one `posN` key per position.
                for i in 0..(*cur).mit_pos_count {
                    let llpos = (*cur).mit_pos_array.offset(i as isize);
                    if (*llpos).lnum == 0 {
                        break;
                    }
                    // A column of zero means the whole line, and is reported
                    // as a one-element list.
                    let sub = tv_list_alloc(1 + if (*llpos).col > 0 { 2 } else { 0 });
                    tv_list_append_number(sub, (*llpos).lnum as varnumber_T);
                    if (*llpos).col > 0 {
                        tv_list_append_number(sub, (*llpos).col as varnumber_T);
                        tv_list_append_number(sub, (*llpos).len as varnumber_T);
                    }
                    let key = format!("pos{}", i + 1);
                    tv_dict_add_list(dict, key.as_ptr().cast(), key.len(), sub);
                }
            } else {
                put_str(dict, "pattern", (*cur).mit_pattern);
            }
            put_str(dict, "group", syn_id2name((*cur).mit_hlg_id));
            put_nr(dict, "priority", (*cur).mit_priority as varnumber_T);
            put_nr(dict, "id", (*cur).mit_id as varnumber_T);

            if (*cur).mit_conceal_char != 0 {
                let mut buf = [0 as c_char; MB_MAXCHAR + 1];
                let len = utf_char2bytes((*cur).mit_conceal_char, buf.as_mut_ptr());
                buf[len as usize] = 0;
                put_str(dict, "conceal", buf.as_ptr());
            }

            tv_list_append_dict(l, dict);
            cur = (*cur).mit_next;
        }
    }
}

/// `setmatches(list [, win])`.
///
/// Rebuilds a whole match list from `getmatches()`' answer. The list is
/// validated in full *before* anything is cleared, so a malformed entry
/// leaves the window's matches alone.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe fn f_setmatches(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let win = get_optional_window(argvars, 1);

        (*rettv).vval.v_number = -1;
        if (*argvars).v_type != VAR_LIST {
            emsg(gettext(&raw const e_listreq as *const c_char));
            return;
        }
        if win.is_null() {
            return;
        }
        let l = (*argvars).vval.v_list;

        // To some extent make sure this really came from getmatches().
        let mut li_idx = 0;
        let mut li = tv_list_first(l);
        while !li.is_null() {
            let tv = &raw mut (*li).li_tv;
            if (*tv).v_type != VAR_DICT || (*tv).vval.v_dict.is_null() {
                semsg_c!(
                    gettext(
                        c"E474: List item %d is either not a dictionary or an empty one".as_ptr(),
                    ),
                    li_idx,
                );
                return;
            }
            let d = (*tv).vval.v_dict;
            let ok = !find(d, "group").is_null()
                && (!find(d, "pattern").is_null() || !find(d, "pos1").is_null())
                && !find(d, "priority").is_null()
                && !find(d, "id").is_null();
            if !ok {
                semsg_c!(
                    gettext(c"E474: List item %d is missing one of the required keys".as_ptr()),
                    li_idx,
                );
                return;
            }
            li_idx += 1;
            li = (*li).li_next;
        }

        clear_matches(win);
        let mut match_add_failed = false;
        let mut li = tv_list_first(l);
        while !li.is_null() {
            let d = (*li).li_tv.vval.v_dict;

            // A match with no `pattern` is a position match: collect
            // pos1..pos8 into the list `match_add` wants.
            let mut positions: *mut list_T = ::core::ptr::null_mut();
            if find(d, "pattern").is_null() {
                positions = tv_list_alloc(MAX_SAVED_POS as ptrdiff_t + 1);
                for i in 1..MAX_SAVED_POS + 1 {
                    let key = format!("pos{i}");
                    let pos_di = tv_dict_find(d, key.as_ptr().cast(), key.len() as ptrdiff_t);
                    if pos_di.is_null() {
                        break;
                    }
                    if (*pos_di).di_tv.v_type != VAR_LIST {
                        // Leaks `positions` exactly as upstream does, and
                        // leaves the earlier entries of the list already
                        // restored — the validation above does not look
                        // inside a `posN` key.
                        return;
                    }
                    tv_list_append_tv(positions, &raw mut (*pos_di).di_tv);
                    tv_list_ref(positions);
                }
            }

            // Three number buffers are in play here — this one,
            // `tv_dict_get_string`'s and `tv_get_string`'s — and none may be
            // reused before its value is.
            let mut group_buf = [0 as c_char; NUMBUFLEN];
            let group = tv_dict_get_string_buf(d, c"group".as_ptr(), group_buf.as_mut_ptr());
            let priority = tv_dict_get_number(d, c"priority".as_ptr()) as c_int;
            let id = tv_dict_get_number(d, c"id".as_ptr()) as c_int;
            let conceal_di = find(d, "conceal");
            let conceal = if conceal_di.is_null() {
                ::core::ptr::null()
            } else {
                tv_get_string(&raw mut (*conceal_di).di_tv)
            };

            let added = if positions.is_null() {
                let pattern = tv_dict_get_string(d, c"pattern".as_ptr(), false);
                match_add(
                    win,
                    group,
                    pattern,
                    priority,
                    id,
                    ::core::ptr::null_mut(),
                    conceal,
                )
            } else {
                let rc = match_add(
                    win,
                    group,
                    ::core::ptr::null(),
                    priority,
                    id,
                    positions,
                    conceal,
                );
                tv_list_unref(positions);
                rc
            };
            if added != id {
                match_add_failed = true;
            }

            li = (*li).li_next;
        }
        if !match_add_failed {
            (*rettv).vval.v_number = 0;
        }
    }
}

/// The optional `priority`, `id` and dictionary arguments `matchadd()` and
/// `matchaddpos()` share, read from `argvars[2..=4]`.
///
/// Answers `None` once something has been diagnosed.
///
/// # Safety
/// The evaluator's argument slots.
unsafe fn optional_args(
    argvars: *mut typval_T,
) -> Option<(c_int, c_int, *const c_char, *mut win_T)> {
    // SAFETY: the evaluator's slots.
    unsafe {
        let mut prio = DEFAULT_PRIORITY;
        let mut id = -1;
        let mut conceal_char: *const c_char = ::core::ptr::null();
        let mut win = curwin.get();
        let mut error = false;

        // Nested, not sequential: an `id` is only read when a `priority` was
        // given, and the dictionary only when an `id` was.
        if (*argvars.offset(2)).v_type != VAR_UNKNOWN {
            prio = tv_get_number_chk(argvars.offset(2), &raw mut error) as c_int;
            if (*argvars.offset(3)).v_type != VAR_UNKNOWN {
                id = tv_get_number_chk(argvars.offset(3), &raw mut error) as c_int;
                if (*argvars.offset(4)).v_type != VAR_UNKNOWN
                    && matchadd_dict_arg(argvars.offset(4), &raw mut conceal_char, &raw mut win)
                        == FAIL
                {
                    return None;
                }
            }
        }
        if error {
            None
        } else {
            Some((prio, id, conceal_char, win))
        }
    }
}

/// `matchadd(group, pattern [, priority [, id [, options]]])`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe fn f_matchadd(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let mut grpbuf = [0 as c_char; NUMBUFLEN];
        let mut patbuf = [0 as c_char; NUMBUFLEN];
        let grp = tv_get_string_buf_chk(argvars, grpbuf.as_mut_ptr());
        let pat = tv_get_string_buf_chk(argvars.offset(1), patbuf.as_mut_ptr());

        (*rettv).vval.v_number = -1;
        if grp.is_null() || pat.is_null() {
            return;
        }
        let Some((prio, id, conceal_char, win)) = optional_args(argvars) else {
            return;
        };
        if (1..=3).contains(&id) {
            semsg_c!(
                gettext(c"E798: ID is reserved for \":match\": %d".as_ptr()),
                id,
            );
            return;
        }

        (*rettv).vval.v_number = match_add(
            win,
            grp,
            pat,
            prio,
            id,
            ::core::ptr::null_mut(),
            conceal_char,
        ) as varnumber_T;
    }
}

/// `matchaddpos(group, positions [, priority [, id [, options]]])`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe fn f_matchaddpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    unsafe {
        (*rettv).vval.v_number = -1;

        let mut buf = [0 as c_char; NUMBUFLEN];
        let group = tv_get_string_buf_chk(argvars, buf.as_mut_ptr());
        if group.is_null() {
            return;
        }
        if (*argvars.offset(1)).v_type != VAR_LIST {
            semsg_c!(
                gettext(&raw const e_listarg as *const c_char),
                c"matchaddpos()".as_ptr(),
            );
            return;
        }
        let l = (*argvars.offset(1)).vval.v_list;
        if tv_list_len(l) == 0 {
            return;
        }

        let Some((prio, id, conceal_char, win)) = optional_args(argvars) else {
            return;
        };
        // 3 is allowed: matchaddpos() is meant to stand in for `:3match`.
        if id == 1 || id == 2 {
            semsg_c!(
                gettext(c"E798: ID is reserved for \"match\": %d".as_ptr()),
                id,
            );
            return;
        }

        (*rettv).vval.v_number =
            match_add(win, group, ::core::ptr::null(), prio, id, l, conceal_char) as varnumber_T;
    }
}

/// `matcharg(id)` — the `[group, pattern]` of `:match`, `:2match` or
/// `:3match`, or `["", ""]` when that one is not set.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe fn f_matcharg(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let id = tv_get_number(argvars) as c_int;
        let is_excmd = (1..=3).contains(&id);
        // Any other id answers an empty list, not an error.
        let l = tv_list_alloc_ret(rettv, if is_excmd { 2 } else { 0 });
        if !is_excmd {
            return;
        }
        let m = get_match(curwin.get(), id);
        if m.is_null() {
            tv_list_append_string(l, ::core::ptr::null(), 0);
            tv_list_append_string(l, ::core::ptr::null(), 0);
        } else {
            tv_list_append_string(l, syn_id2name((*m).mit_hlg_id), -1);
            tv_list_append_string(l, (*m).mit_pattern, -1);
        }
    }
}

/// `matchdelete(id [, win])`.
///
/// # Safety
/// The evaluator's argument and return slots.
pub unsafe fn f_matchdelete(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's slots.
    unsafe {
        let win = get_optional_window(argvars, 1);
        (*rettv).vval.v_number = if win.is_null() {
            -1
        } else {
            match_delete(win, tv_get_number(argvars) as c_int, true) as varnumber_T
        };
    }
}
