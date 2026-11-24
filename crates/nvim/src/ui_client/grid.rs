//! The grid events: the screen's size, its lines, scrolls, clears and
//! the cursor.
//!
//! Each carries rows, columns and attribute ids the TUI indexes its shadow
//! grid and attribute table with. The server is not necessarily this build
//! (`--remote-ui` connects to whatever listens), so each is checked against
//! the grid the TUI holds: out of range is a bad event, logged and skipped,
//! except a cursor position or a clear's end, which are clamped into the
//! grid. Upstream checks none of them.

#![forbid(unsafe_code)]

use super::{bad_event, take_arg, tui_state};
use crate::drawscreen::{MAX_COLUMNS, MAX_ROWS};
use crate::grid::defs::GridLineEvent;
use crate::msgpack_rpc::unpacker::Unpacker;
use crate::narrow::number_as_int;
use crate::tui::paint::{
    tui_grid_clear, tui_grid_cursor_goto, tui_grid_resize, tui_grid_scroll, tui_raw_line,
};
use crate::tui::state::TUIData;
use crate::types::{Array, Integer, OBJECT_TYPE_INTEGER, ScreenAttr};
use crate::ui::state::LINE_FLAG_WRAP;
use core::ffi::c_int;

/// The narrowest grid the client accepts: a wrapped row's last cell may be
/// the second half of a double-width one, which the TUI reprints from the
/// cell before it.
const MIN_GRID_WIDTH: Integer = 2;

/// The extent of the TUI's shadow grid and attribute table, as the
/// `Integer`s the events carry, so every check is plain arithmetic.
#[derive(Debug)]
struct GridBounds {
    /// Zero, both, until the first `grid_resize`.
    width: Integer,
    height: Integer,
    /// How many attribute ids are defined (`0..attrs`).
    attrs: Integer,
}

impl GridBounds {
    fn of(t: &TUIData) -> Self {
        let sized = t.grid.cells.is_some();
        let extent = |n: c_int| if sized { Integer::from(n) } else { 0 };
        GridBounds {
            width: extent(t.grid.width),
            height: extent(t.grid.height),
            attrs: Integer::try_from(t.attrs.len()).unwrap_or(Integer::MAX),
        }
    }

    fn holds_attr(&self, id: Integer) -> bool {
        (0..self.attrs).contains(&id)
    }

    /// `(row, col)` moved to the nearest cell of the grid (or the origin
    /// before there is one). The TUI reads the cells around the cursor when
    /// choosing how to move it, so it must not stand outside them.
    fn clamp_cursor(&self, row: Integer, col: Integer) -> (c_int, c_int) {
        let row = row.clamp(0, (self.height - 1).max(0));
        let col = col.clamp(0, (self.width - 1).max(0));
        // Both are within a grid of at most `MAX_ROWS` x `MAX_COLUMNS`.
        (number_as_int(row), number_as_int(col))
    }

    /// Does scrolling `rows` rows of `[top, bot) x [left, right)` stay
    /// inside the grid? A distance past the region's height would copy rows
    /// from outside it.
    fn holds_scroll(
        &self,
        top: Integer,
        bot: Integer,
        left: Integer,
        right: Integer,
        rows: Integer,
    ) -> bool {
        0 <= top
            && top < bot
            && bot <= self.height
            && 0 <= left
            && left < right
            && right <= self.width
            && rows.unsigned_abs() <= (bot - top).unsigned_abs()
    }

    /// Does a `grid_line` painting `attrs` from `startcol` on `row`, and
    /// clearing to `clearcol` in `clearattr`, stay inside the grid and name
    /// only defined attributes? Answers the clear's end, clamped to the
    /// row's: a clear past it is a clear to the end of the row.
    fn holds_line(
        &self,
        row: Integer,
        startcol: Integer,
        attrs: &[ScreenAttr],
        clearcol: Integer,
        clearattr: Integer,
    ) -> Option<Integer> {
        let endcol = startcol.checked_add(Integer::try_from(attrs.len()).ok()?)?;
        let clearcol = clearcol.min(self.width);
        let fits = (0..self.height).contains(&row)
            && 0 <= startcol
            && endcol <= self.width
            && attrs.iter().all(|&id| self.holds_attr(id.into()))
            && (clearcol <= endcol || self.holds_attr(clearattr));
        fits.then_some(clearcol)
    }
}

/// Resizes a grid, and the buffers a `grid_line` is decoded into. A size
/// past the largest screen a server keeps, or too narrow for the TUI to
/// paint, is refused.
pub(crate) fn ui_client_event_grid_resize(mut args: Array) {
    let (Some(grid), Some(width), Some(height)) = (
        take_arg(&mut args, 0, tag!(Integer)),
        take_arg(&mut args, 1, tag!(Integer)),
        take_arg(&mut args, 2, tag!(Integer)),
    ) else {
        return bad_event(c"grid_resize", c"ui_client_event_grid_resize");
    };
    let (grid, width, height) = (
        payload!(Integer, grid),
        payload!(Integer, width),
        payload!(Integer, height),
    );
    let fits = (MIN_GRID_WIDTH..=Integer::from(MAX_COLUMNS)).contains(&width)
        && (1..=Integer::from(MAX_ROWS)).contains(&height);
    if !fits {
        return bad_event(c"grid_resize", c"ui_client_event_grid_resize");
    }
    tui_state(|t| {
        tui_grid_resize(t, grid, width, height);
        // A cursor the server placed before the resize may now be outside;
        // the next flush moves to it.
        let (row, col) = (Integer::from(t.row), Integer::from(t.col));
        (t.row, t.col) = GridBounds::of(t).clamp_cursor(row, col);
    });

    // The decoder writes cells straight into these rather than
    // building an array, so they have to hold the widest grid.
    Unpacker::widen_grid_line_buf(usize::try_from(width).unwrap_or(0));
}

/// Blanks the grid; there must be one.
pub(crate) fn ui_client_event_grid_clear(mut args: Array) {
    let Some(grid) = take_arg(&mut args, 0, tag!(Integer)) else {
        return bad_event(c"grid_clear", c"ui_client_event_grid_clear");
    };
    let grid = payload!(Integer, grid);
    let cleared = tui_state(|t| {
        let sized = t.grid.cells.is_some();
        if sized {
            tui_grid_clear(t, grid);
        }
        sized
    });
    if !cleared {
        bad_event(c"grid_clear", c"ui_client_event_grid_clear");
    }
}

/// Where the cursor goes at the next flush, clamped into the grid.
pub(crate) fn ui_client_event_grid_cursor_goto(mut args: Array) {
    let (Some(grid), Some(row), Some(col)) = (
        take_arg(&mut args, 0, tag!(Integer)),
        take_arg(&mut args, 1, tag!(Integer)),
        take_arg(&mut args, 2, tag!(Integer)),
    ) else {
        return bad_event(c"grid_cursor_goto", c"ui_client_event_grid_cursor_goto");
    };
    let (grid, row, col) = (
        payload!(Integer, grid),
        payload!(Integer, row),
        payload!(Integer, col),
    );
    tui_state(|t| {
        let (row, col) = GridBounds::of(t).clamp_cursor(row, col);
        tui_grid_cursor_goto(t, grid, row.into(), col.into());
    });
}

/// Scrolls a region of the grid; a region or distance outside it is
/// refused.
pub(crate) fn ui_client_event_grid_scroll(mut args: Array) {
    let mut numbers = [0; 7];
    for (index, number) in numbers.iter_mut().enumerate() {
        let Some(arg) = take_arg(&mut args, index, tag!(Integer)) else {
            return bad_event(c"grid_scroll", c"ui_client_event_grid_scroll");
        };
        *number = payload!(Integer, arg);
    }
    let [grid, top, bot, left, right, rows, cols] = numbers;
    let scrolled = tui_state(|t| {
        let fits = GridBounds::of(t).holds_scroll(top, bot, left, right, rows);
        if fits {
            tui_grid_scroll(t, grid, top, bot, left, right, rows, cols);
        }
        fits
    });
    if !scrolled {
        bad_event(c"grid_scroll", c"ui_client_event_grid_scroll");
    }
}

/// Never called: the decoder recognises `grid_line` by this function's
/// address and decodes the cells itself, ending at
/// [`ui_client_event_raw_line`].
pub(crate) fn ui_client_event_grid_line(_args: Array) {
    unreachable!("grid_line is decoded by the unpacker, not dispatched");
}

/// Paints the cells the decoder wrote into the shared buffers. A row, a
/// column run or an attribute id outside what the TUI holds is a bad event.
pub(crate) fn ui_client_event_raw_line(g: &GridLineEvent) {
    let [grid, row, startcol] = g.args.map(Integer::from);
    // In `Integer`: the decoder's numbers are a server's, and a start
    // column near `INT_MAX` plus the run would overflow an `int`.
    let endcol = startcol + Integer::from(g.coloff);
    let clearcol = endcol + Integer::from(g.clear_width);
    let flags = if g.wrap { LINE_FLAG_WRAP } else { 0 };
    let attr = Integer::from(g.cur_attr);
    let painted = Unpacker::with_grid_line_cells(|chars, attrs, width| {
        // The decoder only counts a cell it wrote, which bounds the run the
        // TUI reads; an event that claims more is not drawn.
        let Some(cells) = usize::try_from(g.coloff).ok().filter(|&n| n <= width) else {
            return false;
        };
        let (chars, attrs) = (&chars[..cells], &attrs[..cells]);
        tui_state(|t| {
            let bounds = GridBounds::of(t);
            let Some(clearcol) = bounds.holds_line(row, startcol, attrs, clearcol, attr) else {
                return false;
            };
            tui_raw_line(
                t, grid, row, startcol, endcol, clearcol, attr, flags, chars, attrs,
            );
            true
        })
    });
    if !painted {
        bad_event(c"grid_line", c"ui_client_event_raw_line");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;
    use crate::types::Object;

    /// A grid size past the largest screen a server keeps, or too narrow
    /// for the TUI to paint, is refused before it reaches the TUI (none is
    /// started here, so one that got through would panic in `tui_state`).
    #[test]
    fn a_grid_size_out_of_range_is_a_bad_event() {
        let _held = editor_state_lock();
        let (cols, rows) = (Integer::from(MAX_COLUMNS), Integer::from(MAX_ROWS));
        let sizes = [
            (-1, 10),
            (0, 10),
            (1, 10),
            (cols + 1, 10),
            (Integer::MAX, 10),
            (80, 0),
            (80, -1),
            (80, rows + 1),
            (80, Integer::MIN),
        ];
        for (width, height) in sizes {
            ui_client_event_grid_resize(Array::from(vec![
                Object::Integer(1),
                Object::Integer(width),
                Object::Integer(height),
            ]));
        }
    }

    /// An 80x24 grid with three attributes defined.
    const BOUNDS: GridBounds = GridBounds {
        width: 80,
        height: 24,
        attrs: 3,
    };

    /// Before the first resize there is no grid at all.
    const UNSIZED: GridBounds = GridBounds {
        width: 0,
        height: 0,
        attrs: 1,
    };

    #[test]
    fn a_cursor_outside_the_grid_is_clamped_into_it() {
        assert_eq!(BOUNDS.clamp_cursor(5, 7), (5, 7));
        assert_eq!(BOUNDS.clamp_cursor(-1, Integer::MIN), (0, 0));
        assert_eq!(BOUNDS.clamp_cursor(24, 80), (23, 79));
        // Past `INT_MAX`: clamped, not wrapped to a small number first.
        assert_eq!(BOUNDS.clamp_cursor(Integer::MAX, 1 << 32), (23, 79));
        assert_eq!(UNSIZED.clamp_cursor(3, 3), (0, 0));
    }

    #[test]
    fn a_scroll_outside_the_grid_is_refused() {
        assert!(BOUNDS.holds_scroll(0, 24, 0, 80, 1));
        assert!(BOUNDS.holds_scroll(2, 10, 5, 20, -8));
        let refused = [
            (-1, 24, 0, 80, 1),
            (0, 25, 0, 80, 1),
            (5, 5, 0, 80, 0),
            (0, 24, -1, 80, 1),
            (0, 24, 0, 81, 1),
            (0, 24, 10, 10, 1),
            (0, 10, 0, 80, 11),
            (0, 10, 0, 80, -11),
            (0, 24, 0, 80, Integer::MIN),
            (Integer::MIN, Integer::MAX, 0, 80, 1),
        ];
        for (top, bot, left, right, rows) in refused {
            let region = (top, bot, left, right, rows);
            assert!(
                !BOUNDS.holds_scroll(top, bot, left, right, rows),
                "{region:?}"
            );
        }
        assert!(!UNSIZED.holds_scroll(0, 1, 0, 1, 0));
    }

    #[test]
    fn a_grid_line_outside_the_grid_is_refused() {
        assert_eq!(BOUNDS.holds_line(0, 0, &[0, 1, 2], 3, 0), Some(3));
        // A clear past the row is a clear to its end.
        assert_eq!(BOUNDS.holds_line(23, 70, &[1], Integer::MAX, 2), Some(80));
        // No clear, so its attribute is never used and not checked.
        assert_eq!(BOUNDS.holds_line(0, 78, &[0, 0], 80, -1), Some(80));
        let refused: [(Integer, Integer, &[ScreenAttr], Integer, Integer); 8] = [
            (-1, 0, &[0], 1, 0),
            (24, 0, &[0], 1, 0),
            (0, -1, &[0], 0, 0),
            (0, 79, &[0, 0], 81, 0),
            (0, 0, &[-1], 1, 0),
            (0, 0, &[3], 1, 0),
            (0, 0, &[0], 5, -1),
            (0, Integer::MAX, &[0], 0, 0),
        ];
        for (row, startcol, attrs, clearcol, clearattr) in refused {
            let line = (row, startcol, attrs, clearcol, clearattr);
            assert_eq!(
                BOUNDS.holds_line(row, startcol, attrs, clearcol, clearattr),
                None,
                "{line:?}"
            );
        }
        assert_eq!(UNSIZED.holds_line(0, 0, &[], 0, 0), None);
    }
}
