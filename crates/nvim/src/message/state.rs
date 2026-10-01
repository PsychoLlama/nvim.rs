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
//! `redir_*` and `msg_capture` are here for the same reason: they are read on
//! the way *out* of every message, by the tee in [`redir`] that `:redir` and
//! `'verbosefile'` turn on.
//!
//! [`redir`]: super::redir
//!
//! # One record
//!
//! Upstream declares each of these as its own `EXTERN` in `globals.h` and
//! `message.c`. They are one [`MsgState`] here, behind one cell, and every
//! name below is a `const` selector into it rather than a cell of its own --
//! so a reader still writes `msg_col.get()` and `msg_scroll.set(1)`, and what
//! changed is that nothing can take the address of one.
//!
//! The record is never borrowed whole. A number or a flag is copied in and
//! out; a field that owns something is projected into a closure that may not
//! call back into the editor, or moved out and back (`update`; see
//! [`state_record`](crate::global_cell::state_record)). So no reference into
//! the record is open across a call that can run user code -- an autocommand, a `:redir => var` assignment, a UI event, an
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
// The selectors keep upstream's spelling, so the hundred-odd files that read
// them did not change; `message/mod.rs`'s `non_upper_case_globals` allow
// covers them, and upper-casing them is a per-module rewrite.

use super::{PROGRESS_TARGET_CMD, SB_CLEAR_NONE, ScrollbackClear};
use crate::global_cell::{GlobalCell, state_record};
use crate::memory::XString;
use crate::options::{
    OptMoptFlags, kOptMoptFlagHistory, kOptMoptFlagHitEnter, kOptMoptFlagProgress,
};
use crate::os::fs::CFile;
use crate::types::{Array, Callback, Object, ScreenAttr, ScreenGrid, String_0, int64_t};
use core::ffi::{CStr, c_int, c_long};

pub(crate) static on_print: GlobalCell<Callback> = GlobalCell::new(Callback::None);
pub(crate) static top_bot_msg: &CStr = c"search hit TOP, continuing at BOTTOM";
pub(crate) static bot_top_msg: &CStr = c"search hit BOTTOM, continuing at TOP";
pub(crate) static line_msg: &CStr = c" line ";
pub(crate) static no_lines_msg: &CStr = c"--No lines in buffer--";

/// The message grid. Its own cell rather than a field of [`MsgState`]: the
/// grid layer holds its address (`GridRef::of_cell`) across calls.
pub(crate) static msg_grid: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid::empty());

state_record! {
    /// What the message machinery knows about the message area and the one
    /// being built. See the [module docs](self).
    pub(crate) struct MsgState in MSG as MsgField;

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
    /// The message to show again after the next redraw.
    pub(crate) keep_msg: Option<XString> = None;
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
    /// The first error the command under `assert_fails()` gave.
    pub(crate) emsg_assert_fails_msg: Option<XString> = None;
    /// The line it was given from.
    pub(crate) emsg_assert_fails_lnum: c_long = 0;
    /// The script or function it was given from; empty for none.
    pub(crate) emsg_assert_fails_context: Option<XString> = None;

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
    /// The file `:redir > file` writes to.
    pub(crate) redir_fd: Option<CFile> = None;
    /// `'verbosefile'`, opened on the first message that goes to it.
    pub(super) verbose_fd: Option<CFile> = None;
    /// The register `:redir @x` writes to, or 0.
    pub(crate) redir_reg: c_int = 0;
    /// `:redir => var` is active.
    pub(crate) redir_vname: bool = false;
    /// What `execute()`, `nvim_exec2()` and `nvim_cmd()` are collecting,
    /// while one of them is (upstream's `capture_ga`). Each saves the outer
    /// capture and puts it back; see `capture_start`.
    pub(crate) msg_capture: Option<Vec<u8>> = None;

    // -- message/ internals: what the module's own files share --
    /// Nonzero while the confirm message is being written, so `q` at the more
    /// prompt cannot truncate it away.
    pub(super) confirm_msg_used: c_int = 0;
    /// The dialog's message text, as `display_confirm_msg` prints it.
    pub(super) confirm_msg: Option<XString> = None;
    /// The rendered button list, used as the command-line prompt.
    pub(super) confirm_buttons: Option<XString> = None;
    /// Drop the temporary history entries before adding the next message.
    pub(super) do_clear_hist_temp: bool = true;
    /// `'messagesopt'`'s flag set.
    pub(super) msg_flags: OptMoptFlags =
        kOptMoptFlagHitEnter | kOptMoptFlagHistory | kOptMoptFlagProgress;
    /// `'messagesopt'`'s `wait:` delay, in milliseconds.
    pub(super) msg_wait: c_int = 0;
    /// Where `'messagesopt'`'s `progress:` sends progress messages.
    pub(super) progress_msg_target: c_int = PROGRESS_TARGET_CMD;
    /// Whether, and how much of, the scrollback to drop before the next
    /// message.
    pub(super) do_clear_sb_text: ScrollbackClear = SB_CLEAR_NONE;
    /// The kept message is a `msgmore` report, which the next one may
    /// replace.
    pub(super) keep_msg_more: bool = false;
    /// Nonzero while `msg_multihl` is emitting a chunk, so `msg_keep` knows
    /// not to start or end a message of its own.
    pub(super) is_multihl: c_int = 0;
    /// How deep `msg_keep` is in itself: a message whose display raises
    /// another stops at three.
    pub(super) msg_keep_depth: c_int = 0;
    /// `msg_source` is running, and must not report its own source.
    pub(super) msg_source_busy: bool = false;
    /// The more prompt is up: a timer's message must not raise another.
    pub(super) more_prompt_busy: bool = false;
    /// The showmode text went away, and the UI is owed one empty event.
    pub(super) showmode_clear_pending: bool = false;
    /// The script/function the last error was reported from.
    pub(super) last_sourcing_name: Option<XString> = None;
    /// The line the last error was reported from.
    pub(super) last_sourcing_lnum: c_int = 0;
    /// The message kind in force when the current verbose section started.
    pub(super) pre_verbose_kind: String_0 = String_0::NULL;
    /// Whether opening `'verbosefile'` has been attempted, so the failure
    /// is reported once rather than on every message.
    pub(super) verbose_did_open: bool = false;
    /// The column `redir_write` has written up to, tracked separately from
    /// `msg_col` because the redirection sees no screen.
    pub(super) redir_col: c_int = 0;

    // -- the message being built for an ext_messages UI --
    /// The kind the message being composed carries, **owned**.
    ///
    /// The kind outlives the call that set it -- it is read again when the
    /// message is flushed to the UI or copied into the history -- so this
    /// keeps its own copy rather than the caller's pointer. `nvim_echo`'s
    /// `kind` lives in a keyset that dies with the call, which is how a
    /// borrowed kind became a use-after-free.
    pub(super) msg_ext_kind: String_0 = String_0::NULL;
    /// What caused the message being built, for a UI that groups by it.
    /// Both callers name a compiled-in kind, so the field holds the literal
    /// rather than a pointer whose lifetime it would have to answer for.
    pub(super) msg_ext_trigger: Option<&'static CStr> = None;
    pub(super) msg_ext_id: Object = Object::Integer(1);
    pub(super) msg_ext_chunks: Option<Array> = None;
    /// The text written under the current highlight, waiting to be closed
    /// off into a `msg_show` chunk by `msg_ext_emit_chunk`.
    pub(super) msg_ext_last_chunk: Vec<u8> = Vec::new();
    pub(super) msg_ext_last_attr: ScreenAttr = -1;
    pub(super) msg_ext_last_hl_id: c_int = 0;
    pub(super) msg_ext_history: bool = false;
    pub(super) msg_ext_append: bool = false;
    pub(super) msg_grid_pos_at_flush: c_int = 0;
    pub(super) msg_id_next: int64_t = 1;
}
