//! Where the message area is, and what is on it.
//!
//! The cursor into the message area (`msg_col`, `msg_row`), how far it has
//! scrolled and whether the user still owes it a `<CR>` (`msg_scrolled`,
//! `need_wait_return`, `lines_left`, `quit_more`), the message that survives
//! the next redraw (`keep_msg`), and the flags every `msg_*` entry point
//! consults on the way in (`msg_silent`, `msg_scroll`, `msg_didout`,
//! `msg_hist_off`).
//!
//! The `emsg_*` half is the same thing for errors: whether one has been
//! given (`did_emsg`, `called_emsg`), whether the next is to be swallowed
//! (`emsg_off`, `emsg_silent`, `emsg_skip`), and the three cells
//! `assert_fails()` uses to capture one instead of showing it.
//!
//! `redir_*` and `capture_ga` are here for the same reason: they are read on
//! the way *out* of every message, by the tee in [`redir`] that `:redir` and
//! `'verbosefile'` turn on.
//!
//! [`redir`]: super::redir
//!
//! # One record
//!
//! Upstream declares each of these as its own `EXTERN` in `globals.h` and
//! `message.c`. They are one [`MsgState`] here, behind one cell, and every
//! name below is a `const` [`Field`] selector into it rather than a cell of
//! its own -- so a reader still writes `msg_col.get()` and `msg_scroll.set(1)`,
//! and what changed is that nothing can take the address of one.
//!
//! The record is never borrowed whole. A number or a flag is copied in and
//! out; a field that owns something is projected into a closure that may not
//! call back into the editor, or moved out and back ([`Field::update`]). So
//! no reference into the record is open across a call that can run user
//! code -- an autocommand, a `:redir => var` assignment, a UI event, an
//! `on_print` callback -- which is where every `msg_*` entry point can end
//! up.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The selectors here keep upstream's spelling, so the hundred-odd files that
// read them did not change; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::{Field, GlobalCell, field};
use crate::types::{Callback, FILE, GArray, ScreenGrid};
use core::ffi::{CStr, c_char, c_int, c_long};

pub(crate) static on_print: GlobalCell<Callback> = GlobalCell::new(Callback::None);
pub(crate) static top_bot_msg: &CStr = c"search hit TOP, continuing at BOTTOM";
pub(crate) static bot_top_msg: &CStr = c"search hit BOTTOM, continuing at TOP";
pub(crate) static line_msg: &CStr = c" line ";
pub(crate) static no_lines_msg: &CStr = c"--No lines in buffer--";

/// The message grid. Its own cell rather than a field of [`MsgState`]: the
/// grid layer holds its address (`GridRef::of_cell`) across calls.
pub(crate) static msg_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid::empty());

/// A field of [`MsgState`].
pub type MsgField<T> = Field<MsgState, T>;

/// Every field reads and writes through the one cell, a field at a time.
///
/// The rules are [`GlobalCell::get_at`]'s: untracked, and never a reference
/// that outlives the access. That is what lets a counter be raised in one
/// frame and read in a callback three frames down.
impl<T> Field<MsgState, T> {
    /// What the field holds.
    #[inline(always)]
    pub fn get(self) -> T
    where
        T: Copy,
    {
        MSG.get_at(self)
    }

    /// Overwrite the field.
    #[inline(always)]
    pub fn set(self, value: T) {
        MSG.set_at(self, value);
    }

    /// Overwrite the field, answering what it held.
    #[inline(always)]
    pub fn replace(self, value: T) -> T {
        MSG.replace_at(self, value)
    }

    /// Move the value out, leaving the type's empty value behind.
    #[inline(always)]
    pub fn take(self) -> T
    where
        T: Default,
    {
        MSG.replace_at(self, T::default())
    }

    /// Look at the field. `f` must not reach back into the editor: it is
    /// for a test or a clone, never for a call that could write the field.
    #[inline(always)]
    pub fn with<R>(self, f: impl FnOnce(&T) -> R) -> R {
        MSG.with_at(self, f)
    }

    /// Change the field in place.
    ///
    /// The value is moved out for the duration and put back after, so
    /// there is no borrow of the record for `f` to outlive: anything `f`
    /// reaches that reads the same field sees it empty, and anything that
    /// writes it is overwritten. Meant for a push or a clear, not for a
    /// call that can run user code.
    #[inline(always)]
    pub fn update<R>(self, f: impl FnOnce(&mut T) -> R) -> R
    where
        T: Default,
    {
        let mut value = self.take();
        let answer = f(&mut value);
        self.set(value);
        answer
    }
}

/// The record, the one cell, and a selector per field under the field's
/// own name.
macro_rules! msg_state {
    ($(
        $(#[$doc:meta])*
        $vis:vis $field:ident: $ty:ty = $init:expr;
    )*) => {
        /// What the message machinery knows about the message area and the
        /// one being built. See the [module docs](self).
        pub struct MsgState {
            $($field: $ty,)*
        }

        static MSG: GlobalCell<MsgState> = GlobalCell::new(MsgState {
            $($field: $init,)*
        });

        $(
            $(#[$doc])*
            $vis const $field: MsgField<$ty> = field!(MsgState, $field);
        )*
    };
}

msg_state! {
    // -- ext_messages --
    /// Hold back the `msg_show` flush until the whole message is written.
    pub(crate) msg_ext_skip_flush: bool = false;
    /// The next `msg_show` replaces the last one rather than adding to it.
    pub(crate) msg_ext_overwrite: bool = false;
    /// Keep the current kind across the next `verbose_enter`.
    pub(crate) msg_ext_skip_verbose: bool = false;

    // -- the message grid --
    /// The row of the default grid the message grid starts at.
    pub(crate) msg_grid_pos: c_int = 0;
    /// `msg_scrolled` when the grid was last flushed to the UI.
    pub(crate) msg_scrolled_at_flush: c_int = 0;
    /// Lines scrolled since the last flush that the UI has not been told of.
    pub(crate) msg_grid_scroll_discount: c_int = 0;
    /// Nonzero while `:argdo` and friends want each file message kept.
    pub(crate) msg_listdo_overwrite: c_int = 0;
    /// The command line is drawn right to left.
    pub(crate) cmdmsg_rl: bool = false;

    // -- the cursor into the message area --
    pub(crate) msg_col: c_int = 0;
    pub(crate) msg_row: c_int = 0;
    /// How many lines the message area has scrolled up the screen.
    pub(crate) msg_scrolled: c_int = 0;
    /// Leave `msg_scrolled` alone when this message scrolls.
    pub(crate) msg_scrolled_ign: bool = false;
    /// The message area scrolled since the last `msg_start`.
    pub(crate) msg_did_scroll: bool = false;

    // -- the message kept across a redraw --
    pub(crate) keep_msg: *mut c_char = core::ptr::null_mut();
    pub(crate) keep_msg_hl_id: c_int = 0;
    /// Show the file info when the redraw is done.
    pub(crate) need_fileinfo: bool = false;

    // -- what the next message does --
    /// Nonzero: the next message scrolls rather than overwrites.
    pub(crate) msg_scroll: c_int = 0;
    /// Something is on the current line.
    pub(crate) msg_didout: bool = false;
    /// Anything was shown since the last `msg_start`.
    pub(crate) msg_didany: bool = false;
    /// Don't wait for this message.
    pub(crate) msg_nowait: bool = false;
    /// Nonzero: errors are not shown at all.
    pub(crate) emsg_off: c_int = 0;
    /// Printing an informative message.
    pub(crate) info_message: bool = false;
    /// Don't add messages to the history.
    pub(crate) msg_hist_off: bool = false;
    /// Clear the rest of the line before the next message.
    pub(crate) need_clr_eos: bool = false;
    /// Nonzero while parsing an expression that is not executed.
    pub(crate) emsg_skip: c_int = 0;
    /// The next error is one `:try` must not hide.
    pub(crate) emsg_severe: bool = false;

    // -- assert_fails() --
    pub(crate) emsg_assert_fails_msg: *mut c_char = core::ptr::null_mut();
    pub(crate) emsg_assert_fails_lnum: c_long = 0;
    pub(crate) emsg_assert_fails_context: *mut c_char = core::ptr::null_mut();

    // -- errors --
    /// Errors given since this was last reset.
    pub(crate) did_emsg: c_int = 0;
    /// Errors given, ever: a caller compares it before and after.
    pub(crate) called_emsg: c_int = 0;
    /// An error is on the screen.
    pub(crate) emsg_on_display: bool = false;

    // -- the hit-enter prompt --
    /// Nonzero: don't wait for a return, the caller redraws.
    pub(crate) no_wait_return: c_int = 0;
    /// The screen needs a hit-enter before it can be redrawn.
    pub(crate) need_wait_return: bool = false;
    /// `wait_return` was just called.
    pub(crate) did_wait_return: bool = false;
    /// `q` was typed at the more prompt.
    pub(crate) quit_more: bool = false;
    /// Lines left before the more prompt; -1 resets it at `msg_start`.
    pub(crate) lines_left: c_int = -1;
    /// Don't give a more prompt.
    pub(crate) msg_no_more: bool = false;

    // -- silence --
    /// Nonzero under `:silent`: messages are computed, not shown.
    pub(crate) msg_silent: c_int = 0;
    /// Nonzero under `:silent!`: errors are raised, not shown.
    pub(crate) emsg_silent: c_int = 0;
    /// Don't redirect an error under `:silent!`.
    pub(crate) emsg_noredir: bool = false;
    /// Don't echo the command line.
    pub(crate) cmd_silent: bool = false;
    /// Inside `assert_fails()`.
    pub(crate) in_assert_fails: bool = false;

    // -- where messages go besides the screen --
    /// Don't redirect (prompts and the like).
    pub(crate) redir_off: bool = false;
    pub(crate) redir_fd: *mut FILE = core::ptr::null_mut();
    /// The register `:redir @x` writes to, or 0.
    pub(crate) redir_reg: c_int = 0;
    /// `:redir => var` is active.
    pub(crate) redir_vname: bool = false;
    pub(crate) capture_ga: *mut GArray = core::ptr::null_mut();
}
