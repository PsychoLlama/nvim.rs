//! Testing framework for Neovim Rust components
//!
//! This crate provides testing infrastructure for Neovim's Rust migration,
//! including test harness, assertions, mocks, and fixtures.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)] // FFI functions cannot be const
#![allow(clippy::cast_sign_loss)] // c_char to u8 casts in FFI code
#![allow(clippy::cast_possible_truncation)] // c_int to smaller types in FFI
#![allow(clippy::cast_possible_wrap)] // u8 to c_char (i8) casts in FFI

pub mod assert;
pub mod fixture;
pub mod harness;
pub mod mock;
pub mod viml_assert;

use std::ffi::c_int;

// =============================================================================
// Test Result
// =============================================================================

/// Result of a test execution.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestResult {
    /// Test passed
    #[default]
    Pass = 0,
    /// Test failed
    Fail = 1,
    /// Test was skipped
    Skip = 2,
    /// Test encountered an error
    Error = 3,
    /// Test timed out
    Timeout = 4,
}

impl TestResult {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Fail,
            2 => Self::Skip,
            3 => Self::Error,
            4 => Self::Timeout,
            _ => Self::Pass,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if result indicates success.
    #[must_use]
    pub const fn is_success(self) -> bool {
        matches!(self, Self::Pass | Self::Skip)
    }

    /// Check if result indicates failure.
    #[must_use]
    pub const fn is_failure(self) -> bool {
        matches!(self, Self::Fail | Self::Error | Self::Timeout)
    }
}

/// FFI: Check if test result is success.
#[no_mangle]
pub extern "C" fn rs_test_result_is_success(result: c_int) -> c_int {
    c_int::from(TestResult::from_c_int(result).is_success())
}

/// FFI: Check if test result is failure.
#[no_mangle]
pub extern "C" fn rs_test_result_is_failure(result: c_int) -> c_int {
    c_int::from(TestResult::from_c_int(result).is_failure())
}

// =============================================================================
// Test Status
// =============================================================================

/// Current status of a test.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestStatus {
    /// Test not yet started
    #[default]
    Pending = 0,
    /// Test is running
    Running = 1,
    /// Test completed
    Completed = 2,
    /// Test was cancelled
    Cancelled = 3,
}

impl TestStatus {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Running,
            2 => Self::Completed,
            3 => Self::Cancelled,
            _ => Self::Pending,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if test is done.
    #[must_use]
    pub const fn is_done(self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled)
    }
}

// =============================================================================
// Test Category
// =============================================================================

/// Category of test for organization.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestCategory {
    /// Unit test (tests single function/module)
    #[default]
    Unit = 0,
    /// Integration test (tests component interaction)
    Integration = 1,
    /// Functional test (tests user-level features)
    Functional = 2,
    /// Performance test
    Performance = 3,
    /// Regression test
    Regression = 4,
    /// Smoke test (basic sanity check)
    Smoke = 5,
}

impl TestCategory {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Integration,
            2 => Self::Functional,
            3 => Self::Performance,
            4 => Self::Regression,
            5 => Self::Smoke,
            _ => Self::Unit,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_result() {
        assert!(TestResult::Pass.is_success());
        assert!(TestResult::Skip.is_success());
        assert!(!TestResult::Fail.is_success());

        assert!(TestResult::Fail.is_failure());
        assert!(TestResult::Error.is_failure());
        assert!(!TestResult::Pass.is_failure());
    }

    #[test]
    fn test_test_status() {
        assert!(!TestStatus::Pending.is_done());
        assert!(!TestStatus::Running.is_done());
        assert!(TestStatus::Completed.is_done());
        assert!(TestStatus::Cancelled.is_done());
    }

    #[test]
    fn test_test_category() {
        assert_eq!(TestCategory::from_c_int(0), TestCategory::Unit);
        assert_eq!(TestCategory::from_c_int(1), TestCategory::Integration);
    }
}
