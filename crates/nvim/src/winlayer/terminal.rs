//! The terminal handle: [`Term`], a `:terminal`'s state named by address.
//!
//! A terminal is reached from its buffer (`Buffer::terminal`), from the
//! channel that drives it, and from the callbacks its emulator reports
//! through -- and every one of those paths can come back into the others
//! while the first is still running: the emulator reports while the child's
//! output is being written into it, and an autocommand fired from any of them
//! can run Vimscript that reaches the terminal again. So the editor carries a
//! `Copy` handle, as it does a [`Win`] or a [`Buf`].
//!
//! The handle lends only `&Terminal`. Everything about a terminal that
//! changes is a `Cell` or a `RefCell` (see `Terminal`), so two copies of the
//! handle in use at once share it the way two `&RefCell`s do: a store is a
//! `set`, and a borrow held across a call that re-enters -- the emulator's
//! `vt.borrow_mut()` across `input_write`, whose callbacks reach the terminal
//! through their own copy -- makes a second `borrow_mut` of the same field
//! panic instead of aliasing.
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

use core::ops::Deref;
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
        NonNull::new(self.terminal()).map(Term)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{bare_buffer, free_bare_buffer};
    use crate::global_cell::editor_state_lock;
    use crate::terminal::state::TerminalOptions;
    use crate::types::Handle;

    fn options() -> TerminalOptions {
        TerminalOptions {
            owner: 1,
            width: 4,
            height: 2,
            read_pause_cb: None,
            write_cb: None,
            resize_cb: None,
            resume_cb: None,
            close_cb: None,
            force_crlf: false,
        }
    }

    /// A terminal's life: allocated once, reached through copies held by
    /// its buffer and its driver -- writes through either seen by the
    /// other, a field one copy holds borrowed refused to the other -- then
    /// freed once, with what it owns.
    #[test]
    fn copies_of_a_terminal_share_it_until_the_free() {
        let _held = editor_state_lock();
        let driver = Term::alloc(Terminal::new(options(), Handle::new(1)));
        let buffer = bare_buffer();
        assert_eq!(buffer.terminal_handle(), None);
        buffer.set_terminal(driver.raw());
        let drawn = buffer.terminal_handle().expect("a terminal buffer");
        assert_eq!(drawn, driver);

        drawn.title.borrow_mut().extend_from_slice(b"shell");
        driver.title.borrow_mut().push(b'!');
        assert_eq!(*drawn.title.borrow(), b"shell!");
        driver.textbuf.borrow_mut().resize(64, b' ');
        drawn.closed.set(true);
        assert!(driver.closed.get());
        assert_eq!(drawn.textbuf.borrow().len(), 64);
        assert_eq!(drawn.opts.width, 4);
        {
            let _writing = driver.vt.borrow_mut();
            assert!(drawn.vt.try_borrow().is_err(), "no read under a write");
        }
        assert!(drawn.vt.try_borrow_mut().is_ok());

        buffer.set_terminal(::core::ptr::null_mut());
        driver.free();
        free_bare_buffer(buffer);
    }
}
