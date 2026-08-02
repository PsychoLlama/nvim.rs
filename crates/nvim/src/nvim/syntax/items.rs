//! The state stack's item operations.
//!
//! The stack is a `garray_T` of `stateitem_T`, innermost last. These are the
//! operations on it: pushing what a match found ([`push_next_match`]), working
//! out an item's highlight attributes and containment ([`update_si_attr`]) and
//! where it ends ([`update_si_end`]), applying `keepend`/`extend`
//! ([`check_keepend`]), and popping items whose end the driver has reached
//! ([`check_state_ends`]).

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn push_next_match() -> *mut stateitem_T {
    unsafe {
        let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
        let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
        let mut save_flags: ::core::ffi::c_int = 0;
        spp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
            .offset(next_match_idx.get() as isize);
        push_current_state(next_match_idx.get());
        cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
            .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
        (*cur_si).si_h_startpos = next_match_h_startpos.get();
        (*cur_si).si_m_startcol = current_col.get() as ::core::ffi::c_int;
        (*cur_si).si_m_lnum = current_lnum.get() as ::core::ffi::c_int;
        (*cur_si).si_flags = (*spp).sp_flags;
        let c2rust_fresh4 = next_seqnr.get();
        next_seqnr.set(next_seqnr.get() + 1);
        (*cur_si).si_seqnr = c2rust_fresh4;
        (*cur_si).si_cchar = (*spp).sp_cchar;
        if (*current_state.ptr()).ga_len > 1 as ::core::ffi::c_int {
            (*cur_si).si_flags |= (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                .offset(((*current_state.ptr()).ga_len - 2 as ::core::ffi::c_int) as isize))
            .si_flags
                & HL_CONCEAL;
        }
        (*cur_si).si_next_list = (*spp).sp_next_list;
        (*cur_si).si_extmatch = ref_extmatch(next_match_extmatch.get());
        if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START && (*spp).sp_flags & HL_ONELINE == 0
        {
            update_si_end(cur_si, (*next_match_m_endpos.ptr()).col, true_0 != 0);
            check_keepend();
        } else {
            (*cur_si).si_m_endpos = next_match_m_endpos.get();
            (*cur_si).si_h_endpos = next_match_h_endpos.get();
            (*cur_si).si_ends = true_0;
            (*cur_si).si_flags |= next_match_flags.get();
            (*cur_si).si_eoe_pos = next_match_eoe_pos.get();
            (*cur_si).si_end_idx = next_match_end_idx.get();
        }
        if keepend_level.get() < 0 as ::core::ffi::c_int && (*cur_si).si_flags & HL_KEEPEND != 0 {
            keepend_level.set((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
        }
        check_keepend();
        update_si_attr((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
        save_flags = (*cur_si).si_flags & (HL_CONCEAL | HL_CONCEALENDS);
        if (*spp).sp_type as ::core::ffi::c_int == SPTYPE_START
            && (*spp).sp_syn_match_id as ::core::ffi::c_int != 0 as ::core::ffi::c_int
        {
            push_current_state(next_match_idx.get());
            cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
                .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
            (*cur_si).si_h_startpos = next_match_h_startpos.get();
            (*cur_si).si_m_startcol = current_col.get() as ::core::ffi::c_int;
            (*cur_si).si_m_lnum = current_lnum.get() as ::core::ffi::c_int;
            (*cur_si).si_m_endpos = next_match_eos_pos.get();
            (*cur_si).si_h_endpos = next_match_eos_pos.get();
            (*cur_si).si_ends = true_0;
            (*cur_si).si_end_idx = 0 as ::core::ffi::c_int;
            (*cur_si).si_flags = HL_MATCH;
            let c2rust_fresh5 = next_seqnr.get();
            next_seqnr.set(next_seqnr.get() + 1);
            (*cur_si).si_seqnr = c2rust_fresh5;
            (*cur_si).si_flags |= save_flags;
            if (*cur_si).si_flags & HL_CONCEALENDS != 0 {
                (*cur_si).si_flags |= HL_CONCEAL;
            }
            (*cur_si).si_next_list = ::core::ptr::null_mut::<int16_t>();
            check_keepend();
            update_si_attr((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
        }
        next_match_idx.set(-1 as ::core::ffi::c_int);
        return cur_si;
    }
}

pub(crate) unsafe extern "C" fn check_state_ends() {
    unsafe {
        let mut cur_si: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
        let mut had_extend: ::core::ffi::c_int = 0;
        cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
            .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
        while (*cur_si).si_ends != 0
            && ((*cur_si).si_m_endpos.lnum < current_lnum.get()
                || (*cur_si).si_m_endpos.lnum == current_lnum.get()
                    && (*cur_si).si_m_endpos.col <= current_col.get())
        {
            if (*cur_si).si_end_idx != 0
                && ((*cur_si).si_eoe_pos.lnum > current_lnum.get()
                    || (*cur_si).si_eoe_pos.lnum == current_lnum.get()
                        && (*cur_si).si_eoe_pos.col > current_col.get())
            {
                (*cur_si).si_idx = (*cur_si).si_end_idx;
                (*cur_si).si_end_idx = 0 as ::core::ffi::c_int;
                (*cur_si).si_m_endpos = (*cur_si).si_eoe_pos;
                (*cur_si).si_h_endpos = (*cur_si).si_eoe_pos;
                (*cur_si).si_flags |= HL_MATCH;
                let c2rust_fresh0 = next_seqnr.get();
                next_seqnr.set(next_seqnr.get() + 1);
                (*cur_si).si_seqnr = c2rust_fresh0;
                if (*cur_si).si_flags & HL_CONCEALENDS != 0 {
                    (*cur_si).si_flags |= HL_CONCEAL;
                }
                update_si_attr((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int);
                current_next_list.set(::core::ptr::null_mut::<int16_t>());
                next_match_idx.set(0 as ::core::ffi::c_int);
                next_match_col.set(MAXCOL as ::core::ffi::c_int);
                break;
            } else {
                current_next_list.set((*cur_si).si_next_list);
                current_next_flags.set((*cur_si).si_flags);
                if current_next_flags.get() & (HL_SKIPNL | HL_SKIPEMPTY) == 0
                    && *syn_getcurline().offset(current_col.get() as isize) as ::core::ffi::c_int
                        == NUL
                {
                    current_next_list.set(::core::ptr::null_mut::<int16_t>());
                }
                had_extend = (*cur_si).si_flags & HL_EXTEND;
                pop_current_state();
                if (*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int {
                    break;
                }
                if had_extend != 0 && keepend_level.get() >= 0 as ::core::ffi::c_int {
                    syn_update_ends(false_0 != 0);
                    if (*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int {
                        break;
                    }
                }
                cur_si = ((*current_state.ptr()).ga_data as *mut stateitem_T)
                    .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize);
                if !((*cur_si).si_idx >= 0 as ::core::ffi::c_int
                    && (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                        .offset((*cur_si).si_idx as isize))
                    .sp_type as ::core::ffi::c_int
                        == SPTYPE_START
                    && (*cur_si).si_flags & (HL_MATCH | HL_KEEPEND) == 0)
                {
                    continue;
                }
                update_si_end(cur_si, current_col.get(), true_0 != 0);
                check_keepend();
                if current_next_flags.get() & HL_HAS_EOL != 0
                    && keepend_level.get() < 0 as ::core::ffi::c_int
                    && *syn_getcurline().offset(current_col.get() as isize) as ::core::ffi::c_int
                        == NUL
                {
                    break;
                }
            }
        }
    }
}

pub(crate) unsafe extern "C" fn update_si_attr(mut idx: ::core::ffi::c_int) {
    unsafe {
        let mut sip: *mut stateitem_T =
            ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(idx as isize);
        let mut spp: *mut synpat_T = ::core::ptr::null_mut::<synpat_T>();
        if (*sip).si_idx < 0 as ::core::ffi::c_int {
            return;
        }
        spp = ((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
            .offset((*sip).si_idx as isize);
        if (*sip).si_flags & HL_MATCH != 0 {
            (*sip).si_id = (*spp).sp_syn_match_id as ::core::ffi::c_int;
        } else {
            (*sip).si_id = (*spp).sp_syn.id as ::core::ffi::c_int;
        }
        (*sip).si_attr = syn_id2attr((*sip).si_id);
        (*sip).si_trans_id = (*sip).si_id;
        if (*sip).si_flags & HL_MATCH != 0 {
            (*sip).si_cont_list = ::core::ptr::null_mut::<int16_t>();
        } else {
            (*sip).si_cont_list = (*spp).sp_cont_list;
        }
        if (*spp).sp_flags & HL_TRANSP != 0 && (*sip).si_flags & HL_MATCH == 0 {
            if idx == 0 as ::core::ffi::c_int {
                (*sip).si_attr = 0 as ::core::ffi::c_int;
                (*sip).si_trans_id = 0 as ::core::ffi::c_int;
                if (*sip).si_cont_list.is_null() {
                    (*sip).si_cont_list = ID_LIST_ALL;
                }
            } else {
                (*sip).si_attr = (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                    .offset((idx - 1 as ::core::ffi::c_int) as isize))
                .si_attr;
                (*sip).si_trans_id = (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                    .offset((idx - 1 as ::core::ffi::c_int) as isize))
                .si_trans_id;
                if (*sip).si_cont_list.is_null() {
                    (*sip).si_flags |= HL_TRANS_CONT;
                    (*sip).si_cont_list = (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                        .offset((idx - 1 as ::core::ffi::c_int) as isize))
                    .si_cont_list;
                }
            }
        }
    }
}

pub(crate) unsafe extern "C" fn check_keepend() {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        let mut maxpos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut maxpos_h: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut sip: *mut stateitem_T = ::core::ptr::null_mut::<stateitem_T>();
        if keepend_level.get() < 0 as ::core::ffi::c_int {
            return;
        }
        i = (*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int;
        while i > keepend_level.get() {
            if (*((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize)).si_flags
                & HL_EXTEND
                != 0
            {
                break;
            }
            i -= 1;
        }
        maxpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
        maxpos.col = 0 as ::core::ffi::c_int as colnr_T;
        maxpos_h.lnum = 0 as ::core::ffi::c_int as linenr_T;
        maxpos_h.col = 0 as ::core::ffi::c_int as colnr_T;
        while i < (*current_state.ptr()).ga_len {
            sip = ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize);
            if maxpos.lnum != 0 as linenr_T {
                limit_pos_zero(&raw mut (*sip).si_m_endpos, &raw mut maxpos);
                limit_pos_zero(&raw mut (*sip).si_h_endpos, &raw mut maxpos_h);
                limit_pos_zero(&raw mut (*sip).si_eoe_pos, &raw mut maxpos);
                (*sip).si_ends = true_0;
            }
            if (*sip).si_ends != 0 && (*sip).si_flags & HL_KEEPEND != 0 {
                if maxpos.lnum == 0 as linenr_T
                    || maxpos.lnum > (*sip).si_m_endpos.lnum
                    || maxpos.lnum == (*sip).si_m_endpos.lnum && maxpos.col > (*sip).si_m_endpos.col
                {
                    maxpos = (*sip).si_m_endpos;
                }
                if maxpos_h.lnum == 0 as linenr_T
                    || maxpos_h.lnum > (*sip).si_h_endpos.lnum
                    || maxpos_h.lnum == (*sip).si_h_endpos.lnum
                        && maxpos_h.col > (*sip).si_h_endpos.col
                {
                    maxpos_h = (*sip).si_h_endpos;
                }
            }
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn update_si_end(
    mut sip: *mut stateitem_T,
    mut startcol: ::core::ffi::c_int,
    mut force: bool,
) {
    unsafe {
        let mut hl_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
        let mut end_endpos: lpos_T = lpos_T { lnum: 0, col: 0 };
        if (*sip).si_idx < 0 as ::core::ffi::c_int {
            return;
        }
        if !force && (*sip).si_m_endpos.lnum >= current_lnum.get() {
            return;
        }
        let mut end_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut startpos: lpos_T = lpos_T {
            lnum: current_lnum.get(),
            col: startcol as colnr_T,
        };
        let mut endpos: lpos_T = lpos_T {
            lnum: 0 as linenr_T,
            col: 0,
        };
        find_endpos(
            (*sip).si_idx,
            &raw mut startpos,
            &raw mut endpos,
            &raw mut hl_endpos,
            &raw mut (*sip).si_flags,
            &raw mut end_endpos,
            &raw mut end_idx,
            (*sip).si_extmatch,
        );
        if endpos.lnum == 0 as linenr_T {
            if (*((*syn_block.get()).b_syn_patterns.ga_data as *mut synpat_T)
                .offset((*sip).si_idx as isize))
            .sp_flags
                & HL_ONELINE
                != 0
            {
                (*sip).si_ends = true_0;
                (*sip).si_m_endpos.lnum = current_lnum.get();
                (*sip).si_m_endpos.col = syn_getcurline_len();
            } else {
                (*sip).si_ends = false_0;
                (*sip).si_m_endpos.lnum = 0 as ::core::ffi::c_int as linenr_T;
            }
            (*sip).si_h_endpos = (*sip).si_m_endpos;
        } else {
            (*sip).si_m_endpos = endpos;
            (*sip).si_h_endpos = hl_endpos;
            (*sip).si_eoe_pos = end_endpos;
            (*sip).si_ends = true_0;
            (*sip).si_end_idx = end_idx;
        };
    }
}

pub(crate) unsafe extern "C" fn push_current_state(mut idx: ::core::ffi::c_int) {
    unsafe {
        let mut p: *mut stateitem_T =
            ga_append_via_ptr(current_state.ptr(), ::core::mem::size_of::<stateitem_T>())
                as *mut stateitem_T;
        memset(
            p as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<stateitem_T>(),
        );
        (*p).si_idx = idx;
    }
}

pub(crate) unsafe extern "C" fn pop_current_state() {
    unsafe {
        if !((*current_state.ptr()).ga_len <= 0 as ::core::ffi::c_int) {
            unref_extmatch(
                (*((*current_state.ptr()).ga_data as *mut stateitem_T)
                    .offset(((*current_state.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .si_extmatch,
            );
            (*current_state.ptr()).ga_len -= 1;
        }
        next_match_idx.set(-1 as ::core::ffi::c_int);
        if keepend_level.get() >= (*current_state.ptr()).ga_len {
            keepend_level.set(-1 as ::core::ffi::c_int);
        }
    }
}
