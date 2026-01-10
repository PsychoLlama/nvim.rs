//! Core logging infrastructure
//!
//! This module provides the core logger implementation
//! for Neovim's logging system.

use std::ffi::c_int;

use crate::level::{LevelConfig, LogFilter, LogLevel};
use crate::{LogContext, LogEntry, LogStats};

// =============================================================================
// Logger State
// =============================================================================

/// State of the logger.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoggerState {
    /// Logger not initialized
    #[default]
    Uninitialized = 0,
    /// Logger is initializing
    Initializing = 1,
    /// Logger is ready
    Ready = 2,
    /// Logger is flushing
    Flushing = 3,
    /// Logger is shutting down
    ShuttingDown = 4,
    /// Logger is stopped
    Stopped = 5,
}

impl LoggerState {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Initializing,
            2 => Self::Ready,
            3 => Self::Flushing,
            4 => Self::ShuttingDown,
            5 => Self::Stopped,
            _ => Self::Uninitialized,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if logger can accept messages.
    #[must_use]
    pub const fn can_log(self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// FFI: Check if logger can log.
#[no_mangle]
pub extern "C" fn rs_logger_state_can_log(state: c_int) -> c_int {
    c_int::from(LoggerState::from_c_int(state).can_log())
}

// =============================================================================
// Logger Configuration
// =============================================================================

/// Logger configuration.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LoggerConfig {
    /// Filter configuration
    pub filter: LogFilter,
    /// Level configuration
    pub level_config: LevelConfig,
    /// Buffer size for log messages
    pub buffer_size: c_int,
    /// Flush interval (milliseconds)
    pub flush_interval_ms: c_int,
    /// Enable async logging
    pub async_logging: bool,
    /// Enable structured logging (JSON)
    pub structured: bool,
    /// Enable colors in output
    pub colors: bool,
}

impl LoggerConfig {
    /// Create default configuration.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            filter: LogFilter::pass_all(),
            level_config: LevelConfig::new(LogLevel::Info),
            buffer_size: 4096,
            flush_interval_ms: 100,
            async_logging: false,
            structured: false,
            colors: true,
        }
    }

    /// Create production configuration.
    #[must_use]
    pub const fn production() -> Self {
        Self {
            filter: LogFilter::production(),
            level_config: LevelConfig::new(LogLevel::Info),
            buffer_size: 8192,
            flush_interval_ms: 1000,
            async_logging: true,
            structured: false,
            colors: false,
        }
    }

    /// Create debug configuration.
    #[must_use]
    pub const fn debug() -> Self {
        Self {
            filter: LogFilter::debug(),
            level_config: LevelConfig::new(LogLevel::Debug),
            buffer_size: 16384,
            flush_interval_ms: 0,
            async_logging: false,
            structured: false,
            colors: true,
        }
    }
}

/// FFI: Create default logger config.
#[no_mangle]
pub extern "C" fn rs_logger_config_new() -> LoggerConfig {
    LoggerConfig::new()
}

/// FFI: Create production logger config.
#[no_mangle]
pub extern "C" fn rs_logger_config_production() -> LoggerConfig {
    LoggerConfig::production()
}

/// FFI: Create debug logger config.
#[no_mangle]
pub extern "C" fn rs_logger_config_debug() -> LoggerConfig {
    LoggerConfig::debug()
}

// =============================================================================
// Logger Info
// =============================================================================

/// Information about the logger.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LoggerInfo {
    /// Current state
    pub state: c_int,
    /// Configuration
    pub config: LoggerConfig,
    /// Statistics
    pub stats: LogStats,
    /// Start time (unix timestamp)
    pub start_time: i64,
    /// Last flush time
    pub last_flush_time: i64,
    /// Pending messages count
    pub pending_count: c_int,
}

impl LoggerInfo {
    /// Create new logger info.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: LoggerState::Uninitialized as c_int,
            config: LoggerConfig::new(),
            stats: LogStats::new(),
            start_time: 0,
            last_flush_time: 0,
            pending_count: 0,
        }
    }

    /// Get state.
    #[must_use]
    pub const fn get_state(&self) -> LoggerState {
        LoggerState::from_c_int(self.state)
    }

    /// Set state.
    pub fn set_state(&mut self, state: LoggerState) {
        self.state = state as c_int;
    }

    /// Check if can log.
    #[must_use]
    pub const fn can_log(&self) -> bool {
        self.get_state().can_log()
    }
}

/// FFI: Create logger info.
#[no_mangle]
pub extern "C" fn rs_logger_info_new() -> LoggerInfo {
    LoggerInfo::new()
}

/// FFI: Check if can log.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_logger_info_can_log(info: *const LoggerInfo) -> c_int {
    if info.is_null() {
        return 0;
    }
    c_int::from((*info).can_log())
}

// =============================================================================
// Log Message Builder
// =============================================================================

/// Builder for log messages.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LogMessageBuilder {
    /// Entry being built
    pub entry: LogEntry,
    /// Context
    pub context: LogContext,
    /// Is valid
    pub valid: bool,
}

impl LogMessageBuilder {
    /// Create new builder.
    #[must_use]
    pub const fn new(level: LogLevel) -> Self {
        Self {
            entry: LogEntry::new(level as c_int, 0),
            context: LogContext::new(),
            valid: true,
        }
    }

    /// Set line number.
    #[must_use]
    pub const fn line(mut self, line: c_int) -> Self {
        self.entry.line = line;
        self
    }

    /// Set timestamp.
    #[must_use]
    pub const fn timestamp(mut self, timestamp_us: i64) -> Self {
        self.entry.timestamp_us = timestamp_us;
        self
    }

    /// Set component.
    #[must_use]
    pub const fn component(mut self, component_id: c_int) -> Self {
        self.context.component_id = component_id;
        self
    }

    /// Set correlation ID.
    #[must_use]
    pub const fn correlation(mut self, correlation_id: u64) -> Self {
        self.context.correlation_id = correlation_id;
        self
    }

    /// Build the entry and context.
    #[must_use]
    pub const fn build(self) -> (LogEntry, LogContext) {
        (self.entry, self.context)
    }
}

/// FFI: Create message builder.
#[no_mangle]
pub extern "C" fn rs_log_message_builder_new(level: c_int) -> LogMessageBuilder {
    LogMessageBuilder::new(LogLevel::from_c_int(level))
}

/// FFI: Set builder line.
#[no_mangle]
pub extern "C" fn rs_log_message_builder_line(
    builder: LogMessageBuilder,
    line: c_int,
) -> LogMessageBuilder {
    builder.line(line)
}

/// FFI: Set builder timestamp.
#[no_mangle]
pub extern "C" fn rs_log_message_builder_timestamp(
    builder: LogMessageBuilder,
    timestamp_us: i64,
) -> LogMessageBuilder {
    builder.timestamp(timestamp_us)
}

/// FFI: Set builder component.
#[no_mangle]
pub extern "C" fn rs_log_message_builder_component(
    builder: LogMessageBuilder,
    component_id: c_int,
) -> LogMessageBuilder {
    builder.component(component_id)
}

// =============================================================================
// Rate Limiter
// =============================================================================

/// Rate limiter for log messages.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LogRateLimiter {
    /// Maximum messages per window
    pub max_messages: c_int,
    /// Window duration (milliseconds)
    pub window_ms: c_int,
    /// Current window start
    pub window_start: i64,
    /// Messages in current window
    pub current_count: c_int,
    /// Total messages limited
    pub total_limited: u64,
}

impl LogRateLimiter {
    /// Create new rate limiter.
    #[must_use]
    pub const fn new(max_messages: c_int, window_ms: c_int) -> Self {
        Self {
            max_messages,
            window_ms,
            window_start: 0,
            current_count: 0,
            total_limited: 0,
        }
    }

    /// Create disabled limiter.
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            max_messages: 0,
            window_ms: 0,
            window_start: 0,
            current_count: 0,
            total_limited: 0,
        }
    }

    /// Check if rate limiting is enabled.
    #[must_use]
    pub const fn is_enabled(&self) -> bool {
        self.max_messages > 0 && self.window_ms > 0
    }

    /// Check if message should be allowed (and update state).
    pub fn should_allow(&mut self, current_time_ms: i64) -> bool {
        if !self.is_enabled() {
            return true;
        }

        // Check if we're in a new window
        if current_time_ms - self.window_start >= i64::from(self.window_ms) {
            self.window_start = current_time_ms;
            self.current_count = 0;
        }

        if self.current_count < self.max_messages {
            self.current_count += 1;
            true
        } else {
            self.total_limited += 1;
            false
        }
    }

    /// Reset the limiter.
    pub fn reset(&mut self) {
        self.window_start = 0;
        self.current_count = 0;
    }
}

/// FFI: Create rate limiter.
#[no_mangle]
pub extern "C" fn rs_log_rate_limiter_new(max_messages: c_int, window_ms: c_int) -> LogRateLimiter {
    LogRateLimiter::new(max_messages, window_ms)
}

/// FFI: Create disabled limiter.
#[no_mangle]
pub extern "C" fn rs_log_rate_limiter_disabled() -> LogRateLimiter {
    LogRateLimiter::disabled()
}

/// FFI: Check if should allow.
///
/// # Safety
/// `limiter` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_log_rate_limiter_allow(
    limiter: *mut LogRateLimiter,
    current_time_ms: i64,
) -> c_int {
    if limiter.is_null() {
        return 1;
    }
    c_int::from((*limiter).should_allow(current_time_ms))
}

/// FFI: Reset limiter.
///
/// # Safety
/// `limiter` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_log_rate_limiter_reset(limiter: *mut LogRateLimiter) {
    if !limiter.is_null() {
        (*limiter).reset();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_state() {
        assert!(!LoggerState::Uninitialized.can_log());
        assert!(LoggerState::Ready.can_log());
        assert!(!LoggerState::ShuttingDown.can_log());
    }

    #[test]
    fn test_logger_config() {
        let config = LoggerConfig::new();
        assert!(config.colors);
        assert!(!config.async_logging);

        let prod = LoggerConfig::production();
        assert!(prod.async_logging);
        assert!(!prod.colors);

        let debug = LoggerConfig::debug();
        assert_eq!(debug.flush_interval_ms, 0);
    }

    #[test]
    fn test_logger_info() {
        let mut info = LoggerInfo::new();
        assert!(!info.can_log());

        info.set_state(LoggerState::Ready);
        assert!(info.can_log());
    }

    #[test]
    fn test_message_builder() {
        let builder = LogMessageBuilder::new(LogLevel::Info)
            .line(42)
            .timestamp(1_234_567_890)
            .component(5);

        let (entry, context) = builder.build();
        assert_eq!(entry.line, 42);
        assert_eq!(entry.timestamp_us, 1_234_567_890);
        assert_eq!(context.component_id, 5);
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = LogRateLimiter::new(3, 1000);
        assert!(limiter.is_enabled());

        // First 3 should pass
        assert!(limiter.should_allow(0));
        assert!(limiter.should_allow(100));
        assert!(limiter.should_allow(200));

        // Fourth should fail
        assert!(!limiter.should_allow(300));
        assert_eq!(limiter.total_limited, 1);

        // New window
        assert!(limiter.should_allow(1000));
        assert_eq!(limiter.current_count, 1);

        // Disabled limiter
        let mut disabled = LogRateLimiter::disabled();
        assert!(!disabled.is_enabled());
        assert!(disabled.should_allow(0));
    }
}
