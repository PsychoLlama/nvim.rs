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
// Constant Exports (FFI)
// =============================================================================

// Backslash constants
#[no_mangle]
pub const extern "C" fn rs_xp_bs_none() -> c_int {
    backslash::XP_BS_NONE
}

#[no_mangle]
pub const extern "C" fn rs_xp_bs_one() -> c_int {
    backslash::XP_BS_ONE
}

#[no_mangle]
pub const extern "C" fn rs_xp_bs_three() -> c_int {
    backslash::XP_BS_THREE
}

#[no_mangle]
pub const extern "C" fn rs_xp_bs_comma() -> c_int {
    backslash::XP_BS_COMMA
}

// Wild mode constants
#[no_mangle]
pub const extern "C" fn rs_wild_free() -> c_int {
    wild_mode::WILD_FREE
}

#[no_mangle]
pub const extern "C" fn rs_wild_expand_free() -> c_int {
    wild_mode::WILD_EXPAND_FREE
}

#[no_mangle]
pub const extern "C" fn rs_wild_expand_keep() -> c_int {
    wild_mode::WILD_EXPAND_KEEP
}

#[no_mangle]
pub const extern "C" fn rs_wild_next() -> c_int {
    wild_mode::WILD_NEXT
}

#[no_mangle]
pub const extern "C" fn rs_wild_prev() -> c_int {
    wild_mode::WILD_PREV
}

#[no_mangle]
pub const extern "C" fn rs_wild_all() -> c_int {
    wild_mode::WILD_ALL
}

#[no_mangle]
pub const extern "C" fn rs_wild_longest() -> c_int {
    wild_mode::WILD_LONGEST
}

#[no_mangle]
pub const extern "C" fn rs_wild_all_keep() -> c_int {
    wild_mode::WILD_ALL_KEEP
}

#[no_mangle]
pub const extern "C" fn rs_wild_cancel() -> c_int {
    wild_mode::WILD_CANCEL
}

#[no_mangle]
pub const extern "C" fn rs_wild_apply() -> c_int {
    wild_mode::WILD_APPLY
}

#[no_mangle]
pub const extern "C" fn rs_wild_pageup() -> c_int {
    wild_mode::WILD_PAGEUP
}

#[no_mangle]
pub const extern "C" fn rs_wild_pagedown() -> c_int {
    wild_mode::WILD_PAGEDOWN
}

#[no_mangle]
pub const extern "C" fn rs_wild_pum_want() -> c_int {
    wild_mode::WILD_PUM_WANT
}

// Wild options constants
#[no_mangle]
pub const extern "C" fn rs_wild_list_notfound() -> c_int {
    wild_options::WILD_LIST_NOTFOUND
}

#[no_mangle]
pub const extern "C" fn rs_wild_home_replace() -> c_int {
    wild_options::WILD_HOME_REPLACE
}

#[no_mangle]
pub const extern "C" fn rs_wild_use_nl() -> c_int {
    wild_options::WILD_USE_NL
}

#[no_mangle]
pub const extern "C" fn rs_wild_no_beep() -> c_int {
    wild_options::WILD_NO_BEEP
}

#[no_mangle]
pub const extern "C" fn rs_wild_add_slash() -> c_int {
    wild_options::WILD_ADD_SLASH
}

#[no_mangle]
pub const extern "C" fn rs_wild_keep_all() -> c_int {
    wild_options::WILD_KEEP_ALL
}

#[no_mangle]
pub const extern "C" fn rs_wild_silent() -> c_int {
    wild_options::WILD_SILENT
}

#[no_mangle]
pub const extern "C" fn rs_wild_escape() -> c_int {
    wild_options::WILD_ESCAPE
}

#[no_mangle]
pub const extern "C" fn rs_wild_icase() -> c_int {
    wild_options::WILD_ICASE
}

#[no_mangle]
pub const extern "C" fn rs_wild_alllinks() -> c_int {
    wild_options::WILD_ALLLINKS
}

#[no_mangle]
pub const extern "C" fn rs_wild_ignore_completeslash() -> c_int {
    wild_options::WILD_IGNORE_COMPLETESLASH
}

#[no_mangle]
pub const extern "C" fn rs_wild_noerror() -> c_int {
    wild_options::WILD_NOERROR
}

#[no_mangle]
pub const extern "C" fn rs_wild_buflastused() -> c_int {
    wild_options::WILD_BUFLASTUSED
}

#[no_mangle]
pub const extern "C" fn rs_buf_diff_filter() -> c_int {
    wild_options::BUF_DIFF_FILTER
}

#[no_mangle]
pub const extern "C" fn rs_wild_noselect() -> c_int {
    wild_options::WILD_NOSELECT
}

#[no_mangle]
pub const extern "C" fn rs_wild_may_expand_pattern() -> c_int {
    wild_options::WILD_MAY_EXPAND_PATTERN
}

#[no_mangle]
pub const extern "C" fn rs_wild_func_trigger() -> c_int {
    wild_options::WILD_FUNC_TRIGGER
}

// Expand context constants
#[no_mangle]
pub const extern "C" fn rs_expand_unsuccessful() -> c_int {
    ExpandContext::Unsuccessful.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_ok() -> c_int {
    ExpandContext::Ok.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_nothing() -> c_int {
    ExpandContext::Nothing.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_commands() -> c_int {
    ExpandContext::Commands.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_files() -> c_int {
    ExpandContext::Files.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_directories() -> c_int {
    ExpandContext::Directories.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_settings() -> c_int {
    ExpandContext::Settings.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_buffers() -> c_int {
    ExpandContext::Buffers.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_help() -> c_int {
    ExpandContext::Help.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_functions() -> c_int {
    ExpandContext::Functions.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_user_commands() -> c_int {
    ExpandContext::UserCommands.to_raw()
}

#[no_mangle]
pub const extern "C" fn rs_expand_lua() -> c_int {
    ExpandContext::Lua.to_raw()
}

// Validation functions
#[no_mangle]
pub extern "C" fn rs_expand_context_valid(context: c_int) -> c_int {
    c_int::from(ExpandContext::from_raw(context).is_some())
}

#[no_mangle]
pub extern "C" fn rs_is_file_expand_context(context: c_int) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return 0;
    };
    c_int::from(matches!(
        ctx,
        ExpandContext::Files
            | ExpandContext::Directories
            | ExpandContext::FilesInPath
            | ExpandContext::DirsInCdpath
    ))
}

#[no_mangle]
pub extern "C" fn rs_is_user_expand_context(context: c_int) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return 0;
    };
    c_int::from(matches!(
        ctx,
        ExpandContext::UserDefined
            | ExpandContext::UserList
            | ExpandContext::UserLua
            | ExpandContext::UserFunc
    ))
}

#[no_mangle]
pub extern "C" fn rs_wild_mode_is_navigation(mode: c_int) -> c_int {
    c_int::from(matches!(
        mode,
        x if x == wild_mode::WILD_NEXT
            || x == wild_mode::WILD_PREV
            || x == wild_mode::WILD_PAGEUP
            || x == wild_mode::WILD_PAGEDOWN
    ))
}

#[no_mangle]
pub extern "C" fn rs_wild_mode_needs_list(mode: c_int) -> c_int {
    c_int::from(matches!(
        mode,
        x if x == wild_mode::WILD_ALL || x == wild_mode::WILD_ALL_KEEP
    ))
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
