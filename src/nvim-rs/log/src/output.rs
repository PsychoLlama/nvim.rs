//! Log output destinations
//!
//! This module provides output destination infrastructure
//! for routing log messages to various targets.

use std::ffi::c_int;

// =============================================================================
// Output Type
// =============================================================================

/// Type of log output destination.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputType {
    /// No output
    #[default]
    None = 0,
    /// Standard output (stdout)
    Stdout = 1,
    /// Standard error (stderr)
    Stderr = 2,
    /// File output
    File = 3,
    /// Syslog
    Syslog = 4,
    /// Memory buffer
    Buffer = 5,
    /// Callback function
    Callback = 6,
    /// Network (remote logging)
    Network = 7,
}

impl OutputType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Stdout,
            2 => Self::Stderr,
            3 => Self::File,
            4 => Self::Syslog,
            5 => Self::Buffer,
            6 => Self::Callback,
            7 => Self::Network,
            _ => Self::None,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if output is a stream type.
    #[must_use]
    pub const fn is_stream(self) -> bool {
        matches!(self, Self::Stdout | Self::Stderr)
    }

    /// Check if output requires close/cleanup.
    #[must_use]
    pub const fn requires_cleanup(self) -> bool {
        matches!(self, Self::File | Self::Network)
    }
}

/// FFI: Check if output is stream.
#[no_mangle]
pub extern "C" fn rs_output_type_is_stream(output_type: c_int) -> c_int {
    c_int::from(OutputType::from_c_int(output_type).is_stream())
}

/// FFI: Check if output requires cleanup.
#[no_mangle]
pub extern "C" fn rs_output_type_requires_cleanup(output_type: c_int) -> c_int {
    c_int::from(OutputType::from_c_int(output_type).requires_cleanup())
}

// =============================================================================
// Output State
// =============================================================================

/// State of an output destination.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputState {
    /// Output not initialized
    #[default]
    Closed = 0,
    /// Output is opening
    Opening = 1,
    /// Output is ready
    Open = 2,
    /// Output has error
    Error = 3,
    /// Output is closing
    Closing = 4,
}

impl OutputState {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Opening,
            2 => Self::Open,
            3 => Self::Error,
            4 => Self::Closing,
            _ => Self::Closed,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if output can write.
    #[must_use]
    pub const fn can_write(self) -> bool {
        matches!(self, Self::Open)
    }
}

/// FFI: Check if output can write.
#[no_mangle]
pub extern "C" fn rs_output_state_can_write(state: c_int) -> c_int {
    c_int::from(OutputState::from_c_int(state).can_write())
}

// =============================================================================
// Output Configuration
// =============================================================================

/// Configuration for an output destination.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct OutputConfig {
    /// Output type
    pub output_type: c_int,
    /// Buffer size
    pub buffer_size: c_int,
    /// Auto flush
    pub auto_flush: bool,
    /// Include newlines
    pub newlines: bool,
    /// Maximum file size (for rotation, 0 = no limit)
    pub max_file_size: u64,
    /// Maximum files to keep (for rotation)
    pub max_files: c_int,
}

impl OutputConfig {
    /// Create config for stdout.
    #[must_use]
    pub const fn stdout() -> Self {
        Self {
            output_type: OutputType::Stdout as c_int,
            buffer_size: 0,
            auto_flush: true,
            newlines: true,
            max_file_size: 0,
            max_files: 0,
        }
    }

    /// Create config for stderr.
    #[must_use]
    pub const fn stderr() -> Self {
        Self {
            output_type: OutputType::Stderr as c_int,
            buffer_size: 0,
            auto_flush: true,
            newlines: true,
            max_file_size: 0,
            max_files: 0,
        }
    }

    /// Create config for file.
    #[must_use]
    pub const fn file(buffer_size: c_int) -> Self {
        Self {
            output_type: OutputType::File as c_int,
            buffer_size,
            auto_flush: false,
            newlines: true,
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_files: 5,
        }
    }

    /// Create config for buffer.
    #[must_use]
    pub const fn buffer(buffer_size: c_int) -> Self {
        Self {
            output_type: OutputType::Buffer as c_int,
            buffer_size,
            auto_flush: false,
            newlines: true,
            max_file_size: 0,
            max_files: 0,
        }
    }

    /// Get output type.
    #[must_use]
    pub const fn get_type(&self) -> OutputType {
        OutputType::from_c_int(self.output_type)
    }
}

/// FFI: Create stdout config.
#[no_mangle]
pub extern "C" fn rs_output_config_stdout() -> OutputConfig {
    OutputConfig::stdout()
}

/// FFI: Create stderr config.
#[no_mangle]
pub extern "C" fn rs_output_config_stderr() -> OutputConfig {
    OutputConfig::stderr()
}

/// FFI: Create file config.
#[no_mangle]
pub extern "C" fn rs_output_config_file(buffer_size: c_int) -> OutputConfig {
    OutputConfig::file(buffer_size)
}

/// FFI: Create buffer config.
#[no_mangle]
pub extern "C" fn rs_output_config_buffer(buffer_size: c_int) -> OutputConfig {
    OutputConfig::buffer(buffer_size)
}

// =============================================================================
// Output Info
// =============================================================================

/// Information about an output destination.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct OutputInfo {
    /// Output ID
    pub id: c_int,
    /// Configuration
    pub config: OutputConfig,
    /// Current state
    pub state: c_int,
    /// Bytes written
    pub bytes_written: u64,
    /// Messages written
    pub messages_written: u64,
    /// Write errors
    pub write_errors: u64,
    /// Last error code
    pub last_error: c_int,
}

impl OutputInfo {
    /// Create new output info.
    #[must_use]
    pub const fn new(id: c_int, config: OutputConfig) -> Self {
        Self {
            id,
            config,
            state: OutputState::Closed as c_int,
            bytes_written: 0,
            messages_written: 0,
            write_errors: 0,
            last_error: 0,
        }
    }

    /// Get state.
    #[must_use]
    pub const fn get_state(&self) -> OutputState {
        OutputState::from_c_int(self.state)
    }

    /// Set state.
    pub fn set_state(&mut self, state: OutputState) {
        self.state = state as c_int;
    }

    /// Record successful write.
    pub fn record_write(&mut self, bytes: u64) {
        self.bytes_written += bytes;
        self.messages_written += 1;
    }

    /// Record write error.
    pub fn record_error(&mut self, error_code: c_int) {
        self.write_errors += 1;
        self.last_error = error_code;
    }

    /// Check if can write.
    #[must_use]
    pub const fn can_write(&self) -> bool {
        self.get_state().can_write()
    }
}

/// FFI: Create output info.
#[no_mangle]
pub extern "C" fn rs_output_info_new(id: c_int, config: OutputConfig) -> OutputInfo {
    OutputInfo::new(id, config)
}

/// FFI: Record write.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_output_record_write(info: *mut OutputInfo, bytes: u64) {
    if !info.is_null() {
        (*info).record_write(bytes);
    }
}

/// FFI: Record error.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_output_record_error(info: *mut OutputInfo, error_code: c_int) {
    if !info.is_null() {
        (*info).record_error(error_code);
    }
}

/// FFI: Check if can write.
///
/// # Safety
/// `info` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_output_can_write(info: *const OutputInfo) -> c_int {
    if info.is_null() {
        return 0;
    }
    c_int::from((*info).can_write())
}

// =============================================================================
// Formatter Type
// =============================================================================

/// Type of log formatter.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormatterType {
    /// Plain text format
    #[default]
    Plain = 0,
    /// JSON format
    Json = 1,
    /// Syslog format
    Syslog = 2,
    /// Custom format
    Custom = 3,
}

impl FormatterType {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Json,
            2 => Self::Syslog,
            3 => Self::Custom,
            _ => Self::Plain,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }
}

// =============================================================================
// Formatter Configuration
// =============================================================================

/// Configuration for log formatter.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FormatterConfig {
    /// Formatter type
    pub formatter_type: c_int,
    /// Include timestamp
    pub timestamp: bool,
    /// Include level
    pub level: bool,
    /// Include source location
    pub location: bool,
    /// Include component
    pub component: bool,
    /// Use colors (for plain text)
    pub colors: bool,
    /// Timestamp format (0=ISO8601, 1=Unix, 2=Relative)
    pub timestamp_format: c_int,
}

impl FormatterConfig {
    /// Create plain text formatter.
    #[must_use]
    pub const fn plain() -> Self {
        Self {
            formatter_type: FormatterType::Plain as c_int,
            timestamp: true,
            level: true,
            location: false,
            component: false,
            colors: true,
            timestamp_format: 0,
        }
    }

    /// Create JSON formatter.
    #[must_use]
    pub const fn json() -> Self {
        Self {
            formatter_type: FormatterType::Json as c_int,
            timestamp: true,
            level: true,
            location: true,
            component: true,
            colors: false,
            timestamp_format: 1, // Unix timestamp
        }
    }

    /// Create verbose formatter.
    #[must_use]
    pub const fn verbose() -> Self {
        Self {
            formatter_type: FormatterType::Plain as c_int,
            timestamp: true,
            level: true,
            location: true,
            component: true,
            colors: true,
            timestamp_format: 0,
        }
    }

    /// Get formatter type.
    #[must_use]
    pub const fn get_type(&self) -> FormatterType {
        FormatterType::from_c_int(self.formatter_type)
    }
}

/// FFI: Create plain formatter config.
#[no_mangle]
pub extern "C" fn rs_formatter_config_plain() -> FormatterConfig {
    FormatterConfig::plain()
}

/// FFI: Create JSON formatter config.
#[no_mangle]
pub extern "C" fn rs_formatter_config_json() -> FormatterConfig {
    FormatterConfig::json()
}

/// FFI: Create verbose formatter config.
#[no_mangle]
pub extern "C" fn rs_formatter_config_verbose() -> FormatterConfig {
    FormatterConfig::verbose()
}

// =============================================================================
// Output Manager Stats
// =============================================================================

/// Statistics for output manager.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct OutputManagerStats {
    /// Total outputs registered
    pub total_outputs: c_int,
    /// Active outputs
    pub active_outputs: c_int,
    /// Total bytes written
    pub bytes_written: u64,
    /// Total messages written
    pub messages_written: u64,
    /// Total errors
    pub errors: u64,
    /// Outputs in error state
    pub outputs_in_error: c_int,
}

impl OutputManagerStats {
    /// Create new stats.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_outputs: 0,
            active_outputs: 0,
            bytes_written: 0,
            messages_written: 0,
            errors: 0,
            outputs_in_error: 0,
        }
    }

    /// Add output.
    pub fn add_output(&mut self) {
        self.total_outputs += 1;
        self.active_outputs += 1;
    }

    /// Remove output.
    pub fn remove_output(&mut self) {
        if self.active_outputs > 0 {
            self.active_outputs -= 1;
        }
    }

    /// Record write.
    pub fn record_write(&mut self, bytes: u64) {
        self.bytes_written += bytes;
        self.messages_written += 1;
    }

    /// Record error.
    pub fn record_error(&mut self) {
        self.errors += 1;
    }
}

/// FFI: Create output manager stats.
#[no_mangle]
pub extern "C" fn rs_output_manager_stats_new() -> OutputManagerStats {
    OutputManagerStats::new()
}

/// FFI: Add output.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_output_manager_add_output(stats: *mut OutputManagerStats) {
    if !stats.is_null() {
        (*stats).add_output();
    }
}

/// FFI: Record write.
///
/// # Safety
/// `stats` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_output_manager_record_write(
    stats: *mut OutputManagerStats,
    bytes: u64,
) {
    if !stats.is_null() {
        (*stats).record_write(bytes);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_type() {
        assert!(OutputType::Stdout.is_stream());
        assert!(OutputType::Stderr.is_stream());
        assert!(!OutputType::File.is_stream());

        assert!(OutputType::File.requires_cleanup());
        assert!(OutputType::Network.requires_cleanup());
        assert!(!OutputType::Stdout.requires_cleanup());
    }

    #[test]
    fn test_output_state() {
        assert!(!OutputState::Closed.can_write());
        assert!(OutputState::Open.can_write());
        assert!(!OutputState::Error.can_write());
    }

    #[test]
    fn test_output_config() {
        let stdout = OutputConfig::stdout();
        assert_eq!(stdout.get_type(), OutputType::Stdout);
        assert!(stdout.auto_flush);

        let file = OutputConfig::file(4096);
        assert_eq!(file.get_type(), OutputType::File);
        assert_eq!(file.buffer_size, 4096);
        assert!(!file.auto_flush);
    }

    #[test]
    fn test_output_info() {
        let config = OutputConfig::stdout();
        let mut info = OutputInfo::new(1, config);
        assert!(!info.can_write());

        info.set_state(OutputState::Open);
        assert!(info.can_write());

        info.record_write(100);
        assert_eq!(info.bytes_written, 100);
        assert_eq!(info.messages_written, 1);

        info.record_error(42);
        assert_eq!(info.write_errors, 1);
        assert_eq!(info.last_error, 42);
    }

    #[test]
    fn test_formatter_config() {
        let plain = FormatterConfig::plain();
        assert_eq!(plain.get_type(), FormatterType::Plain);
        assert!(plain.colors);

        let json = FormatterConfig::json();
        assert_eq!(json.get_type(), FormatterType::Json);
        assert!(!json.colors);
    }

    #[test]
    fn test_output_manager_stats() {
        let mut stats = OutputManagerStats::new();
        assert_eq!(stats.active_outputs, 0);

        stats.add_output();
        stats.add_output();
        assert_eq!(stats.total_outputs, 2);
        assert_eq!(stats.active_outputs, 2);

        stats.record_write(500);
        assert_eq!(stats.bytes_written, 500);
        assert_eq!(stats.messages_written, 1);

        stats.remove_output();
        assert_eq!(stats.active_outputs, 1);
    }
}
