//! Line printing utilities
//!
//! Provides helpers for printing lines with the :print and :list commands,
//! handling special characters, tabs, and list mode display.

use std::ffi::c_int;

// C function declarations
extern "C" {
    static Columns: c_int;
    static mut got_int: bool;
    /// Check if in list mode (curwin->w_p_list)
    fn nvim_curwin_w_p_list() -> c_int;
    /// Get current message column
    static mut msg_col: c_int;
}

// ============================================================================
// Line Display Constants
// ============================================================================

/// Default tab width
pub const DEFAULT_TAB_WIDTH: c_int = 8;

/// Character for end of line in list mode (typically $)
pub const LIST_EOL_CHAR: c_int = b'$' as c_int;

/// Character for tab in list mode (typically >)
pub const LIST_TAB_CHAR: c_int = b'>' as c_int;

/// Character for trailing space in list mode (typically ·)
pub const LIST_TRAIL_CHAR: c_int = b'.' as c_int;

// ============================================================================
// Tab Handling
// ============================================================================

/// Calculate tab padding based on current column.
///
/// Returns the number of spaces to the next tab stop.
#[no_mangle]
pub const extern "C" fn rs_tab_padding(col: c_int, tabstop: c_int) -> c_int {
    if tabstop <= 0 {
        return DEFAULT_TAB_WIDTH - (col % DEFAULT_TAB_WIDTH);
    }
    tabstop - (col % tabstop)
}

/// Calculate the column position after a tab.
#[no_mangle]
pub const extern "C" fn rs_tab_next_col(col: c_int, tabstop: c_int) -> c_int {
    col + rs_tab_padding(col, tabstop)
}

/// Check if column is at a tab stop.
#[no_mangle]
pub const extern "C" fn rs_is_tab_stop(col: c_int, tabstop: c_int) -> c_int {
    if tabstop <= 0 {
        return (col % DEFAULT_TAB_WIDTH == 0) as c_int;
    }
    (col % tabstop == 0) as c_int
}

// ============================================================================
// Line State Helpers
// ============================================================================

/// Check if line printing should continue.
///
/// Returns false if interrupted (unsafe { got_int } set).
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_line_continue() -> c_int {
    c_int::from(!unsafe { got_int })
}

/// Calculate remaining columns on current line.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_line_remaining() -> c_int {
    let columns = Columns;
    if msg_col < columns {
        columns - msg_col
    } else {
        0
    }
}

/// Check if current position is at end of line.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_at_line_end() -> c_int {
    c_int::from(rs_line_remaining() == 0)
}

// ============================================================================
// Character Classification for Line Printing
// ============================================================================

// Note: rs_is_whitespace is defined in edit/insert.rs

/// Check if character is a space.
#[no_mangle]
pub const extern "C" fn rs_is_space(c: c_int) -> c_int {
    (c == b' ' as c_int) as c_int
}

/// Check if character is a tab.
#[no_mangle]
pub const extern "C" fn rs_is_tab(c: c_int) -> c_int {
    (c == b'\t' as c_int) as c_int
}

/// Check if character is printable ASCII.
#[no_mangle]
pub const extern "C" fn rs_is_printable_ascii(c: c_int) -> c_int {
    (c >= 0x20 && c < 0x7F) as c_int
}

/// Check if character needs special display in list mode.
#[no_mangle]
pub const extern "C" fn rs_needs_list_display(c: c_int) -> c_int {
    // Needs special display: NUL, TAB, trailing space, non-printable
    (c == 0 || c == b'\t' as c_int || c < 0x20 || c >= 0x7F) as c_int
}

// ============================================================================
// List Mode Display
// ============================================================================

/// Get the display character for a control character in list mode.
///
/// Returns the character to show after ^ (e.g., ^I for tab).
#[no_mangle]
pub const extern "C" fn rs_ctrl_display_char(c: c_int) -> c_int {
    if c >= 0 && c < 0x20 {
        // Control characters display as ^@ through ^_
        b'@' as c_int + c
    } else if c == 0x7F {
        // DEL displays as ^?
        b'?' as c_int
    } else {
        // Not a control character
        c
    }
}

// Note: rs_char_cells is defined in edit/replace.rs

/// Calculate display width of a character for line printing.
///
/// Control characters take 2 cells (^X), TAB variable, others 1.
/// This is a const helper for use in this module.
const fn line_char_cells(c: c_int, col: c_int, tabstop: c_int) -> c_int {
    if c == b'\t' as c_int {
        rs_tab_padding(col, tabstop)
    } else if (c >= 0 && c < 0x20) || c == 0x7F {
        // Control characters (^X format) and DEL (^?)
        2
    } else {
        1
    }
}

/// Check if in list mode.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_in_list_mode() -> c_int {
    nvim_curwin_w_p_list()
}

// ============================================================================
// Column Tracking
// ============================================================================

/// Calculate new column after outputting a character.
#[no_mangle]
pub const extern "C" fn rs_advance_col(col: c_int, c: c_int, tabstop: c_int) -> c_int {
    col + line_char_cells(c, col, tabstop)
}

/// Calculate columns needed to display a string of given length.
///
/// Assumes all characters are single-width printable ASCII.
#[no_mangle]
pub const extern "C" fn rs_string_cells(len: c_int) -> c_int {
    len
}

// ============================================================================
// Phase 3: msg_prt_line migrated to Rust
// ============================================================================

/// Type alias for schar_T (u32 in C, types_defs.h)
type ScharT = u32;

/// MB_MAXBYTES constant (mbyte_defs.h)
const MB_MAX_BYTES: usize = 21;

/// MAX_SCHAR_SIZE constant (types_defs.h)
const MAX_SCHAR_SIZE: usize = 32;

extern "C" {
    // lcs_chars_T accessors
    fn nvim_lcs_eol() -> ScharT;
    fn nvim_lcs_trail() -> ScharT;
    fn nvim_lcs_lead() -> ScharT;
    fn nvim_lcs_has_leadmultispace() -> c_int;
    fn nvim_lcs_leadmultispace_at(idx: c_int) -> ScharT;
    fn nvim_lcs_tab1() -> ScharT;
    fn nvim_lcs_tab2() -> ScharT;
    fn nvim_lcs_tab3() -> ScharT;
    fn nvim_lcs_nbsp() -> ScharT;
    fn nvim_lcs_space() -> ScharT;
    fn nvim_lcs_has_multispace() -> c_int;
    fn nvim_lcs_multispace_at(idx: c_int) -> ScharT;
    fn nvim_curbuf_ts() -> i64;
    fn nvim_curbuf_vts_array() -> *mut c_int;
    fn nvim_schar_from_ascii(c: c_int) -> ScharT;
    fn nvim_hlf_at() -> c_int;
    fn nvim_hlf_0() -> c_int;

    // String/multibyte functions
    fn utfc_ptr2len(s: *const std::ffi::c_char) -> c_int;
    fn utf_ptr2cells(s: *const std::ffi::c_char) -> c_int;
    fn utf_ptr2char(s: *const std::ffi::c_char) -> c_int;
    fn utf_char2cells(c: c_int) -> c_int;
    fn schar_get(buf_out: *mut std::ffi::c_char, sc: ScharT) -> usize;
    fn xstrlcpy(dst: *mut std::ffi::c_char, src: *const std::ffi::c_char, dstsize: usize) -> usize;
    fn byte2cells(b: c_int) -> c_int;
    fn transchar_byte_buf(buf: *mut std::ffi::c_char, c: c_int) -> *mut std::ffi::c_char;
    fn tabstop_padding(col: c_int, ts_arg: i64, vts: *const c_int) -> c_int;

    // Output functions
    fn msg_puts(s: *const std::ffi::c_char);
    fn msg_putchar(c: c_int);
    fn msg_puts_hl(s: *const std::ffi::c_char, hl_id: c_int, hist: bool);
    fn msg_clr_eos();
}

/// Check if character is ASCII whitespace (space or tab).
///
/// Inline equivalent of C's `ascii_iswhite()` (which is a static inline function).
#[inline]
fn ascii_iswhite(c: c_int) -> bool {
    c == c_int::from(b' ') || c == c_int::from(b'\t')
}

/// Print a line for `:print` or `:list` commands.
///
/// Handles special character display in list mode (tabs, trailing spaces,
/// leading spaces, multispace patterns, non-breaking spaces, etc).
///
/// # Safety
/// - `s` must be a valid NUL-terminated C string
#[export_name = "msg_prt_line"]
#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::redundant_else,
    unused_assignments
)]
pub unsafe extern "C" fn rs_msg_prt_line(s_in: *const std::ffi::c_char, list_in: bool) {
    let mut sc: ScharT = 0;
    let mut col: c_int = 0;
    let mut n_extra: c_int = 0;
    let mut sc_extra: ScharT = 0;
    let mut sc_final: ScharT = 0;
    let mut p_extra: *const std::ffi::c_char = std::ptr::null();
    let mut hl_id: c_int = 0;
    let mut lead: *const std::ffi::c_char = std::ptr::null();
    let mut in_multispace = false;
    let mut multispace_pos: c_int = 0;
    let mut trail: *const std::ffi::c_char = std::ptr::null();

    let mut list = list_in;
    let mut s = s_in;

    if nvim_curwin_w_p_list() != 0 {
        list = true;
    }

    if list {
        // find start of trailing whitespace
        if nvim_lcs_trail() != 0 {
            // find NUL terminator
            let mut t = s;
            while *t != 0 {
                t = t.add(1);
            }
            trail = t;
            while trail > s && ascii_iswhite(c_int::from(*trail.sub(1) as u8)) {
                trail = trail.sub(1);
            }
        }
        // find end of leading whitespace
        if nvim_lcs_lead() != 0 || nvim_lcs_has_leadmultispace() != 0 {
            lead = s;
            while ascii_iswhite(c_int::from(*lead as u8)) {
                lead = lead.add(1);
            }
            // in a line full of spaces all treated as trailing
            if *lead == 0 {
                lead = std::ptr::null();
            }
        }
    }

    // output a space for an empty line, otherwise the line will be overwritten
    if *s == 0 && !(list && nvim_lcs_eol() != 0) {
        msg_putchar(c_int::from(b' '));
    }

    while !got_int {
        if n_extra > 0 {
            n_extra -= 1;
            sc = if n_extra == 0 && sc_final != 0 {
                sc_final
            } else if sc_extra != 0 {
                sc_extra
            } else {
                debug_assert!(!p_extra.is_null());
                let c = *p_extra as u8;
                p_extra = p_extra.add(1);
                nvim_schar_from_ascii(c_int::from(c))
            };
        } else {
            let l = utfc_ptr2len(s);
            if l > 1 {
                col += utf_ptr2cells(s);
                let mut buf = [0u8; MB_MAX_BYTES + 1];
                let buf_ptr = buf.as_mut_ptr().cast::<std::ffi::c_char>();
                if l >= MB_MAX_BYTES as c_int {
                    xstrlcpy(buf_ptr, c"?".as_ptr(), buf.len());
                } else if nvim_lcs_nbsp() != 0
                    && list
                    && (utf_ptr2char(s) == 160 || utf_ptr2char(s) == 0x202f)
                {
                    schar_get(buf_ptr, nvim_lcs_nbsp());
                } else {
                    std::ptr::copy_nonoverlapping(s.cast::<u8>(), buf.as_mut_ptr(), l as usize);
                    buf[l as usize] = 0;
                }
                msg_puts(buf_ptr);
                s = s.add(l as usize);
                continue;
            } else {
                hl_id = 0;
                let c = c_int::from(*s as u8);
                s = s.add(1);
                if c >= 0x80 {
                    // Illegal byte
                    col += utf_char2cells(c);
                    msg_putchar(c);
                    continue;
                }
                sc_extra = 0;
                sc_final = 0;
                if list {
                    in_multispace = c == c_int::from(b' ')
                        && (*s == b' ' as i8 || (col > 0 && *s.sub(2) == b' ' as i8));
                    if !in_multispace {
                        multispace_pos = 0;
                    }
                }
                if c == c_int::from(b'\t') && (!list || nvim_lcs_tab1() != 0) {
                    // tab amount depends on current column
                    n_extra = tabstop_padding(col, nvim_curbuf_ts(), nvim_curbuf_vts_array()) - 1;
                    if list {
                        sc = if n_extra == 0 && nvim_lcs_tab3() != 0 {
                            nvim_lcs_tab3()
                        } else {
                            nvim_lcs_tab1()
                        };
                        sc_extra = nvim_lcs_tab2();
                        sc_final = nvim_lcs_tab3();
                        hl_id = nvim_hlf_0();
                    } else {
                        sc = nvim_schar_from_ascii(c_int::from(b' '));
                        sc_extra = nvim_schar_from_ascii(c_int::from(b' '));
                    }
                } else if c == 0 && list && nvim_lcs_eol() != 0 {
                    p_extra = c"".as_ptr();
                    n_extra = 1;
                    sc = nvim_lcs_eol();
                    hl_id = nvim_hlf_at();
                    s = s.sub(1);
                } else if c != 0 && byte2cells(c) > 1 {
                    let n = byte2cells(c);
                    n_extra = n - 1;
                    p_extra = transchar_byte_buf(std::ptr::null_mut(), c).cast_const();
                    let c2 = *p_extra as u8;
                    p_extra = p_extra.add(1);
                    sc = nvim_schar_from_ascii(c_int::from(c2));
                    hl_id = nvim_hlf_0();
                } else if c == c_int::from(b' ') {
                    if !lead.is_null()
                        && s <= lead
                        && in_multispace
                        && nvim_lcs_has_leadmultispace() != 0
                    {
                        sc = nvim_lcs_leadmultispace_at(multispace_pos);
                        multispace_pos += 1;
                        if nvim_lcs_leadmultispace_at(multispace_pos) == 0 {
                            multispace_pos = 0;
                        }
                        hl_id = nvim_hlf_0();
                    } else if !lead.is_null() && s <= lead && nvim_lcs_lead() != 0 {
                        sc = nvim_lcs_lead();
                        hl_id = nvim_hlf_0();
                    } else if !trail.is_null() && s > trail {
                        sc = nvim_lcs_trail();
                        hl_id = nvim_hlf_0();
                    } else if in_multispace && nvim_lcs_has_multispace() != 0 {
                        sc = nvim_lcs_multispace_at(multispace_pos);
                        multispace_pos += 1;
                        if nvim_lcs_multispace_at(multispace_pos) == 0 {
                            multispace_pos = 0;
                        }
                        hl_id = nvim_hlf_0();
                    } else if list && nvim_lcs_space() != 0 {
                        sc = nvim_lcs_space();
                        hl_id = nvim_hlf_0();
                    } else {
                        sc = nvim_schar_from_ascii(c_int::from(b' '));
                    }
                } else {
                    sc = nvim_schar_from_ascii(c);
                }
            }
        }

        if sc == 0 {
            break;
        }

        // TODO(bfredl): this is such baloney. need msg_put_schar
        let mut buf = [0i8; MAX_SCHAR_SIZE];
        schar_get(buf.as_mut_ptr(), sc);
        msg_puts_hl(buf.as_ptr(), hl_id, false);
        col += 1;
    }
    msg_clr_eos();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_padding() {
        assert_eq!(rs_tab_padding(0, 8), 8);
        assert_eq!(rs_tab_padding(1, 8), 7);
        assert_eq!(rs_tab_padding(7, 8), 1);
        assert_eq!(rs_tab_padding(8, 8), 8);
    }

    #[test]
    fn test_tab_next_col() {
        assert_eq!(rs_tab_next_col(0, 8), 8);
        assert_eq!(rs_tab_next_col(1, 8), 8);
        assert_eq!(rs_tab_next_col(8, 8), 16);
    }

    #[test]
    fn test_is_tab_stop() {
        assert_eq!(rs_is_tab_stop(0, 8), 1);
        assert_eq!(rs_is_tab_stop(8, 8), 1);
        assert_eq!(rs_is_tab_stop(1, 8), 0);
        assert_eq!(rs_is_tab_stop(7, 8), 0);
    }

    #[test]
    fn test_is_printable_ascii() {
        assert_eq!(rs_is_printable_ascii(c_int::from(b' ')), 1);
        assert_eq!(rs_is_printable_ascii(c_int::from(b'~')), 1);
        assert_eq!(rs_is_printable_ascii(0x1F), 0);
        assert_eq!(rs_is_printable_ascii(0x7F), 0);
    }

    #[test]
    fn test_ctrl_display_char() {
        assert_eq!(rs_ctrl_display_char(0), c_int::from(b'@')); // ^@
        assert_eq!(rs_ctrl_display_char(9), c_int::from(b'I')); // ^I (tab)
        assert_eq!(rs_ctrl_display_char(0x7F), c_int::from(b'?')); // ^?
    }

    #[test]
    fn test_line_char_cells() {
        assert_eq!(line_char_cells(c_int::from(b'a'), 0, 8), 1);
        assert_eq!(line_char_cells(0, 0, 8), 2); // ^@
        assert_eq!(line_char_cells(c_int::from(b'\t'), 0, 8), 8);
        assert_eq!(line_char_cells(c_int::from(b'\t'), 4, 8), 4);
    }
}
