//! Provider callback invocation
//!
//! This module provides infrastructure for invoking decoration provider
//! callbacks with proper error handling and context.

use std::ffi::c_int;

// =============================================================================
// Callback Type
// =============================================================================

/// Type of decoration provider callback.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CallbackType {
    /// Called when redraw starts
    #[default]
    OnStart = 0,
    /// Called for each buffer
    OnBuf = 1,
    /// Called for each window
    OnWin = 2,
    /// Called for each line
    OnLine = 3,
    /// Called when redraw ends
    OnEnd = 4,
}

impl CallbackType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::OnBuf,
            2 => Self::OnWin,
            3 => Self::OnLine,
            4 => Self::OnEnd,
            _ => Self::OnStart,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Get callback name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::OnStart => "on_start",
            Self::OnBuf => "on_buf",
            Self::OnWin => "on_win",
            Self::OnLine => "on_line",
            Self::OnEnd => "on_end",
        }
    }
}

/// FFI: Get callback name length.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn rs_callback_type_name_len(cb_type: c_int) -> c_int {
    CallbackType::from_c_int(cb_type).name().len() as c_int
}

// =============================================================================
// Callback Result
// =============================================================================

/// Result of callback invocation.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CallbackResult {
    /// Callback succeeded
    #[default]
    Ok = 0,
    /// Callback requested skip (e.g., skip this buffer)
    Skip = 1,
    /// Callback encountered error
    Error = 2,
    /// Callback timed out
    Timeout = 3,
    /// Callback not registered
    NotRegistered = 4,
    /// Provider suspended
    Suspended = 5,
}

impl CallbackResult {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Ok,
            1 => Self::Skip,
            2 => Self::Error,
            3 => Self::Timeout,
            4 => Self::NotRegistered,
            5 => Self::Suspended,
            _ => Self::Error,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if callback should continue.
    #[must_use]
    pub const fn should_continue(self) -> bool {
        matches!(self, Self::Ok)
    }

    /// Check if callback had error.
    #[must_use]
    pub const fn is_error(self) -> bool {
        matches!(self, Self::Error | Self::Timeout)
    }
}

/// FFI: Check if callback should continue.
#[no_mangle]
pub extern "C" fn rs_callback_result_should_continue(result: c_int) -> c_int {
    c_int::from(CallbackResult::from_c_int(result).should_continue())
}

/// FFI: Check if callback had error.
#[no_mangle]
pub extern "C" fn rs_callback_result_is_error(result: c_int) -> c_int {
    c_int::from(CallbackResult::from_c_int(result).is_error())
}

// =============================================================================
// Callback Context
// =============================================================================

/// Context for callback invocation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CallbackContext {
    /// Provider ID
    pub provider_id: c_int,
    /// Namespace ID
    pub ns_id: u64,
    /// Buffer ID (for on_buf, on_win, on_line)
    pub buf_id: c_int,
    /// Window ID (for on_win, on_line)
    pub win_id: c_int,
    /// Top visible row (for on_win)
    pub toprow: c_int,
    /// Bottom visible row (for on_win)
    pub botrow: c_int,
    /// Line number (for on_line)
    pub lnum: c_int,
    /// Redraw tick
    pub tick: u64,
}

impl CallbackContext {
    /// Create context for on_start.
    #[must_use]
    pub const fn on_start(provider_id: c_int, ns_id: u64, tick: u64) -> Self {
        Self {
            provider_id,
            ns_id,
            buf_id: 0,
            win_id: 0,
            toprow: 0,
            botrow: 0,
            lnum: 0,
            tick,
        }
    }

    /// Create context for on_buf.
    #[must_use]
    pub const fn on_buf(provider_id: c_int, ns_id: u64, buf_id: c_int, tick: u64) -> Self {
        Self {
            provider_id,
            ns_id,
            buf_id,
            win_id: 0,
            toprow: 0,
            botrow: 0,
            lnum: 0,
            tick,
        }
    }

    /// Create context for on_win.
    #[must_use]
    pub const fn on_win(
        provider_id: c_int,
        ns_id: u64,
        buf_id: c_int,
        win_id: c_int,
        toprow: c_int,
        botrow: c_int,
        tick: u64,
    ) -> Self {
        Self {
            provider_id,
            ns_id,
            buf_id,
            win_id,
            toprow,
            botrow,
            lnum: 0,
            tick,
        }
    }

    /// Create context for on_line.
    #[must_use]
    pub const fn on_line(
        provider_id: c_int,
        ns_id: u64,
        buf_id: c_int,
        win_id: c_int,
        lnum: c_int,
        tick: u64,
    ) -> Self {
        Self {
            provider_id,
            ns_id,
            buf_id,
            win_id,
            toprow: 0,
            botrow: 0,
            lnum,
            tick,
        }
    }
}

/// FFI: Create on_start context.
#[no_mangle]
pub extern "C" fn rs_callback_context_on_start(
    provider_id: c_int,
    ns_id: u64,
    tick: u64,
) -> CallbackContext {
    CallbackContext::on_start(provider_id, ns_id, tick)
}

/// FFI: Create on_buf context.
#[no_mangle]
pub extern "C" fn rs_callback_context_on_buf(
    provider_id: c_int,
    ns_id: u64,
    buf_id: c_int,
    tick: u64,
) -> CallbackContext {
    CallbackContext::on_buf(provider_id, ns_id, buf_id, tick)
}

/// FFI: Create on_win context.
#[no_mangle]
pub extern "C" fn rs_callback_context_on_win(
    provider_id: c_int,
    ns_id: u64,
    buf_id: c_int,
    win_id: c_int,
    toprow: c_int,
    botrow: c_int,
    tick: u64,
) -> CallbackContext {
    CallbackContext::on_win(provider_id, ns_id, buf_id, win_id, toprow, botrow, tick)
}

/// FFI: Create on_line context.
#[no_mangle]
pub extern "C" fn rs_callback_context_on_line(
    provider_id: c_int,
    ns_id: u64,
    buf_id: c_int,
    win_id: c_int,
    lnum: c_int,
    tick: u64,
) -> CallbackContext {
    CallbackContext::on_line(provider_id, ns_id, buf_id, win_id, lnum, tick)
}

// =============================================================================
// Invocation Record
// =============================================================================

/// Record of a callback invocation.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InvocationRecord {
    /// Callback type
    pub cb_type: c_int,
    /// Result
    pub result: c_int,
    /// Duration (microseconds)
    pub duration_us: i64,
    /// Error code (if error)
    pub error_code: c_int,
    /// Context
    pub context: CallbackContext,
}

impl InvocationRecord {
    /// Create new record.
    #[must_use]
    pub const fn new(cb_type: CallbackType, context: CallbackContext) -> Self {
        Self {
            cb_type: cb_type as c_int,
            result: CallbackResult::Ok as c_int,
            duration_us: 0,
            error_code: 0,
            context,
        }
    }

    /// Get callback type.
    #[must_use]
    pub const fn get_type(&self) -> CallbackType {
        CallbackType::from_c_int(self.cb_type)
    }

    /// Get result.
    #[must_use]
    pub const fn get_result(&self) -> CallbackResult {
        CallbackResult::from_c_int(self.result)
    }

    /// Complete with success.
    pub fn complete_ok(&mut self, duration_us: i64) {
        self.result = CallbackResult::Ok as c_int;
        self.duration_us = duration_us;
    }

    /// Complete with skip.
    pub fn complete_skip(&mut self, duration_us: i64) {
        self.result = CallbackResult::Skip as c_int;
        self.duration_us = duration_us;
    }

    /// Complete with error.
    pub fn complete_error(&mut self, duration_us: i64, error_code: c_int) {
        self.result = CallbackResult::Error as c_int;
        self.duration_us = duration_us;
        self.error_code = error_code;
    }
}

/// FFI: Create invocation record.
#[no_mangle]
pub extern "C" fn rs_invocation_record_new(
    cb_type: c_int,
    context: CallbackContext,
) -> InvocationRecord {
    InvocationRecord::new(CallbackType::from_c_int(cb_type), context)
}

/// FFI: Complete record with success.
///
/// # Safety
/// `record` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_invocation_complete_ok(
    record: *mut InvocationRecord,
    duration_us: i64,
) {
    if !record.is_null() {
        (*record).complete_ok(duration_us);
    }
}

/// FFI: Complete record with error.
///
/// # Safety
/// `record` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_invocation_complete_error(
    record: *mut InvocationRecord,
    duration_us: i64,
    error_code: c_int,
) {
    if !record.is_null() {
        (*record).complete_error(duration_us, error_code);
    }
}

// =============================================================================
// Invocation Statistics
// =============================================================================

/// Statistics for callback invocations.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InvocationStats {
    /// Counts by callback type
    pub counts: [u64; 5],
    /// Total time by callback type (microseconds)
    pub times: [i64; 5],
    /// Errors by callback type
    pub errors: [u64; 5],
    /// Skips by callback type
    pub skips: [u64; 5],
}

impl InvocationStats {
    /// Create new stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            counts: [0; 5],
            times: [0; 5],
            errors: [0; 5],
            skips: [0; 5],
        }
    }

    /// Record an invocation.
    #[allow(clippy::cast_sign_loss)]
    pub fn record(&mut self, record: &InvocationRecord) {
        let idx = record.cb_type.clamp(0, 4) as usize;
        self.counts[idx] += 1;
        self.times[idx] += record.duration_us;

        match record.get_result() {
            CallbackResult::Error | CallbackResult::Timeout => {
                self.errors[idx] += 1;
            }
            CallbackResult::Skip => {
                self.skips[idx] += 1;
            }
            _ => {}
        }
    }

    /// Get total invocations.
    #[must_use]
    pub const fn total_count(&self) -> u64 {
        self.counts[0] + self.counts[1] + self.counts[2] + self.counts[3] + self.counts[4]
    }

    /// Get total errors.
    #[must_use]
    pub const fn total_errors(&self) -> u64 {
        self.errors[0] + self.errors[1] + self.errors[2] + self.errors[3] + self.errors[4]
    }

    /// Reset statistics.
    pub fn reset(&mut self) {
        self.counts = [0; 5];
        self.times = [0; 5];
        self.errors = [0; 5];
        self.skips = [0; 5];
    }
}

/// FFI: Create invocation stats.
#[no_mangle]
pub extern "C" fn rs_invocation_stats_new() -> InvocationStats {
    InvocationStats::new()
}

/// FFI: Record invocation.
///
/// # Safety
/// Both `stats` and `record` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_invocation_stats_record(
    stats: *mut InvocationStats,
    record: *const InvocationRecord,
) {
    if !stats.is_null() && !record.is_null() {
        (*stats).record(&*record);
    }
}

/// FFI: Get total count.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_invocation_stats_total(stats: *const InvocationStats) -> u64 {
    if stats.is_null() {
        return 0;
    }
    (*stats).total_count()
}

/// FFI: Reset stats.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_invocation_stats_reset(stats: *mut InvocationStats) {
    if !stats.is_null() {
        (*stats).reset();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_callback_type() {
        assert_eq!(CallbackType::OnStart.name(), "on_start");
        assert_eq!(CallbackType::OnBuf.name(), "on_buf");
        assert_eq!(CallbackType::OnWin.name(), "on_win");
        assert_eq!(CallbackType::OnLine.name(), "on_line");
        assert_eq!(CallbackType::OnEnd.name(), "on_end");
    }

    #[test]
    fn test_callback_result() {
        assert!(CallbackResult::Ok.should_continue());
        assert!(!CallbackResult::Skip.should_continue());
        assert!(!CallbackResult::Error.should_continue());

        assert!(CallbackResult::Error.is_error());
        assert!(CallbackResult::Timeout.is_error());
        assert!(!CallbackResult::Ok.is_error());
    }

    #[test]
    fn test_callback_context() {
        let ctx = CallbackContext::on_start(1, 100, 1000);
        assert_eq!(ctx.provider_id, 1);
        assert_eq!(ctx.ns_id, 100);
        assert_eq!(ctx.tick, 1000);

        let ctx = CallbackContext::on_win(1, 100, 5, 10, 0, 50, 1000);
        assert_eq!(ctx.buf_id, 5);
        assert_eq!(ctx.win_id, 10);
        assert_eq!(ctx.toprow, 0);
        assert_eq!(ctx.botrow, 50);

        let ctx = CallbackContext::on_line(1, 100, 5, 10, 25, 1000);
        assert_eq!(ctx.lnum, 25);
    }

    #[test]
    fn test_invocation_record() {
        let ctx = CallbackContext::on_start(1, 100, 1000);
        let mut record = InvocationRecord::new(CallbackType::OnStart, ctx);
        assert_eq!(record.get_type(), CallbackType::OnStart);
        assert_eq!(record.get_result(), CallbackResult::Ok);

        record.complete_ok(50);
        assert_eq!(record.duration_us, 50);
        assert_eq!(record.get_result(), CallbackResult::Ok);

        record.complete_error(100, 42);
        assert_eq!(record.error_code, 42);
        assert_eq!(record.get_result(), CallbackResult::Error);
    }

    #[test]
    fn test_invocation_stats() {
        let mut stats = InvocationStats::new();
        assert_eq!(stats.total_count(), 0);

        let ctx = CallbackContext::on_start(1, 100, 1000);
        let mut record = InvocationRecord::new(CallbackType::OnStart, ctx);
        record.complete_ok(50);
        stats.record(&record);

        assert_eq!(stats.total_count(), 1);
        assert_eq!(stats.counts[0], 1);
        assert_eq!(stats.times[0], 50);

        let ctx = CallbackContext::on_line(1, 100, 5, 10, 25, 1000);
        let mut record = InvocationRecord::new(CallbackType::OnLine, ctx);
        record.complete_error(100, 42);
        stats.record(&record);

        assert_eq!(stats.total_count(), 2);
        assert_eq!(stats.total_errors(), 1);
        assert_eq!(stats.errors[3], 1);

        stats.reset();
        assert_eq!(stats.total_count(), 0);
    }
}
