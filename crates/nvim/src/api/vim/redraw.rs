//! `nvim__redraw()`: asking for parts of the screen to be redrawn.
//!
//! The keyset says *what* is stale -- a buffer, a window, a line range,
//! the statusline, the tabline, the cursor -- and this walks the affected
//! windows marking each of them, then optionally flushes.  `redraw_status`
//! is the per-window half.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::guard::Allow;
use crate::types::NUL;
use crate::winlayer::Buf;
use crate::winlayer::{Live, Win, first_window, windows};
use core::ffi::CStr;

/// The decoded keyset, whose caller has promised it outlives the value.
type Redraw = Live<KeyDict_redraw>;

/// One window's share of the redraw -- its status column, winbar and status
/// line -- answering what `flush` becomes.
fn redraw_status(mut window: Win, opts: Redraw, flush: bool) -> bool {
    let (statuscolumn, statusline, winbar) = (
        opts.statuscolumn.unwrap_or(false),
        opts.statusline.unwrap_or(false),
        opts.winbar.unwrap_or(false),
    );
    // SAFETY: a window's `'statuscolumn'` is a live NUL-terminated string.
    let has_statuscolumn = ::core::ffi::c_int::from(unsafe { *window.w_onebuf_opt.wo_stc }) != NUL;
    if statuscolumn && has_statuscolumn {
        window.w_nrwidth_line_count = 0 as LineNr;
        changed_window_setting(window);
    }
    let old_row_offset = window.w_grid.row_offset;
    win_grid_alloc(window);
    let flush = flush || window.w_lines_valid == 0 || window.w_grid.row_offset != old_row_offset;
    let status = statusline || winbar;
    if flush && status {
        window.w_redr_status = true;
    } else if status {
        win_check_ns_hl(Some(window));
        if winbar {
            win_redr_winbar(window);
        }
        if statusline {
            win_redr_status(window);
        }
        win_check_ns_hl(None);
    }
    flush
}

/// Mark stale whatever `opts` names, and flush the UI if it asks.
///
/// # Safety
/// `opts` must be the caller's decoded keyset, whose `range` array names its
/// own items.
// `nvim__redraw` is an API method's own name, published over msgpack-RPC.
#[allow(non_snake_case)]
pub unsafe fn nvim__redraw(opts: *mut KeyDict_redraw) -> Result<(), Error> {
    // SAFETY: the caller's keyset, live for the whole call.
    let opts = unsafe { Redraw::new(opts) };
    let mut win: Option<Win> = None;
    let mut buf: Option<Buf> = None;
    if let Some(handle) = opts.win {
        win = find_window_by_handle(handle)?;
    }
    if let Some(handle) = opts.buf {
        if win.is_some() {
            return Err(report(c"cannot use both 'buf' and 'win'"));
        }
        buf = find_buffer_by_handle(handle)?;
    }
    // `win` and `buf` say *where*; at least one other key has to say *what*.
    let keys_named = u32::from(opts.flush.is_some())
        + u32::from(opts.cursor.is_some())
        + u32::from(opts.valid.is_some())
        + u32::from(opts.statuscolumn.is_some())
        + u32::from(opts.statusline.is_some())
        + u32::from(opts.tabline.is_some())
        + u32::from(opts.winbar.is_some())
        + u32::from(opts.range.is_some())
        + u32::from(opts.win.is_some())
        + u32::from(opts.buf.is_some());
    let placed = u32::from(win.is_some()) + u32::from(buf.is_some());
    if keys_named <= placed {
        return Err(report(c"at least one action required"));
    }
    if let Some(valid) = opts.valid {
        let type_0 = if valid { UPD_VALID } else { UPD_NOT_VALID };
        if let Some(win) = win {
            redraw_later(win, type_0);
        } else if let Some(buf) = buf {
            redraw_buf_later(buf, type_0);
        } else {
            redraw_all_later(type_0);
        }
    }
    if let Some(range) = opts.range.as_ref() {
        // SAFETY: the caller's keyset -- `range` names its own items.
        let pair = (range.len() == 2).then(|| (&range[0], &range[1]));
        let range = pair
            .and_then(|(begin, end)| begin.as_integer().zip(end.as_integer()))
            .filter(|&(begin, end)| begin >= 0 && end >= -1);
        let Some((begin_raw, end_raw)) = range else {
            return Err(report(c"Invalid 'range': Expected 2-tuple of Integers"));
        };
        let rbuf = win.map(Win::buffer).or(buf).unwrap_or_else(Buf::current);
        let line_count = int64_t::from(rbuf.b_ml.ml_line_count);
        // The range is clamped to the buffer, and `-1` means "to the end".
        let begin = begin_raw.min(line_count);
        let end = if end_raw == -1 {
            line_count
        } else {
            end_raw.max(begin).min(line_count)
        };
        if begin < end {
            let first = 1 + LineNr::try_from(begin).expect("the range is clamped to the buffer");
            let last = LineNr::try_from(end).expect("the range is clamped to the buffer");
            redraw_buf_range_later(rbuf, first, last);
        }
    }
    // Marking lines stale flushes by default; every other key does not.
    let mut flush = if opts.valid.is_some() || opts.range.is_some() {
        opts.flush.unwrap_or(true)
    } else {
        opts.flush.unwrap_or(false)
    };
    let mut flush_ui = flush;
    if opts.tabline.unwrap_or(false) {
        // A window that has never been drawn cannot have its tabline drawn
        // on its own; the whole screen has to go first.
        if redraw_tabline.get() && first_window().is_some_and(|wp| wp.w_lines_valid == 0) {
            flush = true;
        } else {
            draw_tabline();
        }
        flush_ui = true;
    }
    let save_lz = p_lz.get() != 0;
    let redraw = Allow::redraw();
    p_lz.set(0);
    if opts.statuscolumn.unwrap_or(false)
        || opts.statusline.unwrap_or(false)
        || opts.winbar.unwrap_or(false)
    {
        if let Some(wp) = win {
            flush = redraw_status(wp, opts, flush);
        } else {
            for wp in windows() {
                if buf.is_none_or(|b| wp.w_buffer == b.raw()) {
                    flush = redraw_status(wp, opts, flush);
                }
            }
        }
        flush_ui = true;
    }
    let cwin = win.unwrap_or_else(Win::current);
    // SAFETY: the grid's target is a live grid or null.
    let stale_grid = unsafe {
        let target = cwin.w_grid.target;
        target.is_null() || !(*target).valid
    };
    let cursor = opts.cursor.unwrap_or(false);
    if cursor && stale_grid {
        flush = true;
    }
    if flush && !cmdpreview.get() {
        let cur = Win::current();
        validate_cursor(cur);
        update_topline(cur);
        let _ = update_screen();
    }
    if cursor {
        setcursor_mayforce(cwin, true);
        flush_ui = true;
    }
    if flush_ui {
        ui_flush();
    }
    drop(redraw);
    p_lz.set(::core::ffi::c_int::from(save_lz));
    Ok(())
}

/// One of this file's three validation messages.
fn report(msg: &CStr) -> Error {
    Error::validation(msg)
}
