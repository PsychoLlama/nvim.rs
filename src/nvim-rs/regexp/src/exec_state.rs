//! Unified execution state for regex engines.
//!
//! This module provides a unified execution state that's shared between
//! the backtracking (BT) and NFA regex engines. It consolidates the
//! rex (regexec_T) structure from C and provides Rust-native access.
//!
//! # Key Concepts
//!
//! - **Execution Context**: Captures all state needed during regex execution
//! - **Multi-mode Support**: Handles both single-line and multi-line matching
//! - **Submatch Tracking**: Maintains start/end positions for captured groups
//! - **Line Fetching**: Callback-based access to buffer lines for multi-line

use std::ffi::{c_int, c_void};
use std::ptr;

use crate::{BufHandle, LposHandle, RegmatchHandle, RegmmatchHandle, WinHandle};

/// Number of subexpressions supported.
pub const NSUBEXP: usize = 10;

// =============================================================================
// Line Number Types
// =============================================================================

/// Line number type (matches C linenr_T).
pub type LineNr = c_int;

/// Column number type (matches C colnr_T).
pub type ColNr = c_int;

// =============================================================================
// FFI Declarations - Rex Accessors
// =============================================================================

#[allow(dead_code)] // Some accessors are infrastructure for future phases
extern "C" {
    // Current position accessors
    fn nvim_rex_get_lnum() -> LineNr;
    fn nvim_rex_set_lnum(lnum: LineNr);
    fn nvim_rex_get_line() -> *mut u8;
    fn nvim_rex_set_line(line: *mut u8);
    fn nvim_rex_get_input() -> *mut u8;
    fn nvim_rex_set_input(input: *mut u8);

    // Match state accessors
    fn nvim_rex_get_reg_match() -> RegmatchHandle;
    fn nvim_rex_set_reg_match(m: RegmatchHandle);
    fn nvim_rex_get_reg_mmatch() -> RegmmatchHandle;
    fn nvim_rex_set_reg_mmatch(m: RegmmatchHandle);

    // Submatch position accessors
    fn nvim_rex_get_reg_startp() -> *mut *mut u8;
    fn nvim_rex_set_reg_startp(p: *mut *mut u8);
    fn nvim_rex_get_reg_endp() -> *mut *mut u8;
    fn nvim_rex_set_reg_endp(p: *mut *mut u8);
    fn nvim_rex_get_reg_startpos() -> LposHandle;
    fn nvim_rex_set_reg_startpos(p: LposHandle);
    fn nvim_rex_get_reg_endpos() -> LposHandle;
    fn nvim_rex_set_reg_endpos(p: LposHandle);

    // Buffer/window context accessors
    fn nvim_rex_get_reg_win() -> WinHandle;
    fn nvim_rex_set_reg_win(win: WinHandle);
    fn nvim_rex_get_reg_buf() -> BufHandle;
    fn nvim_rex_set_reg_buf(buf: BufHandle);
    fn nvim_rex_get_reg_firstlnum() -> LineNr;
    fn nvim_rex_set_reg_firstlnum(lnum: LineNr);
    fn nvim_rex_get_reg_maxline() -> LineNr;
    fn nvim_rex_set_reg_maxline(lnum: LineNr);

    // Flag accessors
    fn nvim_rex_get_reg_ic() -> bool;
    fn nvim_rex_set_reg_ic(ic: bool);
    fn nvim_rex_get_reg_icombine() -> bool;
    fn nvim_rex_set_reg_icombine(ic: bool);
    fn nvim_rex_get_reg_line_lbr() -> bool;
    fn nvim_rex_set_reg_line_lbr(lbr: bool);
    fn nvim_rex_get_reg_nobreak() -> bool;
    fn nvim_rex_set_reg_nobreak(nb: bool);
    fn nvim_rex_get_reg_maxcol() -> ColNr;
    fn nvim_rex_set_reg_maxcol(col: ColNr);

    // Subexpression clearing flags
    fn nvim_rex_get_need_clear_subexpr() -> c_int;
    fn nvim_rex_set_need_clear_subexpr(v: c_int);
    fn nvim_rex_get_need_clear_zsubexpr() -> c_int;
    fn nvim_rex_set_need_clear_zsubexpr(v: c_int);

    // NFA engine state accessors
    fn nvim_rex_get_nfa_has_zend() -> c_int;
    fn nvim_rex_set_nfa_has_zend(v: c_int);
    fn nvim_rex_get_nfa_has_backref() -> c_int;
    fn nvim_rex_set_nfa_has_backref(v: c_int);
    fn nvim_rex_get_nfa_nsubexpr() -> c_int;
    fn nvim_rex_set_nfa_nsubexpr(v: c_int);
    fn nvim_rex_get_nfa_listid() -> c_int;
    fn nvim_rex_set_nfa_listid(v: c_int);
    fn nvim_rex_get_nfa_alt_listid() -> c_int;
    fn nvim_rex_set_nfa_alt_listid(v: c_int);
    fn nvim_rex_get_nfa_has_zsubexpr() -> c_int;
    fn nvim_rex_set_nfa_has_zsubexpr(v: c_int);

    // rex_in_use flag
    fn nvim_rex_in_use() -> bool;
    fn nvim_rex_set_in_use(in_use: bool);
}

// =============================================================================
// Execution State
// =============================================================================

/// Unified execution state for regex matching.
///
/// This structure mirrors the C `regexec_T` (rex) structure and provides
/// a Rust-native interface to the execution state. It supports both
/// single-line and multi-line matching modes.
#[derive(Debug)]
pub struct ExecState {
    /// Whether this state is for multi-line matching
    pub is_multi: bool,

    /// Current line number (relative to first line, multi-line mode)
    pub lnum: LineNr,

    /// Current input position (single-line mode)
    pub input: *mut u8,

    /// Start of current line
    pub line: *mut u8,

    /// First line number for multi-line search
    pub first_lnum: LineNr,

    /// Maximum line number (0 = last line of buffer)
    pub max_line: LineNr,

    /// Case-insensitive matching
    pub ignore_case: bool,

    /// Ignore case for combining characters
    pub ignore_combine: bool,

    /// Treat "\n" in string as line break
    pub line_lbr: bool,

    /// Don't call breakcheck
    pub no_break: bool,

    /// Maximum column to search (0 = no limit)
    pub max_col: ColNr,

    /// Subexpressions need to be cleared
    pub need_clear_subexpr: bool,

    /// Extmatch subexpressions need to be cleared
    pub need_clear_zsubexpr: bool,

    /// NFA: \ze encountered
    pub nfa_has_zend: bool,

    /// NFA: \1..\9 encountered
    pub nfa_has_backref: bool,

    /// NFA: number of subexpressions in use
    pub nfa_nsubexpr: c_int,

    /// NFA: current list ID
    pub nfa_listid: c_int,

    /// NFA: alternate list ID
    pub nfa_alt_listid: c_int,

    /// NFA: \z( ) encountered
    pub nfa_has_zsubexpr: bool,
}

impl Default for ExecState {
    fn default() -> Self {
        Self {
            is_multi: false,
            lnum: 0,
            input: ptr::null_mut(),
            line: ptr::null_mut(),
            first_lnum: 0,
            max_line: 0,
            ignore_case: false,
            ignore_combine: false,
            line_lbr: false,
            no_break: false,
            max_col: 0,
            need_clear_subexpr: false,
            need_clear_zsubexpr: false,
            nfa_has_zend: false,
            nfa_has_backref: false,
            nfa_nsubexpr: 0,
            nfa_listid: 0,
            nfa_alt_listid: 0,
            nfa_has_zsubexpr: false,
        }
    }
}

impl ExecState {
    /// Create a new execution state for single-line matching.
    pub fn new_single(input: *mut u8) -> Self {
        Self {
            is_multi: false,
            input,
            line: input,
            ..Default::default()
        }
    }

    /// Create a new execution state for multi-line matching.
    pub fn new_multi(first_lnum: LineNr, max_line: LineNr) -> Self {
        Self {
            is_multi: true,
            first_lnum,
            max_line,
            ..Default::default()
        }
    }

    /// Load state from the global rex structure.
    ///
    /// # Safety
    /// Must be called when rex is properly initialized.
    pub unsafe fn load_from_rex(&mut self) {
        self.lnum = nvim_rex_get_lnum();
        self.line = nvim_rex_get_line();
        self.input = nvim_rex_get_input();
        self.first_lnum = nvim_rex_get_reg_firstlnum();
        self.max_line = nvim_rex_get_reg_maxline();
        self.ignore_case = nvim_rex_get_reg_ic();
        self.ignore_combine = nvim_rex_get_reg_icombine();
        self.line_lbr = nvim_rex_get_reg_line_lbr();
        self.no_break = nvim_rex_get_reg_nobreak();
        self.max_col = nvim_rex_get_reg_maxcol();
        self.need_clear_subexpr = nvim_rex_get_need_clear_subexpr() != 0;
        self.need_clear_zsubexpr = nvim_rex_get_need_clear_zsubexpr() != 0;
        self.nfa_has_zend = nvim_rex_get_nfa_has_zend() != 0;
        self.nfa_has_backref = nvim_rex_get_nfa_has_backref() != 0;
        self.nfa_nsubexpr = nvim_rex_get_nfa_nsubexpr();
        self.nfa_listid = nvim_rex_get_nfa_listid();
        self.nfa_alt_listid = nvim_rex_get_nfa_alt_listid();
        self.nfa_has_zsubexpr = nvim_rex_get_nfa_has_zsubexpr() != 0;
    }

    /// Save state back to the global rex structure.
    ///
    /// # Safety
    /// Must be called when rex is properly initialized.
    pub unsafe fn save_to_rex(&self) {
        nvim_rex_set_lnum(self.lnum);
        nvim_rex_set_line(self.line);
        nvim_rex_set_input(self.input);
        nvim_rex_set_reg_firstlnum(self.first_lnum);
        nvim_rex_set_reg_maxline(self.max_line);
        nvim_rex_set_reg_ic(self.ignore_case);
        nvim_rex_set_reg_icombine(self.ignore_combine);
        nvim_rex_set_reg_line_lbr(self.line_lbr);
        nvim_rex_set_reg_nobreak(self.no_break);
        nvim_rex_set_reg_maxcol(self.max_col);
        nvim_rex_set_need_clear_subexpr(c_int::from(self.need_clear_subexpr));
        nvim_rex_set_need_clear_zsubexpr(c_int::from(self.need_clear_zsubexpr));
        nvim_rex_set_nfa_has_zend(c_int::from(self.nfa_has_zend));
        nvim_rex_set_nfa_has_backref(c_int::from(self.nfa_has_backref));
        nvim_rex_set_nfa_nsubexpr(self.nfa_nsubexpr);
        nvim_rex_set_nfa_listid(self.nfa_listid);
        nvim_rex_set_nfa_alt_listid(self.nfa_alt_listid);
        nvim_rex_set_nfa_has_zsubexpr(c_int::from(self.nfa_has_zsubexpr));
    }

    /// Get the current byte at input position.
    ///
    /// # Safety
    /// Input must be valid.
    #[inline]
    pub unsafe fn current_byte(&self) -> u8 {
        if self.input.is_null() {
            0
        } else {
            *self.input
        }
    }

    /// Advance input by one byte.
    ///
    /// # Safety
    /// Input must be valid and have more bytes.
    #[inline]
    pub unsafe fn advance(&mut self) {
        if !self.input.is_null() {
            self.input = self.input.add(1);
        }
    }

    /// Advance input by n bytes.
    ///
    /// # Safety
    /// Input must be valid and have at least n more bytes.
    #[inline]
    pub unsafe fn advance_by(&mut self, n: usize) {
        if !self.input.is_null() {
            self.input = self.input.add(n);
        }
    }

    /// Check if at end of line.
    ///
    /// # Safety
    /// Input must be valid.
    #[inline]
    pub unsafe fn at_eol(&self) -> bool {
        self.input.is_null() || *self.input == 0 || *self.input == b'\n'
    }

    /// Check if at beginning of line.
    #[inline]
    pub fn at_bol(&self) -> bool {
        self.input == self.line
    }

    /// Get the offset from line start.
    ///
    /// # Safety
    /// Both input and line must be valid pointers into the same buffer.
    #[inline]
    pub unsafe fn col_offset(&self) -> ColNr {
        if self.input.is_null() || self.line.is_null() {
            0
        } else {
            self.input.offset_from(self.line) as ColNr
        }
    }
}

// =============================================================================
// Saved Position for Backtracking
// =============================================================================

/// Saved position for backtracking.
///
/// This can store either a pointer (single-line) or line/col (multi-line).
#[derive(Clone, Copy)]
pub union SavedPos {
    /// Single-line: pointer into input
    pub ptr: *mut u8,
    /// Multi-line: line and column
    pub multi: MultiPos,
}

/// Multi-line position.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MultiPos {
    /// Line number (relative to first line)
    pub lnum: LineNr,
    /// Column (byte offset)
    pub col: ColNr,
}

impl Default for SavedPos {
    fn default() -> Self {
        Self {
            ptr: ptr::null_mut(),
        }
    }
}

/// Save the current position from execution state.
pub fn save_pos(state: &ExecState) -> SavedPos {
    if state.is_multi {
        SavedPos {
            multi: MultiPos {
                lnum: state.lnum,
                col: unsafe { state.col_offset() },
            },
        }
    } else {
        SavedPos { ptr: state.input }
    }
}

/// Restore position to execution state.
///
/// # Safety
/// The saved position must correspond to the state's mode (multi vs single).
pub unsafe fn restore_pos(state: &mut ExecState, saved: &SavedPos) {
    if state.is_multi {
        state.lnum = saved.multi.lnum;
        // Note: input pointer needs to be recalculated based on line content
        // This is typically done by the caller after fetching the line
    } else {
        state.input = saved.ptr;
    }
}

// =============================================================================
// Line Fetcher Callback
// =============================================================================

/// Type for line fetching callback (multi-line matching).
///
/// Called to get line content from a buffer.
///
/// # Parameters
/// - `lnum`: Absolute line number (1-based)
/// - `userdata`: User-provided context
///
/// # Returns
/// Pointer to NUL-terminated line content, or null if line doesn't exist.
pub type LineFetcher = unsafe extern "C" fn(lnum: LineNr, userdata: *mut c_void) -> *const u8;

/// Line fetcher context for multi-line matching.
pub struct LineFetchContext {
    /// The callback function
    pub fetcher: Option<LineFetcher>,
    /// User data passed to callback
    pub userdata: *mut c_void,
    /// Cached line number
    pub cached_lnum: LineNr,
    /// Cached line pointer
    pub cached_line: *const u8,
}

impl Default for LineFetchContext {
    fn default() -> Self {
        Self {
            fetcher: None,
            userdata: ptr::null_mut(),
            cached_lnum: 0,
            cached_line: ptr::null(),
        }
    }
}

impl LineFetchContext {
    /// Create a new line fetch context.
    pub fn new(fetcher: LineFetcher, userdata: *mut c_void) -> Self {
        Self {
            fetcher: Some(fetcher),
            userdata,
            cached_lnum: 0,
            cached_line: ptr::null(),
        }
    }

    /// Fetch a line, using cache if possible.
    ///
    /// # Safety
    /// The callback must be valid if set.
    pub unsafe fn get_line(&mut self, lnum: LineNr) -> *const u8 {
        if lnum == self.cached_lnum && !self.cached_line.is_null() {
            return self.cached_line;
        }

        if let Some(fetcher) = self.fetcher {
            self.cached_line = fetcher(lnum, self.userdata);
            self.cached_lnum = lnum;
            self.cached_line
        } else {
            ptr::null()
        }
    }

    /// Invalidate the cache.
    pub fn invalidate_cache(&mut self) {
        self.cached_lnum = 0;
        self.cached_line = ptr::null();
    }
}

// =============================================================================
// Guard for rex_in_use
// =============================================================================

/// RAII guard for rex_in_use flag.
///
/// Ensures the flag is properly cleared when execution completes,
/// even if there's a panic.
pub struct RexGuard {
    was_in_use: bool,
}

impl RexGuard {
    /// Create a new guard, setting rex_in_use.
    ///
    /// # Safety
    /// Must be called from a context where rex can be accessed.
    pub unsafe fn new() -> Option<Self> {
        if nvim_rex_in_use() {
            // Already in use - recursive call
            None
        } else {
            nvim_rex_set_in_use(true);
            Some(Self { was_in_use: false })
        }
    }

    /// Create a guard that doesn't check existing state.
    ///
    /// Used for nested calls that know it's safe.
    ///
    /// # Safety
    /// Caller must ensure rex state is correct.
    pub unsafe fn new_unchecked() -> Self {
        let was_in_use = nvim_rex_in_use();
        nvim_rex_set_in_use(true);
        Self { was_in_use }
    }
}

impl Drop for RexGuard {
    fn drop(&mut self) {
        // SAFETY: We're just clearing the flag we set
        unsafe {
            nvim_rex_set_in_use(self.was_in_use);
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Initialize execution state for single-line matching.
///
/// # Safety
/// `input` must be valid or null.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_state_init_single(input: *mut u8) -> *mut ExecState {
    Box::into_raw(Box::new(ExecState::new_single(input)))
}

/// Initialize execution state for multi-line matching.
#[no_mangle]
pub extern "C" fn rs_exec_state_init_multi(first_lnum: LineNr, max_line: LineNr) -> *mut ExecState {
    Box::into_raw(Box::new(ExecState::new_multi(first_lnum, max_line)))
}

/// Free execution state.
///
/// # Safety
/// `state` must be a valid pointer from `rs_exec_state_init_*`.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_state_free(state: *mut ExecState) {
    if !state.is_null() {
        drop(Box::from_raw(state));
    }
}

/// Load state from global rex.
///
/// # Safety
/// `state` must be valid and rex must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_state_load_from_rex(state: *mut ExecState) {
    if !state.is_null() {
        (*state).load_from_rex();
    }
}

/// Save state to global rex.
///
/// # Safety
/// `state` must be valid and rex must be initialized.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_state_save_to_rex(state: *const ExecState) {
    if !state.is_null() {
        (*state).save_to_rex();
    }
}

/// Get current byte at input position.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_state_current_byte(state: *const ExecState) -> u8 {
    if state.is_null() {
        0
    } else {
        (*state).current_byte()
    }
}

/// Advance input by one byte.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_state_advance(state: *mut ExecState) {
    if !state.is_null() {
        (*state).advance();
    }
}

/// Check if at end of line.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_exec_state_at_eol(state: *const ExecState) -> c_int {
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
pub unsafe extern "C" fn rs_exec_state_at_bol(state: *const ExecState) -> c_int {
    if state.is_null() {
        0
    } else {
        c_int::from((*state).at_bol())
    }
}

/// Check if rex is in use.
#[no_mangle]
pub extern "C" fn rs_rex_in_use() -> c_int {
    unsafe { c_int::from(nvim_rex_in_use()) }
}

/// Set rex in use flag.
#[no_mangle]
pub extern "C" fn rs_rex_set_in_use(in_use: c_int) {
    unsafe {
        nvim_rex_set_in_use(in_use != 0);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_state_default() {
        let state = ExecState::default();
        assert!(!state.is_multi);
        assert!(state.input.is_null());
        assert!(state.line.is_null());
        assert_eq!(state.lnum, 0);
        assert!(!state.ignore_case);
    }

    #[test]
    fn test_exec_state_new_single() {
        let mut data = [b'a', b'b', b'c', 0];
        let state = ExecState::new_single(data.as_mut_ptr());
        assert!(!state.is_multi);
        assert_eq!(state.input, data.as_mut_ptr());
        assert_eq!(state.line, data.as_mut_ptr());
    }

    #[test]
    fn test_exec_state_new_multi() {
        let state = ExecState::new_multi(10, 100);
        assert!(state.is_multi);
        assert_eq!(state.first_lnum, 10);
        assert_eq!(state.max_line, 100);
    }

    #[test]
    fn test_current_byte_and_advance() {
        let mut data = [b'h', b'e', b'l', b'l', b'o', 0];
        let mut state = ExecState::new_single(data.as_mut_ptr());

        unsafe {
            assert_eq!(state.current_byte(), b'h');
            state.advance();
            assert_eq!(state.current_byte(), b'e');
            state.advance_by(3);
            assert_eq!(state.current_byte(), b'o');
            state.advance();
            assert!(state.at_eol());
        }
    }

    #[test]
    fn test_at_bol() {
        let mut data = [b'a', b'b', 0];
        let mut state = ExecState::new_single(data.as_mut_ptr());

        assert!(state.at_bol());
        unsafe {
            state.advance();
        }
        assert!(!state.at_bol());
    }

    #[test]
    fn test_saved_pos_single() {
        let mut data = [b'a', b'b', b'c', 0];
        let mut state = ExecState::new_single(data.as_mut_ptr());

        unsafe {
            state.advance();
            let saved = save_pos(&state);
            state.advance();
            assert_eq!(state.current_byte(), b'c');

            restore_pos(&mut state, &saved);
            assert_eq!(state.current_byte(), b'b');
        }
    }

    #[test]
    fn test_saved_pos_multi() {
        let state = ExecState::new_multi(1, 100);
        let saved = save_pos(&state);

        unsafe {
            assert_eq!(saved.multi.lnum, 0);
            assert_eq!(saved.multi.col, 0);
        }
    }

    #[test]
    fn test_multi_pos_default() {
        let pos = MultiPos::default();
        assert_eq!(pos.lnum, 0);
        assert_eq!(pos.col, 0);
    }

    #[test]
    fn test_line_fetch_context_default() {
        let ctx = LineFetchContext::default();
        assert!(ctx.fetcher.is_none());
        assert!(ctx.userdata.is_null());
        assert_eq!(ctx.cached_lnum, 0);
        assert!(ctx.cached_line.is_null());
    }

    #[test]
    fn test_line_fetch_context_invalidate() {
        let mut ctx = LineFetchContext {
            cached_lnum: 42,
            cached_line: 0x1234 as *const u8,
            ..Default::default()
        };

        ctx.invalidate_cache();

        assert_eq!(ctx.cached_lnum, 0);
        assert!(ctx.cached_line.is_null());
    }

    #[test]
    fn test_nsubexp_constant() {
        assert_eq!(NSUBEXP, 10);
    }
}
