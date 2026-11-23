//! [`Edit`] and [`Read`]: the writable and the shared borrow a value handle
//! lends, and the [`Lent`] word every build checks so that a writable one
//! never overlaps another borrow -- `RefCell`'s rule, kept in the object.
//!
//! The lending itself -- turning the handle's address into a reference --
//! is the handle file's (`handle.rs`); everything here is safe code.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::handle::{Loan, Share};
use crate::deref::deref_to_field;
use crate::eval::vars::flush_message_rows;
use ::core::cell::Cell;

/// A writable borrow of a value object, taken through a shared handle: what
/// [`ListRef::edit`](super::ListRef::edit) and its siblings answer.
///
/// Every build records the object as lent for as long as the borrow lives
/// (the object's [`Lent`] flag), so a second `edit()` of the same object --
/// through any handle -- or a `DerefMut` of an owning handle over it panics
/// where it happens instead of aliasing: the half of the `Live` promise
/// that two writers break, checked the way `GlobalCell::with_mut` checks a
/// cell. A [`Read`] out while it starts, or one asked for while it lives,
/// panics the same way.
///
/// The loan waits for the end of the scope (drop the binding to end it
/// sooner). The object is held by address, not as a `&mut` field: an edit
/// passed by value (`drop(edit)`) has its reference fields retagged as
/// protected for the call, and the loan's release writes the word inside
/// the object under it. `Deref`, `DerefMut` and the projections are the
/// handle file's.
#[clippy::has_significant_drop]
#[derive(Debug)]
pub struct Edit<'a, T> {
    pub(super) target: ::core::ptr::NonNull<T>,
    pub(super) loan: Loan<'a>,
    pub(super) borrow: ::core::marker::PhantomData<&'a mut T>,
}

/// A shared borrow of a value object, taken through a handle: what
/// [`BlobRef::read`](super::BlobRef::read) and its siblings answer.
///
/// Every build counts it in the object's [`Lent`] word for as long as it
/// lives, so an [`Edit`] of the same object -- through any handle -- panics
/// while it is out, instead of writing under it. A body that runs user
/// code, which can reach the object through another holder, takes its
/// `Read` per step and lets it go before the call.
#[clippy::has_significant_drop]
#[derive(Debug)]
pub struct Read<'a, T: ?Sized> {
    pub(super) target: &'a T,
    pub(super) share: Option<Share>,
}

impl<'a, T: ?Sized> Read<'a, T> {
    /// A borrow of nothing any handle lends: a constant, or an absent
    /// object's empty stand-in.
    #[inline(always)]
    pub const fn unshared(target: &'a T) -> Read<'a, T> {
        Read {
            target,
            share: None,
        }
    }

    /// A borrow of part of the object, under the same share.
    #[inline(always)]
    pub fn map<U: ?Sized>(mut self, part: impl FnOnce(&'a T) -> &'a U) -> Read<'a, U> {
        Read {
            target: part(self.target),
            share: self.share.take(),
        }
    }
}

/// Empty, and there to be there: a `Read` with a destructor keeps the
/// borrow of its source alive until the share it holds goes, so the
/// compiler refuses a `&mut` of the holder (`tv_clear(tv)`) under it
/// rather than the count refusing it at run time.
impl<T: ?Sized> Drop for Read<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {}
}

deref_to_field!(impl['a, T: ?Sized] Read<'a, T> => T, target);

/// Shares kept for as long as the scope lives, so that a borrow taken under
/// one can be a plain reference of the scope's lifetime: what a walk over a
/// tree of containers holds, each container's items borrowed from its
/// parent's (the encoders' explicit stack).
///
/// [`ReadScope::hold`] answers `&'s T` from a [`Read`] whose share it keeps
/// until the scope is dropped -- which the borrow `'s` of the scope rules
/// out while the reference lives. Every object read under it refuses an
/// edit until then, so a scope never spans a call that runs user code.
///
/// A message put out while a scope holds `v:` writes its row (`v:errmsg`)
/// once the scope ends, as under any other read of `v:`
/// ([`set_message_row`](crate::eval::vars::set_message_row)).
#[derive(Debug)]
pub(crate) struct ReadScope(::core::cell::RefCell<Vec<Share>>);

impl ReadScope {
    pub(crate) fn new() -> ReadScope {
        ReadScope(::core::cell::RefCell::new(Vec::new()))
    }

    /// The object `read` borrows, for the life of the scope.
    #[inline]
    pub(crate) fn hold<'s, T: ?Sized>(&'s self, read: Read<'s, T>) -> &'s T {
        let mut read = read;
        if let Some(share) = read.share.take() {
            self.0.borrow_mut().push(share);
        }
        read.target
    }
}

impl Drop for ReadScope {
    fn drop(&mut self) {
        // The shares first: one may be `v:`'s, which the rows wait on.
        self.0.get_mut().clear();
        flush_message_rows();
    }
}

/// How a value object (list, dictionary, blob) is borrowed through its
/// handles right now: free, lent writable to one [`Edit`], or shared by a
/// count of [`Read`]s. A field of the object itself, so checking it is a
/// load beside the data the borrow is about to touch, not a lookup.
///
/// Only a borrow changes it, and only its own guard undoes the change; the
/// handle file reaches it through the field's own bytes, never through a
/// borrow of the whole object, which an edit holds.
#[derive(Debug, Default)]
pub struct Lent(Cell<u32>);

/// [`Lent`]'s word while an [`Edit`] holds the object; below it, the count
/// of [`Read`]s.
const WRITING: u32 = u32::MAX;

impl Lent {
    /// Not lent: what every object is made with.
    pub const fn new() -> Lent {
        Lent(Cell::new(0))
    }

    /// Lend the object writable; any other borrow of it out meanwhile is
    /// the aliasing the `Live` promise forbids.
    #[inline(always)]
    #[track_caller]
    pub(super) fn lend<T>(&self) {
        let state = self.0.get();
        if state != 0 {
            refused::<T>("edited", state, self);
        }
        self.0.set(WRITING);
    }

    /// The edit has gone.
    #[inline(always)]
    pub(super) fn give_back(&self) {
        self.0.set(0);
    }

    /// Share the object; an edit of it out meanwhile is the same aliasing.
    #[inline(always)]
    #[track_caller]
    pub(super) fn share<T>(&self) {
        let state = self.0.get();
        // One compare for both refusals: an edit, and a count that would
        // reach the edit's word.
        if state >= WRITING - 1 {
            refused::<T>("read", state, self);
        }
        self.0.set(state + 1);
    }

    /// Whether no borrow holds the object: an `edit()` now would not be
    /// refused.
    #[inline(always)]
    pub(super) fn is_free(&self) -> bool {
        self.0.get() == 0
    }

    /// One read has gone.
    #[inline(always)]
    pub(super) fn unshare(&self) {
        self.0.set(self.0.get() - 1);
    }
}

/// A refused borrow's panic, out of every borrow's line: `what` was asked
/// of the object while `state` held it.
#[cold]
#[inline(never)]
#[track_caller]
fn refused<T>(what: &str, state: u32, at: &Lent) -> ! {
    let ty = ::core::any::type_name::<T>();
    match state {
        WRITING => panic!("a {ty} {what} while edit() lends it ({at:p})"),
        _ if what == "read" => panic!("a {ty} read by too many borrows at once ({at:p})"),
        readers => panic!("a {ty} {what} while {readers} read() borrows hold it ({at:p})"),
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::global_cell::editor_state_lock;

    /// Two live edits of one object, through two handles, are the aliasing
    /// the `Live` promise forbids: every build stops at the second.
    #[test]
    #[should_panic(expected = "edited while edit() lends it")]
    fn a_second_edit_of_one_object_panics() {
        let _held = editor_state_lock();
        let list = tv_list_alloc(0);
        let other = list.clone();
        let _first = list.edit();
        let _second = other.edit();
    }

    /// An edit of an object a read holds, through another handle, is the
    /// read/write aliasing: every build stops at the edit.
    #[test]
    #[should_panic(expected = "edited while 1 read() borrows hold it")]
    fn an_edit_under_a_read_panics() {
        let _held = editor_state_lock();
        let blob = tv_blob_alloc();
        let other = blob.clone();
        let _read = blob.read();
        other.edit().push(1);
    }

    /// A read of an object an edit holds is the same aliasing.
    #[test]
    #[should_panic(expected = "read while edit() lends it")]
    fn a_read_under_an_edit_panics() {
        let _held = editor_state_lock();
        let blob = tv_blob_alloc();
        let other = blob.clone();
        let _edit = blob.edit();
        let _ = other.read().len();
    }

    /// Reads share: any number nest, a part mapped out of one keeps the
    /// share, and once the last goes the object is free to edit.
    #[test]
    fn reads_nest_and_end_with_their_last_guard() {
        let _held = editor_state_lock();
        let blob = tv_blob_alloc();
        blob.edit().extend(b"ab");
        let other = blob.clone();
        let first = blob.read();
        let bytes = other.read().map(Blob::bytes);
        assert_eq!((&*bytes, first.len()), (&b"ab"[..], 2));
        drop(first);
        let reading = ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| {
            other.edit().push(b'c');
        }));
        assert!(reading.is_err(), "the mapped part still shares the blob");
        drop(bytes);
        other.edit().push(b'c');
        assert_eq!(blob.read().bytes(), b"abc");
    }

    /// An edit's loan ends with it: edits one after another, and a part
    /// mapped out of one, are fine.
    #[test]
    fn edits_in_sequence_do_not_overlap() {
        let _held = editor_state_lock();
        let list = tv_list_alloc(0);
        list.edit().push_number(1);
        let other = list.clone();
        list.edit().lv_lock = VarLock::Locked;
        other.edit().lv_lock = VarLock::Unlocked;
        {
            let mut first = other.edit().map(|l| &mut l.lv_items[0]);
            first.li_lock = VarLock::Locked;
        }
        assert_eq!(list.edit().lv_items[0].li_lock, VarLock::Locked);
    }

    /// An edit handed by value to a function that ends it: the loan's
    /// release writes the word inside the object, which a `&mut` field
    /// protected for the call would forbid (Miri).
    #[test]
    fn an_edit_moved_into_a_call_gives_its_loan_back() {
        let _held = editor_state_lock();
        let list = tv_list_alloc(0);
        let mut edit = list.edit();
        edit.push_number(1);
        drop(edit);
        let part = list.edit().map(|l| &mut l.lv_items[0]);
        drop(part);
        assert_eq!(list.read().len(), 1);
    }
}
