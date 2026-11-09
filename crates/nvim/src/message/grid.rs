//! The message grid, and scrolling it.
//!
//! Messages are drawn onto their own grid ([`msg_grid_validate`] sizes it and
//! hands it to the compositor) floating over the bottom of the screen, so the
//! window text underneath survives a message that scrolls. [`msg_scroll_up`]
//! and [`msg_scroll_flush`] move that grid; [`msg_reset_scroll`] puts it back
//! once the message is gone.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

use super::*;
use crate::grid::{SCHAR_BUF_SIZE, default_grid_ref, default_gridview, schar_text};
use crate::narrow::len_as_integer;
use crate::narrow::number_as_int;
use crate::window::Z_INDEX_MESSAGES;
use crate::winlayer::Win;
use core::ffi::{c_int, c_uint};

/// The message grid, as a handle.
///
/// One acquisition per path. The grid's address is registered with the
/// compositor and reached from there while it is being drawn on, so it is
/// named rather than borrowed -- see [`GridRef`] for why that matters.
pub(crate) fn msg_grid_ref() -> GridRef {
    GridRef::of_cell(&MSG_GRID)
}

/// Where message output goes, in screen coordinates.
///
/// The message grid when it has one -- its row 0 sits at `MSG_GRID_POS`, so
/// the view offsets screen rows back onto it -- and the default grid
/// otherwise, where a screen row already is a grid row.
///
/// Computed, never stored. As a cell (`msg_grid_adj`) this was a second name
/// for `MSG_GRID`'s address, held in step by hand from three places in
/// [`msg_grid_validate`] and [`msg_grid_set_pos`]; between the `grid_alloc`
/// and the assignment it named the *previous* target, and before the first
/// [`msg_grid_validate`] it named nothing at all -- which is why the
/// unit-test harness had to point it at `DEFAULT_GRID` before it could let a
/// message be printed.
pub(crate) fn msg_grid_view() -> GridView {
    let grid = msg_grid_ref();
    if grid.is_allocated() {
        GridView {
            target: grid.raw(),
            row_offset: -MSG_GRID_POS.get(),
            col_offset: 0,
        }
    } else {
        default_gridview()
    }
}

/// Has `id` been handed out as a message id?
pub fn msg_id_exists(id: int64_t) -> bool {
    id > 0 && id < MSG_ID_NEXT.get()
}

/// Tell the UI where the message grid now sits.
pub(crate) fn ui_ext_msg_set_pos(row: c_int, scrolled: bool) {
    let mut grid = msg_grid_ref();
    let mut sep = [0; SCHAR_BUF_SIZE];
    let sep = ApiString::from_bytes(schar_text(Win::current().w_p_fcs_chars.msgsep, &mut sep));
    ui_call_msg_set_pos(
        grid.handle.into(),
        row.into(),
        scrolled,
        sep,
        grid.zindex.into(),
        len_as_integer(grid.comp_index),
    );
    grid.pending_comp_index_update = false;
}

/// Move the message grid to `row`, telling the UI unless output is throttled.
pub fn msg_grid_set_pos(row: c_int, scrolled: bool) {
    if !msg_grid_ref().throttled {
        ui_ext_msg_set_pos(row, scrolled);
        MSG_GRID_POS_AT_FLUSH.set(row);
    }
    // Where [`msg_grid_view`] reads the view's row offset back from.
    MSG_GRID_POS.set(row);
}

/// Are messages drawn on a grid at all?
///
/// They are not before the first redraw, and not under `ext_messages`.
pub fn msg_use_grid() -> bool {
    default_grid_ref().is_allocated() && !ui_has(UI_MESSAGES)
}

/// Allocate, resize, reposition or free the message grid to match the screen.
pub fn msg_grid_validate() {
    let mut grid = msg_grid_ref();
    grid_assign_handle(&mut grid);
    let should_alloc = msg_use_grid();
    let max_rows = ROWS.get() - number_as_int(p_ch());

    if should_alloc
        && (grid.rows != ROWS.get() || grid.cols != COLUMNS.get() || !grid.is_allocated())
    {
        // Force a valid screen size.
        grid_alloc(&mut grid, ROWS.get(), COLUMNS.get(), false, true);
        grid.zindex = Z_INDEX_MESSAGES;
        grid.track_dirty_cols(ROWS.get());

        // Tell the compositor to put the grid at the bottom, or at the top
        // while the pager owns the screen.
        let pos = if STATE.get() & MODE_ASKMORE != 0 {
            0
        } else {
            (max_rows - MSG_SCROLLED.get()).max(0)
        };
        grid.throttled = false; // don't throttle in 'cmdheight' area
        msg_grid_set_pos(pos, MSG_SCROLLED.get() != 0);
        let (rows, cols) = (grid.rows, grid.cols);
        ui_comp_put_grid(grid, pos, 0, rows, cols, false, true);
        ui_call_grid_resize(grid.handle.into(), cols.into(), rows.into());

        MSG_SCROLLED_AT_FLUSH.set(MSG_SCROLLED.get());
        grid.mouse_enabled = false;
    } else if !should_alloc && grid.is_allocated() {
        // Note: we run this both on moving to ext_messages, and on
        // resizing the screen while ext_messages is active.
        ui_comp_remove_grid(grid);
        grid.free();
        grid.forget_dirty_cols();
        ui_call_grid_destroy(grid.handle.into());
        grid.throttled = false;
        REDRAW_CMDLINE.set(true);
    } else if grid.is_allocated() && MSG_SCROLLED.get() == 0 && MSG_GRID_POS.get() != max_rows {
        let diff = MSG_GRID_POS.get() - max_rows;
        msg_grid_set_pos(max_rows, false);
        if diff > 0 {
            clear_msg_area(ROWS.get() - diff, ROWS.get(), 0, COLUMNS.get());
        }
    }

    if grid.is_allocated() && MSG_SCROLLED.get() == 0 && CMDLINE_ROW.get() < MSG_GRID_POS.get() {
        CMDLINE_ROW.set(MSG_GRID_POS.get());
    }
}

/// Send the line being built to the UI, mirrored if `'rightleft'` applies.
pub fn msg_line_flush() {
    if CMDMSG_RL.get() {
        grid_line_mirror(msg_grid_ref().cols);
    }
    grid_line_flush_if_valid_row();
}

/// Put the cursor at `row`/`col` of the message area.
pub fn msg_cursor_goto(row: c_int, mut col: c_int) {
    let mut row = row;
    if CMDMSG_RL.get() {
        col = COLUMNS.get() - 1 - col;
    }
    let grid = grid_adjust(msg_grid_view(), &mut row, &mut col);
    ui_grid_cursor_goto(grid.handle, row, col);
}

/// How many screen lines the message area currently occupies.
pub fn msg_scrollsize() -> c_int {
    MSG_SCROLLED.get() + number_as_int(p_ch()) + c_int::from(p_ch() > 0 || MSG_SCROLLED.get() > 1)
}

/// Should message output be batched into one scroll at flush time?
pub fn msg_do_throttle() -> bool {
    msg_use_grid() && RDB_FLAGS.get() & OPT_RDB_FLAG_NOTHROTTLE as c_uint == 0
}

/// Scroll the message area up one line.
///
/// `zerocmd` is set when this is making room under `'cmdheight'` zero, where
/// the freed line has to be cleared rather than scrolled into.
pub fn msg_scroll_up(may_throttle: bool, zerocmd: bool) {
    let mut grid = msg_grid_ref();
    if may_throttle && msg_do_throttle() {
        grid.throttled = true;
    }
    MSG_DID_SCROLL.set(true);
    if MSG_GRID_POS.get() > 0 {
        msg_grid_set_pos(MSG_GRID_POS.get() - 1, !zerocmd);
        if zerocmd && grid.is_allocated() {
            // When zerocmd is true, we're scrolling the first line of
            // msg_grid onto the screen; it must be cleared first.
            let (off, cols) = (grid.row_start(0), grid.cols);
            grid.clear_line(off, cols, false);
        }
    } else {
        let (rows, cols) = (grid.rows, grid.cols);
        grid_del_lines(grid, 0, 1, rows, 0, cols);
        // The dirty columns move up with the lines they describe.
        grid.scroll_dirty_cols();
    }
    // Ensure the message area is cleared to the default background.
    clear_msg_area(ROWS.get() - 1, ROWS.get(), 0, COLUMNS.get());
}

/// Send everything a throttled run of messages accumulated, as one scroll
/// plus the dirty part of each line.
pub fn msg_scroll_flush() {
    let mut grid = msg_grid_ref();
    // SAFETY: the caller's promise -- the grids are initialised, and
    // `ui_ext_msg_set_pos` only wants `curwin` alongside.
    if grid.throttled {
        grid.throttled = false;
        let pos_delta = MSG_GRID_POS_AT_FLUSH.get() - MSG_GRID_POS.get();
        debug_assert!(pos_delta >= 0);
        let delta = (MSG_SCROLLED.get() - MSG_SCROLLED_AT_FLUSH.get()).min(grid.rows);

        if pos_delta > 0 {
            ui_ext_msg_set_pos(MSG_GRID_POS.get(), true);
        }

        let to_scroll = delta - pos_delta - MSG_GRID_SCROLL_DISCOUNT.get();
        debug_assert!(to_scroll >= 0);

        // No scrolling to do while the grid is still moving down: the
        // repositioning above already showed the new lines.
        if to_scroll > 0 && MSG_GRID_POS.get() == 0 {
            ui_call_grid_scroll(
                grid.handle.into(),
                0,
                ROWS.get().into(),
                0,
                COLUMNS.get().into(),
                to_scroll.into(),
                0,
            );
        }

        // `ROWS` is re-read each time round, as upstream does: it is a
        // global that a resize changes, and this loop talks to the UI.
        let mut i = (ROWS.get() - delta.max(1)).max(0);
        while i < ROWS.get() {
            let row = i - MSG_GRID_POS.get();
            debug_assert!(row >= 0);
            let (dirty, cols) = (grid.take_dirty_col(row), grid.cols);
            let attr = hl_attr(HLF_MSG as c_int);
            ui_line(grid, row, false, 0, dirty, cols, attr, false);
            i += 1;
        }
    }
    MSG_SCROLLED_AT_FLUSH.set(MSG_SCROLLED.get());
    MSG_GRID_SCROLL_DISCOUNT.set(0);
    MSG_GRID_POS_AT_FLUSH.set(MSG_GRID_POS.get());
}

/// The messages are gone: put the grid back under `'cmdheight'` and clear it.
pub fn msg_reset_scroll() {
    let mut grid = msg_grid_ref();
    if ui_has(UI_MESSAGES) {
        // TODO(bfredl): some duplicate logic with update_screen(). Later
        // on we should properly disentangle message clear with full screen
        // redraw.
        return;
    }
    grid.throttled = false;
    // TODO(bfredl): calculate the conflict in the compositor instead.
    msg_grid_set_pos(ROWS.get() - number_as_int(p_ch()), false);
    CLEAR_CMDLINE.set(true);
    if grid.is_allocated() {
        // The bound is re-evaluated each time round, as upstream does.
        let mut i = 0;
        while i < msg_scrollsize().min(grid.rows) {
            let (off, cols) = (grid.row_start(i), grid.cols);
            grid.clear_line(off, cols, false);
            i += 1;
        }
    }
    MSG_SCROLLED.set(0);
    MSG_SCROLLED_AT_FLUSH.set(0);
    MSG_GRID_SCROLL_DISCOUNT.set(0);
}

/// The UI reattached or resized: restate the grid's size and position.
pub fn msg_ui_refresh() {
    let grid = msg_grid_ref();
    if ui_has(UI_MULTIGRID) && grid.is_allocated() {
        ui_call_grid_resize(grid.handle.into(), grid.cols.into(), grid.rows.into());
        ui_ext_msg_set_pos(MSG_GRID_POS.get(), MSG_SCROLLED.get() != 0);
    }
}

/// The compositor restacked the grid: tell the UI its new position.
pub fn msg_ui_flush() {
    let grid = msg_grid_ref();
    if ui_has(UI_MULTIGRID) && grid.is_allocated() && grid.pending_comp_index_update {
        ui_ext_msg_set_pos(MSG_GRID_POS.get(), MSG_SCROLLED.get() != 0);
    }
}

/// One more line of messages has scrolled off; remember where it started.
pub(crate) fn inc_msg_scrolled() {
    if with_vim_var_str(Vv::Scrollstart, CStr::is_empty) {
        // v:scrollstart is empty: set it to the script/function name and
        // line number the scrolling started at.
        let start = match crate::runtime::sourcing_name_copy() {
            None => gettext(c"Unknown").to_bytes().to_vec(),
            Some(name) => {
                // Upstream sizes the buffer for the name and forty bytes.
                let room = name.len() + 40;
                let lnum = int64_t::from(crate::runtime::sourcing_lnum());
                crate::formatted!(room, gettext(c"%s line %ld").to_bytes(), &name[..], lnum)
            }
        };
        set_vim_var_string(Vv::Scrollstart, Some(&start));
    }
    MSG_SCROLLED.set(MSG_SCROLLED.get() + 1);
    set_must_redraw(UPD_VALID);
}

/// Clear from the cursor to the end of the message area, unless silenced.
pub fn msg_clr_eos() {
    if MSG_SILENT.get() == 0 {
        msg_clr_eos_force();
    }
}

/// [`msg_clr_eos`], `'shortmess'` and `:silent` notwithstanding.
pub fn msg_clr_eos_force() {
    if ui_has(UI_MESSAGES) {
        return;
    }
    let (msg_startcol, msg_endcol) = if CMDMSG_RL.get() {
        (0, COLUMNS.get() - MSG_COL.get())
    } else {
        (MSG_COL.get(), COLUMNS.get())
    };

    // Avoid clearing the line the grid is about to be moved off.
    if msg_grid_ref().is_allocated() && MSG_ROW.get() < MSG_GRID_POS.get() {
        msg_grid_validate();
        if MSG_ROW.get() < MSG_GRID_POS.get() {
            MSG_ROW.set(MSG_GRID_POS.get());
        }
    }

    clear_msg_area(MSG_ROW.get(), MSG_ROW.get() + 1, msg_startcol, msg_endcol);
    clear_msg_area(MSG_ROW.get() + 1, ROWS.get(), 0, COLUMNS.get());

    REDRAW_CMDLINE.set(true);
    if MSG_ROW.get() < ROWS.get() - 1 || MSG_COL.get() == 0 {
        CLEAR_CMDLINE.set(false);
        MODE_DISPLAYED.set(false);
        CMDLINE_WAS_LAST_DRAWN.set(false);
    }
}

/// Clear the command line.
pub fn msg_clr_cmdline() {
    MSG_ROW.set(CMDLINE_ROW.get());
    MSG_COL.set(0);
    msg_clr_eos_force();
}
