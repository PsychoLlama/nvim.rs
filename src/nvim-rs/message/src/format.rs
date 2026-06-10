//! Message formatting utilities
//!
//! Provides Rust implementations for message formatting operations
//! including string truncation, column calculation, and attribute handling.

use std::ffi::{c_char, c_int};

// C accessor declarations
extern "C" {
    static Rows: c_int;
    static Columns: c_int;
    static mut msg_silent: c_int;
    /// Get `msg_col` global
    static mut msg_col: c_int;
    /// Set `msg_col` global
    /// Get `msg_row` global
    static mut msg_row: c_int;
    /// Set `msg_row` global
    /// Get `Rows` global (screen rows)
    /// Get `Columns` global (screen columns)
    /// Get `msg_scrolled` global
    static mut msg_scrolled: c_int;
    /// Get `sc_col` global (showcmd column)
    static mut sc_col: c_int;
    /// Get `msg_scroll` flag
    static mut msg_scroll: c_int;
    /// Get `need_wait_return` flag
    static mut need_wait_return: bool;
    /// Check shortmess option flag
    fn shortmess(flag: c_int) -> bool;
    /// Get the 'shortmess' option string value
    static mut p_shm: *mut c_char;
    /// Search for character in string (returns pointer or NULL)
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    /// Get `exmode_active` flag
    static mut exmode_active: bool;
    /// Check if UI has messages capability
    fn ui_has(ext: c_int) -> bool;
    /// Calculate string width in cells
    fn vim_strsize(s: *const c_char) -> c_int;
    /// Get `cmdline_row` global
    static mut cmdline_row: c_int;
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

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

/// Get the current message column.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_col() -> c_int {
    msg_col
}

/// Set the current message column.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_col(col: c_int) {
    msg_col = col;
}

/// Get the current message row.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_row() -> c_int {
    msg_row
}

/// Set the current message row.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_row(row: c_int) {
    msg_row = row;
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
    let rows = Rows;
    let columns = Columns;
    let scrolled = msg_scrolled;

    if scrolled != 0 {
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
    let scroll = msg_scroll != 0;
    let has_truncall = shortmess(SHM_TRUNCALL);
    let ui_has_messages = ui_has(K_UI_MESSAGES);

    c_int::from(
        !scroll
            && !need_wait_return
            && has_truncall
            && !exmode_active
            && msg_silent == 0
            && !ui_has_messages,
    )
}

/// Characters abbreviated by the 'a' flag in 'shortmess'.
/// Matches SHM_ALL_ABBREVIATIONS in option_vars.h: SHM_RO='r', SHM_MOD='m', SHM_LINES='l', SHM_WRI='w'.
const SHM_ALL_ABBREVIATIONS: &[u8] = b"rmlw\0";

/// Check if character `x` is present in 'shortmess' option.
///
/// Returns 1 (true) if `x` is in `p_shm`, or if 'a' is in `p_shm` and `x`
/// is one of the abbreviated chars (`SHM_ALL_ABBREVIATIONS`).
///
/// # Safety
/// Calls C accessor functions.
#[allow(clippy::must_use_candidate)]
#[export_name = "shortmess"]
pub unsafe extern "C" fn rs_shortmess(x: c_int) -> c_int {
    let shm = p_shm.cast_const();
    if shm.is_null() {
        return 0;
    }
    if !vim_strchr(shm, x).is_null() {
        return 1;
    }
    // Check if 'a' flag enables abbreviation for x
    if !vim_strchr(shm, c_int::from(b'a')).is_null()
        && !vim_strchr(SHM_ALL_ABBREVIATIONS.as_ptr().cast(), x).is_null()
    {
        return 1;
    }
    0
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
    vim_strsize(s)
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

    let width = vim_strsize(s);
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
    Columns
}

/// Get the number of screen rows.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_screen_rows() -> c_int {
    Rows
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
    let mut len: c_int = 0;
    let mut cells: c_int = 0;
    let mut p = s;
    while *p.cast::<u8>() != 0 && cells < width {
        let char_cells = rs_ptr2cells(p);
        if cells + char_cells > width {
            break;
        }
        cells += char_cells;
        let char_len = utfc_ptr2len(p);
        len += char_len;
        p = p.add(char_len.unsigned_abs() as usize);
    }
    len
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
    cmdline_row
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
    Columns - msg_col
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
    msg_silent
}

/// Check if message is silent (non-zero).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_is_msg_silent() -> c_int {
    c_int::from(msg_silent != 0)
}

/// Clamp column to valid range.
///
/// Ensures column is between 0 and Columns - 1.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_clamp_col(col: c_int) -> c_int {
    let columns = Columns;
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
    #[link_name = "ptr2cells"]
    fn rs_ptr2cells(p: *const c_char) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;
    fn utf_head_off(base: *const c_char, p: *const c_char) -> c_int;
    fn utf_ptr2cells(p: *const c_char) -> c_int;
    fn utf_ptr2char(p: *const c_char) -> c_int;
    fn utfc_ptr2len_len(p: *const c_char, maxlen: c_int) -> c_int;
    fn vim_isprintc(c: c_int) -> bool;
    fn char2cells(c: c_int) -> c_int;
    // transchar_buf(NULL, c) returns a static buffer
    fn transchar_buf(buf: *const std::ffi::c_void, c: c_int) -> *mut c_char;

    // Character translation
    fn transchar_byte_buf(buf: *mut c_char, c: c_int) -> *mut c_char;
    fn msg_puts_hl(s: *const c_char, hl_id: c_int, hist: bool);
    fn msg_puts_len(s: *const c_char, len: isize, hl_id: c_int, hist: bool);

    // Memory management
    fn xmalloc(size: usize) -> *mut c_char;
    fn xfree(ptr: *mut std::ffi::c_void);

    // History management
    fn msg_hist_add(s: *const c_char, len: c_int, hl_id: c_int);
    static mut msg_hist_off: bool;
    // msg() — #[export_name = "msg"] is in output_core.rs, callable via C name
    fn msg(s: *const c_char, hl_id: c_int) -> bool;

    static mut got_int: bool;
    // clear_cmdline, mode_displayed (normal_shim.c)
    static mut clear_cmdline: bool;
    static mut mode_displayed: bool;
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
#[export_name = "trunc_string"]
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
        let char_len = utfc_ptr2len(s.offset(e as isize));
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
        let head_off = utf_head_off(s, s.offset((half_end - 1) as isize));
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
#[export_name = "msg_strtrunc"]
#[must_use]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_msg_strtrunc(s: *const c_char, force: c_int) -> *mut c_char {
    // Check conditions for truncation
    let should_truncate = force != 0
        || (msg_scroll == 0
            && !need_wait_return
            && shortmess(SHM_TRUNCALL)
            && !exmode_active
            && msg_silent == 0
            && !ui_has(K_UI_MESSAGES));

    if !should_truncate {
        return std::ptr::null_mut();
    }

    let len = vim_strsize(s);
    let scrolled = msg_scrolled;
    let rows = Rows;
    let columns = Columns;

    let room = if scrolled != 0 {
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
        xfree(ptr.cast());
    }
}

// ============================================================================
// Output Translation Functions
// ============================================================================

/// HLF_8 = 1 (special-char highlight, enum position 1 in hlf_T)
const HLF_8: c_int = 1;

/// Output a string with length and unprintable character translation.
///
/// Translates unprintable characters to their visible form and outputs them.
/// Returns the number of screen cells used.
///
/// # Arguments
/// * `msgstr` - The string to output
/// * `len` - Length in bytes
/// * `hl_id` - Highlight group ID (0 for default)
/// * `hist` - If true, add to message history
///
/// # Safety
/// - `msgstr` must be a valid string of at least `len` bytes
#[export_name = "msg_outtrans_len"]
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_msg_outtrans_len(
    msgstr: *const c_char,
    len: c_int,
    hl_id: c_int,
    hist: bool,
) -> c_int {
    let mut retval: c_int = 0;
    let mut str = msgstr;
    let mut plain_start = msgstr;
    let mut remaining = len;

    let save_got_int = unsafe { got_int };
    // Only quit when got_int was set in here.
    unsafe {
        got_int = false;
    }

    if hist {
        msg_hist_add(str, len, hl_id);
    }

    // When drawing over the command line no need to clear it later or remove
    // the mode message.
    if msg_silent == 0 && len > 0 && msg_row >= cmdline_row && msg_col == 0 {
        clear_cmdline = false;
        mode_displayed = false;
    }

    // Go over the string. Special characters are translated and printed.
    // Normal characters are printed several at a time.
    while remaining > 0 && !unsafe { got_int } {
        remaining -= 1;
        // Don't include composing chars after the end.
        let mb_l = utfc_ptr2len_len(str, remaining + 1);
        if mb_l > 1 {
            let c = utf_ptr2char(str);
            if vim_isprintc(c) {
                // Printable multi-byte char: count the cells.
                retval += utf_ptr2cells(str);
            } else {
                // Unprintable multi-byte char: print the printable chars so
                // far and the translation of the unprintable char.
                if str > plain_start {
                    msg_puts_len(
                        plain_start,
                        (str as usize - plain_start as usize) as isize,
                        hl_id,
                        hist,
                    );
                }
                plain_start = str.offset(mb_l as isize);
                msg_puts_hl(
                    transchar_buf(std::ptr::null(), c).cast_const(),
                    if hl_id == 0 { HLF_8 } else { hl_id },
                    false,
                );
                retval += char2cells(c);
            }
            remaining -= mb_l - 1;
            str = str.offset(mb_l as isize);
        } else {
            let s = transchar_byte_buf(std::ptr::null_mut(), c_int::from(*str as u8));
            if *s.offset(1) != 0 {
                // Unprintable char: print the printable chars so far and the
                // translation of the unprintable char.
                if str > plain_start {
                    msg_puts_len(
                        plain_start,
                        (str as usize - plain_start as usize) as isize,
                        hl_id,
                        hist,
                    );
                }
                plain_start = str.offset(1);
                msg_puts_hl(
                    s.cast_const(),
                    if hl_id == 0 { HLF_8 } else { hl_id },
                    false,
                );
                // count translated bytes (like strlen(s))
                let mut sp = s;
                while *sp != 0 {
                    retval += 1;
                    sp = sp.offset(1);
                }
            } else {
                retval += 1;
            }
            str = str.offset(1);
        }
    }

    if (str > plain_start || plain_start == msgstr) && !unsafe { got_int } {
        // Print the printable chars at the end (or emit empty string).
        msg_puts_len(
            plain_start,
            (str as usize - plain_start as usize) as isize,
            hl_id,
            hist,
        );
    }

    unsafe {
        got_int |= save_got_int;
    }

    retval
}

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
#[export_name = "msg_outtrans"]
#[must_use]
pub unsafe extern "C" fn rs_msg_outtrans(str: *const c_char, hl_id: c_int, hist: bool) -> c_int {
    if *str == 0 {
        return 0;
    }
    let mut len: c_int = 0;
    let mut p = str;
    while *p != 0 {
        len += 1;
        p = p.offset(1);
    }
    rs_msg_outtrans_len(str, len, hl_id, hist)
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
#[export_name = "msg_outtrans_one"]
#[must_use]
pub unsafe extern "C" fn rs_msg_outtrans_one(
    p: *const c_char,
    hl_id: c_int,
    hist: bool,
) -> *const c_char {
    let l = utfc_ptr2len(p);
    if l > 1 {
        let _ = rs_msg_outtrans_len(p, l, hl_id, hist);
        return p.offset(l as isize);
    }
    // Single byte: translate and output
    #[allow(clippy::cast_sign_loss)]
    let byte = (*p) as u8;
    let translated = transchar_byte_buf(std::ptr::null_mut(), c_int::from(byte));
    msg_puts_hl(translated.cast_const(), hl_id, hist);
    p.offset(1)
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
#[export_name = "msg_outtrans_special"]
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_msg_outtrans_special(
    strstart: *const c_char,
    from: c_int,
    maxlen: c_int,
) -> c_int {
    use crate::rs_str2special;

    if strstart.is_null() {
        return 0;
    }

    let mut str = strstart;
    let mut retval: c_int = 0;

    while *str != 0 {
        let text: *const c_char;
        // Leading and trailing spaces need to be displayed in <> form.
        if (str == strstart || *str.add(1) == 0) && *str == b' ' as c_char {
            text = c"<Space>".as_ptr();
            str = str.add(1);
        } else {
            text = rs_str2special(std::ptr::addr_of_mut!(str), from != 0, false);
        }

        // Single-byte: translate via transchar_byte_buf for display
        let text = if *text != 0 && *text.add(1) == 0 {
            transchar_byte_buf(std::ptr::null_mut(), c_int::from(*text as u8)).cast_const()
        } else {
            text
        };

        let len = vim_strsize(text);
        if maxlen > 0 && retval + len >= maxlen {
            break;
        }
        // Highlight special keys (multi-cell but not multi-byte → HLF_8)
        let hl = if len > 1 && utfc_ptr2len(text) <= 1 {
            HLF_8
        } else {
            0
        };
        msg_puts_hl(text, hl, false);
        retval += len;
    }
    retval
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
    let scroll = msg_scroll != 0;
    let has_truncall = shortmess(SHM_TRUNCALL);
    let ui_has_messages = ui_has(K_UI_MESSAGES);

    c_int::from(
        !scroll
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

// ============================================================================
// Phase 76: msg_may_trunc and msg_trunc
// ============================================================================

/// Truncate message at the start with '<' for filename messages.
///
/// Checks if `s` is too long for the current message area and, if so,
/// truncates at the start by replacing a character with '<'.
///
/// # Arguments
/// * `force` - If true, force truncation even without 'shortmess' 't' flag
/// * `s` - Mutable pointer to the message string (may be modified in-place)
///
/// # Returns
/// Pointer into `s` (possibly offset) at the truncation point.
///
/// # Safety
/// - `s` must be a valid NUL-terminated mutable C string
#[export_name = "msg_may_trunc"]
#[must_use]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_msg_may_trunc(force: bool, s: *mut c_char) -> *mut c_char {
    if ui_has(K_UI_MESSAGES) {
        return s;
    }

    let rows = Rows;
    let crow = cmdline_row;
    let columns = Columns;
    let room = (rows - crow - 1) * columns + sc_col - 1;

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let len = libc_strlen(s.cast_const()) as c_int;
    if (force || (shortmess(SHM_TRUNC) && !exmode_active)) && len - room > 0 {
        let size = vim_strsize(s.cast_const());
        if size <= room {
            return s;
        }
        let mut sz = size;
        let mut n: c_int = 0;
        while sz >= room {
            sz -= utf_ptr2cells(s.offset(n as isize));
            n += utfc_ptr2len(s.offset(n as isize));
        }
        n -= 1;
        let p = s.offset(n as isize);
        #[allow(clippy::cast_possible_wrap)]
        let lt = b'<' as c_char;
        *p = lt;
        return p;
    }
    s
}

/// Like msg(), but truncate to a single line if p_shm contains 't', or when
/// `force` is true. Also adds the message to history.
///
/// # Arguments
/// * `s` - The message string (may be modified in-place by truncation)
/// * `force` - If true, force truncation
/// * `hl_id` - Highlight group ID
///
/// # Returns
/// Pointer to the printed message if wait_return() was not called, else NULL.
///
/// # Safety
/// - `s` must be a valid NUL-terminated mutable C string
#[export_name = "msg_trunc"]
pub unsafe extern "C" fn rs_msg_trunc(s: *mut c_char, force: bool, hl_id: c_int) -> *mut c_char {
    // Add message to history before truncating.
    msg_hist_add(s.cast_const(), -1, hl_id);

    let ts = rs_msg_may_trunc(force, s);

    msg_hist_off = true;
    let n = msg(ts.cast_const(), hl_id);
    msg_hist_off = false;

    if n {
        ts
    } else {
        std::ptr::null_mut()
    }
}

/// Calculate strlen of a C string (wrapper to avoid libc dep).
///
/// # Safety
/// `s` must be a valid NUL-terminated C string.
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.offset(1);
    }
    (p as usize) - (s as usize)
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
