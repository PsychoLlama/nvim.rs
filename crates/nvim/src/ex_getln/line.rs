//! [`Cc`]: which command line, and every way into it.
//!
//! The command line being edited is the [`CCLINE`] cell; the ones
//! [`save_cmdline`] suspended under it are entries of [`SAVED_CMDLINES`]. A
//! `Cc` names one of those *places* -- the cell, or a stack entry by its
//! index from the bottom -- and holds no address. Every read and write goes
//! through the place's tracked cell, one leaf borrow at a time:
//!
//! - a scalar field has a getter and a setter of its own name
//!   (`cc.cmdpos()`, `cc.set_cmdpos(n)`), each one borrow that ends before it
//!   returns;
//! - anything else is [`Cc::with`]/[`Cc::with_mut`], whose closure must be a
//!   leaf: it runs with the cell borrowed, so reaching the command line again
//!   from inside it (through any `Cc`, or user code that does) panics, which
//!   is the cell's tracker doing its job.
//!
//! So two copies of a `Cc` can no longer lend overlapping `&mut`s, and a
//! `Cc` kept past the stack entry it named (popped by [`restore_cmdline`])
//! panics at its next use instead of reading a freed `Box`.
//!
//! What a `Cc` means does not change under it: [`Cc::current`] is whatever
//! line the cell holds *at each use*, so one taken before a `save_cmdline`
//! reads the fresh line after it, as the C's `&ccline` did.

#![forbid(unsafe_code)]

use core::ffi::{c_char, c_int, c_uint};

use super::state::{CmdRedraw, CmdlineInfo};
use super::{CCLINE, SAVED_CMDLINES};
use crate::cmdexpand::defs::ExpandContext;

/// A command line: the one being edited, or one suspended under it.
///
/// `Copy`, and holds no address (module docs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Cc {
    /// `None` for the cell, else the saved stack's entry at this index from
    /// the bottom.
    saved: Option<usize>,
}

impl Cc {
    /// The command line the editor is on: whatever the cell holds at each
    /// use.
    #[inline]
    pub(crate) const fn current() -> Self {
        Cc { saved: None }
    }

    /// The command line suspended `depth` levels under the current one
    /// (`depth` >= 1), or `None` past the bottom of the saved stack.
    pub(crate) fn saved(depth: usize) -> Option<Self> {
        let len = SAVED_CMDLINES.with(Vec::len);
        Some(Cc {
            saved: Some(len.checked_sub(depth)?),
        })
    }

    /// Run `f` on the command line, borrowed for the call: a leaf (module
    /// docs).
    ///
    /// # Panics
    /// When the line is a saved one since restored, or when `f` reaches the
    /// command line again.
    #[inline]
    #[track_caller]
    pub(crate) fn with<R>(self, f: impl FnOnce(&CmdlineInfo) -> R) -> R {
        match self.saved {
            None => CCLINE.with(f),
            Some(index) => SAVED_CMDLINES.with(|stack| {
                f(stack
                    .get(index)
                    .expect("a saved command line is used only while it is saved"))
            }),
        }
    }

    /// [`Cc::with`] for a write.
    #[inline]
    #[track_caller]
    pub(crate) fn with_mut<R>(self, f: impl FnOnce(&mut CmdlineInfo) -> R) -> R {
        match self.saved {
            None => CCLINE.with_mut(f),
            Some(index) => SAVED_CMDLINES.with_mut(|stack| {
                f(stack
                    .get_mut(index)
                    .expect("a saved command line is used only while it is saved"))
            }),
        }
    }
}

/// A getter and a setter per scalar field, each one leaf borrow.
macro_rules! scalar_fields {
    ($($(#[$doc:meta])* $field:ident, $set:ident: $ty:ty;)*) => {
        impl Cc {
            $(
                $(#[$doc])*
                #[inline]
                pub(crate) fn $field(self) -> $ty {
                    self.with(|line| line.$field)
                }

                #[doc = concat!("Set [`Cc::", stringify!($field), "`].")]
                #[inline]
                pub(crate) fn $set(self, value: $ty) {
                    self.with_mut(|line| line.$field = value);
                }
            )*
        }
    };
}

scalar_fields! {
    /// The cursor's byte offset in the text.
    cmdpos, set_cmdpos: c_int;
    /// The cursor's screen column.
    cmdspos, set_cmdspos: c_int;
    /// The character shown before the line, 0 for none.
    cmdfirstc, set_cmdfirstc: u8;
    /// The indent shown before the text.
    cmdindent, set_cmdindent: c_int;
    /// The highlight of the prompt.
    hl_id, set_hl_id: c_int;
    /// Whether typing replaces rather than inserts.
    overstrike, set_overstrike: bool;
    /// What `input()`'s completion argument asked for.
    xp_context, set_xp_context: ExpandContext;
    /// Whether the line is `input()`'s.
    input_fn, set_input_fn: bool;
    /// Whether the text was replaced by a callback.
    cmdbuff_replaced, set_cmdbuff_replaced: bool;
    /// The number of the prompt the line belongs to.
    prompt_id, set_prompt_id: c_uint;
    /// How deep the line is nested.
    level, set_level: c_int;
    /// The character shown at the cursor after CTRL-V or CTRL-K.
    special_char, set_special_char: c_char;
    /// Whether that character is shifted.
    special_shift, set_special_shift: bool;
    /// What the next redraw of the line has to do.
    redraw_state, set_redraw_state: CmdRedraw;
    /// Whether the line is waiting for a single key.
    one_key, set_one_key: bool;
    /// Whether a click may answer the prompt (a number prompt).
    mouse_prompt, set_mouse_prompt: bool;
    /// Whether a click did.
    mouse_used, set_mouse_used: bool;
}
