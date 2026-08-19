//! The current syntax state and the per-line driver.
//!
//! [`syntax_start`] is the entry point every reader goes through: it points the
//! module's statics at a window/buffer, finds a state to start from (the cache
//! in `stack.rs`, or a `syn_sync` scan) and parses forward to the wanted line.
//! [`syn_finish_line`] is one line of that walk, [`syn_start_line`] resets the
//! per-line state, and [`syn_update_ends`] recomputes where the items on the
//! stack end after the state was loaded from the cache.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;
use crate::types::NUL;

/// Start syntax recognition for a line.
///
/// Normally called from the screen update, once per displayed line. The window
/// and buffer are remembered in `syn_win`/`syn_buf`/`syn_block`, because
/// [`get_syntax_attr`] is not given them -- and careful: `curwin` and `curbuf`
/// are likely to point somewhere else entirely.
pub unsafe fn syntax_start(wp: *mut win_T, lnum: linenr_T) {
    unsafe {
        // The last change id we parsed at. A change may have invalidated the
        // current state, so this is checked as if it were part of the identity
        // of the buffer.
        static changedtick: GlobalCell<varnumber_T> = GlobalCell::new(0);

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
            return; // out of memory
        }
        (*syn_block.get()).b_sst_lasttick = display_tick.get();

        // If the state at the end of the previous line is useful, store it.
        if current_state_valid()
            && current_lnum.get() < lnum
            && current_lnum.get() < (*syn_buf.get()).b_ml.ml_line_count
        {
            syn_finish_line(false);
            if !current_state_stored.get() {
                current_lnum.set(current_lnum.get() + 1);
                store_current_state();
            }
            // If current_lnum is now "lnum", keep the current state -- which
            // happens very often. Otherwise work it out below.
            if current_lnum.get() != lnum {
                invalidate_current_state();
            }
        } else {
            invalidate_current_state();
        }

        // Try to synchronise from a saved state, but only if "lnum" is neither
        // before one nor too far beyond one.
        let mut last_valid = ::core::ptr::null_mut::<synstate_T>();
        if !current_state_valid() {
            let mut last_min_valid = ::core::ptr::null_mut::<synstate_T>();
            let mut p = (*syn_block.get()).b_sst_first;
            while !p.is_null() && (*p).sst_lnum <= lnum {
                if (*p).sst_change_lnum == 0 {
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

        // Still nothing: re-synchronise.
        let first_stored = if !current_state_valid() {
            syn_sync(wp, lnum, last_valid);
            if current_lnum.get() == 1 {
                1 // the first line is always valid, whatever "minlines" says
            } else {
                // "minlines" lines have to be parsed before a state can be
                // considered valid enough to store.
                current_lnum.get() + (*syn_block.get()).b_syn_sync_minlines
            }
        } else {
            current_lnum.get()
        };

        // Advance from the sync point or the saved state to the wanted line,
        // saving some entries along the way to sync with later on.
        let dist = store_distance();
        let mut prev = ::core::ptr::null_mut::<synstate_T>();
        while current_lnum.get() < lnum {
            syn_start_line();
            syn_finish_line(false);
            current_lnum.set(current_lnum.get() + 1);
            if current_lnum.get() >= first_stored {
                prev = record_line(prev, lnum, dist);
            }

            // This can take a long time: stop when CTRL-C is pressed. The
            // current state is then wrong.
            line_breakcheck();
            if got_int.get() {
                current_lnum.set(lnum);
                break;
            }
        }
        syn_start_line();
    }
}

/// Is the current state valid, i.e. does it describe a real position?
///
/// Upstream spells this `VALID_STATE`, and stores the answer in the growarray's
/// `ga_itemsize`: `invalidate_current_state` zeroes it.
#[inline]
pub(crate) unsafe fn current_state_valid() -> bool {
    unsafe { (*current_state.ptr()).ga_itemsize != 0 }
}

/// How many lines apart to store cache entries for lines that are not
/// displayed. Displayed lines get one each; the rest share what is left.
unsafe fn store_distance() -> linenr_T {
    unsafe {
        if (*syn_block.get()).b_sst_len <= Rows.get() {
            999999
        } else {
            (*syn_buf.get()).b_ml.ml_line_count
                / ((*syn_block.get()).b_sst_len - Rows.get()) as linenr_T
                + 1
        }
    }
}

/// After parsing up to `current_lnum`, either adopt the cached state for this
/// line or store the one we just computed. Answers the cache entry to carry
/// into the next line.
///
/// When the cached entry for this line matches what we parsed, every entry
/// below it that was only waiting on a change *before* this line becomes valid
/// again -- which is what turns one re-parse into a whole valid tail.
unsafe fn record_line(
    mut prev: *mut synstate_T,
    lnum: linenr_T,
    dist: linenr_T,
) -> *mut synstate_T {
    unsafe {
        if prev.is_null() {
            prev = syn_stack_find_entry(current_lnum.get() - 1);
        }
        let mut sp = if prev.is_null() {
            (*syn_block.get()).b_sst_first
        } else {
            prev
        };
        while !sp.is_null() && (*sp).sst_lnum < current_lnum.get() {
            sp = (*sp).sst_next;
        }

        if !sp.is_null() && (*sp).sst_lnum == current_lnum.get() && syn_stack_equal(sp) {
            let parsed_lnum = current_lnum.get();
            prev = sp;
            while !sp.is_null() && (*sp).sst_change_lnum <= parsed_lnum {
                if (*sp).sst_lnum <= lnum {
                    prev = sp; // a valid state before the desired line
                } else if (*sp).sst_change_lnum == 0 {
                    break; // past the states that depend on a change
                }
                (*sp).sst_change_lnum = 0;
                sp = (*sp).sst_next;
            }
            load_current_state(prev);
            return prev;
        }

        // Store the state at this line when it is the first one, the line we
        // are parsing for, or far enough from the last stored one.
        if prev.is_null()
            || current_lnum.get() == lnum
            || current_lnum.get() >= (*prev).sst_lnum + dist
        {
            return store_current_state();
        }
        prev
    }
}

/// Release the extmatch references a cached state holds.
///
/// A growarray full of `bufstate_T` cannot simply be discarded: each entry may
/// hold a reference to the submatches of the pattern that started it.
pub(crate) unsafe fn clear_syn_state(p: *mut synstate_T) {
    unsafe {
        if (*p).sst_stacksize > SST_FIX_STATES {
            let gap = &raw mut (*p).sst_union.sst_ga;
            if !(*gap).ga_data.is_null() {
                let mut i = 0;
                while i < (*gap).ga_len {
                    unref_extmatch(
                        (*((*gap).ga_data as *mut bufstate_T).offset(i as isize)).bs_extmatch,
                    );
                    i += 1;
                }
            }
            ga_clear(gap);
        } else {
            let mut i = 0;
            while i < (*p).sst_stacksize {
                unref_extmatch((*p).sst_union.sst_stack[i as usize].bs_extmatch);
                i += 1;
            }
        }
    }
}

/// Empty the current state stack, releasing its extmatch references.
pub(crate) unsafe fn clear_current_state() {
    unsafe {
        let gap = current_state.ptr();
        if !(*gap).ga_data.is_null() {
            let mut i = 0;
            while i < (*gap).ga_len {
                unref_extmatch(
                    (*((*gap).ga_data as *mut stateitem_T).offset(i as isize)).si_extmatch,
                );
                i += 1;
            }
        }
        ga_clear(gap);
    }
}

/// Reset the per-line state before parsing a line.
pub(crate) unsafe fn syn_start_line() {
    unsafe {
        current_finished.set(false);
        current_col.set(0);

        // The end of a start/skip/end that continues from the previous line
        // needs updating, and so do regions with "keepend".
        if state_len() > 0 {
            syn_update_ends(true);
            check_state_ends();
        }

        next_match_idx.set(-1);
        current_line_id.set(current_line_id.get() + 1);
        next_seqnr.set(1);
    }
}

/// Recompute where the items on the stack end.
///
/// `startofline` says we are at the start of a line, in which case the
/// innermost item is always updated; otherwise the update is forced only on the
/// items with "keepend", because they influence what they contain.
pub(crate) unsafe fn syn_update_ends(startofline: bool) {
    unsafe {
        if startofline {
            // A match carried over from a previous line with a contained
            // region ends as soon as that region ends, so drop the end it has
            // and mark it as continued.
            let mut i = 0;
            while i < state_len() {
                let cur_si = state_at(i);
                if (*cur_si).si_idx >= 0
                    && (*syn_pattern((*cur_si).si_idx)).sp_type as c_int == SPTYPE_MATCH
                    && (*cur_si).si_m_endpos.lnum < current_lnum.get()
                {
                    (*cur_si).si_flags |= SynFlags::MATCHCONT;
                    (*cur_si).si_m_endpos = lpos_T { lnum: 0, col: 0 };
                    (*cur_si).si_h_endpos = (*cur_si).si_m_endpos;
                    (*cur_si).si_ends = 1;
                }
                i += 1;
            }
        }

        // Start from the innermost "extend" item, as check_keepend does: a
        // "keepend" outside it does nothing. If "extend" has just been removed
        // (`!startofline`) the normal regions inside a "keepend" need updating
        // too, because "extend" could have extended those as well.
        let mut i = state_len() - 1;
        if keepend_level.get() >= 0 {
            while i > keepend_level.get() {
                if (*state_at(i)).si_flags.has(SynFlags::EXTEND) {
                    break;
                }
                i -= 1;
            }
        }

        let mut seen_keepend = false;
        while i < state_len() {
            let cur_si = state_at(i);
            let innermost = i == state_len() - 1;
            if (*cur_si).si_flags.has(SynFlags::KEEPEND)
                || (seen_keepend && !startofline)
                || (innermost && startofline)
            {
                // Highlighting starts in column 0.
                (*cur_si).si_h_startpos.col = 0;
                (*cur_si).si_h_startpos.lnum = current_lnum.get();

                if !(*cur_si).si_flags.has(SynFlags::MATCHCONT) {
                    update_si_end(cur_si, current_col.get(), !startofline);
                }
                if !startofline && (*cur_si).si_flags.has(SynFlags::KEEPEND) {
                    seen_keepend = true;
                }
            }
            i += 1;
        }
        check_keepend();
    }
}

/// Stop parsing syntax above line `lnum`.
///
/// If the stored state at or below this line depended on a change before it, it
/// now depends on the line below the last parsed one. The window looks like:
/// the line which changed, the displayed lines, then `lnum` -- the line below
/// the window.
pub unsafe fn syntax_end_parsing(wp: *mut win_T, lnum: linenr_T) {
    unsafe {
        if syn_block.get() != (*wp).w_s {
            return; // not the right window
        }
        let mut sp = syn_stack_find_entry(lnum);
        if !sp.is_null() && (*sp).sst_lnum < lnum {
            sp = (*sp).sst_next;
        }
        if !sp.is_null() && (*sp).sst_change_lnum != 0 {
            (*sp).sst_change_lnum = lnum;
        }
    }
}

/// Throw the current state away and mark it invalid.
pub(crate) unsafe fn invalidate_current_state() {
    unsafe {
        clear_current_state();
        (*current_state.ptr()).ga_itemsize = 0; // marks current_state invalid
        current_next_list.set(::core::ptr::null_mut());
        keepend_level.set(-1);
    }
}

/// Mark the current state valid and ready to be pushed onto.
pub(crate) unsafe fn validate_current_state() {
    unsafe {
        (*current_state.ptr()).ga_itemsize = ::core::mem::size_of::<stateitem_T>() as c_int;
        ga_set_growsize(current_state.ptr(), 3);
    }
}

/// Has the syntax at the start of `lnum` changed since last time?
///
/// Only called just after [`get_syntax_attr`] for the previous line, to decide
/// whether the next line has to be redrawn too.
pub unsafe fn syntax_check_changed(lnum: linenr_T) -> bool {
    unsafe {
        // Only worth checking when `lnum` is just below the line we last
        // parsed and there is a saved state for it.
        if !current_state_valid() || lnum != current_lnum.get() + 1 {
            return true;
        }
        let sp = syn_stack_find_entry(lnum);
        if sp.is_null() || (*sp).sst_lnum != lnum {
            return true;
        }

        // Finish the previous line, which is needed when not all of it was
        // drawn, and compare with the state saved for this one.
        syn_finish_line(false);
        let changed = !syn_stack_equal(sp);

        // Store the current state for later use.
        current_lnum.set(current_lnum.get() + 1);
        store_current_state();
        changed
    }
}

/// Parse to the end of the current line without answering any attributes; only
/// the state at the end of the line is wanted.
///
/// May start anywhere in the line, as long as the current state is valid.
/// While syncing, answers whether a sync point was found.
pub(crate) unsafe fn syn_finish_line(syncing: bool) -> bool {
    unsafe {
        while !current_finished.get() {
            syn_current_attr(syncing, false, ::core::ptr::null_mut(), false);

            if syncing && state_len() != 0 {
                // Check for a match with a sync item.
                let cur_si = state_top();
                if (*cur_si).si_idx >= 0
                    && (*syn_pattern((*cur_si).si_idx))
                        .sp_flags
                        .has(SynFlags::SYNC_HERE | SynFlags::SYNC_THERE)
                {
                    return true;
                }

                // syn_current_attr() skipped the check for an item that ends
                // here; do it now. Be careful not to go past the NUL.
                let prev_col = current_col.get();
                if *syn_getcurline().offset(current_col.get() as isize) as c_int != NUL {
                    current_col.set(current_col.get() + 1);
                }
                check_state_ends();
                current_col.set(prev_col);
            }
            current_col.set(current_col.get() + 1);
        }
        false
    }
}
