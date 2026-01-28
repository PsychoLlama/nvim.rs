//! Sign management for Neovim
//!
//! This crate provides Rust implementations of sign-related functions
//! from `src/nvim/sign.c`. Signs are markers displayed in the sign column
//! that can indicate breakpoints, errors, warnings, and other information.
//!
//! # Architecture
//!
//! Signs are built on top of extmarks and use the decoration system.
//! This crate follows the opaque handle pattern used elsewhere in the
//! Neovim Rust migration.
//!
//! # Modules
//!
//! - `text` - Sign text utilities (initialization, display)
//! - `define` - Sign definition management
//! - `place` - Sign placement operations
//! - `remove` - Sign removal operations
//! - `query` - Sign querying and listing
//! - `commands` - Ex command handlers

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const

use std::ffi::{c_char, c_int, c_void, CStr};

// =============================================================================
// Submodules
// =============================================================================

pub mod commands;
pub mod define;
pub mod place;
pub mod query;
pub mod remove;
pub mod storage;
pub mod text;

// Re-exports
pub use define::{SignDefineError, SignDefineParams, SignHighlights};
pub use storage::{SignMapIterator, SignNamespace, SignProperties};
pub use text::{ScharT, SignTextResult, MAX_SCHAR_SIZE};

// =============================================================================
// Constants
// =============================================================================

/// Maximum number of signs shown on a single line
pub const SIGN_SHOW_MAX: c_int = 9;

/// Default sign highlight priority
pub const SIGN_DEF_PRIO: c_int = 10;

/// Sign text width (2 characters)
pub const SIGN_WIDTH: usize = 2;

/// Global namespace (ns = 0)
pub const NS_GLOBAL: i64 = 0;

/// All namespaces sentinel value (matches UINT32_MAX)
pub const NS_ALL: i64 = u32::MAX as i64;

/// Invalid namespace sentinel value
pub const NS_INVALID: i64 = -1;

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
#[derive(Clone, Copy, Debug)]
pub struct SignBufHandle(*mut c_void);

impl SignBufHandle {
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

    /// Get raw pointer
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to C's DecorSignHighlight structure
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct DecorSignHighlightHandle(*mut c_void);

impl DecorSignHighlightHandle {
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

    /// Get raw pointer
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Opaque handle to C's marktree iterator
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct MarkTreeIterHandle(*mut c_void);

impl MarkTreeIterHandle {
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

/// Opaque handle to C's MTKey structure (marktree key)
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct MTKeyHandle(*mut c_void);

impl MTKeyHandle {
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

    /// Get raw pointer
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }
}

/// Line number type (matches linenr_T in Neovim)
pub type LinenrT = i32;

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    // sign_T accessors
    fn nvim_sign_get_name(sp: SignHandle) -> *const c_char;
    fn nvim_sign_get_icon(sp: SignHandle) -> *const c_char;
    fn nvim_sign_get_text_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_line_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_num_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_cul_hl(sp: SignHandle) -> c_int;
    fn nvim_sign_get_priority(sp: SignHandle) -> c_int;

    // DecorSignHighlight accessors
    fn nvim_decor_sh_get_flags(sh: DecorSignHighlightHandle) -> u16;
    fn nvim_decor_sh_get_priority(sh: DecorSignHighlightHandle) -> u16;
    fn nvim_decor_sh_get_hl_id(sh: DecorSignHighlightHandle) -> c_int;
    fn nvim_decor_sh_get_sign_name(sh: DecorSignHighlightHandle) -> *const c_char;
    fn nvim_decor_sh_get_sign_add_id(sh: DecorSignHighlightHandle) -> c_int;
    fn nvim_decor_sh_get_number_hl_id(sh: DecorSignHighlightHandle) -> c_int;
    fn nvim_decor_sh_get_line_hl_id(sh: DecorSignHighlightHandle) -> c_int;
    fn nvim_decor_sh_get_cursorline_hl_id(sh: DecorSignHighlightHandle) -> c_int;
    fn nvim_decor_sh_get_next(sh: DecorSignHighlightHandle) -> u32;

    // Buffer sign accessors
    fn nvim_buf_get_marktree(buf: SignBufHandle) -> *mut c_void;
    fn nvim_buf_get_fname(buf: SignBufHandle) -> *const c_char;
    fn nvim_buf_get_fnum(buf: SignBufHandle) -> c_int;
    fn nvim_buf_get_next(buf: SignBufHandle) -> SignBufHandle;

    // Marktree/MTKey accessors
    fn nvim_mtkey_get_row(key: MTKeyHandle) -> c_int;
    fn nvim_mtkey_get_col(key: MTKeyHandle) -> c_int;
    fn nvim_mtkey_get_ns(key: MTKeyHandle) -> u32;
    fn nvim_mtkey_get_id(key: MTKeyHandle) -> u32;
    fn nvim_mtkey_is_end(key: MTKeyHandle) -> bool;
    fn nvim_mtkey_is_decor_sign(key: MTKeyHandle) -> bool;

    // Sign map operations
    fn nvim_sign_map_get(name: *const c_char) -> SignHandle;
    fn nvim_sign_map_has(name: *const c_char) -> bool;

    // Namespace operations
    fn nvim_namespace_lookup(name: *const c_char) -> c_int;
    fn nvim_describe_ns(ns: c_int, empty: *const c_char) -> *const c_char;
    fn nvim_create_namespace(name: *const c_char) -> c_int;

    // Decoration helpers
    fn nvim_decor_find_sign(decor: *const c_void) -> DecorSignHighlightHandle;
    fn nvim_mt_decor(key: MTKeyHandle) -> *const c_void;

    // Buffer meta totals
    fn nvim_buf_meta_total_sign_hl(buf: SignBufHandle) -> u64;
    fn nvim_buf_meta_total_sign_text(buf: SignBufHandle) -> u64;

    // Global buffer list
    fn nvim_get_firstbuf() -> SignBufHandle;
    fn nvim_get_curbuf() -> SignBufHandle;

    // Extmark operations
    fn nvim_extmark_del_id(buf: SignBufHandle, ns: u32, id: u32) -> bool;
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
/// Returns the command index (0-5) or SIGNCMD_LAST (6) if not found.
/// This matches the C behavior where the loop exits when cmds[idx] == NULL.
fn sign_cmd_idx_impl(cmd: &str) -> c_int {
    for (idx, name) in SIGN_CMD_NAMES.iter().enumerate() {
        if cmd == *name {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            return idx as c_int;
        }
    }
    // Return SIGNCMD_LAST (6) for unrecognized commands
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    (SignCmd::COUNT as c_int)
}

/// Parse a sign command name from a C string.
///
/// Returns the command index (0-5) or SIGNCMD_LAST (6) if not found.
/// Returns SIGNCMD_LAST for null input as well.
///
/// # Safety
/// `cmd` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_cmd_idx(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        return SignCmd::COUNT as c_int;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    CStr::from_ptr(cmd)
        .to_str()
        .map_or(SignCmd::COUNT as c_int, sign_cmd_idx_impl)
}

// =============================================================================
// Namespace Filtering
// =============================================================================

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
// Sign Item Comparison
// =============================================================================

/// Compare two sign items for priority-based sorting.
///
/// Signs are compared by (all descending - higher values first):
/// 1. Priority
/// 2. Sign ID
/// 3. Sign add ID (recency)
///
/// Returns:
/// - positive if s1 < s2 (s2 should come first)
/// - negative if s1 > s2 (s1 should come first)
/// - 0 if equal
#[no_mangle]
pub extern "C" fn rs_sign_item_cmp(
    priority1: c_int,
    id1: u32,
    add_id1: u32,
    priority2: c_int,
    id2: u32,
    add_id2: u32,
) -> c_int {
    // Compare by priority (descending - higher priority first)
    if priority1 != priority2 {
        return if priority1 < priority2 { 1 } else { -1 };
    }

    // Compare by ID (descending - higher ID first)
    if id1 != id2 {
        return if id1 < id2 { 1 } else { -1 };
    }

    // Compare by sign_add_id (descending - more recent first)
    if add_id1 != add_id2 {
        return if add_id1 < add_id2 { 1 } else { -1 };
    }

    0
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
// Sign Priority Utilities
// =============================================================================

/// Check if a priority value is valid.
///
/// Priority must be non-negative. -1 is special (use SIGN_DEF_PRIO default).
#[no_mangle]
pub extern "C" fn rs_sign_priority_valid(prio: c_int) -> c_int {
    c_int::from(prio >= -1)
}

/// Get the effective priority for a sign.
///
/// If prio is -1, returns the default priority (SIGN_DEF_PRIO).
/// Otherwise returns the given priority.
#[no_mangle]
pub extern "C" fn rs_sign_effective_priority(prio: c_int) -> c_int {
    if prio == -1 {
        SIGN_DEF_PRIO
    } else {
        prio
    }
}

// =============================================================================
// Sign Argument Parsing Helpers
// =============================================================================

/// Check if a character is a valid sign argument delimiter.
///
/// Returns true for '=' (key=value separator).
#[no_mangle]
pub extern "C" fn rs_sign_is_arg_delim(c: c_int) -> c_int {
    c_int::from(c == i32::from(b'='))
}

/// Sign argument prefixes
const SIGN_ARG_PREFIXES: [&[u8]; 6] = [
    b"line=",
    b"name=",
    b"group=",
    b"priority=",
    b"file=",
    b"buffer=",
];

/// Check if a string starts with a sign argument prefix.
///
/// Valid prefixes: "line=", "name=", "group=", "priority=", "file=", "buffer="
/// Returns 1-6 for the prefix index (1-based), 0 if no match.
///
/// # Safety
/// `s` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_arg_prefix(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }

    let cstr = CStr::from_ptr(s);
    let bytes = cstr.to_bytes();

    for (idx, prefix) in SIGN_ARG_PREFIXES.iter().enumerate() {
        if bytes.starts_with(prefix) {
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            return (idx + 1) as c_int;
        }
    }

    0 // No recognized prefix
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
        // Valid commands (exact match)
        assert_eq!(sign_cmd_idx_impl("define"), 0);
        assert_eq!(sign_cmd_idx_impl("undefine"), 1);
        assert_eq!(sign_cmd_idx_impl("list"), 2);
        assert_eq!(sign_cmd_idx_impl("place"), 3);
        assert_eq!(sign_cmd_idx_impl("unplace"), 4);
        assert_eq!(sign_cmd_idx_impl("jump"), 5);
        // Invalid commands return SIGNCMD_LAST (6)
        assert_eq!(sign_cmd_idx_impl("def"), 6); // Partial match not accepted
        assert_eq!(sign_cmd_idx_impl("invalid"), 6);
        assert_eq!(sign_cmd_idx_impl(""), 6);
    }

    #[test]
    fn test_sign_row_cmp() {
        assert!(rs_sign_row_cmp(1, 2) < 0);
        assert!(rs_sign_row_cmp(2, 1) > 0);
        assert_eq!(rs_sign_row_cmp(1, 1), 0);
    }

    #[test]
    fn test_sign_item_cmp() {
        // Higher priority comes first (returns negative)
        assert!(rs_sign_item_cmp(10, 1, 1, 5, 1, 1) < 0);
        // Lower priority comes later (returns positive)
        assert!(rs_sign_item_cmp(5, 1, 1, 10, 1, 1) > 0);

        // Same priority, higher ID comes first
        assert!(rs_sign_item_cmp(10, 100, 1, 10, 50, 1) < 0);
        assert!(rs_sign_item_cmp(10, 50, 1, 10, 100, 1) > 0);

        // Same priority and ID, higher add_id comes first
        assert!(rs_sign_item_cmp(10, 1, 100, 10, 1, 50) < 0);
        assert!(rs_sign_item_cmp(10, 1, 50, 10, 1, 100) > 0);

        // All equal
        assert_eq!(rs_sign_item_cmp(10, 1, 1, 10, 1, 1), 0);
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

    #[test]
    fn test_sign_priority_valid() {
        assert_eq!(rs_sign_priority_valid(-1), 1); // -1 is valid (default)
        assert_eq!(rs_sign_priority_valid(0), 1);
        assert_eq!(rs_sign_priority_valid(10), 1);
        assert_eq!(rs_sign_priority_valid(-2), 0); // Invalid
    }

    #[test]
    fn test_sign_effective_priority() {
        assert_eq!(rs_sign_effective_priority(-1), SIGN_DEF_PRIO);
        assert_eq!(rs_sign_effective_priority(0), 0);
        assert_eq!(rs_sign_effective_priority(5), 5);
        assert_eq!(rs_sign_effective_priority(100), 100);
    }

    #[test]
    fn test_sign_is_arg_delim() {
        assert_eq!(rs_sign_is_arg_delim(c_int::from(b'=')), 1);
        assert_eq!(rs_sign_is_arg_delim(c_int::from(b' ')), 0);
        assert_eq!(rs_sign_is_arg_delim(c_int::from(b':')), 0);
    }
}
