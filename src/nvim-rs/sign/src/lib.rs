//! Sign management for Neovim
//!
//! This crate provides Rust implementations of sign-related functions
//! from `src/nvim/sign.c`. Signs are markers displayed in the sign column
//! that can indicate breakpoints, errors, warnings, and other information.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::ffi::{c_char, c_int, c_void, CStr};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of signs shown on a single line
pub const SIGN_SHOW_MAX: c_int = 9;

/// Default sign highlight priority
pub const SIGN_DEF_PRIO: c_int = 10;

/// Sign text width (2 characters)
pub const SIGN_WIDTH: usize = 2;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to C's sign_T structure
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SignHandle(*mut c_void);

impl SignHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque handle to C's buf_T structure (for sign operations)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct SignBufHandle(*mut c_void);

impl SignBufHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// Sign Command Enumeration
// =============================================================================

/// Sign command types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignCmd {
    /// Define a new sign
    Define = 0,
    /// Undefine an existing sign
    Undefine = 1,
    /// List defined signs
    List = 2,
    /// Place a sign
    Place = 3,
    /// Remove a placed sign
    Unplace = 4,
    /// Jump to a sign
    Jump = 5,
}

impl SignCmd {
    /// Total number of sign commands
    pub const COUNT: usize = 6;

    /// Convert from integer, returning None if invalid
    pub const fn from_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Define),
            1 => Some(Self::Undefine),
            2 => Some(Self::List),
            3 => Some(Self::Place),
            4 => Some(Self::Unplace),
            5 => Some(Self::Jump),
            _ => None,
        }
    }
}

// =============================================================================
// Sign Command Parsing
// =============================================================================

/// Sign command names for parsing
const SIGN_CMD_NAMES: [&str; SignCmd::COUNT] =
    ["define", "undefine", "list", "place", "unplace", "jump"];

/// Parse a sign command name and return its index.
///
/// Returns the command index (0-5) or -1 if not found.
fn sign_cmd_idx_impl(cmd: &str) -> c_int {
    for (idx, name) in SIGN_CMD_NAMES.iter().enumerate() {
        if cmd.starts_with(name) || name.starts_with(cmd) {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            return idx as c_int;
        }
    }
    -1
}

/// Parse a sign command name from a C string.
///
/// Returns the command index (0-5) or -1 if not found.
///
/// # Safety
/// `cmd` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_cmd_idx(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        return -1;
    }

    CStr::from_ptr(cmd)
        .to_str()
        .map_or(-1, sign_cmd_idx_impl)
}

// =============================================================================
// Namespace Filtering
// =============================================================================

/// Special namespace values
pub const NS_GLOBAL: i64 = 0;
pub const NS_ALL: i64 = u32::MAX as i64;
pub const NS_INVALID: i64 = -1;

/// Convert a group name to a namespace filter value.
///
/// Returns:
/// - 0 for global namespace (NULL group)
/// - UINT32_MAX for all namespaces ("*")
/// - -1 for invalid/non-existing namespace
/// - The namespace ID for a valid named namespace
///
/// # Safety
/// `group` must be null or a valid null-terminated C string.
/// `ns_lookup` must be a valid function pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_group_get_ns(
    group: *const c_char,
    ns_lookup: extern "C" fn(*const c_char) -> c_int,
) -> i64 {
    if group.is_null() {
        return NS_GLOBAL;
    }

    let group_cstr = CStr::from_ptr(group);
    let group_bytes = group_cstr.to_bytes();

    // Check for "*" (all namespaces)
    if group_bytes == b"*" {
        return NS_ALL;
    }

    // Look up the namespace
    let ns = ns_lookup(group);
    if ns != 0 {
        i64::from(ns)
    } else {
        NS_INVALID
    }
}

// =============================================================================
// Sign Row Comparison
// =============================================================================

/// Compare two sign rows for sorting.
///
/// Signs are sorted by:
/// 1. Row number (ascending)
/// 2. Priority (handled by caller via sign_item_cmp)
///
/// Returns:
/// - negative if row1 < row2
/// - 0 if row1 == row2
/// - positive if row1 > row2
#[no_mangle]
pub extern "C" fn rs_sign_row_cmp(row1: c_int, row2: c_int) -> c_int {
    row1.cmp(&row2) as c_int
}

// =============================================================================
// Buffer Sign Queries
// =============================================================================

/// Check if a buffer has any signs.
///
/// This is a placeholder that will delegate to the marktree.
///
/// # Safety
/// `buf` must be a valid buffer handle or null.
#[no_mangle]
pub extern "C" fn rs_buf_has_signs(_buf: SignBufHandle) -> c_int {
    // This will need to call into marktree iteration
    // For now, return 0 (no signs) as a safe default
    // The actual implementation requires marktree access
    0
}

// =============================================================================
// Sign Name Validation
// =============================================================================

/// Check if a sign name is valid.
///
/// A valid sign name:
/// - Is not empty
/// - Starts with a letter or underscore
/// - Contains only alphanumeric characters and underscores
fn is_valid_sign_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();
    let first = chars.next().unwrap();

    // First character must be letter or underscore
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }

    // Rest must be alphanumeric or underscore
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Check if a sign name is valid.
///
/// # Safety
/// `name` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_name_valid(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    CStr::from_ptr(name)
        .to_str()
        .map_or(0, |s| c_int::from(is_valid_sign_name(s)))
}

// =============================================================================
// Sign Text Utilities
// =============================================================================

/// Calculate the display width of sign text.
///
/// Sign text should be exactly 2 display cells wide.
/// Returns the width in cells, or -1 if invalid.
///
/// # Safety
/// `text` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_text_width(text: *const c_char) -> c_int {
    if text.is_null() {
        return -1;
    }

    // The actual width calculation requires UTF-8 cell width logic
    // For ASCII, it's simply the byte length (up to 2)
    let cstr = CStr::from_ptr(text);
    let bytes = cstr.to_bytes();

    // Simple ASCII check - for full implementation need mbyte functions
    if bytes.is_empty() {
        return 0;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let len = bytes.len().min(SIGN_WIDTH) as c_int;
    len
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_cmd_from_int() {
        assert_eq!(SignCmd::from_int(0), Some(SignCmd::Define));
        assert_eq!(SignCmd::from_int(1), Some(SignCmd::Undefine));
        assert_eq!(SignCmd::from_int(2), Some(SignCmd::List));
        assert_eq!(SignCmd::from_int(3), Some(SignCmd::Place));
        assert_eq!(SignCmd::from_int(4), Some(SignCmd::Unplace));
        assert_eq!(SignCmd::from_int(5), Some(SignCmd::Jump));
        assert_eq!(SignCmd::from_int(6), None);
        assert_eq!(SignCmd::from_int(-1), None);
    }

    #[test]
    fn test_sign_cmd_idx() {
        assert_eq!(sign_cmd_idx_impl("define"), 0);
        assert_eq!(sign_cmd_idx_impl("def"), 0);
        assert_eq!(sign_cmd_idx_impl("undefine"), 1);
        assert_eq!(sign_cmd_idx_impl("list"), 2);
        assert_eq!(sign_cmd_idx_impl("place"), 3);
        assert_eq!(sign_cmd_idx_impl("unplace"), 4);
        assert_eq!(sign_cmd_idx_impl("jump"), 5);
        assert_eq!(sign_cmd_idx_impl("invalid"), -1);
        assert_eq!(sign_cmd_idx_impl(""), -1);
    }

    #[test]
    fn test_sign_row_cmp() {
        assert!(rs_sign_row_cmp(1, 2) < 0);
        assert!(rs_sign_row_cmp(2, 1) > 0);
        assert_eq!(rs_sign_row_cmp(1, 1), 0);
    }

    #[test]
    fn test_is_valid_sign_name() {
        assert!(is_valid_sign_name("MySign"));
        assert!(is_valid_sign_name("_private"));
        assert!(is_valid_sign_name("sign123"));
        assert!(is_valid_sign_name("a"));
        assert!(!is_valid_sign_name(""));
        assert!(!is_valid_sign_name("123sign"));
        assert!(!is_valid_sign_name("-invalid"));
    }

    #[test]
    fn test_sign_handle_null() {
        let handle = SignHandle::null();
        assert!(handle.is_null());
    }

    #[test]
    fn test_namespace_constants() {
        assert_eq!(NS_GLOBAL, 0);
        assert_eq!(NS_ALL, i64::from(u32::MAX));
        assert_eq!(NS_INVALID, -1);
    }

    #[test]
    fn test_sign_constants() {
        assert_eq!(SIGN_SHOW_MAX, 9);
        assert_eq!(SIGN_DEF_PRIO, 10);
        assert_eq!(SIGN_WIDTH, 2);
    }
}
