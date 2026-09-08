//! Entering a window -- `win_goto()`, `win_enter()` and the directional
//! moves.
//!
//! [`enter_ext`] is the one that actually changes `curwin`: it fires
//! `WinLeave`/`BufLeave` and `WinEnter`/`BufEnter`, syncs undo, updates the
//! window-local directory ([`win_fix_current_dir`]) and revalidates the
//! cursor -- and every one of those may close the window it was entering.
//! [`win_vert_neighbor`] and [`win_horz_neighbor`] answer which window lies
//! in a given direction, and the `buf_jump_open_*` pair finds a window
//! already showing a buffer.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::types::AutoEvent;
use crate::winlayer::WinId;
use crate::winlayer::prev_window;
use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::buffer::{do_autochdir, maketitle};
use crate::drawscreen::state::redraw_tabline;
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID, redraw_win_line};
use crate::ex_eval::aborting;
use crate::ex_getln::text_or_buf_locked;
use crate::file_search::do_autocmd_dirchanged;
use crate::fileio::shorten_fnames;
use crate::memory::xstrdup;
use crate::message::state::msg_scrolled;
use crate::mouse::setmouse;
use crate::r#move::{changed_line_abv_curs, update_topline};
use crate::normal::{reset_visual_and_resel, visual_active};
use crate::option::buf_copy_options;
use crate::option::vars::{p_acd, p_spk, p_wh, p_wiw};
use crate::os::fs::{os_chdir, os_dirname};
use crate::os::state::{globaldir, last_chdir_reason};
use crate::path::pathcmp;
use crate::state::mode::restart_edit;
use crate::state::{MODE_CMDLINE, MODE_NORMAL, MODE_TERMINAL, get_real_state, virtual_active};
use crate::types::{
    CdScope, MAXPATHL, NUL, OptInt, kCdScopeGlobal, kCdScopeTabpage, kCdScopeWindow,
};
use crate::undo::u_sync;
use crate::winlayer::graph::prevwin;
use crate::winlayer::{first_window, frames, tabs, windows_in_tab};

pub fn win_goto(window: Win) {
    goto_win(window);
}

/// Make `window` the current window and redraw what the move uncovers.
pub(crate) fn goto_win(window: Win) {
    let mut window = window;
    let owp = Win::current();
    if text_or_buf_locked() {
        beep();
        return;
    }

    if window.w_buffer != Buf::current_raw() {
        // careful: triggers ModeChanged autocommand
        reset_visual_and_resel();
    } else if visual_active() {
        window.w_cursor = Win::current().w_cursor;
    }

    // autocommand may have made `window` invalid
    let Some(window) = valid_win(window.id()) else {
        return;
    };
    enter(window, true);

    // Conceal cursor line in previous window, unconceal in current window.
    if let Some(owp) = valid_win(owp.id())
        && owp.w_onebuf_opt.wo_cole > 0 as OptInt
        && msg_scrolled.get() == 0
    {
        redraw_winline(owp);
    }
    if Win::current().w_onebuf_opt.wo_cole > 0 as OptInt && msg_scrolled.get() == 0 {
        redraw_winline(Win::current());
    }
}

/// Redraw the line the cursor of `window` is on.
fn redraw_winline(window: Win) {
    let lnum = window.w_cursor.lnum;
    redraw_win_line(window, lnum);
}

/// The tab page `win` is on, or null.
///
/// `win` stays a raw pointer and is **only compared**, never read: this is one
/// of the questions the editor asks about a window an autocommand may already
/// have closed. `nvim_open_win` calls it right after `win_set_buf`, whose
/// `BufEnter`/`BufLeave` handlers can close the very window being asked about.
pub(crate) fn win_find_tabpage(win: WinId) -> Option<TabPage> {
    tabs().find(|tp| windows_in_tab(*tp).any(|wp| wp.id() == win))
}

/// The axis a directional move travels along.
///
/// The vertical and horizontal searches are exact duals: each looks for a
/// parent frame laid out along its own axis, and descends through a frame laid
/// out along the other one by following the cursor's position across it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Axis {
    /// Up and down, through `FR_COL` parents.
    Vertical,
    /// Left and right, through `FR_ROW` parents.
    Horizontal,
}

impl Axis {
    /// The parent layout a neighbour along this axis is found in.
    fn own(self) -> c_int {
        match self {
            Axis::Vertical => FR_COL,
            Axis::Horizontal => FR_ROW,
        }
    }

    /// The layout of a frame that must be crossed to reach the cursor.
    fn cross(self) -> c_int {
        match self {
            Axis::Vertical => FR_ROW,
            Axis::Horizontal => FR_COL,
        }
    }

    /// Where the cursor of `window` sits along the crossed axis.
    fn cursor(self, window: Win) -> c_int {
        match self {
            Axis::Vertical => window.w_wincol + window.w_wcol,
            Axis::Horizontal => window.w_winrow + window.w_wrow,
        }
    }

    /// One past the last row or column `fr` covers along the crossed axis.
    fn frame_end(self, fr: FrameRef) -> c_int {
        let win = frame2window(fr);
        match self {
            Axis::Vertical => win.w_wincol + fr.fr_width,
            Axis::Horizontal => win.w_winrow + fr.fr_height,
        }
    }
}

pub fn win_vert_neighbor(tabpage: TabPage, window: Win, up: bool, count: c_int) -> Option<Win> {
    let (tp, wp) = (tabpage, window);
    neighbor(tp, wp, Axis::Vertical, up, count)
}

/// Move to the window above or below, `count` times.
pub(crate) fn goto_ver(up: bool, count: c_int) {
    if let Some(win) = neighbor(
        TabPage::current(),
        Win::current(),
        Axis::Vertical,
        up,
        count,
    ) {
        goto_win(win);
    }
}

pub fn win_horz_neighbor(tabpage: TabPage, window: Win, left: bool, count: c_int) -> Option<Win> {
    let (tp, wp) = (tabpage, window);
    neighbor(tp, wp, Axis::Horizontal, left, count)
}

/// Move to the window left or right, `count` times.
pub(crate) fn goto_hor(left: bool, count: c_int) {
    if let Some(win) = neighbor(
        TabPage::current(),
        Win::current(),
        Axis::Horizontal,
        left,
        count,
    ) {
        goto_win(win);
    }
}

/// The `count`th neighbour of `window` along `axis`, `backwards` for up or left.
///
/// Answers `window` itself when there is no such neighbour, and the previous
/// window (or the first) when `window` floats, since a float is not in the tree.
fn neighbor(
    tabpage: TabPage,
    window: Win,
    axis: Axis,
    backwards: bool,
    count: c_int,
) -> Option<Win> {
    if window.w_floating {
        let prev = prev_window().filter(|p| !p.w_floating);
        return Some(prev.or_else(first_window).expect("the editor has a window"));
    }

    let mut foundfr = window.frame();
    let mut count = count;
    'end: loop {
        let more = count != 0;
        count -= 1;
        if !more {
            break;
        }
        // First go upwards in the tree of frames until we find a neighbour
        // along this axis.
        let mut fr = foundfr;
        let mut nfr = loop {
            if fr == tabpage.topframe() {
                break 'end;
            }
            let next = if backwards { fr.prev() } else { fr.next() };
            let parent = fr.parent().expect("not the top frame");
            match next {
                Some(nfr) if parent.fr_layout as c_int == axis.own() => break nfr,
                _ => fr = parent,
            }
        };

        // Now go downwards to find the frame at the far end of it.
        loop {
            if nfr.fr_layout as c_int == FR_LEAF {
                foundfr = nfr;
                break;
            }
            let mut fr = nfr.child().expect("a frame that is not a leaf has a child");
            if nfr.fr_layout as c_int == axis.cross() {
                // Find the frame the cursor is at, across the other axis.
                while fr.next().is_some() && axis.frame_end(fr) <= axis.cursor(window) {
                    fr = fr.next().expect("just tested");
                }
            }
            if nfr.fr_layout as c_int == axis.own() && backwards {
                fr = frames(Some(fr)).last().expect("at least one");
            }
            nfr = fr;
        }
    }
    foundfr.win()
}

pub fn win_enter(window: Win, undo_sync: bool) {
    enter(window, undo_sync);
}

/// Make `window` the current window.
///
/// Autocommands may close it immediately, so the caller must re-check it with
/// [`valid_win`].
pub(crate) fn enter(window: Win, undo_sync: bool) {
    let sync = if undo_sync { WEE_UNDO_SYNC as c_int } else { 0 };
    let enter = WEE_TRIGGER_ENTER_AUTOCMDS as c_int;
    enter_ext(window, sync | enter | WEE_TRIGGER_LEAVE_AUTOCMDS as c_int);
}

/// Make `window` the current window, `flags` saying which autocommands to fire.
///
/// `WEE_CURWIN_INVALID` means `curwin` has just been closed and must not be
/// read.
pub(crate) fn enter_ext(window: Win, flags: c_int) {
    let curwin_invalid = flags & WEE_CURWIN_INVALID as c_int != 0;
    if window.is_current() && !curwin_invalid {
        return; // nothing to do
    }
    let mut other_buffer = false;
    if !curwin_invalid {
        leave_window(Win::current());
    }
    if !curwin_invalid && flags & WEE_TRIGGER_LEAVE_AUTOCMDS as c_int != 0 {
        // Be careful: if autocommands delete the window, return now.
        if window.w_buffer != Buf::current_raw() {
            fire(AutoEvent::BufLeave, Buf::current());
            other_buffer = true;
            if valid_win(window.id()).is_none() {
                return;
            }
        }
        fire(AutoEvent::WinLeave, Buf::current());
        if valid_win(window.id()).is_none() {
            return;
        }
        // autocmds may abort script processing
        if aborting() {
            return;
        }
    }

    // sync undo before leaving the current buffer
    if flags & WEE_UNDO_SYNC as c_int != 0 && Buf::current_raw() != window.w_buffer {
        // SAFETY: reads the current buffer's undo state.
        u_sync(false);
    }
    // Might need to scroll the old window before switching, e.g. when the
    // cursor was moved.
    if split_keep_cursor() && !curwin_invalid {
        update_topline(Win::current());
    }
    // may have to copy the buffer options when 'cpo' contains 'S'
    if window.w_buffer != Buf::current_raw() {
        let flags = BCO_ENTER as c_int | BCO_NOHELP as c_int;
        // SAFETY: a live window's buffer.
        unsafe { buf_copy_options(window.buffer(), flags) };
    }
    if !curwin_invalid {
        prevwin.set(Win::current_or_none().map(Win::id)); // remember for CTRL-W p
        Win::current().w_redr_status = true;
    }
    window.make_current();
    window.buffer().make_current();

    revalidate_cursor(Win::current());
    // SAFETY: a live window.
    if !virtual_active(Win::current()) {
        Win::current().w_cursor.coladd = 0;
    }
    if split_keep_cursor() {
        changed_line_abv_curs(); // assume cursor position needs updating
    } else {
        // Make sure the cursor position is valid, either by moving the cursor
        // or by scrolling the text.
        let state = get_real_state();
        fix_cursor(state & (MODE_NORMAL | MODE_CMDLINE | MODE_TERMINAL) != 0);
    }
    fix_current_dir();
    enter_window(Win::current());

    // Careful: autocommands may close the window and make `window` invalid.
    if flags & WEE_TRIGGER_NEW_AUTOCMDS as c_int != 0 {
        fire(AutoEvent::WinNew, Buf::current());
    }
    if flags & WEE_TRIGGER_ENTER_AUTOCMDS as c_int != 0 {
        fire(AutoEvent::WinEnter, Buf::current());
        if other_buffer {
            fire(AutoEvent::BufEnter, Buf::current());
        }
    }

    maketitle();
    Win::current().w_redr_status = true;
    redraw_tabline.set(true);
    if restart_edit.get() != 0 {
        Win::current().redraw_later(UPD_VALID); // causes status line redraw
    }
    // Change background colour according to NormalNC, but only if actually
    // defined (otherwise no extra redraw).
    if Win::current().w_hl_attr_normal != Win::current().w_hl_attr_normalnc {
        Win::current().redraw_later(UPD_NOT_VALID);
    }
    if let Some(prev) = current_prevwin()
        && prev.w_hl_attr_normal != prev.w_hl_attr_normalnc
    {
        prev.redraw_later(UPD_NOT_VALID);
    }

    // set window height to desired minimal value
    let cur = Win::current();
    if (cur.w_height as OptInt) < p_wh.get() && cur.w_onebuf_opt.wo_wfh == 0 && !cur.w_floating {
        setheight_win(p_wh.get() as c_int, cur);
    } else if cur.w_height == 0 {
        setheight_win(1, cur);
    }
    // set window width to desired minimal value
    let cur = Win::current();
    if (cur.w_width as OptInt) < p_wiw.get() && cur.w_onebuf_opt.wo_wfw == 0 && !cur.w_floating {
        setwidth_win(p_wiw.get() as c_int, cur);
    }

    setmouse(); // in case jumped to/from help buffer
    // Change directories when the 'acd' option is set.
    do_autochdir();
}

/// Whether `'splitkeep'` is `"cursor"`, which keeps the cursor line put and
/// scrolls the text instead.
fn split_keep_cursor() -> bool {
    // SAFETY: `'splitkeep'` is a NUL-terminated option string.
    unsafe { *p_spk.get() as c_int == 'c' as c_int }
}

/// The window CTRL-W p goes back to, `None` when there is none.
fn current_prevwin() -> Option<Win> {
    prev_window()
}

pub fn win_fix_current_dir() {
    fix_current_dir();
}

/// Change directory after another window became the current one.
///
/// The new directory is the window's own, or its tab page's, or -- when it has
/// neither -- the global one saved when the first local directory was entered.
fn fix_current_dir() {
    // The new directory is the window's own, the tab page's, or none.
    let new_dir = if Win::current().w_localdir.is_null() {
        TabPage::current().tp_localdir
    } else {
        Win::current().w_localdir
    };
    let mut cwd = [0 as c_char; MAXPATHL as usize];
    // SAFETY: a buffer of exactly `MAXPATHL` bytes to fill in.
    if unsafe { os_dirname(cwd.as_mut_ptr(), MAXPATHL as size_t) }.is_err() {
        cwd[0] = NUL as c_char;
    }

    if !new_dir.is_null() {
        // Window or tab page has a local directory: save the current one as
        // global (unless that was done already) and change to the local one.
        if globaldir.get().is_null() && cwd[0] as c_int != NUL {
            // SAFETY: `cwd` is NUL-terminated and `xstrdup` copies it.
            globaldir.set(unsafe { xstrdup(cwd.as_ptr()) });
        }
        let scope = if Win::current().w_localdir.is_null() {
            kCdScopeTabpage
        } else {
            kCdScopeWindow
        };
        chdir_to(new_dir, scope as CdScope, cwd.as_ptr());
    } else if !globaldir.get().is_null() {
        // The window has no local directory and we are not in the global one:
        // change back to it.
        chdir_to(globaldir.get(), kCdScopeGlobal, cwd.as_ptr());
        free(globaldir.get());
        globaldir.set(ptr::null_mut::<c_char>());
    } else {
        return;
    }
    last_chdir_reason.set(ptr::null_mut::<c_char>());
    shorten_fnames(1);
}

/// `chdir()` to `dir`, firing `DirChangedPre` and `DirChanged` around it when
/// it actually differs from `cwd` and `'autochdir'` is off.
fn chdir_to(dir: *mut c_char, scope: CdScope, cwd: *const c_char) {
    // SAFETY: two NUL-terminated paths; -1 means "compare all of both".
    let differs = unsafe { pathcmp(dir, cwd, -1) } != 0;
    let announce = p_acd.get() == 0 && differs;
    if announce {
        dirchanged(dir, scope, true);
    }
    // SAFETY: a NUL-terminated path.
    if unsafe { os_chdir(dir) } == 0 && announce {
        dirchanged(dir, scope, false);
    }
}

/// `do_autocmd_dirchanged()`: fire `DirChanged`, or `DirChangedPre` with
/// `pre`.
fn dirchanged(dir: *mut c_char, scope: CdScope, pre: bool) {
    // SAFETY: a NUL-terminated path.
    unsafe { do_autocmd_dirchanged(dir, scope, kCdCauseWindow, pre) };
}

pub fn buf_jump_open_win(buffer: Buf) -> Option<Win> {
    jump_open_win(buffer)
}

/// Enter the first window of the current tab page showing `buffer`, if there is
/// one.
pub(crate) fn jump_open_win(buffer: Buf) -> Option<Win> {
    if Win::current().w_buffer == buffer.raw() {
        enter(Win::current(), false);
        return Some(Win::current());
    }
    let wp = windows().find(|wp| wp.w_buffer == buffer.raw())?;
    enter(wp, false);
    Some(wp)
}

pub fn buf_jump_open_tab(buffer: Buf) -> Option<Win> {
    jump_open_tab(buffer)
}

/// [`jump_open_win`] over every tab page, the current one first.
pub(crate) fn jump_open_tab(buffer: Buf) -> Option<Win> {
    // First try the current tab page.
    if let Some(wp) = jump_open_win(buffer) {
        return Some(wp);
    }
    for tp in tabs() {
        // Skip the current tab page, which was checked above.
        if tp.is_current() {
            continue;
        }
        for wp in windows_in_tab(tp) {
            if wp.w_buffer == buffer.raw() {
                goto_tab_win(tp, wp);
                // If the current window did not switch, something went wrong.
                return wp.is_current().then_some(wp);
            }
        }
    }
    // If we made it this far, we did not find the buffer.
    None
}
