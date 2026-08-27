//! The compositor: flattens the editor's stack of grids into the single
//! screen a UI without `ext_multigrid` can draw.
//!
//! [`LAYERS`] holds every grid the editor has put on screen — `default_grid`
//! at the bottom, then the floats, the message grid and the popup menu —
//! sorted by `zindex`, and each grid's `comp_index` is its position in that
//! stack. Whenever the editor draws a line into any of them,
//! [`ui_comp_raw_line`] decides whether it can be forwarded untouched or has
//! to be rebuilt from every layer overlapping it ([`compose_line`]). Both
//! paths end in `ui_composed_call_*`, which reaches only the UIs this module
//! draws for.
//!
//! Every grid in the stack is owned by something else — a window, the
//! message area, the popup menu, or the `default_grid` static — so a layer
//! is a bare pointer in C and a [`GridRef`] here: one unsafe constructor
//! carrying the invariant, safe field access afterwards, which is what lets
//! the composing itself be safe code. The scratch line and the stack are
//! reached only through momentary [`GlobalCell`] borrows: composing
//! re-enters this module, so no borrow may span one.
//!
//! Derived from Neovim's `src/nvim/ui_compositor.c`. Copyright Neovim
//! contributors; licensed under the Apache License, Version 2.0, as recorded
//! in `LICENSE.txt`.

#![deny(unsafe_op_in_unsafe_fn)]

mod scratch;

use core::ffi::{CStr, c_char, c_int, c_uint};
use core::ptr;

use crate::drawscreen::windows_in_curtab;
use crate::global_cell::GlobalCell;
use crate::grid::{GridRef, default_grid_ref, schar_from_ascii, schar_from_buf};
use crate::highlight::hl_blend_attrs;
use crate::highlight_group::{HLF_MSGSEP, syn_check_group, syn_id2attr};
use crate::log::{LOGLVL_DBG, logmsg_c};
use crate::main::{Columns, Rows, curwin, hl_attr_active, p_wd, rdb_flags};
use crate::message::msg_grid_ref;
use crate::options::{kOptRdbFlagCompositor, kOptRdbFlagInvalid};
use crate::os::time::os_sleep;
use crate::types::ui::{kLineFlagInvalid, kLineFlagWrap, kUIMultigrid};
use crate::types::{
    Boolean, Integer, LineFlags, NUL, RemoteUI, ScreenGrid, String_0, handle_T, sattr_T, schar_T,
    win_T,
};
use crate::ui::{
    ui_call_flush, ui_composed_call_grid_cursor_goto, ui_composed_call_grid_resize,
    ui_composed_call_grid_scroll, ui_composed_call_raw_line, ui_has,
};
use scratch::{Bufs, blend, clear_invalid_attrs};

/// The screen every other layer is composed onto.
fn default_layer() -> GridRef {
    default_grid_ref()
}

/// The message grid, which the `'msgsep'` row is also attributed to.
fn msg_layer() -> GridRef {
    msg_grid_ref()
}

/// The window's own grid.
///
/// # Safety
/// `wp` must be a live window.
unsafe fn win_layer(wp: *mut win_T) -> GridRef {
    // SAFETY: a live window owns its `w_grid_alloc` outright.
    unsafe { GridRef::new(&raw mut (*wp).w_grid_alloc) }
}

/// How many UIs this module draws for. Zero means nothing is composed.
static composed_uis: GlobalCell<c_int> = GlobalCell::new(0);

/// The layer stack, bottom first.
static LAYERS: GlobalCell<Vec<GridRef>> = GlobalCell::new(Vec::new());

static BUFS: GlobalCell<Bufs> = GlobalCell::new(Bufs::EMPTY);

/// The grid the last [`ui_comp_set_grid`] selected: every coordinate the
/// entry points take is relative to it. `None` until [`ui_comp_init`], and
/// only then: nothing clears it once it is set.
static curgrid: GlobalCell<Option<GridRef>> = GlobalCell::new(None);

/// [`curgrid`], for the callers that have already established there is one.
///
/// Every one of them reached here through an [`ui_comp_set_grid`] that
/// answered `true`, which is what selects a layer.
fn cur_layer() -> GridRef {
    curgrid.get().expect("a layer is selected")
}

/// The size the default grid was last resized to, for the assertions that
/// composing stays inside the scratch line. C keeps both under `NDEBUG`.
static chk_width: GlobalCell<c_int> = GlobalCell::new(0);
static chk_height: GlobalCell<c_int> = GlobalCell::new(0);

/// False between "the screen is about to be cleared" and the clear, which is
/// when floats must not be redrawn.
static valid_screen: GlobalCell<bool> = GlobalCell::new(true);

/// Where the message grid sits and whether it was scrolled into place —
/// `INT_MAX` until the first `msg_set_pos`.
static msg_current_row: GlobalCell<c_int> = GlobalCell::new(c_int::MAX);
static msg_was_scrolled: GlobalCell<bool> = GlobalCell::new(false);

/// The `'msgsep'` row drawn just above the message grid, and its fill
/// character. `-1` means there is none.
static msg_sep_row: GlobalCell<c_int> = GlobalCell::new(-1);
static msg_sep_char: GlobalCell<schar_T> = GlobalCell::new(schar_from_ascii(b' '));

/// Highlight ids for the `'redrawdebug'` overlays: a forwarded line, the
/// cleared tail, a composed line, and a recomposed area.
static dbghl_normal: GlobalCell<c_int> = GlobalCell::new(0);
static dbghl_clear: GlobalCell<c_int> = GlobalCell::new(0);
static dbghl_composed: GlobalCell<c_int> = GlobalCell::new(0);
static dbghl_recompose: GlobalCell<c_int> = GlobalCell::new(0);

fn layer_count() -> usize {
    LAYERS.with(Vec::len)
}

fn layer_at(index: usize) -> GridRef {
    LAYERS.with(|stack| stack[index])
}

/// The topmost layer above the default grid covering (`row`, `col`) that
/// `accept` takes.
fn topmost_at(row: c_int, col: c_int, accept: impl Fn(GridRef) -> bool) -> Option<GridRef> {
    (1..layer_count())
        .rev()
        .map(layer_at)
        .find(|&layer| accept(layer) && layer.covers(row, col))
}

pub fn ui_comp_init() {
    LAYERS.with_mut(|stack| stack.push(default_layer()));
    curgrid.set(Some(default_layer()));
}

/// # Safety
/// The highlight tables must exist.
pub unsafe fn ui_comp_syn_init() {
    dbghl_normal.set(syn_group(c"RedrawDebugNormal"));
    dbghl_clear.set(syn_group(c"RedrawDebugClear"));
    dbghl_composed.set(syn_group(c"RedrawDebugComposed"));
    dbghl_recompose.set(syn_group(c"RedrawDebugRecompose"));
}

/// The id of a highlight group, defining it if it is new.
fn syn_group(name: &'static CStr) -> c_int {
    // SAFETY: a literal name with its own length, C's `S_LEN`.
    unsafe { syn_check_group(name.as_ptr(), name.count_bytes()) }
}

/// # Safety
/// `ui` must be a live UI.
pub unsafe fn ui_comp_attach(ui: *mut RemoteUI) {
    composed_uis.set(composed_uis.get() + 1);
    // SAFETY: the caller's obligation.
    unsafe { (*ui).composed = true };
}

/// # Safety
/// As [`ui_comp_attach`].
pub unsafe fn ui_comp_detach(ui: *mut RemoteUI) {
    composed_uis.set(composed_uis.get() - 1);
    if composed_uis.get() == 0 {
        BUFS.with_mut(|bufs| *bufs = Bufs::EMPTY);
    }
    // SAFETY: the caller's obligation.
    unsafe { (*ui).composed = false };
}

pub fn ui_comp_should_draw() -> bool {
    composed_uis.get() != 0 && valid_screen.get()
}

/// Raises or lowers the layer at `layer_idx`, bringing `comp_index` back in
/// line with `zindex`.
pub fn ui_comp_layers_adjust(mut layer_idx: usize, raise: bool) {
    let size = layer_count();
    let mut layer = layer_at(layer_idx);
    if raise {
        while layer_idx < size - 1 && layer.zindex > layer_at(layer_idx + 1).zindex {
            let mut above = layer_at(layer_idx + 1);
            LAYERS.with_mut(|stack| stack[layer_idx] = above);
            above.set_comp_index(layer_idx);
            layer_idx += 1;
        }
    } else {
        while layer_idx > 0 && layer.zindex < layer_at(layer_idx - 1).zindex {
            let mut below = layer_at(layer_idx - 1);
            LAYERS.with_mut(|stack| stack[layer_idx] = below);
            below.set_comp_index(layer_idx);
            layer_idx -= 1;
        }
    }
    LAYERS.with_mut(|stack| stack[layer_idx] = layer);
    layer.set_comp_index(layer_idx);
}

/// Places `grid` at (`col`, `row`) with a `width` × `height` size, adding it
/// as a new layer if it is not one already. Answers whether it moved.
///
/// # Safety
/// `grid` must satisfy [`GridRef`]'s contract.
pub unsafe fn ui_comp_put_grid(
    grid: *mut ScreenGrid,
    row: c_int,
    col: c_int,
    height: c_int,
    width: c_int,
    valid: bool,
    on_top: bool,
) -> bool {
    // SAFETY: the caller's obligation.
    let mut grid = unsafe { GridRef::new(grid) };
    let moved;
    grid.pending_comp_index_update = true;

    if grid.comp_index != 0 {
        moved = row != grid.comp_row || col != grid.comp_col;
        if ui_comp_should_draw() {
            // Redraw what the old position covered and the new one does not.
            // Disabling the grid keeps `compose_area` off it.
            let (old_row, old_col) = (grid.comp_row, grid.comp_col);
            let (old_bot, old_right) = (old_row + grid.comp_height, old_col + grid.comp_width);
            let (top, bot) = (row.max(old_row), (row + height).min(old_bot));
            grid.comp_disabled = true;
            compose_area(old_row, row, old_col, old_right);
            if old_col < col {
                compose_area(top, bot, old_col, col);
            }
            if col + width < old_right {
                compose_area(top, bot, col + width, old_right);
            }
            compose_area(row + height, old_bot, old_col, old_right);
            grid.comp_disabled = false;
        }
        grid.comp_row = row;
        grid.comp_col = col;
    } else {
        moved = true;
        // C guards this scan with `#ifndef NDEBUG`, and so does the port: a
        // grid pushed twice would corrupt the stack silently.
        debug_assert!(
            !LAYERS.with(|stack| stack.iter().any(|layer| layer.same(grid))),
            "grid is not already a layer",
        );

        let mut insert_at = layer_count();
        while insert_at > 0 && layer_at(insert_at - 1).zindex > grid.zindex {
            insert_at -= 1;
        }

        // A new grid of the current window's own zindex goes *under* that
        // window unless it asked to be on top. Upstream reads
        // `layers[insert_at - 1]` without checking `insert_at` first; that
        // read needs a grid below `default_grid`'s zindex of 0, which
        // nothing produces.
        if insert_at > 0 && !curwin.get().is_null() && !on_top {
            let below = layer_at(insert_at - 1);
            // SAFETY: `curwin` is a live window whenever it is non-null.
            let curwin_grid = unsafe { win_layer(curwin.get()) };
            if below.same(curwin_grid) && below.zindex == grid.zindex {
                insert_at -= 1;
            }
        }

        LAYERS.with_mut(|stack| stack.insert(insert_at, grid));
        for i in insert_at + 1..layer_count() {
            layer_at(i).set_comp_index(i);
        }

        grid.comp_row = row;
        grid.comp_col = col;
        grid.comp_index = insert_at;
        grid.pending_comp_index_update = true;
    }

    grid.comp_height = height;
    grid.comp_width = width;
    if moved && valid && ui_comp_should_draw() {
        compose_under(grid);
    }
    moved
}

/// # Safety
/// `grid` must satisfy [`GridRef`]'s contract.
pub unsafe fn ui_comp_remove_grid(grid: *mut ScreenGrid) {
    // SAFETY: the caller's obligation.
    let mut grid = unsafe { GridRef::new(grid) };
    debug_assert!(!grid.same(default_layer()), "grid != &default_grid");
    if grid.comp_index == 0 {
        return; // The grid was not a layer.
    }
    if curgrid.get().is_some_and(|cur| cur.same(grid)) {
        curgrid.set(Some(default_layer()));
    }

    let removed_at = grid.comp_index;
    LAYERS.with_mut(|stack| {
        stack.remove(removed_at);
    });
    for i in removed_at..layer_count() {
        layer_at(i).set_comp_index(i);
    }
    grid.comp_index = 0;
    grid.pending_comp_index_update = true;

    // Recompose the area the grid was covering. Inefficient when it was
    // itself overlapped: only the layers up to `comp_index` needed it.
    if ui_comp_should_draw() {
        compose_under(grid);
    }
}

/// Selects the layer `handle` names as the one coordinates are relative to.
pub fn ui_comp_set_grid(handle: handle_T) -> bool {
    if curgrid.get().is_some_and(|cur| cur.handle == handle) {
        return true;
    }
    let found = LAYERS.with(|stack| stack.iter().find(|layer| layer.handle == handle).copied());
    found.inspect(|&grid| curgrid.set(Some(grid))).is_some()
}

/// Moves `grid` up to `new_index`, sliding everything it passes down one,
/// then recomposes each overlap it uncovered.
fn raise_grid(mut grid: GridRef, new_index: usize) {
    let old_index = grid.comp_index;
    for i in old_index..new_index {
        let mut above = layer_at(i + 1);
        LAYERS.with_mut(|stack| stack[i] = above);
        above.set_comp_index(i);
    }
    LAYERS.with_mut(|stack| stack[new_index] = grid);
    grid.set_comp_index(new_index);

    for i in old_index..new_index {
        let other = layer_at(i);
        let top = grid.comp_row.max(other.comp_row);
        let bot = (grid.comp_row + grid.rows).min(other.comp_row + other.rows);
        let left = grid.comp_col.max(other.comp_col);
        let right = (grid.comp_col + grid.cols).min(other.comp_col + other.cols);
        compose_area(top, bot, left, right);
    }
}

pub fn ui_comp_grid_cursor_goto(grid_handle: Integer, r: Integer, c: Integer) {
    if !ui_comp_set_grid(grid_handle as handle_T) {
        return;
    }
    let cursor_row = cur_layer().comp_row + r as c_int;
    let cursor_col = cur_layer().comp_col + c as c_int;

    // Upstream's TODO: for efficiency all grids should be configured before
    // `win_update` runs, rather than here.
    if !cur_layer().same(default_layer()) {
        let mut new_index = layer_count() - 1;
        while new_index > 1 && layer_at(new_index).zindex > cur_layer().zindex {
            new_index -= 1;
        }
        if cur_layer().comp_index < new_index {
            raise_grid(cur_layer(), new_index);
        }
    }

    let default = default_layer();
    if cursor_col >= default.cols || cursor_row >= default.rows {
        return; // Upstream's TODO: this happens with 'writedelay'.
    }
    ui_composed_call_grid_cursor_goto(1, cursor_row.into(), cursor_col.into());
}

/// The grid that owns screen cell (`row`, `col`) for mouse purposes, or null
/// if none does.
///
/// # Safety
/// The window list must be live.
pub unsafe fn ui_comp_mouse_focus(row: c_int, col: c_int) -> *mut ScreenGrid {
    if let Some(grid) = topmost_at(row, col, |grid| grid.mouse_enabled) {
        return grid.raw();
    }
    if ui_has(kUIMultigrid) {
        // With `ext_multigrid` a window's grid is not composed and so has no
        // `comp_row`/`comp_col`; the window's own position stands in.
        // SAFETY: the caller's obligation; nothing here restructures the
        // window list.
        for wp in windows_in_curtab() {
            // SAFETY: `wp` came from the live window list.
            let (grid, winrow, wincol) = unsafe { (win_layer(wp), (*wp).w_winrow, (*wp).w_wincol) };
            if grid.mouse_enabled
                && row >= winrow
                && row < winrow + grid.rows
                && col >= wincol
                && col < wincol + grid.cols
            {
                return grid.raw();
            }
        }
    }
    ptr::null_mut()
}

/// The topmost grid at screen coordinates (`row`, `col`).
///
/// # Safety
/// As [`ui_comp_mouse_focus`].
pub unsafe fn ui_comp_get_grid_at_coord(row: c_int, col: c_int) -> *mut ScreenGrid {
    if let Some(grid) = topmost_at(row, col, |_| true) {
        return grid.raw();
    }
    // SAFETY: the caller's obligation.
    for wp in windows_in_curtab() {
        // SAFETY: `wp` came from the live window list.
        let (grid, hidden) = unsafe { (win_layer(wp), (*wp).w_config.hide) };
        if grid.covers(row, col) && !hidden {
            return grid.raw();
        }
    }
    default_layer().raw()
}

/// Rebuilds `[startcol, endcol)` of `row` from every layer overlapping it,
/// and sends the result on.
///
/// The baseline implementation: always correct, but sometimes more work than
/// the downstream UI needed — see [`ui_comp_raw_line`], which forwards a line
/// untouched when nothing covers it.
fn compose_line(row: Integer, startcol: Integer, endcol: Integer, mut flags: LineFlags) {
    let default = default_layer();
    // With 'rightleft' `startcol` can be -1: no layer overlaps that, and the
    // assertions below would fail on it.
    let mut startcol = startcol.max(0);
    let mut endcol = endcol;
    // We may be starting on the right half of a double-width character, so
    // take in the left half too — and skip it in the output if it was not.
    let (mut skipstart, mut skipend) = (0, 0);
    if startcol > 0 && flags & kLineFlagInvalid != 0 {
        startcol -= 1;
        skipstart = 1;
    }
    if endcol < Integer::from(default.cols) && flags & kLineFlagInvalid != 0 {
        endcol += 1;
        skipend = 1;
    }

    let skips = (skipstart, skipend);
    let (top, skipstart, skipend) =
        BUFS.with_mut(|bufs| compose_into(bufs, row, startcol, endcol, skips));

    debug_assert!(endcol <= chk_width.get().into(), "endcol <= chk_width");
    debug_assert!(row < chk_height.get().into(), "row < chk_height");

    // A line keeps its wrap flag only if it came from a full-width layer.
    if !top.is_some_and(|top| top.same(default) || (top.comp_col == 0 && top.cols == Columns.get()))
    {
        flags &= !kLineFlagWrap;
    }

    let (chars, attrs) = BUFS.with(|bufs| (bufs.chars.as_ptr(), bufs.attrs.as_ptr()));
    let start = startcol + skipstart as Integer;
    let end = endcol - skipend as Integer;
    // SAFETY: `compose_into` filled `endcol - startcol` cells of both
    // buffers, and both skips are inside that range.
    unsafe {
        let (chars, attrs) = (chars.add(skipstart), attrs.add(skipstart));
        ui_composed_call_raw_line(1, row, start, end, end, 0, flags, chars, attrs);
    }
}

/// Flattens `[startcol, endcol)` of `row` into `bufs`, answering the topmost
/// grid that contributed and the two skip counts as they ended up.
fn compose_into(
    bufs: &mut Bufs,
    row: Integer,
    startcol: Integer,
    endcol: Integer,
    skips: (usize, usize),
) -> (Option<GridRef>, usize, usize) {
    let default = default_layer();
    let (mut skipstart, mut skipend) = skips;
    let mut top = None;
    let width = (endcol - startcol) as usize;
    let line = &mut bufs.chars[..width];
    let attrbuf = &mut bufs.attrs[..width];
    // The backdrop 'winblend' and 'pumblend' blend against.
    let bg_off = default.row_start(row as c_int) + startcol as usize;
    let bg = default.cells(bg_off, width);

    let mut col = startcol as c_int;
    while Integer::from(col) < endcol {
        // How far the topmost layer covering `col` owns.
        let mut until = 0;
        LAYERS.with(|stack| {
            for &g in stack {
                // Composing can run after a shrink was requested but before
                // the resize landed, so trust the smaller of the allocated
                // and the composed size — cells the resize invalidated must
                // not reach the scratch line.
                let grid_width = g.cols.min(g.comp_width);
                let grid_height = g.rows.min(g.comp_height);
                if Integer::from(g.comp_row) > row
                    || row >= Integer::from(g.comp_row + grid_height)
                    || g.comp_disabled
                {
                    continue;
                }
                if g.comp_col <= col && col < g.comp_col + grid_width {
                    top = Some(g);
                    until = g.comp_col + grid_width;
                } else if g.comp_col > col {
                    until = until.min(g.comp_col);
                }
            }
        });
        until = until.min(endcol as c_int);

        debug_assert!(until > col, "until > col");
        debug_assert!(until <= default.cols, "until <= default_grid.cols");
        let at = (Integer::from(col) - startcol) as usize;
        let n = (until - col) as usize;

        // The default grid covers every column of every row it has, and is
        // the bottom layer, so some layer always won the column above.
        let mut grid = top.expect("a layer covers the column");
        if Integer::from(msg_sep_row.get()) == row && grid.comp_index <= msg_layer().comp_index {
            // Upstream's TODO: once floats have borders, the separator can
            // just be one around the message grid.
            grid = msg_layer();
            // SAFETY: the highlight table is built before anything draws.
            let sep_attr = unsafe { *hl_attr_active.get().add(HLF_MSGSEP as usize) };
            line[at..at + n].fill(msg_sep_char.get());
            attrbuf[at..at + n].fill(sep_attr);
        } else {
            let grid_row = row - Integer::from(grid.comp_row);
            let off = grid.row_start(grid_row as c_int) + (col - grid.comp_col) as usize;
            let (chars, attrs) = grid.cells(off, n);
            line[at..at + n].copy_from_slice(chars);
            attrbuf[at..at + n].copy_from_slice(attrs);
            if grid.comp_col + grid.cols > until && grid.char_at(off + n) == NUL as c_uint {
                // The run ends on the left half of a double-width character;
                // show a space instead.
                line[at + n - 1] = schar_from_ascii(b' ');
                if at == 0 && n == 1 {
                    skipstart = 0;
                }
            }
        }

        if grid.blending {
            blend(line, attrbuf, bg, at..at + n, width, blend_attrs);
        }

        // Tricky: an overlap that cut a double-width character in half must
        // show a space for the visible half.
        if line[at] == NUL as c_uint {
            line[at] = schar_from_ascii(b' ');
            if Integer::from(col) == endcol - 1 {
                skipend = 0;
            }
        } else if at == 0 && n > 1 && line[1] == NUL as c_uint {
            skipstart = 0;
        }

        col = until;
    }
    // A zero-width range leaves both skips at 0 either way, so nothing the
    // upstream read one cell before the buffer could have changed.
    if width > 0 && line[width - 1] == NUL as c_uint {
        skipend = 0;
    }

    let fatal = rdb_flags.get() & kOptRdbFlagInvalid != 0;
    clear_invalid_attrs(attrbuf, skipstart..width - skipend, fatal);
    (top, skipstart, skipend)
}

/// C's `hl_blend_attrs`, as [`blend`] wants it.
fn blend_attrs(back: sattr_T, front: sattr_T, thru: &mut bool) -> sattr_T {
    // SAFETY: the attribute tables are the editor's own.
    unsafe { hl_blend_attrs(back, front, thru) as sattr_T }
}

/// Paints an area in one of the `'redrawdebug'` colours, so the work the
/// compositor did is visible on screen.
fn compose_debug(rows: (Integer, Integer), cols: (Integer, Integer), syn_id: c_int, delay: bool) {
    let ((startrow, mut endrow), (startcol, mut endcol)) = (rows, cols);
    if rdb_flags.get() & kOptRdbFlagCompositor == 0 || startcol >= endcol {
        return;
    }
    let default = default_layer();
    endrow = endrow.min(default.rows.into());
    endcol = endcol.min(default.cols.into());
    // SAFETY: the highlight tables are built before anything draws.
    let attr = Integer::from(unsafe { syn_id2attr(syn_id) });

    if delay {
        debug_delay(endrow - startrow);
    }
    let (chars, attrs) = BUFS.with(|bufs| (bufs.chars.as_ptr(), bufs.attrs.as_ptr()));
    for r in startrow as c_int..endrow as c_int {
        let row = Integer::from(r);
        // SAFETY: the chunk is empty on the wire (`startcol` twice), so only
        // the clear out to `endcol` reaches the UI.
        unsafe {
            ui_composed_call_raw_line(1, row, startcol, startcol, endcol, attr, 0, chars, attrs);
        }
    }
    if delay {
        debug_delay(endrow - startrow);
    }
}

/// Flushes and sleeps, so `'redrawdebug'`'s overlay is visible before the
/// real content replaces it. `'writedelay'` is the unit.
fn debug_delay(lines: Integer) {
    ui_call_flush();
    let wd = p_wd.get().unsigned_abs();
    let factor = lines.clamp(1, 5) as u64;
    os_sleep(factor * wd);
}

/// Recomposes every row of an area.
fn compose_area<T: Into<Integer>>(startrow: T, endrow: T, startcol: T, endcol: T) {
    let (startrow, startcol) = (startrow.into(), startcol.into());
    let (endrow, endcol) = (endrow.into(), endcol.into());
    let recompose = dbghl_recompose.get();
    compose_debug((startrow, endrow), (startcol, endcol), recompose, true);
    let default = default_layer();
    let endrow = endrow.min(default.rows.into());
    let endcol = endcol.min(default.cols.into());
    if endcol <= startcol {
        return;
    }
    for r in startrow as c_int..endrow as c_int {
        compose_line(r.into(), startcol, endcol, kLineFlagInvalid);
    }
}

/// Recomposes the area under `grid`.
fn compose_under(grid: GridRef) {
    let bot = grid.comp_row + grid.rows;
    let right = grid.comp_col + grid.cols;
    compose_area(grid.comp_row, bot, grid.comp_col, right);
}

/// Recomposes the area under `grid`, which is what an option affecting
/// composition — `'pumblend'` for the popup menu, say — needs after a change.
///
/// # Safety
/// `grid` must satisfy [`GridRef`]'s contract.
pub unsafe fn ui_comp_compose_grid(grid: *mut ScreenGrid) {
    if ui_comp_should_draw() {
        // SAFETY: the caller's obligation.
        compose_under(unsafe { GridRef::new(grid) });
    }
}

/// C's `DLOG`, for the two geometry complaints below.
///
/// # Safety
/// `fmt` must take two `Integer`s.
unsafe fn dlog(line: c_int, fmt: *const c_char, value: Integer, grid: Integer) {
    let here = c"ui_comp_raw_line".as_ptr();
    // SAFETY: the caller's obligation.
    unsafe { logmsg_c!(LOGLVL_DBG, ptr::null(), here, line, true, fmt, value, grid) };
}

/// One drawn line, straight from the grid the editor drew it into: either
/// forwarded as it is, or recomposed when something covers it.
///
/// # Safety
/// `chunk` and `attrs` must each address `endcol - startcol` readable cells.
#[expect(clippy::too_many_arguments, reason = "one parameter per wire field")]
pub unsafe fn ui_comp_raw_line(
    grid: Integer,
    row: Integer,
    startcol: Integer,
    endcol: Integer,
    clearcol: Integer,
    clearattr: Integer,
    mut flags: LineFlags,
    chunk: *const schar_T,
    attrs: *const sattr_T,
) {
    if !ui_comp_should_draw() || !ui_comp_set_grid(grid as handle_T) {
        return;
    }
    let cur = cur_layer();
    let row = row + Integer::from(cur.comp_row);
    let startcol = startcol + Integer::from(cur.comp_col);
    let mut endcol = endcol + Integer::from(cur.comp_col);
    let mut clearcol = clearcol + Integer::from(cur.comp_col);
    if !cur.same(default_layer()) {
        flags &= !kLineFlagWrap;
    }
    debug_assert!(endcol <= clearcol, "endcol <= clearcol");

    // Upstream's TODO: this should not be necessary, but on some resize paths
    // a window is drawn against the older, larger screen size.
    let default = default_layer();
    if row >= Integer::from(default.rows) {
        let fmt = c"compositor: invalid row %ld on grid %ld".as_ptr();
        // SAFETY: two `Integer`s, as the format asks.
        unsafe { dlog(580, fmt, row, grid) };
        return;
    }
    if clearcol > Integer::from(default.cols) {
        let fmt = c"compositor: invalid last column %ld on grid %ld".as_ptr();
        // SAFETY: as above.
        unsafe { dlog(585, fmt, clearcol, grid) };
        if startcol >= Integer::from(default.cols) {
            return;
        }
        clearcol = default.cols.into();
        endcol = endcol.min(clearcol);
    }

    let covered = curgrid_covered_above(row as c_int);
    // Upstream's TODO: `compose_line` should learn to respect clearing, and
    // be optimized for uncovered lines.
    if flags & kLineFlagInvalid != 0 || covered || cur.blending {
        let composed = dbghl_composed.get();
        compose_debug((row, row + 1), (startcol, clearcol), composed, true);
        compose_line(row, startcol, clearcol, flags);
    } else {
        let (normal, clear) = (dbghl_normal.get(), dbghl_clear.get());
        compose_debug(
            (row, row + 1),
            (startcol, endcol),
            normal,
            endcol >= clearcol,
        );
        compose_debug((row, row + 1), (endcol, clearcol), clear, true);
        #[cfg(debug_assertions)]
        {
            // The only `slice` use left in the module, and it is behind this
            // cfg -- so the import is too, or a release build warns.
            use core::slice;

            let n = (endcol - startcol).max(0) as usize;
            // SAFETY: the caller's obligation on `attrs`.
            let drawn = unsafe { slice::from_raw_parts(attrs, n) };
            debug_assert!(drawn.iter().all(|&attr| attr >= 0), "attrs[i] >= 0");
        }
        let (cc, ca) = (clearcol, clearattr);
        // SAFETY: the caller's obligation on `chunk` and `attrs`.
        unsafe {
            ui_composed_call_raw_line(1, row, startcol, endcol, cc, ca, flags, chunk, attrs);
        }
    }
}

/// Marks the screen invalid — it is about to be cleared, and floats must not
/// be redrawn until it has been. Answers the previous state.
pub fn ui_comp_set_screen_valid(valid: bool) -> bool {
    let old_val = valid_screen.get();
    valid_screen.set(valid);
    if !valid {
        msg_sep_row.set(-1);
    }
    old_val
}

/// # Safety
/// `sep_char` must be a valid string.
pub unsafe fn ui_comp_msg_set_pos(
    _grid: Integer,
    row: Integer,
    scrolled: Boolean,
    sep_char: String_0,
    _zindex: Integer,
    _compindex: Integer,
) {
    let mut msg = msg_layer();
    msg.pending_comp_index_update = true;
    msg.comp_row = row as c_int;
    if scrolled && row > 0 {
        msg_sep_row.set(row as c_int - 1);
        if !sep_char.data().is_null() {
            // SAFETY: the caller's obligation.
            msg_sep_char.set(unsafe { schar_from_buf(sep_char.data(), sep_char.len()) });
        }
    } else {
        msg_sep_row.set(-1);
    }

    let was_at = msg_current_row.get();
    if row > Integer::from(was_at) && ui_comp_should_draw() {
        let first_row = Integer::from((was_at - 1).max(0));
        compose_area(first_row, row, 0, default_layer().cols.into());
    } else if row < Integer::from(was_at)
        && ui_comp_should_draw()
        && (was_at < Rows.get() || (scrolled && !msg_was_scrolled.get()))
    {
        let delta = was_at - row as c_int;
        if msg.blending {
            let first_row = (row as c_int - c_int::from(scrolled)).max(0);
            compose_area(first_row, Rows.get() - delta, 0, Columns.get());
        } else {
            // Scroll the separator together with the message text.
            let first_row = (row as c_int - c_int::from(msg_was_scrolled.get())).max(0);
            let (bot, right) = (Integer::from(Rows.get()), Integer::from(Columns.get()));
            ui_composed_call_grid_scroll(1, first_row.into(), bot, 0, right, delta.into(), 0);
            if scrolled && !msg_was_scrolled.get() && row > 0 {
                compose_area(row - 1, row, 0, Columns.get().into());
            }
        }
    }

    msg_current_row.set(row as c_int);
    msg_was_scrolled.set(scrolled);
}

/// Whether `curgrid` has something over it on `row` or above.
///
/// Upstream's TODO: this only handles the message row.
fn curgrid_covered_above(row: c_int) -> bool {
    let above_msg = layer_at(layer_count() - 1).same(msg_layer())
        && row < msg_current_row.get() - c_int::from(msg_was_scrolled.get());
    layer_count() - usize::from(above_msg) > cur_layer().comp_index + 1
}

pub fn ui_comp_grid_scroll(
    grid: Integer,
    top: Integer,
    bot: Integer,
    left: Integer,
    right: Integer,
    rows: Integer,
    cols: Integer,
) {
    if !ui_comp_should_draw() || !ui_comp_set_grid(grid as handle_T) {
        return;
    }
    let cur = cur_layer();
    let top = top + Integer::from(cur.comp_row);
    let bot = bot + Integer::from(cur.comp_row);
    let left = left + Integer::from(cur.comp_col);
    let right = right + Integer::from(cur.comp_col);
    let covered = curgrid_covered_above((bot - rows.max(0)) as c_int);

    if covered || cur.blending {
        // Upstream's TODO: check whether the rectangles overlap at all, and
        // work out the subareas that could still scroll.
        let recompose = dbghl_recompose.get();
        compose_debug((top, bot), (left, right), recompose, true);
        for r in (top + (-rows).max(0)) as c_int..(bot - rows.max(0)) as c_int {
            // Upstream's TODO: a workaround for `win_update` scrolling twice
            // in a row, the second over space the first invalidated.
            let row_off = cur.row_start(r - cur.comp_row);
            let off = row_off + left as usize - cur.comp_col as usize;
            if cur.attr_at(off) >= 0 {
                compose_line(r.into(), left, right, 0);
            }
        }
    } else {
        ui_composed_call_grid_scroll(1, top, bot, left, right, rows, cols);
        if rdb_flags.get() & kOptRdbFlagCompositor != 0 {
            debug_delay(2);
        }
    }
}

/// Resizes the composed screen, and with it the scratch line.
pub fn ui_comp_grid_resize(grid: Integer, width: Integer, height: Integer) {
    if grid == 1 {
        ui_composed_call_grid_resize(1, width, height);
        chk_width.set(width as c_int);
        chk_height.set(height as c_int);
        let new_bufsize = width as usize;
        BUFS.with_mut(|bufs| {
            if bufs.chars.len() != new_bufsize {
                bufs.chars = vec![0; new_bufsize];
                bufs.attrs = vec![0; new_bufsize];
            }
        });
    }
}
