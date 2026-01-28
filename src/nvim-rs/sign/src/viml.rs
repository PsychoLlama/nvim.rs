//! VimL function support for signs
//!
//! This module provides structures and helpers for the VimL sign_*() functions:
//! - sign_define()
//! - sign_getdefined()
//! - sign_getplaced()
//! - sign_jump()
//! - sign_place()
//! - sign_placelist()
//! - sign_undefine()
//! - sign_unplace()
//! - sign_unplacelist()

use std::ffi::{c_char, c_int};

use crate::{LinenrT, SignBufHandle, SignHandle, SIGN_DEF_PRIO};

// =============================================================================
// C FFI declarations
// =============================================================================

extern "C" {
    /// Get sign by name from the sign map
    fn nvim_sign_map_get(name: *const c_char) -> SignHandle;
}

// =============================================================================
// VimL Function Result Types
// =============================================================================

/// Result from sign_define() function.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignDefineResult {
    /// Success (returns 0)
    Success = 0,
    /// Invalid argument
    InvalidArg = -1,
    /// Sign already exists (update may have occurred)
    Updated = 1,
}

/// Result from sign_place() function.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignPlaceResultCode {
    /// Success - returns sign ID
    Success = 0,
    /// Invalid argument
    InvalidArg = -1,
    /// Sign not defined
    NotDefined = -2,
    /// Buffer not found
    NoBuffer = -3,
    /// Invalid line number
    InvalidLine = -4,
}

/// Result from sign_unplace() function.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignUnplaceResult {
    /// Success
    Success = 0,
    /// Sign not found
    NotFound = -1,
    /// Invalid argument
    InvalidArg = -2,
}

// =============================================================================
// sign_define() Support
// =============================================================================

/// Parameters for sign_define() VimL function.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignDefineVimlParams {
    /// Sign name (required)
    pub name: *const c_char,
    /// Icon path (from 'icon' key)
    pub icon: *const c_char,
    /// Sign text (from 'text' key)
    pub text: *const c_char,
    /// Line highlight group (from 'linehl' key)
    pub linehl: *const c_char,
    /// Text highlight group (from 'texthl' key)
    pub texthl: *const c_char,
    /// Cursorline highlight (from 'culhl' key)
    pub culhl: *const c_char,
    /// Number column highlight (from 'numhl' key)
    pub numhl: *const c_char,
    /// Priority (from 'priority' key, -1 for unset)
    pub priority: c_int,
}

impl Default for SignDefineVimlParams {
    fn default() -> Self {
        Self {
            name: std::ptr::null(),
            icon: std::ptr::null(),
            text: std::ptr::null(),
            linehl: std::ptr::null(),
            texthl: std::ptr::null(),
            culhl: std::ptr::null(),
            numhl: std::ptr::null(),
            priority: -1,
        }
    }
}

/// Create default sign_define params.
#[no_mangle]
pub extern "C" fn rs_sign_define_viml_params_default() -> SignDefineVimlParams {
    SignDefineVimlParams::default()
}

/// Check if sign_define params are valid.
///
/// # Safety
/// All string pointers must be null or valid C strings.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_define_viml_params_valid(
    params: *const SignDefineVimlParams,
) -> bool {
    if params.is_null() {
        return false;
    }
    let p = &*params;
    // Name is required and must not be empty
    !p.name.is_null() && *p.name.cast::<u8>() != 0
}

// =============================================================================
// sign_place() Support
// =============================================================================

/// Parameters for sign_place() VimL function.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignPlaceVimlParams {
    /// Sign ID (0 for auto-generate)
    pub id: c_int,
    /// Sign group (null for global)
    pub group: *const c_char,
    /// Sign name
    pub name: *const c_char,
    /// Buffer handle
    pub buf: SignBufHandle,
    /// Line number (from 'lnum' key)
    pub lnum: LinenrT,
    /// Priority (from 'priority' key, -1 for default)
    pub priority: c_int,
}

impl Default for SignPlaceVimlParams {
    fn default() -> Self {
        Self {
            id: 0,
            group: std::ptr::null(),
            name: std::ptr::null(),
            buf: SignBufHandle::null(),
            lnum: 0,
            priority: -1,
        }
    }
}

/// Create default sign_place params.
#[no_mangle]
pub extern "C" fn rs_sign_place_viml_params_default() -> SignPlaceVimlParams {
    SignPlaceVimlParams::default()
}

/// Validate sign_place params.
///
/// Returns 0 if valid, or a negative error code.
///
/// # Safety
/// All pointers must be null or valid.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_place_viml_validate(
    params: *const SignPlaceVimlParams,
) -> SignPlaceResultCode {
    if params.is_null() {
        return SignPlaceResultCode::InvalidArg;
    }

    let p = &*params;

    // ID must be >= 0
    if p.id < 0 {
        return SignPlaceResultCode::InvalidArg;
    }

    // Name is required
    if p.name.is_null() || *p.name.cast::<u8>() == 0 {
        return SignPlaceResultCode::InvalidArg;
    }

    // Sign must be defined
    let sp = nvim_sign_map_get(p.name);
    if sp.is_null() {
        return SignPlaceResultCode::NotDefined;
    }

    // Buffer is required
    if p.buf.is_null() {
        return SignPlaceResultCode::NoBuffer;
    }

    // Line number must be valid for new placements
    if p.id == 0 && p.lnum <= 0 {
        return SignPlaceResultCode::InvalidLine;
    }

    SignPlaceResultCode::Success
}

/// Get effective priority for sign_place.
#[no_mangle]
pub extern "C" fn rs_sign_place_viml_priority(prio: c_int, def_prio: c_int) -> c_int {
    if prio >= 0 {
        prio
    } else if def_prio >= 0 {
        def_prio
    } else {
        SIGN_DEF_PRIO
    }
}

// =============================================================================
// sign_getplaced() Support
// =============================================================================

/// Filter parameters for sign_getplaced() function.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignGetPlacedFilter {
    /// Buffer to query (null for all buffers)
    pub buf: SignBufHandle,
    /// Line number filter (0 for all lines)
    pub lnum: LinenrT,
    /// Sign ID filter (0 for all signs)
    pub id: c_int,
    /// Group filter (null for global, "*" for all)
    pub group: *const c_char,
}

impl Default for SignGetPlacedFilter {
    fn default() -> Self {
        Self {
            buf: SignBufHandle::null(),
            lnum: 0,
            id: 0,
            group: std::ptr::null(),
        }
    }
}

/// Create default sign_getplaced filter.
#[no_mangle]
pub extern "C" fn rs_sign_get_placed_filter_default() -> SignGetPlacedFilter {
    SignGetPlacedFilter::default()
}

/// Check if filter specifies all signs (no restrictions).
#[no_mangle]
pub extern "C" fn rs_sign_get_placed_filter_is_all(filter: &SignGetPlacedFilter) -> bool {
    filter.buf.is_null() && filter.lnum == 0 && filter.id == 0 && filter.group.is_null()
}

/// Check if filter restricts to specific buffer.
#[no_mangle]
pub extern "C" fn rs_sign_get_placed_filter_has_buf(filter: &SignGetPlacedFilter) -> bool {
    !filter.buf.is_null()
}

// =============================================================================
// sign_unplace() Support
// =============================================================================

/// Parameters for sign_unplace() VimL function.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignUnplaceVimlParams {
    /// Sign group ("*" for all groups)
    pub group: *const c_char,
    /// Buffer handle (null for all buffers)
    pub buf: SignBufHandle,
    /// Sign ID (0 for all)
    pub id: c_int,
}

impl Default for SignUnplaceVimlParams {
    fn default() -> Self {
        Self {
            group: std::ptr::null(),
            buf: SignBufHandle::null(),
            id: 0,
        }
    }
}

/// Create default sign_unplace params.
#[no_mangle]
pub extern "C" fn rs_sign_unplace_viml_params_default() -> SignUnplaceVimlParams {
    SignUnplaceVimlParams::default()
}

/// Check if unplace params specify "all" groups ("*").
///
/// # Safety
/// `group` must be null or a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_unplace_viml_is_all_groups(
    params: *const SignUnplaceVimlParams,
) -> bool {
    if params.is_null() {
        return false;
    }
    let p = &*params;
    !p.group.is_null() && *p.group.cast::<u8>() == b'*'
}

// =============================================================================
// sign_jump() Support
// =============================================================================

/// Parameters for sign_jump() VimL function.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SignJumpVimlParams {
    /// Sign ID (required, must be > 0)
    pub id: c_int,
    /// Sign group (empty string means global)
    pub group: *const c_char,
    /// Buffer to search in
    pub buf: SignBufHandle,
}

impl Default for SignJumpVimlParams {
    fn default() -> Self {
        Self {
            id: 0,
            group: std::ptr::null(),
            buf: SignBufHandle::null(),
        }
    }
}

/// Create default sign_jump params.
#[no_mangle]
pub extern "C" fn rs_sign_jump_viml_params_default() -> SignJumpVimlParams {
    SignJumpVimlParams::default()
}

/// Validate sign_jump params.
///
/// # Safety
/// All pointers must be null or valid.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_jump_viml_validate(params: *const SignJumpVimlParams) -> bool {
    if params.is_null() {
        return false;
    }
    let p = &*params;

    // ID must be > 0
    if p.id <= 0 {
        return false;
    }

    // Buffer is required
    if p.buf.is_null() {
        return false;
    }

    true
}

// =============================================================================
// Return Value Helpers
// =============================================================================

/// Return value type for VimL sign functions.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignVimlReturnType {
    /// Return a number (sign ID or error code)
    Number = 0,
    /// Return a list
    List = 1,
    /// Return a dict
    Dict = 2,
}

/// Determine return type for sign_define().
#[no_mangle]
pub extern "C" fn rs_sign_define_return_type(is_list_arg: c_int) -> SignVimlReturnType {
    if is_list_arg != 0 {
        SignVimlReturnType::List
    } else {
        SignVimlReturnType::Number
    }
}

/// Determine return type for sign_place().
#[no_mangle]
pub extern "C" fn rs_sign_place_return_type(is_list_mode: c_int) -> SignVimlReturnType {
    if is_list_mode != 0 {
        SignVimlReturnType::List
    } else {
        SignVimlReturnType::Number
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_define_viml_params_default() {
        let params = SignDefineVimlParams::default();
        assert!(params.name.is_null());
        assert!(params.icon.is_null());
        assert!(params.text.is_null());
        assert_eq!(params.priority, -1);
    }

    #[test]
    fn test_sign_place_viml_params_default() {
        let params = SignPlaceVimlParams::default();
        assert_eq!(params.id, 0);
        assert!(params.group.is_null());
        assert!(params.name.is_null());
        assert!(params.buf.is_null());
        assert_eq!(params.lnum, 0);
        assert_eq!(params.priority, -1);
    }

    #[test]
    fn test_sign_place_viml_priority() {
        // Explicit priority takes precedence
        assert_eq!(rs_sign_place_viml_priority(5, 10), 5);
        // Fall back to definition priority
        assert_eq!(rs_sign_place_viml_priority(-1, 10), 10);
        // Fall back to default
        assert_eq!(rs_sign_place_viml_priority(-1, -1), SIGN_DEF_PRIO);
    }

    #[test]
    fn test_sign_get_placed_filter_default() {
        let filter = SignGetPlacedFilter::default();
        assert!(filter.buf.is_null());
        assert_eq!(filter.lnum, 0);
        assert_eq!(filter.id, 0);
        assert!(filter.group.is_null());
    }

    #[test]
    fn test_sign_get_placed_filter_is_all() {
        let filter = SignGetPlacedFilter::default();
        assert!(rs_sign_get_placed_filter_is_all(&filter));
    }

    #[test]
    fn test_sign_unplace_viml_params_default() {
        let params = SignUnplaceVimlParams::default();
        assert!(params.group.is_null());
        assert!(params.buf.is_null());
        assert_eq!(params.id, 0);
    }

    #[test]
    fn test_sign_jump_viml_params_default() {
        let params = SignJumpVimlParams::default();
        assert_eq!(params.id, 0);
        assert!(params.group.is_null());
        assert!(params.buf.is_null());
    }

    #[test]
    fn test_sign_define_return_type() {
        assert_eq!(rs_sign_define_return_type(0), SignVimlReturnType::Number);
        assert_eq!(rs_sign_define_return_type(1), SignVimlReturnType::List);
    }

    #[test]
    fn test_sign_place_return_type() {
        assert_eq!(rs_sign_place_return_type(0), SignVimlReturnType::Number);
        assert_eq!(rs_sign_place_return_type(1), SignVimlReturnType::List);
    }

    #[test]
    fn test_sign_define_result_values() {
        assert_eq!(SignDefineResult::Success as c_int, 0);
        assert_eq!(SignDefineResult::InvalidArg as c_int, -1);
        assert_eq!(SignDefineResult::Updated as c_int, 1);
    }

    #[test]
    fn test_sign_place_result_code_values() {
        assert_eq!(SignPlaceResultCode::Success as c_int, 0);
        assert_eq!(SignPlaceResultCode::InvalidArg as c_int, -1);
        assert_eq!(SignPlaceResultCode::NotDefined as c_int, -2);
        assert_eq!(SignPlaceResultCode::NoBuffer as c_int, -3);
        assert_eq!(SignPlaceResultCode::InvalidLine as c_int, -4);
    }

    #[test]
    fn test_sign_unplace_result_values() {
        assert_eq!(SignUnplaceResult::Success as c_int, 0);
        assert_eq!(SignUnplaceResult::NotFound as c_int, -1);
        assert_eq!(SignUnplaceResult::InvalidArg as c_int, -2);
    }
}
