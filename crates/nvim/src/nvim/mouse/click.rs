//! What a click on a statusline, tabline or winbar item does --
//! `%@Func@` click definitions and the popup menu.
//!
//! [`call_click_def_func`] turns a recorded [`StlClickDefinition`] back into a
//! call: it rebuilds the `<LeftMouse>`-style modifier prefix, the click count
//! and the mouse position the handler is documented to receive, and either
//! switches or closes a tab page or calls the user's function.
//! [`do_popup`] is the `'mousemodel'=popup` right-click path, and
//! [`get_fpos_of_mouse`] the position lookup both of them and the `v:mouse_*`
//! variables share.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::drawscreen::{
    UPD_INVERTED, UPD_VALID, redraw_curbuf_later, setcursor, update_screen,
};
use crate::src::nvim::eval::call_vim_function;
use crate::src::nvim::eval::typval::tv_clear;
use crate::src::nvim::keycodes::Ctrl_V;
use crate::src::nvim::main::{
    Rows, VIsual, VIsual_active, VIsual_mode, curwin, mod_mask, mouse_col, mouse_grid, mouse_row,
    p_ch, p_mousem,
};
use crate::src::nvim::menu::show_popupmenu;
use crate::src::nvim::r#move::win_col_off;
use crate::src::nvim::os::libc::strcmp;
use crate::src::nvim::plines::{getvcol, getvcols};
use crate::src::nvim::pos::{lt, ltoreq};
use crate::src::nvim::types::{
    OptInt, StlClickDefinition, VAR_FIXED, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED,
    colnr_T, pos_T, typval_T, typval_vval_union, varnumber_T, win_T,
};
use crate::src::nvim::ui::ui_flush;
use crate::src::nvim::window::global_stl_height;

pub(crate) unsafe extern "C" fn call_click_def_func(
    mut click_defs: *mut StlClickDefinition,
    mut col: ::core::ffi::c_int,
    mut which_button: ::core::ffi::c_int,
) {
    unsafe {
        let mut c2rust_lvalue: [::core::ffi::c_char; 5] = [
            (if mod_mask.get() & MOD_MASK_SHIFT != 0 {
                's' as ::core::ffi::c_int
            } else {
                ' ' as ::core::ffi::c_int
            }) as ::core::ffi::c_char,
            (if mod_mask.get() & MOD_MASK_CTRL != 0 {
                'c' as ::core::ffi::c_int
            } else {
                ' ' as ::core::ffi::c_int
            }) as ::core::ffi::c_char,
            (if mod_mask.get() & MOD_MASK_ALT != 0 {
                'a' as ::core::ffi::c_int
            } else {
                ' ' as ::core::ffi::c_int
            }) as ::core::ffi::c_char,
            (if mod_mask.get() & MOD_MASK_META != 0 {
                'm' as ::core::ffi::c_int
            } else {
                ' ' as ::core::ffi::c_int
            }) as ::core::ffi::c_char,
            NUL as ::core::ffi::c_char,
        ];
        let mut argv: [typval_T; 4] = [
            typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_FIXED,
                vval: typval_vval_union {
                    v_number: (*click_defs.offset(col as isize)).tabnr as varnumber_T,
                },
            },
            typval_T {
                v_type: VAR_NUMBER,
                v_lock: VAR_FIXED,
                vval: typval_vval_union {
                    v_number: (if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_4CLICK {
                        4 as ::core::ffi::c_int
                    } else if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_3CLICK {
                        3 as ::core::ffi::c_int
                    } else if mod_mask.get() & MOD_MASK_MULTI_CLICK == MOD_MASK_2CLICK {
                        2 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    }) as varnumber_T,
                },
            },
            typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_FIXED,
                vval: typval_vval_union {
                    v_string: (if which_button == MOUSE_LEFT as ::core::ffi::c_int {
                        c"l".as_ptr()
                    } else if which_button == MOUSE_RIGHT as ::core::ffi::c_int {
                        c"r".as_ptr()
                    } else if which_button == MOUSE_MIDDLE as ::core::ffi::c_int {
                        c"m".as_ptr()
                    } else if which_button == MOUSE_X1 as ::core::ffi::c_int {
                        c"x1".as_ptr()
                    } else if which_button == MOUSE_X2 as ::core::ffi::c_int {
                        c"x2".as_ptr()
                    } else {
                        c"?".as_ptr()
                    }) as *mut ::core::ffi::c_char,
                },
            },
            typval_T {
                v_type: VAR_STRING,
                v_lock: VAR_FIXED,
                vval: typval_vval_union {
                    v_string: &raw mut c2rust_lvalue as *mut ::core::ffi::c_char,
                },
            },
        ];
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        call_vim_function(
            (*click_defs.offset(col as isize)).func,
            ::core::mem::size_of::<[typval_T; 4]>()
                .wrapping_div(::core::mem::size_of::<typval_T>())
                .wrapping_div(
                    (::core::mem::size_of::<[typval_T; 4]>()
                        .wrapping_rem(::core::mem::size_of::<typval_T>())
                        == 0) as ::core::ffi::c_int as usize,
                ) as ::core::ffi::c_int,
            &raw mut argv as *mut typval_T,
            &raw mut rettv,
        );
        tv_clear(&raw mut rettv);
        got_click.set(false_0 != 0);
    }
}

pub(crate) unsafe extern "C" fn get_fpos_of_mouse(mut mpos: *mut pos_T) -> ::core::ffi::c_int {
    unsafe {
        let mut grid: ::core::ffi::c_int = mouse_grid.get();
        let mut row: ::core::ffi::c_int = mouse_row.get();
        let mut col: ::core::ffi::c_int = mouse_col.get();
        if row < 0 as ::core::ffi::c_int || col < 0 as ::core::ffi::c_int {
            return IN_UNKNOWN as ::core::ffi::c_int;
        }
        let mut wp: *mut win_T = mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col);
        if wp.is_null() {
            return IN_UNKNOWN as ::core::ffi::c_int;
        }
        let mut winrow: ::core::ffi::c_int = row;
        let mut wincol: ::core::ffi::c_int = col;
        let mut below_buffer: bool =
            mouse_comp_pos(wp, &raw mut row, &raw mut col, &raw mut (*mpos).lnum);
        if !below_buffer
            && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
            && (if (*wp).w_onebuf_opt.wo_rl != 0 {
                (wincol >= (*wp).w_view_width - win_col_off(wp)) as ::core::ffi::c_int
            } else {
                (wincol < win_col_off(wp)) as ::core::ffi::c_int
            }) != 0
        {
            return MOUSE_STATUSCOL as ::core::ffi::c_int;
        }
        if winrow >= (*wp).w_view_height + (*wp).w_status_height {
            if mouse_grid.get() <= 1 as ::core::ffi::c_int
                && (mouse_row.get() as OptInt) < Rows.get() as OptInt - p_ch.get()
                && mouse_row.get() as OptInt
                    >= Rows.get() as OptInt - p_ch.get() - global_stl_height() as OptInt
            {
                return IN_STATUS_LINE as ::core::ffi::c_int;
            }
            return IN_UNKNOWN as ::core::ffi::c_int;
        } else if winrow >= (*wp).w_view_height {
            return IN_STATUS_LINE as ::core::ffi::c_int;
        }
        if winrow < 0 as ::core::ffi::c_int
            && winrow + (*wp).w_winbar_height >= 0 as ::core::ffi::c_int
        {
            return MOUSE_WINBAR as ::core::ffi::c_int;
        }
        if wincol >= (*wp).w_view_width {
            return IN_SEP_LINE as ::core::ffi::c_int;
        }
        if wp != curwin.get() || below_buffer as ::core::ffi::c_int != 0 {
            return IN_UNKNOWN as ::core::ffi::c_int;
        }
        (*mpos).col = vcol2col(wp, (*mpos).lnum, col as colnr_T, &raw mut (*mpos).coladd);
        return IN_BUFFER as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn do_popup(
    mut which_button: ::core::ffi::c_int,
    mut m_pos_flag: ::core::ffi::c_int,
    mut m_pos: pos_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut jump_flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if strcmp(p_mousem.get(), c"popup_setpos".as_ptr()) == 0 as ::core::ffi::c_int {
            if VIsual_active.get() {
                if m_pos_flag != IN_BUFFER as ::core::ffi::c_int {
                    jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                } else if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
                    if (*curwin.get()).w_cursor.lnum <= (*VIsual.ptr()).lnum
                        && (m_pos.lnum < (*curwin.get()).w_cursor.lnum
                            || (*VIsual.ptr()).lnum < m_pos.lnum)
                        || (*VIsual.ptr()).lnum < (*curwin.get()).w_cursor.lnum
                            && (m_pos.lnum < (*VIsual.ptr()).lnum
                                || (*curwin.get()).w_cursor.lnum < m_pos.lnum)
                    {
                        jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                    }
                } else if ltoreq((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0
                    && (lt(m_pos, (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
                        || lt(VIsual.get(), m_pos) as ::core::ffi::c_int != 0)
                    || lt(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
                        && (lt(m_pos, VIsual.get()) as ::core::ffi::c_int != 0
                            || lt((*curwin.get()).w_cursor, m_pos) as ::core::ffi::c_int != 0)
                {
                    jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                } else if VIsual_mode.get() == Ctrl_V {
                    let mut leftcol: colnr_T = 0;
                    let mut rightcol: colnr_T = 0;
                    getvcols(
                        curwin.get(),
                        &raw mut (*curwin.get()).w_cursor,
                        VIsual.ptr(),
                        &raw mut leftcol,
                        &raw mut rightcol,
                    );
                    getvcol(
                        curwin.get(),
                        &raw mut m_pos,
                        ::core::ptr::null_mut::<colnr_T>(),
                        &raw mut m_pos.col,
                        ::core::ptr::null_mut::<colnr_T>(),
                    );
                    if m_pos.col < leftcol || m_pos.col > rightcol {
                        jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
                    }
                }
            } else {
                jump_flags = MOUSE_MAY_STOP_VIS as ::core::ffi::c_int;
            }
        }
        if jump_flags != 0 {
            jump_flags = jump_to_mouse(jump_flags, ::core::ptr::null_mut::<bool>(), which_button);
            redraw_curbuf_later(if VIsual_active.get() as ::core::ffi::c_int != 0 {
                UPD_INVERTED
            } else {
                UPD_VALID
            });
            update_screen();
            setcursor();
            ui_flush();
        }
        show_popupmenu();
        got_click.set(false_0 != 0);
        return jump_flags;
    }
}
