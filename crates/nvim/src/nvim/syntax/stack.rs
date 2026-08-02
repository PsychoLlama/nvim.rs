//! The `synstate_T` cache.
//!
//! Parsing a line means parsing every line before it, so the state at the start
//! of a line is remembered for every `SST_DIST`th line and reused. This is that
//! store: allocation ([`syn_stack_alloc`]), the free list and its recycling
//! ([`syn_stack_cleanup`]), the save and restore of the current state
//! ([`store_current_state`], [`load_current_state`]), the equality test that
//! decides whether a re-parse can stop early ([`syn_stack_equal`]), and the
//! invalidation an edit causes ([`syn_stack_apply_changes`]).

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn syn_stack_free_block(mut block: *mut synblock_T) {
    unsafe {
        if (*block).b_sst_array.is_null() {
            return;
        }
        let mut p: *mut synstate_T = (*block).b_sst_first;
        while !p.is_null() {
            clear_syn_state(p);
            p = (*p).sst_next;
        }
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*block).b_sst_array as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
        (*block).b_sst_first = ::core::ptr::null_mut::<synstate_T>();
        (*block).b_sst_len = 0 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn syn_stack_free_all(mut block: *mut synblock_T) {
    unsafe {
        syn_stack_free_block(block);
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_s == block && foldmethodIsSyntax(wp) as ::core::ffi::c_int != 0 {
                foldUpdateAll(wp);
            }
            wp = (*wp).w_next;
        }
    }
}

pub(crate) unsafe extern "C" fn syn_stack_alloc() {
    unsafe {
        let mut len: ::core::ffi::c_int = (*syn_buf.get()).b_ml.ml_line_count as ::core::ffi::c_int
            / SST_DIST
            + Rows.get() * 2 as ::core::ffi::c_int;
        if len < SST_MIN_ENTRIES {
            len = SST_MIN_ENTRIES;
        } else if len > SST_MAX_ENTRIES {
            len = SST_MAX_ENTRIES;
        }
        if (*syn_block.get()).b_sst_len > len * 2 as ::core::ffi::c_int
            || (*syn_block.get()).b_sst_len < len
        {
            len = (*syn_buf.get()).b_ml.ml_line_count as ::core::ffi::c_int;
            len = (len + len / 2 as ::core::ffi::c_int) / SST_DIST
                + Rows.get() * 2 as ::core::ffi::c_int;
            if len < SST_MIN_ENTRIES {
                len = SST_MIN_ENTRIES;
            } else if len > SST_MAX_ENTRIES {
                len = SST_MAX_ENTRIES;
            }
            if !(*syn_block.get()).b_sst_array.is_null() {
                while (*syn_block.get()).b_sst_len - (*syn_block.get()).b_sst_freecount
                    + 2 as ::core::ffi::c_int
                    > len
                    && syn_stack_cleanup() as ::core::ffi::c_int != 0
                {}
                if len
                    < (*syn_block.get()).b_sst_len - (*syn_block.get()).b_sst_freecount
                        + 2 as ::core::ffi::c_int
                {
                    len = (*syn_block.get()).b_sst_len - (*syn_block.get()).b_sst_freecount
                        + 2 as ::core::ffi::c_int;
                }
            }
            '_c2rust_label: {
                if len >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"len >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/syntax.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        926 as ::core::ffi::c_uint,
                        b"void syn_stack_alloc(void)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut sstp: *mut synstate_T =
                xcalloc(len as size_t, ::core::mem::size_of::<synstate_T>()) as *mut synstate_T;
            let mut to: *mut synstate_T = sstp.offset(-(1 as ::core::ffi::c_int as isize));
            if !(*syn_block.get()).b_sst_array.is_null() {
                let mut from: *mut synstate_T = (*syn_block.get()).b_sst_first;
                while !from.is_null() {
                    to = to.offset(1);
                    *to = *from;
                    (*to).sst_next = to.offset(1 as ::core::ffi::c_int as isize);
                    from = (*from).sst_next;
                }
            }
            if to != sstp.offset(-(1 as ::core::ffi::c_int as isize)) {
                (*to).sst_next = ::core::ptr::null_mut::<synstate_T>();
                (*syn_block.get()).b_sst_first = sstp;
                (*syn_block.get()).b_sst_freecount =
                    len - to.offset_from(sstp) as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
            } else {
                (*syn_block.get()).b_sst_first = ::core::ptr::null_mut::<synstate_T>();
                (*syn_block.get()).b_sst_freecount = len;
            }
            (*syn_block.get()).b_sst_firstfree = to.offset(1 as ::core::ffi::c_int as isize);
            loop {
                to = to.offset(1);
                if to >= sstp.offset(len as isize) {
                    break;
                }
                (*to).sst_next = to.offset(1 as ::core::ffi::c_int as isize);
            }
            (*sstp
                .offset(len as isize)
                .offset(-(1 as ::core::ffi::c_int as isize)))
            .sst_next = ::core::ptr::null_mut::<synstate_T>();
            xfree((*syn_block.get()).b_sst_array as *mut ::core::ffi::c_void);
            (*syn_block.get()).b_sst_array = sstp;
            (*syn_block.get()).b_sst_len = len;
        }
    }
}

pub unsafe extern "C" fn syn_stack_apply_changes(mut buf: *mut buf_T) {
    unsafe {
        syn_stack_apply_changes_block(&raw mut (*buf).b_s, buf);
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf && (*wp).w_s != &raw mut (*buf).b_s {
                syn_stack_apply_changes_block((*wp).w_s, buf);
            }
            wp = (*wp).w_next;
        }
    }
}

pub(crate) unsafe extern "C" fn syn_stack_apply_changes_block(
    mut block: *mut synblock_T,
    mut buf: *mut buf_T,
) {
    unsafe {
        let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
        let mut p: *mut synstate_T = (*block).b_sst_first;
        while !p.is_null() {
            if (*p).sst_lnum + (*block).b_syn_sync_linebreaks > (*buf).b_mod_top {
                let mut n: linenr_T = (*p).sst_lnum + (*buf).b_mod_xlines;
                if n <= (*buf).b_mod_bot {
                    let mut np: *mut synstate_T = (*p).sst_next;
                    if prev.is_null() {
                        (*block).b_sst_first = np;
                    } else {
                        (*prev).sst_next = np;
                    }
                    syn_stack_free_entry(block, p);
                    p = np;
                    continue;
                } else {
                    if (*p).sst_change_lnum != 0 as linenr_T
                        && (*p).sst_change_lnum > (*buf).b_mod_top
                    {
                        if (*p).sst_change_lnum + (*buf).b_mod_xlines > (*buf).b_mod_top {
                            (*p).sst_change_lnum += (*buf).b_mod_xlines;
                        } else {
                            (*p).sst_change_lnum = (*buf).b_mod_top;
                        }
                    }
                    if (*p).sst_change_lnum == 0 as linenr_T
                        || (*p).sst_change_lnum < (*buf).b_mod_bot
                    {
                        (*p).sst_change_lnum = (*buf).b_mod_bot;
                    }
                    (*p).sst_lnum = n;
                }
            }
            prev = p;
            p = (*p).sst_next;
        }
    }
}

pub(crate) unsafe extern "C" fn syn_stack_cleanup() -> bool {
    unsafe {
        let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
        let mut tick: disptick_T = 0;
        let mut dist: ::core::ffi::c_int = 0;
        let mut retval: bool = false_0 != 0;
        if (*syn_block.get()).b_sst_first.is_null() {
            return retval;
        }
        if (*syn_block.get()).b_sst_len <= Rows.get() {
            dist = 999999 as ::core::ffi::c_int;
        } else {
            dist = ((*syn_buf.get()).b_ml.ml_line_count
                / ((*syn_block.get()).b_sst_len as linenr_T - Rows.get() as linenr_T)
                + 1 as linenr_T) as ::core::ffi::c_int;
        }
        tick = (*syn_block.get()).b_sst_lasttick;
        let mut above: bool = false_0 != 0;
        prev = (*syn_block.get()).b_sst_first;
        let mut p: *mut synstate_T = (*prev).sst_next;
        while !p.is_null() {
            if (*prev).sst_lnum + dist as linenr_T > (*p).sst_lnum {
                if (*p).sst_tick > (*syn_block.get()).b_sst_lasttick {
                    if !above || (*p).sst_tick < tick {
                        tick = (*p).sst_tick;
                    }
                    above = true_0 != 0;
                } else if !above && (*p).sst_tick < tick {
                    tick = (*p).sst_tick;
                }
            }
            prev = p;
            p = (*p).sst_next;
        }
        prev = (*syn_block.get()).b_sst_first;
        let mut p_0: *mut synstate_T = (*prev).sst_next;
        while !p_0.is_null() {
            if (*p_0).sst_tick == tick && (*prev).sst_lnum + dist as linenr_T > (*p_0).sst_lnum {
                (*prev).sst_next = (*p_0).sst_next;
                syn_stack_free_entry(syn_block.get(), p_0);
                p_0 = prev;
                retval = true_0 != 0;
            }
            prev = p_0;
            p_0 = (*p_0).sst_next;
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn syn_stack_free_entry(
    mut block: *mut synblock_T,
    mut p: *mut synstate_T,
) {
    unsafe {
        clear_syn_state(p);
        (*p).sst_next = (*block).b_sst_firstfree;
        (*block).b_sst_firstfree = p;
        (*block).b_sst_freecount += 1;
    }
}

pub(crate) unsafe extern "C" fn syn_stack_find_entry(mut lnum: linenr_T) -> *mut synstate_T {
    unsafe {
        let mut prev: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
        let mut p: *mut synstate_T = (*syn_block.get()).b_sst_first;
        while !p.is_null() {
            if (*p).sst_lnum == lnum {
                return p;
            }
            if (*p).sst_lnum > lnum {
                break;
            }
            prev = p;
            p = (*p).sst_next;
        }
        return prev;
    }
}

pub(crate) unsafe extern "C" fn store_current_state() -> *mut synstate_T {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        let mut p: *mut synstate_T = ::core::ptr::null_mut::<synstate_T>();
        let mut bp: *mut bufstate_T = ::core::ptr::null_mut::<bufstate_T>();
        let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
        let mut sp: *mut synstate_T = syn_stack_find_entry(current_lnum.get());
        i = (*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize);
            if (*cur_si).si_h_startpos.lnum >= current_lnum.get()
                || (*cur_si).si_m_endpos.lnum >= current_lnum.get()
                || (*cur_si).si_h_endpos.lnum >= current_lnum.get()
                || (*cur_si).si_end_idx != 0 && (*cur_si).si_eoe_pos.lnum >= current_lnum.get()
            {
                break;
            }
            i -= 1;
        }
        if i >= 0 as ::core::ffi::c_int {
            if !sp.is_null() {
                if (*syn_block.get()).b_sst_first == sp {
                    (*syn_block.get()).b_sst_first = (*sp).sst_next;
                } else {
                    p = (*syn_block.get()).b_sst_first;
                    while !p.is_null() {
                        if (*p).sst_next == sp {
                            break;
                        }
                        p = (*p).sst_next;
                    }
                    if !p.is_null() {
                        (*p).sst_next = (*sp).sst_next;
                    }
                }
                syn_stack_free_entry(syn_block.get(), sp);
                sp = ::core::ptr::null_mut::<synstate_T>();
            }
        } else if sp.is_null() || (*sp).sst_lnum != current_lnum.get() {
            if (*syn_block.get()).b_sst_freecount == 0 as ::core::ffi::c_int {
                syn_stack_cleanup();
                sp = syn_stack_find_entry(current_lnum.get());
            }
            if (*syn_block.get()).b_sst_freecount == 0 as ::core::ffi::c_int {
                sp = ::core::ptr::null_mut::<synstate_T>();
            } else {
                p = (*syn_block.get()).b_sst_firstfree;
                (*syn_block.get()).b_sst_firstfree = (*p).sst_next;
                (*syn_block.get()).b_sst_freecount -= 1;
                if sp.is_null() {
                    (*p).sst_next = (*syn_block.get()).b_sst_first;
                    (*syn_block.get()).b_sst_first = p;
                } else {
                    (*p).sst_next = (*sp).sst_next;
                    (*sp).sst_next = p;
                }
                sp = p;
                (*sp).sst_stacksize = 0 as ::core::ffi::c_int;
                (*sp).sst_lnum = current_lnum.get();
            }
        }
        if !sp.is_null() {
            clear_syn_state(sp);
            (*sp).sst_stacksize = (*current_state.ptr()).ga_len;
            if (*current_state.ptr()).ga_len > SST_FIX_STATES {
                ga_init(
                    &raw mut (*sp).sst_union.sst_ga,
                    ::core::mem::size_of::<bufstate_T>() as ::core::ffi::c_int,
                    1 as ::core::ffi::c_int,
                );
                ga_grow(
                    &raw mut (*sp).sst_union.sst_ga,
                    (*current_state.ptr()).ga_len,
                );
                (*sp).sst_union.sst_ga.ga_len = (*current_state.ptr()).ga_len;
                bp = (*sp).sst_union.sst_ga.ga_data as *mut bufstate_T;
            } else {
                bp = &raw mut (*sp).sst_union.sst_stack as *mut bufstate_T;
            }
            i = 0 as ::core::ffi::c_int;
            while i < (*sp).sst_stacksize {
                (*bp.offset(i as isize)).bs_idx =
                    (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                        .si_idx;
                (*bp.offset(i as isize)).bs_flags =
                    (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                        .si_flags;
                (*bp.offset(i as isize)).bs_seqnr =
                    (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                        .si_seqnr;
                (*bp.offset(i as isize)).bs_cchar =
                    (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                        .si_cchar;
                (*bp.offset(i as isize)).bs_extmatch = ref_extmatch(
                    (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                        .si_extmatch,
                );
                i += 1;
            }
            (*sp).sst_next_flags = current_next_flags.get();
            (*sp).sst_next_list = current_next_list.get();
            (*sp).sst_tick = display_tick.get();
            (*sp).sst_change_lnum = 0 as ::core::ffi::c_int as linenr_T;
        }
        current_state_stored.set(true_0 != 0);
        return sp;
    }
}

pub(crate) unsafe extern "C" fn load_current_state(mut from: *mut synstate_T) {
    unsafe {
        let mut bp: *mut bufstate_T = ::core::ptr::null_mut::<bufstate_T>();
        clear_current_state();
        validate_current_state();
        keepend_level.set(-1 as ::core::ffi::c_int);
        if (*from).sst_stacksize != 0 {
            ga_grow(current_state.ptr(), (*from).sst_stacksize);
            if (*from).sst_stacksize > SST_FIX_STATES {
                bp = (*from).sst_union.sst_ga.ga_data as *mut bufstate_T;
            } else {
                bp = &raw mut (*from).sst_union.sst_stack as *mut bufstate_T;
            }
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < (*from).sst_stacksize {
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_idx =
                    (*bp.offset(i as isize)).bs_idx;
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_flags = (*bp.offset(i as isize)).bs_flags;
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_seqnr = (*bp.offset(i as isize)).bs_seqnr;
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_cchar = (*bp.offset(i as isize)).bs_cchar;
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_extmatch = ref_extmatch((*bp.offset(i as isize)).bs_extmatch);
                if keepend_level.get() < 0 as ::core::ffi::c_int
                    && (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                        .si_flags
                        & HL_KEEPEND
                        != 0
                {
                    keepend_level.set(i);
                }
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_ends = false_0;
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_m_lnum = 0 as ::core::ffi::c_int;
                if (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_idx
                    >= 0 as ::core::ffi::c_int
                {
                    (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                        .si_next_list =
                        (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T).offset(
                            (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                                .offset(i as isize))
                            .si_idx as isize,
                        ))
                        .sp_next_list;
                } else {
                    (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                        .si_next_list = ::core::ptr::null_mut::<int16_t>();
                }
                update_si_attr(i);
                i += 1;
            }
            (*current_state.ptr()).ga_len = (*from).sst_stacksize;
        }
        current_next_list.set((*from).sst_next_list);
        current_next_flags.set((*from).sst_next_flags);
        current_lnum.set((*from).sst_lnum);
    }
}

pub(crate) unsafe extern "C" fn syn_stack_equal(mut sp: *mut synstate_T) -> bool {
    unsafe {
        let mut bp: *mut bufstate_T = ::core::ptr::null_mut::<bufstate_T>();
        if (*sp).sst_stacksize != (*current_state.ptr()).ga_len
            || (*sp).sst_next_list != current_next_list.get()
        {
            return false_0 != 0;
        }
        if (*sp).sst_stacksize > SST_FIX_STATES {
            bp = (*sp).sst_union.sst_ga.ga_data as *mut bufstate_T;
        } else {
            bp = &raw mut (*sp).sst_union.sst_stack as *mut bufstate_T;
        }
        let mut i: ::core::ffi::c_int = 0;
        i = (*current_state.ptr()).ga_len;
        loop {
            i -= 1;
            if i < 0 as ::core::ffi::c_int {
                break;
            }
            if (*bp.offset(i as isize)).bs_idx
                != (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_idx
            {
                break;
            }
            if (*bp.offset(i as isize)).bs_extmatch
                == (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_extmatch
            {
                continue;
            }
            let mut bsx: *mut reg_extmatch_T = (*bp.offset(i as isize)).bs_extmatch;
            let mut six: *mut reg_extmatch_T =
                (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize))
                    .si_extmatch;
            if bsx.is_null() || six.is_null() {
                break;
            }
            let mut j: ::core::ffi::c_int = 0;
            j = 0 as ::core::ffi::c_int;
            while j < NSUBEXP as ::core::ffi::c_int {
                if (*bsx).matches[j as usize] != (*six).matches[j as usize] {
                    if (*bsx).matches[j as usize].is_null() || (*six).matches[j as usize].is_null()
                    {
                        break;
                    }
                    if mb_strcmp_ic(
                        (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T).offset(
                            (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                                .offset(i as isize))
                            .si_idx as isize,
                        ))
                        .sp_ic
                            != 0,
                        (*bsx).matches[j as usize] as *const ::core::ffi::c_char,
                        (*six).matches[j as usize] as *const ::core::ffi::c_char,
                    ) != 0 as ::core::ffi::c_int
                    {
                        break;
                    }
                }
                j += 1;
            }
            if j != NSUBEXP as ::core::ffi::c_int {
                break;
            }
        }
        return if i < 0 as ::core::ffi::c_int {
            true_0
        } else {
            false_0
        } != 0;
    }
}
