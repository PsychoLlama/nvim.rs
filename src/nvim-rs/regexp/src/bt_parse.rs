//! Backtracking (BT) regex engine pattern parsing.
//!
//! This module implements the pattern parsing phase for the BT regex engine.
//! It converts regex pattern strings into a stream of parsed tokens/operations.
//!
//! # Pattern Grammar
//!
//! The Vim regex grammar follows this hierarchy:
//! - reg() - parses alternatives (|)
//! - regbranch() - parses concatenation (&)
//! - regconcat() - parses sequence of atoms
//! - regpiece() - parses atom with quantifiers
//! - regatom() - parses single atoms
//!
//! # Magic Mode
//!
//! Vim has different "magic" levels that affect which characters are special:
//! - `nomagic` - almost all special chars need backslash
//! - `magic` (default) - many chars are special without backslash
//! - `very magic` - most chars are special
//! - `very nomagic` - almost nothing is special

use std::ffi::c_int;
use std::ptr;

use crate::bt_compile::{RegCompiler, NODE_SIZE};
use crate::bt_opcodes::*;
use crate::parser::MAX_LIMIT;

// =============================================================================
// Magic Mode Constants
// =============================================================================

/// No magic mode - backslash needed for specials.
pub const MAGIC_OFF: c_int = 0;

/// Normal magic mode (default).
pub const MAGIC_ON: c_int = 1;

/// Very magic mode - most chars are special.
pub const MAGIC_ALL: c_int = 2;

/// Very nomagic mode - almost nothing special.
pub const MAGIC_NONE: c_int = 3;

// =============================================================================
// Parser State
// =============================================================================

/// Pattern parser state.
///
/// Tracks current position in pattern and handles magic mode parsing.
#[allow(dead_code)]
pub struct PatternParser<'a> {
    /// Pattern being parsed
    pattern: &'a [u8],
    /// Current position in pattern
    pos: usize,
    /// Current character (may be magic-encoded)
    curchr: c_int,
    /// Previous character
    prevchr: c_int,
    /// Byte length of previous character
    prevchr_len: usize,
    /// Next character (lookahead)
    nextchr: c_int,
    /// Magic mode
    reg_magic: c_int,
    /// Strict mode for regex matching
    reg_strict: bool,
    /// C-compatibility option
    reg_cpo_lit: bool,
    /// C-compatibility option for backslash
    reg_cpo_bsl: bool,
    /// Number of parentheses seen
    regnpar: c_int,
    /// Number of \z() parentheses seen
    regnzpar: c_int,
    /// Complex brace count
    num_complex_braces: c_int,
    /// Has \z special
    re_has_z: bool,
    /// Parser encountered error
    has_error: bool,
    /// Error message
    error_msg: Option<&'static str>,
}

impl<'a> PatternParser<'a> {
    /// Create a new parser for the given pattern.
    pub fn new(pattern: &'a [u8], re_flags: c_int) -> Self {
        // Determine magic mode from flags
        let reg_magic = if re_flags & 1 != 0 {
            MAGIC_ON
        } else {
            MAGIC_OFF
        };

        let mut parser = Self {
            pattern,
            pos: 0,
            curchr: -1,
            prevchr: -1,
            prevchr_len: 0,
            nextchr: -1,
            reg_magic,
            reg_strict: false,
            reg_cpo_lit: false,
            reg_cpo_bsl: false,
            regnpar: 1, // 0 is for whole match
            regnzpar: 0,
            num_complex_braces: 0,
            re_has_z: false,
            has_error: false,
            error_msg: None,
        };

        // Initialize lookahead
        parser.next_char();
        parser
    }

    /// Check if we're at end of pattern.
    #[inline]
    pub fn at_end(&self) -> bool {
        self.pos >= self.pattern.len()
    }

    /// Get current byte at position.
    #[inline]
    pub fn current_byte(&self) -> u8 {
        if self.pos < self.pattern.len() {
            self.pattern[self.pos]
        } else {
            0
        }
    }

    /// Advance to next character, handling multi-byte.
    pub fn next_char(&mut self) {
        self.prevchr = self.curchr;
        self.prevchr_len = self.char_len_at(self.pos);

        if self.pos >= self.pattern.len() {
            self.curchr = 0; // NUL at end
            return;
        }

        let c = self.pattern[self.pos] as c_int;
        self.pos += self.char_len_at(self.pos);

        // Handle escape sequences
        if c == b'\\' as c_int && self.pos < self.pattern.len() {
            let next = self.pattern[self.pos] as c_int;

            // Check for magic mode switches
            match next {
                // \v = very magic
                0x76 => {
                    // 'v'
                    self.reg_magic = MAGIC_ALL;
                    self.pos += 1;
                    return self.next_char();
                }
                // \V = very nomagic
                0x56 => {
                    // 'V'
                    self.reg_magic = MAGIC_NONE;
                    self.pos += 1;
                    return self.next_char();
                }
                // \m = magic
                0x6D => {
                    // 'm'
                    self.reg_magic = MAGIC_ON;
                    self.pos += 1;
                    return self.next_char();
                }
                // \M = nomagic
                0x4D => {
                    // 'M'
                    self.reg_magic = MAGIC_OFF;
                    self.pos += 1;
                    return self.next_char();
                }
                _ => {}
            }

            // Handle escaped characters based on magic mode
            self.curchr = self.parse_escaped(next);
        } else {
            // Regular character - apply magic
            self.curchr = self.apply_magic(c);
        }
    }

    /// Parse an escaped character.
    fn parse_escaped(&mut self, next: c_int) -> c_int {
        self.pos += 1; // consume the escaped char

        match next as u8 {
            // Character classes
            b'd' => magic(b'd' as c_int), // \d - digit
            b'D' => magic(b'D' as c_int), // \D - non-digit
            b'w' => magic(b'w' as c_int), // \w - word
            b'W' => magic(b'W' as c_int), // \W - non-word
            b's' => magic(b's' as c_int), // \s - whitespace
            b'S' => magic(b'S' as c_int), // \S - non-whitespace
            b'x' => magic(b'x' as c_int), // \x - hex
            b'X' => magic(b'X' as c_int), // \X - non-hex
            b'o' => magic(b'o' as c_int), // \o - octal
            b'O' => magic(b'O' as c_int), // \O - non-octal
            b'h' => magic(b'h' as c_int), // \h - head
            b'H' => magic(b'H' as c_int), // \H - non-head
            b'a' => magic(b'a' as c_int), // \a - alpha
            b'A' => magic(b'A' as c_int), // \A - non-alpha
            b'l' => magic(b'l' as c_int), // \l - lowercase
            b'L' => magic(b'L' as c_int), // \L - non-lowercase
            b'u' => magic(b'u' as c_int), // \u - uppercase
            b'U' => magic(b'U' as c_int), // \U - non-uppercase
            b'i' => magic(b'i' as c_int), // \i - identifier
            b'I' => magic(b'I' as c_int), // \I - non-identifier
            b'k' => magic(b'k' as c_int), // \k - keyword
            b'K' => magic(b'K' as c_int), // \K - non-keyword
            b'f' => magic(b'f' as c_int), // \f - filename
            b'F' => magic(b'F' as c_int), // \F - non-filename
            b'p' => magic(b'p' as c_int), // \p - printable
            b'P' => magic(b'P' as c_int), // \P - non-printable

            // Anchors and assertions
            b'^' => magic(b'^' as c_int), // \^ - BOL
            b'$' => magic(b'$' as c_int), // \$ - EOL
            b'<' => magic(b'<' as c_int), // \< - BOW
            b'>' => magic(b'>' as c_int), // \> - EOW
            b'n' => magic(b'n' as c_int), // \n - newline

            // Grouping
            b'(' => magic(b'(' as c_int), // \( - group start
            b')' => magic(b')' as c_int), // \) - group end
            b'|' => magic(b'|' as c_int), // \| - alternation
            b'&' => magic(b'&' as c_int), // \& - concat (match)

            // Quantifiers (in magic mode, these need backslash)
            b'*' => {
                if self.reg_magic == MAGIC_OFF || self.reg_magic == MAGIC_NONE {
                    magic(b'*' as c_int)
                } else {
                    b'*' as c_int // literal
                }
            }
            b'+' => magic(b'+' as c_int),
            b'?' => magic(b'?' as c_int),
            b'{' => magic(b'{' as c_int),
            b'=' => magic(b'=' as c_int), // \= - optional (same as \?)

            // Special escapes
            b'.' => {
                if self.reg_magic == MAGIC_OFF || self.reg_magic == MAGIC_NONE {
                    magic(b'.' as c_int)
                } else {
                    b'.' as c_int // literal
                }
            }
            b'[' => {
                if self.reg_magic == MAGIC_OFF || self.reg_magic == MAGIC_NONE {
                    magic(b'[' as c_int)
                } else {
                    b'[' as c_int // literal
                }
            }
            b'~' => magic(b'~' as c_int), // \~ - previous substitute
            b'_' => magic(b'_' as c_int), // \_ prefix
            b'%' => magic(b'%' as c_int), // \% prefix
            b'z' => magic(b'z' as c_int), // \z prefix
            b'@' => magic(b'@' as c_int), // \@ for assertions

            // Backreferences \1-\9
            b'1'..=b'9' => magic(next),

            // Literal backslash
            b'\\' => b'\\' as c_int,

            // Escape sequences for special chars
            b'e' => 0x1B, // ESC
            b't' => 0x09, // TAB
            b'r' => 0x0D, // CR
            b'b' => 0x08, // BS

            // Default: literal character
            _ => next,
        }
    }

    /// Apply magic mode to a character.
    fn apply_magic(&self, c: c_int) -> c_int {
        match self.reg_magic {
            MAGIC_ALL => {
                // Very magic: most chars are special
                match c as u8 {
                    b'^' | b'$' | b'.' | b'*' | b'+' | b'?' | b'{' | b'(' | b')' | b'|' | b'['
                    | b'\\' | b'~' => magic(c),
                    _ => c,
                }
            }
            MAGIC_ON => {
                // Normal magic: some chars are special
                match c as u8 {
                    b'^' | b'$' | b'.' | b'*' | b'[' | b'~' | b'\\' => magic(c),
                    _ => c,
                }
            }
            MAGIC_OFF | MAGIC_NONE => {
                // No magic: only backslash is special
                if c == b'\\' as c_int {
                    magic(c)
                } else {
                    c
                }
            }
            _ => c,
        }
    }

    /// Get length of character at position (handles multi-byte UTF-8).
    fn char_len_at(&self, pos: usize) -> usize {
        if pos >= self.pattern.len() {
            return 0;
        }

        let b = self.pattern[pos];
        // UTF-8 byte length from first byte
        // 0x00-0x7F: ASCII (1 byte)
        // 0x80-0xBF: continuation byte (treat as 1, shouldn't happen at start)
        // 0xC0-0xDF: 2-byte sequence
        // 0xE0-0xEF: 3-byte sequence
        // 0xF0-0xFF: 4-byte sequence
        if b < 0xC0 {
            1
        } else if b < 0xE0 {
            2
        } else if b < 0xF0 {
            3
        } else {
            4
        }
    }

    /// Peek at current character without consuming.
    #[inline]
    pub fn peekchr(&self) -> c_int {
        self.curchr
    }

    /// Get and consume current character.
    pub fn getchr(&mut self) -> c_int {
        let c = self.curchr;
        self.next_char();
        c
    }

    /// Skip current character.
    pub fn skipchr(&mut self) {
        self.next_char();
    }

    /// Peek at next character (two-char lookahead).
    pub fn peek_next(&mut self) -> c_int {
        // Save state
        let saved_pos = self.pos;
        let saved_chr = self.curchr;

        // Get next
        self.next_char();
        let result = self.curchr;

        // Restore state
        self.pos = saved_pos;
        self.curchr = saved_chr;

        result
    }

    /// Check if current char matches (considering magic).
    pub fn is_char(&self, c: u8) -> bool {
        self.curchr == magic(c as c_int)
    }

    /// Get subexpression number (for \1-\9).
    pub fn get_backref_num(&self) -> Option<c_int> {
        if is_magic(self.curchr) {
            let c = un_magic(self.curchr);
            if c >= b'1' as c_int && c <= b'9' as c_int {
                return Some(c - b'0' as c_int);
            }
        }
        None
    }

    /// Allocate next paren number.
    pub fn next_paren(&mut self) -> c_int {
        let n = self.regnpar;
        self.regnpar += 1;
        n
    }

    /// Allocate next \z() paren number.
    pub fn next_zparen(&mut self) -> c_int {
        let n = self.regnzpar;
        self.regnzpar += 1;
        n
    }

    /// Set error state.
    pub fn set_error(&mut self, msg: &'static str) {
        if !self.has_error {
            self.has_error = true;
            self.error_msg = Some(msg);
        }
    }

    /// Check if parser has error.
    pub fn has_error(&self) -> bool {
        self.has_error
    }

    /// Get error message.
    pub fn error_message(&self) -> Option<&'static str> {
        self.error_msg
    }

    /// Set the re_has_z flag (pattern uses \z special).
    pub fn set_has_z(&mut self, val: bool) {
        self.re_has_z = val;
    }

    /// Check if pattern uses \z special.
    #[allow(dead_code)]
    pub fn has_z(&self) -> bool {
        self.re_has_z
    }

    /// Read and parse limits for `\{n,m}` quantifier.
    ///
    /// Parses:
    /// - `{n}` - exactly n
    /// - `{n,}` - at least n
    /// - `{n,m}` - between n and m
    /// - `{,m}` - at most m
    /// - `{-n,m}` - non-greedy variant
    ///
    /// Returns (minval, maxval, reverse) tuple, or None on error.
    pub fn read_limits(&mut self) -> Option<(c_int, c_int, bool)> {
        let mut reverse = false;
        let mut minval: c_int;
        let maxval: c_int;

        // Check for '-' at start (non-greedy)
        if self.pos < self.pattern.len() && self.pattern[self.pos] == b'-' {
            self.pos += 1;
            reverse = true;
        }

        // Track whether we have seen a first digit
        let first_pos = self.pos;

        // Parse minimum value
        minval = 0;
        while self.pos < self.pattern.len() && self.pattern[self.pos].is_ascii_digit() {
            minval = minval
                .saturating_mul(10)
                .saturating_add((self.pattern[self.pos] - b'0') as c_int);
            self.pos += 1;
        }
        let had_first_digit = self.pos > first_pos;

        // Check for comma
        if self.pos < self.pattern.len() && self.pattern[self.pos] == b',' {
            self.pos += 1; // consume comma

            // Parse maximum value if present
            if self.pos < self.pattern.len() && self.pattern[self.pos].is_ascii_digit() {
                maxval = self.parse_decimal();
            } else {
                maxval = MAX_LIMIT;
            }
        } else if had_first_digit {
            // It was {n} or {-n}
            maxval = minval;
        } else {
            // It was {} or {-}
            maxval = MAX_LIMIT;
        }

        // Allow either \{...} or \{...\}
        if self.pos < self.pattern.len() && self.pattern[self.pos] == b'\\' {
            self.pos += 1;
        }

        // Must end with }
        if self.pos >= self.pattern.len() || self.pattern[self.pos] != b'}' {
            self.set_error("E554: Syntax error in \\{...}");
            return None;
        }
        self.pos += 1; // consume '}'

        // Reinitialize parser character after manually advancing pos
        self.next_char();

        Some((minval, maxval, reverse))
    }

    /// Parse a decimal number from pattern.
    fn parse_decimal(&mut self) -> c_int {
        let mut val: c_int = 0;
        while self.pos < self.pattern.len() && self.pattern[self.pos].is_ascii_digit() {
            val = val
                .saturating_mul(10)
                .saturating_add((self.pattern[self.pos] - b'0') as c_int);
            self.pos += 1;
        }
        val
    }
}

// =============================================================================
// Assertion Type Parsing
// =============================================================================

/// Parse the assertion type after `\@`.
///
/// Returns the appropriate opcode:
/// - `=` → MATCH (positive lookahead)
/// - `!` → NOMATCH (negative lookahead)
/// - `>` → SUBPAT (atomic grouping)
/// - `<=` → BEHIND (positive lookbehind)
/// - `<!` → NOBEHIND (negative lookbehind)
/// - Otherwise → END (invalid)
fn parse_assertion_type(parser: &mut PatternParser) -> c_int {
    let c = parser.peekchr();

    // Handle magic or literal character
    let ch = if is_magic(c) {
        un_magic(c) as u8
    } else if c > 0 && c < 256 {
        c as u8
    } else {
        return END;
    };

    match ch {
        b'=' => {
            parser.skipchr();
            MATCH // \@= positive lookahead
        }
        b'!' => {
            parser.skipchr();
            NOMATCH // \@! negative lookahead
        }
        b'>' => {
            parser.skipchr();
            SUBPAT // \@> atomic grouping
        }
        b'<' => {
            // Need to look at next character for \@<= or \@<!
            parser.skipchr();
            let next = parser.peekchr();
            let next_ch = if is_magic(next) {
                un_magic(next) as u8
            } else if next > 0 && next < 256 {
                next as u8
            } else {
                return END;
            };

            match next_ch {
                b'=' => {
                    parser.skipchr();
                    BEHIND // \@<= positive lookbehind
                }
                b'!' => {
                    parser.skipchr();
                    NOBEHIND // \@<! negative lookbehind
                }
                _ => END,
            }
        }
        _ => END,
    }
}

// =============================================================================
// Parsing Functions - Main Compiler Interface
// =============================================================================

/// Parse a complete regex pattern.
///
/// This is the main entry point for pattern compilation.
/// Returns the bytecode start position, or null on error.
///
/// # Safety
/// Compiler must be valid.
pub unsafe fn parse_pattern(
    compiler: *mut RegCompiler,
    pattern: &[u8],
    re_flags: c_int,
    out_flags: *mut c_int,
) -> *mut u8 {
    if compiler.is_null() || pattern.is_empty() {
        return ptr::null_mut();
    }

    let mut parser = PatternParser::new(pattern, re_flags);

    // Emit magic marker
    (*compiler).emit_byte(REGMAGIC);

    // Parse top-level expression
    let result = parse_reg(compiler, &mut parser, REG_NOPAREN, out_flags);

    if parser.has_error() || (*compiler).is_too_long() {
        return ptr::null_mut();
    }

    result
}

/// Parse a parenthesized expression or top level.
///
/// # Safety
/// Compiler and parser must be valid.
unsafe fn parse_reg(
    compiler: *mut RegCompiler,
    parser: &mut PatternParser,
    paren: c_int,
    flagp: *mut c_int,
) -> *mut u8 {
    let mut ret: *mut u8 = ptr::null_mut();
    let parno: c_int;

    *flagp = HASWIDTH; // Tentatively

    // Handle paren type
    match paren {
        REG_ZPAREN => {
            if parser.regnzpar >= 10 {
                parser.set_error("E50: Too many \\z(");
                return ptr::null_mut();
            }
            parno = parser.next_zparen();
            ret = (*compiler).emit_node(ZOPEN + parno);
        }
        REG_PAREN => {
            if parser.regnpar >= 10 {
                parser.set_error("E51: Too many \\(");
                return ptr::null_mut();
            }
            parno = parser.next_paren();
            ret = (*compiler).emit_node(MOPEN + parno);
        }
        REG_NPAREN => {
            parno = 0;
            ret = (*compiler).emit_node(NOPEN);
        }
        _ => {
            parno = 0;
        }
    }

    // Parse first branch
    let mut flags: c_int = 0;
    let br = parse_branch(compiler, parser, &mut flags);
    if br.is_null() {
        return ptr::null_mut();
    }

    if !ret.is_null() {
        // Link paren opener to first branch
        (*compiler).chain(ret, br);
    } else {
        ret = br;
    }

    // Propagate flags
    if (flags & HASWIDTH) == 0 {
        *flagp &= !HASWIDTH;
    }
    *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);

    // Parse alternations
    while parser.is_char(b'|') {
        parser.skipchr();

        let br = parse_branch(compiler, parser, &mut flags);
        if br.is_null() || (*compiler).is_too_long() {
            return ptr::null_mut();
        }

        // Link branches
        (*compiler).chain(ret, br);

        if (flags & HASWIDTH) == 0 {
            *flagp &= !HASWIDTH;
        }
        *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);
    }

    // Make closing node
    let ender = match paren {
        REG_ZPAREN => (*compiler).emit_node(ZCLOSE + parno),
        REG_PAREN => (*compiler).emit_node(MCLOSE + parno),
        REG_NPAREN => (*compiler).emit_node(NCLOSE),
        _ => (*compiler).emit_node(END),
    };

    // Link branches to closer
    (*compiler).chain(ret, ender);

    // Check for proper termination
    if paren != REG_NOPAREN && !parser.is_char(b')') {
        parser.set_error(match paren {
            REG_ZPAREN => "E52: Unmatched \\z(",
            REG_NPAREN => "E53: Unmatched \\%(",
            _ => "E54: Unmatched \\(",
        });
        return ptr::null_mut();
    }

    if paren != REG_NOPAREN {
        parser.skipchr(); // consume ')'
    } else if parser.peekchr() != 0 {
        if parser.is_char(b')') {
            parser.set_error("E55: Unmatched \\)");
        } else {
            parser.set_error("E56: Trailing characters");
        }
        return ptr::null_mut();
    }

    ret
}

/// Parse a branch (concatenation with &).
///
/// # Safety
/// Compiler must be valid.
unsafe fn parse_branch(
    compiler: *mut RegCompiler,
    parser: &mut PatternParser,
    flagp: *mut c_int,
) -> *mut u8 {
    *flagp = WORST | HASNL;

    let ret = (*compiler).emit_node(BRANCH);
    let mut chain: *mut u8 = ptr::null_mut();

    loop {
        let mut flags: c_int = 0;
        let latest = parse_concat(compiler, parser, &mut flags);
        if latest.is_null() {
            return ptr::null_mut();
        }

        *flagp |= flags & (HASWIDTH | SPSTART | HASLOOKBH);
        *flagp &= !HASNL | (flags & HASNL);

        if !chain.is_null() {
            (*compiler).chain(chain, latest);
        }

        if !parser.is_char(b'&') {
            break;
        }

        parser.skipchr();
        (*compiler).chain(latest, (*compiler).emit_node(END));

        if (*compiler).is_too_long() {
            break;
        }

        (*compiler).insert_node(MATCH, latest);
        chain = latest;
    }

    ret
}

/// Parse a concatenation of atoms.
///
/// # Safety
/// Compiler must be valid.
unsafe fn parse_concat(
    compiler: *mut RegCompiler,
    parser: &mut PatternParser,
    flagp: *mut c_int,
) -> *mut u8 {
    *flagp = WORST;

    let mut ret: *mut u8 = ptr::null_mut();
    let mut first = true;

    loop {
        // Check for end of concatenation
        let c = parser.peekchr();
        if c == 0
            || c == magic(b'|' as c_int)
            || c == magic(b'&' as c_int)
            || c == magic(b')' as c_int)
        {
            if first {
                // Empty atom - emit NOTHING
                ret = (*compiler).emit_node(NOTHING);
            }
            break;
        }

        let mut flags: c_int = 0;
        let latest = parse_piece(compiler, parser, &mut flags);
        if latest.is_null() {
            return ptr::null_mut();
        }

        *flagp |= flags & (HASWIDTH | HASNL | HASLOOKBH);
        if first {
            *flagp |= flags & SPSTART;
            first = false;
            ret = latest;
        } else {
            (*compiler).chain(ret, latest);
        }
    }

    ret
}

/// Parse an atom with optional quantifier.
///
/// # Safety
/// Compiler must be valid.
unsafe fn parse_piece(
    compiler: *mut RegCompiler,
    parser: &mut PatternParser,
    flagp: *mut c_int,
) -> *mut u8 {
    let mut flags: c_int = 0;
    let ret = parse_atom(compiler, parser, &mut flags);
    if ret.is_null() {
        return ptr::null_mut();
    }

    // Check for quantifier
    let op = parser.peekchr();
    if !is_magic(op) {
        *flagp = flags;
        return ret;
    }

    let op_char = un_magic(op) as u8;

    // Handle assertion operators (\@=, \@!, \@>, \@<=, \@<!)
    if op_char == b'@' {
        parser.skipchr(); // consume '@'
        let lop = parse_assertion_type(parser);
        if lop == END {
            parser.set_error("E869: Invalid character after \\@");
            return ptr::null_mut();
        }

        // Look behind assertions set HASLOOKBH flag
        if lop == BEHIND || lop == NOBEHIND {
            (*compiler).chain(ret, (*compiler).emit_node(BHPOS));
            *flagp |= HASLOOKBH;
        }

        // Terminate the operand with END
        (*compiler).chain(ret, (*compiler).emit_node(END));

        // Insert the assertion opcode before the operand
        (*compiler).insert_node(lop, ret);

        // Assertions are zero-width, so no HASWIDTH flag
        *flagp = WORST | (flags & (HASNL | HASLOOKBH));
        return ret;
    }

    if !matches!(op_char, b'*' | b'+' | b'?' | b'{' | b'=') {
        *flagp = flags;
        return ret;
    }

    if (flags & HASWIDTH) == 0 && op_char != b'?' && op_char != b'=' {
        parser.set_error("E57: Quantifier follows nothing");
        return ptr::null_mut();
    }

    *flagp = WORST;
    if op_char != b'+' {
        *flagp |= HASWIDTH;
    }
    *flagp |= flags & (HASNL | HASLOOKBH);

    // Emit quantifier
    match op_char {
        b'*' => {
            parser.skipchr();
            if (flags & SIMPLE) != 0 {
                (*compiler).insert_node(STAR, ret);
            } else {
                // Complex: need BRANCH structure
                (*compiler).insert_node(BRANCH, ret);
                (*compiler).chain(ret, (*compiler).emit_node(BACK));
                (*compiler).chain(ret, ret);
                (*compiler).chain(ret, (*compiler).emit_node(BRANCH));
                (*compiler).chain(ret, (*compiler).emit_node(NOTHING));
            }
        }
        b'+' => {
            parser.skipchr();
            if (flags & SIMPLE) != 0 {
                (*compiler).insert_node(PLUS, ret);
            } else {
                // Complex: need structure
                let next = (*compiler).emit_node(BRANCH);
                (*compiler).chain(ret, next);
                (*compiler).chain((*compiler).emit_node(BACK), ret);
                (*compiler).chain(next, (*compiler).emit_node(BRANCH));
                (*compiler).chain(ret, (*compiler).emit_node(NOTHING));
            }
        }
        b'?' | b'=' => {
            parser.skipchr();
            // Optional: BRANCH -> atom -> BRANCH -> NOTHING
            (*compiler).insert_node(BRANCH, ret);
            (*compiler).chain(ret, (*compiler).emit_node(BRANCH));
            let next = (*compiler).emit_node(NOTHING);
            (*compiler).chain(ret, next);
            // Skip second branch to NOTHING
            if !(*compiler).is_counting() && !ret.is_null() {
                (*compiler).set_next(ret.add(NODE_SIZE), next);
            }
        }
        b'{' => {
            // Brace quantifier \{n,m}
            parser.skipchr(); // consume '{'

            // Read the limits - note: read_limits consumes up to and including '}'
            let Some((minval, maxval, reverse)) = parser.read_limits() else {
                return ptr::null_mut();
            };

            // Swap min/max for non-greedy if reverse specified
            let (minval, maxval) = if reverse {
                (maxval, minval)
            } else {
                (minval, maxval)
            };

            if (flags & SIMPLE) != 0 {
                // Simple atom: BRACE_SIMPLE + BRACE_LIMITS
                (*compiler).insert_node(BRACE_SIMPLE, ret);
                (*compiler).insert_limits(BRACE_LIMITS, minval, maxval, ret);
            } else {
                // Complex atom: BRACE_COMPLEX + BACK structure + BRACE_LIMITS
                if (*compiler).complex_brace_count() >= 10 {
                    parser.set_error("E60: Too many complex \\{...}s");
                    return ptr::null_mut();
                }

                let brace_num = (*compiler).next_complex_brace();
                (*compiler).insert_node(BRACE_COMPLEX + brace_num, ret);

                // Add BACK node and chain it
                let back = (*compiler).emit_node(BACK);
                (*compiler).chain(ret, back);
                (*compiler).chain(ret, ret);

                // Insert BRACE_LIMITS at the beginning
                (*compiler).insert_limits(BRACE_LIMITS, minval, maxval, ret);
            }
        }
        _ => {}
    }

    // Check for greedy/non-greedy modifier
    if parser.peekchr() == magic(b'?' as c_int) {
        // Non-greedy - not implemented yet
        parser.skipchr();
    }

    ret
}

/// Parse a single atom.
///
/// # Safety
/// Compiler must be valid.
unsafe fn parse_atom(
    compiler: *mut RegCompiler,
    parser: &mut PatternParser,
    flagp: *mut c_int,
) -> *mut u8 {
    *flagp = WORST;

    let c = parser.getchr();

    if c == 0 {
        return ptr::null_mut();
    }

    // Check magic characters
    if is_magic(c) {
        let mc = un_magic(c) as u8;
        match mc {
            b'^' => {
                // Beginning of line
                let ret = (*compiler).emit_node(BOL);
                return ret;
            }
            b'$' => {
                // End of line
                let ret = (*compiler).emit_node(EOL);
                return ret;
            }
            b'<' => {
                // Beginning of word
                let ret = (*compiler).emit_node(BOW);
                return ret;
            }
            b'>' => {
                // End of word
                let ret = (*compiler).emit_node(EOW);
                return ret;
            }
            b'.' => {
                // Any character
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(ANY);
                return ret;
            }
            b'(' => {
                // Grouped expression
                let mut inner_flags: c_int = 0;
                let ret = parse_reg(compiler, parser, REG_PAREN, &mut inner_flags);
                *flagp |= inner_flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
                return ret;
            }
            b'n' => {
                // Newline
                *flagp |= HASWIDTH | HASNL;
                let ret = (*compiler).emit_node(NEWL);
                return ret;
            }
            b'd' => {
                // Digit
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(DIGIT);
                return ret;
            }
            b'D' => {
                // Non-digit
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(NDIGIT);
                return ret;
            }
            b'w' => {
                // Word character
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(WORD);
                return ret;
            }
            b'W' => {
                // Non-word character
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(NWORD);
                return ret;
            }
            b's' => {
                // Whitespace
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(WHITE);
                return ret;
            }
            b'S' => {
                // Non-whitespace
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(NWHITE);
                return ret;
            }
            b'x' => {
                // Hex digit
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(HEX);
                return ret;
            }
            b'X' => {
                // Non-hex digit
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(NHEX);
                return ret;
            }
            b'a' => {
                // Alpha
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(ALPHA);
                return ret;
            }
            b'A' => {
                // Non-alpha
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(NALPHA);
                return ret;
            }
            b'l' => {
                // Lowercase
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(LOWER);
                return ret;
            }
            b'L' => {
                // Non-lowercase
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(NLOWER);
                return ret;
            }
            b'u' => {
                // Uppercase
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(UPPER);
                return ret;
            }
            b'U' => {
                // Non-uppercase
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(NUPPER);
                return ret;
            }
            b'i' => {
                // Identifier char
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(IDENT);
                return ret;
            }
            b'I' => {
                // Identifier start char
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(SIDENT);
                return ret;
            }
            b'k' => {
                // Keyword char
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(KWORD);
                return ret;
            }
            b'K' => {
                // Keyword start char
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(SKWORD);
                return ret;
            }
            b'f' => {
                // Filename char
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(FNAME);
                return ret;
            }
            b'F' => {
                // Filename start char
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(SFNAME);
                return ret;
            }
            b'p' => {
                // Printable char
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(PRINT);
                return ret;
            }
            b'P' => {
                // Non-printable char
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(SPRINT);
                return ret;
            }
            b'h' => {
                // Head char
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(HEAD);
                return ret;
            }
            b'H' => {
                // Non-head char
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(NHEAD);
                return ret;
            }
            b'o' => {
                // Octal digit
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(OCTAL);
                return ret;
            }
            b'O' => {
                // Non-octal digit
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(NOCTAL);
                return ret;
            }
            b'[' => {
                // Character class - skip for now
                parser.set_error("E59: Character classes not yet implemented");
                return ptr::null_mut();
            }
            b'z' => {
                // \z prefix: \zs, \ze, \z1-\z9, \z(...)
                // The character after \z is checked directly (not through magic)
                let next_chr = parser.peekchr();
                // Convert any magic encoding back to literal, or use literal directly
                let next_byte = if is_magic(next_chr) {
                    un_magic(next_chr) as u8
                } else if next_chr > 0 && next_chr < 256 {
                    next_chr as u8
                } else {
                    0 // Invalid
                };

                match next_byte {
                    b's' => {
                        // \zs - set match start
                        parser.skipchr();
                        // Emit MOPEN (submatch 0) to mark match start
                        // Note: submatch 0 is the whole match, so this effectively sets
                        // the visible start of the match, not a capturing group.
                        let ret = (*compiler).emit_node(MOPEN);
                        // No HASWIDTH flag - this is zero-width
                        return ret;
                    }
                    b'e' => {
                        // \ze - set match end
                        parser.skipchr();
                        // Emit MCLOSE (submatch 0) to mark match end
                        let ret = (*compiler).emit_node(MCLOSE);
                        // No HASWIDTH flag - this is zero-width
                        return ret;
                    }
                    b'(' => {
                        // \z(...\) - external capturing group
                        // Note: In nomagic mode, this would be \z\(
                        // but we handle the magic ( case here too
                        parser.skipchr();
                        let mut inner_flags: c_int = 0;
                        let ret = parse_reg(compiler, parser, REG_ZPAREN, &mut inner_flags);
                        *flagp |= inner_flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
                        parser.set_has_z(true);
                        return ret;
                    }
                    b'1'..=b'9' => {
                        // \z1-\z9 - external backreference
                        parser.skipchr();
                        let n = next_byte - b'0';
                        let ret = (*compiler).emit_node(ZREF + n as c_int);
                        *flagp |= HASWIDTH;
                        parser.set_has_z(true);
                        return ret;
                    }
                    _ => {
                        parser.set_error("E68: Invalid character after \\z");
                        return ptr::null_mut();
                    }
                }
            }
            b'1'..=b'9' => {
                // Backreference
                let n = mc - b'0';
                let ret = (*compiler).emit_node(BACKREF + n as c_int);
                *flagp |= HASWIDTH;
                return ret;
            }
            _ => {
                // Literal magic character
                *flagp |= HASWIDTH | SIMPLE;
                let ret = (*compiler).emit_node(EXACTLY);
                (*compiler).emit_byte(mc);
                (*compiler).emit_byte(0);
                return ret;
            }
        }
    }

    // Literal character - emit EXACTLY
    *flagp |= HASWIDTH | SIMPLE;
    let ret = (*compiler).emit_node(EXACTLY);
    (*compiler).emit_byte(c as u8);

    // Collect consecutive literal chars
    loop {
        let nc = parser.peekchr();
        if nc == 0 || is_magic(nc) {
            break;
        }
        (*compiler).emit_byte(nc as u8);
        parser.skipchr();
    }

    (*compiler).emit_byte(0); // NUL terminator

    ret
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Parse a pattern and emit bytecode.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_parse_pattern(
    compiler: *mut RegCompiler,
    pattern: *const u8,
    pattern_len: usize,
    re_flags: c_int,
    out_flags: *mut c_int,
) -> *mut u8 {
    if compiler.is_null() || pattern.is_null() || pattern_len == 0 {
        return ptr::null_mut();
    }

    let pattern_slice = std::slice::from_raw_parts(pattern, pattern_len);
    parse_pattern(compiler, pattern_slice, re_flags, out_flags)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_modes() {
        assert_eq!(MAGIC_OFF, 0);
        assert_eq!(MAGIC_ON, 1);
        assert_eq!(MAGIC_ALL, 2);
        assert_eq!(MAGIC_NONE, 3);
    }

    #[test]
    fn test_parser_simple() {
        let pattern = b"abc";
        let parser = PatternParser::new(pattern, 0);
        assert!(!parser.at_end());
        assert!(!parser.has_error());
    }

    #[test]
    fn test_parser_magic() {
        let pattern = b"a.c";
        let mut parser = PatternParser::new(pattern, 1); // MAGIC_ON

        // 'a' is literal
        assert_eq!(parser.peekchr(), b'a' as c_int);
        parser.skipchr();

        // '.' is magic in MAGIC_ON mode
        assert!(is_magic(parser.peekchr()));
        assert_eq!(un_magic(parser.peekchr()), b'.' as c_int);
    }

    #[test]
    fn test_parser_escape() {
        let pattern = b"a\\dc";
        let mut parser = PatternParser::new(pattern, 0);

        // 'a'
        parser.skipchr();

        // '\d' should be magic 'd'
        let c = parser.peekchr();
        assert!(is_magic(c));
        assert_eq!(un_magic(c), b'd' as c_int);
    }

    #[test]
    fn test_parser_at_end() {
        let pattern = b"x";
        let mut parser = PatternParser::new(pattern, 0);
        parser.skipchr();
        assert_eq!(parser.peekchr(), 0);
    }

    #[test]
    fn test_parser_z_prefix() {
        // Test \z prefix parsing - \z becomes magic 'z', and the following
        // character is parsed normally. The atom parser handles \z specially.
        let pattern = b"a\\zsb";
        let mut parser = PatternParser::new(pattern, 0);

        // 'a'
        assert_eq!(parser.peekchr(), b'a' as c_int);
        parser.skipchr();

        // '\z' should produce magic 'z'
        let c = parser.peekchr();
        assert!(is_magic(c));
        assert_eq!(un_magic(c), b'z' as c_int);
        parser.skipchr();

        // 's' is a regular literal character - the atom parser handles
        // the \zs combination at a higher level
        let c = parser.peekchr();
        assert_eq!(c, b's' as c_int);
        parser.skipchr();

        // 'b' is literal
        assert_eq!(parser.peekchr(), b'b' as c_int);
    }

    #[test]
    fn test_parser_z_paren() {
        // Test \z( parsing - \z becomes magic, ( needs \( in nomagic mode
        let pattern = b"\\z\\(";
        let mut parser = PatternParser::new(pattern, 0);

        // '\z' should produce magic 'z'
        let c = parser.peekchr();
        assert!(is_magic(c));
        assert_eq!(un_magic(c), b'z' as c_int);
        parser.skipchr();

        // '\(' should be magic '(' (grouping)
        let c = parser.peekchr();
        assert!(is_magic(c));
        assert_eq!(un_magic(c), b'(' as c_int);
    }

    #[test]
    fn test_parser_z_backref() {
        // Test \z1 parsing - \z is prefix, 1 is literal
        let pattern = b"\\z1";
        let mut parser = PatternParser::new(pattern, 0);

        // '\z' should produce magic 'z'
        let c = parser.peekchr();
        assert!(is_magic(c));
        assert_eq!(un_magic(c), b'z' as c_int);
        parser.skipchr();

        // '1' is literal - the atom parser handles \z1 as external backreference
        let c = parser.peekchr();
        assert_eq!(c, b'1' as c_int);
    }

    #[test]
    fn test_z_prefix_needs_atom_handling() {
        // The \z prefix is converted to magic 'z', and the parse_atom
        // function is responsible for looking at the following character
        // to determine if it's \zs, \ze, \z1-9, or \z(
        let pattern = b"\\ze";
        let mut parser = PatternParser::new(pattern, 0);

        // '\z' produces magic 'z'
        let c = parser.peekchr();
        assert!(is_magic(c));
        assert_eq!(un_magic(c), b'z' as c_int);
        parser.skipchr();

        // 'e' is literal - parse_atom checks for this and handles \ze
        let c = parser.peekchr();
        assert_eq!(c, b'e' as c_int);
    }

    #[test]
    fn test_parse_assertion_type() {
        // Test \@= (positive lookahead)
        let pattern = b"=rest";
        let mut parser = PatternParser::new(pattern, 0);
        assert_eq!(parse_assertion_type(&mut parser), MATCH);
        assert_eq!(parser.peekchr(), b'r' as c_int);

        // Test \@! (negative lookahead)
        let pattern = b"!rest";
        let mut parser = PatternParser::new(pattern, 0);
        assert_eq!(parse_assertion_type(&mut parser), NOMATCH);

        // Test \@> (atomic grouping)
        let pattern = b">rest";
        let mut parser = PatternParser::new(pattern, 0);
        assert_eq!(parse_assertion_type(&mut parser), SUBPAT);

        // Test \@<= (positive lookbehind)
        let pattern = b"<=rest";
        let mut parser = PatternParser::new(pattern, 0);
        assert_eq!(parse_assertion_type(&mut parser), BEHIND);
        assert_eq!(parser.peekchr(), b'r' as c_int);

        // Test \@<! (negative lookbehind)
        let pattern = b"<!rest";
        let mut parser = PatternParser::new(pattern, 0);
        assert_eq!(parse_assertion_type(&mut parser), NOBEHIND);

        // Test invalid character
        let pattern = b"xrest";
        let mut parser = PatternParser::new(pattern, 0);
        assert_eq!(parse_assertion_type(&mut parser), END);
        // Parser shouldn't have consumed any characters on failure
        assert_eq!(parser.peekchr(), b'x' as c_int);
    }

    #[test]
    fn test_at_prefix_parsing() {
        // Test \@ is recognized as magic '@'
        let pattern = b"\\@=";
        let mut parser = PatternParser::new(pattern, 0);

        // '\@' produces magic '@'
        let c = parser.peekchr();
        assert!(is_magic(c));
        assert_eq!(un_magic(c), b'@' as c_int);
        parser.skipchr();

        // '=' is literal
        assert_eq!(parser.peekchr(), b'=' as c_int);
    }
}
