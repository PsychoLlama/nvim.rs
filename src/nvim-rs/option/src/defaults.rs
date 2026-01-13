//! Option default value handling.
//!
//! This module provides helpers for option defaults:
//! - Default value retrieval
//! - Default value comparison
//! - Reset to defaults

#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Default Source Constants
// =============================================================================

/// Default is hard-coded in source.
pub const DEFAULT_BUILTIN: c_int = 0;
/// Default is from modeline.
pub const DEFAULT_MODELINE: c_int = 1;
/// Default is from vimrc.
pub const DEFAULT_VIMRC: c_int = 2;
/// Default is from environment variable.
pub const DEFAULT_ENV: c_int = 3;
/// Default is from system config.
pub const DEFAULT_SYSTEM: c_int = 4;

// =============================================================================
// Default Flags
// =============================================================================

/// Option has been changed from default.
pub const OPT_CHANGED: c_int = 0x01;
/// Option was set by user.
pub const OPT_USER_SET: c_int = 0x02;
/// Option was set by modeline.
pub const OPT_MODELINE_SET: c_int = 0x04;
/// Option was set by script.
pub const OPT_SCRIPT_SET: c_int = 0x08;
/// Option is at factory default.
pub const OPT_FACTORY: c_int = 0x10;

// =============================================================================
// Default Value Helpers
// =============================================================================

/// Check if option has been changed from default.
fn is_changed_from_default(flags: c_int) -> bool {
    (flags & OPT_CHANGED) != 0
}

/// Check if option was user-set.
fn is_user_set(flags: c_int) -> bool {
    (flags & OPT_USER_SET) != 0
}

/// Check if option was set by modeline.
fn is_modeline_set(flags: c_int) -> bool {
    (flags & OPT_MODELINE_SET) != 0
}

/// Check if option was set by script.
fn is_script_set(flags: c_int) -> bool {
    (flags & OPT_SCRIPT_SET) != 0
}

/// Check if option is at factory default.
fn is_factory_default(flags: c_int) -> bool {
    (flags & OPT_FACTORY) != 0 && !is_changed_from_default(flags)
}

/// Get priority of default source.
fn default_source_priority(source: c_int) -> c_int {
    match source {
        DEFAULT_BUILTIN => 0,
        DEFAULT_SYSTEM => 1,
        DEFAULT_ENV => 2,
        DEFAULT_VIMRC => 3,
        DEFAULT_MODELINE => 4,
        _ => -1,
    }
}

/// Check if first source has higher priority than second.
fn source_has_higher_priority(source1: c_int, source2: c_int) -> bool {
    default_source_priority(source1) > default_source_priority(source2)
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Get DEFAULT_BUILTIN constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_builtin() -> c_int {
    DEFAULT_BUILTIN
}

/// FFI: Get DEFAULT_MODELINE constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_modeline() -> c_int {
    DEFAULT_MODELINE
}

/// FFI: Get DEFAULT_VIMRC constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_vimrc() -> c_int {
    DEFAULT_VIMRC
}

/// FFI: Get DEFAULT_ENV constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_env() -> c_int {
    DEFAULT_ENV
}

/// FFI: Get DEFAULT_SYSTEM constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_system() -> c_int {
    DEFAULT_SYSTEM
}

/// FFI: Get OPT_CHANGED constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_changed_flag() -> c_int {
    OPT_CHANGED
}

/// FFI: Get OPT_USER_SET constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_user_set_flag() -> c_int {
    OPT_USER_SET
}

/// FFI: Get OPT_MODELINE_SET constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_modeline_set_flag() -> c_int {
    OPT_MODELINE_SET
}

/// FFI: Get OPT_SCRIPT_SET constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_script_set_flag() -> c_int {
    OPT_SCRIPT_SET
}

/// FFI: Get OPT_FACTORY constant.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_factory_flag() -> c_int {
    OPT_FACTORY
}

/// FFI: Check if changed from default.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_changed(flags: c_int) -> c_int {
    c_int::from(is_changed_from_default(flags))
}

/// FFI: Check if user-set.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_user_set(flags: c_int) -> c_int {
    c_int::from(is_user_set(flags))
}

/// FFI: Check if modeline-set.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_modeline_set(flags: c_int) -> c_int {
    c_int::from(is_modeline_set(flags))
}

/// FFI: Check if script-set.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_script_set(flags: c_int) -> c_int {
    c_int::from(is_script_set(flags))
}

/// FFI: Check if factory default.
#[unsafe(no_mangle)]
pub extern "C" fn rs_opt_is_factory_default(flags: c_int) -> c_int {
    c_int::from(is_factory_default(flags))
}

/// FFI: Get default source priority.
#[unsafe(no_mangle)]
pub extern "C" fn rs_default_source_priority(source: c_int) -> c_int {
    default_source_priority(source)
}

/// FFI: Check if source has higher priority.
#[unsafe(no_mangle)]
pub extern "C" fn rs_source_has_higher_priority(source1: c_int, source2: c_int) -> c_int {
    c_int::from(source_has_higher_priority(source1, source2))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_source_constants() {
        assert_eq!(DEFAULT_BUILTIN, 0);
        assert_eq!(DEFAULT_MODELINE, 1);
        assert_eq!(DEFAULT_VIMRC, 2);
        assert_eq!(DEFAULT_ENV, 3);
        assert_eq!(DEFAULT_SYSTEM, 4);
    }

    #[test]
    fn test_default_flags() {
        assert_eq!(OPT_CHANGED, 0x01);
        assert_eq!(OPT_USER_SET, 0x02);
        assert_eq!(OPT_MODELINE_SET, 0x04);
        assert_eq!(OPT_SCRIPT_SET, 0x08);
        assert_eq!(OPT_FACTORY, 0x10);
    }

    #[test]
    fn test_is_changed_from_default() {
        assert!(is_changed_from_default(OPT_CHANGED));
        assert!(!is_changed_from_default(0));
        assert!(!is_changed_from_default(OPT_USER_SET));
    }

    #[test]
    fn test_is_user_set() {
        assert!(is_user_set(OPT_USER_SET));
        assert!(!is_user_set(OPT_MODELINE_SET));
    }

    #[test]
    fn test_is_modeline_set() {
        assert!(is_modeline_set(OPT_MODELINE_SET));
        assert!(!is_modeline_set(OPT_USER_SET));
    }

    #[test]
    fn test_is_script_set() {
        assert!(is_script_set(OPT_SCRIPT_SET));
        assert!(!is_script_set(OPT_USER_SET));
    }

    #[test]
    fn test_is_factory_default() {
        assert!(is_factory_default(OPT_FACTORY));
        assert!(!is_factory_default(OPT_FACTORY | OPT_CHANGED));
        assert!(!is_factory_default(0));
    }

    #[test]
    fn test_default_source_priority() {
        assert_eq!(default_source_priority(DEFAULT_BUILTIN), 0);
        assert_eq!(default_source_priority(DEFAULT_SYSTEM), 1);
        assert_eq!(default_source_priority(DEFAULT_ENV), 2);
        assert_eq!(default_source_priority(DEFAULT_VIMRC), 3);
        assert_eq!(default_source_priority(DEFAULT_MODELINE), 4);
        assert_eq!(default_source_priority(99), -1);
    }

    #[test]
    fn test_source_has_higher_priority() {
        // modeline > vimrc > env > system > builtin
        assert!(source_has_higher_priority(DEFAULT_MODELINE, DEFAULT_VIMRC));
        assert!(source_has_higher_priority(DEFAULT_VIMRC, DEFAULT_ENV));
        assert!(source_has_higher_priority(DEFAULT_ENV, DEFAULT_SYSTEM));
        assert!(source_has_higher_priority(DEFAULT_SYSTEM, DEFAULT_BUILTIN));

        assert!(!source_has_higher_priority(
            DEFAULT_BUILTIN,
            DEFAULT_MODELINE
        ));
        assert!(!source_has_higher_priority(DEFAULT_SYSTEM, DEFAULT_ENV));
    }
}
