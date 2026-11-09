//! The terminal handle: [`Term`], a `:terminal`'s state named by address.
//!
//! A terminal is reached from its buffer (`Buffer::terminal`), from the
//! channel that drives it, and from the callbacks its emulator reports
//! through -- and every one of those paths can come back into the others
//! while the first is still running: the emulator reports while the child's
//! output is being written into it, and an autocommand fired from any of them
//! can run Vimscript that reaches the terminal again. So the editor carries a
//! `Copy` handle, as it does a [`Win`] or a [`Buf`], and every borrow of the
//! terminal through it lasts one expression: **no reference into a terminal
//! is held across a callback, an autocommand, an emulator write or a
//! refresh**. Read the field out first and call afterwards.
//!
//! A terminal is allocated once ([`Term::alloc`]) and freed once
//! ([`Term::free`], which `terminal_destroy` reaches when nothing stands on
//! it); in between, its buffer and its channel hold the handle.
//!
//! [`Win`]: super::Win
//! [`Buf`]: super::Buf

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `winlayer/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use crate::terminal::state::Terminal;

use super::Buf;

/// A live terminal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Term(NonNull<Terminal>);

impl Deref for Term {
    type Target = Terminal;

    #[inline(always)]
    fn deref(&self) -> &Terminal {
        // SAFETY: a handle is made by `alloc` or read from the buffer that
        // owns the terminal, and the terminal lives until `free`, after which
        // its buffer and channel have dropped every copy (the module docs).
        unsafe { self.0.as_ref() }
    }
}

impl DerefMut for Term {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Terminal {
        // SAFETY: as above; each borrow lasts one expression.
        unsafe { self.0.as_mut() }
    }
}

impl Term {
    /// Moves `terminal` to the heap, where it stays until [`Term::free`].
    pub(crate) fn alloc(terminal: Terminal) -> Self {
        Term(NonNull::from(Box::leak(Box::new(terminal))))
    }

    /// Frees the terminal and everything it owns. The caller drops every copy
    /// of the handle it can reach; the buffer and channel have already let
    /// go of theirs.
    pub(crate) fn free(self) {
        // SAFETY: the other half of `alloc`'s leak, reached once.
        drop(unsafe { Box::from_raw(self.0.as_ptr()) });
    }

    /// The address, as `Buffer::terminal` stores it.
    pub(crate) fn raw(self) -> *mut Terminal {
        self.0.as_ptr()
    }
}

impl Buf {
    /// The terminal this buffer draws, if it is a terminal buffer.
    pub(crate) fn terminal_handle(self) -> Option<Term> {
        NonNull::new(self.terminal).map(Term)
    }
}
