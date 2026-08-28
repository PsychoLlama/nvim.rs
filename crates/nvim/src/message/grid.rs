//! The message grid, and scrolling it.
//!
//! Messages are drawn onto their own grid ([`msg_grid_validate`] sizes it and
//! hands it to the compositor) floating over the bottom of the screen, so the
//! window text underneath survives a message that scrolls. [`msg_scroll_up`]
//! and [`msg_scroll_flush`] move that grid; [`msg_reset_scroll`] puts it back
//! once the message is gone.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::grid::{default_grid_ref, default_gridview};
use core::ffi::{c_char, c_int, c_uint};
use core::ptr;

/// The message grid, as a handle.
///
/// One acquisition per path. The grid's address is registered with the
/// compositor and reached from there while it is being drawn on, so it is
/// named rather than borrowed -- see [`GridRef`] for why that matters.
pub(crate) fn msg_grid_ref() -> GridRef {
    GridRef::of_cell(&msg_grid)
}

/// Where message output goes, in screen coordinates.
///
/// The message grid when it has one -- its row 0 sits at `msg_grid_pos`, so
/// the view offsets screen rows back onto it -- and the default grid
/// otherwise, where a screen row already is a grid row.
///
/// Computed, never stored. As a cell (`msg_grid_adj`) this was a second name
/// for `msg_grid`'s address, held in step by hand from three places in
/// [`msg_grid_validate`] and [`msg_grid_set_pos`]; between the `grid_alloc`
/// and the assignment it named the *previous* target, and before the first
/// [`msg_grid_validate`] it named nothing at all -- which is why the
/// unit-test harness had to point it at `default_grid` before it could let a
/// message be printed.
pub(crate) fn msg_grid_view() -> GridView {
    let grid = msg_grid_ref();
    if grid.is_allocated() {
        GridView {
            target: grid.raw(),
            row_offset: -msg_grid_pos.get(),
            col_offset: 0,
        }
    } else {
        default_gridview()
    }
}

/// Has `id` been handed out as a message id?
pub fn msg_id_exists(id: int64_t) -> bool {
    id > 0 && id < msg_id_next.get()
}

/// Tell the UI where the message grid now sits.
///
/// # Safety
/// The message grid must be allocated and `curwin` valid.
pub(crate) unsafe fn ui_ext_msg_set_pos(row: c_int, scrolled: bool) {
    let mut grid = msg_grid_ref();
    let mut sep = [0 as c_char; 32];
    // SAFETY: the caller's promise -- a live window, whose 'fillchars' this
    // reads; `schar_get` writes at most `MAX_SCHAR_SIZE` bytes plus a NUL.
    // `sep` outlives the call below, which copies the separator out.
    let sep = unsafe {
        let size = schar_get(sep.as_mut_ptr(), (*curwin.get()).w_p_fcs_chars.msgsep);
        String_0::from_raw_parts(sep.as_mut_ptr(), size)
    };
    ui_call_msg_set_pos(
        grid.handle.into(),
        row.into(),
        scrolled,
        sep,
        grid.zindex.into(),
        grid.comp_index as Integer,
    );
    grid.pending_comp_index_update = false;
}

/// Move the message grid to `row`, telling the UI unless output is throttled.
///
/// # Safety
/// The message grid must be initialised.
pub unsafe fn msg_grid_set_pos(row: c_int, scrolled: bool) {
    if !msg_grid_ref().throttled {
        // SAFETY: the caller's promise.
        unsafe { ui_ext_msg_set_pos(row, scrolled) };
        msg_grid_pos_at_flush.set(row);
    }
    // Where [`msg_grid_view`] reads the view's row offset back from.
    msg_grid_pos.set(row);
}

/// Are messages drawn on a grid at all?
///
/// They are not before the first redraw, and not under `ext_messages`.
///
/// # Safety
/// Only that the default grid is initialised.
pub unsafe fn msg_use_grid() -> bool {
    // SAFETY: a `static`'s address is always valid.
    default_grid_ref().is_allocated() && !ui_has(kUIMessages)
}

/// Allocate, resize, reposition or free the message grid to match the screen.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_grid_validate() {
    let mut grid = msg_grid_ref();
    grid_assign_handle(&mut grid);
    // SAFETY: the caller's promise, throughout -- the grids are initialised
    // and `curwin` is live, which is all `ui_ext_msg_set_pos` needs.
    let should_alloc = unsafe { msg_use_grid() };
    let max_rows = Rows.get() - p_ch.get() as c_int;

    if should_alloc
        && (grid.rows != Rows.get() || grid.cols != Columns.get() || !grid.is_allocated())
    {
        // Force a valid screen size.
        grid_alloc(&mut grid, Rows.get(), Columns.get(), false, true);
        grid.zindex = kZIndexMessages as c_int;
        grid.track_dirty_cols(Rows.get());

        // Tell the compositor to put the grid at the bottom, or at the top
        // while the pager owns the screen.
        let pos = if State.get() & MODE_ASKMORE != 0 {
            0
        } else {
            (max_rows - msg_scrolled.get()).max(0)
        };
        grid.throttled = false; // don't throttle in 'cmdheight' area
        unsafe { msg_grid_set_pos(pos, msg_scrolled.get() != 0) };
        let (rows, cols) = (grid.rows, grid.cols);
        unsafe { ui_comp_put_grid(grid.raw(), pos, 0, rows, cols, false, true) };
        ui_call_grid_resize(grid.handle.into(), cols.into(), rows.into());

        msg_scrolled_at_flush.set(msg_scrolled.get());
        grid.mouse_enabled = false;
    } else if !should_alloc && grid.is_allocated() {
        // Note: we run this both on moving to ext_messages, and on
        // resizing the screen while ext_messages is active.
        unsafe { ui_comp_remove_grid(grid.raw()) };
        grid.free();
        grid.forget_dirty_cols();
        ui_call_grid_destroy(grid.handle.into());
        grid.throttled = false;
        redraw_cmdline.set(true);
    } else if grid.is_allocated() && msg_scrolled.get() == 0 && msg_grid_pos.get() != max_rows {
        let diff = msg_grid_pos.get() - max_rows;
        unsafe { msg_grid_set_pos(max_rows, false) };
        if diff > 0 {
            clear_msg_area(Rows.get() - diff, Rows.get(), 0, Columns.get());
        }
    }

    if grid.is_allocated() && msg_scrolled.get() == 0 && cmdline_row.get() < msg_grid_pos.get() {
        cmdline_row.set(msg_grid_pos.get());
    }
}

/// Send the line being built to the UI, mirrored if `'rightleft'` applies.
///
/// # Safety
/// A grid line must be under construction.
pub unsafe fn msg_line_flush() {
    // SAFETY: the caller's promise -- a batch is open.
    if cmdmsg_rl.get() {
        grid_line_mirror(msg_grid_ref().cols);
    }
    unsafe { grid_line_flush_if_valid_row() };
}

/// Put the cursor at `row`/`col` of the message area.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_cursor_goto(row: c_int, mut col: c_int) {
    let mut row = row;
    if cmdmsg_rl.get() {
        col = Columns.get() - 1 - col;
    }
    let grid = unsafe { grid_adjust(msg_grid_view(), &mut row, &mut col) };
    ui_grid_cursor_goto(grid.handle, row, col);
}

/// How many screen lines the message area currently occupies.
pub fn msg_scrollsize() -> c_int {
    msg_scrolled.get() + p_ch.get() as c_int + c_int::from(p_ch.get() > 0 || msg_scrolled.get() > 1)
}

/// Should message output be batched into one scroll at flush time?
///
/// # Safety
/// See [`msg_use_grid`].
pub unsafe fn msg_do_throttle() -> bool {
    unsafe { msg_use_grid() && rdb_flags.get() & kOptRdbFlagNothrottle as c_uint == 0 }
}

/// Scroll the message area up one line.
///
/// `zerocmd` is set when this is making room under `'cmdheight'` zero, where
/// the freed line has to be cleared rather than scrolled into.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_scroll_up(may_throttle: bool, zerocmd: bool) {
    let mut grid = msg_grid_ref();
    // SAFETY: the caller's promise -- the grids are initialised.
    if may_throttle && unsafe { msg_do_throttle() } {
        grid.throttled = true;
    }
    msg_did_scroll.set(true);
    if msg_grid_pos.get() > 0 {
        unsafe { msg_grid_set_pos(msg_grid_pos.get() - 1, !zerocmd) };
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
    clear_msg_area(Rows.get() - 1, Rows.get(), 0, Columns.get());
}

/// Send everything a throttled run of messages accumulated, as one scroll
/// plus the dirty part of each line.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_scroll_flush() {
    let mut grid = msg_grid_ref();
    // SAFETY: the caller's promise -- the grids are initialised, and
    // `ui_ext_msg_set_pos` only wants `curwin` alongside.
    if grid.throttled {
        grid.throttled = false;
        let pos_delta = msg_grid_pos_at_flush.get() - msg_grid_pos.get();
        debug_assert!(pos_delta >= 0);
        let delta = (msg_scrolled.get() - msg_scrolled_at_flush.get()).min(grid.rows);

        if pos_delta > 0 {
            unsafe { ui_ext_msg_set_pos(msg_grid_pos.get(), true) };
        }

        let to_scroll = delta - pos_delta - msg_grid_scroll_discount.get();
        debug_assert!(to_scroll >= 0);

        // No scrolling to do while the grid is still moving down: the
        // repositioning above already showed the new lines.
        if to_scroll > 0 && msg_grid_pos.get() == 0 {
            ui_call_grid_scroll(
                grid.handle.into(),
                0,
                Rows.get().into(),
                0,
                Columns.get().into(),
                to_scroll.into(),
                0,
            );
        }

        // `Rows` is re-read each time round, as upstream does: it is a
        // global that a resize changes, and this loop talks to the UI.
        let mut i = (Rows.get() - delta.max(1)).max(0);
        while i < Rows.get() {
            let row = i - msg_grid_pos.get();
            debug_assert!(row >= 0);
            let (dirty, cols) = (grid.take_dirty_col(row), grid.cols);
            let attr = hl_attr(HLF_MSG as c_int);
            unsafe { ui_line(grid, row, false, 0, dirty, cols, attr, false) };
            i += 1;
        }
    }
    msg_scrolled_at_flush.set(msg_scrolled.get());
    msg_grid_scroll_discount.set(0);
    msg_grid_pos_at_flush.set(msg_grid_pos.get());
}

/// The messages are gone: put the grid back under `'cmdheight'` and clear it.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_reset_scroll() {
    let mut grid = msg_grid_ref();
    // SAFETY: the caller's promise -- the grids are initialised.
    if ui_has(kUIMessages) {
        // TODO(bfredl): some duplicate logic with update_screen(). Later
        // on we should properly disentangle message clear with full screen
        // redraw.
        return;
    }
    grid.throttled = false;
    // TODO(bfredl): calculate the conflict in the compositor instead.
    unsafe { msg_grid_set_pos(Rows.get() - p_ch.get() as c_int, false) };
    clear_cmdline.set(true);
    if grid.is_allocated() {
        // The bound is re-evaluated each time round, as upstream does.
        let mut i = 0;
        while i < msg_scrollsize().min(grid.rows) {
            let (off, cols) = (grid.row_start(i), grid.cols);
            grid.clear_line(off, cols, false);
            i += 1;
        }
    }
    msg_scrolled.set(0);
    msg_scrolled_at_flush.set(0);
    msg_grid_scroll_discount.set(0);
}

/// The UI reattached or resized: restate the grid's size and position.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_ui_refresh() {
    let grid = msg_grid_ref();
    if ui_has(kUIMultigrid) && grid.is_allocated() {
        ui_call_grid_resize(grid.handle.into(), grid.cols.into(), grid.rows.into());
        // SAFETY: the caller's promise, plus a live `curwin`.
        unsafe { ui_ext_msg_set_pos(msg_grid_pos.get(), msg_scrolled.get() != 0) };
    }
}

/// The compositor restacked the grid: tell the UI its new position.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_ui_flush() {
    let grid = msg_grid_ref();
    if ui_has(kUIMultigrid) && grid.is_allocated() && grid.pending_comp_index_update {
        // SAFETY: the caller's promise, plus a live `curwin`.
        unsafe { ui_ext_msg_set_pos(msg_grid_pos.get(), msg_scrolled.get() != 0) };
    }
}

/// One more line of messages has scrolled off; remember where it started.
///
/// # Safety
/// The exec stack must be non-empty. See [`sourcing_top`].
pub(crate) unsafe fn inc_msg_scrolled() {
    if unsafe { *get_vim_var_str(Vv::Scrollstart) } == 0 {
        // v:scrollstart is empty: set it to the script/function name and
        // line number the scrolling started at.
        let mut p = String_0::from_raw_parts(sourcing_top().es_name, 0);
        let mut tofree: *mut c_char = ptr::null_mut();
        if p.data().is_null() {
            p = unsafe { cstr_as_string(gettext(c"Unknown".as_ptr())) };
        } else {
            let tofreesize = unsafe { strlen(p.data()) } + 40;
            tofree = unsafe { xmalloc(tofreesize) }.cast();
            let fmt = unsafe { gettext(c"%s line %ld".as_ptr()) };
            let name = p.data();
            let lnum = sourcing_top().es_lnum as int64_t;
            let len = unsafe { vim_snprintf_safelen(tofree, tofreesize, fmt, name, lnum) };
            p.set_len(len);
            p.set_data(tofree);
        }
        unsafe { set_vim_var_string(Vv::Scrollstart, p.data(), p.len() as ptrdiff_t) };
        unsafe { xfree(tofree.cast()) };
    }
    msg_scrolled.set(msg_scrolled.get() + 1);
    set_must_redraw(UPD_VALID);
}

/// Clear from the cursor to the end of the message area, unless silenced.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_clr_eos() {
    if msg_silent.get() == 0 {
        unsafe { msg_clr_eos_force() };
    }
}

/// [`msg_clr_eos`], `'shortmess'` and `:silent` notwithstanding.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_clr_eos_force() {
    if ui_has(kUIMessages) {
        return;
    }
    let (msg_startcol, msg_endcol) = if cmdmsg_rl.get() {
        (0, Columns.get() - msg_col.get())
    } else {
        (msg_col.get(), Columns.get())
    };

    // Avoid clearing the line the grid is about to be moved off.
    if msg_grid_ref().is_allocated() && msg_row.get() < msg_grid_pos.get() {
        unsafe { msg_grid_validate() };
        if msg_row.get() < msg_grid_pos.get() {
            msg_row.set(msg_grid_pos.get());
        }
    }

    clear_msg_area(msg_row.get(), msg_row.get() + 1, msg_startcol, msg_endcol);
    clear_msg_area(msg_row.get() + 1, Rows.get(), 0, Columns.get());

    redraw_cmdline.set(true);
    if msg_row.get() < Rows.get() - 1 || msg_col.get() == 0 {
        clear_cmdline.set(false);
        mode_displayed.set(false);
        cmdline_was_last_drawn.set(false);
    }
}

/// Clear the command line.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_clr_cmdline() {
    msg_row.set(cmdline_row.get());
    msg_col.set(0);
    unsafe { msg_clr_eos_force() };
}
