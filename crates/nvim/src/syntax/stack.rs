//! The `SynState` cache.
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
#![allow(unsafe_code)]

use crate::winlayer::Buf;
use core::ffi::c_int;

use super::*;
use crate::winlayer::windows;

/// The state stack of one cached entry, whether it is short enough to live
/// inline in the entry or long enough to need a growarray.
///
/// `sst_union` is a C union discriminated by `sst_stacksize`, and this is the
/// one place that discrimination is written down.
///
/// # Safety
///
/// `p` must point at a live syntax state, unaliased for the call.
unsafe fn entry_states(p: *mut SynState, stacksize: c_int) -> *mut BufState {
    if stacksize > SST_FIX_STATES {
        unsafe { (*p).sst_union.sst_heap }
    } else {
        unsafe { &raw mut (*p).sst_union.sst_stack as *mut BufState }
    }
}

/// A `BufState` with nothing in it, which is what a fresh heap arm holds
/// until [`fill_entry`] copies the state stack over it.
const EMPTY_BUFSTATE: BufState = BufState {
    bs_idx: 0,
    bs_flags: SynFlags::NONE,
    bs_seqnr: 0,
    bs_cchar: 0,
    bs_extmatch: ::core::ptr::null_mut(),
};

/// Free a synblock's whole cache.
pub(crate) fn syn_stack_free_block(mut block: SynBlockRef) {
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
pub(crate) fn syn_stack_free_all(block: SynBlockRef) {
    // SAFETY: the handle's promise -- a live syntax block.
    syn_stack_free_block(block);

    // With 'foldmethod' "syntax", every fold has to be recomputed too.
    for wp in windows() {
        if wp.w_s == block.raw() && foldmethod_is_syntax(wp) {
            fold_update_all(wp);
        }
    }
}

/// Allocate `syn_buf`'s cache, or resize it when the buffer's length has moved
/// far enough that the current size is a poor fit.
pub(crate) fn syn_stack_alloc() {
    let block = syn_block();
    let lines = syn_buffer().line_count() as c_int;
    let want = clamp_entries(lines / SST_DIST + Rows.get() * 2);
    if block.b_sst_len <= want * 2 && block.b_sst_len >= want {
        return; // neither much too big nor a bit too small
    }

    // Allocate 50% too much, to avoid reallocating too often.
    resize_cache(
        block,
        clamp_entries((lines + lines / 2) / SST_DIST + Rows.get() * 2),
    );
}

/// Give `block`'s cache room for `len` entries -- more, if that many are in
/// use -- keeping the used ones in order at the front.
fn resize_cache(mut block: SynBlockRef, mut len: c_int) {
    if !block.b_sst_array.is_null() {
        // When shrinking, clean up the existing stack first and make sure
        // every entry that is still valid fits in the new array.
        while block.b_sst_len - block.b_sst_freecount + 2 > len && syn_stack_cleanup() {}
        len = len.max(block.b_sst_len - block.b_sst_freecount + 2);
    }
    debug_assert!(len >= 0);

    let sstp =
        unsafe { xcalloc(len as size_t, ::core::mem::size_of::<SynState>()) } as *mut SynState;

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

/// Adjust the cached states of every synblock showing `buffer` for the change
/// recorded in its `b_mod_*` fields.
///
/// Called from `update_screen()` before the screen is updated, once for each
/// displayed buffer.
pub(crate) fn syn_stack_apply_changes(buffer: Buf) {
    unsafe { syn_stack_apply_changes_block(SynBlockRef::new(buffer.syntax_block()), buffer) };

    for wp in windows() {
        if wp.w_buffer == buffer && wp.w_s != buffer.syntax_block() {
            unsafe { syn_stack_apply_changes_block(SynBlockRef::new(wp.w_s), buffer) };
        }
    }
}

/// Remove the entries inside the changed area and shift the ones below it.
///
/// An entry below the change is not thrown away: it is moved by the number of
/// inserted or deleted lines and given an `sst_change_lnum`, which records the
/// line that has to be re-parsed before the entry can be trusted again.
fn syn_stack_apply_changes_block(mut block: SynBlockRef, buffer: Buf) {
    let mut prev = ::core::ptr::null_mut::<SynState>();
    let mut p = block.b_sst_first;
    while !p.is_null() {
        if unsafe { (*p).sst_lnum } + block.b_syn_sync_linebreaks > buffer.b_mod_top {
            let n = unsafe { (*p).sst_lnum } + buffer.b_mod_xlines;
            if n <= buffer.b_mod_bot {
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
                && unsafe { (*p).sst_change_lnum } > buffer.b_mod_top
            {
                if unsafe { (*p).sst_change_lnum } + buffer.b_mod_xlines > buffer.b_mod_top {
                    unsafe { (*p).sst_change_lnum += buffer.b_mod_xlines };
                } else {
                    unsafe { (*p).sst_change_lnum = buffer.b_mod_top };
                }
            }
            if unsafe { (*p).sst_change_lnum } == 0
                || unsafe { (*p).sst_change_lnum } < buffer.b_mod_bot
            {
                unsafe { (*p).sst_change_lnum = buffer.b_mod_bot };
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
pub(crate) fn syn_stack_cleanup() -> bool {
    let block = syn_block();
    if block.b_sst_first.is_null() {
        return false;
    }

    // Normal distance between entries for lines that are not displayed.
    let entries = block.b_sst_len;
    let dist: LineNr = if entries <= Rows.get() {
        999999
    } else {
        let lines = syn_buffer().line_count();
        lines / (entries - Rows.get()) as LineNr + 1
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
///
/// # Safety
///
/// `block` must be an initialized `SynBlockRef` whose pointer fields point at
/// live data for the call. `p` must point at a live syntax state, unaliased
/// for the call.
pub(crate) unsafe fn syn_stack_free_entry(mut block: SynBlockRef, p: *mut SynState) {
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
pub(crate) fn syn_stack_find_entry(lnum: LineNr) -> *mut SynState {
    let mut prev = ::core::ptr::null_mut::<SynState>();
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
pub(crate) fn store_current_state() -> *mut SynState {
    let block = syn_block();
    let mut sp = syn_stack_find_entry(current_lnum.get());

    // A state that contains a start or end pattern continuing from the
    // previous line cannot be used as a starting point, so it is not
    // stored -- and any entry that already exists for this line is wrong.
    if state_continues_from_previous_line() {
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
fn state_continues_from_previous_line() -> bool {
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

/// Take `state` out of the used list.
///
/// # Safety
///
/// `block` must be an initialized `SynBlockRef` whose pointer fields point at
/// live data for the call. `state` must point at a live syntax state,
/// unaliased for the call.
unsafe fn unlink_entry(mut block: SynBlockRef, state: *mut SynState) {
    if block.b_sst_first == state {
        unsafe { block.b_sst_first = (*state).sst_next };
        return;
    }
    let mut p = block.b_sst_first;
    while !p.is_null() && unsafe { (*p).sst_next } != state {
        p = unsafe { (*p).sst_next };
    }
    if !p.is_null() {
        // "just in case": an entry that is not in the list is left alone.
        unsafe { (*p).sst_next = (*state).sst_next };
    }
}

/// Take an entry off the free list for `current_lnum` and link it in after
/// `after` (or at the front when that is null).
///
/// Answers null when there is no room even after a cleanup.
///
/// # Safety
///
/// `block` must be an initialized `SynBlockRef` whose pointer fields point at
/// live data for the call. `after` must point at a live syntax state,
/// unaliased for the call.
unsafe fn new_entry(mut block: SynBlockRef, mut after: *mut SynState) -> *mut SynState {
    if block.b_sst_freecount == 0 {
        syn_stack_cleanup();
        // "after" may have been moved to the free list by the cleanup.
        after = syn_stack_find_entry(current_lnum.get());
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

/// Copy the current state stack into `state`, overwriting whatever was there.
///
/// # Safety
///
/// `state` must point at a live syntax state, unaliased for the call.
unsafe fn fill_entry(state: *mut SynState) {
    unsafe { clear_syn_state(state) };
    let size = state_len();
    unsafe { (*state).sst_stacksize = size };
    if size > SST_FIX_STATES {
        // The entry takes a heap arm of exactly `size` items. `clear_syn_state`
        // released whatever was there, inline arm included, so nothing of the
        // previous stack survives into this one.
        let states = vec![EMPTY_BUFSTATE; size as usize].into_boxed_slice();
        // SAFETY: `state` is the cache entry being filled; the box is leaked into
        // the union arm and put back together by `clear_syn_state`.
        unsafe { (*state).sst_union.sst_heap = Box::into_raw(states).cast() };
    }
    let bp = unsafe { entry_states(state, size) };
    let mut i = 0;
    while i < size {
        let si = unsafe { state_at(i) };
        let b = unsafe { bp.offset(i as isize) };
        unsafe { (*b).bs_idx = si.si_idx };
        unsafe { (*b).bs_flags = si.si_flags };
        unsafe { (*b).bs_seqnr = si.si_seqnr };
        unsafe { (*b).bs_cchar = si.si_cchar };
        unsafe { (*b).bs_extmatch = ref_extmatch(si.si_extmatch) };
        i += 1;
    }
    unsafe { (*state).sst_next_flags = current_next_flags.get() };
    unsafe { (*state).sst_next_list = current_next_list.get() };
    unsafe { (*state).sst_tick = display_tick.get() };
    unsafe { (*state).sst_change_lnum = 0 };
}

/// Copy a cached state stack into the current state.
///
/// # Safety
///
/// `from` must point at a live syntax state, unaliased for the call.
pub(crate) unsafe fn load_current_state(from: *mut SynState) {
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
            si.si_next_list = if si.si_idx >= 0 {
                syn_block().pattern(si.si_idx).sp_next_list.as_ptr()
            } else {
                ::core::ptr::null_mut()
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

/// Is the saved state stack `state` equal to the current one?
///
/// Equality means the re-parse that produced the current state has arrived
/// back at what was cached, so everything below can be trusted again.
///
/// # Safety
///
/// `state` must point at a live syntax state, unaliased for the call.
pub(crate) unsafe fn syn_stack_equal(state: *mut SynState) -> bool {
    // A quick check first: same size and same nextlist.
    let size = state_len();
    if unsafe { (*state).sst_stacksize } != size
        || unsafe { (*state).sst_next_list } != current_next_list.get()
    {
        return false;
    }

    let bp = unsafe { entry_states(state, (*state).sst_stacksize) };
    let mut i = size;
    while i > 0 {
        i -= 1;
        let b = unsafe { bp.offset(i as isize) };
        let si = unsafe { state_at(i) };
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
///
/// # Safety
///
/// `a` must point at a live `RegExtMatch`, unaliased for the call. `b` must
/// point at a live `RegExtMatch`, unaliased for the call.
unsafe fn extmatch_equal(a: *mut RegExtMatch, b: *mut RegExtMatch, idx: c_int) -> bool {
    if a.is_null() || b.is_null() {
        return false;
    }
    let ic = syn_block().pattern(idx).sp_ic != 0;
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

#[cfg(test)]
mod tests {
    //! The state cache and the state stack, driven through the functions the
    //! parser uses, over a block built here rather than a buffer's: what a
    //! stored entry holds, what loading it gives back, when two states are
    //! equal, and that every extmatch reference taken is given back.

    use super::*;
    use crate::drawscreen::state::display_tick;
    use crate::global_cell::{editor_state::Held, editor_state_lock};
    use crate::regexp::make_extmatch;
    use crate::strings::xstrnsave;
    use core::ffi::c_char;
    use core::mem::MaybeUninit;

    /// A syntax block with `patterns` plain match patterns, installed as the
    /// one being parsed, with a cache of `entries` entries and an empty,
    /// valid state stack. Dropping it puts the parser back as it found it.
    struct Fixture {
        /// Leaked from a `Box`, which `drop` takes back: the parser reaches
        /// the block through `parsed_block`, and a `Box` moved after that
        /// would assert a uniqueness the parser's pointer contradicts.
        block: *mut SynBlock,
        rows: c_int,
        _held: Held,
    }

    impl Fixture {
        fn new(patterns: usize, entries: c_int) -> Fixture {
            let held = editor_state_lock();
            let mut block: Box<MaybeUninit<SynBlock>> = Box::new_zeroed();
            // SAFETY: a zeroed block is what `init_synblock` completes.
            unsafe { init_synblock(block.as_mut_ptr()) };
            // SAFETY: completed just above.
            let mut block = unsafe { block.assume_init() };
            for _ in 0..patterns {
                let mut pattern = EMPTY_SYNPAT;
                pattern.sp_type = SPTYPE_MATCH as c_char;
                block.b_syn_patterns.push(pattern);
            }
            let block = Box::into_raw(block);
            parsed_block.set(block);
            // More rows than entries keeps the cleanup from asking the
            // (absent) buffer how long it is.
            let rows = Rows.replace(10_000);
            invalidate_current_state();
            validate_current_state();
            // SAFETY: the block installed above.
            resize_cache(syn_block(), entries);
            Fixture {
                block,
                rows,
                _held: held,
            }
        }

        /// The line numbers of the used entries, in list order.
        fn used_lines(&self) -> Vec<LineNr> {
            let mut lines = Vec::new();
            let mut p = syn_block().b_sst_first;
            while !p.is_null() {
                // SAFETY: a live entry of the block's cache.
                lines.push(unsafe { (*p).sst_lnum });
                p = unsafe { (*p).sst_next };
            }
            lines
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            invalidate_current_state();
            syn_stack_free_block(syn_block());
            parsed_block.set(::core::ptr::null_mut());
            Rows.set(self.rows);
            // SAFETY: the box `new` leaked, which nothing points at now.
            drop(unsafe { Box::from_raw(self.block) });
            current_lnum.set(0);
            current_next_list.set(::core::ptr::null_mut());
        }
    }

    /// A fresh extmatch whose `\z1` is `text`; the caller holds the one
    /// reference.
    fn extmatch(text: &str) -> *mut RegExtMatch {
        let em = make_extmatch();
        // SAFETY: a fresh set; its slot takes an owned copy.
        unsafe {
            (*em).matches[1] = xstrnsave(text.as_ptr().cast(), text.len()).cast();
        }
        em
    }

    fn refs(em: *mut RegExtMatch) -> int16_t {
        // SAFETY: the test holds a reference, so it is live.
        unsafe { (*em).refcnt }
    }

    /// Push an item for pattern `idx` with distinct flags, sequence number
    /// and `cchar`, holding a reference to `em`.
    fn push(idx: c_int, em: *mut RegExtMatch) {
        push_current_state(idx);
        // SAFETY: just pushed.
        let mut item = unsafe { state_top() };
        item.si_flags = SynFlags::FOLD;
        item.si_seqnr = 100 + idx;
        item.si_cchar = c_int::from(b'a') + idx;
        // SAFETY: the test's own extmatch, or null.
        item.si_extmatch = unsafe { ref_extmatch(em) };
    }

    /// The stack as `(idx, seqnr, cchar, extmatch)` rows.
    fn stack() -> Vec<(c_int, c_int, c_int, *mut RegExtMatch)> {
        (0..state_len())
            .map(|i| {
                // SAFETY: inside the stack.
                let item = unsafe { state_at(i) };
                (item.si_idx, item.si_seqnr, item.si_cchar, item.si_extmatch)
            })
            .collect()
    }

    #[test]
    fn a_stored_state_loads_back_with_its_references() {
        let _fixture = Fixture::new(3, 20);
        let em = extmatch("x");
        push(0, ::core::ptr::null_mut());
        push(1, em);
        push(2, ::core::ptr::null_mut());
        let before = stack();
        assert_eq!(refs(em), 2);

        current_lnum.set(5);
        let entry = store_current_state();
        assert!(!entry.is_null());
        assert_eq!(refs(em), 3, "the entry holds its own reference");
        // SAFETY: the entry just stored.
        assert!(unsafe { syn_stack_equal(entry) });

        invalidate_current_state();
        assert_eq!(refs(em), 2, "invalidating gives the stack's back");
        assert_eq!(syn_stack_find_entry(5), entry);
        assert_eq!(syn_stack_find_entry(9), entry, "the last one before");
        assert!(syn_stack_find_entry(4).is_null());

        current_lnum.set(0);
        // SAFETY: as above.
        unsafe { load_current_state(entry) };
        assert_eq!(current_lnum.get(), 5);
        assert_eq!(stack(), before);
        assert_eq!(refs(em), 3);
        // SAFETY: as above.
        assert!(unsafe { syn_stack_equal(entry) });

        // SAFETY: inside the stack.
        unsafe { state_at(2) }.si_idx = 1;
        // SAFETY: as above.
        assert!(!unsafe { syn_stack_equal(entry) }, "a different pattern");

        pop_current_state();
        assert_eq!(state_len(), 2);
        pop_current_state();
        assert_eq!(refs(em), 2, "popping gives the item's back");
        // SAFETY: the test's own reference.
        unsafe { unref_extmatch(em) };
    }

    #[test]
    fn a_deep_stack_round_trips_through_the_heap() {
        let _fixture = Fixture::new(1, 20);
        let em = extmatch("deep");
        for i in 0..SST_FIX_STATES + 3 {
            push(
                0,
                if i % 2 == 0 {
                    em
                } else {
                    ::core::ptr::null_mut()
                },
            );
            // SAFETY: just pushed.
            unsafe { state_top() }.si_seqnr = i;
        }
        let before = stack();
        current_lnum.set(1);
        let entry = store_current_state();
        invalidate_current_state();
        // SAFETY: the entry just stored.
        unsafe { load_current_state(entry) };
        assert_eq!(stack(), before);
        invalidate_current_state();
        // SAFETY: the test's own reference; the cache still holds five.
        unsafe { unref_extmatch(em) };
    }

    #[test]
    fn extmatches_compare_by_their_text() {
        let _fixture = Fixture::new(1, 20);
        let (a, b, c) = (extmatch("same"), extmatch("same"), extmatch("other"));
        push(0, a);
        current_lnum.set(3);
        let entry = store_current_state();

        let replace = |em: *mut RegExtMatch| {
            // SAFETY: inside the stack; the item's old reference goes, a
            // new one to `em` comes.
            let mut item = unsafe { state_at(0) };
            unsafe { unref_extmatch(item.si_extmatch) };
            item.si_extmatch = unsafe { ref_extmatch(em) };
        };
        replace(b);
        // SAFETY: the entry just stored.
        assert!(unsafe { syn_stack_equal(entry) }, "same text, other set");
        replace(c);
        assert!(!unsafe { syn_stack_equal(entry) }, "other text");
        replace(::core::ptr::null_mut());
        assert!(!unsafe { syn_stack_equal(entry) }, "none against some");

        invalidate_current_state();
        for em in [a, b, c] {
            // SAFETY: the test's own references.
            unsafe { unref_extmatch(em) };
        }
    }

    #[test]
    fn entries_stay_sorted_and_the_cleanup_frees_the_oldest() {
        let fixture = Fixture::new(1, 20);
        push(0, ::core::ptr::null_mut());
        for (lnum, tick) in [(30, 3), (10, 1), (20, 2)] {
            current_lnum.set(lnum);
            display_tick.set(tick);
            store_current_state();
        }
        assert_eq!(fixture.used_lines(), [10, 20, 30]);
        assert_eq!(syn_block().b_sst_freecount, 17);

        // The first entry is never a candidate; of the rest, the one with
        // the oldest tick goes.
        syn_block().b_sst_lasttick = 3;
        assert!(syn_stack_cleanup());
        assert_eq!(fixture.used_lines(), [10, 30]);
        assert_eq!(syn_block().b_sst_freecount, 18);

        // Storing over an existing line reuses its entry.
        current_lnum.set(30);
        store_current_state();
        assert_eq!(fixture.used_lines(), [10, 30]);
        assert_eq!(syn_block().b_sst_freecount, 18);
    }

    #[test]
    fn a_resize_keeps_the_used_entries_in_order() {
        let fixture = Fixture::new(1, 20);
        push(0, ::core::ptr::null_mut());
        for lnum in [4, 8, 2] {
            current_lnum.set(lnum);
            store_current_state();
        }
        resize_cache(syn_block(), 40);
        assert_eq!(syn_block().b_sst_len, 40);
        assert_eq!(fixture.used_lines(), [2, 4, 8]);
        assert_eq!(syn_block().b_sst_freecount, 37);

        // Shrinking below what is in use first thins the cache out -- every
        // entry after the first carries the same tick, so all of them go --
        // and then keeps room for what is left and two more.
        resize_cache(syn_block(), 1);
        assert_eq!(fixture.used_lines(), [2]);
        assert_eq!(syn_block().b_sst_len, 3);
        assert_eq!(syn_block().b_sst_freecount, 2);
        let entry = syn_stack_find_entry(4);
        // SAFETY: an entry of the cache.
        assert_eq!(unsafe { (*entry).sst_lnum }, 2);
    }
}
