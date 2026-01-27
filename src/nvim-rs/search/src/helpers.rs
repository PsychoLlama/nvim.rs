//! Pattern matching helper functions
//!
//! This module provides helper functions for pattern compilation,
//! search argument initialization, and pattern matching utilities.

use std::ffi::c_int;

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
    fn nvim_get_magic_overruled() -> c_int;
    fn nvim_get_p_magic() -> c_int;
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
        match nvim_get_magic_overruled() {
            OPTION_MAGIC_ON => true,
            OPTION_MAGIC_OFF => false,
            _ => nvim_get_p_magic() != 0,
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
    fn nvim_skip_regexp_ex(
        startp: *const std::ffi::c_char,
        dirc: c_int,
        magic: c_int,
        newp: *mut *mut std::ffi::c_char,
        dropped: *mut c_int,
        magic_val: *mut c_int,
    ) -> *const std::ffi::c_char;

    /// Check if ctrl-x mode is not default.
    fn nvim_ctrl_x_mode_not_default() -> c_int;

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
    nvim_skip_regexp_ex(
        pat,
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
            nvim_ctrl_x_mode_not_default() != 0 && nvim_curbuf_get_b_p_inf() != 0;

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
