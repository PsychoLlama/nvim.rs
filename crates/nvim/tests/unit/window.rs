//! The window-layout arithmetic (`src/nvim/window/arith.rs`).
//!
//! Between the frame-tree writes that `win_split_ins()`, `win_equal()` and
//! `frame_setheight()` make sit a handful of decisions that are pure integer
//! work: how small a frame may become given `'winminheight'` and the rows that
//! are not text, whether a frame ended up the size it was asked for, where the
//! cursor sits as a fraction of a window's height, and — new here — what a
//! `:resize` with an absurd count should do instead of overflowing.
//!
//! Each expectation is derived from `v0.12.4`'s `src/nvim/window.c` rather than
//! from the port: the C function each case is aiming at is named in its
//! comment. The frame trees are built here out of plain `frame_T`s and zeroed
//! `win_T`s; no editor is started, no global is read, which is also what lets
//! Miri run the lot.

use std::ffi::c_int;
use std::ptr;

use c2rust_neovim::src::nvim::types::{frame_T, win_T};
use c2rust_neovim::src::nvim::window::arith::{
    MinSize, NextCurwin, cursor_fraction, fraction_row, frame_check_height, frame_check_width,
    frame_minheight, frame_minwidth, height_with_chrome, parent_target, sort_columns,
    width_with_chrome,
};
use c2rust_neovim::src::nvim::winlayer::Frame;

// The layouts below mirror the C's: `FR_LEAF` 0, `FR_ROW` 1, `FR_COL` 2.
const FR_LEAF: i8 = 0;
const FR_ROW: i8 = 1;
const FR_COL: i8 = 2;

// ----------------------------------------------------------- the test layouts
//
// A tree is built out of boxed nodes that live for the whole test; `Frame` is
// only ever handed a pointer into one of them, which is exactly the promise its
// constructor takes.

/// A window with the chrome a leaf frame's minimum size is made of.
#[derive(Clone, Copy, Default)]
struct Chrome {
    winbar: c_int,
    hsep: c_int,
    status: c_int,
    vsep: c_int,
}

/// A frame tree, kept alive as long as the `Frame`s handed out of it.
///
/// The nodes are `Box::into_raw`'d rather than kept as `Box`es: a raw pointer
/// taken out of a live `Box` is invalidated as soon as the `Box` itself is
/// used again (Miri's stacked borrows says so, and it is right), and the whole
/// point here is that the tree points at them.
struct Tree {
    nodes: Vec<*mut frame_T>,
    windows: Vec<*mut win_T>,
}

impl Drop for Tree {
    fn drop(&mut self) {
        for &node in &self.nodes {
            // SAFETY: each was `Box::into_raw`'d by this tree and is freed
            // exactly once.
            drop(unsafe { Box::from_raw(node) });
        }
        for &win in &self.windows {
            // SAFETY: as above.
            drop(unsafe { Box::from_raw(win) });
        }
    }
}

impl Tree {
    fn new() -> Self {
        Tree {
            nodes: Vec::new(),
            windows: Vec::new(),
        }
    }

    fn zeroed_frame(&mut self) -> *mut frame_T {
        // SAFETY: `frame_T` is a plain C struct of integers and pointers, and
        // an all-zero one is what `xcalloc` hands `new_frame()`.
        let node: Box<frame_T> = Box::new(unsafe { std::mem::zeroed() });
        let ptr = Box::into_raw(node);
        self.nodes.push(ptr);
        ptr
    }

    /// A leaf frame holding a window with the given chrome, `height` rows and
    /// `width` columns.
    fn leaf(&mut self, chrome: Chrome, height: c_int, width: c_int) -> *mut frame_T {
        // SAFETY: as above; `arith` reads only the four chrome fields.
        let win: Box<win_T> = Box::new(unsafe { std::mem::zeroed() });
        let wp = Box::into_raw(win);
        self.windows.push(wp);
        // SAFETY: the window was just allocated and outlives the tree.
        unsafe {
            (*wp).w_winbar_height = chrome.winbar;
            (*wp).w_hsep_height = chrome.hsep;
            (*wp).w_status_height = chrome.status;
            (*wp).w_vsep_width = chrome.vsep;
        }

        let fr = self.zeroed_frame();
        // SAFETY: the node was just allocated and outlives every use.
        unsafe {
            (*fr).fr_layout = FR_LEAF;
            (*fr).fr_win = wp;
            (*fr).fr_height = height;
            (*fr).fr_width = width;
        }
        fr
    }

    /// A row (`FR_ROW`) or column (`FR_COL`) of `children`, linked as the
    /// layout tree links them.
    fn branch(&mut self, layout: i8, children: &[*mut frame_T]) -> *mut frame_T {
        let parent = self.zeroed_frame();
        // SAFETY: every node here was allocated by this `Tree` and outlives it.
        unsafe {
            (*parent).fr_layout = layout;
            (*parent).fr_child = children[0];
            for (i, &child) in children.iter().enumerate() {
                (*child).fr_parent = parent;
                (*child).fr_prev = if i == 0 {
                    ptr::null_mut()
                } else {
                    children[i - 1]
                };
                (*child).fr_next = children.get(i + 1).copied().unwrap_or(ptr::null_mut());
            }
            let (h, w) = ((*children[0]).fr_height, (*children[0]).fr_width);
            (*parent).fr_height = h;
            (*parent).fr_width = w;
        }
        parent
    }

    fn set_size(&self, fr: *mut frame_T, height: c_int, width: c_int) {
        // SAFETY: a node of this tree.
        unsafe {
            (*fr).fr_height = height;
            (*fr).fr_width = width;
        }
    }

    fn window_of(&self, fr: *mut frame_T) -> *mut win_T {
        // SAFETY: a node of this tree.
        unsafe { (*fr).fr_win }
    }

    fn frame(&self, fr: *mut frame_T) -> Frame {
        // SAFETY: a node of this tree, which outlives the `Frame`.
        unsafe { Frame::new(fr) }
    }
}

/// `'winheight'` 10, `'winminheight'` 1, and no current window to reserve for.
fn heights(wanted: c_int, minimum: c_int) -> MinSize {
    MinSize {
        wanted,
        minimum,
        curwin: ptr::null_mut(),
    }
}

const PLAIN: Chrome = Chrome {
    winbar: 0,
    hsep: 0,
    status: 0,
    vsep: 0,
};

// ------------------------------------------------------------ frame_minheight
//
// `frame_minheight()`:
//
//   if (topfrp->fr_win != NULL) {
//     int extra = winbar + hsep + status;
//     m = (topfrp->fr_win == next_curwin ? p_wh : p_wmh) + extra;
//     if (fr_win == curwin && next_curwin == NULL && p_wmh == 0) m++;
//   } else if (fr_layout == FR_ROW) { m = max over children }
//   else { m = sum over children }

#[test]
fn a_leaf_costs_winminheight_plus_its_chrome() {
    let mut t = Tree::new();
    let status = Chrome { status: 1, ..PLAIN };
    let leaf = t.leaf(status, 5, 20);
    assert_eq!(
        frame_minheight(t.frame(leaf), NextCurwin::NoWin, heights(10, 1)),
        2
    );
}

#[test]
fn a_leaf_with_a_window_bar_and_a_separator_costs_all_three() {
    let mut t = Tree::new();
    let chrome = Chrome {
        winbar: 1,
        hsep: 1,
        status: 1,
        vsep: 0,
    };
    let leaf = t.leaf(chrome, 5, 20);
    assert_eq!(
        frame_minheight(t.frame(leaf), NextCurwin::NoWin, heights(10, 2)),
        5
    );
}

#[test]
fn the_next_curwin_leaf_costs_winheight_instead() {
    let mut t = Tree::new();
    let status = Chrome { status: 1, ..PLAIN };
    let leaf = t.leaf(status, 5, 20);
    let win = t.window_of(leaf);
    assert_eq!(
        frame_minheight(t.frame(leaf), NextCurwin::Win(win), heights(10, 1)),
        11
    );
}

#[test]
fn another_window_being_next_curwin_changes_nothing() {
    let mut t = Tree::new();
    let leaf = t.leaf(PLAIN, 5, 20);
    let other = t.leaf(PLAIN, 5, 20);
    let win = t.window_of(other);
    assert_eq!(
        frame_minheight(t.frame(leaf), NextCurwin::Win(win), heights(10, 1)),
        1
    );
}

#[test]
fn the_current_window_keeps_one_line_when_winminheight_is_zero() {
    // The `NULL` arm, and only it: `p_wmh == 0 && fr_win == curwin`.
    let mut t = Tree::new();
    let leaf = t.leaf(PLAIN, 5, 20);
    let win = t.window_of(leaf);
    let opts = MinSize {
        wanted: 10,
        minimum: 0,
        curwin: win,
    };
    assert_eq!(frame_minheight(t.frame(leaf), NextCurwin::Unset, opts), 1);
}

#[test]
fn nowin_makes_no_such_reservation() {
    // The trap `winsweep`'s `s3minx` exists for: `win_equal()` passes NOWIN,
    // not NULL, and only the NULL path bumps the current window.
    let mut t = Tree::new();
    let leaf = t.leaf(PLAIN, 5, 20);
    let win = t.window_of(leaf);
    let opts = MinSize {
        wanted: 10,
        minimum: 0,
        curwin: win,
    };
    assert_eq!(frame_minheight(t.frame(leaf), NextCurwin::NoWin, opts), 0);
}

#[test]
fn a_nonzero_winminheight_makes_no_reservation_either() {
    let mut t = Tree::new();
    let leaf = t.leaf(PLAIN, 5, 20);
    let win = t.window_of(leaf);
    let opts = MinSize {
        wanted: 10,
        minimum: 1,
        curwin: win,
    };
    assert_eq!(frame_minheight(t.frame(leaf), NextCurwin::Unset, opts), 1);
}

#[test]
fn a_row_costs_the_tallest_of_its_frames() {
    let mut t = Tree::new();
    let plain = t.leaf(PLAIN, 5, 10);
    let tall = t.leaf(
        Chrome {
            winbar: 1,
            hsep: 1,
            status: 1,
            vsep: 0,
        },
        5,
        10,
    );
    let row = t.branch(FR_ROW, &[plain, tall]);
    assert_eq!(
        frame_minheight(t.frame(row), NextCurwin::NoWin, heights(10, 2)),
        5
    );
}

#[test]
fn a_column_costs_the_sum_of_its_frames() {
    let mut t = Tree::new();
    let a = t.leaf(Chrome { status: 1, ..PLAIN }, 5, 10);
    let b = t.leaf(Chrome { status: 1, ..PLAIN }, 5, 10);
    let c = t.leaf(PLAIN, 5, 10);
    let col = t.branch(FR_COL, &[a, b, c]);
    assert_eq!(
        frame_minheight(t.frame(col), NextCurwin::NoWin, heights(10, 1)),
        2 + 2 + 1
    );
}

#[test]
fn an_empty_row_and_column_both_cost_nothing() {
    let mut t = Tree::new();
    let row = t.zeroed_frame();
    // SAFETY: a node of `t`, with no children.
    unsafe { (*row).fr_layout = FR_ROW };
    assert_eq!(
        frame_minheight(t.frame(row), NextCurwin::NoWin, heights(10, 1)),
        0
    );
}

#[test]
fn a_column_of_rows_adds_the_tallest_of_each() {
    let mut t = Tree::new();
    let a = t.leaf(Chrome { status: 1, ..PLAIN }, 5, 5);
    let b = t.leaf(PLAIN, 5, 5);
    let top = t.branch(FR_ROW, &[a, b]);
    let c = t.leaf(
        Chrome {
            winbar: 1,
            status: 1,
            ..PLAIN
        },
        5,
        10,
    );
    let col = t.branch(FR_COL, &[top, c]);
    // Row costs max(2, 1) = 2; the leaf below costs 1 + 1 + 1 = 3.
    assert_eq!(
        frame_minheight(t.frame(col), NextCurwin::NoWin, heights(10, 1)),
        5
    );
}

#[test]
fn an_absurd_winheight_saturates_rather_than_wrapping() {
    // `:set winheight=2147483647` reaches `p_wh + extra_height`, which the C
    // lets overflow.
    let mut t = Tree::new();
    let leaf = t.leaf(Chrome { status: 1, ..PLAIN }, 5, 20);
    let win = t.window_of(leaf);
    let opts = heights(c_int::MAX, 1);
    assert_eq!(
        frame_minheight(t.frame(leaf), NextCurwin::Win(win), opts),
        c_int::MAX
    );
}

#[test]
fn a_column_of_absurd_frames_saturates_too() {
    let mut t = Tree::new();
    let a = t.leaf(PLAIN, 5, 10);
    let b = t.leaf(PLAIN, 5, 10);
    let col = t.branch(FR_COL, &[a, b]);
    let opts = heights(10, c_int::MAX);
    assert_eq!(
        frame_minheight(t.frame(col), NextCurwin::NoWin, opts),
        c_int::MAX
    );
}

// ------------------------------------------------------------- frame_minwidth
//
// `frame_minwidth()` is the same with the axes exchanged: a *column* takes the
// widest child and a *row* the sum, and the separator column stands in for the
// status line.

#[test]
fn a_leaf_costs_winminwidth_plus_its_separator() {
    let mut t = Tree::new();
    let leaf = t.leaf(Chrome { vsep: 1, ..PLAIN }, 5, 20);
    assert_eq!(
        frame_minwidth(t.frame(leaf), NextCurwin::NoWin, heights(20, 1)),
        2
    );
}

#[test]
fn the_next_curwin_leaf_costs_winwidth() {
    let mut t = Tree::new();
    let leaf = t.leaf(Chrome { vsep: 1, ..PLAIN }, 5, 20);
    let win = t.window_of(leaf);
    assert_eq!(
        frame_minwidth(t.frame(leaf), NextCurwin::Win(win), heights(20, 1)),
        21
    );
}

#[test]
fn the_current_window_keeps_one_column_when_winminwidth_is_zero() {
    let mut t = Tree::new();
    let leaf = t.leaf(PLAIN, 5, 20);
    let win = t.window_of(leaf);
    let opts = MinSize {
        wanted: 20,
        minimum: 0,
        curwin: win,
    };
    assert_eq!(frame_minwidth(t.frame(leaf), NextCurwin::Unset, opts), 1);
    assert_eq!(frame_minwidth(t.frame(leaf), NextCurwin::NoWin, opts), 0);
}

#[test]
fn a_column_costs_the_widest_of_its_frames() {
    let mut t = Tree::new();
    let a = t.leaf(PLAIN, 5, 10);
    let b = t.leaf(Chrome { vsep: 1, ..PLAIN }, 5, 10);
    let col = t.branch(FR_COL, &[a, b]);
    assert_eq!(
        frame_minwidth(t.frame(col), NextCurwin::NoWin, heights(20, 1)),
        2
    );
}

#[test]
fn a_row_costs_the_sum_of_its_frames() {
    let mut t = Tree::new();
    let a = t.leaf(Chrome { vsep: 1, ..PLAIN }, 5, 10);
    let b = t.leaf(Chrome { vsep: 1, ..PLAIN }, 5, 10);
    let c = t.leaf(PLAIN, 5, 10);
    let row = t.branch(FR_ROW, &[a, b, c]);
    assert_eq!(
        frame_minwidth(t.frame(row), NextCurwin::NoWin, heights(20, 1)),
        2 + 2 + 1
    );
}

// --------------------------------------------------------- frame_check_height
//
// `frame_check_height()`:
//
//   if (topfrp->fr_height != height) return false;
//   if (topfrp->fr_layout == FR_ROW)
//     FOR_ALL_FRAMES(frp, topfrp->fr_child)
//       if (frp->fr_height != height) return false;
//   return true;

#[test]
fn a_leaf_at_the_right_height_checks_out() {
    let mut t = Tree::new();
    let leaf = t.leaf(PLAIN, 7, 20);
    assert!(frame_check_height(t.frame(leaf), 7));
    assert!(!frame_check_height(t.frame(leaf), 6));
}

#[test]
fn a_rows_children_must_all_be_at_the_rows_height() {
    let mut t = Tree::new();
    let a = t.leaf(PLAIN, 7, 10);
    let b = t.leaf(PLAIN, 7, 10);
    let row = t.branch(FR_ROW, &[a, b]);
    t.set_size(row, 7, 20);
    assert!(frame_check_height(t.frame(row), 7));
    t.set_size(b, 6, 10);
    assert!(!frame_check_height(t.frame(row), 7));
}

#[test]
fn a_columns_children_are_not_checked_at_all() {
    // Only a *row* shares its height with its children; a column's are
    // expected to differ, and upstream does not look at them.
    let mut t = Tree::new();
    let a = t.leaf(PLAIN, 3, 10);
    let b = t.leaf(PLAIN, 4, 10);
    let col = t.branch(FR_COL, &[a, b]);
    t.set_size(col, 7, 10);
    assert!(frame_check_height(t.frame(col), 7));
}

#[test]
fn a_columns_children_must_all_be_at_the_columns_width() {
    let mut t = Tree::new();
    let a = t.leaf(PLAIN, 3, 10);
    let b = t.leaf(PLAIN, 4, 10);
    let col = t.branch(FR_COL, &[a, b]);
    t.set_size(col, 7, 10);
    assert!(frame_check_width(t.frame(col), 10));
    t.set_size(b, 4, 9);
    assert!(!frame_check_width(t.frame(col), 10));
}

#[test]
fn a_rows_children_are_not_width_checked() {
    let mut t = Tree::new();
    let a = t.leaf(PLAIN, 7, 10);
    let b = t.leaf(PLAIN, 7, 11);
    let row = t.branch(FR_ROW, &[a, b]);
    t.set_size(row, 7, 21);
    assert!(frame_check_width(t.frame(row), 21));
}

// -------------------------------------------------------- the chrome additions
//
// `win_setheight_win()` adds the rows below the window to the height it was
// asked for, and `win_setwidth_win()` the separator column. `:resize
// 2147483647` reaches both, and the C's overflow wraps the tallest window
// anyone can ask for into the shortest.

#[test]
fn an_ordinary_height_gains_its_chrome() {
    assert_eq!(height_with_chrome(10, 0, 1), 11);
    assert_eq!(height_with_chrome(10, 1, 1), 12);
    assert_eq!(height_with_chrome(10, 0, 0), 10);
}

#[test]
fn an_absurd_height_saturates_instead_of_wrapping() {
    assert_eq!(height_with_chrome(c_int::MAX, 0, 1), c_int::MAX);
    assert_eq!(height_with_chrome(c_int::MAX, 1, 1), c_int::MAX);
}

#[test]
fn a_negative_height_keeps_its_sign() {
    // `:resize -2147483647` is not clamped away either; `frame_setheight()`
    // takes it as "as small as possible".
    assert_eq!(height_with_chrome(-2147483647, 0, 1), -2147483646);
    assert_eq!(height_with_chrome(c_int::MIN, 0, 0), c_int::MIN);
}

#[test]
fn a_width_gains_its_separator_and_saturates() {
    assert_eq!(width_with_chrome(10, 1), 11);
    assert_eq!(width_with_chrome(10, 0), 10);
    assert_eq!(width_with_chrome(c_int::MAX, 1), c_int::MAX);
}

// ------------------------------------------------------------- parent_target
//
// `frame_setheight()`'s first run, when the siblings cannot pay:
//
//   frame_setheight(curfrp->fr_parent,
//                   height + frame_minheight(curfrp->fr_parent, NOWIN) - p_wmh - 1);

#[test]
fn the_parent_is_asked_for_the_size_plus_its_own_minimum() {
    assert_eq!(parent_target(20, 6, 1), 24);
    assert_eq!(parent_target(20, 6, 0), 25);
}

#[test]
fn an_absurd_request_saturates_on_the_way_up() {
    assert_eq!(parent_target(c_int::MAX, 6, 1), c_int::MAX - 2);
    assert_eq!(parent_target(c_int::MAX, c_int::MAX, 1), c_int::MAX - 2);
}

#[test]
fn a_very_negative_request_saturates_downwards() {
    assert_eq!(parent_target(c_int::MIN, 0, 0), c_int::MIN);
}

// ----------------------------------------------------------- the cursor's row
//
// `set_fraction()`: wp->w_fraction = (w_wrow * FRACTION_MULT + FRACTION_MULT/2)
//                                    / w_view_height
// `scroll_to_fraction()`: wp->w_wrow = (w_fraction * height - 1) / FRACTION_MULT

const MULT: c_int = 16384;

#[test]
fn a_cursor_on_the_first_line_counts_as_halfway_down_it() {
    // Two lines: 25%. Three: about 17%.
    assert_eq!(cursor_fraction(0, 2), MULT / 4);
    assert_eq!(cursor_fraction(0, 3), MULT / 6);
}

#[test]
fn a_cursor_on_the_last_line_counts_as_halfway_down_that() {
    // Two lines: 75%. Three: about 83%.
    assert_eq!(cursor_fraction(1, 2), 3 * MULT / 4);
    assert_eq!(cursor_fraction(2, 3), 5 * MULT / 6);
}

#[test]
fn the_fraction_of_a_middle_row_is_its_share_of_the_height() {
    assert_eq!(cursor_fraction(5, 10), MULT / 2 + MULT / 20);
}

#[test]
fn the_row_a_fraction_names_is_its_share_of_the_new_height() {
    // A cursor halfway down a ten-row window lands on row five of a ten-row
    // one, and on row two of a five-row one.
    assert_eq!(fraction_row(MULT / 2, 10), 4);
    assert_eq!(fraction_row(MULT / 2, 20), 9);
    // Integer division truncates towards zero, so `(0 * height - 1) / MULT` is
    // zero rather than -1.
    assert_eq!(fraction_row(0, 20), 0);
    assert_eq!(fraction_row(MULT, 10), 9);
}

#[test]
fn a_full_fraction_lands_on_the_last_row_of_any_height() {
    for height in 1..40 {
        assert_eq!(fraction_row(MULT, height), height - 1);
    }
}

#[test]
fn the_two_are_each_others_inverse_to_within_a_row() {
    for height in 2..40 {
        for row in 0..height {
            let back = fraction_row(cursor_fraction(row, height), height);
            assert!(
                (back - row).abs() <= 1,
                "row {row} of {height} came back as {back}"
            );
        }
    }
}

// -------------------------------------------------------------- sort_columns
//
// `check_colorcolumn()` sorts the parsed columns with `qsort(..., int_cmp)`
// before dropping the duplicates.

#[test]
fn columns_come_back_ascending() {
    let mut cols = [30, 10, 20, 0, -1];
    sort_columns(&mut cols);
    assert_eq!(cols, [-1, 0, 10, 20, 30]);
}

#[test]
fn duplicates_survive_the_sort_next_to_each_other() {
    // `check_colorcolumn()` drops them afterwards, and only trusts the sort to
    // have put them together.
    let mut cols = [20, 10, 20, 10, 20];
    sort_columns(&mut cols);
    assert_eq!(cols, [10, 10, 20, 20, 20]);
}

#[test]
fn an_empty_or_single_column_list_is_left_alone() {
    let mut none: [c_int; 0] = [];
    sort_columns(&mut none);
    let mut one = [7];
    sort_columns(&mut one);
    assert_eq!(one, [7]);
}
