//! Migrated C function implementations for ex_session.c
//!
//! Each `rs_*` function is the Rust implementation of a corresponding C function.
//! The C function body is replaced with a thin wrapper that calls the `rs_*` version.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_int, c_uint};

use crate::ffi;

/// OK return value (matches C OK = 1)
pub const OK: c_int = 1;
/// FAIL return value (matches C FAIL = 0)
pub const FAIL: c_int = 0;

/// Session option flag constants (verified via _Static_assert in C)
pub const K_OPT_SSOP_FLAG_BLANK: c_uint = 0x80;
pub const K_OPT_SSOP_FLAG_HELP: c_uint = 0x40;
pub const K_OPT_SSOP_FLAG_TERMINAL: c_uint = 0x10000;

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
