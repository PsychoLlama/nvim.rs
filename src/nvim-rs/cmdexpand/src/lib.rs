//! Command-line completion and expansion for Neovim
//!
//! This crate provides the command-line completion engine, including:
//! - Wildcard expansion
//! - Completion source management
//! - Fuzzy matching integration
//! - Popup menu support for completions

#![allow(unsafe_code)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

pub mod callbacks;
pub mod context;
pub mod context_helpers;
pub mod helpers;
pub mod navigation;
pub mod pattern;

pub use context::*;

use libc::{c_char, c_int};
use std::ffi::CStr;

/// Opaque handle to `expand_T` (C struct).
pub type ExpandHandle = *mut libc::c_void;

// =============================================================================
// External C functions
// =============================================================================

extern "C" {
    fn nvim_get_wop_flags() -> libc::c_uint;
    fn nvim_get_compl_match_array_not_null() -> c_int;
}

// =============================================================================
// Fuzzy completion support
// =============================================================================

/// Returns true if fuzzy completion is supported for the given context.
///
/// Not all completion contexts support fuzzy matching. This function
/// checks the context type and returns whether fuzzy completion can be used.
#[must_use]
pub const fn cmdline_fuzzy_completion_supported(context: i32) -> bool {
    // These contexts do NOT support fuzzy completion
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return false;
    };

    !matches!(
        ctx,
        ExpandContext::BoolSettings
            | ExpandContext::Colors
            | ExpandContext::Compiler
            | ExpandContext::Directories
            | ExpandContext::DirsInCdpath
            | ExpandContext::Files
            | ExpandContext::FilesInPath
            | ExpandContext::Filetype
            | ExpandContext::Filetypecmd
            | ExpandContext::Findfunc
            | ExpandContext::Help
            | ExpandContext::Keymap
            | ExpandContext::Lua
            | ExpandContext::OldSetting
            | ExpandContext::StringSetting
            | ExpandContext::SettingSubtract
            | ExpandContext::Ownsyntax
            | ExpandContext::Packadd
            | ExpandContext::Runtime
            | ExpandContext::Shellcmd
            | ExpandContext::Shellcmdline
            | ExpandContext::Tags
            | ExpandContext::TagsListfiles
            | ExpandContext::UserList
            | ExpandContext::UserLua
    )
}

/// Check if fuzzy completion is enabled and the pattern is not empty.
///
/// Returns true if:
/// 1. The 'wildoptions' setting has the fuzzy flag set
/// 2. The fuzzy string is not empty
#[must_use]
pub fn cmdline_fuzzy_complete(fuzzystr: &str) -> bool {
    if fuzzystr.is_empty() {
        return false;
    }

    // Check if fuzzy flag is set in wildoptions
    // SAFETY: nvim_get_wop_flags is a simple accessor that reads a global variable
    let wop_flags = unsafe { nvim_get_wop_flags() };
    (wop_flags & K_OPT_WOP_FLAG_FUZZY) != 0
}

/// Check if the cmdline popup menu is active.
#[must_use]
pub fn cmdline_pum_active() -> bool {
    // SAFETY: nvim_get_compl_match_array_not_null is a simple accessor
    unsafe { nvim_get_compl_match_array_not_null() != 0 }
}

// =============================================================================
// FFI Interface
// =============================================================================

/// Convert C string pointer to Rust &str
///
/// # Safety
///
/// `ptr` must be a valid null-terminated C string or null.
unsafe fn cstr_to_str<'a>(ptr: *const c_char) -> Option<&'a str> {
    if ptr.is_null() {
        return None;
    }
    CStr::from_ptr(ptr).to_str().ok()
}

/// Check if fuzzy completion is enabled for the given string (FFI version).
///
/// # Safety
///
/// `fuzzystr` must be a valid null-terminated C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_fuzzy_complete(fuzzystr: *const c_char) -> c_int {
    let Some(s) = cstr_to_str(fuzzystr) else {
        return 0;
    };

    c_int::from(cmdline_fuzzy_complete(s))
}

/// Check if cmdline popup menu is active (FFI version).
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_pum_active() -> c_int {
    c_int::from(cmdline_pum_active())
}

/// Check if fuzzy completion is supported for the given context (FFI version).
#[unsafe(no_mangle)]
pub extern "C" fn rs_cmdline_fuzzy_completion_supported(context: c_int) -> c_int {
    c_int::from(cmdline_fuzzy_completion_supported(context))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_completion_supported() {
        // Files/directories do NOT support fuzzy completion
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Files.to_raw()
        ));
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Directories.to_raw()
        ));
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Help.to_raw()
        ));
        assert!(!cmdline_fuzzy_completion_supported(
            ExpandContext::Tags.to_raw()
        ));

        // Commands and other contexts DO support fuzzy completion
        assert!(cmdline_fuzzy_completion_supported(
            ExpandContext::Commands.to_raw()
        ));
        assert!(cmdline_fuzzy_completion_supported(
            ExpandContext::Buffers.to_raw()
        ));
        assert!(cmdline_fuzzy_completion_supported(
            ExpandContext::Functions.to_raw()
        ));

        // Invalid context
        assert!(!cmdline_fuzzy_completion_supported(999));
    }
}
