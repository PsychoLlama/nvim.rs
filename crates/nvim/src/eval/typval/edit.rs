//! [`Edit`]: the writable borrow a value handle lends from a shared
//! reference, and the debug builds' check that two of them never overlap.
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

#[cfg(debug_assertions)]
use ::core::ptr::NonNull;

/// A writable borrow of a value object, taken through a shared handle: what
/// [`ListRef::edit`] and its siblings answer.
///
/// Release builds compile it to the `&mut T` it holds and nothing else.
/// Debug builds record the object as lent for as long as the borrow lives,
/// so a second `edit()` of the same object -- through any handle -- or a
/// `DerefMut` of an owning handle over it panics where it happens instead
/// of aliasing: the half of the `Live` promise that two writers break,
/// checked the way `GlobalCell::with_mut` checks a cell. A *read* through
/// another handle meanwhile is not checked.
///
/// The loan is a field of its own rather than a `Drop` on the borrow, so
/// the borrow checker sees the same type in both builds: a binding's
/// borrow ends at its last use, and only the debug loan waits for the end
/// of the scope (drop the binding to end it sooner).
pub struct Edit<'a, T> {
    pub(super) target: &'a mut T,
    #[cfg(debug_assertions)]
    pub(super) loan: Loan,
}

impl<'a, T> Edit<'a, T> {
    /// A borrow of part of the object, under the same loan.
    #[inline(always)]
    pub fn map<U>(self, part: impl FnOnce(&'a mut T) -> &'a mut U) -> Edit<'a, U> {
        let Edit {
            target,
            #[cfg(debug_assertions)]
            loan,
        } = self;
        Edit {
            target: part(target),
            #[cfg(debug_assertions)]
            loan,
        }
    }

    /// [`Edit::map`] for a part that may be absent.
    #[inline(always)]
    pub fn filter_map<U>(
        self,
        part: impl FnOnce(&'a mut T) -> Option<&'a mut U>,
    ) -> Option<Edit<'a, U>> {
        let Edit {
            target,
            #[cfg(debug_assertions)]
            loan,
        } = self;
        Some(Edit {
            target: part(target)?,
            #[cfg(debug_assertions)]
            loan,
        })
    }
}

impl<T> ::core::ops::Deref for Edit<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        self.target
    }
}

impl<T> ::core::ops::DerefMut for Edit<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        self.target
    }
}

/// An [`Edit`]'s record in the debug builds' loan table: the object's
/// address, given back when the edit goes.
#[cfg(debug_assertions)]
pub(super) struct Loan(usize);

#[cfg(debug_assertions)]
impl Loan {
    /// Record `at` as lent writable; a second loan of it is the aliasing
    /// the `Live` promise forbids.
    #[track_caller]
    pub(super) fn new<T>(at: NonNull<T>) -> Loan {
        assert!(
            nvim_mainthread::EditTable::lend(at.as_ptr().addr()),
            "a {} edited through two handles at once ({at:p})",
            ::core::any::type_name::<T>(),
        );
        Loan(at.as_ptr().addr())
    }
}

#[cfg(debug_assertions)]
impl Drop for Loan {
    fn drop(&mut self) {
        nvim_mainthread::EditTable::give_back(self.0);
    }
}

/// An owning handle's `DerefMut` while an [`Edit`] of the same object lives
/// is the same aliasing.
#[cfg(debug_assertions)]
#[track_caller]
pub(super) fn assert_not_lent<T>(at: NonNull<T>) {
    assert!(
        !nvim_mainthread::EditTable::is_lent(at.as_ptr().addr()),
        "a {} written through its handle while edit() lends it ({at:p})",
        ::core::any::type_name::<T>(),
    );
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::global_cell::editor_state_lock;

    /// Two live edits of one object, through two handles, are the aliasing
    /// the `Live` promise forbids: debug builds stop at the second.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "edited through two handles at once")]
    fn a_second_edit_of_one_object_panics_in_debug_builds() {
        let _held = editor_state_lock();
        let list = tv_list_alloc(0);
        let other = list.clone();
        let _first = list.edit();
        let _second = other.edit();
    }

    /// An owning handle's `DerefMut` while an edit lends the object is the
    /// same aliasing.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "while edit() lends it")]
    fn a_deref_mut_under_an_edit_panics_in_debug_builds() {
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
        let mut first = other.edit().map(|l| &mut l.lv_items[0]);
        first.li_lock = VarLock::Locked;
        drop(first);
        assert_eq!(list.edit().lv_items[0].li_lock, VarLock::Locked);
    }
}
