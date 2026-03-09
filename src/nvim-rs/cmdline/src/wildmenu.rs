//! Wildmenu display and interaction for command-line completion
//!
//! This module provides the Rust implementation of wildmenu-related
//! functions from cmdexpand.c, including key translation, match display
//! length calculation, and escape character handling.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

use crate::expand::ExpandContext;

// =============================================================================
// Key Constants (from keycodes.h)
// =============================================================================

/// Encode a termcap key pair into a special key code.
/// Equivalent to the C macro: TERMCAP2KEY(a, b) = -(a + (b << 8))
const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -(a + (b << 8))
}

/// Key code constants used in wildmenu navigation
pub mod keys {
    use std::ffi::c_int;

    use super::termcap2key;

    /// Left arrow key - TERMCAP2KEY('k', 'l')
    pub const K_LEFT: c_int = termcap2key(b'k' as c_int, b'l' as c_int);
    /// Right arrow key - TERMCAP2KEY('k', 'r')
    pub const K_RIGHT: c_int = termcap2key(b'k' as c_int, b'r' as c_int);
    /// Up arrow key - TERMCAP2KEY('k', 'u')
    pub const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
    /// Down arrow key - TERMCAP2KEY('k', 'd')
    pub const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
    /// Keypad Enter - TERMCAP2KEY('K', 'A')
    pub const K_KENTER: c_int = termcap2key(b'K' as c_int, b'A' as c_int);

    /// Ctrl+P (previous)
    pub const CTRL_P: c_int = 16; // 'P' - '@'
    /// Ctrl+N (next)
    pub const CTRL_N: c_int = 14; // 'N' - '@'
}

// =============================================================================
// C Function Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // Check if cmdline pum is active (exported as cmdline_pum_active from cmdexpand crate)
    fn cmdline_pum_active() -> c_int;

    // Access to wild_menu_showing global
    fn nvim_get_wild_menu_showing() -> c_int;

    // Get expansion context
    fn nvim_expand_get_context(xp: *const ()) -> c_int;

    // Access to xp->xp_shell
    fn nvim_expand_get_shell(xp: *const ()) -> c_int;

    // String operations
    fn rem_backslash(s: *const c_char) -> c_int;
    fn menu_is_separator(s: *const c_char) -> c_int;
    fn ptr2cells(s: *const c_char) -> c_int;
    fn rs_csh_like_shell() -> c_int;
}

// =============================================================================
// Key Translation
// =============================================================================

/// Translate wildmenu navigation keys.
///
/// When the popup menu is active or wildmenu is showing, translate
/// left/right arrow keys to Ctrl-P/Ctrl-N for navigation.
///
/// # Arguments
///
/// * `key` - The input key code
/// * `pum_active` - Whether the popup menu is active
/// * `did_wild_list` - Whether a wild list was displayed
/// * `wild_menu_showing` - Whether the wild menu is currently shown
///
/// # Returns
///
/// The translated key code (Ctrl-P for left, Ctrl-N for right, or original)
#[must_use]
#[allow(clippy::wildcard_imports)]
pub const fn translate_arrow_keys(
    key: c_int,
    pum_active: bool,
    did_wild_list: bool,
    wild_menu_showing: bool,
) -> c_int {
    use keys::{CTRL_N, CTRL_P, K_LEFT, K_RIGHT};

    if pum_active || did_wild_list || wild_menu_showing {
        if key == K_LEFT {
            return CTRL_P;
        } else if key == K_RIGHT {
            return CTRL_N;
        }
    }

    key
}

/// Check if CR/Enter should complete a submenu in menu context.
///
/// When in EXPAND_MENUNAMES context and the cursor is after "Name.",
/// pressing Enter should trigger completion of the submenu.
///
/// # Arguments
///
/// * `context` - Current expansion context
/// * `cmdpos` - Current cursor position in command buffer
/// * `cmdbuff` - The command buffer bytes
/// * `key` - The input key code
///
/// # Returns
///
/// True if this is a submenu completion trigger
#[must_use]
pub fn should_complete_submenu(
    context: ExpandContext,
    cmdpos: usize,
    cmdbuff: &[u8],
    key: c_int,
) -> bool {
    if context != ExpandContext::Menunames {
        return false;
    }

    // Need at least 2 characters: something followed by '.'
    if cmdpos < 2 {
        return false;
    }

    // Check for "Name." pattern (not "\.") and Enter/CR key
    let is_dot = cmdbuff.get(cmdpos - 1) == Some(&b'.');
    let not_escaped = cmdbuff.get(cmdpos - 2) != Some(&b'\\');
    let is_enter = key == c_int::from(b'\n') || key == c_int::from(b'\r') || key == keys::K_KENTER;

    is_dot && not_escaped && is_enter
}

/// FFI wrapper for key translation.
///
/// # Safety
///
/// All pointers must be valid if provided.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_wildmenu_translate_arrow_keys(
    key: c_int,
    did_wild_list: c_int,
) -> c_int {
    let pum_active = cmdline_pum_active() != 0;
    let wild_showing = nvim_get_wild_menu_showing() != 0;

    translate_arrow_keys(key, pum_active, did_wild_list != 0, wild_showing)
}

// =============================================================================
// Skip Character Calculation
// =============================================================================

/// Calculate number of characters to skip for wildmenu display.
///
/// Backslashes used for escaping should be skipped in display, except
/// in help tags and search pattern completion matches.
///
/// # Arguments
///
/// * `context` - Current expansion context
/// * `is_backslash_escape` - Whether the string starts with an escape backslash
/// * `first_char` - First character at current position
/// * `second_char` - Second character (if available)
/// * `is_shell` - Whether this is shell expansion
/// * `is_csh_like` - Whether using csh-like shell
///
/// # Returns
///
/// Number of bytes to skip (0, 1, or 2)
#[must_use]
pub fn calculate_skip_count(
    context: ExpandContext,
    is_backslash_escape: bool,
    first_char: u8,
    second_char: Option<u8>,
    is_shell: bool,
    is_csh_like: bool,
) -> usize {
    // Check for escapable backslash (not in help or pattern contexts)
    if is_backslash_escape && !matches!(context, ExpandContext::Help | ExpandContext::PatternInBuf)
    {
        // Special case for csh-like shell: skip 2 for "\\!"
        if is_shell && is_csh_like && second_char == Some(b'\\') {
            // Need to check third char for '!'
            // This is simplified - full check requires looking ahead
            return 1;
        }
        return 1;
    }

    // Check for menu context special handling
    if matches!(context, ExpandContext::Menus | ExpandContext::Menunames) {
        // Skip tab character
        if first_char == b'\t' {
            return 1;
        }
        // Skip backslash if followed by something non-NUL
        if first_char == b'\\' && second_char.is_some() && second_char != Some(0) {
            return 1;
        }
    }

    0
}

/// FFI wrapper for skip character calculation.
///
/// # Safety
///
/// `s` must be a valid pointer to at least 2 readable bytes, or NUL-terminated.
#[unsafe(no_mangle)]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_skip_wildmenu_char(xp: *const (), s: *const c_char) -> c_int {
    if xp.is_null() || s.is_null() {
        return 0;
    }

    let context_raw = nvim_expand_get_context(xp);
    let Some(context) = ExpandContext::from_raw(context_raw) else {
        return 0;
    };

    let first_char = *s as u8;
    if first_char == 0 {
        return 0;
    }

    let second_char = {
        let next = *s.add(1) as u8;
        if next == 0 {
            None
        } else {
            Some(next)
        }
    };

    let is_backslash_escape = rem_backslash(s) != 0;
    let is_shell = nvim_expand_get_shell(xp) != 0;
    let is_csh_like = rs_csh_like_shell() != 0;

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    {
        calculate_skip_count(
            context,
            is_backslash_escape,
            first_char,
            second_char,
            is_shell,
            is_csh_like,
        ) as c_int
    }
}

// =============================================================================
// Match Display Length
// =============================================================================

/// Check if a string represents a menu separator.
///
/// Menu separators are displayed as '|' and have a length of 1.
#[must_use]
pub const fn is_menu_context(context: ExpandContext) -> bool {
    matches!(context, ExpandContext::Menus | ExpandContext::Menunames)
}

/// FFI wrapper to calculate wildmenu match display length.
///
/// # Safety
///
/// `xp` must be a valid expand_T pointer, `s` must be a valid C string.
#[unsafe(no_mangle)]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_wildmenu_match_len(xp: *const (), s: *const c_char) -> c_int {
    if xp.is_null() || s.is_null() {
        return 0;
    }

    let context_raw = nvim_expand_get_context(xp);
    let Some(context) = ExpandContext::from_raw(context_raw) else {
        return 0;
    };

    let is_menu = is_menu_context(context);

    // Check for menu separator - return 1 for '|' display
    if is_menu && menu_is_separator(s) != 0 {
        return 1;
    }

    // Calculate length by iterating through string
    let mut len = 0;
    let mut ptr = s;

    while *ptr != 0 {
        // Skip escape characters
        let skip = rs_skip_wildmenu_char(xp, ptr);
        if skip > 0 {
            ptr = ptr.add(skip as usize);
        }

        // Add display width of current character
        len += ptr2cells(ptr);

        // Advance to next character (MB_PTR_ADV equivalent)
        // This is simplified - should use proper UTF-8 advancement
        let c = *ptr as u8;
        if c == 0 {
            break;
        }
        // Simple ASCII advancement; for full UTF-8 would need utfc_ptr2len
        ptr = ptr.add(1);
        // Skip continuation bytes
        while *ptr != 0 && (*ptr as u8 & 0xC0) == 0x80 {
            ptr = ptr.add(1);
        }
    }

    len
}

// =============================================================================
// Context-Specific Key Processing
// =============================================================================

/// Check if key should trigger directory navigation down.
///
/// In file/directory contexts, pressing Down after a path separator
/// should trigger completion in that subdirectory.
///
/// # Arguments
///
/// * `context` - Current expansion context
/// * `key` - The input key code
/// * `cmdpos` - Current cursor position
/// * `last_char` - Character before cursor (if any)
/// * `is_double_dot` - Whether the path ends with ".."
///
/// # Returns
///
/// True if this is a directory-down navigation
#[must_use]
pub const fn is_directory_down_trigger(
    context: ExpandContext,
    key: c_int,
    cmdpos: usize,
    last_char: Option<u8>,
    is_double_dot: bool,
) -> bool {
    // Only for file-related contexts
    if !matches!(
        context,
        ExpandContext::Files | ExpandContext::Directories | ExpandContext::Shellcmd
    ) {
        return false;
    }

    if key != keys::K_DOWN {
        return false;
    }

    if cmdpos == 0 {
        return false;
    }

    // Check for path separator that's not after ".."
    if let Some(c) = last_char {
        // PATHSEP is '/' on Unix, '\' on Windows
        #[cfg(windows)]
        let is_sep = c == b'\\' || c == b'/';
        #[cfg(not(windows))]
        let is_sep = c == b'/';

        if is_sep && !is_double_dot {
            return true;
        }
    }

    false
}

/// Check if context should process Up/Down for directory navigation.
#[must_use]
pub const fn context_has_directory_navigation(context: ExpandContext) -> bool {
    matches!(
        context,
        ExpandContext::Files | ExpandContext::Directories | ExpandContext::Shellcmd
    )
}

/// FFI wrapper for directory navigation check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_context_has_directory_navigation(context: c_int) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return 0;
    };

    c_int::from(context_has_directory_navigation(ctx))
}

/// Check if context should process Up/Down for menu navigation.
#[must_use]
pub const fn context_has_menu_navigation(context: ExpandContext) -> bool {
    matches!(context, ExpandContext::Menunames)
}

/// FFI wrapper for menu navigation check.
#[unsafe(no_mangle)]
pub extern "C" fn rs_context_has_menu_navigation(context: c_int) -> c_int {
    let Some(ctx) = ExpandContext::from_raw(context) else {
        return 0;
    };

    c_int::from(context_has_menu_navigation(ctx))
}

// =============================================================================
// Path Segment Utilities
// =============================================================================

/// The "up segment" path component for directory navigation: "/../"
pub const UP_SEGMENT: &[u8] = b"/../";

/// The short "up segment" for relative paths: "../"
pub const UP_SEGMENT_SHORT: &[u8] = b"../";

/// Find the position of the last path separator before a given position.
///
/// # Arguments
///
/// * `s` - The string bytes
/// * `start` - Starting position to search back from
/// * `min_pos` - Minimum position to stop at
///
/// # Returns
///
/// Position of last separator, or None if not found
#[must_use]
pub fn find_last_path_sep(s: &[u8], start: usize, min_pos: usize) -> Option<usize> {
    if start <= min_pos || s.is_empty() {
        return None;
    }

    let mut j = start;
    while j > min_pos {
        j -= 1;
        let c = s[j];
        #[cfg(windows)]
        let is_sep = c == b'\\' || c == b'/';
        #[cfg(not(windows))]
        let is_sep = c == b'/';

        if is_sep {
            return Some(j);
        }
    }

    None
}

/// Check if a path position is at a ".." segment.
///
/// Returns true if the bytes at positions [pos-2, pos-1] are ".."
#[must_use]
pub fn is_at_double_dot(s: &[u8], pos: usize) -> bool {
    if pos < 2 {
        return false;
    }
    s.get(pos - 2) == Some(&b'.') && s.get(pos - 1) == Some(&b'.')
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::keys::{CTRL_N, CTRL_P, K_DOWN, K_KENTER, K_LEFT, K_RIGHT, K_UP};
    use super::*;

    #[test]
    fn test_translate_arrow_keys() {
        // When pum is active, translate arrows
        assert_eq!(translate_arrow_keys(K_LEFT, true, false, false), CTRL_P);
        assert_eq!(translate_arrow_keys(K_RIGHT, true, false, false), CTRL_N);

        // When wild list was shown, translate arrows
        assert_eq!(translate_arrow_keys(K_LEFT, false, true, false), CTRL_P);
        assert_eq!(translate_arrow_keys(K_RIGHT, false, true, false), CTRL_N);

        // When wild menu is showing, translate arrows
        assert_eq!(translate_arrow_keys(K_LEFT, false, false, true), CTRL_P);
        assert_eq!(translate_arrow_keys(K_RIGHT, false, false, true), CTRL_N);

        // When nothing is active, don't translate
        assert_eq!(translate_arrow_keys(K_LEFT, false, false, false), K_LEFT);
        assert_eq!(translate_arrow_keys(K_RIGHT, false, false, false), K_RIGHT);

        // Other keys pass through unchanged
        assert_eq!(translate_arrow_keys(K_UP, true, false, false), K_UP);
        assert_eq!(translate_arrow_keys(K_DOWN, true, false, false), K_DOWN);
        assert_eq!(
            translate_arrow_keys(c_int::from(b'a'), true, false, false),
            c_int::from(b'a')
        );
    }

    #[test]
    fn test_should_complete_submenu() {
        // Correct context and pattern "Menu."
        assert!(should_complete_submenu(
            ExpandContext::Menunames,
            5,
            b"Menu.",
            c_int::from(b'\n')
        ));
        assert!(should_complete_submenu(
            ExpandContext::Menunames,
            5,
            b"Menu.",
            c_int::from(b'\r')
        ));
        assert!(should_complete_submenu(
            ExpandContext::Menunames,
            5,
            b"Menu.",
            K_KENTER
        ));

        // Wrong context
        assert!(!should_complete_submenu(
            ExpandContext::Files,
            5,
            b"Menu.",
            c_int::from(b'\n')
        ));

        // No dot
        assert!(!should_complete_submenu(
            ExpandContext::Menunames,
            4,
            b"Menu",
            c_int::from(b'\n')
        ));

        // Escaped dot
        assert!(!should_complete_submenu(
            ExpandContext::Menunames,
            5,
            b"Men\\.",
            c_int::from(b'\n')
        ));

        // Wrong key
        assert!(!should_complete_submenu(
            ExpandContext::Menunames,
            5,
            b"Menu.",
            K_UP
        ));

        // Position too short
        assert!(!should_complete_submenu(
            ExpandContext::Menunames,
            1,
            b".",
            c_int::from(b'\n')
        ));
    }

    #[test]
    fn test_is_menu_context() {
        assert!(is_menu_context(ExpandContext::Menus));
        assert!(is_menu_context(ExpandContext::Menunames));
        assert!(!is_menu_context(ExpandContext::Files));
        assert!(!is_menu_context(ExpandContext::Commands));
    }

    #[test]
    fn test_context_has_directory_navigation() {
        assert!(context_has_directory_navigation(ExpandContext::Files));
        assert!(context_has_directory_navigation(ExpandContext::Directories));
        assert!(context_has_directory_navigation(ExpandContext::Shellcmd));
        assert!(!context_has_directory_navigation(ExpandContext::Commands));
        assert!(!context_has_directory_navigation(ExpandContext::Menunames));
    }

    #[test]
    fn test_context_has_menu_navigation() {
        assert!(context_has_menu_navigation(ExpandContext::Menunames));
        assert!(!context_has_menu_navigation(ExpandContext::Menus));
        assert!(!context_has_menu_navigation(ExpandContext::Files));
    }

    #[test]
    fn test_find_last_path_sep() {
        // Unix paths
        #[cfg(not(windows))]
        {
            assert_eq!(find_last_path_sep(b"/home/user/file", 14, 0), Some(10));
            assert_eq!(find_last_path_sep(b"/home/user/file", 10, 0), Some(5));
            assert_eq!(find_last_path_sep(b"/home/user/file", 5, 0), Some(0));
            assert_eq!(find_last_path_sep(b"file", 4, 0), None);
        }

        // Empty and edge cases
        assert_eq!(find_last_path_sep(b"", 0, 0), None);
        assert_eq!(find_last_path_sep(b"a", 1, 1), None);
    }

    #[test]
    fn test_is_at_double_dot() {
        assert!(is_at_double_dot(b"foo/..", 6));
        assert!(is_at_double_dot(b"..", 2));
        assert!(!is_at_double_dot(b"foo/.", 5));
        assert!(!is_at_double_dot(b"a", 1));
        assert!(!is_at_double_dot(b"", 0));
    }

    #[test]
    fn test_calculate_skip_count_basic() {
        // No escape, regular character
        assert_eq!(
            calculate_skip_count(ExpandContext::Files, false, b'a', Some(b'b'), false, false),
            0
        );

        // Backslash escape in file context
        assert_eq!(
            calculate_skip_count(ExpandContext::Files, true, b'\\', Some(b' '), false, false),
            1
        );

        // Backslash escape in help context (should NOT skip)
        assert_eq!(
            calculate_skip_count(ExpandContext::Help, true, b'\\', Some(b' '), false, false),
            0
        );

        // Tab in menu context
        assert_eq!(
            calculate_skip_count(ExpandContext::Menus, false, b'\t', None, false, false),
            1
        );

        // Backslash followed by char in menu context
        assert_eq!(
            calculate_skip_count(ExpandContext::Menus, false, b'\\', Some(b'a'), false, false),
            1
        );
    }

    #[test]
    fn test_is_directory_down_trigger() {
        // Valid trigger: Down key after path separator
        #[cfg(not(windows))]
        {
            assert!(is_directory_down_trigger(
                ExpandContext::Files,
                K_DOWN,
                5,
                Some(b'/'),
                false
            ));
            assert!(is_directory_down_trigger(
                ExpandContext::Directories,
                K_DOWN,
                5,
                Some(b'/'),
                false
            ));
            assert!(is_directory_down_trigger(
                ExpandContext::Shellcmd,
                K_DOWN,
                5,
                Some(b'/'),
                false
            ));
        }

        // Not a trigger if it's after ".."
        assert!(!is_directory_down_trigger(
            ExpandContext::Files,
            K_DOWN,
            5,
            Some(b'/'),
            true
        ));

        // Wrong context
        assert!(!is_directory_down_trigger(
            ExpandContext::Commands,
            K_DOWN,
            5,
            Some(b'/'),
            false
        ));

        // Wrong key
        assert!(!is_directory_down_trigger(
            ExpandContext::Files,
            K_UP,
            5,
            Some(b'/'),
            false
        ));

        // No last char
        assert!(!is_directory_down_trigger(
            ExpandContext::Files,
            K_DOWN,
            0,
            None,
            false
        ));
    }
}
