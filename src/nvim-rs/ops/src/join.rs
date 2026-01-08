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
}
