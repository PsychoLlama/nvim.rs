//! The screen: the grid of cells the terminal state paints onto.
//!
//! The state reports what changed — a glyph here, a scroll there — and the
//! screen keeps the resulting cells, batches the damage at whatever
//! granularity the host asked for, and hands cells back on demand. This
//! module holds the cell grids, the damage bookkeeping and the interface the
//! state and the host use; the state's reports arrive in
//! [`callbacks`](self::callbacks), and the resize, which is where the
//! interesting work is, lives in [`resize`](self::resize).
//!
//! The host's callbacks may run anything, but they reach only the host: the
//! screen calls them with itself borrowed, and nothing a host is handed leads
//! back into the emulator.
//!
//! Ported from libvterm, Copyright (c) 2008 Paul Evans, under the MIT
//! license; the notice is reproduced in licenses/libvterm-LICENSE.txt.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

mod callbacks;
pub mod resize;

use crate::narrow::as_index;
use crate::narrow::count_as_int;
use core::ffi::c_int;

use crate::types::{
    ScreenCell, ScreenPen, VTerm, VTermDamageSize, VTermLineInfo, VTermPos, VTermRect, VTermScreen,
    VTermScreenCell, VTermScreenCellAttrs, VTermState,
};
use crate::vterm::cell::{SCHAR_CONTINUATION, blank_cells, erased_pen, export_pen};
use crate::vterm::color::UNSET;
use crate::vterm::damage::{Damage, NO_RECT, follow_scroll, intersects, merge_damage};
use crate::vterm::host::ScreenHost;
use crate::vterm::vterm::{ScrollTarget, VTERM_DAMAGE_CELL, VTERM_DAMAGE_SCROLL, scroll_rect};

/// Which of the screen's two cell buffers a call means: the one the child
/// normally paints into, or the alternate one `VTERM_PROP_ALTSCREEN` swaps
/// in.
pub const BUFIDX_PRIMARY: usize = 0;
pub const BUFIDX_ALTSCREEN: usize = 1;

/// A cell with every byte zero, as libvterm's zeroed allocations started.
const ZERO_CELL: ScreenCell = ScreenCell {
    schar: 0,
    pen: erased_pen(UNSET, UNSET),
};

/// A reported cell with every byte zero.
pub(crate) const ZERO_SCREEN_CELL: VTermScreenCell = VTermScreenCell {
    schar: 0,
    width: 0,
    attrs: VTermScreenCellAttrs {
        bold_underline_italic_blink_reverse_conceal_strike_font_dwl_dhl_small_baseline_dim_overline:
            [0; 3],
        _pad: [0; 1],
    },
    fg: UNSET,
    bg: UNSET,
    uri: 0,
};

/// What a screen owns that C does not read.
pub(crate) struct ScreenOwned {
    /// The cells, row-major, `rows * cols` of them: the primary screen's at
    /// [`BUFIDX_PRIMARY`], the alternate screen's at [`BUFIDX_ALTSCREEN`]
    /// (empty until enabled).
    buffers: [Vec<ScreenCell>; 2],
    /// Whether the alternate screen has been enabled.
    has_altscreen: bool,
    /// Which of `buffers` is showing.
    active: usize,
    /// One row of reported cells, carried to and from the host's scrollback.
    sb_buffer: Vec<VTermScreenCell>,
    pub(crate) host: Option<Box<dyn ScreenHost>>,
}

impl VTermScreen {
    /// A screen `rows` by `cols` for the terminal `vt`, whose state machine is
    /// `state`; both are kept only for a C reader.
    pub(crate) fn new(rows: c_int, cols: c_int, vt: *mut VTerm, state: *mut VTermState) -> Self {
        let mut screen = VTermScreen {
            vt,
            state,
            callbacks: core::ptr::null(),
            cbdata: core::ptr::null_mut(),
            damage_merge: VTERM_DAMAGE_CELL,
            damaged: VTermRect {
                start_row: NO_RECT,
                end_row: 0,
                start_col: 0,
                end_col: 0,
            },
            pending_scrollrect: VTermRect {
                start_row: NO_RECT,
                end_row: 0,
                start_col: 0,
                end_col: 0,
            },
            pending_scroll_downward: 0,
            pending_scroll_rightward: 0,
            rows,
            cols,
            global_reverse_reflow: [0; 1],
            _pad: [0; 3],
            owned: Box::new(ScreenOwned {
                buffers: [Vec::new(), Vec::new()],
                has_altscreen: false,
                active: BUFIDX_PRIMARY,
                sb_buffer: Vec::new(),
                host: None,
            }),
            pen: ZERO_CELL.pen,
        };
        screen.owned.buffers[BUFIDX_PRIMARY] = new_buffer(screen.pen, rows, cols);
        screen.realloc_sb_buffer(cols);
        screen
    }

    // ---------------------------------------------------------- the cells

    /// The cells of the buffer showing.
    fn grid(&self) -> &[ScreenCell] {
        &self.owned.buffers[self.owned.active]
    }

    fn grid_mut(&mut self) -> &mut [ScreenCell] {
        let owned = &mut *self.owned;
        &mut owned.buffers[owned.active]
    }

    /// Where the cell at `row`/`col` is in a grid, or `None` outside it.
    fn index(&self, row: c_int, col: c_int) -> Option<usize> {
        if row < 0 || row >= self.rows || col < 0 || col >= self.cols {
            return None;
        }
        Some(as_index(self.cols * row + col))
    }

    /// The cell at `row`/`col`, to write through; `None` outside the grid.
    pub(crate) fn cell_mut(&mut self, row: c_int, col: c_int) -> Option<&mut ScreenCell> {
        let at = self.index(row, col)?;
        self.grid_mut().get_mut(at)
    }

    /// Whether the buffer showing is the primary one.
    fn on_primary(&self) -> bool {
        self.owned.active == BUFIDX_PRIMARY
    }

    // ------------------------------------------------- the host's callbacks

    /// Installs the host's callbacks.
    pub(crate) fn set_host(&mut self, host: Option<Box<dyn ScreenHost>>) {
        self.owned.host = host;
    }

    fn host(&mut self) -> Option<&mut (dyn ScreenHost + 'static)> {
        self.owned.host.as_deref_mut()
    }

    /// Tells the host `rect` has to be redrawn.
    fn report_damage(&mut self, rect: VTermRect) {
        if let Some(host) = self.host() {
            let _ = host.damage(rect);
        }
    }

    /// Offers the host a move it may be able to perform itself. False if it
    /// did not, and the destination has to be reported as damaged instead.
    fn report_moverect(&mut self, dest: VTermRect, src: VTermRect) -> bool {
        if !self
            .owned
            .host
            .as_ref()
            .is_some_and(|host| host.takes_moves())
        {
            return false;
        }
        // Flushing under scroll merging would recurse back into here.
        if self.damage_merge != VTERM_DAMAGE_SCROLL {
            self.flush_damage();
        }
        self.host()
            .and_then(|host| host.moverect(dest, src))
            .is_some_and(|moved| moved != 0)
    }

    /// Whether the host takes rows scrolled off the top.
    fn takes_scrollback(&self) -> bool {
        self.owned
            .host
            .as_ref()
            .is_some_and(|host| host.takes_scrollback())
    }

    // -------------------------------------------------------------- damage

    /// Records damage to `rect`, telling the host at once or holding it back,
    /// according to the merge level.
    fn damage(&mut self, rect: VTermRect) {
        let merge = self.damage_merge;
        let emit = match merge_damage(&mut self.damaged, rect, merge) {
            Damage::Pending => return,
            Damage::Emit(pending) => pending,
            Damage::FlushFirst(pending) => {
                self.flush_damage();
                pending
            }
        };
        self.report_damage(emit);
    }

    /// Records damage to every cell.
    fn damage_screen(&mut self) {
        let whole = VTermRect {
            start_row: 0,
            end_row: self.rows,
            start_col: 0,
            end_col: self.cols,
        };
        self.damage(whole);
    }

    /// Hands the host everything held back by the merge level.
    pub(crate) fn flush_damage(&mut self) {
        if self.pending_scrollrect.start_row != NO_RECT {
            let (region, downward) = (self.pending_scrollrect, self.pending_scroll_downward);
            let rightward = self.pending_scroll_rightward;
            scroll_rect(region, downward, rightward, &mut ReportMoves(self));
            self.pending_scrollrect.start_row = NO_RECT;
        }
        if self.damaged.start_row != NO_RECT {
            let damaged = self.damaged;
            self.report_damage(damaged);
            self.damaged.start_row = NO_RECT;
        }
    }

    /// Forgets the damage held back, as a reset does before repainting.
    pub(crate) fn discard_damage(&mut self) {
        self.damaged.start_row = NO_RECT;
        self.pending_scrollrect.start_row = NO_RECT;
    }

    /// Changes how damage is batched, handing over what the old level held
    /// back first.
    pub(crate) fn set_damage_merge(&mut self, size: VTermDamageSize) {
        self.flush_damage();
        self.damage_merge = size;
    }

    // ------------------------------------------- painting, moving, erasing

    /// Copies the screen's row `row` into the scrollback staging buffer and
    /// hands it over to the host.
    fn push_line(&mut self, row: c_int) {
        let cols = as_index(self.cols.max(0));
        let reverse = self.global_reverse() != 0;
        let start = self.index(row, 0);
        let ScreenOwned {
            buffers,
            active,
            sb_buffer,
            host,
            ..
        } = &mut *self.owned;
        let grid: &[ScreenCell] = &buffers[*active];
        // A row outside the grid exports nothing, leaving the staging
        // buffer as it was.
        if let Some(start) = start {
            let line = &grid[start..start + cols];
            for (col, out) in sb_buffer[..cols].iter_mut().enumerate() {
                export_at(line, col, reverse, out);
            }
        }
        if let Some(host) = host {
            let _ = host.sb_pushline(&sb_buffer[..cols]);
        }
    }

    /// Moves cells within the grid. Rows scrolled off the top of the primary
    /// buffer go out to scrollback on the way.
    fn move_cells(&mut self, dest: VTermRect, src: VTermRect) {
        let full_width_from_top = dest.start_row == 0
            && dest.start_col == 0
            && dest.end_col == self.cols
            && self.on_primary();
        if full_width_from_top && self.takes_scrollback() {
            for row in 0..src.start_row {
                self.push_line(row);
            }
        }
        let width = as_index(src.end_col - src.start_col);
        let downward = src.start_row - dest.start_row;
        // Overlapping ranges: copy away from the direction of travel.
        let (mut row, limit, step) = if downward < 0 {
            (dest.end_row - 1, dest.start_row - 1, -1)
        } else {
            (dest.start_row, dest.end_row, 1)
        };
        while row != limit {
            let to = self
                .index(row, dest.start_col)
                .expect("a move lands inside the grid");
            let from = self
                .index(row + downward, src.start_col)
                .expect("a move starts inside the grid");
            self.grid_mut().copy_within(from..from + width, to);
            row += step;
        }
    }

    /// Blanks the cells in `rect`, keeping the screen's current colours. A
    /// selective erase spares cells the host marked protected. `lines` are the
    /// state's line marks, the first `state_rows` of which are live.
    fn erase_cells(
        &mut self,
        rect: VTermRect,
        selective: bool,
        state_rows: c_int,
        lines: &[VTermLineInfo],
    ) {
        let (fg, bg) = (self.pen.fg, self.pen.bg);
        // The columns of `rect` that are inside the grid; the rows are
        // checked one at a time below.
        let (first, last) = (rect.start_col.max(0), rect.end_col.min(self.cols));
        let mut row = rect.start_row;
        while row < state_rows && row < rect.end_row {
            let info = lines[as_index(row)];
            let (dwl, dhl) = (info.doublewidth(), info.doubleheight());
            if let Some(start) = self.index(row, first).filter(|_| first < last) {
                let width = as_index(last - first);
                for cell in &mut self.grid_mut()[start..start + width] {
                    if selective && cell.pen.protected_cell() != 0 {
                        continue;
                    }
                    cell.schar = 0;
                    cell.pen = erased_pen(fg, bg);
                    cell.pen.set_dwl(dwl);
                    cell.pen.set_dhl(dhl);
                }
            }
            row += 1;
        }
    }

    // ------------------------------------------------------- the interface

    /// The reported form of the cell at `pos` into `out`. Returns 0 for a
    /// position outside the screen, leaving `out` alone.
    pub(crate) fn export_cell(&self, pos: VTermPos, out: &mut VTermScreenCell) -> c_int {
        let Some(at) = self.index(pos.row, pos.col) else {
            return 0;
        };
        // The row runs to the end of the grid's row, so the cell after this
        // one is in it exactly when it is on the same row.
        let end = at + as_index(self.cols - pos.col);
        let line = &self.grid()[at..end];
        export_at(line, 0, self.global_reverse() != 0, out);
        1
    }

    /// Turns reflow on resize on or off.
    pub(crate) fn enable_reflow(&mut self, reflow: bool) {
        self.set_reflow(u32::from(reflow));
    }

    /// Allocates the alternate screen buffer, which the terminal starts
    /// without, at the terminal's size.
    pub(crate) fn enable_altscreen(&mut self, rows: c_int, cols: c_int) {
        if !self.owned.has_altscreen {
            self.owned.buffers[BUFIDX_ALTSCREEN] = new_buffer(self.pen, rows, cols);
            self.owned.has_altscreen = true;
        }
    }

    /// Resizes the one-row staging buffer that carries cells to and from the
    /// host's scrollback.
    fn realloc_sb_buffer(&mut self, cols: c_int) {
        self.owned.sb_buffer = vec![ZERO_SCREEN_CELL; as_index(cols.max(0))];
    }
}

/// One row of the screen, to read cells out of in their reported form
/// without finding the row again for each.
pub(crate) struct ScreenRow<'a> {
    line: &'a [ScreenCell],
    global_reverse: bool,
}

impl ScreenRow<'_> {
    /// The reported form of cell `col` into `out`; false, leaving `out`
    /// alone, outside the row.
    pub(crate) fn export(&self, col: c_int, out: &mut VTermScreenCell) -> bool {
        match usize::try_from(col) {
            Ok(col) if col < self.line.len() => {
                export_at(self.line, col, self.global_reverse, out);
                true
            }
            _ => false,
        }
    }
}

impl VTermScreen {
    /// Row `row`, or `None` outside the grid.
    pub(crate) fn row(&self, row: c_int) -> Option<ScreenRow<'_>> {
        let start = self.index(row, 0)?;
        Some(ScreenRow {
            line: &self.grid()[start..start + as_index(self.cols)],
            global_reverse: self.global_reverse() != 0,
        })
    }
}

/// The reported form of cell `col` of `line`, a whole row of the grid.
fn export_at(line: &[ScreenCell], col: usize, global_reverse: bool, out: &mut VTermScreenCell) {
    let cell = &line[col];
    // The gap behind a double-width glyph reports as empty.
    out.schar = if cell.schar == SCHAR_CONTINUATION {
        0
    } else {
        cell.schar
    };
    export_pen(&cell.pen, global_reverse, out);
    let followed_by_gap = line
        .get(col + 1)
        .is_some_and(|next| next.schar == SCHAR_CONTINUATION);
    out.width = if followed_by_gap { 2 } else { 1 };
}

/// A freshly allocated, fully blanked cell grid.
fn new_buffer(pen: ScreenPen, rows: c_int, cols: c_int) -> Vec<ScreenCell> {
    let mut cells = vec![ZERO_CELL; as_index(rows.max(0)) * as_index(cols.max(0))];
    blank_cells(&mut cells, &pen);
    cells
}

/// How many leading cells of `row` are non-blank, i.e. where its trailing run
/// of blanks starts.
fn line_popcount(row: &[ScreenCell]) -> c_int {
    let filled = row
        .iter()
        .rposition(|cell| cell.schar != 0)
        .map_or(0, |last| last + 1);
    count_as_int(filled)
}

/// A scroll as the host sees it: a move it may perform itself, and damage.
struct ReportMoves<'a>(&'a mut VTermScreen);

impl ScrollTarget for ReportMoves<'_> {
    fn moverect(&mut self, dest: VTermRect, src: VTermRect) {
        if !self.0.report_moverect(dest, src) {
            self.0.damage(dest);
        }
    }

    fn erase(&mut self, rect: VTermRect) {
        self.0.damage(rect);
    }
}

/// A scroll as the cells see it: moved and blanked.
struct MoveCells<'a> {
    screen: &'a mut VTermScreen,
    state_rows: c_int,
    lines: &'a [VTermLineInfo],
}

impl ScrollTarget for MoveCells<'_> {
    fn moverect(&mut self, dest: VTermRect, src: VTermRect) {
        self.screen.move_cells(dest, src);
    }

    fn erase(&mut self, rect: VTermRect) {
        self.screen
            .erase_cells(rect, false, self.state_rows, self.lines);
    }
}

impl VTermScreen {
    /// Scrolls a region. Under cell or row merging the move happens at once,
    /// in two passes so that the host sees the cells settle before it is
    /// told. Under scroll merging the move is coalesced with whatever is
    /// already pending.
    pub(crate) fn scroll_rect(
        &mut self,
        region: VTermRect,
        downward: c_int,
        rightward: c_int,
        state_rows: c_int,
        lines: &[VTermLineInfo],
    ) {
        if self.damage_merge != VTERM_DAMAGE_SCROLL {
            let mut cells = MoveCells {
                screen: self,
                state_rows,
                lines,
            };
            scroll_rect(region, downward, rightward, &mut cells);
            self.flush_damage();
            scroll_rect(region, downward, rightward, &mut ReportMoves(self));
            return;
        }
        if self.damaged.start_row != NO_RECT && !intersects(&region, &self.damaged) {
            self.flush_damage();
        }
        let pending_matches = self.pending_scrollrect == region
            && (self.pending_scroll_downward == 0 && downward == 0
                || self.pending_scroll_rightward == 0 && rightward == 0);
        if self.pending_scrollrect.start_row != NO_RECT && pending_matches {
            self.pending_scroll_downward += downward;
            self.pending_scroll_rightward += rightward;
        } else {
            if self.pending_scrollrect.start_row != NO_RECT {
                self.flush_damage();
            }
            self.pending_scrollrect = region;
            self.pending_scroll_downward = downward;
            self.pending_scroll_rightward = rightward;
        }
        let mut cells = MoveCells {
            screen: self,
            state_rows,
            lines,
        };
        scroll_rect(region, downward, rightward, &mut cells);
        if self.damaged.start_row != NO_RECT {
            follow_scroll(&mut self.damaged, &region, downward, rightward);
        }
    }
}
