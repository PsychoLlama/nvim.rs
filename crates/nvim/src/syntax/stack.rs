//! The `synstate_T` cache.
//!
//! Parsing a line means parsing every line before it, so the state at the start
//! of a line is remembered for every `SST_DIST`th line and reused. This is that
//! store: allocation ([`syn_stack_alloc`]), the free list and its recycling
//! ([`syn_stack_cleanup`]), the save and restore of the current state
//! ([`store_current_state`], [`load_current_state`]), the equality test that
//! decides whether a re-parse can stop early ([`syn_stack_equal`]), and the
//! invalidation an edit causes ([`syn_stack_apply_changes`]).
//!
//! The entries live in one `b_sst_array` per synblock, threaded into two
//! singly-linked lists: the used one (`b_sst_first`, sorted by line number) and
//! the free one (`b_sst_firstfree`). Recycling rather than allocating is what
//! keeps a scroll through a large file from thrashing the allocator.
//!
//! Displayed lines get an entry each; lines that are not displayed share what
//! is left over, at a distance that depends on how long the buffer is.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;
use crate::winlayer::windows;

/// The state stack of one cached entry, whether it is short enough to live
/// inline in the entry or long enough to need a growarray.
///
/// `sst_union` is a C union discriminated by `sst_stacksize`, and this is the
/// one place that discrimination is written down.
unsafe fn entry_states(p: *mut synstate_T, stacksize: c_int) -> *mut bufstate_T {
    unsafe {
        if stacksize > SST_FIX_STATES {
            (*p).sst_union.sst_ga.ga_data as *mut bufstate_T
        } else {
            &raw mut (*p).sst_union.sst_stack as *mut bufstate_T
        }
    }
}

/// Free a synblock's whole cache.
pub(crate) unsafe fn syn_stack_free_block(block: *mut synblock_T) {
    unsafe {
        if (*block).b_sst_array.is_null() {
            return;
        }
        let mut p = (*block).b_sst_first;
        while !p.is_null() {
            clear_syn_state(p);
            p = (*p).sst_next;
        }
        xfree((*block).b_sst_array as *mut ::core::ffi::c_void);
        (*block).b_sst_array = ::core::ptr::null_mut();
        (*block).b_sst_first = ::core::ptr::null_mut();
        (*block).b_sst_len = 0;
    }
}

/// Free a synblock's cache and force a resync everywhere.
///
/// Used when the syntax items themselves changed, so nothing cached can be
/// trusted any more.
pub(crate) unsafe fn syn_stack_free_all(block: *mut synblock_T) {
    unsafe {
        syn_stack_free_block(block);

        // With 'foldmethod' "syntax", every fold has to be recomputed too.
        for wp in windows() {
            if wp.w_s == block && foldmethod_is_syntax(wp) {
                fold_update_all(wp);
            }
        }
    }
}

/// Allocate `syn_buf`'s cache, or resize it when the buffer's length has moved
/// far enough that the current size is a poor fit.
pub(crate) unsafe fn syn_stack_alloc() {
    unsafe {
        let block = syn_block.get();
        let lines = (*syn_buf.get()).b_ml.ml_line_count as c_int;
        let want = clamp_entries(lines / SST_DIST + Rows.get() * 2);
        if (*block).b_sst_len <= want * 2 && (*block).b_sst_len >= want {
            return; // neither much too big nor a bit too small
        }

        // Allocate 50% too much, to avoid reallocating too often.
        let mut len = clamp_entries((lines + lines / 2) / SST_DIST + Rows.get() * 2);
        if !(*block).b_sst_array.is_null() {
            // When shrinking, clean up the existing stack first and make sure
            // every entry that is still valid fits in the new array.
            while (*block).b_sst_len - (*block).b_sst_freecount + 2 > len && syn_stack_cleanup() {}
            len = len.max((*block).b_sst_len - (*block).b_sst_freecount + 2);
        }
        debug_assert!(len >= 0);

        let sstp = xcalloc(len as size_t, ::core::mem::size_of::<synstate_T>()) as *mut synstate_T;

        // Move the states from the old array into the front of the new one.
        // Upstream walks a `to` pointer that starts at `sstp - 1`, which is an
        // out-of-bounds pointer Rust may not form; counting moved entries says
        // the same thing.
        let mut moved = 0usize;
        if !(*block).b_sst_array.is_null() {
            let mut from = (*block).b_sst_first;
            while !from.is_null() {
                let to = sstp.add(moved);
                *to = *from;
                (*to).sst_next = to.add(1);
                moved += 1;
                from = (*from).sst_next;
            }
        }
        if moved > 0 {
            (*sstp.add(moved - 1)).sst_next = ::core::ptr::null_mut();
            (*block).b_sst_first = sstp;
        } else {
            (*block).b_sst_first = ::core::ptr::null_mut();
        }
        (*block).b_sst_freecount = len - moved as c_int;

        // Thread everything after them into the free list.
        (*block).b_sst_firstfree = sstp.add(moved);
        let mut i = moved;
        while i < len as usize {
            (*sstp.add(i)).sst_next = sstp.add(i + 1);
            i += 1;
        }
        (*sstp.add(len as usize - 1)).sst_next = ::core::ptr::null_mut();

        xfree((*block).b_sst_array as *mut ::core::ffi::c_void);
        (*block).b_sst_array = sstp;
        (*block).b_sst_len = len;
    }
}

/// Keep a wanted entry count inside the array's size limits.
#[inline]
fn clamp_entries(len: c_int) -> c_int {
    len.clamp(SST_MIN_ENTRIES, SST_MAX_ENTRIES)
}

/// Adjust the cached states of every synblock showing `buf` for the change
/// recorded in its `b_mod_*` fields.
///
/// Called from `update_screen()` before the screen is updated, once for each
/// displayed buffer.
pub(crate) unsafe fn syn_stack_apply_changes(buf: *mut buf_T) {
    unsafe {
        syn_stack_apply_changes_block(&raw mut (*buf).b_s, buf);

        for wp in windows() {
            if wp.w_buffer == buf && wp.w_s != &raw mut (*buf).b_s {
                syn_stack_apply_changes_block(wp.w_s, buf);
            }
        }
    }
}

/// Remove the entries inside the changed area and shift the ones below it.
///
/// An entry below the change is not thrown away: it is moved by the number of
/// inserted or deleted lines and given an `sst_change_lnum`, which records the
/// line that has to be re-parsed before the entry can be trusted again.
unsafe fn syn_stack_apply_changes_block(block: *mut synblock_T, buf: *mut buf_T) {
    unsafe {
        let mut prev = ::core::ptr::null_mut::<synstate_T>();
        let mut p = (*block).b_sst_first;
        while !p.is_null() {
            if (*p).sst_lnum + (*block).b_syn_sync_linebreaks > (*buf).b_mod_top {
                let n = (*p).sst_lnum + (*buf).b_mod_xlines;
                if n <= (*buf).b_mod_bot {
                    // Inside the changed area: remove it.
                    let np = (*p).sst_next;
                    if prev.is_null() {
                        (*block).b_sst_first = np;
                    } else {
                        (*prev).sst_next = np;
                    }
                    syn_stack_free_entry(block, p);
                    p = np;
                    continue;
                }
                // Below the changed area: remember the line that has to be
                // parsed before this entry is valid again.
                if (*p).sst_change_lnum != 0 && (*p).sst_change_lnum > (*buf).b_mod_top {
                    if (*p).sst_change_lnum + (*buf).b_mod_xlines > (*buf).b_mod_top {
                        (*p).sst_change_lnum += (*buf).b_mod_xlines;
                    } else {
                        (*p).sst_change_lnum = (*buf).b_mod_top;
                    }
                }
                if (*p).sst_change_lnum == 0 || (*p).sst_change_lnum < (*buf).b_mod_bot {
                    (*p).sst_change_lnum = (*buf).b_mod_bot;
                }
                (*p).sst_lnum = n;
            }
            prev = p;
            p = (*p).sst_next;
        }
    }
}

/// Thin out the cache for `syn_buf`, answering whether anything was freed.
///
/// Entries closer together than the normal distance are candidates; of those,
/// the ones carrying the oldest display tick go. Freeing the oldest rather than
/// the closest is what keeps the lines the user is actually looking at cached.
pub(crate) unsafe fn syn_stack_cleanup() -> bool {
    unsafe {
        let block = syn_block.get();
        if (*block).b_sst_first.is_null() {
            return false;
        }

        // Normal distance between entries for lines that are not displayed.
        let dist: linenr_T = if (*block).b_sst_len <= Rows.get() {
            999999
        } else {
            (*syn_buf.get()).b_ml.ml_line_count / ((*block).b_sst_len - Rows.get()) as linenr_T + 1
        };

        // Find the tick of the oldest removable entry. `above` records that the
        // oldest tick is *above* `b_sst_lasttick`, because the display tick
        // wraps around.
        let mut tick = (*block).b_sst_lasttick;
        let mut above = false;
        let mut prev = (*block).b_sst_first;
        let mut p = (*prev).sst_next;
        while !p.is_null() {
            if (*prev).sst_lnum + dist > (*p).sst_lnum {
                if (*p).sst_tick > (*block).b_sst_lasttick {
                    if !above || (*p).sst_tick < tick {
                        tick = (*p).sst_tick;
                    }
                    above = true;
                } else if !above && (*p).sst_tick < tick {
                    tick = (*p).sst_tick;
                }
            }
            prev = p;
            p = (*p).sst_next;
        }

        // Free the entries carrying that tick which sit closer than `dist`.
        let mut freed = false;
        let mut prev = (*block).b_sst_first;
        let mut p = (*prev).sst_next;
        while !p.is_null() {
            if (*p).sst_tick == tick && (*prev).sst_lnum + dist > (*p).sst_lnum {
                // Move this entry from the used list to the free list.
                (*prev).sst_next = (*p).sst_next;
                syn_stack_free_entry(block, p);
                p = prev;
                freed = true;
            }
            prev = p;
            p = (*p).sst_next;
        }
        freed
    }
}

/// Release an entry's memory and put it on the free list.
pub(crate) unsafe fn syn_stack_free_entry(block: *mut synblock_T, p: *mut synstate_T) {
    unsafe {
        clear_syn_state(p);
        (*p).sst_next = (*block).b_sst_firstfree;
        (*block).b_sst_firstfree = p;
        (*block).b_sst_freecount += 1;
    }
}

/// The cached entry for `lnum`, or the last one before it.
///
/// Answers null when the list is empty or starts after `lnum` -- which is not
/// the same as "no entry for this line", so callers that need an exact hit
/// compare `sst_lnum` themselves.
pub(crate) unsafe fn syn_stack_find_entry(lnum: linenr_T) -> *mut synstate_T {
    unsafe {
        let mut prev = ::core::ptr::null_mut::<synstate_T>();
        let mut p = (*syn_block.get()).b_sst_first;
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
        prev
    }
}

/// Save the current state in the cache for `current_lnum`.
///
/// The current state must be valid for the *start* of that line. Answers the
/// entry it went into, or null when there was nothing to store or no room.
pub(crate) unsafe fn store_current_state() -> *mut synstate_T {
    unsafe {
        let block = syn_block.get();
        let mut sp = syn_stack_find_entry(current_lnum.get());

        // A state that contains a start or end pattern continuing from the
        // previous line cannot be used as a starting point, so it is not
        // stored -- and any entry that already exists for this line is wrong.
        if state_continues_from_previous_line() {
            if !sp.is_null() {
                unlink_entry(block, sp);
                syn_stack_free_entry(block, sp);
            }
            current_state_stored.set(true);
            return ::core::ptr::null_mut();
        }

        if sp.is_null() || (*sp).sst_lnum != current_lnum.get() {
            sp = new_entry(block, sp);
        }
        if !sp.is_null() {
            fill_entry(sp);
        }
        current_state_stored.set(true);
        sp
    }
}

/// Does any item on the current state stack carry a position at or after
/// `current_lnum`, i.e. does it continue from the previous line?
unsafe fn state_continues_from_previous_line() -> bool {
    unsafe {
        let mut i = state_len() - 1;
        while i >= 0 {
            let cur_si = state_at(i);
            if (*cur_si).si_h_startpos.lnum >= current_lnum.get()
                || (*cur_si).si_m_endpos.lnum >= current_lnum.get()
                || (*cur_si).si_h_endpos.lnum >= current_lnum.get()
                || ((*cur_si).si_end_idx != 0 && (*cur_si).si_eoe_pos.lnum >= current_lnum.get())
            {
                return true;
            }
            i -= 1;
        }
        false
    }
}

/// Take `sp` out of the used list.
unsafe fn unlink_entry(block: *mut synblock_T, sp: *mut synstate_T) {
    unsafe {
        if (*block).b_sst_first == sp {
            (*block).b_sst_first = (*sp).sst_next;
            return;
        }
        let mut p = (*block).b_sst_first;
        while !p.is_null() && (*p).sst_next != sp {
            p = (*p).sst_next;
        }
        if !p.is_null() {
            // "just in case": an entry that is not in the list is left alone.
            (*p).sst_next = (*sp).sst_next;
        }
    }
}

/// Take an entry off the free list for `current_lnum` and link it in after
/// `after` (or at the front when that is null).
///
/// Answers null when there is no room even after a cleanup.
unsafe fn new_entry(block: *mut synblock_T, mut after: *mut synstate_T) -> *mut synstate_T {
    unsafe {
        if (*block).b_sst_freecount == 0 {
            syn_stack_cleanup();
            // "after" may have been moved to the free list by the cleanup.
            after = syn_stack_find_entry(current_lnum.get());
        }
        if (*block).b_sst_freecount == 0 {
            return ::core::ptr::null_mut(); // must be a strange problem
        }
        let p = (*block).b_sst_firstfree;
        (*block).b_sst_firstfree = (*p).sst_next;
        (*block).b_sst_freecount -= 1;
        if after.is_null() {
            (*p).sst_next = (*block).b_sst_first;
            (*block).b_sst_first = p;
        } else {
            (*p).sst_next = (*after).sst_next;
            (*after).sst_next = p;
        }
        (*p).sst_stacksize = 0;
        (*p).sst_lnum = current_lnum.get();
        p
    }
}

/// Copy the current state stack into `sp`, overwriting whatever was there.
unsafe fn fill_entry(sp: *mut synstate_T) {
    unsafe {
        clear_syn_state(sp);
        let size = state_len();
        (*sp).sst_stacksize = size;
        if size > SST_FIX_STATES {
            // Needs clearing: something may remain from when the length was
            // below SST_FIX_STATES and the inline array was in use.
            ga_init(
                &raw mut (*sp).sst_union.sst_ga,
                ::core::mem::size_of::<bufstate_T>() as c_int,
                1,
            );
            ga_grow(&raw mut (*sp).sst_union.sst_ga, size);
            (*sp).sst_union.sst_ga.ga_len = size;
        }
        let bp = entry_states(sp, size);
        let mut i = 0;
        while i < size {
            let si = state_at(i);
            let b = bp.offset(i as isize);
            (*b).bs_idx = (*si).si_idx;
            (*b).bs_flags = (*si).si_flags;
            (*b).bs_seqnr = (*si).si_seqnr;
            (*b).bs_cchar = (*si).si_cchar;
            (*b).bs_extmatch = ref_extmatch((*si).si_extmatch);
            i += 1;
        }
        (*sp).sst_next_flags = current_next_flags.get();
        (*sp).sst_next_list = current_next_list.get();
        (*sp).sst_tick = display_tick.get();
        (*sp).sst_change_lnum = 0;
    }
}

/// Copy a cached state stack into the current state.
pub(crate) unsafe fn load_current_state(from: *mut synstate_T) {
    clear_current_state();
    validate_current_state();
    keepend_level.set(-1);

    // SAFETY: the caller's cached state entry.
    let size = unsafe { (*from).sst_stacksize };
    if size != 0 {
        current_state.with_mut(|stack| {
            if let Some(items) = stack {
                items.resize(size as usize, EMPTY_STATE_ITEM);
            }
        });
        // SAFETY: `entry_states` answers the entry's own `size` items, and
        // the stack was just grown to hold them.
        unsafe {
            let bp = entry_states(from, size);
            let mut i = 0;
            while i < size {
                let b = bp.offset(i as isize);
                let si = state_at(i);
                (*si).si_idx = (*b).bs_idx;
                (*si).si_flags = (*b).bs_flags;
                (*si).si_seqnr = (*b).bs_seqnr;
                (*si).si_cchar = (*b).bs_cchar;
                (*si).si_extmatch = ref_extmatch((*b).bs_extmatch);
                if keepend_level.get() < 0 && (*si).si_flags.has(SynFlags::KEEPEND) {
                    keepend_level.set(i);
                }
                (*si).si_ends = 0;
                (*si).si_m_lnum = 0;
                (*si).si_next_list = if (*si).si_idx >= 0 {
                    (*syn_pattern((*si).si_idx)).sp_next_list
                } else {
                    ::core::ptr::null_mut()
                };
                update_si_attr(i);
                i += 1;
            }
        }
    }
    // SAFETY: the caller's cached state entry.
    unsafe {
        current_next_list.set((*from).sst_next_list);
        current_next_flags.set((*from).sst_next_flags);
        current_lnum.set((*from).sst_lnum);
    }
}

/// Is the saved state stack `sp` equal to the current one?
///
/// Equality means the re-parse that produced the current state has arrived
/// back at what was cached, so everything below can be trusted again.
pub(crate) unsafe fn syn_stack_equal(sp: *mut synstate_T) -> bool {
    unsafe {
        // A quick check first: same size and same nextlist.
        let size = state_len();
        if (*sp).sst_stacksize != size || (*sp).sst_next_list != current_next_list.get() {
            return false;
        }

        let bp = entry_states(sp, (*sp).sst_stacksize);
        let mut i = size;
        while i > 0 {
            i -= 1;
            let b = bp.offset(i as isize);
            let si = state_at(i);
            // A different pattern index means a different state.
            if (*b).bs_idx != (*si).si_idx {
                return false;
            }
            if (*b).bs_extmatch == (*si).si_extmatch {
                continue;
            }
            // Different extmatch pointers can still hold the same strings, so
            // compare what they reference. One of them being NULL is a
            // difference outright.
            if !extmatch_equal((*b).bs_extmatch, (*si).si_extmatch, (*si).si_idx) {
                return false;
            }
        }
        true
    }
}

/// Do two extmatch references hold the same submatch strings?
///
/// Case is ignored when the item's start pattern had `sp_ic` set.
unsafe fn extmatch_equal(a: *mut reg_extmatch_T, b: *mut reg_extmatch_T, idx: c_int) -> bool {
    unsafe {
        if a.is_null() || b.is_null() {
            return false;
        }
        let ic = (*syn_pattern(idx)).sp_ic != 0;
        let mut j = 0;
        while j < NSUBEXP as c_int {
            let (am, bm) = ((*a).matches[j as usize], (*b).matches[j as usize]);
            if am != bm {
                // A different pointer can still be the same text.
                if am.is_null() || bm.is_null() {
                    return false;
                }
                if mb_strcmp_ic(
                    ic,
                    am as *const ::core::ffi::c_char,
                    bm as *const ::core::ffi::c_char,
                ) != 0
                {
                    return false;
                }
            }
            j += 1;
        }
        true
    }
}
