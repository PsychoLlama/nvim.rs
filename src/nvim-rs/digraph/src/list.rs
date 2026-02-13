//! Digraph list operations.
//!
//! This module provides Rust implementations for building digraph lists,
//! used by `digraph_getlist()` Vimscript function and `:digraphs` listing.

use std::ffi::{c_char, c_void};

use libc::c_int;

use crate::data::{
    DG_START_ARABIC, DG_START_ARROWS, DG_START_BLOCK, DG_START_BOPOMOFO, DG_START_CJK_SYMBOLS,
    DG_START_CURRENCY, DG_START_CYRILLIC, DG_START_DINGBATS, DG_START_DRAWING, DG_START_GREEK,
    DG_START_GREEK_EXTENDED, DG_START_HEBREW, DG_START_HIRAGANA, DG_START_KATAKANA, DG_START_LATIN,
    DG_START_LATIN_EXTENDED, DG_START_MATH, DG_START_OTHER1, DG_START_OTHER2, DG_START_OTHER3,
    DG_START_PUNCTUATION, DG_START_ROMAN, DG_START_SHAPES, DG_START_SUB_SUPER, DG_START_SYMBOLS,
    DG_START_TECHNICAL, DIGRAPH_DEFAULT,
};
use crate::DigrT;

// C accessor functions
extern "C" {
    /// Get pointer to user digraphs array data.
    fn nvim_get_user_digraphs_data() -> *const c_void;

    /// Get length of user digraphs array.
    fn nvim_get_user_digraphs_len() -> c_int;

    /// Get exact digraph match.
    fn rs_getexactdigraph(char1: c_int, char2: c_int, meta_char: c_int) -> c_int;

    /// Convert character to UTF-8.
    fn rs_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;

    /// Check if a character is a composing character that should be displayed
    /// with a preceding space.
    fn nvim_utf_iscomposing_first(c: c_int) -> c_int;

    /// Get display width of a character in cells.
    fn nvim_char2cells(c: c_int) -> c_int;

    /// Put a character to the message output.
    fn msg_putchar(c: c_int);

    /// Output a translated string with highlighting.
    fn msg_outtrans(s: *const c_char, hl_id: c_int, hist: bool) -> c_int;

    /// Set the message extension kind.
    fn msg_ext_set_kind(kind: *const c_char);

    /// Translate a string via gettext.
    fn nvim_gettext(s: *const c_char) -> *const c_char;

    /// Get the current message column.
    fn nvim_get_msg_col() -> c_int;

    /// Get the screen width (Columns).
    fn nvim_get_Columns() -> c_int;
}

/// Highlight group for special keys text (`HLF_8` = 1).
const HLF_8: c_int = 1;

/// Highlight group for mode message (`HLF_CM` = 11).
const HLF_CM: c_int = 11;

/// Header entries for digraph categories.
/// Each entry has `(start_code, header_index)`.
/// The `header_index` corresponds to predefined header strings.
static HEADER_TABLE: &[(c_int, c_int)] = &[
    (DG_START_LATIN, 0),          // "Latin supplement"
    (DG_START_GREEK, 1),          // "Greek and Coptic"
    (DG_START_CYRILLIC, 2),       // "Cyrillic"
    (DG_START_HEBREW, 3),         // "Hebrew"
    (DG_START_ARABIC, 4),         // "Arabic"
    (DG_START_LATIN_EXTENDED, 5), // "Latin extended"
    (DG_START_GREEK_EXTENDED, 6), // "Greek extended"
    (DG_START_PUNCTUATION, 7),    // "Punctuation"
    (DG_START_SUB_SUPER, 8),      // "Super- and subscripts"
    (DG_START_CURRENCY, 9),       // "Currency"
    (DG_START_OTHER1, 10),        // "Other"
    (DG_START_ROMAN, 11),         // "Roman numbers"
    (DG_START_ARROWS, 12),        // "Arrows"
    (DG_START_MATH, 13),          // "Mathematical operators"
    (DG_START_TECHNICAL, 14),     // "Technical"
    (DG_START_OTHER2, 15),        // "Other"
    (DG_START_DRAWING, 16),       // "Box drawing"
    (DG_START_BLOCK, 17),         // "Block elements"
    (DG_START_SHAPES, 18),        // "Geometric shapes"
    (DG_START_SYMBOLS, 19),       // "Symbols"
    (DG_START_DINGBATS, 20),      // "Dingbats"
    (DG_START_CJK_SYMBOLS, 21),   // "CJK symbols and punctuation"
    (DG_START_HIRAGANA, 22),      // "Hiragana"
    (DG_START_KATAKANA, 23),      // "Katakana"
    (DG_START_BOPOMOFO, 24),      // "Bopomofo"
    (DG_START_OTHER3, 25),        // "Other"
    (0x0fff_ffff, -1),            // Sentinel
];

/// Translatable header strings for digraph categories.
/// Index matches the `header_index` in `HEADER_TABLE`.
static HEADER_STRINGS: &[&std::ffi::CStr] = &[
    c"Latin supplement",
    c"Greek and Coptic",
    c"Cyrillic",
    c"Hebrew",
    c"Arabic",
    c"Latin extended",
    c"Greek extended",
    c"Punctuation",
    c"Super- and subscripts",
    c"Currency",
    c"Other",
    c"Roman numbers",
    c"Arrows",
    c"Mathematical operators",
    c"Technical",
    c"Other",
    c"Box drawing",
    c"Block elements",
    c"Geometric shapes",
    c"Symbols",
    c"Dingbats",
    c"CJK symbols and punctuation",
    c"Hiragana",
    c"Katakana",
    c"Bopomofo",
    c"Other",
];

/// Find the header index for a digraph result code.
///
/// Given the previous result and current result, determines if a new
/// header section should be displayed.
///
/// # Arguments
/// * `previous` - Previous result code (or value < 0 for no previous)
/// * `current` - Current result code
///
/// # Returns
/// Header index (0-25) if a new header should be shown, -1 otherwise.
#[no_mangle]
pub extern "C" fn rs_digraph_get_header_index(previous: c_int, current: c_int) -> c_int {
    for i in 0..HEADER_TABLE.len() - 1 {
        let (start, idx) = HEADER_TABLE[i];
        let next_start = HEADER_TABLE[i + 1].0;

        if previous < start && current >= start && current < next_start {
            return idx;
        }
    }
    -1
}

/// Format a digraph entry for display.
///
/// Formats a digraph as "{c1}{c2} {result} {decimal}".
///
/// # Arguments
/// * `char1` - First character of digraph
/// * `char2` - Second character of digraph
/// * `result` - Result character code
/// * `buf` - Output buffer (at least 32 bytes)
/// * `buf_len` - Length of buffer
///
/// # Returns
/// Number of bytes written (not including NUL).
///
/// # Safety
/// `buf` must point to at least `buf_len` writable bytes.
#[no_mangle]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_digraph_format_entry(
    char1: u8,
    char2: u8,
    result: c_int,
    buf: *mut c_char,
    buf_len: c_int,
) -> c_int {
    if buf.is_null() || buf_len < 16 {
        return 0;
    }

    let mut pos = 0usize;
    let len = buf_len as usize;

    // Write "{c1}{c2} "
    unsafe {
        *buf.add(pos) = char1 as c_char;
        pos += 1;
        *buf.add(pos) = char2 as c_char;
        pos += 1;
        *buf.add(pos) = b' ' as c_char;
        pos += 1;
    }

    // Check if result needs a leading space for composing character
    let is_composing = unsafe { nvim_utf_iscomposing_first(result) } != 0;
    if is_composing && pos < len - 10 {
        unsafe {
            *buf.add(pos) = b' ' as c_char;
        }
        pos += 1;
    }

    // Write UTF-8 result character
    let utf8_len = unsafe { rs_utf_char2bytes(result, buf.add(pos)) };
    if utf8_len > 0 {
        pos += utf8_len as usize;
    }

    // Check cell width and add padding space if single-width
    let cells = unsafe { nvim_char2cells(result) };
    if cells == 1 && pos < len - 8 {
        unsafe {
            *buf.add(pos) = b' ' as c_char;
        }
        pos += 1;
    }

    // Write decimal value " %3d"
    if pos < len - 6 {
        unsafe {
            *buf.add(pos) = b' ' as c_char;
        }
        pos += 1;

        // Format the decimal number (up to 7 digits for Unicode)
        let mut num_buf = [0u8; 8];
        let num_str = format_decimal(result, &mut num_buf);
        // Right-align in 3 characters minimum
        let padding = if num_str.len() < 3 {
            3 - num_str.len()
        } else {
            0
        };
        for _ in 0..padding {
            if pos < len - 1 {
                unsafe {
                    *buf.add(pos) = b' ' as c_char;
                }
                pos += 1;
            }
        }
        for &b in num_str {
            if pos < len - 1 {
                unsafe {
                    *buf.add(pos) = b as c_char;
                }
                pos += 1;
            }
        }
    }

    // NUL terminate
    unsafe {
        *buf.add(pos) = 0;
    }

    #[allow(clippy::cast_possible_truncation)]
    {
        pos as c_int
    }
}

/// Format a decimal number into a buffer.
fn format_decimal(mut n: c_int, buf: &mut [u8; 8]) -> &[u8] {
    if n == 0 {
        buf[0] = b'0';
        return &buf[0..1];
    }

    let negative = n < 0;
    if negative {
        n = -n;
    }

    let mut pos = buf.len();
    while n > 0 && pos > 0 {
        pos -= 1;
        #[allow(clippy::cast_sign_loss)]
        {
            buf[pos] = b'0' + (n % 10) as u8;
        }
        n /= 10;
    }

    if negative && pos > 0 {
        pos -= 1;
        buf[pos] = b'-';
    }

    &buf[pos..]
}

// C accessor for interrupt checking
extern "C" {
    /// Check if user pressed Ctrl-C (`got_int`).
    fn nvim_digraph_got_int() -> c_int;

    /// Fast check for user interrupt.
    fn nvim_digraph_fast_breakcheck();
}

/// Callback type for iterating over digraphs.
///
/// Called for each digraph with:
/// - `char1`, `char2`: The digraph input characters
/// - `result`: The digraph result character
/// - `ctx`: User context pointer
///
/// Return `true` to continue iteration, `false` to stop.
pub type DigraphIterCallback =
    unsafe extern "C" fn(char1: u8, char2: u8, result: c_int, ctx: *mut c_void) -> c_int;

/// Iterate over all digraphs (user and default).
///
/// Calls the callback for each digraph. Default digraphs are iterated first,
/// then user digraphs.
///
/// # Arguments
/// * `list_all` - If true, include default digraphs; if false, only user digraphs
/// * `callback` - Function called for each digraph
/// * `ctx` - User context passed to callback
///
/// # Returns
/// Number of digraphs iterated.
///
/// # Safety
/// Callback must be a valid function pointer, ctx can be null.
#[no_mangle]
pub unsafe extern "C" fn rs_digraph_iterate(
    list_all: c_int,
    callback: DigraphIterCallback,
    ctx: *mut c_void,
) -> c_int {
    let mut count = 0;

    // Iterate default digraphs first (if requested)
    if list_all != 0 {
        for dp in DIGRAPH_DEFAULT {
            // Get actual result (may be overridden by user digraph)
            let result =
                unsafe { rs_getexactdigraph(c_int::from(dp.char1), c_int::from(dp.char2), 0) };

            // Skip if result is 0 or same as char2 (no digraph)
            if result != 0 && result != c_int::from(dp.char2) {
                let should_continue = unsafe { callback(dp.char1, dp.char2, result, ctx) };
                if should_continue == 0 {
                    return count;
                }
                count += 1;
            }
        }
    }

    // Iterate user digraphs
    let user_data = unsafe { nvim_get_user_digraphs_data() };
    let user_len = unsafe { nvim_get_user_digraphs_len() };

    if !user_data.is_null() && user_len > 0 {
        let user_digraphs = user_data.cast::<DigrT>();
        #[allow(clippy::cast_sign_loss)]
        let len = user_len as usize;

        for i in 0..len {
            let dp = unsafe { &*user_digraphs.add(i) };
            let should_continue = unsafe { callback(dp.char1, dp.char2, dp.result, ctx) };
            if should_continue == 0 {
                return count;
            }
            count += 1;
        }
    }

    count
}

/// Iterate over default digraphs with interrupt checking.
///
/// Calls the callback for each default digraph, checking for user interrupt
/// between each call.
///
/// # Arguments
/// * `callback` - Function called for each digraph
/// * `ctx` - User context passed to callback
///
/// # Returns
/// * 1 if iteration completed successfully
/// * 0 if interrupted by user (`got_int`)
///
/// # Safety
/// Callback must be a valid function pointer, ctx can be null.
#[no_mangle]
pub unsafe extern "C" fn rs_digraph_iterate_default(
    callback: DigraphIterCallback,
    ctx: *mut c_void,
) -> c_int {
    for dp in DIGRAPH_DEFAULT {
        // Check for user interrupt
        if unsafe { nvim_digraph_got_int() } != 0 {
            return 0;
        }

        // Get actual result (may be overridden by user digraph)
        let result = unsafe { rs_getexactdigraph(c_int::from(dp.char1), c_int::from(dp.char2), 0) };

        // Skip if result is 0 or same as char2 (no digraph)
        if result != 0 && result != c_int::from(dp.char2) {
            let should_continue = unsafe { callback(dp.char1, dp.char2, result, ctx) };
            if should_continue == 0 {
                return 0;
            }
        }

        // Check for breakcheck
        unsafe { nvim_digraph_fast_breakcheck() };
    }

    1
}

/// Iterate over user digraphs with interrupt checking.
///
/// Calls the callback for each user digraph, checking for user interrupt
/// between each call.
///
/// # Arguments
/// * `callback` - Function called for each digraph
/// * `ctx` - User context passed to callback
///
/// # Returns
/// * 1 if iteration completed successfully
/// * 0 if interrupted by user (`got_int`)
///
/// # Safety
/// Callback must be a valid function pointer, ctx can be null.
#[no_mangle]
pub unsafe extern "C" fn rs_digraph_iterate_user(
    callback: DigraphIterCallback,
    ctx: *mut c_void,
) -> c_int {
    let user_data = unsafe { nvim_get_user_digraphs_data() };
    let user_len = unsafe { nvim_get_user_digraphs_len() };

    if user_data.is_null() || user_len <= 0 {
        return 1;
    }

    let user_digraphs = user_data.cast::<DigrT>();
    #[allow(clippy::cast_sign_loss)]
    let len = user_len as usize;

    for i in 0..len {
        // Check for user interrupt
        if unsafe { nvim_digraph_got_int() } != 0 {
            return 0;
        }

        let dp = unsafe { &*user_digraphs.add(i) };
        let should_continue = unsafe { callback(dp.char1, dp.char2, dp.result, ctx) };
        if should_continue == 0 {
            return 0;
        }

        // Check for breakcheck
        unsafe { nvim_digraph_fast_breakcheck() };
    }

    1
}

/// Format a digraph as a two-character string.
///
/// Writes the two digraph characters to `buf` followed by NUL.
///
/// # Arguments
/// * `char1` - First character
/// * `char2` - Second character
/// * `buf` - Output buffer (at least 3 bytes)
///
/// # Safety
/// `buf` must point to at least 3 writable bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_digraph_format_pair(char1: u8, char2: u8, buf: *mut c_char) {
    if buf.is_null() {
        return;
    }
    unsafe {
        #[allow(clippy::cast_possible_wrap)]
        {
            *buf = char1 as c_char;
            *buf.add(1) = char2 as c_char;
        }
        *buf.add(2) = 0;
    }
}

/// Format a digraph result as UTF-8.
///
/// Writes the UTF-8 representation of the result character to `buf`.
///
/// # Arguments
/// * `result` - The digraph result character
/// * `buf` - Output buffer (at least 7 bytes for UTF-8 + NUL)
///
/// # Returns
/// Number of bytes written (not including NUL).
///
/// # Safety
/// `buf` must point to at least 7 writable bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_digraph_format_result(result: c_int, buf: *mut c_char) -> c_int {
    if buf.is_null() {
        return 0;
    }

    let len = unsafe { rs_utf_char2bytes(result, buf) };

    // NUL terminate
    #[allow(clippy::cast_sign_loss)]
    if len >= 0 {
        unsafe { *buf.add(len as usize) = 0 };
    }

    len
}

/// Display a digraph header line.
///
/// If the cursor is past column 0, output a newline first.
/// Then output the translated header text with `HLF_CM` highlighting,
/// followed by a newline.
///
/// # Safety
/// `msg` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_digraph_header(msg: *const c_char) {
    if msg.is_null() {
        return;
    }
    if unsafe { nvim_get_msg_col() } > 0 {
        unsafe { msg_putchar(c_int::from(b'\n')) };
    }
    unsafe { msg_outtrans(msg, HLF_CM, false) };
    unsafe { msg_putchar(c_int::from(b'\n')) };
}

/// Print a single digraph entry to the message output.
///
/// Formats and displays a digraph as: `{c1}{c2} {char} {decimal}`
/// with appropriate highlighting and column alignment.
///
/// If `previous` is non-null, checks whether a category header should be
/// displayed before this entry (based on the previous result code).
///
/// # Arguments
/// * `char1` - First character of the digraph
/// * `char2` - Second character of the digraph
/// * `result` - The digraph result character
/// * `previous` - Pointer to the previous result code (nullable)
///
/// # Safety
/// `previous` must be null or a valid pointer to a writable `c_int`.
#[no_mangle]
pub unsafe extern "C" fn rs_printdigraph(
    char1: u8,
    char2: u8,
    result: c_int,
    previous: *mut c_int,
) {
    if result == 0 {
        return;
    }

    let list_width: c_int = 13;

    // Show category header if needed
    if !previous.is_null() {
        let prev_val = unsafe { *previous };
        let header_idx = rs_digraph_get_header_index(prev_val, result);
        #[allow(clippy::cast_sign_loss)]
        if header_idx >= 0 && (header_idx as usize) < HEADER_STRINGS.len() {
            let header_str = HEADER_STRINGS[header_idx as usize];
            let translated = unsafe { nvim_gettext(header_str.as_ptr()) };
            unsafe { rs_digraph_header(translated) };
        }
        unsafe { *previous = result };
    }

    // Wrap to next line if not enough room
    if unsafe { nvim_get_msg_col() } > unsafe { nvim_get_Columns() } - list_width {
        unsafe { msg_putchar(c_int::from(b'\n')) };
    }

    // Align to multiple of list_width
    let msg_col = unsafe { nvim_get_msg_col() };
    if msg_col % list_width != 0 {
        let mut spaces = (msg_col / list_width + 1) * list_width - msg_col;
        while spaces > 0 {
            unsafe { msg_putchar(c_int::from(b' ')) };
            spaces -= 1;
        }
    }

    // Output "{c1}{c2} " with default highlighting
    let mut buf = [0i8; 4];
    #[allow(clippy::cast_possible_wrap)]
    {
        buf[0] = char1 as c_char;
        buf[1] = char2 as c_char;
        buf[2] = b' ' as c_char;
        buf[3] = 0;
    }
    unsafe { msg_outtrans(buf.as_ptr(), 0, false) };

    // Output the result character with HLF_8 highlighting
    let mut charbuf = [0i8; 8];
    let mut pos = 0usize;

    // Add space for composing characters
    if unsafe { nvim_utf_iscomposing_first(result) } != 0 {
        #[allow(clippy::cast_possible_wrap)]
        {
            charbuf[pos] = b' ' as c_char;
        }
        pos += 1;
    }

    let utf8_len = unsafe { rs_utf_char2bytes(result, charbuf.as_mut_ptr().add(pos)) };
    #[allow(clippy::cast_sign_loss)]
    if utf8_len > 0 {
        pos += utf8_len as usize;
    }
    charbuf[pos] = 0;
    unsafe { msg_outtrans(charbuf.as_ptr(), HLF_8, false) };

    // Output " {decimal}" with default highlighting
    let mut numbuf = [0i8; 16];
    let mut npos = 0usize;

    // Add padding space for single-width characters
    if unsafe { nvim_char2cells(result) } == 1 {
        #[allow(clippy::cast_possible_wrap)]
        {
            numbuf[npos] = b' ' as c_char;
        }
        npos += 1;
    }
    #[allow(clippy::cast_possible_wrap)]
    {
        numbuf[npos] = b' ' as c_char;
    }
    npos += 1;

    // Format decimal number right-aligned in 3 chars
    let mut dec_buf = [0u8; 8];
    let dec_str = format_decimal(result, &mut dec_buf);
    let padding = if dec_str.len() < 3 {
        3 - dec_str.len()
    } else {
        0
    };
    for _ in 0..padding {
        #[allow(clippy::cast_possible_wrap)]
        {
            numbuf[npos] = b' ' as c_char;
        }
        npos += 1;
    }
    for &b in dec_str {
        #[allow(clippy::cast_possible_wrap)]
        {
            numbuf[npos] = b as c_char;
        }
        npos += 1;
    }
    numbuf[npos] = 0;
    unsafe { msg_outtrans(numbuf.as_ptr(), 0, false) };
}

/// List all digraphs to the message output.
///
/// Iterates over default digraphs (with optional category headers)
/// and user digraphs, displaying each with `rs_printdigraph`.
///
/// # Arguments
/// * `use_headers` - If nonzero, display category headers between sections
///
/// # Safety
/// Calls C message output functions.
#[no_mangle]
pub unsafe extern "C" fn rs_listdigraphs(use_headers: c_int) {
    let mut previous: c_int = 0;

    unsafe { msg_ext_set_kind(c"list_cmd".as_ptr()) };
    unsafe { msg_putchar(c_int::from(b'\n')) };

    // Iterate default digraphs
    for dp in DIGRAPH_DEFAULT {
        // Check for user interrupt
        if unsafe { nvim_digraph_got_int() } != 0 {
            return;
        }

        // Get actual result (may be overridden by user digraph)
        let result = unsafe { rs_getexactdigraph(c_int::from(dp.char1), c_int::from(dp.char2), 0) };

        if result != 0 && result != c_int::from(dp.char2) {
            let prev_ptr = if use_headers != 0 {
                &raw mut previous
            } else {
                std::ptr::null_mut()
            };
            unsafe { rs_printdigraph(dp.char1, dp.char2, result, prev_ptr) };
        }
        unsafe { nvim_digraph_fast_breakcheck() };
    }

    // Iterate user digraphs
    let user_data = unsafe { nvim_get_user_digraphs_data() };
    let user_len = unsafe { nvim_get_user_digraphs_len() };

    if !user_data.is_null() && user_len > 0 {
        let user_digraphs = user_data.cast::<DigrT>();
        #[allow(clippy::cast_sign_loss)]
        let len = user_len as usize;

        for i in 0..len {
            if unsafe { nvim_digraph_got_int() } != 0 {
                return;
            }

            let dp = unsafe { &*user_digraphs.add(i) };

            // Show "Custom" header before first user digraph
            if previous >= 0 && use_headers != 0 {
                let translated = unsafe { nvim_gettext(c"Custom".as_ptr()) };
                unsafe { rs_digraph_header(translated) };
            }
            previous = -1;

            unsafe { rs_printdigraph(dp.char1, dp.char2, dp.result, std::ptr::null_mut()) };
            unsafe { nvim_digraph_fast_breakcheck() };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_format_pair() {
        let mut buf = [0i8; 4];
        unsafe { rs_digraph_format_pair(b'a', b':', buf.as_mut_ptr()) };
        assert_eq!(buf[0], b'a' as i8);
        assert_eq!(buf[1], b':' as i8);
        assert_eq!(buf[2], 0);
    }

    #[test]
    fn test_format_pair_null_safe() {
        // Should not crash with null
        unsafe { rs_digraph_format_pair(b'a', b'b', std::ptr::null_mut()) };
    }

    #[test]
    fn test_format_decimal() {
        let mut buf = [0u8; 8];

        // Test zero
        assert_eq!(format_decimal(0, &mut buf), b"0");

        // Test positive numbers
        assert_eq!(format_decimal(1, &mut buf), b"1");
        assert_eq!(format_decimal(42, &mut buf), b"42");
        assert_eq!(format_decimal(123, &mut buf), b"123");
        assert_eq!(format_decimal(9999, &mut buf), b"9999");

        // Test negative numbers
        assert_eq!(format_decimal(-1, &mut buf), b"-1");
        assert_eq!(format_decimal(-42, &mut buf), b"-42");
    }

    #[test]
    fn test_header_lookup() {
        // No header for values below DG_START_LATIN
        assert_eq!(rs_digraph_get_header_index(0, 0x50), -1);

        // Latin supplement header (0xa1)
        assert_eq!(rs_digraph_get_header_index(0, DG_START_LATIN), 0);

        // Greek header (0x0386)
        assert_eq!(rs_digraph_get_header_index(0, DG_START_GREEK), 1);
        assert_eq!(
            rs_digraph_get_header_index(DG_START_LATIN, DG_START_GREEK),
            1
        );

        // No new header if still in same range
        assert_eq!(
            rs_digraph_get_header_index(DG_START_LATIN, DG_START_LATIN + 1),
            -1
        );

        // Cyrillic header (0x0401)
        assert_eq!(
            rs_digraph_get_header_index(DG_START_GREEK, DG_START_CYRILLIC),
            2
        );
    }

    #[test]
    fn test_header_table_order() {
        // Verify header table is in ascending order by start code
        for i in 1..HEADER_TABLE.len() {
            assert!(
                HEADER_TABLE[i].0 > HEADER_TABLE[i - 1].0,
                "Header table not sorted at index {i}"
            );
        }
    }

    #[test]
    fn test_dg_start_constants() {
        // Verify all 26 DG_START constants match C #define values
        assert_eq!(DG_START_LATIN, 0xa1);
        assert_eq!(DG_START_GREEK, 0x0386);
        assert_eq!(DG_START_CYRILLIC, 0x0401);
        assert_eq!(DG_START_HEBREW, 0x05d0);
        assert_eq!(DG_START_ARABIC, 0x060c);
        assert_eq!(DG_START_LATIN_EXTENDED, 0x1e02);
        assert_eq!(DG_START_GREEK_EXTENDED, 0x1f00);
        assert_eq!(DG_START_PUNCTUATION, 0x2002);
        assert_eq!(DG_START_SUB_SUPER, 0x2070);
        assert_eq!(DG_START_CURRENCY, 0x20a4);
        assert_eq!(DG_START_OTHER1, 0x2103);
        assert_eq!(DG_START_ROMAN, 0x2160);
        assert_eq!(DG_START_ARROWS, 0x2190);
        assert_eq!(DG_START_MATH, 0x2200);
        assert_eq!(DG_START_TECHNICAL, 0x2302);
        assert_eq!(DG_START_OTHER2, 0x2423);
        assert_eq!(DG_START_DRAWING, 0x2500);
        assert_eq!(DG_START_BLOCK, 0x2580);
        assert_eq!(DG_START_SHAPES, 0x25a0);
        assert_eq!(DG_START_SYMBOLS, 0x2605);
        assert_eq!(DG_START_DINGBATS, 0x2713);
        assert_eq!(DG_START_CJK_SYMBOLS, 0x3000);
        assert_eq!(DG_START_HIRAGANA, 0x3041);
        assert_eq!(DG_START_KATAKANA, 0x30a1);
        assert_eq!(DG_START_BOPOMOFO, 0x3105);
        assert_eq!(DG_START_OTHER3, 0x3220);
    }
}
