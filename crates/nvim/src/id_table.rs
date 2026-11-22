//! A table that owns values at fixed addresses and names them by id.
//!
//! What the editor used to keep as a raw global pointer to a heap object --
//! the exception being thrown, a condition stack, a quickfix stack -- is an
//! id into one of these instead, and the table is the object's one owner:
//!
//! - a [`TableId`] is a slot index plus the slot's generation, which moves
//!   every time the slot is emptied, so an id kept past its value resolves
//!   to a panic rather than to freed memory or to the next tenant;
//! - the values live in a [`SlotPool`]: each in a slot that is never moved
//!   and never given back to the allocator, so the value's address is fixed
//!   and a [`SlotRef`] to it ([`IdTable::view`]) checks the slot's
//!   generation at every dereference -- a view kept past its value panics
//!   instead of reading freed memory;
//! - emptying a slot drops nothing of the value unless the owner asks for
//!   it back ([`IdTable::remove`]): the editor gives back what such an
//!   object holds by hand.
//!
//! The table is not itself a global: an owner keeps one in its own cell.
//!
//! It is not [`crate::registry`]'s `SlotTable`, which files objects under
//! the handle the user sees, in khash's order; nor its `Owned<T>`, which
//! always runs `T`'s destructor. Here nothing user-visible names the value.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::fmt;
use core::marker::PhantomData;
use core::num::NonZeroU32;

use crate::slot::SlotPool;
pub(crate) use crate::slot::{Detached, SlotRef};

/// A value's place in a [`IdTable<T>`] (or an [`RcTable<T>`]).
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

/// Values of type `T`, owned at fixed addresses, named by [`TableId`].
#[derive(Debug)]
pub(crate) struct IdTable<T> {
    pool: SlotPool<T>,
}

impl<T> IdTable<T> {
    /// An empty table.
    pub(crate) const fn new() -> Self {
        IdTable {
            pool: SlotPool::new(),
        }
    }

    /// The id of the value `r` names.
    fn id_of(r: SlotRef<T>) -> TableId<T> {
        TableId {
            index: r.index(),
            generation: NonZeroU32::new(r.generation()).expect("a handed-out generation is odd"),
            kind: PhantomData,
        }
    }

    /// `id`'s value as a handle.
    ///
    /// # Panics
    /// When `id`'s value is gone.
    #[track_caller]
    fn live(&self, id: TableId<T>) -> SlotRef<T> {
        self.pool
            .lookup(id.index, id.generation.get())
            .expect("an id outlived its value")
    }

    /// Take ownership of `value`. Answers its id and its fixed address.
    pub(crate) fn insert(&mut self, value: T) -> (TableId<T>, *mut T) {
        let r = self.pool.insert(value);
        (Self::id_of(r), r.raw())
    }

    /// Empty `id`'s slot, dropping nothing of the value: the caller has
    /// given back what it held.
    ///
    /// # Panics
    /// When `id`'s value is gone already.
    pub(crate) fn free(&mut self, id: TableId<T>) {
        let r = self.live(id);
        self.pool.forget(r);
    }

    /// Empty `id`'s slot and hand its value back, for the caller to give
    /// back what it holds.
    ///
    /// # Panics
    /// When `id`'s value is gone already.
    pub(crate) fn remove(&mut self, id: TableId<T>) -> T {
        let r = self.live(id);
        self.pool.remove(r)
    }

    /// Take `id`'s value out with its slot, for an owner that clears it and
    /// files it again ([`IdTable::attach`]) rather than build the next one.
    ///
    /// # Panics
    /// When `id`'s value is gone already.
    pub(crate) fn detach(&mut self, id: TableId<T>) -> Detached<T> {
        let r = self.live(id);
        self.pool.detach(r)
    }

    /// File a detached value again, under a new id.
    pub(crate) fn attach(&mut self, value: Detached<T>) -> TableId<T> {
        Self::id_of(self.pool.attach(value))
    }

    /// Give a detached value's slot back, answering the value.
    pub(crate) fn release(&mut self, value: Detached<T>) -> T {
        self.pool.release(value)
    }

    /// `id`'s value, borrowed for as long as the table is.
    ///
    /// # Panics
    /// When `id`'s value is gone.
    pub(crate) fn get_mut(&mut self, id: TableId<T>) -> &mut T {
        let r = self.live(id);
        self.pool.get_mut(r)
    }

    /// A view of `id`'s value: a handle that may be held across code that
    /// empties the slot, and panics at its next dereference if it was.
    ///
    /// # Panics
    /// When `id`'s value is gone.
    pub(crate) fn view(&self, id: TableId<T>) -> SlotRef<T> {
        self.live(id)
    }
}

/// Values of type `T`, each shared (`Rc`) with whoever is using it, with an
/// `M` beside it, named by [`TableId`].
///
/// [`IdTable`]'s sibling for a value whose holders keep it alive by sharing
/// rather than by a raw pointer into a fixed box: the table owns one
/// reference, an id resolves to another, and emptying the slot drops the
/// table's -- the value goes when its last holder does.
#[derive(Debug)]
pub(crate) struct RcTable<T, M = ()> {
    slots: Vec<RcSlot<T, M>>,
    vacant: Vec<u32>,
}

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_id_names_its_value_until_it_is_removed() {
        let mut table: IdTable<u64> = IdTable::new();
        let (a, pa) = table.insert(10);
        let (b, _) = table.insert(20);
        assert_ne!(a, b);
        assert_eq!(table.view(a).raw(), pa);
        assert_eq!(*table.get_mut(b), 20);
        let view = table.view(a);
        table.free(a);
        assert!(!view.is_live());
        // The slot is reused, under a new generation.
        let (c, pc) = table.insert(0);
        assert_eq!(pc, pa);
        assert_ne!(c, a);
        assert_eq!(table.remove(c), 0);
        assert_eq!(table.remove(b), 20);
    }

    #[test]
    #[should_panic(expected = "an id outlived its value")]
    fn a_stale_id_panics() {
        let mut table: IdTable<u64> = IdTable::new();
        let (a, _) = table.insert(1);
        table.free(a);
        let _ = table.insert(2);
        let _ = table.view(a);
    }

    #[test]
    fn a_detached_value_is_filed_again_under_a_new_id() {
        let mut table: IdTable<Vec<u8>> = IdTable::new();
        let (a, pa) = table.insert(vec![1]);
        let mut value = table.detach(a);
        value.clear();
        let b = table.attach(value);
        assert_ne!(a, b);
        assert_eq!(table.view(b).raw(), pa);
        let value = table.detach(b);
        assert!(table.release(value).is_empty());
    }
}
