//! The current syntax state and the per-line driver.
//!
//! [`syntax_start`] is the entry point every reader goes through: it points the
//! module's statics at a window/buffer, finds a state to start from (the cache
//! in `stack.rs`, or a `syn_sync` scan) and parses forward to the wanted line.
//! [`syn_finish_line`] is one line of that walk, [`syn_start_line`] resets the
//! per-line state, and [`syn_update_ends`] recomputes where the items on the
//! stack end after the state was loaded from the cache.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn syntax_start(mut wp: *mut win_T, mut lnum: linenr_T) {
    unsafe {
        let mut last_valid: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
        let mut last_min_valid: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
        let mut sp: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
        let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
        let mut first_stored: linenr_T = 0;
        let mut dist: ::core::ffi::c_int = 0;
        static changedtick: GlobalCell<varnumber_T> = GlobalCell::new(0 as varnumber_T);
        current_sub_char.set(NUL);
        if syn_block.get() != (*wp).w_s
            || syn_buf.get() != (*wp).w_buffer
            || changedtick.get() != buf_get_changedtick(syn_buf.get())
        {
            invalidate_current_state();
            syn_buf.set((*wp).w_buffer);
            syn_block.set((*wp).w_s);
        }
        changedtick.set(buf_get_changedtick(syn_buf.get()));
        syn_win.set(wp);
        syn_stack_alloc();
        if (*syn_block.get()).b_sst_array.is_null() {
            return;
        }
        (*syn_block.get()).b_sst_lasttick = display_tick.get();
        if (*current_state.ptr()).ga_itemsize != 0 as ::core::ffi::c_int
            && current_lnum.get() < lnum
            && current_lnum.get() < (*syn_buf.get()).b_ml.ml_line_count
        {
            syn_finish_line(false_0 != 0);
            if !current_state_stored.get() {
                (*current_lnum.ptr()) += 1;
                store_current_state();
            }
            if current_lnum.get() != lnum {
                invalidate_current_state();
            }
        } else {
            invalidate_current_state();
        }
        if (*current_state.ptr()).ga_itemsize == 0 as ::core::ffi::c_int
            && !(*syn_block.get()).b_sst_array.is_null()
        {
            let mut p: *mut synstate_T = (*syn_block.get()).b_sst_first;
            while !p.is_null() {
                if (*p).sst_lnum > lnum {
                    break;
                }
                if (*p).sst_change_lnum == 0 as linenr_T {
                    last_valid = p;
                    if (*p).sst_lnum >= lnum - (*syn_block.get()).b_syn_sync_minlines {
                        last_min_valid = p;
                    }
                }
                p = (*p).sst_next;
            }
            if !last_min_valid.is_null() {
                load_current_state(last_min_valid);
            }
        }
        if (*current_state.ptr()).ga_itemsize == 0 as ::core::ffi::c_int {
            syn_sync(wp, lnum, last_valid);
            if current_lnum.get() == 1 as linenr_T {
                first_stored = 1 as ::core::ffi::c_int as linenr_T;
            } else {
                first_stored = current_lnum.get() + (*syn_block.get()).b_syn_sync_minlines;
            }
        } else {
            first_stored = current_lnum.get();
        }
        if (*syn_block.get()).b_sst_len <= Rows.get() {
            dist = 999999 as ::core::ffi::c_int;
        } else {
            dist = ((*syn_buf.get()).b_ml.ml_line_count
                / ((*syn_block.get()).b_sst_len as linenr_T - Rows.get() as linenr_T)
                + 1 as linenr_T) as ::core::ffi::c_int;
        }
        while current_lnum.get() < lnum {
            syn_start_line();
            syn_finish_line(false_0 != 0);
            (*current_lnum.ptr()) += 1;
            if current_lnum.get() >= first_stored {
                if prev.is_null() {
                    prev = syn_stack_find_entry(current_lnum.get() - 1 as linenr_T);
                }
                if prev.is_null() {
                    sp = (*syn_block.get()).b_sst_first;
                } else {
                    sp = prev;
                }
                while !sp.is_null() && (*sp).sst_lnum < current_lnum.get() {
                    sp = (*sp).sst_next;
                }
                if !sp.is_null()
                    && (*sp).sst_lnum == current_lnum.get()
                    && syn_stack_equal(sp) as ::core::ffi::c_int != 0
                {
                    let mut parsed_lnum: linenr_T = current_lnum.get();
                    prev = sp;
                    while !sp.is_null() && (*sp).sst_change_lnum <= parsed_lnum {
                        if (*sp).sst_lnum <= lnum {
                            prev = sp;
                        } else if (*sp).sst_change_lnum == 0 as linenr_T {
                            break;
                        }
                        (*sp).sst_change_lnum = 0 as ::core::ffi::c_int as linenr_T;
                        sp = (*sp).sst_next;
                    }
                    load_current_state(prev);
                } else if prev.is_null()
                    || current_lnum.get() == lnum
                    || current_lnum.get() >= (*prev).sst_lnum + dist as linenr_T
                {
                    prev = store_current_state();
                }
            }
            line_breakcheck();
            if !got_int.get() {
                continue;
            }
            current_lnum.set(lnum);
            break;
        }
        syn_start_line();
    }
}

pub(crate) unsafe extern "C" fn clear_syn_state(mut p: *mut synstate_T) {
    unsafe {
        if (*p).sst_stacksize > SST_FIX_STATES {
            let mut _gap: *mut garray_T = &raw mut (*p).sst_union.sst_ga;
            if !(*_gap).ga_data.is_null() {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*_gap).ga_len {
                    let mut _item: *mut bufstate_T =
                        ((*_gap).ga_data as *mut bufstate_T).offset(i as isize);
                    unref_extmatch((*_item).bs_extmatch);
                    i += 1;
                }
            }
            ga_clear(_gap);
        } else {
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < (*p).sst_stacksize {
                unref_extmatch((*p).sst_union.sst_stack[i_0 as usize].bs_extmatch);
                i_0 += 1;
            }
        };
    }
}

pub(crate) unsafe extern "C" fn clear_current_state() {
    unsafe {
        let mut _gap: *mut garray_T = current_state.ptr();
        if !(*_gap).ga_data.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*_gap).ga_len {
                let mut _item: *mut stateitem_T =
                    ((*_gap).ga_data as *mut stateitem_T).offset(i as isize);
                unref_extmatch((*_item).si_extmatch);
                i += 1;
            }
        }
        ga_clear(_gap);
    }
}

pub(crate) unsafe extern "C" fn syn_start_line() {
    unsafe {
        current_finished.set(false_0 != 0);
        current_col.set(0 as ::core::ffi::c_int as colnr_T);
        if !((*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int) {
            syn_update_ends(true_0 != 0);
            check_state_ends();
        }
        next_match_idx.set(-1 as ::core::ffi::c_int);
        (*current_line_id.ptr()) += 1;
        next_seqnr.set(1 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn syn_update_ends(mut startofline: bool) {
    unsafe {
        let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
        if startofline {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*current_state.ptr()).ga_len {
                cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize);
                if (*cur_si).si_idx >= 0 as ::core::ffi::c_int
                    && (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset((*cur_si).si_idx as isize))
                    .sp_type as ::core::ffi::c_int
                        == SPTYPE_MATCH
                    && (*cur_si).si_m_endpos.lnum < current_lnum.get()
                {
                    (*cur_si).si_flags |= HL_MATCHCONT;
                    (*cur_si).si_m_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
                    (*cur_si).si_m_endpos.col = 0 as ::core::ffi::c_int as colnr_T;
                    (*cur_si).si_h_endpos = (*cur_si).si_m_endpos;
                    (*cur_si).si_ends = true_0;
                }
                i += 1;
            }
        }
        let mut i_0: ::core::ffi::c_int = (*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int;
        if keepend_level.get() >= 0 as ::core::ffi::c_int {
            while i_0 > keepend_level.get() {
                if (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i_0 as isize))
                    .si_flags
                    & HL_EXTEND
                    != 0
                {
                    break;
                }
                i_0 -= 1;
            }
        }
        let mut seen_keepend: bool = false_0 != 0;
        while i_0 < (*current_state.ptr()).ga_len {
            cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i_0 as isize);
            if (*cur_si).si_flags & HL_KEEPEND != 0
                || seen_keepend as ::core::ffi::c_int != 0 && !startofline
                || i_0 == (*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int
                    && startofline as ::core::ffi::c_int != 0
            {
                (*cur_si).si_h_startpos.col = 0 as ::core::ffi::c_int as colnr_T;
                (*cur_si).si_h_startpos.lnum = current_lnum.get();
                if (*cur_si).si_flags & HL_MATCHCONT == 0 {
                    update_si_end(cur_si, current_col.get(), !startofline);
                }
                if !startofline && (*cur_si).si_flags & HL_KEEPEND != 0 {
                    seen_keepend = true_0 != 0;
                }
            }
            i_0 += 1;
        }
        check_keepend();
    }
}

pub unsafe extern "C" fn syntax_end_parsing(mut wp: *mut win_T, mut lnum: linenr_T) {
    unsafe {
        let mut sp: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
        if syn_block.get() != (*wp).w_s {
            return;
        }
        sp = syn_stack_find_entry(lnum);
        if !sp.is_null() && (*sp).sst_lnum < lnum {
            sp = (*sp).sst_next;
        }
        if !sp.is_null() && (*sp).sst_change_lnum != 0 as linenr_T {
            (*sp).sst_change_lnum = lnum;
        }
    }
}

pub(crate) unsafe extern "C" fn invalidate_current_state() {
    unsafe {
        clear_current_state();
        (*current_state.ptr()).ga_itemsize = 0 as ::core::ffi::c_int;
        current_next_list.set(::core::ptr::null_mut::<int16_t>());
        keepend_level.set(-1 as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn validate_current_state() {
    unsafe {
        (*current_state.ptr()).ga_itemsize =
            ::core::mem::size_of::<stateitem_T>() as ::core::ffi::c_int;
        ga_set_growsize(current_state.ptr(), 3 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn syntax_check_changed(mut lnum: linenr_T) -> bool {
    unsafe {
        let mut retval: bool = true_0 != 0;
        let mut sp: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
        if (*current_state.ptr()).ga_itemsize != 0 as ::core::ffi::c_int
            && lnum == current_lnum.get() + 1 as linenr_T
        {
            sp = syn_stack_find_entry(lnum);
            if !sp.is_null() && (*sp).sst_lnum == lnum {
                syn_finish_line(false_0 != 0);
                if syn_stack_equal(sp) {
                    retval = false_0 != 0;
                }
                (*current_lnum.ptr()) += 1;
                store_current_state();
            }
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn syn_finish_line(syncing: bool) -> bool {
    unsafe {
        while !current_finished.get() {
            syn_current_attr(
                syncing,
                false_0 != 0,
                ::core::ptr::null_mut::<bool>(),
                false_0 != 0,
            );
            if syncing as ::core::ffi::c_int != 0 && (*current_state.ptr()).ga_len != 0 {
                let cur_si: *const stateitem_T = ((*current_state.ptr()).ga_data
                    as *mut stateitem_T)
                    .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
                if (*cur_si).si_idx >= 0 as ::core::ffi::c_int
                    && (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset((*cur_si).si_idx as isize))
                    .sp_flags
                        & (HL_SYNC_HERE | HL_SYNC_THERE)
                        != 0
                {
                    return true_0 != 0;
                }
                let prev_current_col: colnr_T = current_col.get();
                if *syn_getcurline().offset(current_col.get() as isize) as ::core::ffi::c_int != NUL
                {
                    (*current_col.ptr()) += 1;
                }
                check_state_ends();
                current_col.set(prev_current_col);
            }
            (*current_col.ptr()) += 1;
        }
        return false_0 != 0;
    }
}
