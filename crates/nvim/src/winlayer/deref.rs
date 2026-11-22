//! Field access through a graph handle, and handle equality — both checked
//! against the slot the handle names.
//!
//! A [`Win`], [`Buf`], [`TabPage`] or [`FrameRef`] holds its object's slot
//! address and the generation the slot had when the handle was made
//! ([`crate::slot`]). A slot's memory is never given back, so its generation
//! word is readable through any handle ever made, and every [`Deref`] and
//! [`DerefMut`] here reads it and compares: **one load, one compare**, and a
//! cold panic when the object the handle was made for has gone. That turns
//! the re-entry rule (`winlayer`'s docs) from a promise into a check: a
//! handle held across code that freed its object can no longer reach freed
//! memory or the slot's next tenant, it stops the editor instead, naming
//! the line that used it.
//!
//! Equality is the same question: two handles are equal when they name the
//! same slot **under the same generation**, so a handle that outlived its
//! window is not equal to the window that took its slot.

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
use core::ptr;

use crate::slot;
use crate::types::{Buffer, Frame, Handle, Tabpage, Window};
use crate::winlayer::{Buf, FrameRef, TabPage, Win};

/// The panic a dereference through a stale handle ends in: the invariant is
/// the re-entry rule, broken by the caller at the reported location.
#[cold]
#[inline(never)]
#[track_caller]
fn stale_handle(kind: &str, handle: Handle) -> ! {
    panic!(
        "a {kind} handle (id {}) was used after its {kind} was freed: it was held across code that \
         freed it, and must be re-derived from its id instead (the re-entry rule)",
        handle.get()
    )
}

macro_rules! checked_handles {
    ($($ty:ident => $target:ty, $kind:literal;)+) => { $(
        impl $ty {
            /// Whether the object this handle was made for is still in its
            /// slot: what every dereference asks first. Reads the slot's
            /// generation word, never the object. Not for a null handle.
            #[inline(always)]
            pub(crate) fn is_live(self) -> bool {
                // SAFETY: a handle's address is null or its slot's value
                // address (the constructors' promise), and a slot's memory is
                // never given back. A null handle's dereference was never
                // sound; it faults here as it did on the field access.
                let generation = unsafe { slot::generation_at(self.ptr) };
                generation == self.stamp.generation()
            }

            /// Panic unless the object this handle was made for is still in
            /// its slot.
            #[inline(always)]
            #[track_caller]
            fn check_live(&self) {
                if !self.is_live() {
                    stale_handle($kind, self.stamp.handle());
                }
            }
        }

        impl Deref for $ty {
            type Target = $target;

            #[inline(always)]
            #[track_caller]
            fn deref(&self) -> &$target {
                self.check_live();
                // SAFETY: the generation matched, and a slot's generation is
                // the one it was handed out under exactly while that value
                // is initialised in it. The borrow lasts as long as the field
                // access that asked for it.
                unsafe { &*self.ptr }
            }
        }

        /// The same slot under the same generation. `wp == curwin` stays an
        /// address test for every live handle; a stale one equals nothing
        /// but its own copies.
        impl PartialEq for $ty {
            #[inline(always)]
            fn eq(&self, other: &Self) -> bool {
                ptr::eq(self.ptr, other.ptr)
                    && self.stamp.generation() == other.stamp.generation()
            }
        }

        impl Eq for $ty {}
    )+ };
}

checked_handles! {
    Win => Window, "window";
    Buf => Buffer, "buffer";
    FrameRef => Frame, "frame";
    TabPage => Tabpage, "tab page";
}

// Spelled out rather than generated: `just ratchet` finds the types that
// write through a by-value accessor by their `impl DerefMut for` lines.
macro_rules! checked_deref_mut {
    ($target:ty) => {
        #[inline(always)]
        #[track_caller]
        fn deref_mut(&mut self) -> &mut $target {
            self.check_live();
            // SAFETY: as `deref`.
            unsafe { &mut *self.ptr }
        }
    };
}

impl DerefMut for Win {
    checked_deref_mut!(Window);
}

impl DerefMut for Buf {
    checked_deref_mut!(Buffer);
}

impl DerefMut for FrameRef {
    checked_deref_mut!(Frame);
}

impl DerefMut for TabPage {
    checked_deref_mut!(Tabpage);
}

#[cfg(test)]
mod tests {
    use std::panic::{AssertUnwindSafe, catch_unwind};

    use crate::buffer::{bare_buffer, free_bare_buffer};
    use crate::global_cell::editor_state_lock;
    use crate::window::{bare_window, free_bare_window};

    /// A live handle reads and writes its object; once the object is freed
    /// every copy reports it gone and a dereference panics -- even after the
    /// slot holds a new tenant, which the old copy does not equal.
    #[test]
    fn a_freed_window_stops_its_handles() {
        let _held = editor_state_lock();
        let mut win = bare_window();
        let copy = win;
        win.w_topline = crate::types::LineNr::new(3);
        assert_eq!(copy.w_topline, crate::types::LineNr::new(3));
        assert!(copy.is_live());
        free_bare_window(win);
        assert!(!copy.is_live());
        let caught = catch_unwind(AssertUnwindSafe(|| copy.w_topline));
        assert!(caught.is_err(), "a stale window handle dereferenced");
        let tenant = bare_window();
        if tenant.raw() == copy.raw() {
            assert_ne!(tenant, copy, "a new tenant is not the old window");
        }
        free_bare_window(tenant);
    }

    /// A buffer handle's mutable dereference checks the slot as well.
    #[test]
    fn a_freed_buffer_refuses_a_write() {
        let _held = editor_state_lock();
        let buffer = bare_buffer();
        let mut copy = buffer;
        copy.b_changed = 1;
        assert_eq!(buffer.b_changed, 1);
        free_bare_buffer(buffer);
        let caught = catch_unwind(AssertUnwindSafe(move || copy.b_changed = 0));
        assert!(caught.is_err(), "a stale buffer handle written through");
    }
}
