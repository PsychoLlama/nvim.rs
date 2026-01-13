//! Buffer state serialization for sessions
//!
//! This module provides helpers for serializing buffer state to session files,
//! including buffer list, buffer options, and argument list handling.
//! Phase 176 of Rust migration.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::similar_names)]

use std::ffi::{c_char, c_int};

use crate::SessionFlags;

// =============================================================================
// Buffer Handle Type
// =============================================================================

/// Opaque handle to a buffer (buf_T pointer)
pub type BufHandle = *mut std::ffi::c_void;

/// Line number type (matches C's linenr_T)
pub type LineNr = i64;

// =============================================================================
// Buffer List Serialization Helpers
// =============================================================================

/// Check if a buffer should be included in the session buffer list (logic only).
///
/// A buffer is included if:
/// - It has a file name
/// - It's in the buffer list ('buflisted')
/// - Either: has windows OR 'buffers' flag is set
/// - Either: not help OR 'help' flag is set
/// - Either: not terminal OR 'terminal' flag is set
///
/// This is the pure logic version; C code provides the buffer state.
#[no_mangle]
pub extern "C" fn rs_session_should_save_buffer_check(
    has_fname: c_int,
    is_buflisted: c_int,
    has_windows: c_int,
    is_help: c_int,
    is_terminal: c_int,
    flags: u32,
    only_save_windows: c_int,
) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);

    // Must have a file name
    if has_fname == 0 {
        return 0;
    }

    // Must be in buffer list
    if is_buflisted == 0 {
        return 0;
    }

    // If only_save_windows, buffer must have windows
    if only_save_windows != 0 && has_windows == 0 {
        return 0;
    }

    // Skip help buffers unless HELP flag is set
    if is_help != 0 && !f.contains(SessionFlags::HELP) {
        return 0;
    }

    // Skip terminal buffers unless TERMINAL flag is set
    if is_terminal != 0 && !f.contains(SessionFlags::TERMINAL) {
        return 0;
    }

    1
}

/// Get the line number to use for badd command.
///
/// Returns the line number from the first wininfo entry, or 1 if <= 0.
#[no_mangle]
pub extern "C" fn rs_session_buffer_lnum_or_default(lnum: LineNr) -> LineNr {
    if lnum <= 0 { 1 } else { lnum }
}

/// Check if buffer has a short file name that can be used.
///
/// The short name is usable when:
/// - Buffer has b_sfname
/// - Using session flags (not view flags)
/// - CURDIR or SESDIR is set
/// - 'autochdir' is not set
/// - No :lcd was used
#[no_mangle]
pub extern "C" fn rs_session_can_use_sfname(
    has_sfname: c_int,
    is_session: c_int,
    flags: u32,
    p_acd: c_int,
    did_lcd: c_int,
) -> c_int {
    // Must have short name
    if has_sfname == 0 {
        return 0;
    }

    // Must be session (not view)
    if is_session == 0 {
        return 0;
    }

    let f = SessionFlags::from_bits_truncate(flags);

    // Must have curdir or sesdir
    if !f.contains(SessionFlags::CURDIR) && !f.contains(SessionFlags::SESDIR) {
        return 0;
    }

    // Must not have autochdir or did lcd
    if p_acd != 0 || did_lcd != 0 {
        return 0;
    }

    1
}

/// Get the appropriate file name type for a buffer in session file.
///
/// Returns 0 for short name (b_sfname), 1 for full name (b_ffname).
#[no_mangle]
pub extern "C" fn rs_session_fname_type(
    has_sfname: c_int,
    is_session: c_int,
    flags: u32,
    p_acd: c_int,
    did_lcd: c_int,
) -> c_int {
    // Return 1 (full name) if can't use short name, 0 otherwise
    c_int::from(rs_session_can_use_sfname(has_sfname, is_session, flags, p_acd, did_lcd) == 0)
}

// =============================================================================
// Buffer Options Helpers
// =============================================================================

/// Check if buffer options should be saved based on flags.
#[no_mangle]
pub extern "C" fn rs_session_should_save_buf_options(flags: u32) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(f.contains(SessionFlags::OPTIONS) || f.contains(SessionFlags::LOCALOPTIONS))
}

/// Check if buffer mappings should be saved.
#[no_mangle]
pub extern "C" fn rs_session_should_save_buf_mappings(flags: u32) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(f.contains(SessionFlags::OPTIONS) || f.contains(SessionFlags::LOCALOPTIONS))
}

// =============================================================================
// Argument List Serialization Helpers
// =============================================================================

/// Check if full path names should be used for arglist.
///
/// Full paths are used when:
/// - Using view flags, OR
/// - CURDIR is not set, OR
/// - Tab or window has local directory
#[no_mangle]
pub extern "C" fn rs_session_arglist_use_fullname(
    is_view: c_int,
    flags: u32,
    has_tp_localdir: c_int,
    has_wp_localdir: c_int,
) -> c_int {
    // Always use full names for view
    if is_view != 0 {
        return 1;
    }

    let f = SessionFlags::from_bits_truncate(flags);

    // Use full names if curdir is not set
    if !f.contains(SessionFlags::CURDIR) {
        return 1;
    }

    // Use full names if tab or window has local directory
    if has_tp_localdir != 0 || has_wp_localdir != 0 {
        return 1;
    }

    0
}

/// Check if window uses global argument list.
#[no_mangle]
pub extern "C" fn rs_session_uses_global_arglist(uses_global: c_int) -> c_int {
    uses_global
}

/// Check if argument index should be restored in session.
///
/// Only restore if:
/// - Index differs from current
/// - Index is valid (< WARGCOUNT)
/// - Using session flags (not view)
#[no_mangle]
pub extern "C" fn rs_session_should_restore_arg_idx(
    w_arg_idx: c_int,
    current_arg_idx: c_int,
    wargcount: c_int,
    is_session: c_int,
) -> c_int {
    c_int::from(
        w_arg_idx != current_arg_idx && w_arg_idx < wargcount && is_session != 0,
    )
}

// =============================================================================
// Buffer Add Command Generation
// =============================================================================

/// Get the badd command format string.
/// Format: "badd +%d %s\n" where %d is line number and %s is escaped filename.
#[no_mangle]
pub extern "C" fn rs_session_badd_fmt() -> *const c_char {
    static FMT: &[u8] = b"badd +%ld %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the argadd command format string.
/// Format: "$argadd %s\n"
#[no_mangle]
pub extern "C" fn rs_session_argadd_fmt() -> *const c_char {
    static FMT: &[u8] = b"$argadd %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the argglobal command string.
#[no_mangle]
pub extern "C" fn rs_session_argglobal_cmd() -> *const c_char {
    static CMD: &[u8] = b"argglobal\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the arglocal command string.
#[no_mangle]
pub extern "C" fn rs_session_arglocal_cmd() -> *const c_char {
    static CMD: &[u8] = b"arglocal\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the %argdel command string.
#[no_mangle]
pub extern "C" fn rs_session_argdel_cmd() -> *const c_char {
    static CMD: &[u8] = b"%argdel\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the argu command format string.
/// Format: "%dargu\n" where %d is the argument index + 1.
#[no_mangle]
pub extern "C" fn rs_session_argu_fmt() -> *const c_char {
    static FMT: &[u8] = b"%ldargu\n\0";
    FMT.as_ptr().cast::<c_char>()
}

// =============================================================================
// Alternate Buffer Handling
// =============================================================================

/// Check if alternate buffer should be set in session.
///
/// The alternate buffer is set if:
/// - Using session flags
/// - Alt buffer exists
/// - Alt buffer has a non-empty file name
/// - Alt buffer is listed
/// - Alt buffer is not terminal without terminal flag
#[no_mangle]
pub extern "C" fn rs_session_should_set_altbuf(
    is_session: c_int,
    has_alt: c_int,
    has_fname: c_int,
    alt_listed: c_int,
    alt_is_terminal: c_int,
    flags: u32,
) -> c_int {
    if is_session == 0 {
        return 0;
    }
    if has_alt == 0 || has_fname == 0 || alt_listed == 0 {
        return 0;
    }

    let f = SessionFlags::from_bits_truncate(flags);

    // Skip terminal alt buffer unless TERMINAL is set
    if alt_is_terminal != 0 && !f.contains(SessionFlags::TERMINAL) {
        return 0;
    }

    1
}

/// Get the balt command format string.
#[no_mangle]
pub extern "C" fn rs_session_balt_fmt() -> *const c_char {
    static FMT: &[u8] = b"balt %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

// =============================================================================
// Buffer Edit Command Helpers
// =============================================================================

/// Check if buffer should use :edit command (normal file).
///
/// Returns true if buffer has ffname and is not nofile type (or is terminal).
#[no_mangle]
pub extern "C" fn rs_session_buf_needs_edit(
    has_ffname: c_int,
    is_nofile: c_int,
    is_terminal: c_int,
) -> c_int {
    c_int::from(has_ffname != 0 && (is_nofile == 0 || is_terminal != 0))
}

/// Check if buffer is a help buffer that needs special handling.
#[no_mangle]
pub extern "C" fn rs_session_buf_is_help(is_help: c_int) -> c_int {
    is_help
}

/// Get the edit command with bufexists check format.
/// This prevents losing folding info when re-editing.
#[no_mangle]
pub extern "C" fn rs_session_edit_bufexists_fmt() -> *const c_char {
    static FMT: &[u8] = b"if bufexists(fnamemodify(\"%s\", \":p\")) | buffer %s | else | edit %s | endif\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the terminal file fixup format.
#[no_mangle]
pub extern "C" fn rs_session_terminal_fixup_fmt() -> *const c_char {
    static FMT: &[u8] = b"if &buftype ==# 'terminal'\n  silent file %s\nendif\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the enew command for empty buffer.
#[no_mangle]
pub extern "C" fn rs_session_enew_cmd() -> *const c_char {
    static CMD: &[u8] = b"enew\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the help buffer setup command.
#[no_mangle]
pub extern "C" fn rs_session_help_setup_cmd() -> *const c_char {
    static CMD: &[u8] = b"enew | setl bt=help\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the help command format.
#[no_mangle]
pub extern "C" fn rs_session_help_fmt() -> *const c_char {
    static FMT: &[u8] = b"help %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the file command format for naming buffers.
#[no_mangle]
pub extern "C" fn rs_session_file_cmd_fmt() -> *const c_char {
    static FMT: &[u8] = b"file %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_save_buffer_check() {
        // Normal buffer with file name and listed
        assert_eq!(
            rs_session_should_save_buffer_check(1, 1, 1, 0, 0, 0, 0),
            1
        );

        // No file name
        assert_eq!(
            rs_session_should_save_buffer_check(0, 1, 1, 0, 0, 0, 0),
            0
        );

        // Not listed
        assert_eq!(
            rs_session_should_save_buffer_check(1, 0, 1, 0, 0, 0, 0),
            0
        );

        // Only save windows mode, no windows
        assert_eq!(
            rs_session_should_save_buffer_check(1, 1, 0, 0, 0, 0, 1),
            0
        );

        // Help buffer without HELP flag
        assert_eq!(
            rs_session_should_save_buffer_check(1, 1, 1, 1, 0, 0, 0),
            0
        );

        // Help buffer with HELP flag
        assert_eq!(
            rs_session_should_save_buffer_check(1, 1, 1, 1, 0, SessionFlags::HELP.bits(), 0),
            1
        );

        // Terminal without TERMINAL flag
        assert_eq!(
            rs_session_should_save_buffer_check(1, 1, 1, 0, 1, 0, 0),
            0
        );

        // Terminal with TERMINAL flag
        assert_eq!(
            rs_session_should_save_buffer_check(1, 1, 1, 0, 1, SessionFlags::TERMINAL.bits(), 0),
            1
        );
    }

    #[test]
    fn test_buffer_lnum_or_default() {
        assert_eq!(rs_session_buffer_lnum_or_default(0), 1);
        assert_eq!(rs_session_buffer_lnum_or_default(-1), 1);
        assert_eq!(rs_session_buffer_lnum_or_default(42), 42);
    }

    #[test]
    fn test_should_save_buf_options() {
        assert_eq!(rs_session_should_save_buf_options(0), 0);
        assert_eq!(
            rs_session_should_save_buf_options(SessionFlags::OPTIONS.bits()),
            1
        );
        assert_eq!(
            rs_session_should_save_buf_options(SessionFlags::LOCALOPTIONS.bits()),
            1
        );
    }

    #[test]
    fn test_arglist_use_fullname() {
        // View always uses full names
        assert_eq!(rs_session_arglist_use_fullname(1, 0, 0, 0), 1);

        // No CURDIR means full names
        assert_eq!(rs_session_arglist_use_fullname(0, 0, 0, 0), 1);

        // With CURDIR and no local dirs, use short names
        assert_eq!(
            rs_session_arglist_use_fullname(0, SessionFlags::CURDIR.bits(), 0, 0),
            0
        );

        // Tab local dir forces full names
        assert_eq!(
            rs_session_arglist_use_fullname(0, SessionFlags::CURDIR.bits(), 1, 0),
            1
        );

        // Window local dir forces full names
        assert_eq!(
            rs_session_arglist_use_fullname(0, SessionFlags::CURDIR.bits(), 0, 1),
            1
        );
    }

    #[test]
    fn test_should_restore_arg_idx() {
        // Different index, valid, is session
        assert_eq!(rs_session_should_restore_arg_idx(2, 0, 5, 1), 1);

        // Same index
        assert_eq!(rs_session_should_restore_arg_idx(2, 2, 5, 1), 0);

        // Invalid index (>= wargcount)
        assert_eq!(rs_session_should_restore_arg_idx(5, 0, 5, 1), 0);

        // Not session (view)
        assert_eq!(rs_session_should_restore_arg_idx(2, 0, 5, 0), 0);
    }

    #[test]
    fn test_should_set_altbuf() {
        // Valid case
        assert_eq!(rs_session_should_set_altbuf(1, 1, 1, 1, 0, 0), 1);

        // Not session
        assert_eq!(rs_session_should_set_altbuf(0, 1, 1, 1, 0, 0), 0);

        // No alt buffer
        assert_eq!(rs_session_should_set_altbuf(1, 0, 1, 1, 0, 0), 0);

        // Terminal without flag
        assert_eq!(rs_session_should_set_altbuf(1, 1, 1, 1, 1, 0), 0);

        // Terminal with flag
        assert_eq!(
            rs_session_should_set_altbuf(1, 1, 1, 1, 1, SessionFlags::TERMINAL.bits()),
            1
        );
    }

    #[test]
    fn test_buf_needs_edit() {
        // Normal file
        assert_eq!(rs_session_buf_needs_edit(1, 0, 0), 1);

        // No filename
        assert_eq!(rs_session_buf_needs_edit(0, 0, 0), 0);

        // Nofile type
        assert_eq!(rs_session_buf_needs_edit(1, 1, 0), 0);

        // Terminal (nofile but special)
        assert_eq!(rs_session_buf_needs_edit(1, 1, 1), 1);
    }

    #[test]
    fn test_can_use_sfname() {
        // Has sfname, is session, CURDIR set, no acd, no lcd
        assert_eq!(
            rs_session_can_use_sfname(1, 1, SessionFlags::CURDIR.bits(), 0, 0),
            1
        );

        // No sfname
        assert_eq!(
            rs_session_can_use_sfname(0, 1, SessionFlags::CURDIR.bits(), 0, 0),
            0
        );

        // Not session (view)
        assert_eq!(
            rs_session_can_use_sfname(1, 0, SessionFlags::CURDIR.bits(), 0, 0),
            0
        );

        // No CURDIR or SESDIR
        assert_eq!(rs_session_can_use_sfname(1, 1, 0, 0, 0), 0);

        // With SESDIR
        assert_eq!(
            rs_session_can_use_sfname(1, 1, SessionFlags::SESDIR.bits(), 0, 0),
            1
        );

        // With autochdir
        assert_eq!(
            rs_session_can_use_sfname(1, 1, SessionFlags::CURDIR.bits(), 1, 0),
            0
        );

        // After lcd
        assert_eq!(
            rs_session_can_use_sfname(1, 1, SessionFlags::CURDIR.bits(), 0, 1),
            0
        );
    }

    #[test]
    fn test_fname_type() {
        // Can use short name -> 0
        assert_eq!(
            rs_session_fname_type(1, 1, SessionFlags::CURDIR.bits(), 0, 0),
            0
        );

        // Cannot use short name -> 1
        assert_eq!(rs_session_fname_type(0, 1, SessionFlags::CURDIR.bits(), 0, 0), 1);
    }

    #[test]
    fn test_command_strings() {
        // Verify command strings are null-terminated
        unsafe {
            let s = std::ffi::CStr::from_ptr(rs_session_argglobal_cmd());
            assert!(s.to_str().unwrap().contains("argglobal"));

            let s = std::ffi::CStr::from_ptr(rs_session_arglocal_cmd());
            assert!(s.to_str().unwrap().contains("arglocal"));

            let s = std::ffi::CStr::from_ptr(rs_session_enew_cmd());
            assert!(s.to_str().unwrap().contains("enew"));

            let s = std::ffi::CStr::from_ptr(rs_session_badd_fmt());
            assert!(s.to_str().unwrap().contains("badd"));

            let s = std::ffi::CStr::from_ptr(rs_session_balt_fmt());
            assert!(s.to_str().unwrap().contains("balt"));
        }
    }
}
