//! [`SlotTable`]: the shape an editor registry takes once it is owned Rust.
//!
//! The editor keeps its long-lived objects — timers, channels, buffers,
//! windows and tab pages — in a table keyed by the monotone id the user
//! sees. Upstream spells that as a khash `Map_*` reached through a raw
//! pointer; this is the same table with the raw pointer gone.
//!
//! [`SlotTable`] is the table itself. [`HandleRegistry`] is the graph
//! registries' shared shape on top of it — see its own docs for the
//! liveness invariant that makes a lookup a *safe* call — and
//! [`OwnedRegistry`] is that shape with the allocation moved in, so that the
//! table releases what it holds and an object in it may own a [`Vec`].
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

use core::borrow::Borrow;
use core::hash::{BuildHasherDefault, Hash, Hasher};
use std::collections::{HashMap, HashSet};

use crate::allocator::Owned;
use crate::types::handle_T;

/// A `HashMap` on [`IdHasher`]: `const`-constructible, so one can be a
/// `static` or a field with a `const` initializer, which
/// `std::collections::HashMap`'s randomly seeded default cannot.
pub(crate) type IdMap<K, V> = HashMap<K, V, BuildHasherDefault<IdHasher>>;

/// The set half of [`IdMap`].
pub(crate) type IdSet<K> = HashSet<K, BuildHasherDefault<IdHasher>>;

/// An empty [`IdMap`]. The editor hands out its own keys, so there is
/// nothing for a randomly seeded hasher to defend against.
pub(crate) const fn id_map<K, V>() -> IdMap<K, V> {
    HashMap::with_hasher(BuildHasherDefault::new())
}

/// An empty [`IdSet`]. See [`id_map`].
pub(crate) const fn id_set<K>() -> IdSet<K> {
    HashSet::with_hasher(BuildHasherDefault::new())
}

/// A registry: values found by id, iterated in khash's order.
///
/// See the module docs for why the order is modelled rather than left to a
/// `HashMap`, and for the reentrancy rule the API enforces.
pub(crate) struct SlotTable<K, V> {
    /// The slots, in iteration order. `swap_remove` keeps it dense.
    slots: Vec<(K, V)>,
    /// Key to its position in `slots`.
    index: IdMap<K, usize>,
}

impl<K, V> SlotTable<K, V> {
    /// An empty table. `const`, so a registry can be a `static`.
    pub(crate) const fn new() -> Self {
        SlotTable {
            slots: Vec::new(),
            index: id_map(),
        }
    }
}

impl<K, V> Default for SlotTable<K, V> {
    fn default() -> Self {
        SlotTable::new()
    }
}

impl<K: Eq + Hash, V> SlotTable<K, V> {
    /// How many entries the table holds.
    pub(crate) fn len(&self) -> usize {
        self.slots.len()
    }

    /// The slots, borrowed, in khash's order.
    ///
    /// The exception to the no-borrow rule in the module docs, for the two
    /// tables that are *interned names* rather than registries: the
    /// namespace map and the augroup map hand out `*const c_char`s into
    /// their own keys, which a clone would dangle. Nothing reachable from
    /// walking either one starts or stops an entry, so the borrow is a
    /// leaf. A registry whose walk fires a callback uses
    /// [`Self::snapshot_keys`]/[`Self::snapshot_values`] instead.
    pub(crate) fn entries(&self) -> &[(K, V)] {
        &self.slots
    }
}

impl<K: Eq + Hash + Clone, V> SlotTable<K, V> {
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
                self.index.insert(key.clone(), self.slots.len());
                self.slots.push((key, value));
            }
        }
    }

    /// Take `key` out, moving the last slot into the hole it leaves.
    pub(crate) fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let i = self.index.remove(key)?;
        let (_, value) = self.slots.swap_remove(i);
        if let Some((moved, _)) = self.slots.get(i) {
            self.index.insert(moved.clone(), i);
        }
        Some(value)
    }

    /// Every key, in order, as an owned `Vec` — see the module docs on
    /// reentrancy. Callers walk this, never the table.
    pub(crate) fn snapshot_keys(&self) -> Vec<K> {
        self.slots.iter().map(|(key, _)| key.clone()).collect()
    }
}

impl<K: Eq + Hash, V: Copy> SlotTable<K, V> {
    /// The value registered under `key`, if any.
    pub(crate) fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let i = *self.index.get(key)?;
        Some(self.slots[i].1)
    }

    /// Every value, in order, as an owned `Vec`. As [`Self::snapshot_keys`].
    pub(crate) fn snapshot_values(&self) -> Vec<V> {
        self.slots.iter().map(|&(_, value)| value).collect()
    }
}

/// A registry keyed by a **monotone** handle: a `Vec` indexed by the handle,
/// so a lookup is one load rather than a hash probe.
///
/// The graph's three registries are on this rather than on [`SlotTable`] for
/// one reason: the list links are handles now, so **every step of every
/// window, buffer or tab page walk is a lookup here**. A `HashMap` probe is
/// two dependent cache misses; `nvim_win_get_number` over twenty windows is
/// quadratic in that, and the window-churn benchmark measured the difference
/// at **+55% on `:split`, +81% on `nvim_list_wins`** before this type
/// existed. `SlotTable`'s khash iteration order buys these three nothing —
/// none of them is ever walked, only asked — so the order is dropped and the
/// index is the storage.
///
/// `base` is the handle slot 0 stands for, and rises as the front empties,
/// which keeps the vector proportional to the *live* window and tab page
/// handles rather than to every one ever issued. Buffers, whose number 1
/// outlives the session, keep `base` at 1 and pay four to eight bytes per
/// buffer ever created — under a megabyte for any plausible session, and
/// upstream's khash was not free either.
struct HandleMap<V> {
    /// `slots[h - base]` is what handle `h` names, `None` for a hole.
    slots: Vec<Option<V>>,
    /// The handle `slots[0]` stands for. Meaningless while `slots` is empty.
    base: handle_T,
}

/// An index into a [`HandleMap`]'s slots as the handle difference it is.
///
/// The vector is indexed by `handle - base`, both of which are `handle_T`,
/// so its length can never exceed the handle range and the conversion back
/// is total.
fn offset(index: usize) -> handle_T {
    handle_T::try_from(index).expect("a handle-indexed vector is handle-sized")
}

impl<V> HandleMap<V> {
    /// An empty map. `const`, so a registry can be a `static`.
    const fn new() -> Self {
        HandleMap {
            slots: Vec::new(),
            base: 1,
        }
    }

    /// Where `handle` would sit. A handle below `base` wraps to an index no
    /// vector can hold, so the bounds check `Vec::get` already makes covers
    /// both ends and a lookup is one subtract and one load — which is the
    /// whole point of this type.
    #[inline]
    fn at(&self, handle: handle_T) -> usize {
        handle.wrapping_sub(self.base).cast_unsigned() as usize
    }

    /// The value `handle` names, borrowed.
    ///
    /// Every step of every window, buffer and tab page walk lands here, so
    /// it is `inline` for the profile that is not `codegen-units = 1`.
    #[inline]
    fn get(&self, handle: handle_T) -> Option<&V> {
        self.slots.get(self.at(handle))?.as_ref()
    }

    /// File `value` under `handle`, replacing whatever was there.
    fn insert(&mut self, handle: handle_T, value: V) {
        if self.slots.is_empty() {
            self.base = handle;
        } else if handle < self.base {
            // A handle below the window the vector covers. The three
            // counters are monotone, so this is the wrap-around
            // `top_file_num` warns about with `W14`; rebuild rather than
            // grow downwards without bound.
            let old = core::mem::take(&mut self.slots);
            let old_base = self.base;
            self.base = handle;
            self.slots.push(Some(value));
            for (i, slot) in old.into_iter().enumerate() {
                let Some(slot) = slot else { continue };
                let h = old_base.wrapping_add(offset(i));
                self.insert(h, slot);
            }
            return;
        }
        let i = self.at(handle);
        if i >= self.slots.len() {
            self.slots.resize_with(i + 1, || None);
        }
        self.slots[i] = Some(value);
    }

    /// Take `handle` out, if it was in.
    fn remove(&mut self, handle: handle_T) -> Option<V> {
        let i = self.at(handle);
        let value = self.slots.get_mut(i)?.take();
        // Give the front back once it is all holes, so a session that churns
        // windows or tab pages does not grow this without bound.
        let empty = self.slots.iter().take_while(|slot| slot.is_none()).count();
        if empty == self.slots.len() {
            self.slots.clear();
        } else if empty > 0 {
            self.slots.drain(..empty);
            self.base = self.base.wrapping_add(offset(empty));
        }
        value
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
/// **The values are bare addresses: this registry does not own its objects.**
/// [`OwnedRegistry`] is the same table with the allocation moved in, and the
/// buffer and tab page registries use it. Windows have not followed, and the
/// reason is the autocommand window: `aucmd_restbuf` takes it *out* of the
/// registry while it stays alive and `aucmd_prepbuf` puts it back, so
/// "registered" and "owned" are not the same lifetime for a `win_T` the way
/// they are for a `buf_T`. Moving windows across means giving the idle
/// autocommand window a named owner first.
pub(crate) struct HandleRegistry<T> {
    /// Handle to the object it names. The value is `Copy` — see the module
    /// docs on reentrancy; an autocommand fires between two of these calls
    /// all the time, so no borrow may outlive one.
    live: HandleMap<*mut T>,
}

impl<T> HandleRegistry<T> {
    /// An empty registry. `const`, so it can be a `static`.
    pub(crate) const fn new() -> Self {
        HandleRegistry {
            live: HandleMap::new(),
        }
    }

    /// The object `handle` names, or `None` when nothing is registered
    /// under it — which is what the khash miss answered with a null.
    #[inline]
    pub(crate) fn get(&self, handle: handle_T) -> Option<*mut T> {
        self.live.get(handle).copied()
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

/// The live objects of one kind that the registry **owns**, found by the
/// handle the user sees.
///
/// [`HandleRegistry`]'s table with the allocation moved in. The invariant is
/// the same one, and so is what a lookup answers — a bare address, because
/// that is the currency the transpiled editor deals in and because an
/// address disturbs nothing when it is copied out ([`Owned::address`]).
/// What changed is who releases the memory: taking a handle out with
/// [`forget`](Self::forget) hands the caller an [`Owned`], and dropping that
/// runs the object's destructor. So an object in here may own a [`Vec`], a
/// [`String`] or anything else with a [`Drop`], which a `xcalloc`ed struct
/// released with `xfree` could not.
///
/// **Why the value is not a `Box<T>`.** A `Box` retags every time it moves,
/// which invalidates every raw pointer derived from it — and the editor's
/// whole point is that those pointers escape (`curbuf`, `w_buffer`, the
/// `firstbuf` list, whatever an autocommand kept). [`Owned`] is the `Box`
/// with its address taken once; see its own docs.
///
/// The free paths still call [`forget`](Self::forget) *first* and hold the
/// `Owned` while they tear the object down, so nothing can find it
/// half-freed, and the drop happens exactly where the `xfree` used to.
pub(crate) struct OwnedRegistry<T> {
    /// Handle to the object it names, which this table owns.
    live: HandleMap<Owned<T>>,
}

impl<T> OwnedRegistry<T> {
    /// An empty registry. `const`, so it can be a `static`.
    pub(crate) const fn new() -> Self {
        OwnedRegistry {
            live: HandleMap::new(),
        }
    }

    /// The address of the object `handle` names, or `None` when nothing is
    /// registered under it — the khash miss, which answered a null.
    #[inline]
    pub(crate) fn get(&self, handle: handle_T) -> Option<*mut T> {
        Some(self.live.get(handle)?.address())
    }

    /// Take ownership of `object` and file it under `handle`, answering its
    /// address for the caller to work from.
    pub(crate) fn register(&mut self, handle: handle_T, object: Owned<T>) -> *mut T {
        let address = object.address();
        self.live.insert(handle, object);
        address
    }

    /// Take `handle`'s object out, handing its allocation to the caller.
    ///
    /// `None` for a handle that was never filed, which `map_del` treats as a
    /// no-op upstream too. The object is unfindable from here on, but it is
    /// not yet freed: the caller decides when to drop what it was given, and
    /// the free paths do it where the `xfree` used to be.
    pub(crate) fn forget(&mut self, handle: handle_T) -> Option<Owned<T>> {
        self.live.remove(handle)
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
pub(crate) struct PendingFree<V> {
    /// What was parked, oldest first: an [`Owned`] for the objects a
    /// registry owns, a bare address for the ones it does not yet.
    parked: Vec<V>,
}

impl<V> PendingFree<V> {
    /// An empty set. `const`, so it can be a `static`.
    pub(crate) const fn new() -> Self {
        PendingFree { parked: Vec::new() }
    }

    /// Park `object` until the deferral ends.
    pub(crate) fn park(&mut self, object: V) {
        self.parked.push(object);
    }

    /// Take the most recently parked allocation out, `None` when the set is
    /// empty. Callers loop on this rather than draining, so that nothing is
    /// borrowed while a free runs — the C re-reads its list head for the same
    /// reason.
    pub(crate) fn take_next(&mut self) -> Option<V> {
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
pub(crate) struct IdHasher(u64);

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

    /// The byte path, for the interned-name tables whose key is a
    /// `Box<[u8]>`. FNV-1a over the bytes, folded into whatever the length
    /// prefix already mixed in: cheap, `const`-constructible (which
    /// `RandomState` is not, and these tables are `static`s), and good
    /// enough for keys the editor itself hands out.
    fn write(&mut self, bytes: &[u8]) {
        let mut h = self.0 ^ 0xcbf2_9ce4_8422_2325;
        for &b in bytes {
            h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
        }
        self.0 = h;
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use super::{HandleMap, HandleRegistry, Owned, OwnedRegistry, PendingFree, SlotTable};

    /// The table's own view of itself, checked after every mutation: the
    /// index agrees with the slots, and every key is findable.
    fn check(table: &SlotTable<u64, u32>, expected: &[u64]) {
        assert_eq!(table.snapshot_keys(), expected, "iteration order");
        let values = table.snapshot_values();
        assert_eq!(values.len(), expected.len());
        for (i, &key) in expected.iter().enumerate() {
            assert_eq!(table.get(&key), Some(u32::try_from(key).unwrap() * 10));
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
        assert_eq!(table.get(&1), None);
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
        assert_eq!(table.remove(&20), Some(200));
        check(&table, &[10, 40, 30]);
        assert_eq!(table.get(&20), None);
    }

    #[test]
    fn removing_the_last_slot_leaves_the_rest_alone() {
        let mut table = filled(&[10, 20, 30]);
        assert_eq!(table.remove(&30), Some(300));
        check(&table, &[10, 20]);
    }

    #[test]
    fn removing_an_absent_key_is_a_no_op() {
        let mut table = filled(&[10, 20]);
        assert_eq!(table.remove(&99), None);
        check(&table, &[10, 20]);
    }

    #[test]
    fn reinsertion_keeps_the_slot_and_overwrites_the_value() {
        let mut table = filled(&[10, 20, 30]);
        table.insert(20, 7);
        assert_eq!(table.snapshot_keys(), [10, 20, 30]);
        assert_eq!(table.get(&20), Some(7));
        assert_eq!(table.snapshot_values(), [100, 7, 300]);
    }

    #[test]
    fn a_removed_key_can_come_back_at_the_end() {
        let mut table = filled(&[10, 20, 30]);
        table.remove(&10);
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
                    table.remove(&victim),
                    Some(u32::try_from(victim).unwrap() * 10)
                );
                // The model: swap the last live key into the hole.
                let i = live.iter().position(|&k| k == victim).unwrap();
                live.swap_remove(i);
            }
            check(&table, &live);
        }
        for key in live.clone() {
            table.remove(&key);
        }
        assert!(table.snapshot_keys().is_empty());
    }

    /// The interned-name shape: an owned, unhashable-by-`IdHasher`-integer
    /// key, looked up by a borrowed slice. `namespace_ids` and the augroup
    /// map are this, and their user-visible order is this order.
    #[test]
    fn an_owned_key_is_found_by_a_borrowed_one() {
        let mut table: SlotTable<Box<[u8]>, i32> = SlotTable::new();
        for (i, name) in [&b"alpha"[..], b"beta", b"gamma"].iter().enumerate() {
            table.insert((*name).into(), i32::try_from(i).unwrap() + 1);
        }
        assert_eq!(table.len(), 3);
        assert_eq!(table.get(&b"gamma"[..]), Some(3));
        assert_eq!(table.get(&b"delta"[..]), None);
        let names: Vec<&[u8]> = table.entries().iter().map(|(k, _)| &**k).collect();
        assert_eq!(names, [&b"alpha"[..], b"beta", b"gamma"], "iteration order");
        assert_eq!(table.remove(&b"alpha"[..]), Some(1));
        let names: Vec<&[u8]> = table.entries().iter().map(|(k, _)| &**k).collect();
        assert_eq!(names, [&b"gamma"[..], b"beta"], "swap-remove order");
    }

    /// A key's bytes never move while it is in the table: `describe_ns`
    /// hands out a `*const c_char` into one and the caller reads it after
    /// the table has grown.
    #[test]
    fn a_keys_bytes_stay_put_while_the_table_grows() {
        let mut table: SlotTable<Box<[u8]>, i32> = SlotTable::new();
        table.insert(b"first\0"[..].into(), 1);
        let addr = table.entries()[0].0.as_ptr();
        for i in 0..64i32 {
            table.insert(format!("n{i}\0").into_bytes().into(), i);
        }
        assert_eq!(table.entries()[0].0.as_ptr(), addr);
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

    // -- OwnedRegistry -----------------------------------------------------
    //
    // Miri-sized, like the above. What these check is that the registry
    // *owns*: an object it still holds is not dropped, one taken out is
    // dropped by whoever took it, and the address stays the same throughout.

    /// Counts its own drops through a shared cell, so a test can say when the
    /// allocation went back.
    struct Tracked<'a>(&'a Cell<u32>);

    impl Drop for Tracked<'_> {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    fn tracked(drops: &Cell<u32>) -> Owned<Tracked<'_>> {
        Owned::new(Box::new(Tracked(drops)))
    }

    #[test]
    fn an_empty_owned_registry_finds_nothing() {
        let reg: OwnedRegistry<Tracked<'_>> = OwnedRegistry::new();
        assert_eq!(reg.get(1), None);
    }

    #[test]
    fn a_registered_object_answers_the_address_it_was_filed_under() {
        let drops = Cell::new(0);
        let mut reg = OwnedRegistry::new();
        let object = tracked(&drops);
        let address = object.address();
        assert_eq!(reg.register(7, object), address);
        assert_eq!(reg.get(7), Some(address));
        assert_eq!(reg.get(8), None);
        assert_eq!(drops.get(), 0, "the registry holds it");
        drop(reg);
        assert_eq!(drops.get(), 1, "and releases it");
    }

    /// The free path's shape: take the object out, keep working through the
    /// address, and drop it at the end. Nothing is freed until then.
    #[test]
    fn forgetting_hands_the_allocation_over_without_freeing_it() {
        let drops = Cell::new(0);
        let mut reg = OwnedRegistry::new();
        let address = reg.register(7, tracked(&drops));
        let owned = reg.forget(7).expect("registered");
        assert_eq!(reg.get(7), None, "unfindable at once");
        assert_eq!(owned.address(), address, "the same allocation");
        assert_eq!(drops.get(), 0, "not yet freed");
        drop(owned);
        assert_eq!(drops.get(), 1);
    }

    #[test]
    fn forgetting_an_unregistered_handle_answers_nothing() {
        let drops = Cell::new(0);
        let mut reg = OwnedRegistry::new();
        reg.register(7, tracked(&drops));
        assert!(reg.forget(8).is_none());
        assert_eq!(drops.get(), 0);
    }

    /// Dropping the registry releases everything left in it, which is what
    /// makes it the owner rather than a directory.
    #[test]
    fn dropping_the_registry_frees_what_it_still_holds() {
        let drops = Cell::new(0);
        let mut reg = OwnedRegistry::new();
        for handle in 1..4 {
            reg.register(handle, tracked(&drops));
        }
        drop(reg.forget(2).expect("registered"));
        assert_eq!(drops.get(), 1);
        drop(reg);
        assert_eq!(drops.get(), 3);
    }

    // -- PendingFree -------------------------------------------------------
    //
    // Miri-sized: the set is generic over what it parks -- an `Owned` for
    // buffers, a bare address for windows -- so these park the addresses of
    // local `Object`s and check only the order.

    #[test]
    fn an_empty_pending_set_hands_back_nothing() {
        let mut pending: PendingFree<*mut Object> = PendingFree::new();
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
        let mut pending: PendingFree<*mut Object> = PendingFree::new();
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
        let mut pending: PendingFree<*mut Object> = PendingFree::new();
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
        let mut pending: PendingFree<*mut Object> = PendingFree::new();
        for _ in 0..3 {
            pending.park(pa);
            assert_eq!(pending.take_next(), Some(pa));
            assert_eq!(pending.take_next(), None);
        }
    }

    /// The front-trim, which is what keeps the handle-indexed vector
    /// proportional to the *live* handles rather than to every one ever
    /// issued. A session that churns windows issues handles for ever.
    #[test]
    fn a_handle_map_gives_the_front_back() {
        let mut map: HandleMap<i32> = HandleMap::new();
        for h in 1..=8 {
            map.insert(h, h * 10);
        }
        assert_eq!(map.slots.len(), 8);
        for h in 1..=6 {
            assert_eq!(map.remove(h), Some(h * 10));
        }
        // Only 7 and 8 are left, and the vector is two slots long.
        assert_eq!(map.slots.len(), 2);
        assert_eq!(map.base, 7);
        assert_eq!(map.get(7).copied(), Some(70));
        assert_eq!(map.get(8).copied(), Some(80));
        assert_eq!(map.get(1), None);
        assert_eq!(map.get(6), None);
        // Emptied completely, it starts over wherever the next handle lands.
        assert_eq!(map.remove(7), Some(70));
        assert_eq!(map.remove(8), Some(80));
        assert!(map.slots.is_empty());
        map.insert(9000, 1);
        assert_eq!(map.get(9000).copied(), Some(1));
        assert_eq!(map.slots.len(), 1);
    }

    /// `top_file_num` wraps to 1 after 2^31 buffers (upstream warns `W14`),
    /// so a handle below the window the vector covers is possible. It has to
    /// rebuild rather than index out of bounds or grow downwards.
    #[test]
    fn a_handle_map_rebuilds_when_a_handle_wraps() {
        let mut map: HandleMap<i32> = HandleMap::new();
        map.insert(100, 1);
        map.insert(102, 3);
        map.insert(1, 42);
        assert_eq!(map.base, 1);
        assert_eq!(map.get(1).copied(), Some(42));
        assert_eq!(map.get(100).copied(), Some(1));
        assert_eq!(map.get(102).copied(), Some(3));
        assert_eq!(map.get(101), None);
        assert_eq!(map.get(0), None);
    }

    /// A hole in the middle stays a hole, and an absent handle is a miss
    /// rather than a panic — `map_del` on an absent key is a no-op upstream
    /// and the reused autocommand window relies on that.
    #[test]
    fn a_handle_map_holes_are_misses() {
        let mut map: HandleMap<i32> = HandleMap::new();
        map.insert(4, 1);
        map.insert(5, 2);
        map.insert(6, 3);
        assert_eq!(map.remove(5), Some(2));
        assert_eq!(map.get(5), None);
        assert_eq!(map.remove(5), None);
        assert_eq!(map.remove(999), None);
        assert_eq!(map.base, 4);
        assert_eq!(map.get(4).copied(), Some(1));
        assert_eq!(map.get(6).copied(), Some(3));
    }
}
