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

use std::ffi::{c_char, c_int, c_void};

use crate::{messages, BufHandle, WinHandle};

extern "C" {
    static p_icon: c_int;
    static p_title: c_int;
    static stl_syntax: c_int;
    static p_ru: c_int;
    static mut need_maketitle: bool;
    static Columns: c_int;
    static p_titlelen: i64;
    static p_titlestring: *const c_char;
    static p_iconstring: *const c_char;
}

// Buffer flags (from buffer_defs.h) — correct values matching C header.
const BF_NOTEDITED: c_int = 0x08;
const BF_NEW: c_int = 0x10;
const BF_READERR: c_int = 0x40;
const BF_WRITE_MASK: c_int = BF_NOTEDITED + BF_NEW + BF_READERR;

/// File I/O and sprintf buffer size (matches IOSIZE in globals.h).
const IOSIZE: usize = 1025;

// ML_EMPTY flag (from memline_defs.h)
const ML_EMPTY: c_int = 0x01;

// shortmess() argument constants (from option_vars.h, ShmFlags enum)
const SHM_MOD: c_int = b'm' as c_int; // 'modified' flag
const SHM_RO: c_int = b'r' as c_int; // 'readonly' flag

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

    fn shortmess(x: c_int) -> bool;

    fn nvim_win_get_cursor_lnum(wp: WinHandle) -> c_int;
    fn nvim_win_get_cursor_col(wp: WinHandle) -> c_int;
    fn nvim_win_get_virtcol(wp: WinHandle) -> c_int;
    fn nvim_validate_virtcol(wp: WinHandle);

    fn rs_calc_percentage(part: i64, whole: i64) -> c_int;
    fn rs_col_print(buf: *mut u8, buflen: usize, col: c_int, vcol: c_int) -> c_int;
    fn rs_append_arg_number(wp: WinHandle, buf: *mut c_char, buflen: usize) -> c_int;

    fn nvim_msg_start();
    static mut msg_scroll: c_int;
    static restart_edit: c_int;
    static msg_scrolled: c_int;
    static need_wait_return: bool;
    fn msg(s: *const c_char, hl_id: c_int) -> bool;
    fn msg_trunc(s: *mut c_char, force: bool, hl_id: c_int) -> *const c_char;
    fn set_keep_msg(s: *const c_char, hl_id: c_int);

    fn home_replace(
        buf: BufHandle,
        src: *const c_char,
        dst: *mut c_char,
        dstlen: usize,
        one: bool,
    ) -> usize;

    // --- maketitle / resettitle / free_titles ---
    fn nvim_redrawing() -> c_int;
    /// Call `rs_build_stl_str_hl_wrap` (the actual Rust impl of `build_stl_str_hl`).
    fn rs_build_stl_str_hl_wrap(
        wp: WinHandle,
        out: *mut c_char,
        outlen: usize,
        fmt: *mut c_char,
        opt_idx: c_int,
        opt_scope: c_int,
        fillchar: u32,
        maxwidth: c_int,
        hltab: *mut c_void,
        hltab_len: *mut c_void,
        tabtab: *mut c_void,
        stcp: *mut c_void,
    ) -> c_int;
    fn nvim_buf_get_b_ffname(buf: BufHandle) -> *const c_char;
    fn nvim_trans_characters(buf: *mut c_char, bufsize: usize);
    fn nvim_ui_call_set_title(s: *const c_char);
    fn nvim_ui_call_set_icon(s: *const c_char);
    fn nvim_buf_get_lasttitle() -> *const c_char;
    fn nvim_buf_set_lasttitle(s: *mut c_char);
    fn nvim_buf_get_lasticon() -> *const c_char;
    fn nvim_buf_set_lasticon(s: *mut c_char);
    fn nvim_xstrdup(s: *const c_char) -> *mut c_char;
    fn nvim_xfree(p: *mut c_void);
    /// Get the `end_off` from `utf_cp_bounds(str, ptr)`.
    fn nvim_utf_cp_bounds_end_off(str_: *const c_char, ptr: *const c_char) -> c_int;
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

// maketitle constants
const STL_IN_ICON: c_int = 1;
const STL_IN_TITLE: c_int = 2;
const K_OPT_TITLESTRING: c_int = 327;
const K_OPT_ICONSTRING: c_int = 138;

// =============================================================================
// maketitle helpers
// =============================================================================

/// Rust equivalent of `path_tail`: return pointer past the last path separator.
/// Returns the original pointer if no separator found.
unsafe fn path_tail(name: *const c_char) -> *const c_char {
    if name.is_null() {
        return name;
    }
    let mut tail = name;
    let mut p = name;
    loop {
        let b = *p as u8;
        if b == 0 {
            break;
        }
        if b == b'/' || b == b'\\' {
            tail = p.add(1);
        }
        p = p.add(1);
    }
    tail
}

/// Rust equivalent of C `strlen` for a C string.
unsafe fn cstr_len(s: *const c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    p.offset_from(s) as usize
}

/// Copy a C string into a fixed buffer, clamping to `bufsize - 1` bytes.
unsafe fn strcpy_to_buf(buf: *mut c_char, bufsize: usize, src: *const c_char) {
    let mut src = src;
    let mut dst = buf;
    let end = buf.add(bufsize.saturating_sub(1));
    while dst < end && *src != 0 {
        *dst = *src;
        dst = dst.add(1);
        src = src.add(1);
    }
    *dst = 0;
}

/// Check if `str_ptr` differs from current C-side `last` value.
/// If different: frees old, sets new via setter, calls resettitle if setting to NULL.
/// Returns `true` if caller should call resettitle (new value is non-null).
///
/// `get` and `set` are the C getter/setter pair for the static variable.
unsafe fn value_change_via_accessor(
    str_ptr: *const c_char,
    get: unsafe extern "C" fn() -> *const c_char,
    set: unsafe extern "C" fn(*mut c_char),
) -> bool {
    let last = get();
    let str_null = str_ptr.is_null();
    let last_null = last.is_null();

    let changed = if str_null != last_null {
        true
    } else if !str_null && !last_null {
        let mut a = str_ptr;
        let mut b = last;
        loop {
            if *a != *b {
                break true;
            }
            if *a == 0 {
                break false;
            }
            a = a.add(1);
            b = b.add(1);
        }
    } else {
        false
    };

    if changed {
        nvim_xfree(last.cast_mut().cast::<c_void>());
        if str_ptr.is_null() {
            set(std::ptr::null_mut());
            resettitle_impl();
            return false;
        }
        set(nvim_xstrdup(str_ptr));
        return true;
    }
    false
}

/// Core implementation of `maketitle()`.
pub unsafe fn maketitle_impl() {
    let mut title_str: *const c_char = std::ptr::null();
    let mut icon_str: *const c_char = std::ptr::null();
    let mut buf = [0u8; IOSIZE];

    if nvim_redrawing() == 0 {
        need_maketitle = true;
        return;
    }

    need_maketitle = false;

    let opt_p_title = unsafe { p_title };
    let opt_p_icon = unsafe { p_icon };

    if opt_p_title == 0
        && opt_p_icon == 0
        && nvim_buf_get_lasttitle().is_null()
        && nvim_buf_get_lasticon().is_null()
    {
        return;
    }

    let curwin = nvim_get_curwin();

    if opt_p_title != 0 {
        let titlelen = unsafe { p_titlelen };
        let maxlen: c_int = if titlelen > 0 {
            #[allow(clippy::cast_possible_truncation)]
            let tl = titlelen as c_int;
            (tl * unsafe { Columns } / 100).max(10)
        } else {
            0
        };

        let opt_titlestring = unsafe { p_titlestring };
        let titlestring_empty = opt_titlestring.is_null() || *opt_titlestring == 0;

        if titlestring_empty {
            // Default title: "%t%( %M%)%( (%{expand(\"%:~:h\")})%)%a - Nvim"
            let default_title = b"%t%( %M%)%( (%{expand(\"%:~:h\")})%)%a - Nvim\0";
            rs_build_stl_str_hl_wrap(
                curwin,
                buf.as_mut_ptr().cast::<c_char>(),
                IOSIZE,
                default_title.as_ptr().cast::<c_char>().cast_mut(),
                K_OPT_TITLESTRING,
                0,
                0,
                maxlen,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            title_str = buf.as_ptr().cast::<c_char>();
        } else if unsafe { stl_syntax } & STL_IN_TITLE != 0 {
            rs_build_stl_str_hl_wrap(
                curwin,
                buf.as_mut_ptr().cast::<c_char>(),
                IOSIZE,
                opt_titlestring.cast_mut(),
                K_OPT_TITLESTRING,
                0,
                0,
                maxlen,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
            title_str = buf.as_ptr().cast::<c_char>();
        } else {
            title_str = opt_titlestring;
        }
    }

    let mustset =
        value_change_via_accessor(title_str, nvim_buf_get_lasttitle, nvim_buf_set_lasttitle);

    if opt_p_icon != 0 {
        let opt_iconstring = unsafe { p_iconstring };
        let iconstring_empty = opt_iconstring.is_null() || *opt_iconstring == 0;

        if iconstring_empty {
            let curbuf = nvim_get_curbuf();
            let mut name = rs_buf_spname(curbuf).cast_const();
            if name.is_null() {
                name = path_tail(nvim_buf_get_b_ffname(curbuf));
            }
            // Truncate to last 100 bytes (with UTF-8 boundary adjustment)
            let namelen = cstr_len(name) as isize;
            let name = if namelen > 100 {
                let skip = namelen - 100;
                let end_off = nvim_utf_cp_bounds_end_off(name, name.offset(skip));
                name.offset(skip + end_off as isize)
            } else {
                name
            };
            strcpy_to_buf(buf.as_mut_ptr().cast::<c_char>(), IOSIZE, name);
            nvim_trans_characters(buf.as_mut_ptr().cast::<c_char>(), IOSIZE);
        } else if unsafe { stl_syntax } & STL_IN_ICON != 0 {
            rs_build_stl_str_hl_wrap(
                curwin,
                buf.as_mut_ptr().cast::<c_char>(),
                IOSIZE,
                opt_iconstring.cast_mut(),
                K_OPT_ICONSTRING,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
            );
        } else {
            strcpy_to_buf(buf.as_mut_ptr().cast::<c_char>(), IOSIZE, opt_iconstring);
        }
        icon_str = buf.as_ptr().cast::<c_char>();
    }

    let mustset =
        mustset | value_change_via_accessor(icon_str, nvim_buf_get_lasticon, nvim_buf_set_lasticon);

    if mustset {
        resettitle_impl();
    }
}

/// Core implementation of `resettitle()`.
pub unsafe fn resettitle_impl() {
    nvim_ui_call_set_icon(nvim_buf_get_lasticon());
    nvim_ui_call_set_title(nvim_buf_get_lasttitle());
}

/// Core implementation of `free_titles()`.
pub unsafe fn free_titles_impl() {
    let lasttitle = nvim_buf_get_lasttitle().cast_mut();
    if !lasttitle.is_null() {
        nvim_xfree(lasttitle.cast::<c_void>());
        nvim_buf_set_lasttitle(std::ptr::null_mut());
    }
    let lasticon = nvim_buf_get_lasticon().cast_mut();
    if !lasticon.is_null() {
        nvim_xfree(lasticon.cast::<c_void>());
        nvim_buf_set_lasticon(std::ptr::null_mut());
    }
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
        let written = home_replace(
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
        if shortmess(SHM_MOD) {
            pos = append_cstr(&mut buffer, pos, c" [+]".as_ptr().cast::<c_char>());
        } else {
            pos = append_cstr(&mut buffer, pos, messages::msg_modified());
        }
    } else if pos < IOSIZE - 1 {
        buffer[pos] = b' ';
        pos += 1;
    }

    // [Not edited]
    if (flags & BF_NOTEDITED) != 0 && !dontwrite {
        pos = append_cstr(&mut buffer, pos, messages::msg_not_edited());
    }

    // [New]
    if (flags & BF_NEW) != 0 && !dontwrite {
        pos = append_cstr(&mut buffer, pos, messages::msg_new());
    }

    // [Read errors]
    if (flags & BF_READERR) != 0 {
        pos = append_cstr(&mut buffer, pos, messages::msg_read_errors());
    }

    // [readonly]
    if is_ro {
        let ro_str = if shortmess(SHM_RO) {
            messages::msg_ro()
        } else {
            messages::msg_readonly()
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
        pos = append_cstr(&mut buffer, pos, messages::no_lines_msg());
    } else if unsafe { p_ru } != 0 {
        // Ruler already on screen — just show "N line(s) --P%--"
        let lnum = nvim_win_get_cursor_lnum(curwin);
        let pct = rs_calc_percentage(i64::from(lnum), i64::from(ml_line_count));
        let fmt = messages::ngettext_line_count(i64::from(ml_line_count));
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
        let fmt = messages::fileinfo_line_fmt();
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
        let saved_scroll = msg_scroll;
        msg_scroll = 1;
        let _ = msg(buffer.as_ptr().cast::<c_char>(), 0);
        msg_scroll = saved_scroll;
    } else {
        let p = msg_trunc(buffer.as_mut_ptr().cast::<c_char>(), false, 0);
        if restart_edit != 0 || (msg_scrolled != 0 && !need_wait_return) {
            set_keep_msg(p, 0);
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

// =============================================================================
// maketitle / resettitle / free_titles FFI Exports
// =============================================================================

/// rs_-prefixed export: `maketitle` implementation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_maketitle() {
    maketitle_impl();
}

/// C export: `maketitle`.
#[unsafe(export_name = "maketitle")]
pub unsafe extern "C" fn maketitle_export() {
    maketitle_impl();
}

/// rs_-prefixed export: `resettitle` implementation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_resettitle() {
    resettitle_impl();
}

/// C export: `resettitle`.
#[unsafe(export_name = "resettitle")]
pub unsafe extern "C" fn resettitle_export() {
    resettitle_impl();
}

/// rs_-prefixed export: `free_titles` implementation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_free_titles() {
    free_titles_impl();
}

/// C export: `free_titles`.
#[unsafe(export_name = "free_titles")]
pub unsafe extern "C" fn free_titles_export() {
    free_titles_impl();
}
