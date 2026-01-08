//! NFA pattern parsing for the regex engine.
//!
//! This module implements the pattern-to-postfix conversion phase of the NFA regex engine.
//! It converts regex patterns into a postfix representation that can be consumed by `post2nfa()`.
//!
//! # Pattern Parsing Hierarchy
//!
//! The parsing is structured as a recursive descent parser:
//! - `nfa_reg()` - parses pattern with alternations (\|)
//! - `nfa_regbranch()` - parses branches with lookahead (\&)
//! - `nfa_regconcat()` - parses concatenation of pieces
//! - `nfa_regpiece()` - parses atom with optional quantifier
//! - `nfa_regatom()` - parses a single atom
//!
//! # Postfix Representation
//!
//! The postfix buffer contains a sequence of integers where:
//! - Positive values are literal characters or NFA opcodes
//! - NFA_CONCAT, NFA_OR, etc. are operators that combine fragments

use std::ffi::{c_char, c_int};

use crate::nfa_states::{
    NFA_ANY, NFA_BACKREF1, NFA_BOF, NFA_BOL, NFA_BOW, NFA_CLASS_ALNUM, NFA_CLASS_ALPHA,
    NFA_CLASS_BACKSPACE, NFA_CLASS_BLANK, NFA_CLASS_CNTRL, NFA_CLASS_DIGIT, NFA_CLASS_ESCAPE,
    NFA_CLASS_FNAME, NFA_CLASS_GRAPH, NFA_CLASS_IDENT, NFA_CLASS_KEYWORD, NFA_CLASS_LOWER,
    NFA_CLASS_PRINT, NFA_CLASS_PUNCT, NFA_CLASS_RETURN, NFA_CLASS_SPACE, NFA_CLASS_TAB,
    NFA_CLASS_UPPER, NFA_CLASS_XDIGIT, NFA_COL, NFA_COL_GT, NFA_COL_LT, NFA_COMPOSING, NFA_CONCAT,
    NFA_CURSOR, NFA_EMPTY, NFA_END_COLL, NFA_END_NEG_COLL, NFA_EOF, NFA_EOL, NFA_EOW, NFA_LNUM,
    NFA_LNUM_GT, NFA_LNUM_LT, NFA_MARK, NFA_MARK_GT, NFA_MARK_LT, NFA_MOPEN, NFA_NEWL, NFA_NOPEN,
    NFA_OR, NFA_PREV_ATOM_JUST_BEFORE, NFA_PREV_ATOM_JUST_BEFORE_NEG, NFA_PREV_ATOM_LIKE_PATTERN,
    NFA_PREV_ATOM_NO_WIDTH, NFA_PREV_ATOM_NO_WIDTH_NEG, NFA_QUEST, NFA_QUEST_NONGREEDY, NFA_RANGE,
    NFA_SKIP, NFA_STAR, NFA_STAR_NONGREEDY, NFA_VCOL, NFA_VCOL_GT, NFA_VCOL_LT, NFA_VISUAL,
    NFA_ZEND, NFA_ZOPEN, NFA_ZREF1, NFA_ZSTART,
};
use crate::parser::{read_limits, MAX_LIMIT};
use crate::scanner::{getchr, peekchr, skipchr, skipchr_keepstart, ungetchr};
use crate::{re_multi_type_impl, NOT_MULTI};

// =============================================================================
// Constants
// =============================================================================

/// Return codes
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;

/// Paren types for nfa_reg()
pub const REG_NOPAREN: c_int = 0; // toplevel, no parens
pub const REG_PAREN: c_int = 1; // \(\)
pub const REG_NPAREN: c_int = 2; // \%(\)
pub const REG_ZPAREN: c_int = 3; // \z(\)

/// Magic character offset
const MAGIC_OFFSET: c_int = 256;

/// Magic modes
const MAGIC_NONE: c_int = 1;
const MAGIC_OFF: c_int = 2;
const MAGIC_ON: c_int = 3;
const MAGIC_ALL: c_int = 4;

/// Regex flags
const RF_ICASE: u32 = 1;
const RF_NOICASE: u32 = 2;
const RF_ICOMBINE: u32 = 8;

/// Character class values (must match char_class.rs)
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
const CLASS_NONE: c_int = 99;

/// Number of subexpressions
pub const NSUBEXP: usize = 10;

/// Newline character
const NL: c_int = 10;

// =============================================================================
// FFI Declarations
// =============================================================================

extern "C" {
    // Parse state accessors
    fn nvim_parse_get_regparse() -> *mut c_char;
    fn nvim_parse_set_regparse(p: *mut c_char);
    fn nvim_parse_get_reg_magic() -> c_int;
    fn nvim_parse_set_reg_magic(m: c_int);
    fn nvim_parse_get_curchr() -> c_int;
    fn nvim_parse_set_curchr(c: c_int);
    fn nvim_parse_get_regnpar() -> c_int;
    fn nvim_parse_set_regnpar(n: c_int);

    // Parenthesis tracking
    fn nvim_parse_get_regnzpar() -> c_int;
    fn nvim_parse_set_regnzpar(n: c_int);
    fn nvim_parse_get_had_endbrace(i: c_int) -> c_int;
    fn nvim_parse_set_had_endbrace(i: c_int, v: c_int);

    // Regex flags
    fn nvim_parse_get_regflags() -> c_int;
    fn nvim_parse_set_regflags(f: c_int);
    fn nvim_parse_get_reg_string() -> c_int;

    // Error reporting
    fn nvim_regexp_report_error(error_id: c_int, is_magic_all: c_int);

    // UTF-8 functions
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_char2len(c: c_int) -> c_int;
    fn rs_utf_iscomposing_legacy(c: c_int) -> c_int;

    // Character class parsing
    fn nvim_get_char_class(pp: *mut *mut c_char) -> c_int;

    // Collating elements and equivalence classes
    fn rs_get_equi_class(pp: *mut *mut c_char) -> c_int;
    fn rs_get_coll_element(pp: *mut *mut c_char) -> c_int;

    // Number parsing
    fn rs_getdecchrs() -> i64;
    fn rs_gethexchrs(maxinputlen: c_int) -> i64;
    fn rs_getoctchrs() -> i64;

    // NFA state accessors
    fn nvim_rex_set_nfa_has_zend(v: c_int);
    fn nvim_rex_set_nfa_has_backref(v: c_int);

    // Global parser state
    fn nvim_parse_get_reg_cpo_lit() -> c_int;
    fn nvim_parse_get_reg_strict() -> c_int;

    // Pattern wants NFA engine
    fn nvim_parse_get_wants_nfa() -> c_int;
    fn nvim_parse_set_wants_nfa(v: c_int);

    // RE_AUTO flag
    fn nvim_parse_get_nfa_re_flags() -> c_int;
}

// =============================================================================
// Magic Functions
// =============================================================================

/// Convert an ASCII character to its magic form.
#[inline]
const fn magic(x: c_int) -> c_int {
    x - MAGIC_OFFSET
}

/// Convert a magic character back to its ASCII value.
#[inline]
const fn un_magic(x: c_int) -> c_int {
    x + MAGIC_OFFSET
}

/// Check if a character is magic (negative value).
#[inline]
const fn is_magic(x: c_int) -> bool {
    x < 0
}

/// Remove magic from a character.
#[inline]
const fn no_magic(x: c_int) -> c_int {
    if is_magic(x) {
        un_magic(x)
    } else {
        x
    }
}

// =============================================================================
// Postfix Buffer
// =============================================================================

/// Buffer for building postfix representation of regex.
pub struct PostfixBuffer {
    /// The postfix opcodes
    data: Vec<c_int>,
    /// Position in data for \{n,m} handling
    start_pos: usize,
}

impl PostfixBuffer {
    /// Create a new postfix buffer with estimated capacity.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            data: Vec::with_capacity(cap),
            start_pos: 0,
        }
    }

    /// Emit an opcode to the buffer.
    #[inline]
    pub fn emit(&mut self, c: c_int) {
        self.data.push(c);
    }

    /// Get current position in the buffer.
    #[inline]
    pub fn pos(&self) -> usize {
        self.data.len()
    }

    /// Truncate buffer to given position (for backtracking).
    #[inline]
    pub fn truncate(&mut self, pos: usize) {
        self.data.truncate(pos);
    }

    /// Get reference to the postfix data.
    pub fn data(&self) -> &[c_int] {
        &self.data
    }

    /// Remove the last element if it equals the given value.
    pub fn remove_last_if(&mut self, expected: c_int) -> bool {
        if self.data.last() == Some(&expected) {
            self.data.pop();
            true
        } else {
            false
        }
    }

    /// Set the start position marker.
    pub fn mark_start(&mut self) {
        self.start_pos = self.data.len();
    }

    /// Get the start position marker.
    pub fn get_start(&self) -> usize {
        self.start_pos
    }
}

// =============================================================================
// Parse State
// =============================================================================

/// Saved parse state for backtracking in quantifier handling.
#[derive(Clone)]
pub struct ParseState {
    regparse: *mut c_char,
    prevchr_len: c_int,
    curchr: c_int,
    prevchr: c_int,
    prevprevchr: c_int,
    nextchr: c_int,
    at_start: c_int,
    prev_at_start: c_int,
}

impl ParseState {
    /// Save current parse state.
    ///
    /// # Safety
    /// Must be called when parse state is valid.
    pub unsafe fn save() -> Self {
        extern "C" {
            fn nvim_parse_get_prevchr_len() -> c_int;
            fn nvim_parse_get_prevchr() -> c_int;
            fn nvim_parse_get_prevprevchr() -> c_int;
            fn nvim_parse_get_nextchr() -> c_int;
            fn nvim_parse_get_at_start() -> c_int;
            fn nvim_parse_get_prev_at_start() -> c_int;
        }

        Self {
            regparse: nvim_parse_get_regparse(),
            prevchr_len: nvim_parse_get_prevchr_len(),
            curchr: nvim_parse_get_curchr(),
            prevchr: nvim_parse_get_prevchr(),
            prevprevchr: nvim_parse_get_prevprevchr(),
            nextchr: nvim_parse_get_nextchr(),
            at_start: nvim_parse_get_at_start(),
            prev_at_start: nvim_parse_get_prev_at_start(),
        }
    }

    /// Restore parse state.
    ///
    /// # Safety
    /// Must restore to a previously valid state.
    pub unsafe fn restore(&self) {
        extern "C" {
            fn nvim_parse_set_prevchr_len(len: c_int);
            fn nvim_parse_set_prevchr(c: c_int);
            fn nvim_parse_set_prevprevchr(c: c_int);
            fn nvim_parse_set_nextchr(c: c_int);
            fn nvim_parse_set_at_start(v: c_int);
            fn nvim_parse_set_prev_at_start(v: c_int);
        }

        nvim_parse_set_regparse(self.regparse);
        nvim_parse_set_prevchr_len(self.prevchr_len);
        nvim_parse_set_curchr(self.curchr);
        nvim_parse_set_prevchr(self.prevchr);
        nvim_parse_set_prevprevchr(self.prevprevchr);
        nvim_parse_set_nextchr(self.nextchr);
        nvim_parse_set_at_start(self.at_start);
        nvim_parse_set_prev_at_start(self.prev_at_start);
    }
}

// =============================================================================
// Error IDs
// =============================================================================

// Error message IDs (must match error codes in C)
const E_MISSINGBRACKET: c_int = 769;
const E_REVERSE_RANGE: c_int = 770;
const E_TOO_MANY_PAREN: c_int = 872;
const E_TOO_MANY_ZPAREN: c_int = 879;
const E_UNMATCHED_PAREN: c_int = 873;
const E_UNMATCHED_CLOSE_PAREN: c_int = 874;
const E_NESTED_STAR: c_int = 871;
const E_UNKNOWN_AT: c_int = 869;
const E_READ_LIMITS: c_int = 870;

// =============================================================================
// Helper Functions
// =============================================================================

/// Characters always special in [] range after '\'
const REGEXP_INRANGE: &[u8] = b"]^-n\\";

/// Abbreviation characters after '\'
const REGEXP_ABBR: &[u8] = b"nrtebdoxuU";

/// Check if a byte is in a byte slice.
#[inline]
fn byte_in_slice(b: u8, slice: &[u8]) -> bool {
    slice.contains(&b)
}

/// Translate '\x' to its control character.
#[inline]
const fn backslash_trans(c: c_int) -> c_int {
    match c as u8 {
        b'r' => 13, // CAR
        b't' => 9,  // TAB
        b'e' => 27, // ESC
        b'b' => 8,  // BS
        _ => c,
    }
}

/// Get a character for a collection: \d, \o, \x, \u, \U.
///
/// # Safety
/// regparse must point to valid string.
unsafe fn coll_get_char() -> c_int {
    let regparse = nvim_parse_get_regparse();
    let c = *regparse as u8;

    let nr = match c {
        b'd' => {
            nvim_parse_set_regparse(regparse.add(1));
            rs_getdecchrs()
        }
        b'o' => {
            nvim_parse_set_regparse(regparse.add(1));
            rs_getoctchrs()
        }
        b'x' => {
            nvim_parse_set_regparse(regparse.add(1));
            rs_gethexchrs(2)
        }
        b'u' => {
            nvim_parse_set_regparse(regparse.add(1));
            rs_gethexchrs(4)
        }
        b'U' => {
            nvim_parse_set_regparse(regparse.add(1));
            rs_gethexchrs(8)
        }
        _ => -1,
    };

    if nr < 0 || nr > i64::from(i32::MAX) {
        // Invalid or too large
        i32::MAX
    } else {
        nr as c_int
    }
}

// =============================================================================
// NFA Pattern Parser
// =============================================================================

/// Parse the lowest level: an atom.
///
/// An atom can be:
/// - A literal character
/// - A character class (., \w, \d, etc.)
/// - A bracket expression [...]
/// - A grouped subexpression \(...\)
/// - An anchor (^, $, \<, \>, etc.)
/// - A position match (\%23l, \%5c, etc.)
/// - A backreference (\1-\9)
/// - And more...
///
/// # Safety
/// Parse state must be valid.
pub unsafe fn nfa_regatom(buf: &mut PostfixBuffer) -> c_int {
    let c = getchr();

    if c == 0 {
        // End of pattern
        return FAIL;
    }

    let old_regparse = nvim_parse_get_regparse();

    // Handle based on the character
    let no_magic_c = no_magic(c);

    // Magic characters that have special meaning
    if no_magic_c == b'^' as c_int && is_magic(c) {
        buf.emit(NFA_BOL);
    } else if no_magic_c == b'$' as c_int && is_magic(c) {
        buf.emit(NFA_EOL);
    } else if no_magic_c == b'.' as c_int && is_magic(c) {
        // . matches any character (except newline by default)
        buf.emit(NFA_ANY);
    } else if no_magic_c == b'(' as c_int && is_magic(c) {
        // Check what kind of group
        let next = peekchr();
        if next == magic(b'?' as c_int) {
            // \%(...\) - non-capturing group
            skipchr();
            if nfa_reg(buf, REG_NPAREN) == FAIL {
                return FAIL;
            }
            buf.emit(NFA_NOPEN);
        } else {
            // \(...\) - capturing group
            if nfa_reg(buf, REG_PAREN) == FAIL {
                return FAIL;
            }
        }
    } else if no_magic_c == b')' as c_int && is_magic(c) {
        // Unmatched )
        let reg_magic = nvim_parse_get_reg_magic();
        nvim_regexp_report_error(E_UNMATCHED_CLOSE_PAREN, (reg_magic == MAGIC_ALL) as c_int);
        return FAIL;
    } else if (no_magic_c == b'|' as c_int || no_magic_c == b'&' as c_int) && is_magic(c) {
        // Alternation/concatenation operators handled at higher levels
        ungetchr();
        return FAIL;
    } else if (no_magic_c == b'*' as c_int
        || no_magic_c == b'+' as c_int
        || no_magic_c == b'?' as c_int
        || no_magic_c == b'{' as c_int
        || no_magic_c == b'@' as c_int)
        && is_magic(c)
    {
        // Multi on empty - error
        let reg_magic = nvim_parse_get_reg_magic();
        nvim_regexp_report_error(E_NESTED_STAR, (reg_magic == MAGIC_ALL) as c_int);
        return FAIL;
    } else if no_magic_c == b'\\' as c_int {
        // Character class starting with \
        return nfa_regatom_backslash(buf, old_regparse);
    } else if no_magic_c == b'[' as c_int && is_magic(c) {
        // Bracket expression
        return nfa_regatom_collection(buf);
    } else if no_magic_c == b'~' as c_int && is_magic(c) {
        // Tilde - previous substitute string
        // TODO: Handle ~ for previous substitute
        buf.emit(b'~' as c_int);
    } else {
        // Default: literal character
        return nfa_emit_literal(buf, c, old_regparse);
    }

    OK
}

/// Handle backslash escapes in nfa_regatom.
///
/// # Safety
/// Parse state must be valid.
unsafe fn nfa_regatom_backslash(buf: &mut PostfixBuffer, _old_regparse: *mut c_char) -> c_int {
    let c = no_magic(peekchr());

    match c as u8 {
        // Word boundaries
        b'<' => {
            skipchr();
            buf.emit(NFA_BOW);
        }
        b'>' => {
            skipchr();
            buf.emit(NFA_EOW);
        }

        // Zero-width assertions
        b'z' => {
            skipchr();
            let next = no_magic(getchr());
            match next as u8 {
                b's' => buf.emit(NFA_ZSTART),
                b'e' => {
                    buf.emit(NFA_ZEND);
                    nvim_rex_set_nfa_has_zend(1);
                }
                b'(' => {
                    // \z(...\) - external subexpression
                    if nfa_reg(buf, REG_ZPAREN) == FAIL {
                        return FAIL;
                    }
                }
                b'1'..=b'9' => {
                    // \z1-\z9 - external backreference
                    let n = (next as u8 - b'0') as c_int;
                    buf.emit(NFA_ZREF1 + n - 1);
                    nvim_rex_set_nfa_has_backref(1);
                }
                _ => {
                    return FAIL;
                }
            }
        }

        // Backreferences
        b'1'..=b'9' => {
            skipchr();
            let n = (c as u8 - b'0') as c_int;
            // Check if the group has been seen
            if nvim_parse_get_had_endbrace(n) == 0 {
                // Group not closed yet - error
                return FAIL;
            }
            buf.emit(NFA_BACKREF1 + n - 1);
            nvim_rex_set_nfa_has_backref(1);
        }

        // Character classes
        b'd' => {
            skipchr();
            buf.emit(NFA_CLASS_DIGIT);
        }
        b'D' => {
            skipchr();
            buf.emit(NFA_CLASS_DIGIT);
            buf.emit(NFA_SKIP);
        }
        b'w' => {
            skipchr();
            buf.emit(NFA_CLASS_ALNUM);
        }
        b'W' => {
            skipchr();
            buf.emit(NFA_CLASS_ALNUM);
            buf.emit(NFA_SKIP);
        }
        b's' => {
            skipchr();
            buf.emit(NFA_CLASS_SPACE);
        }
        b'S' => {
            skipchr();
            buf.emit(NFA_CLASS_SPACE);
            buf.emit(NFA_SKIP);
        }
        b'h' => {
            skipchr();
            buf.emit(NFA_CLASS_IDENT);
        }
        b'H' => {
            skipchr();
            buf.emit(NFA_CLASS_IDENT);
            buf.emit(NFA_SKIP);
        }
        b'a' => {
            skipchr();
            buf.emit(NFA_CLASS_ALPHA);
        }
        b'A' => {
            skipchr();
            buf.emit(NFA_CLASS_ALPHA);
            buf.emit(NFA_SKIP);
        }
        b'l' => {
            skipchr();
            buf.emit(NFA_CLASS_LOWER);
            nvim_parse_set_wants_nfa(1);
        }
        b'L' => {
            skipchr();
            buf.emit(NFA_CLASS_LOWER);
            buf.emit(NFA_SKIP);
            nvim_parse_set_wants_nfa(1);
        }
        b'u' => {
            skipchr();
            buf.emit(NFA_CLASS_UPPER);
            nvim_parse_set_wants_nfa(1);
        }
        b'U' => {
            skipchr();
            buf.emit(NFA_CLASS_UPPER);
            buf.emit(NFA_SKIP);
            nvim_parse_set_wants_nfa(1);
        }
        b'x' => {
            skipchr();
            buf.emit(NFA_CLASS_XDIGIT);
        }
        b'X' => {
            skipchr();
            buf.emit(NFA_CLASS_XDIGIT);
            buf.emit(NFA_SKIP);
        }
        b'o' => {
            skipchr();
            // Octal digit class [0-7]
            buf.emit(NFA_CLASS_DIGIT); // Approximate
        }
        b'O' => {
            skipchr();
            buf.emit(NFA_CLASS_DIGIT);
            buf.emit(NFA_SKIP);
        }

        // Special newline handling
        b'n' => {
            skipchr();
            let reg_string = nvim_parse_get_reg_string();
            if reg_string != 0 {
                buf.emit(NL);
            } else {
                buf.emit(NFA_NEWL);
            }
        }

        // Position matching: \%
        b'%' => {
            return nfa_regatom_percent(buf);
        }

        // Cursor position
        b'#' => {
            skipchr();
            buf.emit(NFA_CURSOR);
        }

        // Visual area
        b'V' => {
            skipchr();
            buf.emit(NFA_VISUAL);
        }

        // Underscore combinations
        b'_' => {
            skipchr();
            let next = no_magic(getchr());
            match next as u8 {
                b'^' => buf.emit(NFA_BOL),
                b'$' => buf.emit(NFA_EOL),
                b'.' => {
                    // \_. matches any character including newline
                    buf.emit(NFA_ANY);
                    buf.emit(NFA_NEWL);
                    buf.emit(NFA_OR);
                }
                b's' => {
                    buf.emit(NFA_CLASS_SPACE);
                    buf.emit(NFA_NEWL);
                    buf.emit(NFA_OR);
                }
                _ => {
                    // \_x where x is a character class
                    ungetchr();
                    return FAIL;
                }
            }
        }

        // Any other escaped character - treat as literal
        _ => {
            skipchr();
            let literal = backslash_trans(c);
            buf.emit(literal);
        }
    }

    OK
}

/// Handle \% position matching.
///
/// # Safety
/// Parse state must be valid.
unsafe fn nfa_regatom_percent(buf: &mut PostfixBuffer) -> c_int {
    skipchr(); // skip the %

    let mut c = getchr();
    let _first_c = c;

    // Check for comparison operator
    let mut cmp_op = 0;
    if no_magic(c) == b'>' as c_int {
        cmp_op = 1;
        c = getchr();
    } else if no_magic(c) == b'<' as c_int {
        cmp_op = 2;
        c = getchr();
    }

    match no_magic(c) as u8 {
        b'(' => {
            // \%(...\) - non-capturing group
            if nfa_reg(buf, REG_NPAREN) == FAIL {
                return FAIL;
            }
            buf.emit(NFA_NOPEN);
        }
        b'^' => {
            // \%^ - beginning of file
            buf.emit(NFA_BOF);
        }
        b'$' => {
            // \%$ - end of file
            buf.emit(NFA_EOF);
        }
        b'#' => {
            // \%# - cursor position
            buf.emit(NFA_CURSOR);
        }
        b'V' => {
            // \%V - inside visual area
            buf.emit(NFA_VISUAL);
        }
        b'\'' => {
            // \%'m - mark position
            let mark = getchr();
            if mark == 0 {
                return FAIL;
            }
            let opcode = match cmp_op {
                1 => NFA_MARK_GT,
                2 => NFA_MARK_LT,
                _ => NFA_MARK,
            };
            buf.emit(opcode);
            buf.emit(mark);
        }
        b'0'..=b'9' => {
            // \%23l, \%5c, \%7v - line/column/vcol
            ungetchr();
            let n = rs_getdecchrs();
            if n < 0 {
                return FAIL;
            }
            c = getchr();
            match no_magic(c) as u8 {
                b'l' => {
                    let opcode = match cmp_op {
                        1 => NFA_LNUM_GT,
                        2 => NFA_LNUM_LT,
                        _ => NFA_LNUM,
                    };
                    buf.emit(opcode);
                    buf.emit(n as c_int);
                }
                b'c' => {
                    let opcode = match cmp_op {
                        1 => NFA_COL_GT,
                        2 => NFA_COL_LT,
                        _ => NFA_COL,
                    };
                    buf.emit(opcode);
                    buf.emit(n as c_int);
                }
                b'v' => {
                    let opcode = match cmp_op {
                        1 => NFA_VCOL_GT,
                        2 => NFA_VCOL_LT,
                        _ => NFA_VCOL,
                    };
                    buf.emit(opcode);
                    buf.emit(n as c_int);
                }
                _ => {
                    // Put the character back and treat as \%N (Nth submatch)
                    ungetchr();
                    // Need to re-parse the number
                    return FAIL;
                }
            }
        }
        b'[' => {
            // \%[...] - optionally matched atoms
            // TODO: Implement optional atoms
            return FAIL;
        }
        _ => {
            // Unknown \% sequence
            ungetchr();
            return FAIL;
        }
    }

    OK
}

/// Handle bracket expression [...].
///
/// # Safety
/// Parse state must be valid.
unsafe fn nfa_regatom_collection(buf: &mut PostfixBuffer) -> c_int {
    let reg_string = nvim_parse_get_reg_string();
    let reg_cpo_lit = nvim_parse_get_reg_cpo_lit() != 0;
    let old_regparse = nvim_parse_get_regparse();

    // Check for negated collection
    let mut regparse = nvim_parse_get_regparse();
    let negated = *regparse as u8 == b'^';
    if negated {
        regparse = regparse.add(1);
        nvim_parse_set_regparse(regparse);
    }

    // Find the end of the collection
    let endp = find_collection_end(regparse);
    if *endp == 0 {
        // No closing ]
        let reg_strict = nvim_parse_get_reg_strict();
        if reg_strict != 0 {
            nvim_regexp_report_error(E_MISSINGBRACKET, 0);
            return FAIL;
        }
        // Treat [ as literal
        buf.emit(b'[' as c_int);
        nvim_parse_set_regparse(old_regparse);
        return OK;
    }

    // Emit start of collection
    if negated {
        buf.emit(NFA_START_COLL_NEG);
    } else {
        buf.emit(NFA_START_COLL);
    }

    let mut startc: c_int = -1;
    let mut emit_range = false;
    let mut extra = 0;

    // Parse collection contents
    regparse = nvim_parse_get_regparse();
    while regparse < endp {
        let oldstartc = startc;
        startc = -1;

        // Check for character class [:class:]
        let mut pp = regparse;
        let char_class = nvim_get_char_class(&mut pp);

        // Check for equivalence class [=c=]
        let equiclass = if char_class == CLASS_NONE {
            let mut pp2 = regparse;
            rs_get_equi_class(&mut pp2)
        } else {
            0
        };

        // Check for collating element [.c.]
        let collclass = if char_class == CLASS_NONE && equiclass == 0 {
            let mut pp2 = regparse;
            rs_get_coll_element(&mut pp2)
        } else {
            0
        };

        if char_class != CLASS_NONE {
            // Emit character class
            nvim_parse_set_regparse(pp);
            let class_opcode = match char_class {
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
                _ => return FAIL,
            };
            buf.emit(class_opcode);
            buf.emit(NFA_CONCAT);
            regparse = nvim_parse_get_regparse();
            continue;
        }

        if equiclass != 0 {
            // Emit equivalence class
            emit_equivalence_class(buf, equiclass);
            regparse = nvim_parse_get_regparse();
            continue;
        }

        if collclass != 0 {
            startc = collclass;
        }

        // Try range a-b
        if *regparse as u8 == b'-' && oldstartc != -1 {
            emit_range = true;
            startc = oldstartc;
            regparse = regparse.add(1);
            nvim_parse_set_regparse(regparse);
            continue;
        }

        // Handle escaped characters in collection
        if *regparse as u8 == b'\\' && (regparse as usize) + 1 < endp as usize {
            let next = *regparse.add(1) as u8;
            if byte_in_slice(next, REGEXP_INRANGE)
                || (!reg_cpo_lit && byte_in_slice(next, REGEXP_ABBR))
            {
                regparse = regparse.add(1);
                nvim_parse_set_regparse(regparse);

                if next == b'n' {
                    startc = if reg_string != 0 || emit_range {
                        NL
                    } else {
                        NFA_NEWL
                    };
                } else if matches!(next, b'd' | b'o' | b'x' | b'u' | b'U') {
                    startc = coll_get_char();
                    if startc == i32::MAX {
                        return FAIL;
                    }
                } else {
                    startc = backslash_trans(next as c_int);
                }
            }
        }

        // Normal character
        if startc == -1 {
            startc = utf_ptr2char(regparse);
        }

        // Handle range end
        if emit_range {
            let endc = startc;
            startc = oldstartc;
            if startc > endc {
                nvim_regexp_report_error(E_REVERSE_RANGE, 0);
                return FAIL;
            }

            if endc > startc + 2 {
                // Emit as range
                if startc == 0 {
                    buf.emit(1);
                } else {
                    buf.remove_last_if(NFA_CONCAT);
                }
                buf.emit(endc);
                buf.emit(NFA_RANGE);
                buf.emit(NFA_CONCAT);
            } else {
                // Emit individual characters
                for ch in (startc + 1)..=endc {
                    buf.emit(ch);
                    buf.emit(NFA_CONCAT);
                }
            }
            emit_range = false;
            startc = -1;
        } else {
            // Single character
            if startc == NFA_NEWL {
                if !negated {
                    extra = 1; // NFA_ADD_NL
                }
            } else {
                buf.emit(startc);
                buf.emit(NFA_CONCAT);
            }
        }

        // Advance to next character
        let char_len = utfc_ptr2len(regparse);
        regparse = regparse.add(char_len.max(1) as usize);
        nvim_parse_set_regparse(regparse);
    }

    // Handle trailing -
    let final_regparse = nvim_parse_get_regparse();
    if final_regparse > old_regparse {
        let prev = final_regparse.sub(1);
        if *prev as u8 == b'-' {
            buf.emit(b'-' as c_int);
            buf.emit(NFA_CONCAT);
        }
    }

    // Skip the ]
    nvim_parse_set_regparse(endp.add(1) as *mut c_char);

    // Mark end of collection
    if negated {
        buf.emit(NFA_END_NEG_COLL);
    } else {
        buf.emit(NFA_END_COLL);
    }

    // Handle \_[] which also matches newline
    if extra != 0 {
        let reg_string = nvim_parse_get_reg_string();
        buf.emit(if reg_string != 0 { NL } else { NFA_NEWL });
        buf.emit(NFA_OR);
    }

    OK
}

/// Find the end of a bracket expression.
///
/// # Safety
/// p must point to a valid null-terminated string.
unsafe fn find_collection_end(mut p: *mut c_char) -> *mut c_char {
    // ] or - at start are literal
    if *p as u8 == b']' || *p as u8 == b'-' {
        p = p.add(1);
    }

    while *p != 0 && *p as u8 != b']' {
        if *p as u8 == b'\\' && *p.add(1) != 0 {
            p = p.add(2);
        } else {
            let len = utfc_ptr2len(p);
            p = p.add(len.max(1) as usize);
        }
    }

    p
}

/// Emit equivalence class characters.
fn emit_equivalence_class(buf: &mut PostfixBuffer, _c: c_int) {
    // TODO: Implement full equivalence class expansion
    buf.emit(_c);
    buf.emit(NFA_CONCAT);
}

/// Emit a literal character, handling composing characters.
///
/// # Safety
/// Parse state must be valid.
unsafe fn nfa_emit_literal(
    buf: &mut PostfixBuffer,
    c: c_int,
    old_regparse: *const c_char,
) -> c_int {
    let plen = utfc_ptr2len(old_regparse as *const c_char);
    let base_len = utf_char2len(no_magic(c));

    if base_len != plen || rs_utf_iscomposing_legacy(no_magic(c)) != 0 {
        // Character with composing characters
        let mut i = 0;
        let mut chr = no_magic(c);

        loop {
            buf.emit(chr);
            if i > 0 {
                buf.emit(NFA_CONCAT);
            }
            i += utf_char2len(chr);
            if i >= plen {
                break;
            }
            chr = utf_ptr2char((old_regparse as *const c_char).add(i as usize));
        }
        buf.emit(NFA_COMPOSING);

        let new_regparse = (old_regparse as *mut c_char).add(plen as usize);
        nvim_parse_set_regparse(new_regparse);
    } else {
        buf.emit(no_magic(c));
    }

    OK
}

// Start collection marker (internal)
const NFA_START_COLL: c_int = -10000;
const NFA_START_COLL_NEG: c_int = -10001;

/// Parse something followed by possible [*+=].
///
/// A piece is an atom, possibly followed by a multi operator.
///
/// # Safety
/// Parse state must be valid.
pub unsafe fn nfa_regpiece(buf: &mut PostfixBuffer) -> c_int {
    // Save state for potential re-parsing with quantifiers
    let old_state = ParseState::save();
    let my_post_start = buf.pos();

    // Parse the atom
    if nfa_regatom(buf) == FAIL {
        return FAIL;
    }

    // Check for quantifier
    let op = peekchr();
    if re_multi_type_impl(op) == NOT_MULTI {
        return OK;
    }

    skipchr();

    match no_magic(op) as u8 {
        b'*' => {
            buf.emit(NFA_STAR);
        }

        b'+' => {
            // a+ = aa* - reparse the atom
            old_state.restore();
            nvim_parse_set_curchr(-1);
            if nfa_regatom(buf) == FAIL {
                return FAIL;
            }
            buf.emit(NFA_STAR);
            buf.emit(NFA_CONCAT);
            skipchr(); // skip the +
        }

        b'@' => {
            let c2 = rs_getdecchrs();
            let next = no_magic(getchr());

            let i = match next as u8 {
                b'=' => NFA_PREV_ATOM_NO_WIDTH,
                b'!' => NFA_PREV_ATOM_NO_WIDTH_NEG,
                b'<' => {
                    let op2 = no_magic(getchr());
                    if op2 as u8 == b'=' {
                        NFA_PREV_ATOM_JUST_BEFORE
                    } else if op2 as u8 == b'!' {
                        NFA_PREV_ATOM_JUST_BEFORE_NEG
                    } else {
                        0
                    }
                }
                b'>' => NFA_PREV_ATOM_LIKE_PATTERN,
                _ => 0,
            };

            if i == 0 {
                nvim_regexp_report_error(E_UNKNOWN_AT, next);
                return FAIL;
            }

            buf.emit(i);
            if i == NFA_PREV_ATOM_JUST_BEFORE || i == NFA_PREV_ATOM_JUST_BEFORE_NEG {
                buf.emit(c2 as c_int);
            }
        }

        b'?' | b'=' => {
            buf.emit(NFA_QUEST);
        }

        b'{' => {
            return nfa_regpiece_brace(buf, &old_state, my_post_start);
        }

        _ => {}
    }

    // Check for nested multi
    if re_multi_type_impl(peekchr()) != NOT_MULTI {
        let reg_magic = nvim_parse_get_reg_magic();
        nvim_regexp_report_error(E_NESTED_STAR, (reg_magic == MAGIC_ALL) as c_int);
        return FAIL;
    }

    OK
}

/// Handle \{n,m} quantifier.
///
/// # Safety
/// Parse state must be valid.
unsafe fn nfa_regpiece_brace(
    buf: &mut PostfixBuffer,
    old_state: &ParseState,
    my_post_start: usize,
) -> c_int {
    let mut greedy = true;

    // Check for non-greedy
    let c2 = peekchr();
    if c2 == b'-' as c_int || c2 == magic(b'-' as c_int) {
        skipchr();
        greedy = false;
    }

    // Read the limits
    let mut minval: c_int = 0;
    let mut maxval: c_int = 0;
    if read_limits(&mut minval, &mut maxval) == 0 {
        let reg_magic = nvim_parse_get_reg_magic();
        nvim_regexp_report_error(E_READ_LIMITS, (reg_magic == MAGIC_ALL) as c_int);
        return FAIL;
    }

    // Handle special cases
    if minval == 0 && maxval == MAX_LIMIT {
        // {0,} or {} = *
        buf.emit(if greedy { NFA_STAR } else { NFA_STAR_NONGREEDY });
        return OK;
    }

    if maxval == 0 {
        // {0} = empty match
        buf.truncate(my_post_start);
        buf.emit(NFA_EMPTY);
        return OK;
    }

    // Check for too complex patterns
    let nfa_re_flags = nvim_parse_get_nfa_re_flags();
    let wants_nfa = nvim_parse_get_wants_nfa() != 0;
    if (nfa_re_flags & 1) != 0
        && (maxval > 500 || maxval > minval + 200)
        && (maxval != MAX_LIMIT && minval < 200)
        && !wants_nfa
    {
        return FAIL;
    }

    // Save state after the quantifier
    let new_state = ParseState::save();

    // Truncate and rebuild
    buf.truncate(my_post_start);

    let quest = if greedy {
        NFA_QUEST
    } else {
        NFA_QUEST_NONGREEDY
    };

    for i in 0..maxval {
        // Re-parse the atom
        old_state.restore();
        let old_post_pos = buf.pos();
        if nfa_regatom(buf) == FAIL {
            return FAIL;
        }

        // After minval, atoms are optional
        if i + 1 > minval {
            if maxval == MAX_LIMIT {
                buf.emit(if greedy { NFA_STAR } else { NFA_STAR_NONGREEDY });
            } else {
                buf.emit(quest);
            }
        }

        if old_post_pos != my_post_start {
            buf.emit(NFA_CONCAT);
        }

        if i + 1 > minval && maxval == MAX_LIMIT {
            break;
        }
    }

    // Restore to after the quantifier
    new_state.restore();
    nvim_parse_set_curchr(-1);

    OK
}

/// Parse one or more pieces, concatenated.
///
/// # Safety
/// Parse state must be valid.
pub unsafe fn nfa_regconcat(buf: &mut PostfixBuffer) -> c_int {
    let mut cont = true;
    let mut first = true;

    while cont {
        let c = peekchr();

        match c {
            0 => cont = false,
            _ if no_magic(c) == b'|' as c_int => cont = false,
            _ if no_magic(c) == b'&' as c_int => cont = false,
            _ if no_magic(c) == b')' as c_int => cont = false,

            // Magic mode switches
            _ if no_magic(c) == b'Z' as c_int && is_magic(c) => {
                let regflags = nvim_parse_get_regflags();
                nvim_parse_set_regflags(regflags | RF_ICOMBINE as c_int);
                skipchr_keepstart();
            }
            _ if no_magic(c) == b'c' as c_int && is_magic(c) => {
                let regflags = nvim_parse_get_regflags();
                nvim_parse_set_regflags(regflags | RF_ICASE as c_int);
                skipchr_keepstart();
            }
            _ if no_magic(c) == b'C' as c_int && is_magic(c) => {
                let regflags = nvim_parse_get_regflags();
                nvim_parse_set_regflags(regflags | RF_NOICASE as c_int);
                skipchr_keepstart();
            }
            _ if no_magic(c) == b'v' as c_int && is_magic(c) => {
                nvim_parse_set_reg_magic(MAGIC_ALL);
                skipchr_keepstart();
                nvim_parse_set_curchr(-1);
            }
            _ if no_magic(c) == b'm' as c_int && is_magic(c) => {
                nvim_parse_set_reg_magic(MAGIC_ON);
                skipchr_keepstart();
                nvim_parse_set_curchr(-1);
            }
            _ if no_magic(c) == b'M' as c_int && is_magic(c) => {
                nvim_parse_set_reg_magic(MAGIC_OFF);
                skipchr_keepstart();
                nvim_parse_set_curchr(-1);
            }
            _ if no_magic(c) == b'V' as c_int && is_magic(c) => {
                nvim_parse_set_reg_magic(MAGIC_NONE);
                skipchr_keepstart();
                nvim_parse_set_curchr(-1);
            }

            _ => {
                if nfa_regpiece(buf) == FAIL {
                    return FAIL;
                }
                if !first {
                    buf.emit(NFA_CONCAT);
                } else {
                    first = false;
                }
            }
        }
    }

    OK
}

/// Parse a branch, one or more concats, separated by "\&".
///
/// # Safety
/// Parse state must be valid.
pub unsafe fn nfa_regbranch(buf: &mut PostfixBuffer) -> c_int {
    let old_post_pos = buf.pos();

    // First concat
    if nfa_regconcat(buf) == FAIL {
        return FAIL;
    }

    // Try more concats with \&
    while peekchr() == magic(b'&' as c_int) {
        skipchr();

        // If concat is empty, emit a node
        if old_post_pos == buf.pos() {
            buf.emit(NFA_EMPTY);
        }
        buf.emit(NFA_NOPEN);
        buf.emit(NFA_PREV_ATOM_NO_WIDTH);

        let new_old_pos = buf.pos();
        if nfa_regconcat(buf) == FAIL {
            return FAIL;
        }

        if new_old_pos == buf.pos() {
            buf.emit(NFA_EMPTY);
        }
        buf.emit(NFA_CONCAT);
    }

    // If branch is empty, emit a node
    if old_post_pos == buf.pos() {
        buf.emit(NFA_EMPTY);
    }

    OK
}

/// Parse a pattern, one or more branches, separated by "\|".
///
/// # Safety
/// Parse state must be valid.
pub unsafe fn nfa_reg(buf: &mut PostfixBuffer, paren: c_int) -> c_int {
    let mut parno = 0;

    if paren == REG_PAREN {
        let regnpar = nvim_parse_get_regnpar();
        if regnpar >= NSUBEXP as c_int {
            let reg_magic = nvim_parse_get_reg_magic();
            nvim_regexp_report_error(E_TOO_MANY_PAREN, (reg_magic == MAGIC_ALL) as c_int);
            return FAIL;
        }
        parno = regnpar;
        nvim_parse_set_regnpar(regnpar + 1);
    } else if paren == REG_ZPAREN {
        let regnzpar = nvim_parse_get_regnzpar();
        if regnzpar >= NSUBEXP as c_int {
            let reg_magic = nvim_parse_get_reg_magic();
            nvim_regexp_report_error(E_TOO_MANY_ZPAREN, (reg_magic == MAGIC_ALL) as c_int);
            return FAIL;
        }
        parno = regnzpar;
        nvim_parse_set_regnzpar(regnzpar + 1);
    }

    // First branch
    if nfa_regbranch(buf) == FAIL {
        return FAIL;
    }

    // More branches with \|
    while peekchr() == magic(b'|' as c_int) {
        skipchr();
        if nfa_regbranch(buf) == FAIL {
            return FAIL;
        }
        buf.emit(NFA_OR);
    }

    // Check for proper termination
    if paren != REG_NOPAREN && getchr() != magic(b')' as c_int) {
        let reg_magic = nvim_parse_get_reg_magic();
        nvim_regexp_report_error(E_UNMATCHED_PAREN, (reg_magic == MAGIC_ALL) as c_int);
        return FAIL;
    } else if paren == REG_NOPAREN && peekchr() != 0 {
        if peekchr() == magic(b')' as c_int) {
            let reg_magic = nvim_parse_get_reg_magic();
            nvim_regexp_report_error(E_UNMATCHED_CLOSE_PAREN, (reg_magic == MAGIC_ALL) as c_int);
            return FAIL;
        }
        // Other termination error
        return FAIL;
    }

    // Emit group markers
    if paren == REG_PAREN {
        nvim_parse_set_had_endbrace(parno, 1);
        buf.emit(NFA_MOPEN + parno);
    } else if paren == REG_ZPAREN {
        buf.emit(NFA_ZOPEN + parno);
    }

    OK
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Parse a regex pattern into postfix form.
///
/// Returns a pointer to the postfix buffer (caller must free), or null on error.
///
/// # Safety
/// `pattern` must be a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_parse_pattern(
    pattern: *mut c_char,
    out_len: *mut c_int,
) -> *mut c_int {
    use crate::scanner::initchr;

    // Initialize scanner
    initchr(pattern);

    // Estimate capacity
    let len = libc::strlen(pattern);
    let cap = (len + 1) * 25 + 1000;

    let mut buf = PostfixBuffer::with_capacity(cap);

    // Parse the pattern
    if nfa_reg(&mut buf, REG_NOPAREN) == FAIL {
        return std::ptr::null_mut();
    }

    // Return the data
    let data = buf.data;
    *out_len = data.len() as c_int;

    // Leak the vec and return pointer
    let ptr = data.as_ptr() as *mut c_int;
    std::mem::forget(data);
    ptr
}

/// Free postfix buffer returned by rs_nfa_parse_pattern.
///
/// # Safety
/// `ptr` must be a pointer returned by rs_nfa_parse_pattern.
#[no_mangle]
pub unsafe extern "C" fn rs_nfa_free_postfix(ptr: *mut c_int, len: c_int) {
    if !ptr.is_null() {
        let _ = Vec::from_raw_parts(ptr, len as usize, len as usize);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postfix_buffer() {
        let mut buf = PostfixBuffer::with_capacity(100);
        assert_eq!(buf.pos(), 0);

        buf.emit(1);
        buf.emit(2);
        buf.emit(3);
        assert_eq!(buf.pos(), 3);
        assert_eq!(buf.data(), &[1, 2, 3]);

        buf.truncate(1);
        assert_eq!(buf.pos(), 1);
        assert_eq!(buf.data(), &[1]);
    }

    #[test]
    fn test_postfix_buffer_remove_last() {
        let mut buf = PostfixBuffer::with_capacity(10);
        buf.emit(1);
        buf.emit(2);
        buf.emit(NFA_CONCAT);

        assert!(buf.remove_last_if(NFA_CONCAT));
        assert_eq!(buf.data(), &[1, 2]);

        assert!(!buf.remove_last_if(NFA_CONCAT));
        assert_eq!(buf.data(), &[1, 2]);
    }

    #[test]
    fn test_magic_functions() {
        assert!(is_magic(magic(b'*' as c_int)));
        assert!(!is_magic(b'*' as c_int));
        assert_eq!(no_magic(magic(b'*' as c_int)), b'*' as c_int);
        assert_eq!(no_magic(b'*' as c_int), b'*' as c_int);
    }

    #[test]
    fn test_parse_state_size() {
        // Ensure ParseState is not too large
        assert!(std::mem::size_of::<ParseState>() < 64);
    }
}
