//! The popup menu: turning the match list into `pumitem_T`s and showing it.
//!
//! [`ins_compl_build_pum`] is the whole of it — it filters the list by the
//! current leader, scores and sorts what survives, and fills
//! `compl_match_array`.  [`ins_compl_show_pum`] then hands that to
//! `pum_display`, and [`trigger_complete_changed_event`] fires
//! `CompleteChanged` with the selected item.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn ins_compl_col_range_attr(
    mut lnum: linenr_T,
    mut col: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let has_preinsert: bool = ins_compl_has_preinsert() as ::core::ffi::c_int != 0
            || ins_compl_preinsert_longest() as ::core::ffi::c_int != 0;
        let mut attr: ::core::ffi::c_int = 0;
        if cot_fuzzy() as ::core::ffi::c_int != 0
            || !compl_hi_on_autocompl_longest.get()
                && ins_compl_preinsert_longest() as ::core::ffi::c_int != 0
            || {
                attr = syn_name2attr(if has_preinsert as ::core::ffi::c_int != 0 {
                    b"PreInsert\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"ComplMatchIns\0".as_ptr() as *const ::core::ffi::c_char
                });
                attr == 0 as ::core::ffi::c_int
            }
        {
            return -1 as ::core::ffi::c_int;
        }
        let mut start_col: ::core::ffi::c_int =
            compl_col.get() as ::core::ffi::c_int + ins_compl_leader_len() as ::core::ffi::c_int;
        if !ins_compl_has_multiple() {
            return if col >= start_col && col < compl_ins_end_col.get() {
                attr
            } else {
                -1 as ::core::ffi::c_int
            };
        }
        if lnum == compl_lnum.get() && col >= start_col && col < MAXCOL as ::core::ffi::c_int
            || lnum > compl_lnum.get() && lnum < (*curwin.get()).w_cursor.lnum
            || lnum == (*curwin.get()).w_cursor.lnum && col <= compl_ins_end_col.get()
        {
            return attr;
        }
        return -1 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_del_pum() {
    unsafe {
        if (*compl_match_array.ptr()).is_null() {
            return;
        }
        pum_undisplay(false_0 != 0);
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            compl_match_array.ptr() as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
}

pub unsafe extern "C" fn pum_wanted() -> bool {
    unsafe {
        return get_cot_flags()
            & (kOptCotFlagMenu as ::core::ffi::c_int | kOptCotFlagMenuone as ::core::ffi::c_int)
                as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint
            || compl_autocomplete.get() as ::core::ffi::c_int != 0;
    }
}

pub(crate) unsafe extern "C" fn pum_enough_matches() -> bool {
    unsafe {
        let mut comp: *mut compl_T = compl_first_match.get();
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !(comp.is_null()
            || !match_at_original_text(comp) && {
                i += 1;
                i == 2 as ::core::ffi::c_int
            })
        {
            comp = (*comp).cp_next;
            if is_first_match(comp) {
                break;
            }
        }
        if get_cot_flags() & kOptCotFlagMenuone as ::core::ffi::c_int as ::core::ffi::c_uint != 0
            || compl_autocomplete.get() as ::core::ffi::c_int != 0
        {
            return i >= 1 as ::core::ffi::c_int;
        }
        return i >= 2 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn trigger_complete_changed_event(mut cur: ::core::ffi::c_int) {
    unsafe {
        static recursive: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
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
        if recursive.get() {
            return;
        }
        let mut item: *mut dict_T = if cur < 0 as ::core::ffi::c_int {
            tv_dict_alloc()
        } else {
            ins_compl_dict_alloc(compl_curr_match.get())
        };
        let mut v_event: *mut dict_T = get_v_event(&raw mut save_v_event);
        tv_dict_add_dict(
            v_event,
            b"completed_item\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 15]>().wrapping_sub(1 as size_t),
            item,
        );
        pum_set_event_info(v_event);
        tv_dict_set_keys_readonly(v_event);
        recursive.set(true_0 != 0);
        (*textlock.ptr()) += 1;
        apply_autocmds(
            EVENT_COMPLETECHANGED,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            false_0 != 0,
            curbuf.get(),
        );
        (*textlock.ptr()) -= 1;
        recursive.set(false_0 != 0);
        restore_v_event(v_event, &raw mut save_v_event);
    }
}

pub(crate) unsafe extern "C" fn prepend_startcol_text(
    mut dest: *mut String_0,
    mut src: *mut String_0,
    mut startcol: ::core::ffi::c_int,
) {
    unsafe {
        let mut prepend_len: ::core::ffi::c_int = compl_col.get() as ::core::ffi::c_int - startcol;
        let mut new_length: ::core::ffi::c_int = prepend_len + (*src).size as ::core::ffi::c_int;
        (*dest).size = new_length as size_t;
        (*dest).data =
            xmalloc((new_length as size_t).wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        let mut line: *mut ::core::ffi::c_char = ml_get((*curwin.get()).w_cursor.lnum);
        memmove(
            (*dest).data as *mut ::core::ffi::c_void,
            line.offset(startcol as isize) as *const ::core::ffi::c_void,
            prepend_len as size_t,
        );
        memmove(
            (*dest).data.offset(prepend_len as isize) as *mut ::core::ffi::c_void,
            (*src).data as *const ::core::ffi::c_void,
            (*src).size,
        );
        *(*dest).data.offset(new_length as isize) = NUL as ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn get_leader_for_startcol(
    mut match_0: *mut compl_T,
    mut cached: bool,
) -> *mut String_0 {
    unsafe {
        let mut cpt_idx: ::core::ffi::c_int = 0;
        let mut startcol: ::core::ffi::c_int = 0;
        static adjusted_leader: GlobalCell<String_0> = GlobalCell::new(String_0 {
            data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            size: 0 as size_t,
        });
        if match_0.is_null() {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*adjusted_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            (*adjusted_leader.ptr()).size = 0 as size_t;
            return ::core::ptr::null_mut::<String_0>();
        }
        if !(*cpt_sources_array.ptr()).is_null() {
            cpt_idx = (*match_0).cp_cpt_source_idx;
            if cpt_idx >= 0 as ::core::ffi::c_int {
                startcol = (*(*cpt_sources_array.ptr()).offset(cpt_idx as isize)).cs_startcol;
                if (*compl_leader.ptr()).data.is_null() {
                    if startcol < 0 as ::core::ffi::c_int || startcol >= compl_col.get() {
                        return compl_orig_text.ptr();
                    }
                    return compl_leader.ptr();
                }
                if compl_col.get() > 0 as ::core::ffi::c_int {
                    if startcol >= 0 as ::core::ffi::c_int && startcol < compl_col.get() {
                        let mut prepend_len: ::core::ffi::c_int =
                            compl_col.get() as ::core::ffi::c_int - startcol;
                        let mut new_length: ::core::ffi::c_int =
                            prepend_len + (*compl_leader.ptr()).size as ::core::ffi::c_int;
                        if cached as ::core::ffi::c_int != 0
                            && new_length as size_t == (*adjusted_leader.ptr()).size
                            && !(*adjusted_leader.ptr()).data.is_null()
                        {
                            return adjusted_leader.ptr();
                        }
                        let mut ptr__0: *mut *mut ::core::ffi::c_void =
                            &raw mut (*adjusted_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr__0);
                        *ptr__0 = NULL;
                        let _ = *ptr__0;
                        (*adjusted_leader.ptr()).size = 0 as size_t;
                        prepend_startcol_text(adjusted_leader.ptr(), compl_leader.ptr(), startcol);
                        return adjusted_leader.ptr();
                    }
                }
            }
        }
        return compl_leader.ptr();
    }
}

pub(crate) unsafe extern "C" fn ins_compl_build_pum() -> ::core::ffi::c_int {
    unsafe {
        compl_match_arraysize.set(0 as ::core::ffi::c_int);
        if ins_compl_need_restart() {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                compl_leader.ptr() as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
        let mut compl_no_select: bool = get_cot_flags()
            & kOptCotFlagNoselect as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint
            || compl_autocomplete.get() as ::core::ffi::c_int != 0 && !ins_compl_has_preinsert();
        let mut match_head: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
        let mut match_tail: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
        let mut match_count: *mut ::core::ffi::c_int =
            ::core::ptr::null_mut::<::core::ffi::c_int>();
        let mut is_forward: bool = compl_shows_dir_forward();
        let mut is_cpt_completion: bool = !(*cpt_sources_array.ptr()).is_null();
        let mut shown_match_ok: bool = match_at_original_text(compl_shown_match.get());
        if strequal((*compl_leader.ptr()).data, (*compl_orig_text.ptr()).data) as ::core::ffi::c_int
            != 0
            && !shown_match_ok
        {
            compl_shown_match.set(if compl_no_select as ::core::ffi::c_int != 0 {
                compl_first_match.get()
            } else {
                (*compl_first_match.get()).cp_next
            });
        }
        let mut did_find_shown_match: bool = false_0 != 0;
        let mut comp: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
        let mut shown_compl: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut cur: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if is_cpt_completion {
            match_count = xcalloc(
                cpt_sources_count.get() as size_t,
                ::core::mem::size_of::<::core::ffi::c_int>(),
            ) as *mut ::core::ffi::c_int;
        }
        get_leader_for_startcol(::core::ptr::null_mut::<compl_T>(), true_0 != 0);
        comp = compl_first_match.get();
        loop {
            (*comp).cp_in_match_array = false_0 != 0;
            let mut leader: *mut String_0 = get_leader_for_startcol(comp, true_0 != 0);
            if ctrl_x_mode_normal() as ::core::ffi::c_int != 0
                && p_inf.get() == 0
                && !(*leader).data.is_null()
                && ignorecase((*leader).data) == 0
                && !cot_fuzzy()
            {
                (*comp).cp_flags &= !(CP_ICASE);
            }
            if !match_at_original_text(comp)
                && ((*leader).data.is_null()
                    || ins_compl_equal(comp, (*leader).data, (*leader).size) as ::core::ffi::c_int
                        != 0
                    || cot_fuzzy() as ::core::ffi::c_int != 0
                        && (*comp).cp_score != FUZZY_SCORE_NONE)
            {
                let mut match_limit_exceeded: bool = false_0 != 0;
                let mut cur_source: ::core::ffi::c_int = (*comp).cp_cpt_source_idx;
                if is_forward as ::core::ffi::c_int != 0
                    && cur_source != -1 as ::core::ffi::c_int
                    && is_cpt_completion as ::core::ffi::c_int != 0
                {
                    *match_count.offset(cur_source as isize) += 1;
                    let mut max_matches: ::core::ffi::c_int =
                        (*(*cpt_sources_array.ptr()).offset(cur_source as isize)).cs_max_matches;
                    if max_matches > 0 as ::core::ffi::c_int
                        && *match_count.offset(cur_source as isize) > max_matches
                    {
                        match_limit_exceeded = true_0 != 0;
                    }
                }
                if !match_limit_exceeded {
                    (*compl_match_arraysize.ptr()) += 1;
                    (*comp).cp_in_match_array = true_0 != 0;
                    if match_head.is_null() {
                        match_head = comp;
                    } else {
                        (*match_tail).cp_match_next = comp;
                    }
                    match_tail = comp;
                    if !shown_match_ok && !cot_fuzzy() {
                        if comp == compl_shown_match.get()
                            || did_find_shown_match as ::core::ffi::c_int != 0
                        {
                            compl_shown_match.set(comp);
                            did_find_shown_match = true_0 != 0;
                            shown_match_ok = true_0 != 0;
                        } else {
                            shown_compl = comp;
                        }
                        cur = i;
                    } else if cot_fuzzy() {
                        if i == 0 as ::core::ffi::c_int {
                            shown_compl = comp;
                        }
                        if !shown_match_ok && comp == compl_shown_match.get() {
                            cur = i;
                            shown_match_ok = true_0 != 0;
                        }
                    }
                    i += 1;
                }
            }
            if comp == compl_shown_match.get() && !cot_fuzzy() {
                did_find_shown_match = true_0 != 0;
                if match_at_original_text(comp) {
                    shown_match_ok = true_0 != 0;
                }
                if !shown_match_ok && !shown_compl.is_null() {
                    compl_shown_match.set(shown_compl);
                    shown_match_ok = true_0 != 0;
                }
            }
            comp = (*comp).cp_next;
            if !(!comp.is_null() && !is_first_match(comp)) {
                break;
            }
        }
        xfree(match_count as *mut ::core::ffi::c_void);
        if compl_match_arraysize.get() == 0 as ::core::ffi::c_int {
            return -1 as ::core::ffi::c_int;
        }
        if cot_fuzzy() as ::core::ffi::c_int != 0 && !compl_no_select && !shown_match_ok {
            compl_shown_match.set(shown_compl);
            shown_match_ok = true_0 != 0;
            cur = 0 as ::core::ffi::c_int;
        }
        '_c2rust_label: {
            if compl_match_arraysize.get() >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"compl_match_arraysize >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1663 as ::core::ffi::c_uint,
                    b"int ins_compl_build_pum(void)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        compl_match_array.set(xcalloc(
            compl_match_arraysize.get() as size_t,
            ::core::mem::size_of::<pumitem_T>(),
        ) as *mut pumitem_T);
        i = 0 as ::core::ffi::c_int;
        comp = match_head;
        while !comp.is_null() {
            (*(*compl_match_array.ptr()).offset(i as isize)).pum_text =
                if !(*comp).cp_text[CPT_ABBR as usize].is_null() {
                    (*comp).cp_text[CPT_ABBR as usize] as *mut ::core::ffi::c_char
                } else {
                    (*comp).cp_str.data
                };
            (*(*compl_match_array.ptr()).offset(i as isize)).pum_kind =
                (*comp).cp_text[CPT_KIND as usize] as *mut ::core::ffi::c_char;
            (*(*compl_match_array.ptr()).offset(i as isize)).pum_info =
                (*comp).cp_text[CPT_INFO as usize] as *mut ::core::ffi::c_char;
            (*(*compl_match_array.ptr()).offset(i as isize)).pum_cpt_source_idx =
                (*comp).cp_cpt_source_idx;
            (*(*compl_match_array.ptr()).offset(i as isize)).pum_user_abbr_hlattr =
                (*comp).cp_user_abbr_hlattr;
            (*(*compl_match_array.ptr()).offset(i as isize)).pum_user_kind_hlattr =
                (*comp).cp_user_kind_hlattr;
            let c2rust_fresh2 = i;
            i = i + 1;
            let c2rust_lvalue_ptr =
                &raw mut (*(*compl_match_array.ptr()).offset(c2rust_fresh2 as isize)).pum_extra;
            *c2rust_lvalue_ptr = if !(*comp).cp_text[CPT_MENU as usize].is_null() {
                (*comp).cp_text[CPT_MENU as usize] as *mut ::core::ffi::c_char
            } else {
                (*comp).cp_fname
            };
            let mut match_next: *mut compl_T = (*comp).cp_match_next;
            (*comp).cp_match_next = ::core::ptr::null_mut::<compl_T>();
            comp = match_next;
        }
        if !shown_match_ok {
            cur = -1 as ::core::ffi::c_int;
        }
        return cur;
    }
}

pub unsafe extern "C" fn ins_compl_show_pum() {
    unsafe {
        if !pum_wanted() || !pum_enough_matches() {
            return;
        }
        update_screen();
        let mut cur: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut array_changed: bool = false_0 != 0;
        if (*compl_match_array.ptr()).is_null() {
            array_changed = true_0 != 0;
            cur = ins_compl_build_pum();
        } else {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < compl_match_arraysize.get() {
                if (*(*compl_match_array.ptr()).offset(i as isize)).pum_text
                    == (*compl_shown_match.get()).cp_str.data
                    || (*(*compl_match_array.ptr()).offset(i as isize)).pum_text
                        == (*compl_shown_match.get()).cp_text[CPT_ABBR as usize]
                {
                    cur = i;
                    break;
                } else {
                    i += 1;
                }
            }
        }
        if (*compl_match_array.ptr()).is_null() {
            if compl_started.get() as ::core::ffi::c_int != 0
                && has_event(EVENT_COMPLETECHANGED) as ::core::ffi::c_int != 0
            {
                trigger_complete_changed_event(cur);
            }
            return;
        }
        dollar_vcol.set(-1 as ::core::ffi::c_int as colnr_T);
        let col: colnr_T = (*curwin.get()).w_cursor.col;
        (*curwin.get()).w_cursor.col = compl_col.get();
        compl_selected_item.set(cur);
        pum_display(
            compl_match_array.get(),
            compl_match_arraysize.get(),
            cur,
            array_changed,
            0 as ::core::ffi::c_int,
        );
        (*curwin.get()).w_cursor.col = col;
        if compl_started.get() as ::core::ffi::c_int != 0
            && compl_curr_match.get() != compl_shown_match.get()
        {
            compl_curr_match.set(compl_shown_match.get());
        }
        if has_event(EVENT_COMPLETECHANGED) {
            trigger_complete_changed_event(cur);
        }
    }
}

pub unsafe extern "C" fn compl_match_curr_select(mut selected: ::core::ffi::c_int) -> bool {
    unsafe {
        if selected < 0 as ::core::ffi::c_int {
            return false_0 != 0;
        }
        let mut match_0: *mut compl_T = compl_first_match.get();
        let mut selected_idx: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut list_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            if !match_at_original_text(match_0) {
                if !(*compl_curr_match.ptr()).is_null()
                    && (*compl_curr_match.get()).cp_number == (*match_0).cp_number
                {
                    selected_idx = list_idx;
                    break;
                } else {
                    list_idx += 1 as ::core::ffi::c_int;
                }
            }
            match_0 = (*match_0).cp_next;
            if !(!match_0.is_null() && !is_first_match(match_0)) {
                break;
            }
        }
        return selected == selected_idx;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_show_filename() {
    unsafe {
        let lead: *mut ::core::ffi::c_char =
            gettext(b"match in file\0".as_ptr() as *const ::core::ffi::c_char);
        let mut space: ::core::ffi::c_int =
            sc_col.get() - vim_strsize(lead) - 2 as ::core::ffi::c_int;
        if space <= 0 as ::core::ffi::c_int {
            return;
        }
        let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut e: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        e = (*compl_shown_match.get()).cp_fname;
        s = e;
        while *e as ::core::ffi::c_int != NUL {
            space -= ptr2cells(e);
            while space < 0 as ::core::ffi::c_int {
                space += ptr2cells(s);
                s = s.offset(utfc_ptr2len(s) as isize);
            }
            e = e.offset(utfc_ptr2len(e) as isize);
        }
        if !compl_autocomplete.get() {
            msg_hist_off.set(true_0 != 0);
            vim_snprintf(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                IOSIZE as size_t,
                b"%s %s%s\0".as_ptr() as *const ::core::ffi::c_char,
                lead,
                if s > (*compl_shown_match.get()).cp_fname {
                    b"<\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                },
                s,
            );
            msg(
                IObuff.ptr() as *mut ::core::ffi::c_char,
                0 as ::core::ffi::c_int,
            );
            msg_hist_off.set(false_0 != 0);
            redraw_cmdline.set(false_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn find_next_match_in_menu() -> *mut compl_T {
    unsafe {
        let mut is_forward: bool = compl_shows_dir_forward();
        let mut match_0: *mut compl_T = compl_shown_match.get();
        loop {
            match_0 = if is_forward as ::core::ffi::c_int != 0 {
                (*match_0).cp_next
            } else {
                (*match_0).cp_prev
            };
            if !(!(*match_0).cp_next.is_null()
                && !(*match_0).cp_in_match_array
                && !match_at_original_text(match_0))
            {
                break;
            }
        }
        return match_0;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_show_statusmsg() {
    unsafe {
        if is_first_match((*compl_first_match.get()).cp_next) {
            edit_submode_extra.set(
                if compl_status_adding() as ::core::ffi::c_int != 0
                    && compl_length.get() > 1 as ::core::ffi::c_int
                {
                    gettext((e_hitend.ptr() as *const _) as *const ::core::ffi::c_char)
                } else {
                    gettext(&raw const e_patnotf as *const ::core::ffi::c_char)
                },
            );
            edit_submode_highl.set(HLF_E);
        }
        if (*edit_submode_extra.ptr()).is_null() {
            if match_at_original_text(compl_curr_match.get()) {
                edit_submode_extra.set(gettext(
                    b"Back at original\0".as_ptr() as *const ::core::ffi::c_char
                ));
                edit_submode_highl.set(HLF_W);
            } else if compl_cont_status.get() & CONT_S_IPOS != 0 {
                edit_submode_extra.set(gettext(
                    b"Word from other line\0".as_ptr() as *const ::core::ffi::c_char
                ));
                edit_submode_highl.set(HLF_COUNT);
            } else if (*compl_curr_match.get()).cp_next == (*compl_curr_match.get()).cp_prev {
                edit_submode_extra.set(gettext(
                    b"The only match\0".as_ptr() as *const ::core::ffi::c_char
                ));
                edit_submode_highl.set(HLF_COUNT);
                (*compl_curr_match.get()).cp_number = 1 as ::core::ffi::c_int;
            } else {
                if (*compl_curr_match.get()).cp_number == -1 as ::core::ffi::c_int {
                    ins_compl_update_sequence_numbers();
                }
                if (*compl_curr_match.get()).cp_number != -1 as ::core::ffi::c_int {
                    static match_ref: GlobalCell<[::core::ffi::c_char; 81]> =
                        GlobalCell::new([0; 81]);
                    if compl_matches.get() > 0 as ::core::ffi::c_int {
                        vim_snprintf(
                            match_ref.ptr() as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 81]>(),
                            gettext(b"match %d of %d\0".as_ptr() as *const ::core::ffi::c_char),
                            (*compl_curr_match.get()).cp_number,
                            compl_matches.get(),
                        );
                    } else {
                        vim_snprintf(
                            match_ref.ptr() as *mut ::core::ffi::c_char,
                            ::core::mem::size_of::<[::core::ffi::c_char; 81]>(),
                            gettext(b"match %d\0".as_ptr() as *const ::core::ffi::c_char),
                            (*compl_curr_match.get()).cp_number,
                        );
                    }
                    edit_submode_extra.set(match_ref.ptr() as *mut ::core::ffi::c_char);
                    edit_submode_highl.set(HLF_R);
                    if dollar_vcol.get() >= 0 as ::core::ffi::c_int {
                        curs_columns(curwin.get(), false_0);
                    }
                }
            }
        }
        redraw_mode.set(true_0 != 0);
        if !shortmess(SHM_COMPLETIONMENU) {
            if !(*edit_submode_extra.ptr()).is_null() {
                if p_smd.get() == 0 {
                    msg_hist_off.set(true_0 != 0);
                    msg_ext_set_kind(b"completion\0".as_ptr() as *const ::core::ffi::c_char);
                    msg(
                        edit_submode_extra.get(),
                        if (edit_submode_highl.get() as ::core::ffi::c_uint)
                            < HLF_COUNT as ::core::ffi::c_uint
                        {
                            edit_submode_highl.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int
                        } else {
                            0 as ::core::ffi::c_int
                        },
                    );
                    msg_hist_off.set(false_0 != 0);
                }
            } else {
                msg_clr_cmdline();
            }
        }
    }
}

pub(crate) unsafe extern "C" fn show_pum(
    mut prev_w_wrow: ::core::ffi::c_int,
    mut prev_w_leftcol: ::core::ffi::c_int,
) {
    unsafe {
        let mut n: ::core::ffi::c_int = RedrawingDisabled.get();
        RedrawingDisabled.set(0 as ::core::ffi::c_int);
        setcursor();
        if prev_w_wrow != (*curwin.get()).w_wrow || prev_w_leftcol != (*curwin.get()).w_leftcol {
            ins_compl_del_pum();
        }
        ins_compl_show_pum();
        setcursor();
        RedrawingDisabled.set(n);
    }
}
