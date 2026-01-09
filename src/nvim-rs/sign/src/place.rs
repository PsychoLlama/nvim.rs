//! Sign placement operations
//!
//! This module handles placing signs at specific locations in buffers.
//! Signs are placed via extmarks and integrate with the decoration system.

use std::ffi::{c_char, c_int};

use crate::{LinenrT, SignBufHandle, SignHandle, SIGN_DEF_PRIO};

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

extern "C" {
    // Sign map lookup
    fn nvim_sign_map_get(name: *const c_char) -> SignHandle;

    // Sign properties
    fn nvim_sign_get_priority(sp: SignHandle) -> c_int;

    // Namespace operations
    fn nvim_namespace_lookup(name: *const c_char) -> c_int;
    fn nvim_create_namespace(name: *const c_char) -> c_int;
}

// =============================================================================
// Sign Placement Validation
// =============================================================================

/// Validate sign placement parameters.
///
/// Returns true if the parameters are valid for placing a sign.
///
/// # Parameters
///
/// - `id`: Sign ID (must be > 0 for modification, 0 for auto-assign)
/// - `group`: Sign group (null for global, non-empty for named group)
/// - `name`: Sign name (must be non-null and defined)
/// - `buf`: Buffer handle (must be valid)
/// - `lnum`: Line number (must be > 0 for placement, 0 for modification only)
///
/// # Safety
///
/// `group` and `name` must be null or valid null-terminated C strings.
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_place_validate(
    id: u32,
    group: *const c_char,
    name: *const c_char,
    buf: SignBufHandle,
    lnum: LinenrT,
) -> bool {
    // Name must be provided
    if name.is_null() {
        return false;
    }

    // Buffer must be valid
    if buf.is_null() {
        return false;
    }

    // Check for reserved character '*' in group name
    if !group.is_null() {
        let group_byte = *group.cast::<u8>();
        if group_byte == b'*' || group_byte == 0 {
            return false;
        }
    }

    // Sign must be defined
    let sp = nvim_sign_map_get(name);
    if sp.is_null() {
        return false;
    }

    // For modification (lnum == 0), ID must be specified
    if lnum == 0 && id == 0 {
        return false;
    }

    true
}

/// Get the effective priority for sign placement.
///
/// If `prio` is -1, uses the sign's default priority or SIGN_DEF_PRIO.
///
/// # Safety
///
/// `name` must be a valid null-terminated C string pointing to a defined sign.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_get_effective_priority(prio: c_int, name: *const c_char) -> c_int {
    if prio != -1 {
        return prio;
    }

    if name.is_null() {
        return SIGN_DEF_PRIO;
    }

    let sp = nvim_sign_map_get(name);
    if sp.is_null() {
        return SIGN_DEF_PRIO;
    }

    let sign_prio = nvim_sign_get_priority(sp);
    if sign_prio == -1 {
        SIGN_DEF_PRIO
    } else {
        sign_prio
    }
}

// =============================================================================
// Namespace Resolution
// =============================================================================

/// Resolve a group name to a namespace ID, creating if necessary.
///
/// Returns the namespace ID (>= 0) on success, -1 on error.
///
/// # Safety
///
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_resolve_namespace(group: *const c_char) -> c_int {
    if group.is_null() {
        return 0; // Global namespace
    }

    // Create or get the namespace
    nvim_create_namespace(group)
}

/// Check if a group name represents a valid namespace.
///
/// Returns true if the group is null (global) or represents an existing namespace.
///
/// # Safety
///
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_group_exists(group: *const c_char) -> bool {
    if group.is_null() {
        return true; // Global namespace always exists
    }

    let ns = nvim_namespace_lookup(group);
    ns != 0
}

// =============================================================================
// Sign ID Generation
// =============================================================================

/// Check if a sign ID is valid for placement.
///
/// Returns true if the ID is valid (>= 0).
#[no_mangle]
pub extern "C" fn rs_sign_id_valid(id: c_int) -> bool {
    id >= 0
}

/// Check if a sign ID should be auto-generated.
///
/// Returns true if ID is 0 (auto-assign).
#[no_mangle]
pub extern "C" fn rs_sign_id_is_auto(id: u32) -> bool {
    id == 0
}

// =============================================================================
// Line Number Validation
// =============================================================================

/// Clamp a line number to valid buffer range.
///
/// Ensures the line number is within [1, max_line].
#[no_mangle]
pub extern "C" fn rs_sign_clamp_lnum(lnum: LinenrT, max_line: LinenrT) -> LinenrT {
    if lnum < 1 {
        1
    } else if lnum > max_line {
        max_line
    } else {
        lnum
    }
}

/// Check if a line number is valid for sign placement.
///
/// Line numbers must be >= 1.
#[no_mangle]
pub extern "C" fn rs_sign_lnum_valid(lnum: LinenrT) -> bool {
    lnum >= 1
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_id_valid() {
        assert!(rs_sign_id_valid(0));
        assert!(rs_sign_id_valid(1));
        assert!(rs_sign_id_valid(100));
        assert!(!rs_sign_id_valid(-1));
    }

    #[test]
    fn test_sign_id_is_auto() {
        assert!(rs_sign_id_is_auto(0));
        assert!(!rs_sign_id_is_auto(1));
        assert!(!rs_sign_id_is_auto(100));
    }

    #[test]
    fn test_sign_clamp_lnum() {
        assert_eq!(rs_sign_clamp_lnum(0, 100), 1);
        assert_eq!(rs_sign_clamp_lnum(-5, 100), 1);
        assert_eq!(rs_sign_clamp_lnum(1, 100), 1);
        assert_eq!(rs_sign_clamp_lnum(50, 100), 50);
        assert_eq!(rs_sign_clamp_lnum(100, 100), 100);
        assert_eq!(rs_sign_clamp_lnum(150, 100), 100);
    }

    #[test]
    fn test_sign_lnum_valid() {
        assert!(!rs_sign_lnum_valid(0));
        assert!(!rs_sign_lnum_valid(-1));
        assert!(rs_sign_lnum_valid(1));
        assert!(rs_sign_lnum_valid(100));
    }
}
