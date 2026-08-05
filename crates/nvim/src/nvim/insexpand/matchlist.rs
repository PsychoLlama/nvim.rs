//! The `compl_T` match list: adding, freeing and ordering the matches.
//!
//! [`ins_compl_add`] links a new match into the circular doubly-linked list
//! `compl_first_match` heads, rejecting duplicates unless the caller allows
//! them; [`ins_compl_make_cyclic`] closes the ring and
//! [`ins_compl_make_linear`] opens it again.  The comparators and
//! [`sort_compl_match_list`] are `'completeopt'`'s `fuzzy` and `nearest`
//! orderings.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[inline]
pub(crate) unsafe extern "C" fn free_cptext(cptext: *const *mut ::core::ffi::c_char) {
    unsafe {
        if !cptext.is_null() {
            let mut i: size_t = 0 as size_t;
            while i < CPT_COUNT as ::core::ffi::c_int as size_t {
                xfree(*cptext.offset(i as isize) as *mut ::core::ffi::c_void);
                i = i.wrapping_add(1);
            }
        }
    }
}

pub(crate) unsafe extern "C" fn ins_compl_add(
    str: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    fname: *mut ::core::ffi::c_char,
    cptext: *const *mut ::core::ffi::c_char,
    cptext_allocated: bool,
    mut user_data: *mut typval_T,
    cdir: Direction,
    mut flags_arg: ::core::ffi::c_int,
    adup: bool,
    mut user_hl: *const ::core::ffi::c_int,
    score: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut match_0: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
        let dir: Direction =
            (if cdir as ::core::ffi::c_int == kDirectionNotSet as ::core::ffi::c_int {
                compl_direction.get() as ::core::ffi::c_int
            } else {
                cdir as ::core::ffi::c_int
            }) as Direction;
        let mut flags: ::core::ffi::c_int = flags_arg;
        let mut inserted: bool = false_0 != 0;
        if flags & CP_FAST as ::core::ffi::c_int != 0 {
            fast_breakcheck();
        } else {
            os_breakcheck();
        }
        if got_int.get() {
            if cptext_allocated {
                free_cptext(cptext);
            }
            return FAIL;
        }
        if len < 0 as ::core::ffi::c_int {
            len = strlen(str) as ::core::ffi::c_int;
        }
        if !(*compl_first_match.ptr()).is_null() && !adup {
            match_0 = compl_first_match.get();
            loop {
                if !match_at_original_text(match_0)
                    && strncmp((*match_0).cp_str.data, str, len as size_t)
                        == 0 as ::core::ffi::c_int
                    && ((*match_0).cp_str.size as ::core::ffi::c_int <= len
                        || *(*match_0).cp_str.data.offset(len as isize) as ::core::ffi::c_int
                            == NUL)
                {
                    if is_nearest_active() as ::core::ffi::c_int != 0
                        && score > 0 as ::core::ffi::c_int
                        && score < (*match_0).cp_score
                    {
                        (*match_0).cp_score = score;
                    }
                    if cptext_allocated {
                        free_cptext(cptext);
                    }
                    return NOTDONE;
                }
                match_0 = (*match_0).cp_next;
                if !(!match_0.is_null() && !is_first_match(match_0)) {
                    break;
                }
            }
        }
        ins_compl_del_pum();
        match_0 = xcalloc(1 as size_t, ::core::mem::size_of::<compl_T>()) as *mut compl_T;
        (*match_0).cp_number = if flags & CP_ORIGINAL_TEXT as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        };
        (*match_0).cp_str = cbuf_to_string(str, len as size_t);
        if !fname.is_null()
            && !(*compl_curr_match.ptr()).is_null()
            && !(*compl_curr_match.get()).cp_fname.is_null()
            && strcmp(fname, (*compl_curr_match.get()).cp_fname) == 0 as ::core::ffi::c_int
        {
            (*match_0).cp_fname = (*compl_curr_match.get()).cp_fname;
        } else if !fname.is_null() {
            (*match_0).cp_fname = xstrdup(fname);
            flags |= CP_FREE_FNAME as ::core::ffi::c_int;
        } else {
            (*match_0).cp_fname = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        (*match_0).cp_flags = flags;
        (*match_0).cp_user_abbr_hlattr = if !user_hl.is_null() {
            *user_hl.offset(0 as ::core::ffi::c_int as isize)
        } else {
            -1 as ::core::ffi::c_int
        };
        (*match_0).cp_user_kind_hlattr = if !user_hl.is_null() {
            *user_hl.offset(1 as ::core::ffi::c_int as isize)
        } else {
            -1 as ::core::ffi::c_int
        };
        (*match_0).cp_score = score;
        (*match_0).cp_cpt_source_idx = cpt_sources_index.get();
        if !cptext.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < CPT_COUNT as ::core::ffi::c_int {
                if !(*cptext.offset(i as isize)).is_null() {
                    if **cptext.offset(i as isize) as ::core::ffi::c_int != NUL {
                        (*match_0).cp_text[i as usize] =
                            (if cptext_allocated as ::core::ffi::c_int != 0 {
                                *cptext.offset(i as isize)
                            } else {
                                xstrdup(*cptext.offset(i as isize))
                            }) as *mut ::core::ffi::c_char;
                    } else if cptext_allocated {
                        xfree(*cptext.offset(i as isize) as *mut ::core::ffi::c_void);
                    }
                }
                i += 1;
            }
        }
        if !user_data.is_null() {
            (*match_0).cp_user_data = *user_data;
        }
        if (*compl_first_match.ptr()).is_null() {
            (*match_0).cp_prev = ::core::ptr::null_mut::<compl_T>();
            (*match_0).cp_next = (*match_0).cp_prev;
        } else if cot_fuzzy() as ::core::ffi::c_int != 0
            && score != FUZZY_SCORE_NONE as ::core::ffi::c_int
            && compl_get_longest.get() as ::core::ffi::c_int != 0
        {
            let mut current: *mut compl_T = (*compl_first_match.get()).cp_next;
            let mut prev: *mut compl_T = compl_first_match.get();
            inserted = false_0 != 0;
            while !current.is_null() && current != compl_first_match.get() {
                if (*current).cp_score < score {
                    (*match_0).cp_next = current;
                    (*match_0).cp_prev = (*current).cp_prev;
                    if !(*current).cp_prev.is_null() {
                        (*(*current).cp_prev).cp_next = match_0;
                    }
                    (*current).cp_prev = match_0;
                    inserted = true_0 != 0;
                    break;
                } else {
                    prev = current;
                    current = (*current).cp_next;
                }
            }
            if !inserted {
                (*prev).cp_next = match_0;
                (*match_0).cp_prev = prev;
                (*match_0).cp_next = compl_first_match.get();
                (*compl_first_match.get()).cp_prev = match_0;
            }
        } else if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
            (*match_0).cp_next = (*compl_curr_match.get()).cp_next;
            (*match_0).cp_prev = compl_curr_match.get();
        } else {
            (*match_0).cp_next = compl_curr_match.get();
            (*match_0).cp_prev = (*compl_curr_match.get()).cp_prev;
        }
        if !(*match_0).cp_next.is_null() {
            (*(*match_0).cp_next).cp_prev = match_0;
        }
        if !(*match_0).cp_prev.is_null() {
            (*(*match_0).cp_prev).cp_next = match_0;
        } else {
            compl_first_match.set(match_0);
        }
        compl_curr_match.set(match_0);
        if compl_get_longest.get() as ::core::ffi::c_int != 0
            && flags & CP_ORIGINAL_TEXT as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && !cot_fuzzy()
            && !ins_compl_preinsert_longest()
            && !ctrl_x_mode_thesaurus()
        {
            ins_compl_longest_match(match_0);
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_equal(
    mut match_0: *mut compl_T,
    mut str: *mut ::core::ffi::c_char,
    mut len: size_t,
) -> bool {
    unsafe {
        if (*match_0).cp_flags & CP_EQUAL as ::core::ffi::c_int != 0 {
            return true_0 != 0;
        }
        if (*match_0).cp_flags & CP_ICASE as ::core::ffi::c_int != 0 {
            return strncasecmp((*match_0).cp_str.data, str, len) == 0 as ::core::ffi::c_int;
        }
        return strncmp((*match_0).cp_str.data, str, len) == 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn ins_compl_longest_match(mut match_0: *mut compl_T) {
    unsafe {
        if (*compl_leader.ptr()).data.is_null() {
            compl_leader.set(copy_string(
                (*match_0).cp_str,
                ::core::ptr::null_mut::<Arena>(),
            ));
            let mut had_match: bool = (*curwin.get()).w_cursor.col > compl_col.get();
            ins_compl_longest_insert((*compl_leader.ptr()).data);
            if !had_match {
                ins_compl_delete(false_0 != 0);
            }
            compl_used_match.set(false_0 != 0);
            return;
        }
        let mut p: *mut ::core::ffi::c_char = (*compl_leader.ptr()).data;
        let mut s: *mut ::core::ffi::c_char = (*match_0).cp_str.data;
        while *p as ::core::ffi::c_int != NUL {
            let mut c1: ::core::ffi::c_int = utf_ptr2char(p);
            let mut c2: ::core::ffi::c_int = utf_ptr2char(s);
            if if (*match_0).cp_flags & CP_ICASE as ::core::ffi::c_int != 0 {
                (mb_tolower(c1) != mb_tolower(c2)) as ::core::ffi::c_int
            } else {
                (c1 != c2) as ::core::ffi::c_int
            } != 0
            {
                break;
            }
            p = p.offset(utfc_ptr2len(p) as isize);
            s = s.offset(utfc_ptr2len(s) as isize);
        }
        if *p as ::core::ffi::c_int != NUL {
            *p = NUL as ::core::ffi::c_char;
            (*compl_leader.ptr()).size = p.offset_from((*compl_leader.ptr()).data) as size_t;
            let mut had_match_0: bool = (*curwin.get()).w_cursor.col > compl_col.get();
            ins_compl_longest_insert((*compl_leader.ptr()).data);
            if !had_match_0 {
                ins_compl_delete(false_0 != 0);
            }
        }
        compl_used_match.set(false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn ins_compl_add_matches(
    mut num_matches: ::core::ffi::c_int,
    mut matches: *mut *mut ::core::ffi::c_char,
    mut icase: ::core::ffi::c_int,
) {
    unsafe {
        let mut add_r: ::core::ffi::c_int = OK;
        let mut dir: Direction = compl_direction.get();
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < num_matches && add_r != FAIL {
            add_r = ins_compl_add(
                *matches.offset(i as isize),
                -1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null::<*mut ::core::ffi::c_char>(),
                false_0 != 0,
                ::core::ptr::null_mut::<typval_T>(),
                dir,
                CP_FAST as ::core::ffi::c_int
                    | (if icase != 0 {
                        CP_ICASE as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    }),
                false_0 != 0,
                ::core::ptr::null::<::core::ffi::c_int>(),
                FUZZY_SCORE_NONE as ::core::ffi::c_int,
            );
            if add_r == OK {
                dir = FORWARD;
            }
            i += 1;
        }
        FreeWild(num_matches, matches);
    }
}

pub(crate) unsafe extern "C" fn ins_compl_make_cyclic() -> ::core::ffi::c_int {
    unsafe {
        if (*compl_first_match.ptr()).is_null() {
            return 0 as ::core::ffi::c_int;
        }
        let mut match_0: *mut compl_T = compl_first_match.get();
        let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !(*match_0).cp_next.is_null() && !is_first_match((*match_0).cp_next) {
            match_0 = (*match_0).cp_next;
            count += 1;
        }
        (*match_0).cp_next = compl_first_match.get();
        (*compl_first_match.get()).cp_prev = match_0;
        return count;
    }
}

pub(crate) unsafe extern "C" fn cp_get_next(
    mut node: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    unsafe {
        return (*(node as *mut compl_T)).cp_next as *mut ::core::ffi::c_void;
    }
}

pub(crate) unsafe extern "C" fn cp_set_next(
    mut node: *mut ::core::ffi::c_void,
    mut next: *mut ::core::ffi::c_void,
) {
    unsafe {
        (*(node as *mut compl_T)).cp_next = next as *mut compl_T;
    }
}

pub(crate) unsafe extern "C" fn cp_get_prev(
    mut node: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_void {
    unsafe {
        return (*(node as *mut compl_T)).cp_prev as *mut ::core::ffi::c_void;
    }
}

pub(crate) unsafe extern "C" fn cp_set_prev(
    mut node: *mut ::core::ffi::c_void,
    mut prev: *mut ::core::ffi::c_void,
) {
    unsafe {
        (*(node as *mut compl_T)).cp_prev = prev as *mut compl_T;
    }
}

pub(crate) unsafe extern "C" fn cp_compare_fuzzy(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut score_a: ::core::ffi::c_int = (*(a as *mut compl_T)).cp_score;
        let mut score_b: ::core::ffi::c_int = (*(b as *mut compl_T)).cp_score;
        return if score_b > score_a {
            1 as ::core::ffi::c_int
        } else if score_b < score_a {
            -1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
}

pub(crate) unsafe extern "C" fn cp_compare_nearest(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut score_a: ::core::ffi::c_int = (*(a as *mut compl_T)).cp_score;
        let mut score_b: ::core::ffi::c_int = (*(b as *mut compl_T)).cp_score;
        if score_a == FUZZY_SCORE_NONE as ::core::ffi::c_int
            || score_b == FUZZY_SCORE_NONE as ::core::ffi::c_int
        {
            return 0 as ::core::ffi::c_int;
        }
        return if score_a > score_b {
            1 as ::core::ffi::c_int
        } else if score_a < score_b {
            -1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        };
    }
}

pub(crate) unsafe extern "C" fn set_fuzzy_score() {
    unsafe {
        if (*compl_first_match.ptr()).is_null() {
            return;
        }
        let mut use_leader: bool =
            !(*compl_leader.ptr()).data.is_null() && (*compl_leader.ptr()).size > 0 as size_t;
        let mut pattern: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !use_leader {
            if (*compl_orig_text.ptr()).data.is_null()
                || (*compl_orig_text.ptr()).size == 0 as size_t
            {
                return;
            }
            pattern = (*compl_orig_text.ptr()).data;
        } else {
            get_leader_for_startcol(::core::ptr::null_mut::<compl_T>(), true_0 != 0);
            pattern = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        let mut comp: *mut compl_T = compl_first_match.get();
        loop {
            if use_leader {
                pattern = (*get_leader_for_startcol(comp, true_0 != 0)).data;
            }
            (*comp).cp_score = fuzzy_match_str((*comp).cp_str.data, pattern);
            comp = (*comp).cp_next;
            if !(!comp.is_null() && !is_first_match(comp)) {
                break;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn sort_compl_match_list(mut compare: MergeSortCompareFunc) {
    unsafe {
        if (*compl_first_match.ptr()).is_null()
            || is_first_match((*compl_first_match.get()).cp_next) as ::core::ffi::c_int != 0
        {
            return;
        }
        let mut comp: *mut compl_T = (*compl_first_match.get()).cp_prev;
        ins_compl_make_linear();
        if compl_shows_dir_forward() {
            (*(*compl_first_match.get()).cp_next).cp_prev = ::core::ptr::null_mut::<compl_T>();
            (*compl_first_match.get()).cp_next = mergesort_list(
                (*compl_first_match.get()).cp_next as *mut ::core::ffi::c_void,
                Some(
                    cp_get_next
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                        )
                            -> *mut ::core::ffi::c_void,
                ),
                Some(
                    cp_set_next
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                Some(
                    cp_get_prev
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                        )
                            -> *mut ::core::ffi::c_void,
                ),
                Some(
                    cp_set_prev
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                compare,
            ) as *mut compl_T;
            (*(*compl_first_match.get()).cp_next).cp_prev = compl_first_match.get();
        } else {
            (*(*comp).cp_prev).cp_next = ::core::ptr::null_mut::<compl_T>();
            compl_first_match.set(mergesort_list(
                compl_first_match.get() as *mut ::core::ffi::c_void,
                Some(
                    cp_get_next
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                        )
                            -> *mut ::core::ffi::c_void,
                ),
                Some(
                    cp_set_next
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                Some(
                    cp_get_prev
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                        )
                            -> *mut ::core::ffi::c_void,
                ),
                Some(
                    cp_set_prev
                        as unsafe extern "C" fn(
                            *mut ::core::ffi::c_void,
                            *mut ::core::ffi::c_void,
                        ) -> (),
                ),
                compare,
            ) as *mut compl_T);
            let mut tail: *mut compl_T = compl_first_match.get();
            while !(*tail).cp_next.is_null() {
                tail = (*tail).cp_next;
            }
            (*tail).cp_next = comp;
            (*comp).cp_prev = tail;
        }
        ins_compl_make_cyclic();
    }
}

pub(crate) unsafe extern "C" fn ins_compl_item_free(mut match_0: *mut compl_T) {
    unsafe {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*match_0).cp_str.data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*match_0).cp_str.size = 0 as size_t;
        if (*match_0).cp_flags & CP_FREE_FNAME as ::core::ffi::c_int != 0 {
            xfree((*match_0).cp_fname as *mut ::core::ffi::c_void);
        }
        free_cptext(&raw mut (*match_0).cp_text as *mut *mut ::core::ffi::c_char);
        tv_clear(&raw mut (*match_0).cp_user_data);
        xfree(match_0 as *mut ::core::ffi::c_void);
    }
}

pub(crate) unsafe extern "C" fn ins_compl_free() {
    unsafe {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*compl_pattern.ptr()).size = 0 as size_t;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
        (*compl_leader.ptr()).size = 0 as size_t;
        if (*compl_first_match.ptr()).is_null() {
            return;
        }
        ins_compl_del_pum();
        pum_clear();
        compl_curr_match.set(compl_first_match.get());
        loop {
            let mut match_0: *mut compl_T = compl_curr_match.get();
            compl_curr_match.set((*compl_curr_match.get()).cp_next);
            ins_compl_item_free(match_0);
            if !(!(*compl_curr_match.ptr()).is_null() && !is_first_match(compl_curr_match.get())) {
                break;
            }
        }
        compl_curr_match.set(::core::ptr::null_mut::<compl_T>());
        compl_first_match.set(compl_curr_match.get());
        compl_shown_match.set(::core::ptr::null_mut::<compl_T>());
        compl_old_match.set(::core::ptr::null_mut::<compl_T>());
    }
}

pub unsafe extern "C" fn ins_compl_clear() {
    unsafe {
        compl_cont_status.set(0 as ::core::ffi::c_int);
        compl_started.set(false_0 != 0);
        compl_matches.set(0 as ::core::ffi::c_int);
        compl_selected_item.set(-1 as ::core::ffi::c_int);
        compl_ins_end_col.set(0 as ::core::ffi::c_int as colnr_T);
        compl_curr_win.set(::core::ptr::null_mut::<win_T>());
        compl_curr_buf.set(::core::ptr::null_mut::<buf_T>());
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_pattern.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*compl_pattern.ptr()).size = 0 as size_t;
        let mut ptr__0: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_leader.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__0);
        *ptr__0 = NULL;
        let _ = *ptr__0;
        (*compl_leader.ptr()).size = 0 as size_t;
        edit_submode_extra.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        xfree((*compl_orig_extmarks.ptr()).items as *mut ::core::ffi::c_void);
        (*compl_orig_extmarks.ptr()).capacity = 0 as size_t;
        (*compl_orig_extmarks.ptr()).size = (*compl_orig_extmarks.ptr()).capacity;
        (*compl_orig_extmarks.ptr()).items = ::core::ptr::null_mut::<ExtmarkUndoObject>();
        let mut ptr__1: *mut *mut ::core::ffi::c_void =
            &raw mut (*compl_orig_text.ptr()).data as *mut *mut ::core::ffi::c_void;
        xfree(*ptr__1);
        *ptr__1 = NULL;
        let _ = *ptr__1;
        (*compl_orig_text.ptr()).size = 0 as size_t;
        compl_enter_selects.set(false_0 != 0);
        cpt_sources_clear();
        compl_autocomplete.set(false_0 != 0);
        compl_from_nonkeyword.set(false_0 != 0);
        compl_num_bests.set(0 as ::core::ffi::c_int);
        set_vim_var_dict(VV_COMPLETED_ITEM, tv_dict_alloc_lock(VAR_FIXED));
    }
}

pub(crate) unsafe extern "C" fn ins_compl_fuzzy_sort() {
    unsafe {
        let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
        set_fuzzy_score();
        if cur_cot_flags & kOptCotFlagNosort as ::core::ffi::c_int as ::core::ffi::c_uint == 0 {
            sort_compl_match_list(Some(
                cp_compare_fuzzy
                    as unsafe extern "C" fn(
                        *const ::core::ffi::c_void,
                        *const ::core::ffi::c_void,
                    ) -> ::core::ffi::c_int,
            ));
            if cur_cot_flags
                & (kOptCotFlagNoinsert as ::core::ffi::c_int
                    | kOptCotFlagNoselect as ::core::ffi::c_int)
                    as ::core::ffi::c_uint
                == kOptCotFlagNoinsert as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut none_selected: bool = compl_shown_match.get()
                    == (if compl_shows_dir_forward() as ::core::ffi::c_int != 0 {
                        compl_first_match.get()
                    } else {
                        (*compl_first_match.get()).cp_prev
                    });
                if !none_selected {
                    compl_shown_match.set(
                        if !compl_autocomplete.get()
                            && compl_shows_dir_forward() as ::core::ffi::c_int != 0
                        {
                            (*compl_first_match.get()).cp_next
                        } else {
                            compl_first_match.get()
                        },
                    );
                }
            }
        }
    }
}

pub(crate) unsafe extern "C" fn ins_compl_update_sequence_numbers() {
    unsafe {
        let mut number: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut match_0: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
        if compl_dir_forward() {
            match_0 = (*compl_curr_match.get()).cp_prev;
            while !match_0.is_null() && !is_first_match(match_0) {
                if (*match_0).cp_number != -1 as ::core::ffi::c_int {
                    number = (*match_0).cp_number;
                    break;
                } else {
                    match_0 = (*match_0).cp_prev;
                }
            }
            if !match_0.is_null() {
                match_0 = (*match_0).cp_next;
                while !match_0.is_null() && (*match_0).cp_number == -1 as ::core::ffi::c_int {
                    number += 1;
                    (*match_0).cp_number = number;
                    match_0 = (*match_0).cp_next;
                }
            }
        } else {
            '_c2rust_label: {
                if compl_direction.get() as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"compl_direction == BACKWARD\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/insexpand.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        3532 as ::core::ffi::c_uint,
                        b"void ins_compl_update_sequence_numbers(void)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            match_0 = (*compl_curr_match.get()).cp_next;
            while !match_0.is_null() && !is_first_match(match_0) {
                if (*match_0).cp_number != -1 as ::core::ffi::c_int {
                    number = (*match_0).cp_number;
                    break;
                } else {
                    match_0 = (*match_0).cp_next;
                }
            }
            if !match_0.is_null() {
                match_0 = (*match_0).cp_prev;
                while !match_0.is_null() && (*match_0).cp_number == -1 as ::core::ffi::c_int {
                    number += 1;
                    (*match_0).cp_number = number;
                    match_0 = (*match_0).cp_prev;
                }
            }
        };
    }
}

pub(crate) unsafe extern "C" fn compare_scores(
    mut a: *const ::core::ffi::c_void,
    mut b: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut idx_a: ::core::ffi::c_int = *(a as *const ::core::ffi::c_int);
        let mut idx_b: ::core::ffi::c_int = *(b as *const ::core::ffi::c_int);
        let mut score_a: ::core::ffi::c_int = *(*compl_fuzzy_scores.ptr()).offset(idx_a as isize);
        let mut score_b: ::core::ffi::c_int = *(*compl_fuzzy_scores.ptr()).offset(idx_b as isize);
        return if score_a == score_b {
            if idx_a == idx_b {
                0 as ::core::ffi::c_int
            } else if idx_a < idx_b {
                -1 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            }
        } else if score_a > score_b {
            -1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
    }
}

pub(crate) unsafe extern "C" fn ins_compl_make_linear() {
    unsafe {
        if (*compl_first_match.ptr()).is_null() || (*compl_first_match.get()).cp_prev.is_null() {
            return;
        }
        let mut m: *mut compl_T = (*compl_first_match.get()).cp_prev;
        (*m).cp_next = ::core::ptr::null_mut::<compl_T>();
        (*compl_first_match.get()).cp_prev = ::core::ptr::null_mut::<compl_T>();
    }
}

pub(crate) unsafe extern "C" fn remove_old_matches() {
    unsafe {
        let mut shown_match_removed: bool = false_0 != 0;
        let mut forward: bool =
            (*compl_first_match.get()).cp_cpt_source_idx < 0 as ::core::ffi::c_int;
        if cpt_sources_index.get() < 0 as ::core::ffi::c_int {
            return;
        }
        compl_direction.set(
            (if forward as ::core::ffi::c_int != 0 {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            }) as Direction,
        );
        compl_shows_dir.set(compl_direction.get());
        let mut current: *mut compl_T = compl_first_match.get();
        while !current.is_null() {
            if (*current).cp_cpt_source_idx == cpt_sources_index.get() {
                let mut to_delete: *mut compl_T = current;
                if !shown_match_removed && compl_shown_match.get() == current {
                    shown_match_removed = true_0 != 0;
                }
                current = (*current).cp_next;
                if to_delete == compl_first_match.get() {
                    compl_first_match.set((*to_delete).cp_next);
                    (*compl_first_match.get()).cp_prev = ::core::ptr::null_mut::<compl_T>();
                } else if (*to_delete).cp_next.is_null() {
                    (*(*to_delete).cp_prev).cp_next = ::core::ptr::null_mut::<compl_T>();
                } else {
                    (*(*to_delete).cp_prev).cp_next = (*to_delete).cp_next;
                    (*(*to_delete).cp_next).cp_prev = (*to_delete).cp_prev;
                }
                ins_compl_item_free(to_delete);
            } else {
                current = (*current).cp_next;
            }
        }
        if shown_match_removed {
            if forward {
                compl_shown_match.set(compl_first_match.get());
            } else {
                let mut current_0: *mut compl_T = ::core::ptr::null_mut::<compl_T>();
                current_0 = compl_first_match.get();
                while !(*current_0).cp_next.is_null() {
                    current_0 = (*current_0).cp_next;
                }
                compl_shown_match.set(current_0);
            }
        }
        compl_curr_match.set(compl_first_match.get());
        let mut current_1: *mut compl_T = compl_first_match.get();
        while !current_1.is_null() {
            if if forward as ::core::ffi::c_int != 0 {
                ((*current_1).cp_cpt_source_idx < cpt_sources_index.get()) as ::core::ffi::c_int
            } else {
                ((*current_1).cp_cpt_source_idx > cpt_sources_index.get()) as ::core::ffi::c_int
            } == 0
            {
                break;
            }
            compl_curr_match.set(if forward as ::core::ffi::c_int != 0 {
                current_1
            } else {
                (*current_1).cp_next
            });
            current_1 = (*current_1).cp_next;
        }
    }
}
