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
    if stacksize > SST_FIX_STATES {
        unsafe { (*p).sst_union.sst_ga.ga_data as *mut bufstate_T }
    } else {
        unsafe { &raw mut (*p).sst_union.sst_stack as *mut bufstate_T }
    }
}

/// Free a synblock's whole cache.
pub(crate) unsafe fn syn_stack_free_block(mut block: SynBlock) {
    if block.b_sst_array.is_null() {
        return;
    }
    let mut p = block.b_sst_first;
    while !p.is_null() {
        unsafe { clear_syn_state(p) };
        p = unsafe { (*p).sst_next };
    }
    unsafe { xfree(block.b_sst_array as *mut ::core::ffi::c_void) };
    block.b_sst_array = ::core::ptr::null_mut();
    block.b_sst_first = ::core::ptr::null_mut();
    block.b_sst_len = 0;
}

/// Free a synblock's cache and force a resync everywhere.
///
/// Used when the syntax items themselves changed, so nothing cached can be
/// trusted any more.
pub(crate) unsafe fn syn_stack_free_all(block: *mut synblock_T) {
    // SAFETY: the caller's promise -- a live syntax block.
    let mut block = unsafe { SynBlock::new(block) };
    unsafe { syn_stack_free_block(block) };

    // With 'foldmethod' "syntax", every fold has to be recomputed too.
    for wp in windows() {
        if wp.w_s == block.raw() && foldmethod_is_syntax(wp) {
            fold_update_all(wp);
        }
    }
}

/// Allocate `syn_buf`'s cache, or resize it when the buffer's length has moved
/// far enough that the current size is a poor fit.
pub(crate) unsafe fn syn_stack_alloc() {
    let mut block = syn_block();
    let lines = unsafe { (*syn_buf.get()).b_ml.ml_line_count } as c_int;
    let want = clamp_entries(lines / SST_DIST + Rows.get() * 2);
    if block.b_sst_len <= want * 2 && block.b_sst_len >= want {
        return; // neither much too big nor a bit too small
    }

    // Allocate 50% too much, to avoid reallocating too often.
    let mut len = clamp_entries((lines + lines / 2) / SST_DIST + Rows.get() * 2);
    if !block.b_sst_array.is_null() {
        // When shrinking, clean up the existing stack first and make sure
        // every entry that is still valid fits in the new array.
        while block.b_sst_len - block.b_sst_freecount + 2 > len && unsafe { syn_stack_cleanup() } {}
        len = len.max(block.b_sst_len - block.b_sst_freecount + 2);
    }
    debug_assert!(len >= 0);

    let sstp =
        unsafe { xcalloc(len as size_t, ::core::mem::size_of::<synstate_T>()) } as *mut synstate_T;

    // Move the states from the old array into the front of the new one.
    // Upstream walks a `to` pointer that starts at `sstp - 1`, which is an
    // out-of-bounds pointer Rust may not form; counting moved entries says
    // the same thing.
    let mut moved = 0usize;
    if !block.b_sst_array.is_null() {
        let mut from = block.b_sst_first;
        while !from.is_null() {
            let to = unsafe { sstp.add(moved) };
            unsafe { *to = *from };
            unsafe { (*to).sst_next = to.add(1) };
            moved += 1;
            from = unsafe { (*from).sst_next };
        }
    }
    if moved > 0 {
        unsafe { (*sstp.add(moved - 1)).sst_next = ::core::ptr::null_mut() };
        block.b_sst_first = sstp;
    } else {
        block.b_sst_first = ::core::ptr::null_mut();
    }
    block.b_sst_freecount = len - moved as c_int;

    // Thread everything after them into the free list.
    unsafe { block.b_sst_firstfree = sstp.add(moved) };
    let mut i = moved;
    while i < len as usize {
        unsafe { (*sstp.add(i)).sst_next = sstp.add(i + 1) };
        i += 1;
    }
    unsafe { (*sstp.add(len as usize - 1)).sst_next = ::core::ptr::null_mut() };

    unsafe { xfree(block.b_sst_array as *mut ::core::ffi::c_void) };
    block.b_sst_array = sstp;
    block.b_sst_len = len;
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
    unsafe { syn_stack_apply_changes_block(SynBlock::new(&raw mut (*buf).b_s), buf) };

    for wp in windows() {
        if wp.w_buffer == buf && wp.w_s != unsafe { &raw mut (*buf).b_s } {
            unsafe { syn_stack_apply_changes_block(SynBlock::new(wp.w_s), buf) };
        }
    }
}

/// Remove the entries inside the changed area and shift the ones below it.
///
/// An entry below the change is not thrown away: it is moved by the number of
/// inserted or deleted lines and given an `sst_change_lnum`, which records the
/// line that has to be re-parsed before the entry can be trusted again.
unsafe fn syn_stack_apply_changes_block(mut block: SynBlock, buf: *mut buf_T) {
    let mut prev = ::core::ptr::null_mut::<synstate_T>();
    let mut p = block.b_sst_first;
    while !p.is_null() {
        if unsafe { (*p).sst_lnum } + block.b_syn_sync_linebreaks > unsafe { (*buf).b_mod_top } {
            let n = unsafe { (*p).sst_lnum } + unsafe { (*buf).b_mod_xlines };
            if n <= unsafe { (*buf).b_mod_bot } {
                // Inside the changed area: remove it.
                let np = unsafe { (*p).sst_next };
                if prev.is_null() {
                    block.b_sst_first = np;
                } else {
                    unsafe { (*prev).sst_next = np };
                }
                unsafe { syn_stack_free_entry(block, p) };
                p = np;
                continue;
            }
            // Below the changed area: remember the line that has to be
            // parsed before this entry is valid again.
            if unsafe { (*p).sst_change_lnum } != 0
                && unsafe { (*p).sst_change_lnum } > unsafe { (*buf).b_mod_top }
            {
                if unsafe { (*p).sst_change_lnum } + unsafe { (*buf).b_mod_xlines }
                    > unsafe { (*buf).b_mod_top }
                {
                    unsafe { (*p).sst_change_lnum += (*buf).b_mod_xlines };
                } else {
                    unsafe { (*p).sst_change_lnum = (*buf).b_mod_top };
                }
            }
            if unsafe { (*p).sst_change_lnum } == 0
                || unsafe { (*p).sst_change_lnum } < unsafe { (*buf).b_mod_bot }
            {
                unsafe { (*p).sst_change_lnum = (*buf).b_mod_bot };
            }
            unsafe { (*p).sst_lnum = n };
        }
        prev = p;
        p = unsafe { (*p).sst_next };
    }
}

/// Thin out the cache for `syn_buf`, answering whether anything was freed.
///
/// Entries closer together than the normal distance are candidates; of those,
/// the ones carrying the oldest display tick go. Freeing the oldest rather than
/// the closest is what keeps the lines the user is actually looking at cached.
pub(crate) unsafe fn syn_stack_cleanup() -> bool {
    let mut block = syn_block();
    if block.b_sst_first.is_null() {
        return false;
    }

    // Normal distance between entries for lines that are not displayed.
    let entries = block.b_sst_len;
    let dist: linenr_T = if entries <= Rows.get() {
        999999
    } else {
        let lines = unsafe { (*syn_buf.get()).b_ml.ml_line_count };
        lines / (entries - Rows.get()) as linenr_T + 1
    };

    // Find the tick of the oldest removable entry. `above` records that the
    // oldest tick is *above* `b_sst_lasttick`, because the display tick
    // wraps around.
    let mut tick = block.b_sst_lasttick;
    let mut above = false;
    let mut prev = block.b_sst_first;
    let mut p = unsafe { (*prev).sst_next };
    while !p.is_null() {
        if unsafe { (*prev).sst_lnum } + dist > unsafe { (*p).sst_lnum } {
            if unsafe { (*p).sst_tick } > block.b_sst_lasttick {
                if !above || unsafe { (*p).sst_tick } < tick {
                    tick = unsafe { (*p).sst_tick };
                }
                above = true;
            } else if !above && unsafe { (*p).sst_tick } < tick {
                tick = unsafe { (*p).sst_tick };
            }
        }
        prev = p;
        p = unsafe { (*p).sst_next };
    }

    // Free the entries carrying that tick which sit closer than `dist`.
    let mut freed = false;
    let mut prev = block.b_sst_first;
    let mut p = unsafe { (*prev).sst_next };
    while !p.is_null() {
        if unsafe { (*p).sst_tick } == tick
            && unsafe { (*prev).sst_lnum } + dist > unsafe { (*p).sst_lnum }
        {
            // Move this entry from the used list to the free list.
            unsafe { (*prev).sst_next = (*p).sst_next };
            unsafe { syn_stack_free_entry(block, p) };
            p = prev;
            freed = true;
        }
        prev = p;
        p = unsafe { (*p).sst_next };
    }
    freed
}

/// Release an entry's memory and put it on the free list.
pub(crate) unsafe fn syn_stack_free_entry(mut block: SynBlock, p: *mut synstate_T) {
    unsafe { clear_syn_state(p) };
    unsafe { (*p).sst_next = block.b_sst_firstfree };
    block.b_sst_firstfree = p;
    block.b_sst_freecount += 1;
}

/// The cached entry for `lnum`, or the last one before it.
///
/// Answers null when the list is empty or starts after `lnum` -- which is not
/// the same as "no entry for this line", so callers that need an exact hit
/// compare `sst_lnum` themselves.
pub(crate) unsafe fn syn_stack_find_entry(lnum: linenr_T) -> *mut synstate_T {
    let mut prev = ::core::ptr::null_mut::<synstate_T>();
    let mut p = syn_block().b_sst_first;
    while !p.is_null() {
        if unsafe { (*p).sst_lnum } == lnum {
            return p;
        }
        if unsafe { (*p).sst_lnum } > lnum {
            break;
        }
        prev = p;
        p = unsafe { (*p).sst_next };
    }
    prev
}

/// Save the current state in the cache for `current_lnum`.
///
/// The current state must be valid for the *start* of that line. Answers the
/// entry it went into, or null when there was nothing to store or no room.
pub(crate) unsafe fn store_current_state() -> *mut synstate_T {
    let mut block = syn_block();
    let mut sp = unsafe { syn_stack_find_entry(current_lnum.get()) };

    // A state that contains a start or end pattern continuing from the
    // previous line cannot be used as a starting point, so it is not
    // stored -- and any entry that already exists for this line is wrong.
    if unsafe { state_continues_from_previous_line() } {
        if !sp.is_null() {
            unsafe { unlink_entry(block, sp) };
            unsafe { syn_stack_free_entry(block, sp) };
        }
        current_state_stored.set(true);
        return ::core::ptr::null_mut();
    }

    if sp.is_null() || unsafe { (*sp).sst_lnum } != current_lnum.get() {
        sp = unsafe { new_entry(block, sp) };
    }
    if !sp.is_null() {
        unsafe { fill_entry(sp) };
    }
    current_state_stored.set(true);
    sp
}

/// Does any item on the current state stack carry a position at or after
/// `current_lnum`, i.e. does it continue from the previous line?
unsafe fn state_continues_from_previous_line() -> bool {
    let mut i = state_len() - 1;
    while i >= 0 {
        let cur_si = unsafe { state_at(i) };
        if cur_si.si_h_startpos.lnum >= current_lnum.get()
            || cur_si.si_m_endpos.lnum >= current_lnum.get()
            || cur_si.si_h_endpos.lnum >= current_lnum.get()
            || (cur_si.si_end_idx != 0 && cur_si.si_eoe_pos.lnum >= current_lnum.get())
        {
            return true;
        }
        i -= 1;
    }
    false
}

/// Take `sp` out of the used list.
unsafe fn unlink_entry(mut block: SynBlock, sp: *mut synstate_T) {
    if block.b_sst_first == sp {
        unsafe { block.b_sst_first = (*sp).sst_next };
        return;
    }
    let mut p = block.b_sst_first;
    while !p.is_null() && unsafe { (*p).sst_next } != sp {
        p = unsafe { (*p).sst_next };
    }
    if !p.is_null() {
        // "just in case": an entry that is not in the list is left alone.
        unsafe { (*p).sst_next = (*sp).sst_next };
    }
}

/// Take an entry off the free list for `current_lnum` and link it in after
/// `after` (or at the front when that is null).
///
/// Answers null when there is no room even after a cleanup.
unsafe fn new_entry(mut block: SynBlock, mut after: *mut synstate_T) -> *mut synstate_T {
    if block.b_sst_freecount == 0 {
        unsafe { syn_stack_cleanup() };
        // "after" may have been moved to the free list by the cleanup.
        after = unsafe { syn_stack_find_entry(current_lnum.get()) };
    }
    if block.b_sst_freecount == 0 {
        return ::core::ptr::null_mut(); // must be a strange problem
    }
    let p = block.b_sst_firstfree;
    unsafe { block.b_sst_firstfree = (*p).sst_next };
    block.b_sst_freecount -= 1;
    if after.is_null() {
        unsafe { (*p).sst_next = block.b_sst_first };
        block.b_sst_first = p;
    } else {
        unsafe { (*p).sst_next = (*after).sst_next };
        unsafe { (*after).sst_next = p };
    }
    unsafe { (*p).sst_stacksize = 0 };
    unsafe { (*p).sst_lnum = current_lnum.get() };
    p
}

/// Copy the current state stack into `sp`, overwriting whatever was there.
unsafe fn fill_entry(sp: *mut synstate_T) {
    unsafe { clear_syn_state(sp) };
    let size = state_len();
    unsafe { (*sp).sst_stacksize = size };
    if size > SST_FIX_STATES {
        // Needs clearing: something may remain from when the length was
        // below SST_FIX_STATES and the inline array was in use.
        // SAFETY: `sp` is the cache entry being resized.
        let ga = unsafe { &raw mut (*sp).sst_union.sst_ga };
        let item_size = ::core::mem::size_of::<bufstate_T>() as c_int;
        // SAFETY: a growarray the entry owns.
        unsafe { ga_init(ga, item_size, 1) };
        unsafe { ga_grow(&raw mut (*sp).sst_union.sst_ga, size) };
        unsafe { (*sp).sst_union.sst_ga.ga_len = size };
    }
    let bp = unsafe { entry_states(sp, size) };
    let mut i = 0;
    while i < size {
        let mut si = unsafe { state_at(i) };
        let b = unsafe { bp.offset(i as isize) };
        unsafe { (*b).bs_idx = si.si_idx };
        unsafe { (*b).bs_flags = si.si_flags };
        unsafe { (*b).bs_seqnr = si.si_seqnr };
        unsafe { (*b).bs_cchar = si.si_cchar };
        unsafe { (*b).bs_extmatch = ref_extmatch(si.si_extmatch) };
        i += 1;
    }
    unsafe { (*sp).sst_next_flags = current_next_flags.get() };
    unsafe { (*sp).sst_next_list = current_next_list.get() };
    unsafe { (*sp).sst_tick = display_tick.get() };
    unsafe { (*sp).sst_change_lnum = 0 };
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
        let bp = unsafe { entry_states(from, size) };
        let mut i = 0;
        while i < size {
            let b = unsafe { bp.offset(i as isize) };
            let mut si = unsafe { state_at(i) };
            unsafe { si.si_idx = (*b).bs_idx };
            unsafe { si.si_flags = (*b).bs_flags };
            unsafe { si.si_seqnr = (*b).bs_seqnr };
            unsafe { si.si_cchar = (*b).bs_cchar };
            unsafe { si.si_extmatch = ref_extmatch((*b).bs_extmatch) };
            if keepend_level.get() < 0 && si.si_flags.has(SynFlags::KEEPEND) {
                keepend_level.set(i);
            }
            si.si_ends = 0;
            si.si_m_lnum = 0;
            unsafe {
                si.si_next_list = if si.si_idx >= 0 {
                    syn_pattern(si.si_idx).sp_next_list
                } else {
                    ::core::ptr::null_mut()
                }
            };
            unsafe { update_si_attr(i) };
            i += 1;
        }
    }
    // SAFETY: the caller's cached state entry.
    current_next_list.set(unsafe { (*from).sst_next_list });
    current_next_flags.set(unsafe { (*from).sst_next_flags });
    current_lnum.set(unsafe { (*from).sst_lnum });
}

/// Is the saved state stack `sp` equal to the current one?
///
/// Equality means the re-parse that produced the current state has arrived
/// back at what was cached, so everything below can be trusted again.
pub(crate) unsafe fn syn_stack_equal(sp: *mut synstate_T) -> bool {
    // A quick check first: same size and same nextlist.
    let size = state_len();
    if unsafe { (*sp).sst_stacksize } != size
        || unsafe { (*sp).sst_next_list } != current_next_list.get()
    {
        return false;
    }

    let bp = unsafe { entry_states(sp, (*sp).sst_stacksize) };
    let mut i = size;
    while i > 0 {
        i -= 1;
        let b = unsafe { bp.offset(i as isize) };
        let mut si = unsafe { state_at(i) };
        // A different pattern index means a different state.
        if unsafe { (*b).bs_idx } != si.si_idx {
            return false;
        }
        if unsafe { (*b).bs_extmatch } == si.si_extmatch {
            continue;
        }
        // Different extmatch pointers can still hold the same strings, so
        // compare what they reference. One of them being NULL is a
        // difference outright.
        if !unsafe { extmatch_equal((*b).bs_extmatch, si.si_extmatch, si.si_idx) } {
            return false;
        }
    }
    true
}

/// Do two extmatch references hold the same submatch strings?
///
/// Case is ignored when the item's start pattern had `sp_ic` set.
unsafe fn extmatch_equal(a: *mut reg_extmatch_T, b: *mut reg_extmatch_T, idx: c_int) -> bool {
    if a.is_null() || b.is_null() {
        return false;
    }
    let ic = unsafe { syn_pattern(idx).sp_ic } != 0;
    let mut j = 0;
    while j < NSUBEXP as c_int {
        let (am, bm) = (unsafe { (*a).matches[j as usize] }, unsafe {
            (*b).matches[j as usize]
        });
        if am != bm {
            // A different pointer can still be the same text.
            if am.is_null() || bm.is_null() {
                return false;
            }
            let am = am as *const ::core::ffi::c_char;
            let bm = bm as *const ::core::ffi::c_char;
            // SAFETY: both are NUL-terminated keywords.
            if unsafe { mb_strcmp_ic(ic, am, bm) } != 0 {
                return false;
            }
        }
        j += 1;
    }
    true
}
