//! Message formatting utilities
//!
//! Provides Rust implementations for message formatting operations
//! including string truncation, column calculation, and attribute handling.

use std::ffi::{c_char, c_int};

// C accessor declarations
extern "C" {
    /// Get `msg_col` global
    fn nvim_get_msg_col() -> c_int;
    /// Set `msg_col` global
    fn nvim_set_msg_col(col: c_int);
    /// Get `msg_row` global
    fn nvim_get_msg_row() -> c_int;
    /// Set `msg_row` global
    fn nvim_set_msg_row(row: c_int);
    /// Get `Rows` global (screen rows)
    fn nvim_get_rows() -> c_int;
    /// Get `Columns` global (screen columns)
    fn nvim_get_columns() -> c_int;
    /// Get `msg_scrolled` global
    fn nvim_get_msg_scrolled() -> c_int;
    /// Get `sc_col` global (showcmd column)
    fn nvim_get_sc_col() -> c_int;
    /// Get `msg_scroll` flag
    fn nvim_get_msg_scroll() -> c_int;
    /// Get `need_wait_return` flag
    fn nvim_get_need_wait_return() -> c_int;
    /// Check shortmess option flag
    fn nvim_shortmess(flag: c_int) -> c_int;
    /// Get `exmode_active` flag
    fn nvim_get_exmode_active() -> bool;
    /// Get `msg_silent` count
    fn nvim_get_msg_silent() -> c_int;
    /// Check if UI has messages capability
    fn nvim_ui_has_messages() -> c_int;
    /// Calculate string width in cells
    fn nvim_vim_strsize(s: *const c_char) -> c_int;
    /// Calculate truncation point
    fn nvim_mb_trunc_len(s: *const c_char, width: c_int) -> c_int;
    /// Get `cmdline_row` global
    fn nvim_get_cmdline_row() -> c_int;
}

/// Shortmess flag for "truncate all messages"
pub const SHM_TRUNCALL: c_int = b'T' as c_int;

/// Shortmess flag for "truncate file messages"
pub const SHM_TRUNC: c_int = b't' as c_int;

/// Shortmess flag for "avoid hit-enter prompts"
pub const SHM_OVERALL: c_int = b'O' as c_int;

/// Shortmess flag for "avoid hit-enter after :!cmd"
pub const SHM_LAST: c_int = b'l' as c_int;

/// Shortmess flag for "intro message"
pub const SHM_INTRO: c_int = b'I' as c_int;

/// Get the current message column.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_col() -> c_int {
    nvim_get_msg_col()
}

/// Set the current message column.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_col(col: c_int) {
    nvim_set_msg_col(col);
}

/// Get the current message row.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_row() -> c_int {
    nvim_get_msg_row()
}

/// Set the current message row.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_row(row: c_int) {
    nvim_set_msg_row(row);
}

/// Calculate available room for a message without causing scrolling.
///
/// Returns the number of screen cells available for a message.
/// Takes into account current position and whether scrolling has occurred.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_room() -> c_int {
    let rows = nvim_get_rows();
    let columns = nvim_get_columns();
    let msg_row = nvim_get_msg_row();
    let msg_scrolled = nvim_get_msg_scrolled();
    let sc_col = nvim_get_sc_col();

    if msg_scrolled != 0 {
        // Use all the columns
        (rows - msg_row) * columns - 1
    } else {
        // Use up to 'showcmd' column
        (rows - msg_row - 1) * columns + sc_col - 1
    }
}

/// Check if message truncation should be applied.
///
/// Returns true when:
/// - Not scrolling messages AND
/// - Don't need wait_return AND
/// - Shortmess has truncall flag AND
/// - Not in ex mode AND
/// - Message is silent (0) AND
/// - Not using external messages UI
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_should_truncate() -> c_int {
    let msg_scroll = nvim_get_msg_scroll() != 0;
    let need_wait_return = nvim_get_need_wait_return() != 0;
    let has_truncall = nvim_shortmess(SHM_TRUNCALL) != 0;
    let exmode_active = nvim_get_exmode_active();
    let msg_silent = nvim_get_msg_silent();
    let ui_has_messages = nvim_ui_has_messages() != 0;

    c_int::from(
        !msg_scroll
            && !need_wait_return
            && has_truncall
            && !exmode_active
            && msg_silent == 0
            && !ui_has_messages,
    )
}

/// Check if a specific shortmess flag is set.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_shortmess(flag: c_int) -> c_int {
    nvim_shortmess(flag)
}

/// Calculate the width a string would take on screen.
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_strsize(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }
    nvim_vim_strsize(s)
}

/// Check if a message would need truncation.
///
/// Returns true if the string width exceeds the available room.
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_needs_truncation(s: *const c_char) -> c_int {
    if s.is_null() {
        return 0;
    }

    let width = nvim_vim_strsize(s);
    let room = rs_msg_room();

    c_int::from(width > room && room > 0)
}

/// Advance the message position to a specific column.
///
/// If the current column is already past the target, inserts a newline first.
/// Otherwise, pads with spaces to reach the target column.
///
/// Returns the number of spaces/newlines that would need to be output.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_advance_spaces(col: c_int) -> c_int {
    let msg_col = nvim_get_msg_col();

    if msg_col >= col {
        // Need to wrap to next line first
        -1 // Signal to output newline first
    } else {
        col - msg_col // Number of spaces to add
    }
}

/// Get the number of screen columns.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_columns() -> c_int {
    nvim_get_columns()
}

/// Get the number of screen rows.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_rows() -> c_int {
    nvim_get_rows()
}

/// Calculate the truncation point for a string.
///
/// Returns the byte length of the string that would fit in `width` cells.
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_trunc_len(s: *const c_char, width: c_int) -> c_int {
    if s.is_null() || width <= 0 {
        return 0;
    }
    nvim_mb_trunc_len(s, width)
}

/// Calculate buffer size needed for truncated string.
///
/// Returns the number of bytes needed for a truncated message buffer.
/// Accounts for up to 18 bytes per cell (6 per char, up to two composing chars).
///
/// # Arguments
/// * `room` - Available screen cells
///
/// # Returns
/// Buffer size in bytes
#[no_mangle]
pub const extern "C" fn rs_msg_trunc_bufsize(room: c_int) -> c_int {
    // May have up to 18 bytes per cell (6 per char, up to two composing chars)
    (room + 2) * 18
}

/// Calculate room for truncation with ellipsis.
///
/// Subtracts 3 cells for the "..." ellipsis.
///
/// # Arguments
/// * `room_in` - Total available room
///
/// # Returns
/// Room available for content (excluding ellipsis)
#[no_mangle]
pub const extern "C" fn rs_msg_trunc_room(room_in: c_int) -> c_int {
    let room = room_in - 3; // "..." takes 3 chars
    if room < 0 {
        0
    } else {
        room
    }
}

/// Calculate half of truncation room for middle truncation.
///
/// Used for "start...end" style truncation.
///
/// # Arguments
/// * `room_in` - Total available room
///
/// # Returns
/// Half of the available room for each side
#[no_mangle]
pub const extern "C" fn rs_msg_trunc_half(room_in: c_int) -> c_int {
    let room = rs_msg_trunc_room(room_in);
    room / 2
}

/// Get the command line row.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_cmdline_row() -> c_int {
    nvim_get_cmdline_row()
}

/// Check if a message will fit in the available space.
///
/// # Arguments
/// * `len` - String length in cells
/// * `room` - Available room in cells
///
/// # Returns
/// 1 if fits, 0 if truncation needed
#[no_mangle]
pub const extern "C" fn rs_msg_fits(len: c_int, room: c_int) -> c_int {
    if len <= room {
        1
    } else {
        0
    }
}

/// Calculate the outtrans long room (for truncating long strings).
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_outtrans_long_room() -> c_int {
    nvim_get_columns() - nvim_get_msg_col()
}

/// Calculate if outtrans long should truncate.
///
/// Returns true if len > room and room >= 20.
///
/// # Arguments
/// * `len` - String length
/// * `room` - Available room
///
/// # Returns
/// 1 if should truncate, 0 otherwise
#[no_mangle]
pub const extern "C" fn rs_msg_outtrans_long_should_trunc(len: c_int, room: c_int) -> c_int {
    if len > room && room >= 20 {
        1
    } else {
        0
    }
}

/// Calculate the truncated length for outtrans long.
///
/// Returns (room - 3) / 2 for middle truncation.
///
/// # Arguments
/// * `room` - Available room
///
/// # Returns
/// Length for each side of the truncation
#[no_mangle]
pub const extern "C" fn rs_msg_outtrans_long_slen(room: c_int) -> c_int {
    (room - 3) / 2
}

/// Check if message silent is set.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_silent() -> c_int {
    nvim_get_msg_silent()
}

/// Check if message is silent (non-zero).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_msg_silent() -> c_int {
    c_int::from(nvim_get_msg_silent() != 0)
}

/// Clamp column to valid range.
///
/// Ensures column is between 0 and Columns - 1.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_clamp_col(col: c_int) -> c_int {
    let columns = nvim_get_columns();
    if col < 0 {
        0
    } else if col >= columns {
        columns - 1
    } else {
        col
    }
}

// ============================================================================
// Additional C Function Declarations for Phase 422
// ============================================================================

extern "C" {
    // Character width and length functions (from mbyte crate)
    fn rs_ptr2cells(p: *const c_char) -> c_int;
    fn rs_utfc_ptr2len(p: *const c_char) -> c_int;
    fn rs_utf_head_off(base: *const c_char, p: *const c_char) -> c_int;

    // Character translation
    fn msg_outtrans(str: *const c_char, hl_id: c_int, hist: c_int) -> c_int;
    fn msg_outtrans_len(msgstr: *const c_char, len: c_int, hl_id: c_int, hist: c_int) -> c_int;
    fn msg_outtrans_one(p: *const c_char, hl_id: c_int, hist: c_int) -> *const c_char;
    fn msg_outtrans_special(strstart: *const c_char, from: c_int, maxlen: c_int) -> c_int;

    // Memory management
    fn xmalloc(size: usize) -> *mut c_char;
    fn nvim_xfree(ptr: *mut c_char);
}

// ============================================================================
// String Truncation Functions (Phase 3.1: Pure Rust Implementation)
// ============================================================================

/// Truncate a string "s" to "buf" with cell width "room".
///
/// The result is truncated in the middle with "..." if needed.
/// "s" and "buf" may be equal for in-place truncation.
///
/// # Arguments
/// * `s` - Source string
/// * `buf` - Destination buffer
/// * `room_in` - Available screen cells
/// * `buflen` - Size of destination buffer
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
/// - `buf` must be a valid buffer of at least `buflen` bytes
#[no_mangle]
#[allow(
    clippy::too_many_lines,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_trunc_string(
    s: *const c_char,
    buf: *mut c_char,
    room_in: c_int,
    buflen: c_int,
) {
    let room = if room_in < 3 { 0 } else { room_in - 3 }; // "..." takes 3 chars

    // Empty string case
    if *s == 0 {
        if buflen > 0 {
            *buf = 0;
        }
        return;
    }

    let half = room / 2;
    let mut len: c_int = 0;
    let mut e: c_int = 0;

    // First part: Start of the string
    while len < half && e < buflen {
        if *s.offset(e as isize) == 0 {
            // Text fits without truncating!
            *buf.offset(e as isize) = 0;
            return;
        }

        let n = rs_ptr2cells(s.offset(e as isize));
        if len + n > half {
            break;
        }
        len += n;
        *buf.offset(e as isize) = *s.offset(e as isize);

        // Handle multi-byte characters
        let char_len = rs_utfc_ptr2len(s.offset(e as isize));
        for _ in 1..char_len {
            e += 1;
            if e >= buflen {
                break;
            }
            *buf.offset(e as isize) = *s.offset(e as isize);
        }
        e += 1;
    }

    // Calculate string length
    let mut slen: c_int = 0;
    while *s.offset(slen as isize) != 0 {
        slen += 1;
    }

    // Last part: End of the string - find where to start
    let mut i = slen;
    let mut half_end = slen;

    loop {
        // Move back to start of previous character
        let head_off = rs_utf_head_off(s, s.offset((half_end - 1) as isize));
        half_end = half_end - head_off - 1;

        let n = rs_ptr2cells(s.offset(half_end as isize));
        if len + n > room || half_end == 0 {
            break;
        }
        len += n;
        i = half_end;
    }

    if i <= e + 3 {
        // Text fits without truncating
        if s != buf {
            let mut copy_len = slen;
            if copy_len >= buflen {
                copy_len = buflen - 1;
            }
            copy_len = copy_len - e + 1;
            if copy_len < 1 {
                *buf.offset((e - 1) as isize) = 0;
            } else {
                std::ptr::copy(
                    s.offset(e as isize),
                    buf.offset(e as isize),
                    copy_len as usize,
                );
            }
        }
    } else if e + 3 < buflen {
        // Set the middle "..." and copy the last part
        *buf.offset(e as isize) = b'.' as c_char;
        *buf.offset((e + 1) as isize) = b'.' as c_char;
        *buf.offset((e + 2) as isize) = b'.' as c_char;

        // Calculate remaining length
        let mut remaining = slen - i + 1;
        if remaining >= buflen - e - 3 {
            remaining = buflen - e - 3 - 1;
        }
        std::ptr::copy(
            s.offset(i as isize),
            buf.offset((e + 3) as isize),
            remaining as usize,
        );
        *buf.offset((e + 3 + remaining - 1) as isize) = 0;
    } else {
        // Can't fit in the "...", just truncate
        *buf.offset((buflen - 1) as isize) = 0;
    }
}

/// Truncate a message string if it would cause a hit-return prompt.
///
/// Returns NULL if no truncation is needed, or an allocated truncated string.
/// The returned string must be freed by the caller using `rs_msg_free_trunc`.
///
/// # Arguments
/// * `s` - The string to potentially truncate
/// * `force` - If true, always truncate regardless of message settings
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
/// - Returned pointer (if not NULL) must be freed with `rs_msg_free_trunc`
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_msg_strtrunc(s: *const c_char, force: c_int) -> *mut c_char {
    // Check conditions for truncation
    let should_truncate = force != 0
        || (nvim_get_msg_scroll() == 0
            && nvim_get_need_wait_return() == 0
            && nvim_shortmess(SHM_TRUNCALL) != 0
            && !nvim_get_exmode_active()
            && nvim_get_msg_silent() == 0
            && nvim_ui_has_messages() == 0);

    if !should_truncate {
        return std::ptr::null_mut();
    }

    let len = nvim_vim_strsize(s);
    let msg_scrolled = nvim_get_msg_scrolled();
    let rows = nvim_get_rows();
    let msg_row = nvim_get_msg_row();
    let columns = nvim_get_columns();
    let sc_col = nvim_get_sc_col();

    let room = if msg_scrolled != 0 {
        // Use all the columns
        (rows - msg_row) * columns - 1
    } else {
        // Use up to 'showcmd' column
        (rows - msg_row - 1) * columns + sc_col - 1
    };

    if len > room && room > 0 {
        // May have up to 18 bytes per cell (6 per char, up to two composing chars)
        let buf_size = ((room + 2) * 18) as usize;
        let buf = xmalloc(buf_size);
        if buf.is_null() {
            return std::ptr::null_mut();
        }
        rs_trunc_string(s, buf, room, buf_size as c_int);
        buf
    } else {
        std::ptr::null_mut()
    }
}

/// Free a truncated string allocated by `rs_msg_strtrunc`.
///
/// # Safety
/// - `ptr` must be NULL or a pointer returned by `rs_msg_strtrunc`
#[no_mangle]
pub unsafe extern "C" fn rs_msg_free_trunc(ptr: *mut c_char) {
    if !ptr.is_null() {
        nvim_xfree(ptr);
    }
}

// ============================================================================
// Output Translation Functions
// ============================================================================

/// Output a string with unprintable character translation.
///
/// Outputs characters, translating unprintable ones to their visible form
/// (like ^I for tab, <80> for special chars, etc).
///
/// # Arguments
/// * `str` - The string to output (NUL-terminated)
/// * `hl_id` - Highlight group ID (0 for default)
/// * `hist` - If true, add to message history
///
/// # Returns
/// Number of screen cells used
///
/// # Safety
/// - `str` must be a valid NUL-terminated C string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_outtrans(str: *const c_char, hl_id: c_int, hist: c_int) -> c_int {
    msg_outtrans(str, hl_id, hist)
}

/// Output a string with length and unprintable character translation.
///
/// Like `rs_msg_outtrans` but takes an explicit length.
///
/// # Arguments
/// * `msgstr` - The string to output
/// * `len` - Length in bytes (-1 for NUL-terminated)
/// * `hl_id` - Highlight group ID (0 for default)
/// * `hist` - If true, add to message history
///
/// # Returns
/// Number of screen cells used
///
/// # Safety
/// - `msgstr` must be a valid string of at least `len` bytes
#[no_mangle]
pub unsafe extern "C" fn rs_msg_outtrans_len(
    msgstr: *const c_char,
    len: c_int,
    hl_id: c_int,
    hist: c_int,
) -> c_int {
    msg_outtrans_len(msgstr, len, hl_id, hist)
}

/// Output one character at position and return pointer to next.
///
/// Handles multi-byte characters correctly.
///
/// # Arguments
/// * `p` - Pointer to the character
/// * `hl_id` - Highlight group ID
/// * `hist` - If true, add to message history
///
/// # Returns
/// Pointer to the next character in the string
///
/// # Safety
/// - `p` must be a valid pointer into a string
#[no_mangle]
pub unsafe extern "C" fn rs_msg_outtrans_one(
    p: *const c_char,
    hl_id: c_int,
    hist: c_int,
) -> *const c_char {
    msg_outtrans_one(p, hl_id, hist)
}

/// Output a string showing special keys in <> form.
///
/// Used for displaying mappings. K_SPECIAL sequences are shown as <F1>,
/// <S-Up>, etc. Unprintable characters are also shown in <> form.
///
/// # Arguments
/// * `strstart` - The string to output (NUL-terminated)
/// * `from` - True for LHS of a mapping (shows space as <Space>)
/// * `maxlen` - Maximum screen columns (0 for unlimited)
///
/// # Returns
/// Number of screen cells used
///
/// # Safety
/// - `strstart` must be a valid NUL-terminated C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_msg_outtrans_special(
    strstart: *const c_char,
    from: c_int,
    maxlen: c_int,
) -> c_int {
    msg_outtrans_special(strstart, from, maxlen)
}

// ============================================================================
// String Conversion Utilities
// ============================================================================

/// Check if truncation should be applied based on current state.
///
/// This implements the logic from msg_strtrunc to determine if
/// truncation is appropriate without allocating a buffer.
///
/// # Returns
/// * 1 if truncation should be applied
/// * 0 if message should be shown in full
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_should_trunc_impl() -> c_int {
    let msg_scroll = nvim_get_msg_scroll() != 0;
    let need_wait_return = nvim_get_need_wait_return() != 0;
    let has_truncall = nvim_shortmess(SHM_TRUNCALL) != 0;
    let exmode_active = nvim_get_exmode_active();
    let msg_silent = nvim_get_msg_silent();
    let ui_has_messages = nvim_ui_has_messages() != 0;

    c_int::from(
        !msg_scroll
            && !need_wait_return
            && has_truncall
            && !exmode_active
            && msg_silent == 0
            && !ui_has_messages,
    )
}

/// Calculate if string needs truncation and return room available.
///
/// # Arguments
/// * `str_width` - Width of the string in cells
///
/// # Returns
/// * Available room if truncation needed (positive)
/// * 0 if no truncation needed
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_calc_trunc_room(str_width: c_int) -> c_int {
    let room = rs_msg_room();
    if str_width > room && room > 0 {
        room
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trunc_bufsize() {
        assert_eq!(rs_msg_trunc_bufsize(0), 36); // (0+2)*18
        assert_eq!(rs_msg_trunc_bufsize(10), 216); // (10+2)*18
        assert_eq!(rs_msg_trunc_bufsize(50), 936); // (50+2)*18
    }

    #[test]
    fn test_trunc_room() {
        assert_eq!(rs_msg_trunc_room(0), 0);
        assert_eq!(rs_msg_trunc_room(3), 0);
        assert_eq!(rs_msg_trunc_room(10), 7); // 10-3
        assert_eq!(rs_msg_trunc_room(50), 47); // 50-3
    }

    #[test]
    fn test_trunc_half() {
        assert_eq!(rs_msg_trunc_half(0), 0);
        assert_eq!(rs_msg_trunc_half(3), 0);
        assert_eq!(rs_msg_trunc_half(10), 3); // (10-3)/2 = 3
        assert_eq!(rs_msg_trunc_half(50), 23); // (50-3)/2 = 23
    }

    #[test]
    fn test_msg_fits() {
        assert_eq!(rs_msg_fits(10, 20), 1);
        assert_eq!(rs_msg_fits(20, 20), 1);
        assert_eq!(rs_msg_fits(21, 20), 0);
    }

    #[test]
    fn test_outtrans_long_should_trunc() {
        // len > room and room >= 20
        assert_eq!(rs_msg_outtrans_long_should_trunc(30, 25), 1);
        assert_eq!(rs_msg_outtrans_long_should_trunc(30, 20), 1);
        // room < 20
        assert_eq!(rs_msg_outtrans_long_should_trunc(30, 19), 0);
        // len <= room
        assert_eq!(rs_msg_outtrans_long_should_trunc(20, 25), 0);
    }

    #[test]
    fn test_outtrans_long_slen() {
        assert_eq!(rs_msg_outtrans_long_slen(20), 8); // (20-3)/2
        assert_eq!(rs_msg_outtrans_long_slen(50), 23); // (50-3)/2
    }

    #[test]
    fn test_shortmess_constants() {
        assert_eq!(SHM_TRUNCALL, c_int::from(b'T'));
        assert_eq!(SHM_TRUNC, c_int::from(b't'));
        assert_eq!(SHM_OVERALL, c_int::from(b'O'));
        assert_eq!(SHM_LAST, c_int::from(b'l'));
        assert_eq!(SHM_INTRO, c_int::from(b'I'));
    }
}
