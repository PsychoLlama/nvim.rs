//! Ex command handlers for runtime operations
//!
//! This module handles :source, :runtime, :packadd, and related commands.

use std::ffi::{c_char, c_int};

use crate::dip;

// =============================================================================
// Command Types
// =============================================================================

/// Runtime command types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeCmd {
    /// :runtime command
    Runtime = 0,
    /// :source command
    Source = 1,
    /// :packadd command
    Packadd = 2,
    /// :packloadall command
    Packloadall = 3,
    /// :scriptnames command
    Scriptnames = 4,
}

impl RuntimeCmd {
    /// Convert from integer
    pub const fn from_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Runtime),
            1 => Some(Self::Source),
            2 => Some(Self::Packadd),
            3 => Some(Self::Packloadall),
            4 => Some(Self::Scriptnames),
            _ => None,
        }
    }
}

// =============================================================================
// :runtime Command
// =============================================================================

/// Parse :runtime command arguments.
///
/// Syntax: :runtime[!] [where] {file}
///
/// Returns DIP flags based on arguments.
#[allow(clippy::fn_params_excessive_bools)]
pub fn rs_runtime_flags(bang: bool, start: bool, opt: bool, after: bool) -> c_int {
    let mut flags = 0;

    // Bang means find all matches
    if bang {
        flags |= dip::ALL;
    }

    // where argument
    if start {
        flags |= dip::START;
    }
    if opt {
        flags |= dip::OPT;
    }
    if after {
        flags |= dip::AFTER;
    }

    flags
}

/// Check if :runtime should search all matches (bang used).
pub fn rs_runtime_find_all(flags: c_int) -> bool {
    (flags & dip::ALL) != 0
}

// =============================================================================
// :source Command
// =============================================================================

/// Parse :source command modifiers.
///
/// Returns true if this is a :source! (re-source) command.
pub fn rs_source_is_reload(bang: bool) -> bool {
    bang
}

// =============================================================================
// :packadd Command
// =============================================================================

/// Parse :packadd command.
///
/// Returns DIP flags for searching.
pub fn rs_packadd_flags(bang: bool) -> c_int {
    let mut flags = dip::OPT | dip::ALL | dip::DIRFILE;

    // Without bang, also search start directories
    if !bang {
        flags |= dip::START;
    }

    flags
}

/// Check if :packadd should only search opt directories (bang used).
pub fn rs_packadd_opt_only(bang: bool) -> bool {
    bang
}

// =============================================================================
// :packloadall Command
// =============================================================================

/// Check if packloadall should force reload (bang used).
pub fn rs_packloadall_force(bang: bool) -> bool {
    bang
}

// =============================================================================
// Argument Keyword Matching
// =============================================================================

/// :runtime "where" argument values
pub const WHERE_START: &[u8] = b"START\0";
pub const WHERE_OPT: &[u8] = b"OPT\0";
pub const WHERE_PACK: &[u8] = b"PACK\0";
pub const WHERE_ALL: &[u8] = b"ALL\0";

/// Get :runtime START keyword.
pub fn rs_where_start() -> *const c_char {
    WHERE_START.as_ptr().cast()
}

/// Get :runtime OPT keyword.
pub fn rs_where_opt() -> *const c_char {
    WHERE_OPT.as_ptr().cast()
}

/// Get :runtime PACK keyword.
pub fn rs_where_pack() -> *const c_char {
    WHERE_PACK.as_ptr().cast()
}

/// Get :runtime ALL keyword.
pub fn rs_where_all() -> *const c_char {
    WHERE_ALL.as_ptr().cast()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_cmd() {
        assert_eq!(RuntimeCmd::from_int(0), Some(RuntimeCmd::Runtime));
        assert_eq!(RuntimeCmd::from_int(1), Some(RuntimeCmd::Source));
        assert_eq!(RuntimeCmd::from_int(5), None);
    }

    #[test]
    fn test_runtime_flags() {
        let flags = rs_runtime_flags(true, false, false, false);
        assert!(rs_runtime_find_all(flags));

        let flags = rs_runtime_flags(false, true, false, false);
        assert!(!rs_runtime_find_all(flags));
        assert!((flags & dip::START) != 0);

        let flags = rs_runtime_flags(true, true, true, false);
        assert!(rs_runtime_find_all(flags));
        assert!((flags & dip::START) != 0);
        assert!((flags & dip::OPT) != 0);
    }

    #[test]
    fn test_packadd_flags() {
        let flags = rs_packadd_flags(false);
        assert!((flags & dip::START) != 0);
        assert!((flags & dip::OPT) != 0);

        let flags = rs_packadd_flags(true);
        assert!((flags & dip::START) == 0);
        assert!((flags & dip::OPT) != 0);
    }

    #[test]
    fn test_source_reload() {
        assert!(!rs_source_is_reload(false));
        assert!(rs_source_is_reload(true));
    }

    #[test]
    fn test_packloadall_force() {
        assert!(!rs_packloadall_force(false));
        assert!(rs_packloadall_force(true));
    }

    #[test]
    fn test_where_keywords() {
        assert!(!rs_where_start().is_null());
        assert!(!rs_where_opt().is_null());
        assert!(!rs_where_pack().is_null());
        assert!(!rs_where_all().is_null());
    }
}
