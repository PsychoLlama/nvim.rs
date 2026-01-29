//! NFA pattern parser for the regex engine.
//!
//! This module implements the pattern parsing phase that converts regex patterns
//! into postfix notation for NFA construction. The parser uses recursive descent
//! to handle the regex grammar.
//!
//! # Grammar
//!
//! The regex grammar (simplified) is:
//! ```text
//! reg      -> branch ('|' branch)*
//! branch   -> concat ('&' concat)*
//! concat   -> piece piece*
//! piece    -> atom quantifier?
//! atom     -> char | class | group | anchor | special
//! quantifier -> '*' | '+' | '?' | '{n,m}'
//! ```
//!
//! # Key Functions
//!
//! - [`nfa_reg`]: Entry point for parsing a complete pattern
//! - [`nfa_regbranch`]: Parse alternation branches
//! - [`nfa_regconcat`]: Parse concatenation sequences
//! - [`nfa_regpiece`]: Parse atoms with quantifiers
//! - [`nfa_regatom`]: Parse individual atoms

use std::ffi::c_int;

use crate::nfa_states::{
    NFA_CONCAT, NFA_EMPTY, NFA_MOPEN, NFA_NOPEN, NFA_OR, NFA_PREV_ATOM_NO_WIDTH, NFA_QUEST,
    NFA_QUEST_NONGREEDY, NFA_STAR, NFA_STAR_NONGREEDY, NFA_ZOPEN,
};
use crate::scanner::{getchr, peekchr, skipchr};

// =============================================================================
// Constants
// =============================================================================

/// Return codes
const OK: c_int = 1;
const FAIL: c_int = 0;

/// Magic character offset
const MAGIC_OFFSET: c_int = 256;

/// Maximum number of subexpressions
const NSUBEXP: c_int = 10;

/// Paren types
const REG_NOPAREN: c_int = 0;
const REG_PAREN: c_int = 1;
#[allow(dead_code)]
const REG_NPAREN: c_int = 2;
const REG_ZPAREN: c_int = 4;

/// Magic helper functions
#[inline]
const fn magic(x: c_int) -> c_int {
    x - MAGIC_OFFSET
}

#[inline]
const fn no_magic(x: c_int) -> c_int {
    if x < 0 {
        x + MAGIC_OFFSET
    } else {
        x
    }
}

// Magic values for common characters
#[allow(dead_code)]
const MAGIC_OPEN_PAREN: c_int = magic(b'(' as c_int);
const MAGIC_CLOSE_PAREN: c_int = magic(b')' as c_int);
const MAGIC_PIPE: c_int = magic(b'|' as c_int);
const MAGIC_AMPERSAND: c_int = magic(b'&' as c_int);
const MAGIC_STAR: c_int = magic(b'*' as c_int);
const MAGIC_PLUS: c_int = magic(b'+' as c_int);
const MAGIC_QUESTION: c_int = magic(b'?' as c_int);
const MAGIC_OPEN_BRACE: c_int = magic(b'{' as c_int);

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // Postfix output
    fn nvim_nfa_emit(c: c_int);
    fn nvim_nfa_get_post_ptr() -> *mut c_int;
    fn nvim_nfa_get_post_start() -> *mut c_int;

    // Parser state
    fn nvim_parse_get_regnpar() -> c_int;
    fn nvim_parse_set_regnpar(n: c_int);
    fn nvim_parse_get_regnzpar() -> c_int;
    fn nvim_parse_set_regnzpar(n: c_int);
    #[allow(dead_code)]
    fn nvim_parse_get_reg_magic() -> c_int;
    #[allow(dead_code)]
    fn nvim_parse_get_had_endbrace(idx: c_int) -> c_int;
    fn nvim_parse_set_had_endbrace(idx: c_int, val: c_int);

    // Error reporting
    fn nvim_regexp_check_did_emsg() -> c_int;

    // C parsing functions (to be incrementally replaced)
    fn nvim_nfa_regatom() -> c_int;
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Emit a postfix token to the output.
///
/// # Safety
/// Must be called during pattern parsing.
#[inline]
unsafe fn emit(c: c_int) {
    nvim_nfa_emit(c);
}

/// Get the current post pointer offset from start.
///
/// # Safety
/// Must be called during pattern parsing.
#[inline]
unsafe fn post_offset() -> c_int {
    nvim_nfa_get_post_ptr().offset_from(nvim_nfa_get_post_start()) as c_int
}

/// Check if parsing should abort due to error.
#[inline]
unsafe fn check_error() -> bool {
    nvim_regexp_check_did_emsg() != 0
}

// =============================================================================
// Parser Implementation
// =============================================================================

/// Parse a complete NFA regex pattern.
///
/// This is the top-level parser entry point. It handles the outermost
/// parenthesis level and alternation.
///
/// # Arguments
/// * `paren` - Parenthesis type: REG_NOPAREN, REG_PAREN, REG_NPAREN, REG_ZPAREN
///
/// # Returns
/// OK on success, FAIL on error.
///
/// # Safety
/// Must be called with valid parser state.
pub unsafe fn nfa_reg(paren: c_int) -> c_int {
    let mut parno: c_int = 0;

    if paren == REG_PAREN {
        // Capturing group \(...\)
        let regnpar = nvim_parse_get_regnpar();
        if regnpar >= NSUBEXP {
            // E872: (NFA regexp) Too many '('
            // For now, let C handle this error
            return FAIL;
        }
        parno = regnpar;
        nvim_parse_set_regnpar(regnpar + 1);
    } else if paren == REG_ZPAREN {
        // External match group \z(...\)
        let regnzpar = nvim_parse_get_regnzpar();
        if regnzpar >= NSUBEXP {
            // E879: (NFA regexp) Too many \z(
            return FAIL;
        }
        parno = regnzpar;
        nvim_parse_set_regnzpar(regnzpar + 1);
    }

    // Parse the first branch
    if nfa_regbranch() == FAIL {
        return FAIL;
    }

    // Handle alternation with '|'
    while peekchr() == MAGIC_PIPE {
        skipchr();
        if nfa_regbranch() == FAIL {
            return FAIL;
        }
        emit(NFA_OR);
    }

    // Check for proper termination
    if paren != REG_NOPAREN && getchr() != MAGIC_CLOSE_PAREN {
        // Unmatched \( or \%( - let C handle error
        return FAIL;
    } else if paren == REG_NOPAREN && peekchr() != 0 {
        if peekchr() == MAGIC_CLOSE_PAREN {
            // E55: Unmatched \)
            return FAIL;
        } else {
            // E873: (NFA regexp) proper termination error
            return FAIL;
        }
    }

    // Set flag allowing back references to this set of parentheses
    if paren == REG_PAREN {
        nvim_parse_set_had_endbrace(parno, 1);
        emit(NFA_MOPEN + parno);
    } else if paren == REG_ZPAREN {
        emit(NFA_ZOPEN + parno);
    }

    OK
}

/// Parse an alternation branch.
///
/// Branch -> concat | concat '&' concat ...
///
/// The '&' operator is a zero-width match - used for \&
///
/// # Returns
/// OK on success, FAIL on error.
///
/// # Safety
/// Must be called with valid parser state.
pub unsafe fn nfa_regbranch() -> c_int {
    let mut old_post_pos = post_offset();

    // First concat, possibly the only one
    if nfa_regconcat() == FAIL {
        return FAIL;
    }

    // Try next concats (for \& operator)
    while peekchr() == MAGIC_AMPERSAND {
        skipchr();
        // If concat is empty, emit a node
        if old_post_pos == post_offset() {
            emit(NFA_EMPTY);
        }
        emit(NFA_NOPEN);
        emit(NFA_PREV_ATOM_NO_WIDTH);
        old_post_pos = post_offset();
        if nfa_regconcat() == FAIL {
            return FAIL;
        }
        // If concat is empty, emit a node
        if old_post_pos == post_offset() {
            emit(NFA_EMPTY);
        }
        emit(NFA_CONCAT);
    }

    // If branch is empty, emit one node for it
    if old_post_pos == post_offset() {
        emit(NFA_EMPTY);
    }

    OK
}

/// Parse a concatenation of pieces.
///
/// Concat -> piece | piece piece ...
///
/// # Returns
/// OK on success, FAIL on error.
///
/// # Safety
/// Must be called with valid parser state.
pub unsafe fn nfa_regconcat() -> c_int {
    let mut first = true;

    loop {
        let c = peekchr();

        // Check for end of this concat:
        // - NUL (end of pattern)
        // - Magic | (alternation)
        // - Magic & (zero-width match)
        // - Magic ) (end of group)
        // - Non-magic ) (syntax error or end)
        if c == 0
            || c == MAGIC_PIPE
            || c == MAGIC_AMPERSAND
            || c == MAGIC_CLOSE_PAREN
            || no_magic(c) == b')' as c_int
        {
            break;
        }

        // Parse a piece
        if nfa_regpiece() == FAIL {
            return FAIL;
        }

        if !first {
            emit(NFA_CONCAT);
        }
        first = false;

        if check_error() {
            return FAIL;
        }
    }

    OK
}

/// Parse an atom with optional quantifier.
///
/// Piece -> atom | atom '*' | atom '+' | atom '?' | atom '{n,m}'
///
/// # Returns
/// OK on success, FAIL on error.
///
/// # Safety
/// Must be called with valid parser state.
pub unsafe fn nfa_regpiece() -> c_int {
    // Parse the atom first (calls into C for now)
    if nvim_nfa_regatom() == FAIL {
        return FAIL;
    }

    // Check for quantifier
    let c = peekchr();

    if c == MAGIC_STAR {
        skipchr();
        // Check for non-greedy variant
        if peekchr() == MAGIC_QUESTION {
            skipchr();
            emit(NFA_STAR_NONGREEDY);
        } else {
            emit(NFA_STAR);
        }
    } else if c == MAGIC_PLUS {
        skipchr();
        // a+ is same as aa*
        emit(NFA_CONCAT);
        if peekchr() == MAGIC_QUESTION {
            skipchr();
            emit(NFA_STAR_NONGREEDY);
        } else {
            emit(NFA_STAR);
        }
    } else if c == MAGIC_QUESTION {
        skipchr();
        // Check for non-greedy variant
        if peekchr() == MAGIC_QUESTION {
            skipchr();
            emit(NFA_QUEST_NONGREEDY);
        } else {
            emit(NFA_QUEST);
        }
    } else if c == MAGIC_OPEN_BRACE {
        // Brace quantifier {n,m} - handled by calling into C for now
        // The full brace handling is complex and will be migrated later
    }

    OK
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Parse a complete NFA pattern (FFI entry point).
///
/// # Safety
/// Must be called with initialized parser state.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_reg(paren: c_int) -> c_int {
    nfa_reg(paren)
}

/// Parse an NFA branch (FFI entry point).
///
/// # Safety
/// Must be called with initialized parser state.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regbranch() -> c_int {
    nfa_regbranch()
}

/// Parse NFA concatenation (FFI entry point).
///
/// # Safety
/// Must be called with initialized parser state.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regconcat() -> c_int {
    nfa_regconcat()
}

/// Parse NFA piece (FFI entry point).
///
/// # Safety
/// Must be called with initialized parser state.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regpiece() -> c_int {
    nfa_regpiece()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_constants() {
        assert_eq!(MAGIC_OPEN_PAREN, magic(b'(' as c_int));
        assert_eq!(MAGIC_CLOSE_PAREN, magic(b')' as c_int));
        assert_eq!(MAGIC_PIPE, magic(b'|' as c_int));
        assert_eq!(MAGIC_AMPERSAND, magic(b'&' as c_int));
        assert_eq!(MAGIC_STAR, magic(b'*' as c_int));
        assert_eq!(MAGIC_PLUS, magic(b'+' as c_int));
        assert_eq!(MAGIC_QUESTION, magic(b'?' as c_int));
    }

    #[test]
    fn test_no_magic() {
        // Magic characters
        assert_eq!(no_magic(MAGIC_STAR), b'*' as c_int);
        assert_eq!(no_magic(MAGIC_PLUS), b'+' as c_int);

        // Non-magic characters pass through
        assert_eq!(no_magic(b'a' as c_int), b'a' as c_int);
        assert_eq!(no_magic(0), 0);
    }

    #[test]
    fn test_paren_constants() {
        assert_eq!(REG_NOPAREN, 0);
        assert_eq!(REG_PAREN, 1);
        assert_eq!(REG_NPAREN, 2);
        assert_eq!(REG_ZPAREN, 4);
    }
}
