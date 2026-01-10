//! Log levels and filtering
//!
//! This module provides log level definitions and filtering infrastructure
//! for controlling log output verbosity.

use std::ffi::c_int;

// =============================================================================
// Log Level
// =============================================================================

/// Log level.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum LogLevel {
    /// Debug messages (most verbose)
    Debug = 0,
    /// Informational messages
    #[default]
    Info = 1,
    /// Warning messages
    Warn = 2,
    /// Error messages (least verbose)
    Error = 3,
    /// No logging (completely silent)
    Off = 4,
}

impl LogLevel {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Debug,
            2 => Self::Warn,
            3 => Self::Error,
            4 => Self::Off,
            // 1 and unrecognized values default to Info
            _ => Self::Info,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Get level name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Off => "OFF",
        }
    }

    /// Get short name (single character).
    #[must_use]
    pub const fn short_name(self) -> u8 {
        match self {
            Self::Debug => b'D',
            Self::Info => b'I',
            Self::Warn => b'W',
            Self::Error => b'E',
            Self::Off => b'-',
        }
    }

    /// Check if this level is enabled for given threshold.
    #[must_use]
    pub const fn is_enabled_for(self, threshold: Self) -> bool {
        (self as c_int) >= (threshold as c_int)
    }
}

/// FFI: Get level name length.
#[no_mangle]
pub extern "C" fn rs_log_level_name_len(level: c_int) -> c_int {
    LogLevel::from_c_int(level).name().len() as c_int
}

/// FFI: Get level short name.
#[no_mangle]
pub extern "C" fn rs_log_level_short_name(level: c_int) -> u8 {
    LogLevel::from_c_int(level).short_name()
}

/// FFI: Check if level enabled.
#[no_mangle]
pub extern "C" fn rs_log_level_is_enabled(level: c_int, threshold: c_int) -> c_int {
    c_int::from(
        LogLevel::from_c_int(level).is_enabled_for(LogLevel::from_c_int(threshold)),
    )
}

// =============================================================================
// Log Filter
// =============================================================================

/// Filter for log messages.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LogFilter {
    /// Minimum level to pass
    pub min_level: c_int,
    /// Component filter (0 = all)
    pub component_id: c_int,
    /// Include timestamps
    pub include_timestamp: bool,
    /// Include source location
    pub include_location: bool,
    /// Include component name
    pub include_component: bool,
    /// Max messages per second (0 = unlimited)
    pub rate_limit: c_int,
}

impl LogFilter {
    /// Create filter that passes all messages.
    #[must_use]
    pub const fn pass_all() -> Self {
        Self {
            min_level: LogLevel::Debug as c_int,
            component_id: 0,
            include_timestamp: true,
            include_location: true,
            include_component: true,
            rate_limit: 0,
        }
    }

    /// Create filter with minimum level.
    #[must_use]
    pub const fn with_level(level: LogLevel) -> Self {
        Self {
            min_level: level as c_int,
            component_id: 0,
            include_timestamp: true,
            include_location: true,
            include_component: true,
            rate_limit: 0,
        }
    }

    /// Create production filter (info and above).
    #[must_use]
    pub const fn production() -> Self {
        Self {
            min_level: LogLevel::Info as c_int,
            component_id: 0,
            include_timestamp: true,
            include_location: false,
            include_component: false,
            rate_limit: 100,
        }
    }

    /// Create debug filter.
    #[must_use]
    pub const fn debug() -> Self {
        Self {
            min_level: LogLevel::Debug as c_int,
            component_id: 0,
            include_timestamp: true,
            include_location: true,
            include_component: true,
            rate_limit: 0,
        }
    }

    /// Get minimum level.
    #[must_use]
    pub const fn get_min_level(&self) -> LogLevel {
        LogLevel::from_c_int(self.min_level)
    }

    /// Check if message should pass filter.
    #[must_use]
    pub const fn should_pass(&self, level: LogLevel, component_id: c_int) -> bool {
        // Check level
        if (level as c_int) < self.min_level {
            return false;
        }
        // Check component
        if self.component_id != 0 && component_id != self.component_id {
            return false;
        }
        true
    }
}

/// FFI: Create pass-all filter.
#[no_mangle]
pub extern "C" fn rs_log_filter_pass_all() -> LogFilter {
    LogFilter::pass_all()
}

/// FFI: Create filter with level.
#[no_mangle]
pub extern "C" fn rs_log_filter_with_level(level: c_int) -> LogFilter {
    LogFilter::with_level(LogLevel::from_c_int(level))
}

/// FFI: Create production filter.
#[no_mangle]
pub extern "C" fn rs_log_filter_production() -> LogFilter {
    LogFilter::production()
}

/// FFI: Create debug filter.
#[no_mangle]
pub extern "C" fn rs_log_filter_debug() -> LogFilter {
    LogFilter::debug()
}

/// FFI: Check if message should pass.
///
/// # Safety
/// `filter` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_log_filter_should_pass(
    filter: *const LogFilter,
    level: c_int,
    component_id: c_int,
) -> c_int {
    if filter.is_null() {
        return 1; // Pass by default
    }
    c_int::from((*filter).should_pass(LogLevel::from_c_int(level), component_id))
}

// =============================================================================
// Level Configuration
// =============================================================================

/// A level override for a specific component.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LevelOverride {
    /// Component ID
    pub component_id: c_int,
    /// Log level for this component
    pub level: c_int,
}

impl LevelOverride {
    /// Create new override.
    #[must_use]
    pub const fn new(component_id: c_int, level: LogLevel) -> Self {
        Self {
            component_id,
            level: level as c_int,
        }
    }
}

/// Configuration for log levels per component.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LevelConfig {
    /// Default level for all components
    pub default_level: c_int,
    /// Level overrides for specific components (up to 8)
    pub overrides: [LevelOverride; 8],
    /// Number of overrides
    pub override_count: c_int,
}

impl LevelConfig {
    /// Create new config with default level.
    #[must_use]
    pub const fn new(default_level: LogLevel) -> Self {
        Self {
            default_level: default_level as c_int,
            overrides: [LevelOverride { component_id: 0, level: 0 }; 8],
            override_count: 0,
        }
    }

    /// Get level for component.
    #[must_use]
    pub const fn get_level(&self, component_id: c_int) -> LogLevel {
        let mut i = 0;
        while i < self.override_count as usize && i < 8 {
            if self.overrides[i].component_id == component_id {
                return LogLevel::from_c_int(self.overrides[i].level);
            }
            i += 1;
        }
        LogLevel::from_c_int(self.default_level)
    }

    /// Add override (returns false if full).
    pub fn add_override(&mut self, component_id: c_int, level: LogLevel) -> bool {
        if self.override_count >= 8 {
            return false;
        }
        self.overrides[self.override_count as usize] = LevelOverride::new(component_id, level);
        self.override_count += 1;
        true
    }

    /// Clear all overrides.
    pub fn clear_overrides(&mut self) {
        self.override_count = 0;
    }
}

/// FFI: Create level config.
#[no_mangle]
pub extern "C" fn rs_level_config_new(default_level: c_int) -> LevelConfig {
    LevelConfig::new(LogLevel::from_c_int(default_level))
}

/// FFI: Get level for component.
///
/// # Safety
/// `config` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_level_config_get(config: *const LevelConfig, component_id: c_int) -> c_int {
    if config.is_null() {
        return LogLevel::Info as c_int;
    }
    (*config).get_level(component_id) as c_int
}

/// FFI: Add override.
///
/// # Safety
/// `config` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_level_config_add_override(
    config: *mut LevelConfig,
    component_id: c_int,
    level: c_int,
) -> c_int {
    if config.is_null() {
        return 0;
    }
    c_int::from((*config).add_override(component_id, LogLevel::from_c_int(level)))
}

/// FFI: Clear overrides.
///
/// # Safety
/// `config` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_level_config_clear(config: *mut LevelConfig) {
    if !config.is_null() {
        (*config).clear_overrides();
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level() {
        assert_eq!(LogLevel::Debug.name(), "DEBUG");
        assert_eq!(LogLevel::Info.name(), "INFO");
        assert_eq!(LogLevel::Warn.name(), "WARN");
        assert_eq!(LogLevel::Error.name(), "ERROR");

        assert_eq!(LogLevel::Debug.short_name(), b'D');
        assert_eq!(LogLevel::Error.short_name(), b'E');

        assert!(LogLevel::Error.is_enabled_for(LogLevel::Info));
        assert!(LogLevel::Info.is_enabled_for(LogLevel::Info));
        assert!(!LogLevel::Debug.is_enabled_for(LogLevel::Info));
    }

    #[test]
    fn test_log_filter() {
        let filter = LogFilter::pass_all();
        assert!(filter.should_pass(LogLevel::Debug, 0));
        assert!(filter.should_pass(LogLevel::Error, 0));

        let filter = LogFilter::with_level(LogLevel::Warn);
        assert!(!filter.should_pass(LogLevel::Debug, 0));
        assert!(!filter.should_pass(LogLevel::Info, 0));
        assert!(filter.should_pass(LogLevel::Warn, 0));
        assert!(filter.should_pass(LogLevel::Error, 0));

        let prod = LogFilter::production();
        assert!(!prod.should_pass(LogLevel::Debug, 0));
        assert!(prod.should_pass(LogLevel::Info, 0));
    }

    #[test]
    fn test_level_config() {
        let mut config = LevelConfig::new(LogLevel::Info);
        assert_eq!(config.get_level(1), LogLevel::Info);
        assert_eq!(config.get_level(2), LogLevel::Info);

        assert!(config.add_override(1, LogLevel::Debug));
        assert_eq!(config.get_level(1), LogLevel::Debug);
        assert_eq!(config.get_level(2), LogLevel::Info);

        config.clear_overrides();
        assert_eq!(config.get_level(1), LogLevel::Info);
    }
}
