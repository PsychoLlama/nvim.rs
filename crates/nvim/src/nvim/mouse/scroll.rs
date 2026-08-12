//! The wheel, and the mouse in Insert mode -- `do_mousescroll()`,
//! `ins_mouse()` and `ins_mousescroll()`.
//!
//! [`do_mousescroll`] applies `'mousescroll'` to a wheel event, scrolling by
//! lines or by pages and honouring `'scrolloff'`; the `ins_*` pair is the
//! Insert-mode form, which has to leave and re-enter Insert mode around the
//! move so undo and `'backspace'` see a sane state.  [`is_mouse_key`] is the
//! predicate the mapping layer asks, and `dragwin` the window a drag started
//! in.
//!
//! Original: `src/nvim/mouse.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::siemsg_c;
use crate::src::nvim::buffer::bt_prompt;
use crate::src::nvim::drawscreen::redraw_statuslines;
use crate::src::nvim::edit::{set_can_cindent, start_arrow, undisplay_dollar};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::keycodes::{
    KE_LEFTDRAG, KE_LEFTMOUSE, KE_LEFTMOUSE_NM, KE_LEFTRELEASE, KE_LEFTRELEASE_NM, KE_MIDDLEDRAG,
    KE_MIDDLEMOUSE, KE_MIDDLERELEASE, KE_MOUSEDOWN, KE_MOUSELEFT, KE_MOUSEMOVE, KE_MOUSERIGHT,
    KE_MOUSEUP, KE_RIGHTDRAG, KE_RIGHTMOUSE, KE_RIGHTRELEASE, KE_X1DRAG, KE_X1MOUSE, KE_X1RELEASE,
    KE_X2DRAG, KE_X2MOUSE, KE_X2RELEASE,
};
use crate::src::nvim::main::{
    State, curbuf, curwin, mod_mask, mouse_col, mouse_grid, mouse_row, p_mousem, p_mousescroll_hor,
    p_mousescroll_vert,
};
use crate::src::nvim::r#move::pagescroll;
use crate::src::nvim::normal::nv_scroll_line;
use crate::src::nvim::ops::clear_oparg;
use crate::src::nvim::os::libc::memset;
use crate::src::nvim::popupmenu::pum_visible;
use crate::src::nvim::pos::equalpos;
use crate::src::nvim::search::{BACKWARD, FORWARD};
use crate::src::nvim::state::MODE_NORMAL;
use crate::src::nvim::types::{Direction, cmdarg_T, colnr_T, oparg_T, pos_T, win_T};
use crate::src::nvim::window::win_valid;

pub unsafe extern "C" fn ins_mouse(mut c: ::core::ffi::c_int) {
    unsafe {
        let mut old_curwin: *mut win_T = curwin.get();
        undisplay_dollar();
        let mut tpos: pos_T = (*curwin.get()).w_cursor;
        if do_mouse(
            ::core::ptr::null_mut::<oparg_T>(),
            c,
            BACKWARD as ::core::ffi::c_int,
            1 as ::core::ffi::c_int,
            false,
        ) {
            let mut new_curwin: *mut win_T = curwin.get();
            if curwin.get() != old_curwin && win_valid(old_curwin) as ::core::ffi::c_int != 0 {
                curwin.set(old_curwin);
                curbuf.set((*curwin.get()).w_buffer);
                if bt_prompt(curbuf.get()) {
                    (*curbuf.get()).b_prompt_insert = 'A' as ::core::ffi::c_int;
                }
            }
            start_arrow(if curwin.get() == old_curwin {
                &raw mut tpos
            } else {
                ::core::ptr::null_mut::<pos_T>()
            });
            if curwin.get() != new_curwin && win_valid(new_curwin) as ::core::ffi::c_int != 0 {
                curwin.set(new_curwin);
                curbuf.set((*curwin.get()).w_buffer);
            }
            set_can_cindent(true_0 != 0);
        }
        redraw_statuslines();
    }
}

pub unsafe extern "C" fn do_mousescroll(mut cap: *mut cmdarg_T) {
    unsafe {
        let mut shift_or_ctrl: bool = mod_mask.get() & (MOD_MASK_SHIFT | MOD_MASK_CTRL) != 0;
        if (*cap).arg == MSCR_UP as ::core::ffi::c_int
            || (*cap).arg == MSCR_DOWN as ::core::ffi::c_int
        {
            if State.get() & MODE_NORMAL != 0 && shift_or_ctrl as ::core::ffi::c_int != 0 {
                pagescroll(
                    (if (*cap).arg != 0 {
                        FORWARD as ::core::ffi::c_int
                    } else {
                        BACKWARD as ::core::ffi::c_int
                    }) as Direction,
                    1 as ::core::ffi::c_int,
                    false_0 != 0,
                );
            } else {
                if shift_or_ctrl {
                    (*cap).count1 = ((*curwin.get()).w_botline - (*curwin.get()).w_topline)
                        as ::core::ffi::c_int;
                } else {
                    (*cap).count1 = p_mousescroll_vert.get() as ::core::ffi::c_int;
                }
                if (*cap).count1 > 0 as ::core::ffi::c_int {
                    (*cap).count0 = (*cap).count1;
                    nv_scroll_line(cap);
                }
            }
        } else {
            let mut step: ::core::ffi::c_int = if shift_or_ctrl as ::core::ffi::c_int != 0 {
                (*curwin.get()).w_view_width
            } else {
                p_mousescroll_hor.get() as ::core::ffi::c_int
            };
            let mut leftcol: colnr_T = (*curwin.get()).w_leftcol
                + (if (*cap).arg == MSCR_RIGHT as ::core::ffi::c_int {
                    -(step as colnr_T)
                } else {
                    step as colnr_T
                });
            leftcol = (if leftcol > 0 as ::core::ffi::c_int {
                leftcol as ::core::ffi::c_int
            } else {
                0 as ::core::ffi::c_int
            }) as colnr_T;
            do_mousescroll_horiz(leftcol);
        };
    }
}

pub unsafe extern "C" fn ins_mousescroll(mut dir: ::core::ffi::c_int) {
    unsafe {
        let mut cap: cmdarg_T = cmdarg_T {
            oap: ::core::ptr::null_mut::<oparg_T>(),
            prechar: 0,
            cmdchar: 0,
            nchar: 0,
            nchar_composing: [0; 32],
            nchar_len: 0,
            extra_char: 0,
            opcount: 0,
            count0: 0,
            count1: 0,
            arg: 0,
            retval: 0,
            searchbuf: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut oa: oparg_T = oparg_T {
            op_type: 0,
            regname: 0,
            motion_type: kMTCharWise,
            motion_force: 0,
            use_reg_one: false,
            inclusive: false,
            end_adjusted: false,
            start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            end: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cursor_start: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            line_count: 0,
            empty: false,
            is_VIsual: false,
            start_vcol: 0,
            end_vcol: 0,
            prev_opcount: 0,
            prev_count0: 0,
            excl_tr_ws: false,
        };
        memset(
            &raw mut cap as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<cmdarg_T>(),
        );
        clear_oparg(&raw mut oa);
        cap.oap = &raw mut oa;
        cap.arg = dir;
        match dir {
            1 => {
                cap.cmdchar = -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            }
            0 => {
                cap.cmdchar = -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            }
            -1 => {
                cap.cmdchar = -(253 as ::core::ffi::c_int
                    + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            }
            -2 => {
                cap.cmdchar = -(253 as ::core::ffi::c_int
                    + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
            }
            _ => {
                siemsg_c!(c"Invalid ins_mousescroll() argument: %d".as_ptr(), dir,);
            }
        }
        let mut old_curwin: *mut win_T = curwin.get();
        if mouse_row.get() >= 0 as ::core::ffi::c_int && mouse_col.get() >= 0 as ::core::ffi::c_int
        {
            let mut grid: ::core::ffi::c_int = mouse_grid.get();
            let mut row: ::core::ffi::c_int = mouse_row.get();
            let mut col: ::core::ffi::c_int = mouse_col.get();
            curwin.set(mouse_find_win_inner(
                &raw mut grid,
                &raw mut row,
                &raw mut col,
            ));
            if (*curwin.ptr()).is_null() {
                curwin.set(old_curwin);
                return;
            }
            curbuf.set((*curwin.get()).w_buffer);
        }
        if curwin.get() == old_curwin {
            if pum_visible() {
                return;
            }
            undisplay_dollar();
        }
        let mut orig_cursor: pos_T = (*curwin.get()).w_cursor;
        do_mousescroll(&raw mut cap);
        (*curwin.get()).w_redr_status = true_0 != 0;
        curwin.set(old_curwin);
        curbuf.set((*curwin.get()).w_buffer);
        if !equalpos((*curwin.get()).w_cursor, orig_cursor) {
            start_arrow(&raw mut orig_cursor);
            set_can_cindent(true_0 != 0);
        }
    }
}

pub unsafe extern "C" fn is_mouse_key(mut c: ::core::ffi::c_int) -> bool {
    return c
        == -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTMOUSE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_LEFTRELEASE_NM as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MIDDLEDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MIDDLERELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_RIGHTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_RIGHTDRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_RIGHTRELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEDOWN as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSEUP as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSELEFT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_MOUSERIGHT as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X1MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X1DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X1RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X2MOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X2DRAG as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        || c == -(253 as ::core::ffi::c_int
            + ((KE_X2RELEASE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
}

pub(crate) unsafe extern "C" fn mouse_model_popup() -> bool {
    unsafe {
        return *(*p_mousem.ptr()).offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == 'p' as ::core::ffi::c_int;
    }
}

pub(crate) static dragwin: GlobalCell<*mut win_T> =
    GlobalCell::new(::core::ptr::null_mut::<win_T>());

pub unsafe extern "C" fn reset_dragwin() {
    dragwin.set(::core::ptr::null_mut::<win_T>());
}
