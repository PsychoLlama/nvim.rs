//! Command line utilities for Neovim
//!
//! Provides Rust implementations of command line functions.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]

pub mod api;
pub mod cmdbuff;
pub mod cmdwin;
pub mod color;

pub mod command_line_state;
pub mod completion;
pub mod constants;
pub mod context;
pub mod edit;
pub mod entry;
pub mod expand;
pub mod expr;
pub mod fname;
pub mod getln;
pub mod history;
pub mod input;
pub mod keys;
pub mod matchstate;
pub mod mode;
pub mod pattern;
pub mod preview;
pub mod screen;
pub mod search;
pub mod sources;
pub mod state;
pub mod ui;
pub mod usercomplete;
pub mod viewstate;
pub mod viml;
pub mod wildmenu;

use std::os::raw::c_int;

extern "C" {
    fn nvim_get_ccline_overstrike() -> c_int;
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    static cmdwin_type: c_int;
    fn nvim_get_cmdline_type() -> c_int;
    fn nvim_get_cmdpreview_ns() -> c_int;
    fn nvim_get_ccline_cmdfirstc() -> c_int;
}

/// Check if command line is in overstrike mode.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdline_overstrike() -> c_int {
    nvim_get_ccline_overstrike()
}

/// Direct C replacement for cmdline_overstrike().
///
/// # Safety
///
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdline_overstrike"]
pub unsafe extern "C" fn cmdline_overstrike_rs() -> bool {
    nvim_get_ccline_overstrike() != 0
}

/// Check if cursor is at the end of the command line.
///
/// Returns true if cmdpos >= cmdlen.
///
/// # Safety
///
/// Calls external C function to access static variable.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdline_at_end() -> c_int {
    c_int::from(nvim_get_ccline_cmdpos() >= nvim_get_ccline_cmdlen())
}

/// Direct C replacement for cmdline_at_end().
///
/// # Safety
///
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdline_at_end"]
pub unsafe extern "C" fn cmdline_at_end_rs() -> bool {
    nvim_get_ccline_cmdpos() >= nvim_get_ccline_cmdlen()
}

/// NUL character constant
const NUL: c_int = 0;

/// Check if in the cmdwin, not editing the command line.
///
/// Returns true if `cmdwin_type` != 0 AND `get_cmdline_type()` == NUL.
///
/// # Safety
///
/// Calls external C functions to access global state.
#[no_mangle]
pub unsafe extern "C" fn rs_is_in_cmdwin() -> c_int {
    let cmdline_type = nvim_get_cmdline_type();

    c_int::from(cmdwin_type != 0 && cmdline_type == NUL)
}

/// Direct C replacement for is_in_cmdwin().
///
/// # Safety
///
/// Calls external C functions to access global state.
#[must_use]
#[export_name = "is_in_cmdwin"]
pub unsafe extern "C" fn is_in_cmdwin_rs() -> bool {
    let cmdline_type = nvim_get_cmdline_type();
    cmdwin_type != 0 && cmdline_type == NUL
}

/// Get the command preview namespace.
///
/// Returns the `cmdpreview_ns` static variable.
///
/// # Safety
/// Calls external C function to access static variable.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdpreview_get_ns() -> c_int {
    nvim_get_cmdpreview_ns()
}

/// Direct C replacement for cmdpreview_get_ns().
///
/// # Safety
/// Calls external C function to access static variable.
#[must_use]
#[export_name = "cmdpreview_get_ns"]
pub unsafe extern "C" fn cmdpreview_get_ns_rs() -> c_int {
    nvim_get_cmdpreview_ns()
}

/// Get the first character of the current command line.
///
/// Returns `ccline.cmdfirstc`.
///
/// # Safety
/// Calls external C function to access struct field.
#[no_mangle]
pub unsafe extern "C" fn rs_get_cmdline_firstc() -> c_int {
    nvim_get_ccline_cmdfirstc()
}

/// Direct C replacement for get_cmdline_firstc().
///
/// # Safety
/// Calls external C function to access struct field.
#[must_use]
#[export_name = "get_cmdline_firstc"]
pub unsafe extern "C" fn get_cmdline_firstc_rs() -> c_int {
    nvim_get_ccline_cmdfirstc()
}

// =============================================================================
// Phase 80: Command-Line Mode Helpers
// =============================================================================

/// Command-line mode result codes
pub mod cmdline_result {
    use std::os::raw::c_int;

    /// Command line not changed - skip further processing
    pub const NOT_CHANGED: c_int = 1;
    /// Command line changed - update display
    pub const CHANGED: c_int = 2;
    /// Normal return - continue processing
    pub const NORMAL: c_int = 0;
    /// Got escape - abort command
    pub const GOTESC: c_int = -1;
    /// Process done - exit loop
    pub const PROCESS_DONE: c_int = 3;
}

/// History type indices
pub mod hist_type {
    use std::os::raw::c_int;

    /// Command history ':'
    pub const CMD: c_int = 0;
    /// Search history '/' and '?'
    pub const SEARCH: c_int = 1;
    /// Expression history '='
    pub const EXPR: c_int = 2;
    /// Input history '@'
    pub const INPUT: c_int = 3;
    /// Debug history '>'
    pub const DEBUG: c_int = 4;

    /// Number of history types
    pub const COUNT: c_int = 5;
}

/// Prompt first character types
pub mod prompt_char {
    use std::os::raw::c_char;

    /// Ex command prompt ':'
    pub const EX_CMD: c_char = b':' as c_char;
    /// Forward search '/'
    pub const SEARCH_FWD: c_char = b'/' as c_char;
    /// Backward search '?'
    pub const SEARCH_BWD: c_char = b'?' as c_char;
    /// Expression '='
    pub const EXPR: c_char = b'=' as c_char;
    /// Debug '>'
    pub const DEBUG: c_char = b'>' as c_char;
    /// Input '@'
    pub const INPUT: c_char = b'@' as c_char;
}

/// Get the history type for a given prompt character.
#[no_mangle]
pub const extern "C" fn rs_cmdline_get_hist_type(firstc: c_int) -> c_int {
    match firstc as u8 {
        b'/' | b'?' => hist_type::SEARCH,
        b'=' => hist_type::EXPR,
        b'@' => hist_type::INPUT,
        b'>' => hist_type::DEBUG,
        // Default to command history for ':' and unknown characters
        _ => hist_type::CMD,
    }
}

/// Check if a prompt character indicates search mode.
#[no_mangle]
pub const extern "C" fn rs_cmdline_is_search_prompt(firstc: c_int) -> bool {
    matches!(firstc as u8, b'/' | b'?')
}

/// Check if a prompt character indicates forward search.
#[no_mangle]
pub const extern "C" fn rs_cmdline_is_forward_search(firstc: c_int) -> bool {
    firstc as u8 == b'/'
}

/// Check if a prompt character indicates backward search.
#[no_mangle]
pub const extern "C" fn rs_cmdline_is_backward_search(firstc: c_int) -> bool {
    firstc as u8 == b'?'
}

/// Check if a prompt character indicates expression mode.
#[no_mangle]
pub const extern "C" fn rs_cmdline_is_expr_prompt(firstc: c_int) -> bool {
    firstc as u8 == b'='
}

/// Check if a prompt character indicates input mode.
#[no_mangle]
pub const extern "C" fn rs_cmdline_is_input_prompt(firstc: c_int) -> bool {
    firstc as u8 == b'@'
}

/// Check if a prompt character indicates debug mode.
#[no_mangle]
pub const extern "C" fn rs_cmdline_is_debug_prompt(firstc: c_int) -> bool {
    firstc as u8 == b'>'
}

/// Check if a character is a valid history firstc.
#[no_mangle]
pub const extern "C" fn rs_cmdline_is_valid_firstc(firstc: c_int) -> bool {
    matches!(firstc as u8, b':' | b'/' | b'?' | b'=' | b'@' | b'>')
}

/// Flip search direction (/ <-> ?).
#[no_mangle]
pub const extern "C" fn rs_cmdline_flip_search_dir(firstc: c_int) -> c_int {
    match firstc as u8 {
        b'/' => b'?' as c_int,
        b'?' => b'/' as c_int,
        _ => firstc,
    }
}

// =============================================================================
// Command-Line Buffer Size Helpers
// =============================================================================

/// Minimum command buffer size
pub const MIN_CMDBUFF_SIZE: c_int = 256;

/// Calculate the next buffer size for reallocation.
///
/// Uses a growth strategy to minimize reallocations.
#[no_mangle]
pub const extern "C" fn rs_cmdline_next_bufsize(current_len: c_int, needed: c_int) -> c_int {
    if needed <= MIN_CMDBUFF_SIZE {
        return MIN_CMDBUFF_SIZE;
    }

    // Double the current length or use needed, whichever is larger
    let doubled = if current_len > 0 {
        current_len * 2
    } else {
        MIN_CMDBUFF_SIZE
    };

    if doubled >= needed {
        doubled
    } else {
        // Round up to next power of 2
        let mut size = MIN_CMDBUFF_SIZE;
        while size < needed && size < i32::MAX / 2 {
            size *= 2;
        }
        if size < needed {
            needed
        } else {
            size
        }
    }
}

/// Check if command-line buffer needs reallocation.
#[no_mangle]
pub const extern "C" fn rs_cmdline_buf_needs_grow(bufsize: c_int, needed: c_int) -> bool {
    needed > bufsize
}

// =============================================================================
// Incremental Search Helpers
// =============================================================================

/// Incremental search state flags
pub mod incsearch_flags {
    use std::os::raw::c_int;

    /// Incremental search is active
    pub const ACTIVE: c_int = 0x01;
    /// Search is postponed (waiting for more input)
    pub const POSTPONED: c_int = 0x02;
    /// Match found
    pub const MATCH_FOUND: c_int = 0x04;
    /// Match is on current line
    pub const MATCH_ON_LINE: c_int = 0x08;
}

/// Check if incremental search state has a specific flag.
#[no_mangle]
pub const extern "C" fn rs_incsearch_has_flag(flags: c_int, flag: c_int) -> bool {
    (flags & flag) != 0
}

/// Set an incremental search flag.
#[no_mangle]
pub const extern "C" fn rs_incsearch_set_flag(flags: c_int, flag: c_int) -> c_int {
    flags | flag
}

/// Clear an incremental search flag.
#[no_mangle]
pub const extern "C" fn rs_incsearch_clear_flag(flags: c_int, flag: c_int) -> c_int {
    flags & !flag
}

// =============================================================================
// Pattern Validation Helpers
// =============================================================================

/// Check if a search pattern is empty.
///
/// Empty patterns: "", or just the delimiter(s), or very-magic prefix only
#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub const unsafe extern "C" fn rs_cmdline_is_empty_pattern(
    pattern: *const u8,
    len: usize,
    delim: c_int,
) -> bool {
    if pattern.is_null() || len == 0 {
        return true;
    }

    let first = *pattern;

    // Single character that is the delimiter
    if len == 1 && first == delim as u8 {
        return true;
    }

    // Just the very-magic or nomagic prefix
    if len == 2 && first == b'\\' {
        let second = *pattern.add(1);
        if matches!(second, b'v' | b'V' | b'm' | b'M') {
            return true;
        }
    }

    false
}

/// Check if a character could be a magic prefix.
#[no_mangle]
pub const extern "C" fn rs_cmdline_is_magic_prefix(c: c_int) -> bool {
    matches!(c as u8, b'v' | b'V' | b'm' | b'M')
}

// =============================================================================
// Command-Line Level Helpers
// =============================================================================

/// Maximum command-line recursion depth
pub const MAX_CMDLINE_LEVEL: c_int = 50;

/// Check if command-line level is valid (not too deep).
#[no_mangle]
pub const extern "C" fn rs_cmdline_level_ok(level: c_int) -> bool {
    level >= 0 && level < MAX_CMDLINE_LEVEL
}

/// Get the current nesting depth text for messages.
#[no_mangle]
pub const extern "C" fn rs_cmdline_level_to_depth(level: c_int) -> c_int {
    if level < 0 {
        0
    } else {
        level + 1
    }
}

// =============================================================================
// Wildcard Mode Helpers
// =============================================================================

/// Wildcard expansion mode flags
pub mod wild_mode {
    use std::os::raw::c_int;

    /// No wildcard expansion
    pub const NONE: c_int = 0;
    /// Expand wildcards
    pub const EXPAND: c_int = 0x01;
    /// List matches
    pub const LIST: c_int = 0x02;
    /// Show longest common match
    pub const LONGEST: c_int = 0x04;
    /// Show full match
    pub const FULL: c_int = 0x08;
    /// Navigate to next match
    pub const NEXT: c_int = 0x10;
    /// Navigate to previous match
    pub const PREV: c_int = 0x20;
    /// Use fuzzy matching
    pub const FUZZY: c_int = 0x40;
    /// PUM (popup menu) is active
    pub const PUM: c_int = 0x80;
}

/// Check if wildcard mode has a specific flag.
#[no_mangle]
pub const extern "C" fn rs_wild_has_mode(mode: c_int, flag: c_int) -> bool {
    (mode & flag) != 0
}

/// Combine wildcard mode flags.
#[no_mangle]
pub const extern "C" fn rs_wild_combine_mode(mode: c_int, flag: c_int) -> c_int {
    mode | flag
}

/// Remove wildcard mode flag.
#[no_mangle]
pub const extern "C" fn rs_wild_remove_mode(mode: c_int, flag: c_int) -> c_int {
    mode & !flag
}

/// Check if wildcard mode requires expansion.
#[no_mangle]
pub const extern "C" fn rs_wild_needs_expand(mode: c_int) -> bool {
    (mode & (wild_mode::EXPAND | wild_mode::LONGEST | wild_mode::FULL | wild_mode::FUZZY)) != 0
}

/// Check if wildcard mode is navigating (next/prev).
#[no_mangle]
pub const extern "C" fn rs_wild_is_navigating(mode: c_int) -> bool {
    (mode & (wild_mode::NEXT | wild_mode::PREV)) != 0
}

// =============================================================================
// Phase 80 Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::cast_possible_wrap)]
mod tests {
    use super::*;
    use std::os::raw::c_uint;

    // Flag value from option_vars.generated.h
    const K_OPT_WOP_FLAG_FUZZY: c_uint = 0x01;

    #[test]
    fn test_wildoption_flag() {
        // K_OPT_WOP_FLAG_FUZZY should be 0x01
        assert_eq!(K_OPT_WOP_FLAG_FUZZY, 0x01);
    }

    #[test]
    fn test_nul_constant() {
        // NUL should be 0
        assert_eq!(NUL, 0);
    }

    #[test]
    fn test_wildoption_flag_is_power_of_two() {
        // Flag should be a single bit (power of 2)
        let flag = K_OPT_WOP_FLAG_FUZZY;
        assert_ne!(flag, 0);
        assert_eq!(flag & (flag - 1), 0, "Flag should be a power of 2");
    }

    #[test]
    fn test_nul_matches_ascii() {
        // NUL constant should match ASCII NUL character
        assert_eq!(NUL, 0);
        assert_eq!(NUL, c_int::from(b'\0'));
    }

    #[test]
    fn test_fuzzy_flag_bit_position() {
        // K_OPT_WOP_FLAG_FUZZY should be bit 0 (1 << 0)
        assert_eq!(K_OPT_WOP_FLAG_FUZZY, 1 << 0);
    }

    #[test]
    #[allow(clippy::cast_possible_truncation)]
    fn test_nul_is_string_terminator() {
        // NUL should work as C string terminator (NUL is 0, so fits in u8)
        let nul_char = NUL as u8;
        assert_eq!(nul_char, 0);
    }

    #[test]
    fn test_hist_type() {
        assert_eq!(rs_cmdline_get_hist_type(b':' as c_int), hist_type::CMD);
        assert_eq!(rs_cmdline_get_hist_type(b'/' as c_int), hist_type::SEARCH);
        assert_eq!(rs_cmdline_get_hist_type(b'?' as c_int), hist_type::SEARCH);
        assert_eq!(rs_cmdline_get_hist_type(b'=' as c_int), hist_type::EXPR);
        assert_eq!(rs_cmdline_get_hist_type(b'@' as c_int), hist_type::INPUT);
        assert_eq!(rs_cmdline_get_hist_type(b'>' as c_int), hist_type::DEBUG);
    }

    #[test]
    fn test_search_prompt_detection() {
        assert!(rs_cmdline_is_search_prompt(b'/' as c_int));
        assert!(rs_cmdline_is_search_prompt(b'?' as c_int));
        assert!(!rs_cmdline_is_search_prompt(b':' as c_int));
        assert!(!rs_cmdline_is_search_prompt(b'=' as c_int));
    }

    #[test]
    fn test_flip_search_dir() {
        assert_eq!(rs_cmdline_flip_search_dir(b'/' as c_int), b'?' as c_int);
        assert_eq!(rs_cmdline_flip_search_dir(b'?' as c_int), b'/' as c_int);
        assert_eq!(rs_cmdline_flip_search_dir(b':' as c_int), b':' as c_int);
    }

    #[test]
    fn test_valid_firstc() {
        assert!(rs_cmdline_is_valid_firstc(b':' as c_int));
        assert!(rs_cmdline_is_valid_firstc(b'/' as c_int));
        assert!(rs_cmdline_is_valid_firstc(b'?' as c_int));
        assert!(rs_cmdline_is_valid_firstc(b'=' as c_int));
        assert!(rs_cmdline_is_valid_firstc(b'@' as c_int));
        assert!(rs_cmdline_is_valid_firstc(b'>' as c_int));
        assert!(!rs_cmdline_is_valid_firstc(b'x' as c_int));
        assert!(!rs_cmdline_is_valid_firstc(0));
    }

    #[test]
    fn test_bufsize_calculation() {
        // Small needs should get MIN_CMDBUFF_SIZE
        assert_eq!(rs_cmdline_next_bufsize(0, 10), MIN_CMDBUFF_SIZE);
        assert_eq!(rs_cmdline_next_bufsize(0, 256), MIN_CMDBUFF_SIZE);

        // Larger needs should double or exceed
        assert_eq!(rs_cmdline_next_bufsize(256, 300), 512);
        assert_eq!(rs_cmdline_next_bufsize(256, 600), 1024);
    }

    #[test]
    fn test_buf_needs_grow() {
        assert!(!rs_cmdline_buf_needs_grow(256, 100));
        assert!(!rs_cmdline_buf_needs_grow(256, 256));
        assert!(rs_cmdline_buf_needs_grow(256, 257));
    }

    #[test]
    fn test_incsearch_flags() {
        let flags = 0;
        let flags = rs_incsearch_set_flag(flags, incsearch_flags::ACTIVE);
        assert!(rs_incsearch_has_flag(flags, incsearch_flags::ACTIVE));
        assert!(!rs_incsearch_has_flag(flags, incsearch_flags::POSTPONED));

        let flags = rs_incsearch_set_flag(flags, incsearch_flags::MATCH_FOUND);
        assert!(rs_incsearch_has_flag(flags, incsearch_flags::ACTIVE));
        assert!(rs_incsearch_has_flag(flags, incsearch_flags::MATCH_FOUND));

        let flags = rs_incsearch_clear_flag(flags, incsearch_flags::ACTIVE);
        assert!(!rs_incsearch_has_flag(flags, incsearch_flags::ACTIVE));
        assert!(rs_incsearch_has_flag(flags, incsearch_flags::MATCH_FOUND));
    }

    #[test]
    fn test_empty_pattern() {
        unsafe {
            // Null pointer
            assert!(rs_cmdline_is_empty_pattern(
                std::ptr::null(),
                0,
                b'/' as c_int
            ));

            // Empty length
            assert!(rs_cmdline_is_empty_pattern(b"/".as_ptr(), 0, b'/' as c_int));

            // Just delimiter
            assert!(rs_cmdline_is_empty_pattern(b"/".as_ptr(), 1, b'/' as c_int));

            // Very-magic prefix only
            assert!(rs_cmdline_is_empty_pattern(
                b"\\v".as_ptr(),
                2,
                b'/' as c_int
            ));
            assert!(rs_cmdline_is_empty_pattern(
                b"\\V".as_ptr(),
                2,
                b'/' as c_int
            ));
            assert!(rs_cmdline_is_empty_pattern(
                b"\\m".as_ptr(),
                2,
                b'/' as c_int
            ));
            assert!(rs_cmdline_is_empty_pattern(
                b"\\M".as_ptr(),
                2,
                b'/' as c_int
            ));

            // Non-empty patterns
            assert!(!rs_cmdline_is_empty_pattern(
                b"foo".as_ptr(),
                3,
                b'/' as c_int
            ));
            assert!(!rs_cmdline_is_empty_pattern(
                b"\\vfoo".as_ptr(),
                5,
                b'/' as c_int
            ));
        }
    }

    #[test]
    fn test_magic_prefix() {
        assert!(rs_cmdline_is_magic_prefix(b'v' as c_int));
        assert!(rs_cmdline_is_magic_prefix(b'V' as c_int));
        assert!(rs_cmdline_is_magic_prefix(b'm' as c_int));
        assert!(rs_cmdline_is_magic_prefix(b'M' as c_int));
        assert!(!rs_cmdline_is_magic_prefix(b'x' as c_int));
    }

    #[test]
    fn test_cmdline_level() {
        assert!(rs_cmdline_level_ok(0));
        assert!(rs_cmdline_level_ok(49));
        assert!(!rs_cmdline_level_ok(50));
        assert!(!rs_cmdline_level_ok(-1));

        assert_eq!(rs_cmdline_level_to_depth(0), 1);
        assert_eq!(rs_cmdline_level_to_depth(1), 2);
        assert_eq!(rs_cmdline_level_to_depth(-1), 0);
    }

    #[test]
    fn test_wild_mode() {
        assert!(rs_wild_needs_expand(wild_mode::EXPAND));
        assert!(rs_wild_needs_expand(wild_mode::LONGEST));
        assert!(rs_wild_needs_expand(wild_mode::FULL));
        assert!(rs_wild_needs_expand(wild_mode::FUZZY));
        assert!(!rs_wild_needs_expand(wild_mode::NONE));
        assert!(!rs_wild_needs_expand(wild_mode::LIST));

        assert!(rs_wild_is_navigating(wild_mode::NEXT));
        assert!(rs_wild_is_navigating(wild_mode::PREV));
        assert!(!rs_wild_is_navigating(wild_mode::EXPAND));
    }
}
