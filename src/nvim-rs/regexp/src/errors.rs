//! Error handling for regex operations.
//!
//! This module provides:
//! - Error types for regex compilation and execution
//! - Error message generation compatible with Vim's error system
//! - Timeout and interrupt handling
//! - Pattern complexity detection

use std::ffi::{c_char, c_int};

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    /// Display an error message.
    fn emsg(s: *const c_char);

    /// Check if user interrupted (Ctrl-C).
    fn nvim_get_got_int() -> c_int;
}

// =============================================================================
// Error Codes
// =============================================================================

/// Error codes for regex operations.
///
/// These match the return values used in C code.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegexError {
    /// No error.
    None = 0,
    /// Pattern syntax error.
    Syntax = 1,
    /// Invalid backreference.
    InvalidBackref = 2,
    /// Unmatched parenthesis.
    UnmatchedParen = 3,
    /// Unmatched bracket.
    UnmatchedBracket = 4,
    /// Pattern too complex.
    TooComplex = 5,
    /// Out of memory.
    OutOfMemory = 6,
    /// Invalid quantifier.
    InvalidQuantifier = 7,
    /// Timeout occurred.
    Timeout = 8,
    /// User interrupted.
    Interrupted = 9,
    /// NFA conversion failed.
    NfaFailed = 10,
    /// Invalid range.
    InvalidRange = 11,
    /// Too many subexpressions.
    TooManySubexpr = 12,
}

impl RegexError {
    /// Convert from C error code.
    pub fn from_code(code: c_int) -> Self {
        match code {
            0 => Self::None,
            1 => Self::Syntax,
            2 => Self::InvalidBackref,
            3 => Self::UnmatchedParen,
            4 => Self::UnmatchedBracket,
            5 => Self::TooComplex,
            6 => Self::OutOfMemory,
            7 => Self::InvalidQuantifier,
            8 => Self::Timeout,
            9 => Self::Interrupted,
            10 => Self::NfaFailed,
            11 => Self::InvalidRange,
            12 => Self::TooManySubexpr,
            _ => Self::Syntax, // Default to syntax error
        }
    }

    /// Convert to C error code.
    pub fn to_code(self) -> c_int {
        self as c_int
    }

    /// Check if this is an error (not None).
    pub fn is_error(self) -> bool {
        self != Self::None
    }
}

// =============================================================================
// Error Messages
// =============================================================================

// Static error message strings (null-terminated for C compatibility)
static E_SYNTAX: &[u8] = b"E383: Invalid search string\0";
static E_INVALID_BACKREF: &[u8] = b"E65: Illegal backreference\0";
static E_UNMATCHED_PAREN: &[u8] = b"E54: Unmatched \\(\0";
static E_UNMATCHED_BRACKET: &[u8] = b"E769: Missing ]\0";
static E_TOO_COMPLEX: &[u8] = b"E339: Pattern too long\0";
static E_OUT_OF_MEMORY: &[u8] = b"E342: Out of memory!\0";
static E_INVALID_QUANTIFIER: &[u8] = b"E871: (NFA regexp) invalid quantifier\0";
static E_TIMEOUT: &[u8] = b"E867: Search timeout\0";
static E_INTERRUPTED: &[u8] = b"E385: Search hit BOTTOM without match\0";
static E_NFA_FAILED: &[u8] = b"E874: (NFA) NFA regexp: Could not pop the stack!\0";
static E_INVALID_RANGE: &[u8] = b"E16: Invalid range\0";
static E_TOO_MANY_SUBEXPR: &[u8] = b"E872: (NFA regexp) Too many \\(\0";

/// Additional error messages for specific situations (infrastructure for future phases).
#[allow(dead_code)]
static E_TRAILING_BACKSLASH: &[u8] = b"E55: Unmatched \\)\0";
#[allow(dead_code)]
static E_EMPTY_PATTERN: &[u8] = b"E35: No previous regular expression\0";
#[allow(dead_code)]
static E_UNKNOWN_OPTION: &[u8] = b"E361: Missing :\0";
#[allow(dead_code)]
static E_NESTING_TOO_DEEP: &[u8] = b"E363: Pattern uses more memory than 'maxmempattern'\0";
#[allow(dead_code)]
static E_INVALID_CHAR_CLASS: &[u8] = b"E63: Invalid use of \\_\0";
#[allow(dead_code)]
static E_NO_Z_ALLOWED: &[u8] = b"E66: \\z( not allowed here\0";
#[allow(dead_code)]
static E_RECURSIVE_CALL: &[u8] = b"E861: Pattern not found\0";

/// Get error message for a regex error.
pub fn error_message(error: RegexError) -> *const c_char {
    let msg = match error {
        RegexError::None => return std::ptr::null(),
        RegexError::Syntax => E_SYNTAX.as_ptr(),
        RegexError::InvalidBackref => E_INVALID_BACKREF.as_ptr(),
        RegexError::UnmatchedParen => E_UNMATCHED_PAREN.as_ptr(),
        RegexError::UnmatchedBracket => E_UNMATCHED_BRACKET.as_ptr(),
        RegexError::TooComplex => E_TOO_COMPLEX.as_ptr(),
        RegexError::OutOfMemory => E_OUT_OF_MEMORY.as_ptr(),
        RegexError::InvalidQuantifier => E_INVALID_QUANTIFIER.as_ptr(),
        RegexError::Timeout => E_TIMEOUT.as_ptr(),
        RegexError::Interrupted => E_INTERRUPTED.as_ptr(),
        RegexError::NfaFailed => E_NFA_FAILED.as_ptr(),
        RegexError::InvalidRange => E_INVALID_RANGE.as_ptr(),
        RegexError::TooManySubexpr => E_TOO_MANY_SUBEXPR.as_ptr(),
    };
    msg.cast()
}

/// Display an error message for a regex error.
///
/// # Safety
/// Must be called from a valid Neovim context.
pub unsafe fn report_error(error: RegexError) {
    let msg = error_message(error);
    if !msg.is_null() {
        emsg(msg);
    }
}

// =============================================================================
// Result Type
// =============================================================================

/// Result type for regex operations.
pub type RegexResult<T> = Result<T, RegexError>;

/// Extension trait for Result with regex errors.
pub trait RegexResultExt<T> {
    /// Report error and return None.
    ///
    /// # Safety
    /// Must be called from a valid Neovim context.
    unsafe fn report_err(self) -> Option<T>;
}

impl<T> RegexResultExt<T> for RegexResult<T> {
    unsafe fn report_err(self) -> Option<T> {
        match self {
            Ok(val) => Some(val),
            Err(e) => {
                report_error(e);
                None
            }
        }
    }
}

// =============================================================================
// Interrupt Checking
// =============================================================================

/// Check if regex operation should be aborted due to interrupt.
///
/// Returns true if user pressed Ctrl-C.
///
/// # Safety
/// Must be called from a valid Neovim context.
#[inline]
pub unsafe fn regex_should_abort() -> bool {
    nvim_get_got_int() != 0
}

/// Check if user interrupted (Ctrl-C).
///
/// # Safety
/// Must be called from a valid Neovim context.
#[inline]
pub unsafe fn is_interrupted() -> bool {
    nvim_get_got_int() != 0
}

/// Abort status for regex execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbortReason {
    /// No abort needed.
    None,
    /// User pressed Ctrl-C.
    Interrupted,
}

impl AbortReason {
    /// Check if this indicates an abort.
    pub fn should_abort(self) -> bool {
        self != Self::None
    }

    /// Convert to RegexError.
    pub fn to_error(self) -> Option<RegexError> {
        match self {
            Self::None => None,
            Self::Interrupted => Some(RegexError::Interrupted),
        }
    }
}

/// Check abort status.
///
/// # Safety
/// Must be called from a valid Neovim context.
#[inline]
pub unsafe fn check_abort() -> AbortReason {
    if nvim_get_got_int() != 0 {
        AbortReason::Interrupted
    } else {
        AbortReason::None
    }
}

// =============================================================================
// Pattern Complexity
// =============================================================================

/// Maximum pattern length before warning.
pub const MAX_PATTERN_LEN: usize = 32768;

/// Maximum recursion depth for pattern matching.
pub const MAX_RECURSION_DEPTH: usize = 1000;

/// Maximum number of states in NFA.
pub const MAX_NFA_STATES: usize = 100_000;

/// Check if pattern might be too complex.
pub fn pattern_too_complex(pattern_len: usize, nfa_states: usize) -> bool {
    pattern_len > MAX_PATTERN_LEN || nfa_states > MAX_NFA_STATES
}

/// Complexity check result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Complexity {
    /// Pattern is within normal limits.
    Ok,
    /// Pattern is complex but acceptable.
    Warning,
    /// Pattern is too complex to handle.
    TooComplex,
}

/// Evaluate pattern complexity.
pub fn evaluate_complexity(pattern_len: usize, nfa_states: usize) -> Complexity {
    if nfa_states > MAX_NFA_STATES {
        Complexity::TooComplex
    } else if pattern_len > MAX_PATTERN_LEN / 2 || nfa_states > MAX_NFA_STATES / 2 {
        Complexity::Warning
    } else {
        Complexity::Ok
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get error message for error code.
#[no_mangle]
pub extern "C" fn rs_regex_error_message(code: c_int) -> *const c_char {
    error_message(RegexError::from_code(code))
}

/// Report regex error.
///
/// # Safety
/// Must be called from valid Neovim context.
#[no_mangle]
pub unsafe extern "C" fn rs_regex_report_error(code: c_int) {
    report_error(RegexError::from_code(code));
}

/// Check if regex operation should abort.
///
/// # Safety
/// Must be called from valid Neovim context.
#[no_mangle]
pub unsafe extern "C" fn rs_regex_should_abort() -> c_int {
    c_int::from(regex_should_abort())
}

/// Check abort status for regex.
///
/// Returns: 0 = no abort, 1 = interrupted.
///
/// # Safety
/// Must be called from valid Neovim context.
#[no_mangle]
pub unsafe extern "C" fn rs_regex_check_abort() -> c_int {
    match check_abort() {
        AbortReason::None => 0,
        AbortReason::Interrupted => 1,
    }
}

/// Check pattern complexity.
///
/// Returns: 0 = ok, 1 = warning, 2 = too complex.
#[no_mangle]
pub extern "C" fn rs_evaluate_complexity(pattern_len: usize, nfa_states: usize) -> c_int {
    match evaluate_complexity(pattern_len, nfa_states) {
        Complexity::Ok => 0,
        Complexity::Warning => 1,
        Complexity::TooComplex => 2,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_roundtrip() {
        for code in 0..=12 {
            let error = RegexError::from_code(code);
            // from_code doesn't necessarily roundtrip for values > 12
            if code <= 12 {
                assert_eq!(error.to_code(), code);
            }
        }
    }

    #[test]
    fn test_error_is_error() {
        assert!(!RegexError::None.is_error());
        assert!(RegexError::Syntax.is_error());
        assert!(RegexError::Timeout.is_error());
    }

    #[test]
    fn test_abort_reason() {
        assert!(!AbortReason::None.should_abort());
        assert!(AbortReason::Interrupted.should_abort());

        assert!(AbortReason::None.to_error().is_none());
        assert_eq!(
            AbortReason::Interrupted.to_error(),
            Some(RegexError::Interrupted)
        );
    }

    #[test]
    fn test_complexity_evaluation() {
        assert_eq!(evaluate_complexity(100, 100), Complexity::Ok);
        assert_eq!(
            evaluate_complexity(MAX_PATTERN_LEN / 2 + 1, 100),
            Complexity::Warning
        );
        assert_eq!(
            evaluate_complexity(100, MAX_NFA_STATES + 1),
            Complexity::TooComplex
        );
    }

    #[test]
    fn test_pattern_too_complex() {
        assert!(!pattern_too_complex(100, 100));
        assert!(pattern_too_complex(MAX_PATTERN_LEN + 1, 100));
        assert!(pattern_too_complex(100, MAX_NFA_STATES + 1));
    }

    #[test]
    fn test_error_message_not_null() {
        // All errors except None should have messages
        assert!(error_message(RegexError::None).is_null());
        assert!(!error_message(RegexError::Syntax).is_null());
        assert!(!error_message(RegexError::Timeout).is_null());
        assert!(!error_message(RegexError::TooComplex).is_null());
    }

    #[test]
    fn test_constants() {
        // Verify constants have expected values
        assert_eq!(MAX_PATTERN_LEN, 32768);
        assert_eq!(MAX_RECURSION_DEPTH, 1000);
        assert_eq!(MAX_NFA_STATES, 100_000);
    }
}
