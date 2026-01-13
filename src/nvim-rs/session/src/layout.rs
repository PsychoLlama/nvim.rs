//! Window layout serialization for sessions
//!
//! This module provides helpers for serializing window layout state to session files,
//! including frame tree traversal, window size calculations, and tab page handling.
//! Phase 177 of Rust migration.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::similar_names)]

use std::ffi::{c_char, c_int};

use crate::{SessionFlags, FR_COL, FR_LEAF, FR_ROW};

// =============================================================================
// Handle Types
// =============================================================================

/// Opaque handle to a window (win_T pointer)
pub type WinHandle = *mut std::ffi::c_void;

/// Opaque handle to a frame (frame_T pointer)
pub type FrameHandle = *mut std::ffi::c_void;

/// Opaque handle to a tab page (tabpage_T pointer)
pub type TabpageHandle = *mut std::ffi::c_void;

// =============================================================================
// Frame Layout Serialization
// =============================================================================

/// Check if a frame is a leaf (contains a single window).
#[no_mangle]
pub extern "C" fn rs_session_is_frame_leaf(layout: c_int) -> c_int {
    c_int::from(layout == FR_LEAF)
}

/// Check if a frame is a column (vertically split).
#[no_mangle]
pub extern "C" fn rs_session_is_frame_col(layout: c_int) -> c_int {
    c_int::from(layout == FR_COL)
}

/// Check if a frame is a row (horizontally split).
#[no_mangle]
pub extern "C" fn rs_session_is_frame_row(layout: c_int) -> c_int {
    c_int::from(layout == FR_ROW)
}

/// Get the split command for a frame layout.
///
/// Returns "split\n" for column, "vsplit\n" for row.
#[no_mangle]
pub extern "C" fn rs_session_get_split_cmd(layout: c_int) -> *const c_char {
    static SPLIT: &[u8] = b"split\n\0";
    static VSPLIT: &[u8] = b"vsplit\n\0";
    static EMPTY: &[u8] = b"\0";

    let cmd = match layout {
        FR_COL => SPLIT,
        FR_ROW => VSPLIT,
        _ => EMPTY,
    };
    cmd.as_ptr().cast::<c_char>()
}

/// Get the wincmd to move back to first window in layout.
///
/// Returns "k" (up) for column, "h" (left) for row.
#[no_mangle]
#[allow(clippy::cast_possible_wrap)]
pub extern "C" fn rs_session_get_wincmd_back(layout: c_int) -> c_char {
    match layout {
        FR_COL => b'k' as c_char,
        FR_ROW => b'h' as c_char,
        _ => 0,
    }
}

/// Get the format string for wincmd to go back N windows.
#[no_mangle]
pub extern "C" fn rs_session_wincmd_back_fmt(layout: c_int) -> *const c_char {
    static WINCMD_K: &[u8] = b"%dwincmd k\n\0";
    static WINCMD_H: &[u8] = b"%dwincmd h\n\0";
    static EMPTY: &[u8] = b"\0";

    let fmt = match layout {
        FR_COL => WINCMD_K,
        FR_ROW => WINCMD_H,
        _ => EMPTY,
    };
    fmt.as_ptr().cast::<c_char>()
}

// =============================================================================
// Window Session Predicates
// =============================================================================

/// Check if a window should be included in the session.
///
/// A window is included if:
/// - It's not a floating window
/// - Either: has a file OR 'blank' flag is set
/// - Either: not help OR 'help' flag is set
/// - Either: not terminal OR 'terminal' flag is set
#[no_mangle]
pub extern "C" fn rs_session_should_save_window(
    is_floating: c_int,
    has_fname: c_int,
    is_nofile: c_int,
    is_terminal: c_int,
    is_help: c_int,
    flags: u32,
) -> c_int {
    // Skip floating windows
    if is_floating != 0 {
        return 0;
    }

    let f = SessionFlags::from_bits_truncate(flags);

    // Check blank/nofile windows
    if has_fname == 0 && is_nofile != 0 && !is_terminal_flag_ok(is_terminal, f) {
        // No file name and nofile type (and not terminal with flag)
        if !f.contains(SessionFlags::BLANK) {
            return 0;
        }
    }

    // Check help windows
    if is_help != 0 && !f.contains(SessionFlags::HELP) {
        return 0;
    }

    // Check terminal windows
    if is_terminal != 0 && !f.contains(SessionFlags::TERMINAL) {
        return 0;
    }

    1
}

/// Helper to check if terminal flag is OK
fn is_terminal_flag_ok(is_terminal: c_int, flags: SessionFlags) -> bool {
    is_terminal != 0 && flags.contains(SessionFlags::TERMINAL)
}

/// Check if a frame contains any saveable windows.
///
/// For leaf frames, checks the window directly.
/// For container frames, returns true if any child has saveable windows.
#[no_mangle]
pub extern "C" fn rs_session_frame_has_saveable(
    is_leaf: c_int,
    win_is_saveable: c_int,
    children_have_saveable: c_int,
) -> c_int {
    if is_leaf != 0 {
        win_is_saveable
    } else {
        children_have_saveable
    }
}

// =============================================================================
// Window Size Serialization
// =============================================================================

/// Check if window sizes should be restored.
///
/// Returns true if 'winsize' is in sessionoptions.
#[no_mangle]
pub extern "C" fn rs_session_should_restore_winsize(flags: u32) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(f.contains(SessionFlags::WINSIZE))
}

/// Calculate proportional window height for resize command.
///
/// Formula: (height * rows + rows/2) / rows (for rounding)
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn rs_session_calc_resize_height(height: c_int, rows: c_int) -> c_int {
    if rows <= 0 {
        return height;
    }
    ((i64::from(height) * i64::from(rows) + i64::from(rows) / 2) / i64::from(rows)) as c_int
}

/// Calculate proportional window width for resize command.
///
/// Formula: (width * cols + cols/2) / cols (for rounding)
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub extern "C" fn rs_session_calc_resize_width(width: c_int, cols: c_int) -> c_int {
    if cols <= 0 {
        return width;
    }
    ((i64::from(width) * i64::from(cols) + i64::from(cols) / 2) / i64::from(cols)) as c_int
}

/// Check if a window needs height restoration.
///
/// Returns true if window height + separators < topframe height.
#[no_mangle]
pub extern "C" fn rs_session_win_needs_height_restore(
    win_height: c_int,
    hsep_height: c_int,
    status_height: c_int,
    topframe_height: c_int,
) -> c_int {
    c_int::from(win_height + hsep_height + status_height < topframe_height)
}

/// Check if a window needs width restoration.
///
/// Returns true if window width < total columns.
#[no_mangle]
pub extern "C" fn rs_session_win_needs_width_restore(win_width: c_int, columns: c_int) -> c_int {
    c_int::from(win_width < columns)
}

/// Get the resize command format string.
#[no_mangle]
pub extern "C" fn rs_session_resize_fmt() -> *const c_char {
    static FMT: &[u8] = b"exe '%dresize ' . ((&lines * %ld + %ld) / %ld)\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the vertical resize command format string.
#[no_mangle]
pub extern "C" fn rs_session_vert_resize_fmt() -> *const c_char {
    static FMT: &[u8] = b"exe 'vert %dresize ' . ((&columns * %ld + %ld) / %ld)\n\0";
    FMT.as_ptr().cast::<c_char>()
}

// =============================================================================
// Tab Page Serialization
// =============================================================================

/// Check if tab pages should be saved.
#[no_mangle]
pub extern "C" fn rs_session_should_save_tabpages(flags: u32) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(f.contains(SessionFlags::TABPAGES))
}

/// Get the tabnew command string.
#[no_mangle]
pub extern "C" fn rs_session_tabnew_cmd() -> *const c_char {
    static CMD: &[u8] = b"tabnew +setlocal\\ bufhidden=wipe\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the tabrewind command string.
#[no_mangle]
pub extern "C" fn rs_session_tabrewind_cmd() -> *const c_char {
    static CMD: &[u8] = b"tabrewind\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the tabnext command string.
#[no_mangle]
pub extern "C" fn rs_session_tabnext_cmd() -> *const c_char {
    static CMD: &[u8] = b"tabnext\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the tabnext N command format.
#[no_mangle]
pub extern "C" fn rs_session_tabnext_n_fmt() -> *const c_char {
    static FMT: &[u8] = b"tabnext %d\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Check if this is the first tab page.
#[no_mangle]
pub extern "C" fn rs_session_is_first_tab(tab_index: c_int) -> c_int {
    c_int::from(tab_index == 0)
}

/// Check if tabnext is needed before processing this tab.
#[no_mangle]
pub extern "C" fn rs_session_need_tabnext(tab_index: c_int, is_tabpages_mode: c_int) -> c_int {
    c_int::from(is_tabpages_mode != 0 && tab_index > 0)
}

// =============================================================================
// Window Navigation Commands
// =============================================================================

/// Get the "wincmd w" command for moving to next window.
#[no_mangle]
pub extern "C" fn rs_session_wincmd_w_cmd() -> *const c_char {
    static CMD: &[u8] = b"wincmd w\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the "wincmd t" command for moving to top-left window.
#[no_mangle]
pub extern "C" fn rs_session_wincmd_t_cmd() -> *const c_char {
    static CMD: &[u8] = b"wincmd t\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the wincmd format for going to window N.
#[no_mangle]
pub extern "C" fn rs_session_wincmd_n_fmt() -> *const c_char {
    static FMT: &[u8] = b"%dwincmd w\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the "wincmd =" command for equalizing windows.
#[no_mangle]
pub extern "C" fn rs_session_wincmd_equal_cmd() -> *const c_char {
    static CMD: &[u8] = b"wincmd =\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the maximize window command.
#[no_mangle]
pub extern "C" fn rs_session_wincmd_maximize_cmd() -> *const c_char {
    static CMD: &[u8] = b"wincmd _ | wincmd |\n\0";
    CMD.as_ptr().cast::<c_char>()
}

// =============================================================================
// Split State Preservation
// =============================================================================

/// Get the save splitbelow command.
#[no_mangle]
pub extern "C" fn rs_session_save_splitbelow_cmd() -> *const c_char {
    static CMD: &[u8] = b"let s:save_splitbelow = &splitbelow\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the save splitright command.
#[no_mangle]
pub extern "C" fn rs_session_save_splitright_cmd() -> *const c_char {
    static CMD: &[u8] = b"let s:save_splitright = &splitright\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the set split command.
#[no_mangle]
pub extern "C" fn rs_session_set_split_cmd() -> *const c_char {
    static CMD: &[u8] = b"set splitbelow splitright\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the restore splitbelow command.
#[no_mangle]
pub extern "C" fn rs_session_restore_splitbelow_cmd() -> *const c_char {
    static CMD: &[u8] = b"let &splitbelow = s:save_splitbelow\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the restore splitright command.
#[no_mangle]
pub extern "C" fn rs_session_restore_splitright_cmd() -> *const c_char {
    static CMD: &[u8] = b"let &splitright = s:save_splitright\n\0";
    CMD.as_ptr().cast::<c_char>()
}

// =============================================================================
// Window Height/Width Preservation
// =============================================================================

/// Get the save winminheight command.
#[no_mangle]
pub extern "C" fn rs_session_save_winminheight_cmd() -> *const c_char {
    static CMD: &[u8] = b"let s:save_winminheight = &winminheight\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the save winminwidth command.
#[no_mangle]
pub extern "C" fn rs_session_save_winminwidth_cmd() -> *const c_char {
    static CMD: &[u8] = b"let s:save_winminwidth = &winminwidth\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the set winheight/winwidth to 1 commands.
#[no_mangle]
pub extern "C" fn rs_session_set_winheight_1_cmd() -> *const c_char {
    static CMD: &[u8] =
        b"set winminheight=0\nset winheight=1\nset winminwidth=0\nset winwidth=1\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the restore winminheight command.
#[no_mangle]
pub extern "C" fn rs_session_restore_winminheight_cmd() -> *const c_char {
    static CMD: &[u8] = b"let &winminheight = s:save_winminheight\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the restore winminwidth command.
#[no_mangle]
pub extern "C" fn rs_session_restore_winminwidth_cmd() -> *const c_char {
    static CMD: &[u8] = b"let &winminwidth = s:save_winminwidth\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the set winheight/winwidth format.
#[no_mangle]
pub extern "C" fn rs_session_set_winheight_fmt() -> *const c_char {
    static FMT: &[u8] = b"set winheight=%ld winwidth=%ld\n\0";
    FMT.as_ptr().cast::<c_char>()
}

// =============================================================================
// Tab-local Directory
// =============================================================================

/// Get the tcd command format.
#[no_mangle]
pub extern "C" fn rs_session_tcd_fmt() -> *const c_char {
    static FMT: &[u8] = b"tcd %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Check if tab-local directory should be saved.
#[no_mangle]
pub extern "C" fn rs_session_should_save_tcd(flags: u32, has_localdir: c_int) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(f.contains(SessionFlags::CURDIR) && has_localdir != 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_type_checks() {
        assert_eq!(rs_session_is_frame_leaf(FR_LEAF), 1);
        assert_eq!(rs_session_is_frame_leaf(FR_COL), 0);
        assert_eq!(rs_session_is_frame_col(FR_COL), 1);
        assert_eq!(rs_session_is_frame_col(FR_ROW), 0);
        assert_eq!(rs_session_is_frame_row(FR_ROW), 1);
        assert_eq!(rs_session_is_frame_row(FR_LEAF), 0);
    }

    #[test]
    #[allow(clippy::cast_possible_wrap)]
    fn test_get_wincmd_back() {
        assert_eq!(rs_session_get_wincmd_back(FR_COL), b'k' as c_char);
        assert_eq!(rs_session_get_wincmd_back(FR_ROW), b'h' as c_char);
        assert_eq!(rs_session_get_wincmd_back(FR_LEAF), 0);
    }

    #[test]
    fn test_should_save_window() {
        // Normal window
        assert_eq!(rs_session_should_save_window(0, 1, 0, 0, 0, 0), 1);

        // Floating window
        assert_eq!(rs_session_should_save_window(1, 1, 0, 0, 0, 0), 0);

        // Help window without HELP flag
        assert_eq!(rs_session_should_save_window(0, 1, 0, 0, 1, 0), 0);

        // Help window with HELP flag
        assert_eq!(
            rs_session_should_save_window(0, 1, 0, 0, 1, SessionFlags::HELP.bits()),
            1
        );

        // Terminal without TERMINAL flag
        assert_eq!(rs_session_should_save_window(0, 1, 0, 1, 0, 0), 0);

        // Terminal with TERMINAL flag
        assert_eq!(
            rs_session_should_save_window(0, 1, 0, 1, 0, SessionFlags::TERMINAL.bits()),
            1
        );
    }

    #[test]
    fn test_should_restore_winsize() {
        assert_eq!(rs_session_should_restore_winsize(0), 0);
        assert_eq!(
            rs_session_should_restore_winsize(SessionFlags::WINSIZE.bits()),
            1
        );
    }

    #[test]
    fn test_calc_resize() {
        // 50 height at 100 rows
        assert_eq!(rs_session_calc_resize_height(50, 100), 50);

        // Zero rows
        assert_eq!(rs_session_calc_resize_height(50, 0), 50);

        // 80 width at 160 cols
        assert_eq!(rs_session_calc_resize_width(80, 160), 80);
    }

    #[test]
    fn test_win_needs_restore() {
        // Full height window
        assert_eq!(rs_session_win_needs_height_restore(100, 0, 0, 100), 0);

        // Partial height window
        assert_eq!(rs_session_win_needs_height_restore(50, 0, 1, 100), 1);

        // Full width window
        assert_eq!(rs_session_win_needs_width_restore(80, 80), 0);

        // Partial width window
        assert_eq!(rs_session_win_needs_width_restore(40, 80), 1);
    }

    #[test]
    fn test_should_save_tabpages() {
        assert_eq!(rs_session_should_save_tabpages(0), 0);
        assert_eq!(
            rs_session_should_save_tabpages(SessionFlags::TABPAGES.bits()),
            1
        );
    }

    #[test]
    fn test_tab_navigation() {
        assert_eq!(rs_session_is_first_tab(0), 1);
        assert_eq!(rs_session_is_first_tab(1), 0);
        assert_eq!(rs_session_need_tabnext(0, 1), 0);
        assert_eq!(rs_session_need_tabnext(1, 1), 1);
        assert_eq!(rs_session_need_tabnext(1, 0), 0);
    }

    #[test]
    fn test_should_save_tcd() {
        assert_eq!(rs_session_should_save_tcd(0, 0), 0);
        assert_eq!(rs_session_should_save_tcd(0, 1), 0);
        assert_eq!(
            rs_session_should_save_tcd(SessionFlags::CURDIR.bits(), 0),
            0
        );
        assert_eq!(
            rs_session_should_save_tcd(SessionFlags::CURDIR.bits(), 1),
            1
        );
    }

    #[test]
    fn test_command_strings() {
        unsafe {
            let s = std::ffi::CStr::from_ptr(rs_session_tabnew_cmd());
            assert!(s.to_str().unwrap().contains("tabnew"));

            let s = std::ffi::CStr::from_ptr(rs_session_wincmd_t_cmd());
            assert!(s.to_str().unwrap().contains("wincmd t"));

            let s = std::ffi::CStr::from_ptr(rs_session_wincmd_equal_cmd());
            assert!(s.to_str().unwrap().contains("wincmd ="));

            let s = std::ffi::CStr::from_ptr(rs_session_set_split_cmd());
            assert!(s.to_str().unwrap().contains("splitbelow"));
        }
    }

    #[test]
    fn test_split_commands() {
        unsafe {
            let col_split = std::ffi::CStr::from_ptr(rs_session_get_split_cmd(FR_COL));
            assert!(col_split.to_str().unwrap().contains("split"));
            assert!(!col_split.to_str().unwrap().contains("vsplit"));

            let row_split = std::ffi::CStr::from_ptr(rs_session_get_split_cmd(FR_ROW));
            assert!(row_split.to_str().unwrap().contains("vsplit"));
        }
    }
}
