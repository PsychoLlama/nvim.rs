//! File info display (`fileinfo`) for Neovim buffers.
//!
//! This module implements `fileinfo()` which formats and displays the Ctrl-G
//! status line for the current buffer.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::as_ptr_cast_mut)]

use std::ffi::{c_char, c_int};

use crate::{BufHandle, WinHandle};

// Buffer flags (from buffer_defs.h) — correct values matching C header.
const BF_NOTEDITED: c_int = 0x08;
const BF_NEW: c_int = 0x10;
const BF_READERR: c_int = 0x40;
const BF_WRITE_MASK: c_int = BF_NOTEDITED + BF_NEW + BF_READERR;

/// File I/O and sprintf buffer size (matches IOSIZE in globals.h).
const IOSIZE: usize = 1025;

// ML_EMPTY flag (from memline_defs.h)
const ML_EMPTY: c_int = 0x01;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_curwin() -> WinHandle;

    fn nvim_buf_get_fnum(buf: BufHandle) -> c_int;
    fn nvim_buf_get_flags(buf: BufHandle) -> c_int;
    fn nvim_buf_get_b_p_ro(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> c_int;
    fn nvim_buf_get_ml_flags(buf: BufHandle) -> c_int;

    fn nvim_curbuf_get_ffname() -> *const c_char;
    fn nvim_curbuf_get_fname() -> *const c_char;

    fn rs_buf_spname(buf: BufHandle) -> *mut c_char;
    fn rs_bt_dontwrite(buf: BufHandle) -> bool;
    fn nvim_curbufIsChanged() -> c_int;

    fn nvim_shortmess_mod() -> c_int;
    fn nvim_shortmess_ro() -> c_int;

    fn nvim_get_p_ru() -> c_int;

    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_get_cursor_col(wp: WinHandle) -> c_int;
    fn nvim_win_get_virtcol(wp: WinHandle) -> c_int;
    fn nvim_validate_virtcol(wp: WinHandle);

    fn rs_calc_percentage(part: i64, whole: i64) -> c_int;
    fn rs_col_print(buf: *mut u8, buflen: usize, col: c_int, vcol: c_int) -> c_int;
    fn rs_append_arg_number(wp: WinHandle, buf: *mut c_char, buflen: usize) -> c_int;

    fn nvim_msg_start();
    fn nvim_msg_scroll_get() -> c_int;
    fn nvim_msg_scroll_set(val: c_int);
    fn nvim_msg_call(s: *const c_char, hl_id: c_int) -> bool;
    fn nvim_msg_trunc(s: *mut c_char, force: bool, hl_id: c_int) -> *const c_char;
    fn nvim_restart_edit_get() -> c_int;
    fn nvim_msg_scrolled_get() -> c_int;
    fn nvim_need_wait_return_get() -> bool;
    fn nvim_set_keep_msg(s: *const c_char);

    fn nvim_home_replace(
        buf: BufHandle,
        src: *const c_char,
        dst: *mut c_char,
        dstlen: usize,
        one: bool,
    ) -> usize;

    fn nvim_msg_modified() -> *const c_char;
    fn nvim_msg_not_edited() -> *const c_char;
    fn nvim_msg_new() -> *const c_char;
    fn nvim_msg_read_errors() -> *const c_char;
    fn nvim_msg_ro() -> *const c_char;
    fn nvim_msg_readonly() -> *const c_char;
    fn nvim_no_lines_msg() -> *const c_char;
    fn nvim_ngettext_line_count(n: i64) -> *const c_char;
    fn nvim_fileinfo_line_fmt() -> *const c_char;
}

// =============================================================================
// Buffer helpers
// =============================================================================

/// Append a C string to a fixed buffer, returning the new write position.
/// Stops at IOSIZE - 1 bytes (leaves room for NUL).
unsafe fn append_cstr(buf: &mut [u8; IOSIZE], pos: usize, cstr: *const c_char) -> usize {
    if cstr.is_null() {
        return pos;
    }
    let mut i = pos;
    let mut p = cstr;
    while i < IOSIZE - 1 {
        let b = *p as u8;
        if b == 0 {
            break;
        }
        buf[i] = b;
        i += 1;
        p = p.add(1);
    }
    i
}

/// Append a Rust `&str` to a fixed buffer, returning the new write position.
fn append_str(buf: &mut [u8; IOSIZE], pos: usize, s: &str) -> usize {
    let bytes = s.as_bytes();
    let available = (IOSIZE - 1).saturating_sub(pos);
    let copy_len = bytes.len().min(available);
    buf[pos..pos + copy_len].copy_from_slice(&bytes[..copy_len]);
    pos + copy_len
}

// =============================================================================
// fileinfo implementation
// =============================================================================

/// Print info about the current buffer (Ctrl-G output).
///
/// # Safety
///
/// Calls external C functions. Must be called on the main thread with valid
/// Neovim state.
pub unsafe fn fileinfo_impl(fullname: c_int, shorthelp: bool, dont_truncate: bool) {
    let mut buffer = [0u8; IOSIZE];
    let mut pos: usize = 0;

    let curbuf = nvim_get_curbuf();
    let curwin = nvim_get_curwin();

    // If fullname > 1: prepend "buf N: "
    if fullname > 1 {
        let fnum = nvim_buf_get_fnum(curbuf);
        pos = append_str(&mut buffer, pos, &format!("buf {fnum}: "));
    }

    // Opening quote
    if pos < IOSIZE - 1 {
        buffer[pos] = b'"';
        pos += 1;
    }

    // Buffer name: special name or home-replaced filename
    let spname = rs_buf_spname(curbuf);
    if spname.is_null() {
        let name: *const c_char = if fullname == 0 {
            let fname = nvim_curbuf_get_fname();
            if fname.is_null() {
                nvim_curbuf_get_ffname()
            } else {
                fname
            }
        } else {
            nvim_curbuf_get_ffname()
        };
        // home_replace writes directly into remaining buffer space
        let null_buf = BufHandle(std::ptr::null_mut());
        let shorthelp_buf = if shorthelp { curbuf } else { null_buf };
        let written = nvim_home_replace(
            shorthelp_buf,
            name,
            buffer[pos..].as_mut_ptr().cast::<c_char>(),
            IOSIZE - pos,
            true,
        );
        pos += written;
    } else {
        pos = append_cstr(&mut buffer, pos, spname);
    }

    // Status flags: closing quote, modified, not-edited, new, read-errors, readonly, trailing space
    let flags = nvim_buf_get_flags(curbuf);
    let dontwrite = rs_bt_dontwrite(curbuf);
    let is_changed = nvim_curbufIsChanged() != 0;
    let is_ro = nvim_buf_get_b_p_ro(curbuf) != 0;

    // Closing quote
    if pos < IOSIZE - 1 {
        buffer[pos] = b'"';
        pos += 1;
    }

    // Modified indicator (or space)
    if is_changed {
        if nvim_shortmess_mod() != 0 {
            pos = append_cstr(&mut buffer, pos, c" [+]".as_ptr().cast::<c_char>());
        } else {
            pos = append_cstr(&mut buffer, pos, nvim_msg_modified());
        }
    } else if pos < IOSIZE - 1 {
        buffer[pos] = b' ';
        pos += 1;
    }

    // [Not edited]
    if (flags & BF_NOTEDITED) != 0 && !dontwrite {
        pos = append_cstr(&mut buffer, pos, nvim_msg_not_edited());
    }

    // [New]
    if (flags & BF_NEW) != 0 && !dontwrite {
        pos = append_cstr(&mut buffer, pos, nvim_msg_new());
    }

    // [Read errors]
    if (flags & BF_READERR) != 0 {
        pos = append_cstr(&mut buffer, pos, nvim_msg_read_errors());
    }

    // [readonly]
    if is_ro {
        let ro_str = if nvim_shortmess_ro() != 0 {
            nvim_msg_ro()
        } else {
            nvim_msg_readonly()
        };
        pos = append_cstr(&mut buffer, pos, ro_str);
    }

    // Trailing space if any of the writable-state markers appeared
    if (is_changed || (flags & BF_WRITE_MASK) != 0 || is_ro) && pos < IOSIZE - 1 {
        buffer[pos] = b' ';
        pos += 1;
    }

    // Line count section
    let ml_line_count = nvim_buf_get_ml_line_count(curbuf);
    let ml_flags = nvim_buf_get_ml_flags(curbuf);

    if (ml_flags & ML_EMPTY) != 0 {
        // "--No lines in buffer--"
        pos = append_cstr(&mut buffer, pos, nvim_no_lines_msg());
    } else if nvim_get_p_ru() != 0 {
        // Ruler already on screen — just show "N line(s) --P%--"
        let lnum = nvim_win_get_cursor_lnum(curwin);
        let pct = rs_calc_percentage(i64::from(lnum), i64::from(ml_line_count));
        let fmt = nvim_ngettext_line_count(i64::from(ml_line_count));
        let remaining = IOSIZE - pos;
        let n = libc::snprintf(
            buffer[pos..].as_mut_ptr().cast::<c_char>(),
            remaining,
            fmt,
            i64::from(ml_line_count),
            pct,
        );
        if n > 0 {
            pos += (n as usize).min(remaining.saturating_sub(1));
        }
    } else {
        // Full: "line N of M --P%-- col C"
        let lnum = nvim_win_get_cursor_lnum(curwin);
        let pct = rs_calc_percentage(i64::from(lnum), i64::from(ml_line_count));
        let fmt = nvim_fileinfo_line_fmt();
        let remaining = IOSIZE - pos;
        let n = libc::snprintf(
            buffer[pos..].as_mut_ptr().cast::<c_char>(),
            remaining,
            fmt,
            i64::from(lnum),
            i64::from(ml_line_count),
            pct,
        );
        if n > 0 {
            pos += (n as usize).min(remaining.saturating_sub(1));
        }

        // validate_virtcol updates w_virtcol
        nvim_validate_virtcol(curwin);
        let col = nvim_win_get_cursor_col(curwin) + 1;
        let vcol = nvim_win_get_virtcol(curwin) + 1;
        let n = rs_col_print(buffer[pos..].as_mut_ptr(), IOSIZE - pos, col, vcol);
        if n > 0 {
            pos += n as usize;
        }
    }

    // Argument number appended to buffer
    let written = rs_append_arg_number(
        curwin,
        buffer[pos..].as_mut_ptr().cast::<c_char>(),
        IOSIZE - pos,
    );
    if written > 0 {
        pos += written as usize;
    }

    // NUL-terminate
    if pos < IOSIZE {
        buffer[pos] = 0;
    }

    // Display the message
    if dont_truncate {
        nvim_msg_start();
        let saved_scroll = nvim_msg_scroll_get();
        nvim_msg_scroll_set(1);
        let _ = nvim_msg_call(buffer.as_ptr().cast::<c_char>(), 0);
        nvim_msg_scroll_set(saved_scroll);
    } else {
        let p = nvim_msg_trunc(buffer.as_mut_ptr().cast::<c_char>(), false, 0);
        if nvim_restart_edit_get() != 0
            || (nvim_msg_scrolled_get() != 0 && !nvim_need_wait_return_get())
        {
            nvim_set_keep_msg(p);
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Print info about the current buffer (rs_ prefixed version).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_fileinfo(fullname: c_int, shorthelp: bool, dont_truncate: bool) {
    fileinfo_impl(fullname, shorthelp, dont_truncate);
}

/// C export: `fileinfo`.
#[unsafe(export_name = "fileinfo")]
pub unsafe extern "C" fn fileinfo_export(fullname: c_int, shorthelp: bool, dont_truncate: bool) {
    fileinfo_impl(fullname, shorthelp, dont_truncate);
}
