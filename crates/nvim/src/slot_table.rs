//! A table that owns values at fixed addresses and names them by id.
//!
//! What the editor used to keep as a raw global pointer to a heap object --
//! the funccall in progress, the exception being thrown -- is an id into one
//! of these instead, and the table is the object's one owner:
//!
//! - a [`SlotId`] is a slot index plus the slot's generation, which moves
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

/// A value's place in a [`SlotTable<T>`].
pub(crate) struct SlotId<T> {
    index: u32,
    generation: NonZeroU32,
    kind: PhantomData<fn() -> T>,
}

impl<T> Clone for SlotId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for SlotId<T> {}

impl<T> PartialEq for SlotId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for SlotId<T> {}

impl<T> fmt::Debug for SlotId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SlotId({}#{})", self.index, self.generation)
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
pub(crate) struct SlotTable<T, M = ()> {
    slots: Vec<Slot<T, M>>,
    /// Empty slots, reused last-emptied first.
    vacant: Vec<u32>,
}

impl<T, M: Default> SlotTable<T, M> {
    /// An empty table.
    pub(crate) const fn new() -> Self {
        SlotTable {
            slots: Vec::new(),
            vacant: Vec::new(),
        }
    }

    /// Take ownership of `value`, already in its box -- for a value too big
    /// to build on the stack and move -- and set `meta` beside it; `init`
    /// finishes the value in place, knowing its id, before anything else can
    /// reach it. Answers the id and the value's fixed address.
    pub(crate) fn insert_boxed(
        &mut self,
        meta: M,
        mut value: Boxed<T>,
        init: impl FnOnce(SlotId<T>, &mut T),
    ) -> (SlotId<T>, *mut T) {
        let id = self.vacant_id();
        init(id, value.get_mut());
        self.fill(id, meta, value)
    }

    /// The id the next value will have.
    fn vacant_id(&mut self) -> SlotId<T> {
        if self.vacant.is_empty() {
            let index = u32::try_from(self.slots.len()).expect("fewer than 2^32 slots");
            self.slots.push(Slot {
                generation: NonZeroU32::MIN,
                value: None,
                meta: M::default(),
            });
            self.vacant.push(index);
        }
        let index = *self.vacant.last().expect("a vacant slot");
        SlotId {
            index,
            generation: self.slots[index as usize].generation,
            kind: PhantomData,
        }
    }

    fn fill(&mut self, id: SlotId<T>, meta: M, value: Boxed<T>) -> (SlotId<T>, *mut T) {
        let popped = self.vacant.pop();
        debug_assert_eq!(popped, Some(id.index));
        let slot = &mut self.slots[id.index as usize];
        slot.meta = meta;
        let value = slot.value.insert(value);
        (id, value.get().cast())
    }

    /// Empty `id`'s slot and give back the value's block, dropping nothing
    /// of the value: the caller has given back what it held.
    ///
    /// # Panics
    /// When `id`'s value is gone already.
    pub(crate) fn free(&mut self, id: SlotId<T>) {
        let value = self.empty(id);
        // Freed in place: moving a large value out only to forget it is a
        // copy for nothing.
        drop(value);
    }

    fn empty(&mut self, id: SlotId<T>) -> Boxed<T> {
        let slot = self.live_mut(id);
        slot.generation = slot.generation.checked_add(1).unwrap_or(NonZeroU32::MIN);
        slot.meta = M::default();
        let value = slot.value.take().expect("a live slot holds a value");
        self.vacant.push(id.index);
        value
    }
}

impl<T, M> SlotTable<T, M> {
    fn live(&self, id: SlotId<T>) -> &Slot<T, M> {
        let slot = &self.slots[id.index as usize];
        assert!(
            slot.generation == id.generation && slot.value.is_some(),
            "an id outlived its value"
        );
        slot
    }

    fn live_mut(&mut self, id: SlotId<T>) -> &mut Slot<T, M> {
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
    pub(crate) fn address(&self, id: SlotId<T>) -> *mut T {
        match &self.live(id).value {
            Some(value) => value.get().cast(),
            None => unreachable!("`live` checked the slot is full"),
        }
    }

    /// What was set beside `id`'s value.
    pub(crate) fn meta(&self, id: SlotId<T>) -> &M {
        &self.live(id).meta
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
        let mut table: SlotTable<u64, u8> = SlotTable::new();
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
        let mut table: SlotTable<u64> = SlotTable::new();
        let (a, _) = table.insert_boxed((), boxed(1), |_, _| {});
        table.free(a);
        let _ = table.insert_boxed((), boxed(2), |_, _| {});
        let _ = table.address(a);
    }

    #[test]
    fn a_value_stays_put_while_the_table_grows() {
        let mut table: SlotTable<[u8; 64]> = SlotTable::new();
        let (first, at) = table.insert_boxed((), boxed([7; 64]), |_, _| {});
        for _ in 0..100 {
            let _ = table.insert_boxed((), boxed([0; 64]), |_, _| {});
        }
        assert_eq!(table.address(first), at);
    }
}
