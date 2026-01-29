//! Join operations (J, gJ)
//!
//! This module implements calculation logic for the `J` and `gJ` operators
//! (line joining).

use std::ffi::c_int;

/// NUL character
const NUL: c_int = 0;
/// Tab character
const TAB: c_int = b'\t' as c_int;

/// Calculate the number of spaces to insert when joining lines.
///
/// This implements the space-insertion logic from `do_join()` in ops.c.
/// The logic depends on:
/// - The last character(s) of the previous line
/// - The first character of the current line
/// - Format options (joinspaces, mbyte handling)
///
/// # Arguments
/// * `endcurr1` - Last character of previous line (0 if empty)
/// * `endcurr2` - Second-to-last character of previous line (for double-space after sentence)
/// * `firstcurr` - First non-whitespace character of current line
/// * `sumsize` - Current sum size (0 means this is the first line)
/// * `joinspaces` - Whether 'joinspaces' option is set
/// * `fo_mbyte_join` - Whether FO_MBYTE_JOIN format option is set
/// * `fo_mbyte_join2` - Whether FO_MBYTE_JOIN2 format option is set
///
/// # Returns
/// Number of spaces to insert (0, 1, or 2)
#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn calc_join_spaces(
    endcurr1: c_int,
    endcurr2: c_int,
    firstcurr: c_int,
    sumsize: c_int,
    joinspaces: bool,
    fo_mbyte_join: bool,
    fo_mbyte_join2: bool,
) -> c_int {
    // No spaces needed for empty line or if current line is NUL or ')'
    if firstcurr == NUL || firstcurr == c_int::from(b')') || sumsize == 0 || endcurr1 == TAB {
        return 0;
    }

    // Check multibyte join options
    // FO_MBYTE_JOIN: don't add space if either char is wide (>= 0x100)
    if fo_mbyte_join && (firstcurr >= 0x100 || endcurr1 >= 0x100) {
        return 0;
    }

    // FO_MBYTE_JOIN2: more complex wide char handling
    if fo_mbyte_join2 && (firstcurr >= 0x100 || endcurr1 >= 0x100) {
        // We can't call utf_eat_space from pure Rust, so this is handled in C wrapper
        // For the pure Rust version, we approximate: wide chars don't need space
        return 0;
    }

    // C logic from do_join():
    // if (endcurr1 == ' ')
    //   endcurr1 = endcurr2;  // use char before trailing space for joinspaces
    // else
    //   spaces[t]++;          // add a space
    //
    // So if previous line ends in space, we DON'T add a space but check endcurr2
    // Otherwise we add a space and check endcurr1 for joinspaces
    let (add_space, check_char) = if endcurr1 == c_int::from(b' ') {
        (false, endcurr2)
    } else {
        (true, endcurr1)
    };

    let mut spaces = c_int::from(add_space);

    // Extra space for 'joinspaces' after sentence-ending punctuation
    if joinspaces && is_sentence_end(check_char) {
        spaces += 1;
    }

    spaces
}

/// Check if a character is sentence-ending punctuation.
///
/// Returns true for '.', '?', '!' which trigger double-spacing with 'joinspaces'.
#[must_use]
#[inline]
pub const fn is_sentence_end(c: c_int) -> bool {
    c == b'.' as c_int || c == b'?' as c_int || c == b'!' as c_int
}

/// Check if a character is considered "wide" for join purposes.
///
/// Characters >= 0x100 are considered wide (non-ASCII).
#[must_use]
#[inline]
pub const fn is_wide_char(c: c_int) -> bool {
    c >= 0x100
}

/// Calculate total size after joining lines.
///
/// # Arguments
/// * `line_sizes` - Sizes of each line to join
/// * `spaces` - Spaces to insert before each line
///
/// # Returns
/// Total size of joined line
///
/// # Panics
/// Panics if `line_sizes` and `spaces` have different lengths.
#[must_use]
pub fn calc_join_total_size(line_sizes: &[c_int], spaces: &[c_int]) -> c_int {
    assert_eq!(line_sizes.len(), spaces.len());
    let mut total = 0;
    for (size, space) in line_sizes.iter().zip(spaces.iter()) {
        total += size + space;
    }
    total
}

/// Calculate cursor column after join.
///
/// When 'cpoptions' contains 'j', cursor goes to first column of joined text.
/// Otherwise, it goes to the join position of the last line.
///
/// # Arguments
/// * `sumsize` - Total size of joined line
/// * `currsize` - Size of last line's contribution
/// * `last_spaces` - Spaces before last line
/// * `cpo_joincol` - Whether CPO_JOINCOL is set
///
/// # Returns
/// Column position for cursor
#[must_use]
#[inline]
pub const fn calc_join_cursor_col(
    sumsize: c_int,
    currsize: c_int,
    last_spaces: c_int,
    cpo_joincol: bool,
) -> c_int {
    if cpo_joincol {
        currsize
    } else {
        sumsize - currsize - last_spaces
    }
}

/// FFI wrapper for calc_join_spaces.
#[no_mangle]
pub extern "C" fn rs_calc_join_spaces(
    endcurr1: c_int,
    endcurr2: c_int,
    firstcurr: c_int,
    sumsize: c_int,
    joinspaces: c_int,
    fo_mbyte_join: c_int,
    fo_mbyte_join2: c_int,
) -> c_int {
    calc_join_spaces(
        endcurr1,
        endcurr2,
        firstcurr,
        sumsize,
        joinspaces != 0,
        fo_mbyte_join != 0,
        fo_mbyte_join2 != 0,
    )
}

/// FFI wrapper for is_sentence_end.
#[no_mangle]
pub extern "C" fn rs_is_sentence_end(c: c_int) -> c_int {
    c_int::from(is_sentence_end(c))
}

/// FFI wrapper for calc_join_cursor_col.
#[no_mangle]
pub extern "C" fn rs_calc_join_cursor_col(
    sumsize: c_int,
    currsize: c_int,
    last_spaces: c_int,
    cpo_joincol: c_int,
) -> c_int {
    calc_join_cursor_col(sumsize, currsize, last_spaces, cpo_joincol != 0)
}

// =============================================================================
// Additional Join Helpers
// =============================================================================

/// Determine if we should use gJ behavior (no space insertion).
///
/// The `gJ` command joins lines without inserting or removing any spaces.
///
/// # Arguments
/// * `use_formatoptions` - Whether to respect 'formatoptions'
///
/// # Returns
/// true if gJ behavior should be used (no space manipulation)
#[must_use]
#[inline]
pub const fn is_gj_mode(use_formatoptions: bool) -> bool {
    !use_formatoptions
}

/// Calculate the size of whitespace to skip at the start of a line.
///
/// When joining lines, leading whitespace on the second line is typically
/// removed (unless using `gJ`).
///
/// # Arguments
/// * `skip_whitespace` - Whether to skip leading whitespace
/// * `leading_space_count` - Number of leading spaces/tabs
///
/// # Returns
/// Number of characters to skip (0 if not skipping)
#[must_use]
#[inline]
pub const fn calc_skip_whitespace(skip_whitespace: bool, leading_space_count: c_int) -> c_int {
    if skip_whitespace {
        leading_space_count
    } else {
        0
    }
}

/// Calculate the new line size after joining.
///
/// # Arguments
/// * `first_line_len` - Length of first line (without newline)
/// * `second_line_len` - Length of second line (without leading whitespace to skip)
/// * `spaces_to_insert` - Number of spaces to insert between lines
/// * `skip_ws` - Number of whitespace characters to skip on second line
///
/// # Returns
/// Total size needed for the joined line
#[must_use]
#[inline]
pub const fn calc_joined_line_size(
    first_line_len: c_int,
    second_line_len: c_int,
    spaces_to_insert: c_int,
    skip_ws: c_int,
) -> c_int {
    let result = first_line_len + spaces_to_insert + (second_line_len - skip_ws);
    if result < 0 {
        0
    } else {
        result
    }
}

/// Space character
const SPACE: c_int = b' ' as c_int;

/// Check if character is whitespace for join purposes.
///
/// Space and tab are considered whitespace.
#[must_use]
#[inline]
pub const fn is_join_whitespace(c: c_int) -> bool {
    c == SPACE || c == TAB
}

/// Determine if joining would create a line that's too long.
///
/// # Arguments
/// * `current_size` - Current line size
/// * `add_size` - Size to add
/// * `max_line_len` - Maximum allowed line length (MAXCOL in C)
///
/// # Returns
/// true if the join would exceed the maximum line length
#[must_use]
#[inline]
pub const fn would_exceed_line_limit(
    current_size: c_int,
    add_size: c_int,
    max_line_len: c_int,
) -> bool {
    // Check for overflow
    if current_size > max_line_len - add_size {
        return true;
    }
    current_size + add_size > max_line_len
}

// =============================================================================
// FFI Wrappers for Additional Helpers
// =============================================================================

/// FFI wrapper for is_gj_mode.
#[no_mangle]
pub extern "C" fn rs_is_gj_mode(use_formatoptions: c_int) -> c_int {
    c_int::from(is_gj_mode(use_formatoptions != 0))
}

/// FFI wrapper for calc_skip_whitespace.
#[no_mangle]
pub extern "C" fn rs_calc_skip_whitespace(
    skip_whitespace: c_int,
    leading_space_count: c_int,
) -> c_int {
    calc_skip_whitespace(skip_whitespace != 0, leading_space_count)
}

/// FFI wrapper for calc_joined_line_size.
#[no_mangle]
pub extern "C" fn rs_calc_joined_line_size(
    first_line_len: c_int,
    second_line_len: c_int,
    spaces_to_insert: c_int,
    skip_ws: c_int,
) -> c_int {
    calc_joined_line_size(first_line_len, second_line_len, spaces_to_insert, skip_ws)
}

/// FFI wrapper for is_join_whitespace.
#[no_mangle]
pub extern "C" fn rs_is_join_whitespace(c: c_int) -> c_int {
    c_int::from(is_join_whitespace(c))
}

/// FFI wrapper for would_exceed_line_limit.
#[no_mangle]
pub extern "C" fn rs_would_exceed_line_limit(
    current_size: c_int,
    add_size: c_int,
    max_line_len: c_int,
) -> c_int {
    c_int::from(would_exceed_line_limit(
        current_size,
        add_size,
        max_line_len,
    ))
}

/// FFI wrapper for is_wide_char.
#[no_mangle]
pub extern "C" fn rs_is_wide_char(c: c_int) -> c_int {
    c_int::from(is_wide_char(c))
}

// =============================================================================
// Phase O1: Additional Join Operation Helpers
// =============================================================================

/// Check if join should remove comment leaders.
///
/// Comment leaders are removed when:
/// - use_formatoptions is true
/// - FO_REMOVE_COMS is set in formatoptions
///
/// # Arguments
/// * `use_formatoptions` - Whether to use formatoptions
/// * `fo_remove_coms` - Whether FO_REMOVE_COMS is set
///
/// # Returns
/// true if comment leaders should be removed
#[must_use]
#[inline]
pub const fn should_remove_comments(use_formatoptions: bool, fo_remove_coms: bool) -> bool {
    use_formatoptions && fo_remove_coms
}

/// Calculate cursor column after join operation.
///
/// After joining, the cursor is positioned based on 'cpoptions':
/// - With CPO_JOINCOL ('j'): at currsize (first column of joined text)
/// - Without: at the end of the first line (before join point)
///
/// # Arguments
/// * `sumsize` - Total size after join
/// * `currsize` - Size of last joined line
/// * `last_spaces` - Spaces before last line
/// * `cpo_joincol` - Whether CPO_JOINCOL is set
///
/// # Returns
/// Cursor column position
#[must_use]
#[inline]
pub const fn calc_cursor_col_after_join(
    sumsize: c_int,
    currsize: c_int,
    last_spaces: c_int,
    cpo_joincol: bool,
) -> c_int {
    if cpo_joincol {
        currsize
    } else {
        sumsize - currsize - last_spaces
    }
}

/// Calculate mark column adjustment for joined lines.
///
/// When joining lines, marks need to be adjusted. This calculates the
/// column adjustment amount.
///
/// # Arguments
/// * `cend_offset` - Current position in new line (cend - newp)
/// * `spaces_removed` - Number of spaces that were removed
///
/// # Returns
/// Column amount for mark adjustment
#[must_use]
#[inline]
pub const fn calc_mark_col_adjust(cend_offset: c_int, spaces_removed: c_int) -> c_int {
    cend_offset - spaces_removed
}

/// Calculate total join buffer size needed.
///
/// # Arguments
/// * `total_text_size` - Sum of all line sizes
/// * `total_spaces` - Sum of all spaces to insert
///
/// # Returns
/// Total buffer size needed (excluding NUL terminator)
#[must_use]
#[inline]
pub const fn calc_join_buffer_size(total_text_size: c_int, total_spaces: c_int) -> c_int {
    total_text_size + total_spaces
}

/// Check if line should be skipped for comment removal.
///
/// The first line is never processed for comment removal.
/// Subsequent lines only have comments removed if previous was a comment.
///
/// # Arguments
/// * `line_index` - Index of current line (0-based)
/// * `prev_was_comment` - Whether previous line was a comment
///
/// # Returns
/// true if comment removal should be attempted
#[must_use]
#[inline]
pub const fn should_try_remove_comment(line_index: c_int, prev_was_comment: bool) -> bool {
    line_index > 0 && prev_was_comment
}

/// Check if join operation should save undo state.
///
/// # Arguments
/// * `save_undo` - Whether undo save was requested
///
/// # Returns
/// true if undo should be saved
#[must_use]
#[inline]
pub const fn should_save_join_undo(save_undo: bool) -> bool {
    save_undo
}

/// Calculate the undo save range for join.
///
/// # Arguments
/// * `cursor_lnum` - Current cursor line number
/// * `count` - Number of lines to join
///
/// # Returns
/// `(start_lnum, end_lnum)` for u_save
#[must_use]
#[inline]
pub const fn calc_join_undo_range(cursor_lnum: c_int, count: c_int) -> (c_int, c_int) {
    (cursor_lnum - 1, cursor_lnum + count)
}

/// Calculate lines to delete after join.
///
/// After joining, we delete (count - 1) lines since they're merged.
///
/// # Arguments
/// * `count` - Number of lines joined
///
/// # Returns
/// Number of lines to delete
#[must_use]
#[inline]
pub const fn calc_lines_to_delete_after_join(count: c_int) -> c_int {
    if count > 0 {
        count - 1
    } else {
        0
    }
}

/// Calculate extmark splice parameters for join.
///
/// # Arguments
/// * `removed_chars` - Characters removed from line start
/// * `spaces_to_insert` - Spaces to insert
///
/// # Returns
/// `(old_col, old_byte, new_col, new_byte)` for extmark_splice
#[must_use]
#[inline]
pub const fn calc_join_extmark_params(
    removed_chars: c_int,
    spaces_to_insert: c_int,
) -> (c_int, c_int, c_int, c_int) {
    (
        removed_chars,
        removed_chars + 1, // +1 for newline
        spaces_to_insert,
        spaces_to_insert,
    )
}

/// Check if this is the first line in a join (no spaces added).
///
/// # Arguments
/// * `sumsize` - Running sum size (0 for first line)
///
/// # Returns
/// true if this is the first line
#[must_use]
#[inline]
pub const fn is_first_join_line(sumsize: c_int) -> bool {
    sumsize == 0
}

/// Get last two characters of a string for joinspaces check.
///
/// This is a helper for the C code that calls calc_join_spaces.
/// Returns (0, 0) for empty strings.
///
/// # Arguments
/// * `len` - String length
/// * `last_char` - Last character
/// * `second_last_char` - Second to last character
///
/// # Returns
/// `(endcurr1, endcurr2)` for join spaces calculation
#[must_use]
#[inline]
pub const fn get_join_end_chars(
    len: c_int,
    last_char: c_int,
    second_last_char: c_int,
) -> (c_int, c_int) {
    if len == 0 {
        (0, 0)
    } else if len == 1 {
        (last_char, 0)
    } else {
        (last_char, second_last_char)
    }
}

// =============================================================================
// FFI Wrappers for Phase O1 Additions
// =============================================================================

/// FFI wrapper for should_remove_comments.
#[no_mangle]
pub extern "C" fn rs_should_remove_comments(
    use_formatoptions: c_int,
    fo_remove_coms: c_int,
) -> c_int {
    c_int::from(should_remove_comments(
        use_formatoptions != 0,
        fo_remove_coms != 0,
    ))
}

/// FFI wrapper for calc_cursor_col_after_join.
#[no_mangle]
pub extern "C" fn rs_calc_cursor_col_after_join(
    sumsize: c_int,
    currsize: c_int,
    last_spaces: c_int,
    cpo_joincol: c_int,
) -> c_int {
    calc_cursor_col_after_join(sumsize, currsize, last_spaces, cpo_joincol != 0)
}

/// FFI wrapper for calc_mark_col_adjust.
#[no_mangle]
pub extern "C" fn rs_calc_mark_col_adjust(cend_offset: c_int, spaces_removed: c_int) -> c_int {
    calc_mark_col_adjust(cend_offset, spaces_removed)
}

/// FFI wrapper for calc_join_buffer_size.
#[no_mangle]
pub extern "C" fn rs_calc_join_buffer_size(total_text_size: c_int, total_spaces: c_int) -> c_int {
    calc_join_buffer_size(total_text_size, total_spaces)
}

/// FFI wrapper for should_try_remove_comment.
#[no_mangle]
pub extern "C" fn rs_should_try_remove_comment(
    line_index: c_int,
    prev_was_comment: c_int,
) -> c_int {
    c_int::from(should_try_remove_comment(line_index, prev_was_comment != 0))
}

/// FFI wrapper for should_save_join_undo.
#[no_mangle]
pub extern "C" fn rs_should_save_join_undo(save_undo: c_int) -> c_int {
    c_int::from(should_save_join_undo(save_undo != 0))
}

/// FFI wrapper for calc_join_undo_range.
///
/// # Safety
/// `start_out` and `end_out` must be valid pointers if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_calc_join_undo_range(
    cursor_lnum: c_int,
    count: c_int,
    start_out: *mut c_int,
    end_out: *mut c_int,
) {
    let (start, end) = calc_join_undo_range(cursor_lnum, count);
    if !start_out.is_null() {
        unsafe {
            *start_out = start;
        }
    }
    if !end_out.is_null() {
        unsafe {
            *end_out = end;
        }
    }
}

/// FFI wrapper for calc_lines_to_delete_after_join.
#[no_mangle]
pub extern "C" fn rs_calc_lines_to_delete_after_join(count: c_int) -> c_int {
    calc_lines_to_delete_after_join(count)
}

/// FFI wrapper for is_first_join_line.
#[no_mangle]
pub extern "C" fn rs_is_first_join_line(sumsize: c_int) -> c_int {
    c_int::from(is_first_join_line(sumsize))
}

/// FFI wrapper for get_join_end_chars.
///
/// # Safety
/// `endcurr1_out` and `endcurr2_out` must be valid pointers if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_get_join_end_chars(
    len: c_int,
    last_char: c_int,
    second_last_char: c_int,
    endcurr1_out: *mut c_int,
    endcurr2_out: *mut c_int,
) {
    let (endcurr1, endcurr2) = get_join_end_chars(len, last_char, second_last_char);
    if !endcurr1_out.is_null() {
        unsafe {
            *endcurr1_out = endcurr1;
        }
    }
    if !endcurr2_out.is_null() {
        unsafe {
            *endcurr2_out = endcurr2;
        }
    }
}

#[cfg(test)]
#[allow(clippy::cast_lossless)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sentence_end() {
        assert!(is_sentence_end(b'.' as c_int));
        assert!(is_sentence_end(b'?' as c_int));
        assert!(is_sentence_end(b'!' as c_int));
        assert!(!is_sentence_end(b',' as c_int));
        assert!(!is_sentence_end(b' ' as c_int));
        assert!(!is_sentence_end(b'a' as c_int));
    }

    #[test]
    fn test_is_wide_char() {
        assert!(!is_wide_char(b'a' as c_int));
        assert!(!is_wide_char(0xFF));
        assert!(is_wide_char(0x100));
        assert!(is_wide_char(0x3042)); // Hiragana
    }

    #[test]
    fn test_calc_join_spaces_basic() {
        // Normal join: word<space>word
        let spaces = calc_join_spaces(
            b'a' as c_int, // endcurr1
            0,             // endcurr2
            b'b' as c_int, // firstcurr
            10,            // sumsize
            false,         // joinspaces
            false,         // fo_mbyte_join
            false,         // fo_mbyte_join2
        );
        assert_eq!(spaces, 1);
    }

    #[test]
    fn test_calc_join_spaces_first_line() {
        // First line (sumsize == 0): no spaces
        let spaces = calc_join_spaces(
            b'a' as c_int,
            0,
            b'b' as c_int,
            0, // sumsize = 0
            false,
            false,
            false,
        );
        assert_eq!(spaces, 0);
    }

    #[test]
    fn test_calc_join_spaces_empty_next() {
        // Empty next line (firstcurr == NUL): no spaces
        let spaces = calc_join_spaces(
            b'a' as c_int,
            0,
            NUL, // firstcurr
            10,
            false,
            false,
            false,
        );
        assert_eq!(spaces, 0);
    }

    #[test]
    fn test_calc_join_spaces_paren() {
        // Next line starts with ')': no spaces
        let spaces = calc_join_spaces(
            b'a' as c_int,
            0,
            b')' as c_int, // firstcurr
            10,
            false,
            false,
            false,
        );
        assert_eq!(spaces, 0);
    }

    #[test]
    fn test_calc_join_spaces_tab() {
        // Previous line ends with tab: no spaces
        let spaces = calc_join_spaces(
            TAB, // endcurr1
            0,
            b'b' as c_int,
            10,
            false,
            false,
            false,
        );
        assert_eq!(spaces, 0);
    }

    #[test]
    fn test_calc_join_spaces_joinspaces() {
        // 'joinspaces' after period: 2 spaces
        let spaces = calc_join_spaces(
            b'.' as c_int, // endcurr1
            0,
            b'T' as c_int, // firstcurr
            10,
            true, // joinspaces
            false,
            false,
        );
        assert_eq!(spaces, 2);

        // 'joinspaces' after question mark
        let spaces = calc_join_spaces(b'?' as c_int, 0, b'W' as c_int, 10, true, false, false);
        assert_eq!(spaces, 2);

        // 'joinspaces' after exclamation mark
        let spaces = calc_join_spaces(b'!' as c_int, 0, b'H' as c_int, 10, true, false, false);
        assert_eq!(spaces, 2);

        // 'joinspaces' with regular char: still 1 space
        let spaces = calc_join_spaces(b'a' as c_int, 0, b'b' as c_int, 10, true, false, false);
        assert_eq!(spaces, 1);
    }

    #[test]
    fn test_calc_join_spaces_trailing_space() {
        // Previous line ends in space: no additional space added
        // but joinspaces uses endcurr2
        let spaces = calc_join_spaces(
            b' ' as c_int, // endcurr1 (space)
            b'a' as c_int, // endcurr2
            b'b' as c_int,
            10,
            false,
            false,
            false,
        );
        assert_eq!(spaces, 0);

        // Trailing space with joinspaces, sentence end before space
        let spaces = calc_join_spaces(
            b' ' as c_int, // endcurr1 (space)
            b'.' as c_int, // endcurr2 (sentence end)
            b'T' as c_int,
            10,
            true, // joinspaces
            false,
            false,
        );
        assert_eq!(spaces, 1); // Only joinspaces extra, not base space
    }

    #[test]
    fn test_calc_join_spaces_mbyte_join() {
        // FO_MBYTE_JOIN: no space between wide chars
        let spaces = calc_join_spaces(
            0x3042, // endcurr1 (wide)
            0,
            b'a' as c_int, // firstcurr (ASCII)
            10,
            false,
            true, // fo_mbyte_join
            false,
        );
        assert_eq!(spaces, 0);

        let spaces = calc_join_spaces(
            b'a' as c_int, // endcurr1 (ASCII)
            0,
            0x3042, // firstcurr (wide)
            10,
            false,
            true, // fo_mbyte_join
            false,
        );
        assert_eq!(spaces, 0);

        // Both ASCII: normal space
        let spaces = calc_join_spaces(
            b'a' as c_int,
            0,
            b'b' as c_int,
            10,
            false,
            true, // fo_mbyte_join
            false,
        );
        assert_eq!(spaces, 1);
    }

    #[test]
    fn test_calc_join_total_size() {
        let sizes = [10, 20, 15];
        let spaces = [0, 1, 2];
        let total = calc_join_total_size(&sizes, &spaces);
        assert_eq!(total, 48); // 10 + 0 + 20 + 1 + 15 + 2
    }

    #[test]
    fn test_calc_join_cursor_col() {
        // Without CPO_JOINCOL: cursor at last join position
        let col = calc_join_cursor_col(
            100, // sumsize
            20,  // currsize
            1,   // last_spaces
            false,
        );
        assert_eq!(col, 100 - 20 - 1); // 79

        // With CPO_JOINCOL: cursor at first column of joined text
        let col = calc_join_cursor_col(
            100, // sumsize
            20,  // currsize
            1,   // last_spaces
            true,
        );
        assert_eq!(col, 20);
    }

    #[test]
    fn test_ffi_wrappers() {
        assert_eq!(rs_is_sentence_end(b'.' as c_int), 1);
        assert_eq!(rs_is_sentence_end(b'a' as c_int), 0);

        assert_eq!(
            rs_calc_join_spaces(b'a' as c_int, 0, b'b' as c_int, 10, 0, 0, 0),
            1
        );

        assert_eq!(rs_calc_join_cursor_col(100, 20, 1, 0), 79);
        assert_eq!(rs_calc_join_cursor_col(100, 20, 1, 1), 20);
    }

    // =========================================================================
    // Additional Helper Function Tests
    // =========================================================================

    #[test]
    fn test_is_gj_mode() {
        // gJ mode is when NOT using formatoptions
        assert!(is_gj_mode(false));
        assert!(!is_gj_mode(true));
    }

    #[test]
    fn test_calc_skip_whitespace() {
        // When skipping, return the count
        assert_eq!(calc_skip_whitespace(true, 5), 5);
        assert_eq!(calc_skip_whitespace(true, 0), 0);

        // When not skipping, always return 0
        assert_eq!(calc_skip_whitespace(false, 5), 0);
        assert_eq!(calc_skip_whitespace(false, 100), 0);
    }

    #[test]
    fn test_calc_joined_line_size() {
        // Normal case: 10 + 20 + 1 space - 2 skipped = 29
        assert_eq!(calc_joined_line_size(10, 20, 1, 2), 29);

        // No spaces inserted, no whitespace skipped
        assert_eq!(calc_joined_line_size(10, 20, 0, 0), 30);

        // All whitespace skipped
        assert_eq!(calc_joined_line_size(10, 5, 1, 5), 11);

        // Edge case: negative result clamped to 0
        assert_eq!(calc_joined_line_size(0, 0, 0, 10), 0);
    }

    #[test]
    fn test_is_join_whitespace() {
        assert!(is_join_whitespace(b' ' as c_int));
        assert!(is_join_whitespace(TAB));
        assert!(!is_join_whitespace(b'a' as c_int));
        assert!(!is_join_whitespace(b'\n' as c_int));
        assert!(!is_join_whitespace(0));
    }

    #[test]
    fn test_would_exceed_line_limit() {
        // Normal case: within limit
        assert!(!would_exceed_line_limit(50, 30, 100));

        // Exactly at limit
        assert!(!would_exceed_line_limit(50, 50, 100));

        // Exceeds limit
        assert!(would_exceed_line_limit(50, 51, 100));

        // Already at limit
        assert!(would_exceed_line_limit(100, 1, 100));

        // Overflow protection: large values
        assert!(would_exceed_line_limit(i32::MAX - 10, 20, i32::MAX));
    }

    #[test]
    fn test_additional_ffi_wrappers() {
        // rs_is_gj_mode
        assert_eq!(rs_is_gj_mode(0), 1); // false -> gJ mode
        assert_eq!(rs_is_gj_mode(1), 0); // true -> not gJ mode

        // rs_calc_skip_whitespace
        assert_eq!(rs_calc_skip_whitespace(1, 5), 5);
        assert_eq!(rs_calc_skip_whitespace(0, 5), 0);

        // rs_calc_joined_line_size
        assert_eq!(rs_calc_joined_line_size(10, 20, 1, 2), 29);

        // rs_is_join_whitespace
        assert_eq!(rs_is_join_whitespace(b' ' as c_int), 1);
        assert_eq!(rs_is_join_whitespace(b'a' as c_int), 0);

        // rs_would_exceed_line_limit
        assert_eq!(rs_would_exceed_line_limit(50, 30, 100), 0);
        assert_eq!(rs_would_exceed_line_limit(50, 51, 100), 1);

        // rs_is_wide_char
        assert_eq!(rs_is_wide_char(b'a' as c_int), 0);
        assert_eq!(rs_is_wide_char(0x100), 1);
    }

    // =========================================================================
    // Phase O1 Addition Tests
    // =========================================================================

    #[test]
    fn test_should_remove_comments() {
        // Both true
        assert!(should_remove_comments(true, true));

        // Either false
        assert!(!should_remove_comments(false, true));
        assert!(!should_remove_comments(true, false));
        assert!(!should_remove_comments(false, false));
    }

    #[test]
    fn test_calc_cursor_col_after_join() {
        // With CPO_JOINCOL: use currsize
        assert_eq!(calc_cursor_col_after_join(100, 20, 1, true), 20);

        // Without CPO_JOINCOL: sumsize - currsize - last_spaces
        assert_eq!(calc_cursor_col_after_join(100, 20, 1, false), 79);
    }

    #[test]
    fn test_calc_mark_col_adjust() {
        assert_eq!(calc_mark_col_adjust(50, 5), 45);
        assert_eq!(calc_mark_col_adjust(100, 0), 100);
    }

    #[test]
    fn test_calc_join_buffer_size() {
        assert_eq!(calc_join_buffer_size(100, 10), 110);
        assert_eq!(calc_join_buffer_size(0, 0), 0);
    }

    #[test]
    fn test_should_try_remove_comment() {
        // First line: never remove
        assert!(!should_try_remove_comment(0, true));
        assert!(!should_try_remove_comment(0, false));

        // Later lines: only if prev was comment
        assert!(should_try_remove_comment(1, true));
        assert!(!should_try_remove_comment(1, false));
        assert!(should_try_remove_comment(5, true));
    }

    #[test]
    fn test_should_save_join_undo() {
        assert!(should_save_join_undo(true));
        assert!(!should_save_join_undo(false));
    }

    #[test]
    fn test_calc_join_undo_range() {
        let (start, end) = calc_join_undo_range(10, 5);
        assert_eq!(start, 9);
        assert_eq!(end, 15);
    }

    #[test]
    fn test_calc_lines_to_delete_after_join() {
        assert_eq!(calc_lines_to_delete_after_join(5), 4);
        assert_eq!(calc_lines_to_delete_after_join(1), 0);
        assert_eq!(calc_lines_to_delete_after_join(0), 0);
    }

    #[test]
    fn test_calc_join_extmark_params() {
        let (old_col, old_byte, new_col, new_byte) = calc_join_extmark_params(5, 2);
        assert_eq!(old_col, 5);
        assert_eq!(old_byte, 6); // +1 for newline
        assert_eq!(new_col, 2);
        assert_eq!(new_byte, 2);
    }

    #[test]
    fn test_is_first_join_line() {
        assert!(is_first_join_line(0));
        assert!(!is_first_join_line(1));
        assert!(!is_first_join_line(100));
    }

    #[test]
    fn test_get_join_end_chars() {
        // Empty string
        let (e1, e2) = get_join_end_chars(0, 0, 0);
        assert_eq!(e1, 0);
        assert_eq!(e2, 0);

        // Single char
        let (e1, e2) = get_join_end_chars(1, b'a' as c_int, 0);
        assert_eq!(e1, b'a' as c_int);
        assert_eq!(e2, 0);

        // Multiple chars
        let (e1, e2) = get_join_end_chars(5, b'.' as c_int, b'x' as c_int);
        assert_eq!(e1, b'.' as c_int);
        assert_eq!(e2, b'x' as c_int);
    }

    #[test]
    fn test_phase_o1_join_ffi_wrappers() {
        // rs_should_remove_comments
        assert_eq!(rs_should_remove_comments(1, 1), 1);
        assert_eq!(rs_should_remove_comments(0, 1), 0);

        // rs_calc_cursor_col_after_join
        assert_eq!(rs_calc_cursor_col_after_join(100, 20, 1, 1), 20);
        assert_eq!(rs_calc_cursor_col_after_join(100, 20, 1, 0), 79);

        // rs_calc_mark_col_adjust
        assert_eq!(rs_calc_mark_col_adjust(50, 5), 45);

        // rs_calc_join_buffer_size
        assert_eq!(rs_calc_join_buffer_size(100, 10), 110);

        // rs_should_try_remove_comment
        assert_eq!(rs_should_try_remove_comment(0, 1), 0);
        assert_eq!(rs_should_try_remove_comment(1, 1), 1);

        // rs_should_save_join_undo
        assert_eq!(rs_should_save_join_undo(1), 1);
        assert_eq!(rs_should_save_join_undo(0), 0);

        // rs_calc_lines_to_delete_after_join
        assert_eq!(rs_calc_lines_to_delete_after_join(5), 4);

        // rs_is_first_join_line
        assert_eq!(rs_is_first_join_line(0), 1);
        assert_eq!(rs_is_first_join_line(1), 0);
    }
}
