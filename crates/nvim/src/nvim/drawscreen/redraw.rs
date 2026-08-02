//! Marking things to be redrawn later.
//!
//! None of this draws: every function sets a flag that [`update_screen`] reads on
//! the next pass through the main loop. [`redraw_later`] is the primitive -- it
//! raises one window's `w_redr_type` and the global `must_redraw` -- and the rest
//! name a scope: all windows ([`redraw_all_later`]), every window on one buffer
//! ([`redraw_buf_later`]), a line range ([`redraw_win_range_later`]), a status
//! line ([`status_redraw_buf`]).
//!
//! [`show_cursor_info_later`] is the one that decides *whether* anything changed:
//! it compares the cursor position, the Visual selection and the recording state
//! against what the status line was last drawn with.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn redraw_custom_title_later() -> bool {
    if p_icon.get() != 0 && stl_syntax.get() & STL_IN_ICON != 0
        || p_title.get() != 0 && stl_syntax.get() & STL_IN_TITLE != 0
    {
        need_maketitle.set(true_0 != 0);
        return true_0 != 0;
    }
    return false_0 != 0;
}

pub unsafe extern "C" fn show_cursor_info_later(mut force: bool) {
    unsafe {
        let mut state: ::core::ffi::c_int = get_real_state();
        let mut empty_line: ::core::ffi::c_int = (State.get() & MODE_INSERT
            == 0 as ::core::ffi::c_int
            && *ml_get_buf((*curwin.get()).w_buffer, (*curwin.get()).w_cursor.lnum)
                as ::core::ffi::c_int
                == NUL) as ::core::ffi::c_int;
        validate_virtcol(curwin.get());
        if force as ::core::ffi::c_int != 0
            || (*curwin.get()).w_cursor.lnum != (*curwin.get()).w_stl_cursor.lnum
            || (*curwin.get()).w_cursor.col != (*curwin.get()).w_stl_cursor.col
            || (*curwin.get()).w_virtcol != (*curwin.get()).w_stl_virtcol
            || (*curwin.get()).w_cursor.coladd != (*curwin.get()).w_stl_cursor.coladd
            || (*curwin.get()).w_topline != (*curwin.get()).w_stl_topline
            || (*(*curwin.get()).w_buffer).b_ml.ml_line_count != (*curwin.get()).w_stl_line_count
            || (*curwin.get()).w_topfill != (*curwin.get()).w_stl_topfill
            || empty_line != (*curwin.get()).w_stl_empty as ::core::ffi::c_int
            || reg_recording.get() != (*curwin.get()).w_stl_recording
            || state != (*curwin.get()).w_stl_state
            || VIsual_active.get() as ::core::ffi::c_int != 0
                && (VIsual_mode.get() != (*curwin.get()).w_stl_visual_mode
                    || (*VIsual.ptr()).lnum != (*curwin.get()).w_stl_visual_pos.lnum
                    || (*VIsual.ptr()).col != (*curwin.get()).w_stl_visual_pos.col
                    || (*VIsual.ptr()).coladd != (*curwin.get()).w_stl_visual_pos.coladd)
        {
            if (*curwin.get()).w_status_height != 0 || global_stl_height() != 0 {
                (*curwin.get()).w_redr_status = true_0 != 0;
            } else {
                redraw_cmdline.set(true_0 != 0);
            }
            if *p_wbr.get() as ::core::ffi::c_int != NUL
                || *(*curwin.get()).w_onebuf_opt.wo_wbr as ::core::ffi::c_int != NUL
            {
                (*curwin.get()).w_redr_status = true_0 != 0;
            }
            redraw_custom_title_later();
        }
        (*curwin.get()).w_stl_cursor = (*curwin.get()).w_cursor;
        (*curwin.get()).w_stl_virtcol = (*curwin.get()).w_virtcol;
        (*curwin.get()).w_stl_empty = empty_line as ::core::ffi::c_char;
        (*curwin.get()).w_stl_topline = (*curwin.get()).w_topline;
        (*curwin.get()).w_stl_line_count = (*(*curwin.get()).w_buffer).b_ml.ml_line_count;
        (*curwin.get()).w_stl_topfill = (*curwin.get()).w_topfill;
        (*curwin.get()).w_stl_recording = reg_recording.get();
        (*curwin.get()).w_stl_state = state;
        if VIsual_active.get() {
            (*curwin.get()).w_stl_visual_mode = VIsual_mode.get();
            (*curwin.get()).w_stl_visual_pos = VIsual.get();
        }
    }
}

pub unsafe extern "C" fn redraw_later(mut wp: *mut win_T, mut type_0: ::core::ffi::c_int) {
    unsafe {
        '_c2rust_label: {
            if !wp.is_null() || exiting.get() as ::core::ffi::c_int != 0 {
            } else {
                __assert_fail(
                    b"wp != NULL || exiting\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2623 as ::core::ffi::c_uint,
                    b"void redraw_later(win_T *, int)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        if !exiting.get() && !redraw_not_allowed.get() && (*wp).w_redr_type < type_0 {
            (*wp).w_redr_type = type_0;
            if type_0 >= UPD_NOT_VALID {
                (*wp).w_lines_valid = 0 as ::core::ffi::c_int;
            }
            must_redraw.set(if must_redraw.get() > type_0 {
                must_redraw.get()
            } else {
                type_0
            });
        }
    }
}

pub unsafe extern "C" fn redraw_all_later(mut type_0: ::core::ffi::c_int) {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            redraw_later(wp, type_0);
            wp = (*wp).w_next;
        }
        set_must_redraw(type_0);
    }
}

pub unsafe extern "C" fn set_must_redraw(mut type_0: ::core::ffi::c_int) {
    if !redraw_not_allowed.get() {
        must_redraw.set(if must_redraw.get() > type_0 {
            must_redraw.get()
        } else {
            type_0
        });
    }
}

pub unsafe extern "C" fn screen_invalidate_highlights() {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            redraw_later(wp, UPD_NOT_VALID);
            (*wp).w_grid_alloc.valid = false_0 != 0;
            wp = (*wp).w_next;
        }
    }
}

pub unsafe extern "C" fn redraw_curbuf_later(mut type_0: ::core::ffi::c_int) {
    unsafe {
        redraw_buf_later(curbuf.get(), type_0);
    }
}

pub unsafe extern "C" fn redraw_buf_later(mut buf: *mut buf_T, mut type_0: ::core::ffi::c_int) {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                redraw_later(wp, type_0);
            }
            wp = (*wp).w_next;
        }
    }
}

pub unsafe extern "C" fn redraw_buf_line_later(
    mut buf: *mut buf_T,
    mut line: linenr_T,
    mut force: bool,
) {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                redrawWinline(
                    wp,
                    if line < (*buf).b_ml.ml_line_count {
                        line
                    } else {
                        (*buf).b_ml.ml_line_count
                    },
                );
                if force as ::core::ffi::c_int != 0 && line > (*buf).b_ml.ml_line_count {
                    (*wp).w_redraw_bot = line;
                }
            }
            wp = (*wp).w_next;
        }
    }
}

pub unsafe extern "C" fn redraw_win_range_later(
    mut wp: *mut win_T,
    mut first: linenr_T,
    mut last: linenr_T,
) {
    unsafe {
        if last >= (*wp).w_topline && first < (*wp).w_botline {
            if (*wp).w_redraw_top == 0 as linenr_T || (*wp).w_redraw_top > first {
                (*wp).w_redraw_top = first;
            }
            if (*wp).w_redraw_bot == 0 as linenr_T || (*wp).w_redraw_bot < last {
                (*wp).w_redraw_bot = last;
            }
            redraw_later(wp, UPD_VALID);
        }
    }
}

pub unsafe extern "C" fn redrawWinline(mut wp: *mut win_T, mut lnum: linenr_T) {
    unsafe {
        redraw_win_range_later(wp, lnum, lnum);
    }
}

pub unsafe extern "C" fn redraw_buf_range_later(
    mut buf: *mut buf_T,
    mut first: linenr_T,
    mut last: linenr_T,
) {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf {
                redraw_win_range_later(wp, first, last);
            }
            wp = (*wp).w_next;
        }
    }
}

pub unsafe extern "C" fn redraw_buf_status_later(mut buf: *mut buf_T) {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf
                && ((*wp).w_status_height != 0
                    || wp == curwin.get() && global_stl_height() != 0
                    || (*wp).w_winbar_height != 0)
            {
                (*wp).w_redr_status = true_0 != 0;
                set_must_redraw(UPD_VALID);
            }
            wp = (*wp).w_next;
        }
    }
}

pub unsafe extern "C" fn status_redraw_all() {
    unsafe {
        let mut is_stl_global: bool = global_stl_height() != 0 as ::core::ffi::c_int;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if !is_stl_global && (*wp).w_status_height != 0
                || wp == curwin.get()
                || (*wp).w_winbar_height != 0
            {
                (*wp).w_redr_status = true_0 != 0;
                redraw_later(wp, UPD_VALID);
            }
            wp = (*wp).w_next;
        }
    }
}

pub unsafe extern "C" fn status_redraw_curbuf() {
    unsafe {
        status_redraw_buf(curbuf.get());
    }
}

pub unsafe extern "C" fn status_redraw_buf(mut buf: *mut buf_T) {
    unsafe {
        let mut is_stl_global: bool = global_stl_height() != 0 as ::core::ffi::c_int;
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_buffer == buf
                && (!is_stl_global && (*wp).w_status_height != 0
                    || is_stl_global as ::core::ffi::c_int != 0 && wp == curwin.get()
                    || (*wp).w_winbar_height != 0)
            {
                (*wp).w_redr_status = true_0 != 0;
                redraw_later(wp, UPD_VALID);
            }
            wp = (*wp).w_next;
        }
        if p_ru.get() != 0 && (*curwin.get()).w_status_height == 0 && !(*curwin.get()).w_redr_status
        {
            redraw_cmdline.set(true_0 != 0);
            redraw_later(curwin.get(), UPD_VALID);
        }
    }
}

pub unsafe extern "C" fn redraw_statuslines() {
    unsafe {
        let mut wp: *mut win_T = if curtab.get() == curtab.get() {
            firstwin.get()
        } else {
            (*curtab.get()).tp_firstwin
        };
        while !wp.is_null() {
            if (*wp).w_redr_status {
                win_check_ns_hl(wp);
                win_redr_winbar(wp);
                win_redr_status(wp);
            }
            wp = (*wp).w_next;
        }
        win_check_ns_hl(::core::ptr::null_mut::<win_T>());
        if redraw_tabline.get() {
            draw_tabline();
        }
        if need_maketitle.get() {
            maketitle();
        }
    }
}

pub unsafe extern "C" fn win_redraw_last_status(mut frp: *const frame_T) {
    unsafe {
        if (*frp).fr_layout as ::core::ffi::c_int == FR_LEAF {
            (*(*frp).fr_win).w_redr_status = true_0 != 0;
        } else if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
            frp = (*frp).fr_child;
            while !frp.is_null() {
                win_redraw_last_status(frp);
                frp = (*frp).fr_next;
            }
        } else {
            '_c2rust_label: {
                if (*frp).fr_layout as ::core::ffi::c_int == 2 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"frp->fr_layout == FR_COL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/drawscreen.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2806 as ::core::ffi::c_uint,
                        b"void win_redraw_last_status(const frame_T *)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            frp = (*frp).fr_child;
            while !(*frp).fr_next.is_null() {
                frp = (*frp).fr_next;
            }
            win_redraw_last_status(frp);
        };
    }
}
