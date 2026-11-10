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

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use crate::narrow::number_as_int;
use core::ffi::c_int;

use super::arith::NextCurwin;
use super::*;
use crate::drawscreen::UPD_NOT_VALID;
use crate::drawscreen::state::CMDLINE_ROW;
use crate::option::vars::{p_ead, p_ls, p_wh, p_wiw, p_wmh, p_wmw};
use crate::types::OptInt;
use crate::ui::state::COLUMNS;
use crate::winlayer::{FrameRef, Win};

pub fn win_equal(next_curwin: Option<Win>, current: bool, dir: u8) {
    equal(next_curwin, current, dir);
}

/// Make all windows the same size, from `win_equal()`.
///
/// `next_curwin` will soon be the current window, so it is the one that gets
/// `'winheight'`/`'winwidth'` if there is room; `dir` is `'v'`, `'h'`, `'b'`
/// or 0 to take it from `'eadirection'`.
pub(crate) fn equal(next_curwin: Option<Win>, current: bool, dir: u8) {
    let dir = if dir == 0 { p_ead(cstr::first) } else { dir };
    let topfr = current_topframe();
    equal_rec(
        next_curwin.unwrap_or_else(Win::current),
        current,
        topfr,
        dir,
        0,
        tabline_rows(),
        COLUMNS.get(),
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
    topfr: FrameRef,
    dir: u8,
    col: c_int,
    row: c_int,
    width: c_int,
    height: c_int,
) {
    if c_int::from(topfr.fr_layout) == FR_LEAF {
        topfr.equalize_leaf(col, row, width, height);
    } else {
        let axis = if c_int::from(topfr.fr_layout) == FR_ROW {
            Axis::Width
        } else {
            Axis::Height
        };
        equal_line(
            axis,
            next_curwin,
            current,
            topfr,
            dir,
            col,
            row,
            width,
            height,
        );
    }
}

impl FrameRef {
    /// A leaf: move and resize the one window, and redraw if either changed.
    fn equalize_leaf(self, col: c_int, row: c_int, width: c_int, height: c_int) {
        let mut win = self.win().expect("a leaf frame holds a window");
        if self.fr_height != height
            || win.w_winrow != row
            || self.fr_width != width
            || win.w_wincol != col
        {
            win.w_winrow = row;
            new_height(self, height, false, false, false);
            win.w_wincol = col;
            new_width(self, width, false, false);
            redraw_all(UPD_NOT_VALID);
        }
    }
}

/// How much room a row or column has to give away, and how many windows will
/// be sharing it: the state upstream carries in `room`, `totwincount`,
/// `next_curwin_size`, `extra_sep` and `has_next_curwin`. `Default` is the
/// state before either axis has been equalized, which is what the `dir` that
/// skips this axis leaves in place.
#[derive(Debug, Default)]
struct Share {
    room: c_int,
    totwincount: c_int,
    next_curwin_size: c_int,
    extra_sep: c_int,
    has_next_curwin: bool,
}

/// A row (widths) or a column (heights) of frames: equalize along `axis`
/// unless `'eadirection'` says the other one only, then hand each child its
/// stretch of the screen.
#[expect(clippy::too_many_arguments, reason = "upstream's eight and the axis")]
fn equal_line(
    axis: Axis,
    next_curwin: Win,
    current: bool,
    topfr: FrameRef,
    dir: u8,
    col: c_int,
    row: c_int,
    width: c_int,
    height: c_int,
) {
    let mut topfr = topfr;
    topfr.fr_width = width;
    topfr.fr_height = height;
    let skipped = dir == axis.skipped_by();
    let (mut pos, mut len) = axis.along(col, row, width, height);
    let mut sh = if skipped {
        Share::default()
    } else {
        share(axis, next_curwin, topfr, pos, len)
    };

    for fr in topfr.children() {
        let mut wincount = 1;
        let new_size = if fr.next().is_none() {
            // Last frame gets all that remains (avoids a rounding error).
            len
        } else if skipped {
            axis.size(fr)
        } else if axis.is_fixed(fr) {
            wincount = 0; // doesn't count as a sizeable window
            axis.new_size(fr)
        } else {
            // The maximum number of windows along the axis in "fr".
            let n = axis.min_size(fr, NextCurwin::NoWin);
            let sep = if fr.next().is_none() { sh.extra_sep } else { 0 };
            wincount = axis.win_count(fr, n + sep);
            let m = axis.min_size(fr, NextCurwin::of(next_curwin));
            let hnc = sh.has_next_curwin && fr.has_win(Some(next_curwin));
            if hnc {
                wincount -= 1; // don't count next_curwin
            }
            let mut new_size = if sh.totwincount == 0 {
                sh.room
            } else {
                (wincount * sh.room + sh.totwincount / 2) / sh.totwincount
            };
            if hnc {
                // Add next_curwin's own size on top of its share. Upstream
                // clamps at zero on the width axis and not on the height.
                sh.next_curwin_size -= number_as_int(axis.wanted()) - (m - n);
                if axis == Axis::Width {
                    sh.next_curwin_size = sh.next_curwin_size.max(0);
                }
                new_size += sh.next_curwin_size;
                sh.room -= new_size - sh.next_curwin_size;
            } else {
                sh.room -= new_size;
            }
            new_size + n
        };

        // Skip a frame that is full size when splitting or closing a window,
        // unless equalizing all frames.
        if !current
            || !skipped
            || topfr.parent().is_some()
            || new_size != axis.size(fr)
            || fr.has_win(Some(next_curwin))
        {
            let (col, row, width, height) = match axis {
                Axis::Width => (pos, row, new_size, height),
                Axis::Height => (col, pos, width, new_size),
            };
            equal_rec(next_curwin, current, fr, dir, col, row, width, height);
        }
        pos += new_size;
        len -= new_size;
        sh.totwincount -= wincount;
    }
}

/// The size `next_curwin` may have along `axis` and the room left for
/// everyone else, in the `len` cells from `pos`.
fn share(axis: Axis, next_curwin: Win, topfr: FrameRef, pos: c_int, len: c_int) -> Share {
    // The maximum number of windows along the axis in this frame, counting
    // the cell the last one gains.
    let mut n = axis.min_size(topfr, NextCurwin::NoWin);
    let extra_sep = axis.extra_sep(pos, len);
    let mut totwincount = axis.win_count(topfr, n + extra_sep);
    let has_next_curwin = topfr.has_win(Some(next_curwin));

    // "m" is the minimal size when counting 'winwidth'/'winheight' for
    // "next_curwin".
    let m = axis.min_size(topfr, NextCurwin::of(next_curwin));
    let mut room = len - m;
    let mut next_curwin_size;
    if room < 0 {
        // Less room than 'winwidth'/'winheight': all of it goes to the
        // current window.
        next_curwin_size = number_as_int(axis.wanted()) + room;
        room = 0;
    } else {
        next_curwin_size = -1;
        for fr in topfr.children() {
            if !axis.is_fixed(fr) {
                continue;
            }
            // A pinned window keeps its size if possible -- watching out for
            // this window being the next_curwin.
            n = axis.min_size(fr, NextCurwin::NoWin);
            let mut new_size = axis.size(fr);
            if fr.has_win(Some(next_curwin)) {
                room += number_as_int(axis.wanted()) - number_as_int(axis.minimum());
                next_curwin_size = 0;
                new_size = new_size.max(number_as_int(axis.wanted()));
            } else {
                // These windows don't use up room.
                let sep = if fr.next().is_none() { extra_sep } else { 0 };
                totwincount -= axis.win_count(fr, n + sep);
            }
            room -= new_size - n;
            if room < 0 {
                new_size += room;
                room = 0;
            }
            axis.set_new_size(fr, new_size);
        }
        if next_curwin_size == -1 {
            if !has_next_curwin {
                next_curwin_size = 0;
            } else if totwincount > 1
                && OptInt::from((room + (totwincount - 2)) / (totwincount - 1)) > axis.wanted()
            {
                // Can make all windows bigger than 'winwidth'/'winheight':
                // spread the room equally.
                next_curwin_size = number_as_int(
                    OptInt::from(room)
                        + axis.wanted()
                        + OptInt::from(totwincount - 1) * axis.minimum()
                        + OptInt::from(totwincount - 1),
                ) / totwincount;
                room -= next_curwin_size - number_as_int(axis.wanted());
            } else {
                next_curwin_size = number_as_int(axis.wanted());
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

/// The direction a row or a column shares its room along: a row hands out
/// widths, a column heights. Upstream wrote `win_equal_rec()`'s two halves
/// out twice with the names exchanged; this is what differs between them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axis {
    Width,
    Height,
}

impl Axis {
    /// The `'eadirection'` letter that leaves this axis as it is.
    fn skipped_by(self) -> u8 {
        match self {
            Axis::Width => b'v',
            Axis::Height => b'h',
        }
    }

    /// The position and the extent along this axis of the rectangle
    /// `col`, `row`, `width`, `height`.
    fn along(self, col: c_int, row: c_int, width: c_int, height: c_int) -> (c_int, c_int) {
        match self {
            Axis::Width => (col, width),
            Axis::Height => (row, height),
        }
    }

    fn size(self, fr: FrameRef) -> c_int {
        match self {
            Axis::Width => fr.fr_width,
            Axis::Height => fr.fr_height,
        }
    }

    fn min_size(self, fr: FrameRef, next_curwin: NextCurwin) -> c_int {
        match self {
            Axis::Width => fr.min_width(next_curwin),
            Axis::Height => fr.min_height(next_curwin),
        }
    }

    /// `'winfixwidth'` / `'winfixheight'`.
    fn is_fixed(self, fr: FrameRef) -> bool {
        match self {
            Axis::Width => fr.fixed_width(),
            Axis::Height => fr.fixed_height(),
        }
    }

    /// The size a pinned frame was given by [`share`].
    fn new_size(self, fr: FrameRef) -> c_int {
        match self {
            Axis::Width => fr.fr_newwidth,
            Axis::Height => fr.fr_newheight,
        }
    }

    fn set_new_size(self, mut fr: FrameRef, size: c_int) {
        match self {
            Axis::Width => fr.fr_newwidth = size,
            Axis::Height => fr.fr_newheight = size,
        }
    }

    /// `'winwidth'` / `'winheight'`: what the next current window wants.
    fn wanted(self) -> OptInt {
        match self {
            Axis::Width => p_wiw(),
            Axis::Height => p_wh(),
        }
    }

    /// `'winminwidth'` / `'winminheight'`.
    fn minimum(self) -> OptInt {
        match self {
            Axis::Width => p_wmw(),
            Axis::Height => p_wmh(),
        }
    }

    /// The maximum number of windows `fr` can hold along this axis in `n`
    /// cells.
    fn win_count(self, fr: FrameRef, n: c_int) -> c_int {
        match self {
            Axis::Width => n / (number_as_int(p_wmw()) + 1),
            Axis::Height => fr.max_win_count(n),
        }
    }

    /// The cell the last window along this axis gains at `pos + len`: the
    /// rightmost window has no separator, and the bottom one may have no
    /// status line.
    fn extra_sep(self, pos: c_int, len: c_int) -> c_int {
        match self {
            Axis::Width => c_int::from(pos + len == COLUMNS.get()),
            Axis::Height if pos + len >= CMDLINE_ROW.get() && p_ls() == 0 as OptInt => {
                STATUS_HEIGHT
            }
            Axis::Height => c_int::from(global_stl_rows() > 0),
        }
    }
}
