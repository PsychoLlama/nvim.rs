//! Windows, frames and tab pages: the layout the editor draws into.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`cmd`] | `do_window()` -- the CTRL-W commands |
//! | [`config`] | which buffer a window shows, and what the UI is told |
//! | [`mod@split`] | `win_split_ins()` -- making a new window |
//! | [`order`] | exchange, rotate, move to an edge |
//! | [`mod@equal`] | `win_equal()` and `'equalalways'` |
//! | [`mod@close`] | may this window close, and `:only` |
//! | [`winclose`] | `win_close()` -- closing one window |
//! | [`frame`] | the frame tree: remove a leaf, give its room away |
//! | [`framesize`] | frame arithmetic and the minimum sizes |
//! | [`alloc`] | allocating windows and frames, and the lists they live on |
//! | [`arith`] | the size arithmetic, with no pointer and no global in it |
//! | [`tabpage`] | tab pages -- create, switch, close |
//! | [`goto`] | entering a window, and the directional moves |
//! | [`screensize`] | the screen resized, and `WinScrolled`/`WinResized` |
//! | [`size`] | `:resize`, and dragging a separator |
//! | [`resize`] | applying a size, and the rows that are not text |
//! | [`snapshot`] | saving and restoring a layout, and revalidation |
//!
//! What stays here is the constant alphabet the sixteen share (`WSP_*`,
//! `FR_*`, `WEE_*`, `SNAP_*`), the layout locks (`window_layout_lock` and the
//! `SPLIT_DISALLOWED`/`CLOSE_DISALLOWED`/`FRAME_LOCKED` counters that back
//! them), the window-handle lookups every caller starts from (`win_valid`,
//! `win_valid_any_tab`, `win_find_by_handle`, `win_count`), and the small
//! globals the children read (`LAST_WIN_ID`, `MIN_SET_CH`,
//! `COMMAND_FRAME_HEIGHT`).
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.

pub(crate) mod state;
use crate::ascii::CAR;
use crate::ascii::TAB;
use crate::autocmd::defs::AutoEvent;
use crate::cstr;
use crate::memory::XString;
use crate::narrow::count_as_int;
use crate::types::CmdIdx;
use crate::winlayer::WinId;
use crate::winlayer::prev_window;
use core::ffi::c_int;

use crate::autocmd::{apply_autocmds, is_aucmd_win};
use crate::buffer::{buf_hide, buf_is_quickfix};
use crate::cursor::check_cursor;
use crate::drawscreen::redraw_all_later;
use crate::ex_docmd::do_cmdline_cmd;
use crate::ex_getln::is_in_cmdwin;
use crate::getchar::beep_flush;
use crate::global_cell::GlobalCell;
use crate::message::{E_AUTOCMD_CLOSE, E_CMDWIN, E_NOALT, E_NOROOM, emsg, msg};
use crate::message::{
    E_NOT_ALLOWED_TO_CHANGE_WINDOW_LAYOUT_IN_THIS_AUTOCMD, E_WINFIXBUF_CANNOT_GO_TO_BUFFER,
};
use crate::normal::defs::MotionType;
use crate::option::vars::SWB_FLAGS;
use crate::options::{OPT_SWB_FLAG_USEOPEN, OPT_SWB_FLAG_USETAB};
use crate::os::cshim::gettext;
use crate::terminal::buf_terminal_check_size;
use crate::types::Failed;
use crate::types::{
    CdCause, Direction, DoBufStart, ERROR_TYPE_EXCEPTION, Error, Handle, OptInt, Window,
};
use crate::ui_compositor::ui_comp_remove_grid;
use crate::winlayer::graph::{FIRST_TABPAGE, FIRSTWIN, LASTWIN};
use crate::winlayer::{
    Buf, FrameRef, TabPage, Win, current_topframe, new_frame, tab_windows, windows, windows_in_tab,
};

// The carve of the transpiled module; see each child's docs.
mod alloc;
pub(crate) mod arith;
mod close;
mod cmd;
mod config;
mod equal;
mod frame;
mod framesize;
mod goto;
mod order;
mod resize;
mod screensize;
mod size;
mod snapshot;
mod split;
mod tabpage;
mod winclose;

pub use self::alloc::*;
pub use self::close::*;
pub use self::cmd::*;
pub use self::config::*;
pub use self::equal::*;
pub(crate) use self::frame::*;
pub(crate) use self::framesize::*;
pub use self::goto::*;
pub use self::order::*;
pub use self::resize::*;
pub use self::screensize::*;
pub use self::size::*;
pub use self::snapshot::*;
pub use self::split::*;
pub use self::tabpage::*;
pub use self::winclose::*;

pub const Z_INDEX_MESSAGES: c_int = 200;
pub const Z_INDEX_FLOAT_DEFAULT: c_int = 50;
pub const DIRECTION_NOT_SET: Direction = 0;
pub const CD_CAUSE_WINDOW: CdCause = 1;
pub const CD_CAUSE_MANUAL: CdCause = 0;
pub const DOBUF_LAST: DoBufStart = 2;
pub const MT_LINE_WISE: MotionType = 1;
pub const MT_CHAR_WISE: MotionType = 0;
pub const WSP_QUICKFIX: c_int = 1024;
pub const WSP_NOENTER: c_int = 512;
pub const WSP_NEWLOC: c_int = 256;
pub const WSP_ABOVE: c_int = 128;
pub const WSP_BELOW: c_int = 64;
pub const WSP_HELP: c_int = 32;
pub const WSP_BOT: c_int = 16;
pub const WSP_TOP: c_int = 8;
pub const WSP_HOR: c_int = 4;
pub const WSP_VERT: c_int = 2;
pub const WSP_ROOM: c_int = 1;
pub const STATUS_HEIGHT: c_int = 1;
pub const MIN_LINES: c_int = 2;
pub const LOWEST_WIN_ID: c_int = 1000;
pub const WEE_TRIGGER_LEAVE_AUTOCMDS: c_int = 16;
pub const WEE_TRIGGER_ENTER_AUTOCMDS: c_int = 8;
pub const WEE_UNDO_SYNC: c_int = 1;
pub const WEE_TRIGGER_NEW_AUTOCMDS: c_int = 4;
pub const WEE_CURWIN_INVALID: c_int = 2;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const SNAP_HELP_IDX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SNAP_QUICKFIX_IDX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SNAP_COUNT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SID_WINLAYOUT: ::core::ffi::c_int = -7 as ::core::ffi::c_int;
pub const NOWIN: *mut Window = -1 as ::core::ffi::c_int as *mut Window;
static E_CANNOT_CLOSE_LAST_WINDOW: &::core::ffi::CStr = c"E444: Cannot close last window";
static E_CANNOT_SPLIT_WINDOW_WHEN_CLOSING_BUFFER: &::core::ffi::CStr =
    c"E1159: Cannot split a window when closing the buffer";
const M_ONLYONE: &::core::ffi::CStr = c"Already only one window";
static SPLIT_DISALLOWED: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static CLOSE_DISALLOWED: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static FRAME_LOCKED: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub fn window_layout_lock() {
    window_lock();
}

pub fn window_layout_unlock() {
    window_unlock();
}

/// Forbid splitting and closing windows until [`window_unlock`].
fn window_lock() {
    SPLIT_DISALLOWED.set(SPLIT_DISALLOWED.get() + 1);
    CLOSE_DISALLOWED.set(CLOSE_DISALLOWED.get() + 1);
}

/// Undo one [`window_lock`].
fn window_unlock() {
    SPLIT_DISALLOWED.set(SPLIT_DISALLOWED.get() - 1);
    CLOSE_DISALLOWED.set(CLOSE_DISALLOWED.get() - 1);
}

pub fn frames_locked() -> bool {
    FRAME_LOCKED.get() != 0
}

pub fn window_layout_locked(cmd: CmdIdx) -> bool {
    layout_locked(cmd)
}

/// Whether an autocommand has forbidden this change to the window layout,
/// reporting the reason itself.
fn layout_locked(cmd: CmdIdx) -> bool {
    match locked_err(cmd) {
        Ok(()) => false,
        Err(e) => {
            // The refusal's message is already translated; it is not a msgid.
            emsg(e.message_or_empty());
            true
        }
    }
}

pub fn window_layout_locked_err(cmd: CmdIdx) -> Result<(), Error> {
    locked_err(cmd)
}

/// [`layout_locked`], answering the reason instead of putting it in the
/// message area.
fn locked_err(cmd: CmdIdx) -> Result<(), Error> {
    if SPLIT_DISALLOWED.get() <= 0 && CLOSE_DISALLOWED.get() <= 0 {
        return Ok(());
    }
    let msg = if CLOSE_DISALLOWED.get() == 0 && cmd == CmdIdx::tabnew {
        E_CANNOT_SPLIT_WINDOW_WHEN_CLOSING_BUFFER
    } else {
        E_NOT_ALLOWED_TO_CHANGE_WINDOW_LAYOUT_IN_THIS_AUTOCMD
    };
    Err(make_err(msg))
}

/// Whether the current window may show another buffer: its 'winfixbuf'
/// refuses, out loud.
pub(crate) fn check_can_set_curbuf_disabled() -> Result<(), WinError> {
    winfixbuf_allows()
}

/// [`check_can_set_curbuf_disabled`], which a `!` overrides.
pub(crate) fn check_can_set_curbuf_forceit(forceit: bool) -> Result<(), WinError> {
    if forceit { Ok(()) } else { winfixbuf_allows() }
}

/// Whether the current window's `'winfixbuf'` lets another buffer in, saying
/// so if it does not.
fn winfixbuf_allows() -> Result<(), WinError> {
    if Win::current().w_onebuf_opt.wo_wfb != 0 {
        return Err(err(WinError::WinFixBuf));
    }
    Ok(())
}

pub(crate) fn prevwin_curwin() -> Win {
    match prev_window() {
        Some(prev) if is_in_cmdwin() => prev,
        _ => Win::current(),
    }
}

pub fn swbuf_goto_win_with_buf(buffer: Option<Buf>) -> Option<Win> {
    buffer.and_then(swbuf_goto_win)
}

/// The window `'switchbuf'` says to jump to for `buffer`, having jumped to it.
fn swbuf_goto_win(buffer: Buf) -> Option<Win> {
    let flag = |f: ::core::ffi::c_uint| SWB_FLAGS.get() & f != 0;
    let mut wp = None;
    if flag(OPT_SWB_FLAG_USEOPEN) {
        wp = jump_open_win(buffer);
    }
    if wp.is_none() && flag(OPT_SWB_FLAG_USETAB) {
        wp = jump_open_tab(buffer);
    }
    wp
}
static MIN_SET_CH: GlobalCell<OptInt> = GlobalCell::new(1 as OptInt);

// ---------------------------------------------------------------------------
// Is this window still there?
//
// These four take a [`WinId`] and never a window: they are asked about a
// window an autocommand may already have closed, and the whole answer is
// whether it is still on a list. Taking a `Win` would mean promising exactly
// what the caller is asking about; taking an *address* — which is what they
// took while the C's shape survived — meant answering "yes" for a window the
// allocator had since handed out again at the same place. `valid_win` below
// is the bridge back for a caller that wants to go on and use the window.
//
// They stay list walks, and the handle registry (`winlayer::window`) does not
// replace them: `win_valid` is **tab-scoped** — a window on another tab page
// is not "valid" — while the registry knows nothing about tab pages, and
// holds windows that are on no list at all (a hidden `win_alloc`). The three
// answers are genuinely different: `win_valid` (this tab page),
// `win_valid_any_tab` (any tab page's list) and `WinId::get` (allocated and
// not yet freed).
//
// A window list is a handful of entries, so the walk is not the cost the O(n)
// makes it look.

/// Whether `win` is a window on the **current tab page**.
pub(crate) fn win_valid(win: WinId) -> bool {
    valid_win_in_tab(TabPage::current(), win)
}

pub(crate) fn tabpage_win_valid(tabpage: TabPage, win: WinId) -> bool {
    valid_win_in_tab(tabpage, win)
}

/// Whether `win` is on `tabpage`'s window list.
fn valid_win_in_tab(tabpage: TabPage, win: WinId) -> bool {
    windows_in_tab(tabpage).any(|wp| wp.id() == win)
}

pub fn win_find_by_handle(handle: Handle) -> Option<Win> {
    windows().find(|wp| wp.handle == handle)
}

pub(crate) fn win_valid_any_tab(win: WinId) -> bool {
    valid_win_any_tab(win)
}

/// Whether `win` is on the window list of any tab page.
fn valid_win_any_tab(win: WinId) -> bool {
    tab_windows().any(|wp| wp.id() == win)
}

pub fn win_count() -> ::core::ffi::c_int {
    count_as_int(windows().count())
}

/// The window `win` names, if it is still on the current tab page's list.
///
/// The one-line bridge from [`win_valid`]'s yes-or-no to a value the rest of
/// the family may dereference.
pub(crate) fn valid_win(win: WinId) -> Option<Win> {
    windows().find(|wp| wp.id() == win)
}

// ---------------------------------------------------------------------------
// The neighbours more than one child reaches
//
// Everything the window family calls outside it is still an `unsafe fn`
// over raw pointers, and all any of these needs is a live window or tab page --
// which `Win` and `TabPage` already carry. Wrapping each *exit* once here, in
// the module every child can see through `use super::*`, costs one unchecked
// line per neighbour rather than one per call site.

/// A refusal the window family has said, at the point it found it.
///
/// Every variant is reported once, by [`err`], where the refusal is found --
/// `emsg` sets `v:errmsg` and raises what `:try` catches there -- so a caller
/// holding one only decides whether to go on, and may turn it into a
/// [`Failed`]. The refusals that say nothing are not here: they are their
/// own answers ([`NotSplit::AutocmdWindow`], most of [`NotClosed`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WinError {
    /// E36: the layout has no line or column to give.
    NoRoom,
    /// E1513: the current window's 'winfixbuf' keeps another buffer out.
    WinFixBuf,
    /// E813: an autocommand window may not be closed.
    AutocmdClose,
    /// E814: closing would leave only the autocommand window.
    OnlyAutocmdWindow,
    /// E444: the last window may not be closed.
    LastWindow,
    /// E11: not in the command-line window.
    CmdWin,
    /// E23: there is no alternate file to split.
    NoAlternate,
    /// E441: there is no preview window.
    NoPreviewWindow,
    /// E442: `:topleft` and `:botright` together.
    TopAndBottom,
    /// E443: a window to rotate past is split the other way.
    RotateSplit,
    /// E445: another window holds changes.
    OtherWindowChanged,
}

impl WinError {
    /// The message, untranslated.
    fn msgid(self) -> &'static ::core::ffi::CStr {
        match self {
            Self::NoRoom => E_NOROOM,
            Self::WinFixBuf => E_WINFIXBUF_CANNOT_GO_TO_BUFFER,
            Self::AutocmdClose => E_AUTOCMD_CLOSE,
            Self::OnlyAutocmdWindow => {
                c"E814: Cannot close window, only autocmd window would remain"
            }
            Self::LastWindow => E_CANNOT_CLOSE_LAST_WINDOW,
            Self::CmdWin => E_CMDWIN,
            Self::NoAlternate => E_NOALT,
            Self::NoPreviewWindow => c"E441: There is no preview window",
            Self::TopAndBottom => c"E442: Can't split topleft and botright at the same time",
            Self::RotateSplit => c"E443: Cannot rotate when another window is split",
            Self::OtherWindowChanged => c"E445: Other window contains changes",
        }
    }
}

impl core::fmt::Display for WinError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.msgid().to_string_lossy())
    }
}

impl core::error::Error for WinError {}

/// The message is out: what is left for a caller outside the family.
impl From<WinError> for Failed {
    fn from(_: WinError) -> Self {
        Self
    }
}

/// Say `refusal` -- `emsg(_(msg))`, the family's only way of reporting one
/// -- and answer it.
fn err(refusal: WinError) -> WinError {
    emsg(gettext(refusal.msgid()));
    refusal
}

/// Mark every window on the screen for redrawing at `redraw_type`.
fn redraw_all(redraw_type: ::core::ffi::c_int) {
    redraw_all_later(redraw_type);
}

/// Whether `win` is the only non-floating window of its tab page.
fn is_only_window(win: Win, tabpage: Option<TabPage>) -> bool {
    only_window(win, tabpage)
}

/// `emsg()` over a message the caller has already translated, or that upstream
/// deliberately does not translate.
fn err_raw(msg: &::core::ffi::CStr) {
    emsg(msg);
}

/// "Already only one window", the answer to `:only` and CTRL-W T when there is
/// nothing to do.
fn only_one_message() {
    // Zero means "no highlight attribute".
    msg(gettext(M_ONLYONE), 0);
}

/// Whether `win` is one of the hidden windows autocommands are executed in.
fn is_autocmd_window(win: Option<Win>) -> bool {
    win.is_some_and(is_aucmd_win)
}

/// The first window of the current tab page.
fn first_win() -> Win {
    crate::winlayer::first_window().expect("the editor always has a window")
}

/// The last window of the current tab page, floats included.
fn last_win() -> Win {
    crate::winlayer::last_window().expect("the editor always has a window")
}

/// The first tab page, which is the only one when it has no `tp_next`.
///
/// The fallible form is `winlayer::first_tab`; the editor has a tab page
/// from startup to exit, which is the promise this one takes.
fn first_tab() -> TabPage {
    crate::winlayer::first_tab().expect("the editor always has a tab page")
}

/// An exception whose whole message is `msg`.
pub(crate) fn make_err(msg: &::core::ffi::CStr) -> Error {
    Error::from_message(ERROR_TYPE_EXCEPTION, msg)
}

/// `apply_autocmds(event, NULL, NULL, false, buf)`.
///
/// **Nothing derived from a window or buffer survives this call**: the event
/// may close windows, switch tab pages or wipe the buffer.
fn fire(event: AutoEvent, buffer: Buf) -> bool {
    apply_autocmds(event, None, None, false, Some(buffer))
}

/// [`fire`] with a name, which the event reports as `<afile>` and matches
/// against: a window id for `WinClosed`, a tab page index for `TabClosed`, a
/// file name for `TabNew`. `None` is the buffer-less form two events take.
fn fire_named(event: AutoEvent, name: Option<&::core::ffi::CStr>, buffer: Option<Buf>) -> bool {
    let name = name.map(XString::from_cstr);
    apply_autocmds(event, name.clone(), name, false, buffer)
}

/// Ring the bell and drop the typeahead, the family's answer to a move that
/// cannot be made.
fn beep() {
    beep_flush();
}

/// Whether `buffer` may be left in a window that is closing (`'hidden'`,
/// `'bufhidden'`).
fn hides(buffer: Buf) -> bool {
    buf_hide(buffer)
}

/// Clamp the cursor of `win` back into its buffer.
fn revalidate_cursor(win: Win) {
    check_cursor(win);
}

/// Tell a terminal buffer its window changed size, if `buffer` has one.
fn resize_terminal(buffer: Buf) {
    buf_terminal_check_size(buffer);
}

/// Run one Ex command line, as if the user had typed it.
fn run_cmd(cmd: &::core::ffi::CStr) {
    let _ = do_cmdline_cmd(cmd);
}

/// Drop `win`'s external grid, which the compositor still holds.
fn drop_grid(win: Win) {
    ui_comp_remove_grid(win.grid_alloc());
}

static COMMAND_FRAME_HEIGHT: GlobalCell<bool> = GlobalCell::new(true);
static LAST_WIN_ID: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(LOWEST_WIN_ID - 1 as ::core::ffi::c_int);
static DID_INITIAL_SCROLL_SIZE_SNAPSHOT: GlobalCell<bool> = GlobalCell::new(false);
pub const FRACTION_MULT: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
