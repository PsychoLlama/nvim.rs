//! `nvim__redraw()`: asking for parts of the screen to be redrawn.
//!
//! The keyset says *what* is stale -- a buffer, a window, a line range,
//! the statusline, the tabline, the cursor -- and this walks the affected
//! windows marking each of them, then optionally flushes.  `redraw_status`
//! is the per-window half.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::{ERROR_INIT, Reported, has_key};
use crate::guard::Allow;
use crate::types::NUL;

unsafe fn redraw_status(mut wp: *mut win_T, mut opts: *mut KeyDict_redraw, mut flush: *mut bool) {
    unsafe {
        if (*opts).statuscolumn as ::core::ffi::c_int != 0
            && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
        {
            (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
            changed_window_setting(wp);
        }
        let mut old_row_offset: ::core::ffi::c_int = (*wp).w_grid.row_offset;
        win_grid_alloc(wp);
        if (*wp).w_lines_valid == 0 as ::core::ffi::c_int
            || (*wp).w_grid.row_offset != old_row_offset
        {
            *flush = true;
        }
        if *flush as ::core::ffi::c_int != 0
            && ((*opts).statusline as ::core::ffi::c_int != 0
                || (*opts).winbar as ::core::ffi::c_int != 0)
        {
            (*wp).w_redr_status = true;
        } else if (*opts).statusline as ::core::ffi::c_int != 0
            || (*opts).winbar as ::core::ffi::c_int != 0
        {
            win_check_ns_hl(wp);
            if (*opts).winbar {
                win_redr_winbar(wp);
            }
            if (*opts).statusline {
                win_redr_status(wp);
            }
            win_check_ns_hl(::core::ptr::null_mut::<win_T>());
        }
    }
}

pub unsafe fn nvim__redraw(opts: *mut KeyDict_redraw) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    unsafe {
        let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
        let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        if has_key((*opts).is_set__redraw_, KEYSET_OPTIDX_redraw__win) {
            win = find_window_by_handle((*opts).win, err);
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                return ().reported(error);
            }
        }
        if has_key((*opts).is_set__redraw_, KEYSET_OPTIDX_redraw__buf) {
            if !win.is_null() {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"%s".as_ptr(),
                    c"cannot use both 'buf' and 'win'".as_ptr(),
                );
                return ().reported(error);
            }
            buf = find_buffer_by_handle((*opts).buf, err);
            if (*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                return ().reported(error);
            }
        }
        let mut count: ::core::ffi::c_uint = (!win.is_null() as ::core::ffi::c_int
            + !buf.is_null() as ::core::ffi::c_int)
            as ::core::ffi::c_uint;
        if !(((*opts).is_set__redraw_ as uint64_t).count_ones() > count) {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"at least one action required".as_ptr(),
            );
            return ().reported(error);
        }
        if has_key((*opts).is_set__redraw_, KEYSET_OPTIDX_redraw__valid) {
            let mut type_0: ::core::ffi::c_int = if (*opts).valid as ::core::ffi::c_int != 0 {
                UPD_VALID
            } else {
                UPD_NOT_VALID
            };
            if !win.is_null() {
                redraw_later(win, type_0);
            } else if !buf.is_null() {
                redraw_buf_later(buf, type_0);
            } else {
                redraw_all_later(type_0);
            }
        }
        if has_key((*opts).is_set__redraw_, KEYSET_OPTIDX_redraw__range) {
            if !((*opts).range.size == 2 as size_t
                && (*(*opts).range.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                    as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*(*opts).range.items.offset(1 as ::core::ffi::c_int as isize)).type_0
                    as ::core::ffi::c_uint
                    == kObjectTypeInteger as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*(*opts).range.items.offset(0 as ::core::ffi::c_int as isize))
                    .data
                    .integer
                    >= 0 as Integer
                && (*(*opts).range.items.offset(1 as ::core::ffi::c_int as isize))
                    .data
                    .integer
                    >= -1 as Integer)
            {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"%s".as_ptr(),
                    c"Invalid 'range': Expected 2-tuple of Integers".as_ptr(),
                );
                return ().reported(error);
            }
            let mut begin_raw: int64_t =
                (*(*opts).range.items.offset(0 as ::core::ffi::c_int as isize))
                    .data
                    .integer as int64_t;
            let mut end_raw: int64_t =
                (*(*opts).range.items.offset(1 as ::core::ffi::c_int as isize))
                    .data
                    .integer as int64_t;
            let mut rbuf: *mut buf_T = if !win.is_null() {
                (*win).w_buffer
            } else if !buf.is_null() {
                buf
            } else {
                curbuf.get()
            };
            let mut line_count: linenr_T = (*rbuf).b_ml.ml_line_count;
            let mut begin: ::core::ffi::c_int = (if begin_raw < line_count as int64_t {
                begin_raw
            } else {
                line_count as int64_t
            }) as ::core::ffi::c_int;
            let mut end: ::core::ffi::c_int = 0;
            if end_raw == -1 as int64_t {
                end = line_count as ::core::ffi::c_int;
            } else {
                end = (if (if begin as int64_t > end_raw {
                    begin as int64_t
                } else {
                    end_raw
                }) < line_count as int64_t
                {
                    if begin as int64_t > end_raw {
                        begin as int64_t
                    } else {
                        end_raw
                    }
                } else {
                    line_count as int64_t
                }) as ::core::ffi::c_int;
            }
            if begin < end {
                redraw_buf_range_later(rbuf, 1 as linenr_T + begin as linenr_T, end as linenr_T);
            }
        }
        if has_key((*opts).is_set__redraw_, KEYSET_OPTIDX_redraw__valid)
            || has_key((*opts).is_set__redraw_, KEYSET_OPTIDX_redraw__range)
        {
            (*opts).flush = if has_key((*opts).is_set__redraw_, KEYSET_OPTIDX_redraw__flush) {
                (*opts).flush as ::core::ffi::c_int
            } else {
                1
            } != 0;
        }
        let mut flush_ui: bool = (*opts).flush;
        if (*opts).tabline {
            if redraw_tabline.get() as ::core::ffi::c_int != 0
                && (*firstwin.get()).w_lines_valid == 0 as ::core::ffi::c_int
            {
                (*opts).flush = true;
            } else {
                draw_tabline();
            }
            flush_ui = true;
        }
        let mut save_lz: bool = p_lz.get() != 0;
        let redraw = Allow::redraw();
        p_lz.set(0);
        if (*opts).statuscolumn as ::core::ffi::c_int != 0
            || (*opts).statusline as ::core::ffi::c_int != 0
            || (*opts).winbar as ::core::ffi::c_int != 0
        {
            if win.is_null() {
                let mut wp: *mut win_T = if curtab.get() == curtab.get() {
                    firstwin.get()
                } else {
                    (*curtab.get()).tp_firstwin
                };
                while !wp.is_null() {
                    if buf.is_null() || (*wp).w_buffer == buf {
                        redraw_status(wp, opts, &raw mut (*opts).flush);
                    }
                    wp = (*wp).w_next;
                }
            } else {
                redraw_status(win, opts, &raw mut (*opts).flush);
            }
            flush_ui = true;
        }
        let mut cwin: *mut win_T = if !win.is_null() { win } else { curwin.get() };
        if (*opts).cursor as ::core::ffi::c_int != 0
            && ((*cwin).w_grid.target.is_null() || !(*(*cwin).w_grid.target).valid)
        {
            (*opts).flush = true;
        }
        if (*opts).flush as ::core::ffi::c_int != 0 && !cmdpreview.get() {
            validate_cursor(curwin.get());
            update_topline(curwin.get());
            update_screen();
        }
        if (*opts).cursor {
            setcursor_mayforce(cwin, true);
            flush_ui = true;
        }
        if flush_ui {
            ui_flush();
        }
        drop(redraw);
        p_lz.set(save_lz as ::core::ffi::c_int);
    }
    ().reported(error)
}
