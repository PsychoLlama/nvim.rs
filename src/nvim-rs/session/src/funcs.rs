//! Migrated C function implementations for ex_session.c
//!
//! Each `rs_*` function is the Rust implementation of a corresponding C function.
//! The C function body is replaced with a thin wrapper that calls the `rs_*` version.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::ptr_cast_constness)]

use std::ffi::{c_char, c_int, c_uint};

use crate::ffi;

/// OK return value (matches C OK = 1)
pub const OK: c_int = 1;
/// FAIL return value (matches C FAIL = 0)
pub const FAIL: c_int = 0;

/// Session option flag constants (verified via _Static_assert in C)
pub const K_OPT_SSOP_FLAG_BLANK: c_uint = 0x80;
pub const K_OPT_SSOP_FLAG_HELP: c_uint = 0x40;
pub const K_OPT_SSOP_FLAG_TERMINAL: c_uint = 0x10000;
pub const K_OPT_SSOP_FLAG_CURDIR: c_uint = 0x1000;
pub const K_OPT_SSOP_FLAG_SESDIR: c_uint = 0x800;

/// Frame layout constants (verified via _Static_assert in C)
pub const FR_LEAF: c_int = 0;

// =============================================================================
// Phase 2: Window/Frame Predicates
// =============================================================================

/// Check if window "wp" should be stored in the session.
///
/// Returns non-zero (truthy) if the window should be saved.
/// Matches the logic of `ses_do_win()` in ex_session.c.
///
/// # Safety
/// `wp` must be a valid win_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_do_win(wp: ffi::WinPtr) -> c_int {
    // Skip floating windows to avoid issues when restoring the Session. #18432
    if ffi::nvim_ses_win_get_floating(wp) {
        return 0; // false
    }

    let buf = ffi::nvim_ses_win_get_buffer(wp);
    let fname = ffi::nvim_ses_buf_get_fname(buf);
    let ssop_flags = ffi::nvim_ses_get_ssop_flags();

    if fname.is_null() || (!ffi::nvim_ses_buf_is_terminal(buf) && ffi::nvim_ses_bt_nofilename(buf))
    {
        // When 'buftype' is "nofile" can't restore the window contents.
        return (ssop_flags & K_OPT_SSOP_FLAG_BLANK) as c_int;
    }
    if ffi::nvim_ses_bt_help(buf) {
        return (ssop_flags & K_OPT_SSOP_FLAG_HELP) as c_int;
    }
    if ffi::nvim_ses_bt_terminal(buf) {
        return (ssop_flags & K_OPT_SSOP_FLAG_TERMINAL) as c_int;
    }
    1 // true
}

/// Check if frame "fr" has a window somewhere that we want to save in the Session.
///
/// # Safety
/// `fr` must be a valid frame_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_do_frame(fr: ffi::FramePtr) -> bool {
    let layout = ffi::nvim_ses_frame_get_layout(fr);
    if layout == FR_LEAF {
        let win = ffi::nvim_ses_frame_get_win(fr);
        return rs_ses_do_win(win) != 0;
    }
    // Iterate children (FOR_ALL_FRAMES equivalent)
    let mut frc = ffi::nvim_ses_frame_get_child(fr);
    while !frc.is_null() {
        if rs_ses_do_frame(frc) {
            return true;
        }
        frc = ffi::nvim_ses_frame_get_next(frc);
    }
    false
}

/// Skip frames that don't contain windows we want to save in the Session.
///
/// Returns NULL when there are none.
///
/// # Safety
/// `fr` must be a valid frame_T pointer or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_skipframe(fr: ffi::FramePtr) -> ffi::FramePtr {
    // FOR_ALL_FRAMES(frc, fr) { if (ses_do_frame(frc)) break; }
    let mut frc = fr;
    while !frc.is_null() {
        if rs_ses_do_frame(frc) {
            break;
        }
        frc = ffi::nvim_ses_frame_get_next(frc);
    }
    frc
}

// =============================================================================
// Phase 3: Filename Helpers
// =============================================================================

/// Get the buffer name for `buf`.
///
/// Uses the short file name when the current directory is known at session
/// source time (session flags with curdir/sesdir, no acd, no lcd).
/// Otherwise uses the full file name.
///
/// # Safety
/// `buf` must be a valid buf_T pointer. `flagp` must be a valid pointer to unsigned.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_get_fname(buf: ffi::BufPtr, flagp: *const c_uint) -> *const c_char {
    let sfname = ffi::nvim_ses_buf_get_sfname(buf);
    let ssop_ptr = ffi::nvim_ses_get_ssop_flags_ptr();

    // Use the short file name if:
    // - Buffer has b_sfname
    // - flagp == &ssop_flags (session mode, not view)
    // - (ssop_flags & (CURDIR | SESDIR)) is set
    // - !p_acd
    // - !did_lcd
    if !sfname.is_null()
        && flagp == ssop_ptr
        && (ffi::nvim_ses_get_ssop_flags() & (K_OPT_SSOP_FLAG_CURDIR | K_OPT_SSOP_FLAG_SESDIR)) != 0
        && ffi::nvim_ses_get_p_acd() == 0
        && ffi::nvim_ses_get_did_lcd() == 0
    {
        return sfname;
    }
    ffi::nvim_ses_buf_get_ffname(buf)
}

/// Escape a filename for session writing.
///
/// Replaces backslashes with forward slashes (always kOptSsopFlagSlash)
/// and escapes special characters.
///
/// Returns an allocated string (caller must free with `nvim_ses_xfree`).
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_escape_fname(
    name: *mut c_char,
    _flagp: *mut c_uint,
) -> *mut c_char {
    let sname = ffi::nvim_ses_home_replace_save(name);

    // Always kOptSsopFlagSlash: change all backslashes to forward slashes.
    // Use MB_PTR_ADV equivalent (utfc_ptr2len) to advance past multibyte chars.
    let mut p = sname;
    while *p != 0 {
        if *p == b'\\' as c_char {
            *p = b'/' as c_char;
        }
        let len = ffi::nvim_ses_utfc_ptr2len(p);
        p = p.add(if len > 0 { len as usize } else { 1 });
    }

    // Escape special characters.
    let result = ffi::nvim_ses_vim_strsave_fnameescape(sname);
    ffi::nvim_ses_xfree(sname.cast());
    result
}

/// Write a file name to the session file.
///
/// Takes care of the "slash" option and escapes special characters.
/// Returns OK or FAIL.
///
/// # Safety
/// `fd` must be a valid FILE*. `name` must be a valid C string. `flagp` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_put_fname(
    fd: *mut libc::FILE,
    name: *mut c_char,
    flagp: *mut c_uint,
) -> c_int {
    let p = rs_ses_escape_fname(name, flagp);
    let mut w = crate::writer::SessionWriter::new(fd);

    // Write the escaped filename (fputs equivalent)
    let cstr = std::ffi::CStr::from_ptr(p);
    let retval = if w.write_bytes(cstr.to_bytes()) {
        OK
    } else {
        FAIL
    };
    ffi::nvim_ses_xfree(p.cast());
    retval
}

/// Write a buffer name to the session file, optionally ending the line.
///
/// Returns OK or FAIL.
///
/// # Safety
/// `fd` must be a valid FILE*. `buf` must be a valid buf_T pointer. `flagp` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_ses_fname(
    fd: *mut libc::FILE,
    buf: ffi::BufPtr,
    flagp: *mut c_uint,
    add_eol: bool,
) -> c_int {
    let name = rs_ses_get_fname(buf, flagp);
    if rs_ses_put_fname(fd, name as *mut c_char, flagp) == FAIL {
        return FAIL;
    }
    if add_eol {
        let mut w = crate::writer::SessionWriter::new(fd);
        if !w.write_bytes(b"\n") {
            return FAIL;
        }
    }
    OK
}
