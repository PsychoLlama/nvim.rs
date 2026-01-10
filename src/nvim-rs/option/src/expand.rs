//! Option value expansion and completions
//!
//! This module provides Rust implementations for option value
//! expansion including environment variables, special characters,
//! and completion candidates.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int};

use crate::{OptFlags, OptValType};

// =============================================================================
// Expansion Types
// =============================================================================

/// Type of expansion to perform.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExpandType {
    /// No expansion
    #[default]
    None = 0,
    /// Expand environment variables
    Env = 1,
    /// Expand ~ to home directory
    Home = 2,
    /// Expand file names
    File = 3,
    /// Expand directory names
    Dir = 4,
    /// Expand option values
    OptVal = 5,
    /// Expand buffer names
    Buffer = 6,
    /// Expand tag names
    Tag = 7,
}

impl ExpandType {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Env,
            2 => Self::Home,
            3 => Self::File,
            4 => Self::Dir,
            5 => Self::OptVal,
            6 => Self::Buffer,
            7 => Self::Tag,
            _ => Self::None,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if this type expands file system entries.
    #[must_use]
    pub const fn is_filesystem(self) -> bool {
        matches!(self, Self::File | Self::Dir)
    }
}

// =============================================================================
// Expansion Context
// =============================================================================

/// Context for expansion operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ExpandContext {
    /// Type of expansion
    pub expand_type: c_int,
    /// Starting column of text to expand
    pub start_col: c_int,
    /// Ending column
    pub end_col: c_int,
    /// Whether expansion is at command start
    pub at_start: bool,
    /// Whether to show all matches
    pub show_all: bool,
    /// Option flags affecting expansion
    pub opt_flags: u32,
}

impl ExpandContext {
    /// Create a new expansion context.
    #[must_use]
    pub const fn new(expand_type: ExpandType) -> Self {
        Self {
            expand_type: expand_type.to_c_int(),
            start_col: 0,
            end_col: 0,
            at_start: false,
            show_all: false,
            opt_flags: 0,
        }
    }

    /// Get expansion type.
    #[must_use]
    pub const fn get_type(&self) -> ExpandType {
        ExpandType::from_c_int(self.expand_type)
    }

    /// Check if environment expansion is needed.
    #[must_use]
    pub const fn needs_env(&self) -> bool {
        self.opt_flags & OptFlags::EXPAND.0 != 0
    }
}

// =============================================================================
// Environment Variable Expansion
// =============================================================================

/// Check if position is at start of an environment variable.
/// Looks for $NAME or ${NAME} patterns.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_at_env_var(str: *const c_char, pos: c_int) -> c_int {
    if str.is_null() || pos < 0 {
        return 0;
    }

    let p = str.add(pos as usize);
    c_int::from(*p as u8 == b'$')
}

/// Find start of environment variable at position.
/// Scans backward to find the $ character.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_find_env_start(str: *const c_char, pos: c_int) -> c_int {
    if str.is_null() || pos <= 0 {
        return -1;
    }

    let mut i = pos - 1;
    while i >= 0 {
        let c = *str.add(i as usize) as u8;
        if c == b'$' {
            return i;
        }
        // Stop at path separators or whitespace
        if c == b'/' || c == b'\\' || c == b' ' || c == b'\t' {
            break;
        }
        i -= 1;
    }

    -1
}

/// Check if character is valid in environment variable name.
#[no_mangle]
pub extern "C" fn rs_is_env_char(c: c_int) -> c_int {
    let ch = c as u8;
    c_int::from(ch.is_ascii_alphanumeric() || ch == b'_')
}

/// Extract environment variable name from string.
/// Returns length of the variable name (excluding $).
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_env_name_len(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }

    let mut p = str;

    // Skip $
    if *p as u8 == b'$' {
        p = p.add(1);
    }

    // Handle ${NAME} form
    let brace = *p as u8 == b'{';
    if brace {
        p = p.add(1);
    }

    let start = p;

    // Count valid env chars
    while *p != 0 {
        let c = *p as u8;
        if brace {
            if c == b'}' {
                break;
            }
        } else if !c.is_ascii_alphanumeric() && c != b'_' {
            break;
        }
        p = p.add(1);
    }

    (p as usize - start as usize) as c_int
}

// =============================================================================
// Home Directory Expansion
// =============================================================================

/// Check if string starts with ~ for home expansion.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_needs_home_expand(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }
    c_int::from(*str as u8 == b'~')
}

/// Get length of ~user prefix.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_tilde_user_len(str: *const c_char) -> c_int {
    if str.is_null() || *str as u8 != b'~' {
        return 0;
    }

    let mut p = str.add(1);
    let start = p;

    // Count username characters
    while *p != 0 {
        let c = *p as u8;
        if c == b'/' || c == b'\\' || c == b' ' || c == b'\t' {
            break;
        }
        p = p.add(1);
    }

    // Add 1 for the ~
    1 + (p as usize - start as usize) as c_int
}

// =============================================================================
// Path Expansion
// =============================================================================

/// Check if character is a path separator.
#[no_mangle]
pub extern "C" fn rs_opt_is_path_sep(c: c_int) -> c_int {
    let ch = c as u8;
    c_int::from(ch == b'/' || ch == b'\\')
}

/// Find last path separator in string.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_find_last_path_sep(str: *const c_char) -> c_int {
    if str.is_null() {
        return -1;
    }

    let mut last: c_int = -1;
    let mut i: c_int = 0;
    let mut p = str;

    while *p != 0 {
        let c = *p as u8;
        if c == b'/' || c == b'\\' {
            last = i;
        }
        p = p.add(1);
        i += 1;
    }

    last
}

/// Check if path is absolute.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_opt_is_absolute_path(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }

    let c = *str as u8;

    // Unix absolute path
    if c == b'/' {
        return 1;
    }

    // Windows absolute path (C:\...)
    if c.is_ascii_alphabetic() && *str.add(1) as u8 == b':' {
        return 1;
    }

    0
}

// =============================================================================
// Comma-Separated Values
// =============================================================================

/// Count items in a comma-separated list.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_count_csv_items(str: *const c_char) -> c_int {
    if str.is_null() || *str == 0 {
        return 0;
    }

    let mut count: c_int = 1;
    let mut p = str;
    let mut in_escape = false;

    while *p != 0 {
        let c = *p as u8;
        if in_escape {
            in_escape = false;
        } else if c == b'\\' {
            in_escape = true;
        } else if c == b',' {
            count += 1;
        }
        p = p.add(1);
    }

    count
}

/// Find start of item at given index in CSV.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_find_csv_item(str: *const c_char, index: c_int) -> c_int {
    if str.is_null() || index < 0 {
        return -1;
    }

    let mut current: c_int = 0;
    let mut pos: c_int = 0;
    let mut p = str;
    let mut in_escape = false;

    while *p != 0 {
        if current == index {
            return pos;
        }

        let c = *p as u8;
        if in_escape {
            in_escape = false;
        } else if c == b'\\' {
            in_escape = true;
        } else if c == b',' {
            current += 1;
        }

        p = p.add(1);
        pos += 1;
    }

    if current == index {
        pos
    } else {
        -1
    }
}

/// Get length of CSV item starting at position.
///
/// # Safety
/// `str` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_csv_item_len(str: *const c_char) -> c_int {
    if str.is_null() {
        return 0;
    }

    let mut len: c_int = 0;
    let mut p = str;
    let mut in_escape = false;

    while *p != 0 {
        let c = *p as u8;
        if in_escape {
            in_escape = false;
        } else if c == b'\\' {
            in_escape = true;
        } else if c == b',' {
            break;
        }

        p = p.add(1);
        len += 1;
    }

    len
}

// =============================================================================
// Value Completion
// =============================================================================

/// Option value completion info.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ValueCompInfo {
    /// Completion start position
    pub start: c_int,
    /// Completion end position
    pub end: c_int,
    /// Type of completion
    pub comp_type: c_int,
    /// Whether in quoted string
    pub in_quote: bool,
}

/// Determine completion info for option value.
///
/// # Safety
/// `value` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_get_value_comp_info(
    value: *const c_char,
    pos: c_int,
    opt_type: c_int,
) -> ValueCompInfo {
    if value.is_null() || pos < 0 {
        return ValueCompInfo::default();
    }

    let mut info = ValueCompInfo {
        start: 0,
        end: pos,
        comp_type: 0,
        in_quote: false,
    };

    // For string options, find word boundaries
    if opt_type == OptValType::String as c_int {
        let mut i = pos;

        // Scan backward for word start
        while i > 0 {
            let c = *value.add((i - 1) as usize) as u8;
            if c == b',' || c == b' ' || c == b'\t' {
                break;
            }
            i -= 1;
        }
        info.start = i;

        // Check for env var - scan from info.start to pos
        let mut scan_pos = info.start;
        while scan_pos < pos {
            if *value.add(scan_pos as usize) as u8 == b'$' {
                info.comp_type = ExpandType::Env as c_int;
                info.start = scan_pos;
                break;
            }
            scan_pos += 1;
        }

        // Check for path
        if info.comp_type == 0 {
            let first = *value.add(info.start as usize) as u8;
            if first == b'~' || first == b'/' || first == b'.' {
                info.comp_type = ExpandType::File as c_int;
            }
        }
    }

    info
}

// =============================================================================
// Expand Context FFI
// =============================================================================

/// FFI: Create expansion context.
#[no_mangle]
pub extern "C" fn rs_expand_context_new(expand_type: c_int) -> ExpandContext {
    ExpandContext::new(ExpandType::from_c_int(expand_type))
}

/// FFI: Check if context needs env expansion.
///
/// # Safety
/// `ctx` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_expand_needs_env(ctx: *const ExpandContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).needs_env())
}

/// FFI: Check if expand type is filesystem.
#[no_mangle]
pub extern "C" fn rs_expand_is_filesystem(expand_type: c_int) -> c_int {
    c_int::from(ExpandType::from_c_int(expand_type).is_filesystem())
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_type() {
        assert_eq!(ExpandType::from_c_int(0), ExpandType::None);
        assert_eq!(ExpandType::from_c_int(1), ExpandType::Env);
        assert!(ExpandType::File.is_filesystem());
        assert!(ExpandType::Dir.is_filesystem());
        assert!(!ExpandType::Env.is_filesystem());
    }

    #[test]
    fn test_expand_context() {
        let ctx = ExpandContext::new(ExpandType::Env);
        assert_eq!(ctx.get_type(), ExpandType::Env);
    }

    #[test]
    fn test_at_env_var() {
        unsafe {
            assert_eq!(rs_at_env_var(c"$HOME".as_ptr(), 0), 1);
            assert_eq!(rs_at_env_var(c"/path/$VAR".as_ptr(), 0), 0);
            assert_eq!(rs_at_env_var(c"/path/$VAR".as_ptr(), 6), 1);
        }
    }

    #[test]
    fn test_env_name_len() {
        unsafe {
            assert_eq!(rs_env_name_len(c"$HOME/path".as_ptr()), 4);
            assert_eq!(rs_env_name_len(c"${HOME}/path".as_ptr()), 4);
            assert_eq!(rs_env_name_len(c"$VAR_NAME".as_ptr()), 8);
        }
    }

    #[test]
    fn test_needs_home_expand() {
        unsafe {
            assert_eq!(rs_needs_home_expand(c"~/.config".as_ptr()), 1);
            assert_eq!(rs_needs_home_expand(c"/home/user".as_ptr()), 0);
        }
    }

    #[test]
    fn test_tilde_user_len() {
        unsafe {
            assert_eq!(rs_tilde_user_len(c"~/".as_ptr()), 1);
            assert_eq!(rs_tilde_user_len(c"~user/".as_ptr()), 5);
            assert_eq!(rs_tilde_user_len(c"/path".as_ptr()), 0);
        }
    }

    #[test]
    fn test_is_path_sep() {
        assert_eq!(rs_opt_is_path_sep(c_int::from(b'/')), 1);
        assert_eq!(rs_opt_is_path_sep(c_int::from(b'\\')), 1);
        assert_eq!(rs_opt_is_path_sep(c_int::from(b'a')), 0);
    }

    #[test]
    fn test_find_last_path_sep() {
        unsafe {
            assert_eq!(rs_find_last_path_sep(c"/a/b/c".as_ptr()), 4);
            assert_eq!(rs_find_last_path_sep(c"nopath".as_ptr()), -1);
        }
    }

    #[test]
    fn test_is_absolute_path() {
        unsafe {
            assert_eq!(rs_opt_is_absolute_path(c"/home".as_ptr()), 1);
            assert_eq!(rs_opt_is_absolute_path(c"relative".as_ptr()), 0);
        }
    }

    #[test]
    fn test_count_csv_items() {
        unsafe {
            assert_eq!(rs_count_csv_items(c"".as_ptr()), 0);
            assert_eq!(rs_count_csv_items(c"one".as_ptr()), 1);
            assert_eq!(rs_count_csv_items(c"one,two,three".as_ptr()), 3);
            assert_eq!(rs_count_csv_items(c"a\\,b,c".as_ptr()), 2); // Escaped comma
        }
    }

    #[test]
    fn test_find_csv_item() {
        unsafe {
            let s = c"one,two,three".as_ptr();
            assert_eq!(rs_find_csv_item(s, 0), 0);
            assert_eq!(rs_find_csv_item(s, 1), 4);
            assert_eq!(rs_find_csv_item(s, 2), 8);
        }
    }

    #[test]
    fn test_csv_item_len() {
        unsafe {
            assert_eq!(rs_csv_item_len(c"one,two".as_ptr()), 3);
            assert_eq!(rs_csv_item_len(c"a\\,b,c".as_ptr()), 4); // "a\,b"
        }
    }
}
