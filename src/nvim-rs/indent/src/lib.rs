//! Indentation utilities for Neovim.
//!
//! This crate provides FFI-compatible indentation calculation functions.

#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

pub mod breakindent;
pub mod change_indent;
pub mod checks;
pub mod copy_indent;
pub mod getters;
pub mod helpers;
pub mod ins_try_si;
pub mod lisp_indent;
pub mod reindent;
pub mod retab;
pub mod set_indent;

use std::ffi::c_char;
use std::ffi::c_int;

use std::ffi::c_void;

use nvim_buffer::BufHandle;
use nvim_memory::xmalloc;

/// Opaque handle to a window (win_T*)
pub type WinHandle = *mut c_void;

// C accessor functions for buffer properties
extern "C" {
    static mut got_int: bool;
    fn nvim_buf_get_p_sw(buf: BufHandle) -> i64;
    fn nvim_buf_get_p_ts(buf: BufHandle) -> i64;
    fn nvim_buf_get_p_vts_array(buf: BufHandle) -> *const c_int;
    fn nvim_get_p_paste() -> c_int;
    fn nvim_curbuf_get_p_cin() -> c_int;
    fn nvim_curbuf_get_inde_nonempty() -> c_int;
    fn nvim_curbuf_get_p_si() -> c_int;
}

// Window option accessors
extern "C" {
    fn nvim_win_get_p_briopt(wp: WinHandle) -> *const c_char;
    fn nvim_win_set_briopt_shift(wp: WinHandle, val: c_int);
    fn nvim_win_set_briopt_min(wp: WinHandle, val: c_int);
    fn nvim_win_set_briopt_sbr(wp: WinHandle, val: c_int);
    fn nvim_win_set_briopt_list(wp: WinHandle, val: c_int);
    fn nvim_win_set_briopt_vcol(wp: WinHandle, val: c_int);
    fn nvim_get_empty_string_option() -> *const c_char;
}

// Error message functions
extern "C" {
    fn emsg(s: *const c_char) -> c_int;
    fn semsg(fmt: *const c_char, ...) -> c_int;
    fn gettext(msgid: *const c_char) -> *const c_char;
}

// Error message constants (need gettext translation)
extern "C" {
    // "E487: Argument must be positive"
    static e_positive: *const c_char;
    // "E475: Invalid argument: %s"
    static e_invarg2: *const c_char;
    // "E1240: Resulting text too long"
    static e_resulting_text_too_long: *const c_char;
}

// Global state
extern "C" {
    static mut trylevel: c_int;
}

/// Translate a gettext message.
///
/// # Safety
/// The pointer must be a valid C string from error constants.
unsafe fn translate(msgid: *const c_char) -> *const c_char {
    gettext(msgid)
}

/// Maximum allowed tabstop value
const TABSTOP_MAX: c_int = 9999;

// Type aliases matching C types
// colnr_T = int (i32)
// OptInt = int64_t (i64)

const TAB: c_char = b'\t' as c_char;

// ============================================================================
// Error Helpers
// ============================================================================

/// Give a "resulting text too long" error and maybe set got_int.
///
/// This is used when text operations would create a line that is too long.
/// When not inside a try/catch block, it also sets got_int to break out
/// of any loop.
///
/// # Safety
/// Accesses global state (trylevel, got_int).
#[export_name = "emsg_text_too_long"]
pub unsafe extern "C" fn rs_emsg_text_too_long() {
    emsg(translate(e_resulting_text_too_long));
    // when not inside a try/catch set got_int to break out of any loop
    if trylevel == 0 {
        unsafe {
            got_int = true;
        }
    }
}

// ============================================================================
// C-indenting State
// ============================================================================

/// Check that C-indenting is on.
///
/// Returns true if paste mode is off and either 'cindent' is set or
/// 'indentexpr' is non-empty.
///
/// # Safety
/// Calls C accessor functions for global state.
#[export_name = "cindent_on"]
pub unsafe extern "C" fn rs_cindent_on() -> bool {
    nvim_get_p_paste() == 0
        && (nvim_curbuf_get_p_cin() != 0 || nvim_curbuf_get_inde_nonempty() != 0)
}

/// Check if the conditions are OK for smart indenting.
///
/// Returns true if:
/// - 'smartindent' is set, AND
/// - 'cindent' is off, AND
/// - 'indentexpr' is empty, AND
/// - 'paste' mode is off
///
/// # Safety
/// Calls C accessor functions for buffer and global options.
#[must_use]
#[export_name = "may_do_si"]
pub unsafe extern "C" fn rs_may_do_si() -> bool {
    nvim_curbuf_get_p_si() != 0
        && nvim_curbuf_get_p_cin() == 0
        && nvim_curbuf_get_inde_nonempty() == 0
        && nvim_get_p_paste() == 0
}

const SPACE: c_char = b' ' as c_char;

/// Calculate the number of screen spaces a tab will occupy.
///
/// If `vts` is set (non-null and vts[0] > 0) then the tab widths are taken
/// from that array, otherwise the value of `ts` is used.
///
/// The `vts` array format: vts[0] = count, vts[1..count+1] = tabstop widths
///
/// # Safety
/// If `vts` is non-null, it must point to a valid array where vts[0] contains
/// the count and vts[1..count+1] contains valid tabstop values.
#[must_use]
#[export_name = "tabstop_padding"]
pub unsafe extern "C" fn rs_tabstop_padding(col: c_int, ts_arg: i64, vts: *const c_int) -> c_int {
    let ts: i64 = if ts_arg == 0 { 8 } else { ts_arg };

    // If no variable tabstops, use fixed width
    if vts.is_null() || *vts == 0 {
        return (ts - (i64::from(col) % ts)) as c_int;
    }

    let tabcount = *vts;
    let mut tabcol: c_int = 0;
    let mut padding: c_int = 0;
    let mut t: c_int = 1;

    while t <= tabcount {
        tabcol += *vts.offset(t as isize);
        if tabcol > col {
            padding = tabcol - col;
            break;
        }
        t += 1;
    }

    if t > tabcount {
        // Past all defined tabstops, use the last one repeatedly
        let last_ts = *vts.offset(tabcount as isize);
        padding = last_ts - ((col - tabcol) % last_ts);
    }

    padding
}

/// Compute the size of the indent (in window cells) in line `ptr`,
/// using tabstops.
///
/// # Safety
/// - `ptr` must point to a valid null-terminated C string.
/// - If `vts` is non-null, it must point to a valid tabstop array.
#[must_use]
#[export_name = "indent_size_ts"]
pub unsafe extern "C" fn rs_indent_size_ts(
    ptr: *const c_char,
    ts: i64,
    vts: *const c_int,
) -> c_int {
    if ptr.is_null() {
        return 0;
    }

    let mut p = ptr;
    let mut vcol: c_int = 0;
    let tabstop_width: c_int;
    let mut next_tab_vcol: c_int;

    if vts.is_null() || *vts < 1 {
        // Tab has fixed width
        tabstop_width = if ts == 0 { 8 } else { ts as c_int };
        next_tab_vcol = tabstop_width;
    } else {
        // Tab has variable width
        let tabcount = *vts;
        let mut cur_tabstop_idx: c_int = 1;
        let last_tabstop_idx = tabcount;

        // Process variable-width tabstops
        while cur_tabstop_idx != last_tabstop_idx {
            let mut cur_vcol = vcol;
            vcol += *vts.offset(cur_tabstop_idx as isize);
            cur_tabstop_idx += 1;

            loop {
                let c = *p;
                p = p.add(1);
                if c == SPACE {
                    cur_vcol += 1;
                } else if c == TAB {
                    // Tab found, break to next tabstop
                    break;
                } else {
                    // Non-whitespace found
                    return cur_vcol;
                }
                if cur_vcol == vcol {
                    break;
                }
            }
        }

        tabstop_width = *vts.offset(last_tabstop_idx as isize);
        next_tab_vcol = vcol + tabstop_width;
    }

    // Process remaining characters with fixed tabstop width
    loop {
        let c = *p;
        p = p.add(1);
        if c == SPACE {
            vcol += 1;
            if vcol == next_tab_vcol {
                next_tab_vcol += tabstop_width;
            }
        } else if c == TAB {
            vcol = next_tab_vcol;
            next_tab_vcol += tabstop_width;
        } else {
            return vcol;
        }
    }
}

/// Find the size of the tab that covers a particular column.
///
/// If this is being called as part of a shift operation, `col` is not the cursor
/// column but is the column number to the left of the first non-whitespace
/// character in the line. If the shift is to the left (`left == true`), then
/// return the size of the tab interval to the left of the column.
///
/// # Safety
/// If `vts` is non-null, it must point to a valid tabstop array where
/// vts[0] is the count and vts[1..count+1] are the tabstop widths.
#[must_use]
#[export_name = "tabstop_at"]
pub unsafe extern "C" fn rs_tabstop_at(
    col: c_int,
    ts: i64,
    vts: *const c_int,
    left: bool,
) -> c_int {
    // If no variable tabstops, use fixed width
    if vts.is_null() || *vts == 0 {
        return ts as c_int;
    }

    let tabcount = *vts;
    let mut tabcol: c_int = 0;
    let mut tab_size: c_int = 0;
    let mut t: c_int = 1;

    while t <= tabcount {
        tabcol += *vts.offset(t as isize);
        if tabcol > col {
            // If shifting left and we're at the first tabstop, shift to left margin
            if left && t == 1 {
                tab_size = col;
            } else {
                let idx = if left { t - 1 } else { t };
                tab_size = *vts.offset(idx as isize);
            }
            break;
        }
        t += 1;
    }

    // Past all defined tabstops, use the last one
    if t > tabcount {
        tab_size = *vts.offset(tabcount as isize);
    }

    tab_size
}

/// Find the column on which a tab starts.
///
/// # Safety
/// If `vts` is non-null, it must point to a valid tabstop array.
#[must_use]
#[export_name = "tabstop_start"]
pub unsafe extern "C" fn rs_tabstop_start(col: c_int, ts: c_int, vts: *const c_int) -> c_int {
    if vts.is_null() || *vts == 0 {
        return col - col % ts;
    }

    let tabcount = *vts;
    let mut tabcol: c_int = 0;

    for t in 1..=tabcount {
        tabcol += *vts.offset(t as isize);
        if tabcol > col {
            return tabcol - *vts.offset(t as isize);
        }
    }

    // Past all defined tabstops
    let last_ts = *vts.offset(tabcount as isize);
    let excess = tabcol % last_ts;
    col - (col - excess) % last_ts
}

/// Result from tabstop_fromto calculation.
#[repr(C)]
pub struct TabstopFromtoResult {
    /// Number of tabs to use
    pub ntabs: c_int,
    /// Number of spaces to use
    pub nspcs: c_int,
}

/// Calculate the number of tabs and spaces needed to go from start_col to end_col.
///
/// Given a range of columns, this function calculates the optimal combination
/// of tabs and spaces to cover that range.
///
/// # Arguments
/// * `start_col` - Starting column position
/// * `end_col` - Ending column position
/// * `ts` - Fixed tabstop width (must be > 0)
/// * `vts` - Variable tabstop array (can be null)
///
/// # Returns
/// A `TabstopFromtoResult` with the number of tabs and spaces needed.
///
/// # Safety
/// If `vts` is non-null, it must point to a valid tabstop array.
#[no_mangle]
pub unsafe extern "C" fn rs_tabstop_fromto(
    start_col: c_int,
    end_col: c_int,
    ts: c_int,
    vts: *const c_int,
) -> TabstopFromtoResult {
    debug_assert!(ts > 0, "ts must be positive");

    let mut spaces = end_col - start_col;

    // If no variable tabstops, use fixed width
    if vts.is_null() || *vts == 0 {
        let mut tabs = 0;

        let initspc = ts - (start_col % ts);
        if spaces >= initspc {
            spaces -= initspc;
            tabs += 1;
        }
        tabs += spaces / ts;
        spaces -= (spaces / ts) * ts;

        return TabstopFromtoResult {
            ntabs: tabs,
            nspcs: spaces,
        };
    }

    // Variable tabstops
    let tabcount = *vts;
    let mut tabcol: c_int = 0;
    let mut padding: c_int = 0;
    let mut t: c_int = 1;

    // Find the padding needed to reach the next tabstop
    while t <= tabcount {
        tabcol += *vts.offset(t as isize);
        if tabcol > start_col {
            padding = tabcol - start_col;
            break;
        }
        t += 1;
    }
    if t > tabcount {
        let last_ts = *vts.offset(tabcount as isize);
        padding = last_ts - ((start_col - tabcol) % last_ts);
    }

    // If the space needed is less than the padding no tabs can be used
    if spaces < padding {
        return TabstopFromtoResult {
            ntabs: 0,
            nspcs: spaces,
        };
    }

    let mut ntabs = 1;
    spaces -= padding;

    // At least one tab has been used. See if any more will fit.
    t += 1;
    while spaces != 0 && t <= tabcount {
        padding = *vts.offset(t as isize);
        if spaces < padding {
            return TabstopFromtoResult {
                ntabs,
                nspcs: spaces,
            };
        }
        ntabs += 1;
        spaces -= padding;
        t += 1;
    }

    let last_ts = *vts.offset(tabcount as isize);
    ntabs += spaces / last_ts;
    let nspcs = spaces % last_ts;

    TabstopFromtoResult { ntabs, nspcs }
}

/// Compare two tabstop arrays for equality.
///
/// # Safety
/// If either pointer is non-null, it must point to a valid tabstop array.
#[must_use]
#[export_name = "tabstop_eq"]
pub unsafe extern "C" fn rs_tabstop_eq(ts1: *const c_int, ts2: *const c_int) -> bool {
    // Handle null cases
    if ts1.is_null() && ts2.is_null() {
        return true;
    }
    if ts1.is_null() || ts2.is_null() {
        return false;
    }
    // Same pointer
    if ts1 == ts2 {
        return true;
    }

    let count1 = *ts1;
    let count2 = *ts2;
    if count1 != count2 {
        return false;
    }

    for t in 1..=count1 {
        if *ts1.offset(t as isize) != *ts2.offset(t as isize) {
            return false;
        }
    }

    true
}

/// Return a count of the number of tabstops.
///
/// # Safety
/// If `ts` is non-null, it must point to a valid tabstop array.
#[must_use]
#[export_name = "tabstop_count"]
pub unsafe extern "C" fn rs_tabstop_count(ts: *const c_int) -> c_int {
    if ts.is_null() {
        0
    } else {
        *ts
    }
}

/// Return the first tabstop, or 8 if there are no tabstops defined.
///
/// # Safety
/// If `ts` is non-null, it must point to a valid tabstop array with at least 2 elements.
#[must_use]
#[export_name = "tabstop_first"]
pub unsafe extern "C" fn rs_tabstop_first(ts: *const c_int) -> c_int {
    if ts.is_null() {
        8
    } else {
        *ts.offset(1)
    }
}

/// Compute the size of the indent (in window cells) in line `ptr`,
/// without tabstops (count tab as ^I or <09>).
///
/// This function treats tabs as their control-character display width
/// (typically 2 characters for ^I).
///
/// # Safety
/// - `ptr` must point to a valid null-terminated C string.
/// - The global `g_chartab` array must be initialized.
#[must_use]
#[export_name = "indent_size_no_ts"]
pub unsafe extern "C" fn rs_indent_size_no_ts(ptr: *const c_char) -> c_int {
    if ptr.is_null() {
        return 0;
    }

    // Get the display width of a tab character (^I = 2 cells)
    // TAB is 0x09 which is < 0x80, so byte2cells will work correctly
    let tab_size = nvim_charset::byte2cells(b'\t');

    let mut p = ptr;
    let mut vcol: c_int = 0;

    loop {
        let c = *p;
        p = p.add(1);
        if c == SPACE {
            vcol += 1;
        } else if c == TAB {
            vcol += tab_size;
        } else {
            return vcol;
        }
    }
}

// ============================================================================
// Tabstop Parsing
// ============================================================================

/// Helper to check if a byte is an ASCII digit.
fn is_ascii_digit(c: c_char) -> bool {
    c >= b'0' as c_char && c <= b'9' as c_char
}

/// Parse a vartabstop string and set the integer values array.
///
/// Parses a comma-separated string of tabstop values (e.g., "4,8,12") and
/// allocates an array where `array[0]` is the count and `array[1..]` are the values.
///
/// # Arguments
/// * `var` - Null-terminated string containing comma-separated tabstop values
/// * `array` - Output pointer for the allocated array (caller must free)
///
/// # Returns
/// * `true` if successful
/// * `false` if there was an error (invalid format, non-positive value, etc.)
///
/// # Safety
/// * `var` must be a valid null-terminated C string
/// * `array` must be a valid pointer to a `*mut c_int` (output parameter)
/// * The returned array must be freed by the caller using `xfree`
#[must_use]
#[export_name = "tabstop_set"]
pub unsafe extern "C" fn rs_tabstop_set(var: *const c_char, array: *mut *mut c_int) -> bool {
    if var.is_null() || array.is_null() {
        return false;
    }

    // Empty string or "0" -> no vartabstops
    if *var == 0 || (*var == b'0' as c_char && *var.add(1) == 0) {
        *array = std::ptr::null_mut();
        return true;
    }

    // First pass: validate and count values
    let mut valcount = 1;
    let mut cp = var;

    while *cp != 0 {
        // At start of string or after comma: parse a number
        if cp == var || *cp.sub(1) == b',' as c_char {
            // Parse the number
            let mut num: i64 = 0;
            let start = cp;
            let mut has_digits = false;

            while is_ascii_digit(*cp) {
                has_digits = true;
                num = num * 10 + i64::from(*cp - b'0' as c_char);
                cp = cp.add(1);
            }

            if num <= 0 {
                if has_digits {
                    // Number was parsed but <= 0
                    emsg(translate(e_positive));
                } else {
                    // No digits found
                    semsg(translate(e_invarg2), start);
                }
                return false;
            }

            continue;
        }

        if is_ascii_digit(*cp) {
            cp = cp.add(1);
            continue;
        }

        // Check for valid comma
        if *cp == b',' as c_char && cp > var && *cp.sub(1) != b',' as c_char && *cp.add(1) != 0 {
            valcount += 1;
            cp = cp.add(1);
            continue;
        }

        // Invalid character
        semsg(translate(e_invarg2), var);
        return false;
    }

    // Allocate the array: count + values
    let size = (valcount + 1) as usize * std::mem::size_of::<c_int>();
    let arr = xmalloc(size).cast::<c_int>();
    *arr = valcount;

    // Second pass: fill the array
    let mut t = 1;
    cp = var;
    while *cp != 0 {
        // Parse the number
        let mut n: c_int = 0;
        while is_ascii_digit(*cp) {
            n = n * 10 + (*cp - b'0' as c_char) as c_int;
            cp = cp.add(1);
        }

        // Validate: catch negative, overflow, ridiculously big values
        if n <= 0 || n > TABSTOP_MAX {
            semsg(translate(e_invarg2), var);
            nvim_memory::xfree(arr.cast());
            *array = std::ptr::null_mut();
            return false;
        }

        *arr.add(t) = n;
        t += 1;

        // Skip to next number (past comma)
        while *cp != 0 && *cp != b',' as c_char {
            cp = cp.add(1);
        }
        if *cp != 0 {
            cp = cp.add(1);
        }
    }

    *array = arr;
    true
}

// ============================================================================
// Breakindent Option Parsing
// ============================================================================

/// Parsed breakindent option values.
struct BrioptValues {
    shift: c_int,
    min: c_int,
    sbr: bool,
    list: c_int,
    vcol: c_int,
}

impl Default for BrioptValues {
    fn default() -> Self {
        Self {
            shift: 0,
            min: 20,
            sbr: false,
            list: 0,
            vcol: 0,
        }
    }
}

/// Parse digits from a string, advancing the pointer.
///
/// # Safety
/// `pp` must point to a valid pointer to a null-terminated string.
unsafe fn parse_digits(pp: &mut *const c_char, allow_negative: bool) -> c_int {
    let mut p = *pp;
    let mut negative = false;

    // Handle optional negative sign
    if allow_negative && *p == b'-' as c_char {
        negative = true;
        p = p.add(1);
    }

    let mut num: c_int = 0;
    while is_ascii_digit(*p) {
        num = num * 10 + (*p - b'0' as c_char) as c_int;
        p = p.add(1);
    }

    *pp = p;
    if negative {
        -num
    } else {
        num
    }
}

/// Check if a string starts with a prefix.
///
/// # Safety
/// Both pointers must be valid null-terminated strings.
unsafe fn starts_with(s: *const c_char, prefix: &[u8]) -> bool {
    for (i, &byte) in prefix.iter().enumerate() {
        if *s.add(i) != byte as c_char {
            return false;
        }
    }
    true
}

/// Check "briopt" as 'breakindentopt' and update the members of "wp".
///
/// Parses options like "shift:N", "min:N", "sbr", "list:N", "column:N".
///
/// # Arguments
/// * `briopt` - The option string to parse (NULL means use wp->w_p_briopt)
/// * `wp` - The window to update (NULL means only validate briopt)
///
/// # Returns
/// * `true` on success
/// * `false` if the option string is invalid
///
/// # Safety
/// * If `briopt` is non-null, it must be a valid null-terminated string
/// * If `wp` is non-null, it must be a valid window handle
#[must_use]
#[export_name = "briopt_check"]
pub unsafe extern "C" fn rs_briopt_check(briopt: *const c_char, wp: WinHandle) -> bool {
    let mut values = BrioptValues::default();

    // Determine which string to parse
    let p_start = if !briopt.is_null() {
        briopt
    } else if !wp.is_null() {
        nvim_win_get_p_briopt(wp)
    } else {
        nvim_get_empty_string_option()
    };

    if p_start.is_null() {
        return true;
    }

    let mut p = p_start;

    while *p != 0 {
        // Parse "shift:N" (can be negative)
        if starts_with(p, b"shift:")
            && ((*p.add(6) == b'-' as c_char && is_ascii_digit(*p.add(7)))
                || is_ascii_digit(*p.add(6)))
        {
            p = p.add(6);
            values.shift = parse_digits(&mut p, true);
        }
        // Parse "min:N"
        else if starts_with(p, b"min:") && is_ascii_digit(*p.add(4)) {
            p = p.add(4);
            values.min = parse_digits(&mut p, false);
        }
        // Parse "sbr"
        else if starts_with(p, b"sbr") {
            p = p.add(3);
            values.sbr = true;
        }
        // Parse "list:N"
        else if starts_with(p, b"list:") {
            p = p.add(5);
            values.list = parse_digits(&mut p, false);
        }
        // Parse "column:N"
        else if starts_with(p, b"column:") {
            p = p.add(7);
            values.vcol = parse_digits(&mut p, false);
        }

        // Must be at comma or end of string
        if *p != b',' as c_char && *p != 0 {
            return false;
        }
        if *p == b',' as c_char {
            p = p.add(1);
        }
    }

    // If wp is NULL, just validate (return OK)
    if wp.is_null() {
        return true;
    }

    // Apply the parsed values to the window
    nvim_win_set_briopt_shift(wp, values.shift);
    nvim_win_set_briopt_min(wp, values.min);
    nvim_win_set_briopt_sbr(wp, if values.sbr { 1 } else { 0 });
    nvim_win_set_briopt_list(wp, values.list);
    nvim_win_set_briopt_vcol(wp, values.vcol);

    true
}

// ============================================================================
// Shiftwidth Calculations
// ============================================================================

/// Get the effective shiftwidth value at a given column.
///
/// If 'shiftwidth' is set (non-zero), returns that value.
/// Otherwise, uses the tabstop size at the given column.
///
/// # Safety
/// The `buf` parameter must be a valid buffer pointer.
#[must_use]
#[export_name = "get_sw_value_col"]
pub unsafe extern "C" fn rs_get_sw_value_col(buf: BufHandle, col: c_int, left: bool) -> c_int {
    if buf.is_null() {
        return 8; // Default shiftwidth
    }

    let sw = nvim_buf_get_p_sw(buf);
    if sw != 0 {
        return sw as c_int;
    }

    // Use tabstop_at when shiftwidth is 0
    let ts = nvim_buf_get_p_ts(buf);
    let vts = nvim_buf_get_p_vts_array(buf);
    rs_tabstop_at(col, ts, vts, left)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_tabstop_padding_fixed() {
        unsafe {
            // With null vts, use fixed tabstops
            // col=0, ts=8 -> padding = 8 - (0 % 8) = 8
            assert_eq!(rs_tabstop_padding(0, 8, std::ptr::null()), 8);
            // col=1, ts=8 -> padding = 8 - (1 % 8) = 7
            assert_eq!(rs_tabstop_padding(1, 8, std::ptr::null()), 7);
            // col=7, ts=8 -> padding = 8 - (7 % 8) = 1
            assert_eq!(rs_tabstop_padding(7, 8, std::ptr::null()), 1);
            // col=8, ts=8 -> padding = 8 - (8 % 8) = 8
            assert_eq!(rs_tabstop_padding(8, 8, std::ptr::null()), 8);
            // col=9, ts=8 -> padding = 8 - (9 % 8) = 7
            assert_eq!(rs_tabstop_padding(9, 8, std::ptr::null()), 7);

            // With ts=4
            assert_eq!(rs_tabstop_padding(0, 4, std::ptr::null()), 4);
            assert_eq!(rs_tabstop_padding(1, 4, std::ptr::null()), 3);
            assert_eq!(rs_tabstop_padding(3, 4, std::ptr::null()), 1);
            assert_eq!(rs_tabstop_padding(4, 4, std::ptr::null()), 4);

            // With ts=0 (should default to 8)
            assert_eq!(rs_tabstop_padding(0, 0, std::ptr::null()), 8);
            assert_eq!(rs_tabstop_padding(4, 0, std::ptr::null()), 4);
        }
    }

    #[test]
    fn test_tabstop_padding_variable() {
        unsafe {
            // Variable tabstops: [count, ts1, ts2, ...]
            // vts = [2, 4, 8] means first tab at 4, second at 4+8=12, then every 8 after
            let vts: [c_int; 3] = [2, 4, 8];

            // col=0 -> next tab at 4, padding = 4
            assert_eq!(rs_tabstop_padding(0, 8, vts.as_ptr()), 4);
            // col=3 -> next tab at 4, padding = 1
            assert_eq!(rs_tabstop_padding(3, 8, vts.as_ptr()), 1);
            // col=4 -> next tab at 12, padding = 8
            assert_eq!(rs_tabstop_padding(4, 8, vts.as_ptr()), 8);
            // col=10 -> next tab at 12, padding = 2
            assert_eq!(rs_tabstop_padding(10, 8, vts.as_ptr()), 2);
            // col=12 -> past all defined, use last (8), next at 20, padding = 8
            assert_eq!(rs_tabstop_padding(12, 8, vts.as_ptr()), 8);
            // col=15 -> past all defined, next at 20, padding = 5
            assert_eq!(rs_tabstop_padding(15, 8, vts.as_ptr()), 5);
        }
    }

    #[test]
    fn test_indent_size_ts_fixed() {
        unsafe {
            // No indent
            let s = CString::new("hello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 0);

            // Spaces only
            let s = CString::new("    hello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 4);

            // Tab only (at col 0, tab goes to col 8)
            let s = CString::new("\thello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 8);

            // Tab with ts=4
            let s = CString::new("\thello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 4, std::ptr::null()), 4);

            // Spaces then tab
            let s = CString::new("  \thello").unwrap();
            // 2 spaces + tab -> tab takes us to 8
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 8);

            // Tab then spaces
            let s = CString::new("\t  hello").unwrap();
            // tab to 8, then 2 spaces = 10
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 10);

            // Multiple tabs
            let s = CString::new("\t\thello").unwrap();
            // tab to 8, tab to 16
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 16);

            // Empty string
            let s = CString::new("").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 0);

            // All whitespace
            let s = CString::new("   ").unwrap();
            // Should return 3 when hitting NUL
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, std::ptr::null()), 3);
        }
    }

    #[test]
    fn test_indent_size_ts_variable() {
        unsafe {
            // vts = [2, 4, 8] means first tab at 4, second at 12, then every 8 after
            let vts: [c_int; 3] = [2, 4, 8];

            // Single tab: goes to position 4
            let s = CString::new("\thello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, vts.as_ptr()), 4);

            // Two tabs: first to 4, second to 12
            let s = CString::new("\t\thello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, vts.as_ptr()), 12);

            // Three tabs: 4, 12, 20
            let s = CString::new("\t\t\thello").unwrap();
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, vts.as_ptr()), 20);

            // Spaces then tab
            let s = CString::new("  \thello").unwrap();
            // 2 spaces, then tab to 4
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, vts.as_ptr()), 4);

            // 4 spaces then tab
            let s = CString::new("    \thello").unwrap();
            // 4 spaces = at position 4, tab to 12
            assert_eq!(rs_indent_size_ts(s.as_ptr(), 8, vts.as_ptr()), 12);
        }
    }

    #[test]
    fn test_tabstop_at_fixed() {
        unsafe {
            // With null vts, returns ts directly
            assert_eq!(rs_tabstop_at(0, 8, std::ptr::null(), false), 8);
            assert_eq!(rs_tabstop_at(5, 8, std::ptr::null(), false), 8);
            assert_eq!(rs_tabstop_at(0, 4, std::ptr::null(), true), 4);
        }
    }

    #[test]
    fn test_tabstop_at_variable() {
        unsafe {
            // vts = [2, 4, 8] means tabstops at 4, 12, then every 8
            let vts: [c_int; 3] = [2, 4, 8];

            // At col 0, not shifting left -> first tabstop size is 4
            assert_eq!(rs_tabstop_at(0, 8, vts.as_ptr(), false), 4);
            // At col 2, not shifting left -> still in first interval, size 4
            assert_eq!(rs_tabstop_at(2, 8, vts.as_ptr(), false), 4);
            // At col 5, not shifting left -> in second interval, size 8
            assert_eq!(rs_tabstop_at(5, 8, vts.as_ptr(), false), 8);
            // Past all defined tabstops -> use last (8)
            assert_eq!(rs_tabstop_at(15, 8, vts.as_ptr(), false), 8);

            // Shifting left at col 2 (in first tabstop) -> returns col value
            assert_eq!(rs_tabstop_at(2, 8, vts.as_ptr(), true), 2);
            // Shifting left at col 5 (in second tabstop) -> use previous tabstop size (4)
            assert_eq!(rs_tabstop_at(5, 8, vts.as_ptr(), true), 4);
        }
    }

    #[test]
    fn test_tabstop_start_fixed() {
        unsafe {
            // With null vts, col - col % ts
            assert_eq!(rs_tabstop_start(0, 8, std::ptr::null()), 0);
            assert_eq!(rs_tabstop_start(5, 8, std::ptr::null()), 0);
            assert_eq!(rs_tabstop_start(8, 8, std::ptr::null()), 8);
            assert_eq!(rs_tabstop_start(10, 8, std::ptr::null()), 8);
            assert_eq!(rs_tabstop_start(16, 8, std::ptr::null()), 16);

            assert_eq!(rs_tabstop_start(0, 4, std::ptr::null()), 0);
            assert_eq!(rs_tabstop_start(3, 4, std::ptr::null()), 0);
            assert_eq!(rs_tabstop_start(4, 4, std::ptr::null()), 4);
            assert_eq!(rs_tabstop_start(7, 4, std::ptr::null()), 4);
        }
    }

    #[test]
    fn test_tabstop_start_variable() {
        unsafe {
            // vts = [2, 4, 8] means tabstops at 4, 12, then every 8
            let vts: [c_int; 3] = [2, 4, 8];

            // col 0-3: tab starts at 0 (before first tabstop at 4)
            assert_eq!(rs_tabstop_start(0, 8, vts.as_ptr()), 0);
            assert_eq!(rs_tabstop_start(3, 8, vts.as_ptr()), 0);
            // col 4-11: tab starts at 4 (first tabstop)
            assert_eq!(rs_tabstop_start(4, 8, vts.as_ptr()), 4);
            assert_eq!(rs_tabstop_start(10, 8, vts.as_ptr()), 4);
            // col 12+: past defined tabstops, use last interval (8)
            assert_eq!(rs_tabstop_start(12, 8, vts.as_ptr()), 12);
            assert_eq!(rs_tabstop_start(15, 8, vts.as_ptr()), 12);
            assert_eq!(rs_tabstop_start(20, 8, vts.as_ptr()), 20);
        }
    }

    #[test]
    fn test_tabstop_eq() {
        unsafe {
            // Both null
            assert!(rs_tabstop_eq(std::ptr::null(), std::ptr::null()));

            // One null, one not
            let ts1: [c_int; 3] = [2, 4, 8];
            assert!(!rs_tabstop_eq(ts1.as_ptr(), std::ptr::null()));
            assert!(!rs_tabstop_eq(std::ptr::null(), ts1.as_ptr()));

            // Same pointer
            assert!(rs_tabstop_eq(ts1.as_ptr(), ts1.as_ptr()));

            // Equal arrays
            let ts2: [c_int; 3] = [2, 4, 8];
            assert!(rs_tabstop_eq(ts1.as_ptr(), ts2.as_ptr()));

            // Different counts
            let ts3: [c_int; 4] = [3, 4, 8, 12];
            assert!(!rs_tabstop_eq(ts1.as_ptr(), ts3.as_ptr()));

            // Same count, different values
            let ts4: [c_int; 3] = [2, 4, 6];
            assert!(!rs_tabstop_eq(ts1.as_ptr(), ts4.as_ptr()));
        }
    }

    #[test]
    fn test_tabstop_count() {
        unsafe {
            // Null returns 0
            assert_eq!(rs_tabstop_count(std::ptr::null()), 0);

            // Non-null returns first element
            let ts: [c_int; 3] = [2, 4, 8];
            assert_eq!(rs_tabstop_count(ts.as_ptr()), 2);

            let ts2: [c_int; 4] = [3, 4, 8, 12];
            assert_eq!(rs_tabstop_count(ts2.as_ptr()), 3);
        }
    }

    #[test]
    fn test_tabstop_first() {
        unsafe {
            // Null returns 8
            assert_eq!(rs_tabstop_first(std::ptr::null()), 8);

            // Non-null returns second element (first tabstop)
            let ts: [c_int; 3] = [2, 4, 8];
            assert_eq!(rs_tabstop_first(ts.as_ptr()), 4);

            let ts2: [c_int; 4] = [3, 6, 8, 12];
            assert_eq!(rs_tabstop_first(ts2.as_ptr()), 6);
        }
    }

    #[test]
    fn test_tabstop_fromto_fixed() {
        unsafe {
            // start_col=0, end_col=8, ts=8 -> 1 tab, 0 spaces
            let result = rs_tabstop_fromto(0, 8, 8, std::ptr::null());
            assert_eq!(result.ntabs, 1);
            assert_eq!(result.nspcs, 0);

            // start_col=0, end_col=4, ts=8 -> 0 tabs, 4 spaces (not enough for a tab)
            let result = rs_tabstop_fromto(0, 4, 8, std::ptr::null());
            assert_eq!(result.ntabs, 0);
            assert_eq!(result.nspcs, 4);

            // start_col=0, end_col=16, ts=8 -> 2 tabs, 0 spaces
            let result = rs_tabstop_fromto(0, 16, 8, std::ptr::null());
            assert_eq!(result.ntabs, 2);
            assert_eq!(result.nspcs, 0);

            // start_col=0, end_col=10, ts=8 -> 1 tab (to 8), 2 spaces
            let result = rs_tabstop_fromto(0, 10, 8, std::ptr::null());
            assert_eq!(result.ntabs, 1);
            assert_eq!(result.nspcs, 2);

            // start_col=2, end_col=10, ts=8 -> 8 spaces to fill
            // From col 2, first tab goes to col 8 (6 spaces), leaving 2
            // 1 tab + 2 spaces
            let result = rs_tabstop_fromto(2, 10, 8, std::ptr::null());
            assert_eq!(result.ntabs, 1);
            assert_eq!(result.nspcs, 2);

            // start_col=2, end_col=18, ts=8 -> 16 spaces to fill
            // From col 2, first tab goes to col 8 (6 spaces)
            // Second tab goes to col 16 (8 spaces)
            // Remaining 2 spaces
            let result = rs_tabstop_fromto(2, 18, 8, std::ptr::null());
            assert_eq!(result.ntabs, 2);
            assert_eq!(result.nspcs, 2);
        }
    }

    #[test]
    fn test_tabstop_fromto_variable() {
        unsafe {
            // vts = [2, 4, 8] means tabstops at positions 4, 12, then every 8
            let vts: [c_int; 3] = [2, 4, 8];

            // start_col=0, end_col=4, ts=8 -> 1 tab (to 4), 0 spaces
            let result = rs_tabstop_fromto(0, 4, 8, vts.as_ptr());
            assert_eq!(result.ntabs, 1);
            assert_eq!(result.nspcs, 0);

            // start_col=0, end_col=2, ts=8 -> 0 tabs (not enough space), 2 spaces
            let result = rs_tabstop_fromto(0, 2, 8, vts.as_ptr());
            assert_eq!(result.ntabs, 0);
            assert_eq!(result.nspcs, 2);

            // start_col=0, end_col=12, ts=8 -> 2 tabs (to 4, then to 12), 0 spaces
            let result = rs_tabstop_fromto(0, 12, 8, vts.as_ptr());
            assert_eq!(result.ntabs, 2);
            assert_eq!(result.nspcs, 0);

            // start_col=0, end_col=20, ts=8 -> 3 tabs (4, 12, 20), 0 spaces
            let result = rs_tabstop_fromto(0, 20, 8, vts.as_ptr());
            assert_eq!(result.ntabs, 3);
            assert_eq!(result.nspcs, 0);

            // start_col=4, end_col=12, ts=8 -> 1 tab (from 4 to 12), 0 spaces
            let result = rs_tabstop_fromto(4, 12, 8, vts.as_ptr());
            assert_eq!(result.ntabs, 1);
            assert_eq!(result.nspcs, 0);
        }
    }

    #[test]
    fn test_whitespace_char_constants() {
        // Verify TAB and SPACE constants match expected values
        assert_eq!(TAB, b'\t' as c_char);
        assert_eq!(SPACE, b' ' as c_char);
    }
}
