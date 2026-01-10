//! Test harness infrastructure
//!
//! This module provides the test harness for running Neovim tests,
//! including test registration, execution, and reporting.

use std::ffi::{c_char, c_int};

use crate::{TestCategory, TestResult, TestStatus};

// =============================================================================
// Test Info
// =============================================================================

/// Information about a single test.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TestInfo {
    /// Test ID
    pub id: c_int,
    /// Test category
    pub category: c_int,
    /// Whether test is enabled
    pub enabled: bool,
    /// Whether test should run isolated
    pub isolated: bool,
    /// Timeout in milliseconds (0 = default)
    pub timeout_ms: c_int,
    /// Expected result (for negative tests)
    pub expected_result: c_int,
}

impl TestInfo {
    /// Create new test info.
    #[must_use]
    pub const fn new(id: c_int) -> Self {
        Self {
            id,
            category: TestCategory::Unit as c_int,
            enabled: true,
            isolated: false,
            timeout_ms: 0,
            expected_result: TestResult::Pass as c_int,
        }
    }

    /// Get the category.
    #[must_use]
    pub const fn get_category(&self) -> TestCategory {
        TestCategory::from_c_int(self.category)
    }

    /// Check if test should run.
    #[must_use]
    pub const fn should_run(&self) -> bool {
        self.enabled
    }
}

/// FFI: Create test info.
#[no_mangle]
pub extern "C" fn rs_test_info_new(id: c_int) -> TestInfo {
    TestInfo::new(id)
}

/// FFI: Check if test should run.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_test_should_run(info: *const TestInfo) -> c_int {
    if info.is_null() {
        return 0;
    }
    c_int::from((*info).should_run())
}

// =============================================================================
// Test Run
// =============================================================================

/// State of a test run.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TestRun {
    /// Test ID
    pub test_id: c_int,
    /// Current status
    pub status: c_int,
    /// Result (if completed)
    pub result: c_int,
    /// Start time (ms since epoch)
    pub start_time_ms: i64,
    /// End time (ms since epoch)
    pub end_time_ms: i64,
    /// Memory used (bytes)
    pub memory_used: i64,
}

impl TestRun {
    /// Create new test run.
    #[must_use]
    pub const fn new(test_id: c_int) -> Self {
        Self {
            test_id,
            status: TestStatus::Pending as c_int,
            result: TestResult::Pass as c_int,
            start_time_ms: 0,
            end_time_ms: 0,
            memory_used: 0,
        }
    }

    /// Get status.
    #[must_use]
    pub const fn get_status(&self) -> TestStatus {
        TestStatus::from_c_int(self.status)
    }

    /// Get result.
    #[must_use]
    pub const fn get_result(&self) -> TestResult {
        TestResult::from_c_int(self.result)
    }

    /// Start the test run.
    pub fn start(&mut self, time_ms: i64) {
        self.status = TestStatus::Running as c_int;
        self.start_time_ms = time_ms;
    }

    /// Finish the test run.
    pub fn finish(&mut self, result: TestResult, time_ms: i64) {
        self.status = TestStatus::Completed as c_int;
        self.result = result as c_int;
        self.end_time_ms = time_ms;
    }

    /// Get elapsed time in milliseconds.
    #[must_use]
    pub const fn elapsed_ms(&self) -> i64 {
        if self.end_time_ms > self.start_time_ms {
            self.end_time_ms - self.start_time_ms
        } else {
            0
        }
    }
}

/// FFI: Create test run.
#[no_mangle]
pub extern "C" fn rs_test_run_new(test_id: c_int) -> TestRun {
    TestRun::new(test_id)
}

/// FFI: Start test run.
///
/// # Safety
/// `run` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_test_run_start(run: *mut TestRun, time_ms: i64) {
    if !run.is_null() {
        (*run).start(time_ms);
    }
}

/// FFI: Finish test run.
///
/// # Safety
/// `run` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_test_run_finish(run: *mut TestRun, result: c_int, time_ms: i64) {
    if !run.is_null() {
        (*run).finish(TestResult::from_c_int(result), time_ms);
    }
}

/// FFI: Get elapsed time.
///
/// # Safety
/// `run` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_test_run_elapsed_ms(run: *const TestRun) -> i64 {
    if run.is_null() {
        return 0;
    }
    (*run).elapsed_ms()
}

// =============================================================================
// Test Suite
// =============================================================================

/// Statistics for a test suite.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TestSuiteStats {
    /// Total number of tests
    pub total: c_int,
    /// Tests passed
    pub passed: c_int,
    /// Tests failed
    pub failed: c_int,
    /// Tests skipped
    pub skipped: c_int,
    /// Tests with errors
    pub errors: c_int,
    /// Tests timed out
    pub timeouts: c_int,
    /// Total elapsed time (ms)
    pub total_time_ms: i64,
}

impl TestSuiteStats {
    /// Create new stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            skipped: 0,
            errors: 0,
            timeouts: 0,
            total_time_ms: 0,
        }
    }

    /// Record a test result.
    pub fn record(&mut self, result: TestResult, elapsed_ms: i64) {
        self.total += 1;
        self.total_time_ms += elapsed_ms;

        match result {
            TestResult::Pass => self.passed += 1,
            TestResult::Fail => self.failed += 1,
            TestResult::Skip => self.skipped += 1,
            TestResult::Error => self.errors += 1,
            TestResult::Timeout => self.timeouts += 1,
        }
    }

    /// Check if all tests passed.
    #[must_use]
    pub const fn all_passed(&self) -> bool {
        self.failed == 0 && self.errors == 0 && self.timeouts == 0
    }

    /// Get pass rate as percentage (0-100).
    #[must_use]
    pub const fn pass_rate(&self) -> c_int {
        if self.total == 0 {
            100
        } else {
            (self.passed * 100) / self.total
        }
    }
}

/// FFI: Create suite stats.
#[no_mangle]
pub extern "C" fn rs_test_suite_stats_new() -> TestSuiteStats {
    TestSuiteStats::new()
}

/// FFI: Record result in stats.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_test_suite_record(
    stats: *mut TestSuiteStats,
    result: c_int,
    elapsed_ms: i64,
) {
    if !stats.is_null() {
        (*stats).record(TestResult::from_c_int(result), elapsed_ms);
    }
}

/// FFI: Check if all passed.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_test_suite_all_passed(stats: *const TestSuiteStats) -> c_int {
    if stats.is_null() {
        return 0;
    }
    c_int::from((*stats).all_passed())
}

/// FFI: Get pass rate.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_test_suite_pass_rate(stats: *const TestSuiteStats) -> c_int {
    if stats.is_null() {
        return 0;
    }
    (*stats).pass_rate()
}

// =============================================================================
// Test Filter
// =============================================================================

/// Filter for selecting tests to run.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TestFilter {
    /// Only run tests matching this category (-1 for all)
    pub category: c_int,
    /// Only run tests with ID >= `min_id`
    pub min_id: c_int,
    /// Only run tests with ID <= `max_id` (-1 for no limit)
    pub max_id: c_int,
    /// Skip disabled tests
    pub skip_disabled: bool,
    /// Only run isolated tests
    pub only_isolated: bool,
}

impl TestFilter {
    /// Create filter that matches all tests.
    #[must_use]
    pub const fn all() -> Self {
        Self {
            category: -1,
            min_id: 0,
            max_id: -1,
            skip_disabled: true,
            only_isolated: false,
        }
    }

    /// Check if filter matches a test.
    #[must_use]
    pub const fn matches(&self, info: &TestInfo) -> bool {
        // Check enabled
        if self.skip_disabled && !info.enabled {
            return false;
        }

        // Check isolated
        if self.only_isolated && !info.isolated {
            return false;
        }

        // Check category
        if self.category >= 0 && info.category != self.category {
            return false;
        }

        // Check ID range
        if info.id < self.min_id {
            return false;
        }

        if self.max_id >= 0 && info.id > self.max_id {
            return false;
        }

        true
    }
}

/// FFI: Create filter for all tests.
#[no_mangle]
pub extern "C" fn rs_test_filter_all() -> TestFilter {
    TestFilter::all()
}

/// FFI: Check if filter matches test.
///
/// # Safety
/// `filter` and `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_test_filter_matches(
    filter: *const TestFilter,
    info: *const TestInfo,
) -> c_int {
    if filter.is_null() || info.is_null() {
        return 0;
    }
    c_int::from((*filter).matches(&*info))
}

// =============================================================================
// Test Output Format
// =============================================================================

/// Output format for test results.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestOutputFormat {
    /// Plain text
    #[default]
    Text = 0,
    /// TAP (Test Anything Protocol)
    Tap = 1,
    /// JUnit-style XML
    JUnit = 2,
    /// JSON
    Json = 3,
}

impl TestOutputFormat {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Tap,
            2 => Self::JUnit,
            3 => Self::Json,
            _ => Self::Text,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Test Message Type
// =============================================================================

/// Type of test message/output.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TestMsgType {
    /// Informational message
    #[default]
    Info = 0,
    /// Warning message
    Warning = 1,
    /// Error message
    Error = 2,
    /// Debug message
    Debug = 3,
    /// Assertion message
    Assert = 4,
}

impl TestMsgType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Warning,
            2 => Self::Error,
            3 => Self::Debug,
            4 => Self::Assert,
            _ => Self::Info,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Test Name Validation
// =============================================================================

/// Validate a test name.
///
/// # Safety
/// `name` must be valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_test_validate_name(name: *const c_char) -> c_int {
    if name.is_null() {
        return 0;
    }

    let mut p = name;
    let mut len = 0;

    // First char must be letter or underscore
    let first = *p as u8;
    if !first.is_ascii_alphabetic() && first != b'_' {
        return 0;
    }

    while *p != 0 {
        let c = *p as u8;
        if !c.is_ascii_alphanumeric() && c != b'_' && c != b':' {
            return 0;
        }
        p = p.add(1);
        len += 1;

        // Limit name length
        if len > 256 {
            return 0;
        }
    }

    c_int::from(len > 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_info() {
        let info = TestInfo::new(42);
        assert_eq!(info.id, 42);
        assert!(info.enabled);
        assert!(info.should_run());
    }

    #[test]
    fn test_test_run() {
        let mut run = TestRun::new(1);
        assert_eq!(run.get_status(), TestStatus::Pending);

        run.start(1000);
        assert_eq!(run.get_status(), TestStatus::Running);

        run.finish(TestResult::Pass, 1500);
        assert_eq!(run.get_status(), TestStatus::Completed);
        assert_eq!(run.get_result(), TestResult::Pass);
        assert_eq!(run.elapsed_ms(), 500);
    }

    #[test]
    fn test_suite_stats() {
        let mut stats = TestSuiteStats::new();
        assert!(stats.all_passed());
        assert_eq!(stats.pass_rate(), 100);

        stats.record(TestResult::Pass, 100);
        stats.record(TestResult::Pass, 100);
        stats.record(TestResult::Fail, 50);

        assert!(!stats.all_passed());
        assert_eq!(stats.total, 3);
        assert_eq!(stats.passed, 2);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.pass_rate(), 66); // 2/3 = 66%
    }

    #[test]
    fn test_filter() {
        let filter = TestFilter::all();

        let info = TestInfo::new(5);
        assert!(filter.matches(&info));

        let mut disabled = TestInfo::new(5);
        disabled.enabled = false;
        assert!(!filter.matches(&disabled));
    }

    #[test]
    fn test_validate_name() {
        unsafe {
            assert_eq!(rs_test_validate_name(c"test_foo".as_ptr()), 1);
            assert_eq!(rs_test_validate_name(c"Test::bar".as_ptr()), 1);
            assert_eq!(rs_test_validate_name(c"_private".as_ptr()), 1);
            assert_eq!(rs_test_validate_name(c"123invalid".as_ptr()), 0);
            assert_eq!(rs_test_validate_name(c"has space".as_ptr()), 0);
        }
    }
}
