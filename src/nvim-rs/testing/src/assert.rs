//! Test assertions
//!
//! This module provides assertion utilities for Neovim tests,
//! including comparison helpers and failure reporting.

use std::ffi::{c_char, c_int};

// =============================================================================
// Assert Result
// =============================================================================

/// Result of an assertion.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AssertResult {
    /// Assertion passed
    #[default]
    Pass = 0,
    /// Assertion failed
    Fail = 1,
    /// Assertion skipped (conditional)
    Skip = 2,
}

impl AssertResult {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Fail,
            2 => Self::Skip,
            _ => Self::Pass,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if passed.
    #[must_use]
    pub const fn passed(self) -> bool {
        matches!(self, Self::Pass)
    }
}

/// FFI: Check if assertion passed.
#[no_mangle]
pub extern "C" fn rs_assert_result_passed(result: c_int) -> c_int {
    c_int::from(AssertResult::from_c_int(result).passed())
}

// =============================================================================
// Assert Context
// =============================================================================

/// Context for assertion reporting.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct AssertContext {
    /// Line number where assertion was made
    pub line: c_int,
    /// Assertion count in current test
    pub count: c_int,
    /// Number of failures
    pub failures: c_int,
    /// Continue after failure
    pub continue_on_fail: bool,
}

impl AssertContext {
    /// Create new context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            line: 0,
            count: 0,
            failures: 0,
            continue_on_fail: false,
        }
    }

    /// Record an assertion.
    pub fn record(&mut self, result: AssertResult, line: c_int) {
        self.count += 1;
        self.line = line;
        if matches!(result, AssertResult::Fail) {
            self.failures += 1;
        }
    }

    /// Check if any assertions failed.
    #[must_use]
    pub const fn has_failures(&self) -> bool {
        self.failures > 0
    }

    /// Reset the context.
    pub fn reset(&mut self) {
        self.line = 0;
        self.count = 0;
        self.failures = 0;
    }
}

/// FFI: Create assert context.
#[no_mangle]
pub extern "C" fn rs_assert_context_new() -> AssertContext {
    AssertContext::new()
}

/// FFI: Record assertion in context.
///
/// # Safety
/// `ctx` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_assert_context_record(
    ctx: *mut AssertContext,
    result: c_int,
    line: c_int,
) {
    if !ctx.is_null() {
        (*ctx).record(AssertResult::from_c_int(result), line);
    }
}

/// FFI: Check if context has failures.
///
/// # Safety
/// `ctx` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_assert_context_has_failures(ctx: *const AssertContext) -> c_int {
    if ctx.is_null() {
        return 0;
    }
    c_int::from((*ctx).has_failures())
}

// =============================================================================
// Integer Assertions
// =============================================================================

/// Assert two integers are equal.
#[no_mangle]
pub extern "C" fn rs_assert_int_eq(actual: i64, expected: i64) -> c_int {
    if actual == expected {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

/// Assert two integers are not equal.
#[no_mangle]
pub extern "C" fn rs_assert_int_ne(actual: i64, expected: i64) -> c_int {
    if actual == expected {
        AssertResult::Fail as c_int
    } else {
        AssertResult::Pass as c_int
    }
}

/// Assert first integer is less than second.
#[no_mangle]
pub extern "C" fn rs_assert_int_lt(actual: i64, expected: i64) -> c_int {
    if actual < expected {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

/// Assert first integer is less than or equal to second.
#[no_mangle]
pub extern "C" fn rs_assert_int_le(actual: i64, expected: i64) -> c_int {
    if actual <= expected {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

/// Assert first integer is greater than second.
#[no_mangle]
pub extern "C" fn rs_assert_int_gt(actual: i64, expected: i64) -> c_int {
    if actual > expected {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

/// Assert first integer is greater than or equal to second.
#[no_mangle]
pub extern "C" fn rs_assert_int_ge(actual: i64, expected: i64) -> c_int {
    if actual >= expected {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

/// Assert integer is in range [min, max].
#[no_mangle]
pub extern "C" fn rs_assert_int_in_range(value: i64, min: i64, max: i64) -> c_int {
    if value >= min && value <= max {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

// =============================================================================
// Boolean Assertions
// =============================================================================

/// Assert value is true (non-zero).
#[no_mangle]
pub extern "C" fn rs_assert_true(value: c_int) -> c_int {
    if value != 0 {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

/// Assert value is false (zero).
#[no_mangle]
pub extern "C" fn rs_assert_false(value: c_int) -> c_int {
    if value == 0 {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

// =============================================================================
// Pointer Assertions
// =============================================================================

/// Assert pointer is null.
#[no_mangle]
pub extern "C" fn rs_assert_null(ptr: *const core::ffi::c_void) -> c_int {
    if ptr.is_null() {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

/// Assert pointer is not null.
#[no_mangle]
pub extern "C" fn rs_assert_not_null(ptr: *const core::ffi::c_void) -> c_int {
    if ptr.is_null() {
        AssertResult::Fail as c_int
    } else {
        AssertResult::Pass as c_int
    }
}

/// Assert two pointers are equal.
#[no_mangle]
pub extern "C" fn rs_assert_ptr_eq(
    a: *const core::ffi::c_void,
    b: *const core::ffi::c_void,
) -> c_int {
    if a == b {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

// =============================================================================
// String Assertions
// =============================================================================

/// Assert two C strings are equal.
///
/// # Safety
/// Both `a` and `b` must be valid C strings or null.
#[no_mangle]
pub unsafe extern "C" fn rs_assert_str_eq(a: *const c_char, b: *const c_char) -> c_int {
    if a.is_null() && b.is_null() {
        return AssertResult::Pass as c_int;
    }
    if a.is_null() || b.is_null() {
        return AssertResult::Fail as c_int;
    }

    let mut pa = a;
    let mut pb = b;

    while *pa != 0 && *pb != 0 {
        if *pa != *pb {
            return AssertResult::Fail as c_int;
        }
        pa = pa.add(1);
        pb = pb.add(1);
    }

    if *pa == *pb {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

/// Assert string starts with prefix.
///
/// # Safety
/// Both `s` and `prefix` must be valid C strings or null.
#[no_mangle]
pub unsafe extern "C" fn rs_assert_str_starts_with(
    s: *const c_char,
    prefix: *const c_char,
) -> c_int {
    if s.is_null() || prefix.is_null() {
        return AssertResult::Fail as c_int;
    }

    let mut ps = s;
    let mut pp = prefix;

    while *pp != 0 {
        if *ps == 0 || *ps != *pp {
            return AssertResult::Fail as c_int;
        }
        ps = ps.add(1);
        pp = pp.add(1);
    }

    AssertResult::Pass as c_int
}

/// Assert string ends with suffix.
///
/// # Safety
/// Both `s` and `suffix` must be valid C strings or null.
#[no_mangle]
pub unsafe extern "C" fn rs_assert_str_ends_with(s: *const c_char, suffix: *const c_char) -> c_int {
    if s.is_null() || suffix.is_null() {
        return AssertResult::Fail as c_int;
    }

    // Get string lengths
    let mut s_len = 0i64;
    let mut p = s;
    while *p != 0 {
        s_len += 1;
        p = p.add(1);
    }

    let mut suffix_len = 0i64;
    p = suffix;
    while *p != 0 {
        suffix_len += 1;
        p = p.add(1);
    }

    if suffix_len > s_len {
        return AssertResult::Fail as c_int;
    }

    // Compare from end
    let offset = (s_len - suffix_len) as usize;
    let mut ps = s.add(offset);
    let mut pp = suffix;

    while *pp != 0 {
        if *ps != *pp {
            return AssertResult::Fail as c_int;
        }
        ps = ps.add(1);
        pp = pp.add(1);
    }

    AssertResult::Pass as c_int
}

/// Assert string contains substring.
///
/// # Safety
/// Both `s` and `needle` must be valid C strings or null.
#[no_mangle]
pub unsafe extern "C" fn rs_assert_str_contains(s: *const c_char, needle: *const c_char) -> c_int {
    if s.is_null() || needle.is_null() {
        return AssertResult::Fail as c_int;
    }

    // Empty needle matches
    if *needle == 0 {
        return AssertResult::Pass as c_int;
    }

    let mut ps = s;
    while *ps != 0 {
        // Try to match starting here
        let mut pm = ps;
        let mut pn = needle;

        while *pn != 0 && *pm != 0 && *pm == *pn {
            pm = pm.add(1);
            pn = pn.add(1);
        }

        if *pn == 0 {
            return AssertResult::Pass as c_int;
        }

        ps = ps.add(1);
    }

    AssertResult::Fail as c_int
}

/// Get length of C string.
///
/// # Safety
/// `s` must be valid C string or null.
#[no_mangle]
pub unsafe extern "C" fn rs_assert_str_len(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }

    let mut len = 0;
    let mut p = s;
    while *p != 0 {
        len += 1;
        p = p.add(1);
    }
    len
}

// =============================================================================
// Memory Assertions
// =============================================================================

/// Assert memory regions are equal.
///
/// # Safety
/// Both `a` and `b` must point to at least `len` bytes of valid memory.
#[no_mangle]
pub unsafe extern "C" fn rs_assert_mem_eq(a: *const u8, b: *const u8, len: usize) -> c_int {
    if a.is_null() || b.is_null() {
        return if a.is_null() && b.is_null() {
            AssertResult::Pass as c_int
        } else {
            AssertResult::Fail as c_int
        };
    }

    for i in 0..len {
        if *a.add(i) != *b.add(i) {
            return AssertResult::Fail as c_int;
        }
    }

    AssertResult::Pass as c_int
}

/// Assert memory region is zeroed.
///
/// # Safety
/// `ptr` must point to at least `len` bytes of valid memory.
#[no_mangle]
pub unsafe extern "C" fn rs_assert_mem_zero(ptr: *const u8, len: usize) -> c_int {
    if ptr.is_null() {
        return AssertResult::Fail as c_int;
    }

    for i in 0..len {
        if *ptr.add(i) != 0 {
            return AssertResult::Fail as c_int;
        }
    }

    AssertResult::Pass as c_int
}

// =============================================================================
// Float Assertions
// =============================================================================

/// Assert two floats are approximately equal.
#[no_mangle]
pub extern "C" fn rs_assert_float_eq(actual: f64, expected: f64, epsilon: f64) -> c_int {
    let diff = (actual - expected).abs();
    if diff <= epsilon {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

/// Assert float is finite (not NaN or infinity).
#[no_mangle]
pub extern "C" fn rs_assert_float_finite(value: f64) -> c_int {
    if value.is_finite() {
        AssertResult::Pass as c_int
    } else {
        AssertResult::Fail as c_int
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_context() {
        let mut ctx = AssertContext::new();
        assert!(!ctx.has_failures());

        ctx.record(AssertResult::Pass, 10);
        assert!(!ctx.has_failures());
        assert_eq!(ctx.count, 1);

        ctx.record(AssertResult::Fail, 20);
        assert!(ctx.has_failures());
        assert_eq!(ctx.failures, 1);
    }

    #[test]
    fn test_int_assertions() {
        assert_eq!(rs_assert_int_eq(5, 5), 0);
        assert_eq!(rs_assert_int_eq(5, 6), 1);

        assert_eq!(rs_assert_int_ne(5, 6), 0);
        assert_eq!(rs_assert_int_ne(5, 5), 1);

        assert_eq!(rs_assert_int_lt(5, 10), 0);
        assert_eq!(rs_assert_int_lt(10, 5), 1);

        assert_eq!(rs_assert_int_in_range(5, 1, 10), 0);
        assert_eq!(rs_assert_int_in_range(15, 1, 10), 1);
    }

    #[test]
    fn test_bool_assertions() {
        assert_eq!(rs_assert_true(1), 0);
        assert_eq!(rs_assert_true(0), 1);
        assert_eq!(rs_assert_false(0), 0);
        assert_eq!(rs_assert_false(1), 1);
    }

    #[test]
    fn test_string_assertions() {
        unsafe {
            assert_eq!(rs_assert_str_eq(c"hello".as_ptr(), c"hello".as_ptr()), 0);
            assert_eq!(rs_assert_str_eq(c"hello".as_ptr(), c"world".as_ptr()), 1);

            assert_eq!(
                rs_assert_str_starts_with(c"hello world".as_ptr(), c"hello".as_ptr()),
                0
            );
            assert_eq!(
                rs_assert_str_starts_with(c"hello world".as_ptr(), c"world".as_ptr()),
                1
            );

            assert_eq!(
                rs_assert_str_ends_with(c"hello world".as_ptr(), c"world".as_ptr()),
                0
            );
            assert_eq!(
                rs_assert_str_ends_with(c"hello world".as_ptr(), c"hello".as_ptr()),
                1
            );

            assert_eq!(
                rs_assert_str_contains(c"hello world".as_ptr(), c"lo wo".as_ptr()),
                0
            );
            assert_eq!(
                rs_assert_str_contains(c"hello world".as_ptr(), c"xyz".as_ptr()),
                1
            );
        }
    }

    #[test]
    fn test_float_assertions() {
        assert_eq!(rs_assert_float_eq(1.0, 1.0, 0.001), 0);
        assert_eq!(rs_assert_float_eq(1.0, 1.0001, 0.001), 0);
        assert_eq!(rs_assert_float_eq(1.0, 2.0, 0.001), 1);

        assert_eq!(rs_assert_float_finite(1.0), 0);
        assert_eq!(rs_assert_float_finite(f64::NAN), 1);
        assert_eq!(rs_assert_float_finite(f64::INFINITY), 1);
    }
}
