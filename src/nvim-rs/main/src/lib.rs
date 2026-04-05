//! Main/Startup infrastructure for Neovim
//!
//! This crate provides Rust implementations for Neovim's startup
//! and initialization code from `src/nvim/main.c`.
//!
//! ## Modules
//!
//! - [`init`]: Initialization sequence
//! - [`args`]: Command-line argument parsing
//! - [`exit`]: Exit and cleanup
//! - [`signals`]: Signal handling infrastructure

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

pub mod args;
pub mod commands;
pub mod config;
pub mod exit;
pub mod helpers;
pub mod init;
pub mod lifecycle;
pub mod output;
pub mod scan;
pub mod setup;
pub mod signals;

use std::ffi::c_int;

// =============================================================================
// Startup Phase
// =============================================================================

/// Startup phase enumeration.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StartupPhase {
    /// Not yet started
    #[default]
    NotStarted = 0,
    /// Early initialization
    EarlyInit = 1,
    /// Command-line parsing
    ArgParse = 2,
    /// User config loading
    UserConfig = 3,
    /// Plugin loading
    Plugins = 4,
    /// File loading
    Files = 5,
    /// Fully started
    Complete = 6,
}

impl StartupPhase {
    /// Create from C int.
    #[must_use]
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            1 => Self::EarlyInit,
            2 => Self::ArgParse,
            3 => Self::UserConfig,
            4 => Self::Plugins,
            5 => Self::Files,
            6 => Self::Complete,
            _ => Self::NotStarted,
        }
    }

    /// Convert to C int.
    #[must_use]
    pub const fn to_c_int(self) -> c_int {
        self as c_int
    }

    /// Check if startup is complete.
    #[must_use]
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }

    /// Check if in early init phase.
    #[must_use]
    pub const fn is_early(self) -> bool {
        matches!(self, Self::NotStarted | Self::EarlyInit)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_startup_phase() {
        assert_eq!(StartupPhase::from_c_int(0), StartupPhase::NotStarted);
        assert_eq!(StartupPhase::from_c_int(6), StartupPhase::Complete);
        assert!(StartupPhase::Complete.is_complete());
        assert!(!StartupPhase::EarlyInit.is_complete());
        assert!(StartupPhase::EarlyInit.is_early());
        assert!(!StartupPhase::Complete.is_early());
    }
}
