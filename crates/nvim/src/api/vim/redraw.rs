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
use crate::winlayer::{Live, Win, first_window, windows};
use core::ffi::CStr;

/// The decoded keyset, whose caller has promised it outlives the value.
type Redraw = Live<KeyDict_redraw>;

/// One window's share of the redraw -- its status column, winbar and status
/// line -- answering what `flush` becomes.
fn redraw_status(mut wp: Win, opts: Redraw, flush: bool) -> bool {
    // SAFETY: a window's `'statuscolumn'` is a live NUL-terminated string.
    let has_statuscolumn = unsafe { *wp.w_onebuf_opt.wo_stc } as ::core::ffi::c_int != NUL;
    if opts.statuscolumn && has_statuscolumn {
        wp.w_nrwidth_line_count = 0 as linenr_T;
        changed_window_setting(wp);
    }
    let old_row_offset = wp.w_grid.row_offset;
    // SAFETY: `wp` is a live window.
    unsafe { win_grid_alloc(wp.raw()) };
    let flush = flush || wp.w_lines_valid == 0 || wp.w_grid.row_offset != old_row_offset;
    let status = opts.statusline || opts.winbar;
    if flush && status {
        wp.w_redr_status = true;
    } else if status {
        // SAFETY: `wp` is a live window, and the last call puts back the
        // namespace the first one set.
        unsafe {
            win_check_ns_hl(wp.raw());
            if opts.winbar {
                win_redr_winbar(wp.raw());
            }
            if opts.statusline {
                win_redr_status(wp.raw());
            }
            win_check_ns_hl(::core::ptr::null_mut::<win_T>());
        }
    }
    flush
}

/// Mark stale whatever `opts` names, and flush the UI if it asks.
///
/// # Safety
/// `opts` must be the caller's decoded keyset, whose `range` array names its
/// own items.
pub unsafe fn nvim__redraw(opts: *mut KeyDict_redraw) -> Result<(), Error> {
    let mut error = ERROR_INIT;
    let err = &raw mut error;
    // SAFETY: the caller's keyset, live for the whole call.
    let mut opts = unsafe { Redraw::new(opts) };
    let keys = opts.is_set__redraw_;
    let set = |key| has_key(keys, key);
    let mut win: *mut win_T = ::core::ptr::null_mut::<win_T>();
    let mut buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    if set(KEYSET_OPTIDX_redraw__win) {
        // SAFETY: `err` is this frame's own slot.
        win = unsafe { find_window_by_handle(opts.win, err) };
        if error.is_set() {
            return ().reported(error);
        }
    }
    if set(KEYSET_OPTIDX_redraw__buf) {
        if !win.is_null() {
            report(&mut error, c"cannot use both 'buf' and 'win'");
            return ().reported(error);
        }
        // SAFETY: `err` is this frame's own slot.
        buf = unsafe { find_buffer_by_handle(opts.buf, err) };
        if error.is_set() {
            return ().reported(error);
        }
    }
    // `win` and `buf` say *where*; at least one other key has to say *what*.
    let named = u32::from(!win.is_null()) + u32::from(!buf.is_null());
    if keys.count_ones() <= named {
        report(&mut error, c"at least one action required");
        return ().reported(error);
    }
    if set(KEYSET_OPTIDX_redraw__valid) {
        let type_0 = if opts.valid { UPD_VALID } else { UPD_NOT_VALID };
        // SAFETY: `win` and `buf` are the live objects the lookups answered.
        unsafe {
            if !win.is_null() {
                redraw_later(win, type_0);
            } else if !buf.is_null() {
                redraw_buf_later(buf, type_0);
            } else {
                redraw_all_later(type_0);
            }
        }
    }
    if set(KEYSET_OPTIDX_redraw__range) {
        // SAFETY: the caller's keyset -- `range` names its own items.
        let pair = unsafe {
            let range = opts.range;
            (range.size == 2).then(|| (*range.items, *range.items.add(1)))
        };
        // SAFETY: the tags say which arm of each union is the live one.
        let range = pair.filter(|(begin, end)| unsafe {
            begin.type_0 == kObjectTypeInteger
                && end.type_0 == kObjectTypeInteger
                && begin.data.integer >= 0
                && end.data.integer >= -1
        });
        let Some((begin_obj, end_obj)) = range else {
            report(&mut error, c"Invalid 'range': Expected 2-tuple of Integers");
            return ().reported(error);
        };
        // SAFETY: as above -- both are Integers.
        let (begin_raw, end_raw) = unsafe { (begin_obj.data.integer, end_obj.data.integer) };
        let rbuf: *mut buf_T = if !win.is_null() {
            // SAFETY: `win` is the live window the lookup answered.
            unsafe { (*win).w_buffer }
        } else if !buf.is_null() {
            buf
        } else {
            curbuf.get()
        };
        // SAFETY: `rbuf` is a live buffer.
        let line_count = int64_t::from(unsafe { (*rbuf).b_ml.ml_line_count });
        // The range is clamped to the buffer, and `-1` means "to the end".
        let begin = begin_raw.min(line_count);
        let end = if end_raw == -1 {
            line_count
        } else {
            end_raw.max(begin).min(line_count)
        };
        if begin < end {
            let (first, last) = (1 + begin as linenr_T, end as linenr_T);
            // SAFETY: as above.
            unsafe { redraw_buf_range_later(rbuf, first, last) };
        }
    }
    // Marking lines stale flushes by default; every other key does not.
    if set(KEYSET_OPTIDX_redraw__valid) || set(KEYSET_OPTIDX_redraw__range) {
        opts.flush = !set(KEYSET_OPTIDX_redraw__flush) || opts.flush;
    }
    let mut flush_ui = opts.flush;
    if opts.tabline {
        // A window that has never been drawn cannot have its tabline drawn
        // on its own; the whole screen has to go first.
        if redraw_tabline.get() && first_window().is_some_and(|wp| wp.w_lines_valid == 0) {
            opts.flush = true;
        } else {
            // SAFETY: the tab line is the editor's own grid.
            unsafe { draw_tabline() };
        }
        flush_ui = true;
    }
    let save_lz = p_lz.get() != 0;
    let redraw = Allow::redraw();
    p_lz.set(0);
    if opts.statuscolumn || opts.statusline || opts.winbar {
        if win.is_null() {
            for wp in windows() {
                if buf.is_null() || wp.w_buffer == buf {
                    opts.flush = redraw_status(wp, opts, opts.flush);
                }
            }
        } else {
            // SAFETY: `win` is the live window the lookup answered.
            let wp = unsafe { Win::new(win) };
            opts.flush = redraw_status(wp, opts, opts.flush);
        }
        flush_ui = true;
    }
    let cwin: *mut win_T = if win.is_null() { curwin.get() } else { win };
    // SAFETY: `cwin` is a live window, and its grid's target is a live grid
    // or null.
    let stale_grid = unsafe {
        let target = (*cwin).w_grid.target;
        target.is_null() || !(*target).valid
    };
    if opts.cursor && stale_grid {
        opts.flush = true;
    }
    if opts.flush && !cmdpreview.get() {
        // SAFETY: `curwin` names a live window for the editor's whole run.
        let cur = unsafe { Win::current() };
        validate_cursor(cur);
        update_topline(cur);
        // SAFETY: the editor's own screen.
        unsafe { update_screen() };
    }
    if opts.cursor {
        // SAFETY: `cwin` is a live window.
        unsafe { setcursor_mayforce(cwin, true) };
        flush_ui = true;
    }
    if flush_ui {
        // SAFETY: the editor's own UI.
        unsafe { ui_flush() };
    }
    drop(redraw);
    p_lz.set(save_lz as ::core::ffi::c_int);
    ().reported(error)
}

/// One of this file's three validation messages.
fn report(err: &mut Error, msg: &CStr) {
    // SAFETY: `err` is the caller's own slot, and the format takes the one C
    // string it is given.
    unsafe { api_set_error(err, kErrorTypeValidation, c"%s".as_ptr(), msg.as_ptr()) };
}
