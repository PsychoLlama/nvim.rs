//! A table that owns values at fixed addresses and names them by id.
//!
//! What the editor used to keep as a raw global pointer to a heap object --
//! the funccall in progress, the exception being thrown -- is an id into one
//! of these instead, and the table is the object's one owner:
//!
//! - a [`TableId`] is a slot index plus the slot's generation, which moves
//!   every time the slot is emptied, so an id kept past its value resolves
//!   to a panic rather than to freed memory or to the next tenant;
//! - a slot keeps its value in a `Box` that is filled once and never moved
//!   until it is emptied, so the value's address is fixed and the editor's
//!   raw pointers into it (a funccall's `l:` dictionary in a value, a
//!   closure's scope) stay good; they are derived through the `UnsafeCell`,
//!   so none of them is a borrow of the box;
//! - the box holds the value in a [`ManuallyDrop`]: emptying a slot gives
//!   back the block and drops nothing of the value, as the editor gives back
//!   what such an object holds by hand;
//! - a slot can carry a small `M` beside the value, readable without going
//!   through the value's pointer.
//!
//! The table is not itself a global: an owner keeps one in its own cell.
//!
//! It is not [`crate::registry`]'s `SlotTable`, which files objects under
//! the handle the user sees, in khash's order; nor its `Owned<T>`, which
//! always runs `T`'s destructor. Here nothing user-visible names the value,
//! and emptying a slot drops nothing of it unless the owner asks for the
//! value back ([`IdTable::remove`]).

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::num::NonZeroU32;

/// A value's place in a [`IdTable<T>`].
pub(crate) struct TableId<T> {
    index: u32,
    generation: NonZeroU32,
    kind: PhantomData<fn() -> T>,
}

impl<T> Clone for TableId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for TableId<T> {}

impl<T> PartialEq for TableId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for TableId<T> {}

impl<T> fmt::Debug for TableId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TableId({}#{})", self.index, self.generation)
    }
}

/// A value's storage in a slot: boxed so its address is fixed, behind an
/// `UnsafeCell` so pointers into it are not borrows of the box, and
/// undropped so the owner decides what of it to give back.
pub(crate) type Boxed<T> = Box<UnsafeCell<ManuallyDrop<T>>>;

struct Slot<T, M> {
    generation: NonZeroU32,
    value: Option<Boxed<T>>,
    meta: M,
}

/// Values of type `T`, each with an `M` beside it, owned at fixed addresses.
pub(crate) struct IdTable<T, M = ()> {
    slots: Vec<Slot<T, M>>,
    /// Empty slots, reused last-emptied first.
    vacant: Vec<u32>,
}

impl<T, M: Default> IdTable<T, M> {
    /// An empty table.
    pub(crate) const fn new() -> Self {
        IdTable {
            slots: Vec::new(),
            vacant: Vec::new(),
        }
    }

    /// Take ownership of `value` and set `meta` beside it. Answers the id and
    /// the value's fixed address.
    pub(crate) fn insert(&mut self, meta: M, value: T) -> (TableId<T>, *mut T) {
        let value = Box::new(UnsafeCell::new(ManuallyDrop::new(value)));
        self.insert_boxed(meta, value, |_, _| {})
    }

    /// Take ownership of `value`, already in its box -- for a value too big
    /// to build on the stack and move -- and set `meta` beside it; `init`
    /// finishes the value in place, knowing its id, before anything else can
    /// reach it. Answers the id and the value's fixed address.
    pub(crate) fn insert_boxed(
        &mut self,
        meta: M,
        mut value: Boxed<T>,
        init: impl FnOnce(TableId<T>, &mut T),
    ) -> (TableId<T>, *mut T) {
        // One pass: take a vacant slot (or add one), name it, fill it. A
        // funccall goes through here on every call.
        let index = self.vacant.pop().unwrap_or_else(|| {
            let index = u32::try_from(self.slots.len()).expect("fewer than 2^32 slots");
            self.slots.push(Slot {
                generation: NonZeroU32::MIN,
                value: None,
                meta: M::default(),
            });
            index
        });
        let slot = &mut self.slots[index as usize];
        debug_assert!(slot.value.is_none(), "a vacant slot is empty");
        let id = TableId {
            index,
            generation: slot.generation,
            kind: PhantomData,
        };
        init(id, value.get_mut());
        slot.meta = meta;
        let value = slot.value.insert(value);
        (id, value.get().cast())
    }

    /// Empty `id`'s slot and give back the value's block, dropping nothing
    /// of the value: the caller has given back what it held.
    ///
    /// # Panics
    /// When `id`'s value is gone already.
    pub(crate) fn free(&mut self, id: TableId<T>) {
        let value = self.empty(id);
        // Freed in place: moving a large value out only to forget it is a
        // copy for nothing.
        drop(value);
    }

    /// Empty `id`'s slot and hand its value back, for the caller to give
    /// back what it holds.
    ///
    /// # Panics
    /// When `id`'s value is gone already.
    pub(crate) fn remove(&mut self, id: TableId<T>) -> T {
        ManuallyDrop::into_inner(UnsafeCell::into_inner(*self.empty(id)))
    }

    fn empty(&mut self, id: TableId<T>) -> Boxed<T> {
        let slot = self.live_mut(id);
        slot.generation = slot.generation.checked_add(1).unwrap_or(NonZeroU32::MIN);
        slot.meta = M::default();
        let value = slot.value.take().expect("a live slot holds a value");
        self.vacant.push(id.index);
        value
    }
}

impl<T, M> IdTable<T, M> {
    fn live(&self, id: TableId<T>) -> &Slot<T, M> {
        let slot = &self.slots[id.index as usize];
        assert!(
            slot.generation == id.generation && slot.value.is_some(),
            "an id outlived its value"
        );
        slot
    }

    fn live_mut(&mut self, id: TableId<T>) -> &mut Slot<T, M> {
        let slot = &mut self.slots[id.index as usize];
        assert!(
            slot.generation == id.generation && slot.value.is_some(),
            "an id outlived its value"
        );
        slot
    }

    /// `id`'s value, by its fixed address.
    ///
    /// # Panics
    /// When `id`'s value is gone.
    pub(crate) fn address(&self, id: TableId<T>) -> *mut T {
        match &self.live(id).value {
            Some(value) => value.get().cast(),
            None => unreachable!("`live` checked the slot is full"),
        }
    }

    /// What was set beside `id`'s value. Nothing outside the tests reads it
    /// since the funccalls moved to an [`RcTable`].
    #[cfg(test)]
    pub(crate) fn meta(&self, id: TableId<T>) -> &M {
        &self.live(id).meta
    }
}

/// Values of type `T`, each shared (`Rc`) with whoever is using it, with an
/// `M` beside it, named by [`TableId`].
///
/// [`IdTable`]'s sibling for a value whose holders keep it alive by sharing
/// rather than by a raw pointer into a fixed box: the table owns one
/// reference, an id resolves to another, and emptying the slot drops the
/// table's -- the value goes when its last holder does.
pub(crate) struct RcTable<T, M = ()> {
    slots: Vec<RcSlot<T, M>>,
    vacant: Vec<u32>,
}

struct RcSlot<T, M> {
    generation: NonZeroU32,
    value: Option<::std::rc::Rc<T>>,
    meta: M,
}

impl<T, M: Default> RcTable<T, M> {
    /// An empty table.
    pub(crate) const fn new() -> Self {
        RcTable {
            slots: Vec::new(),
            vacant: Vec::new(),
        }
    }

    /// Share `value`, whose id `make` is told before it builds it, and set
    /// `meta` beside it.
    pub(crate) fn insert_with(
        &mut self,
        meta: M,
        make: impl FnOnce(TableId<T>) -> ::std::rc::Rc<T>,
    ) -> TableId<T> {
        let index = self.vacant.pop().unwrap_or_else(|| {
            let index = u32::try_from(self.slots.len()).expect("fewer than 2^32 slots");
            self.slots.push(RcSlot {
                generation: NonZeroU32::MIN,
                value: None,
                meta: M::default(),
            });
            index
        });
        let slot = &mut self.slots[index as usize];
        let id = TableId {
            index,
            generation: slot.generation,
            kind: PhantomData,
        };
        slot.value = Some(make(id));
        slot.meta = meta;
        id
    }

    /// Empty `id`'s slot, answering the table's reference.
    ///
    /// # Panics
    /// When `id`'s value is gone already.
    pub(crate) fn remove(&mut self, id: TableId<T>) -> ::std::rc::Rc<T> {
        let slot = &mut self.slots[id.index as usize];
        assert!(
            slot.generation == id.generation && slot.value.is_some(),
            "an id outlived its value"
        );
        slot.generation = slot.generation.checked_add(1).unwrap_or(NonZeroU32::MIN);
        slot.meta = M::default();
        self.vacant.push(id.index);
        slot.value.take().expect("a live slot holds a value")
    }
}

impl<T, M> RcTable<T, M> {
    fn live(&self, id: TableId<T>) -> &RcSlot<T, M> {
        let slot = &self.slots[id.index as usize];
        assert!(
            slot.generation == id.generation && slot.value.is_some(),
            "an id outlived its value"
        );
        slot
    }

    /// `id`'s value.
    ///
    /// # Panics
    /// When `id`'s value is gone.
    pub(crate) fn get(&self, id: TableId<T>) -> &::std::rc::Rc<T> {
        self.live(id)
            .value
            .as_ref()
            .expect("`live` checked the slot is full")
    }

    /// `id`'s value, or `None` when it is gone.
    pub(crate) fn try_get(&self, id: TableId<T>) -> Option<&::std::rc::Rc<T>> {
        let slot = self.slots.get(id.index as usize)?;
        (slot.generation == id.generation)
            .then_some(slot.value.as_ref())
            .flatten()
    }

    /// What was set beside `id`'s value.
    pub(crate) fn meta(&self, id: TableId<T>) -> &M {
        &self.live(id).meta
    }
}

impl<T> TableId<T> {
    /// The id as one integer, for a caller that must carry it through an
    /// address-sized cookie.
    pub(crate) fn to_bits(self) -> usize {
        (usize::try_from(self.generation.get()).expect("32 bits fit") << 32)
            | usize::try_from(self.index).expect("32 bits fit")
    }

    /// The id [`TableId::to_bits`] answered `bits` for, if it is one.
    pub(crate) fn from_bits(bits: usize) -> Option<TableId<T>> {
        let index = u32::try_from(bits & 0xffff_ffff).ok()?;
        let generation = NonZeroU32::new(u32::try_from(bits >> 32).ok()?)?;
        Some(TableId {
            index,
            generation,
            kind: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn boxed<T>(value: T) -> Boxed<T> {
        Box::new(UnsafeCell::new(ManuallyDrop::new(value)))
    }

    #[test]
    fn an_id_names_its_value_until_it_is_removed() {
        let mut table: IdTable<u64, u8> = IdTable::new();
        let (a, pa) = table.insert_boxed(1, boxed(10), |_, _| {});
        let (b, _) = table.insert_boxed(2, boxed(20), |_, _| {});
        assert_ne!(a, b);
        assert_eq!(table.address(a), pa);
        assert_eq!(*table.meta(b), 2);
        table.free(a);
        assert_eq!(*table.meta(b), 2);
        // The slot is reused, under a new generation.
        let (c, _) = table.insert_boxed(3, boxed(0), |id, v| *v = u64::from(id == a));
        assert_ne!(c, a);
        assert_eq!(*table.meta(c), 3);
        table.free(c);
    }

    #[test]
    #[should_panic(expected = "an id outlived its value")]
    fn a_stale_id_panics() {
        let mut table: IdTable<u64> = IdTable::new();
        let (a, _) = table.insert_boxed((), boxed(1), |_, _| {});
        table.free(a);
        let _ = table.insert_boxed((), boxed(2), |_, _| {});
        let _ = table.address(a);
    }

    #[test]
    fn a_value_stays_put_while_the_table_grows() {
        let mut table: IdTable<[u8; 64]> = IdTable::new();
        let (first, at) = table.insert_boxed((), boxed([7; 64]), |_, _| {});
        for _ in 0..100 {
            let _ = table.insert_boxed((), boxed([0; 64]), |_, _| {});
        }
        assert_eq!(table.address(first), at);
    }
}
