//! Core message output functions
//!
//! Provides Rust implementations for the fundamental message output operations:
//! - `msg()` - Display a message with optional highlight
//! - `msg_puts()` - Output a string to the message area
//! - `msg_putchar()` - Output a single character to the message area
//!
//! These functions form the foundation of the message display system.

use std::ffi::{c_char, c_int};

// Use the mbyte crate for UTF-8 encoding
use nvim_mbyte::rs_utf_char2bytes;

// Use msg_outtrans_len from format.rs (same crate)
use crate::rs_msg_outtrans_len;

// ============================================================================
// C Function Declarations
// ============================================================================

/// C-compatible `String` struct (`{char *data, size_t size}`).
///
/// This mirrors the C API `String` typedef used in message functions.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NvimString {
    pub data: *mut c_char,
    pub size: usize,
}

extern "C" {
    // Core message output functions (call into C until fully migrated)
    fn msg_keep(s: *const c_char, hl_id: c_int, keep: c_int, multiline: c_int) -> c_int;
    fn msg_puts_len(s: *const c_char, len: isize, hl_id: c_int, hist: bool);
    fn msg_start();
    fn msg_clr_eos_force();
    fn msg_ext_ui_flush();
    // got_int accessor
    fn nvim_get_got_int() -> c_int;

    // For msg_end
    fn nvim_get_exiting() -> c_int;
    fn nvim_get_need_wait_return() -> c_int;
    fn nvim_get_state() -> c_int;
    fn nvim_wait_return(redraw: bool);

    // Verbose enter/leave
    fn verbose_enter();
    fn verbose_leave();

    // State accessors
    fn nvim_get_msg_silent() -> c_int;
    fn nvim_get_msg_col() -> c_int;
    fn nvim_set_msg_col(col: c_int);
    fn nvim_get_columns() -> c_int;
    fn nvim_get_cmdline_row() -> c_int;
    fn nvim_set_lines_left(val: c_int);
    fn nvim_set_msg_didany(val: c_int);
    fn nvim_set_need_wait_return(val: c_int);
    fn nvim_set_emsg_on_display(val: c_int);
    fn nvim_set_cmdline_row(val: c_int);
    fn nvim_get_msg_row() -> c_int;
}

/// Maximum bytes for a single UTF-8 character (including composing chars)
const MB_MAXCHAR: usize = 6;

/// Special key indicator
const K_SPECIAL: u8 = 0x80;

/// Check if a character is a special key code.
#[inline]
const fn is_special(c: c_int) -> bool {
    c < 0
}

/// Get the second byte of a special key.
#[allow(clippy::cast_sign_loss)]
#[inline]
const fn k_second(c: c_int) -> u8 {
    (((-c - 1) >> 8) & 0xff) as u8
}

/// Get the third byte of a special key.
#[allow(clippy::cast_sign_loss)]
#[inline]
const fn k_third(c: c_int) -> u8 {
    ((-c - 1) & 0xff) as u8
}

// ============================================================================
// Core Message Output Functions
// ============================================================================

/// Display a message to the user.
///
/// This is the primary function for displaying a message string.
/// The message is displayed at the current message position.
///
/// # Arguments
/// * `s` - The message string to display (NUL-terminated)
/// * `hl_id` - Highlight group ID (0 for default)
///
/// # Returns
/// * `true` if wait_return() was not called
/// * `false` if wait_return() was called
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[export_name = "msg"]
#[must_use]
pub unsafe extern "C" fn rs_msg(s: *const c_char, hl_id: c_int) -> c_int {
    msg_keep(s, hl_id, 0, 0)
}

/// Display a message and optionally keep it displayed.
///
/// # Arguments
/// * `s` - The message string to display (NUL-terminated)
/// * `hl_id` - Highlight group ID (0 for default)
/// * `keep` - If true, keep the message displayed (set keep_msg)
///
/// # Returns
/// * `true` if wait_return() was not called
/// * `false` if wait_return() was called
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_keep(s: *const c_char, hl_id: c_int, keep: c_int) -> c_int {
    msg_keep(s, hl_id, keep, 0)
}

/// Display a multiline message.
///
/// # Arguments
/// * `s` - The message string to display (NUL-terminated)
/// * `hl_id` - Highlight group ID (0 for default)
/// * `multiline` - If true, handle embedded newlines specially
///
/// # Returns
/// * `true` if wait_return() was not called
/// * `false` if wait_return() was called
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_multiline_simple(
    s: *const c_char,
    hl_id: c_int,
    multiline: c_int,
) -> c_int {
    msg_keep(s, hl_id, 0, multiline)
}

/// Output a string to the message area.
///
/// Outputs the string at the current msg_row, msg_col position.
/// Does not add the string to message history.
///
/// # Arguments
/// * `s` - The string to output (NUL-terminated)
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[export_name = "msg_puts"]
pub unsafe extern "C" fn rs_msg_puts(s: *const c_char) {
    rs_msg_puts_hl(s, 0, false);
}

/// Output a string with highlight and history option.
///
/// # Arguments
/// * `s` - The string to output (NUL-terminated)
/// * `hl_id` - Highlight group ID (0 for default)
/// * `hist` - If true, add to message history
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[export_name = "msg_puts_hl"]
pub unsafe extern "C" fn rs_msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool) {
    msg_puts_len(s, -1, hl_id, hist);
}

/// Output a single character to the message area.
///
/// Outputs the character at the current msg_row, msg_col position.
/// Handles multi-byte UTF-8 characters and special key codes.
///
/// # Arguments
/// * `c` - The character to output (Unicode code point or special key)
///
/// # Safety
/// This function is safe to call with any integer value.
#[export_name = "msg_putchar"]
pub unsafe extern "C" fn rs_msg_putchar(c: c_int) {
    rs_msg_putchar_hl(c, 0);
}

/// Output a single character with highlight.
///
/// Outputs the character at the current msg_row, msg_col position.
/// Handles multi-byte UTF-8 characters and special key codes.
///
/// # Arguments
/// * `c` - The character to output (Unicode code point or special key)
/// * `hl_id` - Highlight group ID (0 for default)
///
/// # Safety
/// This function is safe to call with any integer value for `c`.
#[export_name = "msg_putchar_hl"]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_msg_putchar_hl(c: c_int, hl_id: c_int) {
    let mut buf: [c_char; MB_MAXCHAR + 1] = [0; MB_MAXCHAR + 1];

    if is_special(c) {
        // Special key code: encode as K_SPECIAL + two bytes
        buf[0] = K_SPECIAL as c_char;
        buf[1] = k_second(c) as c_char;
        buf[2] = k_third(c) as c_char;
        buf[3] = 0; // NUL terminator
    } else {
        // Regular character: encode as UTF-8
        let len = rs_utf_char2bytes(c, buf.as_mut_ptr());
        buf[len as usize] = 0; // NUL terminator
    }

    rs_msg_puts_hl(buf.as_ptr(), hl_id, false);
}

/// Output a number to the message area.
///
/// Converts the number to a string and outputs it.
///
/// # Arguments
/// * `n` - The number to output
///
/// # Safety
/// This function is safe to call with any integer value.
#[export_name = "msg_outnum"]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_msg_outnum(n: c_int) {
    // Format number as string (max 20 chars for i32)
    let mut buf: [c_char; 20] = [0; 20];

    // Use snprintf-like formatting
    let s = format!("{n}");
    let bytes = s.as_bytes();
    let len = bytes.len().min(19);

    for (i, &b) in bytes[..len].iter().enumerate() {
        buf[i] = b as c_char;
    }
    buf[len] = 0;

    rs_msg_puts_hl(buf.as_ptr(), 0, false);
}

// ============================================================================
// Message Control Functions
// ============================================================================

/// Start a new message.
///
/// Prepares the message area for output. This should be called before
/// writing message content.
///
/// # Safety
/// Calls C function that modifies global state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_start() {
    msg_start();
}

/// End a message.
///
/// Finalizes message output and handles wait_return if needed.
///
/// # Returns
/// * `true` (1) if wait_return() was not called
/// * `false` (0) if wait_return() was called
///
/// # Safety
/// Calls C accessor functions that may trigger interactive prompt.
#[export_name = "msg_end"]
#[must_use]
pub unsafe extern "C" fn rs_msg_end() -> c_int {
    // If the string is larger than the window,
    // or the ruler option is set and we run into it,
    // we have to redraw the window.
    // Do not do this if we are abandoning the file or editing the command line.
    const MODE_CMDLINE: c_int = 0x08;
    if nvim_get_exiting() == 0
        && nvim_get_need_wait_return() != 0
        && (nvim_get_state() & MODE_CMDLINE) == 0
    {
        nvim_wait_return(false);
        return 0;
    }

    // NOTE: ui_flush() used to be called here. This had to be removed, as it
    // inhibited substantial performance improvements.
    msg_ext_ui_flush();
    1
}

/// Clear from current message position to end of screen (rs_ alias).
///
/// # Safety
/// Calls Rust implementation.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_clr_eos() {
    msg_clr_eos_impl();
}

/// Internal implementation of msg_clr_eos logic.
#[inline]
unsafe fn msg_clr_eos_impl() {
    if nvim_get_msg_silent() == 0 {
        msg_clr_eos_force();
    }
}

/// Check if messages are silent.
///
/// # Returns
/// * Non-zero if msg_silent > 0 (messages are being suppressed)
/// * Zero if messages are not silent
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_is_silent() -> c_int {
    nvim_get_msg_silent()
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Output a title string (highlighted as HLF_T).
///
/// # Arguments
/// * `s` - The string to output (NUL-terminated)
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[export_name = "msg_puts_title"]
pub unsafe extern "C" fn rs_msg_puts_title(s: *const c_char) {
    rs_msg_puts_hl(s, HLF_T, false);
}

/// Highlight face for title
const HLF_T: c_int = 23; // From highlight_defs.h: HLF_T = 23

/// Highlight face for "8" (truncation indicator)
#[allow(dead_code)]
const HLF_8: c_int = 1; // From highlight_defs.h: HLF_8 = 1

// ============================================================================
// Printf-Style Message Constants
// ============================================================================

/// IOSIZE - Buffer size for sprintf, I/O, etc.
///
/// This matches the C definition in globals.h.
pub const IOSIZE: c_int = 1025;

/// Maximum number of format arguments typically used in messages.
pub const MAX_MSG_ARGS: c_int = 20;

// ============================================================================
// Printf-Style Message Helpers
// ============================================================================

/// Calculate the buffer size needed for a printf-style message.
///
/// Returns IOSIZE as that's the maximum size used for message formatting.
#[no_mangle]
pub const extern "C" fn rs_msg_printf_bufsize() -> c_int {
    IOSIZE
}

/// Check if a buffer size is sufficient for printf-style messages.
///
/// Returns true if size >= IOSIZE.
#[no_mangle]
pub const extern "C" fn rs_msg_bufsize_ok(size: c_int) -> c_int {
    (size >= IOSIZE) as c_int
}

/// Check if character is a printf format specifier start.
///
/// Returns true for '%'.
#[no_mangle]
pub const extern "C" fn rs_is_format_char(c: c_int) -> c_int {
    (c == b'%' as c_int) as c_int
}

/// Check if character is a printf conversion specifier.
///
/// Returns true for d, i, o, u, x, X, e, E, f, F, g, G, a, A, c, s, p, n, %.
#[no_mangle]
pub const extern "C" fn rs_is_printf_spec(c: c_int) -> c_int {
    // Check against character codes directly
    (c == b'd' as c_int
        || c == b'i' as c_int
        || c == b'o' as c_int
        || c == b'u' as c_int
        || c == b'x' as c_int
        || c == b'X' as c_int
        || c == b'e' as c_int
        || c == b'E' as c_int
        || c == b'f' as c_int
        || c == b'F' as c_int
        || c == b'g' as c_int
        || c == b'G' as c_int
        || c == b'a' as c_int
        || c == b'A' as c_int
        || c == b'c' as c_int
        || c == b's' as c_int
        || c == b'p' as c_int
        || c == b'n' as c_int
        || c == b'%' as c_int) as c_int
}

/// Check if character is a printf flag.
///
/// Returns true for -, +, space, #, 0.
#[no_mangle]
pub const extern "C" fn rs_is_printf_flag(c: c_int) -> c_int {
    (c == b'-' as c_int
        || c == b'+' as c_int
        || c == b' ' as c_int
        || c == b'#' as c_int
        || c == b'0' as c_int) as c_int
}

/// Check if character could be part of a printf format field width or precision.
///
/// Returns true for digits and '.'.
#[no_mangle]
pub const extern "C" fn rs_is_printf_width(c: c_int) -> c_int {
    ((c >= b'0' as c_int && c <= b'9' as c_int) || c == b'.' as c_int) as c_int
}

/// Check if character is a printf length modifier.
///
/// Returns true for h, l, L, z, j, t.
#[no_mangle]
pub const extern "C" fn rs_is_printf_length(c: c_int) -> c_int {
    (c == b'h' as c_int
        || c == b'l' as c_int
        || c == b'L' as c_int
        || c == b'z' as c_int
        || c == b'j' as c_int
        || c == b't' as c_int) as c_int
}

// ============================================================================
// Message Control Flow (Phase 1 Migration)
// ============================================================================

/// Advance msg cursor to column "col".
///
/// If msg_silent is set, just update msg_col (for redirection).
/// Otherwise pad with spaces until reaching the column.
///
/// # Safety
/// Calls C accessor functions.
#[export_name = "msg_advance"]
pub unsafe extern "C" fn rs_msg_advance(col: c_int) {
    if nvim_get_msg_silent() != 0 {
        // nothing to advance to (for redirection, may fill it up later)
        nvim_set_msg_col(col);
        return;
    }
    // not enough room - clamp to Columns - 1
    let columns = nvim_get_columns();
    let col = if col > columns - 1 { columns - 1 } else { col };
    while nvim_get_msg_col() < col {
        rs_msg_putchar(c_int::from(b' '));
    }
}

/// Like msg() but keep it silent when 'verbosefile' is set.
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[export_name = "verb_msg"]
#[must_use]
pub unsafe extern "C" fn rs_verb_msg(s: *const c_char) -> c_int {
    verbose_enter();
    let n = msg_keep(s, 0, 0, 0);
    verbose_leave();
    n
}

/// Start collecting messages here.
///
/// Sets lines_left to cmdline_row and clears msg_didany.
///
/// # Safety
/// Calls C accessor functions that modify global state.
#[export_name = "msg_starthere"]
pub unsafe extern "C" fn rs_msg_starthere() {
    let cmdline_row = nvim_get_cmdline_row();
    nvim_set_lines_left(cmdline_row);
    nvim_set_msg_didany(0);
}

/// Clear from current message position to end of screen.
///
/// Only clears if msg_silent is not set.
///
/// # Safety
/// Calls C functions that modify display state.
#[export_name = "msg_clr_eos"]
pub unsafe extern "C" fn rs_msg_clr_eos_export() {
    msg_clr_eos_impl();
}

/// End a prompt message.
///
/// Resets the prompt state: clears need_wait_return, emsg_on_display,
/// updates cmdline_row, resets msg_col, clears eos, resets lines_left.
///
/// # Safety
/// Calls C functions that modify global state and display.
#[export_name = "msg_end_prompt"]
pub unsafe extern "C" fn rs_msg_end_prompt() {
    nvim_set_need_wait_return(0);
    nvim_set_emsg_on_display(0);
    nvim_set_cmdline_row(nvim_get_msg_row());
    nvim_set_msg_col(0);
    msg_clr_eos_impl();
    nvim_set_lines_left(-1);
}

// ============================================================================
// Multiline Message Output (Phase 84)
// ============================================================================

/// Output a string with newline/tab/CR handling.
///
/// Similar to `msg_outtrans_len`, but handles newlines, tabs, and carriage
/// returns specially: flushes the current chunk, optionally clears EOS,
/// and outputs the delimiter character.
///
/// # Arguments
/// * `str` - The string to output (length-delimited, not NUL-terminated)
/// * `hl_id` - Highlight group ID
/// * `check_int` - If true, stop early when `got_int` is set
/// * `hist` - If true, add to message history
/// * `need_clear` - In/out flag: true if EOS needs clearing before next newline
///
/// # Safety
/// - `str.data` must be valid for `str.size` bytes
/// - `need_clear` must be a valid non-null pointer
#[export_name = "msg_multiline"]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_msg_multiline(
    str: NvimString,
    hl_id: c_int,
    check_int: bool,
    hist: bool,
    need_clear: *mut bool,
) {
    let base = str.data;
    let mut s = base;
    let mut chunk = base;
    let end = base.add(str.size);

    while s < end {
        if check_int && nvim_get_got_int() != 0 {
            return;
        }
        let c = *s as u8;
        if c == b'\n' || c == b'\t' || c == b'\r' {
            // Flush chars before this delimiter
            let chunk_len = (s as usize - chunk as usize) as c_int;
            let _ = rs_msg_outtrans_len(chunk, chunk_len, hl_id, hist);

            if c != b'\t' && *need_clear {
                msg_clr_eos_impl();
                *need_clear = false;
            }
            rs_msg_putchar_hl(c_int::from(c), hl_id);
            chunk = s.add(1);
        }
        s = s.add(1);
    }

    // Print the remaining tail
    let tail_len = (s as usize - chunk as usize) as c_int;
    let _ = rs_msg_outtrans_len(chunk, tail_len, hl_id, hist);
}

// ============================================================================
// Unit Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_special() {
        // Special keys are negative
        assert!(is_special(-1));
        assert!(is_special(-100));
        assert!(!is_special(0));
        assert!(!is_special(65)); // 'A'
    }

    #[test]
    fn test_k_second_third() {
        // Test special key byte extraction
        // For a special key c = -(1 + (second << 8) + third)
        let second: u8 = 0x12;
        let third: u8 = 0x34;
        let c = -1 - (c_int::from(second) << 8) - c_int::from(third);

        assert_eq!(k_second(c), second);
        assert_eq!(k_third(c), third);
    }

    #[test]
    fn test_iosize() {
        assert_eq!(IOSIZE, 1025);
        assert_eq!(rs_msg_printf_bufsize(), 1025);
    }

    #[test]
    fn test_bufsize_ok() {
        assert_eq!(rs_msg_bufsize_ok(1025), 1);
        assert_eq!(rs_msg_bufsize_ok(2000), 1);
        assert_eq!(rs_msg_bufsize_ok(1024), 0);
        assert_eq!(rs_msg_bufsize_ok(0), 0);
    }

    #[test]
    fn test_printf_helpers() {
        // Format char
        assert_eq!(rs_is_format_char(c_int::from(b'%')), 1);
        assert_eq!(rs_is_format_char(c_int::from(b'd')), 0);

        // Printf spec
        assert_eq!(rs_is_printf_spec(c_int::from(b'd')), 1);
        assert_eq!(rs_is_printf_spec(c_int::from(b's')), 1);
        assert_eq!(rs_is_printf_spec(c_int::from(b'X')), 1);
        assert_eq!(rs_is_printf_spec(c_int::from(b'a')), 1); // hexfloat
        assert_eq!(rs_is_printf_spec(c_int::from(b'z')), 0); // not a conversion spec

        // Printf flag
        assert_eq!(rs_is_printf_flag(c_int::from(b'-')), 1);
        assert_eq!(rs_is_printf_flag(c_int::from(b'+')), 1);
        assert_eq!(rs_is_printf_flag(c_int::from(b'd')), 0);

        // Printf width
        assert_eq!(rs_is_printf_width(c_int::from(b'0')), 1);
        assert_eq!(rs_is_printf_width(c_int::from(b'9')), 1);
        assert_eq!(rs_is_printf_width(c_int::from(b'.')), 1);
        assert_eq!(rs_is_printf_width(c_int::from(b'd')), 0);

        // Printf length
        assert_eq!(rs_is_printf_length(c_int::from(b'l')), 1);
        assert_eq!(rs_is_printf_length(c_int::from(b'h')), 1);
        assert_eq!(rs_is_printf_length(c_int::from(b'd')), 0);
    }
}
