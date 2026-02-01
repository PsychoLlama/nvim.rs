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

#![allow(clippy::doc_markdown)]
use std::ffi::{c_char, c_int};

use crate::equi_class::emit_nfa_equi_class;
use crate::nfa_states::{
    NFA_ADD_NL, NFA_ANY_COMPOSING, NFA_BACKREF1, NFA_BOF, NFA_BOL, NFA_BOW, NFA_CLASS_ALNUM,
    NFA_CLASS_ALPHA, NFA_CLASS_BACKSPACE, NFA_CLASS_BLANK, NFA_CLASS_CNTRL, NFA_CLASS_DIGIT,
    NFA_CLASS_ESCAPE, NFA_CLASS_FNAME, NFA_CLASS_GRAPH, NFA_CLASS_IDENT, NFA_CLASS_KEYWORD,
    NFA_CLASS_LOWER, NFA_CLASS_PRINT, NFA_CLASS_PUNCT, NFA_CLASS_RETURN, NFA_CLASS_SPACE,
    NFA_CLASS_TAB, NFA_CLASS_UPPER, NFA_CLASS_XDIGIT, NFA_COL, NFA_COL_GT, NFA_COL_LT,
    NFA_COMPOSING, NFA_CONCAT, NFA_CURSOR, NFA_EMPTY, NFA_END_COLL, NFA_END_NEG_COLL, NFA_EOF,
    NFA_EOL, NFA_EOW, NFA_FIRST_NL, NFA_LAST_NL, NFA_LNUM, NFA_LNUM_GT, NFA_LNUM_LT, NFA_MARK,
    NFA_MARK_GT, NFA_MARK_LT, NFA_MOPEN, NFA_NEWL, NFA_NOPEN, NFA_OPT_CHARS, NFA_OR,
    NFA_PREV_ATOM_JUST_BEFORE, NFA_PREV_ATOM_JUST_BEFORE_NEG, NFA_PREV_ATOM_LIKE_PATTERN,
    NFA_PREV_ATOM_NO_WIDTH, NFA_PREV_ATOM_NO_WIDTH_NEG, NFA_QUEST, NFA_QUEST_NONGREEDY, NFA_RANGE,
    NFA_STAR, NFA_START_COLL, NFA_START_NEG_COLL, NFA_STAR_NONGREEDY, NFA_VCOL, NFA_VCOL_GT,
    NFA_VCOL_LT, NFA_VISUAL, NFA_ZEND, NFA_ZOPEN, NFA_ZREF1, NFA_ZSTART,
};
use crate::parser::{read_limits, MAX_LIMIT};
use crate::scanner::{getchr, peekchr, skipchr, skipchr_keepstart, ungetchr};

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
const MAGIC_OPEN_PAREN: c_int = magic(b'(' as c_int);
const MAGIC_CLOSE_PAREN: c_int = magic(b')' as c_int);
const MAGIC_PIPE: c_int = magic(b'|' as c_int);
const MAGIC_AMPERSAND: c_int = magic(b'&' as c_int);
const MAGIC_STAR: c_int = magic(b'*' as c_int);
const MAGIC_PLUS: c_int = magic(b'+' as c_int);
const MAGIC_QUESTION: c_int = magic(b'?' as c_int);
const MAGIC_OPEN_BRACE: c_int = magic(b'{' as c_int);
const MAGIC_CARET: c_int = magic(b'^' as c_int);
const MAGIC_DOLLAR: c_int = magic(b'$' as c_int);
const MAGIC_LESS: c_int = magic(b'<' as c_int);
const MAGIC_GREATER: c_int = magic(b'>' as c_int);
const MAGIC_UNDERSCORE: c_int = magic(b'_' as c_int);
const MAGIC_DOT: c_int = magic(b'.' as c_int);
const MAGIC_TILDE: c_int = magic(b'~' as c_int);
const MAGIC_OPEN_BRACKET: c_int = magic(b'[' as c_int);
const MAGIC_PERCENT: c_int = magic(b'%' as c_int);
const MAGIC_AT: c_int = magic(b'@' as c_int);
const MAGIC_EQUAL: c_int = magic(b'=' as c_int);
const MAGIC_MINUS: c_int = magic(b'-' as c_int);

/// NUL character
const NUL: c_int = 0;

/// Newline character
const NL: c_int = b'\n' as c_int;

/// Maximum bytes in a multibyte character
const MB_MAXBYTES: c_int = 21;

/// REX_SET and REX_USE constants (for external match)
const REX_SET: c_int = 1;
const REX_USE: c_int = 2;

/// RF_HASNL flag (pattern contains newline)
const RF_HASNL: c_int = 1;

/// RF_ICASE flag (ignore case)
const RF_ICASE: c_int = 1;

/// RF_NOICASE flag (don't ignore case)
const RF_NOICASE: c_int = 2;

/// RF_ICOMBINE flag (ignore combining characters)
const RF_ICOMBINE: c_int = 4;

/// Magic mode constants
const MAGIC_NONE: c_int = 1; // \V very nomagic
const MAGIC_MODE_OFF: c_int = 2; // \M or magic off
const MAGIC_MODE_ON: c_int = 3; // \m or magic (default)
const MAGIC_ALL: c_int = 4; // \v very magic

/// CLASS_NONE constant (no character class found)
const CLASS_NONE: c_int = -1;

/// POSIX character class constants
const CLASS_ALNUM: c_int = 0;
const CLASS_ALPHA: c_int = 1;
const CLASS_BLANK: c_int = 2;
const CLASS_CNTRL: c_int = 3;
const CLASS_DIGIT: c_int = 4;
const CLASS_GRAPH: c_int = 5;
const CLASS_LOWER: c_int = 6;
const CLASS_PRINT: c_int = 7;
const CLASS_PUNCT: c_int = 8;
const CLASS_SPACE: c_int = 9;
const CLASS_UPPER: c_int = 10;
const CLASS_XDIGIT: c_int = 11;
const CLASS_TAB: c_int = 12;
const CLASS_RETURN: c_int = 13;
const CLASS_BACKSPACE: c_int = 14;
const CLASS_ESCAPE: c_int = 15;
const CLASS_IDENT: c_int = 16;
const CLASS_KEYWORD: c_int = 17;
const CLASS_FNAME: c_int = 18;

/// Characters inside [...] that need backslash escaping
const REGEXP_INRANGE: &[u8] = b"]^-n\\";

/// Abbreviation characters after '\'
const REGEXP_ABBR: &[u8] = b"nrtebdoxuU";

// =============================================================================
// FFI Declarations
// =============================================================================

#[allow(clashing_extern_declarations)]
extern "C" {
    // Postfix output
    fn nvim_nfa_emit(c: c_int);
    fn nvim_nfa_get_post_ptr() -> *mut c_int;
    fn nvim_nfa_get_post_start() -> *mut c_int;
    fn nvim_nfa_post_ptr_decr();

    // Parser state
    fn nvim_parse_get_regparse() -> *mut c_char;
    fn nvim_parse_set_regparse(p: *mut c_char);
    fn nvim_parse_get_regnpar() -> c_int;
    fn nvim_parse_set_regnpar(n: c_int);
    fn nvim_parse_get_regnzpar() -> c_int;
    fn nvim_parse_set_regnzpar(n: c_int);
    fn nvim_parse_get_reg_magic() -> c_int;
    fn nvim_parse_set_reg_magic(v: c_int);
    fn nvim_parse_set_curchr(v: c_int);
    #[allow(dead_code)]
    fn nvim_parse_get_had_endbrace(idx: c_int) -> c_int;
    fn nvim_parse_set_had_endbrace(idx: c_int, val: c_int);
    fn nvim_parse_get_regflags() -> c_int;
    fn nvim_parse_set_regflags(f: c_int);
    fn nvim_parse_get_reg_string() -> c_int;
    fn nvim_parse_get_reg_strict() -> c_int;
    fn nvim_parse_get_reg_cpo_lit() -> c_int;
    #[allow(dead_code)]
    fn nvim_parse_get_re_has_z() -> c_int;
    fn nvim_parse_set_re_has_z(v: c_int);
    fn nvim_parse_get_reg_do_extmatch() -> c_int;
    #[allow(dead_code)]
    fn nvim_parse_get_had_eol() -> c_int;
    fn nvim_parse_set_had_eol(v: c_int);
    fn nvim_parse_get_save_prev_at_start() -> c_int;

    // Parse state save/restore stack for quantifier handling (handles nested groups)
    fn nvim_save_parse_state();
    fn nvim_restore_parse_state();
    fn nvim_discard_parse_state();
    fn nvim_peek_restore_parse_state(); // Restore without popping (for brace loops)

    // Secondary state for brace quantifier - saves position after {}
    fn nvim_save_new_state();
    fn nvim_restore_new_state();

    // Postfix position tracking for brace quantifiers
    fn nvim_nfa_set_post_ptr_offset(offset: c_int);
    fn nvim_nfa_get_post_ptr_offset() -> c_int;
    fn nvim_parse_set_at_start(v: c_int);
    fn nvim_parse_set_wants_nfa(v: c_int);

    // Character classification and parsing helpers
    fn nvim_parse_get_classchars() -> *const u8;
    fn nvim_parse_get_nfa_classcodes() -> *const c_int;
    fn nvim_re_mult_next(what: *const c_char) -> c_int;
    fn nvim_seen_endbrace(refnum: c_int) -> c_int;
    fn nvim_nfa_recognize_char_class(start: *const u8, end: *const u8, extra: c_int) -> c_int;
    fn nvim_get_char_class(pp: *mut *mut c_char) -> c_int;
    fn nvim_get_equi_class(pp: *mut *mut c_char) -> c_int;
    fn nvim_get_coll_element(pp: *mut *mut c_char) -> c_int;
    fn nvim_skip_anyof(p: *const c_char) -> *const c_char;
    fn nvim_coll_get_char() -> c_int;

    // Substitute pattern
    fn nvim_get_reg_prev_sub() -> *mut c_char;

    // NFA state flags
    fn nvim_rex_set_nfa_has_backref(v: c_int);
    fn nvim_rex_set_nfa_has_zend(v: c_int);

    // UTF-8 functions
    fn utf_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_iscomposing_legacy(c: c_int) -> c_int;

    // Character translation
    fn rs_backslash_trans(c: c_int) -> c_int;

    // Number parsing
    fn rs_getdecchrs() -> i64;
    fn rs_getoctchrs() -> i64;
    fn rs_gethexchrs(maxinputlen: c_int) -> i64;

    // Error reporting
    fn nvim_regexp_check_did_emsg() -> c_int;
    fn semsg(fmt: *const c_char, ...) -> c_int;
    fn emsg(msg: *const c_char);

    // vim_strchr
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // Cursor position for \%.l etc
    fn nvim_rex_get_cursor_lnum() -> c_int;
    fn nvim_rex_get_cursor_col() -> c_int;
    fn nvim_getvvcol_curwin(vcol: *mut c_int);
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

/// Check if a character value is a magic character (negative value)
#[inline]
const fn is_magic(c: c_int) -> bool {
    c < 0
}

/// Parse a concatenation of pieces.
///
/// Concat -> piece | piece piece ...
///
/// Also handles mid-pattern magic mode switches:
/// - \Z - ignore combining characters
/// - \c - ignore case
/// - \C - match case
/// - \v - very magic mode
/// - \m - magic mode (default)
/// - \M - nomagic mode
/// - \V - very nomagic mode
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

        // Handle magic mode switches (these don't emit anything, just change state)
        let unmagic = no_magic(c);
        if is_magic(c) {
            match unmagic as u8 {
                b'Z' => {
                    // \Z - ignore combining characters
                    let regflags = nvim_parse_get_regflags();
                    nvim_parse_set_regflags(regflags | RF_ICOMBINE);
                    skipchr_keepstart();
                    continue;
                }
                b'c' => {
                    // \c - ignore case
                    let regflags = nvim_parse_get_regflags();
                    nvim_parse_set_regflags(regflags | RF_ICASE);
                    skipchr_keepstart();
                    continue;
                }
                b'C' => {
                    // \C - match case
                    let regflags = nvim_parse_get_regflags();
                    nvim_parse_set_regflags(regflags | RF_NOICASE);
                    skipchr_keepstart();
                    continue;
                }
                b'v' => {
                    // \v - very magic mode
                    nvim_parse_set_reg_magic(MAGIC_ALL);
                    skipchr_keepstart();
                    nvim_parse_set_curchr(-1);
                    continue;
                }
                b'm' => {
                    // \m - magic mode (default)
                    nvim_parse_set_reg_magic(MAGIC_MODE_ON);
                    skipchr_keepstart();
                    nvim_parse_set_curchr(-1);
                    continue;
                }
                b'M' => {
                    // \M - nomagic mode
                    nvim_parse_set_reg_magic(MAGIC_MODE_OFF);
                    skipchr_keepstart();
                    nvim_parse_set_curchr(-1);
                    continue;
                }
                b'V' => {
                    // \V - very nomagic mode
                    nvim_parse_set_reg_magic(MAGIC_NONE);
                    skipchr_keepstart();
                    nvim_parse_set_curchr(-1);
                    continue;
                }
                _ => {}
            }
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

// =============================================================================
// Helper functions for nfa_regatom
// =============================================================================

/// Translate backslash character to control character
#[inline]
unsafe fn backslash_trans(c: c_int) -> c_int {
    rs_backslash_trans(c)
}

/// Emit a POSIX character class NFA code based on class type
#[inline]
unsafe fn emit_class_code(charclass: c_int) {
    let code = match charclass {
        CLASS_ALNUM => NFA_CLASS_ALNUM,
        CLASS_ALPHA => NFA_CLASS_ALPHA,
        CLASS_BLANK => NFA_CLASS_BLANK,
        CLASS_CNTRL => NFA_CLASS_CNTRL,
        CLASS_DIGIT => NFA_CLASS_DIGIT,
        CLASS_GRAPH => NFA_CLASS_GRAPH,
        CLASS_LOWER => {
            nvim_parse_set_wants_nfa(1);
            NFA_CLASS_LOWER
        }
        CLASS_PRINT => NFA_CLASS_PRINT,
        CLASS_PUNCT => NFA_CLASS_PUNCT,
        CLASS_SPACE => NFA_CLASS_SPACE,
        CLASS_UPPER => {
            nvim_parse_set_wants_nfa(1);
            NFA_CLASS_UPPER
        }
        CLASS_XDIGIT => NFA_CLASS_XDIGIT,
        CLASS_TAB => NFA_CLASS_TAB,
        CLASS_RETURN => NFA_CLASS_RETURN,
        CLASS_BACKSPACE => NFA_CLASS_BACKSPACE,
        CLASS_ESCAPE => NFA_CLASS_ESCAPE,
        CLASS_IDENT => NFA_CLASS_IDENT,
        CLASS_KEYWORD => NFA_CLASS_KEYWORD,
        CLASS_FNAME => NFA_CLASS_FNAME,
        _ => return,
    };
    emit(code);
}

/// Parse a character atom.
///
/// This is the lowest level parsing function. It handles:
/// - Literal characters
/// - Character classes (., \s, \d, etc.)
/// - Anchors (^, $, \<, \>)
/// - Groups (\(...\), \%(...\), \z(...\))
/// - Back references (\1-\9)
/// - Special sequences (\z, \%)
/// - Character collections ([...])
///
/// # Returns
/// OK on success, FAIL on error.
///
/// # Safety
/// Must be called with valid parser state.
pub unsafe fn nfa_regatom() -> c_int {
    let old_regparse = nvim_parse_get_regparse() as *mut u8;
    let extra = 0; // for \_x patterns that need NFA_ADD_NL
    let save_prev_at_start = nvim_parse_get_save_prev_at_start();

    let c = getchr();

    match c {
        NUL => {
            // End of pattern encountered prematurely
            static E_NUL_FOUND: &[u8] = b"E865: (NFA) Regexp end encountered prematurely\0";
            emsg(E_NUL_FOUND.as_ptr() as *const c_char);
            return FAIL;
        }

        // Anchors
        c if c == MAGIC_CARET => {
            emit(NFA_BOL);
        }
        c if c == MAGIC_DOLLAR => {
            emit(NFA_EOL);
            nvim_parse_set_had_eol(1);
        }
        c if c == MAGIC_LESS => {
            emit(NFA_BOW);
        }
        c if c == MAGIC_GREATER => {
            emit(NFA_EOW);
        }

        // Underscore prefix (\_)
        _ if c == MAGIC_UNDERSCORE => {
            let c2 = no_magic(getchr());
            if c2 == NUL {
                static E_NUL_FOUND: &[u8] = b"E865: (NFA) Regexp end encountered prematurely\0";
                emsg(E_NUL_FOUND.as_ptr() as *const c_char);
                return FAIL;
            }

            if c2 == b'^' as c_int {
                // "\_^" is start-of-line
                emit(NFA_BOL);
                return OK;
            }
            if c2 == b'$' as c_int {
                // "\_$" is end-of-line
                emit(NFA_EOL);
                nvim_parse_set_had_eol(1);
                return OK;
            }

            let extra = NFA_ADD_NL;

            // "\_[" is collection plus newline
            if c2 == b'[' as c_int {
                return parse_collection(old_regparse, extra);
            }

            // "\_x" is character class plus newline - fall through to character class handling
            return parse_char_class(c2, extra, old_regparse);
        }

        // Character classes: . i I k K f F p P s S d D x X o O w W h H a A l L u U
        c if c == MAGIC_DOT
            || c == magic(b'i' as c_int)
            || c == magic(b'I' as c_int)
            || c == magic(b'k' as c_int)
            || c == magic(b'K' as c_int)
            || c == magic(b'f' as c_int)
            || c == magic(b'F' as c_int)
            || c == magic(b'p' as c_int)
            || c == magic(b'P' as c_int)
            || c == magic(b's' as c_int)
            || c == magic(b'S' as c_int)
            || c == magic(b'd' as c_int)
            || c == magic(b'D' as c_int)
            || c == magic(b'x' as c_int)
            || c == magic(b'X' as c_int)
            || c == magic(b'o' as c_int)
            || c == magic(b'O' as c_int)
            || c == magic(b'w' as c_int)
            || c == magic(b'W' as c_int)
            || c == magic(b'h' as c_int)
            || c == magic(b'H' as c_int)
            || c == magic(b'a' as c_int)
            || c == magic(b'A' as c_int)
            || c == magic(b'l' as c_int)
            || c == magic(b'L' as c_int)
            || c == magic(b'u' as c_int)
            || c == magic(b'U' as c_int) =>
        {
            return parse_char_class(c, extra, old_regparse);
        }

        // \n - newline
        c if c == magic(b'n' as c_int) => {
            let reg_string = nvim_parse_get_reg_string();
            if reg_string != 0 {
                // In a string "\n" matches a newline character
                emit(NL);
            } else {
                // In buffer text "\n" matches the end of a line
                emit(NFA_NEWL);
                let flags = nvim_parse_get_regflags();
                nvim_parse_set_regflags(flags | RF_HASNL);
            }
        }

        // \( - capturing group
        c if c == MAGIC_OPEN_PAREN => {
            if nfa_reg(REG_PAREN) == FAIL {
                return FAIL;
            }
        }

        // Misplaced operators
        c if c == MAGIC_PIPE || c == MAGIC_AMPERSAND || c == MAGIC_CLOSE_PAREN => {
            static E_MISPLACED: &[u8] = b"E866: (NFA regexp) Misplaced %c\0";
            semsg(E_MISPLACED.as_ptr() as *const c_char, no_magic(c) as c_int);
            return FAIL;
        }

        // Quantifiers that should follow an atom
        c if c == MAGIC_EQUAL
            || c == MAGIC_QUESTION
            || c == MAGIC_PLUS
            || c == MAGIC_AT
            || c == MAGIC_STAR
            || c == MAGIC_OPEN_BRACE =>
        {
            static E_MISPLACED: &[u8] = b"E866: (NFA regexp) Misplaced %c\0";
            semsg(E_MISPLACED.as_ptr() as *const c_char, no_magic(c) as c_int);
            return FAIL;
        }

        // ~ - Previous substitute pattern
        c if c == MAGIC_TILDE => {
            let reg_prev_sub = nvim_get_reg_prev_sub();
            if reg_prev_sub.is_null() {
                static E_NOPRESUB: &[u8] = b"E33: No previous substitute regular expression\0";
                emsg(E_NOPRESUB.as_ptr() as *const c_char);
                return FAIL;
            }
            // Generate as "\%(pattern\)"
            let mut lp = reg_prev_sub as *const u8;
            let mut first = true;
            while *lp != 0 {
                emit(utf_ptr2char(lp as *const c_char));
                if !first {
                    emit(NFA_CONCAT);
                }
                first = false;
                lp = lp.add(utf_ptr2len(lp as *const c_char) as usize);
            }
            emit(NFA_NOPEN);
        }

        // \1 - \9 - backreferences
        c if c == magic(b'1' as c_int)
            || c == magic(b'2' as c_int)
            || c == magic(b'3' as c_int)
            || c == magic(b'4' as c_int)
            || c == magic(b'5' as c_int)
            || c == magic(b'6' as c_int)
            || c == magic(b'7' as c_int)
            || c == magic(b'8' as c_int)
            || c == magic(b'9' as c_int) =>
        {
            let refnum = no_magic(c) - b'0' as c_int;
            if nvim_seen_endbrace(refnum) == 0 {
                return FAIL;
            }
            emit(NFA_BACKREF1 + refnum - 1);
            nvim_rex_set_nfa_has_backref(1);
        }

        // \z - various \z commands
        c if c == magic(b'z' as c_int) => {
            return parse_z_sequence();
        }

        // \% - various \% commands
        c if c == MAGIC_PERCENT => {
            return parse_percent_sequence(save_prev_at_start);
        }

        // [ - character collection
        c if c == MAGIC_OPEN_BRACKET => {
            return parse_collection(old_regparse, extra);
        }

        // Default: literal character or multibyte
        _ => {
            return parse_literal_char(c, old_regparse);
        }
    }

    OK
}

/// Parse a character class (., \s, \d, etc.)
unsafe fn parse_char_class(c: c_int, extra: c_int, _old_regparse: *mut u8) -> c_int {
    let classchars = nvim_parse_get_classchars();
    let nfa_classcodes = nvim_parse_get_nfa_classcodes();

    let p = vim_strchr(classchars as *const c_char, no_magic(c));
    if p.is_null() {
        if extra == NFA_ADD_NL {
            static E_ILL_CHAR_CLASS: &[u8] = b"E877: (NFA regexp) Invalid character class: %ld\0";
            semsg(E_ILL_CHAR_CLASS.as_ptr() as *const c_char, c as i64);
            return FAIL;
        }
        // INTERNAL error - unknown character class
        static E_INTERNAL: &[u8] = b"INTERNAL: Unknown character class char: %ld\0";
        semsg(E_INTERNAL.as_ptr() as *const c_char, c as i64);
        return FAIL;
    }

    // When '.' is followed by a composing char ignore the dot, so that
    // the composing char is matched here.
    if c == MAGIC_DOT && utf_iscomposing_legacy(peekchr()) != 0 {
        let new_old_regparse = nvim_parse_get_regparse() as *mut u8;
        let new_c = getchr();
        return parse_literal_char(new_c, new_old_regparse);
    }

    let idx = (p as usize - classchars as usize) as isize;
    emit(*nfa_classcodes.offset(idx));

    if extra == NFA_ADD_NL {
        emit(NFA_NEWL);
        emit(NFA_OR);
        let flags = nvim_parse_get_regflags();
        nvim_parse_set_regflags(flags | RF_HASNL);
    }

    OK
}

/// Parse \z sequences (\zs, \ze, \z1-\z9, \z()
unsafe fn parse_z_sequence() -> c_int {
    let c = no_magic(getchr());
    match c as u8 {
        b's' => {
            emit(NFA_ZSTART);
            static ZS: &[u8] = b"\\zs\0";
            if nvim_re_mult_next(ZS.as_ptr() as *const c_char) == 0 {
                return FAIL;
            }
        }
        b'e' => {
            emit(NFA_ZEND);
            nvim_rex_set_nfa_has_zend(1);
            static ZE: &[u8] = b"\\ze\0";
            if nvim_re_mult_next(ZE.as_ptr() as *const c_char) == 0 {
                return FAIL;
            }
        }
        b'1'..=b'9' => {
            // \z1...\z9
            let reg_do_extmatch = nvim_parse_get_reg_do_extmatch();
            if (reg_do_extmatch & REX_USE) == 0 {
                static E_Z1_NOT_ALLOWED: &[u8] = b"E66: \\z1 - \\z9 not allowed here\0";
                emsg(E_Z1_NOT_ALLOWED.as_ptr() as *const c_char);
                return FAIL;
            }
            emit(NFA_ZREF1 + (c - b'1' as c_int));
            nvim_parse_set_re_has_z(REX_USE);
        }
        b'(' => {
            // \z(
            let reg_do_extmatch = nvim_parse_get_reg_do_extmatch();
            if reg_do_extmatch != REX_SET {
                static E_Z_NOT_ALLOWED: &[u8] = b"E67: \\z( not allowed here\0";
                emsg(E_Z_NOT_ALLOWED.as_ptr() as *const c_char);
                return FAIL;
            }
            if nfa_reg(REG_ZPAREN) == FAIL {
                return FAIL;
            }
            nvim_parse_set_re_has_z(REX_SET);
        }
        _ => {
            static E867_UNKNOWN_Z: &[u8] = b"E867: (NFA) Unknown operator '\\z%c'\0";
            semsg(E867_UNKNOWN_Z.as_ptr() as *const c_char, no_magic(c));
            return FAIL;
        }
    }
    OK
}

/// Parse \% sequences
unsafe fn parse_percent_sequence(save_prev_at_start: c_int) -> c_int {
    let c = no_magic(getchr());
    match c as u8 {
        // \%( - non-capturing group
        b'(' => {
            if nfa_reg(REG_NPAREN) == FAIL {
                return FAIL;
            }
            emit(NFA_NOPEN);
        }

        // \%d, \%o, \%x, \%u, \%U - character by code
        b'd' | b'o' | b'x' | b'u' | b'U' => {
            let nr: i64 = match c as u8 {
                b'd' => rs_getdecchrs(),
                b'o' => rs_getoctchrs(),
                b'x' => rs_gethexchrs(2),
                b'u' => rs_gethexchrs(4),
                b'U' => rs_gethexchrs(8),
                _ => -1,
            };

            if nr < 0 || nr > i32::MAX as i64 {
                let reg_magic = nvim_parse_get_reg_magic();
                static E678: &[u8] = b"E678: Invalid character after %s%%[dxouU]\0";
                static EMPTY_PREFIX: &[u8] = b"\0";
                static BACKSLASH_PREFIX: &[u8] = b"\\\0";
                let prefix = if reg_magic == 4 {
                    EMPTY_PREFIX
                } else {
                    BACKSLASH_PREFIX
                };
                semsg(E678.as_ptr() as *const c_char, prefix.as_ptr());
                return FAIL;
            }
            // A NUL is stored in the text as NL
            emit(if nr == 0 { 0x0a } else { nr as c_int });
        }

        // \%^ - buffer start
        b'^' => {
            emit(NFA_BOF);
        }

        // \%$ - buffer end
        b'$' => {
            emit(NFA_EOF);
        }

        // \%# - cursor position
        b'#' => {
            let regparse = nvim_parse_get_regparse() as *const u8;
            // Check for misplaced \%#=1
            if *regparse == b'=' && *regparse.add(1) >= b'0' && *regparse.add(1) <= b'2' {
                static E_ATOM_ENGINE: &[u8] =
                    b"E1281: E64: \\%%#=%c must be at the start of the pattern\0";
                semsg(
                    E_ATOM_ENGINE.as_ptr() as *const c_char,
                    *regparse.add(1) as c_int,
                );
                return FAIL;
            }
            emit(NFA_CURSOR);
        }

        // \%V - visual selection
        b'V' => {
            emit(NFA_VISUAL);
        }

        // \%C - any composing characters
        b'C' => {
            emit(NFA_ANY_COMPOSING);
        }

        // \%[abc] - optional sequence
        b'[' => {
            let mut n = 0;
            loop {
                let c = peekchr();
                if c == b']' as c_int {
                    break;
                }
                if c == NUL {
                    let reg_magic = nvim_parse_get_reg_magic();
                    static E_MISSING_SB: &[u8] = b"E69: Missing ] after %s%%[\0";
                    static EMPTY_PREFIX: &[u8] = b"\0";
                    static BACKSLASH_PREFIX: &[u8] = b"\\\0";
                    let prefix = if reg_magic == 4 {
                        EMPTY_PREFIX
                    } else {
                        BACKSLASH_PREFIX
                    };
                    semsg(E_MISSING_SB.as_ptr() as *const c_char, prefix.as_ptr());
                    return FAIL;
                }
                // Recursive call!
                if nfa_regatom() == FAIL {
                    return FAIL;
                }
                n += 1;
            }
            getchr(); // consume the ]
            if n == 0 {
                let reg_magic = nvim_parse_get_reg_magic();
                static E_EMPTY_SB: &[u8] = b"E70: Empty %s%%[]\0";
                static EMPTY_PREFIX2: &[u8] = b"\0";
                static BACKSLASH_PREFIX2: &[u8] = b"\\\0";
                let prefix = if reg_magic == 4 {
                    EMPTY_PREFIX2
                } else {
                    BACKSLASH_PREFIX2
                };
                semsg(E_EMPTY_SB.as_ptr() as *const c_char, prefix.as_ptr());
                return FAIL;
            }
            emit(NFA_OPT_CHARS);
            emit(n);
            emit(NFA_NOPEN);
        }

        // Position matching: \%23l, \%23c, \%23v, \%<23l, \%>23l, \%'m, etc.
        _ => {
            return parse_percent_position(c, save_prev_at_start);
        }
    }
    OK
}

/// Parse \% position matching (\%23l, \%23c, \%23v, \%<23l, \%>23l, \%'m, etc.)
unsafe fn parse_percent_position(mut c: c_int, save_prev_at_start: c_int) -> c_int {
    let mut n: i64 = 0;
    let cmp = c;
    let mut cur = false;
    let mut got_digit = false;

    if c == b'<' as c_int || c == b'>' as c_int {
        c = getchr();
    }

    if no_magic(c) == b'.' as c_int {
        cur = true;
        c = getchr();
    }

    // Parse digits
    while c >= b'0' as c_int && c <= b'9' as c_int {
        if cur {
            static E_NUMBER_AFTER_DOT: &[u8] = b"E1204: No Number allowed after .: '\\%%%c'\0";
            semsg(E_NUMBER_AFTER_DOT.as_ptr() as *const c_char, no_magic(c));
            return FAIL;
        }
        if n > (i32::MAX as i64 - (c - b'0' as c_int) as i64) / 10 {
            static E_VALUE_TOO_LARGE: &[u8] = b"E1510: Value too large: %s\0";
            static OVERFLOW: &[u8] = b"overflow\0";
            semsg(
                E_VALUE_TOO_LARGE.as_ptr() as *const c_char,
                OVERFLOW.as_ptr(),
            );
            return FAIL;
        }
        n = n * 10 + (c - b'0' as c_int) as i64;
        c = getchr();
        got_digit = true;
    }

    if c == b'l' as c_int || c == b'c' as c_int || c == b'v' as c_int {
        let mut limit: i64 = i32::MAX as i64;

        if !cur && !got_digit {
            static E_MISSING_VALUE: &[u8] = b"E1273: (NFA regexp) missing value in '\\%%%c'\0";
            semsg(E_MISSING_VALUE.as_ptr() as *const c_char, no_magic(c));
            return FAIL;
        }

        if c == b'l' as c_int {
            if cur {
                n = nvim_rex_get_cursor_lnum() as i64;
            }
            // \%{n}l  \%{n}<l  \%{n}>l
            emit(if cmp == b'<' as c_int {
                NFA_LNUM_LT
            } else if cmp == b'>' as c_int {
                NFA_LNUM_GT
            } else {
                NFA_LNUM
            });
            if save_prev_at_start != 0 {
                nvim_parse_set_at_start(1);
            }
        } else if c == b'c' as c_int {
            if cur {
                n = nvim_rex_get_cursor_col() as i64 + 1;
            }
            // \%{n}c  \%{n}<c  \%{n}>c
            emit(if cmp == b'<' as c_int {
                NFA_COL_LT
            } else if cmp == b'>' as c_int {
                NFA_COL_GT
            } else {
                NFA_COL
            });
        } else {
            // c == 'v'
            if cur {
                let mut vcol: c_int = 0;
                nvim_getvvcol_curwin(&mut vcol);
                n = (vcol + 1) as i64;
            }
            // \%{n}v  \%{n}<v  \%{n}>v
            emit(if cmp == b'<' as c_int {
                NFA_VCOL_LT
            } else if cmp == b'>' as c_int {
                NFA_VCOL_GT
            } else {
                NFA_VCOL
            });
            limit = (i32::MAX / MB_MAXBYTES) as i64;
        }

        if n >= limit {
            static E_VALUE_TOO_LARGE: &[u8] = b"E1510: Value too large: %s\0";
            static OVERFLOW: &[u8] = b"overflow\0";
            semsg(
                E_VALUE_TOO_LARGE.as_ptr() as *const c_char,
                OVERFLOW.as_ptr(),
            );
            return FAIL;
        }
        emit(n as c_int);
        return OK;
    } else if no_magic(c) == b'\'' as c_int && n == 0 {
        // \%'m  \%<'m  \%>'m - mark position
        emit(if cmp == b'<' as c_int {
            NFA_MARK_LT
        } else if cmp == b'>' as c_int {
            NFA_MARK_GT
        } else {
            NFA_MARK
        });
        emit(getchr());
        return OK;
    }

    static E867_UNKNOWN_PERCENT: &[u8] = b"E867: (NFA) Unknown operator '\\%%%c'\0";
    semsg(E867_UNKNOWN_PERCENT.as_ptr() as *const c_char, no_magic(c));
    FAIL
}

/// Parse a character collection ([...])
unsafe fn parse_collection(old_regparse: *mut u8, extra: c_int) -> c_int {
    let p = nvim_parse_get_regparse() as *mut u8;
    let endp = nvim_skip_anyof(p as *const c_char) as *mut u8;

    if *endp == b']' {
        // Try to reverse engineer character classes
        let result =
            nvim_nfa_recognize_char_class(p, endp, if extra == NFA_ADD_NL { 1 } else { 0 });
        if result != FAIL {
            if (NFA_FIRST_NL..=NFA_LAST_NL).contains(&result) {
                emit(result - NFA_ADD_NL);
                emit(NFA_NEWL);
                emit(NFA_OR);
            } else {
                emit(result);
            }
            nvim_parse_set_regparse(endp as *mut c_char);
            // Advance past ]
            let regparse = nvim_parse_get_regparse() as *mut u8;
            let len = utf_ptr2len(regparse as *const c_char);
            nvim_parse_set_regparse(regparse.add(len as usize) as *mut c_char);
            return OK;
        }

        // Failed to recognize a character class. Use the simple version.
        let mut negated = false;
        let mut regparse = nvim_parse_get_regparse() as *mut u8;
        if *regparse == b'^' {
            negated = true;
            regparse = regparse.add(utf_ptr2len(regparse as *const c_char) as usize);
            nvim_parse_set_regparse(regparse as *mut c_char);
            emit(NFA_START_NEG_COLL);
        } else {
            emit(NFA_START_COLL);
        }

        regparse = nvim_parse_get_regparse() as *mut u8;
        let mut startc: c_int = -1;
        if *regparse == b'-' {
            startc = b'-' as c_int;
            emit(startc);
            emit(NFA_CONCAT);
            regparse = regparse.add(utf_ptr2len(regparse as *const c_char) as usize);
            nvim_parse_set_regparse(regparse as *mut c_char);
        }

        // Parse the collection
        let mut emit_range = false;
        let mut extra_newl = extra;

        while (nvim_parse_get_regparse() as *mut u8) < endp {
            let oldstartc = startc;
            startc = -1;
            let mut got_coll_char = false;
            regparse = nvim_parse_get_regparse() as *mut u8;

            if *regparse == b'[' {
                // Check for [: :], [= =], [. .]
                let mut pp = regparse as *mut c_char;
                let charclass = nvim_get_char_class(&mut pp);
                if charclass != CLASS_NONE {
                    nvim_parse_set_regparse(pp);
                    emit_class_code(charclass);
                    emit(NFA_CONCAT);
                    continue;
                }

                let equiclass = nvim_get_equi_class(&mut pp);
                if equiclass != 0 {
                    nvim_parse_set_regparse(pp);
                    emit_nfa_equi_class(equiclass);
                    continue;
                }

                let collclass = nvim_get_coll_element(&mut pp);
                if collclass != 0 {
                    nvim_parse_set_regparse(pp);
                    startc = collclass;
                }
            }

            regparse = nvim_parse_get_regparse() as *mut u8;
            // Try a range like 'a-x' or '\t-z'
            if *regparse == b'-' && oldstartc != -1 {
                emit_range = true;
                startc = oldstartc;
                regparse = regparse.add(utf_ptr2len(regparse as *const c_char) as usize);
                nvim_parse_set_regparse(regparse as *mut c_char);
                continue;
            }

            regparse = nvim_parse_get_regparse() as *mut u8;
            // Handle simple and escaped characters
            if *regparse == b'\\'
                && (regparse as *const u8).add(1) <= endp
                && (!vim_strchr(
                    REGEXP_INRANGE.as_ptr() as *const c_char,
                    *regparse.add(1) as c_int,
                )
                .is_null()
                    || (nvim_parse_get_reg_cpo_lit() == 0
                        && !vim_strchr(
                            REGEXP_ABBR.as_ptr() as *const c_char,
                            *regparse.add(1) as c_int,
                        )
                        .is_null()))
            {
                regparse = regparse.add(utf_ptr2len(regparse as *const c_char) as usize);
                nvim_parse_set_regparse(regparse as *mut c_char);
                regparse = nvim_parse_get_regparse() as *mut u8;

                if *regparse == b'n' {
                    let reg_string = nvim_parse_get_reg_string();
                    startc = if reg_string != 0 || emit_range || *regparse.add(1) == b'-' {
                        NL
                    } else {
                        NFA_NEWL
                    };
                } else if *regparse == b'd'
                    || *regparse == b'o'
                    || *regparse == b'x'
                    || *regparse == b'u'
                    || *regparse == b'U'
                {
                    startc = nvim_coll_get_char();
                    if startc == i32::MAX {
                        static E_UNICODE_TOO_LARGE: &[u8] =
                            b"E1541: Unicode value too large: \\u%x\0";
                        emsg(E_UNICODE_TOO_LARGE.as_ptr() as *const c_char);
                        return FAIL;
                    }
                    got_coll_char = true;
                    // Back up regparse
                    regparse = nvim_parse_get_regparse() as *mut u8;
                    let len = utf_ptr2len(regparse as *const c_char);
                    if len > 0 {
                        nvim_parse_set_regparse(regparse.sub(len as usize) as *mut c_char);
                    }
                } else {
                    startc = backslash_trans(*regparse as c_int);
                }
            }

            regparse = nvim_parse_get_regparse() as *mut u8;
            // Normal printable char
            if startc == -1 {
                startc = utf_ptr2char(regparse as *const c_char);
            }

            // Previous char was '-', so this char is end of range
            if emit_range {
                let endc = startc;
                startc = oldstartc;
                if startc > endc {
                    static E_REVERSE_RANGE: &[u8] = b"E16: Invalid range\0";
                    emsg(E_REVERSE_RANGE.as_ptr() as *const c_char);
                    return FAIL;
                }

                if endc > startc + 2 {
                    // Emit a range instead of sequence
                    if startc == 0 {
                        emit(1); // \x00 is translated to \x0a, start at \x01
                    } else {
                        nvim_nfa_post_ptr_decr(); // remove NFA_CONCAT
                    }
                    emit(endc);
                    emit(NFA_RANGE);
                    emit(NFA_CONCAT);
                } else {
                    // Emit the characters in the range
                    let mut ch = startc + 1;
                    while ch <= endc {
                        emit(ch);
                        emit(NFA_CONCAT);
                        ch += 1;
                    }
                }
                emit_range = false;
                startc = -1;
            } else {
                // Not part of a range, just emit it
                if startc == NFA_NEWL {
                    if !negated {
                        extra_newl = NFA_ADD_NL;
                    }
                } else if got_coll_char && startc == 0 {
                    emit(0x0a);
                    emit(NFA_CONCAT);
                } else {
                    emit(startc);
                    regparse = nvim_parse_get_regparse() as *mut u8;
                    let len1 = utf_ptr2len(regparse as *const c_char);
                    let len2 = utfc_ptr2len(regparse as *const c_char);
                    if len1 == len2 {
                        emit(NFA_CONCAT);
                    }
                }
            }

            // Handle composing characters
            regparse = nvim_parse_get_regparse() as *mut u8;
            let len1 = utf_ptr2len(regparse as *const c_char);
            let plen = utfc_ptr2len(regparse as *const c_char);
            if len1 != plen {
                let mut i = len1;
                let mut ch = utf_ptr2char(regparse.add(i as usize) as *const c_char);
                loop {
                    if ch == 0 {
                        emit(1);
                    } else {
                        emit(ch);
                    }
                    emit(NFA_CONCAT);
                    i += utf_char2len(ch);
                    if i >= plen {
                        break;
                    }
                    ch = utf_ptr2char(regparse.add(i as usize) as *const c_char);
                }
                emit(NFA_COMPOSING);
                emit(NFA_CONCAT);
            }

            // Advance regparse
            regparse = nvim_parse_get_regparse() as *mut u8;
            let len = utf_ptr2len(regparse as *const c_char);
            nvim_parse_set_regparse(regparse.add(len as usize) as *mut c_char);
        }

        // Check for trailing '-'
        regparse = nvim_parse_get_regparse() as *mut u8;
        if regparse > old_regparse {
            let len = utf_ptr2len((regparse as *const u8).sub(1) as *const c_char);
            let prev = regparse.sub(len as usize);
            if *prev == b'-' {
                emit(b'-' as c_int);
                emit(NFA_CONCAT);
            }
        }

        // Skip the trailing ]
        nvim_parse_set_regparse(endp as *mut c_char);
        regparse = nvim_parse_get_regparse() as *mut u8;
        let len = utf_ptr2len(regparse as *const c_char);
        nvim_parse_set_regparse(regparse.add(len as usize) as *mut c_char);

        // Mark end of the collection
        if negated {
            emit(NFA_END_NEG_COLL);
        } else {
            emit(NFA_END_COLL);
        }

        // \_[] also matches \n but it's not negated
        if extra_newl == NFA_ADD_NL {
            let reg_string = nvim_parse_get_reg_string();
            emit(if reg_string != 0 { NL } else { NFA_NEWL });
            emit(NFA_OR);
        }

        return OK;
    }

    // No closing ]
    let reg_strict = nvim_parse_get_reg_strict();
    if reg_strict != 0 {
        static E_MISSING_BRACKET: &[u8] = b"E69: Missing ] after \\%%[\0";
        emsg(E_MISSING_BRACKET.as_ptr() as *const c_char);
        return FAIL;
    }

    // Fall through to literal character handling
    ungetchr();
    let c = getchr();
    parse_literal_char(c, old_regparse)
}

/// Parse a literal character (possibly multibyte/composing)
unsafe fn parse_literal_char(c: c_int, old_regparse: *mut u8) -> c_int {
    // plen is length of current char with composing chars
    let charlen = utf_char2len(c);
    let plen = utfc_ptr2len(old_regparse as *const c_char);

    if charlen != plen || utf_iscomposing_legacy(c) != 0 {
        // A base character plus composing characters, or just one
        // or more composing characters.
        let mut i: c_int = 0;
        let mut ch = c;
        loop {
            emit(ch);
            if i > 0 {
                emit(NFA_CONCAT);
            }
            i += utf_char2len(ch);
            if i >= plen {
                break;
            }
            ch = utf_ptr2char(old_regparse.add(i as usize) as *const c_char);
        }
        emit(NFA_COMPOSING);
        nvim_parse_set_regparse(old_regparse.add(plen as usize) as *mut c_char);
    } else {
        let unmagic_c = no_magic(c);
        emit(unmagic_c);
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
    // Save parse state before atom - needed for \+ and \{} which re-parse the atom.
    // This uses a stack because nfa_regpiece can be called recursively (nested groups).
    nvim_save_parse_state();

    // Save postfix position before atom - needed for \{} to discard the atom
    let my_post_start = nvim_nfa_get_post_ptr_offset();

    // Parse the atom first
    if nfa_regatom() == FAIL {
        nvim_discard_parse_state(); // Clean up stack on failure
        return FAIL;
    }

    // Check for quantifier
    let c = peekchr();

    if c == MAGIC_STAR {
        nvim_discard_parse_state(); // Don't need saved state for *
        skipchr();
        // Check for non-greedy variant
        if peekchr() == MAGIC_QUESTION {
            skipchr();
            emit(NFA_STAR_NONGREEDY);
        } else {
            emit(NFA_STAR);
        }
    } else if c == MAGIC_PLUS {
        // a+ is same as aa* - need to re-parse atom to emit it twice
        // Restore parse state to before the atom (pops from stack)
        nvim_restore_parse_state();
        // Re-parse atom (emits second copy)
        if nfa_regatom() == FAIL {
            return FAIL;
        }
        skipchr(); // skip the \+
                   // Check for non-greedy variant
        if peekchr() == MAGIC_QUESTION {
            skipchr();
            emit(NFA_STAR_NONGREEDY);
        } else {
            emit(NFA_STAR);
        }
        emit(NFA_CONCAT);
    } else if c == MAGIC_QUESTION || c == MAGIC_EQUAL {
        // \? and \= both mean zero-or-one (optional)
        nvim_discard_parse_state(); // Don't need saved state for ?/=
        skipchr();
        // Check for non-greedy variant (\?? or \=?)
        if peekchr() == MAGIC_QUESTION {
            skipchr();
            emit(NFA_QUEST_NONGREEDY);
        } else {
            emit(NFA_QUEST);
        }
    } else if c == MAGIC_OPEN_BRACE {
        // Brace quantifier {n,m}
        // Skip the opening brace
        skipchr();

        // Check for non-greedy: \{-n,m}
        // Note: read_limits() handles the '-' internally, so we just need to call it
        let greedy = {
            let c2 = peekchr();
            !(c2 == b'-' as c_int || c2 == MAGIC_MINUS)
        };

        // Read the limits
        let mut minval: c_int = 0;
        let mut maxval: c_int = 0;
        if read_limits(&mut minval, &mut maxval) == FAIL {
            nvim_discard_parse_state();
            static E_READ_LIMITS: &[u8] = b"E870: (NFA regexp) Error reading repetition limits\0";
            emsg(E_READ_LIMITS.as_ptr() as *const c_char);
            return FAIL;
        }

        // Special case: {0,} or {} is equivalent to *
        if minval == 0 && maxval == MAX_LIMIT {
            nvim_discard_parse_state();
            if greedy {
                emit(NFA_STAR);
            } else {
                emit(NFA_STAR_NONGREEDY);
            }
            return OK;
        }

        // Special case: {0} - discard the atom entirely
        if maxval == 0 {
            nvim_discard_parse_state();
            // Reset postfix to before atom was parsed
            nvim_nfa_set_post_ptr_offset(my_post_start);
            emit(NFA_EMPTY);
            return OK;
        }

        // General case: expand {n,m} to aaa?a?... (n required, m-n optional)
        // Discard the atom we already parsed - we'll re-parse it multiple times
        nvim_nfa_set_post_ptr_offset(my_post_start);

        // Save parse state AFTER the {} - this is where we'll continue after the loop
        // The stack already has old_state (before atom), use secondary slot for new_state
        nvim_save_new_state();

        let quest = if greedy {
            NFA_QUEST
        } else {
            NFA_QUEST_NONGREEDY
        };

        for i in 0..maxval {
            // Restore parse state to before the atom (without popping)
            nvim_peek_restore_parse_state();

            let old_post_pos = nvim_nfa_get_post_ptr_offset();
            if nfa_regatom() == FAIL {
                nvim_discard_parse_state(); // Clean up stack
                return FAIL;
            }

            // After minval times, atoms become optional
            if i + 1 > minval {
                if maxval == MAX_LIMIT {
                    // Unbounded max: use STAR for remaining
                    if greedy {
                        emit(NFA_STAR);
                    } else {
                        emit(NFA_STAR_NONGREEDY);
                    }
                } else {
                    emit(quest);
                }
            }

            // Add CONCAT if not first atom
            if old_post_pos != my_post_start {
                emit(NFA_CONCAT);
            }

            // If we're past minval and maxval is unlimited, we're done
            // (the STAR handles the rest)
            if i + 1 > minval && maxval == MAX_LIMIT {
                break;
            }
        }

        // Clean up the stack (pop old_state) and restore to after {}
        nvim_discard_parse_state();
        nvim_restore_new_state();

        return OK;
    } else if c == MAGIC_AT {
        // Zero-width assertions: \@=, \@!, \@<=, \@<!, \@>
        nvim_discard_parse_state();

        // Get optional count (e.g., \@123=)
        let c2 = rs_getdecchrs();

        // Skip the @ and get the operator
        skipchr();
        let op = no_magic(getchr());

        let nfa_state = match op as u8 {
            b'=' => NFA_PREV_ATOM_NO_WIDTH,     // \@=
            b'!' => NFA_PREV_ATOM_NO_WIDTH_NEG, // \@!
            b'<' => {
                // Look-behind: \@<= or \@<!
                let op2 = no_magic(getchr());
                match op2 as u8 {
                    b'=' => NFA_PREV_ATOM_JUST_BEFORE,     // \@<=
                    b'!' => NFA_PREV_ATOM_JUST_BEFORE_NEG, // \@<!
                    _ => {
                        static E_UNKNOWN: &[u8] = b"E869: (NFA) Unknown operator '\\@<%c'\0";
                        semsg(E_UNKNOWN.as_ptr() as *const c_char, op2);
                        return FAIL;
                    }
                }
            }
            b'>' => NFA_PREV_ATOM_LIKE_PATTERN, // \@>
            _ => {
                static E_UNKNOWN: &[u8] = b"E869: (NFA) Unknown operator '\\@%c'\0";
                semsg(E_UNKNOWN.as_ptr() as *const c_char, op);
                return FAIL;
            }
        };

        // Emit optional count
        if c2 > 0 {
            emit(c2 as c_int);
        }
        emit(nfa_state);
    } else {
        // No quantifier - discard the saved state
        nvim_discard_parse_state();
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

/// Parse NFA atom (FFI entry point).
///
/// # Safety
/// Must be called with initialized parser state.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_regatom() -> c_int {
    nfa_regatom()
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
