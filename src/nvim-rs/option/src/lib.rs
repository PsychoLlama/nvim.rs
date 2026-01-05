//! Option system for Neovim
//!
//! This crate provides Rust implementations of Neovim's option handling
//! functionality from `src/nvim/option.c`. It handles option types, scopes,
//! validation, and option value manipulation.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)]

use std::ffi::c_int;

// =============================================================================
// Constants
// =============================================================================

/// Function succeeded
pub const OK: c_int = 1;

/// Function failed
pub const FAIL: c_int = 0;

// =============================================================================
// Option Value Types
// =============================================================================

/// Option value type.
///
/// Corresponds to `OptValType` in option_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptValType {
    /// Nil/unset value
    Nil = -1,
    /// Boolean option (true/false/none)
    Boolean = 0,
    /// Numeric option (integer)
    Number = 1,
    /// String option
    String = 2,
}

// =============================================================================
// Option Scopes
// =============================================================================

/// Scopes that an option can support.
///
/// Corresponds to `OptScope` in option_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptScope {
    /// Global option value
    Global = 0,
    /// Window-local option value
    Win = 1,
    /// Buffer-local option value
    Buf = 2,
}

impl OptScope {
    /// Number of option scopes
    pub const COUNT: usize = 3;
}

// =============================================================================
// Option Flags
// =============================================================================

/// Option flags.
///
/// Corresponds to `OptFlags` in option_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptFlags(pub u32);

impl OptFlags {
    /// Environment expansion
    pub const EXPAND: Self = Self(1 << 0);
    /// Don't expand default value
    pub const NO_DEF_EXP: Self = Self(1 << 1);
    /// Don't set to default value
    pub const NO_DEFAULT: Self = Self(1 << 2);
    /// Option has been set/reset
    pub const WAS_SET: Self = Self(1 << 3);
    /// Don't include in :mkvimrc output
    pub const NO_MKRC: Self = Self(1 << 4);
    /// Send option to remote UI
    pub const UI_OPTION: Self = Self(1 << 5);
    /// Redraw tabline
    pub const REDR_TABL: Self = Self(1 << 6);
    /// Redraw status lines
    pub const REDR_STAT: Self = Self(1 << 7);
    /// Redraw current window and recompute text
    pub const REDR_WIN: Self = Self(1 << 8);
    /// Redraw current buffer and recompute text
    pub const REDR_BUF: Self = Self(1 << 9);
    /// Comma-separated list
    pub const COMMA: Self = Self(1 << 10);
    /// Don't allow duplicate strings
    pub const NO_DUP: Self = Self(1 << 12);
    /// List of single-char flags
    pub const FLAG_LIST: Self = Self(1 << 13);
    /// Cannot change in modeline or secure mode
    pub const SECURE: Self = Self(1 << 14);
    /// Expand default value with _()
    pub const GETTEXT: Self = Self(1 << 15);
    /// Do not use local value for global vimrc
    pub const NO_GLOB: Self = Self(1 << 16);
    /// Only normal file name chars allowed
    pub const N_FNAME: Self = Self(1 << 17);
    /// Option was set from a modeline
    pub const INSECURE: Self = Self(1 << 18);
    /// Priority for :mkvimrc
    pub const PRI_MKRC: Self = Self(1 << 19);
    /// Not allowed in modeline
    pub const NO_ML: Self = Self(1 << 20);
    /// Update curswant required
    pub const CURSWANT: Self = Self(1 << 21);
    /// Only normal directory name chars allowed
    pub const N_DNAME: Self = Self(1 << 22);
    /// Option only changes highlight, not text
    pub const HL_ONLY: Self = Self(1 << 23);
    /// Under control of 'modelineexpr'
    pub const MLE: Self = Self(1 << 24);
    /// Accept a function reference or a lambda
    pub const FUNC: Self = Self(1 << 25);
    /// Values use colons to create sublists
    pub const COLON: Self = Self(1 << 26);

    /// Redraw all windows and recompute text
    pub const REDR_ALL: Self = Self(Self::REDR_BUF.0 | Self::REDR_WIN.0);
    /// Clear and redraw all and recompute text
    pub const REDR_CLEAR: Self = Self(Self::REDR_ALL.0 | Self::REDR_STAT.0);
    /// Comma-separated list that cannot have two consecutive commas
    pub const ONE_COMMA: Self = Self((1 << 11) | Self::COMMA.0);

    /// Check if a flag is set
    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Combine two flag sets
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

// =============================================================================
// :set Operator Types
// =============================================================================

/// :set operator types.
///
/// Corresponds to `set_op_T` in option_defs.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOp {
    /// No operator
    None = 0,
    /// "opt+=arg"
    Adding = 1,
    /// "opt^=arg"
    Prepending = 2,
    /// "opt-=arg"
    Removing = 3,
}

// =============================================================================
// :set Boolean Option Prefix
// =============================================================================

/// :set boolean option prefix.
///
/// Corresponds to `set_prefix_T` in option.c.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetPrefix {
    /// "no" prefix
    No = 0,
    /// No prefix
    None = 1,
    /// "inv" prefix
    Inv = 2,
}

// =============================================================================
// Option Setting Flags
// =============================================================================

/// Flags for option-setting functions.
///
/// Corresponds to `OptionSetFlags` in option.h.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OptionSetFlags(pub u32);

impl OptionSetFlags {
    /// Use global value
    pub const GLOBAL: Self = Self(0x01);
    /// Use local value
    pub const LOCAL: Self = Self(0x02);
    /// Option in modeline
    pub const MODELINE: Self = Self(0x04);
    /// Only set window-local options
    pub const WINONLY: Self = Self(0x08);
    /// Don't set window-local options
    pub const NOWIN: Self = Self(0x10);
    /// List options one per line
    pub const ONECOLUMN: Self = Self(0x20);
    /// Ignore redraw flags on option
    pub const NO_REDRAW: Self = Self(0x40);
    /// "skiprtp" in 'sessionoptions'
    pub const SKIPRTP: Self = Self(0x80);

    /// Check if a flag is set
    #[inline]
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    /// Combine two flag sets
    #[inline]
    #[must_use]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle to a buffer (buf_T*).
pub type BufHandle = *mut std::ffi::c_void;

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_val_type_repr() {
        assert_eq!(OptValType::Nil as i32, -1);
        assert_eq!(OptValType::Boolean as i32, 0);
        assert_eq!(OptValType::Number as i32, 1);
        assert_eq!(OptValType::String as i32, 2);
    }

    #[test]
    fn test_opt_scope_repr() {
        assert_eq!(OptScope::Global as i32, 0);
        assert_eq!(OptScope::Win as i32, 1);
        assert_eq!(OptScope::Buf as i32, 2);
    }

    #[test]
    fn test_opt_flags() {
        let flags = OptFlags::EXPAND.union(OptFlags::COMMA);
        assert!(flags.contains(OptFlags::EXPAND));
        assert!(flags.contains(OptFlags::COMMA));
        assert!(!flags.contains(OptFlags::SECURE));
    }

    #[test]
    fn test_set_op_repr() {
        assert_eq!(SetOp::None as i32, 0);
        assert_eq!(SetOp::Adding as i32, 1);
        assert_eq!(SetOp::Prepending as i32, 2);
        assert_eq!(SetOp::Removing as i32, 3);
    }

    #[test]
    fn test_set_prefix_repr() {
        assert_eq!(SetPrefix::No as i32, 0);
        assert_eq!(SetPrefix::None as i32, 1);
        assert_eq!(SetPrefix::Inv as i32, 2);
    }

    #[test]
    fn test_option_set_flags() {
        let flags = OptionSetFlags::GLOBAL.union(OptionSetFlags::LOCAL);
        assert!(flags.contains(OptionSetFlags::GLOBAL));
        assert!(flags.contains(OptionSetFlags::LOCAL));
        assert!(!flags.contains(OptionSetFlags::MODELINE));
    }
}
