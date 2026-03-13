//! VimL assertion functions for testing
//!
//! This module provides Rust implementations of VimL's `assert_*` and `test_*`
//! functions from `src/nvim/testing.c`.
//!
//! ## Architecture
//!
//! The module uses an opaque handle pattern where `typval_T*` pointers are
//! treated as opaque handles, with field access done through C accessor
//! functions.

#![allow(clippy::doc_markdown)]
// Allow dead code for functions that will be used in later migration phases
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void, CStr};
use std::io::Read;

use nvim_collections::garray::GArray;

// =============================================================================
// Type aliases for opaque handles
// =============================================================================

/// Opaque handle to a typval_T (VimL value).
pub type TypevalHandle = *const c_void;

/// Opaque handle to a mutable typval_T.
pub type TypevalHandleMut = *mut c_void;

/// Opaque handle for EvalFuncData union (8-byte union passed by value as pointer).
pub type EvalFuncData = *mut c_void;

// =============================================================================
// C accessor functions
// =============================================================================

extern "C" {
    // GArray operations (from collections crate, re-exported in C)
    fn ga_init(gap: *mut GArray, itemsize: c_int, growsize: c_int);
    fn ga_clear(gap: *mut GArray);
    fn ga_concat(gap: *mut GArray, s: *const c_char);
    fn ga_append(gap: *mut GArray, c: u8);

    // Sourcing information - for error location
    fn estack_sfile(which: c_int) -> *mut c_char;
    fn xfree(ptr: *mut c_void);

    // Typval string extraction
    fn tv_get_string(tv: TypevalHandle) -> *const c_char;

    // Assert error reporting - adds to v:errors
    fn assert_error(gap: *mut GArray);
}

// =============================================================================
// Constants
// =============================================================================

/// ESTACK_NONE constant from runtime_defs.h
const ESTACK_NONE: c_int = 0;

// =============================================================================
// Helper functions
// =============================================================================

// Access SOURCING_LNUM through C accessor.
extern "C" {
    fn nvim_testing_get_sourcing_lnum() -> i64;
}

/// Prepare a GArray for an assert error and add the sourcing position.
///
/// This mirrors the C `prepare_assert_error` function.
fn prepare_assert_error(gap: *mut GArray) {
    unsafe {
        ga_init(gap, 1, 100);

        let sname = estack_sfile(ESTACK_NONE);
        let sourcing_lnum = nvim_testing_get_sourcing_lnum();

        if !sname.is_null() {
            ga_concat(gap, sname);
            if sourcing_lnum > 0 {
                ga_concat(gap, c" ".as_ptr());
            }
        }

        if sourcing_lnum > 0 {
            // Format "line <number>"
            let mut buf = [0u8; 64];
            let len = format_line_number(&mut buf, sourcing_lnum);
            ga_concat(gap, buf.as_ptr().cast());
            let _ = len; // silence unused warning
        }

        if !sname.is_null() || sourcing_lnum > 0 {
            ga_concat(gap, c": ".as_ptr());
        }

        if !sname.is_null() {
            xfree(sname.cast());
        }
    }
}

/// Format "line <number>" into a buffer. Returns the length written.
fn format_line_number(buf: &mut [u8; 64], lnum: i64) -> usize {
    let prefix = b"line ";
    buf[..prefix.len()].copy_from_slice(prefix);

    // Convert number to string
    let mut num = lnum;
    let mut digits = [0u8; 20];
    let mut digit_count = 0;

    if num == 0 {
        digit_count = 1;
        digits[0] = b'0';
    } else {
        let negative = num < 0;
        if negative {
            num = -num;
        }

        while num > 0 {
            digits[digit_count] = b'0' + (num % 10) as u8;
            digit_count += 1;
            num /= 10;
        }

        if negative {
            digits[digit_count] = b'-';
            digit_count += 1;
        }
    }

    // Copy digits in reverse order
    let mut pos = prefix.len();
    for i in (0..digit_count).rev() {
        buf[pos] = digits[i];
        pos += 1;
    }
    buf[pos] = 0; // null terminator

    pos
}

// =============================================================================
// String escaping functions
// =============================================================================

// ASCII control character constants
const BS: u8 = 0x08; // Backspace
const TAB: u8 = 0x09; // Tab
const NL: u8 = 0x0A; // Newline
const FF: u8 = 0x0C; // Form feed
const CAR: u8 = 0x0D; // Carriage return
const ESC: u8 = 0x1B; // Escape

/// Append a character (possibly multi-byte) to the GArray, escaping unprintable characters.
/// Changes NL to \n, CR to \r, etc.
///
/// This mirrors the C `ga_concat_esc` function.
fn ga_concat_esc(gap: *mut GArray, p: *const u8, clen: usize) {
    unsafe {
        // Multi-byte character: copy as-is
        if clen > 1 {
            let mut buf = [0u8; 8];
            let copy_len = clen.min(7);
            std::ptr::copy_nonoverlapping(p, buf.as_mut_ptr(), copy_len);
            buf[copy_len] = 0;
            ga_concat(gap, buf.as_ptr().cast());
            return;
        }

        let c = *p;
        match c {
            BS => ga_concat(gap, c"\\b".as_ptr()),
            ESC => ga_concat(gap, c"\\e".as_ptr()),
            FF => ga_concat(gap, c"\\f".as_ptr()),
            NL => ga_concat(gap, c"\\n".as_ptr()),
            TAB => ga_concat(gap, c"\\t".as_ptr()),
            CAR => ga_concat(gap, c"\\r".as_ptr()),
            b'\\' => ga_concat(gap, c"\\\\".as_ptr()),
            _ => {
                if c < b' ' || c == 0x7f {
                    // Format as \xNN
                    let mut buf = [0u8; 8];
                    buf[0] = b'\\';
                    buf[1] = b'x';
                    buf[2] = hex_digit(c >> 4);
                    buf[3] = hex_digit(c & 0x0f);
                    buf[4] = 0;
                    ga_concat(gap, buf.as_ptr().cast());
                } else {
                    ga_append(gap, c);
                }
            }
        }
    }
}

/// Convert a nibble (0-15) to a hex digit.
#[inline]
const fn hex_digit(n: u8) -> u8 {
    if n < 10 {
        b'0' + n
    } else {
        b'a' + (n - 10)
    }
}

/// Format an integer into a buffer. Returns the length written (excluding NUL).
fn format_int(buf: &mut [u8], value: i32) -> usize {
    let mut num = value;
    let mut digits = [0u8; 12];
    let mut digit_count = 0;

    if num == 0 {
        digit_count = 1;
        digits[0] = b'0';
    } else {
        let negative = num < 0;
        if negative {
            num = -num;
        }

        while num > 0 {
            digits[digit_count] = b'0' + (num % 10) as u8;
            digit_count += 1;
            num /= 10;
        }

        if negative {
            digits[digit_count] = b'-';
            digit_count += 1;
        }
    }

    // Copy digits in reverse order
    let mut pos = 0;
    for i in (0..digit_count).rev() {
        if pos < buf.len() - 1 {
            buf[pos] = digits[i];
            pos += 1;
        }
    }
    if pos < buf.len() {
        buf[pos] = 0;
    }

    pos
}

/// Append a string to the GArray, escaping unprintable characters.
/// If the same character appears more than 20 times, it's shortened.
///
/// This mirrors the C `ga_concat_shorten_esc` function.
fn ga_concat_shorten_esc(gap: *mut GArray, s: *const c_char) {
    unsafe {
        if s.is_null() {
            ga_concat(gap, c"NULL".as_ptr());
            return;
        }

        let mut p = s.cast::<u8>();

        while *p != 0 {
            // Get the character and its byte length
            let (c, clen) = nvim_mbyte::mb_cptr2char_adv(std::slice::from_raw_parts(p, 6));
            let clen = clen.max(1); // Ensure at least 1 byte

            // Count consecutive occurrences of the same character
            let mut same_len = 1;
            let mut scan = p.add(clen);
            while *scan != 0 {
                let scan_c = nvim_mbyte::utf_ptr2char(std::slice::from_raw_parts(scan, 6));
                if scan_c != c {
                    break;
                }
                same_len += 1;
                scan = scan.add(clen);
            }

            if same_len > 20 {
                // Shorten: "\[<char> occurs <n> times]"
                ga_concat(gap, c"\\[".as_ptr());
                ga_concat_esc(gap, p, clen);
                ga_concat(gap, c" occurs ".as_ptr());

                let mut buf = [0u8; 16];
                format_int(&mut buf, same_len);
                ga_concat(gap, buf.as_ptr().cast());

                ga_concat(gap, c" times]".as_ptr());
                p = scan;
            } else {
                ga_concat_esc(gap, p, clen);
                p = p.add(clen);
            }
        }
    }
}

// =============================================================================
// Assert type enum
// =============================================================================

/// Type of assert_* check being performed.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssertType {
    Equal = 0,
    NotEqual = 1,
    Match = 2,
    NotMatch = 3,
    Fails = 4,
    Other = 5,
}

impl AssertType {
    /// Create from C integer.
    pub const fn from_c_int(val: c_int) -> Self {
        match val {
            0 => Self::Equal,
            1 => Self::NotEqual,
            2 => Self::Match,
            3 => Self::NotMatch,
            4 => Self::Fails,
            _ => Self::Other,
        }
    }
}

// =============================================================================
// Fill assert error
// =============================================================================

extern "C" {
    // Typval encoding functions
    fn encode_tv2echo(tv: TypevalHandle, len: *mut usize) -> *mut c_char;
    fn encode_tv2string(tv: TypevalHandle, len: *mut usize) -> *mut c_char;

    // Typval type checking
    fn nvim_testing_tv_get_type(tv: TypevalHandle) -> c_int;
    fn nvim_testing_tv_string_is_empty(tv: TypevalHandle) -> c_int;

    // Dictionary diffing (keep complex logic in C for now)
    fn nvim_testing_fill_dict_diff(
        gap: *mut GArray,
        exp_tv: TypevalHandle,
        got_tv: TypevalHandle,
        omitted: *mut c_int,
    );
}

// VAR_UNKNOWN constant
const VAR_UNKNOWN: c_int = 0;
const VAR_STRING: c_int = 2;
const VAR_DICT: c_int = 5;

/// Fill a GArray with information about an assert error.
///
/// This mirrors the C `fill_assert_error` function.
fn fill_assert_error(
    gap: *mut GArray,
    opt_msg_tv: TypevalHandle,
    exp_str: *const c_char,
    exp_tv: TypevalHandle,
    got_tv: TypevalHandle,
    atype: AssertType,
) {
    unsafe {
        let mut omitted: c_int = 0;

        // Add optional message prefix
        let opt_type = nvim_testing_tv_get_type(opt_msg_tv);
        if opt_type != VAR_UNKNOWN
            && !(opt_type == VAR_STRING && nvim_testing_tv_string_is_empty(opt_msg_tv) != 0)
        {
            let tofree = encode_tv2echo(opt_msg_tv, std::ptr::null_mut());
            if !tofree.is_null() {
                ga_concat(gap, tofree);
                xfree(tofree.cast());
            }
            ga_concat(gap, c": ".as_ptr());
        }

        // Add "Expected" prefix based on assert type
        match atype {
            AssertType::Match | AssertType::NotMatch => {
                ga_concat(gap, c"Pattern ".as_ptr());
            }
            AssertType::NotEqual => {
                ga_concat(gap, c"Expected not equal to ".as_ptr());
            }
            _ => {
                ga_concat(gap, c"Expected ".as_ptr());
            }
        }

        // Add expected value
        if exp_str.is_null() {
            // Check if both are dicts for diffing
            let exp_type = nvim_testing_tv_get_type(exp_tv);
            let got_type = nvim_testing_tv_get_type(got_tv);

            if atype != AssertType::NotEqual && exp_type == VAR_DICT && got_type == VAR_DICT {
                // Use C helper for dictionary diffing
                nvim_testing_fill_dict_diff(gap, exp_tv, got_tv, &raw mut omitted);
            } else {
                let tofree = encode_tv2string(exp_tv, std::ptr::null_mut());
                ga_concat_shorten_esc(gap, tofree);
                if !tofree.is_null() {
                    xfree(tofree.cast());
                }
            }
        } else {
            if atype == AssertType::Fails {
                ga_concat(gap, c"'".as_ptr());
            }
            ga_concat_shorten_esc(gap, exp_str);
            if atype == AssertType::Fails {
                ga_concat(gap, c"'".as_ptr());
            }
        }

        // Add "but got" and actual value
        if atype != AssertType::NotEqual {
            match atype {
                AssertType::Match => {
                    ga_concat(gap, c" does not match ".as_ptr());
                }
                AssertType::NotMatch => {
                    ga_concat(gap, c" does match ".as_ptr());
                }
                _ => {
                    ga_concat(gap, c" but got ".as_ptr());
                }
            }

            let tofree = encode_tv2string(got_tv, std::ptr::null_mut());
            ga_concat_shorten_esc(gap, tofree);
            if !tofree.is_null() {
                xfree(tofree.cast());
            }

            if omitted != 0 {
                // Format " - N equal item(s) omitted"
                let mut buf = [0u8; 64];
                let prefix = b" - ";
                buf[..prefix.len()].copy_from_slice(prefix);
                let mut pos = prefix.len();

                // Format the number
                let mut num_buf = [0u8; 16];
                format_int(&mut num_buf, omitted);
                let num_len = num_buf.iter().position(|&c| c == 0).unwrap_or(0);
                buf[pos..pos + num_len].copy_from_slice(&num_buf[..num_len]);
                pos += num_len;

                let suffix: &[u8] = if omitted == 1 {
                    b" equal item omitted"
                } else {
                    b" equal items omitted"
                };
                buf[pos..pos + suffix.len()].copy_from_slice(suffix);
                pos += suffix.len();
                buf[pos] = 0;

                ga_concat(gap, buf.as_ptr().cast());
            }
        }
    }
}

// =============================================================================
// Simple assertion helpers
// =============================================================================

extern "C" {
    // Typval comparison and value extraction
    fn tv_equal(tv1: TypevalHandle, tv2: TypevalHandle, ic: bool) -> bool;
    fn tv_get_number_chk(tv: TypevalHandle, err: *mut c_int) -> i64;
    fn nvim_testing_tv_get_bool_value(tv: TypevalHandle) -> c_int;

    // String extraction
    fn tv_get_string_buf_chk(tv: TypevalHandle, buf: *mut c_char) -> *const c_char;
    fn tv_get_string_chk(tv: TypevalHandle) -> *const c_char;

    // Pattern matching
    fn pattern_match(pat: *const c_char, text: *const c_char, ic: c_int) -> c_int;

    // Global state for beep assertions
    fn nvim_testing_get_called_vim_beep() -> c_int;
    fn nvim_testing_set_called_vim_beep(val: c_int);
    fn nvim_testing_get_suppress_errthrow() -> c_int;
    fn nvim_testing_set_suppress_errthrow(val: c_int);
    fn nvim_testing_get_emsg_silent() -> c_int;
    fn nvim_testing_set_emsg_silent(val: c_int);
    fn do_cmdline_cmd(cmd: *const c_char) -> c_int;

    // Vim variable access
    fn get_vim_var_str(idx: c_int) -> *const c_char;
    fn get_vim_var_tv(idx: c_int) -> TypevalHandle;

    // Float type checking and value extraction
    fn nvim_testing_tv_is_float(tv: TypevalHandle) -> c_int;
    fn tv_get_float(tv: TypevalHandle) -> f64;

    // Type checking functions
    fn tv_check_for_float_or_nr_arg(argvars: TypevalHandle, idx: c_int) -> c_int;
    fn tv_check_for_opt_string_arg(argvars: TypevalHandle, idx: c_int) -> c_int;

    // Garbage collection
    fn get_vim_var_nr(idx: c_int) -> i64;
    fn garbage_collect(testing: c_int);
    fn emsg(s: *const c_char) -> c_int;

    // Gettext translation
    fn nvim_testing_gettext(s: *const c_char) -> *const c_char;
}

// BoolVarValue constants
const BOOL_VAR_FALSE: c_int = 0;
const BOOL_VAR_TRUE: c_int = 1;
const VAR_NUMBER: c_int = 1;
const VAR_BOOL: c_int = 7;

/// Common implementation for assert_true() and assert_false().
fn assert_bool(argvars: TypevalHandle, is_true: bool) -> c_int {
    unsafe {
        let mut error: c_int = 0;
        let arg_type = nvim_testing_tv_get_type(argvars);

        // Check if the assertion passes
        let passes = if arg_type == VAR_NUMBER {
            let num = tv_get_number_chk(argvars, &raw mut error);
            error == 0 && ((num != 0) == is_true)
        } else if arg_type == VAR_BOOL {
            let bool_val = nvim_testing_tv_get_bool_value(argvars);
            bool_val
                == (if is_true {
                    BOOL_VAR_TRUE
                } else {
                    BOOL_VAR_FALSE
                })
        } else {
            false
        };

        if !passes {
            let mut ga = GArray::default();
            prepare_assert_error(&raw mut ga);

            let exp_str = if is_true {
                c"True".as_ptr()
            } else {
                c"False".as_ptr()
            };

            // Get second argument (optional message)
            let argvars_1 = argvars.cast::<u8>().add(TYPVAL_SIZE).cast::<c_void>();
            fill_assert_error(
                &raw mut ga,
                argvars_1,
                exp_str,
                std::ptr::null(),
                argvars,
                AssertType::Other,
            );

            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            return 1;
        }

        0
    }
}

/// Common implementation for assert_equal() and assert_notequal().
fn assert_equal_common(argvars: TypevalHandle, atype: AssertType) -> c_int {
    unsafe {
        // Get the two values to compare
        let arg0 = argvars;
        let arg1 = argvars.cast::<u8>().add(TYPVAL_SIZE).cast::<c_void>();

        let equal = tv_equal(arg0, arg1, false);
        let should_be_equal = atype == AssertType::Equal;

        if equal != should_be_equal {
            let mut ga = GArray::default();
            prepare_assert_error(&raw mut ga);

            // Get third argument (optional message)
            let argvars_2 = argvars.cast::<u8>().add(TYPVAL_SIZE * 2).cast::<c_void>();
            fill_assert_error(&raw mut ga, argvars_2, std::ptr::null(), arg0, arg1, atype);

            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            return 1;
        }

        0
    }
}

// Size of typval_T structure (we need this for pointer arithmetic)
// Must match the C sizeof(typval_T). Guarded by _Static_assert in testing.c.
const TYPVAL_SIZE: usize = 16;

// Size of number buffer for string conversions
const NUMBUFLEN: usize = 65;

// Vim variable indices
const VV_EXCEPTION: c_int = 30; // v:exception
const VV_TESTING: c_int = 76; // v:testing

/// Implementation for assert_beeps() and assert_nobeep().
fn assert_beeps(argvars: TypevalHandle, no_beep: bool) -> c_int {
    unsafe {
        let cmd = tv_get_string_chk(argvars);
        if cmd.is_null() {
            return 0;
        }

        // Save and set global state
        nvim_testing_set_called_vim_beep(0);
        nvim_testing_set_suppress_errthrow(1);
        nvim_testing_set_emsg_silent(0);

        // Execute the command
        do_cmdline_cmd(cmd);

        let called_beep = nvim_testing_get_called_vim_beep() != 0;

        // Restore state
        nvim_testing_set_suppress_errthrow(0);

        // Check result
        let failed = if no_beep { called_beep } else { !called_beep };

        if failed {
            let mut ga = GArray::default();
            prepare_assert_error(&raw mut ga);

            if no_beep {
                ga_concat(&raw mut ga, c"command did beep: ".as_ptr());
            } else {
                ga_concat(&raw mut ga, c"command did not beep: ".as_ptr());
            }
            ga_concat(&raw mut ga, cmd);

            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            return 1;
        }

        0
    }
}

/// Implementation for assert_exception().
fn assert_exception_impl(argvars: TypevalHandle, rettv: TypevalHandleMut) {
    unsafe {
        let error = tv_get_string_chk(argvars);
        let exception = get_vim_var_str(VV_EXCEPTION);

        if exception.is_null() || *exception == 0 {
            // v:exception is not set
            let mut ga = GArray::default();
            prepare_assert_error(&raw mut ga);
            ga_concat(&raw mut ga, c"v:exception is not set".as_ptr());
            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            set_rettv_number(rettv, 1);
        } else if !error.is_null() {
            // Check if v:exception contains the expected error string
            let exception_str = std::ffi::CStr::from_ptr(exception);
            let error_str = std::ffi::CStr::from_ptr(error);

            let contains = exception_str
                .to_bytes()
                .windows(error_str.to_bytes().len())
                .any(|window| window == error_str.to_bytes());

            if !contains {
                let mut ga = GArray::default();
                prepare_assert_error(&raw mut ga);

                let argvars_1 = argvars.cast::<u8>().add(TYPVAL_SIZE).cast::<c_void>();
                let exception_tv = get_vim_var_tv(VV_EXCEPTION);
                fill_assert_error(
                    &raw mut ga,
                    argvars_1,
                    std::ptr::null(),
                    argvars,
                    exception_tv,
                    AssertType::Other,
                );

                assert_error(&raw mut ga);
                ga_clear(&raw mut ga);
                set_rettv_number(rettv, 1);
            }
        }
    }
}

/// Implementation for assert_inrange().
fn assert_inrange_impl(argvars: TypevalHandle) -> c_int {
    unsafe {
        let arg0 = argvars;
        let arg1 = argvars.cast::<u8>().add(TYPVAL_SIZE).cast::<c_void>();
        let arg2 = argvars.cast::<u8>().add(TYPVAL_SIZE * 2).cast::<c_void>();
        let arg3 = argvars.cast::<u8>().add(TYPVAL_SIZE * 3).cast::<c_void>();

        // Check if any argument is float
        let is_float = nvim_testing_tv_is_float(arg0) != 0
            || nvim_testing_tv_is_float(arg1) != 0
            || nvim_testing_tv_is_float(arg2) != 0;

        if is_float {
            let lower = tv_get_float(arg0);
            let upper = tv_get_float(arg1);
            let actual = tv_get_float(arg2);

            if actual < lower || actual > upper {
                let mut ga = GArray::default();
                prepare_assert_error(&raw mut ga);

                // Format "range <lower> - <upper>,"
                let mut expected_str = [0u8; 200];
                let msg = format_range_float(&mut expected_str, lower, upper);
                fill_assert_error(
                    &raw mut ga,
                    arg3,
                    msg.as_ptr().cast(),
                    std::ptr::null(),
                    arg2,
                    AssertType::Other,
                );

                assert_error(&raw mut ga);
                ga_clear(&raw mut ga);
                return 1;
            }
        } else {
            let mut error: c_int = 0;
            let lower = tv_get_number_chk(arg0, &raw mut error);
            if error != 0 {
                return 0;
            }
            let upper = tv_get_number_chk(arg1, &raw mut error);
            if error != 0 {
                return 0;
            }
            let actual = tv_get_number_chk(arg2, &raw mut error);
            if error != 0 {
                return 0;
            }

            if actual < lower || actual > upper {
                let mut ga = GArray::default();
                prepare_assert_error(&raw mut ga);

                // Format "range <lower> - <upper>,"
                let mut expected_str = [0u8; 200];
                let msg = format_range_int(&mut expected_str, lower, upper);
                fill_assert_error(
                    &raw mut ga,
                    arg3,
                    msg.as_ptr().cast(),
                    std::ptr::null(),
                    arg2,
                    AssertType::Other,
                );

                assert_error(&raw mut ga);
                ga_clear(&raw mut ga);
                return 1;
            }
        }

        0
    }
}

/// Format "range <lower> - <upper>," for integer values.
fn format_range_int(buf: &mut [u8; 200], lower: i64, upper: i64) -> &[u8] {
    let prefix = b"range ";
    buf[..prefix.len()].copy_from_slice(prefix);
    let mut pos = prefix.len();

    // Format lower
    let mut num_buf = [0u8; 24];
    let len = format_i64(&mut num_buf, lower);
    buf[pos..pos + len].copy_from_slice(&num_buf[..len]);
    pos += len;

    // " - "
    buf[pos..pos + 3].copy_from_slice(b" - ");
    pos += 3;

    // Format upper
    let len = format_i64(&mut num_buf, upper);
    buf[pos..pos + len].copy_from_slice(&num_buf[..len]);
    pos += len;

    // ","
    buf[pos] = b',';
    pos += 1;
    buf[pos] = 0;

    &buf[..=pos]
}

/// Format "range <lower> - <upper>," for float values.
fn format_range_float(buf: &mut [u8; 200], lower: f64, upper: f64) -> &[u8] {
    // Use a simple approach - delegate to C's snprintf
    unsafe {
        nvim_testing_format_range_float(buf.as_mut_ptr().cast(), 200, lower, upper);
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(0);
    &buf[..=len]
}

extern "C" {
    fn nvim_testing_format_range_float(buf: *mut c_char, size: usize, lower: f64, upper: f64);
}

/// Format an i64 into a buffer. Returns the length written (excluding NUL).
fn format_i64(buf: &mut [u8; 24], value: i64) -> usize {
    let mut num = value;
    let mut digits = [0u8; 20];
    let mut digit_count = 0;
    let negative = num < 0;

    if num == 0 {
        digit_count = 1;
        digits[0] = b'0';
    } else {
        if negative {
            num = -num;
        }

        while num > 0 {
            digits[digit_count] = b'0' + (num % 10) as u8;
            digit_count += 1;
            num /= 10;
        }
    }

    // Copy digits in reverse order
    let mut pos = 0;
    if negative {
        buf[pos] = b'-';
        pos += 1;
    }
    for i in (0..digit_count).rev() {
        buf[pos] = digits[i];
        pos += 1;
    }
    buf[pos] = 0;

    pos
}

/// Common implementation for assert_match() and assert_notmatch().
fn assert_match_common(argvars: TypevalHandle, atype: AssertType) -> c_int {
    unsafe {
        let mut buf1 = [0i8; NUMBUFLEN];
        let mut buf2 = [0i8; NUMBUFLEN];

        let pat = tv_get_string_buf_chk(argvars, buf1.as_mut_ptr());
        let arg1 = argvars.cast::<u8>().add(TYPVAL_SIZE).cast::<c_void>();
        let text = tv_get_string_buf_chk(arg1, buf2.as_mut_ptr());

        if pat.is_null() || text.is_null() {
            return 0;
        }

        let matches = pattern_match(pat, text, 0) != 0;
        let should_match = atype == AssertType::Match;

        if matches != should_match {
            let mut ga = GArray::default();
            prepare_assert_error(&raw mut ga);

            // Get third argument (optional message)
            let argvars_2 = argvars.cast::<u8>().add(TYPVAL_SIZE * 2).cast::<c_void>();
            fill_assert_error(
                &raw mut ga,
                argvars_2,
                std::ptr::null(),
                argvars,
                arg1,
                atype,
            );

            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            return 1;
        }

        0
    }
}

// =============================================================================
// Assert equalfile implementation
// =============================================================================

extern "C" {
    fn nvim_testing_format_notread(buf: *mut c_char, size: usize, fname: *const c_char);
    fn xstrdup(s: *const c_char) -> *mut c_char;
}

/// Compare two files byte-by-byte, returning 0 on match or 1 on mismatch.
///
/// On failure, prepares an assert error message with the first difference
/// including byte offset, line number, and context.
///
/// This mirrors the C `assert_equalfile` function, using `std::fs::File`
/// for file I/O instead of Neovim's `os_fopen`.
fn assert_equalfile(argvars: TypevalHandle) -> c_int {
    unsafe {
        let mut buf1 = [0i8; NUMBUFLEN];
        let mut buf2 = [0i8; NUMBUFLEN];

        let fname1 = tv_get_string_buf_chk(argvars, buf1.as_mut_ptr());
        let argvars_1 = argvars.cast::<u8>().add(TYPVAL_SIZE).cast::<c_void>();
        let fname2 = tv_get_string_buf_chk(argvars_1, buf2.as_mut_ptr());

        if fname1.is_null() || fname2.is_null() {
            return 0;
        }

        let fname1_str = CStr::from_ptr(fname1);
        let fname2_str = CStr::from_ptr(fname2);

        // Result buffer: error message (if any)
        let mut errmsg = [0u8; 1024];
        let mut line1 = [0u8; 200];
        let mut line2 = [0u8; 200];
        let mut lineidx: usize = 0;

        // Try to open both files
        let f1_path = std::str::from_utf8_unchecked(fname1_str.to_bytes());

        if let Ok(f1_file) = std::fs::File::open(f1_path) {
            let f2_path = std::str::from_utf8_unchecked(fname2_str.to_bytes());

            if let Ok(f2_file) = std::fs::File::open(f2_path) {
                let mut f1 = std::io::BufReader::new(f1_file);
                let mut f2 = std::io::BufReader::new(f2_file);
                let mut linecount: i64 = 1;
                let mut count: i64 = 0;

                let mut b1 = [0u8; 1];
                let mut b2 = [0u8; 1];

                loop {
                    let r1 = f1.read(&mut b1).unwrap_or(0);
                    let r2 = f2.read(&mut b2).unwrap_or(0);

                    if r1 == 0 {
                        // EOF on file 1
                        if r2 != 0 {
                            let msg = b"first file is shorter\0";
                            errmsg[..msg.len()].copy_from_slice(msg);
                        }
                        break;
                    } else if r2 == 0 {
                        // EOF on file 2
                        let msg = b"second file is shorter\0";
                        errmsg[..msg.len()].copy_from_slice(msg);
                        break;
                    }

                    let c1 = b1[0];
                    let c2 = b2[0];
                    line1[lineidx] = c1;
                    line2[lineidx] = c2;
                    lineidx += 1;

                    if c1 != c2 {
                        // Format "difference at byte N, line M"
                        let s = format!("difference at byte {count}, line {linecount}\0");
                        let bytes = s.as_bytes();
                        let copy_len = bytes.len().min(errmsg.len());
                        errmsg[..copy_len].copy_from_slice(&bytes[..copy_len]);
                        break;
                    }

                    if c1 == b'\n' {
                        linecount += 1;
                        lineidx = 0;
                    } else if lineidx + 2 == line1.len() {
                        // Shift context buffer
                        line1.copy_within(100..lineidx, 0);
                        line2.copy_within(100..lineidx, 0);
                        lineidx -= 100;
                    }

                    count += 1;
                }
            } else {
                nvim_testing_format_notread(errmsg.as_mut_ptr().cast(), errmsg.len(), fname2);
            }
        } else {
            nvim_testing_format_notread(errmsg.as_mut_ptr().cast(), errmsg.len(), fname1);
        }

        // Check if we have an error message
        if errmsg[0] != 0 {
            let mut ga = GArray::default();
            prepare_assert_error(&raw mut ga);

            // Add optional message from argvars[2]
            let argvars_2 = argvars.cast::<u8>().add(TYPVAL_SIZE * 2).cast::<c_void>();
            let type_2 = nvim_testing_tv_get_type(argvars_2);
            if type_2 != VAR_UNKNOWN {
                let tofree = encode_tv2echo(argvars_2, std::ptr::null_mut());
                if !tofree.is_null() {
                    ga_concat(&raw mut ga, tofree);
                    xfree(tofree.cast());
                }
                ga_concat(&raw mut ga, c": ".as_ptr());
            }

            ga_concat(&raw mut ga, errmsg.as_ptr().cast());

            if lineidx > 0 {
                line1[lineidx] = 0;
                line2[lineidx] = 0;
                ga_concat(&raw mut ga, c" after \"".as_ptr());
                ga_concat(&raw mut ga, line1.as_ptr().cast());
                // Compare the context buffers
                if line1[..lineidx] != line2[..lineidx] {
                    ga_concat(&raw mut ga, c"\" vs \"".as_ptr());
                    ga_concat(&raw mut ga, line2.as_ptr().cast());
                }
                ga_concat(&raw mut ga, c"\"".as_ptr());
            }

            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            return 1;
        }

        0
    }
}

// =============================================================================
// Assert helper: append command or argument
// =============================================================================

/// Append the user-provided message (argvars[2]) or the command string to the GArray.
///
/// If argvars[1] and argvars[2] are both set, encode argvars[2] as the message.
/// Otherwise, append the command string.
///
/// This mirrors the C `assert_append_cmd_or_arg` function.
fn assert_append_cmd_or_arg(gap: *mut GArray, argvars: TypevalHandle, cmd: *const c_char) {
    unsafe {
        let argvars_1 = argvars.cast::<u8>().add(TYPVAL_SIZE).cast::<c_void>();
        let argvars_2 = argvars.cast::<u8>().add(TYPVAL_SIZE * 2).cast::<c_void>();
        let type_1 = nvim_testing_tv_get_type(argvars_1);
        let type_2 = nvim_testing_tv_get_type(argvars_2);

        if type_1 != VAR_UNKNOWN && type_2 != VAR_UNKNOWN {
            let tofree = encode_tv2echo(argvars_2, std::ptr::null_mut());
            if !tofree.is_null() {
                ga_concat(gap, tofree);
                xfree(tofree.cast());
            }
        } else {
            ga_concat(gap, cmd);
        }
    }
}

// =============================================================================
// Assert fails implementation
// =============================================================================

/// VAR_LIST constant
const VAR_LIST: c_int = 4;

/// VV_ERRMSG constant
const VV_ERRMSG: c_int = 3;

/// FAIL constant
const FAIL: c_int = 0;

extern "C" {
    // Type checking functions
    fn tv_check_for_string_or_number_arg(argvars: TypevalHandle, idx: c_int) -> c_int;
    fn tv_check_for_opt_string_or_list_arg(argvars: TypevalHandle, idx: c_int) -> c_int;
    fn tv_check_for_opt_number_arg(argvars: TypevalHandle, idx: c_int) -> c_int;

    // Global state accessors
    fn nvim_testing_get_trylevel() -> c_int;
    fn nvim_testing_set_trylevel(val: c_int);
    fn nvim_testing_get_called_emsg() -> c_int;
    fn nvim_testing_set_in_assert_fails(val: c_int);
    fn nvim_testing_increment_no_wait_return();
    fn nvim_testing_decrement_no_wait_return();
    fn nvim_testing_set_did_emsg(val: c_int);
    fn nvim_testing_set_got_int(val: c_int);
    fn nvim_testing_set_msg_col(val: c_int);
    fn nvim_testing_set_need_wait_return(val: c_int);
    fn nvim_testing_set_emsg_on_display(val: c_int);
    fn nvim_testing_reset_lines_left();
    fn nvim_testing_call_msg_reset_scroll();

    // Error capture
    fn nvim_testing_take_emsg_assert_fails_msg() -> *mut c_char;
    fn nvim_testing_get_emsg_assert_fails_lnum() -> i64;
    fn nvim_testing_get_emsg_assert_fails_context() -> *mut c_char;
    fn nvim_testing_clear_vim_var_errmsg();

    // List operations
    fn nvim_testing_tv_list_len(tv: TypevalHandle) -> c_int;
    fn nvim_testing_tv_list_first(tv: TypevalHandle) -> *mut c_void;
    fn nvim_testing_tv_list_last(tv: TypevalHandle) -> *mut c_void;
    fn nvim_listitem_get_tv(li: *mut c_void) -> TypevalHandle;

    // Error string accessors
    fn nvim_testing_get_e_assert_fails_second_arg() -> *const c_char;
    fn nvim_testing_get_e_assert_fails_fourth_argument() -> *const c_char;
    fn nvim_testing_get_e_assert_fails_fifth_argument() -> *const c_char;

    // Temp typval creation
    fn nvim_testing_make_number_tv(val: i64) -> TypevalHandle;
    fn nvim_testing_make_string_tv(val: *mut c_char) -> TypevalHandle;

    // Typval field accessors
    fn nvim_testing_tv_get_number(tv: TypevalHandle) -> i64;
    fn nvim_testing_tv_get_vstring(tv: TypevalHandle) -> *const c_char;

    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
}

/// Result of the comparison phase of assert_fails.
struct AssertFailsCheckResult {
    /// If non-null, a wrong-argument error message to emit after cleanup.
    wrong_arg_msg: *const c_char,
    /// Allocated string that needs to be freed.
    tofree: *mut c_char,
    /// Whether to set rettv to 1.
    set_rettv: bool,
}

/// Run the comparison/checking logic for `f_assert_fails`, after the command
/// has been executed and we know it failed (called_emsg > before).
///
/// This function returns early for `goto theend` equivalents.
unsafe fn assert_fails_check(argvars: TypevalHandle, cmd: *const c_char) -> AssertFailsCheckResult {
    let mut result = AssertFailsCheckResult {
        wrong_arg_msg: std::ptr::null(),
        tofree: std::ptr::null_mut(),
        set_rettv: false,
    };

    let argvars_1 = argvars.cast::<u8>().add(TYPVAL_SIZE).cast::<c_void>();

    let type_1 = nvim_testing_tv_get_type(argvars_1);
    if type_1 == VAR_UNKNOWN {
        return result;
    }

    let mut buf = [0i8; NUMBUFLEN];
    let mut expected_str: *const c_char = std::ptr::null();
    let mut error_found = false;
    let mut error_found_index: usize = 1;

    let emsg_msg = nvim_testing_take_emsg_assert_fails_msg();
    let actual = if emsg_msg.is_null() {
        c"[unknown]".as_ptr()
    } else {
        emsg_msg.cast_const()
    };

    if type_1 == VAR_STRING {
        let expected = tv_get_string_buf_chk(argvars_1, buf.as_mut_ptr());
        error_found = expected.is_null() || strstr(actual, expected).is_null();
    } else if type_1 == VAR_LIST {
        let list_len = nvim_testing_tv_list_len(argvars_1);
        if !(1..=2).contains(&list_len) {
            result.wrong_arg_msg = nvim_testing_get_e_assert_fails_second_arg();
            // Re-store emsg_msg so cleanup frees it
            if !emsg_msg.is_null() {
                xfree(emsg_msg.cast());
            }
            return result;
        }

        let first_item = nvim_testing_tv_list_first(argvars_1);
        let first_tv = nvim_listitem_get_tv(first_item);
        let expected = tv_get_string_buf_chk(first_tv, buf.as_mut_ptr());
        if expected.is_null() {
            if !emsg_msg.is_null() {
                xfree(emsg_msg.cast());
            }
            return result;
        }

        if pattern_match(expected, actual, 0) == 0 {
            error_found = true;
            expected_str = expected;
        } else if list_len == 2 {
            // Make a copy, an error in pattern_match() may free it
            let errmsg_str = get_vim_var_str(VV_ERRMSG);
            result.tofree = xstrdup(errmsg_str);
            let actual2 = result.tofree.cast_const();

            let last_item = nvim_testing_tv_list_last(argvars_1);
            let last_tv = nvim_listitem_get_tv(last_item);
            let expected2 = tv_get_string_buf_chk(last_tv, buf.as_mut_ptr());
            if expected2.is_null() {
                if !emsg_msg.is_null() {
                    xfree(emsg_msg.cast());
                }
                return result;
            }
            if pattern_match(expected2, actual2, 0) == 0 {
                error_found = true;
                expected_str = expected2;
            }
        }
    } else {
        result.wrong_arg_msg = nvim_testing_get_e_assert_fails_second_arg();
        if !emsg_msg.is_null() {
            xfree(emsg_msg.cast());
        }
        return result;
    }

    // Check optional arg3 (line number) and arg4 (context)
    let check = assert_fails_check_extra_args(
        argvars,
        error_found,
        error_found_index,
        &mut result,
        emsg_msg,
    );
    if result.wrong_arg_msg.is_null() {
        error_found = check.0;
        error_found_index = check.1;
    } else {
        return result;
    }

    if error_found {
        assert_fails_report_error(
            argvars,
            cmd,
            expected_str,
            error_found_index,
            emsg_msg,
            &mut result,
        );
    }

    if !emsg_msg.is_null() {
        xfree(emsg_msg.cast());
    }
    result
}

/// Check optional arg3 (line number) and arg4 (context) for assert_fails.
/// Returns (error_found, error_found_index). On wrong-arg error, sets result.wrong_arg_msg.
unsafe fn assert_fails_check_extra_args(
    argvars: TypevalHandle,
    mut error_found: bool,
    mut error_found_index: usize,
    result: &mut AssertFailsCheckResult,
    emsg_msg: *mut c_char,
) -> (bool, usize) {
    let argvars_2 = argvars.cast::<u8>().add(TYPVAL_SIZE * 2).cast::<c_void>();
    let argvars_3 = argvars.cast::<u8>().add(TYPVAL_SIZE * 3).cast::<c_void>();
    let argvars_4 = argvars.cast::<u8>().add(TYPVAL_SIZE * 4).cast::<c_void>();

    let type_2 = nvim_testing_tv_get_type(argvars_2);
    let type_3 = nvim_testing_tv_get_type(argvars_3);
    if error_found || type_2 == VAR_UNKNOWN || type_3 == VAR_UNKNOWN {
        return (error_found, error_found_index);
    }

    if nvim_testing_tv_get_type(argvars_3) != VAR_NUMBER {
        result.wrong_arg_msg = nvim_testing_get_e_assert_fails_fourth_argument();
        if !emsg_msg.is_null() {
            xfree(emsg_msg.cast());
        }
        return (error_found, error_found_index);
    }
    let arg3_num = nvim_testing_tv_get_number(argvars_3);
    if arg3_num >= 0 && arg3_num != nvim_testing_get_emsg_assert_fails_lnum() {
        error_found = true;
        error_found_index = 3;
    }

    let type_4 = nvim_testing_tv_get_type(argvars_4);
    if !error_found && type_4 != VAR_UNKNOWN {
        if type_4 != VAR_STRING {
            result.wrong_arg_msg = nvim_testing_get_e_assert_fails_fifth_argument();
            if !emsg_msg.is_null() {
                xfree(emsg_msg.cast());
            }
            return (error_found, error_found_index);
        }
        let arg4_string = nvim_testing_tv_get_vstring(argvars_4);
        if !arg4_string.is_null() {
            let context = nvim_testing_get_emsg_assert_fails_context();
            if pattern_match(arg4_string, context, 0) == 0 {
                error_found = true;
                error_found_index = 4;
            }
        }
    }

    (error_found, error_found_index)
}

/// Report an assert_fails error with the expected/actual values.
unsafe fn assert_fails_report_error(
    argvars: TypevalHandle,
    cmd: *const c_char,
    expected_str: *const c_char,
    error_found_index: usize,
    emsg_msg: *mut c_char,
    result: &mut AssertFailsCheckResult,
) {
    let argvars_2 = argvars.cast::<u8>().add(TYPVAL_SIZE * 2).cast::<c_void>();

    let mut ga = GArray::default();
    prepare_assert_error(&raw mut ga);

    // Build the actual_tv for fill_assert_error
    let actual_tv = if error_found_index == 3 {
        nvim_testing_make_number_tv(nvim_testing_get_emsg_assert_fails_lnum())
    } else if error_found_index == 4 {
        nvim_testing_make_string_tv(nvim_testing_get_emsg_assert_fails_context())
    } else {
        let actual_ptr = if result.tofree.is_null() {
            emsg_msg
        } else {
            result.tofree
        };
        nvim_testing_make_string_tv(actual_ptr)
    };

    let error_argvar = argvars
        .cast::<u8>()
        .add(TYPVAL_SIZE * error_found_index)
        .cast::<c_void>();
    fill_assert_error(
        &raw mut ga,
        argvars_2,
        expected_str,
        error_argvar,
        actual_tv,
        AssertType::Fails,
    );
    ga_concat(&raw mut ga, c": ".as_ptr());
    assert_append_cmd_or_arg(&raw mut ga, argvars, cmd);
    assert_error(&raw mut ga);
    ga_clear(&raw mut ga);
    result.set_rettv = true;
}

/// Cleanup function for assert_fails - always runs, equivalent to `theend:` label.
unsafe fn assert_fails_cleanup(save_trylevel: c_int, tofree: *mut c_char) {
    nvim_testing_set_trylevel(save_trylevel);
    nvim_testing_set_suppress_errthrow(0);
    nvim_testing_set_in_assert_fails(0);
    nvim_testing_set_did_emsg(0);
    nvim_testing_set_got_int(0);
    nvim_testing_set_msg_col(0);
    nvim_testing_decrement_no_wait_return();
    nvim_testing_set_need_wait_return(0);
    nvim_testing_set_emsg_on_display(0);
    nvim_testing_call_msg_reset_scroll();
    nvim_testing_reset_lines_left();
    if !tofree.is_null() {
        xfree(tofree.cast());
    }
    nvim_testing_clear_vim_var_errmsg();
}

/// Full implementation of `assert_fails(cmd [, error [, msg [, lnum [, context]]]])`.
fn assert_fails_impl(argvars: TypevalHandle, rettv: TypevalHandleMut) {
    unsafe {
        // Type checking (returns before state changes — no cleanup needed)
        let argvars_1 = argvars.cast::<u8>().add(TYPVAL_SIZE).cast::<c_void>();
        let argvars_2 = argvars.cast::<u8>().add(TYPVAL_SIZE * 2).cast::<c_void>();
        let argvars_3 = argvars.cast::<u8>().add(TYPVAL_SIZE * 3).cast::<c_void>();

        if tv_check_for_string_or_number_arg(argvars, 0) == FAIL
            || tv_check_for_opt_string_or_list_arg(argvars, 1) == FAIL
            || (nvim_testing_tv_get_type(argvars_1) != VAR_UNKNOWN
                && (nvim_testing_tv_get_type(argvars_2) != VAR_UNKNOWN
                    && (tv_check_for_opt_number_arg(argvars, 3) == FAIL
                        || (nvim_testing_tv_get_type(argvars_3) != VAR_UNKNOWN
                            && tv_check_for_opt_string_arg(argvars, 4) == FAIL))))
        {
            return;
        }

        // Save & set global state
        let save_trylevel = nvim_testing_get_trylevel();
        let called_emsg_before = nvim_testing_get_called_emsg();

        // trylevel must be zero for a ":throw" command to be considered failed
        nvim_testing_set_trylevel(0);
        nvim_testing_set_suppress_errthrow(1);
        nvim_testing_set_in_assert_fails(1);
        nvim_testing_increment_no_wait_return();

        // Execute the command
        let cmd = tv_get_string_chk(argvars);
        do_cmdline_cmd(cmd);

        // Reset trylevel/suppress_errthrow for any errors reported below
        nvim_testing_set_trylevel(save_trylevel);
        nvim_testing_set_suppress_errthrow(0);

        let called_emsg_after = nvim_testing_get_called_emsg();

        if called_emsg_after == called_emsg_before {
            // Command did not fail
            let mut ga = GArray::default();
            prepare_assert_error(&raw mut ga);
            ga_concat(&raw mut ga, c"command did not fail: ".as_ptr());
            assert_append_cmd_or_arg(&raw mut ga, argvars, cmd);
            assert_error(&raw mut ga);
            ga_clear(&raw mut ga);
            set_rettv_number(rettv, 1);

            assert_fails_cleanup(save_trylevel, std::ptr::null_mut());
        } else {
            // Command failed, check the error
            let check_result = assert_fails_check(argvars, cmd);

            if check_result.set_rettv {
                set_rettv_number(rettv, 1);
            }

            assert_fails_cleanup(save_trylevel, check_result.tofree);

            if !check_result.wrong_arg_msg.is_null() {
                emsg(check_result.wrong_arg_msg);
            }
        }
    }
}

// =============================================================================
// VimL function implementations
// =============================================================================

/// `assert_true(actual[, msg])` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_true"]
pub unsafe extern "C" fn rs_f_assert_true(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    set_rettv_number(rettv, i64::from(assert_bool(argvars, true)));
}

/// `assert_false(actual[, msg])` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_false"]
pub unsafe extern "C" fn rs_f_assert_false(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    set_rettv_number(rettv, i64::from(assert_bool(argvars, false)));
}

/// `assert_equalfile(fname-one, fname-two[, msg])` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_equalfile"]
pub unsafe extern "C" fn rs_f_assert_equalfile(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    set_rettv_number(rettv, i64::from(assert_equalfile(argvars)));
}

/// `assert_fails(cmd [, error [, msg [, lnum [, context]]]])` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_fails"]
pub unsafe extern "C" fn rs_f_assert_fails(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    assert_fails_impl(argvars, rettv);
}

/// `assert_equal(expected, actual[, msg])` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_equal"]
pub unsafe extern "C" fn rs_f_assert_equal(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    set_rettv_number(
        rettv,
        i64::from(assert_equal_common(argvars, AssertType::Equal)),
    );
}

/// `assert_notequal(expected, actual[, msg])` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_notequal"]
pub unsafe extern "C" fn rs_f_assert_notequal(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    set_rettv_number(
        rettv,
        i64::from(assert_equal_common(argvars, AssertType::NotEqual)),
    );
}

/// `assert_match(pattern, actual[, msg])` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_match"]
pub unsafe extern "C" fn rs_f_assert_match(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    set_rettv_number(
        rettv,
        i64::from(assert_match_common(argvars, AssertType::Match)),
    );
}

/// `assert_notmatch(pattern, actual[, msg])` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_notmatch"]
pub unsafe extern "C" fn rs_f_assert_notmatch(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    set_rettv_number(
        rettv,
        i64::from(assert_match_common(argvars, AssertType::NotMatch)),
    );
}

/// `assert_beeps(cmd)` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_beeps"]
pub unsafe extern "C" fn rs_f_assert_beeps(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    set_rettv_number(rettv, i64::from(assert_beeps(argvars, false)));
}

/// `assert_nobeep(cmd)` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_nobeep"]
pub unsafe extern "C" fn rs_f_assert_nobeep(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    set_rettv_number(rettv, i64::from(assert_beeps(argvars, true)));
}

/// `assert_exception(string[, msg])` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_exception"]
pub unsafe extern "C" fn rs_f_assert_exception(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    assert_exception_impl(argvars, rettv);
}

/// `assert_inrange(lower, upper, actual[, msg])` function implementation.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_inrange"]
pub unsafe extern "C" fn rs_f_assert_inrange(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    const FAIL: c_int = 0;
    if tv_check_for_float_or_nr_arg(argvars, 0) == FAIL
        || tv_check_for_float_or_nr_arg(argvars, 1) == FAIL
        || tv_check_for_float_or_nr_arg(argvars, 2) == FAIL
        || tv_check_for_opt_string_arg(argvars, 3) == FAIL
    {
        return;
    }
    set_rettv_number(rettv, i64::from(assert_inrange_impl(argvars)));
}

/// `test_garbagecollect_now()` function implementation.
///
/// This is dangerous, any Lists and Dicts used internally may be freed
/// while still in use.
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_test_garbagecollect_now"]
pub unsafe extern "C" fn rs_f_test_garbagecollect_now(
    _argvars: TypevalHandle,
    _rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    if get_vim_var_nr(VV_TESTING) == 0 {
        let msg = nvim_testing_gettext(
            c"E1142: Calling test_garbagecollect_now() while v:testing is not set".as_ptr(),
        );
        emsg(msg);
    } else {
        garbage_collect(1); // true
    }
}

/// `assert_report(msg)` function implementation.
///
/// This is the simplest assert function - it just adds the message to v:errors.
///
/// # Safety
///
/// - `argvars` must point to a valid array of at least 1 `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_assert_report"]
pub unsafe extern "C" fn rs_f_assert_report(
    argvars: TypevalHandle,
    rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    let mut ga = GArray::default();

    prepare_assert_error(&raw mut ga);

    // Get the message string from argvars[0]
    let msg = tv_get_string(argvars);
    if !msg.is_null() {
        ga_concat(&raw mut ga, msg);
    }

    assert_error(&raw mut ga);
    ga_clear(&raw mut ga);

    // Set return value to 1 (failure count)
    set_rettv_number(rettv, 1);
}

/// `test_write_list_log(fname)` function implementation.
///
/// This is a no-op function in Neovim (`list_log` feature is disabled).
///
/// # Safety
///
/// - `argvars` must point to a valid array of `typval_T`.
/// - `rettv` must point to a valid `typval_T` for the return value.
#[export_name = "f_test_write_list_log"]
pub unsafe extern "C" fn rs_f_test_write_list_log(
    _argvars: TypevalHandle,
    _rettv: TypevalHandleMut,
    _fptr: EvalFuncData,
) {
    // This function is a no-op in Neovim
    // The original C code just extracts the filename and does nothing with it
}

// =============================================================================
// Return value helpers
// =============================================================================

extern "C" {
    fn nvim_testing_set_rettv_number(rettv: TypevalHandleMut, val: i64);
}

/// Set the return typval to a number.
#[inline]
fn set_rettv_number(rettv: TypevalHandleMut, val: i64) {
    unsafe {
        nvim_testing_set_rettv_number(rettv, val);
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_line_number() {
        let mut buf = [0u8; 64];
        format_line_number(&mut buf, 42);
        let s = std::str::from_utf8(&buf[..8]).unwrap();
        assert_eq!(s, "line 42\0");
    }

    #[test]
    fn test_format_line_number_zero() {
        let mut buf = [0u8; 64];
        format_line_number(&mut buf, 0);
        let s = std::str::from_utf8(&buf[..7]).unwrap();
        assert_eq!(s, "line 0\0");
    }

    #[test]
    fn test_format_line_number_large() {
        let mut buf = [0u8; 64];
        format_line_number(&mut buf, 12345);
        let s = std::str::from_utf8(&buf[..11]).unwrap();
        assert_eq!(s, "line 12345\0");
    }

    #[test]
    fn test_hex_digit() {
        assert_eq!(hex_digit(0), b'0');
        assert_eq!(hex_digit(9), b'9');
        assert_eq!(hex_digit(10), b'a');
        assert_eq!(hex_digit(15), b'f');
    }

    #[test]
    fn test_format_int() {
        let mut buf = [0u8; 16];
        format_int(&mut buf, 42);
        assert_eq!(&buf[..3], b"42\0");

        format_int(&mut buf, 0);
        assert_eq!(&buf[..2], b"0\0");

        format_int(&mut buf, 12345);
        assert_eq!(&buf[..6], b"12345\0");
    }
}
