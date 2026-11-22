//! The guards a slot lends its value through: [`edit_at`]'s mutable loan,
//! [`read_at`]'s counted borrow, the parts each narrows to, and
//! [`lend_part_at`]'s loan of one interior part. See the parent module's
//! "Reads and edits".

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `slot.rs`, `slot/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::fmt;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use super::{SLOT_EDIT_BIT, Slot, header, index_of, readers, slot_of, stale_slot, value_ptr};

/// Why a slot refused a handle, for the message: gone, lent out to an edit,
/// or read by a guard that the asked-for write would invalidate.
#[cold]
#[inline(never)]
#[track_caller]
pub(super) fn refused<T>(word: u32, generation: u32, readers: u32) -> ! {
    let type_name = core::any::type_name::<T>();
    if word == generation | SLOT_EDIT_BIT {
        panic!(
            "a {type_name} was reached while an edit of it was held: an `edit()` guard lends \
             it to nothing else, through any copy of its handle, until it is dropped"
        )
    }
    if word != generation {
        stale_slot(type_name)
    }
    panic!(
        "a {type_name} was written or freed while {readers} `read()` guard(s) of it were held: \
         a read guard's borrow may not be written under"
    )
}

/// Panic unless the value at `value` is the one handed out under
/// `generation`, no edit of it is out, and no read guard of it is out:
/// what a field written in place through a handle asks first.
///
/// # Safety
/// As [`generation_at`].
#[inline(always)]
#[track_caller]
pub(crate) unsafe fn check_write<T>(value: *const T, generation: u32) {
    // SAFETY: the caller's promise.
    let slot = unsafe { slot_of(value) };
    let (word, readers) = (header(slot).get(), readers(slot).get());
    if word != generation || readers != 0 {
        refused::<T>(word, generation, readers);
    }
}

/// Lend the value at `value` mutably: the slot's word carries the edit bit
/// until the guard is dropped, so every check through any copy of any
/// handle -- a read, a write, another edit, emptying the slot -- refuses
/// meanwhile, and the guard's borrow is the only one.
///
/// # Panics
/// When the value is gone, already lent to an edit, or read by a guard.
///
/// # Safety
/// As [`generation_at`].
#[inline]
#[track_caller]
pub(crate) unsafe fn edit_at<'a, T>(value: *mut T, generation: u32) -> SlotEdit<'a, T> {
    // SAFETY: the caller's promise.
    let slot = unsafe { slot_of(value) };
    let (word, readers) = (header(slot).get(), readers(slot).get());
    if word != generation || readers != 0 {
        refused::<T>(word, generation, readers);
    }
    header(slot).set(generation | SLOT_EDIT_BIT);
    SlotEdit {
        slot,
        generation,
        _lent: PhantomData,
    }
}

/// Borrow the value at `value` for as long as the guard is held, counted:
/// edits, field writes and emptying the slot refuse until it is dropped.
///
/// # Panics
/// When the value is gone or lent to an edit.
///
/// # Safety
/// As [`generation_at`].
#[inline]
#[track_caller]
pub(crate) unsafe fn read_at<'a, T>(value: *mut T, generation: u32) -> SlotRead<'a, T> {
    // SAFETY: the caller's promise.
    let slot = unsafe { slot_of(value) };
    let (word, count) = (header(slot).get(), readers(slot).get());
    if word != generation {
        refused::<T>(word, generation, count);
    }
    readers(slot).set(count.checked_add(1).expect("fewer than 2^32 read guards"));
    SlotRead {
        slot,
        _shared: PhantomData,
    }
}

/// The bit a part loan ([`lend_part_at`]) sets in its slot's reader count
/// for as long as it lends the value's one interior part. A part loan also
/// counts as a reader, so an edit, a field write and emptying the slot
/// refuse while it is out, as they do for a [`SlotRead`]; a second part
/// loan and a part read ([`check_part_unlent`]) refuse on the bit. Reads of
/// the rest of the value stay welcome.
///
/// A count that reached `2^31` read guards would set it spuriously, which
/// only refuses: no loan is ever granted on a bit it did not set.
const SLOT_PART_BIT: u32 = 0x8000_0000;

/// Why a part loan or a part read refused, for the message.
#[cold]
#[inline(never)]
#[track_caller]
fn refused_part<T>(word: u32, generation: u32, readers: u32) -> ! {
    if word != generation {
        refused::<T>(word, generation, readers & !SLOT_PART_BIT);
    }
    panic!(
        "a part of a {} was reached while a loan of it was held: the loan lends it to nothing \
         else until it is dropped",
        core::any::type_name::<T>()
    )
}

/// Lend `part` -- the value's one part inside an `UnsafeCell`, at `value`'s
/// slot -- mutably for as long as the guard is held: the slot's reader count
/// carries [`SLOT_PART_BIT`] and one reader, so the value stays alive and
/// unedited, reads of the rest of it go on, and the part is lent to nothing
/// else.
///
/// # Panics
/// When the value is gone or lent to an edit, or the part is already lent.
///
/// # Safety
/// `value` as [`generation_at`]. `part` must point into an `UnsafeCell` of
/// the value at `value`, derived from `value` with its provenance, and every
/// way to it other than through a [`SlotEdit`] must be this loan or a read
/// [`check_part_unlent`] allowed.
#[inline(always)]
#[track_caller]
pub(crate) unsafe fn lend_part_at<'a, T, U>(
    value: *mut T,
    generation: u32,
    part: NonNull<U>,
) -> SlotPart<'a, T, U> {
    // SAFETY: the caller's promise.
    let slot = unsafe { slot_of(value) };
    let (word, count) = (header(slot).get(), readers(slot).get());
    if word != generation || count & SLOT_PART_BIT != 0 {
        refused_part::<T>(word, generation, count);
    }
    readers(slot).set(
        count
            .checked_add(1 | SLOT_PART_BIT)
            .expect("fewer than 2^31 read guards"),
    );
    SlotPart {
        slot,
        part,
        _lent: PhantomData,
    }
}

/// Panic unless the value at `value` is the one handed out under
/// `generation`, no edit of it is out, and its part is lent to no
/// [`lend_part_at`] loan: what a read of the part by value asks first.
///
/// # Safety
/// As [`generation_at`].
#[inline(always)]
#[track_caller]
pub(crate) unsafe fn check_part_unlent<T>(value: *const T, generation: u32) {
    // SAFETY: the caller's promise.
    let slot = unsafe { slot_of(value) };
    let (word, count) = (header(slot).get(), readers(slot).get());
    if word != generation || count & SLOT_PART_BIT != 0 {
        refused_part::<T>(word, generation, count);
    }
}

/// A value's part lent by [`lend_part_at`]: its slot carries the part bit
/// and one reader until this is dropped.
pub(crate) struct SlotPart<'a, T, U> {
    slot: NonNull<Slot<T>>,
    part: NonNull<U>,
    _lent: PhantomData<&'a mut U>,
}

impl<T, U> Deref for SlotPart<'_, T, U> {
    type Target = U;

    #[inline(always)]
    fn deref(&self) -> &U {
        // SAFETY: live and unedited (`lend_part_at` checked), the reader
        // keeps it so, and the part bit refuses every other way to the part.
        unsafe { self.part.as_ref() }
    }
}

impl<T, U> DerefMut for SlotPart<'_, T, U> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut U {
        // SAFETY: as `deref`; `&mut self` is the one way in.
        unsafe { self.part.as_mut() }
    }
}

impl<T, U> Drop for SlotPart<'_, T, U> {
    #[inline(always)]
    fn drop(&mut self) {
        let count = readers(self.slot);
        debug_assert!(count.get() & SLOT_PART_BIT != 0);
        count.set(count.get() - (1 | SLOT_PART_BIT));
    }
}

impl<T, U> SlotPart<'_, T, U> {
    /// The lent part's address, for asking which part a loan is of.
    #[inline(always)]
    pub(crate) fn part_ptr(&self) -> *const U {
        self.part.as_ptr()
    }
}

impl<T, U> fmt::Debug for SlotPart<'_, T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SlotPart({} @ {:p})", index_of(self.slot), self.slot)
    }
}

/// A value lent mutably by [`edit_at`]. Holds the slot's address rather than
/// a `&mut T`: a reference field would be retagged as protected when the
/// guard is moved into a call (`drop(edit)`), and the drop's write of the
/// header beside it would then be undefined behaviour.
pub(crate) struct SlotEdit<'a, T> {
    slot: NonNull<Slot<T>>,
    generation: u32,
    _lent: PhantomData<&'a mut T>,
}

impl<T> Deref for SlotEdit<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        // SAFETY: live (`edit_at` checked), and the edit bit refuses every
        // other way to the value until this guard is dropped.
        unsafe { &*value_ptr(self.slot) }
    }
}

impl<T> DerefMut for SlotEdit<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: as `deref`; `&mut self` is the one way in.
        unsafe { &mut *value_ptr(self.slot) }
    }
}

impl<T> Drop for SlotEdit<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        debug_assert_eq!(header(self.slot).get(), self.generation | SLOT_EDIT_BIT);
        header(self.slot).set(self.generation);
    }
}

impl<T> fmt::Debug for SlotEdit<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SlotEdit({} @ {:p})", index_of(self.slot), self.slot)
    }
}

/// A value borrowed by [`read_at`]: counted in its slot until dropped.
pub(crate) struct SlotRead<'a, T> {
    slot: NonNull<Slot<T>>,
    _shared: PhantomData<&'a T>,
}

impl<T> Deref for SlotRead<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        // SAFETY: live and not lent to an edit (`read_at` checked), and the
        // count refuses edits, writes and emptying until this is dropped.
        unsafe { &*value_ptr(self.slot) }
    }
}

impl<T> Drop for SlotRead<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        let count = readers(self.slot);
        count.set(count.get() - 1);
    }
}

impl<T> fmt::Debug for SlotRead<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SlotRead({} @ {:p})", index_of(self.slot), self.slot)
    }
}

impl<'a, T> SlotEdit<'a, T> {
    /// Narrow the guard to one part of the value -- a field, an element --
    /// still holding the whole value lent: the part's borrow is the guard's.
    #[inline(always)]
    pub(crate) fn map<U>(mut self, part: impl FnOnce(&mut T) -> &mut U) -> EditPart<'a, T, U> {
        let part = NonNull::from(part(&mut self));
        EditPart { _edit: self, part }
    }
}

impl<'a, T> SlotEdit<'a, T> {
    /// [`SlotEdit::map`] for a part that may be missing: `None` lets the
    /// edit go.
    #[inline(always)]
    pub(crate) fn try_map<U>(
        mut self,
        part: impl FnOnce(&mut T) -> Option<&mut U>,
    ) -> Option<EditPart<'a, T, U>> {
        let part = NonNull::from(part(&mut self)?);
        Some(EditPart { _edit: self, part })
    }
}

impl<'a, T> SlotRead<'a, T> {
    /// Narrow the read to one part of the value, when it has one: the part
    /// stays counted as the whole; `None` lets the read go.
    #[inline(always)]
    pub(crate) fn try_map<U>(
        self,
        part: impl FnOnce(&T) -> Option<&U>,
    ) -> Option<ReadPart<'a, T, U>> {
        let part = NonNull::from(part(&self)?);
        Some(ReadPart { _read: self, part })
    }
}

/// A part of a value lent by [`edit_at`] ([`SlotEdit::map`]): the edit is
/// held until the part is dropped. The part is kept as an address taken from
/// the guard's own borrow, which nothing else can reach while the edit bit is
/// set.
pub(crate) struct EditPart<'a, T, U> {
    _edit: SlotEdit<'a, T>,
    part: NonNull<U>,
}

impl<T, U> Deref for EditPart<'_, T, U> {
    type Target = U;

    #[inline(always)]
    fn deref(&self) -> &U {
        // SAFETY: borrowed from the value `edit` lends, which is held.
        unsafe { self.part.as_ref() }
    }
}

impl<T, U> DerefMut for EditPart<'_, T, U> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut U {
        // SAFETY: as `deref`; `&mut self` is the one way in.
        unsafe { self.part.as_mut() }
    }
}

/// A part of a value borrowed by [`read_at`] ([`SlotRead::try_map`]): counted
/// until dropped.
pub(crate) struct ReadPart<'a, T, U> {
    _read: SlotRead<'a, T>,
    part: NonNull<U>,
}

impl<T, U> Deref for ReadPart<'_, T, U> {
    type Target = U;

    #[inline(always)]
    fn deref(&self) -> &U {
        // SAFETY: borrowed from the value `read` counts, which is held.
        unsafe { self.part.as_ref() }
    }
}

#[cfg(test)]
mod tests {
    use core::cell::UnsafeCell;
    use core::mem::offset_of;

    use super::*;
    use crate::global_cell::GlobalCell;
    use crate::slot::{SlotPool, SlotRef, generation_at};

    fn edit_of<T>(r: SlotRef<T>) -> SlotEdit<'static, T> {
        // SAFETY: a handle's address is its slot's value address.
        unsafe { edit_at(r.raw(), r.generation()) }
    }

    fn read_of<T>(r: SlotRef<T>) -> SlotRead<'static, T> {
        // SAFETY: as `edit_of`.
        unsafe { read_at(r.raw(), r.generation()) }
    }

    fn write_check_of<T>(r: SlotRef<T>) {
        // SAFETY: as `edit_of`.
        unsafe { check_write(r.raw(), r.generation()) }
    }

    #[test]
    fn an_edit_refuses_every_other_way_in_until_it_is_dropped() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        static POOL: GlobalCell<SlotPool<Vec<u32>>> = GlobalCell::new(SlotPool::new());
        let a = POOL.with_mut(|pool| pool.insert(vec![1]));
        let copy = a;
        {
            let mut edit = edit_of(a);
            edit.push(2);
            assert!(copy.is_live(), "lent out, not gone");
            assert!(catch_unwind(AssertUnwindSafe(|| copy.len())).is_err());
            assert!(catch_unwind(AssertUnwindSafe(|| drop(edit_of(a)))).is_err());
            assert!(catch_unwind(AssertUnwindSafe(|| drop(read_of(a)))).is_err());
            assert!(catch_unwind(AssertUnwindSafe(|| write_check_of(a))).is_err());
            // SAFETY: `a.raw()` is a slot's value address.
            assert_eq!(unsafe { generation_at(a.raw()) }, a.generation());
            edit.push(3);
        }
        assert_eq!(*copy, [1, 2, 3]);
        drop(POOL.with_mut(|pool| pool.remove(a)));
    }

    #[test]
    fn a_read_refuses_writes_and_emptying_until_the_last_one_is_dropped() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        static POOL: GlobalCell<SlotPool<Vec<u32>>> = GlobalCell::new(SlotPool::new());
        let a = POOL.with_mut(|pool| pool.insert(vec![4]));
        let first = read_of(a);
        let second = read_of(a);
        assert_eq!((first[0], second[0], a[0]), (4, 4, 4));
        assert!(catch_unwind(AssertUnwindSafe(|| drop(edit_of(a)))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| write_check_of(a))).is_err());
        POOL.with_mut(|pool| {
            assert!(catch_unwind(AssertUnwindSafe(|| pool.forget(a))).is_err());
        });
        drop(first);
        assert!(catch_unwind(AssertUnwindSafe(|| write_check_of(a))).is_err());
        drop(second);
        edit_of(a).push(5);
        assert_eq!(*a, [4, 5]);
        drop(POOL.with_mut(|pool| pool.remove(a)));
    }

    /// A value with one part in a cell of its own, as a buffer's memline.
    struct WithPart {
        head: u32,
        part: UnsafeCell<Vec<u32>>,
    }

    fn part_of(r: SlotRef<WithPart>) -> SlotPart<'static, WithPart, Vec<u32>> {
        let place = UnsafeCell::raw_get(
            r.raw()
                .wrapping_byte_add(offset_of!(WithPart, part))
                .cast::<UnsafeCell<Vec<u32>>>(),
        );
        // SAFETY: as `edit_of`; the part is the value's one cell, reached
        // here only through this loan or an edit.
        unsafe { lend_part_at(r.raw(), r.generation(), NonNull::new(place).unwrap()) }
    }

    fn part_check_of<T>(r: SlotRef<T>) {
        // SAFETY: as `edit_of`.
        unsafe { check_part_unlent(r.raw(), r.generation()) }
    }

    #[test]
    fn a_part_loan_lends_the_part_alone_and_refuses_edits_until_dropped() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        static POOL: GlobalCell<SlotPool<WithPart>> = GlobalCell::new(SlotPool::new());
        let a = POOL.with_mut(|pool| {
            pool.insert(WithPart {
                head: 1,
                part: UnsafeCell::new(vec![2]),
            })
        });
        part_check_of(a);
        let mut part = part_of(a);
        part.push(3);
        // The rest of the value reads on, guard or no guard.
        let read = read_of(a);
        assert_eq!((read.head, a.head), (1, 1));
        assert!(catch_unwind(AssertUnwindSafe(|| drop(part_of(a)))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| part_check_of(a))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| drop(edit_of(a)))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| write_check_of(a))).is_err());
        drop(read);
        // Still a reader: no write, no emptying.
        assert!(catch_unwind(AssertUnwindSafe(|| write_check_of(a))).is_err());
        POOL.with_mut(|pool| {
            assert!(catch_unwind(AssertUnwindSafe(|| pool.forget(a))).is_err());
        });
        part.push(4);
        drop(part);
        part_check_of(a);
        let mut edit = edit_of(a);
        assert_eq!(*edit.part.get_mut(), [2, 3, 4]);
        // An edit refuses a part loan.
        assert!(catch_unwind(AssertUnwindSafe(|| drop(part_of(a)))).is_err());
        drop(edit);
        drop(POOL.with_mut(|pool| pool.remove(a)));
    }

    #[test]
    fn abandoned_loans_let_every_way_in_through_again() {
        use std::panic::{AssertUnwindSafe, catch_unwind};
        static POOL: GlobalCell<SlotPool<WithPart>> = GlobalCell::new(SlotPool::new());
        let [a, b, c] = [1, 2, 3].map(|head| {
            POOL.with_mut(|pool| {
                pool.insert(WithPart {
                    head,
                    part: UnsafeCell::new(Vec::new()),
                })
            })
        });
        // Guards whose owners never come back, as a panic's frames.
        core::mem::forget(edit_of(a));
        core::mem::forget(read_of(b));
        core::mem::forget(read_of(b));
        core::mem::forget(part_of(c));
        assert!(catch_unwind(AssertUnwindSafe(|| drop(read_of(a)))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| write_check_of(b))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| drop(part_of(c)))).is_err());
        POOL.with(SlotPool::abandon_loans);
        assert!(a.is_live() && b.is_live() && c.is_live());
        edit_of(a).head = 10;
        write_check_of(b);
        part_of(c).push(5);
        assert_eq!((a.head, b.head), (10, 2));
        assert_eq!(*edit_of(c).part.get_mut(), [5]);
        POOL.with_mut(|pool| {
            for r in [a, b, c] {
                drop(pool.remove(r));
            }
        });
    }
}
