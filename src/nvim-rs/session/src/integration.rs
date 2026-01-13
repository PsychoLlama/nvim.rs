//! Session Integration & Cleanup
//!
//! This module provides FFI helpers for session event hooks, auto-save
//! infrastructure, and comprehensive error handling.
//! Phase 182 of Rust migration.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::similar_names)]

use std::ffi::{c_char, c_int};

use crate::SessionFlags;

// =============================================================================
// Event Hook Constants
// =============================================================================

/// Session event types
pub const SESSION_EVENT_NONE: c_int = 0;
pub const SESSION_EVENT_WRITE_PRE: c_int = 1;
pub const SESSION_EVENT_WRITE_POST: c_int = 2;
pub const SESSION_EVENT_LOAD_PRE: c_int = 3;
pub const SESSION_EVENT_LOAD_POST: c_int = 4;

/// Get event name string.
#[no_mangle]
pub extern "C" fn rs_session_event_name(event: c_int) -> *const c_char {
    static NONE: &[u8] = b"\0";
    static WRITE_PRE: &[u8] = b"SessionWritePre\0";
    static WRITE_POST: &[u8] = b"SessionWritePost\0";
    static LOAD_PRE: &[u8] = b"SessionLoadPre\0";
    static LOAD_POST: &[u8] = b"SessionLoadPost\0";
    static UNKNOWN: &[u8] = b"Unknown\0";

    let name = match event {
        SESSION_EVENT_NONE => NONE,
        SESSION_EVENT_WRITE_PRE => WRITE_PRE,
        SESSION_EVENT_WRITE_POST => WRITE_POST,
        SESSION_EVENT_LOAD_PRE => LOAD_PRE,
        SESSION_EVENT_LOAD_POST => LOAD_POST,
        _ => UNKNOWN,
    };
    name.as_ptr().cast::<c_char>()
}

/// Check if event is a write event.
#[no_mangle]
pub extern "C" fn rs_session_is_write_event(event: c_int) -> c_int {
    c_int::from(event == SESSION_EVENT_WRITE_PRE || event == SESSION_EVENT_WRITE_POST)
}

/// Check if event is a load event.
#[no_mangle]
pub extern "C" fn rs_session_is_load_event(event: c_int) -> c_int {
    c_int::from(event == SESSION_EVENT_LOAD_PRE || event == SESSION_EVENT_LOAD_POST)
}

/// Check if event is a pre-operation event.
#[no_mangle]
pub extern "C" fn rs_session_is_pre_event(event: c_int) -> c_int {
    c_int::from(event == SESSION_EVENT_WRITE_PRE || event == SESSION_EVENT_LOAD_PRE)
}

/// Check if event is a post-operation event.
#[no_mangle]
pub extern "C" fn rs_session_is_post_event(event: c_int) -> c_int {
    c_int::from(event == SESSION_EVENT_WRITE_POST || event == SESSION_EVENT_LOAD_POST)
}

// =============================================================================
// Auto-Save Infrastructure
// =============================================================================

/// Auto-save trigger types
pub const AUTOSAVE_TRIGGER_NONE: c_int = 0;
pub const AUTOSAVE_TRIGGER_TIMER: c_int = 1;
pub const AUTOSAVE_TRIGGER_LEAVE: c_int = 2;
pub const AUTOSAVE_TRIGGER_FOCUS: c_int = 3;
pub const AUTOSAVE_TRIGGER_MANUAL: c_int = 4;

/// Get trigger name string.
#[no_mangle]
pub extern "C" fn rs_session_trigger_name(trigger: c_int) -> *const c_char {
    static NONE: &[u8] = b"none\0";
    static TIMER: &[u8] = b"timer\0";
    static LEAVE: &[u8] = b"leave\0";
    static FOCUS: &[u8] = b"focus\0";
    static MANUAL: &[u8] = b"manual\0";
    static UNKNOWN: &[u8] = b"unknown\0";

    let name = match trigger {
        AUTOSAVE_TRIGGER_NONE => NONE,
        AUTOSAVE_TRIGGER_TIMER => TIMER,
        AUTOSAVE_TRIGGER_LEAVE => LEAVE,
        AUTOSAVE_TRIGGER_FOCUS => FOCUS,
        AUTOSAVE_TRIGGER_MANUAL => MANUAL,
        _ => UNKNOWN,
    };
    name.as_ptr().cast::<c_char>()
}

/// Check if auto-save should be triggered.
///
/// Auto-save is triggered when:
/// - A valid session file exists (v:this_session is set)
/// - The trigger is enabled
/// - Enough time has passed since last save (for timer trigger)
#[no_mangle]
pub extern "C" fn rs_session_should_autosave(
    has_session: c_int,
    trigger: c_int,
    interval_elapsed: c_int,
) -> c_int {
    if has_session == 0 {
        return 0;
    }

    match trigger {
        AUTOSAVE_TRIGGER_TIMER => interval_elapsed,
        AUTOSAVE_TRIGGER_LEAVE | AUTOSAVE_TRIGGER_FOCUS | AUTOSAVE_TRIGGER_MANUAL => 1,
        _ => 0,
    }
}

/// Calculate the next auto-save time.
///
/// Returns the number of milliseconds until the next auto-save check.
#[no_mangle]
pub extern "C" fn rs_session_next_autosave_ms(
    interval_ms: c_int,
    last_save_elapsed_ms: c_int,
) -> c_int {
    if interval_ms <= 0 {
        return 0;
    }

    let remaining = interval_ms - last_save_elapsed_ms;
    if remaining <= 0 {
        0
    } else {
        remaining
    }
}

// =============================================================================
// Error Handling
// =============================================================================

/// Comprehensive session error codes
pub const ERR_NONE: c_int = 0;
pub const ERR_FILE_OPEN: c_int = 1;
pub const ERR_FILE_WRITE: c_int = 2;
pub const ERR_FILE_READ: c_int = 3;
pub const ERR_NO_BUFFER_NAME: c_int = 4;
pub const ERR_INVALID_FLAGS: c_int = 5;
pub const ERR_INVALID_VIEW_NUM: c_int = 6;
pub const ERR_SOURCE_FAILED: c_int = 7;
pub const ERR_MKDIR_FAILED: c_int = 8;
pub const ERR_CHDIR_FAILED: c_int = 9;
pub const ERR_INVALID_STATE: c_int = 10;
pub const ERR_USER_ABORT: c_int = 11;

/// Get the error message for a session error code.
#[no_mangle]
pub extern "C" fn rs_session_error_message(error: c_int) -> *const c_char {
    static NONE: &[u8] = b"\0";
    static FILE_OPEN: &[u8] = b"E190: Cannot open for writing\0";
    static FILE_WRITE: &[u8] = b"E190: Cannot write session file\0";
    static FILE_READ: &[u8] = b"Cannot read session file\0";
    static NO_BUFFER_NAME: &[u8] = b"E32: No file name\0";
    static INVALID_FLAGS: &[u8] = b"Invalid session/view options\0";
    static INVALID_VIEW_NUM: &[u8] = b"Invalid view number\0";
    static SOURCE_FAILED: &[u8] = b"E484: Can't open file\0";
    static MKDIR_FAILED: &[u8] = b"E739: Cannot create directory\0";
    static CHDIR_FAILED: &[u8] = b"E344: Can't find directory in cdpath\0";
    static INVALID_STATE: &[u8] = b"Invalid session state\0";
    static USER_ABORT: &[u8] = b"Operation cancelled\0";
    static UNKNOWN: &[u8] = b"Unknown error\0";

    let msg = match error {
        ERR_NONE => NONE,
        ERR_FILE_OPEN => FILE_OPEN,
        ERR_FILE_WRITE => FILE_WRITE,
        ERR_FILE_READ => FILE_READ,
        ERR_NO_BUFFER_NAME => NO_BUFFER_NAME,
        ERR_INVALID_FLAGS => INVALID_FLAGS,
        ERR_INVALID_VIEW_NUM => INVALID_VIEW_NUM,
        ERR_SOURCE_FAILED => SOURCE_FAILED,
        ERR_MKDIR_FAILED => MKDIR_FAILED,
        ERR_CHDIR_FAILED => CHDIR_FAILED,
        ERR_INVALID_STATE => INVALID_STATE,
        ERR_USER_ABORT => USER_ABORT,
        _ => UNKNOWN,
    };
    msg.as_ptr().cast::<c_char>()
}

/// Check if an error is fatal (should abort operation).
#[no_mangle]
pub extern "C" fn rs_session_error_is_fatal(error: c_int) -> c_int {
    c_int::from(matches!(
        error,
        ERR_FILE_OPEN
            | ERR_FILE_WRITE
            | ERR_FILE_READ
            | ERR_SOURCE_FAILED
            | ERR_MKDIR_FAILED
            | ERR_CHDIR_FAILED
            | ERR_USER_ABORT
    ))
}

/// Check if an error is recoverable.
#[no_mangle]
pub extern "C" fn rs_session_error_is_recoverable(error: c_int) -> c_int {
    c_int::from(matches!(
        error,
        ERR_NONE
            | ERR_NO_BUFFER_NAME
            | ERR_INVALID_FLAGS
            | ERR_INVALID_VIEW_NUM
            | ERR_INVALID_STATE
    ))
}

// =============================================================================
// Cleanup Helpers
// =============================================================================

/// Session cleanup actions
pub const CLEANUP_NONE: c_int = 0;
pub const CLEANUP_WIPE_BUFFERS: c_int = 1;
pub const CLEANUP_RESTORE_OPTIONS: c_int = 2;
pub const CLEANUP_FIRE_AUTOCMD: c_int = 4;

/// Determine which cleanup actions to take.
#[no_mangle]
pub extern "C" fn rs_session_get_cleanup_actions(is_mksession: c_int, is_loading: c_int) -> c_int {
    let mut actions = CLEANUP_NONE;

    if is_loading != 0 {
        // After loading: fire autocmd, maybe restore options
        actions |= CLEANUP_FIRE_AUTOCMD | CLEANUP_RESTORE_OPTIONS;
    }

    if is_mksession != 0 && is_loading == 0 {
        // After writing: fire autocmd
        actions |= CLEANUP_FIRE_AUTOCMD;
    }

    actions
}

/// Check if a cleanup action is included.
#[no_mangle]
pub extern "C" fn rs_session_has_cleanup(actions: c_int, action: c_int) -> c_int {
    c_int::from((actions & action) != 0)
}

// =============================================================================
// Session State Validation
// =============================================================================

/// Validate session flags for saving.
#[no_mangle]
pub extern "C" fn rs_session_validate_save_flags(flags: u32) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);

    // Can't have both SESDIR and CURDIR
    if f.contains(SessionFlags::SESDIR) && f.contains(SessionFlags::CURDIR) {
        return ERR_INVALID_FLAGS;
    }

    ERR_NONE
}

/// Validate that we can save a session (minimum requirements).
#[no_mangle]
pub extern "C" fn rs_session_can_save(
    has_changes: c_int,
    file_writable: c_int,
    force: c_int,
) -> c_int {
    // If file exists but not writable, and not forcing, fail
    if file_writable == 0 && force == 0 {
        return ERR_FILE_WRITE;
    }

    // We can save even with no changes if forced
    if has_changes != 0 || force != 0 {
        return ERR_NONE;
    }

    ERR_NONE
}

// =============================================================================
// Integration Helpers
// =============================================================================

/// Get the doautoall command for session loading.
#[no_mangle]
pub extern "C" fn rs_session_doautoall_cmd() -> *const c_char {
    static CMD: &[u8] = b"doautoall SessionLoadPost\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the unlet SessionLoad command.
#[no_mangle]
pub extern "C" fn rs_session_unlet_cmd() -> *const c_char {
    static CMD: &[u8] = b"unlet SessionLoad\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the v:this_session assignment command.
#[no_mangle]
pub extern "C" fn rs_session_set_this_session_cmd() -> *const c_char {
    static CMD: &[u8] = b"let v:this_session=expand(\"<sfile>:p\")\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Check if we're currently in a session loading context.
///
/// This is determined by checking if the SessionLoad variable is set.
#[no_mangle]
pub extern "C" fn rs_session_is_loading_context(session_load_set: c_int) -> c_int {
    session_load_set
}

/// Get the session file header comment.
#[no_mangle]
pub extern "C" fn rs_session_file_header() -> *const c_char {
    static HEADER: &[u8] = b"\" Session/View file generated by Neovim\0";
    HEADER.as_ptr().cast::<c_char>()
}

/// Get the session file footer modeline.
#[no_mangle]
pub extern "C" fn rs_session_file_footer() -> *const c_char {
    static FOOTER: &[u8] = b"\" vim: set ft=vim :\0";
    FOOTER.as_ptr().cast::<c_char>()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn test_event_names() {
        unsafe {
            let write_post = CStr::from_ptr(rs_session_event_name(SESSION_EVENT_WRITE_POST));
            assert_eq!(write_post.to_str().unwrap(), "SessionWritePost");

            let load_post = CStr::from_ptr(rs_session_event_name(SESSION_EVENT_LOAD_POST));
            assert_eq!(load_post.to_str().unwrap(), "SessionLoadPost");
        }
    }

    #[test]
    fn test_event_type_checks() {
        assert_eq!(rs_session_is_write_event(SESSION_EVENT_WRITE_PRE), 1);
        assert_eq!(rs_session_is_write_event(SESSION_EVENT_WRITE_POST), 1);
        assert_eq!(rs_session_is_write_event(SESSION_EVENT_LOAD_POST), 0);

        assert_eq!(rs_session_is_load_event(SESSION_EVENT_LOAD_PRE), 1);
        assert_eq!(rs_session_is_load_event(SESSION_EVENT_LOAD_POST), 1);
        assert_eq!(rs_session_is_load_event(SESSION_EVENT_WRITE_POST), 0);

        assert_eq!(rs_session_is_pre_event(SESSION_EVENT_WRITE_PRE), 1);
        assert_eq!(rs_session_is_pre_event(SESSION_EVENT_LOAD_PRE), 1);
        assert_eq!(rs_session_is_pre_event(SESSION_EVENT_WRITE_POST), 0);

        assert_eq!(rs_session_is_post_event(SESSION_EVENT_WRITE_POST), 1);
        assert_eq!(rs_session_is_post_event(SESSION_EVENT_LOAD_POST), 1);
        assert_eq!(rs_session_is_post_event(SESSION_EVENT_WRITE_PRE), 0);
    }

    #[test]
    fn test_trigger_names() {
        unsafe {
            let timer = CStr::from_ptr(rs_session_trigger_name(AUTOSAVE_TRIGGER_TIMER));
            assert_eq!(timer.to_str().unwrap(), "timer");

            let leave = CStr::from_ptr(rs_session_trigger_name(AUTOSAVE_TRIGGER_LEAVE));
            assert_eq!(leave.to_str().unwrap(), "leave");
        }
    }

    #[test]
    fn test_should_autosave() {
        // No session -> no autosave
        assert_eq!(rs_session_should_autosave(0, AUTOSAVE_TRIGGER_TIMER, 1), 0);

        // Timer trigger: depends on interval
        assert_eq!(rs_session_should_autosave(1, AUTOSAVE_TRIGGER_TIMER, 1), 1);
        assert_eq!(rs_session_should_autosave(1, AUTOSAVE_TRIGGER_TIMER, 0), 0);

        // Leave trigger: always fires when session exists
        assert_eq!(rs_session_should_autosave(1, AUTOSAVE_TRIGGER_LEAVE, 0), 1);

        // Unknown trigger
        assert_eq!(rs_session_should_autosave(1, 99, 1), 0);
    }

    #[test]
    fn test_next_autosave_ms() {
        assert_eq!(rs_session_next_autosave_ms(60000, 0), 60000);
        assert_eq!(rs_session_next_autosave_ms(60000, 30000), 30000);
        assert_eq!(rs_session_next_autosave_ms(60000, 60000), 0);
        assert_eq!(rs_session_next_autosave_ms(60000, 70000), 0);
        assert_eq!(rs_session_next_autosave_ms(0, 0), 0);
    }

    #[test]
    fn test_error_messages() {
        unsafe {
            let file_open = CStr::from_ptr(rs_session_error_message(ERR_FILE_OPEN));
            assert!(file_open.to_str().unwrap().contains("190"));

            let no_name = CStr::from_ptr(rs_session_error_message(ERR_NO_BUFFER_NAME));
            assert!(no_name.to_str().unwrap().contains("32"));
        }
    }

    #[test]
    fn test_error_classification() {
        assert_eq!(rs_session_error_is_fatal(ERR_FILE_OPEN), 1);
        assert_eq!(rs_session_error_is_fatal(ERR_FILE_WRITE), 1);
        assert_eq!(rs_session_error_is_fatal(ERR_INVALID_FLAGS), 0);
        assert_eq!(rs_session_error_is_fatal(ERR_NONE), 0);

        assert_eq!(rs_session_error_is_recoverable(ERR_NONE), 1);
        assert_eq!(rs_session_error_is_recoverable(ERR_INVALID_FLAGS), 1);
        assert_eq!(rs_session_error_is_recoverable(ERR_FILE_OPEN), 0);
    }

    #[test]
    fn test_cleanup_actions() {
        // After loading session
        let load_actions = rs_session_get_cleanup_actions(1, 1);
        assert_eq!(
            rs_session_has_cleanup(load_actions, CLEANUP_FIRE_AUTOCMD),
            1
        );
        assert_eq!(
            rs_session_has_cleanup(load_actions, CLEANUP_RESTORE_OPTIONS),
            1
        );

        // After writing session
        let write_actions = rs_session_get_cleanup_actions(1, 0);
        assert_eq!(
            rs_session_has_cleanup(write_actions, CLEANUP_FIRE_AUTOCMD),
            1
        );
        assert_eq!(
            rs_session_has_cleanup(write_actions, CLEANUP_RESTORE_OPTIONS),
            0
        );
    }

    #[test]
    fn test_validate_save_flags() {
        // Both SESDIR and CURDIR is invalid
        let invalid = SessionFlags::SESDIR.bits() | SessionFlags::CURDIR.bits();
        assert_eq!(rs_session_validate_save_flags(invalid), ERR_INVALID_FLAGS);

        // Either one alone is fine
        assert_eq!(
            rs_session_validate_save_flags(SessionFlags::SESDIR.bits()),
            ERR_NONE
        );
        assert_eq!(
            rs_session_validate_save_flags(SessionFlags::CURDIR.bits()),
            ERR_NONE
        );
    }

    #[test]
    fn test_can_save() {
        // Writable file with changes
        assert_eq!(rs_session_can_save(1, 1, 0), ERR_NONE);

        // Force override
        assert_eq!(rs_session_can_save(0, 0, 1), ERR_NONE);

        // Not writable without force
        assert_eq!(rs_session_can_save(1, 0, 0), ERR_FILE_WRITE);
    }

    #[test]
    fn test_integration_strings() {
        unsafe {
            let doautoall = CStr::from_ptr(rs_session_doautoall_cmd());
            assert!(doautoall.to_str().unwrap().contains("SessionLoadPost"));

            let unlet = CStr::from_ptr(rs_session_unlet_cmd());
            assert!(unlet.to_str().unwrap().contains("SessionLoad"));

            let header = CStr::from_ptr(rs_session_file_header());
            assert!(header.to_str().unwrap().contains("Neovim"));

            let footer = CStr::from_ptr(rs_session_file_footer());
            assert!(footer.to_str().unwrap().contains("vim:"));
        }
    }
}
