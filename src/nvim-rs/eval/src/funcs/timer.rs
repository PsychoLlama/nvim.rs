//! Timer functions for VimL.
//!
//! This module implements timer-related VimL functions from `src/nvim/eval/funcs.c`:
//! - Timer ID management
//! - Timer state types
//! - Timer validation helpers
//!
//! ## Note
//!
//! These are helper types for timer operations.
//! Actual timer scheduling uses the event loop infrastructure.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_int;

// =============================================================================
// Timer ID
// =============================================================================

/// Timer ID type.
pub type TimerId = i64;

/// Invalid timer ID.
pub const INVALID_TIMER_ID: TimerId = -1;

/// Check if a timer ID is valid.
pub const fn is_valid_timer_id(id: TimerId) -> bool {
    id >= 0
}

/// FFI export: validate timer ID.
#[no_mangle]
pub extern "C" fn rs_is_valid_timer_id(id: TimerId) -> bool {
    is_valid_timer_id(id)
}

// =============================================================================
// Timer State
// =============================================================================

/// Timer state enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TimerState {
    /// Timer is active and will fire
    Active = 0,
    /// Timer is paused
    Paused = 1,
    /// Timer has been stopped/removed
    Stopped = 2,
    /// Timer callback is currently executing
    Running = 3,
}

impl TimerState {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Active,
            1 => Self::Paused,
            2 => Self::Stopped,
            3 => Self::Running,
            _ => Self::Stopped,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if timer can fire.
    pub const fn can_fire(&self) -> bool {
        matches!(self, Self::Active)
    }

    /// Check if timer is alive (not stopped).
    pub const fn is_alive(&self) -> bool {
        !matches!(self, Self::Stopped)
    }
}

/// FFI export: check if timer can fire.
#[no_mangle]
pub extern "C" fn rs_timer_state_can_fire(state: c_int) -> bool {
    TimerState::from_c_int(state).can_fire()
}

/// FFI export: check if timer is alive.
#[no_mangle]
pub extern "C" fn rs_timer_state_is_alive(state: c_int) -> bool {
    TimerState::from_c_int(state).is_alive()
}

// =============================================================================
// Timer Info
// =============================================================================

/// Timer information (for timer_info()).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct TimerInfo {
    /// Timer ID
    pub id: TimerId,
    /// Milliseconds remaining until next fire
    pub remaining: i64,
    /// Repeat count (-1 for infinite)
    pub repeat: i64,
    /// Original timeout in milliseconds
    pub time: i64,
    /// Whether timer is paused
    pub paused: bool,
}

impl TimerInfo {
    /// Create new timer info.
    pub const fn new(id: TimerId, time: i64, repeat: i64) -> Self {
        Self {
            id,
            remaining: time,
            repeat,
            time,
            paused: false,
        }
    }

    /// Check if this is a one-shot timer.
    pub const fn is_oneshot(&self) -> bool {
        self.repeat == 0
    }

    /// Check if this timer repeats forever.
    pub const fn is_infinite(&self) -> bool {
        self.repeat < 0
    }

    /// Get remaining repeats (0 for finished, -1 for infinite).
    pub const fn remaining_repeats(&self) -> i64 {
        self.repeat
    }
}

impl Default for TimerInfo {
    fn default() -> Self {
        Self {
            id: INVALID_TIMER_ID,
            remaining: 0,
            repeat: 0,
            time: 0,
            paused: false,
        }
    }
}

/// FFI export: create timer info.
#[no_mangle]
pub extern "C" fn rs_timer_info_new(id: TimerId, time: i64, repeat: i64) -> TimerInfo {
    TimerInfo::new(id, time, repeat)
}

/// FFI export: check if timer is one-shot.
#[no_mangle]
pub extern "C" fn rs_timer_info_is_oneshot(info: *const TimerInfo) -> bool {
    if info.is_null() {
        return true;
    }
    unsafe { (*info).is_oneshot() }
}

// =============================================================================
// Timer Validation
// =============================================================================

/// Minimum timer interval in milliseconds.
pub const MIN_TIMER_INTERVAL: i64 = 0;

/// Maximum timer interval in milliseconds (about 24 days).
pub const MAX_TIMER_INTERVAL: i64 = 2_147_483_647;

/// Validate timer interval.
pub const fn is_valid_timer_interval(ms: i64) -> bool {
    ms >= MIN_TIMER_INTERVAL && ms <= MAX_TIMER_INTERVAL
}

/// Validate repeat count (-1 for infinite, 0+ for finite).
pub const fn is_valid_repeat_count(count: i64) -> bool {
    count >= -1
}

/// FFI export: validate timer interval.
#[no_mangle]
pub extern "C" fn rs_is_valid_timer_interval(ms: i64) -> bool {
    is_valid_timer_interval(ms)
}

/// FFI export: validate repeat count.
#[no_mangle]
pub extern "C" fn rs_is_valid_repeat_count(count: i64) -> bool {
    is_valid_repeat_count(count)
}

// =============================================================================
// Wait/Defer Helpers
// =============================================================================

/// Maximum wait timeout in milliseconds (about 24 days).
pub const MAX_WAIT_TIMEOUT: i64 = 2_147_483_647;

/// Infinite wait timeout.
pub const WAIT_FOREVER: i64 = -1;

/// Wait result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum WaitResult {
    /// Condition was satisfied
    Ok = 0,
    /// Timeout expired
    Timeout = -1,
    /// Wait was interrupted
    Interrupted = -2,
    /// Invalid arguments
    Error = -3,
}

impl WaitResult {
    /// Create from C int.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Ok,
            -1 => Self::Timeout,
            -2 => Self::Interrupted,
            _ => Self::Error,
        }
    }

    /// Convert to C int.
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if wait succeeded.
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

/// FFI export: check if wait succeeded.
#[no_mangle]
pub extern "C" fn rs_wait_result_is_ok(result: c_int) -> bool {
    WaitResult::from_c_int(result).is_ok()
}

/// Validate wait timeout.
pub const fn is_valid_wait_timeout(ms: i64) -> bool {
    ms == WAIT_FOREVER || (ms >= 0 && ms <= MAX_WAIT_TIMEOUT)
}

/// FFI export: validate wait timeout.
#[no_mangle]
pub extern "C" fn rs_is_valid_wait_timeout(ms: i64) -> bool {
    is_valid_wait_timeout(ms)
}

// =============================================================================
// Defer Helpers
// =============================================================================

/// Deferred callback info.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DeferInfo {
    /// Whether defer is pending
    pub pending: bool,
    /// Whether callback has executed
    pub executed: bool,
    /// Error if callback failed
    pub error: bool,
}

impl DeferInfo {
    /// Create new pending defer.
    pub const fn pending() -> Self {
        Self {
            pending: true,
            executed: false,
            error: false,
        }
    }

    /// Mark as executed.
    pub const fn executed() -> Self {
        Self {
            pending: false,
            executed: true,
            error: false,
        }
    }

    /// Mark as error.
    pub const fn errored() -> Self {
        Self {
            pending: false,
            executed: false,
            error: true,
        }
    }

    /// Check if still pending.
    pub const fn is_pending(&self) -> bool {
        self.pending
    }
}

/// FFI export: create pending defer info.
#[no_mangle]
pub extern "C" fn rs_defer_info_pending() -> DeferInfo {
    DeferInfo::pending()
}

/// FFI export: check if defer is pending.
#[no_mangle]
pub extern "C" fn rs_defer_info_is_pending(info: *const DeferInfo) -> bool {
    if info.is_null() {
        return false;
    }
    unsafe { (*info).is_pending() }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_id() {
        assert!(is_valid_timer_id(0));
        assert!(is_valid_timer_id(100));
        assert!(!is_valid_timer_id(-1));
        assert!(!is_valid_timer_id(INVALID_TIMER_ID));
    }

    #[test]
    fn test_timer_state() {
        assert_eq!(TimerState::from_c_int(0), TimerState::Active);
        assert_eq!(TimerState::from_c_int(1), TimerState::Paused);
        assert_eq!(TimerState::from_c_int(2), TimerState::Stopped);

        assert!(TimerState::Active.can_fire());
        assert!(!TimerState::Paused.can_fire());
        assert!(!TimerState::Stopped.can_fire());

        assert!(TimerState::Active.is_alive());
        assert!(TimerState::Paused.is_alive());
        assert!(!TimerState::Stopped.is_alive());
    }

    #[test]
    fn test_timer_info() {
        let info = TimerInfo::new(1, 1000, 5);
        assert_eq!(info.id, 1);
        assert_eq!(info.time, 1000);
        assert_eq!(info.repeat, 5);
        assert!(!info.is_oneshot());
        assert!(!info.is_infinite());

        let oneshot = TimerInfo::new(2, 500, 0);
        assert!(oneshot.is_oneshot());

        let infinite = TimerInfo::new(3, 100, -1);
        assert!(infinite.is_infinite());
    }

    #[test]
    fn test_timer_validation() {
        assert!(is_valid_timer_interval(0));
        assert!(is_valid_timer_interval(1000));
        assert!(is_valid_timer_interval(MAX_TIMER_INTERVAL));
        assert!(!is_valid_timer_interval(-1));

        assert!(is_valid_repeat_count(-1)); // infinite
        assert!(is_valid_repeat_count(0)); // one-shot
        assert!(is_valid_repeat_count(10));
        assert!(!is_valid_repeat_count(-2));
    }

    #[test]
    fn test_wait_result() {
        assert!(WaitResult::Ok.is_ok());
        assert!(!WaitResult::Timeout.is_ok());
        assert!(!WaitResult::Interrupted.is_ok());

        assert_eq!(WaitResult::from_c_int(0), WaitResult::Ok);
        assert_eq!(WaitResult::from_c_int(-1), WaitResult::Timeout);
    }

    #[test]
    fn test_wait_timeout() {
        assert!(is_valid_wait_timeout(0));
        assert!(is_valid_wait_timeout(1000));
        assert!(is_valid_wait_timeout(WAIT_FOREVER));
        assert!(!is_valid_wait_timeout(-2));
    }

    #[test]
    fn test_defer_info() {
        let pending = DeferInfo::pending();
        assert!(pending.is_pending());
        assert!(!pending.executed);

        let executed = DeferInfo::executed();
        assert!(!executed.is_pending());
        assert!(executed.executed);

        let errored = DeferInfo::errored();
        assert!(!errored.is_pending());
        assert!(errored.error);
    }
}
