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
use crate::winlayer::{Win, first_window, windows};

unsafe fn redraw_status(mut wp: *mut win_T, mut opts: *mut KeyDict_redraw, mut flush: *mut bool) {
    if unsafe { (*opts).statuscolumn } as ::core::ffi::c_int != 0
        && unsafe { *(*wp).w_onebuf_opt.wo_stc } as ::core::ffi::c_int != NUL
    {
        unsafe { (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T };
        changed_window_setting(unsafe { Win::new(wp) });
    }
    let mut old_row_offset: ::core::ffi::c_int = unsafe { (*wp).w_grid.row_offset };
    unsafe { win_grid_alloc(wp) };
    if unsafe { (*wp).w_lines_valid } == 0 as ::core::ffi::c_int
        || unsafe { (*wp).w_grid.row_offset } != old_row_offset
    {
        unsafe { *flush = true };
    }
    if unsafe { *flush } as ::core::ffi::c_int != 0
        && (unsafe { (*opts).statusline } as ::core::ffi::c_int != 0
            || unsafe { (*opts).winbar } as ::core::ffi::c_int != 0)
    {
        unsafe { (*wp).w_redr_status = true };
    } else if unsafe { (*opts).statusline } as ::core::ffi::c_int != 0
        || unsafe { (*opts).winbar } as ::core::ffi::c_int != 0
    {
        unsafe { win_check_ns_hl(wp) };
        if unsafe { (*opts).winbar } {
            unsafe { win_redr_winbar(wp) };
        }
        if unsafe { (*opts).statusline } {
            unsafe { win_redr_status(wp) };
        }
        unsafe { win_check_ns_hl(::core::ptr::null_mut::<win_T>()) };
    }
}

pub unsafe fn nvim__redraw(opts: *mut KeyDict_redraw) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if has_key(
        unsafe { (*opts).is_set__redraw_ },
        KEYSET_OPTIDX_redraw__win,
    ) {
        win = unsafe { find_window_by_handle((*opts).win, err) };
        if unsafe { (*err).type_0 } as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return ().reported(error);
        }
    }
    if has_key(
        unsafe { (*opts).is_set__redraw_ },
        KEYSET_OPTIDX_redraw__buf,
    ) {
        if !win.is_null() {
            unsafe {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"%s".as_ptr(),
                    c"cannot use both 'buf' and 'win'".as_ptr(),
                )
            };
            return ().reported(error);
        }
        buf = unsafe { find_buffer_by_handle((*opts).buf, err) };
        if unsafe { (*err).type_0 } as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            return ().reported(error);
        }
    }
    let mut count: ::core::ffi::c_uint = (!win.is_null() as ::core::ffi::c_int
        + !buf.is_null() as ::core::ffi::c_int)
        as ::core::ffi::c_uint;
    if !((unsafe { (*opts).is_set__redraw_ } as uint64_t).count_ones() > count) {
        unsafe {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"%s".as_ptr(),
                c"at least one action required".as_ptr(),
            )
        };
        return ().reported(error);
    }
    if has_key(
        unsafe { (*opts).is_set__redraw_ },
        KEYSET_OPTIDX_redraw__valid,
    ) {
        let mut type_0: ::core::ffi::c_int = if unsafe { (*opts).valid } as ::core::ffi::c_int != 0
        {
            UPD_VALID
        } else {
            UPD_NOT_VALID
        };
        if !win.is_null() {
            unsafe { redraw_later(win, type_0) };
        } else if !buf.is_null() {
            unsafe { redraw_buf_later(buf, type_0) };
        } else {
            unsafe { redraw_all_later(type_0) };
        }
    }
    if has_key(
        unsafe { (*opts).is_set__redraw_ },
        KEYSET_OPTIDX_redraw__range,
    ) {
        // SAFETY: the caller's keyset -- `range` names its own items -- and
        // the tags below say the integer arm of each is the live one.
        let pair = unsafe {
            let range = (*opts).range;
            (range.size == 2).then(|| (*range.items, *range.items.add(1)))
        };
        // SAFETY: as above.
        let range = pair.filter(|(begin, end)| unsafe {
            begin.type_0 == kObjectTypeInteger
                && end.type_0 == kObjectTypeInteger
                && begin.data.integer >= 0
                && end.data.integer >= -1
        });
        let Some((begin_obj, end_obj)) = range else {
            unsafe {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"%s".as_ptr(),
                    c"Invalid 'range': Expected 2-tuple of Integers".as_ptr(),
                )
            };
            return ().reported(error);
        };
        // SAFETY: the tags above say the integer arm of each is the live one.
        let (begin_raw, end_raw) = unsafe { (begin_obj.data.integer, end_obj.data.integer) };
        let mut rbuf: *mut buf_T = if !win.is_null() {
            unsafe { (*win).w_buffer }
        } else if !buf.is_null() {
            buf
        } else {
            curbuf.get()
        };
        let mut line_count: linenr_T = unsafe { (*rbuf).b_ml.ml_line_count };
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
            unsafe {
                redraw_buf_range_later(rbuf, 1 as linenr_T + begin as linenr_T, end as linenr_T)
            };
        }
    }
    if has_key(
        unsafe { (*opts).is_set__redraw_ },
        KEYSET_OPTIDX_redraw__valid,
    ) || has_key(
        unsafe { (*opts).is_set__redraw_ },
        KEYSET_OPTIDX_redraw__range,
    ) {
        unsafe {
            (*opts).flush = if has_key((*opts).is_set__redraw_, KEYSET_OPTIDX_redraw__flush) {
                (*opts).flush as ::core::ffi::c_int
            } else {
                1
            } != 0;
        }
    }
    let mut flush_ui: bool = unsafe { (*opts).flush };
    if unsafe { (*opts).tabline } {
        if redraw_tabline.get() as ::core::ffi::c_int != 0
            && first_window().is_some_and(|wp| wp.w_lines_valid == 0)
        {
            unsafe { (*opts).flush = true };
        } else {
            unsafe { draw_tabline() };
        }
        flush_ui = true;
    }
    let mut save_lz: bool = p_lz.get() != 0;
    let redraw = Allow::redraw();
    p_lz.set(0);
    if unsafe { (*opts).statuscolumn } as ::core::ffi::c_int != 0
        || unsafe { (*opts).statusline } as ::core::ffi::c_int != 0
        || unsafe { (*opts).winbar } as ::core::ffi::c_int != 0
    {
        if win.is_null() {
            for wp in windows().map(Win::raw) {
                if buf.is_null() || unsafe { (*wp).w_buffer } == buf {
                    unsafe { redraw_status(wp, opts, &raw mut (*opts).flush) };
                }
            }
        } else {
            unsafe { redraw_status(win, opts, &raw mut (*opts).flush) };
        }
        flush_ui = true;
    }
    let mut cwin: *mut win_T = if !win.is_null() { win } else { curwin.get() };
    if unsafe { (*opts).cursor } as ::core::ffi::c_int != 0
        && (unsafe { (*cwin).w_grid.target }.is_null()
            || !unsafe { (*(*cwin).w_grid.target).valid })
    {
        unsafe { (*opts).flush = true };
    }
    if unsafe { (*opts).flush } as ::core::ffi::c_int != 0 && !cmdpreview.get() {
        validate_cursor(unsafe { Win::current() });
        update_topline(unsafe { Win::current() });
        unsafe { update_screen() };
    }
    if unsafe { (*opts).cursor } {
        unsafe { setcursor_mayforce(cwin, true) };
        flush_ui = true;
    }
    if flush_ui {
        unsafe { ui_flush() };
    }
    drop(redraw);
    p_lz.set(save_lz as ::core::ffi::c_int);
    ().reported(error)
}
