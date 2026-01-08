//! Command modifier types and utilities for Ex commands.
//!
//! This module defines the types and constants used for command modifiers
//! like `:silent`, `:vertical`, `:noautocmd`, etc.

use std::ffi::c_int;

// =============================================================================
// Command modifier flags (CMOD_*)
// =============================================================================

/// `:sandbox` - execute in sandbox mode
pub const CMOD_SANDBOX: c_int = 0x0001;
/// `:silent` - suppress messages
pub const CMOD_SILENT: c_int = 0x0002;
/// `:silent!` - suppress error messages too
pub const CMOD_ERRSILENT: c_int = 0x0004;
/// `:unsilent` - cancel silence
pub const CMOD_UNSILENT: c_int = 0x0008;
/// `:noautocmd` - disable autocommands
pub const CMOD_NOAUTOCMD: c_int = 0x0010;
/// `:hide` - hide buffer when leaving
pub const CMOD_HIDE: c_int = 0x0020;
/// `:browse` - invoke file dialog
pub const CMOD_BROWSE: c_int = 0x0040;
/// `:confirm` - invoke yes/no dialog
pub const CMOD_CONFIRM: c_int = 0x0080;
/// `:keepalt` - keep alternate file
pub const CMOD_KEEPALT: c_int = 0x0100;
/// `:keepmarks` - keep marks
pub const CMOD_KEEPMARKS: c_int = 0x0200;
/// `:keepjumps` - keep jump list
pub const CMOD_KEEPJUMPS: c_int = 0x0400;
/// `:lockmarks` - lock marks
pub const CMOD_LOCKMARKS: c_int = 0x0800;
/// `:keeppatterns` - keep search patterns
pub const CMOD_KEEPPATTERNS: c_int = 0x1000;
/// `:noswapfile` - don't create swap file
pub const CMOD_NOSWAPFILE: c_int = 0x2000;

// =============================================================================
// Window split flags (WSP_*)
// =============================================================================

/// Split horizontally
pub const WSP_HOR: c_int = 0x01;
/// Split vertically
pub const WSP_VERT: c_int = 0x02;
/// Split at top
pub const WSP_TOP: c_int = 0x04;
/// Split at bottom
pub const WSP_BOT: c_int = 0x08;
/// Split above current window
pub const WSP_ABOVE: c_int = 0x10;
/// Split below current window
pub const WSP_BELOW: c_int = 0x20;

// =============================================================================
// Flag checking utilities
// =============================================================================

/// Check if the CMOD_SANDBOX flag is set.
#[inline]
pub const fn has_sandbox(flags: c_int) -> bool {
    (flags & CMOD_SANDBOX) != 0
}

/// Check if the CMOD_SILENT flag is set.
#[inline]
pub const fn has_silent(flags: c_int) -> bool {
    (flags & CMOD_SILENT) != 0
}

/// Check if the CMOD_ERRSILENT flag is set.
#[inline]
pub const fn has_errsilent(flags: c_int) -> bool {
    (flags & CMOD_ERRSILENT) != 0
}

/// Check if the CMOD_NOAUTOCMD flag is set.
#[inline]
pub const fn has_noautocmd(flags: c_int) -> bool {
    (flags & CMOD_NOAUTOCMD) != 0
}

/// FFI wrapper to check if CMOD_SANDBOX flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_sandbox(flags: c_int) -> c_int {
    c_int::from(has_sandbox(flags))
}

/// FFI wrapper to check if CMOD_SILENT flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_silent(flags: c_int) -> c_int {
    c_int::from(has_silent(flags))
}

/// FFI wrapper to check if CMOD_ERRSILENT flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_errsilent(flags: c_int) -> c_int {
    c_int::from(has_errsilent(flags))
}

/// FFI wrapper to check if CMOD_NOAUTOCMD flag is set.
#[no_mangle]
pub extern "C" fn rs_cmod_has_noautocmd(flags: c_int) -> c_int {
    c_int::from(has_noautocmd(flags))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmod_flag_checks() {
        // Test individual flags
        assert!(has_sandbox(CMOD_SANDBOX));
        assert!(!has_sandbox(CMOD_SILENT));

        assert!(has_silent(CMOD_SILENT));
        assert!(!has_silent(CMOD_SANDBOX));

        assert!(has_errsilent(CMOD_ERRSILENT));
        assert!(!has_errsilent(CMOD_SILENT));

        assert!(has_noautocmd(CMOD_NOAUTOCMD));
        assert!(!has_noautocmd(CMOD_SILENT));

        // Test combined flags
        let combined = CMOD_SANDBOX | CMOD_SILENT | CMOD_NOAUTOCMD;
        assert!(has_sandbox(combined));
        assert!(has_silent(combined));
        assert!(!has_errsilent(combined));
        assert!(has_noautocmd(combined));
    }

    #[test]
    fn test_cmod_ffi_wrappers() {
        assert_eq!(rs_cmod_has_sandbox(CMOD_SANDBOX), 1);
        assert_eq!(rs_cmod_has_sandbox(CMOD_SILENT), 0);

        assert_eq!(rs_cmod_has_silent(CMOD_SILENT), 1);
        assert_eq!(rs_cmod_has_silent(CMOD_SANDBOX), 0);
    }

    #[test]
    fn test_wsp_flags() {
        // Verify flag values match expected constants
        assert_eq!(WSP_HOR, 0x01);
        assert_eq!(WSP_VERT, 0x02);
        assert_eq!(WSP_TOP, 0x04);
        assert_eq!(WSP_BOT, 0x08);
        assert_eq!(WSP_ABOVE, 0x10);
        assert_eq!(WSP_BELOW, 0x20);
    }
}
