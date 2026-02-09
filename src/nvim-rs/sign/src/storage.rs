//! Sign map storage management
//!
//! This module provides sign map management utilities including:
//! - Sign definition lookup and enumeration
//! - Namespace registry management
//! - Sign map iteration support

use std::ffi::{c_char, c_int, c_void, CStr};

use crate::SignHandle;

// =============================================================================
// C FFI declarations
// =============================================================================

#[allow(dead_code)]
extern "C" {
    /// Get sign by name from the sign map (also declared in define.rs)
    fn nvim_sign_map_get(name: *const c_char) -> SignHandle;

    /// Check if sign exists in the map
    fn nvim_sign_map_has(name: *const c_char) -> c_int;

    /// Get sign name from sign handle
    fn nvim_sign_get_name(sp: SignHandle) -> *const c_char;

    /// Get sign icon path
    fn nvim_sign_get_icon(sp: SignHandle) -> *const c_char;

    /// Get sign priority
    fn nvim_sign_get_priority(sp: SignHandle) -> c_int;

    /// Get sign text highlight ID
    fn nvim_sign_get_text_hl(sp: SignHandle) -> c_int;

    /// Get sign line highlight ID
    fn nvim_sign_get_line_hl(sp: SignHandle) -> c_int;

    /// Get sign number highlight ID
    fn nvim_sign_get_num_hl(sp: SignHandle) -> c_int;

    /// Get sign cursorline highlight ID
    fn nvim_sign_get_cul_hl(sp: SignHandle) -> c_int;

    /// Namespace lookup by name
    fn nvim_namespace_lookup(name: *const c_char) -> c_int;

    /// Create or get namespace by name
    fn nvim_create_namespace(name: *const c_char) -> c_int;

    /// Describe namespace by ID
    fn nvim_describe_ns(ns: c_int, empty: *const c_char) -> *const c_char;
}

// =============================================================================
// Sign Lookup
// =============================================================================

// Note: rs_sign_get_by_name is defined in define.rs

/// Check if a sign with the given name exists.
///
/// # Safety
/// `name` must be a valid null-terminated C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_exists(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }
    nvim_sign_map_has(name)
}

// =============================================================================
// Sign Properties Access
// =============================================================================

/// Sign properties bundle for efficient access.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignProperties {
    /// Sign name (may be null)
    pub name: *const c_char,
    /// Icon path (may be null)
    pub icon: *const c_char,
    /// Default priority (-1 for default)
    pub priority: c_int,
    /// Text highlight ID
    pub text_hl: c_int,
    /// Line highlight ID
    pub line_hl: c_int,
    /// Number column highlight ID
    pub num_hl: c_int,
    /// Cursorline highlight ID
    pub cul_hl: c_int,
}

impl Default for SignProperties {
    fn default() -> Self {
        Self {
            name: std::ptr::null(),
            icon: std::ptr::null(),
            priority: -1,
            text_hl: 0,
            line_hl: 0,
            num_hl: 0,
            cul_hl: 0,
        }
    }
}

/// Get all properties of a sign at once.
///
/// More efficient than calling individual getters when multiple
/// properties are needed.
///
/// # Safety
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_get_properties(sp: SignHandle) -> SignProperties {
    if sp.is_null() {
        return SignProperties::default();
    }

    SignProperties {
        name: nvim_sign_get_name(sp),
        icon: nvim_sign_get_icon(sp),
        priority: nvim_sign_get_priority(sp),
        text_hl: nvim_sign_get_text_hl(sp),
        line_hl: nvim_sign_get_line_hl(sp),
        num_hl: nvim_sign_get_num_hl(sp),
        cul_hl: nvim_sign_get_cul_hl(sp),
    }
}

/// Check if a sign has any highlight defined.
///
/// # Safety
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_has_highlight(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }

    let props = rs_sign_get_properties(sp);
    c_int::from(props.text_hl != 0 || props.line_hl != 0 || props.num_hl != 0 || props.cul_hl != 0)
}

/// Check if a sign has an icon defined.
///
/// # Safety
/// `sp` must be a valid sign handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_has_icon(sp: SignHandle) -> c_int {
    if sp.is_null() {
        return 0;
    }
    let icon = nvim_sign_get_icon(sp);
    c_int::from(!icon.is_null() && *icon != 0)
}

// =============================================================================
// Namespace Registry
// =============================================================================

/// Namespace information for sign operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignNamespace {
    /// Namespace ID (0 for global, -1 for invalid)
    pub id: c_int,
    /// Whether this is the global namespace
    pub is_global: bool,
    /// Whether this matches all namespaces
    pub is_all: bool,
    /// Whether this is an invalid/non-existing namespace
    pub is_invalid: bool,
}

impl SignNamespace {
    /// Global namespace (ns = 0)
    pub const GLOBAL: Self = Self {
        id: 0,
        is_global: true,
        is_all: false,
        is_invalid: false,
    };

    /// All namespaces sentinel
    pub const ALL: Self = Self {
        id: -1,
        is_global: false,
        is_all: true,
        is_invalid: false,
    };

    /// Invalid namespace
    pub const INVALID: Self = Self {
        id: -1,
        is_global: false,
        is_all: false,
        is_invalid: true,
    };
}

impl Default for SignNamespace {
    fn default() -> Self {
        Self::GLOBAL
    }
}

/// Parse a group name into a namespace.
///
/// Returns:
/// - Global namespace for NULL group
/// - All namespaces for "*" group
/// - Specific namespace for a named group
/// - Invalid namespace for non-existing group
///
/// # Safety
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_parse_namespace(group: *const c_char) -> SignNamespace {
    if group.is_null() {
        return SignNamespace::GLOBAL;
    }

    let group_cstr = CStr::from_ptr(group);
    let group_bytes = group_cstr.to_bytes();

    // Check for "*" (all namespaces)
    if group_bytes == b"*" {
        return SignNamespace::ALL;
    }

    // Look up the namespace
    let ns = nvim_namespace_lookup(group);
    if ns != 0 {
        SignNamespace {
            id: ns,
            is_global: false,
            is_all: false,
            is_invalid: false,
        }
    } else {
        SignNamespace::INVALID
    }
}

/// Get or create a namespace for the given group name.
///
/// Unlike `rs_sign_parse_namespace`, this will create the namespace
/// if it doesn't exist.
///
/// # Safety
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_get_or_create_namespace(group: *const c_char) -> c_int {
    if group.is_null() {
        return 0; // Global namespace
    }

    let group_cstr = CStr::from_ptr(group);
    if group_cstr.to_bytes() == b"*" {
        return -1; // Can't create "all namespaces"
    }

    nvim_create_namespace(group)
}

/// Empty string constant for C interop.
static EMPTY_CSTR: &[u8] = b"\0";

/// Get the description/name for a namespace ID.
///
/// # Safety
/// `buf` must be valid for writing at least `buflen` bytes.
#[no_mangle]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_sign_describe_namespace(
    ns: c_int,
    buf: *mut c_char,
    buflen: usize,
) -> c_int {
    if buf.is_null() || buflen == 0 {
        return 0;
    }

    let desc = nvim_describe_ns(ns, EMPTY_CSTR.as_ptr().cast());

    if desc.is_null() {
        *buf = 0;
        return 0;
    }

    let desc_cstr = CStr::from_ptr(desc);
    let desc_bytes = desc_cstr.to_bytes();
    let copy_len = desc_bytes.len().min(buflen - 1);

    std::ptr::copy_nonoverlapping(desc_bytes.as_ptr(), buf.cast(), copy_len);
    *buf.add(copy_len) = 0;

    copy_len as c_int
}

// =============================================================================
// Sign Map Iteration
// =============================================================================

/// Callback type for sign map iteration.
///
/// Called for each sign in the map. Return non-zero to stop iteration.
pub type SignMapCallback = unsafe extern "C" fn(sp: SignHandle, user_data: *mut c_void) -> c_int;

/// Callback type for sign name iteration.
///
/// Called for each sign name in the map. Return non-zero to stop iteration.
pub type SignNameCallback =
    unsafe extern "C" fn(name: *const c_char, user_data: *mut c_void) -> c_int;

/// Sign map iterator state.
///
/// This is used internally for C-side iteration support.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignMapIterator {
    /// Internal iteration index
    pub index: c_int,
    /// Whether iteration is complete
    pub done: bool,
}

impl Default for SignMapIterator {
    fn default() -> Self {
        Self::new()
    }
}

impl SignMapIterator {
    /// Create a new iterator at the start.
    pub const fn new() -> Self {
        Self {
            index: 0,
            done: false,
        }
    }

    /// Check if iteration is complete.
    pub const fn is_done(&self) -> bool {
        self.done
    }
}

/// FFI export: Create a new sign map iterator.
#[no_mangle]
pub extern "C" fn rs_sign_map_iter_new() -> SignMapIterator {
    SignMapIterator::new()
}

/// FFI export: Check if iterator is done.
///
/// # Safety
/// `iter` must be null or a valid pointer to a `SignMapIterator`.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_map_iter_done(iter: *const SignMapIterator) -> c_int {
    if iter.is_null() {
        return 1;
    }
    c_int::from((*iter).done)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_properties_default() {
        let props = SignProperties::default();
        assert!(props.name.is_null());
        assert!(props.icon.is_null());
        assert_eq!(props.priority, -1);
        assert_eq!(props.text_hl, 0);
        assert_eq!(props.line_hl, 0);
        assert_eq!(props.num_hl, 0);
        assert_eq!(props.cul_hl, 0);
    }

    #[test]
    #[allow(clippy::assertions_on_constants)]
    fn test_sign_namespace_constants() {
        // Test GLOBAL namespace
        assert!(SignNamespace::GLOBAL.is_global);
        assert!(!SignNamespace::GLOBAL.is_all);
        assert!(!SignNamespace::GLOBAL.is_invalid);
        assert_eq!(SignNamespace::GLOBAL.id, 0);

        // Test ALL namespace
        assert!(!SignNamespace::ALL.is_global);
        assert!(SignNamespace::ALL.is_all);
        assert!(!SignNamespace::ALL.is_invalid);

        // Test INVALID namespace
        assert!(!SignNamespace::INVALID.is_global);
        assert!(!SignNamespace::INVALID.is_all);
        assert!(SignNamespace::INVALID.is_invalid);
    }

    #[test]
    fn test_sign_namespace_default() {
        let ns = SignNamespace::default();
        assert!(ns.is_global);
        assert_eq!(ns.id, 0);
    }

    #[test]
    fn test_sign_map_iterator() {
        let iter = SignMapIterator::new();
        assert_eq!(iter.index, 0);
        assert!(!iter.done);
    }
}
