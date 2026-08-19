//! Windows, frames and tab pages: the layout the editor draws into.
//!
//! Carved by the stage:
//!
//! | child | what |
//! | --- | --- |
//! | [`cmd`] | `do_window()` -- the CTRL-W commands |
//! | [`config`] | which buffer a window shows, and what the UI is told |
//! | [`split`] | `win_split_ins()` -- making a new window |
//! | [`order`] | exchange, rotate, move to an edge |
//! | [`equal`] | `win_equal()` and `'equalalways'` |
//! | [`close`] | may this window close, and `:only` |
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
//! `split_disallowed`/`close_disallowed`/`frame_locked` counters that back
//! them), the window-handle lookups every caller starts from (`win_valid`,
//! `win_valid_any_tab`, `win_find_by_handle`, `win_count`), and the small
//! globals the children read (`last_win_id`, `min_set_ch`,
//! `command_frame_height`).
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int};
use core::ptr;

use crate::api::private::helpers::{api_clear_error, api_set_error};
use crate::autocmd::{apply_autocmds, is_aucmd_win};
use crate::buffer::{bt_quickfix, buf_hide};
use crate::cursor::check_cursor;
use crate::drawscreen::redraw_all_later;
use crate::ex_docmd::do_cmdline_cmd;
use crate::ex_getln::{ERROR_INIT, is_in_cmdwin};
use crate::getchar::beep_flush;
use crate::global_cell::GlobalCell;
use crate::main::{
    c_bytes, curtab, curwin, e_not_allowed_to_change_window_layout_in_this_autocmd,
    e_winfixbuf_cannot_go_to_buffer, first_tabpage, firstwin, lastwin, prevwin, swb_flags,
    topframe,
};
use crate::map::map_put_ref_int_ptr_t;
use crate::memory::xfree;
use crate::message::{emsg, msg};
use crate::options::{kOptSwbFlagUseopen, kOptSwbFlagUsetab};
use crate::os::cshim::gettext;
use crate::terminal::terminal_check_size;
use crate::types::{
    AlignTextPos, CMD_tabnew, CdCause, Direction, Error, Map_int_ptr_t, MapHash, MotionType,
    OptInt, OptValType, Set_uint32_t, WinSplit, WinStyle, aucmdwin_T, bln_values, buf_T,
    cmd_addr_T, cmdidx_T, dobuf_action_values, dobuf_start_values, event_T, getf_values, handle_T,
    kErrorTypeException, kErrorTypeNone, ptr_t, size_t, tabpage_T, uint32_t, win_T,
};
use crate::ui_compositor::ui_comp_remove_grid;
use crate::winlayer::{Buf, Frame, TabPage, Win, tab_windows, windows, windows_in_tab};

// The carve of the transpiled module; see each child's docs.
mod alloc;
pub mod arith;
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
pub use self::frame::*;
pub use self::framesize::*;
pub use self::goto::*;
pub use self::order::*;
pub use self::resize::*;
pub use self::screensize::*;
pub use self::size::*;
pub use self::snapshot::*;
pub use self::split::*;
pub use self::tabpage::*;
pub use self::winclose::*;

pub const kAlignLeft: AlignTextPos = 0;
pub const kWinStyleMinimal: WinStyle = 1;
pub const kWinStyleUnused: WinStyle = 0;
pub const kWinSplitLeft: WinSplit = 0;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const kZIndexMessages: C2Rust_Unnamed_14 = 200;
pub const kZIndexFloatDefault: C2Rust_Unnamed_14 = 50;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const NUMBUFLEN: C2Rust_Unnamed_15 = 65;
pub const kDirectionNotSet: Direction = 0;
pub const kCdCauseWindow: CdCause = 1;
pub const kCdCauseManual: CdCause = 0;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_int;
pub const kWinOptScroll: C2Rust_Unnamed_16 = 33;
pub const kWinOptFoldtext: C2Rust_Unnamed_16 = 22;
pub const kOptValTypeString: OptValType = 2;
pub const kOptValTypeNumber: OptValType = 1;
pub const kOptValTypeBoolean: OptValType = 0;
pub const ADDR_NONE: cmd_addr_T = 11;
pub const ADDR_OTHER: cmd_addr_T = 10;
pub const ADDR_UNSIGNED: cmd_addr_T = 9;
pub const ADDR_QUICKFIX: cmd_addr_T = 8;
pub const ADDR_QUICKFIX_VALID: cmd_addr_T = 7;
pub const ADDR_TABS_RELATIVE: cmd_addr_T = 6;
pub const ADDR_TABS: cmd_addr_T = 5;
pub const ADDR_BUFFERS: cmd_addr_T = 4;
pub const ADDR_LOADED_BUFFERS: cmd_addr_T = 3;
pub const ADDR_ARGUMENTS: cmd_addr_T = 2;
pub const ADDR_WINDOWS: cmd_addr_T = 1;
pub const ADDR_LINES: cmd_addr_T = 0;
#[derive(Copy, Clone)]
pub struct C2Rust_Unnamed_20 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut aucmdwin_T,
}
pub const GETF_SWITCH: getf_values = 4;
pub const GETF_ALT: getf_values = 2;
pub const GETF_SETMARK: getf_values = 1;
pub const BLN_NOOPT: bln_values = 16;
pub const BLN_DUMMY: bln_values = 4;
pub const BLN_LISTED: bln_values = 2;
pub const BLN_CURBUF: bln_values = 1;
pub const DOBUF_WIPE: dobuf_action_values = 4;
pub const DOBUF_DEL: dobuf_action_values = 3;
pub const DOBUF_UNLOAD: dobuf_action_values = 2;
pub const DOBUF_GOTO: dobuf_action_values = 0;
pub const DOBUF_MOD: dobuf_start_values = 3;
pub const DOBUF_LAST: dobuf_start_values = 2;
pub const DOBUF_FIRST: dobuf_start_values = 1;
pub const DOBUF_CURRENT: dobuf_start_values = 0;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const BL_FIX: C2Rust_Unnamed_24 = 4;
pub const BL_SOL: C2Rust_Unnamed_24 = 2;
pub const BL_WHITE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const ECMD_NOWINENTER: C2Rust_Unnamed_25 = 64;
pub const ECMD_ALTBUF: C2Rust_Unnamed_25 = 32;
pub const ECMD_ADDBUF: C2Rust_Unnamed_25 = 16;
pub const ECMD_FORCEIT: C2Rust_Unnamed_25 = 8;
pub const ECMD_OLDBUF: C2Rust_Unnamed_25 = 4;
pub const ECMD_SET_HELP: C2Rust_Unnamed_25 = 2;
pub const ECMD_HIDE: C2Rust_Unnamed_25 = 1;
pub type C2Rust_Unnamed_26 = ::core::ffi::c_int;
pub const ECMD_ONE: C2Rust_Unnamed_26 = 1;
pub const ECMD_LAST: C2Rust_Unnamed_26 = -1;
pub const ECMD_LASTL: C2Rust_Unnamed_26 = 0;
pub const kMTLineWise: MotionType = 1;
pub const kMTCharWise: MotionType = 0;
pub type C2Rust_Unnamed_28 = ::core::ffi::c_uint;
pub const FIND_EVAL: C2Rust_Unnamed_28 = 4;
pub const FIND_STRING: C2Rust_Unnamed_28 = 2;
pub const FIND_IDENT: C2Rust_Unnamed_28 = 1;
pub type C2Rust_Unnamed_29 = ::core::ffi::c_uint;
pub const BCO_NOHELP: C2Rust_Unnamed_29 = 4;
pub const BCO_ENTER: C2Rust_Unnamed_29 = 1;
pub type C2Rust_Unnamed_30 = ::core::ffi::c_uint;
pub const CHECK_PATH: C2Rust_Unnamed_30 = 3;
pub const FIND_DEFINE: C2Rust_Unnamed_30 = 2;
pub const FIND_ANY: C2Rust_Unnamed_30 = 1;
pub type C2Rust_Unnamed_31 = ::core::ffi::c_uint;
pub const ACTION_SHOW_ALL: C2Rust_Unnamed_31 = 4;
pub const ACTION_SPLIT: C2Rust_Unnamed_31 = 3;
pub const ACTION_GOTO: C2Rust_Unnamed_31 = 2;
pub const ACTION_SHOW: C2Rust_Unnamed_31 = 1;
pub const WSP_QUICKFIX: ::core::ffi::c_uint = 1024;
pub const WSP_NOENTER: ::core::ffi::c_uint = 512;
pub const WSP_NEWLOC: ::core::ffi::c_uint = 256;
pub const WSP_ABOVE: ::core::ffi::c_uint = 128;
pub const WSP_BELOW: ::core::ffi::c_uint = 64;
pub const WSP_HELP: ::core::ffi::c_uint = 32;
pub const WSP_BOT: ::core::ffi::c_uint = 16;
pub const WSP_TOP: ::core::ffi::c_uint = 8;
pub const WSP_HOR: ::core::ffi::c_uint = 4;
pub const WSP_VERT: ::core::ffi::c_uint = 2;
pub const WSP_ROOM: ::core::ffi::c_uint = 1;
pub type C2Rust_Unnamed_33 = ::core::ffi::c_uint;
pub const STATUS_HEIGHT: C2Rust_Unnamed_33 = 1;
pub const MIN_LINES: C2Rust_Unnamed_33 = 2;
pub type C2Rust_Unnamed_34 = ::core::ffi::c_uint;
pub const LOWEST_WIN_ID: C2Rust_Unnamed_34 = 1000;
pub const WEE_TRIGGER_LEAVE_AUTOCMDS: C2Rust_Unnamed_35 = 16;
pub const WEE_TRIGGER_ENTER_AUTOCMDS: C2Rust_Unnamed_35 = 8;
pub const WEE_UNDO_SYNC: C2Rust_Unnamed_35 = 1;
pub const WEE_TRIGGER_NEW_AUTOCMDS: C2Rust_Unnamed_35 = 4;
pub const WEE_CURWIN_INVALID: C2Rust_Unnamed_35 = 2;
pub type C2Rust_Unnamed_35 = ::core::ffi::c_uint;
pub const INT64_MAX: ::core::ffi::c_long = 9223372036854775807 as ::core::ffi::c_long;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NULL_0: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096 as ::core::ffi::c_int;
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;
pub const VALID_WCOL: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const VALID_CROW: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const VALID_TOPLINE: ::core::ffi::c_int = 0x80 as ::core::ffi::c_int;
pub const SNAP_HELP_IDX: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const SNAP_QUICKFIX_IDX: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const SNAP_COUNT: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const FR_LEAF: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FR_ROW: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FR_COL: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const SET_INIT: Set_uint32_t = Set_uint32_t {
    h: MAPHASH_INIT,
    keys: ::core::ptr::null_mut::<uint32_t>(),
};
/// `pmap_put(int)`: store `value` under `key`.
#[inline]
fn map_put_int_ptr_t(map: &mut Map_int_ptr_t, key: ::core::ffi::c_int, value: ptr_t) {
    let (nokey, nonew) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: the map is live; a null `oldkey`/`new` means "do not report".
    let slot: *mut ptr_t = unsafe { map_put_ref_int_ptr_t(map, key, nokey, nonew) };
    // SAFETY: `map_put_ref` answers a live slot of the value array.
    unsafe { *slot = value };
}
pub const TAB: ::core::ffi::c_int = 9;
pub const CAR: ::core::ffi::c_int = 13;
pub const SID_WINLAYOUT: ::core::ffi::c_int = -7 as ::core::ffi::c_int;
pub const NOWIN: *mut win_T = -1 as ::core::ffi::c_int as *mut win_T;
static e_cannot_close_last_window: [::core::ffi::c_char; 31] =
    c_bytes(b"E444: Cannot close last window\0");
static e_cannot_split_window_when_closing_buffer: [::core::ffi::c_char; 53] =
    c_bytes(b"E1159: Cannot split a window when closing the buffer\0");
static m_onlyone: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(c"Already only one window".as_ptr() as *mut ::core::ffi::c_char);
static split_disallowed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static close_disallowed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
static frame_locked: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
pub fn window_layout_lock() {
    window_lock();
}

pub fn window_layout_unlock() {
    window_unlock();
}

/// Forbid splitting and closing windows until [`window_unlock`].
fn window_lock() {
    split_disallowed.set(split_disallowed.get() + 1);
    close_disallowed.set(close_disallowed.get() + 1);
}

/// Undo one [`window_lock`].
fn window_unlock() {
    split_disallowed.set(split_disallowed.get() - 1);
    close_disallowed.set(close_disallowed.get() - 1);
}

pub fn frames_locked() -> bool {
    frame_locked.get() != 0
}

pub fn window_layout_locked(cmd: cmdidx_T) -> bool {
    layout_locked(cmd)
}

/// Whether an autocommand has forbidden this change to the window layout,
/// reporting the reason itself.
fn layout_locked(cmd: cmdidx_T) -> bool {
    let mut e = ERROR_INIT;
    let locked = locked_err(cmd, &mut e);
    if e.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
        err(e.msg);
        // SAFETY: an error this call filled in, which owns its message.
        unsafe { api_clear_error(&raw mut e) };
    }
    locked
}

pub unsafe fn window_layout_locked_err(cmd: cmdidx_T, err: *mut Error) -> bool {
    // SAFETY: the caller's promise -- a writable error slot.
    locked_err(cmd, unsafe { &mut *err })
}

/// [`layout_locked`], reporting through `err` instead of the message area.
fn locked_err(cmd: cmdidx_T, err: &mut Error) -> bool {
    if split_disallowed.get() <= 0 && close_disallowed.get() <= 0 {
        return false;
    }
    let msg = if close_disallowed.get() == 0 && cmd as ::core::ffi::c_int == CMD_tabnew as c_int {
        e_cannot_split_window_when_closing_buffer.as_ptr()
    } else {
        &raw const e_not_allowed_to_change_window_layout_in_this_autocmd as *const c_char
    };
    set_err(err, msg);
    true
}

pub fn check_can_set_curbuf_disabled() -> bool {
    winfixbuf_allows()
}

pub fn check_can_set_curbuf_forceit(forceit: ::core::ffi::c_int) -> bool {
    forceit != 0 || winfixbuf_allows()
}

/// Whether the current window's `'winfixbuf'` lets another buffer in, saying
/// so if it does not.
fn winfixbuf_allows() -> bool {
    if cur_win().w_onebuf_opt.wo_wfb != 0 {
        err(&raw const e_winfixbuf_cannot_go_to_buffer as *const c_char);
        return false;
    }
    true
}

pub unsafe fn prevwin_curwin() -> *mut win_T {
    // SAFETY: reads the cmdline-window state, which is always set up.
    let in_cmdwin = unsafe { is_in_cmdwin() };
    let prev = prevwin.get();
    if in_cmdwin && !prev.is_null() {
        prev
    } else {
        curwin.get()
    }
}

pub unsafe fn swbuf_goto_win_with_buf(buf: *mut buf_T) -> *mut win_T {
    // SAFETY: the caller's promise -- a live buffer or null.
    raw_win(unsafe { Buf::from_raw(buf) }.and_then(swbuf_goto_win))
}

/// The window `'switchbuf'` says to jump to for `buf`, having jumped to it.
fn swbuf_goto_win(buf: Buf) -> Option<Win> {
    let flag = |f: ::core::ffi::c_int| swb_flags.get() & f as ::core::ffi::c_uint != 0;
    let mut wp = None;
    if flag(kOptSwbFlagUseopen as ::core::ffi::c_int) {
        wp = jump_open_win(buf);
    }
    if wp.is_none() && flag(kOptSwbFlagUsetab as ::core::ffi::c_int) {
        wp = jump_open_tab(buf);
    }
    wp
}
static min_set_ch: GlobalCell<OptInt> = GlobalCell::new(1 as OptInt);

// ---------------------------------------------------------------------------
// Is this window still there?
//
// These four take a raw `win_T *` and never dereference it, deliberately: they
// are asked about a pointer an autocommand may already have freed, and the
// whole answer is whether it is still on a list. Handing them a `Win` would
// mean promising exactly what the caller is asking about. `valid_win` below is
// the bridge back for a caller that wants to go on and use the window.

pub unsafe fn win_valid(win: *const win_T) -> bool {
    // SAFETY: `win` is only compared, never read; `curtab` is always set.
    unsafe { tabpage_win_valid(curtab.get(), win) }
}

pub unsafe fn tabpage_win_valid(tp: *const tabpage_T, win: *const win_T) -> bool {
    // SAFETY: the caller's promise -- a live tab page. `win` is only compared.
    valid_win_in_tab(unsafe { TabPage::new(tp as *mut tabpage_T) }, win)
}

/// Whether `win` is on `tp`'s window list. `win` is only compared.
fn valid_win_in_tab(tp: TabPage, win: *const win_T) -> bool {
    !win.is_null() && windows_in_tab(tp).any(|wp| wp.raw() as *const win_T == win)
}

pub fn win_find_by_handle(handle: handle_T) -> *mut win_T {
    windows()
        .find(|wp| wp.handle == handle)
        .map_or(ptr::null_mut(), Win::raw)
}

pub fn win_valid_any_tab(win: *mut win_T) -> bool {
    valid_win_any_tab(win)
}

/// Whether `win` is on the window list of any tab page. `win` is only compared.
fn valid_win_any_tab(win: *mut win_T) -> bool {
    !win.is_null() && tab_windows().any(|wp| wp.raw() == win)
}

pub fn win_count() -> ::core::ffi::c_int {
    windows().count() as ::core::ffi::c_int
}

/// The window `win` names, if it is still on the current tab page's list.
///
/// The one-line bridge from [`win_valid`]'s pointer answer to a value the rest
/// of the family may dereference.
pub(crate) fn valid_win(win: *mut win_T) -> Option<Win> {
    // SAFETY: the walk only produced windows that are on the list.
    windows().find(|wp| wp.raw() == win)
}

// ---------------------------------------------------------------------------
// The neighbours more than one child reaches
//
// Everything the window family calls outside it is still an `unsafe fn`
// over raw pointers, and all any of these needs is a live window or tab page --
// which `Win` and `TabPage` already carry. Wrapping each *exit* once here, in
// the module every child can see through `use super::*`, costs one unchecked
// line per neighbour rather than one per call site.

/// `emsg(_(msg))`, the family's only way of reporting a failure.
fn err(msg: *const ::core::ffi::c_char) {
    // SAFETY: every caller passes a static NUL-terminated message.
    unsafe { emsg(gettext(msg)) };
}

/// Mark every window on the screen for redrawing at `redraw_type`.
fn redraw_all(redraw_type: ::core::ffi::c_int) {
    // SAFETY: reads the window list, which is live from startup to exit.
    unsafe { redraw_all_later(redraw_type) };
}

/// Whether `win` is the only non-floating window of its tab page.
fn is_only_window(win: Win, tp: Option<TabPage>) -> bool {
    only_window(win, tp)
}

/// `emsg()` over a message the caller has already translated, or that upstream
/// deliberately does not translate.
fn err_raw(msg: *const ::core::ffi::c_char) {
    // SAFETY: every caller passes a static NUL-terminated message.
    unsafe { emsg(msg) };
}

/// "Already only one window", the answer to `:only` and CTRL-W T when there is
/// nothing to do.
fn only_one_message() {
    // SAFETY: a static message; zero means "no highlight attribute".
    unsafe { msg(gettext(m_onlyone.get()), 0) };
}

/// Whether `win` is one of the hidden windows autocommands are executed in.
fn is_autocmd_window(win: Option<Win>) -> bool {
    // SAFETY: a live window, or the null the callers pass for "no window".
    unsafe { is_aucmd_win(win.map_or(ptr::null_mut(), Win::raw)) }
}

/// `xfree`, for the frames and click definitions the family owns.
fn free<T>(ptr: *mut T) {
    // SAFETY: every caller passes a pointer from the `xmalloc` family, or null.
    unsafe { xfree(ptr as *mut ::core::ffi::c_void) };
}

/// A tab page as the family's entry points take it: null for "the current one".
fn raw_tab(tp: Option<TabPage>) -> *mut tabpage_T {
    tp.map_or(ptr::null_mut(), TabPage::raw)
}

/// A window argument that may be absent, as the entry points take it.
fn raw_win(win: Option<Win>) -> *mut win_T {
    win.map_or(ptr::null_mut(), Win::raw)
}

/// The root of the current tab page's layout tree.
fn current_topframe() -> Frame {
    // SAFETY: `topframe` is set from startup to exit.
    unsafe { Frame::new(topframe.get()) }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

/// The tab page the editor is working in.
fn cur_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The first window of the current tab page.
fn first_win() -> Win {
    // SAFETY: `firstwin` is set from startup to exit.
    unsafe { Win::new(firstwin.get()) }
}

/// The last window of the current tab page, floats included.
fn last_win() -> Win {
    // SAFETY: `lastwin` is set from startup to exit.
    unsafe { Win::new(lastwin.get()) }
}

/// The first tab page, which is the only one when it has no `tp_next`.
fn first_tab() -> TabPage {
    // SAFETY: `first_tabpage` is set from startup to exit.
    unsafe { TabPage::new(first_tabpage.get()) }
}

/// `api_set_error(err, kErrorTypeException, "%s", msg)`.
fn set_err(err: &mut Error, msg: *const ::core::ffi::c_char) {
    // SAFETY: an error slot to fill in, and a NUL-terminated message static.
    unsafe { api_set_error(err, kErrorTypeException, c"%s".as_ptr(), msg) };
}

/// `apply_autocmds(event, NULL, NULL, false, buf)`.
///
/// **Nothing derived from a window or buffer survives this call**: the event
/// may close windows, switch tab pages or wipe the buffer.
fn fire(event: event_T, buf: Buf) -> bool {
    let (none, raw) = (ptr::null_mut(), buf.raw());
    // SAFETY: a live buffer; both name arguments are optional.
    unsafe { apply_autocmds(event, none, none, false, raw) }
}

/// [`fire`] with a name, which the event reports as `<afile>` and matches
/// against: a window id for `WinClosed`, a tab page index for `TabClosed`, a
/// file name for `TabNew`. `None` is the buffer-less form two events take.
fn fire_named(event: event_T, name: *mut ::core::ffi::c_char, buf: Option<Buf>) -> bool {
    let buf = buf.map_or(ptr::null_mut(), Buf::raw);
    // SAFETY: a live buffer or null, and a NUL-terminated name or null.
    unsafe { apply_autocmds(event, name, name, false, buf) }
}

/// Ring the bell and drop the typeahead, the family's answer to a move that
/// cannot be made.
fn beep() {
    // SAFETY: reads no argument of ours.
    unsafe { beep_flush() };
}

/// Whether `buf` may be left in a window that is closing (`'hidden'`,
/// `'bufhidden'`).
fn hides(buf: Buf) -> bool {
    // SAFETY: a live buffer.
    unsafe { buf_hide(buf.raw()) }
}

/// Whether `buf` is a quickfix or location list buffer.
fn is_quickfix(buf: Option<Buf>) -> bool {
    let raw = buf.map_or(ptr::null(), |b| b.raw() as *const buf_T);
    // SAFETY: a live buffer, or the null the callers pass for "no buffer".
    unsafe { bt_quickfix(raw) }
}

/// Clamp the cursor of `win` back into its buffer.
fn revalidate_cursor(win: Win) {
    // SAFETY: a live window.
    unsafe { check_cursor(win.raw()) };
}

/// Tell a terminal buffer its window changed size, if `buf` has one.
fn resize_terminal(buf: Buf) {
    let term = buf.terminal;
    if !term.is_null() {
        // SAFETY: a live buffer's terminal.
        unsafe { terminal_check_size(term) };
    }
}

/// Run one Ex command line, as if the user had typed it.
fn run_cmd(cmd: *const ::core::ffi::c_char) {
    // SAFETY: a NUL-terminated command line.
    unsafe { do_cmdline_cmd(cmd) };
}

/// Drop `win`'s external grid, which the compositor still holds.
fn drop_grid(win: Win) {
    let mut win = win;
    // SAFETY: the window's own grid.
    unsafe { ui_comp_remove_grid(&raw mut win.w_grid_alloc) };
}

static command_frame_height: GlobalCell<bool> = GlobalCell::new(true_0 != 0);
static last_win_id: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(LOWEST_WIN_ID as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
static did_initial_scroll_size_snapshot: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
pub const FRACTION_MULT: ::core::ffi::c_int = 16384 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
