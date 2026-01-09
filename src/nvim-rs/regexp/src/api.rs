//! Unified Regular Expression API
//!
//! This module provides a unified interface for the Vim regex engine,
//! abstracting over both the NFA and backtracking implementations.
//!
//! # Overview
//!
//! Vim supports two regex engines:
//! 1. **NFA engine** (`\%#=1`): Faster for most patterns, uses Thompson construction
//! 2. **BT engine** (`\%#=0`): Supports backreferences, uses recursive descent
//!
//! The engine selection is controlled by the `'regexpengine'` option (`p_re`):
//! - 0: Automatic (try NFA first, fall back to BT for unsupported features)
//! - 1: Force NFA
//! - 2: Force BT
//!
//! # Public API
//!
//! - [`RegEngine`]: Enum for engine selection
//! - [`vim_regcomp`]: Compile a pattern into a regex program
//! - [`vim_regexec`]: Execute a regex match
//! - [`vim_regfree`]: Free a compiled regex program

use std::ffi::c_int;
use std::ptr;

// =============================================================================
// Engine Selection
// =============================================================================

/// Regex engine selection.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RegEngine {
    /// Use backtracking engine
    Bt = 0,
    /// Use NFA engine (default)
    #[default]
    Nfa = 1,
    /// Automatic: try NFA first, fall back to BT
    Auto = 2,
}

impl From<c_int> for RegEngine {
    fn from(v: c_int) -> Self {
        match v {
            0 => Self::Bt,
            1 => Self::Nfa,
            _ => Self::Auto,
        }
    }
}

impl From<RegEngine> for c_int {
    fn from(e: RegEngine) -> Self {
        e as c_int
    }
}

// =============================================================================
// Compilation Flags
// =============================================================================

/// RE_MAGIC: Use 'magic' option setting.
pub const RE_MAGIC: c_int = 1;

/// RE_STRING: Match in a string instead of a buffer line.
pub const RE_STRING: c_int = 2;

/// RE_STRICT: Don't allow \( and \) for () when not RE_MAGIC.
pub const RE_STRICT: c_int = 4;

/// RE_VIM: Match using Vim regex syntax.
pub const RE_VIM: c_int = 8;

/// RE_SEARCH: Pattern is for search command.
pub const RE_SEARCH: c_int = 16;

/// RE_AUTO: Use automatic engine selection.
pub const RE_AUTO: c_int = 32;

// =============================================================================
// Match Flags
// =============================================================================

/// RF_ICASE: Ignore case when matching.
pub const RF_ICASE: c_int = 0x01;

/// RF_NOICASE: Don't ignore case, even with 'ignorecase' set.
pub const RF_NOICASE: c_int = 0x02;

/// RF_HASNL: Pattern contains \n.
pub const RF_HASNL: c_int = 0x04;

/// RF_ICOMBINE: Ignore combining characters.
pub const RF_ICOMBINE: c_int = 0x08;

/// RF_LOOKBH: Pattern uses look-behind.
pub const RF_LOOKBH: c_int = 0x10;

/// RF_NOSUB: Don't set matches in matches[].
pub const RF_NOSUB: c_int = 0x20;

/// RF_ZSUBEXPR: Pattern uses \z external subexpression.
pub const RF_ZSUBEXPR: c_int = 0x40;

// =============================================================================
// Program Types
// =============================================================================

/// Opaque handle to a compiled regex program.
///
/// This can hold either an NFA or BT program, determined at compile time.
#[repr(C)]
pub struct RegProg {
    /// Engine used (0 = BT, 1 = NFA)
    pub engine: c_int,
    /// Pattern flags (RF_*)
    pub regflags: c_int,
    /// Whether program is currently in use
    pub re_in_use: bool,
    // Note: actual program data follows in C
}

// =============================================================================
// Match Result Types
// =============================================================================

/// Position in a buffer (multi-line matching).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LPos {
    /// Line number (1-based).
    pub lnum: c_int,
    /// Column number (0-based byte offset).
    pub col: c_int,
    /// Coladd (virtual column addition).
    pub coladd: c_int,
}

/// Number of submatches (including full match).
pub const NSUBEXP: usize = 10;

/// Match result for single-line matching.
#[repr(C)]
pub struct RegMatch {
    /// Compiled program (may be NULL if using external program).
    pub regprog: *mut RegProg,
    /// Start positions of submatches.
    pub startp: [*const u8; NSUBEXP],
    /// End positions of submatches.
    pub endp: [*const u8; NSUBEXP],
    /// Whether the match succeeded.
    pub rm_ic: bool,
}

impl Default for RegMatch {
    fn default() -> Self {
        Self {
            regprog: ptr::null_mut(),
            startp: [ptr::null(); NSUBEXP],
            endp: [ptr::null(); NSUBEXP],
            rm_ic: false,
        }
    }
}

/// Match result for multi-line matching.
#[repr(C)]
pub struct RegMmatch {
    /// Compiled program.
    pub regprog: *mut RegProg,
    /// Start positions of submatches (line, col).
    pub startpos: [LPos; NSUBEXP],
    /// End positions of submatches (line, col).
    pub endpos: [LPos; NSUBEXP],
    /// Whether the match succeeded.
    pub rmm_ic: bool,
    /// Maximum column for match.
    pub rmm_maxcol: c_int,
}

impl Default for RegMmatch {
    fn default() -> Self {
        Self {
            regprog: ptr::null_mut(),
            startpos: [LPos::default(); NSUBEXP],
            endpos: [LPos::default(); NSUBEXP],
            rmm_ic: false,
            rmm_maxcol: 0,
        }
    }
}

// =============================================================================
// API Functions (Rust implementation)
// =============================================================================

/// Check if a compiled program uses the NFA engine.
///
/// # Safety
/// `prog` must be null or a valid pointer.
pub unsafe fn is_nfa_prog(prog: *const RegProg) -> bool {
    if prog.is_null() {
        false
    } else {
        (*prog).engine == 1
    }
}

/// Check if a compiled program uses the BT engine.
///
/// # Safety
/// `prog` must be null or a valid pointer.
pub unsafe fn is_bt_prog(prog: *const RegProg) -> bool {
    if prog.is_null() {
        false
    } else {
        (*prog).engine == 0
    }
}

/// Get the flags from a compiled program.
///
/// # Safety
/// `prog` must be null or a valid pointer.
pub unsafe fn get_regflags(prog: *const RegProg) -> c_int {
    if prog.is_null() {
        0
    } else {
        (*prog).regflags
    }
}

/// Check if program is currently in use (for recursion detection).
///
/// # Safety
/// `prog` must be null or a valid pointer.
pub unsafe fn prog_in_use(prog: *const RegProg) -> bool {
    if prog.is_null() {
        false
    } else {
        (*prog).re_in_use
    }
}

/// Mark program as in use.
///
/// # Safety
/// `prog` must be a valid pointer.
pub unsafe fn set_prog_in_use(prog: *mut RegProg, in_use: bool) {
    if !prog.is_null() {
        (*prog).re_in_use = in_use;
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Get the engine used by a compiled program.
///
/// Returns 0 for BT, 1 for NFA.
///
/// # Safety
/// `prog` must be null or a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_regprog_get_engine(prog: *const RegProg) -> c_int {
    if prog.is_null() {
        0
    } else {
        (*prog).engine
    }
}

/// Check if program is NFA.
///
/// # Safety
/// `prog` must be null or a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_is_nfa_prog(prog: *const RegProg) -> c_int {
    c_int::from(is_nfa_prog(prog))
}

/// Check if program is BT.
///
/// # Safety
/// `prog` must be null or a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_is_bt_prog(prog: *const RegProg) -> c_int {
    c_int::from(is_bt_prog(prog))
}

/// Get program flags.
///
/// # Safety
/// `prog` must be null or a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_get_regflags(prog: *const RegProg) -> c_int {
    get_regflags(prog)
}

/// Check if program is in use.
///
/// # Safety
/// `prog` must be null or a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_prog_in_use(prog: *const RegProg) -> c_int {
    c_int::from(prog_in_use(prog))
}

/// Set program in use flag.
///
/// # Safety
/// `prog` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_set_prog_in_use(prog: *mut RegProg, in_use: c_int) {
    set_prog_in_use(prog, in_use != 0);
}

/// Convert engine enum to integer.
#[no_mangle]
pub extern "C" fn rs_engine_to_int(engine: RegEngine) -> c_int {
    engine.into()
}

/// Convert integer to engine enum.
#[no_mangle]
pub extern "C" fn rs_int_to_engine(v: c_int) -> RegEngine {
    RegEngine::from(v)
}

/// Get the start position of a submatch (single-line).
///
/// # Safety
/// `rm` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_regmatch_get_startp(rm: *const RegMatch, idx: c_int) -> *const u8 {
    if rm.is_null() || idx < 0 || idx >= NSUBEXP as c_int {
        ptr::null()
    } else {
        (*rm).startp[idx as usize]
    }
}

/// Get the end position of a submatch (single-line).
///
/// # Safety
/// `rm` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_regmatch_get_endp(rm: *const RegMatch, idx: c_int) -> *const u8 {
    if rm.is_null() || idx < 0 || idx >= NSUBEXP as c_int {
        ptr::null()
    } else {
        (*rm).endp[idx as usize]
    }
}

/// Set the start position of a submatch (single-line).
///
/// # Safety
/// `rm` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_regmatch_set_startp(rm: *mut RegMatch, idx: c_int, pos: *const u8) {
    if !rm.is_null() && idx >= 0 && idx < NSUBEXP as c_int {
        (*rm).startp[idx as usize] = pos;
    }
}

/// Set the end position of a submatch (single-line).
///
/// # Safety
/// `rm` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_regmatch_set_endp(rm: *mut RegMatch, idx: c_int, pos: *const u8) {
    if !rm.is_null() && idx >= 0 && idx < NSUBEXP as c_int {
        (*rm).endp[idx as usize] = pos;
    }
}

/// Get the start line of a submatch (multi-line).
///
/// # Safety
/// `rm` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_regmmatch_get_start_lnum(rm: *const RegMmatch, idx: c_int) -> c_int {
    if rm.is_null() || idx < 0 || idx >= NSUBEXP as c_int {
        0
    } else {
        (*rm).startpos[idx as usize].lnum
    }
}

/// Get the start column of a submatch (multi-line).
///
/// # Safety
/// `rm` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_regmmatch_get_start_col(rm: *const RegMmatch, idx: c_int) -> c_int {
    if rm.is_null() || idx < 0 || idx >= NSUBEXP as c_int {
        0
    } else {
        (*rm).startpos[idx as usize].col
    }
}

/// Get the end line of a submatch (multi-line).
///
/// # Safety
/// `rm` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_regmmatch_get_end_lnum(rm: *const RegMmatch, idx: c_int) -> c_int {
    if rm.is_null() || idx < 0 || idx >= NSUBEXP as c_int {
        0
    } else {
        (*rm).endpos[idx as usize].lnum
    }
}

/// Get the end column of a submatch (multi-line).
///
/// # Safety
/// `rm` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_regmmatch_get_end_col(rm: *const RegMmatch, idx: c_int) -> c_int {
    if rm.is_null() || idx < 0 || idx >= NSUBEXP as c_int {
        0
    } else {
        (*rm).endpos[idx as usize].col
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reg_engine_conversion() {
        assert_eq!(RegEngine::from(0), RegEngine::Bt);
        assert_eq!(RegEngine::from(1), RegEngine::Nfa);
        assert_eq!(RegEngine::from(2), RegEngine::Auto);
        assert_eq!(RegEngine::from(99), RegEngine::Auto); // Unknown -> Auto

        assert_eq!(c_int::from(RegEngine::Bt), 0);
        assert_eq!(c_int::from(RegEngine::Nfa), 1);
        assert_eq!(c_int::from(RegEngine::Auto), 2);
    }

    #[test]
    fn test_reg_engine_default() {
        assert_eq!(RegEngine::default(), RegEngine::Nfa);
    }

    #[test]
    fn test_re_flags() {
        assert_eq!(RE_MAGIC, 1);
        assert_eq!(RE_STRING, 2);
        assert_eq!(RE_STRICT, 4);
        assert_eq!(RE_VIM, 8);
        assert_eq!(RE_SEARCH, 16);
        assert_eq!(RE_AUTO, 32);
    }

    #[test]
    fn test_rf_flags() {
        assert_eq!(RF_ICASE, 0x01);
        assert_eq!(RF_NOICASE, 0x02);
        assert_eq!(RF_HASNL, 0x04);
        assert_eq!(RF_ICOMBINE, 0x08);
        assert_eq!(RF_LOOKBH, 0x10);
        assert_eq!(RF_NOSUB, 0x20);
        assert_eq!(RF_ZSUBEXPR, 0x40);
    }

    #[test]
    fn test_lpos_default() {
        let pos = LPos::default();
        assert_eq!(pos.lnum, 0);
        assert_eq!(pos.col, 0);
        assert_eq!(pos.coladd, 0);
    }

    #[test]
    fn test_regmatch_default() {
        let rm = RegMatch::default();
        assert!(rm.regprog.is_null());
        assert!(!rm.rm_ic);
        for p in &rm.startp {
            assert!(p.is_null());
        }
        for p in &rm.endp {
            assert!(p.is_null());
        }
    }

    #[test]
    fn test_regmmatch_default() {
        let rm = RegMmatch::default();
        assert!(rm.regprog.is_null());
        assert!(!rm.rmm_ic);
        assert_eq!(rm.rmm_maxcol, 0);
        for pos in &rm.startpos {
            assert_eq!(pos.lnum, 0);
            assert_eq!(pos.col, 0);
        }
    }

    #[test]
    fn test_nsubexp() {
        assert_eq!(NSUBEXP, 10);
    }
}
