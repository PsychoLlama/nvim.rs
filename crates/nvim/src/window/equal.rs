//! `win_equal()` -- redistributing the space between windows.
//!
//! The CTRL-W = operation and the `'equalalways'` implementation:
//! [`equal_rec`] walks the frame tree handing each child its share of the
//! room, honouring `'eadirection'`, the `'winfix{height,width}'` pins, the
//! minimum sizes and the status lines and separators that are not text, and
//! recursing into rows and columns until every leaf has been given a
//! size.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::arith::NextCurwin;
use super::*;
use crate::drawscreen::UPD_NOT_VALID;
use crate::main::{Columns, cmdline_row, p_ead, p_ls, p_wh, p_wiw, p_wmh, p_wmw};
use crate::types::{OptInt, win_T};
use crate::winlayer::{Frame, Win};

pub unsafe extern "C" fn win_equal(next_curwin: *mut win_T, current: bool, dir: c_int) {
    // SAFETY: the caller's promise -- a live window, or null for "the current
    // one".
    equal(unsafe { Win::from_raw(next_curwin) }, current, dir);
}

/// Make all windows the same size, from `win_equal()`.
///
/// `next_curwin` will soon be the current window, so it is the one that gets
/// `'winheight'`/`'winwidth'` if there is room; `dir` is `'v'`, `'h'`, `'b'`
/// or 0 to take it from `'eadirection'`.
pub(crate) fn equal(next_curwin: Option<Win>, current: bool, dir: c_int) {
    let dir = if dir == 0 {
        // SAFETY: `'eadirection'` is a NUL-terminated option string.
        unsafe { *p_ead.get() as ::core::ffi::c_uchar as c_int }
    } else {
        dir
    };
    let topfr = current_topframe();
    equal_rec(
        next_curwin.unwrap_or_else(cur_win),
        current,
        topfr,
        dir,
        0,
        tabline_rows(),
        Columns.get(),
        topfr.fr_height,
    );
    if !is_autocmd_window(next_curwin) {
        fix_scroll(true);
    }
}

/// Set frame `topfr` to a new position and size, spreading the room equally
/// over the frames it contains, from `win_equal_rec()`.
#[expect(clippy::too_many_arguments, reason = "upstream's eight, all needed")]
fn equal_rec(
    next_curwin: Win,
    current: bool,
    topfr: Frame,
    dir: c_int,
    col: c_int,
    row: c_int,
    width: c_int,
    height: c_int,
) {
    if topfr.fr_layout as c_int == FR_LEAF {
        equal_leaf(topfr, col, row, width, height);
    } else if topfr.fr_layout as c_int == FR_ROW {
        equal_row(next_curwin, current, topfr, dir, col, row, width, height);
    } else {
        equal_col(next_curwin, current, topfr, dir, col, row, width, height);
    }
}

/// A leaf: move and resize the one window, and redraw if either changed.
fn equal_leaf(topfr: Frame, col: c_int, row: c_int, width: c_int, height: c_int) {
    let mut win = topfr.win().expect("a leaf frame holds a window");
    if topfr.fr_height != height
        || win.w_winrow != row
        || topfr.fr_width != width
        || win.w_wincol != col
    {
        win.w_winrow = row;
        new_height(topfr, height, false, false, false);
        win.w_wincol = col;
        new_width(topfr, width, false, false);
        redraw_all(UPD_NOT_VALID);
    }
}

/// How much room a row or column has to give away, and how many windows will
/// be sharing it: the state upstream carries in `room`, `totwincount`,
/// `next_curwin_size`, `extra_sep` and `has_next_curwin`.
struct Share {
    room: c_int,
    totwincount: c_int,
    next_curwin_size: c_int,
    extra_sep: c_int,
    has_next_curwin: bool,
}

impl Share {
    /// The state before either axis has been equalized, which is what the
    /// `dir` that skips this axis leaves in place.
    fn none() -> Self {
        Self {
            room: 0,
            totwincount: 0,
            next_curwin_size: 0,
            extra_sep: 0,
            has_next_curwin: false,
        }
    }
}

/// A row of frames: equalize the widths unless `'eadirection'` says vertical
/// only, then hand each child its column of the screen.
#[expect(clippy::too_many_arguments, reason = "upstream's eight, all needed")]
fn equal_row(
    next_curwin: Win,
    current: bool,
    topfr: Frame,
    dir: c_int,
    col: c_int,
    row: c_int,
    width: c_int,
    height: c_int,
) {
    let mut topfr = topfr;
    topfr.fr_width = width;
    topfr.fr_height = height;
    let mut sh = Share::none();
    if dir != 'v' as c_int {
        sh = row_share(next_curwin, topfr, col, width);
    }

    let (mut col, mut width) = (col, width);
    for fr in topfr.children() {
        let mut wincount = 1;
        let new_size = if fr.next().is_none() {
            // Last frame gets all that remains (avoids a rounding error).
            width
        } else if dir == 'v' as c_int {
            fr.fr_width
        } else if frame_fixed_width(fr) {
            wincount = 0; // doesn't count as a sizeable window
            fr.fr_newwidth
        } else {
            // The maximum number of windows horizontally in "fr".
            let n = minwidth(fr, NextCurwin::NoWin);
            let sep = if fr.next().is_none() { sh.extra_sep } else { 0 };
            wincount = (n + sep) / (p_wmw.get() as c_int + 1);
            let m = minwidth(fr, NextCurwin::Win(next_curwin.raw()));
            let hnc = sh.has_next_curwin && frame_has_win(fr, Some(next_curwin));
            if hnc {
                wincount -= 1; // don't count next_curwin
            }
            let mut new_size = if sh.totwincount == 0 {
                sh.room
            } else {
                (wincount * sh.room + sh.totwincount / 2) / sh.totwincount
            };
            if hnc {
                // Add next_curwin's own size on top of its share.
                sh.next_curwin_size -= p_wiw.get() as c_int - (m - n);
                sh.next_curwin_size = sh.next_curwin_size.max(0);
                new_size += sh.next_curwin_size;
                sh.room -= new_size - sh.next_curwin_size;
            } else {
                sh.room -= new_size;
            }
            new_size + n
        };

        // Skip a frame that is full width when splitting or closing a window,
        // unless equalizing all frames.
        if !current
            || dir != 'v' as c_int
            || topfr.parent().is_some()
            || new_size != fr.fr_width
            || frame_has_win(fr, Some(next_curwin))
        {
            equal_rec(next_curwin, current, fr, dir, col, row, new_size, height);
        }
        col += new_size;
        width -= new_size;
        sh.totwincount -= wincount;
    }
}

/// The width `next_curwin` may have and the room left for everyone else.
fn row_share(next_curwin: Win, topfr: Frame, col: c_int, width: c_int) -> Share {
    // The maximum number of windows horizontally in this frame; the rightmost
    // one has no separator, so it is worth one extra column.
    let mut n = minwidth(topfr, NextCurwin::NoWin);
    let extra_sep = if col + width == Columns.get() { 1 } else { 0 };
    let mut totwincount = (n + extra_sep) / (p_wmw.get() as c_int + 1);
    let has_next_curwin = frame_has_win(topfr, Some(next_curwin));

    // "m" is the minimal width when counting 'winwidth' for "next_curwin".
    let m = minwidth(topfr, NextCurwin::Win(next_curwin.raw()));
    let mut room = width - m;
    let mut next_curwin_size;
    if room < 0 {
        next_curwin_size = p_wiw.get() as c_int + room;
        room = 0;
    } else {
        next_curwin_size = -1;
        for mut fr in topfr.children() {
            if !frame_fixed_width(fr) {
                continue;
            }
            // With 'winfixwidth' keep the window width if possible -- watching
            // out for this window being the next_curwin.
            n = minwidth(fr, NextCurwin::NoWin);
            let mut new_size = fr.fr_width;
            if frame_has_win(fr, Some(next_curwin)) {
                room += p_wiw.get() as c_int - p_wmw.get() as c_int;
                next_curwin_size = 0;
                new_size = new_size.max(p_wiw.get() as c_int);
            } else {
                // These windows don't use up room.
                let sep = if fr.next().is_none() { extra_sep } else { 0 };
                totwincount -= (n + sep) / (p_wmw.get() as c_int + 1);
            }
            room -= new_size - n;
            if room < 0 {
                new_size += room;
                room = 0;
            }
            fr.fr_newwidth = new_size;
        }
        if next_curwin_size == -1 {
            if !has_next_curwin {
                next_curwin_size = 0;
            } else if totwincount > 1
                && ((room + (totwincount - 2)) / (totwincount - 1)) as OptInt > p_wiw.get()
            {
                // Can make all windows wider than 'winwidth': spread the room
                // equally.
                next_curwin_size = (room as OptInt
                    + p_wiw.get()
                    + (totwincount - 1) as OptInt * p_wmw.get()
                    + (totwincount - 1) as OptInt) as c_int
                    / totwincount;
                room -= next_curwin_size - p_wiw.get() as c_int;
            } else {
                next_curwin_size = p_wiw.get() as c_int;
            }
        }
    }
    if has_next_curwin {
        totwincount -= 1; // don't count curwin
    }
    Share {
        room,
        totwincount,
        next_curwin_size,
        extra_sep,
        has_next_curwin,
    }
}

/// A column of frames: [`equal_row`] with the axes exchanged.
#[expect(clippy::too_many_arguments, reason = "upstream's eight, all needed")]
fn equal_col(
    next_curwin: Win,
    current: bool,
    topfr: Frame,
    dir: c_int,
    col: c_int,
    row: c_int,
    width: c_int,
    height: c_int,
) {
    let mut topfr = topfr;
    topfr.fr_width = width;
    topfr.fr_height = height;
    let mut sh = Share::none();
    if dir != 'h' as c_int {
        sh = col_share(next_curwin, topfr, row, height);
    }

    let (mut row, mut height) = (row, height);
    for fr in topfr.children() {
        let mut wincount = 1;
        let new_size = if fr.next().is_none() {
            height
        } else if dir == 'h' as c_int {
            fr.fr_height
        } else if frame_fixed_height(fr) {
            wincount = 0;
            fr.fr_newheight
        } else {
            let n = minheight(fr, NextCurwin::NoWin);
            let sep = if fr.next().is_none() { sh.extra_sep } else { 0 };
            wincount = max_wincount(fr, n + sep);
            let m = minheight(fr, NextCurwin::Win(next_curwin.raw()));
            let hnc = sh.has_next_curwin && frame_has_win(fr, Some(next_curwin));
            if hnc {
                wincount -= 1;
            }
            let mut new_size = if sh.totwincount == 0 {
                sh.room
            } else {
                (wincount * sh.room + sh.totwincount / 2) / sh.totwincount
            };
            if hnc {
                // Upstream clamps at zero on the width axis and not here.
                sh.next_curwin_size -= p_wh.get() as c_int - (m - n);
                new_size += sh.next_curwin_size;
                sh.room -= new_size - sh.next_curwin_size;
            } else {
                sh.room -= new_size;
            }
            new_size + n
        };

        if !current
            || dir != 'h' as c_int
            || topfr.parent().is_some()
            || new_size != fr.fr_height
            || frame_has_win(fr, Some(next_curwin))
        {
            equal_rec(next_curwin, current, fr, dir, col, row, width, new_size);
        }
        row += new_size;
        height -= new_size;
        sh.totwincount -= wincount;
    }
}

/// [`row_share`] with the axes exchanged.
fn col_share(next_curwin: Win, topfr: Frame, row: c_int, height: c_int) -> Share {
    let mut n = minheight(topfr, NextCurwin::NoWin);
    // Add one for the bottom window if it has neither status line nor
    // separator.
    let extra_sep = if row + height >= cmdline_row.get() && p_ls.get() == 0 as OptInt {
        STATUS_HEIGHT as c_int
    } else if global_stl_rows() > 0 {
        1
    } else {
        0
    };
    let mut totwincount = max_wincount(topfr, n + extra_sep);
    let has_next_curwin = frame_has_win(topfr, Some(next_curwin));

    let m = minheight(topfr, NextCurwin::Win(next_curwin.raw()));
    let mut room = height - m;
    let mut next_curwin_size;
    if room < 0 {
        // The room is less than 'winheight': use all space for the current
        // window.
        next_curwin_size = p_wh.get() as c_int + room;
        room = 0;
    } else {
        next_curwin_size = -1;
        for mut fr in topfr.children() {
            if !frame_fixed_height(fr) {
                continue;
            }
            n = minheight(fr, NextCurwin::NoWin);
            let mut new_size = fr.fr_height;
            if frame_has_win(fr, Some(next_curwin)) {
                room += p_wh.get() as c_int - p_wmh.get() as c_int;
                next_curwin_size = 0;
                new_size = new_size.max(p_wh.get() as c_int);
            } else {
                let sep = if fr.next().is_none() { extra_sep } else { 0 };
                totwincount -= max_wincount(fr, n + sep);
            }
            room -= new_size - n;
            if room < 0 {
                new_size += room;
                room = 0;
            }
            fr.fr_newheight = new_size;
        }
        if next_curwin_size == -1 {
            if !has_next_curwin {
                next_curwin_size = 0;
            } else if totwincount > 1
                && ((room + (totwincount - 2)) / (totwincount - 1)) as OptInt > p_wh.get()
            {
                next_curwin_size = (room as OptInt
                    + p_wh.get()
                    + (totwincount - 1) as OptInt * p_wmh.get()
                    + (totwincount - 1) as OptInt) as c_int
                    / totwincount;
                room -= next_curwin_size - p_wh.get() as c_int;
            } else {
                next_curwin_size = p_wh.get() as c_int;
            }
        }
    }
    if has_next_curwin {
        totwincount -= 1;
    }
    Share {
        room,
        totwincount,
        next_curwin_size,
        extra_sep,
        has_next_curwin,
    }
}
