//! Floating windows: creating one, turning its config into geometry, and the
//! walks over the floats of a tab page.
//!
//! A float is a window that is not in the frame tree. It sits at the end of
//! its tab page's window list -- which is why every walk here runs backwards
//! from `lastwin` while `w_floating` holds -- and its size and screen position
//! come from a [`WinConfig`] instead of from a parent frame.
//! [`win_config_float`] is where that config becomes geometry; window.rs's
//! `ui_ext_win_position` is its other half, applying the `anchor` and the
//! on-screen clamp when the float is drawn.
//!
//! Every entry point keeps the C's raw signature and wraps its window in
//! [`Win`] on the first line, so the bodies below are ordinary field access.
//! The neighbours this file reaches into are all still transpiled `unsafe fn`s
//! over raw pointers, and each is reached through
//! exactly one wrapper below rather than through an `unsafe` at every call
//! site; those wrappers are safe because their whole precondition is "a live
//! window, tab page or buffer", which [`Win`], [`TabPage`] and [`Buf`] carry.
//!
//! Original: `src/nvim/winfloat.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int, c_void};
use core::iter;
use core::ptr::{self, NonNull};

use crate::api::private::helpers::{
    api_clear_error, api_set_error, find_buffer_by_handle, find_window_by_handle,
};
use crate::api::vim::nvim_create_buf;
use crate::autocmd::{block_autocmds, unblock_autocmds};
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID, set_must_redraw};
use crate::grid::grid_adjust;
use crate::main::{Columns, Rows, cmdwin_win, e_cmdwin, firstwin, lastwin, p_ch, p_ls, prevwin};
use crate::memory::{xfree, xstrdup};
use crate::message::emsg;
use crate::mouse::{MousePos, find_win_inner};
use crate::r#move::textpos2screenpos;
use crate::option::{parse_winhl_opt, set_option_direct_for};
use crate::options::kOptBufhidden;
use crate::optionstr::{clear_string_option, free_string_option};
use crate::strings::concat_str;
use crate::types::ui::kUIMultigrid;
use crate::types::{
    AlignTextPos, Buffer, Error, FAIL, FloatAnchor, OPT_LOCAL, OptInt, OptScope, OptVal,
    OptValData, OptValType, String_0, VirtText, WinConfig, WinSplit, WinStyle, Window, colnr_T,
    kErrorTypeException, kErrorTypeNone, kFloatRelativeCursor, kFloatRelativeEditor,
    kFloatRelativeLaststatus, kFloatRelativeMouse, kFloatRelativeWindow, linenr_T, lpos_T, pos_T,
    schar_T, tabpage_T, win_T,
};
use crate::ui::ui_has;
use crate::window::{
    last_status, lastwin_nofloating, merge_win_config, tabpage_win_valid, win_alloc, win_append,
    win_close, win_comp_pos, win_enter, win_find_tabpage, win_free, win_init, win_remove,
    win_remove_status_line, win_set_buf, win_set_inner_size, win_valid, winframe_remove,
};
use crate::winlayer::{Buf, TabPage, Win, windows_in_tab};
use ::libc::{qsort, strlen};

/// Above this `zindex` a float is not capped by 'cmdheight'.
const kZIndexMessages: c_int = 200;
const kZIndexFloatDefault: c_int = 50;
const kAlignLeft: AlignTextPos = 0;
const kWinSplitLeft: WinSplit = 0;
const kWinStyleUnused: WinStyle = 0;
const kWinStyleMinimal: WinStyle = 1;
const kOptValTypeString: OptValType = 2;
const kOptScopeBuf: OptScope = 2;
const STATUS_HEIGHT: c_int = 1;

/// An unset title/footer, and an error slot holding no error.
const NO_VIRT_TEXT: VirtText = VirtText {
    size: 0,
    capacity: 0,
    items: ptr::null_mut(),
};
const NO_ERROR: Error = Error {
    type_0: kErrorTypeNone,
    msg: ptr::null_mut(),
};

/// `WIN_CONFIG_INIT`, the config a float starts from before the caller sets
/// the fields it cares about. (`popupmenu/draw.rs` keeps its own copy of the
/// same C macro, for the border it draws without a window.)
pub(crate) const WIN_CONFIG_INIT: WinConfig = WinConfig {
    window: 0,
    bufpos: lpos_T { lnum: -1, col: 0 },
    height: 0,
    width: 0,
    row: 0.0,
    col: 0.0,
    anchor: 0,
    relative: kFloatRelativeEditor,
    external: false,
    focusable: true,
    mouse: true,
    split: kWinSplitLeft,
    zindex: kZIndexFloatDefault,
    style: kWinStyleUnused,
    border: false,
    shadow: false,
    border_chars: [[0; 32]; 8],
    border_hl_ids: [0; 8],
    border_attr: [0; 8],
    title: false,
    title_pos: kAlignLeft,
    title_chunks: NO_VIRT_TEXT,
    title_width: 0,
    footer: false,
    footer_pos: kAlignLeft,
    footer_chunks: NO_VIRT_TEXT,
    footer_width: 0,
    noautocmd: false,
    fixed: false,
    hide: false,
    _cmdline_offset: c_int::MAX,
};

// ---------------------------------------------------------------------------
// The tab page

/// `NULL` for "the current tab page", as window.rs spells it.
fn raw_tab(tp: Option<TabPage>) -> *mut tabpage_T {
    tp.map_or(ptr::null_mut(), TabPage::raw)
}

fn current_tab() -> TabPage {
    // SAFETY: `curtab` is set from startup to exit.
    unsafe { TabPage::current() }
}

fn current_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}

// ---------------------------------------------------------------------------
// The walks

/// The floats of the current tab page, topmost first -- upstream's
/// `for (wp = lastwin; wp && wp->w_floating; wp = wp->w_prev)`. Floats live at
/// the end of the window list, so walking back from `lastwin` while
/// `w_floating` holds visits exactly them. Lazy, as the C's is.
fn floats() -> impl Iterator<Item = Win> {
    let mut next = lastwin.get();
    iter::from_fn(move || {
        // SAFETY: `lastwin`, and every `w_prev` reached from it, is a live
        // window or null.
        let win = (!next.is_null()).then(|| unsafe { Win::new(next) })?;
        if !win.w_floating {
            return None;
        }
        next = win.w_prev;
        Some(win)
    })
}

// ---------------------------------------------------------------------------
// Window-local string options
//
// A string option holds null, the shared empty string every option set to ""
// points at, or a NUL-terminated allocation the window owns. That is the
// whole invariant these five need.

/// The first byte of a string option's value, NUL when it is null. Upstream
/// spells the same test three ways -- `*p != NUL`, `p && *p != NUL` and
/// `p[0] != 'a'`; the unconditional forms would be undefined on a null
/// option, and answering NUL for one lands on the same arm they do.
fn opt_head(s: *const c_char) -> c_char {
    if s.is_null() {
        return 0;
    }
    // SAFETY: a non-null string option is NUL-terminated, so its first byte
    // is there to be read.
    unsafe { *s }
}

fn opt_is_set(s: *const c_char) -> bool {
    opt_head(s) != 0
}

fn opt_len(s: *const c_char) -> usize {
    // SAFETY: a NUL-terminated string option, its caller having ruled out null.
    unsafe { strlen(s) }
}

/// Free a string option's value and put `new` in its place.
fn set_opt(slot: &mut *mut c_char, new: *mut c_char) {
    // SAFETY: an option variable holds what `free_string_option` accepts --
    // null, the shared empty string (which it skips) or an owned allocation.
    unsafe { free_string_option(*slot) };
    *slot = new;
}

/// Free a string option's value and leave it holding the shared empty string.
fn clear_opt(slot: &mut *mut c_char) {
    // SAFETY: an option variable, as `set_opt`.
    unsafe { clear_string_option(slot) };
}

/// `xstrdup` of a literal, for the values `win_set_minimal_style` forces.
fn dup(s: &'static CStr) -> *mut c_char {
    // SAFETY: a NUL-terminated literal.
    unsafe { xstrdup(s.as_ptr()) }
}

/// `concat_str`: a fresh allocation holding `old` followed by `tail`.
fn concat(old: *const c_char, tail: &'static CStr) -> *mut c_char {
    // SAFETY: a non-null option value and a NUL-terminated literal.
    unsafe { concat_str(old, tail.as_ptr()) }
}

// ---------------------------------------------------------------------------
// window.rs and the API helpers, wrapped
//
// Each of these is still an `unsafe fn` over raw pointers, and all
// any of them needs is a live window, tab page or buffer -- which the argument
// types below already carry. So the obligation is discharged once here, and
// every call site in this file is ordinary code. They collapse to nothing when
// window.rs is itself rewritten.

fn last_nofloat(tp: Option<TabPage>) -> *mut win_T {
    // SAFETY: null, or a live tab page.
    unsafe { lastwin_nofloating(raw_tab(tp)) }
}
fn tabpage_of(win: Win) -> Option<TabPage> {
    // SAFETY: a live window; the answer is a live tab page or null.
    unsafe { TabPage::from_raw(win_find_tabpage(win.raw())) }
}

/// A fresh window appended after `after` (at the head when null), not hidden.
fn alloc_window(after: *mut win_T) -> Win {
    // SAFETY: `after` is null or a live window; `win_alloc` aborts on failure
    // rather than answering null.
    unsafe { Win::new(win_alloc(after, false)) }
}
fn init_window(win: Win) {
    // SAFETY: two live windows.
    unsafe { win_init(win.raw(), current_win().raw(), 0) };
}

/// Take `win` out of `tp`'s frame tree, handing its space to a neighbour. The
/// direction it answers is unused here, as it is upstream.
fn remove_from_frame(win: Win, tp: Option<TabPage>) {
    let mut dir: c_int = 0;
    // SAFETY: a live, non-floating window of `tp`; `dir` is a local.
    unsafe { winframe_remove(win.raw(), &raw mut dir, raw_tab(tp), ptr::null_mut()) };
}

/// `XFREE_CLEAR(wp->w_frame)`.
fn free_frame(win: &mut Win) {
    // SAFETY: the frame `remove_from_frame` has just detached.
    unsafe { xfree(win.w_frame.cast::<c_void>()) };
    win.w_frame = ptr::null_mut();
}
fn remove_window(win: Win, tp: Option<TabPage>) {
    // SAFETY: a live window of `tp`.
    unsafe { win_remove(win.raw(), raw_tab(tp)) };
}
fn append_window(after: *mut win_T, win: Win, tp: Option<TabPage>) {
    // SAFETY: `after` is null or a live window of `tp`; `win` is in no list.
    unsafe { win_append(after, win.raw(), raw_tab(tp)) };
}
fn free_window(win: Win, tp: Option<TabPage>) {
    // SAFETY: a live window, unlinked by `remove_window` just before.
    unsafe { win_free(win.raw(), raw_tab(tp)) };
}
fn update_last_status(morewin: bool) {
    last_status(morewin);
}
fn recompute_positions() {
    win_comp_pos();
}
fn remove_status_line(win: Win) {
    // SAFETY: a live window.
    unsafe { win_remove_status_line(win.raw(), false) };
}
fn set_inner_size(win: Win) {
    // SAFETY: a live window.
    unsafe { win_set_inner_size(win.raw(), true) };
}
fn merge_config(win: &mut Win, fconfig: WinConfig) {
    // SAFETY: a live window's own config.
    unsafe { merge_win_config(&raw mut win.w_config, fconfig) };
}

/// Whether `win` is still a window of the current tab page.
///
/// The pointer may already have been freed -- that is what the check is for --
/// so it stays raw until the answer is yes. Neither `win_valid` nor
/// `tabpage_win_valid` reads it; they walk the list comparing addresses.
fn valid_window(win: *mut win_T) -> Option<Win> {
    // SAFETY: `win` is compared, never read, and one found in the list is live.
    unsafe { win_valid(win).then(|| Win::new(win)) }
}
fn valid_in_tab(tp: TabPage, win: *mut win_T) -> Option<Win> {
    // SAFETY: a live tab page; `win` is compared, never read.
    unsafe { tabpage_win_valid(tp.raw(), win).then(|| Win::new(win)) }
}

/// Close a float, keeping its buffer and without forcing.
///
/// Fires `WinClosed`/`WinLeave` and can re-enter this module, so nothing may
/// be borrowed across it.
fn close_window(win: Win) -> c_int {
    // SAFETY: a live window.
    unsafe { win_close(win.raw(), false, false) }
}
fn enter_window(win: Win) {
    // SAFETY: a live window.
    unsafe { win_enter(win.raw(), false) };
}
fn set_window_buf(win: Win, buf: Buf, err: &mut Error) {
    // SAFETY: a live window and buffer, and the caller's error slot.
    unsafe { win_set_buf(win.raw(), buf.raw(), err) };
}
fn find_window(handle: Window, err: &mut Error) -> Option<Win> {
    // SAFETY: the caller's error slot; the answer is a live window or null.
    unsafe { NonNull::new(find_window_by_handle(handle, err)).map(|w| Win::new(w.as_ptr())) }
}
fn find_buffer(handle: Buffer, err: &mut Error) -> Option<Buf> {
    // SAFETY: the caller's error slot; the answer is a live buffer or null.
    unsafe { NonNull::new(find_buffer_by_handle(handle, err)).map(|b| Buf::new(b.as_ptr())) }
}
fn set_error(err: &mut Error, msg: &'static CStr) {
    // SAFETY: the caller's error slot, and a literal format whose one
    // conversion-free spelling takes no variadic arguments.
    unsafe { api_set_error(err, kErrorTypeException, msg.as_ptr()) };
}

/// Set `err` from a message that is not a literal, as upstream's `"%s"` does.
fn set_error_str(err: &mut Error, s: *const c_char) {
    // SAFETY: the caller's error slot; `%s` matches the one NUL-terminated
    // argument.
    unsafe { api_set_error(err, kErrorTypeException, c"%s".as_ptr(), s) };
}
fn clear_error(err: &mut Error) {
    // SAFETY: the caller's error slot.
    unsafe { api_clear_error(err) };
}
fn report_error(err: &Error) {
    // SAFETY: a set error's message is a string the API allocated.
    unsafe { emsg(err.msg) };
}
fn suppress_autocmds() {
    // SAFETY: `block_autocmds` touches only the global block counter.
    unsafe { block_autocmds() };
}
fn resume_autocmds() {
    // SAFETY: paired with `suppress_autocmds`.
    unsafe { unblock_autocmds() };
}
fn parse_winhl(win: Win) {
    // SAFETY: a live window; a null pattern means "use the window's option".
    unsafe { parse_winhl_opt(ptr::null(), win.raw()) };
}
fn adjust_for_grid(win: &mut Win, row: &mut c_int, col: &mut c_int) {
    // SAFETY: a live window's own grid view, and two locals.
    unsafe { grid_adjust(&raw mut win.w_grid, row, col) };
}
fn screen_pos_of(win: Win, pos: &mut pos_T) -> (c_int, c_int) {
    let (mut row, mut scol, mut ccol, mut ecol) = (0, 0, 0, 0);
    let (r, s, c, e) = (&raw mut row, &raw mut scol, &raw mut ccol, &raw mut ecol);
    // SAFETY: a live window, a position in the buffer it shows, and four
    // locals for the answers.
    unsafe { textpos2screenpos(win.raw(), pos, r, s, c, e, true) };
    (row, scol)
}
fn create_scratch_buffer(err: &mut Error) -> Buffer {
    // SAFETY: the caller's error slot.
    unsafe { nvim_create_buf(false, true, err) }
}
fn set_bufhidden_wipe(buf: Buf) {
    let s = String_0 {
        data: c"wipe".as_ptr().cast_mut(),
        size: c"wipe".count_bytes(),
    };
    let wipe = OptVal {
        type_0: kOptValTypeString,
        data: OptValData { string: s },
    };
    let (opt, from) = (kOptBufhidden, buf.raw().cast::<c_void>());
    // SAFETY: `buf` is the live buffer `kOptScopeBuf` names.
    unsafe { set_option_direct_for(opt, wipe, OPT_LOCAL as c_int, 0, kOptScopeBuf, from) };
}

// ---------------------------------------------------------------------------
// Creating a float

/// Creates a new float, or turns an existing window into one.
///
/// `win` is `None` to allocate a new window. `last` makes it the last window
/// in the list, which only the autocommand window asks for. `fconfig` must
/// already have been validated.
fn new_float(win: Option<Win>, last: bool, fconfig: WinConfig, err: &mut Error) -> Option<Win> {
    let mut win = match win {
        None => alloc_new_float(last, &fconfig, err)?,
        Some(win) => {
            debug_assert!(!last, "!last");
            debug_assert!(!win.w_floating, "!wp->w_floating");
            unfloat_to_float(win, err)?
        }
    };
    win.w_floating = true;
    win.w_status_height = if opt_is_set(win.w_onebuf_opt.wo_stl) && show_statusline() {
        STATUS_HEIGHT
    } else {
        0
    };
    win.w_winbar_height = 0;
    win.w_hsep_height = 0;
    win.w_vsep_width = 0;

    config_float(win, fconfig);
    win.redraw_later(UPD_VALID);
    Some(win)
}

/// The `wp == NULL` arm of `win_new_float`: allocate a window at the end of
/// the right tab page's list and make it float-shaped.
fn alloc_new_float(last: bool, fconfig: &WinConfig, err: &mut Error) -> Option<Win> {
    let mut tp_last = if last {
        lastwin.get()
    } else {
        last_nofloat(None)
    };
    if fconfig.window != 0 {
        debug_assert!(!last, "!last");
        let parent = find_window(fconfig.window, err)?;
        tp_last = last_nofloat(tabpage_of(parent)?.into_other());
    }
    let mut win = alloc_window(tp_last);
    init_window(win);
    // A one-line float has no room for a window bar, and a float never draws
    // the global 'statusline'.
    if !win.w_onebuf_opt.wo_wbr.is_null() && fconfig.height == 1 {
        // Upstream tests `!= empty_string_option` before freeing;
        // `free_string_option` makes the same test itself.
        clear_opt(&mut win.w_onebuf_opt.wo_wbr);
    }
    if !win.w_onebuf_opt.wo_stl.is_null() {
        clear_opt(&mut win.w_onebuf_opt.wo_stl);
    }
    Some(win)
}

/// The `wp != NULL` arm of `win_new_float`: take an existing window out of
/// the frame tree and re-append it as a float.
fn unfloat_to_float(win: Win, err: &mut Error) -> Option<Win> {
    let win_tp = tabpage_of(win);
    debug_assert!(win_tp.is_some(), "win_tp");
    let win_tp = win_tp?;
    let first_in_tab = if win_tp.is_current() {
        firstwin.get()
    } else {
        win_tp.tp_firstwin
    };
    if first_in_tab == win.raw() && last_nofloat(win_tp.into_other()) == win.raw() {
        set_error(err, c"Cannot change last window into float");
        return None;
    } else if !cmdwin_win.get().is_null() && !cmdwin_is_float() {
        // The command-line window can't become the only non-float. Check for
        // others.
        let mut other_nonfloat = false;
        for wp2 in windows_in_tab(win_tp) {
            if wp2.w_floating {
                break;
            }
            if wp2.raw() != win.raw() && wp2.raw() != cmdwin_win.get() {
                other_nonfloat = true;
                break;
            }
        }
        if !other_nonfloat {
            set_error_str(err, e_cmdwin.as_ptr());
            return None;
        }
    }
    let tp = win_tp.into_other();
    let mut win = win;
    remove_from_frame(win, tp);
    free_frame(&mut win);
    remove_window(win, tp);
    if win_tp.is_current() {
        update_last_status(false); // may need to remove last status line
        recompute_positions(); // recompute window positions
    }
    append_window(last_nofloat(tp), win, tp);
    Some(win)
}

/// Whether the command-line window is itself a float.
fn cmdwin_is_float() -> bool {
    // SAFETY: a non-null `cmdwin_win` is a live window.
    unsafe { Win::new(cmdwin_win.get()) }.w_floating
}

/// The two 'laststatus' values under which a float draws its own 'statusline'.
fn show_statusline() -> bool {
    p_ls.get() == 1 as OptInt || p_ls.get() == 2 as OptInt
}

pub unsafe fn win_new_float(
    wp: *mut win_T,
    last: bool,
    fconfig: WinConfig,
    err: *mut Error,
) -> *mut win_T {
    // SAFETY: the caller's promise -- a writable error slot, and `wp` null or
    // a live window.
    let (err, win) = unsafe { (&mut *err, (!wp.is_null()).then(|| Win::new(wp))) };
    new_float(win, last, fconfig, err).map_or(ptr::null_mut(), Win::raw)
}

// ---------------------------------------------------------------------------
// 'style' = "minimal"

/// Strip a float of everything that decorates a normal window.
fn set_minimal_style(win: Win) {
    let mut win = win;
    win.w_onebuf_opt.wo_nu = 0;
    win.w_onebuf_opt.wo_rnu = 0;
    win.w_onebuf_opt.wo_cul = 0;
    win.w_onebuf_opt.wo_cuc = 0;
    win.w_onebuf_opt.wo_spell = 0;
    win.w_onebuf_opt.wo_list = 0;

    // Hide EOB region: use " " fillchar and cleared highlighting
    if win.w_p_fcs_chars.eob != ' ' as schar_T {
        let old = win.w_onebuf_opt.wo_fcs;
        let new = if opt_is_set(old) {
            concat(old, c",eob: ")
        } else {
            dup(c"eob: ")
        };
        set_opt(&mut win.w_onebuf_opt.wo_fcs, new);
    }

    // TODO(bfredl): this could use a highlight namespace directly,
    // and avoid peculiarities around window options
    let old = win.w_onebuf_opt.wo_winhl;
    let new = if opt_is_set(old) {
        concat(old, c",EndOfBuffer:")
    } else {
        dup(c"EndOfBuffer:")
    };
    set_opt(&mut win.w_onebuf_opt.wo_winhl, new);
    parse_winhl(win);

    // signcolumn: use 'auto'
    let scl = win.w_onebuf_opt.wo_scl;
    if opt_head(scl) != b'a' as c_char || opt_len(scl) >= 8 {
        set_opt(&mut win.w_onebuf_opt.wo_scl, dup(c"auto"));
    }

    // foldcolumn: use '0'
    if opt_head(win.w_onebuf_opt.wo_fdc) != b'0' as c_char {
        set_opt(&mut win.w_onebuf_opt.wo_fdc, dup(c"0"));
    }

    // colorcolumn: cleared
    if opt_is_set(win.w_onebuf_opt.wo_cc) {
        set_opt(&mut win.w_onebuf_opt.wo_cc, dup(c""));
    }

    // statuscolumn: cleared
    if opt_is_set(win.w_onebuf_opt.wo_stc) {
        clear_opt(&mut win.w_onebuf_opt.wo_stc);
    }

    // statusline: cleared (for floating windows)
    if win.w_floating && opt_is_set(win.w_onebuf_opt.wo_stl) {
        clear_opt(&mut win.w_onebuf_opt.wo_stl);
        if win.w_status_height > 0 {
            config_float(win, win.w_config);
        }
    }
}

pub unsafe fn win_set_minimal_style(wp: *mut win_T) {
    // SAFETY: the caller's promise -- a live window.
    set_minimal_style(unsafe { Win::new(wp) });
}

// ---------------------------------------------------------------------------
// Config to geometry

/// Rows and columns the border adds around the text area.
fn border_height(win: Win) -> c_int {
    win.w_border_adj[0] + win.w_border_adj[2]
}
fn border_width(win: Win) -> c_int {
    win.w_border_adj[1] + win.w_border_adj[3]
}

pub unsafe fn win_border_height(wp: *mut win_T) -> c_int {
    // SAFETY: the caller's promise -- a live window.
    border_height(unsafe { Win::new(wp) })
}

pub unsafe fn win_border_width(wp: *mut win_T) -> c_int {
    // SAFETY: the caller's promise -- a live window.
    border_width(unsafe { Win::new(wp) })
}

/// Apply `fconfig` to `win`: its size, its screen position and everything
/// that has to be redrawn because they changed.
fn config_float(win: Win, mut fconfig: WinConfig) {
    let mut win = win;
    // Process statusline changes before applying new height from config
    let show_stl = opt_is_set(win.w_onebuf_opt.wo_stl) && show_statusline();
    if win.w_status_height != 0 && !show_stl {
        remove_status_line(win);
    } else if win.w_status_height == 0 && show_stl {
        win.w_status_height = STATUS_HEIGHT;
    }

    win.w_width = fconfig.width.max(1);
    win.w_height = fconfig.height.max(1);

    if fconfig.relative == kFloatRelativeCursor {
        let cur = current_win();
        fconfig.relative = kFloatRelativeWindow;
        fconfig.row += f64::from(cur.w_wrow);
        fconfig.col += f64::from(cur.w_wcol);
        fconfig.window = cur.handle as Window;
    } else if fconfig.relative == kFloatRelativeMouse {
        let mut pos = MousePos::current();
        if let Some(mouse_win) = find_win_inner(&mut pos) {
            fconfig.relative = kFloatRelativeWindow;
            fconfig.row += f64::from(pos.row);
            fconfig.col += f64::from(pos.col);
            fconfig.window = mouse_win.handle as Window;
        }
    }

    let change_external = fconfig.external != win.w_config.external;
    let mut change_border = fconfig.border != win.w_config.border
        || fconfig.border_hl_ids != win.w_config.border_hl_ids;

    merge_config(&mut win, fconfig);

    let has_border = win.w_floating && win.w_config.border;
    for i in 0..4 {
        let new_adj = c_int::from(has_border && win.w_config.border_chars[2 * i + 1][0] != 0);
        if new_adj != win.w_border_adj[i] {
            change_border = true;
            win.w_border_adj[i] = new_adj;
        }
    }

    if !ui_has(kUIMultigrid) {
        let above_ch = if win.w_config.zindex < kZIndexMessages {
            p_ch.get() as c_int
        } else {
            0
        };
        win.w_height = win.w_height.min(Rows.get() - border_height(win) - above_ch);
        win.w_width = win.w_width.min(Columns.get() - border_width(win));
    }

    set_inner_size(win);
    set_must_redraw(UPD_VALID);
    win.w_redr_status = win.w_status_height != 0;
    win.w_pos_changed = true;
    if change_external || change_border {
        win.w_hl_needs_update = 1;
        win.redraw_later(UPD_NOT_VALID);
    }

    // compute initial position
    if win.w_config.relative == kFloatRelativeWindow {
        let (row, col) = anchored_position(win);
        win.w_winrow = row;
        win.w_wincol = col;
    } else {
        win.w_winrow = fconfig.row as c_int;
        win.w_wincol = fconfig.col as c_int;
    }

    // changing border style while keeping border only requires redrawing border
    if fconfig.border {
        win.w_redr_border = true;
        win.redraw_later(UPD_VALID);
    }
}

/// Where a `relative='win'` float's config puts it on its parent's grid, with
/// `bufpos` resolved to a screen position when it is set. A parent that has
/// gone away leaves the config's own row and column, which is why the error
/// the lookup sets is thrown away.
fn anchored_position(win: Win) -> (c_int, c_int) {
    let mut row = win.w_config.row as c_int;
    let mut col = win.w_config.col as c_int;
    let mut dummy = NO_ERROR;
    if let Some(parent) = find_window(win.w_config.window, &mut dummy) {
        let mut parent = parent;
        row += parent.w_winrow;
        col += parent.w_wincol;
        adjust_for_grid(&mut parent, &mut row, &mut col);
        if win.w_config.bufpos.lnum >= 0 as linenr_T {
            // Widened: `bufpos={2147483647, ...}` reaches here and the C's
            // `lnum + 1` overflows before the clamp can catch it.
            let lnum =
                (win.w_config.bufpos.lnum as i64 + 1).min(parent.buffer().line_count() as i64);
            let mut pos = pos_T {
                lnum: lnum as linenr_T,
                col: win.w_config.bufpos.col,
                coladd: 0 as colnr_T,
            };
            let (trow, tcol) = screen_pos_of(parent, &mut pos);
            row += trow - 1;
            col += tcol - 1;
        }
    }
    clear_error(&mut dummy);
    (row, col)
}

pub unsafe fn win_config_float(wp: *mut win_T, fconfig: WinConfig) {
    // SAFETY: the caller's promise -- a live window.
    config_float(unsafe { Win::new(wp) }, fconfig);
}

// ---------------------------------------------------------------------------
// Closing floats

/// `qsort` comparator: sort floats by `zindex` DESCENDING, which is what makes
/// `:fclose` close the topmost one first. Comparing `b` against `a` is
/// upstream's `za == zb ? 0 : za < zb ? 1 : -1`.
unsafe extern "C" fn float_zindex_cmp(a: *const c_void, b: *const c_void) -> c_int {
    // SAFETY: `qsort` passes pointers into the array below, whose elements are
    // live windows.
    let z = |p: *const c_void| unsafe { (**p.cast::<*mut win_T>()).w_config.zindex };
    z(b).cmp(&z(a)) as c_int
}

pub unsafe fn win_float_remove(bang: bool, mut count: c_int) {
    // The whole list is collected before anything is closed: `win_close`
    // fires autocommands that can close further floats, which is what the
    // `win_valid` re-check below is for.
    let mut float_win_arr: Vec<*mut win_T> = floats().map(Win::raw).collect();
    if !float_win_arr.is_empty() {
        let items = float_win_arr.as_mut_ptr().cast::<c_void>();
        let (len, size) = (float_win_arr.len(), size_of::<*mut win_T>());
        // SAFETY: `len` elements of `*mut win_T` at `items`, and a comparator
        // that reads exactly that.
        unsafe { qsort(items, len, size, Some(float_zindex_cmp)) };
    }
    for &wp in &float_win_arr {
        if let Some(win) = valid_window(wp)
            && close_window(win) == FAIL
        {
            break;
        }
        if !bang {
            count -= 1;
            if count == 0 {
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Keeping floats in step with what they hang off, and finding one

pub unsafe fn win_check_anchored_floats(win: *mut win_T) {
    // SAFETY: the caller's promise -- a live window.
    let handle = unsafe { Win::new(win) }.handle;
    for mut wp in floats() {
        // float might be anchored to moved window
        if wp.w_config.relative == kFloatRelativeWindow && wp.w_config.window == handle {
            wp.w_pos_changed = true;
        }
    }
}

pub fn win_float_update_statusline() {
    for wp in floats() {
        let has_status = wp.w_status_height > 0;
        let should_show = opt_is_set(wp.w_onebuf_opt.wo_stl) && show_statusline();
        if should_show != has_status {
            config_float(wp, wp.w_config);
        }
    }
}

pub fn win_float_anchor_laststatus() {
    for mut win in windows_in_tab(current_tab()) {
        if win.w_config.relative == kFloatRelativeLaststatus {
            win.w_pos_changed = true;
        }
    }
}

pub fn win_reconfig_floats() {
    for wp in floats() {
        config_float(wp, wp.w_config);
    }
}

pub fn win_float_find_preview() -> *mut win_T {
    floats()
        .find(|wp| wp.w_float_is_info)
        .map_or(ptr::null_mut(), Win::raw)
}

/// Select an alternative window to `win` (assumed floating) in tabpage `tp`,
/// which is `win`'s original tabpage or NULL for the current one -- the window
/// to switch to when `win` is current and is then closed or moved away.
pub unsafe fn win_float_find_altwin(win: *const win_T, tp: *const tabpage_T) -> *mut win_T {
    // `win` itself is only ever compared below, never read, so it stays raw.
    // SAFETY: the caller's promise -- null, or a live tab page.
    let Some(tp) = (unsafe { TabPage::from_raw(tp.cast_mut()) }) else {
        let wp = valid_window(prevwin.get())
            .filter(|wp| wp.raw() != win.cast_mut())
            .filter(|wp| wp.w_config.focusable && !wp.w_config.hide);
        return wp.map_or(firstwin.get(), Win::raw);
    };

    debug_assert!(!tp.is_current(), "tp != curtab");
    let wp = valid_in_tab(tp, tp.tp_prevwin);
    // SAFETY: a live tab page's first window is live.
    let first = unsafe { Win::new(tp.tp_firstwin) };
    let wp = wp.unwrap_or(first);
    if wp.w_config.focusable && !wp.w_config.hide {
        wp.raw()
    } else {
        first.raw()
    }
}

// ---------------------------------------------------------------------------
// The preview float

/// Report and clear `err`, release a half-built float and let autocommands run
/// again: `win_float_create_preview`'s one failure path.
fn handle_error_and_cleanup(win: Option<Win>, err: &mut Error) -> *mut win_T {
    if err.type_0 != kErrorTypeNone {
        report_error(err);
        clear_error(err);
    }
    if let Some(win) = win {
        remove_window(win, None);
        free_window(win, None);
    }
    resume_autocmds();
    ptr::null_mut()
}

/// Create a floating preview window. `enter` makes it current; `new_buf`
/// gives it a scratch buffer of its own.
pub fn win_float_create_preview(enter: bool, new_buf: bool) -> *mut win_T {
    let mut config = WIN_CONFIG_INIT;
    let cur = current_win();
    config.col = f64::from(cur.w_wcol);
    config.row = f64::from(cur.w_wrow);
    config.relative = kFloatRelativeEditor;
    config.focusable = false;
    config.mouse = true;
    config.anchor = 0 as FloatAnchor; // NW
    config.noautocmd = true;
    config.hide = true;
    config.style = kWinStyleMinimal;
    let mut err = NO_ERROR;

    suppress_autocmds();
    let Some(mut win) = new_float(None, false, config, &mut err) else {
        return handle_error_and_cleanup(None, &mut err);
    };

    if new_buf {
        let b = create_scratch_buffer(&mut err);
        if b == 0 {
            return handle_error_and_cleanup(Some(win), &mut err);
        }
        let Some(mut buf) = find_buffer(b, &mut err) else {
            return handle_error_and_cleanup(Some(win), &mut err);
        };
        buf.b_p_bl = 0; // unlist
        set_bufhidden_wipe(buf);
        set_window_buf(win, buf, &mut err);
        if err.type_0 != kErrorTypeNone {
            return handle_error_and_cleanup(Some(win), &mut err);
        }
    }
    resume_autocmds();
    win.w_onebuf_opt.wo_diff = 0;
    win.w_float_is_info = true;
    win.w_onebuf_opt.wo_wrap = 1; // 'wrap' is default on
    win.w_onebuf_opt.wo_so = 0 as OptInt; // 'scrolloff' zero
    if enter {
        enter_window(win);
    }
    win.raw()
}
