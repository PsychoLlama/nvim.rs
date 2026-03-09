//! Search-related utilities for Neovim
//!
//! This crate provides Rust implementations of search-related functions
//! from `src/nvim/search.c`. It uses an accessor pattern where
//! static variables are accessed through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe

pub mod commands;
pub mod core;
pub mod direction;
pub mod do_search;
pub mod helpers;
pub mod incsearch;
pub mod matchparen;
pub mod path_search;
pub mod pattern;
pub mod searchit;
pub mod state;
pub mod stats;
pub mod substitute;

use std::ffi::{c_char, c_int};

// C accessor functions for search state.
// These are defined in search.c and provide safe access to static variables.
extern "C" {
    /// Get the `lastcdir` static variable (FORWARD=1, BACKWARD=-1).
    fn nvim_get_lastcdir() -> c_int;

    /// Get the `last_t_cmd` static variable.
    fn nvim_get_last_t_cmd() -> c_int;

    /// Get the `lastc_bytes` static variable.
    fn nvim_get_lastc_bytes() -> *const c_char;

    /// Get the `last_idx` static variable.
    fn nvim_get_last_idx() -> c_int;

    /// Get whether last vim_regcomp() found EOL.
    fn nvim_regexp_get_had_eol() -> c_int;

    /// Get the `magic_overruled` global value.
    fn nvim_option_get_magic_overruled() -> c_int;

    /// Get the `p_magic` global value.
    fn nvim_option_get_magic() -> c_int;
}

/// Direction constant for FORWARD.
const FORWARD: c_int = 1;

/// optmagic_T values from regexp_defs.h
#[allow(dead_code)]
const OPTION_MAGIC_NOT_SET: c_int = 0;
const OPTION_MAGIC_ON: c_int = 1;
const OPTION_MAGIC_OFF: c_int = 2;

/// Check if last character search direction was forward.
///
/// This is the Rust equivalent of `last_csearch_forward()` in search.c.
#[inline]
fn last_csearch_forward_impl() -> bool {
    // SAFETY: nvim_get_lastcdir is a simple global accessor
    unsafe { nvim_get_lastcdir() == FORWARD }
}

/// FFI wrapper for `last_csearch_forward`.
///
/// Returns non-zero if the last search direction was forward.
#[unsafe(export_name = "last_csearch_forward")]
pub extern "C" fn rs_last_csearch_forward() -> c_int {
    c_int::from(last_csearch_forward_impl())
}

/// Check if last character search was a 't' command (until).
///
/// This is the Rust equivalent of `last_csearch_until()` in search.c.
#[inline]
fn last_csearch_until_impl() -> c_int {
    // SAFETY: nvim_get_last_t_cmd is a simple global accessor
    unsafe { nvim_get_last_t_cmd() }
}

/// FFI wrapper for `last_csearch_until`.
///
/// Returns non-zero if the last search was a 't' command.
#[unsafe(export_name = "last_csearch_until")]
pub extern "C" fn rs_last_csearch_until() -> c_int {
    last_csearch_until_impl()
}

/// Get the last character search bytes.
///
/// Returns a pointer to the static `lastc_bytes` array.
///
/// # Safety
///
/// Calls external C function to get pointer to static variable.
#[unsafe(export_name = "last_csearch")]
pub unsafe extern "C" fn rs_last_csearch() -> *const c_char {
    nvim_get_lastc_bytes()
}

/// Check if search pattern was the last used one.
///
/// Returns true if `last_idx == 0`, meaning the search pattern (not substitute)
/// was last used.
#[inline]
fn search_was_last_used_impl() -> bool {
    // SAFETY: nvim_get_last_idx is a simple global accessor
    unsafe { nvim_get_last_idx() == 0 }
}

/// FFI wrapper for `search_was_last_used`.
#[no_mangle]
pub extern "C" fn rs_search_was_last_used() -> c_int {
    c_int::from(search_was_last_used_impl())
}

/// C ABI export for `search_was_last_used`, returning `bool` to match C callers.
#[unsafe(export_name = "search_was_last_used")]
pub extern "C" fn search_was_last_used_export() -> bool {
    search_was_last_used_impl()
}

/// Check if during the previous call to `vim_regcomp` the EOL item "$" was found.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[no_mangle]
pub unsafe extern "C" fn rs_vim_regcomp_had_eol() -> c_int {
    nvim_regexp_get_had_eol()
}

/// Get the value of 'magic' taking "magic_overruled" into account.
///
/// This is the Rust equivalent of `magic_isset()` in option.c.
///
/// # Safety
/// Calls C accessor functions for global variables.
#[inline]
fn magic_isset_impl() -> bool {
    unsafe {
        match nvim_option_get_magic_overruled() {
            OPTION_MAGIC_ON => true,
            OPTION_MAGIC_OFF => false,
            _ => nvim_option_get_magic() != 0,
        }
    }
}

/// FFI wrapper for `magic_isset`.
///
/// Returns non-zero if magic is set.
#[no_mangle]
pub extern "C" fn rs_magic_isset() -> c_int {
    c_int::from(magic_isset_impl())
}

// =============================================================================
// Search Option Constants
// =============================================================================

/// Search options (from search.h)
pub mod search_options {
    use std::ffi::c_int;

    /// Show search messages
    pub const SEARCH_MSG: c_int = 0x01;
    /// Only show "not found" message
    pub const SEARCH_NFMSG: c_int = 0x02;
    /// Put search pattern in history
    pub const SEARCH_HIS: c_int = 0x08;
    /// Return position at end of match
    pub const SEARCH_END: c_int = 0x10;
    /// Accept match at pos itself
    pub const SEARCH_START: c_int = 0x20;
    /// Keep previous search pattern
    pub const SEARCH_KEEP: c_int = 0x40;
    /// Match only once in a closed fold
    pub const SEARCH_FOLD: c_int = 0x80;
    /// Check for typed char, cancel search
    pub const SEARCH_PEEK: c_int = 0x100;
    /// Start at pos->col instead of zero
    pub const SEARCH_COL: c_int = 0x200;
    /// Don't use strpbrk optimization
    pub const SEARCH_NOSTRPBRK: c_int = 0x400;
    /// Whole word search only
    pub const SEARCH_WW: c_int = 0x800;
}

/// Get search option for showing messages.
#[no_mangle]
pub extern "C" fn rs_search_opt_msg() -> c_int {
    search_options::SEARCH_MSG
}

/// Get search option for showing only not-found message.
#[no_mangle]
pub extern "C" fn rs_search_opt_nfmsg() -> c_int {
    search_options::SEARCH_NFMSG
}

/// Get search option for putting search in history.
#[no_mangle]
pub extern "C" fn rs_search_opt_his() -> c_int {
    search_options::SEARCH_HIS
}

/// Get search option for returning position at end of match.
#[no_mangle]
pub extern "C" fn rs_search_opt_end() -> c_int {
    search_options::SEARCH_END
}

/// Get search option for accepting match at start position.
#[no_mangle]
pub extern "C" fn rs_search_opt_start() -> c_int {
    search_options::SEARCH_START
}

/// Get search option for keeping previous pattern.
#[no_mangle]
pub extern "C" fn rs_search_opt_keep() -> c_int {
    search_options::SEARCH_KEEP
}

/// Get search option for matching in closed folds.
#[no_mangle]
pub extern "C" fn rs_search_opt_fold() -> c_int {
    search_options::SEARCH_FOLD
}

/// Get search option for peeking at typed chars.
#[no_mangle]
pub extern "C" fn rs_search_opt_peek() -> c_int {
    search_options::SEARCH_PEEK
}

/// Get search option for starting at column.
#[no_mangle]
pub extern "C" fn rs_search_opt_col() -> c_int {
    search_options::SEARCH_COL
}

/// Check if search options include MSG flag.
#[no_mangle]
pub extern "C" fn rs_search_has_msg(options: c_int) -> bool {
    (options & search_options::SEARCH_MSG) != 0
}

/// Check if search options include HIS flag.
#[no_mangle]
pub extern "C" fn rs_search_has_his(options: c_int) -> bool {
    (options & search_options::SEARCH_HIS) != 0
}

/// Check if search options include END flag.
#[no_mangle]
pub extern "C" fn rs_search_has_end(options: c_int) -> bool {
    (options & search_options::SEARCH_END) != 0
}

/// Check if search options include START flag.
#[no_mangle]
pub extern "C" fn rs_search_has_start(options: c_int) -> bool {
    (options & search_options::SEARCH_START) != 0
}

/// Check if search options include KEEP flag.
#[no_mangle]
pub extern "C" fn rs_search_has_keep(options: c_int) -> bool {
    (options & search_options::SEARCH_KEEP) != 0
}

/// Check if search options include FOLD flag.
#[no_mangle]
pub extern "C" fn rs_search_has_fold(options: c_int) -> bool {
    (options & search_options::SEARCH_FOLD) != 0
}

/// Check if search options include PEEK flag.
#[no_mangle]
pub extern "C" fn rs_search_has_peek(options: c_int) -> bool {
    (options & search_options::SEARCH_PEEK) != 0
}

/// Check if search options include COL flag.
#[no_mangle]
pub extern "C" fn rs_search_has_col(options: c_int) -> bool {
    (options & search_options::SEARCH_COL) != 0
}

// =============================================================================
// RE Pattern Index Constants
// =============================================================================

/// RE pattern index for search (from vim.h)
pub const RE_SEARCH: c_int = 0;
/// RE pattern index for substitute
pub const RE_SUBST: c_int = 1;
/// RE pattern index for last used
pub const RE_LAST: c_int = 2;
/// RE pattern index for both search and substitute
pub const RE_BOTH: c_int = 3;

/// Get RE_SEARCH constant.
#[no_mangle]
pub extern "C" fn rs_re_search() -> c_int {
    RE_SEARCH
}

/// Get RE_SUBST constant.
#[no_mangle]
pub extern "C" fn rs_re_subst() -> c_int {
    RE_SUBST
}

/// Get RE_LAST constant.
#[no_mangle]
pub extern "C" fn rs_re_last() -> c_int {
    RE_LAST
}

/// Get RE_BOTH constant.
#[no_mangle]
pub extern "C" fn rs_re_both() -> c_int {
    RE_BOTH
}

/// Check if the pattern index is the search pattern.
#[no_mangle]
pub extern "C" fn rs_is_search_pattern(idx: c_int) -> bool {
    idx == RE_SEARCH
}

/// Check if the pattern index is the substitute pattern.
#[no_mangle]
pub extern "C" fn rs_is_subst_pattern(idx: c_int) -> bool {
    idx == RE_SUBST
}

// =============================================================================
// Search Direction Constants
// =============================================================================

/// Direction for forward search.
pub const DIR_FORWARD: c_int = 1;
/// Direction for backward search.
pub const DIR_BACKWARD: c_int = -1;

/// Get forward direction constant.
#[no_mangle]
pub extern "C" fn rs_dir_forward() -> c_int {
    DIR_FORWARD
}

/// Get backward direction constant.
#[no_mangle]
pub extern "C" fn rs_dir_backward() -> c_int {
    DIR_BACKWARD
}

/// Check if direction is forward.
#[no_mangle]
pub extern "C" fn rs_is_forward(dir: c_int) -> bool {
    dir == DIR_FORWARD
}

/// Check if direction is backward.
#[no_mangle]
pub extern "C" fn rs_is_backward(dir: c_int) -> bool {
    dir == DIR_BACKWARD
}

/// Reverse the search direction.
#[no_mangle]
pub extern "C" fn rs_reverse_dir(dir: c_int) -> c_int {
    -dir
}

// =============================================================================
// Character Search Type Constants
// =============================================================================

/// Character search using 'f' command.
pub const CSEARCH_F: c_int = 0;
/// Character search using 'F' command.
pub const CSEARCH_CAPITAL_F: c_int = 1;
/// Character search using 't' command.
pub const CSEARCH_T: c_int = 2;
/// Character search using 'T' command.
pub const CSEARCH_CAPITAL_T: c_int = 3;

/// Check if character search command is forward (f or t).
#[no_mangle]
pub extern "C" fn rs_csearch_is_forward(cmd: c_int) -> bool {
    cmd == CSEARCH_F || cmd == CSEARCH_T
}

/// Check if character search command is 'until' (t or T).
#[no_mangle]
pub extern "C" fn rs_csearch_is_until(cmd: c_int) -> bool {
    cmd == CSEARCH_T || cmd == CSEARCH_CAPITAL_T
}

/// Get the character search type from direction and until flags.
#[no_mangle]
pub extern "C" fn rs_csearch_type(forward: bool, until: bool) -> c_int {
    match (forward, until) {
        (true, false) => CSEARCH_F,
        (false, false) => CSEARCH_CAPITAL_F,
        (true, true) => CSEARCH_T,
        (false, true) => CSEARCH_CAPITAL_T,
    }
}

// =============================================================================
// VimGrep Flag Constants
// =============================================================================

/// Vimgrep flag: find all matches (like 'g' flag).
pub const VGR_GLOBAL: c_int = 0x01;
/// Vimgrep flag: no jump to first match (like 'j' flag).
pub const VGR_NOJUMP: c_int = 0x02;
/// Vimgrep flag: use fuzzy matching (like 'f' flag).
pub const VGR_FUZZY: c_int = 0x04;

/// Get VGR_GLOBAL constant.
#[no_mangle]
pub extern "C" fn rs_vgr_global() -> c_int {
    VGR_GLOBAL
}

/// Get VGR_NOJUMP constant.
#[no_mangle]
pub extern "C" fn rs_vgr_nojump() -> c_int {
    VGR_NOJUMP
}

/// Get VGR_FUZZY constant.
#[no_mangle]
pub extern "C" fn rs_vgr_fuzzy() -> c_int {
    VGR_FUZZY
}

/// Check if vimgrep flags include global matching.
#[no_mangle]
pub extern "C" fn rs_vgr_has_global(flags: c_int) -> bool {
    (flags & VGR_GLOBAL) != 0
}

/// Check if vimgrep flags include no jump.
#[no_mangle]
pub extern "C" fn rs_vgr_has_nojump(flags: c_int) -> bool {
    (flags & VGR_NOJUMP) != 0
}

/// Check if vimgrep flags include fuzzy.
#[no_mangle]
pub extern "C" fn rs_vgr_has_fuzzy(flags: c_int) -> bool {
    (flags & VGR_FUZZY) != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_constants() {
        // FORWARD should be 1 (matches search.c)
        assert_eq!(FORWARD, 1);
    }

    #[test]
    fn test_optmagic_constants() {
        // optmagic_T values from regexp_defs.h
        assert_eq!(OPTION_MAGIC_NOT_SET, 0);
        assert_eq!(OPTION_MAGIC_ON, 1);
        assert_eq!(OPTION_MAGIC_OFF, 2);
    }

    #[test]
    fn test_optmagic_distinct() {
        // Ensure all optmagic values are distinct
        let values = [OPTION_MAGIC_NOT_SET, OPTION_MAGIC_ON, OPTION_MAGIC_OFF];
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                assert_ne!(
                    values[i], values[j],
                    "optmagic values at {i} and {j} should differ"
                );
            }
        }
    }

    #[test]
    fn test_optmagic_valid_for_match() {
        // Test that optmagic values work in match expressions
        let test_values = [OPTION_MAGIC_NOT_SET, OPTION_MAGIC_ON, OPTION_MAGIC_OFF];
        for val in test_values {
            let result = match val {
                OPTION_MAGIC_ON => true,
                OPTION_MAGIC_OFF => false,
                _ => false, // NOT_SET falls through
            };
            // OPTION_MAGIC_ON should return true, others false
            if val == OPTION_MAGIC_ON {
                assert!(result);
            }
        }
    }

    #[test]
    fn test_forward_backward_opposite() {
        // FORWARD is 1, BACKWARD is -1 (opposite sign)
        const BACKWARD: c_int = -1;
        let forward = FORWARD;
        let backward = BACKWARD;
        assert_eq!(forward, -backward);
        assert_eq!(forward + backward, 0);
    }

    #[test]
    fn test_optmagic_sequential() {
        // optmagic values should be sequential starting from 0
        assert_eq!(OPTION_MAGIC_NOT_SET, 0);
        assert_eq!(OPTION_MAGIC_ON, 1);
        assert_eq!(OPTION_MAGIC_OFF, 2);
        // Also verify they're in order
        let not_set = OPTION_MAGIC_NOT_SET;
        let on = OPTION_MAGIC_ON;
        let off = OPTION_MAGIC_OFF;
        assert!(not_set < on);
        assert!(on < off);
    }

    // Search option tests
    #[test]
    fn test_search_options_bits() {
        use search_options::*;
        // Verify options are distinct bits
        assert_eq!(SEARCH_MSG, 0x01);
        assert_eq!(SEARCH_NFMSG, 0x02);
        assert_eq!(SEARCH_HIS, 0x08);
        assert_eq!(SEARCH_END, 0x10);
        assert_eq!(SEARCH_START, 0x20);
        assert_eq!(SEARCH_KEEP, 0x40);
        assert_eq!(SEARCH_FOLD, 0x80);
        assert_eq!(SEARCH_PEEK, 0x100);
        assert_eq!(SEARCH_COL, 0x200);
    }

    #[test]
    fn test_search_has_flags() {
        use search_options::*;
        let opts = SEARCH_MSG | SEARCH_HIS | SEARCH_END;
        assert!(rs_search_has_msg(opts));
        assert!(rs_search_has_his(opts));
        assert!(rs_search_has_end(opts));
        assert!(!rs_search_has_start(opts));
        assert!(!rs_search_has_keep(opts));
    }

    // RE pattern index tests
    #[test]
    fn test_re_constants() {
        assert_eq!(RE_SEARCH, 0);
        assert_eq!(RE_SUBST, 1);
        assert_eq!(RE_LAST, 2);
        assert_eq!(RE_BOTH, 3);
    }

    #[test]
    fn test_is_search_pattern() {
        assert!(rs_is_search_pattern(RE_SEARCH));
        assert!(!rs_is_search_pattern(RE_SUBST));
        assert!(!rs_is_search_pattern(RE_LAST));
    }

    #[test]
    fn test_is_subst_pattern() {
        assert!(!rs_is_subst_pattern(RE_SEARCH));
        assert!(rs_is_subst_pattern(RE_SUBST));
        assert!(!rs_is_subst_pattern(RE_LAST));
    }

    // Direction tests
    #[test]
    fn test_dir_constants() {
        assert_eq!(DIR_FORWARD, 1);
        assert_eq!(DIR_BACKWARD, -1);
    }

    #[test]
    fn test_is_forward_backward() {
        assert!(rs_is_forward(DIR_FORWARD));
        assert!(!rs_is_forward(DIR_BACKWARD));
        assert!(!rs_is_backward(DIR_FORWARD));
        assert!(rs_is_backward(DIR_BACKWARD));
    }

    #[test]
    fn test_reverse_dir() {
        assert_eq!(rs_reverse_dir(DIR_FORWARD), DIR_BACKWARD);
        assert_eq!(rs_reverse_dir(DIR_BACKWARD), DIR_FORWARD);
        // Reversing twice returns original
        assert_eq!(rs_reverse_dir(rs_reverse_dir(DIR_FORWARD)), DIR_FORWARD);
    }

    // Character search tests
    #[test]
    fn test_csearch_is_forward() {
        assert!(rs_csearch_is_forward(CSEARCH_F));
        assert!(!rs_csearch_is_forward(CSEARCH_CAPITAL_F));
        assert!(rs_csearch_is_forward(CSEARCH_T));
        assert!(!rs_csearch_is_forward(CSEARCH_CAPITAL_T));
    }

    #[test]
    fn test_csearch_is_until() {
        assert!(!rs_csearch_is_until(CSEARCH_F));
        assert!(!rs_csearch_is_until(CSEARCH_CAPITAL_F));
        assert!(rs_csearch_is_until(CSEARCH_T));
        assert!(rs_csearch_is_until(CSEARCH_CAPITAL_T));
    }

    #[test]
    fn test_csearch_type() {
        assert_eq!(rs_csearch_type(true, false), CSEARCH_F);
        assert_eq!(rs_csearch_type(false, false), CSEARCH_CAPITAL_F);
        assert_eq!(rs_csearch_type(true, true), CSEARCH_T);
        assert_eq!(rs_csearch_type(false, true), CSEARCH_CAPITAL_T);
    }

    // VimGrep flag tests
    #[test]
    fn test_vgr_constants() {
        assert_eq!(VGR_GLOBAL, 0x01);
        assert_eq!(VGR_NOJUMP, 0x02);
        assert_eq!(VGR_FUZZY, 0x04);
    }

    #[test]
    fn test_vgr_has_flags() {
        let flags = VGR_GLOBAL | VGR_NOJUMP;
        assert!(rs_vgr_has_global(flags));
        assert!(rs_vgr_has_nojump(flags));
        assert!(!rs_vgr_has_fuzzy(flags));
    }
}
