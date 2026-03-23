//! Complex string option callback implementations
//!
//! This module contains Rust implementations of complex string option validation
//! callbacks. These callbacks have more complex parsing logic or require
//! integration with C code for side effects.

#![allow(clippy::missing_safety_doc)] // FFI functions safety is implicit

use std::ffi::{c_char, c_int, c_void};

use super::{callback_ok, CallbackResult};

extern "C" {
    static Rows: c_int;
    static p_ru: c_int;
}

// =============================================================================
// Error Messages
// =============================================================================

/// Error: Invalid argument
const E_INVARG: *const c_char = c"E474: Invalid argument".as_ptr();

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if bytes at pointer match expected bytes.
#[inline]
unsafe fn bytes_match(ptr: *const c_char, expected: &[u8]) -> bool {
    for (i, &expected_byte) in expected.iter().enumerate() {
        if *ptr.add(i) as u8 != expected_byte {
            return false;
        }
    }
    true
}

// =============================================================================
// 'helplang' Option Validation
// =============================================================================

/// Validate 'helplang' option value.
/// Format: "", "ab", "ab,cd", etc. (two-letter language codes)
#[no_mangle]
pub unsafe extern "C" fn rs_validate_helplang(value: *const c_char) -> CallbackResult {
    if value.is_null() {
        return callback_ok();
    }

    // Empty string is valid
    if *value == 0 {
        return callback_ok();
    }

    // Check for "ab", "ab,cd", etc.
    let mut s = value;
    loop {
        // Must have at least two characters
        if *s == 0 || *s.add(1) == 0 {
            return E_INVARG;
        }

        // After two characters, must have comma or end
        let third = *s.add(2) as u8;
        if third == 0 {
            // End of string - valid
            break;
        }
        if third != b',' {
            return E_INVARG;
        }

        // After comma, must have more content
        if *s.add(3) == 0 {
            return E_INVARG;
        }

        // Move to next language code
        s = s.add(3);
    }

    callback_ok()
}

// =============================================================================
// 'shada' Option Validation
// =============================================================================

/// Valid first characters for 'shada' option items
const SHADA_CHARS: &[u8] = b"!\"%':<@cfhnrs";

/// Validate 'shada' option value.
///
/// Returns NULL on success, or an error message on failure.
/// Note: This only validates the syntax. The C side handles the
/// "must specify a ' value" check since it needs `get_shada_parameter()`.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_shada(
    value: *const c_char,
    errbuf: *mut c_char,
    errbuflen: usize,
) -> CallbackResult {
    if value.is_null() || *value == 0 {
        return callback_ok();
    }

    let mut s = value;

    while *s != 0 {
        let ch = *s as u8;

        // Check it's a valid character
        if !SHADA_CHARS.contains(&ch) {
            // Return illegal character error
            if !errbuf.is_null() && errbuflen > 0 {
                write_illegal_char_error(errbuf, errbuflen, ch);
            }
            return if errbuf.is_null() {
                c"".as_ptr()
            } else {
                errbuf
            };
        }

        if ch == b'n' {
            // 'n' (name) is always last one - stop parsing
            break;
        } else if ch == b'r' {
            // skip until next ','
            s = s.add(1);
            while *s != 0 && *s as u8 != b',' {
                s = s.add(1);
            }
        } else if ch == b'%' {
            // optional number
            s = s.add(1);
            while *s != 0 && (*s as u8).is_ascii_digit() {
                s = s.add(1);
            }
        } else if ch == b'!' || ch == b'h' || ch == b'c' {
            // no extra chars
            s = s.add(1);
        } else {
            // must have a number
            s = s.add(1);
            while *s != 0 && (*s as u8).is_ascii_digit() {
                s = s.add(1);
            }

            // Check if we had at least one digit
            let prev = *s.sub(1) as u8;
            if !prev.is_ascii_digit() {
                // E526: Missing number after <x>
                if !errbuf.is_null() && errbuflen > 0 {
                    write_missing_number_error(errbuf, errbuflen, prev);
                }
                return if errbuf.is_null() {
                    c"".as_ptr()
                } else {
                    errbuf
                };
            }
        }

        // Check for comma or end
        if *s as u8 == b',' {
            s = s.add(1);
        } else if *s != 0 {
            // E527: Missing comma
            return c"E527: Missing comma".as_ptr();
        }
    }

    callback_ok()
}

/// Write an illegal character error message to the buffer.
#[allow(clippy::cast_possible_wrap)]
unsafe fn write_illegal_char_error(errbuf: *mut c_char, errbuflen: usize, ch: u8) {
    let msg = b"E langarg: Illegal character: ";
    let msg_len = msg.len().min(errbuflen.saturating_sub(2));

    for (i, &b) in msg.iter().take(msg_len).enumerate() {
        *errbuf.add(i) = b as c_char;
    }

    if msg_len < errbuflen.saturating_sub(1) {
        *errbuf.add(msg_len) = ch as c_char;
        *errbuf.add(msg_len + 1) = 0;
    } else {
        *errbuf.add(msg_len) = 0;
    }
}

/// Write a missing number error message to the buffer.
#[allow(clippy::cast_possible_wrap)]
unsafe fn write_missing_number_error(errbuf: *mut c_char, errbuflen: usize, ch: u8) {
    let msg = b"E526: Missing number after <";
    let msg_len = msg.len().min(errbuflen.saturating_sub(3));

    for (i, &b) in msg.iter().take(msg_len).enumerate() {
        *errbuf.add(i) = b as c_char;
    }

    if msg_len < errbuflen.saturating_sub(2) {
        *errbuf.add(msg_len) = ch as c_char;
        *errbuf.add(msg_len + 1) = b'>' as c_char;
        *errbuf.add(msg_len + 2) = 0;
    } else {
        *errbuf.add(msg_len) = 0;
    }
}

// =============================================================================
// 'mousescroll' Option Validation
// =============================================================================

/// Result from validating 'mousescroll' option.
#[repr(C)]
pub struct MouseScrollResult {
    /// Vertical scroll amount (-1 if not set)
    pub vertical: i64,
    /// Horizontal scroll amount (-1 if not set)
    pub horizontal: i64,
    /// 0 on success, non-zero on error
    pub error: c_int,
}

/// Validate 'mousescroll' option value.
/// Format: "ver:N,hor:N" or "hor:N,ver:N"
#[no_mangle]
pub unsafe extern "C" fn rs_validate_mousescroll(value: *const c_char) -> MouseScrollResult {
    let mut result = MouseScrollResult {
        vertical: -1,
        horizontal: -1,
        error: 0,
    };

    if value.is_null() || *value == 0 {
        result.error = 1;
        return result;
    }

    let mut s = value;

    loop {
        // Find end of this item (comma or null)
        let mut end = s;
        while *end != 0 && *end as u8 != b',' {
            end = end.add(1);
        }

        let len = end.offset_from(s) as usize;

        // Both "ver:" and "hor:" are 4 bytes long
        // They should be followed by at least one digit
        if len <= 4 {
            result.error = 1;
            return result;
        }

        // Check which direction
        let is_ver = *s as u8 == b'v'
            && *s.add(1) as u8 == b'e'
            && *s.add(2) as u8 == b'r'
            && *s.add(3) as u8 == b':';
        let is_hor = *s as u8 == b'h'
            && *s.add(1) as u8 == b'o'
            && *s.add(2) as u8 == b'r'
            && *s.add(3) as u8 == b':';

        if !is_ver && !is_hor {
            result.error = 1;
            return result;
        }

        // Parse the number
        let mut num_start = s.add(4);
        let mut num: i64 = 0;
        let mut has_digit = false;

        while num_start < end {
            let ch = *num_start as u8;
            if !ch.is_ascii_digit() {
                result.error = 1;
                return result;
            }
            has_digit = true;
            num = num * 10 + i64::from(ch - b'0');
            num_start = num_start.add(1);
        }

        if !has_digit {
            result.error = 1;
            return result;
        }

        // Store the value
        if is_ver {
            if result.vertical >= 0 {
                // Already set - duplicate
                result.error = 1;
                return result;
            }
            result.vertical = num;
        } else {
            if result.horizontal >= 0 {
                // Already set - duplicate
                result.error = 1;
                return result;
            }
            result.horizontal = num;
        }

        // Move to next item
        if *end == 0 {
            break;
        }
        s = end.add(1);
    }

    result
}

// =============================================================================
// 'colorcolumn' Option Validation
// =============================================================================

/// Validate 'colorcolumn' option value.
/// Format: comma-separated list of column numbers or +N/-N relative values.
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_colorcolumn(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 0; // Empty is valid
    }

    let mut s = value;

    while *s != 0 {
        // Skip leading whitespace (though unusual)
        while *s as u8 == b' ' {
            s = s.add(1);
        }

        if *s == 0 {
            break;
        }

        // Check for +/- prefix
        if *s as u8 == b'+' || *s as u8 == b'-' {
            s = s.add(1);
        }

        // Must have at least one digit
        if !(*s as u8).is_ascii_digit() {
            return 1;
        }

        // Skip digits
        while (*s as u8).is_ascii_digit() {
            s = s.add(1);
        }

        // Must be comma or end
        if *s != 0 {
            if *s as u8 != b',' {
                return 1;
            }
            s = s.add(1);
        }
    }

    0
}

// =============================================================================
// 'varsofttabstop' / 'vartabstop' Option Validation
// =============================================================================

/// Validate 'varsofttabstop' or 'vartabstop' option value.
/// Format: comma-separated list of positive integers.
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_vartabs(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 0; // Empty is valid
    }

    let mut s = value;

    while *s != 0 {
        // Must start with a digit
        if !(*s as u8).is_ascii_digit() {
            return 1;
        }

        // Parse number and check it's positive
        let mut num: i64 = 0;
        while (*s as u8).is_ascii_digit() {
            num = num * 10 + i64::from(*s as u8 - b'0');
            s = s.add(1);
        }

        // Must be positive (non-zero)
        if num <= 0 {
            return 1;
        }

        // Must be comma or end
        if *s != 0 {
            if *s as u8 != b',' {
                return 1;
            }
            s = s.add(1);

            // After comma, must have more content
            if *s == 0 {
                return 1;
            }
        }
    }

    0
}

// =============================================================================
// 'eventignore' Option Validation Helper
// =============================================================================

/// Check if a string looks like "all" (case-insensitive).
#[inline]
unsafe fn is_all(s: *const c_char, len: usize) -> bool {
    if len != 3 {
        return false;
    }
    let b0 = (*s as u8).to_ascii_lowercase();
    let b1 = (*s.add(1) as u8).to_ascii_lowercase();
    let b2 = (*s.add(2) as u8).to_ascii_lowercase();
    b0 == b'a' && b1 == b'l' && b2 == b'l'
}

/// Validate a single event name in 'eventignore'.
/// Returns 1 if the event name format is valid (actual event checking done in C).
/// Valid format: alphanumeric characters only.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_eventignore_item(item: *const c_char, len: usize) -> c_int {
    if item.is_null() || len == 0 {
        return 0;
    }

    // "all" is always valid
    if is_all(item, len) {
        return 1;
    }

    // Check all characters are alphanumeric
    for i in 0..len {
        let ch = *item.add(i) as u8;
        if !ch.is_ascii_alphanumeric() {
            return 0;
        }
    }

    1
}

// =============================================================================
// 'wildmode' Option Validation
// =============================================================================

/// Valid wildmode keywords
const WILDMODE_KEYWORDS: &[&[u8]] = &[b"", b"full", b"longest", b"list", b"lastused"];

/// Check if bytes match a wildmode keyword.
#[inline]
unsafe fn matches_wildmode_keyword(s: *const c_char, len: usize) -> bool {
    for kw in WILDMODE_KEYWORDS {
        if kw.len() == len {
            let mut matches = true;
            for i in 0..len {
                if *s.add(i) as u8 != kw[i] {
                    matches = false;
                    break;
                }
            }
            if matches {
                return true;
            }
        }
    }
    false
}

/// Validate 'wildmode' option value.
/// Format: colon-separated groups, each containing comma-separated keywords.
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_wildmode(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 0;
    }

    let mut s = value;

    while *s != 0 {
        // Find end of this keyword (comma, colon, or null)
        let start = s;
        while *s != 0 && *s as u8 != b',' && *s as u8 != b':' {
            s = s.add(1);
        }

        let len = s.offset_from(start) as usize;

        // Validate keyword
        if !matches_wildmode_keyword(start, len) {
            return 1;
        }

        // Skip delimiter
        if *s != 0 {
            s = s.add(1);
        }
    }

    0
}

// =============================================================================
// 'mkspellmem' Option Validation
// =============================================================================

/// Validate 'mkspellmem' option value.
/// Format: N,N,N (three comma-separated numbers).
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_mkspellmem(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 1; // Must not be empty
    }

    let mut s = value;
    let mut count = 0;

    while *s != 0 && count < 3 {
        // Must have at least one digit
        if !(*s as u8).is_ascii_digit() {
            return 1;
        }

        // Skip digits
        while (*s as u8).is_ascii_digit() {
            s = s.add(1);
        }

        count += 1;

        // After first two numbers, must have comma
        if count < 3 {
            if *s as u8 != b',' {
                return 1;
            }
            s = s.add(1);
        }
    }

    // Must have exactly 3 numbers and nothing after
    if count != 3 || *s != 0 {
        return 1;
    }

    0
}

// =============================================================================
// 'keymodel' Option Validation
// =============================================================================

/// Validate 'keymodel' option value.
/// Valid values: empty, startsel, stopsel, or comma-separated combination.
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_keymodel(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 0; // Empty is valid
    }

    let mut s = value;

    while *s != 0 {
        let start = s;

        // Find end of item
        while *s != 0 && *s as u8 != b',' {
            s = s.add(1);
        }

        let len = s.offset_from(start) as usize;

        // Check valid keywords: "startsel" or "stopsel"
        let valid = match len {
            8 => bytes_match(start, b"startsel"),
            7 => bytes_match(start, b"stopsel"),
            _ => false,
        };

        if !valid {
            return 1;
        }

        // Skip comma
        if *s as u8 == b',' {
            s = s.add(1);
        }
    }

    0
}

// =============================================================================
// 'messagesopt' Option Validation
// =============================================================================

/// Validate 'messagesopt' option value.
/// Format: comma-separated list of keywords with optional :N suffix.
/// Valid keywords: hit-enter, wait:N, history:N
/// Returns 0 on success, 1 on error.
#[no_mangle]
pub unsafe extern "C" fn rs_validate_messagesopt(value: *const c_char) -> c_int {
    if value.is_null() || *value == 0 {
        return 1; // Must not be empty according to help
    }

    let mut s = value;

    while *s != 0 {
        let start = s;

        // Find end of keyword part (before : or ,)
        while *s != 0 && *s as u8 != b':' && *s as u8 != b',' {
            s = s.add(1);
        }

        let keyword_len = s.offset_from(start) as usize;

        // Validate keyword
        let valid_keyword = match keyword_len {
            9 => bytes_match(start, b"hit-enter"),
            4 => bytes_match(start, b"wait"),
            7 => bytes_match(start, b"history"),
            _ => false,
        };

        if !valid_keyword {
            return 1;
        }

        // Check for :N suffix (required for wait and history)
        if *s as u8 == b':' {
            s = s.add(1);
            if !(*s as u8).is_ascii_digit() {
                return 1;
            }
            while (*s as u8).is_ascii_digit() {
                s = s.add(1);
            }
        }

        // Skip comma
        if *s as u8 == b',' {
            s = s.add(1);
        }
    }

    0
}

// =============================================================================
// Arabic Option Callback
// =============================================================================

extern "C" {
    static mut p_arshape: c_int;
    static mut p_deco: c_int;
    static mut p_enc: *mut c_char;
}

extern "C" {
    fn nvim_win_get_p_arab(win: *const std::ffi::c_void) -> c_int;
    fn nvim_win_set_p_rl(win: *mut std::ffi::c_void, val: c_int);
    fn nvim_win_get_p_rl(win: *const std::ffi::c_void) -> c_int;
    fn nvim_get_p_tbidi() -> c_int;
    fn nvim_buf_set_b_p_iminsert(buf: *mut std::ffi::c_void, val: c_int);
    fn nvim_buf_set_b_p_imsearch(buf: *mut std::ffi::c_void, val: c_int);
    fn nvim_win_get_w_buffer(win: *const std::ffi::c_void) -> *mut std::ffi::c_void;
    fn changed_window_setting(win: *mut std::ffi::c_void);
    fn redraw_all_later(typ: c_int);
    fn set_option_value(opt_idx: c_int, value: *const c_char, opt_flags: c_int) -> *const c_char;
    fn msg_source(hl: c_int);
    fn msg(s: *const c_char, hl: c_int) -> c_int;
    fn set_vim_var_string(idx: c_int, s: *const c_char, len: c_int);
    fn gettext(s: *const c_char) -> *const c_char;
}

/// Constants for iminsert/imsearch values
const B_IMODE_NONE: c_int = 0;
const B_IMODE_USE_INSERT: c_int = -1;

/// Update type: NOT_VALID (from drawscreen.h)
const UPD_NOT_VALID: c_int = 40;

/// kOptKeymap index (get from C)
const K_OPT_KEYMAP: c_int = 155; // This should match the actual index in C

/// Highlight type for warnings
const HLF_W: c_int = 26;

/// VimL variable index for v:warningmsg
const VV_WARNINGMSG: c_int = 4;

/// Callback for 'arabic' option.
///
/// When 'arabic' is set or reset, handle various sub-settings:
/// - rightleft mode
/// - arabicshape
/// - delcombine
/// - keymap
/// - encoding check
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_arabic(win: *mut std::ffi::c_void) -> CallbackResult {
    if win.is_null() {
        return callback_ok();
    }

    let is_arabic = nvim_win_get_p_arab(win) != 0;
    let tbidi = nvim_get_p_tbidi() != 0;

    if is_arabic {
        // 'arabic' is set, handle various sub-settings.
        if !tbidi {
            // Set rightleft mode
            if nvim_win_get_p_rl(win) == 0 {
                nvim_win_set_p_rl(win, 1);
                changed_window_setting(win);
            }

            // Enable Arabic shaping (major part of what Arabic requires)
            if p_arshape == 0 {
                p_arshape = 1;
                redraw_all_later(UPD_NOT_VALID);
            }
        }

        // Arabic requires UTF-8 encoding, inform user if not set
        let enc = p_enc.cast_const();
        if !enc.is_null() && !streq_cstr(enc, b"utf-8\0") {
            let warning = c"W17: Arabic requires UTF-8, do ':set encoding=utf-8'";
            msg_source(HLF_W);
            msg(gettext(warning.as_ptr()), HLF_W);
            set_vim_var_string(VV_WARNINGMSG, gettext(warning.as_ptr()), -1);
        }

        // Set 'delcombine'
        p_deco = 1;

        // Force-set the necessary keymap for Arabic
        // Note: This returns an error message if it fails
        let keymap_val = c"arabic";
        let errmsg = set_option_value(K_OPT_KEYMAP, keymap_val.as_ptr(), 0x02); // OPT_LOCAL
        if !errmsg.is_null() {
            return errmsg;
        }
    } else {
        // 'arabic' is reset, handle various sub-settings.
        if !tbidi {
            // Reset rightleft mode
            if nvim_win_get_p_rl(win) != 0 {
                nvim_win_set_p_rl(win, 0);
                changed_window_setting(win);
            }
            // 'arabicshape' isn't reset, it is a global option and
            // another window may still need it "on".
        }

        // 'delcombine' isn't reset, it is a global option and another
        // window may still want it "on".

        // Revert to the default keymap
        let buf = nvim_win_get_w_buffer(win);
        if !buf.is_null() {
            nvim_buf_set_b_p_iminsert(buf, B_IMODE_NONE);
            nvim_buf_set_b_p_imsearch(buf, B_IMODE_USE_INSERT);
        }
    }

    callback_ok()
}

/// Check if C string equals a null-terminated byte slice.
#[inline]
unsafe fn streq_cstr(s: *const c_char, bytes: &[u8]) -> bool {
    if s.is_null() {
        return false;
    }
    let mut p = s;
    for &b in bytes {
        if b == 0 {
            return *p == 0;
        }
        if *p == 0 || (*p as u8) != b {
            return false;
        }
        p = p.add(1);
    }
    true
}

// =============================================================================
// Scrollbind Option Callback
// =============================================================================

extern "C" {
    fn nvim_win_get_p_scb(win: crate::WinHandle) -> c_int;
    fn do_check_scrollbind(check: c_int);
    fn rs_get_vtopline(win: crate::WinHandle) -> c_int;
    fn nvim_callback_win_set_scbind_pos(win: crate::WinHandle, value: c_int);

    // optset_T field accessors
    fn nvim_optset_get_win(args: *const c_void) -> crate::WinHandle;
    fn nvim_optset_get_buf(args: *const c_void) -> crate::BufHandle;
    fn nvim_optset_get_varp(args: *const c_void) -> *mut c_void;
}

/// Callback for 'scrollbind' option.
/// When 'scrollbind' is set, snapshot the current position to avoid a jump
/// at the end of normal_cmd().
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_scrollbind(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    if nvim_win_get_p_scb(win) == 0 {
        return callback_ok();
    }
    do_check_scrollbind(0); // false: prepare snapshot, don't sync
    nvim_callback_win_set_scbind_pos(win, rs_get_vtopline(win));
    callback_ok()
}

// =============================================================================
// Undofile Option Callback
// =============================================================================

extern "C" {
    fn nvim_buf_get_p_udf(buf: *const std::ffi::c_void) -> c_int;
    fn u_compute_hash(buf: *mut std::ffi::c_void, hash: *mut u8);
    fn u_read_undo(name: *const c_char, hash: *const u8, orig_name: *const c_char) -> c_int;
    fn nvim_buf_get_b_ffname(buf: *const std::ffi::c_void) -> *const c_char;
    static mut p_udf: c_int;
    fn nvim_buf_is_changed(buf: *mut c_void) -> c_int;
    fn nvim_buf_has_memfile(buf: *mut c_void) -> c_int;
    fn nvim_buf_get_b_fname(buf: *const c_void) -> *const c_char;
    fn nvim_for_all_buffers(callback: unsafe extern "C" fn(*mut c_void));
    fn nvim_optset_get_flags(args: *const c_void) -> c_int;
    fn nvim_optset_get_oldval_number(args: *const c_void) -> crate::OptInt;
    fn nvim_optset_get_newval_number(args: *const c_void) -> crate::OptInt;
}

/// SHA256 hash length
const UNDO_HASH_SIZE: usize = 32;

/// OPT_GLOBAL flag value (matches C option.h OPT_GLOBAL = 0x01)
const OPT_GLOBAL: c_int = 0x01;

/// Callback for 'undofile' option.
/// When 'undofile' is set, attempt to read undo file (single buffer version).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_undofile(buf: *mut std::ffi::c_void) -> CallbackResult {
    if buf.is_null() {
        return callback_ok();
    }

    if nvim_buf_get_p_udf(buf) != 0 {
        // 'undofile' was set - try to read undo file
        let fname = nvim_buf_get_b_ffname(buf);
        if !fname.is_null() && *fname != 0 {
            let mut hash = [0u8; UNDO_HASH_SIZE];
            u_compute_hash(buf, hash.as_mut_ptr());
            u_read_undo(std::ptr::null(), hash.as_ptr(), fname);
        }
    }

    callback_ok()
}

// State for the undofile all-buffers callback: the current buffer (the one
// whose option was set) and the os_flags from the optset_T.
static mut UNDOFILE_CB_BUF: *mut c_void = std::ptr::null_mut();
static mut UNDOFILE_CB_FLAGS: c_int = 0;

/// Per-buffer callback used by `rs_did_set_undofile_cb`.
unsafe extern "C" fn undofile_buf_callback(bp: *mut c_void) {
    let buf = UNDOFILE_CB_BUF;
    let flags = UNDOFILE_CB_FLAGS;

    // When 'undofile' is set globally: for every buffer, otherwise
    // only for the current buffer: Try to read in the undofile,
    // if one exists, the buffer wasn't changed and the buffer was loaded.
    if (buf == bp || (flags & OPT_GLOBAL) != 0 || flags == 0)
        && nvim_buf_is_changed(bp) == 0
        && nvim_buf_has_memfile(bp) != 0
    {
        let fname = nvim_buf_get_b_fname(bp);
        if !fname.is_null() && *fname != 0 {
            let mut hash = [0u8; UNDO_HASH_SIZE];
            u_compute_hash(bp, hash.as_mut_ptr());
            u_read_undo(std::ptr::null(), hash.as_ptr(), fname);
        }
    }
}

/// Full optset_T callback for 'undofile' option.
/// Iterates all buffers (matching the C FOR_ALL_BUFFERS behavior).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_undofile_cb(args: *mut c_void) -> CallbackResult {
    // Use buf from args as the primary buffer
    let buf = nvim_optset_get_buf(args);
    let buf_udf = if buf.is_null() {
        0
    } else {
        nvim_buf_get_p_udf(buf)
    };
    let global_udf = p_udf;

    // Only take action when the option was set.
    if buf_udf == 0 && global_udf == 0 {
        return callback_ok();
    }

    let flags = nvim_optset_get_flags(args);
    // Store state for the per-buffer callback (safe since single-threaded)
    UNDOFILE_CB_BUF = buf;
    UNDOFILE_CB_FLAGS = flags;
    nvim_for_all_buffers(undofile_buf_callback);

    callback_ok()
}

// =============================================================================
// Arabic Option Callback (optset_T wrapper)
// =============================================================================

/// Wrapper for 'arabic' callback with standard optset_T signature.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_arabic_cb(args: *mut c_void) -> CallbackResult {
    let win = nvim_optset_get_win(args);
    rs_did_set_arabic(win)
}

// =============================================================================
// Cmdheight Option Callback (optset_T wrapper)
// =============================================================================

/// Wrapper for 'cmdheight' callback with standard optset_T signature.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_cmdheight_cb(args: *mut c_void) -> CallbackResult {
    let old_value = nvim_optset_get_oldval_number(args);
    super::display::rs_did_set_cmdheight(old_value)
}

// =============================================================================
// Laststatus Option Callback (optset_T wrapper)
// =============================================================================

/// Wrapper for 'laststatus' callback with standard optset_T signature.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_laststatus_cb(args: *mut c_void) -> CallbackResult {
    let old_value = nvim_optset_get_oldval_number(args);
    let new_value = nvim_optset_get_newval_number(args);
    super::display::rs_did_set_laststatus_full(old_value, new_value)
}

// =============================================================================
// Lisp Option Callback
// =============================================================================

extern "C" {
    fn buf_init_chartab(buf: crate::BufHandle, global: c_int) -> c_int;
}

/// Callback for 'lisp' option.
/// When 'lisp' option changes, include/exclude '-' in keyword characters.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_lisp(args: *mut c_void) -> CallbackResult {
    let buf = nvim_optset_get_buf(args);
    // Reinitialize character table — this updates iskeyword-like classification
    // to include/exclude '-' depending on whether 'lisp' is set.
    buf_init_chartab(buf, 0); // ignore errors (false = not global)
    callback_ok()
}

// =============================================================================
// Autochdir Option Callback
// =============================================================================

extern "C" {
    fn do_autochdir();
}

/// Callback for 'autochdir' option.
/// Change directories when the 'acd' option is set now.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_autochdir(_args: *mut c_void) -> CallbackResult {
    do_autochdir();
    callback_ok()
}

// =============================================================================
// Paste Option Callback
// =============================================================================

extern "C" {
    static mut p_paste: c_int;
    static mut p_sm: c_int;
    static mut p_sta: c_int;
    static mut p_ri: c_int;
}

extern "C" {
    // Compound paste buf operations (Phase 12 Pass 1)
    fn nvim_buf_paste_save_scalars(buf: *mut c_void);
    fn nvim_buf_paste_save_vsts(buf: *mut c_void);
    fn nvim_buf_paste_activate_scalars(buf: *mut c_void);
    fn nvim_buf_paste_activate_vsts(buf: *mut c_void);
    fn nvim_buf_paste_restore_scalars(buf: *mut c_void);
    fn nvim_buf_paste_restore_vsts(buf: *mut c_void);
    // Compound paste global operations (Phase 12 Pass 1)
    fn nvim_paste_global_save_scalars();
    fn nvim_paste_global_save_vsts();
    fn nvim_paste_global_activate_scalars();
    fn nvim_paste_global_activate_vsts();
    fn nvim_paste_global_restore_scalars(sm: c_int, sta: c_int, ru: c_int, ri: c_int);
    fn nvim_paste_global_restore_vsts();
    fn nvim_paste_didset_sctx_all();
}

// Saved state for when 'paste' is toggled on.
static mut PASTE_OLD_P_PASTE: c_int = 0;
static mut PASTE_SAVE_SM: c_int = 0;
static mut PASTE_SAVE_STA: c_int = 0;
static mut PASTE_SAVE_RU: c_int = 0;
static mut PASTE_SAVE_RI: c_int = 0;

/// Per-buffer callback: save nopaste scalars+vsts and activate paste mode.
unsafe extern "C" fn paste_buf_save_and_activate_cb(buf: *mut c_void) {
    nvim_buf_paste_save_scalars(buf);
    nvim_buf_paste_save_vsts(buf);
    nvim_buf_paste_activate_scalars(buf);
    nvim_buf_paste_activate_vsts(buf);
}

/// Per-buffer callback: activate paste mode only (no save, already saved).
unsafe extern "C" fn paste_buf_activate_only_cb(buf: *mut c_void) {
    nvim_buf_paste_activate_scalars(buf);
    nvim_buf_paste_activate_vsts(buf);
}

/// Per-buffer callback: restore saved nopaste fields.
unsafe extern "C" fn paste_buf_restore_cb(buf: *mut c_void) {
    nvim_buf_paste_restore_scalars(buf);
    nvim_buf_paste_restore_vsts(buf);
}

/// Callback for 'paste' option.
///
/// When 'paste' is set, saves current option values and forces paste-friendly
/// settings across all buffers. When 'paste' is unset, restores the saved values.
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_paste_full(_args: *mut c_void) -> CallbackResult {
    let cur_p_paste = p_paste;

    if cur_p_paste != 0 {
        // Paste switched from off to on.
        if PASTE_OLD_P_PASTE == 0 {
            // First time paste is turned on: save current values.
            nvim_for_all_buffers(paste_buf_save_and_activate_cb);
            PASTE_SAVE_SM = p_sm;
            PASTE_SAVE_STA = p_sta;
            PASTE_SAVE_RU = p_ru;
            PASTE_SAVE_RI = p_ri;
            nvim_paste_global_save_scalars();
            nvim_paste_global_save_vsts();
        } else {
            // Paste was already on: just enforce the paste option values again.
            nvim_for_all_buffers(paste_buf_activate_only_cb);
        }
        nvim_paste_global_activate_scalars();
        nvim_paste_global_activate_vsts();
    } else if PASTE_OLD_P_PASTE != 0 {
        // Paste switched from on to off: restore saved values.
        nvim_for_all_buffers(paste_buf_restore_cb);
        nvim_paste_global_restore_scalars(
            PASTE_SAVE_SM,
            PASTE_SAVE_STA,
            PASTE_SAVE_RU,
            PASTE_SAVE_RI,
        );
        nvim_paste_global_restore_vsts();
    }

    PASTE_OLD_P_PASTE = cur_p_paste;
    nvim_paste_didset_sctx_all();

    callback_ok()
}

// =============================================================================
// Shellslash Option Callback (Windows-specific)
// =============================================================================

/// Callback for 'shellslash' option.
/// Adjusts path separators in buffer names, argument list, and script names.
/// Only active on Windows (BACKSLASH_IN_FILENAME builds).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_shellslash(_args: *mut c_void) -> CallbackResult {
    #[cfg(target_os = "windows")]
    {
        extern "C" {
            fn buflist_slash_adjust();
            fn alist_slash_adjust();
            fn rs_scriptnames_slash_adjust();
        }
        buflist_slash_adjust();
        alist_slash_adjust();
        rs_scriptnames_slash_adjust();
    }
    callback_ok()
}

// =============================================================================
// Wildchar Option Callback
// =============================================================================

/// Ctrl_C key code (verified via _Static_assert in option.c)
const CTRL_C: i64 = 3;
/// K_KENTER = TERMCAP2KEY('K', 'A') = -(75 + (65 << 8)) = -16715
const K_KENTER: i64 = -16715;

/// Callback for 'wildchar' / 'wildcharm' option.
/// Don't allow key values that wouldn't work as wildchar.
/// The value `c` is the current option value (from `*(OptInt *)args->os_varp`).
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_wildchar(args: *mut c_void) -> CallbackResult {
    let varp = nvim_optset_get_varp(args) as *const crate::OptInt;
    let c = *varp;
    if c == CTRL_C || c == i64::from(b'\n') || c == i64::from(b'\r') || c == K_KENTER {
        return (&raw const crate::e_invarg).cast::<c_char>();
    }
    callback_ok()
}

// =============================================================================
// Window Option Callback
// =============================================================================

extern "C" {
    static mut p_window: crate::OptInt;
}

/// Callback for 'window' option.
/// Ensure 'window' is clamped to [1, Rows-1].
#[no_mangle]
pub unsafe extern "C" fn rs_did_set_window(_args: *mut c_void) -> CallbackResult {
    let rows = crate::OptInt::from(Rows);
    let window = p_window;

    if window < 1 || window >= rows {
        p_window = rows - 1;
    }

    callback_ok()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    #[test]
    fn test_validate_helplang() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid_single = CString::new("en").unwrap();
            let valid_multi = CString::new("en,de,fr").unwrap();
            let invalid_short = CString::new("e").unwrap();
            let invalid_no_comma = CString::new("enen").unwrap();
            let invalid_trailing = CString::new("en,").unwrap();

            assert!(rs_validate_helplang(empty.as_ptr()).is_null());
            assert!(rs_validate_helplang(valid_single.as_ptr()).is_null());
            assert!(rs_validate_helplang(valid_multi.as_ptr()).is_null());
            assert!(!rs_validate_helplang(invalid_short.as_ptr()).is_null());
            assert!(!rs_validate_helplang(invalid_no_comma.as_ptr()).is_null());
            assert!(!rs_validate_helplang(invalid_trailing.as_ptr()).is_null());
        }
    }

    #[test]
    fn test_validate_shada() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid = CString::new("'100,<50,s10,h").unwrap();
            let valid_name = CString::new("'100,n~/.shada").unwrap();
            let mut errbuf = [0i8; 256];

            assert!(rs_validate_shada(empty.as_ptr(), errbuf.as_mut_ptr(), 256).is_null());
            assert!(rs_validate_shada(valid.as_ptr(), errbuf.as_mut_ptr(), 256).is_null());
            assert!(rs_validate_shada(valid_name.as_ptr(), errbuf.as_mut_ptr(), 256).is_null());
        }
    }

    #[test]
    fn test_validate_mousescroll() {
        unsafe {
            let valid = CString::new("ver:3,hor:6").unwrap();
            let valid_reverse = CString::new("hor:6,ver:3").unwrap();
            let valid_ver_only = CString::new("ver:5").unwrap();
            let invalid_empty = CString::new("").unwrap();
            let invalid_no_num = CString::new("ver:").unwrap();

            let result = rs_validate_mousescroll(valid.as_ptr());
            assert_eq!(result.error, 0);
            assert_eq!(result.vertical, 3);
            assert_eq!(result.horizontal, 6);

            let result = rs_validate_mousescroll(valid_reverse.as_ptr());
            assert_eq!(result.error, 0);
            assert_eq!(result.vertical, 3);
            assert_eq!(result.horizontal, 6);

            let result = rs_validate_mousescroll(valid_ver_only.as_ptr());
            assert_eq!(result.error, 0);
            assert_eq!(result.vertical, 5);
            assert_eq!(result.horizontal, -1);

            let result = rs_validate_mousescroll(invalid_empty.as_ptr());
            assert_eq!(result.error, 1);

            let result = rs_validate_mousescroll(invalid_no_num.as_ptr());
            assert_eq!(result.error, 1);
        }
    }

    #[test]
    fn test_validate_colorcolumn() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid_single = CString::new("80").unwrap();
            let valid_multi = CString::new("80,120").unwrap();
            let valid_relative = CString::new("+1,-2").unwrap();
            let invalid = CString::new("abc").unwrap();

            assert_eq!(rs_validate_colorcolumn(empty.as_ptr()), 0);
            assert_eq!(rs_validate_colorcolumn(valid_single.as_ptr()), 0);
            assert_eq!(rs_validate_colorcolumn(valid_multi.as_ptr()), 0);
            assert_eq!(rs_validate_colorcolumn(valid_relative.as_ptr()), 0);
            assert_eq!(rs_validate_colorcolumn(invalid.as_ptr()), 1);
        }
    }

    #[test]
    fn test_validate_vartabs() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid_single = CString::new("4").unwrap();
            let valid_multi = CString::new("4,8,4").unwrap();
            let invalid_zero = CString::new("0").unwrap();
            let invalid_trailing = CString::new("4,").unwrap();

            assert_eq!(rs_validate_vartabs(empty.as_ptr()), 0);
            assert_eq!(rs_validate_vartabs(valid_single.as_ptr()), 0);
            assert_eq!(rs_validate_vartabs(valid_multi.as_ptr()), 0);
            assert_eq!(rs_validate_vartabs(invalid_zero.as_ptr()), 1);
            assert_eq!(rs_validate_vartabs(invalid_trailing.as_ptr()), 1);
        }
    }

    #[test]
    fn test_validate_wildmode() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid = CString::new("longest,list,full").unwrap();
            let valid_colon = CString::new("longest:full").unwrap();
            let invalid = CString::new("invalid").unwrap();

            assert_eq!(rs_validate_wildmode(empty.as_ptr()), 0);
            assert_eq!(rs_validate_wildmode(valid.as_ptr()), 0);
            assert_eq!(rs_validate_wildmode(valid_colon.as_ptr()), 0);
            assert_eq!(rs_validate_wildmode(invalid.as_ptr()), 1);
        }
    }

    #[test]
    fn test_validate_mkspellmem() {
        unsafe {
            let valid = CString::new("460000,2000,500").unwrap();
            let invalid_empty = CString::new("").unwrap();
            let invalid_two = CString::new("100,200").unwrap();
            let invalid_four = CString::new("1,2,3,4").unwrap();

            assert_eq!(rs_validate_mkspellmem(valid.as_ptr()), 0);
            assert_eq!(rs_validate_mkspellmem(invalid_empty.as_ptr()), 1);
            assert_eq!(rs_validate_mkspellmem(invalid_two.as_ptr()), 1);
            assert_eq!(rs_validate_mkspellmem(invalid_four.as_ptr()), 1);
        }
    }

    #[test]
    fn test_validate_keymodel() {
        unsafe {
            let empty = CString::new("").unwrap();
            let valid_start = CString::new("startsel").unwrap();
            let valid_stop = CString::new("stopsel").unwrap();
            let valid_both = CString::new("startsel,stopsel").unwrap();
            let invalid = CString::new("invalid").unwrap();

            assert_eq!(rs_validate_keymodel(empty.as_ptr()), 0);
            assert_eq!(rs_validate_keymodel(valid_start.as_ptr()), 0);
            assert_eq!(rs_validate_keymodel(valid_stop.as_ptr()), 0);
            assert_eq!(rs_validate_keymodel(valid_both.as_ptr()), 0);
            assert_eq!(rs_validate_keymodel(invalid.as_ptr()), 1);
        }
    }

    #[test]
    fn test_validate_eventignore_item() {
        unsafe {
            let all = CString::new("all").unwrap();
            let all_upper = CString::new("ALL").unwrap();
            let valid = CString::new("BufEnter").unwrap();
            let invalid = CString::new("buf-enter").unwrap();

            assert_eq!(rs_validate_eventignore_item(all.as_ptr(), 3), 1);
            assert_eq!(rs_validate_eventignore_item(all_upper.as_ptr(), 3), 1);
            assert_eq!(rs_validate_eventignore_item(valid.as_ptr(), 8), 1);
            assert_eq!(rs_validate_eventignore_item(invalid.as_ptr(), 9), 0);
        }
    }
}
