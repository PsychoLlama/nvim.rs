//! Logging Infrastructure
//!
//! This crate provides structured logging infrastructure for Neovim,
//! including log levels, formatters, and output destinations.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)] // FFI functions cannot be const
#![allow(clippy::cast_possible_wrap)] // u64 to i64 in statistics
#![allow(clippy::cast_sign_loss)] // c_int to usize for array indexing
#![allow(clippy::cast_possible_truncation)] // usize to c_int for lengths

pub mod level;
pub mod logger;
pub mod output;

use std::ffi::c_int;

// =============================================================================
// Log Entry
// =============================================================================

/// A log entry.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LogEntry {
    /// Log level
    pub level: c_int,
    /// Timestamp (microseconds since epoch)
    pub timestamp_us: i64,
    /// Source file hash (for identification)
    pub file_hash: u64,
    /// Source line number
    pub line: c_int,
    /// Message hash (for deduplication)
    pub msg_hash: u64,
    /// Message length
    pub msg_len: c_int,
    /// Sequence number
    pub seq: u64,
}

impl LogEntry {
    /// Create new log entry.
    #[must_use]
    pub const fn new(level: c_int, line: c_int) -> Self {
        Self {
            level,
            timestamp_us: 0,
            file_hash: 0,
            line,
            msg_hash: 0,
            msg_len: 0,
            seq: 0,
        }
    }

    /// Create entry with timestamp.
    #[must_use]
    pub const fn with_timestamp(level: c_int, line: c_int, timestamp_us: i64) -> Self {
        Self {
            level,
            timestamp_us,
            file_hash: 0,
            line,
            msg_hash: 0,
            msg_len: 0,
            seq: 0,
        }
    }
}

/// FFI: Create log entry.
#[no_mangle]
pub extern "C" fn rs_log_entry_new(level: c_int, line: c_int) -> LogEntry {
    LogEntry::new(level, line)
}

/// FFI: Create log entry with timestamp.
#[no_mangle]
pub extern "C" fn rs_log_entry_with_timestamp(
    level: c_int,
    line: c_int,
    timestamp_us: i64,
) -> LogEntry {
    LogEntry::with_timestamp(level, line, timestamp_us)
}

// =============================================================================
// Log Statistics
// =============================================================================

/// Statistics for logging.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LogStats {
    /// Total messages logged
    pub total_messages: u64,
    /// Messages by level (indices: DEBUG=0, INFO=1, WARN=2, ERROR=3)
    pub by_level: [u64; 4],
    /// Messages dropped due to filtering
    pub dropped: u64,
    /// Messages dropped due to rate limiting
    pub rate_limited: u64,
    /// Total bytes written
    pub bytes_written: u64,
    /// Write errors
    pub write_errors: u64,
}

impl LogStats {
    /// Create new stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_messages: 0,
            by_level: [0; 4],
            dropped: 0,
            rate_limited: 0,
            bytes_written: 0,
            write_errors: 0,
        }
    }

    /// Record a message.
    pub fn record_message(&mut self, level: c_int, bytes: u64) {
        self.total_messages += 1;
        self.bytes_written += bytes;
        let idx = level.clamp(0, 3) as usize;
        self.by_level[idx] += 1;
    }

    /// Record dropped message.
    pub fn record_dropped(&mut self) {
        self.dropped += 1;
    }

    /// Record rate limited message.
    pub fn record_rate_limited(&mut self) {
        self.rate_limited += 1;
    }

    /// Record write error.
    pub fn record_error(&mut self) {
        self.write_errors += 1;
    }

    /// Reset statistics.
    pub fn reset(&mut self) {
        self.total_messages = 0;
        self.by_level = [0; 4];
        self.dropped = 0;
        self.rate_limited = 0;
        self.bytes_written = 0;
        self.write_errors = 0;
    }
}

/// FFI: Create log stats.
#[no_mangle]
pub extern "C" fn rs_log_stats_new() -> LogStats {
    LogStats::new()
}

/// FFI: Record message.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_log_stats_record_message(
    stats: *mut LogStats,
    level: c_int,
    bytes: u64,
) {
    if !stats.is_null() {
        (*stats).record_message(level, bytes);
    }
}

/// FFI: Record dropped.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_log_stats_record_dropped(stats: *mut LogStats) {
    if !stats.is_null() {
        (*stats).record_dropped();
    }
}

/// FFI: Reset stats.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_log_stats_reset(stats: *mut LogStats) {
    if !stats.is_null() {
        (*stats).reset();
    }
}

// =============================================================================
// Log Context
// =============================================================================

/// Context for log messages.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LogContext {
    /// Component/module ID
    pub component_id: c_int,
    /// Thread ID (if applicable)
    pub thread_id: u64,
    /// Correlation ID (for request tracking)
    pub correlation_id: u64,
    /// User data
    pub user_data: u64,
}

impl LogContext {
    /// Create new context.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            component_id: 0,
            thread_id: 0,
            correlation_id: 0,
            user_data: 0,
        }
    }

    /// Create context for component.
    #[must_use]
    pub const fn for_component(component_id: c_int) -> Self {
        Self {
            component_id,
            thread_id: 0,
            correlation_id: 0,
            user_data: 0,
        }
    }
}

/// FFI: Create log context.
#[no_mangle]
pub extern "C" fn rs_log_context_new() -> LogContext {
    LogContext::new()
}

/// FFI: Create log context for component.
#[no_mangle]
pub extern "C" fn rs_log_context_for_component(component_id: c_int) -> LogContext {
    LogContext::for_component(component_id)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry() {
        let entry = LogEntry::new(1, 100);
        assert_eq!(entry.level, 1);
        assert_eq!(entry.line, 100);
        assert_eq!(entry.timestamp_us, 0);

        let entry_ts = LogEntry::with_timestamp(2, 50, 1234567890);
        assert_eq!(entry_ts.timestamp_us, 1234567890);
    }

    #[test]
    fn test_log_stats() {
        let mut stats = LogStats::new();
        assert_eq!(stats.total_messages, 0);

        stats.record_message(1, 100);
        assert_eq!(stats.total_messages, 1);
        assert_eq!(stats.bytes_written, 100);
        assert_eq!(stats.by_level[1], 1);

        stats.record_dropped();
        assert_eq!(stats.dropped, 1);

        stats.record_rate_limited();
        assert_eq!(stats.rate_limited, 1);

        stats.record_error();
        assert_eq!(stats.write_errors, 1);

        stats.reset();
        assert_eq!(stats.total_messages, 0);
    }

    #[test]
    fn test_log_context() {
        let ctx = LogContext::new();
        assert_eq!(ctx.component_id, 0);

        let ctx_comp = LogContext::for_component(42);
        assert_eq!(ctx_comp.component_id, 42);
    }
}
