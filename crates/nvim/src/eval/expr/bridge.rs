//! The bridge from the walkers that still step through their text by
//! pointer into the evaluator, which walks a [`Cursor`].
//!
//! The lvalue parser, `:call`/`:defer` and `:function`'s argument list keep a
//! `*mut *mut c_char` over their command line and enter the evaluator through
//! [`Cur::with_cursor`] at their own boundary. This module goes when they
//! take a `Cursor` themselves.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::charset::skipwhite;
use crate::cstr;
use crate::eval::Cursor;
use core::ffi::c_char;

/// A `*mut *mut c_char` cursor, for the walkers outside the evaluator that
/// still step through their text by pointer -- the lvalue parser, `:let`,
/// function definitions and calls -- and hand the evaluator what is left of
/// it through [`with_cursor`](Cur::with_cursor).
///
/// The same shape as [`Live<T>`](crate::winlayer::Live): **construction is
/// the one unsafe step**, and every byte read after it is checked against
/// the constructor's promise.
#[derive(Clone, Copy)]
pub(crate) struct Cur(*mut *mut c_char);

impl Cur {
    /// # Safety
    /// `argp` must point at a live `*mut c_char` walking a NUL-terminated
    /// expression, and both must stay valid for as long as the cursor is.
    pub(crate) const unsafe fn new(argp: *mut *mut c_char) -> Self {
        Self(argp)
    }

    /// Where the cursor stands.
    pub(crate) fn get(self) -> *mut c_char {
        // SAFETY: the constructor's promise.
        unsafe { *self.0 }
    }

    /// Move it to `p`.
    pub(crate) fn set(self, p: *mut c_char) {
        // SAFETY: the constructor's promise.
        unsafe { *self.0 = p };
    }

    /// The byte `i` past the cursor.
    ///
    /// Reading past the terminating NUL would be out of bounds, so a caller
    /// asking for `i > 0` has already seen a non-NUL at every offset below
    /// it — which is why the levels below read the second byte of an
    /// operator only inside the arm the first byte selected.
    pub(crate) fn at(self, i: usize) -> u8 {
        // SAFETY: the constructor's promise, plus the caller's: the walk has
        // not stepped past the NUL.
        unsafe { *self.get().add(i) }.cast_unsigned()
    }

    /// The byte under the cursor.
    pub(crate) fn byte(self) -> u8 {
        self.at(0)
    }

    /// Step it `n` bytes on.
    pub(crate) fn bump(self, n: usize) {
        self.set(self.get().wrapping_add(n));
    }

    /// Step it `n` bytes on and then past the white space, which is how a
    /// level consumes an operator it has recognised.
    pub(crate) fn skip(self, n: usize) {
        // SAFETY: the constructor's promise -- `skipwhite` stops at the NUL.
        self.set(unsafe { skipwhite(self.get().wrapping_add(n)) });
    }

    /// The pointer back, for the callees that still take one.
    pub(crate) fn raw(self) -> *mut *mut c_char {
        self.0
    }

    /// **The bridge into the evaluator**, for a pointer walker: run `f` over
    /// a [`Cursor`] on the text from here to the terminator, then move this
    /// cursor on by what `f` consumed. It goes when the last pointer walker
    /// takes a `Cursor` itself.
    ///
    /// # Safety
    /// The text must not be written or freed while `f` runs. The evaluator
    /// writes into nothing it reads; what the caller promises is that no
    /// code `f` can reach -- user functions, autocommands -- owns this text:
    /// a command line being executed, not an option value or a mapping.
    pub(crate) unsafe fn with_cursor<R>(self, f: impl FnOnce(&mut Cursor<'_>) -> R) -> R {
        // SAFETY: the constructor's promise -- the text is NUL-terminated
        // and live -- and the caller's: it is not written while borrowed.
        let text = unsafe { cstr::bytes_at(self.get()) };
        let mut cursor = Cursor::new(text);
        let answer = f(&mut cursor);
        self.bump(cursor.offset());
        answer
    }
}
