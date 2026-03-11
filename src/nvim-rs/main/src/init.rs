//! Initialization sequence
//!
//! This module provides Rust implementations for Neovim's
//! initialization sequence (early_init, event_init, etc.).

use std::ffi::c_int;

// =============================================================================
// Init Flags
// =============================================================================

/// Initialization flags.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct InitFlags {
    /// No swap file
    pub no_swap: bool,
    /// No plugins
    pub no_plugins: bool,
    /// No user config
    pub no_config: bool,
    /// No site config
    pub no_site: bool,
    /// Clean mode (implies no_plugins, no_config)
    pub clean: bool,
    /// Headless mode
    pub headless: bool,
    /// Embed mode
    pub embed: bool,
    /// Listen mode
    pub listen: bool,
}

impl InitFlags {
    /// Create new init flags.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            no_swap: false,
            no_plugins: false,
            no_config: false,
            no_site: false,
            clean: false,
            headless: false,
            embed: false,
            listen: false,
        }
    }

    /// Check if any config is disabled.
    #[must_use]
    pub const fn is_config_disabled(&self) -> bool {
        self.clean || self.no_config
    }

    /// Check if plugins are disabled.
    #[must_use]
    pub const fn is_plugins_disabled(&self) -> bool {
        self.clean || self.no_plugins
    }

    /// Check if running in non-interactive mode.
    #[must_use]
    pub const fn is_non_interactive(&self) -> bool {
        self.headless || self.embed
    }
}

// =============================================================================
// Init Step Result
// =============================================================================

/// Result of an initialization step.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitResult {
    /// Step succeeded
    Ok = 0,
    /// Step failed (non-fatal)
    Warning = 1,
    /// Step failed (fatal)
    Error = 2,
    /// Step skipped
    Skipped = 3,
}

impl InitResult {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Warning,
            2 => Self::Error,
            3 => Self::Skipped,
            _ => Self::Ok,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if result is successful.
    #[must_use]
    pub const fn is_ok(self) -> bool {
        matches!(self, Self::Ok | Self::Warning | Self::Skipped)
    }

    /// Check if result is fatal.
    #[must_use]
    pub const fn is_fatal(self) -> bool {
        matches!(self, Self::Error)
    }
}

// =============================================================================
// Init Order
// =============================================================================

/// Initialization order/priority.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InitOrder {
    /// Core runtime (memory, signals)
    #[default]
    Core = 0,
    /// Event loop
    Events = 1,
    /// Message handling
    Messages = 2,
    /// Options
    Options = 3,
    /// Mappings
    Mappings = 4,
    /// Autocmds
    Autocmds = 5,
    /// Plugins
    Plugins = 6,
    /// User scripts
    UserScripts = 7,
}

impl InitOrder {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::Events,
            2 => Self::Messages,
            3 => Self::Options,
            4 => Self::Mappings,
            5 => Self::Autocmds,
            6 => Self::Plugins,
            7 => Self::UserScripts,
            _ => Self::Core,
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
    fn test_init_flags() {
        let flags = InitFlags::new();
        assert!(!flags.is_config_disabled());
        assert!(!flags.is_plugins_disabled());

        let mut flags = flags;
        flags.clean = true;
        assert!(flags.is_config_disabled());
        assert!(flags.is_plugins_disabled());
    }

    #[test]
    fn test_init_result() {
        assert!(InitResult::Ok.is_ok());
        assert!(InitResult::Warning.is_ok());
        assert!(InitResult::Skipped.is_ok());
        assert!(!InitResult::Error.is_ok());
        assert!(InitResult::Error.is_fatal());
    }

    #[test]
    fn test_init_order() {
        assert_eq!(InitOrder::from_c_int(0), InitOrder::Core);
        assert_eq!(InitOrder::from_c_int(6), InitOrder::Plugins);
    }
}
