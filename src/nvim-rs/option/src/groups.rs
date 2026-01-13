//! Option grouping and categorization.
//!
//! This module provides helpers for option groups:
//! - Window options
//! - Buffer options
//! - Terminal options
//! - Display options

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Option Group Constants
// =============================================================================

/// General options.
pub const OPT_GROUP_GENERAL: c_int = 0;
/// Window-related options.
pub const OPT_GROUP_WINDOW: c_int = 1;
/// Buffer-related options.
pub const OPT_GROUP_BUFFER: c_int = 2;
/// Display/appearance options.
pub const OPT_GROUP_DISPLAY: c_int = 3;
/// Editing behavior options.
pub const OPT_GROUP_EDITING: c_int = 4;
/// Search options.
pub const OPT_GROUP_SEARCH: c_int = 5;
/// Completion options.
pub const OPT_GROUP_COMPLETION: c_int = 6;
/// Indentation options.
pub const OPT_GROUP_INDENT: c_int = 7;
/// Folding options.
pub const OPT_GROUP_FOLD: c_int = 8;
/// Terminal options.
pub const OPT_GROUP_TERMINAL: c_int = 9;
/// Filetype options.
pub const OPT_GROUP_FILETYPE: c_int = 10;
/// Backup/swap options.
pub const OPT_GROUP_BACKUP: c_int = 11;
/// Mouse options.
pub const OPT_GROUP_MOUSE: c_int = 12;
/// Spell options.
pub const OPT_GROUP_SPELL: c_int = 13;

// =============================================================================
// Option Scope Flags
// =============================================================================

/// Option applies globally.
pub const OPT_SCOPE_GLOBAL: c_int = 0x01;
/// Option applies per-window.
pub const OPT_SCOPE_WINDOW: c_int = 0x02;
/// Option applies per-buffer.
pub const OPT_SCOPE_BUFFER: c_int = 0x04;
/// Option has local value.
pub const OPT_SCOPE_LOCAL: c_int = 0x08;

// =============================================================================
// Group Helpers
// =============================================================================

/// Get group name string.
#[allow(dead_code)]
fn group_name(group: c_int) -> &'static str {
    match group {
        OPT_GROUP_GENERAL => "general",
        OPT_GROUP_WINDOW => "window",
        OPT_GROUP_BUFFER => "buffer",
        OPT_GROUP_DISPLAY => "display",
        OPT_GROUP_EDITING => "editing",
        OPT_GROUP_SEARCH => "search",
        OPT_GROUP_COMPLETION => "completion",
        OPT_GROUP_INDENT => "indent",
        OPT_GROUP_FOLD => "fold",
        OPT_GROUP_TERMINAL => "terminal",
        OPT_GROUP_FILETYPE => "filetype",
        OPT_GROUP_BACKUP => "backup",
        OPT_GROUP_MOUSE => "mouse",
        OPT_GROUP_SPELL => "spell",
        _ => "unknown",
    }
}

/// Check if option has global scope.
fn has_global_scope(scope_flags: c_int) -> bool {
    (scope_flags & OPT_SCOPE_GLOBAL) != 0
}

/// Check if option has window scope.
fn has_window_scope(scope_flags: c_int) -> bool {
    (scope_flags & OPT_SCOPE_WINDOW) != 0
}

/// Check if option has buffer scope.
fn has_buffer_scope(scope_flags: c_int) -> bool {
    (scope_flags & OPT_SCOPE_BUFFER) != 0
}

/// Check if option has local scope.
fn has_local_scope(scope_flags: c_int) -> bool {
    (scope_flags & OPT_SCOPE_LOCAL) != 0
}

/// Check if option is global-only.
fn is_global_only(scope_flags: c_int) -> bool {
    scope_flags == OPT_SCOPE_GLOBAL
}

/// Check if option supports both global and local.
fn is_global_local(scope_flags: c_int) -> bool {
    has_global_scope(scope_flags) && has_local_scope(scope_flags)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get OPT_GROUP_GENERAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_group_general() -> c_int {
    OPT_GROUP_GENERAL
}

/// FFI: Get OPT_GROUP_WINDOW constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_group_window() -> c_int {
    OPT_GROUP_WINDOW
}

/// FFI: Get OPT_GROUP_BUFFER constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_group_buffer() -> c_int {
    OPT_GROUP_BUFFER
}

/// FFI: Get OPT_GROUP_DISPLAY constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_group_display() -> c_int {
    OPT_GROUP_DISPLAY
}

/// FFI: Get OPT_GROUP_EDITING constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_group_editing() -> c_int {
    OPT_GROUP_EDITING
}

/// FFI: Get OPT_GROUP_SEARCH constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_group_search() -> c_int {
    OPT_GROUP_SEARCH
}

/// FFI: Get OPT_GROUP_INDENT constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_group_indent() -> c_int {
    OPT_GROUP_INDENT
}

/// FFI: Get OPT_GROUP_TERMINAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_group_terminal() -> c_int {
    OPT_GROUP_TERMINAL
}

/// FFI: Get OPT_SCOPE_GLOBAL constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_scope_global_flag() -> c_int {
    OPT_SCOPE_GLOBAL
}

/// FFI: Get OPT_SCOPE_WINDOW constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_scope_window_flag() -> c_int {
    OPT_SCOPE_WINDOW
}

/// FFI: Get OPT_SCOPE_BUFFER constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_scope_buffer_flag() -> c_int {
    OPT_SCOPE_BUFFER
}

/// FFI: Check if option has global scope.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_has_global_scope(scope_flags: c_int) -> c_int {
    c_int::from(has_global_scope(scope_flags))
}

/// FFI: Check if option has window scope.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_has_window_scope(scope_flags: c_int) -> c_int {
    c_int::from(has_window_scope(scope_flags))
}

/// FFI: Check if option has buffer scope.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_has_buffer_scope(scope_flags: c_int) -> c_int {
    c_int::from(has_buffer_scope(scope_flags))
}

/// FFI: Check if option has local scope.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_has_local_scope(scope_flags: c_int) -> c_int {
    c_int::from(has_local_scope(scope_flags))
}

/// FFI: Check if option is global-only.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_global_only(scope_flags: c_int) -> c_int {
    c_int::from(is_global_only(scope_flags))
}

/// FFI: Check if option supports global and local.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_global_local(scope_flags: c_int) -> c_int {
    c_int::from(is_global_local(scope_flags))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_constants() {
        assert_eq!(OPT_GROUP_GENERAL, 0);
        assert_eq!(OPT_GROUP_WINDOW, 1);
        assert_eq!(OPT_GROUP_BUFFER, 2);
    }

    #[test]
    fn test_scope_flags() {
        assert_eq!(OPT_SCOPE_GLOBAL, 0x01);
        assert_eq!(OPT_SCOPE_WINDOW, 0x02);
        assert_eq!(OPT_SCOPE_BUFFER, 0x04);
        assert_eq!(OPT_SCOPE_LOCAL, 0x08);
    }

    #[test]
    fn test_group_name() {
        assert_eq!(group_name(OPT_GROUP_GENERAL), "general");
        assert_eq!(group_name(OPT_GROUP_WINDOW), "window");
        assert_eq!(group_name(OPT_GROUP_DISPLAY), "display");
        assert_eq!(group_name(99), "unknown");
    }

    #[test]
    fn test_scope_checks() {
        assert!(has_global_scope(OPT_SCOPE_GLOBAL));
        assert!(!has_global_scope(OPT_SCOPE_WINDOW));

        assert!(has_window_scope(OPT_SCOPE_WINDOW));
        assert!(!has_window_scope(OPT_SCOPE_BUFFER));

        assert!(has_buffer_scope(OPT_SCOPE_BUFFER));
        assert!(!has_buffer_scope(OPT_SCOPE_WINDOW));

        assert!(has_local_scope(OPT_SCOPE_LOCAL));
        assert!(!has_local_scope(OPT_SCOPE_GLOBAL));
    }

    #[test]
    fn test_is_global_only() {
        assert!(is_global_only(OPT_SCOPE_GLOBAL));
        assert!(!is_global_only(OPT_SCOPE_GLOBAL | OPT_SCOPE_LOCAL));
        assert!(!is_global_only(OPT_SCOPE_WINDOW));
    }

    #[test]
    fn test_is_global_local() {
        assert!(is_global_local(OPT_SCOPE_GLOBAL | OPT_SCOPE_LOCAL));
        assert!(!is_global_local(OPT_SCOPE_GLOBAL));
        assert!(!is_global_local(OPT_SCOPE_LOCAL));
    }
}
