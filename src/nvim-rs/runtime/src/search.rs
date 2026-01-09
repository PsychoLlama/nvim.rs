//! Runtime path searching
//!
//! This module handles searching for files in 'runtimepath' and 'packpath'.

use std::ffi::{c_char, c_int};

use crate::dip;

// =============================================================================
// Search Mode Utilities
// =============================================================================

/// Determine if we should search 'runtimepath'.
#[no_mangle]
pub extern "C" fn rs_should_search_rtp(flags: c_int) -> bool {
    (flags & dip::NORTP) == 0
}

/// Determine if we should search "after" directories.
#[no_mangle]
pub extern "C" fn rs_should_search_after(flags: c_int) -> bool {
    // Search after unless NOAFTER is set
    // If AFTER is set, we ONLY search after directories
    (flags & dip::NOAFTER) == 0
}

/// Determine if we should ONLY search "after" directories.
#[no_mangle]
pub extern "C" fn rs_search_only_after(flags: c_int) -> bool {
    (flags & dip::AFTER) != 0
}

/// Determine if we should search for directories instead of files.
#[no_mangle]
pub extern "C" fn rs_search_directories(flags: c_int) -> bool {
    (flags & dip::DIR) != 0
}

/// Determine if we should search for both files and directories.
#[no_mangle]
pub extern "C" fn rs_search_dir_and_file(flags: c_int) -> bool {
    (flags & dip::DIRFILE) != 0
}

/// Determine if we should find all matches (not just first).
#[no_mangle]
pub extern "C" fn rs_search_find_all(flags: c_int) -> bool {
    (flags & dip::ALL) != 0
}

/// Determine if we should report errors when nothing found.
#[no_mangle]
pub extern "C" fn rs_search_report_errors(flags: c_int) -> bool {
    (flags & dip::ERR) != 0
}

// =============================================================================
// Package Path Utilities
// =============================================================================

/// Check if searching pack/*/start directories.
#[no_mangle]
pub extern "C" fn rs_search_pack_start(flags: c_int) -> bool {
    (flags & dip::START) != 0
}

/// Check if searching pack/*/opt directories.
#[no_mangle]
pub extern "C" fn rs_search_pack_opt(flags: c_int) -> bool {
    (flags & dip::OPT) != 0
}

/// Build flags for searching pack directories.
///
/// Combines START/OPT with other relevant flags.
#[no_mangle]
pub extern "C" fn rs_pack_search_flags(want_start: bool, want_opt: bool, find_all: bool) -> c_int {
    let mut flags = 0;
    if want_start {
        flags |= dip::START;
    }
    if want_opt {
        flags |= dip::OPT;
    }
    if find_all {
        flags |= dip::ALL;
    }
    flags
}

// =============================================================================
// Path String Patterns
// =============================================================================

/// Subdirectory names for package searching
pub const PACK_START_DIR: &[u8] = b"pack/*/start/*\0";
pub const PACK_OPT_DIR: &[u8] = b"pack/*/opt/*\0";
pub const AFTER_DIR: &[u8] = b"after\0";

/// Get the "pack/*/start/*" pattern.
#[no_mangle]
pub extern "C" fn rs_get_pack_start_pattern() -> *const c_char {
    PACK_START_DIR.as_ptr().cast::<c_char>()
}

/// Get the "pack/*/opt/*" pattern.
#[no_mangle]
pub extern "C" fn rs_get_pack_opt_pattern() -> *const c_char {
    PACK_OPT_DIR.as_ptr().cast::<c_char>()
}

/// Get the "after" directory name.
#[no_mangle]
pub extern "C" fn rs_get_after_dir() -> *const c_char {
    AFTER_DIR.as_ptr().cast::<c_char>()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_flags() {
        assert!(rs_should_search_rtp(0));
        assert!(!rs_should_search_rtp(dip::NORTP));

        assert!(rs_should_search_after(0));
        assert!(!rs_should_search_after(dip::NOAFTER));

        assert!(!rs_search_only_after(0));
        assert!(rs_search_only_after(dip::AFTER));

        assert!(!rs_search_directories(0));
        assert!(rs_search_directories(dip::DIR));
    }

    #[test]
    fn test_pack_search_flags() {
        let flags = rs_pack_search_flags(true, false, true);
        assert!(rs_search_pack_start(flags));
        assert!(!rs_search_pack_opt(flags));
        assert!(rs_search_find_all(flags));

        let flags = rs_pack_search_flags(false, true, false);
        assert!(!rs_search_pack_start(flags));
        assert!(rs_search_pack_opt(flags));
        assert!(!rs_search_find_all(flags));
    }

    #[test]
    fn test_patterns_are_null_terminated() {
        let start = rs_get_pack_start_pattern();
        assert!(!start.is_null());

        let opt = rs_get_pack_opt_pattern();
        assert!(!opt.is_null());

        let after = rs_get_after_dir();
        assert!(!after.is_null());
    }
}
