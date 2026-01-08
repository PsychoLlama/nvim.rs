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

// =============================================================================
// Additional Replace Helpers
// =============================================================================

/// Calculate extra spaces needed for block mode replacement.
///
/// This handles the pre-spaces and post-spaces when tabs are split.
///
/// # Arguments
/// * `startspaces` - Number of start spaces
/// * `start_char_vcols` - Visual columns of start character
/// * `endspaces` - Number of end spaces
/// * `is_one_char` - Whether the block is just one character
/// * `end_char_vcols` - Visual columns of end character
///
/// # Returns
/// Total extra spaces needed
#[must_use]
#[inline]
pub const fn calc_block_extra_spaces(
    startspaces: c_int,
    start_char_vcols: c_int,
    endspaces: c_int,
    is_one_char: bool,
    end_char_vcols: c_int,
) -> c_int {
    // n = (bd.startspaces ? bd.start_char_vcols - 1 : 0);
    let pre = if startspaces != 0 {
        start_char_vcols - 1
    } else {
        0
    };

    // n += (bd.endspaces && !bd.is_oneChar && bd.end_char_vcols > 0)
    //      ? bd.end_char_vcols - 1 : 0;
    let post = if endspaces != 0 && !is_one_char && end_char_vcols > 0 {
        end_char_vcols - 1
    } else {
        0
    };

    pre + post
}

/// Check if a block line should be skipped during replacement.
///
/// A line is skipped if:
/// - textlen is 0 AND
/// - virtual mode is off OR the block is MAX width
///
/// # Arguments
/// * `textlen` - Length of text in block on this line
/// * `virtual_op` - Whether virtual editing is enabled
/// * `is_max` - Whether the block extends to the end of line
///
/// # Returns
/// true if this line should be skipped
#[must_use]
#[inline]
pub const fn should_skip_block_line(textlen: c_int, virtual_op: bool, is_max: bool) -> bool {
    textlen == 0 && (!virtual_op || is_max)
}

/// Calculate extra startspaces for virtual operation on short lines.
///
/// When operating in virtual mode on a short line, we need to add
/// the coladd offset to startspaces.
///
/// # Arguments
/// * `coladd` - The virtual column offset
///
/// # Returns
/// Extra spaces to add
#[must_use]
#[inline]
pub const fn calc_virtual_extra_startspaces(coladd: c_int) -> c_int {
    coladd
}

/// Adjust end column for linewise/non-inclusive motion in replace.
///
/// For linewise motion, start.col is set to 0 and end.col to line length - 1.
/// For non-inclusive charwise motion, end is decremented.
///
/// # Arguments
/// * `is_linewise` - Whether this is a linewise motion
/// * `is_inclusive` - Whether the motion is inclusive
/// * `end_col` - Current end column
/// * `line_len` - Length of the end line
///
/// # Returns
/// (new_start_col, new_end_col) - Adjusted start and end columns
#[must_use]
#[inline]
pub const fn adjust_replace_cols_for_motion(
    is_linewise: bool,
    is_inclusive: bool,
    end_col: c_int,
    line_len: c_int,
) -> (c_int, c_int) {
    if is_linewise {
        // Start at column 0, end at last column
        let new_end = if line_len > 0 { line_len - 1 } else { 0 };
        (0, new_end)
    } else if !is_inclusive && end_col > 0 {
        // Decrement end for non-inclusive
        (0, end_col - 1)
    } else {
        (0, end_col)
    }
}

/// Check if multi-byte handling is needed for character replacement.
///
/// When either the new or old character takes more than 1 byte,
/// we need to use the slower but correct replace_character function.
///
/// # Arguments
/// * `new_byte_len` - Byte length of new character
/// * `old_byte_len` - Byte length of old character
///
/// # Returns
/// true if multi-byte handling is needed
#[must_use]
#[inline]
pub const fn needs_multibyte_replace(new_byte_len: c_int, old_byte_len: c_int) -> bool {
    new_byte_len > 1 || old_byte_len > 1
}

/// Adjust oap->end.col when replacing single-byte with multi-byte or vice versa.
///
/// When on the last line and the byte length changes, the end column
/// must be adjusted accordingly.
///
/// # Arguments
/// * `end_col` - Current end column
/// * `new_byte_len` - Byte length of new character
/// * `old_byte_len` - Byte length of old character
///
/// # Returns
/// Adjusted end column
#[must_use]
#[inline]
pub const fn adjust_end_col_for_byte_change(
    end_col: c_int,
    new_byte_len: c_int,
    old_byte_len: c_int,
) -> c_int {
    end_col + (new_byte_len - old_byte_len)
}

/// Calculate the number of extmark columns for block replacement.
///
/// # Arguments
/// * `textcol` - Column where text starts
/// * `newp_len` - Length of new content
///
/// # Returns
/// Number of new columns for extmark splice
#[must_use]
#[inline]
pub const fn calc_extmark_newcols(textcol: c_int, newp_len: c_int) -> c_int {
    newp_len - textcol
}

/// Calculate virtual columns adjustment for replace in virtual mode.
///
/// # Arguments
/// * `end_coladd` - End position coladd
/// * `start_coladd` - Start position coladd
/// * `same_col` - Whether start and end are on same column
/// * `same_line` - Whether start and end are on same line
///
/// # Returns
/// Number of virtual columns to replace
#[must_use]
#[inline]
pub const fn calc_virtcols_for_replace(
    end_coladd: c_int,
    start_coladd: c_int,
    same_col: bool,
    same_line: bool,
) -> c_int {
    let mut virtcols = end_coladd;
    if same_line && same_col && start_coladd != 0 {
        virtcols -= start_coladd;
    }
    virtcols
}

// =============================================================================
// FFI Wrappers for Additional Helpers
// =============================================================================

/// FFI wrapper for calc_block_extra_spaces.
#[no_mangle]
pub extern "C" fn rs_calc_block_extra_spaces(
    startspaces: c_int,
    start_char_vcols: c_int,
    endspaces: c_int,
    is_one_char: c_int,
    end_char_vcols: c_int,
) -> c_int {
    calc_block_extra_spaces(
        startspaces,
        start_char_vcols,
        endspaces,
        is_one_char != 0,
        end_char_vcols,
    )
}

/// FFI wrapper for should_skip_block_line.
#[no_mangle]
pub extern "C" fn rs_should_skip_block_line(
    textlen: c_int,
    virtual_op: c_int,
    is_max: c_int,
) -> c_int {
    c_int::from(should_skip_block_line(
        textlen,
        virtual_op != 0,
        is_max != 0,
    ))
}

/// FFI wrapper for calc_virtual_extra_startspaces.
#[no_mangle]
pub extern "C" fn rs_calc_virtual_extra_startspaces(coladd: c_int) -> c_int {
    calc_virtual_extra_startspaces(coladd)
}

/// FFI wrapper for needs_multibyte_replace.
#[no_mangle]
pub extern "C" fn rs_needs_multibyte_replace(new_byte_len: c_int, old_byte_len: c_int) -> c_int {
    c_int::from(needs_multibyte_replace(new_byte_len, old_byte_len))
}

/// FFI wrapper for adjust_end_col_for_byte_change.
#[no_mangle]
pub extern "C" fn rs_adjust_end_col_for_byte_change(
    end_col: c_int,
    new_byte_len: c_int,
    old_byte_len: c_int,
) -> c_int {
    adjust_end_col_for_byte_change(end_col, new_byte_len, old_byte_len)
}

/// FFI wrapper for calc_extmark_newcols.
#[no_mangle]
pub extern "C" fn rs_calc_extmark_newcols(textcol: c_int, newp_len: c_int) -> c_int {
    calc_extmark_newcols(textcol, newp_len)
}

/// FFI wrapper for calc_virtcols_for_replace.
#[no_mangle]
pub extern "C" fn rs_calc_virtcols_for_replace(
    end_coladd: c_int,
    start_coladd: c_int,
    same_col: c_int,
    same_line: c_int,
) -> c_int {
    calc_virtcols_for_replace(end_coladd, start_coladd, same_col != 0, same_line != 0)
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

    // =========================================================================
    // Additional Helper Function Tests
    // =========================================================================

    #[test]
    fn test_calc_block_extra_spaces() {
        // No start or end spaces
        assert_eq!(calc_block_extra_spaces(0, 4, 0, false, 4), 0);

        // Only start spaces
        assert_eq!(calc_block_extra_spaces(1, 4, 0, false, 0), 3);

        // Only end spaces (not one char, positive end_char_vcols)
        assert_eq!(calc_block_extra_spaces(0, 0, 1, false, 4), 3);

        // Both start and end spaces
        assert_eq!(calc_block_extra_spaces(1, 4, 1, false, 4), 6);

        // End spaces with is_one_char (no end contribution)
        assert_eq!(calc_block_extra_spaces(1, 4, 1, true, 4), 3);

        // End spaces with zero end_char_vcols (no end contribution)
        assert_eq!(calc_block_extra_spaces(1, 4, 1, false, 0), 3);
    }

    #[test]
    fn test_should_skip_block_line() {
        // textlen > 0: never skip
        assert!(!should_skip_block_line(5, false, false));
        assert!(!should_skip_block_line(5, true, true));

        // textlen == 0, no virtual mode: skip
        assert!(should_skip_block_line(0, false, false));

        // textlen == 0, virtual mode but is_max: skip
        assert!(should_skip_block_line(0, true, true));

        // textlen == 0, virtual mode and not is_max: don't skip
        assert!(!should_skip_block_line(0, true, false));
    }

    #[test]
    fn test_calc_virtual_extra_startspaces() {
        assert_eq!(calc_virtual_extra_startspaces(0), 0);
        assert_eq!(calc_virtual_extra_startspaces(5), 5);
        assert_eq!(calc_virtual_extra_startspaces(10), 10);
    }

    #[test]
    fn test_adjust_replace_cols_for_motion() {
        // Linewise: start at 0, end at line_len - 1
        let (start, end) = adjust_replace_cols_for_motion(true, false, 10, 50);
        assert_eq!(start, 0);
        assert_eq!(end, 49);

        // Linewise with empty line
        let (start, end) = adjust_replace_cols_for_motion(true, false, 0, 0);
        assert_eq!(start, 0);
        assert_eq!(end, 0);

        // Non-inclusive charwise: decrement end
        let (start, end) = adjust_replace_cols_for_motion(false, false, 10, 50);
        assert_eq!(start, 0);
        assert_eq!(end, 9);

        // Inclusive charwise: keep end
        let (start, end) = adjust_replace_cols_for_motion(false, true, 10, 50);
        assert_eq!(start, 0);
        assert_eq!(end, 10);
    }

    #[test]
    fn test_needs_multibyte_replace() {
        // Both single-byte
        assert!(!needs_multibyte_replace(1, 1));

        // New is multi-byte
        assert!(needs_multibyte_replace(3, 1));

        // Old is multi-byte
        assert!(needs_multibyte_replace(1, 3));

        // Both multi-byte
        assert!(needs_multibyte_replace(3, 3));
    }

    #[test]
    fn test_adjust_end_col_for_byte_change() {
        // Same byte length: no change
        assert_eq!(adjust_end_col_for_byte_change(10, 1, 1), 10);

        // New longer: increase end col
        assert_eq!(adjust_end_col_for_byte_change(10, 3, 1), 12);

        // New shorter: decrease end col
        assert_eq!(adjust_end_col_for_byte_change(10, 1, 3), 8);
    }

    #[test]
    fn test_calc_extmark_newcols() {
        assert_eq!(calc_extmark_newcols(10, 20), 10);
        assert_eq!(calc_extmark_newcols(5, 5), 0);
        assert_eq!(calc_extmark_newcols(0, 15), 15);
    }

    #[test]
    fn test_calc_virtcols_for_replace() {
        // Normal case: just return end_coladd
        assert_eq!(calc_virtcols_for_replace(5, 0, false, false), 5);

        // Same line and column with start_coladd: subtract
        assert_eq!(calc_virtcols_for_replace(5, 2, true, true), 3);

        // Same column but different line: no subtraction
        assert_eq!(calc_virtcols_for_replace(5, 2, true, false), 5);

        // Same line but different column: no subtraction
        assert_eq!(calc_virtcols_for_replace(5, 2, false, true), 5);

        // Zero start_coladd: no subtraction
        assert_eq!(calc_virtcols_for_replace(5, 0, true, true), 5);
    }

    #[test]
    fn test_additional_ffi_wrappers() {
        // rs_calc_block_extra_spaces
        assert_eq!(rs_calc_block_extra_spaces(1, 4, 1, 0, 4), 6);
        assert_eq!(rs_calc_block_extra_spaces(1, 4, 1, 1, 4), 3);

        // rs_should_skip_block_line
        assert_eq!(rs_should_skip_block_line(0, 0, 0), 1);
        assert_eq!(rs_should_skip_block_line(5, 0, 0), 0);
        assert_eq!(rs_should_skip_block_line(0, 1, 0), 0);

        // rs_calc_virtual_extra_startspaces
        assert_eq!(rs_calc_virtual_extra_startspaces(5), 5);

        // rs_needs_multibyte_replace
        assert_eq!(rs_needs_multibyte_replace(1, 1), 0);
        assert_eq!(rs_needs_multibyte_replace(3, 1), 1);

        // rs_adjust_end_col_for_byte_change
        assert_eq!(rs_adjust_end_col_for_byte_change(10, 3, 1), 12);

        // rs_calc_extmark_newcols
        assert_eq!(rs_calc_extmark_newcols(10, 20), 10);

        // rs_calc_virtcols_for_replace
        assert_eq!(rs_calc_virtcols_for_replace(5, 2, 1, 1), 3);
        assert_eq!(rs_calc_virtcols_for_replace(5, 2, 0, 1), 5);
    }
}
