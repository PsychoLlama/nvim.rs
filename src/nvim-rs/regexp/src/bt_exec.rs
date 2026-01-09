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

use std::ffi::c_int;
use std::ptr;

use crate::bt_compile::{op, operand};
use crate::bt_opcodes::{ANY, ANYBUT, ANYOF, DIGIT, EXACTLY, HEAD, NEWL, WHITE, WORD};
use crate::bt_state::{BackPosTable, RegSave, RegStack, RegState, NSUBEXP};

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
            ANY => c != b'\n' || state.match_nl,
            ANYOF => match_class(opnd, c),
            ANYBUT => !match_class(opnd, c) && c != b'\n',
            EXACTLY => !opnd.is_null() && *opnd == c,
            NEWL => c == b'\n',
            WHITE => c == b' ' || c == b'\t',
            DIGIT => c.is_ascii_digit(),
            WORD => c.is_ascii_alphanumeric() || c == b'_',
            HEAD => c.is_ascii_alphabetic() || c == b'_',
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

// =============================================================================
// Tests
// =============================================================================

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
