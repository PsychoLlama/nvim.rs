//! The Vimscript face: `complete()`, `complete_info()`, `CompleteDone`.
//!
//! [`set_completion`] is `complete()`; [`ins_compl_add_tv`] turns one list
//! entry — a string or a dict with `word`/`abbr`/`menu`/`info`/`kind` — into
//! a match.  [`get_complete_info`] answers `complete_info()`, and
//! [`do_autocmd_completedone`] fires `CompleteDone` with
//! `v:completed_item`.
//!
//! Every `tv_dict_*` key here is an ordinary Rust `&str`: those functions copy
//! exactly the length they are given, so upstream's `S_LEN(key)` is
//! `key.as_ptr(), key.len()` and the transpile's `b"key\0"` plus
//! `size_of::<[c_char; N]>() - 1` goes away entirely.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::Allow;
use crate::keycodes::{Ctrl_E, Ctrl_N, Ctrl_Y};
use crate::types::{
    FAIL, NUL, OK, VAR_DICT, VAR_FIXED, VAR_LIST, VAR_STRING, VAR_UNKNOWN, kListLenMayKnow,
};

/// Fire `CompleteDone` with `v:event` describing how the completion ended.
pub(crate) unsafe fn do_autocmd_completedone(c: c_int, mode: c_int, word: *mut c_char) {
    unsafe {
        let mut save_v_event = SAVE_V_EVENT_INIT;
        let v_event = get_v_event(&raw mut save_v_event);
        let add_str = |key: &str, val: *const c_char| {
            tv_dict_add_str(v_event, key.as_ptr().cast(), key.len(), val)
        };

        let mode_str = match CTRL_X_MODE_NAMES[(mode & !CTRL_X_WANT_IDENT) as usize] {
            Some(name) => name.as_ptr(),
            None => c"".as_ptr(),
        };
        add_str(
            "complete_word",
            if word.is_null() { c"".as_ptr() } else { word },
        );
        add_str("complete_type", mode_str);
        add_str(
            "reason",
            if c == Ctrl_Y || !word.is_null() {
                c"accept".as_ptr()
            } else if c == Ctrl_E {
                c"cancel".as_ptr()
            } else {
                c"discard".as_ptr()
            },
        );
        tv_dict_set_keys_readonly(v_event);

        ins_apply_autocmds(EVENT_COMPLETEDONE);
        restore_v_event(v_event, &raw mut save_v_event);
    }
}

/// One match as a locked `v:completed_item` dict.
pub(crate) unsafe fn ins_compl_dict_alloc(match_0: *mut compl_T) -> *mut dict_T {
    unsafe {
        // { word, abbr, menu, kind, info, user_data } — the same keys and the
        // same order `complete_info()` fills in, minus its "match" flag.
        let dict = tv_dict_alloc_lock(VAR_FIXED);
        fill_complete_info_dict(dict, match_0, false);
        dict
    }
}

/// Add one match given as a Vimscript value: a string, or a dict with
/// `word`/`abbr`/`menu`/`kind`/`info` and the option keys.
///
/// `fast` uses `fast_breakcheck()` instead of `os_breakcheck()`. Answers
/// NOTDONE if the string is already in the list, OK if it was added, FAIL on
/// error.
pub(crate) unsafe fn ins_compl_add_tv(tv: *mut typval_T, dir: Direction, fast: bool) -> c_int {
    unsafe {
        let word: *const c_char;
        let mut dup = false;
        let mut empty = false;
        let mut flags = if fast { CP_FAST } else { 0 };
        let mut cptext: [*mut c_char; CPT_COUNT as usize] = [ptr::null_mut(); CPT_COUNT as usize];
        let mut user_hl: [c_int; 2] = [-1, -1];
        let mut user_data = TYPVAL_T_INIT;

        if (*tv).v_type == VAR_DICT && !(*tv).vval.v_dict.is_null() {
            let d = (*tv).vval.v_dict;
            // `save` copies the answer; the four cptext strings are owned by
            // the match from here on, the two highlight names are not.
            let get_str = |key: &CStr, save: bool| tv_dict_get_string(d, key.as_ptr(), save);
            let get_nr = |key: &CStr| tv_dict_get_number(d, key.as_ptr());

            word = get_str(c"word", false);
            cptext[CPT_ABBR as usize] = get_str(c"abbr", true);
            cptext[CPT_MENU as usize] = get_str(c"menu", true);
            cptext[CPT_KIND as usize] = get_str(c"kind", true);
            cptext[CPT_INFO as usize] = get_str(c"info", true);

            user_hl[0] = get_user_highlight_attr(get_str(c"abbr_hlgroup", false));
            user_hl[1] = get_user_highlight_attr(get_str(c"kind_hlgroup", false));

            tv_dict_get_tv(d, c"user_data".as_ptr(), &raw mut user_data);

            if get_nr(c"icase") != 0 {
                flags |= CP_ICASE;
            }
            dup = get_nr(c"dup") != 0;
            empty = get_nr(c"empty") != 0;
            if !get_str(c"equal", false).is_null() && get_nr(c"equal") != 0 {
                flags |= CP_EQUAL;
            }
        } else {
            word = tv_get_string_chk(tv);
        }

        if word.is_null() || (!empty && *word as c_int == NUL) {
            free_cptext(cptext.as_ptr());
            tv_clear(&raw mut user_data);
            return FAIL;
        }

        let status = ins_compl_add(
            word.cast_mut(),
            -1,
            ptr::null_mut(),
            cptext.as_ptr(),
            true,
            &raw mut user_data,
            dir,
            flags,
            dup,
            user_hl.as_ptr(),
            FUZZY_SCORE_NONE,
        );
        if status != OK {
            tv_clear(&raw mut user_data);
        }
        status
    }
}

/// Add every entry of `list` as a match.
pub(crate) unsafe fn ins_compl_add_list(list: *mut list_T) {
    unsafe {
        let mut dir = compl_direction.get();
        if list.is_null() {
            return;
        }
        let mut li = (*list).lv_first;
        while !li.is_null() {
            if ins_compl_add_tv(&raw mut (*li).li_tv, dir, true) == OK {
                // If dir was BACKWARD then honour it just once.
                dir = FORWARD;
            } else if did_emsg.get() != 0 {
                break;
            }
            li = (*li).li_next;
        }
    }
}

/// Add the matches a `'completefunc'`-style dict answers, and note its
/// optional `refresh` item.
pub(crate) unsafe fn ins_compl_add_dict(dict: *mut dict_T) {
    unsafe {
        let find = |key: &str| tv_dict_find(dict, key.as_ptr().cast(), key.len() as ptrdiff_t);

        // Check for the optional "refresh" item.
        compl_opt_refresh_always.set(false);
        let di_refresh = find("refresh");
        if !di_refresh.is_null() && (*di_refresh).di_tv.v_type == VAR_STRING {
            let v = (*di_refresh).di_tv.vval.v_string;
            if !v.is_null() && strcmp(v, c"always".as_ptr()) == 0 {
                compl_opt_refresh_always.set(true);
            }
        }

        // Add completions from a "words" list.
        let di_words = find("words");
        if !di_words.is_null() && (*di_words).di_tv.v_type == VAR_LIST {
            ins_compl_add_list((*di_words).di_tv.vval.v_list);
        }
    }
}

/// Save the extmarks over `compl_orig_text`, so they can be restored when the
/// completion is cancelled or the original text is completed.
pub(crate) unsafe fn save_orig_extmarks() {
    unsafe {
        let lnum = (*curwin.get()).w_cursor.lnum as c_int - 1;
        extmark_splice_delete(
            curbuf.get(),
            lnum,
            compl_col.get(),
            lnum,
            compl_col.get() + compl_length.get(),
            compl_orig_extmarks.ptr(),
            true,
            kExtmarkUndo,
        );
    }
}

/// Put the saved extmarks back, newest first.
pub(crate) unsafe fn restore_orig_extmarks() {
    unsafe {
        for i in (0..(*compl_orig_extmarks.ptr()).size as isize).rev() {
            let undo_info = *(*compl_orig_extmarks.ptr()).items.offset(i);
            extmark_apply_undo(undo_info, true);
        }
    }
}

/// Start the completion `complete()` describes: `startcol` is where the
/// matched text starts (1 is the first column) and `list` holds the matches.
pub(crate) unsafe fn set_completion(mut startcol: colnr_T, list: *mut list_T) {
    unsafe {
        let cur_cot_flags = get_cot_flags();
        let compl_longest = cur_cot_flags & kOptCotFlagLongest as c_uint != 0;
        let compl_no_insert = cur_cot_flags & kOptCotFlagNoinsert as c_uint != 0;
        let compl_no_select = cur_cot_flags & kOptCotFlagNoselect as c_uint != 0;

        // If already doing completions stop it.
        if ctrl_x_mode_not_default() {
            ins_compl_prep(' ' as c_int);
        }
        ins_compl_clear();
        ins_compl_free();
        compl_get_longest.set(compl_longest);

        compl_direction.set(FORWARD);
        if startcol > (*curwin.get()).w_cursor.col {
            startcol = (*curwin.get()).w_cursor.col;
        }
        compl_col.set(startcol);
        compl_lnum.set((*curwin.get()).w_cursor.lnum);
        compl_length.set((*curwin.get()).w_cursor.col - startcol);
        // compl_pattern doesn't need to be set.
        compl_orig_text.set(cbuf_to_string(
            get_cursor_line_ptr().offset(compl_col.get() as isize),
            compl_length.get() as size_t,
        ));
        save_orig_extmarks();

        let mut flags = CP_ORIGINAL_TEXT;
        if p_ic.get() != 0 {
            flags |= CP_ICASE;
        }
        if ins_compl_add(
            (*compl_orig_text.ptr()).data(),
            (*compl_orig_text.ptr()).len() as c_int,
            ptr::null_mut(),
            ptr::null(),
            false,
            ptr::null_mut(),
            kDirectionNotSet,
            flags | CP_FAST,
            false,
            ptr::null(),
            FUZZY_SCORE_NONE,
        ) != OK
        {
            return;
        }

        ctrl_x_mode.set(CTRL_X_EVAL);

        ins_compl_add_list(list);
        compl_matches.set(ins_compl_make_cyclic());
        compl_started.set(true);
        compl_used_match.set(true);
        compl_cont_status.set(0);
        let save_w_wrow = (*curwin.get()).w_wrow;
        let save_w_leftcol = (*curwin.get()).w_leftcol;

        compl_curr_match.set(compl_first_match.get());
        let no_select = compl_no_select || compl_longest;
        if compl_no_insert || no_select {
            ins_complete(K_DOWN, false);
            if no_select {
                ins_complete(K_UP, false);
            }
        } else {
            ins_complete(Ctrl_N, false);
        }
        compl_enter_selects.set(compl_no_insert);

        // Lazily show the popup menu, unless we got interrupted.
        if !compl_interrupted.get() {
            show_pum(save_w_wrow, save_w_leftcol);
        }

        may_trigger_modechanged();
        ui_flush();
    }
}

/// The `complete()` function; a `VimLFunc` row in the builtin table.
pub unsafe fn f_complete(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        if State.get() & MODE_INSERT == 0 {
            emsg(gettext(
                c"E785: complete() can only be used in Insert mode".as_ptr(),
            ));
            return;
        }

        // Check for undo allowed here, because if something was already
        // inserted the line was already saved for undo and this check isn't
        // done.
        if !undo_allowed(curbuf.get()) {
            return;
        }

        if (*argvars.offset(1)).v_type != VAR_LIST {
            emsg(gettext(&raw const e_invarg as *const c_char));
        } else {
            let startcol = tv_get_number_chk(argvars, ptr::null_mut()) as colnr_T;
            if startcol > 0 {
                set_completion(startcol - 1, (*argvars.offset(1)).vval.v_list);
            }
        }
    }
}

/// The `complete_add()` function; a `VimLFunc` row in the builtin table.
pub unsafe fn f_complete_add(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        (*rettv).vval.v_number = ins_compl_add_tv(argvars, kDirectionNotSet, false) as varnumber_T;
    }
}

/// The `complete_check()` function; a `VimLFunc` row in the builtin table.
pub unsafe fn f_complete_check(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        let _redraw = Allow::redraw();
        ins_compl_check_keys(0, true);
        (*rettv).vval.v_number = ins_compl_interrupted() as varnumber_T;
    }
}

/// Fill `di` with one match, as `complete_info()` reports it.
pub(crate) unsafe fn fill_complete_info_dict(
    di: *mut dict_T,
    match_0: *mut compl_T,
    add_match: bool,
) {
    unsafe {
        let add_str = |key: &str, val: *const c_char| {
            tv_dict_add_str(di, key.as_ptr().cast(), key.len(), val)
        };

        add_str("word", (*match_0).cp_str.data());
        add_str("abbr", (*match_0).cp_text[CPT_ABBR as usize]);
        add_str("menu", (*match_0).cp_text[CPT_MENU as usize]);
        add_str("kind", (*match_0).cp_text[CPT_KIND as usize]);
        add_str("info", (*match_0).cp_text[CPT_INFO as usize]);
        if add_match {
            tv_dict_add_bool(
                di,
                "match".as_ptr().cast(),
                "match".len(),
                (*match_0).cp_in_match_array as BoolVarValue,
            );
        }
        if (*match_0).cp_user_data.v_type == VAR_UNKNOWN {
            // Add an empty string for backwards compatibility.
            add_str("user_data", c"".as_ptr());
        } else {
            tv_dict_add_tv(
                di,
                "user_data".as_ptr().cast(),
                "user_data".len(),
                &raw mut (*match_0).cp_user_data,
            );
        }
    }
}

/// Fill `retdict` with whatever of `complete_info()` `what_list` asked for.
pub(crate) unsafe fn get_complete_info(what_list: *mut list_T, retdict: *mut dict_T) {
    unsafe {
        let add_nr = |key: &str, val: varnumber_T| {
            tv_dict_add_nr(retdict, key.as_ptr().cast(), key.len(), val)
        };

        let mut what_flag;
        if what_list.is_null() {
            what_flag = CI_WHAT_ALL & !(CI_WHAT_MATCHES | CI_WHAT_COMPLETED);
        } else {
            what_flag = 0;
            let mut item = tv_list_first(what_list);
            while !item.is_null() {
                // `tv_get_string` answers "" rather than NULL for anything it
                // cannot render, so this is never a null pointer.
                let what = CStr::from_ptr(tv_get_string(&raw mut (*item).li_tv));
                what_flag |= match what.to_bytes() {
                    b"mode" => CI_WHAT_MODE,
                    b"pum_visible" => CI_WHAT_PUM_VISIBLE,
                    b"items" => CI_WHAT_ITEMS,
                    b"selected" => CI_WHAT_SELECTED,
                    b"completed" => CI_WHAT_COMPLETED,
                    b"preinserted_text" => CI_WHAT_PREINSERTED_TEXT,
                    b"matches" => CI_WHAT_MATCHES,
                    _ => 0,
                };
                item = (*item).li_next;
            }
        }

        let mut ret = OK;
        if what_flag & CI_WHAT_MODE != 0 {
            ret = tv_dict_add_str(
                retdict,
                "mode".as_ptr().cast(),
                "mode".len(),
                ins_compl_mode(),
            );
        }

        if ret == OK && what_flag & CI_WHAT_PUM_VISIBLE != 0 {
            ret = add_nr("pum_visible", pum_visible() as varnumber_T);
        }

        if ret == OK && what_flag & CI_WHAT_PREINSERTED_TEXT != 0 {
            let line = get_cursor_line_ptr();
            let len = compl_ins_end_col.get() - (*curwin.get()).w_cursor.col;
            ret = tv_dict_add_str_len(
                retdict,
                "preinserted_text".as_ptr().cast(),
                "preinserted_text".len(),
                if len > 0 {
                    line.offset((*curwin.get()).w_cursor.col as isize)
                } else {
                    c"".as_ptr()
                },
                len.max(0),
            );
        }

        if ret != OK
            || what_flag & (CI_WHAT_ITEMS | CI_WHAT_SELECTED | CI_WHAT_MATCHES | CI_WHAT_COMPLETED)
                == 0
        {
            return;
        }

        let mut li: *mut list_T = ptr::null_mut();
        let mut selected_idx = -1;
        let has_items = what_flag & CI_WHAT_ITEMS != 0;
        let has_matches = what_flag & CI_WHAT_MATCHES != 0;
        let has_completed = what_flag & CI_WHAT_COMPLETED != 0;
        if has_items || has_matches {
            li = tv_list_alloc(kListLenMayKnow as ptrdiff_t);
            let key = if has_matches && !has_items {
                "matches"
            } else {
                "items"
            };
            ret = tv_dict_add_list(retdict, key.as_ptr().cast(), key.len(), li);
        }
        if ret == OK
            && what_flag & CI_WHAT_SELECTED != 0
            && !compl_curr_match.get().is_null()
            && (*compl_curr_match.get()).cp_number == -1
        {
            ins_compl_update_sequence_numbers();
        }
        if ret == OK && !compl_first_match.get().is_null() {
            let mut list_idx = 0;
            let mut match_0 = compl_first_match.get();
            loop {
                if !match_at_original_text(match_0) {
                    if has_items || (has_matches && (*match_0).cp_in_match_array) {
                        let di = tv_dict_alloc();
                        tv_list_append_dict(li, di);
                        fill_complete_info_dict(di, match_0, has_matches && has_items);
                    }
                    if !compl_curr_match.get().is_null()
                        && (*compl_curr_match.get()).cp_number == (*match_0).cp_number
                    {
                        selected_idx = list_idx;
                    }
                    if !has_matches || (*match_0).cp_in_match_array {
                        list_idx += 1;
                    }
                }
                match_0 = (*match_0).cp_next;
                if match_0.is_null() || is_first_match(match_0) {
                    break;
                }
            }
        }
        if ret == OK && what_flag & CI_WHAT_SELECTED != 0 {
            ret = add_nr("selected", selected_idx as varnumber_T);
            let wp = win_float_find_preview();
            if !wp.is_null() {
                add_nr("preview_winid", (*wp).handle as varnumber_T);
                add_nr("preview_bufnr", (*(*wp).w_buffer).handle as varnumber_T);
            }
        }
        if ret == OK && selected_idx != -1 && has_completed {
            let di = tv_dict_alloc();
            fill_complete_info_dict(di, compl_curr_match.get(), false);
            tv_dict_add_dict(retdict, "completed".as_ptr().cast(), "completed".len(), di);
        }
    }
}

/// The `complete_info()` function; a `VimLFunc` row in the builtin table.
pub unsafe fn f_complete_info(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    unsafe {
        tv_dict_alloc_ret(rettv);

        let mut what_list: *mut list_T = ptr::null_mut();
        if (*argvars).v_type != VAR_UNKNOWN {
            if (*argvars).v_type != VAR_LIST {
                emsg(gettext(&raw const e_listreq as *const c_char));
                return;
            }
            what_list = (*argvars).vval.v_list;
        }
        get_complete_info(what_list, (*rettv).vval.v_dict);
    }
}
