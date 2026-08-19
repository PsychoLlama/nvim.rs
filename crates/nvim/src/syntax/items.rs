//! The state stack's item operations.
//!
//! The stack is a `garray_T` of `stateitem_T`, innermost last. These are the
//! operations on it: pushing what a match found ([`push_next_match`]), working
//! out an item's highlight attributes and containment ([`update_si_attr`]) and
//! where it ends ([`update_si_end`]), applying `keepend`/`extend`
//! ([`check_keepend`]), and popping items whose end the driver has reached
//! ([`check_state_ends`]).

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;
use crate::pos::MAXCOL;
use crate::types::NUL;

/// Number of items on the current state stack.
#[inline(always)]
pub(crate) unsafe fn state_len() -> c_int {
    unsafe { (*current_state.ptr()).ga_len }
}

/// The item at `i` on the current state stack (0 is the outermost).
///
/// A pointer and not a borrow: the array is a `garray_T` that any push can
/// reallocate, and several callers address two items at once.
#[inline(always)]
pub(crate) unsafe fn state_at(i: c_int) -> *mut stateitem_T {
    unsafe { ((*current_state.ptr()).ga_data as *mut stateitem_T).offset(i as isize) }
}

/// The innermost item on the current state stack. Only valid when
/// [`state_len`] is positive.
#[inline(always)]
pub(crate) unsafe fn state_top() -> *mut stateitem_T {
    unsafe { state_at(state_len() - 1) }
}

/// Push what `next_match_*` found onto the state stack.
///
/// Answers the item now on top, which is the `matchgroup=` item for a region's
/// start pattern when there is one and the region itself otherwise.
pub(crate) unsafe fn push_next_match() -> *mut stateitem_T {
    unsafe {
        let idx = next_match_idx.get();
        let spp = syn_pattern(idx);

        push_current_state(idx);
        let mut cur_si = state_top();
        (*cur_si).si_h_startpos = next_match_h_startpos.get();
        (*cur_si).si_m_startcol = current_col.get();
        (*cur_si).si_m_lnum = current_lnum.get();
        (*cur_si).si_flags = (*spp).sp_flags;
        (*cur_si).si_seqnr = take_seqnr();
        (*cur_si).si_cchar = (*spp).sp_cchar;
        if state_len() > 1 {
            // A concealed item conceals what it contains.
            (*cur_si).si_flags |= (*state_at(state_len() - 2))
                .si_flags
                .masked(SynFlags::CONCEAL);
        }
        (*cur_si).si_next_list = (*spp).sp_next_list;
        (*cur_si).si_extmatch = ref_extmatch(next_match_extmatch.get());

        if (*spp).sp_type as c_int == SPTYPE_START && !(*spp).sp_flags.has(SynFlags::ONELINE) {
            // A start-skip-end that may cross lines: work out how much of it
            // is in this line.
            update_si_end(cur_si, next_match_m_endpos.get().col, true);
            check_keepend();
        } else {
            (*cur_si).si_m_endpos = next_match_m_endpos.get();
            (*cur_si).si_h_endpos = next_match_h_endpos.get();
            (*cur_si).si_ends = true_0;
            (*cur_si).si_flags |= next_match_flags.get();
            (*cur_si).si_eoe_pos = next_match_eoe_pos.get();
            (*cur_si).si_end_idx = next_match_end_idx.get();
        }
        if keepend_level.get() < 0 && (*cur_si).si_flags.has(SynFlags::KEEPEND) {
            keepend_level.set(state_len() - 1);
        }
        check_keepend();
        update_si_attr(state_len() - 1);

        let save_flags = (*cur_si)
            .si_flags
            .masked(SynFlags::CONCEAL | SynFlags::CONCEALENDS);

        // If the start pattern has a `matchgroup=` of its own, push a second
        // item for it, ending where the start match ends.
        if (*spp).sp_type as c_int == SPTYPE_START && (*spp).sp_syn_match_id != 0 {
            push_current_state(idx);
            cur_si = state_top();
            (*cur_si).si_h_startpos = next_match_h_startpos.get();
            (*cur_si).si_m_startcol = current_col.get();
            (*cur_si).si_m_lnum = current_lnum.get();
            (*cur_si).si_m_endpos = next_match_eos_pos.get();
            (*cur_si).si_h_endpos = next_match_eos_pos.get();
            (*cur_si).si_ends = true_0;
            (*cur_si).si_end_idx = 0;
            (*cur_si).si_flags = SynFlags::MATCH | save_flags;
            (*cur_si).si_seqnr = take_seqnr();
            if (*cur_si).si_flags.has(SynFlags::CONCEALENDS) {
                (*cur_si).si_flags |= SynFlags::CONCEAL;
            }
            (*cur_si).si_next_list = ::core::ptr::null_mut();
            check_keepend();
            update_si_attr(state_len() - 1);
        }

        next_match_idx.set(-1); // try another match next time
        cur_si
    }
}

/// The next item sequence number, post-incrementing the counter.
///
/// `si_seqnr` orders items that begin at the same column; `synstack()` reports
/// it and the state-stack equality test compares it.
#[inline]
fn take_seqnr() -> c_int {
    let n = next_seqnr.get();
    next_seqnr.set(n + 1);
    n
}

/// Pop every item on the stack whose end the driver has now reached.
pub(crate) unsafe fn check_state_ends() {
    unsafe {
        let mut cur_si = state_top();
        loop {
            if (*cur_si).si_ends == 0
                || (*cur_si).si_m_endpos.lnum > current_lnum.get()
                || ((*cur_si).si_m_endpos.lnum == current_lnum.get()
                    && (*cur_si).si_m_endpos.col > current_col.get())
            {
                return;
            }

            // If the end pattern has a highlight group of its own and it
            // continues beyond this position, highlight it now. The item stays
            // on the stack, standing in for the end match.
            if (*cur_si).si_end_idx != 0
                && ((*cur_si).si_eoe_pos.lnum > current_lnum.get()
                    || ((*cur_si).si_eoe_pos.lnum == current_lnum.get()
                        && (*cur_si).si_eoe_pos.col > current_col.get()))
            {
                (*cur_si).si_idx = (*cur_si).si_end_idx;
                (*cur_si).si_end_idx = 0;
                (*cur_si).si_m_endpos = (*cur_si).si_eoe_pos;
                (*cur_si).si_h_endpos = (*cur_si).si_eoe_pos;
                (*cur_si).si_flags |= SynFlags::MATCH;
                (*cur_si).si_seqnr = take_seqnr();
                if (*cur_si).si_flags.has(SynFlags::CONCEALENDS) {
                    (*cur_si).si_flags |= SynFlags::CONCEAL;
                }
                update_si_attr(state_len() - 1);

                // `nextgroup=` should not match in the end pattern, and what
                // matches next may be different now.
                current_next_list.set(::core::ptr::null_mut());
                next_match_idx.set(0);
                next_match_col.set(MAXCOL as c_int);
                return;
            }

            // Hand the ended item's `nextgroup=` to the driver, unless we are
            // at end of line and it has neither "skipnl" nor "skipempty".
            current_next_list.set((*cur_si).si_next_list);
            current_next_flags.set((*cur_si).si_flags);
            if !current_next_flags
                .get()
                .has(SynFlags::SKIPNL | SynFlags::SKIPEMPTY)
                && *syn_getcurline().offset(current_col.get() as isize) as c_int == NUL
            {
                current_next_list.set(::core::ptr::null_mut());
            }

            // When the ended item has "extend", another item with "keepend"
            // now needs to check for its end.
            let had_extend = (*cur_si).si_flags.has(SynFlags::EXTEND);

            pop_current_state();
            if state_len() <= 0 {
                return;
            }
            if had_extend && keepend_level.get() >= 0 {
                syn_update_ends(false);
                if state_len() <= 0 {
                    return;
                }
            }
            cur_si = state_top();

            // Only for a region does the search for the end continue after the
            // end of the contained item. If the contained match included the
            // end of the line, stop here and let the region continue. Not when
            // "keepend" is used for the contained item, not when we are away
            // from the end of the line (the end could be `end="x$"me=e-1`), and
            // not when "excludenl" is used (SynFlags::HAS_EOL will not be set).
            if (*cur_si).si_idx >= 0
                && (*syn_pattern((*cur_si).si_idx)).sp_type as c_int == SPTYPE_START
                && !(*cur_si).si_flags.has(SynFlags::MATCH | SynFlags::KEEPEND)
            {
                update_si_end(cur_si, current_col.get(), true);
                check_keepend();
                if current_next_flags.get().has(SynFlags::HAS_EOL)
                    && keepend_level.get() < 0
                    && *syn_getcurline().offset(current_col.get() as isize) as c_int == NUL
                {
                    return;
                }
            }
        }
    }
}

/// Fill in `si_id`, `si_attr`, `si_trans_id` and `si_cont_list` for the item at
/// `idx`, from the pattern it came from.
pub(crate) unsafe fn update_si_attr(idx: c_int) {
    unsafe {
        let sip = state_at(idx);
        if (*sip).si_idx < 0 {
            return; // a keyword; should not happen
        }
        let spp = syn_pattern((*sip).si_idx);
        let is_match = (*sip).si_flags.has(SynFlags::MATCH);

        (*sip).si_id = if is_match {
            (*spp).sp_syn_match_id as c_int
        } else {
            (*spp).sp_syn.id as c_int
        };
        (*sip).si_attr = syn_id2attr((*sip).si_id);
        (*sip).si_trans_id = (*sip).si_id;
        (*sip).si_cont_list = if is_match {
            ::core::ptr::null_mut()
        } else {
            (*spp).sp_cont_list
        };

        // A transparent item takes its attributes from the item around it, and
        // its containment too when it has none of its own. Not for the
        // matchgroup of a start or end pattern.
        if !(*spp).sp_flags.has(SynFlags::TRANSP) || is_match {
            return;
        }
        if idx == 0 {
            (*sip).si_attr = 0;
            (*sip).si_trans_id = 0;
            if (*sip).si_cont_list.is_null() {
                (*sip).si_cont_list = ID_LIST_ALL;
            }
        } else {
            let outer = state_at(idx - 1);
            (*sip).si_attr = (*outer).si_attr;
            (*sip).si_trans_id = (*outer).si_trans_id;
            if (*sip).si_cont_list.is_null() {
                (*sip).si_flags |= SynFlags::TRANS_CONT;
                (*sip).si_cont_list = (*outer).si_cont_list;
            }
        }
    }
}

/// Propagate the end of every "keepend" item on the stack to the items it
/// contains, so none of them can reach past it.
pub(crate) unsafe fn check_keepend() {
    unsafe {
        // This check can consume a lot of time; only do it from the level
        // where there really is a keepend.
        if keepend_level.get() < 0 {
            return;
        }

        // Find the innermost "extend" item: "keepend" items outside it do
        // nothing. With no "extend" item this stops at `keepend_level` and
        // every "keepend" works normally.
        let mut i = state_len() - 1;
        while i > keepend_level.get() {
            if (*state_at(i)).si_flags.has(SynFlags::EXTEND) {
                break;
            }
            i -= 1;
        }

        let mut maxpos = lpos_T { lnum: 0, col: 0 };
        let mut maxpos_h = lpos_T { lnum: 0, col: 0 };
        while i < state_len() {
            let sip = state_at(i);
            if maxpos.lnum != 0 {
                limit_pos_zero(&mut (*sip).si_m_endpos, maxpos);
                limit_pos_zero(&mut (*sip).si_h_endpos, maxpos_h);
                limit_pos_zero(&mut (*sip).si_eoe_pos, maxpos);
                (*sip).si_ends = true_0;
            }
            if (*sip).si_ends != 0 && (*sip).si_flags.has(SynFlags::KEEPEND) {
                if maxpos.lnum == 0 || pos_after(maxpos, (*sip).si_m_endpos) {
                    maxpos = (*sip).si_m_endpos;
                }
                if maxpos_h.lnum == 0 || pos_after(maxpos_h, (*sip).si_h_endpos) {
                    maxpos_h = (*sip).si_h_endpos;
                }
            }
            i += 1;
        }
    }
}

/// Is `a` strictly after `b`?
#[inline]
fn pos_after(a: lpos_T, b: lpos_T) -> bool {
    a.lnum > b.lnum || (a.lnum == b.lnum && a.col > b.col)
}

/// Find where the start-skip-end item `sip` ends, if it ends in this line.
///
/// `startcol` is where to start looking; `force` overrules an end the item
/// already has.
pub(crate) unsafe fn update_si_end(sip: *mut stateitem_T, startcol: c_int, force: bool) {
    unsafe {
        if (*sip).si_idx < 0 {
            return; // a keyword has no end pattern
        }
        // Don't update when it is already done. Can be a match of an end
        // pattern that started in a previous line -- but watch out, it can also
        // be a "keepend" from a containing item.
        if !force && (*sip).si_m_endpos.lnum >= current_lnum.get() {
            return;
        }

        let startpos = lpos_T {
            lnum: current_lnum.get(),
            col: startcol as colnr_T,
        };
        let end = find_endpos((*sip).si_idx, startpos, (*sip).si_extmatch);
        if let Some(flags) = end.flags {
            (*sip).si_flags = flags;
        }

        if end.m_endpos.lnum == 0 {
            // No end pattern matched.
            if (*syn_pattern((*sip).si_idx))
                .sp_flags
                .has(SynFlags::ONELINE)
            {
                // A "oneline" never continues in the next line.
                (*sip).si_ends = true_0;
                (*sip).si_m_endpos.lnum = current_lnum.get();
                (*sip).si_m_endpos.col = syn_getcurline_len();
            } else {
                (*sip).si_ends = false_0;
                (*sip).si_m_endpos.lnum = 0;
            }
            (*sip).si_h_endpos = (*sip).si_m_endpos;
        } else {
            (*sip).si_m_endpos = end.m_endpos;
            (*sip).si_h_endpos = end.hl_endpos;
            (*sip).si_eoe_pos = end.eoe_pos;
            (*sip).si_ends = true_0;
            (*sip).si_end_idx = end.end_idx;
        }
    }
}

/// Push a cleared item for pattern `idx` onto the state stack.
pub(crate) unsafe fn push_current_state(idx: c_int) {
    unsafe {
        let p = ga_append_via_ptr(current_state.ptr(), ::core::mem::size_of::<stateitem_T>())
            as *mut stateitem_T;
        p.write_bytes(0, 1);
        (*p).si_idx = idx;
    }
}

/// Pop the innermost item off the state stack.
pub(crate) unsafe fn pop_current_state() {
    unsafe {
        if state_len() > 0 {
            unref_extmatch((*state_top()).si_extmatch);
            (*current_state.ptr()).ga_len -= 1;
        }
        // After the end of a pattern, try matching a keyword or pattern again.
        next_match_idx.set(-1);
        // If the first "keepend" item was the one popped, there is no keepend
        // level any more.
        if keepend_level.get() >= state_len() {
            keepend_level.set(-1);
        }
    }
}
