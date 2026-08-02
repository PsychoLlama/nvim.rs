//! Allocating the screen and reacting to a size change.
//!
//! [`default_grid_alloc`] is the only place `default_grid` is (re)allocated;
//! [`screenclear`] blanks it and tells every window and the message area to draw
//! themselves again. [`screen_resize`] is what the outside world calls when the
//! terminal changed size: it clamps the new size ([`check_screensize`]), re-lays
//! out the windows, and fires `VimResized` -- up to three times, because an
//! autocommand may change `'lines'` or `'columns'` again.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn default_grid_alloc() -> bool {
    unsafe {
        static resizing: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
        if resizing.get() {
            return false_0 != 0;
        }
        resizing.set(true_0 != 0);
        if !(*default_grid.ptr()).chars.is_null()
            && Rows.get() == (*default_grid.ptr()).rows
            && Columns.get() == (*default_grid.ptr()).cols
            || Rows.get() == 0 as ::core::ffi::c_int
            || Columns.get() == 0 as ::core::ffi::c_int
        {
            resizing.set(false_0 != 0);
            return false_0 != 0;
        }
        grid_alloc(
            default_grid.ptr(),
            Rows.get(),
            Columns.get(),
            true_0 != 0,
            true_0 != 0,
        );
        stl_clear_click_defs(tab_page_click_defs.get(), tab_page_click_defs_size.get());
        tab_page_click_defs.set(stl_alloc_click_defs(
            tab_page_click_defs.get(),
            Columns.get(),
            tab_page_click_defs_size.ptr(),
        ));
        (*default_grid.ptr()).comp_height = Rows.get();
        (*default_grid.ptr()).comp_width = Columns.get();
        (*default_grid.ptr()).handle = DEFAULT_GRID_HANDLE as handle_T;
        resizing.set(false_0 != 0);
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn screenclear() {
    unsafe {
        msg_check_for_delay(false_0 != 0);
        if starting.get() == NO_SCREEN || (*default_grid.ptr()).chars.is_null() {
            return;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*default_grid.ptr()).rows {
            grid_clear_line(
                default_grid.ptr(),
                *(*default_grid.ptr()).line_offset.offset(i as isize),
                (*default_grid.ptr()).cols,
                true_0 != 0,
            );
            i += 1;
        }
        ui_call_grid_clear(1 as Integer);
        ui_comp_set_screen_valid(true_0 != 0);
        ns_hl_fast.set(-1 as ::core::ffi::c_int as NS);
        clear_cmdline.set(false_0 != 0);
        mode_displayed.set(false_0 != 0);
        redraw_all_later(UPD_NOT_VALID);
        cmdline_was_last_drawn.set(false_0 != 0);
        redraw_cmdline.set(true_0 != 0);
        redraw_tabline.set(true_0 != 0);
        redraw_popupmenu.set(true_0 != 0);
        pum_invalidate();
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_floating {
                (*wp).w_redr_type = UPD_CLEAR;
            }
            wp = (*wp).w_next;
        }
        if must_redraw.get() == UPD_CLEAR {
            must_redraw.set(UPD_NOT_VALID);
        }
        compute_cmdrow();
        msg_row.set(cmdline_row.get());
        msg_col.set(0 as ::core::ffi::c_int);
        msg_reset_scroll();
        msg_didany.set(false_0 != 0);
        msg_didout.set(false_0 != 0);
        if *(*hl_attr_active.ptr()).offset(HLF_MSG as isize) > 0 as ::core::ffi::c_int
            && msg_use_grid() as ::core::ffi::c_int != 0
            && !(*msg_grid.ptr()).chars.is_null()
        {
            grid_invalidate(msg_grid.ptr());
            msg_grid_validate();
            msg_grid_invalid.set(false_0 != 0);
            clear_cmdline.set(true_0 != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn cmdline_number_prompt() -> bool {
    unsafe {
        return !ui_has(kUIMessages)
            && State.get() & MODE_CMDLINE != 0
            && !(*get_cmdline_info()).mouse_used.is_null();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn screen_resize(
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
) {
    unsafe {
        if updating_screen.get() as ::core::ffi::c_int != 0
            || resizing_screen.get() as ::core::ffi::c_int != 0
            || cmdline_number_prompt() as ::core::ffi::c_int != 0
        {
            return;
        }
        if width < 0 as ::core::ffi::c_int || height < 0 as ::core::ffi::c_int {
            return;
        }
        if State.get() == MODE_HITRETURN || State.get() == MODE_SETWSIZE {
            State.set(MODE_SETWSIZE);
            return;
        }
        resizing_screen.set(true_0 != 0);
        Rows.set(height);
        Columns.set(width);
        check_screensize();
        if !ui_has(kUIMessages) {
            let mut max_p_ch: ::core::ffi::c_int =
                Rows.get() - min_rows(curtab.get()) + 1 as ::core::ffi::c_int;
            if p_ch.get() > 0 as OptInt && p_ch.get() > max_p_ch as OptInt {
                p_ch.set(
                    (if max_p_ch > 1 as ::core::ffi::c_int {
                        max_p_ch
                    } else {
                        1 as ::core::ffi::c_int
                    }) as OptInt,
                );
                (*curtab.get()).tp_ch_used = p_ch.get();
            }
            let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
            while !tp.is_null() {
                if tp != curtab.get() {
                    let mut max_tp_ch: ::core::ffi::c_int =
                        Rows.get() - min_rows(tp as *mut tabpage_T) + 1 as ::core::ffi::c_int;
                    if (*tp).tp_ch_used > 0 as OptInt && (*tp).tp_ch_used > max_tp_ch as OptInt {
                        (*tp).tp_ch_used = (if max_tp_ch > 1 as ::core::ffi::c_int {
                            max_tp_ch
                        } else {
                            1 as ::core::ffi::c_int
                        }) as OptInt;
                    }
                }
                tp = (*tp).tp_next as *mut tabpage_T;
            }
        }
        height = Rows.get();
        width = Columns.get();
        p_lines.set(Rows.get() as OptInt);
        p_columns.set(Columns.get() as OptInt);
        ui_call_grid_resize(1 as Integer, width as Integer, height as Integer);
        let mut retry_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        resizing_autocmd.set(true_0 != 0);
        while default_grid_alloc() {
            ui_comp_set_screen_valid(false_0 != 0);
            if !(*msg_grid.ptr()).chars.is_null() {
                msg_grid_invalid.set(true_0 != 0);
            }
            (*RedrawingDisabled.ptr()) += 1;
            win_new_screensize();
            comp_col();
            (*RedrawingDisabled.ptr()) -= 1;
            retry_count += 1;
            if retry_count > 3 as ::core::ffi::c_int {
                break;
            }
            apply_autocmds(
                EVENT_VIMRESIZED,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                false_0 != 0,
                curbuf.get(),
            );
        }
        resizing_autocmd.set(false_0 != 0);
        redraw_all_later(UPD_CLEAR);
        if State.get() != MODE_ASKMORE && State.get() != MODE_EXTERNCMD {
            screenclear();
        }
        if starting.get() != NO_SCREEN {
            maketitle();
            changed_line_abv_curs();
            invalidate_botline_win(curwin.get());
            if State.get() == MODE_ASKMORE
                || State.get() == MODE_EXTERNCMD
                || exmode_active.get() as ::core::ffi::c_int != 0
                || State.get() & MODE_CMDLINE != 0
                    && (*get_cmdline_info()).one_key as ::core::ffi::c_int != 0
            {
                if State.get() & MODE_CMDLINE != 0 {
                    update_screen();
                }
                if !(*msg_grid.ptr()).chars.is_null() {
                    msg_grid_validate();
                }
                ui_comp_set_screen_valid(true_0 != 0);
                repeat_message();
            } else {
                if (*curwin.get()).w_onebuf_opt.wo_scb != 0 {
                    do_check_scrollbind(true_0 != 0);
                }
                if State.get() & MODE_CMDLINE != 0 {
                    redraw_popupmenu.set(false_0 != 0);
                    update_screen();
                    redrawcmdline();
                    if pum_drawn() {
                        cmdline_pum_display(false_0 != 0);
                    }
                } else {
                    update_topline(curwin.get());
                    if pum_drawn() {
                        redraw_popupmenu.set(false_0 != 0);
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
        resizing_screen.set(false_0 != 0);
    }
}

pub unsafe extern "C" fn check_screensize() {
    unsafe {
        Rows.set(
            if (if Rows.get() > min_rows_for_all_tabpages() {
                Rows.get()
            } else {
                min_rows_for_all_tabpages()
            }) < 1000 as ::core::ffi::c_int
            {
                if Rows.get() > min_rows_for_all_tabpages() {
                    Rows.get()
                } else {
                    min_rows_for_all_tabpages()
                }
            } else {
                1000 as ::core::ffi::c_int
            },
        );
        Columns.set(
            if (if Columns.get() > MIN_COLUMNS as ::core::ffi::c_int {
                Columns.get()
            } else {
                MIN_COLUMNS as ::core::ffi::c_int
            }) < 10000 as ::core::ffi::c_int
            {
                if Columns.get() > MIN_COLUMNS as ::core::ffi::c_int {
                    Columns.get()
                } else {
                    MIN_COLUMNS as ::core::ffi::c_int
                }
            } else {
                10000 as ::core::ffi::c_int
            },
        );
    }
}
