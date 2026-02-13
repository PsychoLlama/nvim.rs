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
    fn nvim_sign_create_namespace_cstr(name: *const c_char) -> c_int;
    fn nvim_sign_namespace_exists(name: *const c_char) -> c_int;

    // Composite sign operations
    fn nvim_sign_build_decor_and_set(
        buf: SignBufHandle,
        ns: u32,
        id: *mut u32,
        row: c_int,
        sp: SignHandle,
        prio: c_int,
    );
    fn nvim_sign_marktree_lookup_row(buf: SignBufHandle, ns: u32, id: u32) -> LinenrT;
    fn nvim_sign_buf_line_count(buf: SignBufHandle) -> LinenrT;
    fn nvim_sign_ns_push(ns: i64);

    // Namespace filtering
    fn rs_group_get_ns(
        group: *const c_char,
        ns_lookup: extern "C" fn(*const c_char) -> c_int,
    ) -> i64;
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
    nvim_sign_create_namespace_cstr(group)
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
// Sign Placement Parameters
// =============================================================================

/// Parameters for placing a sign.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignPlaceParams {
    /// Buffer handle
    pub buf: SignBufHandle,
    /// Sign ID (0 for auto-assign)
    pub id: u32,
    /// Sign group (null for global)
    pub group: *const c_char,
    /// Sign name
    pub name: *const c_char,
    /// Line number
    pub lnum: LinenrT,
    /// Priority (-1 for default)
    pub priority: c_int,
}

impl Default for SignPlaceParams {
    fn default() -> Self {
        Self {
            buf: SignBufHandle::null(),
            id: 0,
            group: std::ptr::null(),
            name: std::ptr::null(),
            lnum: 0,
            priority: -1,
        }
    }
}

/// FFI export: Create default sign place params.
#[no_mangle]
pub extern "C" fn rs_sign_place_params_default() -> SignPlaceParams {
    SignPlaceParams::default()
}

/// Sign placement result.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignPlaceResult {
    /// Success
    Ok = 0,
    /// Sign name not specified
    NoName = 1,
    /// Sign is not defined
    NotDefined = 2,
    /// Buffer is invalid
    InvalidBuffer = 3,
    /// Line number is invalid
    InvalidLine = 4,
    /// Group name is invalid
    InvalidGroup = 5,
    /// Sign ID is invalid
    InvalidId = 6,
}

// =============================================================================
// Sign Deletion Parameters
// =============================================================================

/// Parameters for deleting signs.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignDeleteParams {
    /// Buffer handle (null for all buffers)
    pub buf: SignBufHandle,
    /// Sign ID (0 for all signs)
    pub id: c_int,
    /// Sign group (null for global)
    pub group: *const c_char,
    /// Line number (-1 for any line)
    pub lnum: LinenrT,
}

impl Default for SignDeleteParams {
    fn default() -> Self {
        Self {
            buf: SignBufHandle::null(),
            id: 0,
            group: std::ptr::null(),
            lnum: -1,
        }
    }
}

/// FFI export: Create default sign delete params.
#[no_mangle]
pub extern "C" fn rs_sign_delete_params_default() -> SignDeleteParams {
    SignDeleteParams::default()
}

/// Sign deletion scope.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignDeleteScope {
    /// Delete specific sign by ID
    ById = 0,
    /// Delete all signs on a line
    ByLine = 1,
    /// Delete all signs in a group
    ByGroup = 2,
    /// Delete all signs in buffer
    All = 3,
}

/// Determine the deletion scope from parameters.
#[no_mangle]
pub extern "C" fn rs_sign_delete_scope(
    id: c_int,
    lnum: LinenrT,
    group_is_all: c_int,
) -> SignDeleteScope {
    if id > 0 {
        SignDeleteScope::ById
    } else if lnum > 0 {
        SignDeleteScope::ByLine
    } else if group_is_all != 0 {
        SignDeleteScope::All
    } else {
        SignDeleteScope::ByGroup
    }
}

// =============================================================================
// Sign Location Query
// =============================================================================

/// Sign location result.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SignLocation {
    /// Line number (0 if not found)
    pub lnum: LinenrT,
    /// Whether the sign was found
    pub found: bool,
}

/// FFI export: Create a sign location for a found sign.
#[no_mangle]
pub extern "C" fn rs_sign_location_found(lnum: LinenrT) -> SignLocation {
    SignLocation { lnum, found: true }
}

/// FFI export: Create a sign location for a not-found sign.
#[no_mangle]
pub extern "C" fn rs_sign_location_not_found() -> SignLocation {
    SignLocation::default()
}

// =============================================================================
// Extmark Decoration Flags
// =============================================================================

/// Marktree flag for sign text decoration.
pub const MT_FLAG_DECOR_SIGNTEXT: u16 = 0x0040;

/// Marktree flag for sign highlight decoration.
pub const MT_FLAG_DECOR_SIGNHL: u16 = 0x0080;

/// Calculate decoration flags for a sign.
///
/// Returns the appropriate MT_FLAG_DECOR_* flags based on sign properties.
#[no_mangle]
pub extern "C" fn rs_sign_calc_decor_flags(
    has_text: c_int,
    has_line_hl: c_int,
    has_num_hl: c_int,
    has_cul_hl: c_int,
) -> u16 {
    let mut flags: u16 = 0;

    if has_text != 0 {
        flags |= MT_FLAG_DECOR_SIGNTEXT;
    }

    if has_line_hl != 0 || has_num_hl != 0 || has_cul_hl != 0 {
        flags |= MT_FLAG_DECOR_SIGNHL;
    }

    flags
}

// =============================================================================
// Core Sign Placement Operations
// =============================================================================

/// Callback used by rs_group_get_ns for namespace lookup.
extern "C" fn namespace_lookup_fn(name: *const c_char) -> c_int {
    unsafe { nvim_namespace_lookup(name) }
}

/// Create or update a sign extmark.
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_set_sign(
    buf: SignBufHandle,
    id: *mut u32,
    group: *const c_char,
    prio: c_int,
    lnum: LinenrT,
    sp: SignHandle,
) {
    // If group is non-null and namespace doesn't exist yet, register it
    if !group.is_null() && nvim_sign_namespace_exists(group) == 0 {
        let ns = nvim_sign_create_namespace_cstr(group);
        nvim_sign_ns_push(i64::from(ns));
    }

    // Resolve namespace
    let ns: u32 = if group.is_null() {
        0
    } else {
        #[allow(clippy::cast_sign_loss)]
        let ns = nvim_sign_create_namespace_cstr(group) as u32;
        ns
    };

    // Clamp lnum to buffer range and convert to 0-based row
    let line_count = nvim_sign_buf_line_count(buf);
    let clamped = if lnum > line_count { line_count } else { lnum };
    let row = clamped - 1;

    nvim_sign_build_decor_and_set(buf, ns, id, row, sp, prio);
}

/// Modify an existing placed sign. Returns the 1-based line number, or 0 if not found.
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_buf_mod_sign(
    buf: SignBufHandle,
    id: *mut u32,
    group: *const c_char,
    prio: c_int,
    sp: SignHandle,
) -> LinenrT {
    let ns = rs_group_get_ns(group, namespace_lookup_fn);
    if ns < 0 || (!group.is_null() && ns == 0) {
        return 0;
    }

    #[allow(clippy::cast_sign_loss)]
    let mark_lnum = nvim_sign_marktree_lookup_row(buf, ns as u32, *id);
    if mark_lnum > 0 {
        // mark_lnum is already 1-based from the accessor
        rs_buf_set_sign(buf, id, group, prio, mark_lnum, sp);
    }
    mark_lnum
}

/// Find the line number of a placed sign. Returns 1-based line number, or 0 if not found.
///
/// # Safety
/// All pointer parameters must be valid.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_buf_findsign(
    buf: SignBufHandle,
    id: c_int,
    group: *const c_char,
) -> c_int {
    let ns = rs_group_get_ns(group, namespace_lookup_fn);
    if ns < 0 || (!group.is_null() && ns == 0) {
        return 0;
    }

    #[allow(clippy::cast_sign_loss)]
    nvim_sign_marktree_lookup_row(buf, ns as u32, id as u32)
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

    #[test]
    fn test_sign_place_params_default() {
        let params = SignPlaceParams::default();
        assert!(params.buf.is_null());
        assert_eq!(params.id, 0);
        assert!(params.group.is_null());
        assert!(params.name.is_null());
        assert_eq!(params.lnum, 0);
        assert_eq!(params.priority, -1);
    }

    #[test]
    fn test_sign_delete_params_default() {
        let params = SignDeleteParams::default();
        assert!(params.buf.is_null());
        assert_eq!(params.id, 0);
        assert!(params.group.is_null());
        assert_eq!(params.lnum, -1);
    }

    #[test]
    fn test_sign_delete_scope() {
        assert_eq!(rs_sign_delete_scope(5, 0, 0), SignDeleteScope::ById);
        assert_eq!(rs_sign_delete_scope(0, 10, 0), SignDeleteScope::ByLine);
        assert_eq!(rs_sign_delete_scope(0, 0, 1), SignDeleteScope::All);
        assert_eq!(rs_sign_delete_scope(0, 0, 0), SignDeleteScope::ByGroup);
    }

    #[test]
    fn test_sign_location() {
        let found = rs_sign_location_found(42);
        assert!(found.found);
        assert_eq!(found.lnum, 42);

        let not_found = rs_sign_location_not_found();
        assert!(!not_found.found);
        assert_eq!(not_found.lnum, 0);
    }

    #[test]
    fn test_sign_calc_decor_flags() {
        assert_eq!(rs_sign_calc_decor_flags(0, 0, 0, 0), 0);
        assert_eq!(rs_sign_calc_decor_flags(1, 0, 0, 0), MT_FLAG_DECOR_SIGNTEXT);
        assert_eq!(rs_sign_calc_decor_flags(0, 1, 0, 0), MT_FLAG_DECOR_SIGNHL);
        assert_eq!(rs_sign_calc_decor_flags(0, 0, 1, 0), MT_FLAG_DECOR_SIGNHL);
        assert_eq!(rs_sign_calc_decor_flags(0, 0, 0, 1), MT_FLAG_DECOR_SIGNHL);
        assert_eq!(
            rs_sign_calc_decor_flags(1, 1, 0, 0),
            MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL
        );
    }

    #[test]
    fn test_sign_place_result() {
        assert_eq!(SignPlaceResult::Ok as c_int, 0);
        assert_eq!(SignPlaceResult::NoName as c_int, 1);
        assert_eq!(SignPlaceResult::NotDefined as c_int, 2);
    }
}
