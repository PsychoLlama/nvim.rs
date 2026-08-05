//! The Vimscript face: `complete()`, `complete_info()`, `CompleteDone`.
//!
//! [`set_completion`] is `complete()`; [`ins_compl_add_tv`] turns one list
//! entry — a string or a dict with `word`/`abbr`/`menu`/`info`/`kind` — into
//! a match.  [`get_complete_info`] answers `complete_info()`, and
//! [`do_autocmd_completedone`] fires `CompleteDone` with
//! `v:completed_item`.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn do_autocmd_completedone(
    mut c: ::core::ffi::c_int,
    mut mode: ::core::ffi::c_int,
    mut word: *mut ::core::ffi::c_char,
) {
    unsafe {
        let mut save_v_event: save_v_event_T = save_v_event_T {
            sve_did_save: false,
            sve_hashtab: hashtab_T {
                ht_mask: 0,
                ht_used: 0,
                ht_filled: 0,
                ht_changed: 0,
                ht_locked: 0,
                ht_array: ::core::ptr::null_mut::<hashitem_T>(),
                ht_smallarray: [hashitem_T {
                    hi_hash: 0,
                    hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                }; 16],
            },
        };
        let mut v_event: *mut dict_T = get_v_event(&raw mut save_v_event);
        mode = mode & !CTRL_X_WANT_IDENT;
        let mut mode_str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !(*ctrl_x_mode_names.ptr())[mode as usize].is_null() {
            mode_str = (*ctrl_x_mode_names.ptr())[mode as usize];
        }
        tv_dict_add_str(
            v_event,
            b"complete_word\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
            if !word.is_null() {
                word as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        tv_dict_add_str(
            v_event,
            b"complete_type\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 14]>().wrapping_sub(1 as size_t),
            if !mode_str.is_null() {
                mode_str as *const ::core::ffi::c_char
            } else {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        tv_dict_add_str(
            v_event,
            b"reason\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 7]>().wrapping_sub(1 as size_t),
            if c == Ctrl_Y || !word.is_null() {
                b"accept\0".as_ptr() as *const ::core::ffi::c_char
            } else if c == Ctrl_E {
                b"cancel\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"discard\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        tv_dict_set_keys_readonly(v_event);
        ins_apply_autocmds(EVENT_COMPLETEDONE);
        restore_v_event(v_event, &raw mut save_v_event);
    }
}

pub(crate) unsafe extern "C" fn ins_compl_dict_alloc(mut match_0: *mut compl_T) -> *mut dict_T {
    unsafe {
        let mut dict: *mut dict_T = tv_dict_alloc_lock(VAR_FIXED);
        tv_dict_add_str(
            dict,
            b"word\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*match_0).cp_str.data,
        );
        tv_dict_add_str(
            dict,
            b"abbr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*match_0).cp_text[CPT_ABBR as usize] as *const ::core::ffi::c_char,
        );
        tv_dict_add_str(
            dict,
            b"menu\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*match_0).cp_text[CPT_MENU as usize] as *const ::core::ffi::c_char,
        );
        tv_dict_add_str(
            dict,
            b"kind\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*match_0).cp_text[CPT_KIND as usize] as *const ::core::ffi::c_char,
        );
        tv_dict_add_str(
            dict,
            b"info\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*match_0).cp_text[CPT_INFO as usize] as *const ::core::ffi::c_char,
        );
        if (*match_0).cp_user_data.v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_dict_add_str(
                dict,
                b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            tv_dict_add_tv(
                dict,
                b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                &raw mut (*match_0).cp_user_data,
            );
        }
        return dict;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_add_tv(
    tv: *mut typval_T,
    dir: Direction,
    mut fast: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut word: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut dup: bool = false_0 != 0;
        let mut empty: bool = false_0 != 0;
        let mut flags: ::core::ffi::c_int = if fast as ::core::ffi::c_int != 0 {
            CP_FAST
        } else {
            0 as ::core::ffi::c_int
        };
        let mut cptext: [*mut ::core::ffi::c_char; 4] =
            [::core::ptr::null_mut::<::core::ffi::c_char>(); 4];
        let mut user_abbr_hlname: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut user_kind_hlname: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut user_hl: [::core::ffi::c_int; 2] =
            [-1 as ::core::ffi::c_int, -1 as ::core::ffi::c_int];
        let mut user_data: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        user_data.v_type = VAR_UNKNOWN;
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*tv).vval.v_dict.is_null()
        {
            word = tv_dict_get_string(
                (*tv).vval.v_dict,
                b"word\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            cptext[CPT_ABBR as usize] = tv_dict_get_string(
                (*tv).vval.v_dict,
                b"abbr\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as *mut ::core::ffi::c_char;
            cptext[CPT_MENU as usize] = tv_dict_get_string(
                (*tv).vval.v_dict,
                b"menu\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as *mut ::core::ffi::c_char;
            cptext[CPT_KIND as usize] = tv_dict_get_string(
                (*tv).vval.v_dict,
                b"kind\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as *mut ::core::ffi::c_char;
            cptext[CPT_INFO as usize] = tv_dict_get_string(
                (*tv).vval.v_dict,
                b"info\0".as_ptr() as *const ::core::ffi::c_char,
                true_0 != 0,
            ) as *mut ::core::ffi::c_char;
            user_abbr_hlname = tv_dict_get_string(
                (*tv).vval.v_dict,
                b"abbr_hlgroup\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            user_hl[0 as ::core::ffi::c_int as usize] = get_user_highlight_attr(user_abbr_hlname);
            user_kind_hlname = tv_dict_get_string(
                (*tv).vval.v_dict,
                b"kind_hlgroup\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            );
            user_hl[1 as ::core::ffi::c_int as usize] = get_user_highlight_attr(user_kind_hlname);
            tv_dict_get_tv(
                (*tv).vval.v_dict,
                b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                &raw mut user_data,
            );
            if tv_dict_get_number(
                (*tv).vval.v_dict,
                b"icase\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0
            {
                flags |= CP_ICASE;
            }
            dup = tv_dict_get_number(
                (*tv).vval.v_dict,
                b"dup\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0;
            empty = tv_dict_get_number(
                (*tv).vval.v_dict,
                b"empty\0".as_ptr() as *const ::core::ffi::c_char,
            ) != 0;
            if !tv_dict_get_string(
                (*tv).vval.v_dict,
                b"equal\0".as_ptr() as *const ::core::ffi::c_char,
                false_0 != 0,
            )
            .is_null()
                && tv_dict_get_number(
                    (*tv).vval.v_dict,
                    b"equal\0".as_ptr() as *const ::core::ffi::c_char,
                ) != 0
            {
                flags |= CP_EQUAL;
            }
        } else {
            word = tv_get_string_chk(tv);
            memset(
                &raw mut cptext as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<[*mut ::core::ffi::c_char; 4]>(),
            );
        }
        if word.is_null() || !empty && *word as ::core::ffi::c_int == NUL {
            free_cptext(&raw mut cptext as *mut *mut ::core::ffi::c_char);
            tv_clear(&raw mut user_data);
            return FAIL;
        }
        let mut status: ::core::ffi::c_int = ins_compl_add(
            word as *mut ::core::ffi::c_char,
            -1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            &raw mut cptext as *mut *mut ::core::ffi::c_char,
            true_0 != 0,
            &raw mut user_data,
            dir,
            flags,
            dup,
            &raw mut user_hl as *mut ::core::ffi::c_int,
            FUZZY_SCORE_NONE,
        );
        if status != OK {
            tv_clear(&raw mut user_data);
        }
        return status;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_add_list(list: *mut list_T) {
    unsafe {
        let mut dir: Direction = compl_direction.get();
        let l_: *mut list_T = list;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                if ins_compl_add_tv(&raw mut (*li).li_tv, dir, true) == 1 as ::core::ffi::c_int {
                    dir = FORWARD;
                } else if did_emsg.get() != 0 {
                    break;
                }
                li = (*li).li_next;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn ins_compl_add_dict(mut dict: *mut dict_T) {
    unsafe {
        compl_opt_refresh_always.set(false_0 != 0);
        let mut di_refresh: *mut dictitem_T = tv_dict_find(
            dict,
            b"refresh\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di_refresh.is_null()
            && (*di_refresh).di_tv.v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut v: *const ::core::ffi::c_char = (*di_refresh).di_tv.vval.v_string;
            if !v.is_null()
                && strcmp(v, b"always\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
            {
                compl_opt_refresh_always.set(true_0 != 0);
            }
        }
        let mut di_words: *mut dictitem_T = tv_dict_find(
            dict,
            b"words\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize)
                as ptrdiff_t,
        );
        if !di_words.is_null()
            && (*di_words).di_tv.v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            ins_compl_add_list((*di_words).di_tv.vval.v_list);
        }
    }
}

pub(crate) unsafe extern "C" fn save_orig_extmarks() {
    unsafe {
        extmark_splice_delete(
            curbuf.get(),
            (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            compl_col.get(),
            (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
            compl_col.get() + compl_length.get() as colnr_T,
            compl_orig_extmarks.ptr(),
            true_0 != 0,
            kExtmarkUndo,
        );
    }
}

pub(crate) unsafe extern "C" fn restore_orig_extmarks() {
    unsafe {
        let mut i: ::core::ffi::c_long = ((*compl_orig_extmarks.ptr()).size as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int)
            as ::core::ffi::c_long;
        while i > -1 as ::core::ffi::c_long {
            let mut undo_info: ExtmarkUndoObject =
                *(*compl_orig_extmarks.ptr()).items.offset(i as isize);
            extmark_apply_undo(undo_info, true_0 != 0);
            i -= 1;
        }
    }
}

pub(crate) unsafe extern "C" fn set_completion(mut startcol: colnr_T, mut list: *mut list_T) {
    unsafe {
        let mut flags: ::core::ffi::c_int = CP_ORIGINAL_TEXT;
        let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
        let mut compl_longest: bool = cur_cot_flags
            & kOptCotFlagLongest as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint;
        let mut compl_no_insert: bool = cur_cot_flags
            & kOptCotFlagNoinsert as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint;
        let mut compl_no_select: bool = cur_cot_flags
            & kOptCotFlagNoselect as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint;
        if ctrl_x_mode_not_default() {
            ins_compl_prep(' ' as ::core::ffi::c_int);
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
        compl_length.set(((*curwin.get()).w_cursor.col - startcol) as ::core::ffi::c_int);
        compl_orig_text.set(cbuf_to_string(
            get_cursor_line_ptr().offset(compl_col.get() as isize),
            compl_length.get() as size_t,
        ));
        save_orig_extmarks();
        if p_ic.get() != 0 {
            flags |= CP_ICASE;
        }
        if ins_compl_add(
            (*compl_orig_text.ptr()).data,
            (*compl_orig_text.ptr()).size as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null::<*mut ::core::ffi::c_char>(),
            false_0 != 0,
            ::core::ptr::null_mut::<typval_T>(),
            kDirectionNotSet,
            flags | CP_FAST,
            false_0 != 0,
            ::core::ptr::null::<::core::ffi::c_int>(),
            FUZZY_SCORE_NONE,
        ) != OK
        {
            return;
        }
        ctrl_x_mode.set(CTRL_X_EVAL);
        ins_compl_add_list(list);
        compl_matches.set(ins_compl_make_cyclic());
        compl_started.set(true_0 != 0);
        compl_used_match.set(true_0 != 0);
        compl_cont_status.set(0 as ::core::ffi::c_int);
        let mut save_w_wrow: ::core::ffi::c_int = (*curwin.get()).w_wrow;
        let mut save_w_leftcol: ::core::ffi::c_int =
            (*curwin.get()).w_leftcol as ::core::ffi::c_int;
        compl_curr_match.set(compl_first_match.get());
        let mut no_select: bool =
            compl_no_select as ::core::ffi::c_int != 0 || compl_longest as ::core::ffi::c_int != 0;
        if compl_no_insert as ::core::ffi::c_int != 0 || no_select as ::core::ffi::c_int != 0 {
            ins_complete(K_DOWN, false_0 != 0);
            if no_select {
                ins_complete(K_UP, false_0 != 0);
            }
        } else {
            ins_complete(Ctrl_N, false_0 != 0);
        }
        compl_enter_selects.set(compl_no_insert);
        if !compl_interrupted.get() {
            show_pum(save_w_wrow, save_w_leftcol);
        }
        may_trigger_modechanged();
        ui_flush();
    }
}

pub unsafe extern "C" fn f_complete(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        if State.get() & MODE_INSERT == 0 as ::core::ffi::c_int {
            emsg(gettext(
                b"E785: complete() can only be used in Insert mode\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            return;
        }
        if !undo_allowed(curbuf.get()) {
            return;
        }
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        } else {
            let startcol: colnr_T = tv_get_number_chk(
                argvars.offset(0 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<bool>(),
            ) as colnr_T;
            if startcol > 0 as ::core::ffi::c_int {
                set_completion(
                    startcol - 1 as colnr_T,
                    (*argvars.offset(1 as ::core::ffi::c_int as isize))
                        .vval
                        .v_list,
                );
            }
        };
    }
}

pub unsafe extern "C" fn f_complete_add(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = ins_compl_add_tv(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            kDirectionNotSet,
            false_0 != 0,
        ) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_complete_check(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut saved: ::core::ffi::c_int = RedrawingDisabled.get();
        RedrawingDisabled.set(0 as ::core::ffi::c_int);
        ins_compl_check_keys(0 as ::core::ffi::c_int, true_0 != 0);
        (*rettv).vval.v_number = ins_compl_interrupted() as varnumber_T;
        RedrawingDisabled.set(saved);
    }
}

pub(crate) unsafe extern "C" fn fill_complete_info_dict(
    mut di: *mut dict_T,
    mut match_0: *mut compl_T,
    mut add_match: bool,
) {
    unsafe {
        tv_dict_add_str(
            di,
            b"word\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*match_0).cp_str.data,
        );
        tv_dict_add_str(
            di,
            b"abbr\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*match_0).cp_text[CPT_ABBR as usize] as *const ::core::ffi::c_char,
        );
        tv_dict_add_str(
            di,
            b"menu\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*match_0).cp_text[CPT_MENU as usize] as *const ::core::ffi::c_char,
        );
        tv_dict_add_str(
            di,
            b"kind\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*match_0).cp_text[CPT_KIND as usize] as *const ::core::ffi::c_char,
        );
        tv_dict_add_str(
            di,
            b"info\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
            (*match_0).cp_text[CPT_INFO as usize] as *const ::core::ffi::c_char,
        );
        if add_match {
            tv_dict_add_bool(
                di,
                b"match\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
                (*match_0).cp_in_match_array as BoolVarValue,
            );
        }
        if (*match_0).cp_user_data.v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_dict_add_str(
                di,
                b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                b"\0".as_ptr() as *const ::core::ffi::c_char,
            );
        } else {
            tv_dict_add_tv(
                di,
                b"user_data\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                &raw mut (*match_0).cp_user_data,
            );
        };
    }
}

pub(crate) unsafe extern "C" fn get_complete_info(
    mut what_list: *mut list_T,
    mut retdict: *mut dict_T,
) {
    unsafe {
        let mut what_flag: ::core::ffi::c_int = 0;
        if what_list.is_null() {
            what_flag = CI_WHAT_ALL & !(CI_WHAT_MATCHES | CI_WHAT_COMPLETED);
        } else {
            what_flag = 0 as ::core::ffi::c_int;
            let mut item: *mut listitem_T = tv_list_first(what_list);
            while !item.is_null() {
                let mut what: *const ::core::ffi::c_char = tv_get_string(&raw mut (*item).li_tv);
                if strcmp(what, b"mode\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    what_flag |= CI_WHAT_MODE;
                } else if strcmp(
                    what,
                    b"pum_visible\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    what_flag |= CI_WHAT_PUM_VISIBLE;
                } else if strcmp(what, b"items\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    what_flag |= CI_WHAT_ITEMS;
                } else if strcmp(what, b"selected\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    what_flag |= CI_WHAT_SELECTED;
                } else if strcmp(what, b"completed\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    what_flag |= CI_WHAT_COMPLETED;
                } else if strcmp(
                    what,
                    b"preinserted_text\0".as_ptr() as *const ::core::ffi::c_char,
                ) == 0 as ::core::ffi::c_int
                {
                    what_flag |= CI_WHAT_PREINSERTED_TEXT;
                } else if strcmp(what, b"matches\0".as_ptr() as *const ::core::ffi::c_char)
                    == 0 as ::core::ffi::c_int
                {
                    what_flag |= CI_WHAT_MATCHES;
                }
                item = (*item).li_next;
            }
        }
        let mut ret: ::core::ffi::c_int = OK;
        if what_flag & CI_WHAT_MODE != 0 {
            ret = tv_dict_add_str(
                retdict,
                b"mode\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
                ins_compl_mode(),
            );
        }
        if ret == OK && what_flag & CI_WHAT_PUM_VISIBLE != 0 {
            ret = tv_dict_add_nr(
                retdict,
                b"pum_visible\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
                pum_visible() as varnumber_T,
            );
        }
        if ret == OK && what_flag & CI_WHAT_PREINSERTED_TEXT != 0 {
            let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
            let mut len: ::core::ffi::c_int = compl_ins_end_col.get() as ::core::ffi::c_int
                - (*curwin.get()).w_cursor.col as ::core::ffi::c_int;
            ret = tv_dict_add_str_len(
                retdict,
                b"preinserted_text\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 17]>().wrapping_sub(1 as size_t),
                if len > 0 as ::core::ffi::c_int {
                    line.offset((*curwin.get()).w_cursor.col as isize) as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                },
                if len > 0 as ::core::ffi::c_int {
                    len
                } else {
                    0 as ::core::ffi::c_int
                },
            );
        }
        if ret == OK
            && what_flag & (CI_WHAT_ITEMS | CI_WHAT_SELECTED | CI_WHAT_MATCHES | CI_WHAT_COMPLETED)
                != 0
        {
            let mut li: *mut list_T = ::core::ptr::null_mut::<list_T>();
            let mut selected_idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut has_items: bool = what_flag & CI_WHAT_ITEMS != 0;
            let mut has_matches: bool = what_flag & CI_WHAT_MATCHES != 0;
            let mut has_completed: bool = what_flag & CI_WHAT_COMPLETED != 0;
            if has_items as ::core::ffi::c_int != 0 || has_matches as ::core::ffi::c_int != 0 {
                li = tv_list_alloc(kListLenMayKnow as ::core::ffi::c_int as ptrdiff_t);
                let mut key: *const ::core::ffi::c_char =
                    if has_matches as ::core::ffi::c_int != 0 && !has_items {
                        b"matches\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"items\0".as_ptr() as *const ::core::ffi::c_char
                    };
                ret = tv_dict_add_list(retdict, key, strlen(key), li);
            }
            if ret == OK && what_flag & CI_WHAT_SELECTED != 0 {
                if !(*compl_curr_match.ptr()).is_null()
                    && (*compl_curr_match.get()).cp_number == -1 as ::core::ffi::c_int
                {
                    ins_compl_update_sequence_numbers();
                }
            }
            if ret == OK && !(*compl_first_match.ptr()).is_null() {
                let mut list_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut match_0: *mut compl_T = compl_first_match.get();
                loop {
                    if !match_at_original_text(match_0) {
                        if has_items as ::core::ffi::c_int != 0
                            || has_matches as ::core::ffi::c_int != 0
                                && (*match_0).cp_in_match_array as ::core::ffi::c_int != 0
                        {
                            let mut di: *mut dict_T = tv_dict_alloc();
                            tv_list_append_dict(li, di);
                            fill_complete_info_dict(
                                di,
                                match_0,
                                has_matches as ::core::ffi::c_int != 0
                                    && has_items as ::core::ffi::c_int != 0,
                            );
                        }
                        if !(*compl_curr_match.ptr()).is_null()
                            && (*compl_curr_match.get()).cp_number == (*match_0).cp_number
                        {
                            selected_idx = list_idx;
                        }
                        if !has_matches || (*match_0).cp_in_match_array as ::core::ffi::c_int != 0 {
                            list_idx += 1;
                        }
                    }
                    match_0 = (*match_0).cp_next;
                    if !(!match_0.is_null() && !is_first_match(match_0)) {
                        break;
                    }
                }
            }
            if ret == OK && what_flag & CI_WHAT_SELECTED != 0 {
                ret = tv_dict_add_nr(
                    retdict,
                    b"selected\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                    selected_idx as varnumber_T,
                );
                let mut wp: *mut win_T = win_float_find_preview();
                if !wp.is_null() {
                    tv_dict_add_nr(
                        retdict,
                        b"preview_winid\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 14]>()
                            .wrapping_sub(1 as size_t),
                        (*wp).handle as varnumber_T,
                    );
                    tv_dict_add_nr(
                        retdict,
                        b"preview_bufnr\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 14]>()
                            .wrapping_sub(1 as size_t),
                        (*(*wp).w_buffer).handle as varnumber_T,
                    );
                }
            }
            if ret == OK
                && selected_idx != -1 as ::core::ffi::c_int
                && has_completed as ::core::ffi::c_int != 0
            {
                let mut di_0: *mut dict_T = tv_dict_alloc();
                fill_complete_info_dict(di_0, compl_curr_match.get(), false_0 != 0);
                ret = tv_dict_add_dict(
                    retdict,
                    b"completed\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 10]>().wrapping_sub(1 as size_t),
                    di_0,
                );
            }
        }
    }
}

pub unsafe extern "C" fn f_complete_info(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_dict_alloc_ret(rettv);
        let mut what_list: *mut list_T = ::core::ptr::null_mut::<list_T>();
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                emsg(gettext(&raw const e_listreq as *const ::core::ffi::c_char));
                return;
            }
            what_list = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                .vval
                .v_list;
        }
        get_complete_info(what_list, (*rettv).vval.v_dict);
    }
}
