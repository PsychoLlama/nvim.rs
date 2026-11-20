//! [`Edit`]: the writable borrow a value handle lends from a shared
//! reference, and the [`Lent`] flag every build checks so that two of them
//! never overlap.
//!
//! The lending itself -- turning the handle's address into a `&mut` -- is
//! the handle file's (`handle.rs`); everything here is safe code.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::handle::Loan;
use crate::deref::deref_to_field;
use ::core::cell::Cell;

/// A writable borrow of a value object, taken through a shared handle: what
/// [`ListRef::edit`](super::ListRef::edit) and its siblings answer.
///
/// Every build records the object as lent for as long as the borrow lives
/// (the object's [`Lent`] flag), so a second `edit()` of the same object --
/// through any handle -- or a `DerefMut` of an owning handle over it panics
/// where it happens instead of aliasing: the half of the `Live` promise
/// that two writers break, checked the way `GlobalCell::with_mut` checks a
/// cell. A *read* through another handle meanwhile is not checked.
///
/// The loan is a field of its own rather than a `Drop` on the borrow: a
/// binding's borrow of the object ends at its last use, and only the loan
/// waits for the end of the scope (drop the binding to end it sooner).
#[derive(Debug)]
pub struct Edit<'a, T> {
    pub(super) target: &'a mut T,
    pub(super) loan: Loan,
}

impl<'a, T> Edit<'a, T> {
    /// A borrow of part of the object, under the same loan.
    #[inline(always)]
    pub fn map<U>(self, part: impl FnOnce(&'a mut T) -> &'a mut U) -> Edit<'a, U> {
        let Edit { target, loan } = self;
        Edit {
            target: part(target),
            loan,
        }
    }

    /// [`Edit::map`] for a part that may be absent.
    #[inline(always)]
    pub fn filter_map<U>(
        self,
        part: impl FnOnce(&'a mut T) -> Option<&'a mut U>,
    ) -> Option<Edit<'a, U>> {
        let Edit { target, loan } = self;
        Some(Edit {
            target: part(target)?,
            loan,
        })
    }
}

deref_to_field!(impl[T] Edit<'_, T> => mut T, target);

/// Whether a value object (list, dictionary, blob, partial) is lent out
/// writable through an [`Edit`] right now: a field of the object itself, so
/// checking it is a load beside the data the edit is about to touch, not a
/// lookup.
///
/// Only an edit sets it, and only its loan clears it; the handle file
/// reaches it through the field's own bytes, never through a borrow of the
/// whole object, which the edit holds.
#[derive(Debug, Default)]
pub struct Lent(Cell<bool>);

impl Lent {
    /// Not lent: what every object is made with.
    pub const fn new() -> Lent {
        Lent(Cell::new(false))
    }

    /// Mark the object lent; a second loan of it is the aliasing the
    /// `Live` promise forbids.
    #[inline(always)]
    #[track_caller]
    pub(super) fn lend<T>(&self) {
        if self.0.replace(true) {
            lent_twice(::core::any::type_name::<T>(), self);
        }
    }

    /// The loan has gone.
    #[inline(always)]
    pub(super) fn give_back(&self) {
        self.0.set(false);
    }

    /// An owning handle's `DerefMut` while an [`Edit`] of the same object
    /// lives is the same aliasing.
    #[inline(always)]
    #[track_caller]
    pub(super) fn assert_not_lent<T>(&self) {
        if self.0.get() {
            written_under_edit(::core::any::type_name::<T>(), self);
        }
    }
}

/// [`Lent::lend`]'s panic, out of every `edit()`'s line.
#[cold]
#[inline(never)]
#[track_caller]
fn lent_twice(what: &str, at: &Lent) -> ! {
    panic!("a {what} edited through two handles at once ({at:p})");
}

/// [`Lent::assert_not_lent`]'s panic.
#[cold]
#[inline(never)]
#[track_caller]
fn written_under_edit(what: &str, at: &Lent) -> ! {
    panic!("a {what} written through its handle while edit() lends it ({at:p})");
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::global_cell::editor_state_lock;

    /// Two live edits of one object, through two handles, are the aliasing
    /// the `Live` promise forbids: every build stops at the second.
    #[test]
    #[should_panic(expected = "edited through two handles at once")]
    fn a_second_edit_of_one_object_panics() {
        let _held = editor_state_lock();
        let list = tv_list_alloc(0);
        let other = list.clone();
        let _first = list.edit();
        let _second = other.edit();
    }

    /// An owning handle's `DerefMut` while an edit lends the object is the
    /// same aliasing.
    #[test]
    #[should_panic(expected = "while edit() lends it")]
    fn a_deref_mut_under_an_edit_panics() {
        let _held = editor_state_lock();
        let mut dict = tv_dict_alloc();
        let other = dict.clone();
        let _edit = other.edit();
        dict.dv_lock = VarLock::Locked;
    }

    /// An edit's loan ends with it: edits one after another, and a part
    /// mapped out of one, are fine.
    #[test]
    fn edits_in_sequence_do_not_overlap() {
        let _held = editor_state_lock();
        let mut list = tv_list_alloc(0);
        list.push_number(1);
        let other = list.clone();
        list.edit().lv_lock = VarLock::Locked;
        other.edit().lv_lock = VarLock::Unlocked;
        {
            let mut first = other.edit().map(|l| &mut l.lv_items[0]);
            first.li_lock = VarLock::Locked;
        }
        assert_eq!(list.edit().lv_items[0].li_lock, VarLock::Locked);
    }
}
