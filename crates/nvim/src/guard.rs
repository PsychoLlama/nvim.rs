//! RAII guards over the editor's suppression and re-entrancy counters.
//!
//! `textlock`, `sandbox`, `msg_silent`, `emsg_off`, `emsg_skip`,
//! `no_wait_return`, `no_mapping`, `allow_keys` and `RedrawingDisabled` are
//! the globals that say "for the duration of this operation, don't show
//! errors / don't redraw / don't let anything touch the buffer". C bumps
//! them by hand and unbumps them at every exit — which is fine until an
//! exit is missed, and in Rust a `?` or a panic between the two halves
//! leaks the suppression for the rest of the session.
//!
//! Everything here is the same two moves the C makes, with the release
//! attached to a scope instead of to the programmer's memory:
//!
//! - [`Bump`] — the counter shape. `+= 1` on acquire, `-= 1` on drop, so
//!   nesting composes exactly as it does in C.
//! - [`Saved`] — the save/restore shape. Overwrite with a fixed value on
//!   acquire, put the old value back on drop. Not the same thing as a
//!   `Bump`: restoring a *saved* value also undoes whatever the scope's
//!   callees did to the counter, which is deliberate at the sites that use
//!   it (a `:silent` that ends at a prompt, an autocmd window that must
//!   redraw regardless of nesting).
//!
//! Neither constructor is public. The vocabulary is the named constructors
//! on [`Suppress`], [`Allow`], [`Lock`] and [`Keys`], which is what makes
//! the counter and its intended direction greppable.
//!
//! ```ignore
//! let _guard = Suppress::emsg();          // emsg_off += 1 for this scope
//! let _guard = cond.then(Suppress::emsg); // ... only when `cond`
//! let _guard = Allow::messages();         // msg_silent = 0, restored after
//! ```
//!
//! Bind to a named `_guard`, never to `let _ =`: `_` drops immediately and
//! the scope runs unguarded. Where the C released the counter in the
//! middle of a long body, bind without the underscore and `drop(guard)` at
//! that exact point rather than inventing a block — the release point is
//! load-bearing (an error raised after it is meant to be seen) and the
//! guard is still what runs it on an early exit.

#![forbid(unsafe_code)]

use crate::global_cell::GlobalCell;
use crate::main::{
    RedrawingDisabled, allow_keys, emsg_off, emsg_skip, expr_map_lock, msg_silent, no_mapping,
    no_wait_return, sandbox, textlock,
};
use core::ffi::c_int;

/// A counter held one higher for the lifetime of the guard.
///
/// Drop subtracts what the constructor added, so a `Bump` nests: the
/// counter is only back at rest once the outermost one is gone.
#[must_use = "the counter is released as soon as the guard is dropped"]
pub struct Bump {
    cell: &'static GlobalCell<c_int>,
    by: c_int,
}

impl Bump {
    fn new(cell: &'static GlobalCell<c_int>) -> Self {
        cell.set(cell.get() + 1);
        Bump { cell, by: 1 }
    }

    /// A bump of `by` rather than 1, for the two sites that add a boolean.
    fn by(cell: &'static GlobalCell<c_int>, by: c_int) -> Self {
        cell.set(cell.get() + by);
        Bump { cell, by }
    }
}

impl Drop for Bump {
    fn drop(&mut self) {
        self.cell.set(self.cell.get() - self.by);
    }
}

/// A counter overwritten for the lifetime of the guard, then put back.
///
/// Unlike [`Bump`] this does not nest — dropping restores the value the
/// constructor saw, whatever happened in between. That is what the C does
/// at these sites and why they are spelled differently.
#[must_use = "the old value is restored as soon as the guard is dropped"]
pub struct Saved {
    cell: &'static GlobalCell<c_int>,
    saved: c_int,
}

impl Saved {
    fn new(cell: &'static GlobalCell<c_int>, value: c_int) -> Self {
        Saved {
            cell,
            saved: cell.replace(value),
        }
    }

    /// Save unconditionally, overwrite only when `cond`.
    ///
    /// Not the same as `cond.then(…)`: the restore happens either way,
    /// which is what a scope that hands control to arbitrary Lua wants.
    fn when(cond: bool, cell: &'static GlobalCell<c_int>, value: c_int) -> Self {
        let saved = cell.get();
        if cond {
            cell.set(value);
        }
        Saved { cell, saved }
    }
}

impl Drop for Saved {
    fn drop(&mut self) {
        self.cell.set(self.saved);
    }
}

/// Both halves of the "read a key literally" pair, released together.
#[must_use = "the counters are released as soon as the guard is dropped"]
pub struct RawKeys {
    _no_mapping: Bump,
    _allow_keys: Bump,
}

/// Guards that turn some part of the editor's output off for a scope.
pub struct Suppress;

impl Suppress {
    /// `emsg_off` — error messages are not displayed at all.
    ///
    /// For the caller who evaluates something it expects to fail and does
    /// not want the user to hear about it.
    pub fn emsg() -> Bump {
        Bump::new(&emsg_off)
    }

    /// `emsg_skip` — errors from an expression that is only being parsed,
    /// not executed (a skipped `:if` branch, a `:for` over a bad list).
    pub fn emsg_skip() -> Bump {
        Bump::new(&emsg_skip)
    }

    /// `msg_silent` — messages are computed but not shown, as under
    /// `:silent`.
    pub fn messages() -> Bump {
        Bump::new(&msg_silent)
    }

    /// [`Suppress::messages`], but by `by` rather than by one — the two
    /// sites that add `ui_has(kUIMessages)`.
    pub fn messages_by(by: c_int) -> Bump {
        Bump::by(&msg_silent, by)
    }

    /// `no_wait_return` — a message shown in this scope does not stop for
    /// the hit-enter prompt.
    pub fn wait_return() -> Bump {
        Bump::new(&no_wait_return)
    }

    /// `RedrawingDisabled` — the screen is not updated while this is held.
    pub fn redraw() -> Bump {
        Bump::new(&RedrawingDisabled)
    }
}

/// Guards that lift a suppression for a scope and put it back afterwards.
///
/// The mirror of [`Suppress`]: these are the sites where the editor has to
/// reach the user *despite* an enclosing `:silent` or a redraw-disabled
/// operation — a prompt, a dialog, a swap-file question.
pub struct Allow;

impl Allow {
    /// `msg_silent = 0` — this scope's messages reach the user even inside
    /// `:silent`.
    pub fn messages() -> Saved {
        Saved::new(&msg_silent, 0)
    }

    /// `RedrawingDisabled = 0` — this scope redraws even inside an
    /// operation that had disabled it.
    pub fn redraw() -> Saved {
        Saved::new(&RedrawingDisabled, 0)
    }

    /// `RedrawingDisabled = value` — the handful of sites that restore a
    /// value they carried in from elsewhere rather than the one they saw.
    pub fn redraw_at(value: c_int) -> Saved {
        Saved::new(&RedrawingDisabled, value)
    }

    /// `no_wait_return = 0` — the hit-enter prompt is armed again for this
    /// scope, whatever the caller had asked for.
    pub fn wait_return() -> Saved {
        Saved::new(&no_wait_return, 0)
    }

    /// `no_wait_return = 1` — startup's inverse: no prompt until the
    /// guard's scope ends. Spelled as a [`Saved`] because startup sets it
    /// unconditionally rather than nesting.
    pub fn no_wait_return() -> Saved {
        Saved::new(&no_wait_return, 1)
    }

    /// `textlock = 0` — the callback about to run is allowed to change
    /// text even though the caller was inside a text-locked operation.
    pub fn text_changes() -> Saved {
        Saved::new(&textlock, 0)
    }

    /// [`Allow::text_changes`], lifting the lock only when `cond`, but
    /// restoring it either way.
    pub fn text_changes_when(cond: bool) -> Saved {
        Saved::when(cond, &textlock, 0)
    }

    /// `expr_map_lock = 0` — [`Allow::text_changes`]'s companion; the two
    /// sites that invite arbitrary Lua in lift both locks together.
    pub fn expr_map() -> Saved {
        Saved::new(&expr_map_lock, 0)
    }

    /// [`Allow::expr_map`], lifting the lock only when `cond`, but
    /// restoring it either way.
    pub fn expr_map_when(cond: bool) -> Saved {
        Saved::when(cond, &expr_map_lock, 0)
    }

    /// `allow_keys = 0` — key codes are *not* recognised in this scope, so
    /// a raw `<BS>` byte stays a byte.
    pub fn no_key_codes() -> Saved {
        Saved::new(&allow_keys, 0)
    }

    /// `no_mapping -= 1` — the inverse of [`Keys::unmapped`], for the
    /// callee that has to read a *mapped* key back out of a caller that
    /// had suppressed mapping (`'langmap'`, composing characters).
    pub fn mapping() -> Bump {
        Bump::by(&no_mapping, -1)
    }

    /// [`Allow::mapping`] for both halves of the pair — the inverse of
    /// [`Keys::unmapped_with_codes`].
    pub fn mapping_with_codes() -> RawKeys {
        RawKeys {
            _no_mapping: Bump::by(&no_mapping, -1),
            _allow_keys: Bump::by(&allow_keys, -1),
        }
    }

    /// `emsg_off = 1` — error display off for the scope, restoring
    /// whatever nesting level was in effect rather than decrementing.
    pub fn no_emsg() -> Saved {
        Saved::new(&emsg_off, 1)
    }
}

/// Guards over the two locks that say what the code running inside them is
/// allowed to do.
pub struct Lock;

impl Lock {
    /// `textlock` — buffer text, window layout and the current
    /// buffer/window must not change while this is held.
    pub fn text() -> Bump {
        Bump::new(&textlock)
    }

    /// `sandbox` — the code about to run came from somewhere untrusted
    /// (a modeline, a `'foldexpr'`, a tag command) and the operations
    /// marked unsafe-in-sandbox are refused.
    pub fn sandbox() -> Bump {
        Bump::new(&sandbox)
    }

    /// `expr_map_lock` — [`Lock::text`]'s companion for the `<expr>`
    /// mapping and abbreviation expansions, which additionally must not
    /// change the mapping tables they are being read from.
    pub fn expr_map() -> Bump {
        Bump::new(&expr_map_lock)
    }
}

/// Guards over how the next key read is decoded.
pub struct Keys;

impl Keys {
    /// `no_mapping` — the keys read in this scope do not go through
    /// mappings or abbreviations.
    pub fn unmapped() -> Bump {
        Bump::new(&no_mapping)
    }

    /// `allow_keys` — special key codes (`<Up>`, `<F1>`, …) are recognised
    /// even while mapping is off.
    pub fn codes() -> Bump {
        Bump::new(&allow_keys)
    }

    /// `no_mapping` + `allow_keys`: the recurring pair. Read the next key
    /// literally, but still decode the multi-byte key codes.
    pub fn unmapped_with_codes() -> RawKeys {
        RawKeys {
            _no_mapping: Self::unmapped(),
            _allow_keys: Self::codes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_nests_and_unwinds() {
        assert_eq!(emsg_off.get(), 0);
        {
            let _outer = Suppress::emsg();
            assert_eq!(emsg_off.get(), 1);
            {
                let _inner = Suppress::emsg();
                assert_eq!(emsg_off.get(), 2);
            }
            assert_eq!(emsg_off.get(), 1);
        }
        assert_eq!(emsg_off.get(), 0);
    }

    #[test]
    fn saved_restores_what_it_saw() {
        msg_silent.set(3);
        {
            let _guard = Allow::messages();
            assert_eq!(msg_silent.get(), 0);
            // A callee bumping it does not survive the restore.
            let inner = Suppress::messages();
            assert_eq!(msg_silent.get(), 1);
            core::mem::forget(inner);
        }
        assert_eq!(msg_silent.get(), 3);
        msg_silent.set(0);
    }

    #[test]
    fn raw_keys_releases_both() {
        {
            let _guard = Keys::unmapped_with_codes();
            assert_eq!((no_mapping.get(), allow_keys.get()), (1, 1));
        }
        assert_eq!((no_mapping.get(), allow_keys.get()), (0, 0));
    }
}
