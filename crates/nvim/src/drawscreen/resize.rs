//! Allocating the screen and reacting to a size change.
//!
//! [`default_grid_alloc`] is the only place `default_grid` is (re)allocated;
//! [`screenclear`] blanks it and tells every window and the message area to draw
//! themselves again. [`screen_resize`] is what the outside world calls when the
//! terminal changed size: it clamps the new size ([`check_screensize`]), re-lays
//! out the windows, and fires `VimResized` -- up to three times, because an
//! autocommand may change `'lines'` or `'columns'` again.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::Suppress;

/// The largest screen this port will allocate, so that `Rows * Columns` cannot
/// overflow.
const MAX_ROWS: c_int = 1000;
const MAX_COLUMNS: c_int = 10000;

/// Resize `default_grid` to `Rows` x `Columns`, and answer whether it moved.
///
/// The allocation only happens when the size actually changed and both
/// dimensions are known: there is a window between setting `Rows`/`Columns` and
/// getting here, at startup and on a manual resize, so everything that indexes
/// the grid uses `default_grid.rows`/`.cols` rather than the globals.
pub unsafe fn default_grid_alloc() -> bool {
    // An out-of-memory message from inside the allocation redraws, which lands
    // back here; break the loop rather than recursing.
    static RESIZING: GlobalCell<bool> = GlobalCell::new(false);

    // SAFETY: `default_grid` is the editor's screen grid, on the main thread.
    unsafe {
        if RESIZING.get() {
            return false;
        }
        RESIZING.set(true);

        let grid = default_grid.ptr();
        let unchanged =
            !(*grid).chars.is_null() && Rows.get() == (*grid).rows && Columns.get() == (*grid).cols;
        if unchanged || Rows.get() == 0 || Columns.get() == 0 {
            RESIZING.set(false);
            return false;
        }

        // Allocates new arrays, moves the old lines across, clears the rest and
        // frees the old ones. On failure the arrays are left NULL rather than
        // at the old size, because the wrong size is a crash.
        grid_alloc(grid, Rows.get(), Columns.get(), true, true);

        stl_clear_click_defs(tab_page_click_defs.get(), tab_page_click_defs_size.get());
        tab_page_click_defs.set(stl_alloc_click_defs(
            tab_page_click_defs.get(),
            Columns.get(),
            tab_page_click_defs_size.ptr(),
        ));

        (*grid).comp_height = Rows.get();
        (*grid).comp_width = Columns.get();
        (*grid).handle = DEFAULT_GRID_HANDLE as handle_T;

        RESIZING.set(false);
        true
    }
}

/// Blank the screen and mark everything on it for redraw.
pub unsafe fn screenclear() {
    // SAFETY: the screen grid and the message grid, on the main thread.
    unsafe {
        msg_check_for_delay(false);

        if starting.get() == NO_SCREEN || (*default_grid.ptr()).chars.is_null() {
            return;
        }

        let grid = default_grid.ptr();
        for row in 0..(*grid).rows {
            grid_clear_line(
                grid,
                *(*grid).line_offset.add(row as usize),
                (*grid).cols,
                true,
            );
        }
        ui_call_grid_clear(1);
        ui_comp_set_screen_valid(true);

        ns_hl_fast.set(-1);

        clear_cmdline.set(false);
        mode_displayed.set(false);

        // Sets UPD_NOT_VALID on every window.
        redraw_all_later(UPD_NOT_VALID);
        cmdline_was_last_drawn.set(false);
        redraw_cmdline.set(true);
        redraw_tabline.set(true);
        redraw_popupmenu.set(true);
        pum_invalidate();
        for wp in windows_in_curtab() {
            if (*wp).w_floating {
                (*wp).w_redr_type = UPD_CLEAR;
            }
        }
        if must_redraw.get() == UPD_CLEAR {
            must_redraw.set(UPD_NOT_VALID); // no need to clear again
        }

        compute_cmdrow();
        msg_row.set(cmdline_row.get()); // put the cursor on the last line for messages
        msg_col.set(0);
        msg_reset_scroll(); // can't scroll back
        msg_didany.set(false);
        msg_didout.set(false);

        if *(*hl_attr_active.ptr()).add(HLF_MSG as usize) > 0
            && msg_use_grid()
            && !(*msg_grid.ptr()).chars.is_null()
        {
            grid_invalidate(msg_grid.ptr());
            msg_grid_validate();
            msg_grid_invalid.set(false);
            clear_cmdline.set(true);
        }
    }
}

/// Whether a `:` prompt that reads one key is on screen.
///
/// Unlike a cmdline "one_key" prompt, the message half of such a prompt is not
/// stored to be re-emitted, so it must not be cleared from the message grid --
/// which is why redrawing is refused entirely while one is up.
pub(crate) unsafe fn cmdline_number_prompt() -> bool {
    // SAFETY: the cmdline state is the editor's, on the main thread.
    unsafe {
        !ui_has(kUIMessages)
            && State.get() & MODE_CMDLINE != 0
            && !(*get_cmdline_info()).mouse_used.is_null()
    }
}

/// Set the dimensions of the Nvim application "screen".
#[unsafe(no_mangle)]
pub unsafe extern "C" fn screen_resize(width: c_int, height: c_int) {
    // SAFETY: the screen, the window layout and the autocommand machinery, all
    // on the main thread.
    unsafe {
        // Setting the window size can produce another window-changed signal.
        if updating_screen.get() || resizing_screen.get() || cmdline_number_prompt() {
            return;
        }
        if width < 0 || height < 0 {
            return;
        }
        if State.get() == MODE_HITRETURN || State.get() == MODE_SETWSIZE {
            State.set(MODE_SETWSIZE); // postpone the resize
            return;
        }

        resizing_screen.set(true);

        Rows.set(height);
        Columns.set(width);
        check_screensize();

        if !ui_has(kUIMessages) {
            // Clamp 'cmdheight' so the windows still fit, on this tab page and
            // on every other one.
            let max_p_ch = Rows.get() - min_rows(curtab.get()) + 1;
            if p_ch.get() > 0 && p_ch.get() > max_p_ch as OptInt {
                p_ch.set(max_p_ch.max(1) as OptInt);
                (*curtab.get()).tp_ch_used = p_ch.get();
            }
            let mut tp = first_tabpage.get();
            while !tp.is_null() {
                if tp != curtab.get() {
                    let max_tp_ch = Rows.get() - min_rows(tp) + 1;
                    if (*tp).tp_ch_used > 0 && (*tp).tp_ch_used > max_tp_ch as OptInt {
                        (*tp).tp_ch_used = max_tp_ch.max(1) as OptInt;
                    }
                }
                tp = (*tp).tp_next;
            }
        }

        // `check_screensize` may have clamped them.
        let height = Rows.get();
        let width = Columns.get();
        p_lines.set(height as OptInt);
        p_columns.set(width as OptInt);

        ui_call_grid_resize(1, width as Integer, height as Integer);

        // An autocommand may change Rows or Columns again, so the allocation is
        // retried -- but at most three times, or an autocommand that always
        // changes them never terminates.
        let mut retry_count = 0;
        resizing_autocmd.set(true);
        while default_grid_alloc() {
            // `win_new_screensize` recomputes float positions; tell the
            // compositor not to draw them yet.
            ui_comp_set_screen_valid(false);
            if !(*msg_grid.ptr()).chars.is_null() {
                msg_grid_invalid.set(true);
            }

            let redraw_off = Suppress::redraw();
            win_new_screensize(); // fit the windows in the new screen
            comp_col(); // recompute the shown-command and ruler columns
            drop(redraw_off);

            retry_count += 1;
            if retry_count > 3 {
                break;
            }
            apply_autocmds(
                EVENT_VIMRESIZED,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                false,
                curbuf.get(),
            );
        }
        resizing_autocmd.set(false);

        redraw_all_later(UPD_CLEAR);

        if State.get() != MODE_ASKMORE && State.get() != MODE_EXTERNCMD {
            screenclear();
        }

        if starting.get() != NO_SCREEN {
            maketitle();

            changed_line_abv_curs();
            invalidate_botline_win(curwin.get());

            // At a more prompt, running an external command, in Ex mode or at a
            // one-key cmdline prompt, only the cursor is repositioned; anywhere
            // else the screen is redrawn now.
            let deferred = State.get() == MODE_ASKMORE
                || State.get() == MODE_EXTERNCMD
                || exmode_active.get()
                || (State.get() & MODE_CMDLINE != 0 && (*get_cmdline_info()).one_key);
            if deferred {
                if State.get() & MODE_CMDLINE != 0 {
                    update_screen();
                }
                if !(*msg_grid.ptr()).chars.is_null() {
                    msg_grid_validate();
                }
                ui_comp_set_screen_valid(true);
                repeat_message();
            } else {
                if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
                    do_check_scrollbind(true);
                }
                if State.get() & MODE_CMDLINE != 0 {
                    // The pum is redrawn by `cmdline_pum_display` below, at the
                    // new position; keep `update_screen` from drawing it at the
                    // old one.
                    redraw_popupmenu.set(false);
                    update_screen();
                    redrawcmdline();
                    if pum_drawn() {
                        cmdline_pum_display(false);
                    }
                } else {
                    update_topline(curwin.get());
                    if pum_drawn() {
                        // Same again: `ins_compl_show_pum` wants the screen
                        // redrawn first, and the nested `update_screen` inside
                        // it must not draw the pum at its old position.
                        redraw_popupmenu.set(false);
                        ins_compl_show_pum();
                    }
                    update_screen();
                    if redrawing() {
                        setcursor();
                    }
                }
            }
            ui_flush();
        }

        resizing_screen.set(false);
    }
}

/// Clamp the screen size to something the editor can lay out and index.
pub unsafe fn check_screensize() {
    Rows.set(Rows.get().max(min_rows_for_all_tabpages()).min(MAX_ROWS));
    Columns.set(Columns.get().max(MIN_COLUMNS as c_int).min(MAX_COLUMNS));
}
