//! User-defined completion support for command-line expansion
//!
//! This module provides helper functions and type definitions for
//! VimL/Lua user-defined completion functions from cmdexpand.c.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

use crate::expand::ExpandContext;

// =============================================================================
// Completion Type String Mapping
// =============================================================================

/// Mapping from completion type strings to ExpandContext values.
///
/// These are the valid type strings for getcompletion() and complete().
pub const COMPLETION_TYPE_NAMES: &[(&str, ExpandContext)] = &[
    ("arglist", ExpandContext::Arglist),
    ("augroup", ExpandContext::Augroup),
    ("behave", ExpandContext::Nothing), // special: returns fixed list
    ("buffer", ExpandContext::Buffers),
    ("breakpoint", ExpandContext::Breakpoint),
    ("checkhealth", ExpandContext::Checkhealth),
    ("cmdline", ExpandContext::Nothing), // special: handled separately
    ("color", ExpandContext::Colors),
    ("command", ExpandContext::Commands),
    ("compiler", ExpandContext::Compiler),
    ("custom", ExpandContext::UserDefined),
    ("customlist", ExpandContext::UserList),
    ("diff_buffer", ExpandContext::DiffBuffers),
    ("dir", ExpandContext::Directories),
    ("environment", ExpandContext::EnvVars),
    ("event", ExpandContext::Events),
    ("expression", ExpandContext::Expression),
    ("file", ExpandContext::Files),
    ("file_in_path", ExpandContext::FilesInPath),
    ("filetype", ExpandContext::Filetype),
    ("function", ExpandContext::Functions),
    ("help", ExpandContext::Help),
    ("highlight", ExpandContext::Highlight),
    ("history", ExpandContext::History),
    ("keymap", ExpandContext::Keymap),
    ("locale", ExpandContext::Locales),
    ("lua", ExpandContext::Lua),
    ("mapclear", ExpandContext::Mapclear),
    ("mapping", ExpandContext::Mappings),
    ("menu", ExpandContext::Menus),
    ("messages", ExpandContext::Messages),
    ("option", ExpandContext::Settings),
    ("packadd", ExpandContext::Packadd),
    ("runtime", ExpandContext::Runtime),
    ("scriptnames", ExpandContext::Scriptnames),
    ("shellcmd", ExpandContext::Shellcmd),
    ("shellcmdline", ExpandContext::Shellcmdline),
    ("sign", ExpandContext::Sign),
    ("syntax", ExpandContext::Syntax),
    ("syntime", ExpandContext::Syntime),
    ("tag", ExpandContext::Tags),
    ("tag_listfiles", ExpandContext::TagsListfiles),
    ("user", ExpandContext::User),
    ("var", ExpandContext::UserVars),
];

/// Convert a completion type string to its ExpandContext value.
///
/// Returns `Some(context)` for known types, `None` for unknown.
#[must_use]
pub fn type_str_to_context(type_str: &str) -> Option<ExpandContext> {
    for &(name, ctx) in COMPLETION_TYPE_NAMES {
        if name == type_str {
            return Some(ctx);
        }
    }
    // Check for "custom," prefix
    if type_str.starts_with("custom,") && type_str.len() > 7 {
        return Some(ExpandContext::UserDefined);
    }
    // Check for "customlist," prefix
    if type_str.starts_with("customlist,") && type_str.len() > 11 {
        return Some(ExpandContext::UserList);
    }
    None
}

/// Convert an ExpandContext value to its type string.
///
/// Returns the canonical type name for known contexts.
#[must_use]
pub fn context_to_type_str(context: ExpandContext) -> Option<&'static str> {
    for &(name, ctx) in COMPLETION_TYPE_NAMES {
        if ctx == context {
            return Some(name);
        }
    }
    None
}

/// FFI wrapper for type string to context conversion.
///
/// # Safety
///
/// `type_str` must be a valid NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdcomplete_str_to_type(type_str: *const c_char) -> c_int {
    if type_str.is_null() {
        return ExpandContext::Nothing.to_raw();
    }

    let c_str = std::ffi::CStr::from_ptr(type_str);
    let Ok(s) = c_str.to_str() else {
        return ExpandContext::Nothing.to_raw();
    };

    type_str_to_context(s).map_or(ExpandContext::Nothing.to_raw(), ExpandContext::to_raw)
}

// =============================================================================
// User Completion Validation
// =============================================================================

/// Check if a context is for user-defined completion.
#[must_use]
pub const fn is_user_completion(context: ExpandContext) -> bool {
    matches!(
        context,
        ExpandContext::UserDefined | ExpandContext::UserList | ExpandContext::UserLua
    )
}

/// FFI wrapper for user completion check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_user_completion(context: c_int) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return 0;
    };

    c_int::from(is_user_completion(ctx))
}

/// Check if a user function name is valid for completion.
///
/// Function names must be non-empty and not start with certain invalid characters.
#[must_use]
pub fn is_valid_completion_func(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Function name should not start with special characters
    let first = name.as_bytes()[0];
    !matches!(first, b'0'..=b'9' | b':')
}

/// FFI wrapper for function name validation.
///
/// # Safety
///
/// `name` must be a valid NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_is_valid_completion_func(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let c_str = std::ffi::CStr::from_ptr(name);
    let Ok(s) = c_str.to_str() else {
        return 0;
    };

    c_int::from(is_valid_completion_func(s))
}

// =============================================================================
// Completion Result Validation
// =============================================================================

/// Check if a string result from user completion is valid for inclusion.
///
/// Valid results are non-empty strings.
#[must_use]
pub const fn is_valid_completion_result(s: &[u8]) -> bool {
    !s.is_empty()
}

/// Count valid string entries in a newline-separated result string.
///
/// Used for ExpandUserDefined which returns newline-separated results.
#[must_use]
pub fn count_newline_entries(s: &[u8]) -> usize {
    if s.is_empty() {
        return 0;
    }

    s.split(|&c| c == b'\n')
        .filter(|entry| !entry.is_empty())
        .count()
}

// =============================================================================
// getcompletion() Helpers
// =============================================================================

/// Default options for getcompletion() function.
///
/// These are the WILD_* flags used by default.
pub mod getcompletion_flags {
    use crate::expand::wild_flags::{
        WILD_ADD_SLASH, WILD_HOME_REPLACE, WILD_NO_BEEP, WILD_SILENT, WILD_USE_NL,
    };

    /// Default flags for getcompletion()
    pub const DEFAULT: i32 =
        WILD_SILENT | WILD_USE_NL | WILD_ADD_SLASH | WILD_NO_BEEP | WILD_HOME_REPLACE;
}

/// Check if a completion type string requires special handling.
///
/// Some types like "cmdline" need custom processing.
#[must_use]
pub fn type_needs_special_handling(type_str: &str) -> bool {
    matches!(type_str, "cmdline" | "behave")
}

/// FFI wrapper for special handling check.
///
/// # Safety
///
/// `type_str` must be a valid NUL-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_completion_type_needs_special_handling(
    type_str: *const c_char,
) -> c_int {
    if type_str.is_null() {
        return 0;
    }

    let c_str = std::ffi::CStr::from_ptr(type_str);
    let Ok(s) = c_str.to_str() else {
        return 0;
    };

    c_int::from(type_needs_special_handling(s))
}

// =============================================================================
// cmdcomplete_info() Helpers
// =============================================================================

/// Check if command completion info is available.
///
/// Returns true if there's an active command line with completion data.
#[must_use]
pub const fn completion_info_available(has_cmdline: bool, has_xpc: bool, has_files: bool) -> bool {
    has_cmdline && has_xpc && has_files
}

/// FFI wrapper for completion info availability check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_completion_info_available(
    has_cmdline: c_int,
    has_xpc: c_int,
    has_files: c_int,
) -> c_int {
    c_int::from(completion_info_available(
        has_cmdline != 0,
        has_xpc != 0,
        has_files != 0,
    ))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_str_to_context() {
        // Known types
        assert_eq!(type_str_to_context("file"), Some(ExpandContext::Files));
        assert_eq!(type_str_to_context("dir"), Some(ExpandContext::Directories));
        assert_eq!(
            type_str_to_context("command"),
            Some(ExpandContext::Commands)
        );
        assert_eq!(type_str_to_context("buffer"), Some(ExpandContext::Buffers));
        assert_eq!(type_str_to_context("help"), Some(ExpandContext::Help));
        assert_eq!(type_str_to_context("option"), Some(ExpandContext::Settings));

        // Custom types
        assert_eq!(
            type_str_to_context("custom,MyFunc"),
            Some(ExpandContext::UserDefined)
        );
        assert_eq!(
            type_str_to_context("customlist,MyFunc"),
            Some(ExpandContext::UserList)
        );

        // Invalid custom (no function name)
        assert_eq!(type_str_to_context("custom,"), None);
        assert_eq!(type_str_to_context("customlist,"), None);

        // Unknown
        assert_eq!(type_str_to_context("unknown"), None);
        assert_eq!(type_str_to_context(""), None);
    }

    #[test]
    fn test_context_to_type_str() {
        assert_eq!(context_to_type_str(ExpandContext::Files), Some("file"));
        assert_eq!(context_to_type_str(ExpandContext::Directories), Some("dir"));
        assert_eq!(
            context_to_type_str(ExpandContext::Commands),
            Some("command")
        );
        assert_eq!(context_to_type_str(ExpandContext::Help), Some("help"));

        // Contexts without direct string mapping
        assert_eq!(context_to_type_str(ExpandContext::Ok), None);
        // Note: Nothing maps to "behave" (a special type that returns a fixed list)
        assert_eq!(context_to_type_str(ExpandContext::Nothing), Some("behave"));
    }

    #[test]
    fn test_is_user_completion() {
        assert!(is_user_completion(ExpandContext::UserDefined));
        assert!(is_user_completion(ExpandContext::UserList));
        assert!(is_user_completion(ExpandContext::UserLua));

        assert!(!is_user_completion(ExpandContext::Files));
        assert!(!is_user_completion(ExpandContext::Commands));
        assert!(!is_user_completion(ExpandContext::UserFunc));
    }

    #[test]
    fn test_is_valid_completion_func() {
        // Valid names
        assert!(is_valid_completion_func("MyFunc"));
        assert!(is_valid_completion_func("s:MyFunc"));
        assert!(is_valid_completion_func("my_func"));
        assert!(is_valid_completion_func("_private"));

        // Invalid names
        assert!(!is_valid_completion_func(""));
        assert!(!is_valid_completion_func("123abc"));
        assert!(!is_valid_completion_func(":command"));
    }

    #[test]
    fn test_count_newline_entries() {
        assert_eq!(count_newline_entries(b""), 0);
        assert_eq!(count_newline_entries(b"one"), 1);
        assert_eq!(count_newline_entries(b"one\ntwo"), 2);
        assert_eq!(count_newline_entries(b"one\ntwo\nthree"), 3);
        assert_eq!(count_newline_entries(b"one\n\nthree"), 2); // empty entry skipped
        assert_eq!(count_newline_entries(b"\n\n"), 0); // all empty
        assert_eq!(count_newline_entries(b"one\n"), 1); // trailing newline
    }

    #[test]
    fn test_type_needs_special_handling() {
        assert!(type_needs_special_handling("cmdline"));
        assert!(type_needs_special_handling("behave"));

        assert!(!type_needs_special_handling("file"));
        assert!(!type_needs_special_handling("command"));
        assert!(!type_needs_special_handling(""));
    }

    #[test]
    fn test_completion_info_available() {
        assert!(completion_info_available(true, true, true));
        assert!(!completion_info_available(false, true, true));
        assert!(!completion_info_available(true, false, true));
        assert!(!completion_info_available(true, true, false));
        assert!(!completion_info_available(false, false, false));
    }

    #[test]
    fn test_is_valid_completion_result() {
        assert!(is_valid_completion_result(b"valid"));
        assert!(is_valid_completion_result(b"a"));
        assert!(!is_valid_completion_result(b""));
    }
}
