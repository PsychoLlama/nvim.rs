//! Session Loading Infrastructure
//!
//! This module provides FFI helpers for session file loading and restoration.
//! Phase 180 of Rust migration.
//!
//! Session files are VimL scripts that are sourced with `:source`. This module
//! provides helpers for validation, state tracking, and restoration operations.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::similar_names)]
#![allow(clippy::case_sensitive_file_extension_comparisons)]
#![allow(clippy::manual_let_else)]
#![allow(clippy::match_same_arms)]

use std::ffi::{c_char, c_int};

use crate::SessionFlags;

// =============================================================================
// Session File Validation
// =============================================================================

/// Check if a line looks like it starts a session file.
///
/// Session files typically start with "let SessionLoad = 1" or a version command.
/// This helps identify if we're loading a session vs a regular script.
///
/// # Safety
/// `line` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_session_is_session_line(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }

    let line_cstr = std::ffi::CStr::from_ptr(line);
    let line_str = match line_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let trimmed = line_str.trim();
    c_int::from(
        trimmed.starts_with("let SessionLoad")
            || trimmed.starts_with("let v:this_session")
            || trimmed.starts_with("version "),
    )
}

/// Check if a line looks like it starts a view file.
///
/// View files typically start with scrolloff save commands.
///
/// # Safety
/// `line` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_session_is_view_line(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }

    let line_cstr = std::ffi::CStr::from_ptr(line);
    let line_str = match line_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let trimmed = line_str.trim();
    c_int::from(
        trimmed.starts_with("let s:so_save")
            || trimmed.starts_with("setg so=0")
            || trimmed.starts_with("keepjumps"),
    )
}

/// Check if a line is a session file modeline.
///
/// Session files end with `" vim: set ft=vim :`
///
/// # Safety
/// `line` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_session_is_modeline(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }

    let line_cstr = std::ffi::CStr::from_ptr(line);
    let line_str = match line_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let trimmed = line_str.trim();
    c_int::from(trimmed.contains("vim:") && trimmed.contains("ft=vim"))
}

// =============================================================================
// Session Restoration State
// =============================================================================

/// Session restoration state constants
pub const SESSION_STATE_NONE: c_int = 0;
pub const SESSION_STATE_LOADING: c_int = 1;
pub const SESSION_STATE_BUFFERS: c_int = 2;
pub const SESSION_STATE_WINDOWS: c_int = 3;
pub const SESSION_STATE_TABS: c_int = 4;
pub const SESSION_STATE_OPTIONS: c_int = 5;
pub const SESSION_STATE_DONE: c_int = 6;
pub const SESSION_STATE_ERROR: c_int = -1;

/// Get the next restoration state.
#[no_mangle]
pub extern "C" fn rs_session_next_state(current: c_int) -> c_int {
    match current {
        SESSION_STATE_NONE => SESSION_STATE_LOADING,
        SESSION_STATE_LOADING => SESSION_STATE_BUFFERS,
        SESSION_STATE_BUFFERS => SESSION_STATE_WINDOWS,
        SESSION_STATE_WINDOWS => SESSION_STATE_TABS,
        SESSION_STATE_TABS => SESSION_STATE_OPTIONS,
        SESSION_STATE_OPTIONS => SESSION_STATE_DONE,
        SESSION_STATE_DONE => SESSION_STATE_DONE,
        _ => SESSION_STATE_ERROR,
    }
}

/// Check if we're in a loading state.
#[no_mangle]
pub extern "C" fn rs_session_is_loading_state(state: c_int) -> c_int {
    c_int::from(
        state > SESSION_STATE_NONE && state < SESSION_STATE_DONE && state != SESSION_STATE_ERROR,
    )
}

/// Check if session restoration is complete.
#[no_mangle]
pub extern "C" fn rs_session_is_done(state: c_int) -> c_int {
    c_int::from(state == SESSION_STATE_DONE)
}

/// Check if session restoration had an error.
#[no_mangle]
pub extern "C" fn rs_session_has_error(state: c_int) -> c_int {
    c_int::from(state == SESSION_STATE_ERROR)
}

/// Get state name for debugging.
#[no_mangle]
pub extern "C" fn rs_session_state_name(state: c_int) -> *const c_char {
    static NONE: &[u8] = b"none\0";
    static LOADING: &[u8] = b"loading\0";
    static BUFFERS: &[u8] = b"buffers\0";
    static WINDOWS: &[u8] = b"windows\0";
    static TABS: &[u8] = b"tabs\0";
    static OPTIONS: &[u8] = b"options\0";
    static DONE: &[u8] = b"done\0";
    static ERROR: &[u8] = b"error\0";
    static UNKNOWN: &[u8] = b"unknown\0";

    let name = match state {
        SESSION_STATE_NONE => NONE,
        SESSION_STATE_LOADING => LOADING,
        SESSION_STATE_BUFFERS => BUFFERS,
        SESSION_STATE_WINDOWS => WINDOWS,
        SESSION_STATE_TABS => TABS,
        SESSION_STATE_OPTIONS => OPTIONS,
        SESSION_STATE_DONE => DONE,
        SESSION_STATE_ERROR => ERROR,
        _ => UNKNOWN,
    };
    name.as_ptr().cast::<c_char>()
}

// =============================================================================
// Source Command Helpers
// =============================================================================

/// Check if a file path looks like a session file by extension.
///
/// # Safety
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_session_is_session_path(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = std::ffi::CStr::from_ptr(path);
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    c_int::from(
        path_str.ends_with("Session.vim")
            || path_str.ends_with(".session")
            || path_str.ends_with(".sess"),
    )
}

/// Check if a file path looks like a view file.
///
/// View files are typically in the viewdir with special naming.
///
/// # Safety
/// `path` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_session_is_view_path(path: *const c_char) -> c_int {
    if path.is_null() {
        return 0;
    }

    let path_cstr = std::ffi::CStr::from_ptr(path);
    let path_str = match path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    // View files contain "=+" or "==" in their path (encoded separators)
    c_int::from(path_str.contains("=+") || path_str.contains("==") || path_str.contains("=-"))
}

// =============================================================================
// Session Variable Helpers
// =============================================================================

/// Get the SessionLoad variable name.
#[no_mangle]
pub extern "C" fn rs_session_load_var_name() -> *const c_char {
    static NAME: &[u8] = b"SessionLoad\0";
    NAME.as_ptr().cast::<c_char>()
}

/// Get the v:this_session variable name.
#[no_mangle]
pub extern "C" fn rs_session_this_session_var() -> *const c_char {
    static NAME: &[u8] = b"v:this_session\0";
    NAME.as_ptr().cast::<c_char>()
}

/// Get the s:wipebuf variable name.
#[no_mangle]
pub extern "C" fn rs_session_wipebuf_var() -> *const c_char {
    static NAME: &[u8] = b"s:wipebuf\0";
    NAME.as_ptr().cast::<c_char>()
}

/// Get the shortmess save variable name.
#[no_mangle]
pub extern "C" fn rs_session_shortmess_var() -> *const c_char {
    static NAME: &[u8] = b"s:shortmess_save\0";
    NAME.as_ptr().cast::<c_char>()
}

// =============================================================================
// Window Restoration Helpers
// =============================================================================

/// Calculate the restored window height.
///
/// Uses the formula: (height * lines + original_lines/2) / original_lines
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn rs_session_restore_height(
    original_height: c_int,
    original_lines: c_int,
    current_lines: c_int,
) -> c_int {
    if original_lines <= 0 {
        return original_height;
    }
    ((i64::from(original_height) * i64::from(current_lines) + i64::from(original_lines) / 2)
        / i64::from(original_lines)) as c_int
}

/// Calculate the restored window width.
///
/// Uses the formula: (width * columns + original_columns/2) / original_columns
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn rs_session_restore_width(
    original_width: c_int,
    original_columns: c_int,
    current_columns: c_int,
) -> c_int {
    if original_columns <= 0 {
        return original_width;
    }
    ((i64::from(original_width) * i64::from(current_columns) + i64::from(original_columns) / 2)
        / i64::from(original_columns)) as c_int
}

// =============================================================================
// Session File Parsing Helpers
// =============================================================================

/// Check if a command line is a wincmd.
///
/// # Safety
/// `line` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_session_is_wincmd(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }

    let line_cstr = std::ffi::CStr::from_ptr(line);
    let line_str = match line_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let trimmed = line_str.trim();
    c_int::from(trimmed.starts_with("wincmd ") || trimmed.contains("wincmd "))
}

/// Check if a command line is a buffer add.
///
/// # Safety
/// `line` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_session_is_badd(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }

    let line_cstr = std::ffi::CStr::from_ptr(line);
    let line_str = match line_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let trimmed = line_str.trim();
    c_int::from(trimmed.starts_with("badd "))
}

/// Check if a command line is a tabnew or tabnext.
///
/// # Safety
/// `line` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_session_is_tab_cmd(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }

    let line_cstr = std::ffi::CStr::from_ptr(line);
    let line_str = match line_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let trimmed = line_str.trim();
    c_int::from(
        trimmed.starts_with("tabnew")
            || trimmed.starts_with("tabnext")
            || trimmed.starts_with("tabrewind"),
    )
}

/// Check if a command line is an edit or buffer command.
///
/// # Safety
/// `line` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_session_is_edit_cmd(line: *const c_char) -> c_int {
    if line.is_null() {
        return 0;
    }

    let line_cstr = std::ffi::CStr::from_ptr(line);
    let line_str = match line_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };

    let trimmed = line_str.trim();
    c_int::from(
        trimmed.starts_with("edit ")
            || trimmed.starts_with("buffer ")
            || trimmed.starts_with("enew"),
    )
}

// =============================================================================
// Post-Load Validation
// =============================================================================

/// Check if post-load cleanup should run.
///
/// Cleanup runs if we're in a loaded state and haven't errored.
#[no_mangle]
pub extern "C" fn rs_session_should_cleanup(state: c_int, flags: u32) -> c_int {
    let _f = SessionFlags::from_bits_truncate(flags);
    c_int::from(state == SESSION_STATE_DONE)
}

/// Get the SessionLoadPost autocmd event name.
#[no_mangle]
pub extern "C" fn rs_session_loadpost_event() -> *const c_char {
    static EVENT: &[u8] = b"SessionLoadPost\0";
    EVENT.as_ptr().cast::<c_char>()
}

/// Get the SessionWritePost autocmd event name.
#[no_mangle]
pub extern "C" fn rs_session_writepost_event() -> *const c_char {
    static EVENT: &[u8] = b"SessionWritePost\0";
    EVENT.as_ptr().cast::<c_char>()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_is_session_line() {
        unsafe {
            let session = CString::new("let SessionLoad = 1").unwrap();
            assert_eq!(rs_session_is_session_line(session.as_ptr()), 1);

            let this_sess = CString::new("let v:this_session=expand(\"<sfile>:p\")").unwrap();
            assert_eq!(rs_session_is_session_line(this_sess.as_ptr()), 1);

            let version = CString::new("version 6.0").unwrap();
            assert_eq!(rs_session_is_session_line(version.as_ptr()), 1);

            let other = CString::new("echo 'hello'").unwrap();
            assert_eq!(rs_session_is_session_line(other.as_ptr()), 0);
        }
    }

    #[test]
    fn test_is_view_line() {
        unsafe {
            let view = CString::new("let s:so_save = &g:so").unwrap();
            assert_eq!(rs_session_is_view_line(view.as_ptr()), 1);

            let keepjumps = CString::new("keepjumps exe s:l").unwrap();
            assert_eq!(rs_session_is_view_line(keepjumps.as_ptr()), 1);

            let other = CString::new("set number").unwrap();
            assert_eq!(rs_session_is_view_line(other.as_ptr()), 0);
        }
    }

    #[test]
    fn test_state_machine() {
        assert_eq!(
            rs_session_next_state(SESSION_STATE_NONE),
            SESSION_STATE_LOADING
        );
        assert_eq!(
            rs_session_next_state(SESSION_STATE_LOADING),
            SESSION_STATE_BUFFERS
        );
        assert_eq!(
            rs_session_next_state(SESSION_STATE_DONE),
            SESSION_STATE_DONE
        );
        assert_eq!(rs_session_next_state(100), SESSION_STATE_ERROR);

        assert_eq!(rs_session_is_loading_state(SESSION_STATE_LOADING), 1);
        assert_eq!(rs_session_is_loading_state(SESSION_STATE_DONE), 0);
        assert_eq!(rs_session_is_loading_state(SESSION_STATE_NONE), 0);

        assert_eq!(rs_session_is_done(SESSION_STATE_DONE), 1);
        assert_eq!(rs_session_is_done(SESSION_STATE_LOADING), 0);

        assert_eq!(rs_session_has_error(SESSION_STATE_ERROR), 1);
        assert_eq!(rs_session_has_error(SESSION_STATE_DONE), 0);
    }

    #[test]
    fn test_is_session_path() {
        unsafe {
            let sess = CString::new("/home/user/Session.vim").unwrap();
            assert_eq!(rs_session_is_session_path(sess.as_ptr()), 1);

            let sess2 = CString::new("project.session").unwrap();
            assert_eq!(rs_session_is_session_path(sess2.as_ptr()), 1);

            let other = CString::new("/home/user/init.vim").unwrap();
            assert_eq!(rs_session_is_session_path(other.as_ptr()), 0);
        }
    }

    #[test]
    fn test_is_view_path() {
        unsafe {
            let view =
                CString::new("~/.local/share/nvim/view/=+home=+user=+file.txt=.vim").unwrap();
            assert_eq!(rs_session_is_view_path(view.as_ptr()), 1);

            let other = CString::new("/home/user/file.vim").unwrap();
            assert_eq!(rs_session_is_view_path(other.as_ptr()), 0);
        }
    }

    #[test]
    fn test_restore_dimensions() {
        // 50% of 100 rows -> scaled to 80 rows
        assert_eq!(rs_session_restore_height(50, 100, 80), 40);

        // Handle zero original
        assert_eq!(rs_session_restore_height(50, 0, 80), 50);

        // 80 cols of 160 -> scaled to 120
        assert_eq!(rs_session_restore_width(80, 160, 120), 60);
    }

    #[test]
    fn test_command_detection() {
        unsafe {
            let wincmd = CString::new("wincmd w").unwrap();
            assert_eq!(rs_session_is_wincmd(wincmd.as_ptr()), 1);

            let badd = CString::new("badd +1 /home/user/file.txt").unwrap();
            assert_eq!(rs_session_is_badd(badd.as_ptr()), 1);

            let tabnew = CString::new("tabnew").unwrap();
            assert_eq!(rs_session_is_tab_cmd(tabnew.as_ptr()), 1);

            let edit = CString::new("edit /home/user/file.txt").unwrap();
            assert_eq!(rs_session_is_edit_cmd(edit.as_ptr()), 1);
        }
    }

    #[test]
    fn test_variable_names() {
        unsafe {
            let load_var = std::ffi::CStr::from_ptr(rs_session_load_var_name());
            assert_eq!(load_var.to_str().unwrap(), "SessionLoad");

            let this_sess = std::ffi::CStr::from_ptr(rs_session_this_session_var());
            assert_eq!(this_sess.to_str().unwrap(), "v:this_session");
        }
    }

    #[test]
    fn test_event_names() {
        unsafe {
            let loadpost = std::ffi::CStr::from_ptr(rs_session_loadpost_event());
            assert_eq!(loadpost.to_str().unwrap(), "SessionLoadPost");

            let writepost = std::ffi::CStr::from_ptr(rs_session_writepost_event());
            assert_eq!(writepost.to_str().unwrap(), "SessionWritePost");
        }
    }
}
