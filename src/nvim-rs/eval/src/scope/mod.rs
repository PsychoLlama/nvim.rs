//! Variable scope operations.
//!
//! This module provides helpers for variable scopes:
//! g:, l:, s:, a:, v:, b:, w:, t: scope management

#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Scope Type Constants
// =============================================================================
// NOTE: SCOPE_* constants are already defined in lib.rs via VarScope enum.
// These are local constants that match the same values for internal use.

/// Global scope (g:).
pub const SCOPE_GLOBAL: c_int = 0;
/// Local scope (l:).
pub const SCOPE_LOCAL: c_int = 1;
/// Script scope (s:).
pub const SCOPE_SCRIPT: c_int = 2;
/// Argument scope (a:).
pub const SCOPE_ARG: c_int = 3;
/// Vim variables scope (v:).
pub const SCOPE_VIM: c_int = 4;
/// Buffer scope (b:).
pub const SCOPE_BUFFER: c_int = 5;
/// Window scope (w:).
pub const SCOPE_WINDOW: c_int = 6;
/// Tab scope (t:).
pub const SCOPE_TAB: c_int = 7;
/// No explicit scope (unscoped variable).
pub const SCOPE_NONE: c_int = -1;

// =============================================================================
// Scope Flags
// =============================================================================

/// Scope is read-only.
pub const SCOPE_READONLY: c_int = 0x01;
/// Scope is being iterated.
pub const SCOPE_ITERATING: c_int = 0x02;
/// Scope allows new variables.
pub const SCOPE_EXTENDABLE: c_int = 0x04;

// =============================================================================
// Scope Helpers
// =============================================================================

/// Get scope from prefix character.
fn scope_from_char(c: u8) -> c_int {
    match c {
        b'g' => SCOPE_GLOBAL,
        b'l' => SCOPE_LOCAL,
        b's' => SCOPE_SCRIPT,
        b'a' => SCOPE_ARG,
        b'v' => SCOPE_VIM,
        b'b' => SCOPE_BUFFER,
        b'w' => SCOPE_WINDOW,
        b't' => SCOPE_TAB,
        _ => SCOPE_NONE,
    }
}

/// Get prefix character for scope.
fn char_from_scope(scope: c_int) -> u8 {
    match scope {
        SCOPE_GLOBAL => b'g',
        SCOPE_LOCAL => b'l',
        SCOPE_SCRIPT => b's',
        SCOPE_ARG => b'a',
        SCOPE_VIM => b'v',
        SCOPE_BUFFER => b'b',
        SCOPE_WINDOW => b'w',
        SCOPE_TAB => b't',
        _ => 0,
    }
}

/// Check if scope prefix is valid.
fn is_valid_scope_prefix(c: u8) -> bool {
    matches!(c, b'g' | b'l' | b's' | b'a' | b'v' | b'b' | b'w' | b't')
}

/// Check if scope is read-only.
fn is_scope_readonly(scope: c_int) -> bool {
    // a: and v: scopes are read-only for user modifications
    matches!(scope, SCOPE_ARG | SCOPE_VIM)
}

/// Check if scope is extendable (allows new variables).
fn is_scope_extendable(scope: c_int) -> bool {
    matches!(
        scope,
        SCOPE_GLOBAL | SCOPE_LOCAL | SCOPE_SCRIPT | SCOPE_BUFFER | SCOPE_WINDOW | SCOPE_TAB
    )
}

/// Check if scope is window-specific.
fn is_window_scope(scope: c_int) -> bool {
    matches!(scope, SCOPE_WINDOW | SCOPE_BUFFER)
}

/// Check if scope is persistent across function calls.
fn is_persistent_scope(scope: c_int) -> bool {
    matches!(
        scope,
        SCOPE_GLOBAL | SCOPE_SCRIPT | SCOPE_BUFFER | SCOPE_WINDOW | SCOPE_TAB
    )
}

// =============================================================================
// FFI Exports
// NOTE: Basic scope constants are exported from lib.rs via VarScope enum.
// Here we export additional helper functions.
// =============================================================================

/// FFI: Get SCOPE_NONE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_none_const() -> c_int {
    SCOPE_NONE
}

/// FFI: Get scope from prefix character.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_from_prefix_char(c: c_int) -> c_int {
    scope_from_char(c as u8)
}

/// FFI: Get prefix character for scope.
#[unsafe(no_mangle)]
pub extern "C" fn rs_prefix_char_from_scope(scope: c_int) -> c_int {
    c_int::from(char_from_scope(scope))
}

/// FFI: Check if valid scope prefix.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_valid_scope_prefix(c: c_int) -> c_int {
    c_int::from(is_valid_scope_prefix(c as u8))
}

/// FFI: Check if scope is read-only.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_scope_readonly(scope: c_int) -> c_int {
    c_int::from(is_scope_readonly(scope))
}

/// FFI: Check if scope is extendable.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_scope_extendable(scope: c_int) -> c_int {
    c_int::from(is_scope_extendable(scope))
}

/// FFI: Check if scope is window-specific.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_window_scope(scope: c_int) -> c_int {
    c_int::from(is_window_scope(scope))
}

/// FFI: Check if scope is persistent.
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_persistent_scope(scope: c_int) -> c_int {
    c_int::from(is_persistent_scope(scope))
}

// =============================================================================
// Additional FFI Exports (E7)
// NOTE: Scope constants (rs_scope_global, etc.) are exported from lib.rs
// =============================================================================

/// FFI: Get SCOPE_READONLY flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_readonly_flag() -> c_int {
    SCOPE_READONLY
}

/// FFI: Get SCOPE_ITERATING flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_iterating_flag() -> c_int {
    SCOPE_ITERATING
}

/// FFI: Get SCOPE_EXTENDABLE flag.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_extendable_flag() -> c_int {
    SCOPE_EXTENDABLE
}

/// FFI: Check if two scopes are the same.
#[unsafe(no_mangle)]
pub extern "C" fn rs_scopes_equal(scope1: c_int, scope2: c_int) -> c_int {
    c_int::from(scope1 == scope2)
}

/// FFI: Check if scope can be written to (not read-only).
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_is_writable(scope: c_int) -> c_int {
    c_int::from(!is_scope_readonly(scope))
}

/// FFI: Check if scope is a function scope (l: or a:).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_function_scope(scope: c_int) -> c_int {
    c_int::from(matches!(scope, SCOPE_LOCAL | SCOPE_ARG))
}

/// FFI: Check if scope is buffer-specific (b:).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_buffer_scope(scope: c_int) -> c_int {
    c_int::from(scope == SCOPE_BUFFER)
}

/// FFI: Check if scope is tab-specific (t:).
#[unsafe(no_mangle)]
pub extern "C" fn rs_is_tab_scope(scope: c_int) -> c_int {
    c_int::from(scope == SCOPE_TAB)
}

/// FFI: Get scope name as string length (including null terminator).
#[unsafe(no_mangle)]
pub extern "C" fn rs_scope_name_len(scope: c_int) -> c_int {
    match scope {
        SCOPE_GLOBAL | SCOPE_SCRIPT | SCOPE_BUFFER | SCOPE_WINDOW => 7, // "global", "script", "buffer", "window"
        SCOPE_LOCAL => 6,                                               // "local"
        SCOPE_ARG | SCOPE_VIM | SCOPE_TAB => 4,                         // "arg", "vim", "tab"
        _ => 8,                                                         // "unknown"
    }
}

/// FFI: Check if variable name starts with scope prefix (e.g., "g:").
///
/// # Safety
/// `name` must be valid for at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_has_scope_prefix(name: *const u8, len: c_int) -> c_int {
    if name.is_null() || len < 2 {
        return 0;
    }

    let first = *name;
    let second = *name.add(1);

    if second != b':' {
        return 0;
    }

    c_int::from(is_valid_scope_prefix(first))
}

/// FFI: Get scope from variable name prefix.
///
/// # Safety
/// `name` must be valid for at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_scope_from_name(name: *const u8, len: c_int) -> c_int {
    if name.is_null() || len < 2 {
        return SCOPE_NONE;
    }

    let first = *name;
    let second = *name.add(1);

    if second != b':' {
        return SCOPE_NONE;
    }

    scope_from_char(first)
}

/// FFI: Get the starting offset after scope prefix (0 if no prefix).
///
/// # Safety
/// `name` must be valid for at least `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_scope_prefix_len(name: *const u8, len: c_int) -> c_int {
    if name.is_null() || len < 2 {
        return 0;
    }

    let first = *name;
    let second = *name.add(1);

    if second == b':' && is_valid_scope_prefix(first) {
        2
    } else {
        0
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_constants() {
        assert_eq!(SCOPE_GLOBAL, 0);
        assert_eq!(SCOPE_LOCAL, 1);
        assert_eq!(SCOPE_SCRIPT, 2);
        assert_eq!(SCOPE_VIM, 4);
        assert_eq!(SCOPE_NONE, -1);
    }

    #[test]
    fn test_scope_from_char() {
        assert_eq!(scope_from_char(b'g'), SCOPE_GLOBAL);
        assert_eq!(scope_from_char(b'l'), SCOPE_LOCAL);
        assert_eq!(scope_from_char(b's'), SCOPE_SCRIPT);
        assert_eq!(scope_from_char(b'a'), SCOPE_ARG);
        assert_eq!(scope_from_char(b'v'), SCOPE_VIM);
        assert_eq!(scope_from_char(b'b'), SCOPE_BUFFER);
        assert_eq!(scope_from_char(b'w'), SCOPE_WINDOW);
        assert_eq!(scope_from_char(b't'), SCOPE_TAB);
        assert_eq!(scope_from_char(b'x'), SCOPE_NONE);
    }

    #[test]
    fn test_char_from_scope() {
        assert_eq!(char_from_scope(SCOPE_GLOBAL), b'g');
        assert_eq!(char_from_scope(SCOPE_LOCAL), b'l');
        assert_eq!(char_from_scope(SCOPE_SCRIPT), b's');
        assert_eq!(char_from_scope(SCOPE_NONE), 0);
    }

    #[test]
    fn test_is_valid_scope_prefix() {
        assert!(is_valid_scope_prefix(b'g'));
        assert!(is_valid_scope_prefix(b'l'));
        assert!(is_valid_scope_prefix(b'v'));
        assert!(!is_valid_scope_prefix(b'x'));
        assert!(!is_valid_scope_prefix(b'G'));
    }

    #[test]
    fn test_is_scope_readonly() {
        assert!(is_scope_readonly(SCOPE_ARG));
        assert!(is_scope_readonly(SCOPE_VIM));
        assert!(!is_scope_readonly(SCOPE_GLOBAL));
        assert!(!is_scope_readonly(SCOPE_LOCAL));
    }

    #[test]
    fn test_is_scope_extendable() {
        assert!(is_scope_extendable(SCOPE_GLOBAL));
        assert!(is_scope_extendable(SCOPE_LOCAL));
        assert!(is_scope_extendable(SCOPE_BUFFER));
        assert!(!is_scope_extendable(SCOPE_ARG));
        assert!(!is_scope_extendable(SCOPE_VIM));
    }

    #[test]
    fn test_is_window_scope() {
        assert!(is_window_scope(SCOPE_WINDOW));
        assert!(is_window_scope(SCOPE_BUFFER));
        assert!(!is_window_scope(SCOPE_GLOBAL));
        assert!(!is_window_scope(SCOPE_LOCAL));
    }

    #[test]
    fn test_is_persistent_scope() {
        assert!(is_persistent_scope(SCOPE_GLOBAL));
        assert!(is_persistent_scope(SCOPE_SCRIPT));
        assert!(!is_persistent_scope(SCOPE_LOCAL));
        assert!(!is_persistent_scope(SCOPE_ARG));
    }
}
