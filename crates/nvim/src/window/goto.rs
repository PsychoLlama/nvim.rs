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

use core::ffi::{c_char, c_int};
use core::ptr;

use super::*;
use crate::autocmd::{
    EVENT_BUFENTER, EVENT_BUFLEAVE, EVENT_WINENTER, EVENT_WINLEAVE, EVENT_WINNEW,
};
use crate::buffer::{do_autochdir, maketitle};
use crate::drawscreen::{UPD_NOT_VALID, UPD_VALID, redrawWinline};
use crate::ex_eval::aborting;
use crate::ex_getln::text_or_buf_locked;
use crate::file_search::do_autocmd_dirchanged;
use crate::fileio::shorten_fnames;
use crate::main::{
    VIsual_active, curbuf, curwin, firstwin, globaldir, last_chdir_reason, msg_scrolled, p_acd,
    p_spk, p_wh, p_wiw, prevwin, redraw_tabline, restart_edit,
};
use crate::memory::xstrdup;
use crate::mouse::setmouse;
use crate::r#move::{changed_line_abv_curs, update_topline};
use crate::normal::reset_VIsual_and_resel;
use crate::option::buf_copy_options;
use crate::os::fs::{os_chdir, os_dirname};
use crate::path::pathcmp;
use crate::state::{MODE_CMDLINE, MODE_NORMAL, MODE_TERMINAL, get_real_state, virtual_active};
use crate::types::{
    CdScope, OK, OptInt, buf_T, kCdScopeGlobal, kCdScopeTabpage, kCdScopeWindow, tabpage_T,
};
use crate::undo::u_sync;
use crate::winlayer::{frames, tabs, windows_in_tab};

pub unsafe fn win_goto(wp: *mut win_T) {
    // SAFETY: the caller's promise -- a live window.
    goto_win(unsafe { Win::new(wp) });
}

/// Make `wp` the current window and redraw what the move uncovers.
pub(crate) fn goto_win(wp: Win) {
    let mut wp = wp;
    let owp = cur_win();
    // SAFETY: reads the editor's lock state.
    if unsafe { text_or_buf_locked() } {
        beep();
        return;
    }

    if wp.w_buffer != curbuf.get() {
        // careful: triggers ModeChanged autocommand
        reset_VIsual_and_resel();
    } else if VIsual_active.get() {
        wp.w_cursor = cur_win().w_cursor;
    }

    // autocommand may have made `wp` invalid
    let Some(wp) = valid_win(wp.raw()) else {
        return;
    };
    enter(wp, true);

    // Conceal cursor line in previous window, unconceal in current window.
    if let Some(owp) = valid_win(owp.raw()) {
        if owp.w_onebuf_opt.wo_cole > 0 as OptInt && msg_scrolled.get() == 0 {
            redraw_winline(owp);
        }
    }
    if cur_win().w_onebuf_opt.wo_cole > 0 as OptInt && msg_scrolled.get() == 0 {
        redraw_winline(cur_win());
    }
}

/// Redraw the line the cursor of `wp` is on.
fn redraw_winline(wp: Win) {
    let lnum = wp.w_cursor.lnum;
    // SAFETY: a live window and a line of its own buffer.
    unsafe { redrawWinline(wp.raw(), lnum) };
}

pub unsafe fn win_find_tabpage(win: *mut win_T) -> *mut tabpage_T {
    // SAFETY: the caller's promise -- a live window.
    raw_tab(find_tab_of(unsafe { Win::new(win) }))
}

/// The tab page `win` is on, `None` when it is on none.
fn find_tab_of(win: Win) -> Option<TabPage> {
    tabs().find(|tp| windows_in_tab(*tp).any(|wp| wp == win))
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

    /// Where the cursor of `wp` sits along the crossed axis.
    fn cursor(self, wp: Win) -> c_int {
        match self {
            Axis::Vertical => wp.w_wincol + wp.w_wcol,
            Axis::Horizontal => wp.w_winrow + wp.w_wrow,
        }
    }

    /// One past the last row or column `fr` covers along the crossed axis.
    fn frame_end(self, fr: Frame) -> c_int {
        let win = frame2window(fr);
        match self {
            Axis::Vertical => win.w_wincol + fr.fr_width,
            Axis::Horizontal => win.w_winrow + fr.fr_height,
        }
    }
}

pub unsafe fn win_vert_neighbor(
    tp: *mut tabpage_T,
    wp: *mut win_T,
    up: bool,
    count: c_int,
) -> *mut win_T {
    // SAFETY: the caller's promise -- a live tab page and a live window.
    let (tp, wp) = unsafe { (TabPage::new(tp), Win::new(wp)) };
    raw_win(neighbor(tp, wp, Axis::Vertical, up, count))
}

/// Move to the window above or below, `count` times.
pub(crate) fn goto_ver(up: bool, count: c_int) {
    if let Some(win) = neighbor(cur_tab(), cur_win(), Axis::Vertical, up, count) {
        goto_win(win);
    }
}

pub unsafe fn win_horz_neighbor(
    tp: *mut tabpage_T,
    wp: *mut win_T,
    left: bool,
    count: c_int,
) -> *mut win_T {
    // SAFETY: the caller's promise -- a live tab page and a live window.
    let (tp, wp) = unsafe { (TabPage::new(tp), Win::new(wp)) };
    raw_win(neighbor(tp, wp, Axis::Horizontal, left, count))
}

/// Move to the window left or right, `count` times.
pub(crate) fn goto_hor(left: bool, count: c_int) {
    if let Some(win) = neighbor(cur_tab(), cur_win(), Axis::Horizontal, left, count) {
        goto_win(win);
    }
}

/// The `count`th neighbour of `wp` along `axis`, `backwards` for up or left.
///
/// Answers `wp` itself when there is no such neighbour, and the previous
/// window (or the first) when `wp` floats, since a float is not in the tree.
fn neighbor(tp: TabPage, wp: Win, axis: Axis, backwards: bool, count: c_int) -> Option<Win> {
    if wp.w_floating {
        let prev = valid_win(prevwin.get()).filter(|p| !p.w_floating);
        // SAFETY: `firstwin` is set from startup to exit.
        return Some(prev.unwrap_or_else(|| unsafe { Win::new(firstwin.get()) }));
    }

    let mut foundfr = wp.frame();
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
            if fr == tp.topframe() {
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
                while fr.next().is_some() && axis.frame_end(fr) <= axis.cursor(wp) {
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

pub unsafe fn win_enter(wp: *mut win_T, undo_sync: bool) {
    // SAFETY: the caller's promise -- a live window.
    enter(unsafe { Win::new(wp) }, undo_sync);
}

/// Make `wp` the current window.
///
/// Autocommands may close it immediately, so the caller must re-check it with
/// [`valid_win`].
pub(crate) fn enter(wp: Win, undo_sync: bool) {
    let sync = if undo_sync { WEE_UNDO_SYNC as c_int } else { 0 };
    let enter = WEE_TRIGGER_ENTER_AUTOCMDS as c_int;
    enter_ext(wp, sync | enter | WEE_TRIGGER_LEAVE_AUTOCMDS as c_int);
}

/// Make `wp` the current window, `flags` saying which autocommands to fire.
///
/// `WEE_CURWIN_INVALID` means `curwin` has just been closed and must not be
/// read.
pub(crate) fn enter_ext(wp: Win, flags: c_int) {
    let mut wp = wp;
    let curwin_invalid = flags & WEE_CURWIN_INVALID as c_int != 0;
    if wp.is_current() && !curwin_invalid {
        return; // nothing to do
    }
    let mut other_buffer = false;
    if !curwin_invalid {
        leave_window(cur_win());
    }
    if !curwin_invalid && flags & WEE_TRIGGER_LEAVE_AUTOCMDS as c_int != 0 {
        // Be careful: if autocommands delete the window, return now.
        if wp.w_buffer != curbuf.get() {
            fire(EVENT_BUFLEAVE, cur_buf());
            other_buffer = true;
            if valid_win(wp.raw()).is_none() {
                return;
            }
        }
        fire(EVENT_WINLEAVE, cur_buf());
        if valid_win(wp.raw()).is_none() {
            return;
        }
        // autocmds may abort script processing
        if aborting() {
            return;
        }
    }

    // sync undo before leaving the current buffer
    if flags & WEE_UNDO_SYNC as c_int != 0 && curbuf.get() != wp.w_buffer {
        // SAFETY: reads the current buffer's undo state.
        unsafe { u_sync(false) };
    }
    // Might need to scroll the old window before switching, e.g. when the
    // cursor was moved.
    if split_keep_cursor() && !curwin_invalid {
        // SAFETY: a live window.
        unsafe { update_topline(curwin.get()) };
    }
    // may have to copy the buffer options when 'cpo' contains 'S'
    if wp.w_buffer != curbuf.get() {
        let (buf, flags) = (wp.w_buffer, BCO_ENTER as c_int | BCO_NOHELP as c_int);
        // SAFETY: a live window's buffer.
        unsafe { buf_copy_options(buf, flags) };
    }
    if !curwin_invalid {
        prevwin.set(curwin.get()); // remember for CTRL-W p
        cur_win().w_redr_status = true;
    }
    curwin.set(wp.raw());
    curbuf.set(wp.w_buffer);

    revalidate_cursor(cur_win());
    // SAFETY: a live window.
    if !unsafe { virtual_active(curwin.get()) } {
        cur_win().w_cursor.coladd = 0;
    }
    if split_keep_cursor() {
        // SAFETY: reads the current window, which was just set.
        unsafe { changed_line_abv_curs() }; // assume cursor position needs updating
    } else {
        // Make sure the cursor position is valid, either by moving the cursor
        // or by scrolling the text.
        let state = get_real_state();
        fix_cursor(state & (MODE_NORMAL | MODE_CMDLINE | MODE_TERMINAL) != 0);
    }
    fix_current_dir();
    enter_window(cur_win());

    // Careful: autocommands may close the window and make `wp` invalid.
    if flags & WEE_TRIGGER_NEW_AUTOCMDS as c_int != 0 {
        fire(EVENT_WINNEW, cur_buf());
    }
    if flags & WEE_TRIGGER_ENTER_AUTOCMDS as c_int != 0 {
        fire(EVENT_WINENTER, cur_buf());
        if other_buffer {
            fire(EVENT_BUFENTER, cur_buf());
        }
    }

    // SAFETY: reads the current buffer's name.
    unsafe { maketitle() };
    cur_win().w_redr_status = true;
    redraw_tabline.set(true);
    if restart_edit.get() != 0 {
        cur_win().redraw_later(UPD_VALID); // causes status line redraw
    }
    // Change background colour according to NormalNC, but only if actually
    // defined (otherwise no extra redraw).
    if cur_win().w_hl_attr_normal != cur_win().w_hl_attr_normalnc {
        cur_win().redraw_later(UPD_NOT_VALID);
    }
    if let Some(prev) = current_prevwin() {
        if prev.w_hl_attr_normal != prev.w_hl_attr_normalnc {
            prev.redraw_later(UPD_NOT_VALID);
        }
    }

    // set window height to desired minimal value
    let cur = cur_win();
    if (cur.w_height as OptInt) < p_wh.get() && cur.w_onebuf_opt.wo_wfh == 0 && !cur.w_floating {
        setheight_win(p_wh.get() as c_int, cur);
    } else if cur.w_height == 0 {
        setheight_win(1, cur);
    }
    // set window width to desired minimal value
    let cur = cur_win();
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
    // SAFETY: non-null, hence a live window.
    unsafe { Win::from_raw(prevwin.get()) }
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
    let new_dir = if cur_win().w_localdir.is_null() {
        cur_tab().tp_localdir
    } else {
        cur_win().w_localdir
    };
    let mut cwd = [0 as c_char; MAXPATHL as usize];
    // SAFETY: a buffer of exactly `MAXPATHL` bytes to fill in.
    if unsafe { os_dirname(cwd.as_mut_ptr(), MAXPATHL as size_t) } != OK {
        cwd[0] = NUL as c_char;
    }

    if !new_dir.is_null() {
        // Window or tab page has a local directory: save the current one as
        // global (unless that was done already) and change to the local one.
        if globaldir.get().is_null() && cwd[0] as c_int != NUL {
            // SAFETY: `cwd` is NUL-terminated and `xstrdup` copies it.
            globaldir.set(unsafe { xstrdup(cwd.as_ptr()) });
        }
        let scope = if cur_win().w_localdir.is_null() {
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
    // SAFETY: reads the buffer list.
    unsafe { shorten_fnames(true_0) };
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

pub unsafe fn buf_jump_open_win(buf: *mut buf_T) -> *mut win_T {
    // SAFETY: the caller's promise -- a live buffer.
    raw_win(jump_open_win(unsafe { Buf::new(buf) }))
}

/// Enter the first window of the current tab page showing `buf`, if there is
/// one.
pub(crate) fn jump_open_win(buf: Buf) -> Option<Win> {
    if cur_win().w_buffer == buf.raw() {
        enter(cur_win(), false);
        return Some(cur_win());
    }
    let wp = windows().find(|wp| wp.w_buffer == buf.raw())?;
    enter(wp, false);
    Some(wp)
}

pub unsafe fn buf_jump_open_tab(buf: *mut buf_T) -> *mut win_T {
    // SAFETY: the caller's promise -- a live buffer.
    raw_win(jump_open_tab(unsafe { Buf::new(buf) }))
}

/// [`jump_open_win`] over every tab page, the current one first.
pub(crate) fn jump_open_tab(buf: Buf) -> Option<Win> {
    // First try the current tab page.
    if let Some(wp) = jump_open_win(buf) {
        return Some(wp);
    }
    for tp in tabs() {
        // Skip the current tab page, which was checked above.
        if tp.is_current() {
            continue;
        }
        for wp in windows_in_tab(tp) {
            if wp.w_buffer == buf.raw() {
                goto_tab_win(tp, wp);
                // If the current window did not switch, something went wrong.
                return wp.is_current().then_some(wp);
            }
        }
    }
    // If we made it this far, we did not find the buffer.
    None
}
