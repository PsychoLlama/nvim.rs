//! Function reference types.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

// =============================================================================
// Funcref Types
// =============================================================================

/// Type of function reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FuncrefType {
    /// Reference to named function
    Named = 0,
    /// Reference to anonymous/lambda function
    Lambda = 1,
    /// Reference to built-in function
    Builtin = 2,
    /// Partial application (funcref with bound args)
    Partial = 3,
}

impl FuncrefType {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Lambda,
            2 => Self::Builtin,
            3 => Self::Partial,
            _ => Self::Named,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Funcref Info
// =============================================================================

/// Information about a function reference.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FuncrefInfo {
    /// Type of funcref
    pub ref_type: i32,
    /// Reference count (for memory management)
    pub refcount: i32,
    /// Whether funcref is still valid
    pub is_valid: bool,
    /// Has bound self/dict
    pub has_self: bool,
}

impl FuncrefInfo {
    /// Create a new funcref info.
    pub const fn new(ref_type: FuncrefType) -> Self {
        Self {
            ref_type: ref_type as i32,
            refcount: 1,
            is_valid: true,
            has_self: false,
        }
    }

    /// Create a partial funcref info.
    pub const fn partial(has_self: bool) -> Self {
        Self {
            ref_type: FuncrefType::Partial as i32,
            refcount: 1,
            is_valid: true,
            has_self,
        }
    }

    /// Check if this is a partial application.
    pub const fn is_partial(&self) -> bool {
        self.ref_type == FuncrefType::Partial as i32
    }

    /// Check if this is a lambda.
    pub const fn is_lambda(&self) -> bool {
        self.ref_type == FuncrefType::Lambda as i32
    }
}

/// FFI export: create funcref info.
#[no_mangle]
pub extern "C" fn rs_funcref_info_new(ref_type: c_int) -> FuncrefInfo {
    FuncrefInfo::new(FuncrefType::from_c_int(ref_type))
}

/// FFI export: check if funcref is partial.
#[no_mangle]
pub extern "C" fn rs_funcref_is_partial(ref_type: c_int) -> bool {
    ref_type == FuncrefType::Partial as c_int
}

// =============================================================================
// Function Name Parsing
// =============================================================================

/// Parse funcref name to determine type.
///
/// Returns the funcref type based on naming convention:
/// - Names starting with '<lambda>' are lambdas
/// - Names starting with uppercase or containing ':' are named functions
pub fn parse_funcref_type(name: &[u8]) -> FuncrefType {
    if name.is_empty() {
        return FuncrefType::Named;
    }

    // Check for lambda prefix
    if name.starts_with(b"<lambda>") {
        return FuncrefType::Lambda;
    }

    // Check for dict function marker
    if name.ends_with(b"()") {
        return FuncrefType::Partial;
    }

    FuncrefType::Named
}

/// FFI export: parse funcref type from name.
///
/// # Safety
/// - `name` must be a valid pointer to `len` bytes, or null.
#[no_mangle]
pub unsafe extern "C" fn rs_funcref_parse_type(name: *const u8, len: c_int) -> c_int {
    if name.is_null() || len < 0 {
        return FuncrefType::Named as c_int;
    }

    let slice = unsafe { std::slice::from_raw_parts(name, len as usize) };
    parse_funcref_type(slice).to_c_int()
}

// =============================================================================
// Bound Arguments
// =============================================================================

/// Maximum number of bound arguments in a partial.
pub const MAX_BOUND_ARGS: usize = 20;

/// Information about bound arguments in a partial.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct BoundArgsInfo {
    /// Number of bound arguments
    pub count: i32,
    /// Position where bound args are inserted (0 = prepend)
    pub insert_pos: i32,
}

impl BoundArgsInfo {
    /// Create new bound args info.
    pub const fn new(count: i32, insert_pos: i32) -> Self {
        Self { count, insert_pos }
    }

    /// Check if any args are bound.
    pub const fn has_bound_args(&self) -> bool {
        self.count > 0
    }

    /// Calculate total args when calling with additional args.
    pub const fn total_args(&self, call_args: i32) -> i32 {
        self.count + call_args
    }
}

/// FFI export: create bound args info.
#[no_mangle]
pub extern "C" fn rs_bound_args_new(count: c_int, insert_pos: c_int) -> BoundArgsInfo {
    BoundArgsInfo::new(count, insert_pos)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_funcref_type() {
        assert_eq!(FuncrefType::from_c_int(0), FuncrefType::Named);
        assert_eq!(FuncrefType::from_c_int(1), FuncrefType::Lambda);
        assert_eq!(FuncrefType::from_c_int(3), FuncrefType::Partial);
    }

    #[test]
    fn test_funcref_info() {
        let named = FuncrefInfo::new(FuncrefType::Named);
        assert!(!named.is_partial());
        assert!(!named.is_lambda());

        let partial = FuncrefInfo::partial(true);
        assert!(partial.is_partial());
        assert!(partial.has_self);
    }

    #[test]
    fn test_parse_funcref_type() {
        assert_eq!(parse_funcref_type(b"MyFunc"), FuncrefType::Named);
        assert_eq!(parse_funcref_type(b"<lambda>1"), FuncrefType::Lambda);
        assert_eq!(parse_funcref_type(b"obj.method()"), FuncrefType::Partial);
    }

    #[test]
    fn test_bound_args() {
        let info = BoundArgsInfo::new(2, 0);
        assert!(info.has_bound_args());
        assert_eq!(info.total_args(3), 5);

        let empty = BoundArgsInfo::new(0, 0);
        assert!(!empty.has_bound_args());
    }
}
