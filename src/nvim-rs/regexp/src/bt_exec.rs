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

use std::ffi::{c_int, c_void};
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
use crate::bt_state::{
    BackPosTable, RegBehind, RegItem, RegSave, RegStack, RegStar, RegState, NSUBEXP,
};
use crate::rs_cstrncmp;

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
    ///
    /// Returns (state, scan pointer) from the popped item.
    pub fn pop_backtrack(&mut self) -> Option<(RegState, *mut u8)> {
        self.stack.pop_item()
    }

    /// Pop a full backtrack item from the stack (includes all fields).
    pub fn pop_backtrack_full(&mut self) -> Option<RegItem> {
        self.stack.pop_item_full()
    }

    /// Peek at the top backtrack item without popping.
    pub fn peek_backtrack(&self) -> Option<&RegItem> {
        self.stack.peek_item()
    }

    /// Peek at the top backtrack item mutably.
    pub fn peek_backtrack_mut(&mut self) -> Option<&mut RegItem> {
        self.stack.peek_item_mut()
    }

    /// Push a RegStar onto the stack (for STAR/PLUS/BRACE_SIMPLE).
    pub fn push_star(&mut self, star: RegStar) -> bool {
        self.stack.push_star(star)
    }

    /// Pop a RegStar from the stack.
    pub fn pop_star(&mut self) -> Option<RegStar> {
        self.stack.pop_star()
    }

    /// Push a RegBehind onto the stack (for BEHIND/NOBEHIND).
    pub fn push_behind(&mut self, behind: RegBehind) -> bool {
        self.stack.push_behind(behind)
    }

    /// Pop a RegBehind from the stack.
    pub fn pop_behind(&mut self) -> Option<RegBehind> {
        self.stack.pop_behind()
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
/// This implements a simplified version that handles the basic states.
/// The full implementation with all state handlers is in the C regmatch().
///
/// # Safety
/// state must be valid with valid program pointer.
pub unsafe fn regmatch(state: &mut MatchState, program: *const u8) -> MatchResult {
    if program.is_null() {
        return MatchResult::Error;
    }

    // Clear stacks at start
    state.clear_backtrack();
    state.clear_backpos();

    // Start at the first instruction (skip REGMAGIC byte)
    let mut scan = program.add(1);
    let mut status;

    // Outer loop: continue until regstack is empty or we have a final result
    loop {
        // Inner loop: match opcodes sequentially until we need to backtrack
        loop {
            if scan.is_null() {
                status = MatchStatus::Fail;
                break;
            }

            let opcode = op(scan);

            // Check for END first
            if opcode == END {
                status = MatchStatus::Match;
                break;
            }

            let next_scan = next(scan);
            status = match_one_op(state, scan, opcode);

            match status {
                MatchStatus::Continue => {
                    // Move to next instruction
                    scan = if next_scan.is_null() {
                        scan.add(3) // Default: advance past node
                    } else {
                        next_scan
                    };
                }
                MatchStatus::Match | MatchStatus::NoMatch | MatchStatus::Fail => {
                    // Break inner loop to handle state
                    break;
                }
                MatchStatus::Break => {
                    // RA_BREAK: skip to state handling (used after pushing for BRANCH/STAR)
                    break;
                }
            }
        }

        // Process backtrack states
        while !state.backtrack_empty() && status != MatchStatus::Fail {
            if let Some(rp) = state.peek_backtrack() {
                let rs_state = rp.state();
                let rs_scan = rp.rs_scan;

                match rs_state {
                    RegState::Nopen => {
                        // RS_NOPEN: Result is passed on as-is, simply pop
                        state.pop_backtrack();
                        scan = rs_scan;
                    }

                    RegState::Mopen => {
                        // RS_MOPEN: Pop, restore startp on no match
                        // Note: Full implementation would restore saved position
                        state.pop_backtrack();
                        scan = rs_scan;
                    }

                    RegState::Mclose => {
                        // RS_MCLOSE: Pop, restore endp on no match
                        state.pop_backtrack();
                        scan = rs_scan;
                    }

                    RegState::Branch => {
                        // RS_BRANCH: Handle alternation
                        if status == MatchStatus::Match {
                            // This branch matched, use it
                            state.pop_backtrack();
                            scan = rs_scan;
                        } else {
                            // Try next branch
                            let branch_scan = rs_scan;
                            if branch_scan.is_null() || op(branch_scan) != BRANCH {
                                // No more branches
                                status = MatchStatus::NoMatch;
                                state.pop_backtrack();
                                scan = rs_scan;
                            } else {
                                // Prepare to try next branch
                                if let Some(rp_mut) = state.peek_backtrack_mut() {
                                    rp_mut.rs_scan = next(branch_scan) as *mut u8;
                                }
                                scan = operand(branch_scan);
                                status = MatchStatus::Continue;
                            }
                        }
                    }

                    // For complex states, fall back to simple behavior
                    RegState::Zopen
                    | RegState::Zclose
                    | RegState::BrcplxMore
                    | RegState::BrcplxLong
                    | RegState::BrcplxShort
                    | RegState::Nomatch
                    | RegState::Behind1
                    | RegState::Behind2
                    | RegState::StarLong
                    | RegState::StarShort => {
                        // Pop and continue (simplified handling)
                        state.pop_backtrack();
                        scan = rs_scan;
                    }
                }
            } else {
                break;
            }

            // If we want to continue the inner loop, break out of state handling
            if status == MatchStatus::Continue {
                break;
            }
        }

        // Check for final result
        if status == MatchStatus::Match {
            return MatchResult::Match;
        }
        if status == MatchStatus::Fail {
            return MatchResult::Error;
        }
        if state.backtrack_empty() {
            return if status == MatchStatus::Match {
                MatchResult::Match
            } else {
                MatchResult::NoMatch
            };
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
/// Returns the scan pointer from the popped item (state is discarded).
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_pop_backtrack(state: *mut MatchState) -> *mut u8 {
    if state.is_null() {
        ptr::null_mut()
    } else {
        (*state)
            .pop_backtrack()
            .map(|(_state, scan)| scan)
            .unwrap_or(ptr::null_mut())
    }
}

/// Pop a backtrack item with state.
///
/// Returns the state type via `out_state` and the scan pointer as return value.
///
/// # Safety
/// `state` and `out_state` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_pop_backtrack_with_state(
    state: *mut MatchState,
    out_state: *mut c_int,
) -> *mut u8 {
    if state.is_null() || out_state.is_null() {
        return ptr::null_mut();
    }
    match (*state).pop_backtrack() {
        Some((rs_state, scan)) => {
            *out_state = rs_state as c_int;
            scan
        }
        None => {
            *out_state = RegState::Nopen as c_int;
            ptr::null_mut()
        }
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
        1 => 1,   // Match
        0 => 0,   // NoMatch
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

// =============================================================================
// FFI for regrepeat
// =============================================================================

use std::ffi::c_char;

extern "C" {
    // Rex state accessors
    fn nvim_rex_get_input() -> *mut u8;
    fn nvim_rex_set_input(input: *mut u8);
    fn nvim_rex_get_lnum() -> c_int;
    fn nvim_rex_get_reg_maxline() -> c_int;
    fn nvim_rex_get_reg_line_lbr() -> bool;
    fn nvim_rex_is_multi() -> c_int;
    fn nvim_rex_get_reg_ic() -> bool;
    fn nvim_rex_get_reg_buf() -> *mut std::ffi::c_void;
    fn nvim_get_got_int() -> c_int;

    // Character classification functions
    fn vim_isIDc(c: c_int) -> c_int;
    fn vim_iswordp_buf(p: *const u8, buf: *mut std::ffi::c_void) -> c_int;
    fn vim_isfilec(c: c_int) -> c_int;
    fn vim_isprintc(c: c_int) -> c_int;

    // UTF-8 functions
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utf_fold(c: c_int) -> c_int;
    fn mb_toupper(c: c_int) -> c_int;
    fn mb_tolower(c: c_int) -> c_int;

    // String search - use Rust helper
    fn rs_cstrchr(s: *const c_char, c: c_int) -> *mut c_char;

    // Next line - use Rust helper
    fn rs_reg_nextline();
}

/// Character class flags for the class table (matching C's RI_* constants)
const RI_DIGIT: i16 = 0x01;
const RI_HEX: i16 = 0x02;
const RI_OCTAL: i16 = 0x04;
const RI_WORD: i16 = 0x08;
const RI_HEAD: i16 = 0x10;
const RI_ALPHA: i16 = 0x20;
const RI_LOWER: i16 = 0x40;
const RI_UPPER: i16 = 0x80;
const RI_WHITE: i16 = 0x100;

/// Lazily initialized character class table
static CLASS_TAB: std::sync::OnceLock<[i16; 256]> = std::sync::OnceLock::new();

fn init_class_tab_local() -> [i16; 256] {
    let mut tab = [0i16; 256];
    for (i, entry) in tab.iter_mut().enumerate() {
        *entry = match i as u8 {
            b'0'..=b'7' => RI_DIGIT | RI_HEX | RI_OCTAL | RI_WORD,
            b'8'..=b'9' => RI_DIGIT | RI_HEX | RI_WORD,
            b'a'..=b'f' => RI_HEX | RI_WORD | RI_HEAD | RI_ALPHA | RI_LOWER,
            b'g'..=b'z' => RI_WORD | RI_HEAD | RI_ALPHA | RI_LOWER,
            b'A'..=b'F' => RI_HEX | RI_WORD | RI_HEAD | RI_ALPHA | RI_UPPER,
            b'G'..=b'Z' => RI_WORD | RI_HEAD | RI_ALPHA | RI_UPPER,
            b'_' => RI_WORD | RI_HEAD,
            b' ' | b'\t' => RI_WHITE,
            _ => 0,
        };
    }
    tab
}

#[inline]
fn class_tab_local() -> &'static [i16; 256] {
    CLASS_TAB.get_or_init(init_class_tab_local)
}

/// Check if opcode includes ADD_NL for newline matching.
/// This checks if the opcode is in the range [FIRST_NL, LAST_NL].
#[inline]
fn with_nl(opcode: c_int) -> bool {
    (FIRST_NL..=LAST_NL).contains(&opcode)
}

/// Get the base opcode (without ADD_NL offset).
#[inline]
fn base_opcode(opcode: c_int) -> c_int {
    if with_nl(opcode) {
        opcode - ADD_NL
    } else {
        opcode
    }
}

/// Advance scan pointer by one multi-byte character.
/// Returns the new pointer position.
#[inline]
unsafe fn mb_ptr_adv(scan: *mut u8) -> *mut u8 {
    let len = utfc_ptr2len(scan as *const c_char);
    if len > 0 {
        scan.add(len as usize)
    } else {
        scan.add(1)
    }
}

/// Count the number of times a simple regexp atom matches.
///
/// This is the full implementation matching the C regrepeat() function.
/// It handles:
/// - Multi-line matching (REG_MULTI, WITH_NL)
/// - Multi-byte characters (UTF-8)
/// - Case-insensitive matching
/// - Buffer-local 'iskeyword'
/// - Interrupt checking (got_int)
///
/// # Safety
/// `p` must point to valid BT regex bytecode.
#[no_mangle]
pub unsafe extern "C" fn rs_regrepeat(p: *mut u8, maxcount: i64) -> c_int {
    let mut count: i64 = 0;
    let mut scan = nvim_rex_get_input();
    let opnd = operand(p);
    let opcode = op(p);

    // Check for invalid input
    if p.is_null() || scan.is_null() {
        return 0;
    }

    // Helper closures for checking conditions
    let is_multi = nvim_rex_is_multi() != 0;
    let line_lbr = nvim_rex_get_reg_line_lbr();
    let maxline = nvim_rex_get_reg_maxline();
    let lnum = || nvim_rex_get_lnum();
    let got_int = || nvim_get_got_int() != 0;

    // Handle each opcode case - use base_opcode to strip ADD_NL if present
    match base_opcode(opcode) {
        ANY => {
            // ANY matches any character (except newline, unless ADD_NL)
            while count < maxcount {
                while *scan != 0 && count < maxcount {
                    count += 1;
                    scan = mb_ptr_adv(scan);
                }
                if !is_multi
                    || !with_nl(opcode)
                    || lnum() > maxline
                    || line_lbr
                    || count == maxcount
                {
                    break;
                }
                count += 1; // count the line-break
                rs_reg_nextline();
                scan = nvim_rex_get_input();
                if got_int() {
                    break;
                }
            }
        }

        IDENT => {
            // IDENT: identifier character (vim_isIDc)
            let include_digit = true;
            while count < maxcount {
                let c = utf_ptr2char(scan as *const c_char);
                if vim_isIDc(c) != 0 && (include_digit || !(*scan).is_ascii_digit()) {
                    scan = mb_ptr_adv(scan);
                } else if *scan == 0 {
                    if !is_multi || !with_nl(opcode) || lnum() > maxline || line_lbr {
                        break;
                    }
                    rs_reg_nextline();
                    scan = nvim_rex_get_input();
                    if got_int() {
                        break;
                    }
                } else if line_lbr && *scan == b'\n' && with_nl(opcode) {
                    scan = scan.add(1);
                } else {
                    break;
                }
                count += 1;
            }
        }

        SIDENT => {
            // SIDENT: identifier character excluding leading digit
            while count < maxcount {
                let c = utf_ptr2char(scan as *const c_char);
                if vim_isIDc(c) != 0 && !(*scan).is_ascii_digit() {
                    scan = mb_ptr_adv(scan);
                } else if *scan == 0 {
                    if !is_multi || !with_nl(opcode) || lnum() > maxline || line_lbr {
                        break;
                    }
                    rs_reg_nextline();
                    scan = nvim_rex_get_input();
                    if got_int() {
                        break;
                    }
                } else if line_lbr && *scan == b'\n' && with_nl(opcode) {
                    scan = scan.add(1);
                } else {
                    break;
                }
                count += 1;
            }
        }

        KWORD => {
            // KWORD: keyword character (vim_iswordp_buf)
            let buf = nvim_rex_get_reg_buf();
            let include_digit = true;
            while count < maxcount {
                if vim_iswordp_buf(scan, buf) != 0 && (include_digit || !(*scan).is_ascii_digit()) {
                    scan = mb_ptr_adv(scan);
                } else if *scan == 0 {
                    if !is_multi || !with_nl(opcode) || lnum() > maxline || line_lbr {
                        break;
                    }
                    rs_reg_nextline();
                    scan = nvim_rex_get_input();
                    if got_int() {
                        break;
                    }
                } else if line_lbr && *scan == b'\n' && with_nl(opcode) {
                    scan = scan.add(1);
                } else {
                    break;
                }
                count += 1;
            }
        }

        SKWORD => {
            // SKWORD: keyword character excluding leading digit
            let buf = nvim_rex_get_reg_buf();
            while count < maxcount {
                if vim_iswordp_buf(scan, buf) != 0 && !(*scan).is_ascii_digit() {
                    scan = mb_ptr_adv(scan);
                } else if *scan == 0 {
                    if !is_multi || !with_nl(opcode) || lnum() > maxline || line_lbr {
                        break;
                    }
                    rs_reg_nextline();
                    scan = nvim_rex_get_input();
                    if got_int() {
                        break;
                    }
                } else if line_lbr && *scan == b'\n' && with_nl(opcode) {
                    scan = scan.add(1);
                } else {
                    break;
                }
                count += 1;
            }
        }

        FNAME => {
            // FNAME: filename character (vim_isfilec)
            let include_digit = true;
            while count < maxcount {
                let c = utf_ptr2char(scan as *const c_char);
                if vim_isfilec(c) != 0 && (include_digit || !(*scan).is_ascii_digit()) {
                    scan = mb_ptr_adv(scan);
                } else if *scan == 0 {
                    if !is_multi || !with_nl(opcode) || lnum() > maxline || line_lbr {
                        break;
                    }
                    rs_reg_nextline();
                    scan = nvim_rex_get_input();
                    if got_int() {
                        break;
                    }
                } else if line_lbr && *scan == b'\n' && with_nl(opcode) {
                    scan = scan.add(1);
                } else {
                    break;
                }
                count += 1;
            }
        }

        SFNAME => {
            // SFNAME: filename character excluding leading digit
            while count < maxcount {
                let c = utf_ptr2char(scan as *const c_char);
                if vim_isfilec(c) != 0 && !(*scan).is_ascii_digit() {
                    scan = mb_ptr_adv(scan);
                } else if *scan == 0 {
                    if !is_multi || !with_nl(opcode) || lnum() > maxline || line_lbr {
                        break;
                    }
                    rs_reg_nextline();
                    scan = nvim_rex_get_input();
                    if got_int() {
                        break;
                    }
                } else if line_lbr && *scan == b'\n' && with_nl(opcode) {
                    scan = scan.add(1);
                } else {
                    break;
                }
                count += 1;
            }
        }

        PRINT => {
            // PRINT: printable character (vim_isprintc)
            let include_digit = true;
            while count < maxcount {
                if *scan == 0 {
                    if !is_multi || !with_nl(opcode) || lnum() > maxline || line_lbr {
                        break;
                    }
                    rs_reg_nextline();
                    scan = nvim_rex_get_input();
                    if got_int() {
                        break;
                    }
                } else {
                    let c = utf_ptr2char(scan as *const c_char);
                    if vim_isprintc(c) == 1 && (include_digit || !(*scan).is_ascii_digit()) {
                        scan = mb_ptr_adv(scan);
                    } else if line_lbr && *scan == b'\n' && with_nl(opcode) {
                        scan = scan.add(1);
                    } else {
                        break;
                    }
                }
                count += 1;
            }
        }

        SPRINT => {
            // SPRINT: printable character excluding leading digit
            while count < maxcount {
                if *scan == 0 {
                    if !is_multi || !with_nl(opcode) || lnum() > maxline || line_lbr {
                        break;
                    }
                    rs_reg_nextline();
                    scan = nvim_rex_get_input();
                    if got_int() {
                        break;
                    }
                } else {
                    let c = utf_ptr2char(scan as *const c_char);
                    if vim_isprintc(c) == 1 && !(*scan).is_ascii_digit() {
                        scan = mb_ptr_adv(scan);
                    } else if line_lbr && *scan == b'\n' && with_nl(opcode) {
                        scan = scan.add(1);
                    } else {
                        break;
                    }
                }
                count += 1;
            }
        }

        // Character class matching using class_tab
        WHITE | NWHITE | DIGIT | NDIGIT | HEX | NHEX | OCTAL | NOCTAL | WORD | NWORD | HEAD
        | NHEAD | ALPHA | NALPHA | LOWER | NLOWER | UPPER | NUPPER => {
            let (testval, mask) = match base_opcode(opcode) {
                WHITE => (RI_WHITE, RI_WHITE),
                NWHITE => (0, RI_WHITE),
                DIGIT => (RI_DIGIT, RI_DIGIT),
                NDIGIT => (0, RI_DIGIT),
                HEX => (RI_HEX, RI_HEX),
                NHEX => (0, RI_HEX),
                OCTAL => (RI_OCTAL, RI_OCTAL),
                NOCTAL => (0, RI_OCTAL),
                WORD => (RI_WORD, RI_WORD),
                NWORD => (0, RI_WORD),
                HEAD => (RI_HEAD, RI_HEAD),
                NHEAD => (0, RI_HEAD),
                ALPHA => (RI_ALPHA, RI_ALPHA),
                NALPHA => (0, RI_ALPHA),
                LOWER => (RI_LOWER, RI_LOWER),
                NLOWER => (0, RI_LOWER),
                UPPER => (RI_UPPER, RI_UPPER),
                NUPPER => (0, RI_UPPER),
                _ => (0, 0),
            };

            let tab = class_tab_local();
            while count < maxcount {
                if *scan == 0 {
                    if !is_multi || !with_nl(opcode) || lnum() > maxline || line_lbr {
                        break;
                    }
                    rs_reg_nextline();
                    scan = nvim_rex_get_input();
                    if got_int() {
                        break;
                    }
                } else {
                    let len = utfc_ptr2len(scan as *const c_char);
                    if len > 1 {
                        // Multi-byte character
                        if testval != 0 {
                            break; // Multi-byte doesn't match single-byte class
                        }
                        scan = scan.add(len as usize);
                    } else if (tab[*scan as usize] & mask) == testval
                        || (line_lbr && *scan == b'\n' && with_nl(opcode))
                    {
                        scan = scan.add(1);
                    } else {
                        break;
                    }
                }
                count += 1;
            }
        }

        EXACTLY => {
            // EXACTLY: match exact character
            let reg_ic = nvim_rex_get_reg_ic();
            if reg_ic {
                let cu = mb_toupper(*opnd as c_int);
                let cl = mb_tolower(*opnd as c_int);
                while count < maxcount && (*scan as c_int == cu || *scan as c_int == cl) {
                    count += 1;
                    scan = scan.add(1);
                }
            } else {
                let cu = *opnd;
                while count < maxcount && *scan == cu {
                    count += 1;
                    scan = scan.add(1);
                }
            }
        }

        MULTIBYTECODE => {
            // MULTIBYTECODE: match multi-byte character
            let len = utfc_ptr2len(opnd as *const c_char);
            if len > 1 {
                let reg_ic = nvim_rex_get_reg_ic();
                let cf = if reg_ic {
                    utf_fold(utf_ptr2char(opnd as *const c_char))
                } else {
                    0
                };

                while count < maxcount && utfc_ptr2len(scan as *const c_char) >= len {
                    // Compare byte by byte
                    let mut i = 0;
                    while i < len {
                        if *opnd.add(i as usize) != *scan.add(i as usize) {
                            break;
                        }
                        i += 1;
                    }
                    if i < len && (!reg_ic || utf_fold(utf_ptr2char(scan as *const c_char)) != cf) {
                        break;
                    }
                    scan = scan.add(len as usize);
                    count += 1;
                }
            }
        }

        ANYOF | ANYBUT => {
            // ANYOF/ANYBUT: match character in/not in set
            let is_anyof = (base_opcode(opcode)) == ANYOF;
            let testval = if is_anyof { 1 } else { 0 };

            while count < maxcount {
                if *scan == 0 {
                    if !is_multi || !with_nl(opcode) || lnum() > maxline || line_lbr {
                        break;
                    }
                    rs_reg_nextline();
                    scan = nvim_rex_get_input();
                    if got_int() {
                        break;
                    }
                } else if line_lbr && *scan == b'\n' && with_nl(opcode) {
                    scan = scan.add(1);
                } else {
                    let len = utfc_ptr2len(scan as *const c_char);
                    if len > 1 {
                        // Multi-byte character
                        let c = utf_ptr2char(scan as *const c_char);
                        let found = !rs_cstrchr(opnd as *const c_char, c).is_null();
                        if found as i32 == testval {
                            break;
                        }
                        scan = scan.add(len as usize);
                    } else {
                        let found = !rs_cstrchr(opnd as *const c_char, *scan as c_int).is_null();
                        if found as i32 == testval {
                            break;
                        }
                        scan = scan.add(1);
                    }
                }
                count += 1;
            }
        }

        NEWL => {
            // NEWL: match newline
            while count < maxcount
                && ((*scan == 0 && lnum() <= maxline && !line_lbr && is_multi)
                    || (*scan == b'\n' && line_lbr))
            {
                count += 1;
                if line_lbr {
                    // Advance rex.input by one byte (the newline char)
                    let new_input = scan.add(1);
                    nvim_rex_set_input(new_input);
                } else {
                    rs_reg_nextline();
                }
                scan = nvim_rex_get_input();
                if got_int() {
                    break;
                }
            }
        }

        _ => {
            // Unknown opcode - this shouldn't happen
            // In C this calls iemsg(), we just return 0
            return 0;
        }
    }

    // Update rex.input with final position
    nvim_rex_set_input(scan);

    count as c_int
}

// Import opcodes which may not be in scope
use crate::bt_opcodes::{FIRST_NL, KWORD, LAST_NL, NOCTAL, SIDENT};

// =============================================================================
// BT Execution Entry Points (Phase 13b)
// =============================================================================

/// LPos structure for multi-line position (matches C lpos_T)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
struct LPosBt {
    lnum: c_int,
    col: c_int,
}

/// Opaque type for reg_extmatch_T*
type ExtmatchHandle = *mut std::ffi::c_void;

/// Opaque type for proftime_T*
type ProfTime = *mut std::ffi::c_void;

/// REX_SET constant - indicates \z(...) is used
const REX_SET: c_int = 2;

extern "C" {
    // Additional rex state accessors
    fn nvim_rex_get_line() -> *mut u8;
    fn nvim_rex_set_need_clear_subexpr(v: c_int);
    fn nvim_rex_set_need_clear_zsubexpr(v: c_int);
    fn nvim_rex_get_reg_startp() -> *mut *mut u8;
    fn nvim_rex_get_reg_endp() -> *mut *mut u8;
    // Use void pointers to avoid redeclaration conflicts - cast to LPosBt internally
    fn nvim_rex_get_reg_startpos() -> *mut c_void;
    fn nvim_rex_get_reg_endpos() -> *mut c_void;
    fn nvim_rex_set_lnum(lnum: c_int);

    // BT regprog accessors
    fn nvim_bt_regprog_get_reghasz(prog: *const c_void) -> c_int;
    fn nvim_bt_regprog_get_program(prog: *const c_void) -> *const u8;

    // Subexpr cleanup
    fn nvim_cleanup_subexpr();
    fn nvim_cleanup_zsubexpr();

    // Z-subexpr accessors - use void pointers and cast
    fn nvim_get_reg_startzpos() -> *mut c_void;
    fn nvim_get_reg_endzpos() -> *mut c_void;
    fn nvim_get_reg_startzp() -> *mut *mut u8;
    fn nvim_get_reg_endzp() -> *mut *mut u8;

    // Extmatch handling
    fn nvim_make_extmatch() -> ExtmatchHandle;
    fn nvim_unref_extmatch(em: ExtmatchHandle);
    fn nvim_get_re_extmatch_out() -> ExtmatchHandle;
    fn nvim_set_re_extmatch_out(em: ExtmatchHandle);
    fn nvim_extmatch_set_match(em: ExtmatchHandle, idx: c_int, match_str: *mut u8);

    // Line fetching
    fn nvim_reg_getline(lnum: c_int) -> *mut c_char;

    // Memory allocation for extmatch strings
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // The C regmatch wrapper function (calls static regmatch)
    fn nvim_bt_regmatch(scan: *mut u8, tm: ProfTime, timed_out: *mut c_int) -> c_int;
}

/// Try match of BT program at rex.line[col].
///
/// This implements the BT version of regtry(), setting up state and calling
/// the existing C regmatch() function.
///
/// @param prog      bt_regprog_T* pointer
/// @param col       column to start matching at
/// @param tm        timeout limit or NULL
/// @param timed_out flag set on timeout or NULL
///
/// @return 0 for failure, or number of lines contained in the match.
///
/// # Safety
/// prog must be a valid bt_regprog_T*. All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_regtry(
    prog: *mut std::ffi::c_void,
    col: c_int,
    tm: ProfTime,
    timed_out: *mut c_int,
) -> c_int {
    if prog.is_null() {
        return 0;
    }

    let line = nvim_rex_get_line();
    if line.is_null() {
        return 0;
    }

    // Set rex.input = rex.line + col
    nvim_rex_set_input(line.add(col as usize));

    // Set rex.need_clear_subexpr = true
    nvim_rex_set_need_clear_subexpr(1);

    // Set rex.need_clear_zsubexpr = (prog->reghasz == REX_SET)
    let reghasz = nvim_bt_regprog_get_reghasz(prog);
    nvim_rex_set_need_clear_zsubexpr(c_int::from(reghasz == REX_SET));

    // Get the program bytecode (skip REGMAGIC byte)
    let program = nvim_bt_regprog_get_program(prog);
    if program.is_null() {
        return 0;
    }

    // Call the C regmatch function (starting after REGMAGIC)
    let program_start = program.add(1) as *mut u8;
    if nvim_bt_regmatch(program_start, tm, timed_out) == 0 {
        return 0;
    }

    // Clean up subexpressions
    nvim_cleanup_subexpr();

    let is_multi = nvim_rex_is_multi() != 0;
    let lnum = nvim_rex_get_lnum();

    if is_multi {
        // Multi-line mode: update reg_startpos/reg_endpos
        let startpos = nvim_rex_get_reg_startpos() as *mut LPosBt;
        let endpos = nvim_rex_get_reg_endpos() as *mut LPosBt;

        // Fix up startpos[0] if not set
        if (*startpos).lnum < 0 {
            (*startpos).lnum = 0;
            (*startpos).col = col;
        }

        // Fix up endpos[0] if not set
        if (*endpos).lnum < 0 {
            (*endpos).lnum = lnum;
            let input = nvim_rex_get_input();
            (*endpos).col = input.offset_from(line) as c_int;
        } else {
            // Use line number of "\ze"
            nvim_rex_set_lnum((*endpos).lnum);
        }
    } else {
        // Single-line mode: update reg_startp/reg_endp
        let startp = nvim_rex_get_reg_startp();
        let endp = nvim_rex_get_reg_endp();

        // Fix up startp[0] if NULL
        if (*startp).is_null() {
            *startp = line.add(col as usize);
        }

        // Fix up endp[0] if NULL
        if (*endp).is_null() {
            *endp = nvim_rex_get_input();
        }
    }

    // Package any found \z(...\) matches for export
    nvim_unref_extmatch(nvim_get_re_extmatch_out());
    nvim_set_re_extmatch_out(std::ptr::null_mut());

    if reghasz == REX_SET {
        nvim_cleanup_zsubexpr();
        let extmatch = nvim_make_extmatch();
        nvim_set_re_extmatch_out(extmatch);

        if is_multi {
            // Multi-line: copy from reg_startzpos/reg_endzpos
            let startzpos = nvim_get_reg_startzpos() as *mut LPosBt;
            let endzpos = nvim_get_reg_endzpos() as *mut LPosBt;

            for i in 0..NSUBEXP {
                let sp = startzpos.add(i);
                let ep = endzpos.add(i);

                // Only accept single line matches
                if (*sp).lnum >= 0 && (*ep).lnum == (*sp).lnum && (*ep).col >= (*sp).col {
                    let zline = nvim_reg_getline((*sp).lnum);
                    if !zline.is_null() {
                        let len = ((*ep).col - (*sp).col) as usize;
                        let src = (zline as *const u8).add((*sp).col as usize);
                        let match_str = xstrnsave(src as *const c_char, len);
                        nvim_extmatch_set_match(extmatch, i as c_int, match_str as *mut u8);
                    }
                }
            }
        } else {
            // Single-line: copy from reg_startzp/reg_endzp
            let startzp = nvim_get_reg_startzp();
            let endzp = nvim_get_reg_endzp();

            for i in 0..NSUBEXP {
                let sp = *startzp.add(i);
                let ep = *endzp.add(i);

                if !sp.is_null() && !ep.is_null() && ep >= sp {
                    let len = ep.offset_from(sp) as usize;
                    let match_str = xstrnsave(sp as *const c_char, len);
                    nvim_extmatch_set_match(extmatch, i as c_int, match_str as *mut u8);
                }
            }
        }
    }

    1 + nvim_rex_get_lnum()
}

// =============================================================================
// bt_regexec_both implementation (Phase 13c)
// =============================================================================

/// Flag constants for regflags
const RF_ICASE: c_int = 1; // \c in pattern
const RF_NOICASE: c_int = 2; // \C in pattern
const RF_ICOMBINE: c_int = 4; // \Z in pattern

// These extern declarations are ONLY for functions not declared elsewhere
// in the crate. Use raw c_void pointers to avoid type conflicts.
// Allow clashing extern declarations since nfa_exec.rs has different signatures
// for some of the same functions (ProfTime vs *const c_void, etc.)
#[allow(clippy::missing_safety_doc)]
#[allow(clashing_extern_declarations)]
extern "C" {
    // New accessors specific to bt_regexec_both (not in lib.rs)
    fn nvim_init_regstack();
    fn nvim_init_backpos();
    fn nvim_cleanup_regstack();
    fn nvim_cleanup_backpos();
    fn nvim_cleanup_reg_tofree();
    fn nvim_set_reg_toolong(v: bool);

    // BT-specific prog accessors (not in lib.rs)
    fn nvim_bt_regprog_get_regmust(prog: *const c_void) -> *const u8;
    fn nvim_bt_regprog_get_regmlen(prog: *const c_void) -> c_int;
    fn nvim_bt_regprog_get_reganch(prog: *const c_void) -> c_int;
    fn nvim_bt_regprog_get_regstart(prog: *const c_void) -> c_int;

    // Rex setters (not in lib.rs or using void pointers)
    fn nvim_rex_set_reg_startpos(p: *mut c_void);
    fn nvim_rex_set_reg_endpos(p: *mut c_void);
    fn nvim_rex_set_reg_startp(p: *mut *mut u8);
    fn nvim_rex_set_reg_endp(p: *mut *mut u8);
    fn nvim_rex_set_reg_ic(ic: bool);
    fn nvim_rex_set_reg_icombine(ic: bool);
    fn nvim_rex_set_line(line: *mut u8);

    // Regmatch/Regmmatch accessors with void pointers (to avoid conflicts)
    #[link_name = "nvim_regmatch_get_regprog"]
    fn nvim_regmatch_get_regprog_ptr(m: *const c_void) -> *const c_void;
    #[link_name = "nvim_regmatch_get_startp"]
    fn nvim_regmatch_get_startp_ptr(m: *const c_void) -> *mut *mut u8;
    #[link_name = "nvim_regmatch_get_endp"]
    fn nvim_regmatch_get_endp_ptr(m: *const c_void) -> *mut *mut u8;
    #[link_name = "nvim_regmatch_set_rm_matchcol"]
    fn nvim_regmatch_set_rm_matchcol_ptr(m: *mut c_void, col: c_int);
    #[link_name = "nvim_regmmatch_get_regprog"]
    fn nvim_regmmatch_get_regprog_ptr(m: *const c_void) -> *const c_void;
    #[link_name = "nvim_regmmatch_get_startpos"]
    fn nvim_regmmatch_get_startpos_ptr(m: *const c_void) -> *mut c_void;
    #[link_name = "nvim_regmmatch_get_endpos"]
    fn nvim_regmmatch_get_endpos_ptr(m: *const c_void) -> *mut c_void;
    #[link_name = "nvim_regmmatch_set_rmm_matchcol"]
    fn nvim_regmmatch_set_rmm_matchcol_ptr(m: *mut c_void, col: c_int);

    // Rex accessors with void pointers
    #[link_name = "nvim_rex_get_reg_mmatch"]
    fn nvim_rex_get_reg_mmatch_ptr() -> *mut c_void;
    #[link_name = "nvim_rex_get_reg_match"]
    fn nvim_rex_get_reg_match_ptr() -> *mut c_void;
    #[link_name = "nvim_rex_get_reg_maxcol"]
    fn nvim_rex_get_reg_maxcol_ptr() -> c_int;
    #[link_name = "nvim_regprog_get_regflags"]
    fn nvim_regprog_get_regflags_ptr(prog: *const c_void) -> c_int;

    // Validation and error
    fn nvim_prog_magic_wrong() -> c_int;
    fn nvim_iemsg_null();

    // String search
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // Timeout handling
    fn profile_passed_limit(tm: *const c_void) -> c_int;
}

/// Match a regexp against a string or multiple lines.
///
/// This implements bt_regexec_both() in Rust.
///
/// @param line       string to match against (NULL for multi-line)
/// @param startcol   column to start looking for match
/// @param tm         timeout limit or NULL
/// @param timed_out  flag set on timeout or NULL
///
/// @return 0 for failure, or number of lines contained in the match.
///
/// # Safety
/// All pointers must be valid. Rex state must be properly initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_bt_regexec_both(
    line: *mut u8,
    startcol: c_int,
    tm: ProfTime,
    timed_out: *mut c_int,
) -> c_int {
    let mut col = startcol;
    let mut retval = 0;
    let mut actual_line = line;

    // Initialize regstack and backpos
    nvim_init_regstack();
    nvim_init_backpos();

    // Get prog and set up match arrays based on REG_MULTI
    let is_multi = nvim_rex_is_multi() != 0;
    let prog: *const c_void;

    if is_multi {
        let mmatch = nvim_rex_get_reg_mmatch_ptr();
        prog = nvim_regmmatch_get_regprog_ptr(mmatch);
        actual_line = nvim_reg_getline(0) as *mut u8;
        nvim_rex_set_reg_startpos(nvim_regmmatch_get_startpos_ptr(mmatch));
        nvim_rex_set_reg_endpos(nvim_regmmatch_get_endpos_ptr(mmatch));
    } else {
        let m = nvim_rex_get_reg_match_ptr();
        prog = nvim_regmatch_get_regprog_ptr(m);
        nvim_rex_set_reg_startp(nvim_regmatch_get_startp_ptr(m));
        nvim_rex_set_reg_endp(nvim_regmatch_get_endp_ptr(m));
    };

    // Be paranoid...
    if prog.is_null() || actual_line.is_null() {
        nvim_iemsg_null();
        return cleanup_and_return(retval, col, is_multi);
    }

    // Check validity of program
    if nvim_prog_magic_wrong() != 0 {
        return cleanup_and_return(retval, col, is_multi);
    }

    // If the start column is past the maximum column: no need to try
    let reg_maxcol = nvim_rex_get_reg_maxcol_ptr();
    if reg_maxcol > 0 && col >= reg_maxcol {
        return cleanup_and_return(retval, col, is_multi);
    }

    // If pattern contains "\c" or "\C": overrule value of rex.reg_ic
    let regflags = nvim_regprog_get_regflags_ptr(prog);
    if regflags & RF_ICASE != 0 {
        nvim_rex_set_reg_ic(true);
    } else if regflags & RF_NOICASE != 0 {
        nvim_rex_set_reg_ic(false);
    }

    // If pattern contains "\Z" overrule value of rex.reg_icombine
    if regflags & RF_ICOMBINE != 0 {
        nvim_rex_set_reg_icombine(true);
    }

    // If there is a "must appear" string, look for it
    let regmust = nvim_bt_regprog_get_regmust(prog);
    if !regmust.is_null() {
        let c = utf_ptr2char(regmust as *const c_char);
        let mut s = actual_line.add(col as usize);

        // Search for the must-appear string
        let reg_ic = nvim_rex_get_reg_ic();
        if !reg_ic {
            // Case-sensitive search
            while let Some(found) = ptr_from_result(vim_strchr(s as *const c_char, c)) {
                let mut len = nvim_bt_regprog_get_regmlen(prog);
                if rs_cstrncmp(found as *const c_char, regmust as *const c_char, &mut len) == 0 {
                    s = found as *mut u8;
                    break;
                }
                s = mb_ptr_adv(found as *mut u8);
            }
            if vim_strchr(s as *const c_char, c).is_null() {
                // Didn't find it, need to search again
                s = std::ptr::null_mut();
            }
        } else {
            // Case-insensitive search
            while let Some(found) = ptr_from_result(rs_cstrchr(s as *const c_char, c)) {
                let mut len = nvim_bt_regprog_get_regmlen(prog);
                if rs_cstrncmp(found as *const c_char, regmust as *const c_char, &mut len) == 0 {
                    s = found as *mut u8;
                    break;
                }
                s = mb_ptr_adv(found as *mut u8);
            }
            if rs_cstrchr(s as *const c_char, c).is_null() {
                s = std::ptr::null_mut();
            }
        }

        if s.is_null() {
            // Not present
            return cleanup_and_return(retval, col, is_multi);
        }
    }

    // Set up rex state for matching
    nvim_rex_set_line(actual_line);
    nvim_rex_set_lnum(0);
    nvim_set_reg_toolong(false);

    let reganch = nvim_bt_regprog_get_reganch(prog);
    let regstart = nvim_bt_regprog_get_regstart(prog);

    if reganch != 0 {
        // Simplest case: Anchored match need be tried only once
        let c = utf_ptr2char((actual_line as *const u8).add(col as usize) as *const c_char);
        let reg_ic = nvim_rex_get_reg_ic();

        if regstart == 0
            || regstart == c
            || (reg_ic
                && (utf_fold(regstart) == utf_fold(c)
                    || (c < 255 && regstart < 255 && mb_tolower(regstart) == mb_tolower(c))))
        {
            retval = rs_bt_regtry(prog as *mut c_void, col, tm, timed_out);
        }
    } else {
        // Messy cases: unanchored match
        let mut tm_count = 0;
        while nvim_get_got_int() == 0 {
            if regstart != 0 {
                // Skip until the char we know it must start with
                let s = rs_cstrchr(
                    (actual_line as *const u8).add(col as usize) as *const c_char,
                    regstart,
                );
                if s.is_null() {
                    retval = 0;
                    break;
                }
                col = s.offset_from(actual_line as *const c_char) as c_int;
            }

            // Check for maximum column to try
            if reg_maxcol > 0 && col >= reg_maxcol {
                retval = 0;
                break;
            }

            retval = rs_bt_regtry(prog as *mut c_void, col, tm, timed_out);
            if retval > 0 {
                break;
            }

            // If not currently on the first line, get it again
            if nvim_rex_get_lnum() != 0 {
                nvim_rex_set_lnum(0);
                actual_line = nvim_reg_getline(0) as *mut u8;
                nvim_rex_set_line(actual_line);
            }

            if *actual_line.add(col as usize) == 0 {
                break;
            }

            col += utfc_ptr2len((actual_line as *const u8).add(col as usize) as *const c_char);

            // Check for timeout once every 20 times to avoid overhead
            if !tm.is_null() {
                tm_count += 1;
                if tm_count == 20 {
                    tm_count = 0;
                    if profile_passed_limit(tm) != 0 {
                        if !timed_out.is_null() {
                            *timed_out = 1;
                        }
                        break;
                    }
                }
            }
        }
    }

    cleanup_and_return(retval, col, is_multi)
}

/// Helper to convert nullable pointer result
#[inline]
unsafe fn ptr_from_result(p: *mut c_char) -> Option<*mut c_char> {
    if p.is_null() {
        None
    } else {
        Some(p)
    }
}

/// Clean up and fix up positions after match attempt
unsafe fn cleanup_and_return(retval: c_int, col: c_int, is_multi: bool) -> c_int {
    // Free reg_tofree when it's a bit big
    nvim_cleanup_reg_tofree();

    // Free regstack and backpos if they are bigger than their initial size
    nvim_cleanup_regstack();
    nvim_cleanup_backpos();

    if retval > 0 {
        // Make sure the end is never before the start (can happen with \zs and \ze)
        if is_multi {
            let mmatch = nvim_rex_get_reg_mmatch_ptr();
            let startpos = nvim_regmmatch_get_startpos_ptr(mmatch) as *mut LPosBt;
            let endpos = nvim_regmmatch_get_endpos_ptr(mmatch) as *mut LPosBt;

            if (*endpos).lnum < (*startpos).lnum
                || ((*endpos).lnum == (*startpos).lnum && (*endpos).col < (*startpos).col)
            {
                *endpos = *startpos;
            }

            // Set rmm_matchcol to the column where the whole pattern matched
            nvim_regmmatch_set_rmm_matchcol_ptr(mmatch, col);
        } else {
            let m = nvim_rex_get_reg_match_ptr();
            let startp = nvim_regmatch_get_startp_ptr(m);
            let endp = nvim_regmatch_get_endp_ptr(m);

            if !(*startp).is_null() && !(*endp).is_null() && (*endp) < (*startp) {
                *endp = *startp;
            }

            // Set rm_matchcol to the column where the whole pattern matched
            nvim_regmatch_set_rm_matchcol_ptr(m, col);
        }
    }

    retval
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
        assert_eq!(popped1, Some((RegState::StarLong, scan2)));

        let popped2 = state.pop_backtrack();
        assert_eq!(popped2, Some((RegState::Branch, scan1)));

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
