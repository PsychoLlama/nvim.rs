//! Mock infrastructure
//!
//! This module provides mock objects and stubs for testing
//! Neovim components in isolation.

use std::ffi::c_int;

// =============================================================================
// Mock State
// =============================================================================

/// State for a mock object.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MockState {
    /// Number of times called
    pub call_count: c_int,
    /// Whether mock is enabled
    pub enabled: bool,
    /// Whether to record calls
    pub recording: bool,
    /// Last call's return value
    pub last_return: c_int,
    /// Last call's first argument
    pub last_arg1: i64,
    /// Last call's second argument
    pub last_arg2: i64,
}

impl MockState {
    /// Create new mock state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            call_count: 0,
            enabled: true,
            recording: true,
            last_return: 0,
            last_arg1: 0,
            last_arg2: 0,
        }
    }

    /// Record a call with no arguments.
    pub fn record_call(&mut self) {
        if self.recording {
            self.call_count += 1;
        }
    }

    /// Record a call with one argument.
    pub fn record_call1(&mut self, arg1: i64) {
        if self.recording {
            self.call_count += 1;
            self.last_arg1 = arg1;
        }
    }

    /// Record a call with two arguments.
    pub fn record_call2(&mut self, arg1: i64, arg2: i64) {
        if self.recording {
            self.call_count += 1;
            self.last_arg1 = arg1;
            self.last_arg2 = arg2;
        }
    }

    /// Reset the mock state.
    pub fn reset(&mut self) {
        self.call_count = 0;
        self.last_return = 0;
        self.last_arg1 = 0;
        self.last_arg2 = 0;
    }

    /// Check if mock was called.
    #[must_use]
    pub const fn was_called(&self) -> bool {
        self.call_count > 0
    }

    /// Check if mock was called exactly n times.
    #[must_use]
    pub const fn was_called_times(&self, n: c_int) -> bool {
        self.call_count == n
    }
}

/// FFI: Create mock state.
#[no_mangle]
pub extern "C" fn rs_mock_state_new() -> MockState {
    MockState::new()
}

/// FFI: Record call.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mock_record_call(state: *mut MockState) {
    if !state.is_null() {
        (*state).record_call();
    }
}

/// FFI: Record call with one arg.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mock_record_call1(state: *mut MockState, arg1: i64) {
    if !state.is_null() {
        (*state).record_call1(arg1);
    }
}

/// FFI: Check if was called.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mock_was_called(state: *const MockState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).was_called())
}

/// FFI: Get call count.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mock_call_count(state: *const MockState) -> c_int {
    if state.is_null() {
        return 0;
    }
    (*state).call_count
}

/// FFI: Reset mock.
///
/// # Safety
/// `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mock_reset(state: *mut MockState) {
    if !state.is_null() {
        (*state).reset();
    }
}

// =============================================================================
// Mock Return Values
// =============================================================================

/// Configuration for mock return values.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MockReturns {
    /// Fixed return value
    pub fixed_value: i64,
    /// Use fixed value
    pub use_fixed: bool,
    /// Sequence of values to return
    pub sequence_index: c_int,
    /// Return value based on argument
    pub arg_based: bool,
    /// Multiplier for arg-based return
    pub arg_multiplier: i64,
}

impl MockReturns {
    /// Create returns with fixed value.
    #[must_use]
    pub const fn fixed(value: i64) -> Self {
        Self {
            fixed_value: value,
            use_fixed: true,
            sequence_index: 0,
            arg_based: false,
            arg_multiplier: 1,
        }
    }

    /// Create arg-based returns.
    #[must_use]
    pub const fn arg_based(multiplier: i64) -> Self {
        Self {
            fixed_value: 0,
            use_fixed: false,
            sequence_index: 0,
            arg_based: true,
            arg_multiplier: multiplier,
        }
    }

    /// Get return value.
    #[must_use]
    pub const fn get(&self, arg: i64) -> i64 {
        if self.use_fixed {
            self.fixed_value
        } else if self.arg_based {
            arg * self.arg_multiplier
        } else {
            0
        }
    }
}

/// FFI: Create fixed returns.
#[no_mangle]
pub extern "C" fn rs_mock_returns_fixed(value: i64) -> MockReturns {
    MockReturns::fixed(value)
}

/// FFI: Get return value.
///
/// # Safety
/// `returns` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mock_returns_get(returns: *const MockReturns, arg: i64) -> i64 {
    if returns.is_null() {
        return 0;
    }
    (*returns).get(arg)
}

// =============================================================================
// Mock Expectations
// =============================================================================

/// Expectation for mock behavior.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MockExpectation {
    /// Minimum call count
    pub min_calls: c_int,
    /// Maximum call count (-1 for unlimited)
    pub max_calls: c_int,
    /// Expected first argument (-1 for any)
    pub expect_arg1: i64,
    /// Expected argument count
    pub expect_arg_count: c_int,
}

impl MockExpectation {
    /// Create expectation for at least n calls.
    #[must_use]
    pub const fn at_least(n: c_int) -> Self {
        Self {
            min_calls: n,
            max_calls: -1,
            expect_arg1: -1,
            expect_arg_count: -1,
        }
    }

    /// Create expectation for exactly n calls.
    #[must_use]
    pub const fn exactly(n: c_int) -> Self {
        Self {
            min_calls: n,
            max_calls: n,
            expect_arg1: -1,
            expect_arg_count: -1,
        }
    }

    /// Create expectation for at most n calls.
    #[must_use]
    pub const fn at_most(n: c_int) -> Self {
        Self {
            min_calls: 0,
            max_calls: n,
            expect_arg1: -1,
            expect_arg_count: -1,
        }
    }

    /// Check if state satisfies expectation.
    #[must_use]
    pub const fn is_satisfied(&self, state: &MockState) -> bool {
        if state.call_count < self.min_calls {
            return false;
        }
        if self.max_calls >= 0 && state.call_count > self.max_calls {
            return false;
        }
        if self.expect_arg1 >= 0 && state.last_arg1 != self.expect_arg1 {
            return false;
        }
        true
    }
}

/// FFI: Create at-least expectation.
#[no_mangle]
pub extern "C" fn rs_mock_expect_at_least(n: c_int) -> MockExpectation {
    MockExpectation::at_least(n)
}

/// FFI: Create exactly expectation.
#[no_mangle]
pub extern "C" fn rs_mock_expect_exactly(n: c_int) -> MockExpectation {
    MockExpectation::exactly(n)
}

/// FFI: Check if expectation satisfied.
///
/// # Safety
/// Both `exp` and `state` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_mock_is_satisfied(
    exp: *const MockExpectation,
    state: *const MockState,
) -> c_int {
    if exp.is_null() || state.is_null() {
        return 0;
    }
    c_int::from((*exp).is_satisfied(&*state))
}

// =============================================================================
// Mock Buffer
// =============================================================================

/// Mock buffer for testing buffer operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MockBuffer {
    /// Buffer ID
    pub id: c_int,
    /// Line count
    pub line_count: c_int,
    /// Is modified
    pub modified: bool,
    /// Is loaded
    pub loaded: bool,
    /// Is listed
    pub listed: bool,
    /// Buffer name hash (for simple comparison)
    pub name_hash: u64,
}

impl MockBuffer {
    /// Create new mock buffer.
    #[must_use]
    pub const fn new(id: c_int) -> Self {
        Self {
            id,
            line_count: 0,
            modified: false,
            loaded: true,
            listed: true,
            name_hash: 0,
        }
    }

    /// Create mock buffer with lines.
    #[must_use]
    pub const fn with_lines(id: c_int, line_count: c_int) -> Self {
        Self {
            id,
            line_count,
            modified: false,
            loaded: true,
            listed: true,
            name_hash: 0,
        }
    }
}

/// FFI: Create mock buffer.
#[no_mangle]
pub extern "C" fn rs_mock_buffer_new(id: c_int) -> MockBuffer {
    MockBuffer::new(id)
}

/// FFI: Create mock buffer with lines.
#[no_mangle]
pub extern "C" fn rs_mock_buffer_with_lines(id: c_int, line_count: c_int) -> MockBuffer {
    MockBuffer::with_lines(id, line_count)
}

// =============================================================================
// Mock Window
// =============================================================================

/// Mock window for testing window operations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct MockWindow {
    /// Window ID
    pub id: c_int,
    /// Buffer ID
    pub buffer_id: c_int,
    /// Width
    pub width: c_int,
    /// Height
    pub height: c_int,
    /// Cursor line
    pub cursor_line: c_int,
    /// Cursor column
    pub cursor_col: c_int,
    /// Top line
    pub topline: c_int,
}

impl MockWindow {
    /// Create new mock window.
    #[must_use]
    pub const fn new(id: c_int, buffer_id: c_int) -> Self {
        Self {
            id,
            buffer_id,
            width: 80,
            height: 24,
            cursor_line: 1,
            cursor_col: 0,
            topline: 1,
        }
    }
}

/// FFI: Create mock window.
#[no_mangle]
pub extern "C" fn rs_mock_window_new(id: c_int, buffer_id: c_int) -> MockWindow {
    MockWindow::new(id, buffer_id)
}

// =============================================================================
// Mock Callback
// =============================================================================

/// Type of mock callback behavior.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MockCallbackType {
    /// Do nothing
    #[default]
    Noop = 0,
    /// Return fixed value
    ReturnFixed = 1,
    /// Return argument
    ReturnArg = 2,
    /// Increment counter
    Increment = 3,
    /// Set flag
    SetFlag = 4,
}

impl MockCallbackType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::ReturnFixed,
            2 => Self::ReturnArg,
            3 => Self::Increment,
            4 => Self::SetFlag,
            _ => Self::Noop,
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
    fn test_mock_state() {
        let mut state = MockState::new();
        assert!(!state.was_called());

        state.record_call();
        assert!(state.was_called());
        assert!(state.was_called_times(1));

        state.record_call1(42);
        assert_eq!(state.call_count, 2);
        assert_eq!(state.last_arg1, 42);

        state.reset();
        assert!(!state.was_called());
    }

    #[test]
    fn test_mock_returns() {
        let fixed = MockReturns::fixed(99);
        assert_eq!(fixed.get(0), 99);
        assert_eq!(fixed.get(100), 99);

        let arg_based = MockReturns::arg_based(2);
        assert_eq!(arg_based.get(5), 10);
        assert_eq!(arg_based.get(10), 20);
    }

    #[test]
    fn test_mock_expectation() {
        let mut state = MockState::new();

        let at_least = MockExpectation::at_least(2);
        assert!(!at_least.is_satisfied(&state));

        state.record_call();
        state.record_call();
        assert!(at_least.is_satisfied(&state));

        let exactly = MockExpectation::exactly(2);
        assert!(exactly.is_satisfied(&state));

        state.record_call();
        assert!(!exactly.is_satisfied(&state));
        assert!(at_least.is_satisfied(&state));
    }

    #[test]
    fn test_mock_buffer() {
        let buf = MockBuffer::new(1);
        assert_eq!(buf.id, 1);
        assert!(buf.loaded);

        let buf_lines = MockBuffer::with_lines(2, 100);
        assert_eq!(buf_lines.line_count, 100);
    }

    #[test]
    fn test_mock_window() {
        let win = MockWindow::new(1, 1);
        assert_eq!(win.id, 1);
        assert_eq!(win.width, 80);
        assert_eq!(win.height, 24);
    }
}
