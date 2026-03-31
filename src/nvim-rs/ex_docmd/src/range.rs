//! Range parsing and manipulation utilities for Ex commands.
//!
//! This module provides functions for parsing and manipulating line ranges
//! in Ex commands, such as `1,5`, `%`, `'a,'b`, `.,$`, etc.

use std::ffi::{c_char, c_int, c_long, c_void};
use std::ptr;

use crate::address::{
    ADDR_ARGUMENTS, ADDR_BUFFERS, ADDR_LINES, ADDR_LOADED_BUFFERS, ADDR_NONE, ADDR_OTHER,
    ADDR_QUICKFIX, ADDR_QUICKFIX_VALID, ADDR_TABS, ADDR_TABS_RELATIVE, ADDR_UNSIGNED, ADDR_WINDOWS,
};
use crate::ExArgHandle;

// =============================================================================
// FFI declarations
// =============================================================================

/// Expansion context: no expansion
const EXPAND_NOTHING: c_int = 0;

/// EX_ZEROR flag: zero in range allowed.
const EX_ZEROR: u32 = 0x1000;

/// EX_RANGE flag (verified with _Static_assert in C)
const EX_RANGE: u32 = 0x001;

extern "C" {
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // Buffer/window/tab navigation
    fn nvim_get_curbuf() -> *mut c_void;
    fn nvim_buf_get_line_count(buf: *mut c_void) -> i32;
    fn nvim_get_argcount() -> c_int;
    fn rs_get_highest_fnum() -> c_int;
    fn nvim_docmd_first_loaded_fnum_or_fail() -> c_int;
    fn nvim_docmd_last_loaded_fnum_or_fail() -> c_int;
    fn nvim_docmd_last_win_nr() -> c_int;
    fn nvim_docmd_last_tab_nr() -> c_int;
    fn qf_get_valid_size(eap: ExArgHandle) -> usize;

    // Error messages
}

// =============================================================================
// Range representation
// =============================================================================

/// A line range for Ex commands.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct LineRange {
    /// First line (1-based), 0 means unset.
    pub line1: c_long,
    /// Second line (1-based), 0 means unset.
    pub line2: c_long,
    /// Number of addresses given (0, 1, or 2).
    pub addr_count: c_int,
}

impl LineRange {
    /// Create an empty range.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            line1: 0,
            line2: 0,
            addr_count: 0,
        }
    }

    /// Create a single-line range.
    #[must_use]
    pub const fn single(line: c_long) -> Self {
        Self {
            line1: line,
            line2: line,
            addr_count: 1,
        }
    }

    /// Create a two-address range.
    #[must_use]
    pub const fn from_pair(line1: c_long, line2: c_long) -> Self {
        Self {
            line1,
            line2,
            addr_count: 2,
        }
    }

    /// Check if the range is empty (no address given).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.addr_count == 0
    }

    /// Check if the range is a single line.
    #[must_use]
    pub const fn is_single(&self) -> bool {
        self.addr_count == 1
    }

    /// Check if the range has two addresses.
    #[must_use]
    pub const fn is_pair(&self) -> bool {
        self.addr_count >= 2
    }

    /// Check if the range is valid (line1 <= line2).
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.line1 <= self.line2
    }

    /// Normalize the range so line1 <= line2.
    pub fn normalize(&mut self) {
        if self.line1 > self.line2 {
            std::mem::swap(&mut self.line1, &mut self.line2);
        }
    }

    /// Get the number of lines in the range.
    #[must_use]
    pub const fn count(&self) -> c_long {
        if self.line1 <= self.line2 && self.addr_count > 0 {
            self.line2 - self.line1 + 1
        } else {
            0
        }
    }

    /// Clamp the range to valid buffer lines.
    pub fn clamp(&mut self, max_line: c_long) {
        if self.line1 < 1 {
            self.line1 = 1;
        }
        if self.line2 < 1 {
            self.line2 = 1;
        }
        if self.line1 > max_line {
            self.line1 = max_line;
        }
        if self.line2 > max_line {
            self.line2 = max_line;
        }
    }

    /// Check if a line is within the range.
    #[must_use]
    pub const fn contains(&self, line: c_long) -> bool {
        line >= self.line1 && line <= self.line2
    }
}

// =============================================================================
// Range parsing state
// =============================================================================

/// State during range parsing.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct RangeParseState {
    /// Whether % was used (whole file).
    pub whole_file: bool,
    /// Whether # was used (alternate file).
    pub alternate: bool,
    /// Whether . was used (current line).
    pub current_line: bool,
    /// Whether $ was used (last line).
    pub last_line: bool,
    /// Whether a mark was used.
    pub used_mark: bool,
    /// Whether a search pattern was used.
    pub used_search: bool,
    /// Whether a visual range was used.
    pub visual_range: bool,
}

impl RangeParseState {
    /// Create new parse state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            whole_file: false,
            alternate: false,
            current_line: false,
            last_line: false,
            used_mark: false,
            used_search: false,
            visual_range: false,
        }
    }

    /// Reset the parse state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Check if any special range was used.
    #[must_use]
    pub const fn has_special(&self) -> bool {
        self.whole_file
            || self.alternate
            || self.current_line
            || self.last_line
            || self.used_mark
            || self.used_search
            || self.visual_range
    }
}

// =============================================================================
// Address element characters
// =============================================================================

/// Check if character is a range address character.
///
/// Address characters: digits, '.', '$', '%', '\'', '/', '?', '+', '-', '\\', '*'
#[inline]
pub const fn is_addr_char(c: u8) -> bool {
    c.is_ascii_digit()
        || matches!(
            c,
            b'.' | b'$' | b'%' | b'\'' | b'/' | b'?' | b'+' | b'-' | b'\\' | b'*'
        )
}

/// Check if character starts a numeric address.
#[inline]
pub const fn is_addr_digit(c: u8) -> bool {
    c.is_ascii_digit() || matches!(c, b'+' | b'-')
}

/// Check if character is a range separator.
#[inline]
pub const fn is_range_sep(c: u8) -> bool {
    c == b',' || c == b';'
}

/// FFI wrapper for is_addr_char.
#[no_mangle]
#[allow(clippy::manual_range_contains)]
pub extern "C" fn rs_is_addr_char(c: c_int) -> c_int {
    if c < 0 || c > 127 {
        return 0;
    }
    c_int::from(is_addr_char(c as u8))
}

/// FFI wrapper for is_addr_digit.
#[no_mangle]
#[allow(clippy::manual_range_contains)]
pub extern "C" fn rs_is_addr_digit(c: c_int) -> c_int {
    if c < 0 || c > 127 {
        return 0;
    }
    c_int::from(is_addr_digit(c as u8))
}

/// FFI wrapper for is_range_sep.
#[no_mangle]
#[allow(clippy::manual_range_contains)]
pub extern "C" fn rs_is_range_sep(c: c_int) -> c_int {
    if c < 0 || c > 127 {
        return 0;
    }
    c_int::from(is_range_sep(c as u8))
}

// =============================================================================
// Range utilities
// =============================================================================

/// Skip over an address specification.
///
/// Returns a pointer past the address, or the original pointer if no address.
///
/// # Safety
///
/// `p` must be a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_skip_address(p: *const c_char) -> *const c_char {
    if p.is_null() {
        return p;
    }

    let mut ptr = p;

    // Skip leading whitespace
    ptr = skipwhite(ptr) as *const c_char;

    // Skip address characters
    loop {
        let c = *ptr as u8;
        if c == 0 {
            break;
        }

        if c.is_ascii_digit() {
            // Skip digits
            while (*ptr as u8).is_ascii_digit() {
                ptr = ptr.add(1);
            }
        } else if c == b'.' || c == b'$' || c == b'%' || c == b'*' {
            ptr = ptr.add(1);
        } else if c == b'\'' {
            // Mark - skip ' and mark character
            ptr = ptr.add(1);
            if *ptr != 0 {
                ptr = ptr.add(1);
            }
        } else if c == b'/' || c == b'?' {
            // Search pattern - skip to matching delimiter
            let delim = c;
            ptr = ptr.add(1);
            while *ptr != 0 && (*ptr as u8) != delim {
                if *ptr as u8 == b'\\' && *ptr.add(1) != 0 {
                    ptr = ptr.add(1);
                }
                ptr = ptr.add(1);
            }
            if *ptr != 0 {
                ptr = ptr.add(1);
            }
        } else if c == b'+' || c == b'-' {
            // Offset
            ptr = ptr.add(1);
            // Skip optional digits
            while (*ptr as u8).is_ascii_digit() {
                ptr = ptr.add(1);
            }
        } else if c == b'\\' {
            // Special marks: \/, \?, \&
            if matches!(*ptr.add(1) as u8, b'/' | b'?' | b'&') {
                ptr = ptr.add(2);
            } else {
                break;
            }
        } else {
            break;
        }

        // Skip whitespace between parts
        while (*ptr as u8) == b' ' || (*ptr as u8) == b'\t' {
            ptr = ptr.add(1);
        }
    }

    ptr
}

/// Parse a simple line number from the start of a string.
///
/// Returns the number and sets `consumed` to the number of characters consumed.
/// Returns 0 if no number found.
///
/// # Safety
///
/// `s` must be a valid null-terminated C string.
/// `consumed` must be valid for writes if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_parse_line_number(s: *const c_char, consumed: *mut c_int) -> c_long {
    if s.is_null() {
        if !consumed.is_null() {
            *consumed = 0;
        }
        return 0;
    }

    let mut ptr = s;
    let mut negative = false;

    // Check for sign
    if *ptr as u8 == b'-' {
        negative = true;
        ptr = ptr.add(1);
    } else if *ptr as u8 == b'+' {
        ptr = ptr.add(1);
    }

    // Parse digits
    let mut value: c_long = 0;
    let mut has_digits = false;
    while (*ptr as u8).is_ascii_digit() {
        has_digits = true;
        value = value
            .saturating_mul(10)
            .saturating_add((*ptr as u8 - b'0') as c_long);
        ptr = ptr.add(1);
    }

    if !consumed.is_null() {
        *consumed = (ptr as usize - s as usize) as c_int;
    }

    if !has_digits {
        return 0;
    }

    if negative {
        -value
    } else {
        value
    }
}

// =============================================================================
// Default range calculation
// =============================================================================

/// Get the default line for a command without an address (usually current line = 0 means use cursor).
#[no_mangle]
pub extern "C" fn rs_default_line() -> c_long {
    0 // Caller should interpret 0 as "use cursor line"
}

/// Create a whole-file range from given line count.
#[no_mangle]
pub extern "C" fn rs_make_whole_file_range(last_line: c_long) -> LineRange {
    if last_line <= 0 {
        LineRange::single(1)
    } else {
        LineRange::from_pair(1, last_line)
    }
}

// =============================================================================
// Range FFI exports
// =============================================================================

/// Create an empty line range.
#[no_mangle]
pub extern "C" fn rs_line_range_new() -> LineRange {
    LineRange::new()
}

/// Create a single-line range.
#[no_mangle]
pub extern "C" fn rs_line_range_single(line: c_long) -> LineRange {
    LineRange::single(line)
}

/// Create a pair range.
#[no_mangle]
pub extern "C" fn rs_line_range_pair(line1: c_long, line2: c_long) -> LineRange {
    LineRange::from_pair(line1, line2)
}

/// Check if range is empty.
///
/// # Safety
/// `range` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_range_is_empty(range: *const LineRange) -> c_int {
    if range.is_null() {
        return 1;
    }
    c_int::from((*range).is_empty())
}

/// Check if range is single line.
///
/// # Safety
/// `range` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_range_is_single(range: *const LineRange) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from((*range).is_single())
}

/// Check if range is valid.
///
/// # Safety
/// `range` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_range_is_valid(range: *const LineRange) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from((*range).is_valid())
}

/// Normalize range so line1 <= line2.
///
/// # Safety
/// `range` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_range_normalize(range: *mut LineRange) {
    if !range.is_null() {
        (*range).normalize();
    }
}

/// Get line count in range.
///
/// # Safety
/// `range` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_range_count(range: *const LineRange) -> c_long {
    if range.is_null() {
        return 0;
    }
    (*range).count()
}

/// Clamp range to buffer bounds.
///
/// # Safety
/// `range` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_range_clamp(range: *mut LineRange, max_line: c_long) {
    if !range.is_null() {
        (*range).clamp(max_line);
    }
}

/// Check if range contains a line.
///
/// # Safety
/// `range` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_line_range_contains(range: *const LineRange, line: c_long) -> c_int {
    if range.is_null() {
        return 0;
    }
    c_int::from((*range).contains(line))
}

// =============================================================================
// RangeParseState FFI
// =============================================================================

/// Create new parse state.
#[no_mangle]
pub extern "C" fn rs_range_parse_state_new() -> RangeParseState {
    RangeParseState::new()
}

/// Reset parse state.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_range_parse_state_reset(state: *mut RangeParseState) {
    if !state.is_null() {
        (*state).reset();
    }
}

/// Check if parse state has special range.
///
/// # Safety
/// `state` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_range_parse_state_has_special(state: *const RangeParseState) -> c_int {
    if state.is_null() {
        return 0;
    }
    c_int::from((*state).has_special())
}

// =============================================================================
// correct_range - Fix zero line numbers
// =============================================================================

/// Correct zero line numbers to 1 when EX_ZEROR is not set.
///
/// Matches C `correct_range()`.
///
/// # Safety
///
/// `eap` must be a valid ExArgHandle.
#[export_name = "correct_range"]
pub unsafe extern "C" fn rs_correct_range(eap: ExArgHandle) {
    if eap.is_null() {
        return;
    }

    let argt = (*eap).argt;
    if (argt & EX_ZEROR) == 0 {
        if (*eap).line1 == 0 {
            (*eap).line1 = 1;
        }
        if (*eap).line2 == 0 {
            (*eap).line2 = 1;
        }
    }
}

// =============================================================================
// Skip colon and whitespace
// =============================================================================

/// Skip over ':' and whitespace in a command line string.
///
/// If `skipleadingwhite` is true, also skip leading whitespace before any ':'.
///
/// Matches C `skip_colon_white()`.
///
/// # Safety
///
/// `p` must be a valid null-terminated C string.
#[export_name = "skip_colon_white"]
pub unsafe extern "C" fn rs_skip_colon_white(
    p: *const c_char,
    skipleadingwhite: c_int,
) -> *const c_char {
    if p.is_null() {
        return p;
    }

    let mut ptr = p;

    if skipleadingwhite != 0 {
        ptr = skipwhite(ptr) as *const c_char;
    }

    while *ptr as u8 == b':' {
        ptr = skipwhite(ptr.add(1)) as *const c_char;
    }

    ptr
}

/// Skip over a range specification in a command line.
///
/// Handles digits, marks, search patterns, offsets, and separators.
/// If `ctx` is non-null, sets `*ctx = EXPAND_NOTHING` when the range
/// ends with an incomplete mark or search pattern.
///
/// Matches C `skip_range()`.
///
/// # Safety
///
/// `cmd` must be a valid null-terminated C string.
/// `ctx` may be null or a valid pointer for writes.
#[export_name = "skip_range"]
pub unsafe extern "C" fn rs_skip_range(cmd: *const c_char, ctx: *mut c_int) -> *const c_char {
    if cmd.is_null() {
        return cmd;
    }

    let mut p = cmd;

    // Characters that can appear in a range specification
    const RANGE_CHARS: &[u8] = b" \t0123456789.$%'/?-+,;\\";

    while RANGE_CHARS.contains(&(*p as u8)) {
        if *p as u8 == b'\\' {
            let next = *p.add(1) as u8;
            if next == b'?' || next == b'/' || next == b'&' {
                p = p.add(1);
            } else {
                break;
            }
        } else if *p as u8 == b'\'' {
            p = p.add(1);
            if *p as u8 == 0 && !ctx.is_null() {
                *ctx = EXPAND_NOTHING;
            }
        } else if *p as u8 == b'/' || *p as u8 == b'?' {
            let delim = *p as u8;
            p = p.add(1);
            while *p as u8 != 0 && *p as u8 != delim {
                if *p as u8 == b'\\' && *p.add(1) as u8 != 0 {
                    p = p.add(1);
                }
                p = p.add(1);
            }
            if *p as u8 == 0 && !ctx.is_null() {
                *ctx = EXPAND_NOTHING;
            }
        }
        if *p as u8 != 0 {
            p = p.add(1);
        }
    }

    // Skip ":" and white space.
    rs_skip_colon_white(p, 0)
}

// =============================================================================
// rs_invalid_range — validate range for command
// =============================================================================

/// Check range in Ex command for validity.
///
/// Returns NULL when valid, error message when invalid.
///
/// Replaces C `invalid_range()`.
#[export_name = "invalid_range"]
pub unsafe extern "C" fn rs_invalid_range(eap: ExArgHandle) -> *mut c_char {
    let line1 = (*eap).line1;
    let line2 = (*eap).line2;

    if line1 < 0 || line2 < 0 || line1 > line2 {
        return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
    }

    let argt = (*eap).argt;
    if (argt & EX_RANGE) != 0 {
        let addr_type = (*eap).addr_type;
        match addr_type {
            x if x == ADDR_LINES => {
                let cmdidx = (*eap).cmdidx;
                let diff_extra = if cmdidx == crate::commands::CMD_DIFFGET
                    || cmdidx == crate::commands::CMD_DIFFPUT
                {
                    1
                } else {
                    0
                };
                if line2 > nvim_buf_get_line_count(nvim_get_curbuf()) + diff_extra {
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
            }
            x if x == ADDR_ARGUMENTS => {
                let argcount = nvim_get_argcount();
                // add 1 if ARGCOUNT is 0
                let limit = argcount + c_int::from(argcount == 0);
                if line2 > limit as i32 {
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
            }
            x if x == ADDR_BUFFERS => {
                if line1 < 1 || line2 > rs_get_highest_fnum() as i32 {
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
            }
            x if x == ADDR_LOADED_BUFFERS => {
                let first_loaded = nvim_docmd_first_loaded_fnum_or_fail();
                if first_loaded < 0 {
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
                if line1 < first_loaded as i32 {
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
                let last_loaded = nvim_docmd_last_loaded_fnum_or_fail();
                if last_loaded < 0 {
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
                if line2 > last_loaded as i32 {
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
            }
            x if x == ADDR_WINDOWS => {
                if line2 > nvim_docmd_last_win_nr() as i32 {
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
            }
            x if x == ADDR_TABS => {
                if line2 > nvim_docmd_last_tab_nr() as i32 {
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
            }
            x if x == ADDR_TABS_RELATIVE || x == ADDR_OTHER => {
                // Any range is OK.
            }
            x if x == ADDR_QUICKFIX => {
                debug_assert!(line2 >= 0);
                if line2 <= 0 {
                    if (*eap).addr_count == 0 {
                        return crate::gt(crate::E_NO_ERRORS_STR.as_ptr()) as *mut c_char;
                    }
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
            }
            x if x == ADDR_QUICKFIX_VALID => {
                if (line2 != 1 && (line2 as usize) > qf_get_valid_size(eap)) || line2 < 0 {
                    return crate::gt(crate::E_INVRANGE_STR.as_ptr()) as *mut c_char;
                }
            }
            x if x == ADDR_UNSIGNED || x == ADDR_NONE => {
                // Will give an error elsewhere.
            }
            _ => {}
        }
    }
    ptr::null_mut()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_range_new() {
        let range = LineRange::new();
        assert!(range.is_empty());
        assert!(!range.is_single());
        assert!(!range.is_pair());
        assert_eq!(range.count(), 0);
    }

    #[test]
    fn test_line_range_single() {
        let range = LineRange::single(5);
        assert!(!range.is_empty());
        assert!(range.is_single());
        assert!(!range.is_pair());
        assert_eq!(range.line1, 5);
        assert_eq!(range.line2, 5);
        assert_eq!(range.count(), 1);
    }

    #[test]
    fn test_line_range_pair() {
        let range = LineRange::from_pair(3, 10);
        assert!(!range.is_empty());
        assert!(!range.is_single());
        assert!(range.is_pair());
        assert_eq!(range.line1, 3);
        assert_eq!(range.line2, 10);
        assert_eq!(range.count(), 8);
    }

    #[test]
    fn test_line_range_normalize() {
        let mut range = LineRange::from_pair(10, 3);
        assert!(!range.is_valid());
        range.normalize();
        assert!(range.is_valid());
        assert_eq!(range.line1, 3);
        assert_eq!(range.line2, 10);
    }

    #[test]
    fn test_line_range_clamp() {
        let mut range = LineRange::from_pair(0, 100);
        range.clamp(50);
        assert_eq!(range.line1, 1);
        assert_eq!(range.line2, 50);

        let mut range = LineRange::from_pair(-5, 30);
        range.clamp(20);
        assert_eq!(range.line1, 1);
        assert_eq!(range.line2, 20);
    }

    #[test]
    fn test_line_range_contains() {
        let range = LineRange::from_pair(5, 15);
        assert!(!range.contains(4));
        assert!(range.contains(5));
        assert!(range.contains(10));
        assert!(range.contains(15));
        assert!(!range.contains(16));
    }

    #[test]
    fn test_is_addr_char() {
        assert!(is_addr_char(b'0'));
        assert!(is_addr_char(b'9'));
        assert!(is_addr_char(b'.'));
        assert!(is_addr_char(b'$'));
        assert!(is_addr_char(b'%'));
        assert!(is_addr_char(b'\''));
        assert!(is_addr_char(b'/'));
        assert!(is_addr_char(b'?'));
        assert!(is_addr_char(b'+'));
        assert!(is_addr_char(b'-'));
        assert!(!is_addr_char(b'x'));
        assert!(!is_addr_char(b' '));
    }

    #[test]
    fn test_is_range_sep() {
        assert!(is_range_sep(b','));
        assert!(is_range_sep(b';'));
        assert!(!is_range_sep(b':'));
        assert!(!is_range_sep(b'.'));
    }

    #[test]
    fn test_range_parse_state() {
        let state = RangeParseState::new();
        assert!(!state.has_special());

        let mut state = RangeParseState::new();
        state.whole_file = true;
        assert!(state.has_special());

        state.reset();
        assert!(!state.has_special());

        state.used_mark = true;
        assert!(state.has_special());
    }

    #[test]
    fn test_ffi_addr_chars() {
        assert_eq!(rs_is_addr_char(b'.' as c_int), 1);
        assert_eq!(rs_is_addr_char(b'$' as c_int), 1);
        assert_eq!(rs_is_addr_char(b'x' as c_int), 0);
        assert_eq!(rs_is_addr_char(-1), 0);
        assert_eq!(rs_is_addr_char(200), 0);

        assert_eq!(rs_is_addr_digit(b'5' as c_int), 1);
        assert_eq!(rs_is_addr_digit(b'+' as c_int), 1);
        assert_eq!(rs_is_addr_digit(b'.' as c_int), 0);

        assert_eq!(rs_is_range_sep(b',' as c_int), 1);
        assert_eq!(rs_is_range_sep(b';' as c_int), 1);
        assert_eq!(rs_is_range_sep(b':' as c_int), 0);
    }

    // Note: rs_skip_colon_white, rs_skip_range tests require C FFI (skipwhite)
    // and are verified through integration tests (just smoke-test) instead.
}
