//! Replace operations (r)
//!
//! This module implements calculation logic for the `r` operator
//! (character replacement in visual mode).

use std::ffi::c_int;

/// Special character constants for replacement operations
pub const REPLACE_CR_NCHAR: c_int = -1;
pub const REPLACE_NL_NCHAR: c_int = -2;

/// Carriage return
pub const CAR: c_int = b'\r' as c_int;
/// Newline
pub const NL: c_int = b'\n' as c_int;
/// Tab
pub const TAB: c_int = b'\t' as c_int;

/// Normalize replacement character.
///
/// Handles special replacement characters that represent literal CR/NL.
/// Returns the normalized character and whether it was originally a special char.
///
/// # Arguments
/// * `c` - The replacement character (may be REPLACE_CR_NCHAR or REPLACE_NL_NCHAR)
///
/// # Returns
/// `(normalized_char, had_ctrl_v_cr)` - The normalized character and flag
#[must_use]
#[inline]
pub const fn normalize_replace_char(c: c_int) -> (c_int, bool) {
    match c {
        REPLACE_CR_NCHAR => (CAR, true),
        REPLACE_NL_NCHAR => (NL, true),
        _ => (c, false),
    }
}

/// Calculate number of characters to replace in block mode.
///
/// # Arguments
/// * `start_vcol` - Start virtual column
/// * `end_vcol` - End virtual column
/// * `is_short` - Whether the line is short (ends before end_vcol)
/// * `bd_end_vcol` - Block def end virtual column
///
/// # Returns
/// Number of characters to replace
#[must_use]
#[inline]
pub const fn calc_replace_numc(
    start_vcol: c_int,
    end_vcol: c_int,
    is_short: bool,
    bd_end_vcol: c_int,
) -> c_int {
    let mut numc = end_vcol - start_vcol + 1;
    if is_short {
        numc -= (end_vcol - bd_end_vcol) + 1;
    }
    if numc < 0 {
        0
    } else {
        numc
    }
}

/// Adjust character count for double-wide replacement character.
///
/// When replacing with a double-wide character, we can only fit half as many.
/// Also calculates if we need an extra space due to odd count.
///
/// # Arguments
/// * `numc` - Number of characters to replace
/// * `is_double_wide` - Whether the replacement character is double-wide
/// * `is_short` - Whether the line is short
///
/// # Returns
/// `(adjusted_numc, extra_end_space)` - Adjusted count and whether extra end space is needed
#[must_use]
#[inline]
pub const fn adjust_for_double_wide(
    numc: c_int,
    is_double_wide: bool,
    is_short: bool,
) -> (c_int, bool) {
    if !is_double_wide {
        return (numc, false);
    }

    let extra_end_space = (numc & 1) != 0 && !is_short;
    let adjusted = numc / 2;
    (adjusted, extra_end_space)
}

/// Calculate new buffer size for block replacement.
///
/// # Arguments
/// * `textcol` - Column where text starts
/// * `startspaces` - Number of start spaces (for tabs, virtual columns)
/// * `numc` - Number of replacement characters (already adjusted for char width)
/// * `char_byte_len` - Byte length of replacement character (UTF-8)
/// * `endspaces` - Number of end spaces
/// * `old_len` - Original line length
/// * `textlen` - Length of text being replaced
/// * `is_short` - Whether line is short
/// * `had_ctrl_v_cr` - Whether replacing with literal CR/NL
/// * `c` - Replacement character
///
/// # Returns
/// New buffer size needed
#[must_use]
#[allow(clippy::too_many_arguments)]
pub const fn calc_replace_buffer_size(
    textcol: usize,
    startspaces: usize,
    numc: usize,
    char_byte_len: usize,
    endspaces: usize,
    old_len: usize,
    textlen: usize,
    is_short: bool,
    had_ctrl_v_cr: bool,
    c: c_int,
) -> usize {
    let mut size = textcol + startspaces;

    if had_ctrl_v_cr || (c != CAR && c != NL) {
        size += numc * char_byte_len;
        if !is_short {
            size += endspaces + old_len - textcol - textlen;
        }
    }

    size
}

/// Check if replacement will split the line.
///
/// Replacing with CR or NL (without Ctrl-V) causes a line split.
///
/// # Arguments
/// * `c` - Replacement character
/// * `had_ctrl_v_cr` - Whether it was entered with Ctrl-V
///
/// # Returns
/// true if the replacement will cause a line split
#[must_use]
#[inline]
pub const fn will_split_line(c: c_int, had_ctrl_v_cr: bool) -> bool {
    !had_ctrl_v_cr && (c == CAR || c == NL)
}

/// Calculate extra spaces needed when splitting tabs.
///
/// When a tab is split during replacement, extra spaces may be needed.
///
/// # Arguments
/// * `startspaces` - Number of spaces for start
/// * `start_char_vcols` - Visual columns of start character
///
/// # Returns
/// Extra space count
#[must_use]
#[inline]
pub const fn calc_extra_spaces(startspaces: c_int, start_char_vcols: c_int) -> c_int {
    if startspaces != 0 {
        start_char_vcols - 1
    } else {
        0
    }
}

/// FFI wrapper for normalize_replace_char.
///
/// # Safety
/// `had_ctrl_v_cr_out` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_normalize_replace_char(
    c: c_int,
    had_ctrl_v_cr_out: *mut c_int,
) -> c_int {
    let (normalized, had_ctrl_v_cr) = normalize_replace_char(c);
    if !had_ctrl_v_cr_out.is_null() {
        // SAFETY: Caller guarantees pointer validity
        unsafe {
            *had_ctrl_v_cr_out = c_int::from(had_ctrl_v_cr);
        }
    }
    normalized
}

/// FFI wrapper for calc_replace_numc.
#[no_mangle]
pub extern "C" fn rs_calc_replace_numc(
    start_vcol: c_int,
    end_vcol: c_int,
    is_short: c_int,
    bd_end_vcol: c_int,
) -> c_int {
    calc_replace_numc(start_vcol, end_vcol, is_short != 0, bd_end_vcol)
}

/// FFI wrapper for will_split_line.
#[no_mangle]
pub extern "C" fn rs_will_split_line(c: c_int, had_ctrl_v_cr: c_int) -> c_int {
    c_int::from(will_split_line(c, had_ctrl_v_cr != 0))
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_replace_char() {
        // Normal characters unchanged
        let (c, flag) = normalize_replace_char(b'x' as c_int);
        assert_eq!(c, b'x' as c_int);
        assert!(!flag);

        // Special CR
        let (c, flag) = normalize_replace_char(REPLACE_CR_NCHAR);
        assert_eq!(c, CAR);
        assert!(flag);

        // Special NL
        let (c, flag) = normalize_replace_char(REPLACE_NL_NCHAR);
        assert_eq!(c, NL);
        assert!(flag);
    }

    #[test]
    fn test_calc_replace_numc() {
        // Basic case: 10 columns (0 to 9 inclusive)
        assert_eq!(calc_replace_numc(0, 9, false, 9), 10);

        // Short line case: line ends at vcol 5, selection ends at 9
        // numc = (9 - 0 + 1) - ((9 - 5) + 1) = 10 - 5 = 5
        assert_eq!(calc_replace_numc(0, 9, true, 5), 5);

        // Single column
        assert_eq!(calc_replace_numc(5, 5, false, 5), 1);

        // Short line with selection fully within line
        // numc = (5 - 0 + 1) - ((5 - 5) + 1) = 6 - 1 = 5
        assert_eq!(calc_replace_numc(0, 5, true, 5), 5);

        // Short line ending before selection start results in negative -> clamped to 0
        assert_eq!(calc_replace_numc(5, 9, true, 3), 0);
    }

    #[test]
    fn test_adjust_for_double_wide() {
        // Single-width character - no change
        let (numc, extra) = adjust_for_double_wide(10, false, false);
        assert_eq!(numc, 10);
        assert!(!extra);

        // Double-width, even count
        let (numc, extra) = adjust_for_double_wide(10, true, false);
        assert_eq!(numc, 5);
        assert!(!extra);

        // Double-width, odd count - needs extra space
        let (numc, extra) = adjust_for_double_wide(11, true, false);
        assert_eq!(numc, 5);
        assert!(extra);

        // Double-width, odd count, short line - no extra space
        let (numc, extra) = adjust_for_double_wide(11, true, true);
        assert_eq!(numc, 5);
        assert!(!extra);
    }

    #[test]
    fn test_will_split_line() {
        // CR without Ctrl-V splits
        assert!(will_split_line(CAR, false));

        // NL without Ctrl-V splits
        assert!(will_split_line(NL, false));

        // CR with Ctrl-V doesn't split
        assert!(!will_split_line(CAR, true));

        // NL with Ctrl-V doesn't split
        assert!(!will_split_line(NL, true));

        // Regular char doesn't split
        assert!(!will_split_line(b'x' as c_int, false));
    }

    #[test]
    fn test_calc_extra_spaces() {
        // No start spaces
        assert_eq!(calc_extra_spaces(0, 4), 0);

        // With start spaces
        assert_eq!(calc_extra_spaces(1, 4), 3);
        assert_eq!(calc_extra_spaces(2, 8), 7);
    }

    #[test]
    fn test_calc_replace_buffer_size() {
        // Simple case: replace 5 single-byte chars
        let size = calc_replace_buffer_size(
            10,            // textcol
            0,             // startspaces
            5,             // numc
            1,             // char_byte_len
            0,             // endspaces
            100,           // old_len
            5,             // textlen
            false,         // is_short
            false,         // had_ctrl_v_cr
            b'x' as c_int, // c
        );
        assert_eq!(size, 100); // textcol + numc + endspaces + old_len - textcol - textlen

        // With startspaces and endspaces
        let size = calc_replace_buffer_size(
            10,            // textcol
            2,             // startspaces
            5,             // numc
            1,             // char_byte_len
            3,             // endspaces
            100,           // old_len
            5,             // textlen
            false,         // is_short
            false,         // had_ctrl_v_cr
            b'x' as c_int, // c
        );
        assert_eq!(size, 105); // with startspaces=2 and endspaces=3

        // Multi-byte replacement char
        let size = calc_replace_buffer_size(
            10,     // textcol
            0,      // startspaces
            5,      // numc
            3,      // char_byte_len (3-byte UTF-8)
            0,      // endspaces
            100,    // old_len
            5,      // textlen
            false,  // is_short
            false,  // had_ctrl_v_cr
            0x3042, // c (hiragana 'a')
        );
        assert_eq!(size, 110); // 3-byte char * 5 = 15 bytes

        // Line split case (CR without Ctrl-V)
        let size = calc_replace_buffer_size(
            10,    // textcol
            0,     // startspaces
            5,     // numc - ignored when splitting
            1,     // char_byte_len
            0,     // endspaces
            100,   // old_len
            5,     // textlen
            false, // is_short
            false, // had_ctrl_v_cr
            CAR,   // c
        );
        assert_eq!(size, 10); // Only textcol + startspaces

        // Short line
        let size = calc_replace_buffer_size(
            10,            // textcol
            0,             // startspaces
            5,             // numc
            1,             // char_byte_len
            0,             // endspaces
            100,           // old_len
            5,             // textlen
            true,          // is_short
            false,         // had_ctrl_v_cr
            b'x' as c_int, // c
        );
        assert_eq!(size, 10 + 5); // Only textcol + startspaces + replacement
    }
}
