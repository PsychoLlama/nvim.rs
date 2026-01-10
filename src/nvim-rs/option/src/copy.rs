//! Option copying between windows/buffers
//!
//! This module provides Rust implementations for copying option
//! values between different scopes (global, buffer-local, window-local).

use std::ffi::{c_char, c_int};

use crate::{OptInt, OptScope, OptValType};

// =============================================================================
// Copy Direction
// =============================================================================

/// Direction for option copying.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CopyDirection {
    /// Copy from global to local
    #[default]
    GlobalToLocal = 0,
    /// Copy from local to global
    LocalToGlobal = 1,
    /// Copy from buffer to window
    BufToWin = 2,
    /// Copy from window to buffer
    WinToBuf = 3,
}

impl CopyDirection {
    /// Create from C integer.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::LocalToGlobal,
            2 => Self::BufToWin,
            3 => Self::WinToBuf,
            _ => Self::GlobalToLocal,
        }
    }

    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Get source scope.
    #[must_use]
    pub const fn source_scope(self) -> OptScope {
        match self {
            Self::GlobalToLocal => OptScope::Global,
            Self::LocalToGlobal | Self::WinToBuf => OptScope::Win,
            Self::BufToWin => OptScope::Buf,
        }
    }

    /// Get destination scope.
    #[must_use]
    pub const fn dest_scope(self) -> OptScope {
        match self {
            Self::GlobalToLocal | Self::BufToWin => OptScope::Win,
            Self::LocalToGlobal => OptScope::Global,
            Self::WinToBuf => OptScope::Buf,
        }
    }
}

// =============================================================================
// Copy Context
// =============================================================================

/// Context for option copy operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CopyContext {
    /// Copy direction
    pub direction: c_int,
    /// Whether to copy all options
    pub all_options: bool,
    /// Whether to skip hidden options
    pub skip_hidden: bool,
    /// Whether to preserve modified flag
    pub keep_modified: bool,
    /// Option flags to filter
    pub flags_filter: u32,
}

impl CopyContext {
    /// Create a new copy context.
    #[must_use]
    pub const fn new(direction: CopyDirection) -> Self {
        Self {
            direction: direction.to_c_int(),
            all_options: true,
            skip_hidden: false,
            keep_modified: false,
            flags_filter: 0,
        }
    }

    /// Get copy direction.
    #[must_use]
    pub const fn get_direction(&self) -> CopyDirection {
        CopyDirection::from_c_int(self.direction)
    }

    /// Check if option should be copied based on flags.
    #[must_use]
    pub const fn should_copy(&self, opt_flags: u32) -> bool {
        if self.flags_filter == 0 {
            return true;
        }
        opt_flags & self.flags_filter != 0
    }
}

// =============================================================================
// Copy Result
// =============================================================================

/// Result of a copy operation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyResult {
    /// Copy succeeded
    Ok = 0,
    /// Source value not set
    NotSet = 1,
    /// Option doesn't support this scope
    InvalidScope = 2,
    /// Copy not allowed
    NotAllowed = 3,
    /// Generic failure
    Fail = 99,
}

impl CopyResult {
    /// Convert to C integer.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if result indicates success.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok)
    }
}

// =============================================================================
// Scope Support
// =============================================================================

/// Check if option supports a given scope.
///
/// # Arguments
/// * `opt_flags` - Option's scope flags
/// * `scope` - Scope to check
#[must_use]
pub const fn supports_scope(opt_flags: u32, scope: OptScope) -> bool {
    const GLOBAL: u32 = 0x01;
    const WIN: u32 = 0x02;
    const BUF: u32 = 0x04;

    match scope {
        OptScope::Global => opt_flags & GLOBAL != 0,
        OptScope::Win => opt_flags & WIN != 0,
        OptScope::Buf => opt_flags & BUF != 0,
    }
}

/// FFI: Check scope support.
#[no_mangle]
pub extern "C" fn rs_opt_supports_scope(opt_flags: u32, scope: c_int) -> c_int {
    let sc = match scope {
        0 => OptScope::Global,
        1 => OptScope::Win,
        2 => OptScope::Buf,
        _ => return 0,
    };
    c_int::from(supports_scope(opt_flags, sc))
}

/// Get default scope for an option.
///
/// Returns the most specific scope the option supports.
#[must_use]
pub const fn default_scope(opt_flags: u32) -> OptScope {
    const WIN: u32 = 0x02;
    const BUF: u32 = 0x04;

    if opt_flags & WIN != 0 {
        OptScope::Win
    } else if opt_flags & BUF != 0 {
        OptScope::Buf
    } else {
        OptScope::Global
    }
}

/// FFI: Get default scope.
#[no_mangle]
pub extern "C" fn rs_opt_default_scope(opt_flags: u32) -> c_int {
    default_scope(opt_flags) as c_int
}

// =============================================================================
// Value Comparison
// =============================================================================

/// Compare two boolean values.
#[no_mangle]
pub extern "C" fn rs_bool_values_equal(a: c_int, b: c_int) -> c_int {
    c_int::from((a != 0) == (b != 0))
}

/// Compare two number values.
#[no_mangle]
pub extern "C" fn rs_num_values_equal(a: OptInt, b: OptInt) -> c_int {
    c_int::from(a == b)
}

/// Compare two string values.
///
/// # Safety
/// Both pointers must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_str_values_equal(a: *const c_char, b: *const c_char) -> c_int {
    if a.is_null() && b.is_null() {
        return 1;
    }
    if a.is_null() || b.is_null() {
        return 0;
    }

    let mut pa = a;
    let mut pb = b;

    while *pa != 0 && *pb != 0 {
        if *pa != *pb {
            return 0;
        }
        pa = pa.add(1);
        pb = pb.add(1);
    }

    c_int::from(*pa == 0 && *pb == 0)
}

// =============================================================================
// Copy Operations
// =============================================================================

/// Information about an option value to copy.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CopyValueInfo {
    /// Option type
    pub opt_type: c_int,
    /// Boolean value (if applicable)
    pub bool_val: c_int,
    /// Number value (if applicable)
    pub num_val: OptInt,
    /// String value pointer (if applicable)
    pub str_val: *const c_char,
    /// Whether value is set
    pub is_set: bool,
}

/// FFI: Create copy value info for boolean.
#[no_mangle]
pub extern "C" fn rs_copy_value_bool(value: c_int, is_set: c_int) -> CopyValueInfo {
    CopyValueInfo {
        opt_type: OptValType::Boolean as c_int,
        bool_val: value,
        num_val: 0,
        str_val: std::ptr::null(),
        is_set: is_set != 0,
    }
}

/// FFI: Create copy value info for number.
#[no_mangle]
pub extern "C" fn rs_copy_value_num(value: OptInt, is_set: c_int) -> CopyValueInfo {
    CopyValueInfo {
        opt_type: OptValType::Number as c_int,
        bool_val: 0,
        num_val: value,
        str_val: std::ptr::null(),
        is_set: is_set != 0,
    }
}

/// FFI: Create copy value info for string.
#[no_mangle]
pub extern "C" fn rs_copy_value_str(value: *const c_char, is_set: c_int) -> CopyValueInfo {
    CopyValueInfo {
        opt_type: OptValType::String as c_int,
        bool_val: 0,
        num_val: 0,
        str_val: value,
        is_set: is_set != 0,
    }
}

// =============================================================================
// Window/Buffer Option Tracking
// =============================================================================

/// Track which local options have been explicitly set.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalOptFlags {
    /// Bitfield of set options (up to 64)
    pub set_bits: u64,
}

impl LocalOptFlags {
    /// Check if option at index is set.
    #[must_use]
    pub const fn is_set(&self, index: c_int) -> bool {
        if index < 0 || index >= 64 {
            return false;
        }
        self.set_bits & (1u64 << index as u32) != 0
    }

    /// Mark option at index as set.
    pub fn mark_set(&mut self, index: c_int) {
        if (0..64).contains(&index) {
            self.set_bits |= 1u64 << index as u32;
        }
    }

    /// Clear option at index.
    pub fn clear(&mut self, index: c_int) {
        if (0..64).contains(&index) {
            self.set_bits &= !(1u64 << index as u32);
        }
    }

    /// Clear all flags.
    pub fn clear_all(&mut self) {
        self.set_bits = 0;
    }

    /// Count set options.
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub const fn count_set(&self) -> c_int {
        self.set_bits.count_ones() as c_int
    }
}

/// FFI: Check if local option is set.
///
/// # Safety
/// `flags` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_local_opt_is_set(flags: *const LocalOptFlags, index: c_int) -> c_int {
    if flags.is_null() {
        return 0;
    }
    c_int::from((*flags).is_set(index))
}

/// FFI: Mark local option as set.
#[no_mangle]
pub unsafe extern "C" fn rs_local_opt_mark_set(flags: *mut LocalOptFlags, index: c_int) {
    if !flags.is_null() {
        (*flags).mark_set(index);
    }
}

/// FFI: Clear local option flag.
#[no_mangle]
pub unsafe extern "C" fn rs_local_opt_clear(flags: *mut LocalOptFlags, index: c_int) {
    if !flags.is_null() {
        (*flags).clear(index);
    }
}

/// FFI: Clear all local option flags.
#[no_mangle]
pub unsafe extern "C" fn rs_local_opt_clear_all(flags: *mut LocalOptFlags) {
    if !flags.is_null() {
        (*flags).clear_all();
    }
}

/// FFI: Count set local options.
///
/// # Safety
/// `flags` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_local_opt_count_set(flags: *const LocalOptFlags) -> c_int {
    if flags.is_null() {
        return 0;
    }
    (*flags).count_set()
}

// =============================================================================
// Copy Context FFI
// =============================================================================

/// FFI: Create copy context.
#[no_mangle]
pub extern "C" fn rs_copy_context_new(direction: c_int) -> CopyContext {
    CopyContext::new(CopyDirection::from_c_int(direction))
}

/// FFI: Check if should copy option.
///
/// # Safety
/// `ctx` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_copy_should_copy(ctx: *const CopyContext, opt_flags: u32) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).should_copy(opt_flags))
}

/// FFI: Get source scope for direction.
#[no_mangle]
pub extern "C" fn rs_copy_source_scope(direction: c_int) -> c_int {
    CopyDirection::from_c_int(direction).source_scope() as c_int
}

/// FFI: Get destination scope for direction.
#[no_mangle]
pub extern "C" fn rs_copy_dest_scope(direction: c_int) -> c_int {
    CopyDirection::from_c_int(direction).dest_scope() as c_int
}

/// FFI: Check if copy result is OK.
#[no_mangle]
pub extern "C" fn rs_copy_result_is_ok(result: c_int) -> c_int {
    c_int::from(result == CopyResult::Ok as c_int)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copy_direction() {
        assert_eq!(CopyDirection::from_c_int(0), CopyDirection::GlobalToLocal);
        assert_eq!(CopyDirection::from_c_int(1), CopyDirection::LocalToGlobal);

        assert_eq!(
            CopyDirection::GlobalToLocal.source_scope(),
            OptScope::Global
        );
        assert_eq!(CopyDirection::LocalToGlobal.dest_scope(), OptScope::Global);
    }

    #[test]
    fn test_copy_context() {
        let ctx = CopyContext::new(CopyDirection::GlobalToLocal);
        assert!(ctx.all_options);
        assert!(ctx.should_copy(0xFFFF));

        let mut ctx = ctx;
        ctx.flags_filter = 0x02;
        assert!(ctx.should_copy(0x02));
        assert!(!ctx.should_copy(0x01));
    }

    #[test]
    fn test_copy_result() {
        assert!(CopyResult::Ok.is_ok());
        assert!(!CopyResult::NotSet.is_ok());
    }

    #[test]
    fn test_supports_scope() {
        let flags = 0x07u32; // Global + Win + Buf
        assert!(supports_scope(flags, OptScope::Global));
        assert!(supports_scope(flags, OptScope::Win));
        assert!(supports_scope(flags, OptScope::Buf));

        let flags = 0x01u32; // Global only
        assert!(supports_scope(flags, OptScope::Global));
        assert!(!supports_scope(flags, OptScope::Win));
    }

    #[test]
    fn test_default_scope() {
        assert_eq!(default_scope(0x02), OptScope::Win);
        assert_eq!(default_scope(0x04), OptScope::Buf);
        assert_eq!(default_scope(0x01), OptScope::Global);
    }

    #[test]
    fn test_value_comparison() {
        assert_eq!(rs_bool_values_equal(1, 1), 1);
        assert_eq!(rs_bool_values_equal(1, 0), 0);
        assert_eq!(rs_num_values_equal(42, 42), 1);
        assert_eq!(rs_num_values_equal(42, 43), 0);
    }

    #[test]
    fn test_str_values_equal() {
        unsafe {
            assert_eq!(rs_str_values_equal(c"hello".as_ptr(), c"hello".as_ptr()), 1);
            assert_eq!(rs_str_values_equal(c"hello".as_ptr(), c"world".as_ptr()), 0);
            assert_eq!(rs_str_values_equal(std::ptr::null(), std::ptr::null()), 1);
            assert_eq!(rs_str_values_equal(c"x".as_ptr(), std::ptr::null()), 0);
        }
    }

    #[test]
    fn test_local_opt_flags() {
        let mut flags = LocalOptFlags::default();
        assert!(!flags.is_set(5));

        flags.mark_set(5);
        assert!(flags.is_set(5));
        assert_eq!(flags.count_set(), 1);

        flags.mark_set(10);
        assert_eq!(flags.count_set(), 2);

        flags.clear(5);
        assert!(!flags.is_set(5));
        assert_eq!(flags.count_set(), 1);

        flags.clear_all();
        assert_eq!(flags.count_set(), 0);
    }

    #[test]
    fn test_copy_value_info() {
        let info = rs_copy_value_bool(1, 1);
        assert_eq!(info.opt_type, OptValType::Boolean as c_int);
        assert!(info.is_set);
        assert_eq!(info.bool_val, 1);

        let info = rs_copy_value_num(42, 1);
        assert_eq!(info.opt_type, OptValType::Number as c_int);
        assert_eq!(info.num_val, 42);
    }
}
