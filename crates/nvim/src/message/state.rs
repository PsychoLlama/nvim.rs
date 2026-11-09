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
//! `ON_PRINT` callback -- which is where every `msg_*` entry point can end
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
use crate::grid::defs::ScreenGrid;
use crate::memory::XString;
use crate::options::{
    OptMoptFlags, kOptMoptFlagHistory, kOptMoptFlagHitEnter, kOptMoptFlagProgress,
};
use crate::os::fs::CFile;
use crate::os::libc::int64_t;
use crate::types::{ApiString, Callback, Object, ScreenAttr};
use core::ffi::{CStr, c_int, c_long};

pub(crate) static ON_PRINT: GlobalCell<Callback> = GlobalCell::new(Callback::None);
pub(crate) static TOP_BOT_MSG: &CStr = c"search hit TOP, continuing at BOTTOM";
pub(crate) static BOT_TOP_MSG: &CStr = c"search hit BOTTOM, continuing at TOP";
pub(crate) static LINE_MSG: &CStr = c" line ";
pub(crate) static NO_LINES_MSG: &CStr = c"--No lines in buffer--";

/// The message grid. Its own cell rather than a field of [`MsgState`]: the
/// grid layer holds its address (`GridRef::of_cell`) across calls.
pub(crate) static MSG_GRID: GlobalCell<ScreenGrid> = GlobalCell::new(ScreenGrid::empty());

state_record! {
    /// What the message machinery knows about the message area and the one
    /// being built. See the [module docs](self).
    pub(crate) struct MsgState in MSG as MsgField;

    // -- ext_messages --
    /// Hold back the `msg_show` flush until the whole message is written.
    pub(crate) MSG_EXT_SKIP_FLUSH: bool = false;
    /// The next `msg_show` replaces the last one rather than adding to it.
    pub(crate) MSG_EXT_OVERWRITE: bool = false;
    /// Keep the current kind across the next `verbose_enter`.
    pub(crate) MSG_EXT_SKIP_VERBOSE: bool = false;

    // -- the message grid --
    /// The row of the default grid the message grid starts at.
    pub(crate) MSG_GRID_POS: c_int = 0;
    /// `msg_scrolled` when the grid was last flushed to the UI.
    pub(crate) MSG_SCROLLED_AT_FLUSH: c_int = 0;
    /// Lines scrolled since the last flush that the UI has not been told of.
    pub(crate) MSG_GRID_SCROLL_DISCOUNT: c_int = 0;
    /// Nonzero while `:argdo` and friends want each file message kept.
    pub(crate) MSG_LISTDO_OVERWRITE: c_int = 0;
    /// The command line is drawn right to left.
    pub(crate) CMDMSG_RL: bool = false;

    // -- the cursor into the message area --
    pub(crate) MSG_COL: c_int = 0;
    pub(crate) MSG_ROW: c_int = 0;
    /// How many lines the message area has scrolled up the screen.
    pub(crate) MSG_SCROLLED: c_int = 0;
    /// Leave `msg_scrolled` alone when this message scrolls.
    pub(crate) MSG_SCROLLED_IGN: bool = false;
    /// The message area scrolled since the last `msg_start`.
    pub(crate) MSG_DID_SCROLL: bool = false;

    // -- the message kept across a redraw --
    /// The message to show again after the next redraw.
    pub(crate) KEEP_MSG: Option<XString> = None;
    pub(crate) KEEP_MSG_HL_ID: c_int = 0;
    /// Show the file info when the redraw is done.
    pub(crate) NEED_FILEINFO: bool = false;

    // -- what the next message does --
    /// Nonzero: the next message scrolls rather than overwrites.
    pub(crate) MSG_SCROLL: c_int = 0;
    /// Something is on the current line.
    pub(crate) MSG_DIDOUT: bool = false;
    /// Anything was shown since the last `msg_start`.
    pub(crate) MSG_DIDANY: bool = false;
    /// Don't wait for this message.
    pub(crate) MSG_NOWAIT: bool = false;
    /// Nonzero: errors are not shown at all.
    pub(crate) EMSG_OFF: c_int = 0;
    /// Printing an informative message.
    pub(crate) INFO_MESSAGE: bool = false;
    /// Don't add messages to the history.
    pub(crate) MSG_HIST_OFF: bool = false;
    /// Clear the rest of the line before the next message.
    pub(crate) NEED_CLR_EOS: bool = false;
    /// Nonzero while parsing an expression that is not executed.
    pub(crate) EMSG_SKIP: c_int = 0;
    /// The next error is one `:try` must not hide.
    pub(crate) EMSG_SEVERE: bool = false;

    // -- assert_fails() --
    /// The first error the command under `assert_fails()` gave.
    pub(crate) EMSG_ASSERT_FAILS_MSG: Option<XString> = None;
    /// The line it was given from.
    pub(crate) EMSG_ASSERT_FAILS_LNUM: c_long = 0;
    /// The script or function it was given from; empty for none.
    pub(crate) EMSG_ASSERT_FAILS_CONTEXT: Option<XString> = None;

    // -- errors --
    /// Errors given since this was last reset.
    pub(crate) DID_EMSG: c_int = 0;
    /// Errors given, ever: a caller compares it before and after.
    pub(crate) CALLED_EMSG: c_int = 0;
    /// An error is on the screen.
    pub(crate) EMSG_ON_DISPLAY: bool = false;

    // -- the hit-enter prompt --
    /// Nonzero: don't wait for a return, the caller redraws.
    pub(crate) NO_WAIT_RETURN: c_int = 0;
    /// The screen needs a hit-enter before it can be redrawn.
    pub(crate) NEED_WAIT_RETURN: bool = false;
    /// `wait_return` was just called.
    pub(crate) DID_WAIT_RETURN: bool = false;
    /// `q` was typed at the more prompt.
    pub(crate) QUIT_MORE: bool = false;
    /// Lines left before the more prompt; -1 resets it at `msg_start`.
    pub(crate) LINES_LEFT: c_int = -1;
    /// Don't give a more prompt.
    pub(crate) MSG_NO_MORE: bool = false;

    // -- silence --
    /// Nonzero under `:silent`: messages are computed, not shown.
    pub(crate) MSG_SILENT: c_int = 0;
    /// Nonzero under `:silent!`: errors are raised, not shown.
    pub(crate) EMSG_SILENT: c_int = 0;
    /// Don't redirect an error under `:silent!`.
    pub(crate) EMSG_NOREDIR: bool = false;
    /// Don't echo the command line.
    pub(crate) CMD_SILENT: bool = false;
    /// Inside `assert_fails()`.
    pub(crate) IN_ASSERT_FAILS: bool = false;

    // -- where messages go besides the screen --
    /// Don't redirect (prompts and the like).
    pub(crate) REDIR_OFF: bool = false;
    /// The file `:redir > file` writes to.
    pub(crate) REDIR_FD: Option<CFile> = None;
    /// `'verbosefile'`, opened on the first message that goes to it.
    pub(super) VERBOSE_FD: Option<CFile> = None;
    /// The register `:redir @x` writes to, or 0.
    pub(crate) REDIR_REG: u8 = 0;
    /// `:redir => var` is active.
    pub(crate) REDIR_VNAME: bool = false;
    /// What `execute()`, `nvim_exec2()` and `nvim_cmd()` are collecting,
    /// while one of them is (upstream's `capture_ga`). Each saves the outer
    /// capture and puts it back; see `capture_start`.
    pub(crate) MSG_CAPTURE: Option<Vec<u8>> = None;

    // -- message/ internals: what the module's own files share --
    /// Nonzero while the confirm message is being written, so `q` at the more
    /// prompt cannot truncate it away.
    pub(super) CONFIRM_MSG_USED: c_int = 0;
    /// The dialog's message text, as `display_confirm_msg` prints it.
    pub(super) CONFIRM_MSG: Option<XString> = None;
    /// The rendered button list, used as the command-line prompt.
    pub(super) CONFIRM_BUTTONS: Option<XString> = None;
    /// Drop the temporary history entries before adding the next message.
    pub(super) DO_CLEAR_HIST_TEMP: bool = true;
    /// `'messagesopt'`'s flag set.
    pub(super) MSG_FLAGS: OptMoptFlags =
        kOptMoptFlagHitEnter | kOptMoptFlagHistory | kOptMoptFlagProgress;
    /// `'messagesopt'`'s `wait:` delay, in milliseconds.
    pub(super) MSG_WAIT: c_int = 0;
    /// Where `'messagesopt'`'s `progress:` sends progress messages.
    pub(super) PROGRESS_MSG_TARGET: c_int = PROGRESS_TARGET_CMD;
    /// Whether, and how much of, the scrollback to drop before the next
    /// message.
    pub(super) DO_CLEAR_SB_TEXT: ScrollbackClear = SB_CLEAR_NONE;
    /// The kept message is a `msgmore` report, which the next one may
    /// replace.
    pub(super) KEEP_MSG_MORE: bool = false;
    /// Nonzero while `msg_multihl` is emitting a chunk, so `msg_keep` knows
    /// not to start or end a message of its own.
    pub(super) IS_MULTIHL: c_int = 0;
    /// How deep `msg_keep` is in itself: a message whose display raises
    /// another stops at three.
    pub(super) MSG_KEEP_DEPTH: c_int = 0;
    /// `msg_source` is running, and must not report its own source.
    pub(super) MSG_SOURCE_BUSY: bool = false;
    /// The more prompt is up: a timer's message must not raise another.
    pub(super) MORE_PROMPT_BUSY: bool = false;
    /// The showmode text went away, and the UI is owed one empty event.
    pub(super) SHOWMODE_CLEAR_PENDING: bool = false;
    /// The script/function the last error was reported from.
    pub(super) LAST_SOURCING_NAME: Option<XString> = None;
    /// The line the last error was reported from.
    pub(super) LAST_SOURCING_LNUM: c_int = 0;
    /// The message kind in force when the current verbose section started.
    pub(super) PRE_VERBOSE_KIND: ApiString = ApiString::NULL;
    /// Whether opening `'verbosefile'` has been attempted, so the failure
    /// is reported once rather than on every message.
    pub(super) VERBOSE_DID_OPEN: bool = false;
    /// The column `redir_write` has written up to, tracked separately from
    /// `msg_col` because the redirection sees no screen.
    pub(super) REDIR_COL: c_int = 0;

    // -- the message being built for an ext_messages UI --
    /// The kind the message being composed carries, **owned**.
    ///
    /// The kind outlives the call that set it -- it is read again when the
    /// message is flushed to the UI or copied into the history -- so this
    /// keeps its own copy rather than the caller's pointer. `nvim_echo`'s
    /// `kind` lives in a keyset that dies with the call, which is how a
    /// borrowed kind became a use-after-free.
    pub(super) MSG_EXT_KIND: ApiString = ApiString::NULL;
    /// What caused the message being built, for a UI that groups by it.
    /// Both callers name a compiled-in kind, so the field holds the literal
    /// rather than a pointer whose lifetime it would have to answer for.
    pub(super) MSG_EXT_TRIGGER: Option<&'static CStr> = None;
    pub(super) MSG_EXT_ID: Object = Object::Integer(1);
    pub(super) MSG_EXT_CHUNKS: Option<Vec<Object>> = None;
    /// The text written under the current highlight, waiting to be closed
    /// off into a `msg_show` chunk by `msg_ext_emit_chunk`.
    pub(super) MSG_EXT_LAST_CHUNK: Vec<u8> = Vec::new();
    pub(super) MSG_EXT_LAST_ATTR: ScreenAttr = -1;
    pub(super) MSG_EXT_LAST_HL_ID: c_int = 0;
    pub(super) MSG_EXT_HISTORY: bool = false;
    pub(super) MSG_EXT_APPEND: bool = false;
    pub(super) MSG_GRID_POS_AT_FLUSH: c_int = 0;
    pub(super) MSG_ID_NEXT: int64_t = 1;
}
