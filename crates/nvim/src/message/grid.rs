//! The message grid, and scrolling it.
//!
//! Messages are drawn onto their own grid ([`msg_grid_validate`] sizes it and
//! hands it to the compositor) floating over the bottom of the screen, so the
//! window text underneath survives a message that scrolls. [`msg_scroll_up`]
//! and [`msg_scroll_flush`] move that grid; [`msg_reset_scroll`] puts it back
//! once the message is gone.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use core::ffi::{c_char, c_int, c_uint, c_void};
use core::ptr;

/// Has `id` been handed out as a message id?
pub fn msg_id_exists(id: int64_t) -> bool {
    id > 0 && id < msg_id_next.get()
}

/// Tell the UI where the message grid now sits.
///
/// # Safety
/// The message grid must be allocated and `curwin` valid.
pub(crate) unsafe fn ui_ext_msg_set_pos(row: c_int, scrolled: bool) {
    unsafe {
        let mut sep = [0 as c_char; 32];
        let size = schar_get(sep.as_mut_ptr(), (*curwin.get()).w_p_fcs_chars.msgsep);
        ui_call_msg_set_pos(
            (*msg_grid.ptr()).handle.into(),
            row.into(),
            scrolled,
            String_0::from_raw_parts(sep.as_mut_ptr(), size),
            (*msg_grid.ptr()).zindex.into(),
            (*msg_grid.ptr()).comp_index as Integer,
        );
        (*msg_grid.ptr()).pending_comp_index_update = false;
    }
}

/// Move the message grid to `row`, telling the UI unless output is throttled.
///
/// # Safety
/// The message grid must be initialised.
pub unsafe fn msg_grid_set_pos(row: c_int, scrolled: bool) {
    unsafe {
        if !(*msg_grid.ptr()).throttled {
            ui_ext_msg_set_pos(row, scrolled);
            msg_grid_pos_at_flush.set(row);
        }
        msg_grid_pos.set(row);
        if !(*msg_grid.ptr()).chars.is_null() {
            (*msg_grid_adj.ptr()).row_offset = -row;
        }
    }
}

/// Are messages drawn on a grid at all?
///
/// They are not before the first redraw, and not under `ext_messages`.
///
/// # Safety
/// Only that the default grid is initialised.
pub unsafe fn msg_use_grid() -> bool {
    unsafe { !(*default_grid.ptr()).chars.is_null() && !ui_has(kUIMessages) }
}

/// Allocate, resize, reposition or free the message grid to match the screen.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_grid_validate() {
    unsafe {
        grid_assign_handle(msg_grid.ptr());
        let should_alloc = msg_use_grid();
        let max_rows = Rows.get() - p_ch.get() as c_int;

        if should_alloc
            && ((*msg_grid.ptr()).rows != Rows.get()
                || (*msg_grid.ptr()).cols != Columns.get()
                || (*msg_grid.ptr()).chars.is_null())
        {
            // Force a valid screen size.
            grid_alloc(msg_grid.ptr(), Rows.get(), Columns.get(), false, true);
            (*msg_grid.ptr()).zindex = kZIndexMessages as c_int;
            xfree((*msg_grid.ptr()).dirty_col.cast());
            (*msg_grid.ptr()).dirty_col =
                xcalloc(Rows.get() as size_t, ::core::mem::size_of::<c_int>()).cast();

            // Tell the compositor to put the grid at the bottom, or at the top
            // while the pager owns the screen.
            let pos = if State.get() & MODE_ASKMORE != 0 {
                0
            } else {
                (max_rows - msg_scrolled.get()).max(0)
            };
            (*msg_grid.ptr()).throttled = false; // don't throttle in 'cmdheight' area
            msg_grid_set_pos(pos, msg_scrolled.get() != 0);
            ui_comp_put_grid(
                msg_grid.ptr(),
                pos,
                0,
                (*msg_grid.ptr()).rows,
                (*msg_grid.ptr()).cols,
                false,
                true,
            );
            ui_call_grid_resize(
                (*msg_grid.ptr()).handle.into(),
                (*msg_grid.ptr()).cols.into(),
                (*msg_grid.ptr()).rows.into(),
            );

            msg_scrolled_at_flush.set(msg_scrolled.get());
            (*msg_grid.ptr()).mouse_enabled = false;
            (*msg_grid_adj.ptr()).target = msg_grid.ptr();
        } else if !should_alloc && !(*msg_grid.ptr()).chars.is_null() {
            // Note: we run this both on moving to ext_messages, and on
            // resizing the screen while ext_messages is active.
            ui_comp_remove_grid(msg_grid.ptr());
            grid_free(msg_grid.ptr());
            xfree((*msg_grid.ptr()).dirty_col.cast::<c_void>());
            (*msg_grid.ptr()).dirty_col = ptr::null_mut();
            ui_call_grid_destroy((*msg_grid.ptr()).handle.into());
            (*msg_grid.ptr()).throttled = false;
            (*msg_grid_adj.ptr()).row_offset = 0;
            (*msg_grid_adj.ptr()).target = default_grid.ptr();
            redraw_cmdline.set(true);
        } else if !(*msg_grid.ptr()).chars.is_null()
            && msg_scrolled.get() == 0
            && msg_grid_pos.get() != max_rows
        {
            let diff = msg_grid_pos.get() - max_rows;
            msg_grid_set_pos(max_rows, false);
            if diff > 0 {
                grid_clear(
                    msg_grid_adj.ptr(),
                    Rows.get() - diff,
                    Rows.get(),
                    0,
                    Columns.get(),
                    hl_attr(HLF_MSG as c_int),
                );
            }
        }

        if !(*msg_grid.ptr()).chars.is_null()
            && msg_scrolled.get() == 0
            && cmdline_row.get() < msg_grid_pos.get()
        {
            cmdline_row.set(msg_grid_pos.get());
        }
    }
}

/// Send the line being built to the UI, mirrored if `'rightleft'` applies.
///
/// # Safety
/// A grid line must be under construction.
pub unsafe fn msg_line_flush() {
    unsafe {
        if cmdmsg_rl.get() {
            grid_line_mirror((*msg_grid.ptr()).cols);
        }
        grid_line_flush_if_valid_row();
    }
}

/// Put the cursor at `row`/`col` of the message area.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_cursor_goto(row: c_int, mut col: c_int) {
    unsafe {
        let mut row = row;
        if cmdmsg_rl.get() {
            col = Columns.get() - 1 - col;
        }
        let grid = grid_adjust(msg_grid_adj.ptr(), &raw mut row, &raw mut col);
        ui_grid_cursor_goto((*grid).handle, row, col);
    }
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
    unsafe {
        if may_throttle && msg_do_throttle() {
            (*msg_grid.ptr()).throttled = true;
        }
        msg_did_scroll.set(true);
        if msg_grid_pos.get() > 0 {
            msg_grid_set_pos(msg_grid_pos.get() - 1, !zerocmd);
            if zerocmd && !(*msg_grid.ptr()).chars.is_null() {
                // When zerocmd is true, we're scrolling the first line of
                // msg_grid onto the screen; it must be cleared first.
                grid_clear_line(
                    msg_grid.ptr(),
                    *(*msg_grid.ptr()).line_offset,
                    (*msg_grid.ptr()).cols,
                    false,
                );
            }
        } else {
            grid_del_lines(
                msg_grid.ptr(),
                0,
                1,
                (*msg_grid.ptr()).rows,
                0,
                (*msg_grid.ptr()).cols,
            );
            // The dirty columns move up with the lines they describe.
            ptr::copy(
                (*msg_grid.ptr()).dirty_col.add(1),
                (*msg_grid.ptr()).dirty_col,
                ((*msg_grid.ptr()).rows - 1) as usize,
            );
            *(*msg_grid.ptr())
                .dirty_col
                .add(((*msg_grid.ptr()).rows - 1) as usize) = 0;
        }
        // Ensure the message area is cleared to the default background.
        grid_clear(
            msg_grid_adj.ptr(),
            Rows.get() - 1,
            Rows.get(),
            0,
            Columns.get(),
            hl_attr(HLF_MSG as c_int),
        );
    }
}

/// Send everything a throttled run of messages accumulated, as one scroll
/// plus the dirty part of each line.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_scroll_flush() {
    unsafe {
        if (*msg_grid.ptr()).throttled {
            (*msg_grid.ptr()).throttled = false;
            let pos_delta = msg_grid_pos_at_flush.get() - msg_grid_pos.get();
            debug_assert!(pos_delta >= 0);
            let delta =
                (msg_scrolled.get() - msg_scrolled_at_flush.get()).min((*msg_grid.ptr()).rows);

            if pos_delta > 0 {
                ui_ext_msg_set_pos(msg_grid_pos.get(), true);
            }

            let to_scroll = delta - pos_delta - msg_grid_scroll_discount.get();
            debug_assert!(to_scroll >= 0);

            // No scrolling to do while the grid is still moving down: the
            // repositioning above already showed the new lines.
            if to_scroll > 0 && msg_grid_pos.get() == 0 {
                ui_call_grid_scroll(
                    (*msg_grid.ptr()).handle.into(),
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
                ui_line(
                    msg_grid.ptr(),
                    row,
                    false,
                    0,
                    *(*msg_grid.ptr()).dirty_col.add(row as usize),
                    (*msg_grid.ptr()).cols,
                    hl_attr(HLF_MSG as c_int),
                    false,
                );
                *(*msg_grid.ptr()).dirty_col.add(row as usize) = 0;
                i += 1;
            }
        }
        msg_scrolled_at_flush.set(msg_scrolled.get());
        msg_grid_scroll_discount.set(0);
        msg_grid_pos_at_flush.set(msg_grid_pos.get());
    }
}

/// The messages are gone: put the grid back under `'cmdheight'` and clear it.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_reset_scroll() {
    unsafe {
        if ui_has(kUIMessages) {
            // TODO(bfredl): some duplicate logic with update_screen(). Later
            // on we should properly disentangle message clear with full screen
            // redraw.
            return;
        }
        (*msg_grid.ptr()).throttled = false;
        // TODO(bfredl): calculate the conflict in the compositor instead.
        msg_grid_set_pos(Rows.get() - p_ch.get() as c_int, false);
        clear_cmdline.set(true);
        if !(*msg_grid.ptr()).chars.is_null() {
            // The bound is re-evaluated each time round, as upstream does.
            let mut i = 0;
            while i < msg_scrollsize().min((*msg_grid.ptr()).rows) {
                grid_clear_line(
                    msg_grid.ptr(),
                    *(*msg_grid.ptr()).line_offset.add(i as usize),
                    (*msg_grid.ptr()).cols,
                    false,
                );
                i += 1;
            }
        }
        msg_scrolled.set(0);
        msg_scrolled_at_flush.set(0);
        msg_grid_scroll_discount.set(0);
    }
}

/// The UI reattached or resized: restate the grid's size and position.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_ui_refresh() {
    unsafe {
        if ui_has(kUIMultigrid) && !(*msg_grid.ptr()).chars.is_null() {
            ui_call_grid_resize(
                (*msg_grid.ptr()).handle.into(),
                (*msg_grid.ptr()).cols.into(),
                (*msg_grid.ptr()).rows.into(),
            );
            ui_ext_msg_set_pos(msg_grid_pos.get(), msg_scrolled.get() != 0);
        }
    }
}

/// The compositor restacked the grid: tell the UI its new position.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_ui_flush() {
    unsafe {
        if ui_has(kUIMultigrid)
            && !(*msg_grid.ptr()).chars.is_null()
            && (*msg_grid.ptr()).pending_comp_index_update
        {
            ui_ext_msg_set_pos(msg_grid_pos.get(), msg_scrolled.get() != 0);
        }
    }
}

/// One more line of messages has scrolled off; remember where it started.
///
/// # Safety
/// The exec stack must be non-empty. See [`sourcing_top`].
pub(crate) unsafe fn inc_msg_scrolled() {
    unsafe {
        if *get_vim_var_str(Vv::Scrollstart) == 0 {
            // v:scrollstart is empty: set it to the script/function name and
            // line number the scrolling started at.
            let mut p = String_0::from_raw_parts(sourcing_top().es_name, 0);
            let mut tofree: *mut c_char = ptr::null_mut();
            if p.data().is_null() {
                p = cstr_as_string(gettext(c"Unknown".as_ptr()));
            } else {
                let tofreesize = strlen(p.data()) + 40;
                tofree = xmalloc(tofreesize).cast();
                p.set_len(vim_snprintf_safelen(
                    tofree,
                    tofreesize,
                    gettext(c"%s line %ld".as_ptr()),
                    p.data(),
                    sourcing_top().es_lnum as int64_t,
                ));
                p.set_data(tofree);
            }
            set_vim_var_string(Vv::Scrollstart, p.data(), p.len() as ptrdiff_t);
            xfree(tofree.cast());
        }
        msg_scrolled.set(msg_scrolled.get() + 1);
        set_must_redraw(UPD_VALID);
    }
}

/// Clear from the cursor to the end of the message area, unless silenced.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_clr_eos() {
    unsafe {
        if msg_silent.get() == 0 {
            msg_clr_eos_force();
        }
    }
}

/// [`msg_clr_eos`], `'shortmess'` and `:silent` notwithstanding.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_clr_eos_force() {
    unsafe {
        if ui_has(kUIMessages) {
            return;
        }
        let (msg_startcol, msg_endcol) = if cmdmsg_rl.get() {
            (0, Columns.get() - msg_col.get())
        } else {
            (msg_col.get(), Columns.get())
        };

        // Avoid clearing the line the grid is about to be moved off.
        if !(*msg_grid.ptr()).chars.is_null() && msg_row.get() < msg_grid_pos.get() {
            msg_grid_validate();
            if msg_row.get() < msg_grid_pos.get() {
                msg_row.set(msg_grid_pos.get());
            }
        }

        grid_clear(
            msg_grid_adj.ptr(),
            msg_row.get(),
            msg_row.get() + 1,
            msg_startcol,
            msg_endcol,
            hl_attr(HLF_MSG as c_int),
        );
        grid_clear(
            msg_grid_adj.ptr(),
            msg_row.get() + 1,
            Rows.get(),
            0,
            Columns.get(),
            hl_attr(HLF_MSG as c_int),
        );

        redraw_cmdline.set(true);
        if msg_row.get() < Rows.get() - 1 || msg_col.get() == 0 {
            clear_cmdline.set(false);
            mode_displayed.set(false);
            cmdline_was_last_drawn.set(false);
        }
    }
}

/// Clear the command line.
///
/// # Safety
/// Only that the grids are initialised.
pub unsafe fn msg_clr_cmdline() {
    unsafe {
        msg_row.set(cmdline_row.get());
        msg_col.set(0);
        msg_clr_eos_force();
    }
}
