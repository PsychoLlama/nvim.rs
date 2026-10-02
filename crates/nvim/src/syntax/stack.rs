//! The state cache.
//!
//! Parsing a line means parsing every line before it, so the state at the start
//! of a line is remembered for every `SST_DIST`th line and reused. This is that
//! store: the [`StateCache`] itself (its two lists, recycling and resizing),
//! the save and restore of the current state ([`store_current_state`],
//! [`load_current_state`]), the equality test that decides whether a re-parse
//! can stop early ([`syn_stack_equal`]), and the invalidation an edit causes
//! ([`syn_stack_apply_changes`]).
//!
//! The entries live in one `Vec` per synblock, threaded into two singly-linked
//! lists by index: the used one (sorted by line number) and the free one.
//! Recycling rather than allocating is what keeps a scroll through a large
//! file from thrashing the allocator.
//!
//! Displayed lines get an entry each; lines that are not displayed share what
//! is left over, at a distance that depends on how long the buffer is.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::winlayer::Buf;
use core::ffi::c_int;

use super::*;
use crate::types::DispTick;
use crate::winlayer::windows;

/// An entry of a [`StateCache`]: its index in the cache's array. Stable
/// until the cache is resized, which only [`syn_stack_alloc`] does, before
/// any is handed out.
pub(crate) type EntryId = usize;

/// One item of a cached state stack: what [`load_current_state`] needs to
/// rebuild a [`StateItem`]. The rest of the item it works out again.
#[derive(Clone)]
struct CachedItem {
    idx: c_int,
    flags: SynFlags,
    seqnr: c_int,
    cchar: c_int,
    extmatch: Option<ExtMatchRef>,
}

/// The parser's state at the start of one line, as the cache keeps it.
pub(crate) struct CachedState {
    /// The next entry on whichever list this one is on.
    next: Option<EntryId>,
    /// The line this is the state at the start of.
    pub(crate) lnum: LineNr,
    /// The state stack, outermost item first.
    items: Vec<CachedItem>,
    /// The pending `nextgroup=` list and its flags.
    next_list: *mut int16_t,
    next_flags: SynFlags,
    /// `display_tick` when the entry was stored, which the cleanup ages
    /// entries by.
    tick: DispTick,
    /// The line that has to be parsed again before the entry can be trusted,
    /// or 0 when it can be now.
    pub(crate) change_lnum: LineNr,
}

impl CachedState {
    const EMPTY: CachedState = CachedState {
        next: None,
        lnum: 0,
        items: Vec::new(),
        next_list: ::core::ptr::null_mut(),
        next_flags: SynFlags::NONE,
        tick: 0,
        change_lnum: 0,
    };
}

/// A syntax block's cache of parser states.
///
/// A fixed number of entries -- [`syn_stack_alloc`] sizes it to the buffer
/// -- each on one of two lists: the used entries, lowest line first, and the
/// free ones. A freed entry keeps its item `Vec`'s capacity, so recycling
/// it does not allocate.
pub(crate) struct StateCache {
    entries: Vec<CachedState>,
    first: Option<EntryId>,
    free: Option<EntryId>,
    free_count: usize,
}

impl StateCache {
    /// No entries at all: not allocated yet.
    pub(crate) const fn new() -> StateCache {
        StateCache {
            entries: Vec::new(),
            first: None,
            free: None,
            free_count: 0,
        }
    }

    /// Whether the cache has been allocated -- upstream's non-null
    /// `b_sst_array`, which every parse checks before it starts.
    pub(crate) fn is_allocated(&self) -> bool {
        !self.entries.is_empty()
    }

    /// How many entries the cache has, used or free.
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    /// How many of them are free.
    pub(crate) fn free_count(&self) -> usize {
        self.free_count
    }

    /// The used entry for the lowest line, if any.
    pub(crate) fn first(&self) -> Option<EntryId> {
        self.first
    }

    /// The used entry after `id`.
    pub(crate) fn next(&self, id: EntryId) -> Option<EntryId> {
        self.entries[id].next
    }

    pub(crate) fn entry(&self, id: EntryId) -> &CachedState {
        &self.entries[id]
    }

    pub(crate) fn entry_mut(&mut self, id: EntryId) -> &mut CachedState {
        &mut self.entries[id]
    }

    /// The used entries, lowest line first.
    pub(crate) fn used(&self) -> impl Iterator<Item = EntryId> + '_ {
        ::core::iter::successors(self.first, |&id| self.next(id))
    }

    /// The entry for `lnum`, or the last one before it.
    ///
    /// `None` when the list is empty or starts after `lnum` -- which is not
    /// the same as "no entry for this line", so a caller that needs an exact
    /// hit compares the entry's `lnum` itself.
    pub(crate) fn find(&self, lnum: LineNr) -> Option<EntryId> {
        let mut prev = None;
        for id in self.used() {
            let at = self.entries[id].lnum;
            if at == lnum {
                return Some(id);
            }
            if at > lnum {
                break;
            }
            prev = Some(id);
        }
        prev
    }

    /// Drop every entry: the cache is unallocated again.
    pub(crate) fn clear(&mut self) {
        *self = StateCache::new();
    }

    /// Rebuild the cache with `len` entries, the used ones first and in
    /// order, the rest free. `len` must be at least the used count.
    fn rebuild(&mut self, len: usize) {
        let mut entries: Vec<CachedState> = Vec::with_capacity(len);
        let mut at = self.first;
        while let Some(id) = at {
            at = self.entries[id].next;
            entries.push(::core::mem::replace(
                &mut self.entries[id],
                CachedState::EMPTY,
            ));
        }
        let used = entries.len();
        debug_assert!(used <= len, "a rebuild keeps every used entry");
        entries.resize_with(len.max(used), || CachedState::EMPTY);
        // Thread the used entries, then the free ones after them.
        for (i, entry) in entries.iter_mut().enumerate() {
            let last = if i < used { used } else { len };
            entry.next = (i + 1 < last).then_some(i + 1);
        }
        self.first = (used > 0).then_some(0);
        self.free = (used < len).then_some(used);
        self.free_count = len - used;
        self.entries = entries;
    }

    /// Take `id` out of the used list. An entry that is not on it is left
    /// alone, "just in case", as upstream does.
    fn unlink(&mut self, id: EntryId) {
        if self.first == Some(id) {
            self.first = self.entries[id].next;
            return;
        }
        let mut at = self.first;
        while let Some(p) = at {
            if self.entries[p].next == Some(id) {
                self.entries[p].next = self.entries[id].next;
                return;
            }
            at = self.entries[p].next;
        }
    }

    /// Put `id`, already off the used list, on the free one, giving back
    /// what its items hold.
    fn release(&mut self, id: EntryId) {
        let entry = &mut self.entries[id];
        entry.items.clear();
        entry.next = self.free;
        self.free = Some(id);
        self.free_count += 1;
    }

    /// Take a free entry for `lnum` and link it in after `after`, or at the
    /// front. `None` when there is no free entry.
    fn take_free(&mut self, after: Option<EntryId>, lnum: LineNr) -> Option<EntryId> {
        if self.free_count == 0 {
            return None;
        }
        let id = self.free.expect("a free count above zero has a free entry");
        self.free = self.entries[id].next;
        self.free_count -= 1;
        let next = match after {
            None => self.first.replace(id),
            Some(after) => self.entries[after].next.replace(id),
        };
        let entry = &mut self.entries[id];
        entry.next = next;
        entry.items.clear();
        entry.lnum = lnum;
        Some(id)
    }

    /// Remove the entries inside a change of lines `top`..`bot` that moved
    /// the lines below it by `xlines`, and shift the ones below it.
    ///
    /// An entry below the change is not thrown away: it is moved and given a
    /// `change_lnum`, the line that has to be re-parsed before the entry can
    /// be trusted again. `linebreaks` widens the change upwards (`:syntax
    /// sync linebreaks`).
    fn apply_changes(&mut self, linebreaks: LineNr, top: LineNr, bot: LineNr, xlines: LineNr) {
        let mut prev: Option<EntryId> = None;
        let mut at = self.first;
        while let Some(id) = at {
            let next = self.entries[id].next;
            let entry = &mut self.entries[id];
            if entry.lnum + linebreaks > top {
                let n = entry.lnum + xlines;
                if n <= bot {
                    // Inside the changed area: remove it.
                    match prev {
                        None => self.first = next,
                        Some(prev) => self.entries[prev].next = next,
                    }
                    self.release(id);
                    at = next;
                    continue;
                }
                // Below the changed area: remember the line that has to be
                // parsed before this entry is valid again.
                if entry.change_lnum != 0 && entry.change_lnum > top {
                    if entry.change_lnum + xlines > top {
                        entry.change_lnum += xlines;
                    } else {
                        entry.change_lnum = top;
                    }
                }
                if entry.change_lnum == 0 || entry.change_lnum < bot {
                    entry.change_lnum = bot;
                }
                entry.lnum = n;
            }
            prev = Some(id);
            at = next;
        }
    }

    /// Free the entries that sit closer than `dist` lines to the one before
    /// them and carry the oldest display tick, answering whether any went.
    ///
    /// Freeing the oldest rather than the closest is what keeps the lines the
    /// user is actually looking at cached. `lasttick` is the tick of the
    /// last parse; the tick wraps around, so an entry *above* it is older
    /// than any below it. The first entry is never a candidate.
    fn cleanup(&mut self, dist: LineNr, lasttick: DispTick) -> bool {
        let Some(first) = self.first else {
            return false;
        };

        // Find the tick of the oldest removable entry.
        let mut tick = lasttick;
        let mut above = false;
        let mut prev = first;
        let mut at = self.entries[prev].next;
        while let Some(id) = at {
            let entry = &self.entries[id];
            if self.entries[prev].lnum + dist > entry.lnum {
                if entry.tick > lasttick {
                    if !above || entry.tick < tick {
                        tick = entry.tick;
                    }
                    above = true;
                } else if !above && entry.tick < tick {
                    tick = entry.tick;
                }
            }
            prev = id;
            at = entry.next;
        }

        // Free the entries carrying that tick which sit closer than `dist`.
        let mut freed = false;
        let mut prev = first;
        let mut at = self.entries[prev].next;
        while let Some(id) = at {
            if self.entries[id].tick == tick
                && self.entries[prev].lnum + dist > self.entries[id].lnum
            {
                // Move this entry from the used list to the free list.
                self.entries[prev].next = self.entries[id].next;
                self.release(id);
                freed = true;
            } else {
                prev = id;
            }
            at = self.entries[prev].next;
        }
        freed
    }
}

impl Default for StateCache {
    fn default() -> StateCache {
        StateCache::new()
    }
}

/// Free a synblock's whole cache.
pub(crate) fn syn_stack_free_block(mut block: SynBlockRef) {
    block.b_sst.clear();
}

/// Free a synblock's cache and force a resync everywhere.
///
/// Used when the syntax items themselves changed, so nothing cached can be
/// trusted any more.
pub(crate) fn syn_stack_free_all(block: SynBlockRef) {
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
    let have = block.b_sst.len() as c_int;
    if have <= want * 2 && have >= want {
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
    if block.b_sst.is_allocated() {
        // When shrinking, clean up the existing stack first and make sure
        // every entry that is still valid fits in the new array.
        let used = |block: &SynBlockRef| (block.b_sst.len() - block.b_sst.free_count()) as c_int;
        while used(&block) + 2 > len && syn_stack_cleanup() {}
        len = len.max(used(&block) + 2);
    }
    debug_assert!(len >= 0);
    block.b_sst.rebuild(len as usize);
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

/// [`StateCache::apply_changes`] for one block, with `buffer`'s change.
fn syn_stack_apply_changes_block(mut block: SynBlockRef, buffer: Buf) {
    let linebreaks = block.b_syn_sync_linebreaks;
    let (top, bot, xlines) = (buffer.b_mod_top, buffer.b_mod_bot, buffer.b_mod_xlines);
    block.b_sst.apply_changes(linebreaks, top, bot, xlines);
}

/// Thin out the cache for `syn_buf`, answering whether anything was freed.
///
/// Entries closer together than the normal distance are candidates; of those,
/// the ones carrying the oldest display tick go.
pub(crate) fn syn_stack_cleanup() -> bool {
    let mut block = syn_block();
    if block.b_sst.first().is_none() {
        return false;
    }

    // Normal distance between entries for lines that are not displayed.
    let dist = store_distance();
    let lasttick = block.b_sst_lasttick;
    block.b_sst.cleanup(dist, lasttick)
}

/// The cached entry for `lnum`, or the last one before it; see
/// [`StateCache::find`].
pub(crate) fn syn_stack_find_entry(lnum: LineNr) -> Option<EntryId> {
    syn_block().b_sst.find(lnum)
}

/// Save the current state in the cache for `current_lnum`.
///
/// The current state must be valid for the *start* of that line. Answers the
/// entry it went into, or `None` when there was nothing to store or no room.
pub(crate) fn store_current_state() -> Option<EntryId> {
    let mut block = syn_block();
    let lnum = current_lnum.get();
    let mut sp = syn_stack_find_entry(lnum);

    // A state that contains a start or end pattern continuing from the
    // previous line cannot be used as a starting point, so it is not
    // stored -- and any entry that already exists for this line is wrong.
    if state_continues_from_previous_line() {
        if let Some(id) = sp {
            block.b_sst.unlink(id);
            block.b_sst.release(id);
        }
        current_state_stored.set(true);
        return None;
    }

    if sp.is_none_or(|id| block.b_sst.entry(id).lnum != lnum) {
        sp = new_entry(block, sp);
    }
    if let Some(id) = sp {
        fill_entry(id);
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

/// Take an entry off the free list for `current_lnum` and link it in after
/// `after` (or at the front when that is `None`), cleaning up first when
/// there is no free one. `None` when there is no room even then.
fn new_entry(mut block: SynBlockRef, mut after: Option<EntryId>) -> Option<EntryId> {
    if block.b_sst.free_count() == 0 {
        syn_stack_cleanup();
        // "after" may have been moved to the free list by the cleanup.
        after = syn_stack_find_entry(current_lnum.get());
    }
    block.b_sst.take_free(after, current_lnum.get())
}

/// Copy the current state stack into entry `id`, overwriting whatever was
/// there.
fn fill_entry(id: EntryId) {
    let mut block = syn_block();
    let entry = block.b_sst.entry_mut(id);
    entry.items.clear();
    current_state.with(|stack| {
        let items = stack.as_deref().unwrap_or_default();
        entry.items.extend(items.iter().map(|si| CachedItem {
            idx: si.si_idx,
            flags: si.si_flags,
            seqnr: si.si_seqnr,
            cchar: si.si_cchar,
            extmatch: si.si_extmatch.clone(),
        }));
    });
    entry.next_flags = current_next_flags.get();
    entry.next_list = current_next_list.get();
    entry.tick = display_tick.get();
    entry.change_lnum = 0;
}

/// Copy cached entry `id` into the current state.
pub(crate) fn load_current_state(id: EntryId) {
    clear_current_state();
    validate_current_state();
    keepend_level.set(-1);

    let block = syn_block();
    let entry = block.b_sst.entry(id);
    let size = entry.items.len() as c_int;
    let items: Vec<StateItem> = entry
        .items
        .iter()
        .map(|cached| StateItem {
            si_idx: cached.idx,
            si_flags: cached.flags,
            si_seqnr: cached.seqnr,
            si_cchar: cached.cchar,
            si_extmatch: cached.extmatch.clone(),
            si_next_list: if cached.idx >= 0 {
                block.pattern(cached.idx).sp_next_list.as_ptr()
            } else {
                ::core::ptr::null_mut()
            },
            ..EMPTY_STATE_ITEM
        })
        .collect();
    let (next_list, next_flags, lnum) = (entry.next_list, entry.next_flags, entry.lnum);
    current_state.set(Some(items));
    for i in 0..size {
        // SAFETY: inside the stack just installed.
        if keepend_level.get() < 0 && unsafe { state_at(i) }.si_flags.has(SynFlags::KEEPEND) {
            keepend_level.set(i);
        }
        // SAFETY: as above; nothing pushes while the attributes are filled.
        unsafe { update_si_attr(i) };
    }
    current_next_list.set(next_list);
    current_next_flags.set(next_flags);
    current_lnum.set(lnum);
}

/// Is the cached entry `id` equal to the current state?
///
/// Equality means the re-parse that produced the current state has arrived
/// back at what was cached, so everything below can be trusted again.
pub(crate) fn syn_stack_equal(id: EntryId) -> bool {
    let block = syn_block();
    let entry = block.b_sst.entry(id);
    // A quick check first: same size and same nextlist.
    if entry.items.len() as c_int != state_len() || entry.next_list != current_next_list.get() {
        return false;
    }
    current_state.with(|stack| {
        let items = stack.as_deref().unwrap_or_default();
        // Innermost first, as upstream compares.
        entry.items.iter().zip(items).rev().all(|(cached, si)| {
            // A different pattern index means a different state.
            cached.idx == si.si_idx
                && extmatch_equal(cached.extmatch.as_ref(), si.si_extmatch.as_ref(), || {
                    block.pattern(si.si_idx).sp_ic != 0
                })
        })
    })
}

/// Do two extmatch references hold the same submatch strings? The same set
/// is equal to itself, and none is equal to none; one being missing is a
/// difference outright. `ic` says whether case is ignored, which is only
/// asked when the strings have to be compared.
fn extmatch_equal(
    a: Option<&ExtMatchRef>,
    b: Option<&ExtMatchRef>,
    ic: impl FnOnce() -> bool,
) -> bool {
    let (a, b) = match (a, b) {
        (None, None) => return true,
        (Some(a), Some(b)) if ExtMatchRef::ptr_eq(a, b) => return true,
        (Some(a), Some(b)) => (a, b),
        _ => return false,
    };
    let ic = ic();
    a.matches
        .iter()
        .zip(&b.matches)
        .all(|(am, bm)| match (am, bm) {
            (None, None) => true,
            (Some(am), Some(bm)) => {
                // SAFETY: both are NUL-terminated captures.
                unsafe { mb_strcmp_ic(ic, am.as_ptr(), bm.as_ptr()) == 0 }
            }
            _ => false,
        })
}

#[cfg(test)]
mod tests {
    //! The state cache and the state stack, driven through the functions the
    //! parser uses, over a block built here rather than a buffer's: what a
    //! stored entry holds, what loading it gives back, when two states are
    //! equal, and that every extmatch reference taken is given back.

    use super::*;
    use crate::global_cell::{editor_state::Held, editor_state_lock};
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
            resize_cache(syn_block(), entries);
            Fixture {
                block,
                rows,
                _held: held,
            }
        }

        /// The line numbers of the used entries, in list order.
        fn used_lines(&self) -> Vec<LineNr> {
            let block = syn_block();
            block
                .b_sst
                .used()
                .map(|id| block.b_sst.entry(id).lnum)
                .collect()
        }

        fn free_count(&self) -> usize {
            syn_block().b_sst.free_count()
        }

        fn len(&self) -> usize {
            syn_block().b_sst.len()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            invalidate_current_state();
            syn_stack_free_block(syn_block());
            parsed_block.set(::core::ptr::null_mut());
            Rows.set(self.rows);
            current_lnum.set(0);
            current_next_list.set(::core::ptr::null_mut());
            // SAFETY: the box `new` leaked, which nothing points at now.
            drop(unsafe { Box::from_raw(self.block) });
        }
    }

    /// A fresh extmatch whose `\z1` is `text`.
    fn extmatch(text: &str) -> ExtMatchRef {
        let mut em = ExtMatch::default();
        em.set(1, text.as_bytes());
        ExtMatchRef::new(em)
    }

    fn refs(em: &ExtMatchRef) -> usize {
        ExtMatchRef::strong_count(em)
    }

    /// Push an item for pattern `idx` with distinct flags, sequence number
    /// and `cchar`, holding a reference to `em`.
    fn push(idx: c_int, em: Option<&ExtMatchRef>) {
        push_current_state(idx);
        // SAFETY: just pushed.
        let mut item = unsafe { state_top() };
        item.si_flags = SynFlags::FOLD;
        item.si_seqnr = 100 + idx;
        item.si_cchar = c_int::from(b'a') + idx;
        item.si_extmatch = em.cloned();
    }

    /// The stack as `(idx, seqnr, cchar, extmatch)` rows.
    fn stack() -> Vec<(c_int, c_int, c_int, Option<*const ExtMatch>)> {
        (0..state_len())
            .map(|i| {
                // SAFETY: inside the stack.
                let item = unsafe { state_at(i) };
                let em = item.si_extmatch.as_ref().map(ExtMatchRef::as_ptr);
                (item.si_idx, item.si_seqnr, item.si_cchar, em)
            })
            .collect()
    }

    #[test]
    fn a_stored_state_loads_back_with_its_references() {
        let _fixture = Fixture::new(3, 20);
        let em = extmatch("x");
        push(0, None);
        push(1, Some(&em));
        push(2, None);
        let before = stack();
        assert_eq!(refs(&em), 2);

        current_lnum.set(5);
        let entry = store_current_state().expect("room in the cache");
        assert_eq!(refs(&em), 3, "the entry holds its own reference");
        assert!(syn_stack_equal(entry));

        invalidate_current_state();
        assert_eq!(refs(&em), 2, "invalidating gives the stack's back");
        assert_eq!(syn_stack_find_entry(5), Some(entry));
        assert_eq!(syn_stack_find_entry(9), Some(entry), "the last one before");
        assert_eq!(syn_stack_find_entry(4), None);

        current_lnum.set(0);
        load_current_state(entry);
        assert_eq!(current_lnum.get(), 5);
        assert_eq!(stack(), before);
        assert_eq!(refs(&em), 3);
        assert!(syn_stack_equal(entry));

        // SAFETY: inside the stack.
        unsafe { state_at(2) }.si_idx = 1;
        assert!(!syn_stack_equal(entry), "a different pattern");

        pop_current_state();
        assert_eq!(state_len(), 2);
        pop_current_state();
        assert_eq!(refs(&em), 2, "popping gives the item's back");
    }

    #[test]
    fn a_deep_stack_round_trips() {
        let _fixture = Fixture::new(1, 20);
        let em = extmatch("deep");
        // Deeper than the seven items upstream's entries held inline.
        for i in 0..10 {
            push(0, (i % 2 == 0).then_some(&em));
            // SAFETY: just pushed.
            unsafe { state_top() }.si_seqnr = i;
        }
        let before = stack();
        current_lnum.set(1);
        let entry = store_current_state().expect("room in the cache");
        invalidate_current_state();
        load_current_state(entry);
        assert_eq!(stack(), before);
        invalidate_current_state();
        assert_eq!(refs(&em), 6, "the test's own, and the cache's five");
    }

    #[test]
    fn extmatches_compare_by_their_text() {
        let _fixture = Fixture::new(1, 20);
        let (a, b, c) = (extmatch("same"), extmatch("same"), extmatch("other"));
        push(0, Some(&a));
        current_lnum.set(3);
        let entry = store_current_state().expect("room in the cache");

        let replace = |em: Option<&ExtMatchRef>| {
            // SAFETY: inside the stack.
            unsafe { state_at(0) }.si_extmatch = em.cloned();
        };
        replace(Some(&b));
        assert!(syn_stack_equal(entry), "same text, other set");
        replace(Some(&c));
        assert!(!syn_stack_equal(entry), "other text");
        replace(None);
        assert!(!syn_stack_equal(entry), "none against some");
        invalidate_current_state();
    }

    #[test]
    fn entries_stay_sorted_and_the_cleanup_frees_the_oldest() {
        let fixture = Fixture::new(1, 20);
        push(0, None);
        for (lnum, tick) in [(30, 3), (10, 1), (20, 2)] {
            current_lnum.set(lnum);
            display_tick.set(tick);
            store_current_state();
        }
        assert_eq!(fixture.used_lines(), [10, 20, 30]);
        assert_eq!(fixture.free_count(), 17);

        // The first entry is never a candidate; of the rest, the one with
        // the oldest tick goes.
        syn_block().b_sst_lasttick = 3;
        assert!(syn_stack_cleanup());
        assert_eq!(fixture.used_lines(), [10, 30]);
        assert_eq!(fixture.free_count(), 18);

        // Storing over an existing line reuses its entry.
        current_lnum.set(30);
        store_current_state();
        assert_eq!(fixture.used_lines(), [10, 30]);
        assert_eq!(fixture.free_count(), 18);
    }

    #[test]
    fn a_resize_keeps_the_used_entries_in_order() {
        let fixture = Fixture::new(1, 20);
        push(0, None);
        for lnum in [4, 8, 2] {
            current_lnum.set(lnum);
            store_current_state();
        }
        resize_cache(syn_block(), 40);
        assert_eq!(fixture.len(), 40);
        assert_eq!(fixture.used_lines(), [2, 4, 8]);
        assert_eq!(fixture.free_count(), 37);

        // Shrinking below what is in use first thins the cache out -- every
        // entry after the first carries the same tick, so all of them go --
        // and then keeps room for what is left and two more.
        resize_cache(syn_block(), 1);
        assert_eq!(fixture.used_lines(), [2]);
        assert_eq!(fixture.len(), 3);
        assert_eq!(fixture.free_count(), 2);
        let entry = syn_stack_find_entry(4).expect("the entry before");
        assert_eq!(syn_block().b_sst.entry(entry).lnum, 2);
    }
}
