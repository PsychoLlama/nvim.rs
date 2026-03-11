//! Option string validation for Neovim
//!
//! This module provides utilities for validating and processing option string values.
//! It handles comma-separated lists, flag lists, fillchars/listchars validation,
//! and various option-specific validation routines.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]

use std::ffi::{c_char, c_int};

mod chars;
mod didset;
mod flags;
mod listval;
mod validate;

pub use chars::*;
pub use flags::*;
pub use listval::*;
pub use validate::*;

// =============================================================================
// Option Flags (matching C's kOptFlag* constants)
// =============================================================================

bitflags::bitflags! {
    /// Option flags matching C's OptFlags enum
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct OptFlags: u32 {
        /// Environment expansion
        const EXPAND = 1 << 0;
        /// Don't expand default value
        const NO_DEF_EXP = 1 << 1;
        /// Don't set to default value
        const NO_DEFAULT = 1 << 2;
        /// Option has been set/reset
        const WAS_SET = 1 << 3;
        /// Don't include in :mkvimrc output
        const NO_MKRC = 1 << 4;
        /// Send option to remote UI
        const UI_OPTION = 1 << 5;
        /// Redraw tabline
        const REDR_TABL = 1 << 6;
        /// Redraw status lines
        const REDR_STAT = 1 << 7;
        /// Redraw current window and recompute text
        const REDR_WIN = 1 << 8;
        /// Redraw current buffer and recompute text
        const REDR_BUF = 1 << 9;
        /// Redraw all windows and recompute text
        const REDR_ALL = Self::REDR_BUF.bits() | Self::REDR_WIN.bits();
        /// Clear and redraw all and recompute text
        const REDR_CLEAR = Self::REDR_ALL.bits() | Self::REDR_STAT.bits();
        /// Comma-separated list
        const COMMA = 1 << 10;
        /// Comma-separated list that cannot have two consecutive commas
        const ONE_COMMA = (1 << 11) | Self::COMMA.bits();
        /// Don't allow duplicate strings
        const NO_DUP = 1 << 12;
        /// List of single-char flags
        const FLAG_LIST = 1 << 13;
        /// Cannot change in modeline or secure mode
        const SECURE = 1 << 14;
        /// Expand default value with _()
        const GETTEXT = 1 << 15;
        /// Do not use local value for global vimrc
        const NO_GLOB = 1 << 16;
        /// Only normal file name chars allowed
        const NFNAME = 1 << 17;
        /// Option was set from a modeline
        const INSECURE = 1 << 18;
        /// Priority for :mkvimrc
        const PRI_MKRC = 1 << 19;
        /// Not allowed in modeline
        const NO_ML = 1 << 20;
        /// Update curswant required
        const CURSWANT = 1 << 21;
        /// Only normal directory name chars allowed
        const NDNAME = 1 << 22;
        /// Option only changes highlight, not text
        const HL_ONLY = 1 << 23;
        /// Under control of 'modelineexpr'
        const MLE = 1 << 24;
        /// Accept a function reference or a lambda
        const FUNC = 1 << 25;
        /// Values use colons to create sublists
        const COLON = 1 << 26;
    }
}

/// Create OptFlags from raw bits
#[no_mangle]
pub extern "C" fn rs_optflags_from_bits(bits: u32) -> OptFlags {
    OptFlags::from_bits_truncate(bits)
}

/// Check if option flags include comma list
#[no_mangle]
pub extern "C" fn rs_optflags_is_comma_list(flags: u32) -> bool {
    OptFlags::from_bits_truncate(flags).contains(OptFlags::COMMA)
}

/// Check if option flags include flag list
#[no_mangle]
pub extern "C" fn rs_optflags_is_flag_list(flags: u32) -> bool {
    OptFlags::from_bits_truncate(flags).contains(OptFlags::FLAG_LIST)
}

/// Check if option can have duplicate values
#[no_mangle]
pub extern "C" fn rs_optflags_no_dup(flags: u32) -> bool {
    OptFlags::from_bits_truncate(flags).contains(OptFlags::NO_DUP)
}

/// Check if option requires redraw
#[no_mangle]
pub extern "C" fn rs_optflags_needs_redraw(flags: u32) -> bool {
    let f = OptFlags::from_bits_truncate(flags);
    f.intersects(
        OptFlags::REDR_TABL
            | OptFlags::REDR_STAT
            | OptFlags::REDR_WIN
            | OptFlags::REDR_BUF
            | OptFlags::REDR_ALL
            | OptFlags::REDR_CLEAR,
    )
}

/// Check if option is secure
#[no_mangle]
pub extern "C" fn rs_optflags_is_secure(flags: u32) -> bool {
    OptFlags::from_bits_truncate(flags).contains(OptFlags::SECURE)
}

/// Check if option should be excluded from :mkvimrc
#[no_mangle]
pub extern "C" fn rs_optflags_no_mkrc(flags: u32) -> bool {
    OptFlags::from_bits_truncate(flags).contains(OptFlags::NO_MKRC)
}

// =============================================================================
// Option Value Types
// =============================================================================

/// Option value type
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptValType {
    /// Nil value (no type)
    Nil = -1_i8 as isize,
    /// Boolean option
    Boolean = 0,
    /// Number option
    Number = 1,
    /// String option
    String = 2,
}

impl OptValType {
    /// Create from C integer
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        match value {
            0 => Self::Boolean,
            1 => Self::Number,
            2 => Self::String,
            _ => Self::Nil,
        }
    }
}

/// Check if option type is boolean
#[no_mangle]
pub extern "C" fn rs_opt_type_is_boolean(opt_type: c_int) -> bool {
    OptValType::from_c(opt_type) == OptValType::Boolean
}

/// Check if option type is number
#[no_mangle]
pub extern "C" fn rs_opt_type_is_number(opt_type: c_int) -> bool {
    OptValType::from_c(opt_type) == OptValType::Number
}

/// Check if option type is string
#[no_mangle]
pub extern "C" fn rs_opt_type_is_string(opt_type: c_int) -> bool {
    OptValType::from_c(opt_type) == OptValType::String
}

// =============================================================================
// Option Scope
// =============================================================================

/// Option scope
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OptScope {
    /// Global option value
    Global = 0,
    /// Window-local option value
    Win = 1,
    /// Buffer-local option value
    Buf = 2,
}

impl OptScope {
    /// Create from C integer
    #[must_use]
    pub fn from_c(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Global),
            1 => Some(Self::Win),
            2 => Some(Self::Buf),
            _ => None,
        }
    }
}

/// Check if scope is global
#[no_mangle]
pub extern "C" fn rs_opt_scope_is_global(scope: c_int) -> bool {
    OptScope::from_c(scope) == Some(OptScope::Global)
}

/// Check if scope is window-local
#[no_mangle]
pub extern "C" fn rs_opt_scope_is_win(scope: c_int) -> bool {
    OptScope::from_c(scope) == Some(OptScope::Win)
}

/// Check if scope is buffer-local
#[no_mangle]
pub extern "C" fn rs_opt_scope_is_buf(scope: c_int) -> bool {
    OptScope::from_c(scope) == Some(OptScope::Buf)
}

// =============================================================================
// Set Operator Types
// =============================================================================

/// :set operator types
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetOp {
    /// No operator (opt=arg)
    None = 0,
    /// Adding (opt+=arg)
    Adding = 1,
    /// Prepending (opt^=arg)
    Prepending = 2,
    /// Removing (opt-=arg)
    Removing = 3,
}

impl SetOp {
    /// Create from C integer
    #[must_use]
    pub fn from_c(value: c_int) -> Self {
        match value {
            1 => Self::Adding,
            2 => Self::Prepending,
            3 => Self::Removing,
            _ => Self::None,
        }
    }
}

/// Check if set operation is adding
#[no_mangle]
pub extern "C" fn rs_set_op_is_adding(op: c_int) -> bool {
    SetOp::from_c(op) == SetOp::Adding
}

/// Check if set operation is prepending
#[no_mangle]
pub extern "C" fn rs_set_op_is_prepending(op: c_int) -> bool {
    SetOp::from_c(op) == SetOp::Prepending
}

/// Check if set operation is removing
#[no_mangle]
pub extern "C" fn rs_set_op_is_removing(op: c_int) -> bool {
    SetOp::from_c(op) == SetOp::Removing
}

// =============================================================================
// Shortmess Flags
// =============================================================================

/// Shortmess flag characters
pub mod shm {
    pub const RO: u8 = b'r'; // Readonly message
    pub const MOD: u8 = b'm'; // Modified message
    pub const LINES: u8 = b'l'; // Lines message
    pub const WRI: u8 = b'w'; // Write message
    pub const ABBREVIATIONS: u8 = b'a'; // Abbreviations
    pub const WRITE: u8 = b'W'; // Write message (no continuation)
    pub const TRUNC: u8 = b't'; // Truncate message
    pub const TRUNCALL: u8 = b'T'; // Truncate all messages
    pub const OVER: u8 = b'o'; // Overwrite message
    pub const OVERALL: u8 = b'O'; // Overwrite all messages
    pub const SEARCH: u8 = b's'; // Search messages
    pub const ATTENTION: u8 = b'A'; // Attention messages
    pub const INTRO: u8 = b'I'; // Intro messages
    pub const COMPLETIONMENU: u8 = b'c'; // Completion menu messages
    pub const COMPLETIONSCAN: u8 = b'C'; // Completion scan messages
    pub const RECORDING: u8 = b'q'; // Recording messages
    pub const FILEINFO: u8 = b'F'; // File info messages
    pub const SEARCHCOUNT: u8 = b'S'; // Search count messages
}

/// All valid shortmess flags
pub const SHM_ALL: &[u8] = &[
    shm::RO,
    shm::MOD,
    shm::LINES,
    shm::WRI,
    shm::ABBREVIATIONS,
    shm::WRITE,
    shm::TRUNC,
    shm::TRUNCALL,
    shm::OVER,
    shm::OVERALL,
    shm::SEARCH,
    shm::ATTENTION,
    shm::INTRO,
    shm::COMPLETIONMENU,
    shm::COMPLETIONSCAN,
    shm::RECORDING,
    shm::FILEINFO,
    shm::SEARCHCOUNT,
    b'n',
    b'f',
    b'x',
    b'i',
];

/// Check if character is a valid shortmess flag
#[no_mangle]
pub extern "C" fn rs_is_valid_shm_flag(c: c_int) -> bool {
    if !(0..=127).contains(&c) {
        return false;
    }
    SHM_ALL.contains(&(c as u8))
}

// =============================================================================
// Format Options Flags
// =============================================================================

/// Format options flag characters (for 'formatoptions')
pub mod fo {
    pub const WRAP: u8 = b't'; // Auto-wrap text
    pub const WRAP_COMS: u8 = b'c'; // Auto-wrap comments
    pub const RET_COMS: u8 = b'r'; // Insert comment leader after Enter
    pub const OPEN_COMS: u8 = b'o'; // Insert comment leader after o/O
    pub const NO_OPEN_COMS: u8 = b'/'; // Don't insert comment leader after o/O
    pub const Q_COMS: u8 = b'q'; // Format comments with "gq"
    pub const Q_NUMBER: u8 = b'n'; // Recognize numbered lists
    pub const Q_SECOND: u8 = b'2'; // Use indent of second line
    pub const INSERT: u8 = b'v'; // Break at blanks in insert mode
    pub const ONE_LETTER: u8 = b'1'; // Don't break after one-letter word
    pub const WHITE_PAR: u8 = b'w'; // Trailing white space continues paragraph
    pub const AUTO: u8 = b'a'; // Auto-format paragraphs
    pub const LONG_LINES: u8 = b'l'; // Long lines not broken in insert mode
    pub const REMOVE_COMS: u8 = b'j'; // Remove comment leader when joining
    pub const MBYTE_BREAK: u8 = b'm'; // Multi-byte char can break
    pub const MBYTE_JOIN: u8 = b'M'; // No space before/after multi-byte char
    pub const MBYTE_JOIN2: u8 = b'B'; // No space between multi-byte chars
    pub const MBYTE_KEEP: u8 = b']'; // Keep blank between multi-byte chars
    pub const CROQL: u8 = b'p'; // Don't break after period
}

/// All valid format options flags
pub const FO_ALL: &[u8] = &[
    fo::WRAP,
    fo::WRAP_COMS,
    fo::RET_COMS,
    fo::OPEN_COMS,
    fo::NO_OPEN_COMS,
    fo::Q_COMS,
    fo::Q_NUMBER,
    fo::Q_SECOND,
    fo::INSERT,
    fo::ONE_LETTER,
    fo::WHITE_PAR,
    fo::AUTO,
    fo::LONG_LINES,
    fo::REMOVE_COMS,
    fo::MBYTE_BREAK,
    fo::MBYTE_JOIN,
    fo::MBYTE_JOIN2,
    fo::MBYTE_KEEP,
    fo::CROQL,
];

/// Check if character is a valid formatoptions flag
#[no_mangle]
pub extern "C" fn rs_is_valid_fo_flag(c: c_int) -> bool {
    if !(0..=127).contains(&c) {
        return false;
    }
    FO_ALL.contains(&(c as u8))
}

// =============================================================================
// CPO Flags
// =============================================================================

/// CPO (compatible options) flags
pub const CPO_VI: &[u8] = b"aABcCdDeEfFgHiIjJkKlLmMnoOpPqrRsStuvwWxXyZ$!%+<>;";

/// Check if character is a valid cpo flag
#[no_mangle]
pub extern "C" fn rs_is_valid_cpo_flag(c: c_int) -> bool {
    if !(0..=127).contains(&c) {
        return false;
    }
    CPO_VI.contains(&(c as u8))
}

// =============================================================================
// Conceal Cursor Flags
// =============================================================================

/// Conceal cursor mode flags
pub const COCU_ALL: &[u8] = b"nvic";

/// Check if character is a valid concealcursor flag
#[no_mangle]
pub extern "C" fn rs_is_valid_cocu_flag(c: c_int) -> bool {
    if !(0..=127).contains(&c) {
        return false;
    }
    COCU_ALL.contains(&(c as u8))
}

// =============================================================================
// Mouse Flags
// =============================================================================

/// Mouse mode flags
pub const MOUSE_ALL: &[u8] = b"anvih";

/// Check if character is a valid mouse flag
#[no_mangle]
pub extern "C" fn rs_is_valid_mouse_flag(c: c_int) -> bool {
    if !(0..=127).contains(&c) {
        return false;
    }
    MOUSE_ALL.contains(&(c as u8))
}

// =============================================================================
// Whichwrap Flags
// =============================================================================

/// Whichwrap flags
pub const WW_ALL: &[u8] = b"bshl<>[]~";

/// Check if character is a valid whichwrap flag
#[no_mangle]
pub extern "C" fn rs_is_valid_ww_flag(c: c_int) -> bool {
    if !(0..=127).contains(&c) {
        return false;
    }
    WW_ALL.contains(&(c as u8))
}

// =============================================================================
// Chars Option Types
// =============================================================================

/// Chars option type (fillchars or listchars)
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CharsOption {
    /// 'fillchars' option
    Fillchars = 0,
    /// 'listchars' option
    Listchars = 1,
}

impl CharsOption {
    /// Create from C integer
    #[must_use]
    pub fn from_c(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::Fillchars),
            1 => Some(Self::Listchars),
            _ => None,
        }
    }
}

/// Check if chars option is fillchars
#[no_mangle]
pub extern "C" fn rs_chars_option_is_fillchars(opt: c_int) -> bool {
    CharsOption::from_c(opt) == Some(CharsOption::Fillchars)
}

/// Check if chars option is listchars
#[no_mangle]
pub extern "C" fn rs_chars_option_is_listchars(opt: c_int) -> bool {
    CharsOption::from_c(opt) == Some(CharsOption::Listchars)
}

// =============================================================================
// String Helpers
// =============================================================================

/// Check if string option value is empty
///
/// # Safety
/// The `s` pointer must be valid for reading if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_optstr_is_empty(s: *const c_char) -> bool {
    if s.is_null() {
        return true;
    }
    *s == 0
}

/// Get length of null-terminated option string
///
/// # Safety
/// The `s` pointer must be valid for reading up to and including the null terminator.
#[no_mangle]
pub unsafe extern "C" fn rs_optstr_len(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opt_flags() {
        assert!(rs_optflags_is_comma_list(OptFlags::COMMA.bits()));
        assert!(rs_optflags_is_comma_list(OptFlags::ONE_COMMA.bits()));
        assert!(!rs_optflags_is_comma_list(OptFlags::FLAG_LIST.bits()));
        assert!(rs_optflags_is_flag_list(OptFlags::FLAG_LIST.bits()));
    }

    #[test]
    fn test_opt_val_type() {
        assert!(rs_opt_type_is_boolean(0));
        assert!(rs_opt_type_is_number(1));
        assert!(rs_opt_type_is_string(2));
    }

    #[test]
    fn test_shm_flags() {
        assert!(rs_is_valid_shm_flag(c_int::from(b'r')));
        assert!(rs_is_valid_shm_flag(c_int::from(b'A')));
        assert!(!rs_is_valid_shm_flag(c_int::from(b'Z')));
    }

    #[test]
    fn test_fo_flags() {
        assert!(rs_is_valid_fo_flag(c_int::from(b't')));
        assert!(rs_is_valid_fo_flag(c_int::from(b'c')));
        assert!(!rs_is_valid_fo_flag(c_int::from(b'Z')));
    }

    #[test]
    fn test_cpo_flags() {
        assert!(rs_is_valid_cpo_flag(c_int::from(b'a')));
        assert!(rs_is_valid_cpo_flag(c_int::from(b'$')));
        assert!(!rs_is_valid_cpo_flag(c_int::from(b'@')));
    }
}
