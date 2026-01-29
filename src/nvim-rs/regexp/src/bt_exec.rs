//! Backtracking (BT) regex engine execution.
//!
//! This module implements the execution phase of the BT regex engine.
//! It uses a stack-based recursive descent algorithm with explicit
//! backtracking when a match attempt fails.
//!
//! # Algorithm Overview
//!
//! 1. Push initial state onto the backtrack stack
//! 2. Try to match at current position
//! 3. On success: advance and continue
//! 4. On failure: pop stack and try alternative
//! 5. Repeat until match found or all alternatives exhausted
//!
//! # Key Functions
//!
//! - [`MatchState`]: Execution state for a match attempt
//! - [`regmatch`]: Main matching function
//! - [`regrepeat`]: Handle repetition operators
//!
//! # Integration with ExecState
//!
//! This module integrates with [`crate::exec_state::ExecState`] for unified
//! execution state management, and [`crate::line_fetch`] for buffer line access.

use std::ffi::c_int;
use std::ptr;

use crate::bt_compile::{next, op, operand};
use crate::bt_opcodes::{
    get_backref_num, get_mclose_num, get_mopen_num, is_backref, is_mclose, is_mopen, operand_cmp,
    operand_max, operand_min, ADD_NL, ALPHA, ANY, ANYBUT, ANYOF, BACK, BEHIND, BHPOS, BOL, BOW,
    BRACE_COMPLEX, BRACE_LIMITS, BRACE_SIMPLE, BRANCH, CURSOR, DIGIT, END, EOL, EOW, EXACTLY,
    FNAME, HEAD, HEX, IDENT, LOWER, MATCH, MULTIBYTECODE, NALPHA, NCLOSE, NDIGIT, NEWL, NHEAD,
    NHEX, NLOWER, NOBEHIND, NOMATCH, NOPEN, NOTHING, NUPPER, NWHITE, NWORD, OCTAL, PLUS, PRINT,
    RE_BOF, RE_COL, RE_COMPOSING, RE_EOF, RE_LNUM, RE_MARK, RE_VCOL, RE_VISUAL, SFNAME, SKWORD,
    SPRINT, STAR, SUBPAT, UPPER, WHITE, WORD,
};
use crate::bt_state::{BackPosTable, RegSave, RegStack, RegStar, RegState, NSUBEXP};

// =============================================================================
// Match Result
// =============================================================================

/// Result of a match attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchResult {
    /// Match found
    Match,
    /// No match
    NoMatch,
    /// Error during matching
    Error,
    /// Timed out
    TimedOut,
}

impl MatchResult {
    /// Check if this is a successful match
    pub fn is_match(&self) -> bool {
        matches!(self, MatchResult::Match)
    }
}

// =============================================================================
// Match State
// =============================================================================

/// Execution state for a regex match attempt.
///
/// This tracks the current position in both the input and the bytecode,
/// as well as submatch positions and other match state.
pub struct MatchState {
    /// Current position in input (single-line mode)
    input: *const u8,

    /// Start of input line
    line_start: *const u8,

    /// Current bytecode position
    #[allow(dead_code)]
    scan: *const u8,

    /// Program bytecode
    #[allow(dead_code)]
    program: *const u8,

    /// Start positions of submatches
    startp: [*const u8; NSUBEXP],

    /// End positions of submatches
    endp: [*const u8; NSUBEXP],

    /// Whether we're in multi-line mode
    multi: bool,

    /// Current line number (multi-line mode)
    lnum: c_int,

    /// Current column (multi-line mode)
    col: c_int,

    /// Whether to match newlines with '.'
    match_nl: bool,

    /// Case-insensitive matching
    #[allow(dead_code)]
    ignore_case: bool,

    /// Whether subexpressions need to be cleared
    #[allow(dead_code)]
    need_clear_subexpr: bool,

    /// Backtrack stack
    stack: RegStack,

    /// Back position table
    backpos: BackPosTable,

    /// BRACE_SIMPLE min/max values (from preceding BRACE_LIMITS)
    bl_minval: i64,
    bl_maxval: i64,

    /// BRACE_COMPLEX min/max/count arrays (for nested braces \{n,m})
    brace_min: [i64; NSUBEXP],
    brace_max: [i64; NSUBEXP],
    brace_count: [i64; NSUBEXP],
}

impl MatchState {
    /// Create a new match state for single-line matching.
    pub fn new_single(input: *const u8, program: *const u8) -> Self {
        Self {
            input,
            line_start: input,
            scan: ptr::null(),
            program,
            startp: [ptr::null(); NSUBEXP],
            endp: [ptr::null(); NSUBEXP],
            multi: false,
            lnum: 0,
            col: 0,
            match_nl: false,
            ignore_case: false,
            need_clear_subexpr: false,
            stack: RegStack::new(),
            backpos: BackPosTable::new(),
            bl_minval: 0,
            bl_maxval: i64::MAX,
            brace_min: [0; NSUBEXP],
            brace_max: [0; NSUBEXP],
            brace_count: [0; NSUBEXP],
        }
    }

    /// Create a new match state for multi-line matching.
    pub fn new_multi(program: *const u8, lnum: c_int, col: c_int) -> Self {
        Self {
            input: ptr::null(),
            line_start: ptr::null(),
            scan: ptr::null(),
            program,
            startp: [ptr::null(); NSUBEXP],
            endp: [ptr::null(); NSUBEXP],
            multi: true,
            lnum,
            col,
            match_nl: false,
            ignore_case: false,
            need_clear_subexpr: false,
            stack: RegStack::new(),
            backpos: BackPosTable::new(),
            bl_minval: 0,
            bl_maxval: i64::MAX,
            brace_min: [0; NSUBEXP],
            brace_max: [0; NSUBEXP],
            brace_count: [0; NSUBEXP],
        }
    }

    /// Set the current input position
    pub fn set_input(&mut self, input: *const u8) {
        self.input = input;
    }

    /// Set the line start
    pub fn set_line_start(&mut self, start: *const u8) {
        self.line_start = start;
    }

    /// Get the current input position
    pub fn input(&self) -> *const u8 {
        self.input
    }

    /// Advance input by one byte
    ///
    /// # Safety
    /// Input must be valid and have at least one more byte
    pub unsafe fn advance_input(&mut self) {
        self.input = self.input.add(1);
    }

    /// Advance input by n bytes
    ///
    /// # Safety
    /// Input must be valid and have at least n more bytes
    pub unsafe fn advance_input_by(&mut self, n: usize) {
        self.input = self.input.add(n);
    }

    /// Get the current byte at input position
    ///
    /// # Safety
    /// Input must be valid
    pub unsafe fn current_byte(&self) -> u8 {
        if self.input.is_null() {
            0
        } else {
            *self.input
        }
    }

    /// Get the previous byte (before current position)
    ///
    /// # Safety
    /// Input must be valid
    pub unsafe fn prev_byte(&self) -> u8 {
        if self.input.is_null() || self.input == self.line_start {
            0
        } else {
            *self.input.sub(1)
        }
    }

    /// Check if at end of line
    ///
    /// # Safety
    /// Input must be valid
    pub unsafe fn at_eol(&self) -> bool {
        self.input.is_null() || *self.input == 0 || *self.input == b'\n'
    }

    /// Check if at beginning of line
    pub fn at_bol(&self) -> bool {
        self.input == self.line_start
    }

    /// Save the current position
    pub fn save_pos(&self) -> RegSave {
        if self.multi {
            RegSave {
                multi: crate::bt_state::RegSaveMulti {
                    line: self.lnum,
                    col: self.col,
                },
            }
        } else {
            RegSave {
                pos: crate::bt_state::RegSavePos {
                    pos: self.input as *mut u8,
                },
            }
        }
    }

    /// Restore a saved position
    ///
    /// # Safety
    /// The saved position must be valid
    pub unsafe fn restore_pos(&mut self, save: &RegSave) {
        if self.multi {
            self.lnum = save.multi.line;
            self.col = save.multi.col;
        } else {
            self.input = save.pos.pos;
        }
    }

    /// Get submatch start position
    pub fn get_startp(&self, idx: usize) -> *const u8 {
        if idx < NSUBEXP {
            self.startp[idx]
        } else {
            ptr::null()
        }
    }

    /// Set submatch start position
    pub fn set_startp(&mut self, idx: usize, pos: *const u8) {
        if idx < NSUBEXP {
            self.startp[idx] = pos;
        }
    }

    /// Get submatch end position
    pub fn get_endp(&self, idx: usize) -> *const u8 {
        if idx < NSUBEXP {
            self.endp[idx]
        } else {
            ptr::null()
        }
    }

    /// Set submatch end position
    pub fn set_endp(&mut self, idx: usize, pos: *const u8) {
        if idx < NSUBEXP {
            self.endp[idx] = pos;
        }
    }

    /// Clear all submatches
    pub fn clear_submatches(&mut self) {
        for i in 0..NSUBEXP {
            self.startp[i] = ptr::null();
            self.endp[i] = ptr::null();
        }
    }

    /// Push a backtrack item onto the stack
    pub fn push_backtrack(&mut self, state: RegState, scan: *mut u8) -> bool {
        self.stack.push_item(state, scan).is_some()
    }

    /// Pop a backtrack item from the stack
    pub fn pop_backtrack(&mut self) -> Option<*mut u8> {
        self.stack.pop_item()
    }

    /// Check if backtrack stack is empty
    pub fn backtrack_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Clear the backtrack stack
    pub fn clear_backtrack(&mut self) {
        self.stack.clear();
    }

    /// Clear the backpos table
    pub fn clear_backpos(&mut self) {
        self.backpos.clear();
    }

    /// Cleanup after a match attempt
    pub fn cleanup(&mut self) {
        self.stack.shrink_if_grown();
        self.backpos.shrink_if_grown();
    }
}

impl Default for MatchState {
    fn default() -> Self {
        Self::new_single(ptr::null(), ptr::null())
    }
}

// =============================================================================
// Match Helpers
// =============================================================================

/// Match a single character against a character class.
///
/// Returns true if the character matches.
///
/// # Safety
/// `class_start` must point to valid bytecode operand
pub unsafe fn match_class(class_start: *const u8, c: u8) -> bool {
    if class_start.is_null() {
        return false;
    }

    // Simple implementation: scan the class string
    let mut p = class_start;
    while *p != 0 {
        if *p == c {
            return true;
        }
        p = p.add(1);
    }
    false
}

/// Count how many times a simple atom can match.
///
/// Used for `*`, `+`, and `{n,m}` operators.
///
/// # Safety
/// `state` and `scan` must be valid
pub unsafe fn regrepeat(state: &mut MatchState, scan: *const u8, maxcount: i64) -> i64 {
    let mut count = 0i64;
    let opcode = op(scan);
    let opnd = operand(scan);

    while count < maxcount {
        let c = state.current_byte();
        if c == 0 {
            break;
        }

        let matches = match opcode {
            // Basic matchers
            ANY => c != b'\n' || state.match_nl,
            ANYOF => match_class(opnd, c),
            ANYBUT => !match_class(opnd, c) && c != b'\n',
            EXACTLY => !opnd.is_null() && *opnd == c,
            NEWL => c == b'\n',

            // Whitespace classes
            WHITE => c == b' ' || c == b'\t',
            NWHITE => c != b' ' && c != b'\t' && c != b'\n',

            // Digit classes
            DIGIT => c.is_ascii_digit(),
            NDIGIT => !c.is_ascii_digit() && c != b'\n',

            // Hex digit classes
            HEX => c.is_ascii_hexdigit(),
            NHEX => !c.is_ascii_hexdigit() && c != b'\n',

            // Octal digit classes
            OCTAL => matches!(c, b'0'..=b'7'),
            // NOCTAL intentionally not added here - would need NOCTAL constant

            // Word/identifier classes
            WORD => c.is_ascii_alphanumeric() || c == b'_',
            NWORD => !c.is_ascii_alphanumeric() && c != b'_' && c != b'\n',

            // Head (start of identifier)
            HEAD => c.is_ascii_alphabetic() || c == b'_',
            NHEAD => !c.is_ascii_alphabetic() && c != b'_' && c != b'\n',

            // Alpha classes
            ALPHA => c.is_ascii_alphabetic(),
            NALPHA => !c.is_ascii_alphabetic() && c != b'\n',

            // Case classes
            LOWER => c.is_ascii_lowercase(),
            NLOWER => !c.is_ascii_lowercase() && c != b'\n',
            UPPER => c.is_ascii_uppercase(),
            NUPPER => !c.is_ascii_uppercase() && c != b'\n',

            // Print classes (ASCII printable)
            PRINT => c.is_ascii_graphic() || c == b' ',
            SPRINT => c.is_ascii_graphic(), // Start of printable (excludes leading space)

            // Identifier classes (for now, treat as word chars for ASCII)
            IDENT => c.is_ascii_alphanumeric() || c == b'_',
            // SIDENT handled same as IDENT for ASCII
            SKWORD => c.is_ascii_alphanumeric() || c == b'_',

            // Filename characters (basic ASCII version)
            FNAME => is_fname_char(c),
            SFNAME => is_fname_char(c),

            _ => false,
        };

        if !matches {
            break;
        }

        state.advance_input();
        count += 1;
    }

    count
}

// =============================================================================
// Main Match Engine
// =============================================================================

/// Internal result codes for matching.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum MatchStatus {
    /// Continue matching
    Continue = 0,
    /// Match failed, try backtrack
    NoMatch = 1,
    /// Match succeeded
    Match = 2,
    /// Error occurred
    Fail = 3,
    /// Break out of inner loop
    Break = 4,
}

/// Main regex matching function for BT engine.
///
/// Tries to match the compiled pattern against the input at the current position.
/// Uses a stack-based approach to handle backtracking.
///
/// # Safety
/// state must be valid with valid program pointer.
pub unsafe fn regmatch(state: &mut MatchState, program: *const u8) -> MatchResult {
    if program.is_null() {
        return MatchResult::Error;
    }

    // Start at the first instruction (skip REGMAGIC byte)
    let mut scan = program.add(1);

    loop {
        if scan.is_null() {
            return MatchResult::Error;
        }

        let opcode = op(scan);

        // Check for END first
        if opcode == END {
            return MatchResult::Match;
        }

        let next_scan = next(scan);
        let status = match_one_op(state, scan, opcode);

        match status {
            MatchStatus::Continue => {
                // Move to next instruction
                scan = if next_scan.is_null() {
                    scan.add(3) // Default: advance past node
                } else {
                    next_scan
                };
            }
            MatchStatus::Match => {
                return MatchResult::Match;
            }
            MatchStatus::NoMatch => {
                // Try backtracking
                if let Some(back_scan) = state.pop_backtrack() {
                    scan = back_scan;
                } else {
                    return MatchResult::NoMatch;
                }
            }
            MatchStatus::Fail => {
                return MatchResult::Error;
            }
            MatchStatus::Break => {
                // Move to next instruction (used for branches that need continuation)
                scan = if next_scan.is_null() {
                    scan.add(3)
                } else {
                    next_scan
                };
            }
        }
    }
}

/// Match a single opcode.
///
/// # Safety
/// scan must point to valid bytecode.
unsafe fn match_one_op(state: &mut MatchState, scan: *const u8, opcode: c_int) -> MatchStatus {
    let next_scan = next(scan);

    match opcode {
        BOL => {
            if state.at_bol() {
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        EOL => {
            if state.at_eol() {
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        BOW => {
            // Beginning of word: previous char is not word, current is
            let at_word_start = state.at_bol() || !is_word_char(state.prev_byte());
            let cur = state.current_byte();
            if at_word_start && is_word_char(cur) {
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        EOW => {
            // End of word: current is not word char but previous was
            let cur = state.current_byte();
            if !is_word_char(cur) && !state.at_bol() && is_word_char(state.prev_byte()) {
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        ANY => {
            let c = state.current_byte();
            if c != 0 && (c != b'\n' || state.match_nl) {
                state.advance_input();
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        EXACTLY => {
            let opnd = operand(scan);
            if opnd.is_null() {
                return MatchStatus::NoMatch;
            }

            // Match each character in the operand
            let mut p = opnd;
            while *p != 0 {
                if state.current_byte() != *p {
                    return MatchStatus::NoMatch;
                }
                state.advance_input();
                p = p.add(1);
            }
            MatchStatus::Continue
        }

        ANYOF => {
            let c = state.current_byte();
            if c != 0 && match_class(operand(scan), c) {
                state.advance_input();
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        ANYBUT => {
            let c = state.current_byte();
            if c != 0 && c != b'\n' && !match_class(operand(scan), c) {
                state.advance_input();
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        NOTHING => MatchStatus::Continue,

        BACK => {
            // BACK is used in loops - just continue
            MatchStatus::Continue
        }

        BRANCH => {
            let next_op = op(next_scan);
            if next_op != BRANCH {
                // No alternative - just continue with operand
                MatchStatus::Continue
            } else {
                // Push alternative onto backtrack stack
                if !state.push_backtrack(RegState::Branch, next_scan as *mut u8) {
                    return MatchStatus::Fail;
                }
                MatchStatus::Continue
            }
        }

        // Character classes
        DIGIT => match_char_class(state, |c| c.is_ascii_digit()),
        NDIGIT => match_char_class(state, |c| !c.is_ascii_digit() && c != b'\n'),
        WHITE => match_char_class(state, |c| c == b' ' || c == b'\t'),
        NWHITE => match_char_class(state, |c| c != b' ' && c != b'\t' && c != b'\n'),
        WORD => match_char_class(state, is_word_char),
        NWORD => match_char_class(state, |c| !is_word_char(c) && c != b'\n'),
        HEAD => match_char_class(state, |c| c.is_ascii_alphabetic() || c == b'_'),
        NHEAD => match_char_class(state, |c| {
            !c.is_ascii_alphabetic() && c != b'_' && c != b'\n'
        }),
        ALPHA => match_char_class(state, |c| c.is_ascii_alphabetic()),
        NALPHA => match_char_class(state, |c| !c.is_ascii_alphabetic() && c != b'\n'),
        LOWER => match_char_class(state, |c| c.is_ascii_lowercase()),
        NLOWER => match_char_class(state, |c| !c.is_ascii_lowercase() && c != b'\n'),
        UPPER => match_char_class(state, |c| c.is_ascii_uppercase()),
        NUPPER => match_char_class(state, |c| !c.is_ascii_uppercase() && c != b'\n'),
        HEX => match_char_class(state, |c| c.is_ascii_hexdigit()),
        NHEX => match_char_class(state, |c| !c.is_ascii_hexdigit() && c != b'\n'),
        OCTAL => match_char_class(state, |c| matches!(c, b'0'..=b'7')),
        IDENT => match_char_class(state, |c| c.is_ascii_alphanumeric() || c == b'_'),
        PRINT => match_char_class(state, |c| (0x20..0x7f).contains(&c)),
        SPRINT => match_char_class(state, |c| (0x21..0x7f).contains(&c)),

        NEWL => {
            if state.current_byte() == b'\n' {
                state.advance_input();
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        // Subexpression markers
        op if is_mopen(op) => {
            let n = get_mopen_num(op) as usize;
            state.set_startp(n, state.input());
            MatchStatus::Continue
        }

        op if is_mclose(op) => {
            let n = get_mclose_num(op) as usize;
            state.set_endp(n, state.input());
            MatchStatus::Continue
        }

        // Non-capturing group markers
        NOPEN | NCLOSE => MatchStatus::Continue,

        // Backreference \1 - \9
        op if is_backref(op) => {
            let n = get_backref_num(op) as usize;
            let start = state.get_startp(n);
            let end = state.get_endp(n);

            if start.is_null() || end.is_null() {
                return MatchStatus::NoMatch;
            }

            // Match the captured text
            let mut p = start;
            while p < end {
                if state.current_byte() != *p {
                    return MatchStatus::NoMatch;
                }
                state.advance_input();
                p = p.add(1);
            }
            MatchStatus::Continue
        }

        // Quantifiers - these require proper backtracking
        STAR => {
            // * matches zero or more
            let opnd = operand(scan);
            let _save_input = state.input(); // For future backtrack restoration

            // Greedy: try to match as many as possible
            let count = regrepeat(state, opnd, i64::MAX);

            // Push backtrack points for each match (starting from max)
            // When we backtrack, we'll try fewer matches
            if count > 0 {
                // Save position for backtracking
                let star = RegStar {
                    count,
                    minval: 0,
                    maxval: count,
                };
                state.stack.push_star(star);
                if !state.push_backtrack(RegState::StarLong, next_scan as *mut u8) {
                    return MatchStatus::Fail;
                }
            }
            MatchStatus::Continue
        }

        PLUS => {
            // + matches one or more
            let opnd = operand(scan);

            // Greedy: try to match as many as possible
            let count = regrepeat(state, opnd, i64::MAX);

            if count < 1 {
                return MatchStatus::NoMatch;
            }

            // Push backtrack points for each extra match (starting from max)
            if count > 1 {
                let star = RegStar {
                    count,
                    minval: 1,
                    maxval: count,
                };
                state.stack.push_star(star);
                if !state.push_backtrack(RegState::StarLong, next_scan as *mut u8) {
                    return MatchStatus::Fail;
                }
            }
            MatchStatus::Continue
        }

        BRACE_SIMPLE => {
            // {n,m} on simple atom - use limits from preceding BRACE_LIMITS
            let opnd = operand(scan);
            let minval = state.bl_minval;
            let maxval = state.bl_maxval;

            // Match up to maxval times
            let count = regrepeat(state, opnd, maxval);

            // Check if we matched enough
            if count < minval {
                return MatchStatus::NoMatch;
            }

            if count > minval {
                // Can backtrack: set up backtrack point
                let star = RegStar {
                    count,
                    minval,
                    maxval,
                };
                state.stack.push_star(star);
                if !state.push_backtrack(RegState::StarLong, next_scan as *mut u8) {
                    return MatchStatus::Fail;
                }
            }
            MatchStatus::Continue
        }

        BRACE_LIMITS => {
            // Store min/max values for following BRACE_SIMPLE or BRACE_COMPLEX
            state.bl_minval = operand_min(scan);
            state.bl_maxval = operand_max(scan);

            // Check if followed by BRACE_COMPLEX (indexed)
            let next_op = op(next_scan);
            if (BRACE_COMPLEX..BRACE_COMPLEX + 10).contains(&next_op) {
                let idx = (next_op - BRACE_COMPLEX) as usize;
                if idx < NSUBEXP {
                    state.brace_min[idx] = state.bl_minval;
                    state.brace_max[idx] = state.bl_maxval;
                    state.brace_count[idx] = 0;
                }
            }
            MatchStatus::Continue
        }

        // Look-around (simplified)
        MATCH | NOMATCH | BEHIND | NOBEHIND | BHPOS | SUBPAT => {
            // These require special handling with saved state
            // For now, just continue (proper implementation requires more infrastructure)
            MatchStatus::Continue
        }

        // Special position matchers
        RE_BOF => {
            // Beginning of file
            if state.lnum == 0 && state.at_bol() {
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        RE_EOF => {
            // End of file - at last line and at NUL
            if state.current_byte() == 0 {
                // In single-line mode or at last line
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        CURSOR => {
            // Match cursor position - requires window context
            // Simplified: treat as no-match for now
            MatchStatus::NoMatch
        }

        RE_LNUM => {
            // Match line number comparison (\%l, \%<l, \%>l)
            let limit = operand_min(scan);
            let cmp_op = operand_cmp(scan);
            // lnum is 0-based internally, but limit is 1-based from pattern
            let actual = state.lnum as i64 + 1;
            if compare_pos(actual, limit, cmp_op) {
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        RE_COL => {
            // Match column comparison (\%c, \%<c, \%>c)
            let limit = operand_min(scan);
            let cmp_op = operand_cmp(scan);
            // col is 0-based internally, limit is 1-based from pattern
            let actual = state.col as i64 + 1;
            if compare_pos(actual, limit, cmp_op) {
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        RE_VCOL => {
            // Match virtual column comparison (\%v, \%<v, \%>v)
            // Virtual column considers tabs - for now treat same as column
            let limit = operand_min(scan);
            let cmp_op = operand_cmp(scan);
            let actual = state.col as i64 + 1;
            if compare_pos(actual, limit, cmp_op) {
                MatchStatus::Continue
            } else {
                MatchStatus::NoMatch
            }
        }

        RE_MARK => {
            // Match mark position - requires buffer context
            MatchStatus::NoMatch
        }

        RE_VISUAL => {
            // Match visual selection - requires window context
            MatchStatus::NoMatch
        }

        RE_COMPOSING => {
            // Match composing characters
            MatchStatus::Continue
        }

        MULTIBYTECODE => {
            // Match a specific multibyte character (stored in operand)
            let opnd = operand(scan);
            // Get the expected character from operand (UTF-8 encoded)
            let expected_len = utf8_char_len(*opnd);
            if expected_len == 0 {
                return MatchStatus::NoMatch;
            }

            // Compare bytes
            for i in 0..expected_len {
                if state.current_byte() != *opnd.add(i) {
                    return MatchStatus::NoMatch;
                }
                state.advance_input();
            }
            MatchStatus::Continue
        }

        // More character classes with digit restriction
        SKWORD => match_char_class(state, |c| {
            !c.is_ascii_digit() && (c.is_ascii_alphanumeric() || c == b'_')
        }),
        SFNAME => match_char_class(state, |c| !c.is_ascii_digit() && is_fname_char(c)),

        // BRACE_COMPLEX: complex brace repetition with backtracking
        // This is handled specially because it needs to modify the next pointer
        // rather than using the normal flow
        local_op if (BRACE_COMPLEX..BRACE_COMPLEX + 10).contains(&local_op) => {
            let idx = (local_op - BRACE_COMPLEX) as usize;
            if idx >= NSUBEXP {
                return MatchStatus::NoMatch;
            }

            // Increment count
            state.brace_count[idx] += 1;
            let count = state.brace_count[idx];
            let minval = state.brace_min[idx];
            let maxval = state.brace_max[idx];

            // Determine effective min/max (handle reversed ranges)
            let effective_min = minval.min(maxval);

            // If not matched enough times yet, must try more
            if count <= effective_min {
                // Push backtrack state to try more
                if !state.push_backtrack(RegState::BrcplxMore, scan as *mut u8) {
                    return MatchStatus::Fail;
                }
                // Signal to continue with operand (handled by main loop)
                MatchStatus::Continue
            } else if minval <= maxval {
                // Normal range: try longest match (greedy)
                if count <= maxval {
                    // Can try more - push backtrack point
                    if !state.push_backtrack(RegState::BrcplxLong, scan as *mut u8) {
                        return MatchStatus::Fail;
                    }
                }
                MatchStatus::Continue
            } else {
                // Reversed range: try shortest match first
                if count <= minval {
                    // Can try more - push backtrack point
                    if !state.push_backtrack(RegState::BrcplxShort, scan as *mut u8) {
                        return MatchStatus::Fail;
                    }
                }
                MatchStatus::Continue
            }
        }

        END => MatchStatus::Match,

        // Handle NL variants (opcode + ADD_NL)
        op if op > ADD_NL && op <= NUPPER + ADD_NL => {
            // First try newline match
            if state.current_byte() == b'\n' {
                state.advance_input();
                return MatchStatus::Continue;
            }
            // Then try base opcode
            match_one_op(state, scan, op - ADD_NL)
        }

        _ => {
            // Unknown opcode
            MatchStatus::NoMatch
        }
    }
}

/// Get UTF-8 character length from first byte.
#[inline]
fn utf8_char_len(b: u8) -> usize {
    if b < 0x80 {
        1
    } else if b < 0xC0 {
        0 // continuation byte, invalid as start
    } else if b < 0xE0 {
        2
    } else if b < 0xF0 {
        3
    } else if b < 0xF8 {
        4
    } else {
        0 // invalid
    }
}

/// Check if a byte is a valid filename character.
#[inline]
fn is_fname_char(c: u8) -> bool {
    c.is_ascii_alphanumeric()
        || c == b'_'
        || c == b'.'
        || c == b'-'
        || c == b'/'
        || c == b'\\'
        || c == b'~'
}

/// Helper to match character classes.
#[inline]
unsafe fn match_char_class<F>(state: &mut MatchState, pred: F) -> MatchStatus
where
    F: Fn(u8) -> bool,
{
    let c = state.current_byte();
    if c != 0 && pred(c) {
        state.advance_input();
        MatchStatus::Continue
    } else {
        MatchStatus::NoMatch
    }
}

/// Check if a byte is a word character.
#[inline]
fn is_word_char(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Compare position value against limit with comparison operator.
///
/// The cmp_op byte encodes the comparison:
/// - '<' (0x3C): actual < limit
/// - '>' (0x3E): actual > limit
/// - other: actual == limit
#[inline]
fn compare_pos(actual: i64, limit: i64, cmp_op: u8) -> bool {
    match cmp_op {
        b'<' => actual < limit,
        b'>' => actual > limit,
        _ => actual == limit,
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create a new match state for single-line matching.
#[no_mangle]
pub extern "C" fn rs_bt_match_state_new(input: *const u8, program: *const u8) -> *mut MatchState {
    Box::into_raw(Box::new(MatchState::new_single(input, program)))
}

/// Create a new match state for multi-line matching.
#[no_mangle]
pub extern "C" fn rs_bt_match_state_new_multi(
    program: *const u8,
    lnum: c_int,
    col: c_int,
) -> *mut MatchState {
    Box::into_raw(Box::new(MatchState::new_multi(program, lnum, col)))
}

/// Free a match state.
///
/// # Safety
/// `state` must be a valid pointer from `rs_bt_match_state_new*`.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_free(state: *mut MatchState) {
    if !state.is_null() {
        drop(Box::from_raw(state));
    }
}

/// Set the input position.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_set_input(state: *mut MatchState, input: *const u8) {
    if !state.is_null() {
        (*state).set_input(input);
    }
}

/// Get the current input byte.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_current_byte(state: *const MatchState) -> u8 {
    if state.is_null() {
        0
    } else {
        (*state).current_byte()
    }
}

/// Advance input by one byte.
///
/// # Safety
/// `state` must be valid and input must have more bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_advance(state: *mut MatchState) {
    if !state.is_null() {
        (*state).advance_input();
    }
}

/// Check if at end of line.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_at_eol(state: *const MatchState) -> c_int {
    if state.is_null() {
        1
    } else {
        c_int::from((*state).at_eol())
    }
}

/// Check if at beginning of line.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_at_bol(state: *const MatchState) -> c_int {
    if state.is_null() {
        0
    } else {
        c_int::from((*state).at_bol())
    }
}

/// Set a submatch start position.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_set_startp(
    state: *mut MatchState,
    idx: c_int,
    pos: *const u8,
) {
    if !state.is_null() && idx >= 0 {
        (*state).set_startp(idx as usize, pos);
    }
}

/// Set a submatch end position.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_set_endp(
    state: *mut MatchState,
    idx: c_int,
    pos: *const u8,
) {
    if !state.is_null() && idx >= 0 {
        (*state).set_endp(idx as usize, pos);
    }
}

/// Get a submatch start position.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_get_startp(
    state: *const MatchState,
    idx: c_int,
) -> *const u8 {
    if state.is_null() || idx < 0 {
        ptr::null()
    } else {
        (*state).get_startp(idx as usize)
    }
}

/// Get a submatch end position.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_get_endp(
    state: *const MatchState,
    idx: c_int,
) -> *const u8 {
    if state.is_null() || idx < 0 {
        ptr::null()
    } else {
        (*state).get_endp(idx as usize)
    }
}

/// Clear all submatches.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_clear_submatches(state: *mut MatchState) {
    if !state.is_null() {
        (*state).clear_submatches();
    }
}

/// Push a backtrack item.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_push_backtrack(
    state: *mut MatchState,
    rs_state: c_int,
    scan: *mut u8,
) -> c_int {
    if state.is_null() {
        0
    } else {
        c_int::from((*state).push_backtrack(RegState::from(rs_state), scan))
    }
}

/// Pop a backtrack item.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_pop_backtrack(state: *mut MatchState) -> *mut u8 {
    if state.is_null() {
        ptr::null_mut()
    } else {
        (*state).pop_backtrack().unwrap_or(ptr::null_mut())
    }
}

/// Check if backtrack stack is empty.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_backtrack_empty(state: *const MatchState) -> c_int {
    if state.is_null() {
        1
    } else {
        c_int::from((*state).backtrack_empty())
    }
}

/// Cleanup after a match attempt.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_cleanup(state: *mut MatchState) {
    if !state.is_null() {
        (*state).cleanup();
    }
}

/// Count repetitions of a simple atom.
///
/// # Safety
/// `state` and `scan` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_regrepeat(
    state: *mut MatchState,
    scan: *const u8,
    maxcount: i64,
) -> i64 {
    if state.is_null() || scan.is_null() {
        0
    } else {
        regrepeat(&mut *state, scan, maxcount)
    }
}

/// Execute regex match.
///
/// Returns 1 for match, 0 for no match, -1 for error.
///
/// # Safety
/// `state` and `program` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_regmatch(state: *mut MatchState, program: *const u8) -> c_int {
    if state.is_null() || program.is_null() {
        return -1;
    }

    match regmatch(&mut *state, program) {
        MatchResult::Match => 1,
        MatchResult::NoMatch => 0,
        MatchResult::Error | MatchResult::TimedOut => -1,
    }
}

/// Initialize a match state for a new match attempt.
///
/// This resets the state for a fresh match at the given position.
///
/// # Safety
/// `state` and `input` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_init_match(
    state: *mut MatchState,
    input: *const u8,
    line_start: *const u8,
) {
    if state.is_null() {
        return;
    }
    let s = &mut *state;
    s.set_input(input);
    s.set_line_start(line_start);
    s.clear_submatches();
    s.stack.clear();
    s.backpos.clear();
}

/// Initialize a multi-line match state.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_init_match_multi(state: *mut MatchState, lnum: c_int, col: c_int) {
    if state.is_null() {
        return;
    }
    let s = &mut *state;
    s.lnum = lnum;
    s.col = col;
    s.clear_submatches();
    s.stack.clear();
    s.backpos.clear();
}

/// Save the current match position.
///
/// # Safety
/// `state` and `out` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_save_pos(state: *const MatchState, out: *mut RegSave) {
    if state.is_null() || out.is_null() {
        return;
    }
    *out = (*state).save_pos();
}

/// Restore a saved match position.
///
/// # Safety
/// `state` and `saved` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_restore_pos(state: *mut MatchState, saved: *const RegSave) {
    if state.is_null() || saved.is_null() {
        return;
    }
    (*state).restore_pos(&*saved);
}

/// Get backpos table from match state.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_get_backpos(state: *mut MatchState) -> *mut BackPosTable {
    if state.is_null() {
        ptr::null_mut()
    } else {
        &mut (*state).backpos as *mut BackPosTable
    }
}

/// Add an entry to the backpos table.
///
/// # Safety
/// `table` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_backpos_add(table: *mut BackPosTable, scan: *mut u8, pos: RegSave) {
    if !table.is_null() {
        (*table).push(scan, pos);
    }
}

/// Find an entry in the backpos table by scan position.
///
/// # Safety
/// `table` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_backpos_find(
    table: *const BackPosTable,
    scan: *const u8,
    out: *mut RegSave,
) -> c_int {
    if table.is_null() || out.is_null() {
        return 0;
    }
    match (*table).find(scan) {
        Some(bp) => {
            *out = bp.bp_pos;
            1
        }
        None => 0,
    }
}

/// Get the stack from match state.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_get_stack(state: *mut MatchState) -> *mut RegStack {
    if state.is_null() {
        ptr::null_mut()
    } else {
        &mut (*state).stack as *mut RegStack
    }
}

/// Push a star (repetition) state onto the stack.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_push_star(
    state: *mut MatchState,
    count: i64,
    minval: i64,
    maxval: i64,
) -> c_int {
    if state.is_null() {
        return 0;
    }
    let star = RegStar {
        count,
        minval,
        maxval,
    };
    c_int::from((*state).stack.push_star(star))
}

/// Pop a star state from the stack.
///
/// Returns the count, or -1 if empty.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_pop_star(
    state: *mut MatchState,
    out_count: *mut i64,
    out_minval: *mut i64,
    out_maxval: *mut i64,
) -> c_int {
    if state.is_null() {
        return 0;
    }
    match (*state).stack.pop_star() {
        Some(star) => {
            if !out_count.is_null() {
                *out_count = star.count;
            }
            if !out_minval.is_null() {
                *out_minval = star.minval;
            }
            if !out_maxval.is_null() {
                *out_maxval = star.maxval;
            }
            1
        }
        None => 0,
    }
}

/// Set the match_nl flag (whether '.' matches newline).
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_set_match_nl(state: *mut MatchState, match_nl: c_int) {
    if !state.is_null() {
        (*state).match_nl = match_nl != 0;
    }
}

/// Get whether we're in multi-line mode.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_is_multi(state: *const MatchState) -> c_int {
    if state.is_null() {
        0
    } else {
        c_int::from((*state).multi)
    }
}

/// Get current line number (multi-line mode).
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_get_lnum(state: *const MatchState) -> c_int {
    if state.is_null() {
        0
    } else {
        (*state).lnum
    }
}

/// Get current column (multi-line mode).
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_get_col(state: *const MatchState) -> c_int {
    if state.is_null() {
        0
    } else {
        (*state).col
    }
}

// =============================================================================
// Tests
// =============================================================================

// =============================================================================
// Additional BT Execution FFI Exports (Phase R4)
// =============================================================================

/// Advance input by n bytes.
///
/// # Safety
/// `state` must be valid and input must have at least n more bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_advance_by(state: *mut MatchState, n: c_int) {
    if !state.is_null() && n > 0 {
        (*state).advance_input_by(n as usize);
    }
}

/// Get the previous byte (before current position).
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_state_prev_byte(state: *const MatchState) -> u8 {
    if state.is_null() {
        0
    } else {
        (*state).prev_byte()
    }
}

/// Clear the backtrack stack.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_clear_backtrack(state: *mut MatchState) {
    if !state.is_null() {
        (*state).clear_backtrack();
    }
}

/// Clear the backpos table.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_clear_backpos(state: *mut MatchState) {
    if !state.is_null() {
        (*state).clear_backpos();
    }
}

/// Get the current input pointer.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_get_input(state: *const MatchState) -> *const u8 {
    if state.is_null() {
        std::ptr::null()
    } else {
        (*state).input()
    }
}

/// Set the line start pointer.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_set_line_start(state: *mut MatchState, start: *const u8) {
    if !state.is_null() {
        (*state).set_line_start(start);
    }
}

/// Check if a byte is a word character (alphanumeric or underscore).
#[no_mangle]
pub extern "C" fn rs_bt_is_word_char(c: u8) -> c_int {
    c_int::from(is_word_char(c))
}

/// Check if a byte is a filename character.
#[no_mangle]
pub extern "C" fn rs_bt_is_fname_char(c: u8) -> c_int {
    c_int::from(is_fname_char(c))
}

/// Get UTF-8 character length from first byte.
#[no_mangle]
pub extern "C" fn rs_bt_utf8_char_len(b: u8) -> c_int {
    utf8_char_len(b) as c_int
}

/// Match a character against a character class.
///
/// # Safety
/// `class_start` must point to valid NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_match_class(class_start: *const u8, c: u8) -> c_int {
    c_int::from(match_class(class_start, c))
}

/// Compare position values for line/column matching.
#[no_mangle]
pub extern "C" fn rs_bt_compare_pos(actual: i64, limit: i64, cmp_op: u8) -> c_int {
    c_int::from(compare_pos(actual, limit, cmp_op))
}

/// Set the lnum field (for multi-line mode).
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_set_lnum(state: *mut MatchState, lnum: c_int) {
    if !state.is_null() {
        (*state).lnum = lnum;
    }
}

/// Set the col field (for multi-line mode).
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_set_col(state: *mut MatchState, col: c_int) {
    if !state.is_null() {
        (*state).col = col;
    }
}

/// Get the BRACE_LIMITS min value.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_get_bl_minval(state: *const MatchState) -> i64 {
    if state.is_null() {
        0
    } else {
        (*state).bl_minval
    }
}

/// Get the BRACE_LIMITS max value.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_get_bl_maxval(state: *const MatchState) -> i64 {
    if state.is_null() {
        i64::MAX
    } else {
        (*state).bl_maxval
    }
}

/// Set the BRACE_LIMITS values.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_set_brace_limits(state: *mut MatchState, minval: i64, maxval: i64) {
    if !state.is_null() {
        (*state).bl_minval = minval;
        (*state).bl_maxval = maxval;
    }
}

/// Get a brace min value by index.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_get_brace_min(state: *const MatchState, idx: c_int) -> i64 {
    if state.is_null() || idx < 0 || idx as usize >= NSUBEXP {
        0
    } else {
        (*state).brace_min[idx as usize]
    }
}

/// Get a brace max value by index.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_get_brace_max(state: *const MatchState, idx: c_int) -> i64 {
    if state.is_null() || idx < 0 || idx as usize >= NSUBEXP {
        i64::MAX
    } else {
        (*state).brace_max[idx as usize]
    }
}

/// Get a brace count value by index.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_get_brace_count(state: *const MatchState, idx: c_int) -> i64 {
    if state.is_null() || idx < 0 || idx as usize >= NSUBEXP {
        0
    } else {
        (*state).brace_count[idx as usize]
    }
}

/// Set a brace min value by index.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_set_brace_min(state: *mut MatchState, idx: c_int, val: i64) {
    if !state.is_null() && idx >= 0 && (idx as usize) < NSUBEXP {
        (*state).brace_min[idx as usize] = val;
    }
}

/// Set a brace max value by index.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_set_brace_max(state: *mut MatchState, idx: c_int, val: i64) {
    if !state.is_null() && idx >= 0 && (idx as usize) < NSUBEXP {
        (*state).brace_max[idx as usize] = val;
    }
}

/// Set a brace count value by index.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_set_brace_count(state: *mut MatchState, idx: c_int, val: i64) {
    if !state.is_null() && idx >= 0 && (idx as usize) < NSUBEXP {
        (*state).brace_count[idx as usize] = val;
    }
}

/// Increment a brace count value by index, returning the new value.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_incr_brace_count(state: *mut MatchState, idx: c_int) -> i64 {
    if state.is_null() || idx < 0 || idx as usize >= NSUBEXP {
        0
    } else {
        (*state).brace_count[idx as usize] += 1;
        (*state).brace_count[idx as usize]
    }
}

/// Get the match result as an integer (Match=1, NoMatch=0, Error=-1, TimedOut=-2).
#[no_mangle]
pub extern "C" fn rs_bt_match_result_to_int(result: c_int) -> c_int {
    match result {
        1 => 1,  // Match
        0 => 0,  // NoMatch
        -1 => -1, // Error
        _ => -2,  // TimedOut or unknown
    }
}

/// Check if match result indicates success.
#[no_mangle]
pub extern "C" fn rs_bt_match_result_is_match(result: c_int) -> c_int {
    c_int::from(result == 1)
}

/// Check if match result indicates error.
#[no_mangle]
pub extern "C" fn rs_bt_match_result_is_error(result: c_int) -> c_int {
    c_int::from(result < 0)
}

/// Get the size of RegSave structure for FFI.
#[no_mangle]
pub extern "C" fn rs_bt_regsave_size() -> usize {
    std::mem::size_of::<RegSave>()
}

/// Get the number of subexpressions constant.
#[no_mangle]
pub extern "C" fn rs_bt_get_nsubexp() -> c_int {
    NSUBEXP as c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_result() {
        assert!(MatchResult::Match.is_match());
        assert!(!MatchResult::NoMatch.is_match());
        assert!(!MatchResult::Error.is_match());
        assert!(!MatchResult::TimedOut.is_match());
    }

    #[test]
    fn test_match_state_single() {
        let input = b"hello\0";
        let program = [0u8; 16];
        let state = MatchState::new_single(input.as_ptr(), program.as_ptr());

        assert!(!state.multi);
        assert_eq!(state.input, input.as_ptr());
        assert_eq!(state.line_start, input.as_ptr());
    }

    #[test]
    fn test_match_state_multi() {
        let program = [0u8; 16];
        let state = MatchState::new_multi(program.as_ptr(), 5, 10);

        assert!(state.multi);
        assert_eq!(state.lnum, 5);
        assert_eq!(state.col, 10);
    }

    #[test]
    fn test_match_state_submatches() {
        let input = b"test\0";
        let program = [0u8; 16];
        let mut state = MatchState::new_single(input.as_ptr(), program.as_ptr());

        // Set and get submatches
        state.set_startp(0, input.as_ptr());
        state.set_endp(0, unsafe { input.as_ptr().add(4) });

        assert_eq!(state.get_startp(0), input.as_ptr());
        assert_eq!(state.get_endp(0), unsafe { input.as_ptr().add(4) });

        // Out of bounds access returns null
        assert!(state.get_startp(NSUBEXP + 1).is_null());
        assert!(state.get_endp(NSUBEXP + 1).is_null());

        // Clear submatches
        state.clear_submatches();
        assert!(state.get_startp(0).is_null());
        assert!(state.get_endp(0).is_null());
    }

    #[test]
    fn test_match_state_position() {
        let input = b"hello\0";
        let program = [0u8; 16];
        let mut state = MatchState::new_single(input.as_ptr(), program.as_ptr());

        assert!(state.at_bol());

        unsafe {
            assert_eq!(state.current_byte(), b'h');

            state.advance_input();
            assert_eq!(state.current_byte(), b'e');
            assert!(!state.at_bol());

            state.advance_input_by(3);
            assert_eq!(state.current_byte(), b'o');

            state.advance_input();
            assert!(state.at_eol()); // at null terminator
        }
    }

    #[test]
    fn test_save_restore_position() {
        let input = b"hello\0";
        let program = [0u8; 16];
        let mut state = MatchState::new_single(input.as_ptr(), program.as_ptr());

        // Save initial position
        let saved = state.save_pos();

        unsafe {
            // Advance
            state.advance_input_by(3);
            assert_eq!(state.current_byte(), b'l');

            // Restore
            state.restore_pos(&saved);
            assert_eq!(state.current_byte(), b'h');
        }
    }

    #[test]
    fn test_backtrack_stack() {
        let input = b"test\0";
        let program = [0u8; 16];
        let mut state = MatchState::new_single(input.as_ptr(), program.as_ptr());

        assert!(state.backtrack_empty());

        // Push items
        let scan1 = program.as_ptr() as *mut u8;
        let scan2 = unsafe { program.as_ptr().add(3) as *mut u8 };

        assert!(state.push_backtrack(RegState::Branch, scan1));
        assert!(state.push_backtrack(RegState::StarLong, scan2));
        assert!(!state.backtrack_empty());

        // Pop items (LIFO order)
        let popped1 = state.pop_backtrack();
        assert_eq!(popped1, Some(scan2));

        let popped2 = state.pop_backtrack();
        assert_eq!(popped2, Some(scan1));

        assert!(state.backtrack_empty());
        assert!(state.pop_backtrack().is_none());
    }

    #[test]
    fn test_match_class() {
        let class = b"abc\0";
        unsafe {
            assert!(match_class(class.as_ptr(), b'a'));
            assert!(match_class(class.as_ptr(), b'b'));
            assert!(match_class(class.as_ptr(), b'c'));
            assert!(!match_class(class.as_ptr(), b'd'));
            assert!(!match_class(class.as_ptr(), b'A'));
        }
    }
}
