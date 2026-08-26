//! [`SlotTable`]: the shape an editor registry takes once it is owned Rust.
//!
//! The editor keeps its long-lived objects — timers, channels, buffers,
//! windows and tab pages — in a table keyed by the monotone id the user
//! sees. Upstream spells that as a khash `Map_*` reached through a raw
//! pointer; this is the same table with the raw pointer gone.
//!
//! [`SlotTable`] is the table itself. [`HandleRegistry`] is the three
//! graph registries' shared shape on top of it — see its own docs for the
//! liveness invariant that makes a lookup a *safe* call.
//! [`PendingFree`] is the other half of a registry's bookkeeping: the
//! allocations an autocommand deferred, which used to be a chain threaded
//! through the graph's own `b_next`/`w_next` links.
//!
//! # The order is part of the contract
//!
//! khash's `Map_*` is not a plain hash map: its keys live in a *dense*
//! array (`set.keys[..set.h.n_keys]`) that every caller iterates directly,
//! so the order objects come back in is insertion order, and deleting one
//! moves the **last** entry into the hole it left (`map::table::delete`).
//! Both halves are visible to users — `timer_info()` answers timers in that
//! order, and a broadcast to every RPC channel goes out in it — so this type
//! reproduces them exactly: a `Vec` of slots plus a `HashMap` from key to
//! slot index, removed with `swap_remove`. It is an index map, not a
//! `HashMap`, for the same reason `memfile::BlockTable` is.
//!
//! # Reentrancy
//!
//! A timer's callback and a channel's callback both enter Vimscript or Lua,
//! and what runs there can start a timer, stop one, open a channel or close
//! one — that is, mutate this table while a caller is walking it. So the
//! type is built so that **no borrow of the table can be held across such a
//! call**:
//!
//! - No method hands out a reference into the table. `V: Copy`, and every
//!   accessor copies the value out, so the borrow ends with the expression
//!   that took it. A registry lives in a [`GlobalCell`], and the
//!   `with`/`with_mut` closure around one of these calls is a leaf: it
//!   cannot re-enter, which is what phase 22's ruling 6 asks for.
//! - Walking is done over a **snapshot** — [`SlotTable::snapshot_keys`] and
//!   [`SlotTable::snapshot_values`] answer an owned `Vec`, so the table is
//!   unborrowed by the time the first callback runs. The snapshot is a
//!   `Vec`, not an iterator, precisely so that it cannot borrow.
//!
//! The cost of the snapshot is one allocation on paths that fire callbacks
//! (teardown, garbage collection, a broadcast, `timer_info()`); the lookups
//! that are actually hot — [`SlotTable::get`] — do not allocate.
//!
//! [`GlobalCell`]: crate::global_cell::GlobalCell

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::hash::{BuildHasherDefault, Hash, Hasher};
use std::collections::HashMap;

use crate::types::handle_T;

/// A registry: values found by id, iterated in khash's order.
///
/// See the module docs for why the order is modelled rather than left to a
/// `HashMap`, and for the reentrancy rule the API enforces.
pub(crate) struct SlotTable<K, V> {
    /// The slots, in iteration order. `swap_remove` keeps it dense.
    slots: Vec<(K, V)>,
    /// Key to its position in `slots`.
    index: HashMap<K, usize, BuildHasherDefault<IdHasher>>,
}

impl<K, V> SlotTable<K, V> {
    /// An empty table. `const`, so a registry can be a `static`.
    pub(crate) const fn new() -> Self {
        SlotTable {
            slots: Vec::new(),
            index: HashMap::with_hasher(BuildHasherDefault::new()),
        }
    }
}

impl<K, V> Default for SlotTable<K, V> {
    fn default() -> Self {
        SlotTable::new()
    }
}

impl<K: Copy + Eq + Hash, V: Copy> SlotTable<K, V> {
    /// The value registered under `key`, if any.
    pub(crate) fn get(&self, key: K) -> Option<V> {
        let i = *self.index.get(&key)?;
        Some(self.slots[i].1)
    }

    /// File `value` under `key`.
    ///
    /// A key that is already present keeps its place in the order and has
    /// its value overwritten, which is what khash's `map_put_ref` followed
    /// by a store does. Registries hand out monotone ids, so in practice
    /// every insert is a fresh key at the end.
    pub(crate) fn insert(&mut self, key: K, value: V) {
        match self.index.get(&key) {
            Some(&i) => self.slots[i].1 = value,
            None => {
                self.index.insert(key, self.slots.len());
                self.slots.push((key, value));
            }
        }
    }

    /// Take `key` out, moving the last slot into the hole it leaves.
    pub(crate) fn remove(&mut self, key: K) -> Option<V> {
        let i = self.index.remove(&key)?;
        let (_, value) = self.slots.swap_remove(i);
        if let Some(&(moved, _)) = self.slots.get(i) {
            self.index.insert(moved, i);
        }
        Some(value)
    }

    /// Every key, in order, as an owned `Vec` — see the module docs on
    /// reentrancy. Callers walk this, never the table.
    pub(crate) fn snapshot_keys(&self) -> Vec<K> {
        self.slots.iter().map(|&(key, _)| key).collect()
    }

    /// Every value, in order, as an owned `Vec`. As [`Self::snapshot_keys`].
    pub(crate) fn snapshot_values(&self) -> Vec<V> {
        self.slots.iter().map(|&(_, value)| value).collect()
    }
}

/// The live objects of one kind, found by the handle the user sees: the
/// shape the window, buffer and tab page registries share. They live in
/// [`crate::winlayer`], which is the only module that may construct one.
///
/// # The liveness invariant
///
/// **Everything in here is live.** There are exactly two ways to change the
/// table — [`register`](Self::register), called by the allocator once the
/// object exists, and [`forget`](Self::forget), called by the free path
/// before the memory goes back — and the free paths call `forget` *first*,
/// so no window between the two is observable. That is what lets
/// [`crate::winlayer`] answer a lookup with a plain `Win`/`Buf`/`TabPage`
/// from a **safe** function: the promise those wrappers' constructors ask
/// for is discharged here, once, rather than at every call site.
///
/// An object that is freed *lazily* is a different matter and is not this
/// type's business: the free paths still `forget` first and only then park
/// the allocation in a [`PendingFree`], so a deferred object is already
/// unfindable here — exactly what the khash map answered once `map_del` had
/// run.
///
/// The values are raw pointers rather than owned allocations. Ownership of
/// the `win_T`/`buf_T`/`tabpage_T` moves in a later slice; what moved here
/// is the *table*.
pub(crate) struct HandleRegistry<T> {
    /// Handle to the object it names. `V: Copy` — see the module docs on
    /// reentrancy; an autocommand fires between two of these calls all the
    /// time, so no borrow may outlive one.
    live: SlotTable<handle_T, *mut T>,
}

impl<T> HandleRegistry<T> {
    /// An empty registry. `const`, so it can be a `static`.
    pub(crate) const fn new() -> Self {
        HandleRegistry {
            live: SlotTable::new(),
        }
    }

    /// The object `handle` names, or `None` when nothing is registered
    /// under it — which is what the khash miss answered with a null.
    pub(crate) fn get(&self, handle: handle_T) -> Option<*mut T> {
        self.live.get(handle)
    }

    /// Record `object` as the live object named by `handle`.
    pub(crate) fn register(&mut self, handle: handle_T, object: *mut T) {
        self.live.insert(handle, object);
    }

    /// Drop `handle`, whether or not it was registered — `map_del` on an
    /// absent key is a no-op upstream too, and the reused autocommand
    /// window relies on that.
    pub(crate) fn forget(&mut self, handle: handle_T) {
        self.live.remove(handle);
    }
}

/// The objects of one kind whose memory must outlive the call that gave them
/// up: the *deferred-free set*.
///
/// A window or buffer closed from inside an autocommand cannot have its
/// allocation given back at once — the handler that closed it, and everything
/// below it in the nesting, may still hold the address. Upstream parks the
/// object on a chain threaded through the very `b_next`/`w_next` fields the
/// editor's own buffer and window lists use; here the set owns its storage,
/// so those fields have one job.
///
/// The order is the C's: pushing at the end and taking from the end is the
/// same last-in-first-out that a push-at-the-head chain drained from the head
/// gave. Nothing here dereferences a parked address — this type stores it and
/// hands it back — which is why the module can `forbid(unsafe_code)` and the
/// free itself lives with the caller.
pub(crate) struct PendingFree<T> {
    /// Parked addresses, oldest first.
    parked: Vec<*mut T>,
}

impl<T> PendingFree<T> {
    /// An empty set. `const`, so it can be a `static`.
    pub(crate) const fn new() -> Self {
        PendingFree { parked: Vec::new() }
    }

    /// Park `object`'s allocation until the deferral ends.
    pub(crate) fn park(&mut self, object: *mut T) {
        self.parked.push(object);
    }

    /// Take the most recently parked allocation out, `None` when the set is
    /// empty. Callers loop on this rather than draining, so that nothing is
    /// borrowed while a free runs — the C re-reads its list head for the same
    /// reason.
    pub(crate) fn take_next(&mut self) -> Option<*mut T> {
        self.parked.pop()
    }

    /// How many allocations are parked. For the tests; the editor only ever
    /// asks by draining.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.parked.len()
    }
}

/// Mixes an id so the top bits — which the hash table probes first — depend
/// on all of it. Registry ids are small and consecutive, which the identity
/// hash spreads badly and SipHash costs too much for. As
/// `memfile::BlockNrHasher`, whose keys have the same shape.
#[derive(Default)]
struct IdHasher(u64);

impl IdHasher {
    fn mix(&mut self, n: u64) {
        // splitmix64's finalizer.
        let mut z = n;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        self.0 = z ^ (z >> 31);
    }
}

impl Hasher for IdHasher {
    fn write_u64(&mut self, n: u64) {
        self.mix(n);
    }

    fn write_i64(&mut self, n: i64) {
        self.mix(n.cast_unsigned());
    }

    fn write_u32(&mut self, n: u32) {
        self.mix(u64::from(n));
    }

    fn write_i32(&mut self, n: i32) {
        self.mix(u64::from(n.cast_unsigned()));
    }

    fn write_usize(&mut self, n: usize) {
        self.mix(n as u64);
    }

    /// The fallback, for a key that is not one of the integers above. No
    /// registry uses one; it is here so the type is a total `Hasher`.
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.mix(self.0.wrapping_mul(31).wrapping_add(u64::from(b)));
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{HandleRegistry, PendingFree, SlotTable};

    /// The table's own view of itself, checked after every mutation: the
    /// index agrees with the slots, and every key is findable.
    fn check(table: &SlotTable<u64, u32>, expected: &[u64]) {
        assert_eq!(table.snapshot_keys(), expected, "iteration order");
        let values = table.snapshot_values();
        assert_eq!(values.len(), expected.len());
        for (i, &key) in expected.iter().enumerate() {
            assert_eq!(table.get(key), Some(u32::try_from(key).unwrap() * 10));
            assert_eq!(values[i], u32::try_from(key).unwrap() * 10);
        }
    }

    fn filled(keys: &[u64]) -> SlotTable<u64, u32> {
        let mut table = SlotTable::new();
        for &key in keys {
            table.insert(key, u32::try_from(key).unwrap() * 10);
        }
        table
    }

    #[test]
    fn empty_table_answers_nothing() {
        let table: SlotTable<u64, u32> = SlotTable::new();
        assert_eq!(table.get(1), None);
        assert!(table.snapshot_keys().is_empty());
        assert!(table.snapshot_values().is_empty());
    }

    #[test]
    fn insertion_order_is_the_iteration_order() {
        check(&filled(&[10, 20, 30, 40]), &[10, 20, 30, 40]);
    }

    /// khash moves the last key into the hole; `tests/unit/map.rs` asserts
    /// exactly this for `Map_uint64_t_ptr_t`, and the two must agree.
    #[test]
    fn removal_swaps_the_last_slot_into_the_hole() {
        let mut table = filled(&[10, 20, 30, 40]);
        assert_eq!(table.remove(20), Some(200));
        check(&table, &[10, 40, 30]);
        assert_eq!(table.get(20), None);
    }

    #[test]
    fn removing_the_last_slot_leaves_the_rest_alone() {
        let mut table = filled(&[10, 20, 30]);
        assert_eq!(table.remove(30), Some(300));
        check(&table, &[10, 20]);
    }

    #[test]
    fn removing_an_absent_key_is_a_no_op() {
        let mut table = filled(&[10, 20]);
        assert_eq!(table.remove(99), None);
        check(&table, &[10, 20]);
    }

    #[test]
    fn reinsertion_keeps_the_slot_and_overwrites_the_value() {
        let mut table = filled(&[10, 20, 30]);
        table.insert(20, 7);
        assert_eq!(table.snapshot_keys(), [10, 20, 30]);
        assert_eq!(table.get(20), Some(7));
        assert_eq!(table.snapshot_values(), [100, 7, 300]);
    }

    #[test]
    fn a_removed_key_can_come_back_at_the_end() {
        let mut table = filled(&[10, 20, 30]);
        table.remove(10);
        check(&table, &[30, 20]);
        table.insert(10, 100);
        check(&table, &[30, 20, 10]);
    }

    /// Churn the table the way the editor does — ids handed out in order,
    /// entries removed from the middle — and check the index never drifts
    /// from the slots. Kept small: it runs under Miri, where the check is
    /// quadratic in the number of live keys.
    #[test]
    fn churn_keeps_the_index_and_the_slots_in_step() {
        let mut table: SlotTable<u64, u32> = SlotTable::new();
        let mut live: Vec<u64> = Vec::new();
        let mut rng: u64 = 0x9e37_79b9_7f4a_7c15;
        for id in 1..60u64 {
            table.insert(id, u32::try_from(id).unwrap() * 10);
            live.push(id);
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            if live.len() > 3 && rng.is_multiple_of(3) {
                let victim = live[usize::try_from(rng >> 33).unwrap() % live.len()];
                assert_eq!(
                    table.remove(victim),
                    Some(u32::try_from(victim).unwrap() * 10)
                );
                // The model: swap the last live key into the hole.
                let i = live.iter().position(|&k| k == victim).unwrap();
                live.swap_remove(i);
            }
            check(&table, &live);
        }
        for key in live.clone() {
            table.remove(key);
        }
        assert!(table.snapshot_keys().is_empty());
    }

    // -- HandleRegistry ----------------------------------------------------
    //
    // Small on purpose: these run under Miri, where every allocation is
    // interpreted. What they check is the wrapper's contract, not the slot
    // table's -- that is covered above.

    /// A stand-in for `win_T`: the registry stores an address and never
    /// reads through it, so an empty type will do.
    struct Object;

    #[test]
    fn an_empty_registry_finds_nothing() {
        let reg: HandleRegistry<Object> = HandleRegistry::new();
        assert_eq!(reg.get(1), None);
    }

    #[test]
    fn a_registered_handle_answers_its_object() {
        let (mut a, mut b) = (Object, Object);
        let (pa, pb) = (&raw mut a, &raw mut b);
        let mut reg: HandleRegistry<Object> = HandleRegistry::new();
        reg.register(7, pa);
        reg.register(9, pb);
        assert_eq!(reg.get(7), Some(pa));
        assert_eq!(reg.get(9), Some(pb));
        assert_eq!(reg.get(8), None);
    }

    /// The free path calls `forget` and the object stops being findable —
    /// which is the whole of the liveness invariant.
    #[test]
    fn a_forgotten_handle_is_gone() {
        let mut a = Object;
        let mut reg: HandleRegistry<Object> = HandleRegistry::new();
        reg.register(7, &raw mut a);
        reg.forget(7);
        assert_eq!(reg.get(7), None);
    }

    /// `map_del` on an absent key is a no-op upstream, and the autocommand
    /// window — unregistered while idle, re-registered when borrowed —
    /// depends on it.
    #[test]
    fn forgetting_an_unregistered_handle_is_a_no_op() {
        let mut a = Object;
        let mut reg: HandleRegistry<Object> = HandleRegistry::new();
        reg.register(7, &raw mut a);
        reg.forget(8);
        reg.forget(8);
        assert_eq!(reg.get(7), Some(&raw mut a));
    }

    /// The autocommand window's cycle: registered at allocation, taken out
    /// when it is given back, put in again when it is borrowed. The handle
    /// never changes and the address never changes.
    #[test]
    fn a_handle_can_be_registered_again_after_being_forgotten() {
        let mut a = Object;
        let pa = &raw mut a;
        let mut reg: HandleRegistry<Object> = HandleRegistry::new();
        for _ in 0..3 {
            reg.register(7, pa);
            assert_eq!(reg.get(7), Some(pa));
            reg.forget(7);
            assert_eq!(reg.get(7), None);
        }
    }

    /// Handles are handed out by a monotone counter and never reused, but a
    /// reused *address* must not confuse the table: what is keyed is the
    /// handle.
    #[test]
    fn a_new_handle_may_name_a_recycled_address() {
        let mut a = Object;
        let pa = &raw mut a;
        let mut reg: HandleRegistry<Object> = HandleRegistry::new();
        reg.register(7, pa);
        reg.forget(7);
        reg.register(8, pa);
        assert_eq!(reg.get(7), None);
        assert_eq!(reg.get(8), Some(pa));
    }

    // -- PendingFree -------------------------------------------------------
    //
    // Miri-sized: the set stores addresses and never reads through one, so
    // these park the addresses of local `Object`s and check only the order.

    #[test]
    fn an_empty_pending_set_hands_back_nothing() {
        let mut pending: PendingFree<Object> = PendingFree::new();
        assert_eq!(pending.len(), 0);
        assert!(pending.take_next().is_none());
    }

    /// The C pushes at the head of a chain and drains from the head, so the
    /// last object deferred is the first one freed. Parking at the end of a
    /// `Vec` and taking from the end is the same order.
    #[test]
    fn parked_allocations_come_back_last_in_first_out() {
        let (mut a, mut b, mut c) = (Object, Object, Object);
        let (pa, pb, pc) = (&raw mut a, &raw mut b, &raw mut c);
        let mut pending: PendingFree<Object> = PendingFree::new();
        pending.park(pa);
        pending.park(pb);
        pending.park(pc);
        assert_eq!(pending.len(), 3);
        assert_eq!(pending.take_next(), Some(pc));
        assert_eq!(pending.take_next(), Some(pb));
        assert_eq!(pending.take_next(), Some(pa));
        assert_eq!(pending.take_next(), None);
    }

    /// The drain loop the editor runs: take one, free it, ask again. A set
    /// that grew while it ran would still be emptied, which is what the C's
    /// re-read of the list head buys.
    #[test]
    fn a_drain_loop_empties_the_set_including_what_it_grew_by() {
        let mut objects = [Object, Object, Object, Object];
        let addresses: Vec<*mut Object> = objects.iter_mut().map(|o| &raw mut *o).collect();
        let mut pending: PendingFree<Object> = PendingFree::new();
        pending.park(addresses[0]);
        pending.park(addresses[1]);
        let mut freed = Vec::new();
        let mut refill = true;
        while let Some(object) = pending.take_next() {
            freed.push(object);
            if refill {
                refill = false;
                pending.park(addresses[2]);
                pending.park(addresses[3]);
            }
        }
        assert_eq!(pending.len(), 0);
        assert_eq!(
            freed,
            [addresses[1], addresses[3], addresses[2], addresses[0]]
        );
    }

    /// Parking, draining and parking again is the shape of two nested
    /// autocommands in a row; the set is reusable, not one-shot.
    #[test]
    fn a_drained_set_can_be_used_again() {
        let mut a = Object;
        let pa = &raw mut a;
        let mut pending: PendingFree<Object> = PendingFree::new();
        for _ in 0..3 {
            pending.park(pa);
            assert_eq!(pending.take_next(), Some(pa));
            assert_eq!(pending.take_next(), None);
        }
    }
}
