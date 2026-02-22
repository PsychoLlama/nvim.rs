//! Quickfix window display
//!
//! This module provides display formatting and rendering for quickfix and
//! location list windows.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::derivable_impls)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Display Constants
// =============================================================================

/// Maximum filename display width
pub const QF_FNAME_MAX_WIDTH: usize = 50;

/// Maximum text display width
pub const QF_TEXT_MAX_WIDTH: usize = 200;

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to quickfix list
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QfListHandle(*mut c_void);

impl QfListHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque handle to quickfix entry
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QfEntryHandle(*mut c_void);

impl QfEntryHandle {
    /// Check if the handle is null
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle
    #[inline]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

// =============================================================================
// Display Entry Info
// =============================================================================

/// Information about a quickfix entry for display purposes
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfDisplayEntry {
    /// Entry index (1-based for display)
    pub index: c_int,
    /// File number
    pub fnum: c_int,
    /// Line number
    pub lnum: i32,
    /// Column number
    pub col: c_int,
    /// End line number
    pub end_lnum: i32,
    /// End column number
    pub end_col: c_int,
    /// Error type character ('E', 'W', 'I', 'N', or ' ')
    pub type_char: u8,
    /// Error number
    pub nr: c_int,
    /// Whether entry is valid (has file/line)
    pub valid: bool,
    /// Whether this is the current entry
    pub is_current: bool,
}

impl Default for QfDisplayEntry {
    fn default() -> Self {
        Self {
            index: 0,
            fnum: 0,
            lnum: 0,
            col: 0,
            end_lnum: 0,
            end_col: 0,
            type_char: b' ',
            nr: 0,
            valid: false,
            is_current: false,
        }
    }
}

// =============================================================================
// Display Format
// =============================================================================

/// Format style for quickfix display
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QfDisplayFormat {
    /// Standard format: "filename|line col|message"
    Standard = 0,
    /// Compact format: "filename:line:col: message"
    Compact = 1,
    /// Long format: "filename|line col type nr|message"
    Long = 2,
}

impl Default for QfDisplayFormat {
    fn default() -> Self {
        Self::Standard
    }
}

// =============================================================================
// Display State
// =============================================================================

/// State of the quickfix window display
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QfDisplayState {
    /// Total number of entries
    pub total_entries: c_int,
    /// Number of valid entries
    pub valid_entries: c_int,
    /// Current entry index (1-based)
    pub current_idx: c_int,
    /// First visible entry index
    pub first_visible: c_int,
    /// Number of visible lines in window
    pub visible_lines: c_int,
    /// Whether there are entries above the visible area
    pub has_entries_above: bool,
    /// Whether there are entries below the visible area
    pub has_entries_below: bool,
}

impl Default for QfDisplayState {
    fn default() -> Self {
        Self {
            total_entries: 0,
            valid_entries: 0,
            current_idx: 0,
            first_visible: 1,
            visible_lines: 0,
            has_entries_above: false,
            has_entries_below: false,
        }
    }
}

impl QfDisplayState {
    /// Check if display has entries
    pub const fn has_entries(&self) -> bool {
        self.total_entries > 0
    }

    /// Check if there are more entries than can fit in the window
    pub const fn needs_scrolling(&self) -> bool {
        self.total_entries > self.visible_lines
    }

    /// Calculate the scroll percentage (0-100)
    pub fn scroll_percent(&self) -> u8 {
        if self.total_entries <= self.visible_lines {
            return 100;
        }
        let visible_end = self.first_visible + self.visible_lines - 1;
        let percent = (visible_end * 100) / self.total_entries;
        if percent > 100 {
            100
        } else {
            percent as u8
        }
    }
}

// =============================================================================
// Position Formatting
// =============================================================================

/// Format a position (line:col) for display
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PositionFormat {
    /// Whether to include column
    pub include_col: bool,
    /// Whether to include range (end_lnum:end_col)
    pub include_range: bool,
    /// Minimum line number width for alignment
    pub min_lnum_width: u8,
    /// Minimum column width for alignment
    pub min_col_width: u8,
}

impl Default for PositionFormat {
    fn default() -> Self {
        Self {
            include_col: true,
            include_range: false,
            min_lnum_width: 0,
            min_col_width: 0,
        }
    }
}

/// Calculate the display width needed for a line number
pub const fn lnum_display_width(lnum: i32) -> u8 {
    if lnum < 10 {
        1
    } else if lnum < 100 {
        2
    } else if lnum < 1000 {
        3
    } else if lnum < 10000 {
        4
    } else if lnum < 100_000 {
        5
    } else if lnum < 1_000_000 {
        6
    } else {
        7
    }
}

// =============================================================================
// Type Character Formatting
// =============================================================================

/// Get the display character for an error type
pub const fn type_display_char(type_code: u8) -> u8 {
    match type_code {
        b'E' | b'e' => b'E',
        b'W' | b'w' => b'W',
        b'I' | b'i' => b'I',
        b'N' | b'n' => b'N',
        _ => b' ',
    }
}

/// Check if a type character should be highlighted as error
pub const fn is_error_type(type_code: u8) -> bool {
    matches!(type_code, b'E' | b'e')
}

/// Check if a type character should be highlighted as warning
pub const fn is_warning_type(type_code: u8) -> bool {
    matches!(type_code, b'W' | b'w')
}

// =============================================================================
// FFI Exports
// =============================================================================

/// FFI export: Calculate line number display width
#[no_mangle]
pub extern "C" fn rs_qf_lnum_display_width(lnum: i32) -> c_int {
    c_int::from(lnum_display_width(lnum))
}

/// FFI export: Get type display character
#[no_mangle]
pub extern "C" fn rs_qf_type_display_char(type_code: u8) -> u8 {
    type_display_char(type_code)
}

/// FFI export: Check if error type
#[no_mangle]
pub extern "C" fn rs_qf_is_error_type(type_code: u8) -> c_int {
    c_int::from(is_error_type(type_code))
}

/// FFI export: Check if warning type
#[no_mangle]
pub extern "C" fn rs_qf_is_warning_type(type_code: u8) -> c_int {
    c_int::from(is_warning_type(type_code))
}

/// FFI export: Calculate scroll percent
#[no_mangle]
pub extern "C" fn rs_qf_display_scroll_percent(
    total: c_int,
    first_visible: c_int,
    visible_lines: c_int,
) -> c_int {
    let state = QfDisplayState {
        total_entries: total,
        valid_entries: 0,
        current_idx: 0,
        first_visible,
        visible_lines,
        has_entries_above: false,
        has_entries_below: false,
    };
    c_int::from(state.scroll_percent())
}

// =============================================================================
// Phase 3: qf_fill_buffer
// =============================================================================

type LinenrT = i32;

/// Opaque buffer handle
type BufHandle = *mut c_void;
/// Opaque qfline handle
type QfLinePtr = *const c_void;
/// Opaque list handle
type ListPtr = *mut c_void;
/// Opaque list item handle
type ListItemPtr = *const c_void;

const MAXPATHL: usize = 4096;

extern "C" {
    fn nvim_qf_buf_is_curbuf(buf: BufHandle) -> bool;
    fn nvim_qf_fill_buffer_internal_error();
    fn nvim_qf_delete_all_lines() -> bool;
    fn nvim_qf_zero_skipcol_for_curbuf();
    fn nvim_qf_u_clearallandblockfree();
    fn nvim_qf_get_start_nonnull(qfl: *const c_void) -> QfLinePtr;
    fn nvim_qfline_get_next(qfp: QfLinePtr) -> QfLinePtr;
    fn nvim_qfline_get_fnum(qfp: QfLinePtr) -> c_int;
    fn nvim_qf_get_count(qfl: *const c_void) -> c_int;
    fn nvim_buf_get_line_count(buf: BufHandle) -> LinenrT;
    fn nvim_call_qftf_func(
        qfl: *mut c_void,
        qf_winid: c_int,
        start: LinenrT,
        count: c_int,
    ) -> ListPtr;
    fn nvim_tv_list_first(list: *const c_void) -> ListItemPtr;
    fn nvim_tv_list_item_next(list: *const c_void, li: ListItemPtr) -> ListItemPtr;
    fn nvim_tv_list_item_string(li: ListItemPtr) -> *mut c_char;
    fn nvim_qf_buf_add_line(
        qfl: *mut c_void,
        buf: BufHandle,
        lnum: LinenrT,
        qfp: *mut c_void,
        dirname: *mut c_char,
        qftf_str: *mut c_char,
        first_in_file: bool,
    ) -> c_int;
    fn nvim_ml_delete_one(lnum: LinenrT);
    fn nvim_qfga_clear();
    fn rs_check_lnums(do_curwin: c_int);
    fn nvim_qf_set_filetype_and_autocmds();
    fn nvim_qf_get_key_typed() -> bool;
    fn nvim_qf_set_key_typed(val: bool);
}

const FAIL: c_int = 0;

/// Fill the quickfix buffer with entries.
///
/// # Safety
///
/// - `buf` must be a valid pointer to a `buf_T`
/// - `qfl` may be NULL (in which case only cleanup is done)
/// - `old_last` may be NULL (full refresh) or a valid `qfline_T` pointer
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub unsafe extern "C" fn rs_qf_fill_buffer(
    qfl: *mut c_void,
    buf: BufHandle,
    old_last: QfLinePtr,
    qf_winid: c_int,
) {
    let old_key_typed = nvim_qf_get_key_typed();

    if old_last.is_null() {
        if !nvim_qf_buf_is_curbuf(buf) {
            nvim_qf_fill_buffer_internal_error();
            return;
        }

        // Delete all existing lines
        if !nvim_qf_delete_all_lines() {
            return;
        }

        nvim_qf_zero_skipcol_for_curbuf();
        nvim_qf_u_clearallandblockfree();
    }

    // Check if there is anything to display
    let qf_start = nvim_qf_get_start_nonnull(qfl);
    if !qfl.is_null() && !qf_start.is_null() {
        let mut dirname = [0u8; MAXPATHL];

        let mut lnum: LinenrT;
        let mut qfp: QfLinePtr;

        // Add one line for each error
        if old_last.is_null() {
            qfp = qf_start;
            lnum = 0;
        } else {
            let next = nvim_qfline_get_next(old_last);
            qfp = if next.is_null() { old_last } else { next };
            lnum = nvim_buf_get_line_count(buf);
        }

        let qf_count = nvim_qf_get_count(qfl);
        let qftf_list = nvim_call_qftf_func(qfl, qf_winid, lnum + 1, qf_count);
        let mut qftf_li = nvim_tv_list_first(qftf_list.cast_const());

        let mut prev_bufnr: c_int = -1;
        let mut invalid_val = false;

        while lnum < qf_count {
            let mut qftf_str: *mut c_char = std::ptr::null_mut();

            // Use the text supplied by the user defined function (if any).
            if !qftf_li.is_null() && !invalid_val {
                qftf_str = nvim_tv_list_item_string(qftf_li);
                if qftf_str.is_null() {
                    invalid_val = true;
                }
            }

            if nvim_qf_buf_add_line(
                qfl,
                buf,
                lnum,
                qfp.cast_mut().cast(),
                dirname.as_mut_ptr().cast(),
                qftf_str,
                prev_bufnr != nvim_qfline_get_fnum(qfp),
            ) == FAIL
            {
                break;
            }
            prev_bufnr = nvim_qfline_get_fnum(qfp);
            lnum += 1;
            qfp = nvim_qfline_get_next(qfp);
            if qfp.is_null() {
                break;
            }

            if !qftf_li.is_null() {
                qftf_li = nvim_tv_list_item_next(qftf_list.cast_const(), qftf_li);
            }
        }

        if old_last.is_null() {
            // Delete the empty line which is now at the end
            nvim_ml_delete_one(lnum + 1);
        }

        nvim_qfga_clear();
    }

    // Correct cursor position.
    rs_check_lnums(1);

    if old_last.is_null() {
        nvim_qf_set_filetype_and_autocmds();
    }

    // Restore KeyTyped, setting 'filetype' may reset it.
    nvim_qf_set_key_typed(old_key_typed);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lnum_display_width() {
        assert_eq!(lnum_display_width(0), 1);
        assert_eq!(lnum_display_width(1), 1);
        assert_eq!(lnum_display_width(9), 1);
        assert_eq!(lnum_display_width(10), 2);
        assert_eq!(lnum_display_width(99), 2);
        assert_eq!(lnum_display_width(100), 3);
        assert_eq!(lnum_display_width(1000), 4);
        assert_eq!(lnum_display_width(10000), 5);
    }

    #[test]
    fn test_type_display_char() {
        assert_eq!(type_display_char(b'E'), b'E');
        assert_eq!(type_display_char(b'e'), b'E');
        assert_eq!(type_display_char(b'W'), b'W');
        assert_eq!(type_display_char(b'w'), b'W');
        assert_eq!(type_display_char(b'I'), b'I');
        assert_eq!(type_display_char(b'N'), b'N');
        assert_eq!(type_display_char(b'X'), b' ');
        assert_eq!(type_display_char(0), b' ');
    }

    #[test]
    fn test_is_error_type() {
        assert!(is_error_type(b'E'));
        assert!(is_error_type(b'e'));
        assert!(!is_error_type(b'W'));
        assert!(!is_error_type(b'I'));
    }

    #[test]
    fn test_is_warning_type() {
        assert!(is_warning_type(b'W'));
        assert!(is_warning_type(b'w'));
        assert!(!is_warning_type(b'E'));
        assert!(!is_warning_type(b'I'));
    }

    #[test]
    fn test_display_state_has_entries() {
        let empty = QfDisplayState::default();
        assert!(!empty.has_entries());

        let with_entries = QfDisplayState {
            total_entries: 5,
            ..Default::default()
        };
        assert!(with_entries.has_entries());
    }

    #[test]
    fn test_display_state_needs_scrolling() {
        let fits = QfDisplayState {
            total_entries: 10,
            visible_lines: 20,
            ..Default::default()
        };
        assert!(!fits.needs_scrolling());

        let needs = QfDisplayState {
            total_entries: 30,
            visible_lines: 20,
            ..Default::default()
        };
        assert!(needs.needs_scrolling());
    }

    #[test]
    fn test_scroll_percent() {
        // All visible
        let all = QfDisplayState {
            total_entries: 10,
            visible_lines: 20,
            first_visible: 1,
            ..Default::default()
        };
        assert_eq!(all.scroll_percent(), 100);

        // Halfway through
        let half = QfDisplayState {
            total_entries: 100,
            visible_lines: 20,
            first_visible: 31,
            ..Default::default()
        };
        assert_eq!(half.scroll_percent(), 50);

        // At end
        let end = QfDisplayState {
            total_entries: 100,
            visible_lines: 20,
            first_visible: 81,
            ..Default::default()
        };
        assert_eq!(end.scroll_percent(), 100);
    }

    #[test]
    fn test_display_format_default() {
        assert_eq!(QfDisplayFormat::default(), QfDisplayFormat::Standard);
    }

    #[test]
    fn test_display_entry_default() {
        let entry = QfDisplayEntry::default();
        assert_eq!(entry.index, 0);
        assert_eq!(entry.type_char, b' ');
        assert!(!entry.valid);
        assert!(!entry.is_current);
    }
}
