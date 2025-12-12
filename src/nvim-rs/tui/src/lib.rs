//! Terminal UI utilities for Neovim
//!
//! This crate provides terminfo-related functions for the terminal UI,
//! including key modifier handling and terminfo format string processing.

use std::ffi::{c_char, c_int, c_long, CStr};
use std::io::Write;

// ============================================================================
// Key Modifier Constants (matching termkey_defs.h and input.c)
// ============================================================================

/// Shift modifier (from libtermkey)
const TERMKEY_KEYMOD_SHIFT: c_int = 1 << 0;
/// Alt modifier (from libtermkey)
const TERMKEY_KEYMOD_ALT: c_int = 1 << 1;
/// Ctrl modifier (from libtermkey)
const TERMKEY_KEYMOD_CTRL: c_int = 1 << 2;
/// Super modifier (D- in Nvim, not from libtermkey)
const KEYMOD_SUPER: c_int = 1 << 3;
/// Meta modifier (T- in Nvim, not from libtermkey)
const KEYMOD_META: c_int = 1 << 5;

// ============================================================================
// Key Modifier Functions
// ============================================================================

/// Handle TERMKEY_KEYMOD_* modifiers (Shift, Alt, Ctrl).
///
/// Writes modifier prefix strings ("S-", "A-", "C-") to the buffer based on
/// the modifier flags. Used to build key notation like "<C-A-x>".
///
/// # Safety
///
/// - `buf` must point to a valid buffer of at least `buflen` bytes
/// - The caller must ensure the buffer has enough space for the modifiers
///   (up to 6 bytes for "S-A-C-")
///
/// # Arguments
/// * `modifiers` - Modifier flags (TERMKEY_KEYMOD_*)
/// * `buf` - Buffer to write to
/// * `buflen` - Size of buffer
///
/// # Returns
/// Number of bytes written to buffer, excluding any NUL terminator
#[no_mangle]
pub unsafe extern "C" fn rs_handle_termkey_modifiers(
    modifiers: c_int,
    buf: *mut c_char,
    buflen: usize,
) -> usize {
    if buf.is_null() || buflen == 0 {
        return 0;
    }

    let buf_slice = std::slice::from_raw_parts_mut(buf as *mut u8, buflen);
    handle_termkey_modifiers_impl(modifiers, buf_slice)
}

/// Internal implementation of termkey modifier handling
fn handle_termkey_modifiers_impl(modifiers: c_int, buf: &mut [u8]) -> usize {
    let mut len = 0usize;

    // Shift
    if (modifiers & TERMKEY_KEYMOD_SHIFT) != 0 {
        if len + 2 <= buf.len() {
            buf[len] = b'S';
            buf[len + 1] = b'-';
            len += 2;
        }
    }

    // Alt
    if (modifiers & TERMKEY_KEYMOD_ALT) != 0 {
        if len + 2 <= buf.len() {
            buf[len] = b'A';
            buf[len + 1] = b'-';
            len += 2;
        }
    }

    // Ctrl
    if (modifiers & TERMKEY_KEYMOD_CTRL) != 0 {
        if len + 2 <= buf.len() {
            buf[len] = b'C';
            buf[len + 1] = b'-';
            len += 2;
        }
    }

    len
}

/// Handle additional modifiers not handled by libtermkey.
///
/// Currently handles Super ("D-") and Meta ("T-") modifiers that are
/// supported in Nvim but not directly by libtermkey.
///
/// # Safety
///
/// - `buf` must point to a valid buffer of at least `buflen` bytes
/// - The caller must ensure the buffer has enough space for the modifiers
///   (up to 4 bytes for "D-T-")
///
/// # Arguments
/// * `modifiers` - Modifier flags (including KEYMOD_SUPER and KEYMOD_META)
/// * `buf` - Buffer to write to
/// * `buflen` - Size of buffer
///
/// # Returns
/// Number of bytes written to buffer, excluding any NUL terminator
#[no_mangle]
pub unsafe extern "C" fn rs_handle_more_modifiers(
    modifiers: c_int,
    buf: *mut c_char,
    buflen: usize,
) -> usize {
    if buf.is_null() || buflen == 0 {
        return 0;
    }

    let buf_slice = std::slice::from_raw_parts_mut(buf as *mut u8, buflen);
    handle_more_modifiers_impl(modifiers, buf_slice)
}

/// Internal implementation of additional modifier handling
fn handle_more_modifiers_impl(modifiers: c_int, buf: &mut [u8]) -> usize {
    let mut len = 0usize;

    // Super (D-)
    if (modifiers & KEYMOD_SUPER) != 0 {
        if len + 2 <= buf.len() {
            buf[len] = b'D';
            buf[len + 1] = b'-';
            len += 2;
        }
    }

    // Meta (T-)
    if (modifiers & KEYMOD_META) != 0 {
        if len + 2 <= buf.len() {
            buf[len] = b'T';
            buf[len + 1] = b'-';
            len += 2;
        }
    }

    len
}

// ============================================================================
// Terminfo Functions
// ============================================================================

/// Checks if `term` is a member of the given `family`.
///
/// A terminal is considered a member of a family if:
/// - `term` starts with `family`
/// - Either `term` equals `family` exactly, or the character following `family`
///   in `term` is '-' or '.'
///
/// For example, "xterm-256color" is in the "xterm" family.
/// "screen.xterm" is in the "screen" family.
///
/// # Safety
///
/// Both `term` and `family` must be valid, NUL-terminated C strings.
///
/// # Arguments
/// * `term` - The terminal name to check (may be NULL)
/// * `family` - The family name to check against
///
/// # Returns
/// 1 if `term` is in the `family`, 0 otherwise
#[no_mangle]
pub unsafe extern "C" fn rs_terminfo_is_term_family(
    term: *const c_char,
    family: *const c_char,
) -> c_int {
    if term.is_null() {
        return 0;
    }

    // Safety: caller guarantees these are valid C strings
    let term_cstr = unsafe { CStr::from_ptr(term) };
    let family_cstr = unsafe { CStr::from_ptr(family) };

    let term_bytes = term_cstr.to_bytes();
    let family_bytes = family_cstr.to_bytes();

    let tlen = term_bytes.len();
    let flen = family_bytes.len();

    if tlen < flen {
        return 0;
    }

    // Check if term starts with family
    if &term_bytes[..flen] != family_bytes {
        return 0;
    }

    // Check the separator condition:
    // Either term equals family exactly, or the next char is '-' or '.'
    if tlen == flen {
        return 1;
    }

    let next_char = term_bytes[flen];
    c_int::from(next_char == b'-' || next_char == b'.')
}

/// Checks if the terminal is a BSD console.
///
/// This function detects BSD console terminals:
/// - On OpenBSD: "vt220"
/// - On NetBSD: "vt100"
/// - On FreeBSD: "xterm" when XTERM_VERSION env var exists (degraded xterm)
///
/// # Safety
///
/// `term` must be a valid, NUL-terminated C string, or NULL.
///
/// # Arguments
/// * `term` - The terminal name to check (may be NULL)
///
/// # Returns
/// 1 if the terminal is a BSD console, 0 otherwise
#[no_mangle]
pub unsafe extern "C" fn rs_terminfo_is_bsd_console(term: *const c_char) -> c_int {
    // This is only relevant on BSD systems
    #[cfg(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    ))]
    {
        if term.is_null() {
            return 0;
        }

        let term_cstr = unsafe { CStr::from_ptr(term) };
        let term_bytes = term_cstr.to_bytes();

        // OpenBSD
        if term_bytes == b"vt220" {
            return 1;
        }

        // NetBSD
        if term_bytes == b"vt100" {
            return 1;
        }

        // FreeBSD specific check
        #[cfg(target_os = "freebsd")]
        {
            if term_bytes == b"xterm" {
                // Check if XTERM_VERSION env var exists
                // FreeBSD console sets TERM=xterm but doesn't support xterm features
                extern "C" {
                    fn os_env_exists(name: *const c_char, use_strequal: c_int) -> c_int;
                }
                let name = c"XTERM_VERSION";
                if unsafe { os_env_exists(name.as_ptr(), 1) } != 0 {
                    return 1;
                }
            }
        }

        0
    }

    #[cfg(not(any(
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly"
    )))]
    {
        let _ = term; // Suppress unused warning
        0
    }
}

// ============================================================================
// Terminfo Format String Processor
// ============================================================================

/// Terminfo parameter variable (matches C TPVAR struct)
#[repr(C)]
pub struct TpVar {
    pub num: c_long,
    pub string: *mut c_char,
}

/// Internal stack for terminfo format processing
struct TpStack {
    nums: [c_long; 20],
    strings: [*mut c_char; 20],
    offset: usize,
}

impl TpStack {
    fn new() -> Self {
        Self {
            nums: [0; 20],
            strings: [std::ptr::null_mut(); 20],
            offset: 0,
        }
    }

    fn push(&mut self, num: c_long, string: *mut c_char) -> bool {
        if self.offset >= self.nums.len() {
            return false;
        }
        self.nums[self.offset] = num;
        self.strings[self.offset] = string;
        self.offset += 1;
        true
    }

    fn pop(&mut self) -> (c_long, *mut c_char) {
        if self.offset == 0 {
            return (0, std::ptr::null_mut());
        }
        self.offset -= 1;
        (self.nums[self.offset], self.strings[self.offset])
    }
}

/// Process a terminfo format string.
///
/// This is a stack-based interpreter for terminfo parameterized strings.
/// It handles format specifiers like %d, %s, arithmetic operations,
/// conditionals (%?...%t...%e...%;), and parameter references (%p1-%p9).
///
/// # Safety
///
/// - `buf_start` and `buf_end` must point to a valid buffer
/// - `str` must be a valid NUL-terminated C string
/// - `params` must point to an array of 9 TpVar elements
///
/// # Returns
/// Number of bytes written to buffer, or 0 on error
#[no_mangle]
pub unsafe extern "C" fn rs_terminfo_fmt(
    buf_start: *mut c_char,
    buf_end: *const c_char,
    str_ptr: *const c_char,
    params: *mut TpVar,
) -> usize {
    if buf_start.is_null() || buf_end.is_null() || str_ptr.is_null() || params.is_null() {
        return 0;
    }

    let buf_len = buf_end.offset_from(buf_start) as usize;
    let buf_slice = std::slice::from_raw_parts_mut(buf_start as *mut u8, buf_len);
    let params_slice = std::slice::from_raw_parts_mut(params, 9);

    // Get string as bytes (without NUL)
    let cstr = CStr::from_ptr(str_ptr);
    let input = cstr.to_bytes();

    terminfo_fmt_impl(buf_slice, input, params_slice).unwrap_or_default()
}

/// Internal implementation of terminfo format string processing
fn terminfo_fmt_impl(buf: &mut [u8], input: &[u8], params: &mut [TpVar]) -> Option<usize> {
    let mut stack = TpStack::new();
    let mut dnums = [0i64; 26]; // dynamic variables a-z
    let mut snums = [0i64; 26]; // static variables A-Z

    let mut out_pos = 0usize;
    let mut i = 0usize;

    while i < input.len() {
        let c = input[i];
        i += 1;

        if c != b'%' {
            // Regular character - output it
            if out_pos >= buf.len() - 1 {
                return None;
            }
            buf[out_pos] = c;
            out_pos += 1;
            continue;
        }

        // Handle %% escape
        if i >= input.len() {
            break;
        }
        let c = input[i];
        i += 1;

        if c == b'%' {
            if out_pos >= buf.len() - 1 {
                return None;
            }
            buf[out_pos] = b'%';
            out_pos += 1;
            continue;
        }

        // Parse format specifiers and commands
        let mut ctx = FmtContext {
            stack: &mut stack,
            dnums: &mut dnums,
            snums: &mut snums,
            params,
        };
        let (new_i, new_out_pos) =
            process_format_command(buf, out_pos, input, i - 1, &mut ctx)?;
        i = new_i;
        out_pos = new_out_pos;
    }

    Some(out_pos)
}

/// Context for format command processing
struct FmtContext<'a> {
    stack: &'a mut TpStack,
    dnums: &'a mut [i64; 26],
    snums: &'a mut [i64; 26],
    params: &'a mut [TpVar],
}

/// Process a single format command starting at position `start` (at the character after %)
fn process_format_command(
    buf: &mut [u8],
    mut out_pos: usize,
    input: &[u8],
    start: usize,
    ctx: &mut FmtContext<'_>,
) -> Option<(usize, usize)> {
    let mut i = start;
    let c = input[i];
    i += 1;

    // Parse format modifiers (width, precision, flags)
    let mut width: usize = 0;
    let mut precision: usize = 0;
    let mut val: i64 = 0;
    let mut dot = false;
    let mut minus_allowed = false;
    let mut done = false;
    let mut cmd = c;

    while !done {
        match cmd {
            b'c' | b's' | b'd' | b'o' | b'x' | b'X' => {
                done = true;
            }
            b'#' | b' ' => {}
            b'.' => {
                if !dot {
                    dot = true;
                    width = val as usize;
                } else {
                    // Error - multiple dots
                    return Some((i, out_pos));
                }
                val = 0;
            }
            b':' => {
                minus_allowed = true;
            }
            b'-' => {
                if !minus_allowed {
                    done = true;
                }
            }
            b'0'..=b'9' => {
                val = val * 10 + i64::from(cmd - b'0');
                if val > 10000 {
                    // Error - value too large
                    return Some((i, out_pos));
                }
            }
            _ => {
                done = true;
            }
        }
        if !done {
            if i >= input.len() {
                break;
            }
            cmd = input[i];
            i += 1;
        }
    }

    if !dot {
        width = val as usize;
    } else {
        precision = val as usize;
    }
    let olen = width.max(precision);

    // Execute the command
    match cmd {
        b'c' => {
            let (val, _) = ctx.stack.pop();
            let ch = if val == 0 { 0o200u8 } else { val as u8 };
            if out_pos >= buf.len() - 1 {
                return None;
            }
            buf[out_pos] = ch;
            out_pos += 1;
        }
        b's' => {
            let (_, str_ptr) = ctx.stack.pop();
            if !str_ptr.is_null() {
                let cstr = unsafe { CStr::from_ptr(str_ptr) };
                let s = cstr.to_bytes();
                let len = s.len().max(olen);
                if out_pos + len + 1 > buf.len() {
                    return None;
                }
                // Simple string copy (ignoring printf formatting for now)
                for &byte in s {
                    buf[out_pos] = byte;
                    out_pos += 1;
                }
            }
        }
        b'l' => {
            let (_, str_ptr) = ctx.stack.pop();
            let len = if str_ptr.is_null() {
                0
            } else {
                unsafe { CStr::from_ptr(str_ptr) }.to_bytes().len()
            };
            ctx.stack.push(len as c_long, std::ptr::null_mut());
        }
        b'd' | b'o' | b'x' | b'X' => {
            let (val, _) = ctx.stack.pop();
            let available = buf.len() - out_pos;
            if available < olen.max(21) + 2 {
                return None;
            }
            // Format the number
            let written = format_number(&mut buf[out_pos..], val, cmd, width);
            out_pos += written;
        }
        b'p' => {
            // Push parameter %p1-%p9
            if i < input.len() {
                let param_char = input[i];
                i += 1;
                if (b'1'..=b'9').contains(&param_char) {
                    let idx = (param_char - b'1') as usize;
                    if !ctx.stack.push(ctx.params[idx].num, ctx.params[idx].string) {
                        return None;
                    }
                }
            }
        }
        b'P' => {
            // Pop to variable
            let (val, _) = ctx.stack.pop();
            if i < input.len() {
                let var = input[i];
                i += 1;
                if var.is_ascii_lowercase() {
                    ctx.dnums[(var - b'a') as usize] = val;
                } else if var.is_ascii_uppercase() {
                    ctx.snums[(var - b'A') as usize] = val;
                }
            }
        }
        b'g' => {
            // Get variable
            if i < input.len() {
                let var = input[i];
                i += 1;
                let val = if var.is_ascii_lowercase() {
                    ctx.dnums[(var - b'a') as usize]
                } else if var.is_ascii_uppercase() {
                    ctx.snums[(var - b'A') as usize]
                } else {
                    0
                };
                if !ctx.stack.push(val as c_long, std::ptr::null_mut()) {
                    return None;
                }
            }
        }
        b'i' => {
            // Increment first two params
            ctx.params[0].num += 1;
            ctx.params[1].num += 1;
        }
        b'\'' => {
            // Character constant
            if i < input.len() {
                let ch = input[i];
                i += 1;
                if !ctx.stack.push(c_long::from(ch), std::ptr::null_mut()) {
                    return None;
                }
                // Skip to closing quote
                while i < input.len() && input[i] != b'\'' {
                    i += 1;
                }
                if i < input.len() {
                    i += 1;
                }
            }
        }
        b'{' => {
            // Numeric constant
            let mut val: i64 = 0;
            while i < input.len() && input[i].is_ascii_digit() {
                val = val * 10 + i64::from(input[i] - b'0');
                i += 1;
            }
            if !ctx.stack.push(val as c_long, std::ptr::null_mut()) {
                return None;
            }
            // Skip to closing brace
            while i < input.len() && input[i] != b'}' {
                i += 1;
            }
            if i < input.len() {
                i += 1;
            }
        }
        b'+' | b'-' | b'*' | b'/' | b'm' | b'A' | b'O' | b'&' | b'|' | b'^' | b'=' | b'<'
        | b'>' => {
            let (val1, _) = ctx.stack.pop();
            let (val2, _) = ctx.stack.pop();
            let result = match cmd {
                b'+' => val1 + val2,
                b'-' => val2 - val1,
                b'*' => val1 * val2,
                b'/' => {
                    if val1 != 0 {
                        val2 / val1
                    } else {
                        0
                    }
                }
                b'm' => {
                    if val1 != 0 {
                        val2 % val1
                    } else {
                        0
                    }
                }
                b'A' => c_long::from(val1 != 0 && val2 != 0),
                b'O' => c_long::from(val1 != 0 || val2 != 0),
                b'&' => val1 & val2,
                b'|' => val1 | val2,
                b'^' => val1 ^ val2,
                b'=' => c_long::from(val1 == val2),
                b'<' => c_long::from(val2 < val1),
                b'>' => c_long::from(val2 > val1),
                _ => 0,
            };
            if !ctx.stack.push(result, std::ptr::null_mut()) {
                return None;
            }
        }
        b'!' | b'~' => {
            let (val, _) = ctx.stack.pop();
            let result = match cmd {
                b'!' => c_long::from(val == 0),
                b'~' => !val,
                _ => val,
            };
            if !ctx.stack.push(result, std::ptr::null_mut()) {
                return None;
            }
        }
        b'?' => {
            // if - nothing to do, just continue
        }
        b't' => {
            // then
            let (val, _) = ctx.stack.pop();
            if val == 0 {
                // Skip to %e or %;
                i = skip_to_else_or_endif(input, i);
            }
        }
        b'e' => {
            // else - skip to %;
            i = skip_to_endif(input, i);
        }
        b';' => {
            // endif - nothing to do
        }
        _ => {}
    }

    Some((i, out_pos))
}

/// Skip to %e or %; handling nested conditionals
fn skip_to_else_or_endif(input: &[u8], mut i: usize) -> usize {
    let mut level = 0usize;
    while i < input.len() {
        if input[i] != b'%' {
            i += 1;
            continue;
        }
        i += 1;
        if i >= input.len() {
            break;
        }
        match input[i] {
            b'?' => level += 1,
            b';' => {
                if level > 0 {
                    level -= 1;
                } else {
                    i += 1;
                    break;
                }
            }
            b'e' if level == 0 => {
                i += 1;
                break;
            }
            _ => {}
        }
        i += 1;
    }
    i
}

/// Skip to %; handling nested conditionals
fn skip_to_endif(input: &[u8], mut i: usize) -> usize {
    let mut level = 0usize;
    while i < input.len() {
        if input[i] != b'%' {
            i += 1;
            continue;
        }
        i += 1;
        if i >= input.len() {
            break;
        }
        match input[i] {
            b'?' => level += 1,
            b';' => {
                if level > 0 {
                    level -= 1;
                } else {
                    i += 1;
                    break;
                }
            }
            _ => {}
        }
        i += 1;
    }
    i
}

/// Format a number according to the command
fn format_number(buf: &mut [u8], val: c_long, cmd: u8, width: usize) -> usize {
    let mut tmp = [0u8; 32];
    let mut cursor = std::io::Cursor::new(&mut tmp[..]);
    let _ = match cmd {
        b'd' => write!(cursor, "{:width$}", val, width = width),
        b'o' => write!(cursor, "{:width$o}", val, width = width),
        b'x' => write!(cursor, "{:width$x}", val, width = width),
        b'X' => write!(cursor, "{:width$X}", val, width = width),
        _ => write!(cursor, "{}", val),
    };
    let len = cursor.position() as usize;
    buf[..len].copy_from_slice(&tmp[..len]);
    len
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn is_term_family(term: &str, family: &str) -> bool {
        let term_c = CString::new(term).unwrap();
        let family_c = CString::new(family).unwrap();
        unsafe { rs_terminfo_is_term_family(term_c.as_ptr(), family_c.as_ptr()) != 0 }
    }

    fn is_term_family_null(family: &str) -> bool {
        let family_c = CString::new(family).unwrap();
        unsafe { rs_terminfo_is_term_family(std::ptr::null(), family_c.as_ptr()) != 0 }
    }

    #[test]
    fn test_terminfo_is_term_family_exact_match() {
        assert!(is_term_family("xterm", "xterm"));
        assert!(is_term_family("screen", "screen"));
        assert!(is_term_family("tmux", "tmux"));
    }

    #[test]
    fn test_terminfo_is_term_family_with_dash() {
        assert!(is_term_family("xterm-256color", "xterm"));
        assert!(is_term_family("screen-256color", "screen"));
        assert!(is_term_family("tmux-256color", "tmux"));
        assert!(is_term_family("rxvt-unicode", "rxvt"));
    }

    #[test]
    fn test_terminfo_is_term_family_with_dot() {
        // screen.xterm is in the screen family
        assert!(is_term_family("screen.xterm", "screen"));
        assert!(is_term_family("iterm.app", "iterm"));
    }

    #[test]
    fn test_terminfo_is_term_family_no_match() {
        // Not a match - different family
        assert!(!is_term_family("xterm", "screen"));
        assert!(!is_term_family("rxvt", "xterm"));

        // Not a match - prefix but no separator
        assert!(!is_term_family("xterminator", "xterm"));
        assert!(!is_term_family("screenx", "screen"));
    }

    #[test]
    fn test_terminfo_is_term_family_null_term() {
        assert!(!is_term_family_null("xterm"));
    }

    #[test]
    fn test_terminfo_is_term_family_shorter_term() {
        // term is shorter than family
        assert!(!is_term_family("xt", "xterm"));
        assert!(!is_term_family("x", "xterm"));
    }

    #[test]
    fn test_terminfo_is_bsd_console_non_bsd() {
        // On non-BSD systems, should always return 0
        #[cfg(not(any(
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "dragonfly"
        )))]
        {
            let term = CString::new("vt100").unwrap();
            assert_eq!(
                unsafe { rs_terminfo_is_bsd_console(term.as_ptr()) },
                0
            );
            let term = CString::new("vt220").unwrap();
            assert_eq!(
                unsafe { rs_terminfo_is_bsd_console(term.as_ptr()) },
                0
            );
        }
    }

    #[test]
    fn test_terminfo_is_bsd_console_null() {
        assert_eq!(unsafe { rs_terminfo_is_bsd_console(std::ptr::null()) }, 0);
    }

    // Helper for testing terminfo_fmt
    fn fmt_test(format: &str, params: &[(i64, Option<&str>)]) -> String {
        let mut buf = [0u8; 256];
        let format_c = CString::new(format).unwrap();

        let mut tpvars: Vec<TpVar> = params
            .iter()
            .map(|(n, s)| TpVar {
                num: *n as c_long,
                string: s.map_or(std::ptr::null_mut(), |s| {
                    CString::new(s).unwrap().into_raw()
                }),
            })
            .collect();

        // Pad to 9 elements
        while tpvars.len() < 9 {
            tpvars.push(TpVar {
                num: 0,
                string: std::ptr::null_mut(),
            });
        }

        let len = unsafe {
            rs_terminfo_fmt(
                buf.as_mut_ptr() as *mut c_char,
                buf.as_ptr().add(buf.len()) as *const c_char,
                format_c.as_ptr(),
                tpvars.as_mut_ptr(),
            )
        };

        String::from_utf8_lossy(&buf[..len]).into_owned()
    }

    #[test]
    fn test_terminfo_fmt_literal() {
        assert_eq!(fmt_test("hello", &[]), "hello");
        assert_eq!(fmt_test("%%", &[]), "%");
        assert_eq!(fmt_test("a%%b", &[]), "a%b");
    }

    #[test]
    fn test_terminfo_fmt_param() {
        // %p1%d pushes param 1 and formats as decimal
        assert_eq!(fmt_test("%p1%d", &[(42, None)]), "42");
        assert_eq!(fmt_test("%p1%d%p2%d", &[(10, None), (20, None)]), "1020");
    }

    #[test]
    fn test_terminfo_fmt_arithmetic() {
        // %p1%p2%+ adds params
        assert_eq!(fmt_test("%p1%p2%+%d", &[(10, None), (20, None)]), "30");
        // %p1%p2%- subtracts
        assert_eq!(fmt_test("%p1%p2%-%d", &[(30, None), (10, None)]), "20");
        // %p1%p2%* multiplies
        assert_eq!(fmt_test("%p1%p2%*%d", &[(5, None), (6, None)]), "30");
    }

    #[test]
    fn test_terminfo_fmt_increment() {
        // %i increments first two params
        assert_eq!(fmt_test("%i%p1%d", &[(0, None)]), "1");
        assert_eq!(fmt_test("%i%p1%d%p2%d", &[(0, None), (0, None)]), "11");
    }

    #[test]
    fn test_terminfo_fmt_conditional() {
        // %?%p1%tTRUE%;
        assert_eq!(fmt_test("%?%p1%tTRUE%;", &[(1, None)]), "TRUE");
        assert_eq!(fmt_test("%?%p1%tTRUE%;", &[(0, None)]), "");
        // %?%p1%tTRUE%eFALSE%;
        assert_eq!(fmt_test("%?%p1%tTRUE%eFALSE%;", &[(1, None)]), "TRUE");
        assert_eq!(fmt_test("%?%p1%tTRUE%eFALSE%;", &[(0, None)]), "FALSE");
    }

    #[test]
    fn test_terminfo_fmt_numeric_constant() {
        // %{42}%d pushes 42 and formats
        assert_eq!(fmt_test("%{42}%d", &[]), "42");
        assert_eq!(fmt_test("%p1%{10}%+%d", &[(5, None)]), "15");
    }

    // ========================================================================
    // Key Modifier Tests
    // ========================================================================

    /// Helper to test termkey modifiers
    fn termkey_modifiers_test(modifiers: c_int) -> String {
        let mut buf = [0u8; 64];
        let len = handle_termkey_modifiers_impl(modifiers, &mut buf);
        String::from_utf8_lossy(&buf[..len]).into_owned()
    }

    /// Helper to test additional modifiers
    fn more_modifiers_test(modifiers: c_int) -> String {
        let mut buf = [0u8; 64];
        let len = handle_more_modifiers_impl(modifiers, &mut buf);
        String::from_utf8_lossy(&buf[..len]).into_owned()
    }

    #[test]
    fn test_termkey_modifiers_none() {
        assert_eq!(termkey_modifiers_test(0), "");
    }

    #[test]
    fn test_termkey_modifiers_shift() {
        assert_eq!(termkey_modifiers_test(TERMKEY_KEYMOD_SHIFT), "S-");
    }

    #[test]
    fn test_termkey_modifiers_alt() {
        assert_eq!(termkey_modifiers_test(TERMKEY_KEYMOD_ALT), "A-");
    }

    #[test]
    fn test_termkey_modifiers_ctrl() {
        assert_eq!(termkey_modifiers_test(TERMKEY_KEYMOD_CTRL), "C-");
    }

    #[test]
    fn test_termkey_modifiers_shift_alt() {
        assert_eq!(
            termkey_modifiers_test(TERMKEY_KEYMOD_SHIFT | TERMKEY_KEYMOD_ALT),
            "S-A-"
        );
    }

    #[test]
    fn test_termkey_modifiers_all() {
        assert_eq!(
            termkey_modifiers_test(TERMKEY_KEYMOD_SHIFT | TERMKEY_KEYMOD_ALT | TERMKEY_KEYMOD_CTRL),
            "S-A-C-"
        );
    }

    #[test]
    fn test_more_modifiers_none() {
        assert_eq!(more_modifiers_test(0), "");
    }

    #[test]
    fn test_more_modifiers_super() {
        assert_eq!(more_modifiers_test(KEYMOD_SUPER), "D-");
    }

    #[test]
    fn test_more_modifiers_meta() {
        assert_eq!(more_modifiers_test(KEYMOD_META), "T-");
    }

    #[test]
    fn test_more_modifiers_both() {
        assert_eq!(more_modifiers_test(KEYMOD_SUPER | KEYMOD_META), "D-T-");
    }

    #[test]
    fn test_termkey_modifiers_ffi() {
        let mut buf = [0i8; 64];
        let len = unsafe {
            rs_handle_termkey_modifiers(
                TERMKEY_KEYMOD_CTRL | TERMKEY_KEYMOD_ALT,
                buf.as_mut_ptr(),
                buf.len(),
            )
        };
        let result =
            String::from_utf8_lossy(unsafe { std::mem::transmute::<&[i8], &[u8]>(&buf[..len]) });
        assert_eq!(result, "A-C-");
    }

    #[test]
    fn test_more_modifiers_ffi() {
        let mut buf = [0i8; 64];
        let len = unsafe {
            rs_handle_more_modifiers(KEYMOD_SUPER | KEYMOD_META, buf.as_mut_ptr(), buf.len())
        };
        let result =
            String::from_utf8_lossy(unsafe { std::mem::transmute::<&[i8], &[u8]>(&buf[..len]) });
        assert_eq!(result, "D-T-");
    }

    #[test]
    fn test_termkey_modifiers_null_buf() {
        let len = unsafe { rs_handle_termkey_modifiers(TERMKEY_KEYMOD_CTRL, std::ptr::null_mut(), 64) };
        assert_eq!(len, 0);
    }

    #[test]
    fn test_termkey_modifiers_zero_len() {
        let mut buf = [0i8; 1];
        let len = unsafe {
            rs_handle_termkey_modifiers(TERMKEY_KEYMOD_CTRL, buf.as_mut_ptr(), 0)
        };
        assert_eq!(len, 0);
    }
}
