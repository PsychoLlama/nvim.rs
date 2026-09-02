//! Marking things to be redrawn later.
//!
//! None of this draws: every function sets a flag that [`update_screen`] reads
//! on the next pass through the main loop. [`redraw_later`] is the primitive --
//! it raises one window's `w_redr_type` and the global `must_redraw` -- and the
//! rest name a scope: all windows ([`redraw_all_later`]), every window on one
//! buffer ([`redraw_buf_later`]), a line range ([`redraw_win_range_later`]), a
//! status line ([`status_redraw_buf`]).
//!
//! [`show_cursor_info_later`] is the one that decides *whether* anything
//! changed: it compares the cursor position, the Visual selection and the
//! recording state against what the status line was last drawn with.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::normal::{visual_active, visual_anchor, visual_mode};
use crate::types::StlSyntax;
use crate::winlayer::Win;

/// Mark the title and icon for redraw if either of them uses statusline format.
///
/// Answers whether either does.
pub unsafe fn redraw_custom_title_later() -> bool {
    let custom = (p_icon.get() != 0 && stl_syntax.get().has(StlSyntax::ICON))
        || (p_title.get() != 0 && stl_syntax.get().has(StlSyntax::TITLE));
    if custom {
        need_maketitle.set(true);
    }
    custom
}

/// Show the current cursor position in the ruler and everywhere else that
/// reports it.
///
/// Nothing is drawn here either: what this decides is whether anything the
/// status line, the window bar or the ruler shows has *changed* since they were
/// last drawn, and if so which of them to mark. `force` reports unconditionally.
pub unsafe fn show_cursor_info_later(force: bool) {
    // SAFETY: `curwin` is the editor's current window on the main thread.
    let mut wp = unsafe { Win::current() };
    let state = get_real_state();
    // "The cursor is on an empty line" is a status-line item of its own, and
    // in Insert mode it is deliberately always reported as false.
    let empty_line = State.get() & MODE_INSERT == 0
        && unsafe { *ml_get_buf(wp.w_buffer, wp.w_cursor.lnum) } == 0;

    validate_virtcol(unsafe { Win::new(wp.raw()) });

    let visual_moved = visual_active()
        && (visual_mode().raw() != wp.w_stl_visual_mode || visual_anchor() != wp.w_stl_visual_pos);
    if force
        || wp.w_cursor != wp.w_stl_cursor
        || wp.w_virtcol != wp.w_stl_virtcol
        || wp.w_topline != wp.w_stl_topline
        || unsafe { (*wp.w_buffer).b_ml.ml_line_count } != wp.w_stl_line_count
        || wp.w_topfill != wp.w_stl_topfill
        || empty_line != wp.w_stl_empty
        || reg_recording.get() != wp.w_stl_recording
        || state != wp.w_stl_state
        || visual_moved
    {
        if wp.w_status_height != 0 || global_stl_height() != 0 {
            wp.w_redr_status = true;
        } else {
            redraw_cmdline.set(true);
        }
        // A window bar can show the same items, and it is never on the
        // command line, so it needs the status-line treatment either way.
        if unsafe { *p_wbr.get() } != 0 || unsafe { *wp.w_onebuf_opt.wo_wbr } != 0 {
            wp.w_redr_status = true;
        }
        unsafe { redraw_custom_title_later() };
    }

    wp.w_stl_cursor = wp.w_cursor;
    wp.w_stl_virtcol = wp.w_virtcol;
    wp.w_stl_empty = empty_line;
    wp.w_stl_topline = wp.w_topline;
    unsafe { wp.w_stl_line_count = (*wp.w_buffer).b_ml.ml_line_count };
    wp.w_stl_topfill = wp.w_topfill;
    wp.w_stl_recording = reg_recording.get();
    wp.w_stl_state = state;
    // Upstream leaves the remembered Visual position alone when Visual mode
    // is not active, so that leaving and re-entering it on the same
    // selection does not count as a change. Reproduced.
    if visual_active() {
        wp.w_stl_visual_mode = visual_mode().raw();
        wp.w_stl_visual_pos = visual_anchor();
    }
}

/// Redraw window `wp` later, with `w_redr_type` at least `redr_type`.
///
/// `must_redraw` is the maximum over all windows, so it only ever rises here;
/// [`update_screen`] resets it.
pub unsafe fn redraw_later(wp: *mut win_T, redr_type: c_int) {
    debug_assert!(!wp.is_null() || exiting.get(), "wp != NULL || exiting");
    if exiting.get() || redraw_not_allowed.get() {
        return;
    }
    // SAFETY: a live window -- the guard above has ruled out the one caller
    // that may pass a null, which is the editor on its way out.
    let mut wp = unsafe { Win::new(wp) };
    if wp.w_redr_type < redr_type {
        wp.w_redr_type = redr_type;
        if redr_type >= UPD_NOT_VALID {
            wp.w_lines_valid = 0;
        }
        set_must_redraw_unchecked(redr_type);
    }
}

/// Mark every window of the current tab page for redraw.
pub unsafe fn redraw_all_later(redr_type: c_int) {
    // SAFETY: walking the current tab page's window list on the main thread.
    for wp in winlayer::windows() {
        unsafe { redraw_later(wp.raw(), redr_type) };
    }
    // Needed as well when switching tab pages: the windows marked above are
    // not the ones that will be drawn.
    set_must_redraw(redr_type);
}

/// Raise `must_redraw` to `redr_type`, unless redrawing is currently forbidden.
pub fn set_must_redraw(redr_type: c_int) {
    if !redraw_not_allowed.get() {
        set_must_redraw_unchecked(redr_type);
    }
}

/// [`set_must_redraw`] without the `redraw_not_allowed` test, for callers that
/// have already made it.
fn set_must_redraw_unchecked(redr_type: c_int) {
    must_redraw.set(must_redraw.get().max(redr_type));
}

/// Drop every window's cached attribute state; used when the highlight tables
/// are rebuilt.
pub unsafe fn screen_invalidate_highlights() {
    // SAFETY: walking the current tab page's window list on the main thread.
    for mut wp in winlayer::windows() {
        unsafe { redraw_later(wp.raw(), UPD_NOT_VALID) };
        wp.w_grid_alloc.valid = false;
    }
}

/// Mark every window showing the current buffer.
///
/// Safe: the only promise is that the editor exists, which `curbuf` carries
/// from startup to exit.
pub fn redraw_curbuf_later(redr_type: c_int) {
    // SAFETY: `curbuf` is the editor's current buffer.
    unsafe { redraw_buf_later(curbuf.get(), redr_type) }
}

/// Mark every window showing `buf`.
pub unsafe fn redraw_buf_later(buf: *mut buf_T, redr_type: c_int) {
    // SAFETY: walking the current tab page's window list on the main thread.
    for wp in winlayer::windows() {
        if wp.w_buffer == buf {
            unsafe { redraw_later(wp.raw(), redr_type) };
        }
    }
}

/// Mark line `line` of `buf` in every window showing it.
///
/// `force` also marks a line *past* the end of the buffer, which is how a
/// deletion gets the rows it used to occupy redrawn.
pub unsafe fn redraw_buf_line_later(buf: *mut buf_T, line: linenr_T, force: bool) {
    // SAFETY: walking the current tab page's window list on the main thread.
    for mut wp in winlayer::windows() {
        if wp.w_buffer == buf {
            unsafe { redraw_win_line(wp.raw(), line.min((*buf).b_ml.ml_line_count)) };
            if force && line > unsafe { (*buf).b_ml.ml_line_count } {
                wp.w_redraw_bot = line;
            }
        }
    }
}

/// Widen window `wp`'s pending redraw range to cover lines `first..=last`.
///
/// Nothing is marked when the range is entirely outside the window.
pub unsafe fn redraw_win_range_later(wp: *mut win_T, first: linenr_T, last: linenr_T) {
    // SAFETY: a live window on the main thread.
    let mut win = unsafe { Win::new(wp) };
    if last >= win.w_topline && first < win.w_botline {
        if win.w_redraw_top == 0 || win.w_redraw_top > first {
            win.w_redraw_top = first;
        }
        if win.w_redraw_bot == 0 || win.w_redraw_bot < last {
            win.w_redraw_bot = last;
        }
        unsafe { redraw_later(wp, UPD_VALID) };
    }
}

/// Mark one line of window `wp`.
///
/// Inserting or deleting lines invalidates the range this widens, so a caller
/// that does either has to mark the whole window instead.
pub unsafe fn redraw_win_line(wp: *mut win_T, lnum: linenr_T) {
    // SAFETY: a live window on the main thread.
    unsafe { redraw_win_range_later(wp, lnum, lnum) }
}

/// Mark lines `first..=last` of `buf` in every window showing it.
pub unsafe fn redraw_buf_range_later(buf: *mut buf_T, first: linenr_T, last: linenr_T) {
    // SAFETY: walking the current tab page's window list on the main thread.
    for wp in winlayer::windows() {
        if wp.w_buffer == buf {
            unsafe { redraw_win_range_later(wp.raw(), first, last) };
        }
    }
}

/// Mark the status lines and window bars of every window showing `buf`.
pub unsafe fn redraw_buf_status_later(buf: *mut buf_T) {
    // SAFETY: walking the current tab page's window list on the main thread.
    for mut wp in winlayer::windows() {
        if wp.w_buffer == buf
            && (wp.w_status_height != 0
                || (wp.raw() == curwin.get() && global_stl_height() != 0)
                || wp.w_winbar_height != 0)
        {
            wp.w_redr_status = true;
            set_must_redraw(UPD_VALID);
        }
    }
}

/// Mark every status line and window bar; used after the first `:cd`.
pub unsafe fn status_redraw_all() {
    // SAFETY: walking the current tab page's window list on the main thread.
    let is_stl_global = global_stl_height() != 0;
    for mut wp in winlayer::windows() {
        if (!is_stl_global && wp.w_status_height != 0)
            || wp.raw() == curwin.get()
            || wp.w_winbar_height != 0
        {
            wp.w_redr_status = true;
            unsafe { redraw_later(wp.raw(), UPD_VALID) };
        }
    }
}

/// Mark the status lines and window bars of the current buffer.
pub unsafe fn status_redraw_curbuf() {
    // SAFETY: `curbuf` is the editor's current buffer.
    unsafe { status_redraw_buf(curbuf.get()) }
}

/// Mark the status lines and window bars of `buf`.
pub unsafe fn status_redraw_buf(buf: *mut buf_T) {
    // SAFETY: walking the current tab page's window list on the main thread.
    let is_stl_global = global_stl_height() != 0;
    for mut wp in winlayer::windows() {
        if wp.w_buffer == buf
            && ((!is_stl_global && wp.w_status_height != 0)
                || (is_stl_global && wp.raw() == curwin.get())
                || wp.w_winbar_height != 0)
        {
            wp.w_redr_status = true;
            unsafe { redraw_later(wp.raw(), UPD_VALID) };
        }
    }
    // With no status line at all the ruler lives on the command line, so it
    // has to be marked separately -- but only if the loop above did not
    // already mark the current window.
    let wp = unsafe { Win::current() };
    if p_ru.get() != 0 && wp.w_status_height == 0 && !wp.w_redr_status {
        redraw_cmdline.set(true);
        unsafe { redraw_later(wp.raw(), UPD_VALID) };
    }
}

/// Draw every status line and window bar that is marked, plus the tab line and
/// the title.
pub unsafe fn redraw_statuslines() {
    // SAFETY: walking the current tab page's window list on the main thread.
    for wp in winlayer::windows() {
        if wp.w_redr_status {
            unsafe { win_check_ns_hl(wp.raw()) };
            unsafe { win_redr_winbar(wp.raw()) };
            unsafe { win_redr_status(wp.raw()) };
        }
    }
    unsafe { win_check_ns_hl(::core::ptr::null_mut()) };

    if redraw_tabline.get() {
        unsafe { draw_tabline() };
    }
    if need_maketitle.get() {
        unsafe { maketitle() };
    }
}

/// Mark the status lines at the bottom of frame `frp`.
///
/// One per column of a row frame; the last one of a column frame.
pub unsafe fn win_redraw_last_status(frp: *const frame_T) {
    // SAFETY: a live frame of the window layout tree, walked on the main thread.
    match unsafe { (*frp).fr_layout } as c_int {
        FR_LEAF => unsafe { (*(*frp).fr_win).w_redr_status = true },
        FR_ROW => {
            let mut child = unsafe { (*frp).fr_child };
            while !child.is_null() {
                unsafe { win_redraw_last_status(child) };
                child = unsafe { (*child).fr_next };
            }
        }
        layout => {
            debug_assert!(layout == FR_COL, "frp->fr_layout == FR_COL");
            let mut last = unsafe { (*frp).fr_child };
            while !unsafe { (*last).fr_next }.is_null() {
                last = unsafe { (*last).fr_next };
            }
            unsafe { win_redraw_last_status(last) };
        }
    }
}
