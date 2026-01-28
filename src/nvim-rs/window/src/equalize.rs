//! Window equalization functions.
//!
//! This module provides Rust helper functions for window equalization operations
//! from `src/nvim/window.c`, including win_equal, win_equal_rec, and related
//! size distribution functions.

// Window dimensions may need truncation when converting between types.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

use crate::{Frame, WinHandle, FR_COL, FR_LEAF, FR_ROW};

/// Status height constant (same as in C).
const STATUS_HEIGHT: c_int = 1;

/// UPD_NOT_VALID constant for redraw.
const UPD_NOT_VALID: c_int = 40;

/// NOWIN sentinel value (same as C: (win_T *)-1).
const NOWIN: WinHandle = unsafe { WinHandle::from_ptr((-1isize) as *mut std::ffi::c_void) };

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    /// Get curwin.
    fn nvim_get_curwin() -> WinHandle;

    /// Get topframe.
    fn nvim_get_topframe() -> *mut Frame;

    /// Get w_frame from window.
    fn nvim_win_get_frame(wp: WinHandle) -> *mut Frame;

    /// Get w_floating from window.
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;

    /// Get w_winbar_height from window.
    fn nvim_win_get_winbar_height(wp: WinHandle) -> c_int;

    /// Get global p_wmh (winminheight).
    fn nvim_get_p_wmh() -> i64;

    /// Get global p_wmw (winminwidth).
    fn nvim_get_p_wmw() -> i64;

    /// Get global p_wh (winheight).
    fn nvim_get_p_wh() -> i64;

    /// Get global p_wiw (winwidth).
    fn nvim_get_p_wiw() -> i64;

    /// Get global cmdline_row.
    fn nvim_get_cmdline_row() -> c_int;

    /// Get global p_ls (laststatus).
    fn nvim_get_p_ls() -> i64;

    /// Get global Columns.
    fn nvim_get_Columns() -> c_int;

    /// Check if winbar is globally enabled.
    fn nvim_global_winbar_height() -> c_int;

    /// Get global_stl_height().
    fn nvim_global_stl_height() -> c_int;

    /// Frame minheight calculation.
    fn rs_frame_minheight(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;

    /// Frame minwidth calculation.
    fn rs_frame_minwidth(topfrp: *const Frame, next_curwin: WinHandle) -> c_int;

    /// Check if frame has window.
    fn rs_frame_has_win(frp: *const Frame, wp: WinHandle) -> c_int;

    /// Check if frame has fixed height.
    fn rs_frame_fixed_height(frp: *const Frame) -> c_int;

    /// Check if frame has fixed width.
    fn rs_frame_fixed_width(frp: *const Frame) -> c_int;

    /// Call frame_new_height (C wrapper).
    fn nvim_frame_new_height(
        topfrp: *mut Frame,
        height: c_int,
        topfirst: bool,
        wfh: bool,
        set_ch: bool,
    );

    /// Call frame_new_width (C wrapper).
    fn nvim_frame_new_width(topfrp: *mut Frame, width: c_int, leftfirst: bool, wfw: bool);

    /// Call redraw_all_later (C wrapper).
    fn nvim_redraw_all_later(redraw_type: c_int);

    /// Check if window is an autocmd window.
    fn is_aucmd_win(wp: WinHandle) -> c_int;

    /// Get the tabline height.
    fn tabline_height() -> c_int;

    /// Get global p_ead (equalalways direction).
    fn nvim_get_p_ead() -> *const std::ffi::c_char;

    /// Call win_fix_scroll (C wrapper).
    fn nvim_win_fix_scroll(resize: bool);

    /// Set w_winrow on a window.
    fn nvim_win_set_winrow(wp: WinHandle, row: c_int);

    /// Set w_wincol on a window.
    fn nvim_win_set_wincol(wp: WinHandle, col: c_int);

    /// Get w_winrow from a window.
    fn nvim_win_get_winrow(wp: WinHandle) -> c_int;

    /// Get w_wincol from a window.
    fn nvim_win_get_wincol(wp: WinHandle) -> c_int;

    /// Get Rows global.
    fn nvim_get_Rows() -> c_int;
}

/// Get the window from a frame (first leaf window).
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
unsafe fn frame2win(frp: *const Frame) -> WinHandle {
    if frp.is_null() {
        return WinHandle::null();
    }

    let frame = &*frp;
    if !frame.fr_win.is_null() {
        return frame.fr_win;
    }

    // Not a leaf, recurse to first child
    if !frame.fr_child.is_null() {
        return frame2win(frame.fr_child);
    }

    WinHandle::null()
}

// =============================================================================
// Maximum Window Count Calculation
// =============================================================================

/// Compute maximum number of windows that can fit within "height" in frame "fr".
///
/// This is the Rust equivalent of `get_maximum_wincount()` in window.c.
fn get_maximum_wincount_impl(frp: *const Frame, height: c_int) -> c_int {
    if frp.is_null() || height <= 0 {
        return 0;
    }

    unsafe {
        let frame = &*frp;
        let p_wmh = nvim_get_p_wmh() as c_int;

        if frame.fr_layout != FR_COL {
            // Not a column layout - simple calculation
            let wp = frame2win(frp);
            let winbar_height = if wp.is_null() {
                0
            } else {
                nvim_win_get_winbar_height(wp)
            };
            return height / (p_wmh + STATUS_HEIGHT + winbar_height);
        }

        // Check if winbar is globally enabled
        if nvim_global_winbar_height() > 0 {
            return height / (p_wmh + STATUS_HEIGHT + 1);
        }

        // Column layout - sum up children
        let mut total_wincount = 0;
        let mut child = frame.fr_child;

        while !child.is_null() {
            let child_frame = &*child;
            let wp = frame2win(child);
            let winbar_height = if wp.is_null() {
                0
            } else {
                nvim_win_get_winbar_height(wp)
            };

            let child_min = p_wmh + STATUS_HEIGHT + winbar_height;
            if child_min > 0 {
                // Each child contributes at least one window worth of count
                total_wincount += height / child_min;
            }

            child = child_frame.fr_next;
        }

        // Return at least 1 if there are any children
        if total_wincount == 0 && !frame.fr_child.is_null() {
            return 1;
        }

        total_wincount
    }
}

// =============================================================================
// Equalization Helpers
// =============================================================================

/// Calculate extra separator adjustment for vertical equalization.
///
/// Returns 1 if this is the rightmost column (no separator needed), 0 otherwise.
fn compute_extra_sep_horizontal_impl(col: c_int, width: c_int) -> c_int {
    unsafe { c_int::from(col + width == nvim_get_Columns()) }
}

/// Calculate extra separator adjustment for horizontal equalization.
///
/// Returns STATUS_HEIGHT if at bottom without statusline, 1 if global statusline,
/// 0 otherwise.
fn compute_extra_sep_vertical_impl(row: c_int, height: c_int) -> c_int {
    unsafe {
        let cmdline_row = nvim_get_cmdline_row();
        let p_ls = nvim_get_p_ls();

        if row + height >= cmdline_row && p_ls == 0 {
            STATUS_HEIGHT
        } else {
            c_int::from(nvim_global_stl_height() > 0)
        }
    }
}

/// Calculate total window count for horizontal (FR_ROW) equalization.
fn compute_total_wincount_horizontal_impl(frp: *const Frame, extra_sep: c_int) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let p_wmw = nvim_get_p_wmw() as c_int;
        let n = rs_frame_minwidth(frp, WinHandle::null());
        (n + extra_sep) / (p_wmw + 1)
    }
}

/// Calculate total window count for vertical (FR_COL) equalization.
fn compute_total_wincount_vertical_impl(frp: *const Frame, extra_sep: c_int) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let n = rs_frame_minheight(frp, WinHandle::null());
        get_maximum_wincount_impl(frp, n + extra_sep)
    }
}

/// Calculate room available for width distribution.
fn compute_room_horizontal_impl(frp: *const Frame, width: c_int, next_curwin: WinHandle) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let m = rs_frame_minwidth(frp, next_curwin);
        (width - m).max(0)
    }
}

/// Calculate room available for height distribution.
fn compute_room_vertical_impl(frp: *const Frame, height: c_int, next_curwin: WinHandle) -> c_int {
    if frp.is_null() {
        return 0;
    }

    unsafe {
        let m = rs_frame_minheight(frp, next_curwin);
        (height - m).max(0)
    }
}

/// Calculate the size for next_curwin in horizontal equalization.
///
/// Returns -1 if size should be computed later based on room distribution.
fn compute_next_curwin_width_impl(
    frp: *const Frame,
    width: c_int,
    next_curwin: WinHandle,
) -> c_int {
    if frp.is_null() || next_curwin.is_null() {
        return 0;
    }

    unsafe {
        let p_wiw = nvim_get_p_wiw() as c_int;
        let m = rs_frame_minwidth(frp, next_curwin);
        let room = width - m;

        if room < 0 {
            // Not enough room - give next_curwin as much as possible
            (p_wiw + room).max(0)
        } else {
            -1 // Will be computed during distribution
        }
    }
}

/// Calculate the size for next_curwin in vertical equalization.
///
/// Returns -1 if size should be computed later based on room distribution.
fn compute_next_curwin_height_impl(
    frp: *const Frame,
    height: c_int,
    next_curwin: WinHandle,
) -> c_int {
    if frp.is_null() || next_curwin.is_null() {
        return 0;
    }

    unsafe {
        let p_wh = nvim_get_p_wh() as c_int;
        let m = rs_frame_minheight(frp, next_curwin);
        let room = height - m;

        if room < 0 {
            // Not enough room - give next_curwin as much as possible
            (p_wh + room).max(0)
        } else {
            -1 // Will be computed during distribution
        }
    }
}

/// Check if a frame should be skipped during equalization.
///
/// Returns true if the frame is the curwin and equalizing only current frame.
fn should_skip_frame_impl(
    frp: *const Frame,
    next_curwin: WinHandle,
    current: bool,
    dir: c_int,
    new_size: c_int,
    is_height: bool,
) -> bool {
    if frp.is_null() {
        return true;
    }

    unsafe {
        let frame = &*frp;

        // Skip if equalizing only current and this frame doesn't need change
        if current {
            let dir_char = if is_height { b'h' } else { b'v' };
            if dir == c_int::from(dir_char) && frame.fr_parent.is_null() {
                let current_size = if is_height {
                    frame.fr_height
                } else {
                    frame.fr_width
                };
                if new_size == current_size && rs_frame_has_win(frp, next_curwin) == 0 {
                    return true;
                }
            }
        }

        false
    }
}

/// Distribute room among windows proportionally.
///
/// Given total room and window count, calculate the share for a specific window.
fn distribute_room_impl(room: c_int, wincount: c_int, totwincount: c_int) -> c_int {
    if totwincount == 0 {
        return room;
    }
    (wincount * room + totwincount / 2) / totwincount
}

/// Check if frame contains any window with fixed height.
fn frame_has_fixed_height_window_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe { rs_frame_fixed_height(frp) != 0 }
}

/// Check if frame contains any window with fixed width.
fn frame_has_fixed_width_window_impl(frp: *const Frame) -> bool {
    if frp.is_null() {
        return false;
    }

    unsafe { rs_frame_fixed_width(frp) != 0 }
}

// =============================================================================
// Main Equalization Functions
// =============================================================================

/// Make all windows the same height and/or width.
///
/// This is the Rust equivalent of `win_equal()` in window.c.
///
/// # Arguments
/// * `next_curwin` - pointer to current window to be or NULL
/// * `current` - do only frame with current window
/// * `dir` - 'v' for vertically, 'h' for horizontally, 'b' for both, 0 for using p_ead
#[allow(clippy::cast_sign_loss)]
fn win_equal_impl(next_curwin: WinHandle, current: bool, mut dir: c_int) {
    unsafe {
        if dir == 0 {
            // Get direction from p_ead option
            let p_ead = nvim_get_p_ead();
            if p_ead.is_null() {
                dir = c_int::from(b'b'); // Default to both
            } else {
                // p_ead is always an ASCII character ('v', 'h', or 'b')
                dir = c_int::from(*p_ead as u8);
            }
        }

        let topframe = nvim_get_topframe();
        if topframe.is_null() {
            return;
        }

        let curwin = nvim_get_curwin();
        let actual_next_curwin = if next_curwin.is_null() {
            curwin
        } else {
            next_curwin
        };

        win_equal_rec_impl(
            actual_next_curwin,
            current,
            topframe,
            dir,
            0,
            tabline_height(),
            nvim_get_Columns(),
            (*topframe).fr_height,
        );

        if is_aucmd_win(next_curwin) == 0 {
            nvim_win_fix_scroll(true);
        }
    }
}

/// Set a frame to a new position and height, spreading the available room
/// equally over contained frames.
///
/// This is the Rust equivalent of `win_equal_rec()` in window.c.
///
/// # Arguments
/// * `next_curwin` - pointer to current window to be or NULL
/// * `current` - do only frame with current window
/// * `topfr` - frame to set size off
/// * `dir` - 'v', 'h' or 'b', see win_equal()
/// * `col` - horizontal position for frame
/// * `row` - vertical position for frame
/// * `width` - new width of frame
/// * `height` - new height of frame
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::similar_names)]
fn win_equal_rec_impl(
    next_curwin: WinHandle,
    current: bool,
    topfr: *mut Frame,
    dir: c_int,
    col: c_int,
    row: c_int,
    mut width: c_int,
    mut height: c_int,
) {
    if topfr.is_null() {
        return;
    }

    unsafe {
        let frame = &mut *topfr;
        let p_wmh = nvim_get_p_wmh() as c_int;
        let p_wmw = nvim_get_p_wmw() as c_int;
        let p_wh = nvim_get_p_wh() as c_int;
        let p_wiw = nvim_get_p_wiw() as c_int;

        if frame.fr_layout == FR_LEAF {
            // Leaf frame - set the width/height of this frame
            let wp = frame.fr_win;
            if !wp.is_null() {
                let win_row = nvim_win_get_winrow(wp);
                let win_col = nvim_win_get_wincol(wp);

                // Redraw when size or position changes
                if frame.fr_height != height
                    || win_row != row
                    || frame.fr_width != width
                    || win_col != col
                {
                    nvim_win_set_winrow(wp, row);
                    nvim_frame_new_height(topfr, height, false, false, false);
                    nvim_win_set_wincol(wp, col);
                    nvim_frame_new_width(topfr, width, false, false);
                    nvim_redraw_all_later(UPD_NOT_VALID);
                }
            }
        } else if frame.fr_layout == FR_ROW {
            // Row of frames
            frame.fr_width = width;
            frame.fr_height = height;

            let mut extra_sep = 0;
            let mut totwincount = 0;
            let mut next_curwin_size = 0;
            let mut room = 0;
            let mut has_next_curwin = false;

            if dir != c_int::from(b'v') {
                // Equalize frame widths
                let n = rs_frame_minwidth(topfr, NOWIN);
                extra_sep = c_int::from(col + width == nvim_get_Columns());
                totwincount = (n + extra_sep) / (p_wmw + 1);
                has_next_curwin = rs_frame_has_win(topfr, next_curwin) != 0;

                let m = rs_frame_minwidth(topfr, next_curwin);
                room = width - m;
                if room < 0 {
                    next_curwin_size = p_wiw + room;
                    room = 0;
                } else {
                    next_curwin_size = -1;
                    let mut fr = frame.fr_child;
                    while !fr.is_null() {
                        let child = &mut *fr;
                        if rs_frame_fixed_width(fr) == 0 {
                            fr = child.fr_next;
                            continue;
                        }
                        let n_child = rs_frame_minwidth(fr, NOWIN);
                        let mut new_size = child.fr_width;
                        if rs_frame_has_win(fr, next_curwin) != 0 {
                            room += p_wiw - p_wmw;
                            next_curwin_size = 0;
                            new_size = std::cmp::max(new_size, p_wiw);
                        } else {
                            totwincount -= (n_child
                                + if child.fr_next.is_null() {
                                    extra_sep
                                } else {
                                    0
                                })
                                / (p_wmw + 1);
                        }
                        room -= new_size - n_child;
                        if room < 0 {
                            new_size += room;
                            room = 0;
                        }
                        child.fr_newwidth = new_size;
                        fr = child.fr_next;
                    }
                    if next_curwin_size == -1 {
                        if !has_next_curwin {
                            next_curwin_size = 0;
                        } else if totwincount > 1
                            && (room + (totwincount - 2)) / (totwincount - 1) > p_wiw
                        {
                            next_curwin_size =
                                (room + p_wiw + (totwincount - 1) * p_wmw + (totwincount - 1))
                                    / totwincount;
                            room -= next_curwin_size - p_wiw;
                        } else {
                            next_curwin_size = p_wiw;
                        }
                    }
                }

                if has_next_curwin {
                    totwincount -= 1;
                }
            }

            let mut fr = frame.fr_child;
            let mut cur_col = col;
            while !fr.is_null() {
                let child = &mut *fr;
                let mut wincount = 1;
                let new_size: c_int;

                if child.fr_next.is_null() {
                    new_size = width;
                } else if dir == c_int::from(b'v') {
                    new_size = child.fr_width;
                } else if rs_frame_fixed_width(fr) != 0 {
                    new_size = child.fr_newwidth;
                    wincount = 0;
                } else {
                    let n = rs_frame_minwidth(fr, NOWIN);
                    wincount = (n + if child.fr_next.is_null() {
                        extra_sep
                    } else {
                        0
                    }) / (p_wmw + 1);
                    let m = rs_frame_minwidth(fr, next_curwin);
                    let hnc = has_next_curwin && rs_frame_has_win(fr, next_curwin) != 0;
                    if hnc {
                        wincount -= 1;
                    }
                    let mut base_size = if totwincount == 0 {
                        room
                    } else {
                        (wincount * room + totwincount / 2) / totwincount
                    };
                    if hnc {
                        next_curwin_size -= p_wiw - (m - n);
                        next_curwin_size = std::cmp::max(next_curwin_size, 0);
                        base_size += next_curwin_size;
                        room -= base_size - next_curwin_size;
                    } else {
                        room -= base_size;
                    }
                    new_size = base_size + n;
                }

                // Skip frame that is full width when splitting or closing a window
                if !current
                    || dir != c_int::from(b'v')
                    || !frame.fr_parent.is_null()
                    || new_size != child.fr_width
                    || rs_frame_has_win(fr, next_curwin) != 0
                {
                    win_equal_rec_impl(
                        next_curwin,
                        current,
                        fr,
                        dir,
                        cur_col,
                        row,
                        new_size,
                        height,
                    );
                }
                cur_col += new_size;
                width -= new_size;
                totwincount -= wincount;
                fr = child.fr_next;
            }
        } else {
            // FR_COL: Column of frames
            frame.fr_width = width;
            frame.fr_height = height;

            let mut extra_sep = 0;
            let mut totwincount = 0;
            let mut next_curwin_size = 0;
            let mut room = 0;
            let mut has_next_curwin = false;

            if dir != c_int::from(b'h') {
                // Equalize frame heights
                let n = rs_frame_minheight(topfr, NOWIN);
                let cmdline_row = nvim_get_cmdline_row();
                let p_ls = nvim_get_p_ls();

                if row + height >= cmdline_row && p_ls == 0 {
                    extra_sep = STATUS_HEIGHT;
                } else if nvim_global_stl_height() > 0 {
                    extra_sep = 1;
                } else {
                    extra_sep = 0;
                }

                totwincount = get_maximum_wincount_impl(topfr, n + extra_sep);
                has_next_curwin = rs_frame_has_win(topfr, next_curwin) != 0;

                let m = rs_frame_minheight(topfr, next_curwin);
                room = height - m;
                if room < 0 {
                    next_curwin_size = p_wh + room;
                    room = 0;
                } else {
                    next_curwin_size = -1;
                    let mut fr = frame.fr_child;
                    while !fr.is_null() {
                        let child = &mut *fr;
                        if rs_frame_fixed_height(fr) == 0 {
                            fr = child.fr_next;
                            continue;
                        }
                        let n_child = rs_frame_minheight(fr, NOWIN);
                        let mut new_size = child.fr_height;
                        if rs_frame_has_win(fr, next_curwin) != 0 {
                            room += p_wh - p_wmh;
                            next_curwin_size = 0;
                            new_size = std::cmp::max(new_size, p_wh);
                        } else {
                            totwincount -= get_maximum_wincount_impl(
                                fr,
                                n_child
                                    + if child.fr_next.is_null() {
                                        extra_sep
                                    } else {
                                        0
                                    },
                            );
                        }
                        room -= new_size - n_child;
                        if room < 0 {
                            new_size += room;
                            room = 0;
                        }
                        child.fr_newheight = new_size;
                        fr = child.fr_next;
                    }
                    if next_curwin_size == -1 {
                        if !has_next_curwin {
                            next_curwin_size = 0;
                        } else if totwincount > 1
                            && (room + (totwincount - 2)) / (totwincount - 1) > p_wh
                        {
                            next_curwin_size =
                                (room + p_wh + (totwincount - 1) * p_wmh + (totwincount - 1))
                                    / totwincount;
                            room -= next_curwin_size - p_wh;
                        } else {
                            next_curwin_size = p_wh;
                        }
                    }
                }

                if has_next_curwin {
                    totwincount -= 1;
                }
            }

            let mut fr = frame.fr_child;
            let mut cur_row = row;
            while !fr.is_null() {
                let child = &mut *fr;
                let mut wincount = 1;
                let new_size: c_int;

                if child.fr_next.is_null() {
                    new_size = height;
                } else if dir == c_int::from(b'h') {
                    new_size = child.fr_height;
                } else if rs_frame_fixed_height(fr) != 0 {
                    new_size = child.fr_newheight;
                    wincount = 0;
                } else {
                    let n = rs_frame_minheight(fr, NOWIN);
                    wincount = get_maximum_wincount_impl(
                        fr,
                        n + if child.fr_next.is_null() {
                            extra_sep
                        } else {
                            0
                        },
                    );
                    let m = rs_frame_minheight(fr, next_curwin);
                    let hnc = has_next_curwin && rs_frame_has_win(fr, next_curwin) != 0;
                    if hnc {
                        wincount -= 1;
                    }
                    let mut base_size = if totwincount == 0 {
                        room
                    } else {
                        (wincount * room + totwincount / 2) / totwincount
                    };
                    if hnc {
                        next_curwin_size -= p_wh - (m - n);
                        base_size += next_curwin_size;
                        room -= base_size - next_curwin_size;
                    } else {
                        room -= base_size;
                    }
                    new_size = base_size + n;
                }

                // Skip frame that is full height when splitting or closing a window
                if !current
                    || dir != c_int::from(b'h')
                    || !frame.fr_parent.is_null()
                    || new_size != child.fr_height
                    || rs_frame_has_win(fr, next_curwin) != 0
                {
                    win_equal_rec_impl(
                        next_curwin,
                        current,
                        fr,
                        dir,
                        col,
                        cur_row,
                        width,
                        new_size,
                    );
                }
                cur_row += new_size;
                height -= new_size;
                totwincount -= wincount;
                fr = child.fr_next;
            }
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Make all windows the same height and/or width.
#[no_mangle]
pub extern "C" fn rs_win_equal(next_curwin: WinHandle, current: c_int, dir: c_int) {
    win_equal_impl(next_curwin, current != 0, dir);
}

/// FFI: Recursive window equalization.
#[no_mangle]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn rs_win_equal_rec(
    next_curwin: WinHandle,
    current: c_int,
    topfr: *mut Frame,
    dir: c_int,
    col: c_int,
    row: c_int,
    width: c_int,
    height: c_int,
) {
    win_equal_rec_impl(
        next_curwin,
        current != 0,
        topfr,
        dir,
        col,
        row,
        width,
        height,
    );
}

/// FFI: Compute maximum window count for a frame.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_max_wincount(frp: *const Frame, height: c_int) -> c_int {
    get_maximum_wincount_impl(frp, height)
}

/// FFI: Compute extra separator for horizontal equalization.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_extra_sep_h(col: c_int, width: c_int) -> c_int {
    compute_extra_sep_horizontal_impl(col, width)
}

/// FFI: Compute extra separator for vertical equalization.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_extra_sep_v(row: c_int, height: c_int) -> c_int {
    compute_extra_sep_vertical_impl(row, height)
}

/// FFI: Compute total window count for horizontal equalization.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_total_wincount_h(
    frp: *const Frame,
    extra_sep: c_int,
) -> c_int {
    compute_total_wincount_horizontal_impl(frp, extra_sep)
}

/// FFI: Compute total window count for vertical equalization.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_total_wincount_v(
    frp: *const Frame,
    extra_sep: c_int,
) -> c_int {
    compute_total_wincount_vertical_impl(frp, extra_sep)
}

/// FFI: Compute room for horizontal distribution.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_room_h(
    frp: *const Frame,
    width: c_int,
    next_curwin: WinHandle,
) -> c_int {
    compute_room_horizontal_impl(frp, width, next_curwin)
}

/// FFI: Compute room for vertical distribution.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_room_v(
    frp: *const Frame,
    height: c_int,
    next_curwin: WinHandle,
) -> c_int {
    compute_room_vertical_impl(frp, height, next_curwin)
}

/// FFI: Compute next_curwin width.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_next_curwin_width(
    frp: *const Frame,
    width: c_int,
    next_curwin: WinHandle,
) -> c_int {
    compute_next_curwin_width_impl(frp, width, next_curwin)
}

/// FFI: Compute next_curwin height.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_next_curwin_height(
    frp: *const Frame,
    height: c_int,
    next_curwin: WinHandle,
) -> c_int {
    compute_next_curwin_height_impl(frp, height, next_curwin)
}

/// FFI: Check if frame should be skipped during equalization.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_should_skip(
    frp: *const Frame,
    next_curwin: WinHandle,
    current: c_int,
    dir: c_int,
    new_size: c_int,
    is_height: c_int,
) -> c_int {
    c_int::from(should_skip_frame_impl(
        frp,
        next_curwin,
        current != 0,
        dir,
        new_size,
        is_height != 0,
    ))
}

/// FFI: Distribute room among windows.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_distribute_room(
    room: c_int,
    wincount: c_int,
    totwincount: c_int,
) -> c_int {
    distribute_room_impl(room, wincount, totwincount)
}

/// FFI: Check if frame has fixed height window.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_has_fixed_height(frp: *const Frame) -> c_int {
    c_int::from(frame_has_fixed_height_window_impl(frp))
}

/// FFI: Check if frame has fixed width window.
///
/// # Safety
/// Caller must ensure `frp` is null or a valid pointer to a Frame.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_equalize_has_fixed_width(frp: *const Frame) -> c_int {
    c_int::from(frame_has_fixed_width_window_impl(frp))
}

/// FFI: Get p_wh (winheight) value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_get_p_wh() -> c_int {
    unsafe { nvim_get_p_wh() as c_int }
}

/// FFI: Get p_wiw (winwidth) value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_get_p_wiw() -> c_int {
    unsafe { nvim_get_p_wiw() as c_int }
}

/// FFI: Get p_wmh (winminheight) value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_get_p_wmh() -> c_int {
    unsafe { nvim_get_p_wmh() as c_int }
}

/// FFI: Get p_wmw (winminwidth) value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_equalize_get_p_wmw() -> c_int {
    unsafe { nvim_get_p_wmw() as c_int }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_frame_max_wincount() {
        assert_eq!(get_maximum_wincount_impl(std::ptr::null(), 100), 0);
    }

    #[test]
    fn test_zero_height_max_wincount() {
        // With null frame, should return 0
        assert_eq!(get_maximum_wincount_impl(std::ptr::null(), 0), 0);
    }

    #[test]
    fn test_distribute_room() {
        // Test even distribution
        assert_eq!(distribute_room_impl(100, 1, 4), 25);
        assert_eq!(distribute_room_impl(100, 2, 4), 50);

        // Test with rounding
        assert_eq!(distribute_room_impl(10, 1, 3), 3); // 10/3 ≈ 3.33, rounds to 3

        // Test edge cases
        assert_eq!(distribute_room_impl(100, 1, 0), 100); // No windows, get all room
        assert_eq!(distribute_room_impl(0, 1, 4), 0); // No room to distribute
    }

    #[test]
    fn test_null_frame_helpers() {
        let null_frame: *const Frame = std::ptr::null();
        assert!(!frame_has_fixed_height_window_impl(null_frame));
        assert!(!frame_has_fixed_width_window_impl(null_frame));
        assert!(!should_skip_frame_impl(
            null_frame,
            WinHandle::null(),
            false,
            0,
            10,
            true
        ));
    }
}
