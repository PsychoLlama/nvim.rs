//! Context determination for command-line expansion
//!
//! This module handles determining the appropriate completion context
//! based on the command line content and cursor position.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_int, c_uint};

use crate::expand::ExpandContext;

// =============================================================================
// UI Capability Flags (from ui_defs.h)
// =============================================================================

/// UI capability flags
pub mod ui_flags {
    /// Command line UI capability
    pub const UI_CMDLINE: u32 = 1 << 0;
    /// Popup menu UI capability
    pub const UI_POPUPMENU: u32 = 1 << 1;
    /// Wildmenu UI capability
    pub const UI_WILDMENU: u32 = 1 << 2;
}

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // UI capability checks
    fn ui_has(what: c_int) -> c_int;

    // Global state accessors (from existing C code)
    fn nvim_get_wop_flags() -> c_uint;
    static cmdwin_type: c_int;

    // Cmdline window check - NULL means not in cmdwin
    fn nvim_get_cmdline_win_is_null() -> c_int;
}

// =============================================================================
// kUIxxx Constants (matching C enum values)
// =============================================================================

/// kUICmdline value from C
const K_UI_CMDLINE: c_int = 5;
/// kUIPopupmenu value from C
const K_UI_POPUPMENU: c_int = 3;
/// kUIWildmenu value from C
const K_UI_WILDMENU: c_int = 6;

/// kOptWopFlagPum from C option_vars.h
const K_OPT_WOP_FLAG_PUM: c_uint = 0x02;

// =============================================================================
// Context Determination Functions
// =============================================================================

/// Check if popup menu should be used for cmdline completion wildmenu.
///
/// The popup menu is used when:
/// 1. Wildmenu is needed AND 'wildoptions' contains "pum" AND
///    not in external cmdline without cmdline window, OR
/// 2. UI has wildmenu capability, OR
/// 3. UI has both cmdline and popupmenu capabilities
///
/// # Safety
///
/// Calls C functions to check UI capabilities and global state.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmdline_compl_use_pum(need_wildmenu: c_int) -> c_int {
    let need_wm = need_wildmenu != 0;

    let has_cmdline = ui_has(K_UI_CMDLINE) != 0;
    let has_popupmenu = ui_has(K_UI_POPUPMENU) != 0;
    let has_wildmenu = ui_has(K_UI_WILDMENU) != 0;
    let cmdline_win_is_null = nvim_get_cmdline_win_is_null() != 0;
    let wop_flags = nvim_get_wop_flags();
    let has_pum_flag = (wop_flags & K_OPT_WOP_FLAG_PUM) != 0;

    // Condition 1: wildmenu needed with pum flag, not in external cmdline
    let condition1 = need_wm && has_pum_flag && !(has_cmdline && cmdline_win_is_null);

    // Condition 2: UI has wildmenu
    let condition2 = has_wildmenu;

    // Condition 3: UI has both cmdline and popupmenu
    let condition3 = has_cmdline && has_popupmenu;

    c_int::from(condition1 || condition2 || condition3)
}

/// Get the direction constant for forward search.
///
/// Returns the FORWARD value used by Neovim.
#[unsafe(no_mangle)]
pub const extern "C" fn rs_direction_forward() -> c_int {
    1 // FORWARD
}

/// Get the direction constant for backward search.
///
/// Returns the BACKWARD value used by Neovim.
#[unsafe(no_mangle)]
pub const extern "C" fn rs_direction_backward() -> c_int {
    -1 // BACKWARD
}

// =============================================================================
// Context Validation Functions
// =============================================================================

/// Check if a context is valid for completion.
///
/// Returns true if the context is a valid expansion context that
/// can produce matches.
#[must_use]
pub const fn is_valid_completion_context(context: ExpandContext) -> bool {
    !matches!(
        context,
        ExpandContext::Unsuccessful | ExpandContext::Nothing
    )
}

/// Check if the context requires command parsing.
///
/// Some contexts like EXPAND_PATTERN_IN_BUF don't need command parsing.
#[must_use]
pub const fn context_needs_command_parsing(context: ExpandContext) -> bool {
    !matches!(context, ExpandContext::PatternInBuf)
}

/// Check if the given first character is a valid command line type.
///
/// Valid types are:
/// - ':' - Ex command
/// - '>' - Debug command
/// - '=' - Expression evaluation
/// - '/' - Forward search
/// - '?' - Backward search
#[must_use]
#[inline]
pub const fn is_valid_cmdline_firstc(firstc: u8) -> bool {
    matches!(firstc, b':' | b'>' | b'=' | b'/' | b'?')
}

/// Check if the first character indicates a search command.
#[must_use]
#[inline]
pub const fn is_search_cmdline(firstc: u8) -> bool {
    matches!(firstc, b'/' | b'?')
}

/// Check if the first character indicates an Ex command.
#[must_use]
#[inline]
pub const fn is_ex_cmdline(firstc: u8) -> bool {
    firstc == b':'
}

/// Check if the first character indicates expression evaluation.
#[must_use]
#[inline]
pub const fn is_expr_cmdline(firstc: u8) -> bool {
    firstc == b'='
}

// =============================================================================
// FFI Wrappers for Context Checks
// =============================================================================

/// Check if a context is valid for completion (FFI).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_valid_completion_context(context: c_int) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return 0;
    };
    c_int::from(is_valid_completion_context(ctx))
}

/// Check if the context requires command parsing (FFI).
#[unsafe(no_mangle)]
pub extern "C" fn rs_context_needs_command_parsing(context: c_int) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return 1; // Default to needing parsing
    };
    c_int::from(context_needs_command_parsing(ctx))
}

/// Check if a cmdline first character is valid (FFI).
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub extern "C" fn rs_is_valid_cmdline_firstc(firstc: c_int) -> c_int {
    if !(0..=255).contains(&firstc) {
        return 0;
    }
    c_int::from(is_valid_cmdline_firstc(firstc as u8))
}

/// Check if cmdline first character indicates search (FFI).
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub extern "C" fn rs_is_search_cmdline(firstc: c_int) -> c_int {
    if !(0..=255).contains(&firstc) {
        return 0;
    }
    c_int::from(is_search_cmdline(firstc as u8))
}

/// Check if cmdline first character indicates Ex command (FFI).
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub extern "C" fn rs_is_ex_cmdline(firstc: c_int) -> c_int {
    if !(0..=255).contains(&firstc) {
        return 0;
    }
    c_int::from(is_ex_cmdline(firstc as u8))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_completion_context() {
        assert!(!is_valid_completion_context(ExpandContext::Unsuccessful));
        assert!(!is_valid_completion_context(ExpandContext::Nothing));
        assert!(is_valid_completion_context(ExpandContext::Ok));
        assert!(is_valid_completion_context(ExpandContext::Commands));
        assert!(is_valid_completion_context(ExpandContext::Files));
    }

    #[test]
    fn test_context_needs_command_parsing() {
        assert!(!context_needs_command_parsing(ExpandContext::PatternInBuf));
        assert!(context_needs_command_parsing(ExpandContext::Commands));
        assert!(context_needs_command_parsing(ExpandContext::Files));
    }

    #[test]
    fn test_is_valid_cmdline_firstc() {
        assert!(is_valid_cmdline_firstc(b':'));
        assert!(is_valid_cmdline_firstc(b'>'));
        assert!(is_valid_cmdline_firstc(b'='));
        assert!(is_valid_cmdline_firstc(b'/'));
        assert!(is_valid_cmdline_firstc(b'?'));
        assert!(!is_valid_cmdline_firstc(b'a'));
        assert!(!is_valid_cmdline_firstc(b'\0'));
    }

    #[test]
    fn test_is_search_cmdline() {
        assert!(is_search_cmdline(b'/'));
        assert!(is_search_cmdline(b'?'));
        assert!(!is_search_cmdline(b':'));
        assert!(!is_search_cmdline(b'='));
    }

    #[test]
    fn test_is_ex_cmdline() {
        assert!(is_ex_cmdline(b':'));
        assert!(!is_ex_cmdline(b'/'));
        assert!(!is_ex_cmdline(b'?'));
    }

    #[test]
    fn test_is_expr_cmdline() {
        assert!(is_expr_cmdline(b'='));
        assert!(!is_expr_cmdline(b':'));
        assert!(!is_expr_cmdline(b'/'));
    }

    #[test]
    fn test_direction_constants() {
        assert_eq!(rs_direction_forward(), 1);
        assert_eq!(rs_direction_backward(), -1);
    }
}
