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
//! The handle lends only `&Terminal`, through a counted guard
//! ([`Term::read`]). Everything about a terminal that changes is a `Cell` or
//! a `RefCell` (see `Terminal`), so two copies of the handle in use at once
//! share it the way two `&RefCell`s do: a store is a `set`, and a borrow
//! held across a call that re-enters -- the emulator's `vt.borrow_mut()`
//! across `input_write`, whose callbacks reach the terminal through their
//! own copy -- makes a second `borrow_mut` of the same field panic instead
//! of aliasing.
//!
//! A terminal is allocated once ([`Term::alloc`]) into a slot of its own
//! pool and freed once ([`Term::free`], which `terminal_destroy` reaches
//! when nothing stands on it); in between, its buffer and its channel hold
//! the handle. The handle carries the slot's generation, as a [`Win`] does:
//! a copy used after the free -- a read, or a second free -- panics, and the
//! free itself panics while a read guard is out, so no borrow of a terminal
//! outlives it.
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

use crate::global_cell::GlobalCell;
use crate::slot::{self, SlotPool, SlotRead, SlotRef};
use crate::terminal::state::Terminal;

use super::Buf;

/// Every live terminal, each in a slot whose memory outlives every handle.
static TERMINAL_SLOTS: GlobalCell<SlotPool<Terminal>> = GlobalCell::new(SlotPool::new());

/// A live terminal: its slot and the generation it was handed, checked at
/// every access.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Term(SlotRef<Terminal>);

impl Term {
    /// Moves `terminal` into a slot, where it stays until [`Term::free`].
    pub(crate) fn alloc(terminal: Terminal) -> Self {
        Term(TERMINAL_SLOTS.with_mut(|pool| pool.insert(terminal)))
    }

    /// Frees the terminal and everything it owns. Every copy of the handle
    /// goes stale: a later access through one, a second free included,
    /// panics instead of reaching freed memory.
    ///
    /// # Panics
    /// When the terminal is gone already, or a [`Term::read`] of it is out
    /// (the free would pull it from under that borrow).
    #[track_caller]
    pub(crate) fn free(self) {
        let terminal = TERMINAL_SLOTS.with_mut(|pool| pool.remove(self.0));
        // Dropped outside the pool's borrow: what it owns is plain data, but
        // nothing here needs to promise so.
        drop(terminal);
    }

    /// Borrow the terminal until the guard is dropped. Counted: freeing it
    /// through any copy panics while the guard is held. Everything about a
    /// terminal that changes is a `Cell` or a `RefCell` (the module docs),
    /// so reads through two copies at once share it as two `&RefCell`s do.
    ///
    /// # Panics
    /// When the terminal is gone.
    #[inline]
    #[track_caller]
    pub(crate) fn read<'a>(self) -> SlotRead<'a, Terminal> {
        // SAFETY: a handle's address is its slot's value address in
        // `TERMINAL_SLOTS` (`alloc`), whose memory is never given back.
        unsafe { slot::read_at(self.0.raw(), self.0.generation()) }
    }
}

/// Forget every read guard out on a terminal, as if each had been dropped.
/// Only for the emergency exits ([`SlotPool::abandon_loans`]).
pub(crate) fn abandon_terminal_loans() {
    TERMINAL_SLOTS.with(SlotPool::abandon_loans);
}

impl Buf {
    /// The terminal this buffer draws, if it is a terminal buffer.
    pub(crate) fn terminal_handle(self) -> Option<Term> {
        self.terminal()
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

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
        buffer.set_terminal(Some(driver));
        let drawn = buffer.terminal_handle().expect("a terminal buffer");
        assert_eq!(drawn, driver);

        drawn.read().title.borrow_mut().extend_from_slice(b"shell");
        driver.read().title.borrow_mut().push(b'!');
        assert_eq!(*drawn.read().title.borrow(), b"shell!");
        driver.read().textbuf.borrow_mut().resize(64, b' ');
        drawn.read().closed.set(true);
        assert!(driver.read().closed.get());
        assert_eq!(drawn.read().textbuf.borrow().len(), 64);
        assert_eq!(drawn.read().opts.width, 4);
        {
            let terminal = driver.read();
            let _writing = terminal.vt.borrow_mut();
            assert!(
                drawn.read().vt.try_borrow().is_err(),
                "no read under a write"
            );
        }
        assert!(drawn.read().vt.try_borrow_mut().is_ok());

        // No free while a copy holds a borrow of it.
        let held = drawn.read();
        assert!(catch_unwind(AssertUnwindSafe(|| driver.free())).is_err());
        assert!(drawn.0.is_live());
        drop(held);

        buffer.set_terminal(None);
        driver.free();
        assert!(!drawn.0.is_live());
        // Every copy is stale now: a read and a second free both refuse.
        assert!(catch_unwind(AssertUnwindSafe(|| drop(drawn.read()))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| drawn.free())).is_err());
        free_bare_buffer(buffer);
    }
}
