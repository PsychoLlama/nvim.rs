//! Pattern matching helper functions
//!
//! This module provides helper functions for pattern compilation,
//! search argument initialization, and pattern matching utilities.

use std::ffi::{c_char, c_int};

use crate::state;

// =============================================================================
// C External Functions
// =============================================================================

extern "C" {
    // Option accessors
    fn nvim_get_p_ic() -> c_int;
    fn nvim_get_p_scs() -> c_int;
    fn nvim_get_p_ws() -> c_int;
    fn nvim_get_p_hls() -> c_int;
    fn nvim_get_no_hlsearch() -> c_int;
    fn nvim_get_no_smartcase() -> c_int;
    fn nvim_set_no_smartcase(val: c_int);

    // Search state
    fn nvim_get_search_match_lines() -> c_int;
    fn nvim_get_search_match_endcol() -> c_int;

    // Magic
    fn nvim_option_get_magic_overruled() -> c_int;
    fn nvim_option_get_magic() -> c_int;
}

// =============================================================================
// Search Options
// =============================================================================

/// Search option flags (from search.h)
pub mod options {
    use std::ffi::c_int;

    /// Reverse direction from previous search
    pub const SEARCH_REV: c_int = 0x01;
    /// Echo the search command and handle options
    pub const SEARCH_ECHO: c_int = 0x02;
    /// Give messages (combination flag)
    pub const SEARCH_MSG: c_int = 0x0c;
    /// Give all messages except not found
    pub const SEARCH_NFMSG: c_int = 0x08;
    /// Interpret optional flags
    pub const SEARCH_OPT: c_int = 0x10;
    /// Put search pattern in history
    pub const SEARCH_HIS: c_int = 0x20;
    /// Put cursor at end of match
    pub const SEARCH_END: c_int = 0x40;
    /// Don't add offset to position
    pub const SEARCH_NOOF: c_int = 0x80;
    /// Start search without col offset
    pub const SEARCH_START: c_int = 0x100;
    /// Set previous context mark
    pub const SEARCH_MARK: c_int = 0x200;
    /// Keep previous search pattern
    pub const SEARCH_KEEP: c_int = 0x400;
    /// Peek for typed char, cancel search
    pub const SEARCH_PEEK: c_int = 0x800;
    /// Start at specified column instead of zero
    pub const SEARCH_COL: c_int = 0x1000;
}

/// Check if options include the reverse flag.
#[inline]
pub fn has_search_rev(opts: c_int) -> bool {
    (opts & options::SEARCH_REV) != 0
}

/// Check if options include the echo flag.
#[inline]
pub fn has_search_echo(opts: c_int) -> bool {
    (opts & options::SEARCH_ECHO) != 0
}

/// Check if options include the message flag.
#[inline]
pub fn has_search_msg(opts: c_int) -> bool {
    (opts & options::SEARCH_MSG) != 0
}

/// Check if options include the not-found message flag.
#[inline]
pub fn has_search_nfmsg(opts: c_int) -> bool {
    (opts & options::SEARCH_NFMSG) != 0
}

/// Check if options include the opt flag.
#[inline]
pub fn has_search_opt(opts: c_int) -> bool {
    (opts & options::SEARCH_OPT) != 0
}

/// Check if options include the history flag.
#[inline]
pub fn has_search_his(opts: c_int) -> bool {
    (opts & options::SEARCH_HIS) != 0
}

/// Check if options include the end flag.
#[inline]
pub fn has_search_end(opts: c_int) -> bool {
    (opts & options::SEARCH_END) != 0
}

/// Check if options include the no-offset flag.
#[inline]
pub fn has_search_noof(opts: c_int) -> bool {
    (opts & options::SEARCH_NOOF) != 0
}

/// Check if options include the start flag.
#[inline]
pub fn has_search_start(opts: c_int) -> bool {
    (opts & options::SEARCH_START) != 0
}

/// Check if options include the mark flag.
#[inline]
pub fn has_search_mark(opts: c_int) -> bool {
    (opts & options::SEARCH_MARK) != 0
}

/// Check if options include the keep flag.
#[inline]
pub fn has_search_keep(opts: c_int) -> bool {
    (opts & options::SEARCH_KEEP) != 0
}

/// Check if options include the peek flag.
#[inline]
pub fn has_search_peek(opts: c_int) -> bool {
    (opts & options::SEARCH_PEEK) != 0
}

/// Check if options include the column flag.
#[inline]
pub fn has_search_col(opts: c_int) -> bool {
    (opts & options::SEARCH_COL) != 0
}

// =============================================================================
// Option Accessors
// =============================================================================

/// optmagic_T values from regexp_defs.h
#[allow(dead_code)]
const OPTION_MAGIC_NOT_SET: c_int = 0;
const OPTION_MAGIC_ON: c_int = 1;
const OPTION_MAGIC_OFF: c_int = 2;

/// Get the 'ignorecase' option value.
#[inline]
pub fn get_p_ic() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_p_ic() != 0 }
}

/// Get the 'smartcase' option value.
#[inline]
pub fn get_p_scs() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_p_scs() != 0 }
}

/// Get the 'wrapscan' option value.
#[inline]
pub fn get_p_ws() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_p_ws() != 0 }
}

/// Get the 'hlsearch' option value.
#[inline]
pub fn get_p_hls() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_p_hls() != 0 }
}

/// Get the no_hlsearch state.
#[inline]
pub fn get_no_hlsearch() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_no_hlsearch() != 0 }
}

/// Get the no_smartcase state.
#[inline]
pub fn get_no_smartcase() -> bool {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_no_smartcase() != 0 }
}

/// Set the no_smartcase state.
#[inline]
pub fn set_no_smartcase(val: bool) {
    // SAFETY: Simple global setter
    unsafe { nvim_set_no_smartcase(c_int::from(val)) }
}

/// Get the value of 'magic' taking "magic_overruled" into account.
#[inline]
pub fn magic_isset() -> bool {
    // SAFETY: Simple global accessors
    unsafe {
        match nvim_option_get_magic_overruled() {
            OPTION_MAGIC_ON => true,
            OPTION_MAGIC_OFF => false,
            _ => nvim_option_get_magic() != 0,
        }
    }
}

// =============================================================================
// Search Match State
// =============================================================================

/// Get the number of lines in the current search match.
#[inline]
pub fn get_search_match_lines() -> c_int {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_search_match_lines() }
}

/// Get the end column of the current search match.
#[inline]
pub fn get_search_match_endcol() -> c_int {
    // SAFETY: Simple global accessor
    unsafe { nvim_get_search_match_endcol() }
}

// =============================================================================
// Searchit Argument Helpers
// =============================================================================

/// Arguments for searchit() function.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SearchitArg {
    /// Stop after this line number when != 0
    pub sa_stop_lnum: i32,
    /// Timeout limit pointer (opaque in Rust)
    pub sa_tm: *const core::ffi::c_void,
    /// Set when timed out
    pub sa_timed_out: c_int,
    /// Search wrapped around
    pub sa_wrapped: c_int,
}

impl Default for SearchitArg {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchitArg {
    /// Create a new SearchitArg with default values.
    pub const fn new() -> Self {
        Self {
            sa_stop_lnum: 0,
            sa_tm: core::ptr::null(),
            sa_timed_out: 0,
            sa_wrapped: 0,
        }
    }

    /// Check if the search timed out.
    #[inline]
    pub fn timed_out(&self) -> bool {
        self.sa_timed_out != 0
    }

    /// Check if the search wrapped around.
    #[inline]
    pub fn wrapped(&self) -> bool {
        self.sa_wrapped != 0
    }

    /// Set the stop line number.
    #[inline]
    pub fn with_stop_lnum(mut self, lnum: i32) -> Self {
        self.sa_stop_lnum = lnum;
        self
    }
}

// =============================================================================
// Pattern Index Determination
// =============================================================================

/// Determine which pattern index to use based on pat_use parameter.
///
/// If pat_use is RE_LAST, returns the last_idx; otherwise returns pat_use.
#[inline]
pub fn resolve_pattern_index(pat_use: c_int) -> c_int {
    if pat_use == state::RE_LAST {
        state::get_last_idx()
    } else {
        pat_use
    }
}

/// Check if a pattern save operation should save to the search pattern.
#[inline]
pub fn should_save_search(pat_save: c_int) -> bool {
    pat_save == state::RE_SEARCH || pat_save == state::RE_BOTH
}

/// Check if a pattern save operation should save to the substitute pattern.
#[inline]
pub fn should_save_subst(pat_save: c_int) -> bool {
    pat_save == state::RE_SUBST || pat_save == state::RE_BOTH
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI: Initialize a searchit_arg structure.
///
/// # Safety
/// The caller must ensure `arg` points to valid, properly aligned memory.
#[no_mangle]
pub unsafe extern "C" fn rs_searchit_arg_init(arg: *mut SearchitArg) {
    if !arg.is_null() {
        *arg = SearchitArg::new();
    }
}

/// FFI: Check if search options have REV flag.
#[no_mangle]
pub extern "C" fn rs_has_search_rev(opts: c_int) -> c_int {
    c_int::from(has_search_rev(opts))
}

/// FFI: Check if search options have ECHO flag.
#[no_mangle]
pub extern "C" fn rs_has_search_echo(opts: c_int) -> c_int {
    c_int::from(has_search_echo(opts))
}

/// FFI: Check if search options have MSG flag.
#[no_mangle]
pub extern "C" fn rs_has_search_msg(opts: c_int) -> c_int {
    c_int::from(has_search_msg(opts))
}

/// FFI: Check if search options have HIS flag.
#[no_mangle]
pub extern "C" fn rs_has_search_his(opts: c_int) -> c_int {
    c_int::from(has_search_his(opts))
}

/// FFI: Check if search options have END flag.
#[no_mangle]
pub extern "C" fn rs_has_search_end(opts: c_int) -> c_int {
    c_int::from(has_search_end(opts))
}

/// FFI: Check if search options have START flag.
#[no_mangle]
pub extern "C" fn rs_has_search_start(opts: c_int) -> c_int {
    c_int::from(has_search_start(opts))
}

/// FFI: Check if search options have KEEP flag.
#[no_mangle]
pub extern "C" fn rs_has_search_keep(opts: c_int) -> c_int {
    c_int::from(has_search_keep(opts))
}

/// FFI: Check if search options have PEEK flag.
#[no_mangle]
pub extern "C" fn rs_has_search_peek(opts: c_int) -> c_int {
    c_int::from(has_search_peek(opts))
}

/// FFI: Check if search options have COL flag.
#[no_mangle]
pub extern "C" fn rs_has_search_col(opts: c_int) -> c_int {
    c_int::from(has_search_col(opts))
}

/// FFI: Get 'ignorecase' option.
#[no_mangle]
pub extern "C" fn rs_get_p_ic() -> c_int {
    c_int::from(get_p_ic())
}

/// FFI: Get 'smartcase' option.
#[no_mangle]
pub extern "C" fn rs_get_p_scs() -> c_int {
    c_int::from(get_p_scs())
}

/// FFI: Get 'wrapscan' option.
#[no_mangle]
pub extern "C" fn rs_get_p_ws() -> c_int {
    c_int::from(get_p_ws())
}

/// FFI: Get 'hlsearch' option.
#[no_mangle]
pub extern "C" fn rs_get_p_hls() -> c_int {
    c_int::from(get_p_hls())
}

/// FFI: Get no_hlsearch state.
#[no_mangle]
pub extern "C" fn rs_get_no_hlsearch() -> c_int {
    c_int::from(get_no_hlsearch())
}

/// FFI: Get no_smartcase state.
#[no_mangle]
pub extern "C" fn rs_get_no_smartcase() -> c_int {
    c_int::from(get_no_smartcase())
}

/// FFI: Set no_smartcase state.
#[no_mangle]
pub extern "C" fn rs_set_no_smartcase(val: c_int) {
    set_no_smartcase(val != 0);
}

/// FFI: Get magic_isset value.
#[no_mangle]
pub extern "C" fn rs_helpers_magic_isset() -> c_int {
    c_int::from(magic_isset())
}

/// FFI: Get search match lines.
#[no_mangle]
pub extern "C" fn rs_get_search_match_lines() -> c_int {
    get_search_match_lines()
}

/// FFI: Get search match end column.
#[no_mangle]
pub extern "C" fn rs_get_search_match_endcol() -> c_int {
    get_search_match_endcol()
}

/// FFI: Resolve pattern index (RE_LAST -> actual index).
#[no_mangle]
pub extern "C" fn rs_resolve_pattern_index(pat_use: c_int) -> c_int {
    resolve_pattern_index(pat_use)
}

/// FFI: Check if should save to search pattern.
#[no_mangle]
pub extern "C" fn rs_should_save_search(pat_save: c_int) -> c_int {
    c_int::from(should_save_search(pat_save))
}

/// FFI: Check if should save to substitute pattern.
#[no_mangle]
pub extern "C" fn rs_should_save_subst(pat_save: c_int) -> c_int {
    c_int::from(should_save_subst(pat_save))
}

/// FFI: Get SEARCH_REV constant.
#[no_mangle]
pub extern "C" fn rs_search_opt_rev() -> c_int {
    options::SEARCH_REV
}

/// FFI: Get SEARCH_ECHO constant.
#[no_mangle]
pub extern "C" fn rs_search_opt_echo() -> c_int {
    options::SEARCH_ECHO
}

/// FFI: Get SEARCH_OPT constant.
#[no_mangle]
pub extern "C" fn rs_search_opt_opt() -> c_int {
    options::SEARCH_OPT
}

/// FFI: Get SEARCH_NOOF constant.
#[no_mangle]
pub extern "C" fn rs_search_opt_noof() -> c_int {
    options::SEARCH_NOOF
}

/// FFI: Get SEARCH_MARK constant.
#[no_mangle]
pub extern "C" fn rs_search_opt_mark() -> c_int {
    options::SEARCH_MARK
}

// =============================================================================
// Pattern Case Sensitivity
// =============================================================================

// External C functions for line/buffer access
extern "C" {
    /// Get skipwhite(ml_get(lnum)) for curbuf.
    fn nvim_search_skipwhite_ml_get(lnum: i32) -> *const c_char;
}

// External C functions for pattern analysis
extern "C" {
    /// Check if character is uppercase (multibyte aware).
    fn nvim_mb_isupper(c: c_int) -> c_int;

    /// Get UTF character from byte pointer.
    fn nvim_utf_ptr2char(p: *const std::ffi::c_char) -> c_int;

    /// Get UTF character length including composing chars.
    fn nvim_utfc_ptr2len(p: *const std::ffi::c_char) -> c_int;

    /// Skip regexp to find magic value.
    /// Returns pointer past pattern, sets magic_val.
    fn rs_skip_regexp_ex(
        startp: *mut std::ffi::c_char,
        dirc: c_int,
        magic: c_int,
        newp: *mut *mut std::ffi::c_char,
        dropped: *mut c_int,
        magic_val: *mut c_int,
    ) -> *mut std::ffi::c_char;

    /// Check if ctrl-x mode is not default (direct Rust, no C roundtrip).
    fn rs_ctrl_x_mode_not_default() -> c_int;

    /// Get curbuf->b_p_inf (infercase option).
    fn nvim_curbuf_get_b_p_inf() -> c_int;
}

/// Magic mode values (from regexp_defs.h)
pub mod magic {
    use std::ffi::c_int;

    /// No magic at all
    pub const MAGIC_NONE: c_int = 1;
    /// 'nomagic' or \M
    pub const MAGIC_OFF: c_int = 2;
    /// 'magic' or \m (default)
    pub const MAGIC_ON: c_int = 3;
    /// Very magic \v
    pub const MAGIC_ALL: c_int = 4;
}

/// Check if pattern has an uppercase character.
///
/// This follows the logic of `pat_has_uppercase()` in search.c.
/// Skips backslash-escaped characters in magic mode.
///
/// # Safety
/// `pat` must be a valid null-terminated C string.
pub unsafe fn pat_has_uppercase(pat: *const std::ffi::c_char) -> bool {
    if pat.is_null() {
        return false;
    }

    // Get the magicness of the pattern
    let mut magic_val: c_int = magic::MAGIC_ON;
    rs_skip_regexp_ex(
        pat as *mut std::ffi::c_char,
        0, // NUL - no delimiter
        c_int::from(magic_isset()),
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        &mut magic_val,
    );

    let mut p = pat;
    while *p != 0 {
        let l = nvim_utfc_ptr2len(p);

        if l > 1 {
            // Multi-byte character
            let c = nvim_utf_ptr2char(p);
            if nvim_mb_isupper(c) != 0 {
                return true;
            }
            p = p.add(l as usize);
        } else if *p == b'\\' as i8 && magic_val <= magic::MAGIC_ON {
            // Backslash escape in magic mode
            let next = *p.add(1);
            if next == b'_' as i8 && *p.add(2) != 0 {
                // skip "\_X"
                p = p.add(3);
            } else if next == b'%' as i8 && *p.add(2) != 0 {
                // skip "\%X"
                p = p.add(3);
            } else if next != 0 {
                // skip "\X"
                p = p.add(2);
            } else {
                p = p.add(1);
            }
        } else if (*p == b'%' as i8 || *p == b'_' as i8) && magic_val == magic::MAGIC_ALL {
            // In very magic mode, % and _ are special
            let next = *p.add(1);
            if next != 0 {
                p = p.add(2);
            } else {
                p = p.add(1);
            }
        } else {
            // Single byte character - check if uppercase
            let c = *p as u8;
            if nvim_mb_isupper(c_int::from(c)) != 0 {
                return true;
            }
            p = p.add(1);
        }
    }
    false
}

/// Determine case sensitivity for a pattern.
///
/// This is the Rust equivalent of `ignorecase()` in search.c.
/// Uses p_ic and p_scs global options.
///
/// # Safety
/// `pat` must be a valid null-terminated C string.
pub unsafe fn ignorecase(pat: *const std::ffi::c_char) -> bool {
    ignorecase_opt(pat, get_p_ic(), get_p_scs())
}

/// Determine case sensitivity with explicit options.
///
/// This is the Rust equivalent of `ignorecase_opt()` in search.c.
///
/// # Safety
/// `pat` must be a valid null-terminated C string.
pub unsafe fn ignorecase_opt(pat: *const std::ffi::c_char, ic_in: bool, scs: bool) -> bool {
    let mut ic = ic_in;

    if ic && !get_no_smartcase() && scs {
        // Check for ctrl-x mode with infercase
        let in_ctrl_x_with_inf =
            rs_ctrl_x_mode_not_default() != 0 && nvim_curbuf_get_b_p_inf() != 0;

        if !in_ctrl_x_with_inf {
            // Smartcase: if pattern has uppercase, don't ignore case
            ic = !pat_has_uppercase(pat);
        }
    }

    // Clear no_smartcase after checking
    set_no_smartcase(false);

    ic
}

/// FFI: Check if pattern has uppercase.
///
/// # Safety
/// `pat` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_pat_has_uppercase(pat: *const std::ffi::c_char) -> c_int {
    c_int::from(pat_has_uppercase(pat))
}

/// FFI: Determine case sensitivity for pattern.
///
/// # Safety
/// `pat` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_ignorecase(pat: *const std::ffi::c_char) -> c_int {
    c_int::from(ignorecase(pat))
}

/// FFI: Determine case sensitivity with explicit options.
///
/// # Safety
/// `pat` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_ignorecase_opt(
    pat: *const std::ffi::c_char,
    ic: c_int,
    scs: c_int,
) -> c_int {
    c_int::from(ignorecase_opt(pat, ic != 0, scs != 0))
}

// =============================================================================
// Line Content Helpers
// =============================================================================

/// Check if line 'lnum' is empty or has white chars only.
///
/// This is the Rust equivalent of `linewhite()` in search.c.
///
/// # Safety
/// Calls C accessor to get line content.
pub unsafe fn linewhite(lnum: i32) -> bool {
    let p = nvim_search_skipwhite_ml_get(lnum);
    !p.is_null() && *p == 0
}

/// FFI: Check if line is empty or whitespace only.
///
/// # Safety
/// Calls C accessor to get line content from curbuf.
#[no_mangle]
pub unsafe extern "C" fn rs_search_linewhite(lnum: i32) -> c_int {
    c_int::from(linewhite(lnum))
}

// =============================================================================
// Phase 7a: check_linecomment
// =============================================================================

extern "C" {
    fn nvim_curbuf_is_lisp() -> c_int;
    fn nvim_is_pos_in_string(line: *const c_char, col: c_int) -> c_int;
}

/// MAXCOL constant (matches C MAXCOL = 0x7fffffff).
const MAXCOL: c_int = 0x7fffffff;

/// Check for a line comment in a line.
///
/// For Lisp, checks for `;` comments outside of strings.
/// For other languages, checks for `//` comments outside of strings.
///
/// Returns the column of the comment start, or MAXCOL if none found.
///
/// # Safety
/// `line` must be a valid null-terminated C string.
#[unsafe(export_name = "check_linecomment")]
pub unsafe extern "C" fn rs_check_linecomment(line: *const c_char) -> c_int {
    if line.is_null() {
        return MAXCOL;
    }

    let bytes = line as *const u8;

    if nvim_curbuf_is_lisp() != 0 {
        // Lisp mode: scan for ';' outside of strings
        // First check if there's a ';' at all
        let mut has_semicolon = false;
        let mut i: usize = 0;
        while *bytes.add(i) != 0 {
            if *bytes.add(i) == b';' {
                has_semicolon = true;
                break;
            }
            i += 1;
        }

        if !has_semicolon {
            return MAXCOL;
        }

        // Scan for '"' and ';' handling string state
        let mut in_str = false;
        let mut idx: usize = 0;
        loop {
            // Find next '"' or ';'
            while *bytes.add(idx) != 0 && *bytes.add(idx) != b'"' && *bytes.add(idx) != b';' {
                idx += 1;
            }
            if *bytes.add(idx) == 0 {
                return MAXCOL;
            }

            if *bytes.add(idx) == b'"' {
                if in_str {
                    // In string: skip escaped quote
                    if idx >= 1 && *bytes.add(idx - 1) != b'\\' {
                        in_str = false;
                    }
                } else if idx == 0
                    || (idx >= 2 && *bytes.add(idx - 1) != b'\\' && *bytes.add(idx - 2) != b'#')
                {
                    // Not #\" form
                    in_str = true;
                }
            } else {
                // ';' found
                if !in_str
                    && (idx < 2 || (*bytes.add(idx - 1) != b'\\' && *bytes.add(idx - 2) != b'#'))
                    && nvim_is_pos_in_string(line, idx as c_int) == 0
                {
                    return idx as c_int;
                }
            }
            idx += 1;
        }
    } else {
        // Non-lisp: scan for '//' outside strings
        let mut idx: usize = 0;
        loop {
            // Find next '/'
            while *bytes.add(idx) != 0 && *bytes.add(idx) != b'/' {
                idx += 1;
            }
            if *bytes.add(idx) == 0 {
                return MAXCOL;
            }

            // Accept a double /, unless preceded with * and followed by *,
            // because * / / * is an end and start of a C comment.
            if *bytes.add(idx + 1) == b'/'
                && (idx == 0 || *bytes.add(idx - 1) != b'*' || *bytes.add(idx + 2) != b'*')
                && nvim_is_pos_in_string(line, idx as c_int) == 0
            {
                return idx as c_int;
            }
            idx += 1;
        }
    }
}

// =============================================================================
// Phase 7b: is_zero_width
// =============================================================================

/// Direction constants.
const FORWARD: c_int = 1;
#[allow(dead_code)]
const BACKWARD: c_int = -1;
/// FAIL constant.
const FAIL: c_int = 0;

extern "C" {
    fn nvim_get_called_emsg() -> c_int;
    fn nvim_get_last_spat_pat(out_len: *mut usize) -> *const c_char;
    fn nvim_regmmatch_alloc() -> *mut c_void;
    fn nvim_regmmatch_free(regmatch: *mut c_void);
    fn nvim_is_zero_width_regcomp(
        pat: *const c_char,
        patlen: usize,
        regmatch: *mut c_void,
    ) -> c_int;
    fn nvim_regmatch_set_startcol(regmatch: *mut c_void, col: c_int);
    fn nvim_regmatch_get_startcol(regmatch: *const c_void) -> c_int;
    fn nvim_regmatch_get_startlnum(regmatch: *const c_void) -> c_int;
    fn nvim_regmatch_get_endcol(regmatch: *const c_void) -> c_int;
    fn nvim_regmatch_get_endlnum(regmatch: *const c_void) -> c_int;
    fn nvim_is_zero_width_regexec(regmatch: *mut c_void, lnum: c_int, col: c_int) -> c_int;
    fn nvim_is_zero_width_searchit(
        pat: *const c_char,
        patlen: usize,
        dir: c_int,
        flags: c_int,
        pos_lnum: *mut c_int,
        pos_col: *mut c_int,
        pos_coladd: *mut c_int,
    ) -> c_int;
}

use std::ffi::c_void;

/// Check if the current search pattern matches a zero-width string.
///
/// Returns 1 if zero-width match, 0 if not, -1 if pattern not found.
///
/// # Safety
/// `pattern` may be null (uses last search pattern). If non-null, must be valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_is_zero_width(
    pattern: *const c_char,
    patternlen: usize,
    do_move: bool,
    cur_lnum: c_int,
    cur_col: c_int,
    cur_coladd: c_int,
    direction: c_int,
) -> c_int {
    let mut result: c_int = -1;

    let (pat, patlen) = if pattern.is_null() {
        let mut len: usize = 0;
        let p = nvim_get_last_spat_pat(&mut len);
        (p, len)
    } else {
        (pattern, patternlen)
    };

    // Allocate regmmatch_T on the heap
    let regmatch = nvim_regmmatch_alloc();
    if regmatch.is_null() {
        return -1;
    }

    if nvim_is_zero_width_regcomp(pat, patlen, regmatch) == FAIL {
        nvim_regmmatch_free(regmatch);
        return -1;
    }

    let called_emsg_before = nvim_get_called_emsg();

    // Init startcol correctly
    nvim_regmatch_set_startcol(regmatch, -1);

    // Move to match
    let mut pos_lnum: c_int;
    let mut pos_col: c_int;
    let mut pos_coladd: c_int;
    let flag: c_int;
    if do_move {
        pos_lnum = 0;
        pos_col = 0;
        pos_coladd = 0;
        flag = 0;
    } else {
        pos_lnum = cur_lnum;
        pos_col = cur_col;
        pos_coladd = cur_coladd;
        // accept a match at the cursor position
        flag = options::SEARCH_START;
    };

    if nvim_is_zero_width_searchit(
        pat,
        patlen,
        direction,
        flag,
        &mut pos_lnum,
        &mut pos_col,
        &mut pos_coladd,
    ) != FAIL
    {
        // Zero-width pattern should match somewhere, then we can check if
        // start and end are in the same position.
        let mut nmatched: c_int;
        loop {
            let startcol = nvim_regmatch_get_startcol(regmatch);
            nvim_regmatch_set_startcol(regmatch, startcol + 1);
            nmatched = nvim_is_zero_width_regexec(
                regmatch,
                pos_lnum,
                nvim_regmatch_get_startcol(regmatch),
            );
            if nmatched != 0 {
                break;
            }
            // Check loop condition: regprog may be NULL (checked implicitly by regexec returning 0)
            let startcol = nvim_regmatch_get_startcol(regmatch);
            let should_continue = if direction == FORWARD {
                startcol < pos_col
            } else {
                startcol > pos_col
            };
            if !should_continue {
                break;
            }
        }

        if nvim_get_called_emsg() == called_emsg_before {
            result = c_int::from(
                nmatched != 0
                    && nvim_regmatch_get_startlnum(regmatch) == nvim_regmatch_get_endlnum(regmatch)
                    && nvim_regmatch_get_startcol(regmatch) == nvim_regmatch_get_endcol(regmatch),
            );
        }
    }

    nvim_regmmatch_free(regmatch);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options() {
        assert_eq!(options::SEARCH_REV, 0x01);
        assert_eq!(options::SEARCH_ECHO, 0x02);
        assert_eq!(options::SEARCH_MSG, 0x0c);
        assert_eq!(options::SEARCH_NFMSG, 0x08);
        assert_eq!(options::SEARCH_OPT, 0x10);
        assert_eq!(options::SEARCH_HIS, 0x20);
        assert_eq!(options::SEARCH_END, 0x40);
        assert_eq!(options::SEARCH_NOOF, 0x80);
        assert_eq!(options::SEARCH_START, 0x100);
        assert_eq!(options::SEARCH_MARK, 0x200);
        assert_eq!(options::SEARCH_KEEP, 0x400);
        assert_eq!(options::SEARCH_PEEK, 0x800);
        assert_eq!(options::SEARCH_COL, 0x1000);
    }

    #[test]
    fn test_has_search_flags() {
        let opts = options::SEARCH_REV | options::SEARCH_HIS | options::SEARCH_END;
        assert!(has_search_rev(opts));
        assert!(has_search_his(opts));
        assert!(has_search_end(opts));
        assert!(!has_search_echo(opts));
        assert!(!has_search_start(opts));
    }

    #[test]
    fn test_searchit_arg_new() {
        let arg = SearchitArg::new();
        assert_eq!(arg.sa_stop_lnum, 0);
        assert!(arg.sa_tm.is_null());
        assert_eq!(arg.sa_timed_out, 0);
        assert_eq!(arg.sa_wrapped, 0);
    }

    #[test]
    fn test_searchit_arg_with_stop_lnum() {
        let arg = SearchitArg::new().with_stop_lnum(100);
        assert_eq!(arg.sa_stop_lnum, 100);
    }

    #[test]
    fn test_should_save_patterns() {
        assert!(should_save_search(state::RE_SEARCH));
        assert!(should_save_search(state::RE_BOTH));
        assert!(!should_save_search(state::RE_SUBST));

        assert!(!should_save_subst(state::RE_SEARCH));
        assert!(should_save_subst(state::RE_BOTH));
        assert!(should_save_subst(state::RE_SUBST));
    }
}
