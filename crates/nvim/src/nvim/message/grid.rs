//! The message grid, and scrolling it.
//!
//! Messages are drawn onto their own grid ([`msg_grid_validate`] sizes it and
//! hands it to the compositor) floating over the bottom of the screen, so the
//! window text underneath survives a message that scrolls. [`msg_scroll_up`]
//! and [`msg_scroll_flush`] move that grid; [`msg_reset_scroll`] puts it back
//! once the message is gone.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn msg_id_exists(mut id: int64_t) -> bool {
    return id > 0 as int64_t && id < msg_id_next.get();
}

pub(crate) unsafe extern "C" fn ui_ext_msg_set_pos(
    mut row: ::core::ffi::c_int,
    mut scrolled: bool,
) {
    unsafe {
        let mut buf: [::core::ffi::c_char; 32] = [0; 32];
        let mut size: size_t = schar_get(
            &raw mut buf as *mut ::core::ffi::c_char,
            (*curwin.get()).w_p_fcs_chars.msgsep,
        );
        ui_call_msg_set_pos(
            (*msg_grid.ptr()).handle as Integer,
            row as Integer,
            scrolled as Boolean,
            String_0 {
                data: &raw mut buf as *mut ::core::ffi::c_char,
                size: size,
            },
            (*msg_grid.ptr()).zindex as Integer,
            (*msg_grid.ptr()).comp_index as ::core::ffi::c_int as Integer,
        );
        (*msg_grid.ptr()).pending_comp_index_update = false_0 != 0;
    }
}

pub unsafe extern "C" fn msg_grid_set_pos(mut row: ::core::ffi::c_int, mut scrolled: bool) {
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

pub unsafe extern "C" fn msg_use_grid() -> bool {
    unsafe {
        return !(*default_grid.ptr()).chars.is_null() && !ui_has(kUIMessages);
    }
}

pub unsafe extern "C" fn msg_grid_validate() {
    unsafe {
        grid_assign_handle(msg_grid.ptr());
        let mut should_alloc: bool = msg_use_grid();
        let mut max_rows: ::core::ffi::c_int = Rows.get() - p_ch.get() as ::core::ffi::c_int;
        if should_alloc as ::core::ffi::c_int != 0
            && ((*msg_grid.ptr()).rows != Rows.get()
                || (*msg_grid.ptr()).cols != Columns.get()
                || (*msg_grid.ptr()).chars.is_null())
        {
            grid_alloc(
                msg_grid.ptr(),
                Rows.get(),
                Columns.get(),
                false_0 != 0,
                true_0 != 0,
            );
            (*msg_grid.ptr()).zindex = kZIndexMessages as ::core::ffi::c_int;
            xfree((*msg_grid.ptr()).dirty_col as *mut ::core::ffi::c_void);
            (*msg_grid.ptr()).dirty_col = xcalloc(
                Rows.get() as size_t,
                ::core::mem::size_of::<::core::ffi::c_int>(),
            ) as *mut ::core::ffi::c_int;
            let mut pos: ::core::ffi::c_int = if State.get() & MODE_ASKMORE != 0 {
                0 as ::core::ffi::c_int
            } else if max_rows - msg_scrolled.get() > 0 as ::core::ffi::c_int {
                max_rows - msg_scrolled.get()
            } else {
                0 as ::core::ffi::c_int
            };
            (*msg_grid.ptr()).throttled = false_0 != 0;
            msg_grid_set_pos(pos, msg_scrolled.get() != 0);
            ui_comp_put_grid(
                msg_grid.ptr(),
                pos,
                0 as ::core::ffi::c_int,
                (*msg_grid.ptr()).rows,
                (*msg_grid.ptr()).cols,
                false_0 != 0,
                true_0 != 0,
            );
            ui_call_grid_resize(
                (*msg_grid.ptr()).handle as Integer,
                (*msg_grid.ptr()).cols as Integer,
                (*msg_grid.ptr()).rows as Integer,
            );
            msg_scrolled_at_flush.set(msg_scrolled.get());
            (*msg_grid.ptr()).mouse_enabled = false_0 != 0;
            (*msg_grid_adj.ptr()).target = msg_grid.ptr();
        } else if !should_alloc && !(*msg_grid.ptr()).chars.is_null() {
            ui_comp_remove_grid(msg_grid.ptr());
            grid_free(msg_grid.ptr());
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*msg_grid.ptr()).dirty_col as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
            ui_call_grid_destroy((*msg_grid.ptr()).handle as Integer);
            (*msg_grid.ptr()).throttled = false_0 != 0;
            (*msg_grid_adj.ptr()).row_offset = 0 as ::core::ffi::c_int;
            (*msg_grid_adj.ptr()).target = default_grid.ptr();
            redraw_cmdline.set(true_0 != 0);
        } else if !(*msg_grid.ptr()).chars.is_null()
            && msg_scrolled.get() == 0
            && msg_grid_pos.get() != max_rows
        {
            let mut diff: ::core::ffi::c_int = msg_grid_pos.get() - max_rows;
            msg_grid_set_pos(max_rows, false_0 != 0);
            if diff > 0 as ::core::ffi::c_int {
                grid_clear(
                    msg_grid_adj.ptr(),
                    Rows.get() - diff,
                    Rows.get(),
                    0 as ::core::ffi::c_int,
                    Columns.get(),
                    *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
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

pub unsafe extern "C" fn msg_line_flush() {
    unsafe {
        if cmdmsg_rl.get() {
            grid_line_mirror((*msg_grid.ptr()).cols);
        }
        grid_line_flush_if_valid_row();
    }
}

pub unsafe extern "C" fn msg_cursor_goto(mut row: ::core::ffi::c_int, mut col: ::core::ffi::c_int) {
    unsafe {
        if cmdmsg_rl.get() {
            col = Columns.get() - 1 as ::core::ffi::c_int - col;
        }
        let mut grid: *mut ScreenGrid = grid_adjust(msg_grid_adj.ptr(), &raw mut row, &raw mut col);
        ui_grid_cursor_goto((*grid).handle, row, col);
    }
}

pub unsafe extern "C" fn msg_scrollsize() -> ::core::ffi::c_int {
    return msg_scrolled.get()
        + p_ch.get() as ::core::ffi::c_int
        + (if p_ch.get() > 0 as OptInt || msg_scrolled.get() > 1 as ::core::ffi::c_int {
            1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        });
}

pub unsafe extern "C" fn msg_do_throttle() -> bool {
    unsafe {
        return msg_use_grid() as ::core::ffi::c_int != 0
            && rdb_flags.get()
                & kOptRdbFlagNothrottle as ::core::ffi::c_int as ::core::ffi::c_uint
                == 0;
    }
}

pub unsafe extern "C" fn msg_scroll_up(mut may_throttle: bool, mut zerocmd: bool) {
    unsafe {
        if may_throttle as ::core::ffi::c_int != 0 && msg_do_throttle() as ::core::ffi::c_int != 0 {
            (*msg_grid.ptr()).throttled = true_0 != 0;
        }
        msg_did_scroll.set(true_0 != 0);
        if msg_grid_pos.get() > 0 as ::core::ffi::c_int {
            msg_grid_set_pos(msg_grid_pos.get() - 1 as ::core::ffi::c_int, !zerocmd);
            if zerocmd as ::core::ffi::c_int != 0 && !(*msg_grid.ptr()).chars.is_null() {
                grid_clear_line(
                    msg_grid.ptr(),
                    *(*msg_grid.ptr())
                        .line_offset
                        .offset(0 as ::core::ffi::c_int as isize),
                    (*msg_grid.ptr()).cols,
                    false_0 != 0,
                );
            }
        } else {
            grid_del_lines(
                msg_grid.ptr(),
                0 as ::core::ffi::c_int,
                1 as ::core::ffi::c_int,
                (*msg_grid.ptr()).rows,
                0 as ::core::ffi::c_int,
                (*msg_grid.ptr()).cols,
            );
            memmove(
                (*msg_grid.ptr()).dirty_col as *mut ::core::ffi::c_void,
                (*msg_grid.ptr())
                    .dirty_col
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *const ::core::ffi::c_void,
                (((*msg_grid.ptr()).rows - 1 as ::core::ffi::c_int) as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
            );
            *(*msg_grid.ptr())
                .dirty_col
                .offset(((*msg_grid.ptr()).rows - 1 as ::core::ffi::c_int) as isize) =
                0 as ::core::ffi::c_int;
        }
        grid_clear(
            msg_grid_adj.ptr(),
            Rows.get() - 1 as ::core::ffi::c_int,
            Rows.get(),
            0 as ::core::ffi::c_int,
            Columns.get(),
            *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
        );
    }
}

pub unsafe extern "C" fn msg_scroll_flush() {
    unsafe {
        if (*msg_grid.ptr()).throttled {
            (*msg_grid.ptr()).throttled = false_0 != 0;
            let mut pos_delta: ::core::ffi::c_int =
                msg_grid_pos_at_flush.get() - msg_grid_pos.get();
            '_c2rust_label: {
                if pos_delta >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"pos_delta >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2689 as ::core::ffi::c_uint,
                        b"void msg_scroll_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut delta: ::core::ffi::c_int =
                if msg_scrolled.get() - msg_scrolled_at_flush.get() < (*msg_grid.ptr()).rows {
                    msg_scrolled.get() - msg_scrolled_at_flush.get()
                } else {
                    (*msg_grid.ptr()).rows
                };
            if pos_delta > 0 as ::core::ffi::c_int {
                ui_ext_msg_set_pos(msg_grid_pos.get(), true_0 != 0);
            }
            let mut to_scroll: ::core::ffi::c_int =
                delta - pos_delta - msg_grid_scroll_discount.get();
            '_c2rust_label_0: {
                if to_scroll >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"to_scroll >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2697 as ::core::ffi::c_uint,
                        b"void msg_scroll_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            if to_scroll > 0 as ::core::ffi::c_int && msg_grid_pos.get() == 0 as ::core::ffi::c_int
            {
                ui_call_grid_scroll(
                    (*msg_grid.ptr()).handle as Integer,
                    0 as Integer,
                    Rows.get() as Integer,
                    0 as Integer,
                    Columns.get() as Integer,
                    to_scroll as Integer,
                    0 as Integer,
                );
            }
            let mut i: ::core::ffi::c_int = if Rows.get()
                - (if delta > 1 as ::core::ffi::c_int {
                    delta
                } else {
                    1 as ::core::ffi::c_int
                })
                > 0 as ::core::ffi::c_int
            {
                Rows.get()
                    - (if delta > 1 as ::core::ffi::c_int {
                        delta
                    } else {
                        1 as ::core::ffi::c_int
                    })
            } else {
                0 as ::core::ffi::c_int
            };
            while i < Rows.get() {
                let mut row: ::core::ffi::c_int = i - msg_grid_pos.get();
                '_c2rust_label_1: {
                    if row >= 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                            b"row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/message.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            2707 as ::core::ffi::c_uint,
                            b"void msg_scroll_flush(void)\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                ui_line(
                    msg_grid.ptr(),
                    row,
                    false_0 != 0,
                    0 as ::core::ffi::c_int,
                    *(*msg_grid.ptr()).dirty_col.offset(row as isize),
                    (*msg_grid.ptr()).cols,
                    *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
                    false_0 != 0,
                );
                *(*msg_grid.ptr()).dirty_col.offset(row as isize) = 0 as ::core::ffi::c_int;
                i += 1;
            }
        }
        msg_scrolled_at_flush.set(msg_scrolled.get());
        msg_grid_scroll_discount.set(0 as ::core::ffi::c_int);
        msg_grid_pos_at_flush.set(msg_grid_pos.get());
    }
}

pub unsafe extern "C" fn msg_reset_scroll() {
    unsafe {
        if ui_has(kUIMessages) {
            return;
        }
        (*msg_grid.ptr()).throttled = false_0 != 0;
        msg_grid_set_pos(Rows.get() - p_ch.get() as ::core::ffi::c_int, false_0 != 0);
        clear_cmdline.set(true_0 != 0);
        if !(*msg_grid.ptr()).chars.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i
                < (if msg_scrollsize() < (*msg_grid.ptr()).rows {
                    msg_scrollsize()
                } else {
                    (*msg_grid.ptr()).rows
                })
            {
                grid_clear_line(
                    msg_grid.ptr(),
                    *(*msg_grid.ptr()).line_offset.offset(i as isize),
                    (*msg_grid.ptr()).cols,
                    false_0 != 0,
                );
                i += 1;
            }
        }
        msg_scrolled.set(0 as ::core::ffi::c_int);
        msg_scrolled_at_flush.set(0 as ::core::ffi::c_int);
        msg_grid_scroll_discount.set(0 as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn msg_ui_refresh() {
    unsafe {
        if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0 && !(*msg_grid.ptr()).chars.is_null() {
            ui_call_grid_resize(
                (*msg_grid.ptr()).handle as Integer,
                (*msg_grid.ptr()).cols as Integer,
                (*msg_grid.ptr()).rows as Integer,
            );
            ui_ext_msg_set_pos(msg_grid_pos.get(), msg_scrolled.get() != 0);
        }
    }
}

pub unsafe extern "C" fn msg_ui_flush() {
    unsafe {
        if ui_has(kUIMultigrid) as ::core::ffi::c_int != 0
            && !(*msg_grid.ptr()).chars.is_null()
            && (*msg_grid.ptr()).pending_comp_index_update as ::core::ffi::c_int != 0
        {
            ui_ext_msg_set_pos(msg_grid_pos.get(), msg_scrolled.get() != 0);
        }
    }
}

pub(crate) unsafe extern "C" fn inc_msg_scrolled() {
    unsafe {
        if *get_vim_var_str(VV_SCROLLSTART) as ::core::ffi::c_int == NUL {
            let mut p: String_0 = String_0 {
                data: (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
                size: 0,
            };
            let mut tofree: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            if p.data.is_null() {
                p = cstr_as_string(gettext(b"Unknown\0".as_ptr() as *const ::core::ffi::c_char));
            } else {
                let mut tofreesize: size_t = strlen(p.data).wrapping_add(40 as size_t);
                tofree = xmalloc(tofreesize) as *mut ::core::ffi::c_char;
                p.size = vim_snprintf_safelen(
                    tofree,
                    tofreesize,
                    gettext(b"%s line %ld\0".as_ptr() as *const ::core::ffi::c_char),
                    p.data,
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum as int64_t,
                );
                p.data = tofree;
            }
            set_vim_var_string(VV_SCROLLSTART, p.data, p.size as ptrdiff_t);
            xfree(tofree as *mut ::core::ffi::c_void);
        }
        (*msg_scrolled.ptr()) += 1;
        set_must_redraw(UPD_VALID);
    }
}

pub unsafe extern "C" fn msg_clr_eos() {
    unsafe {
        if msg_silent.get() == 0 as ::core::ffi::c_int {
            msg_clr_eos_force();
        }
    }
}

pub unsafe extern "C" fn msg_clr_eos_force() {
    unsafe {
        if ui_has(kUIMessages) {
            return;
        }
        let mut msg_startcol: ::core::ffi::c_int = if cmdmsg_rl.get() as ::core::ffi::c_int != 0 {
            0 as ::core::ffi::c_int
        } else {
            msg_col.get()
        };
        let mut msg_endcol: ::core::ffi::c_int = if cmdmsg_rl.get() as ::core::ffi::c_int != 0 {
            Columns.get() - msg_col.get()
        } else {
            Columns.get()
        };
        if !(*msg_grid.ptr()).chars.is_null() && msg_row.get() < msg_grid_pos.get() {
            msg_grid_validate();
            if msg_row.get() < msg_grid_pos.get() {
                msg_row.set(msg_grid_pos.get());
            }
        }
        grid_clear(
            msg_grid_adj.ptr(),
            msg_row.get(),
            msg_row.get() + 1 as ::core::ffi::c_int,
            msg_startcol,
            msg_endcol,
            *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
        );
        grid_clear(
            msg_grid_adj.ptr(),
            msg_row.get() + 1 as ::core::ffi::c_int,
            Rows.get(),
            0 as ::core::ffi::c_int,
            Columns.get(),
            *(*hl_attr_active.ptr()).offset(HLF_MSG as isize),
        );
        redraw_cmdline.set(true_0 != 0);
        if msg_row.get() < Rows.get() - 1 as ::core::ffi::c_int
            || msg_col.get() == 0 as ::core::ffi::c_int
        {
            clear_cmdline.set(false_0 != 0);
            mode_displayed.set(false_0 != 0);
            cmdline_was_last_drawn.set(false_0 != 0);
        }
    }
}

pub unsafe extern "C" fn msg_clr_cmdline() {
    unsafe {
        msg_row.set(cmdline_row.get());
        msg_col.set(0 as ::core::ffi::c_int);
        msg_clr_eos_force();
    }
}
