//! View state serialization for sessions
//!
//! This module provides helpers for serializing view state to session files,
//! including cursor position, fold state, and local options.
//! Phase 178 of Rust migration.

#![allow(clippy::missing_safety_doc)]
#![allow(dead_code)]

use std::ffi::{c_char, c_int};

use crate::SessionFlags;

// =============================================================================
// Type Definitions
// =============================================================================

/// Column type (matches colnr_T)
pub type ColNr = i32;

/// MAXCOL constant for end-of-line cursor
pub const MAXCOL: ColNr = 0x7FFF_FFFF;

// =============================================================================
// Cursor Position Serialization
// =============================================================================

/// Check if cursor position should be saved.
///
/// For :mksession, cursor is always saved.
/// For :mkview, only when 'cursor' is in viewoptions.
#[no_mangle]
pub extern "C" fn rs_session_should_save_cursor(is_session: c_int, flags: u32) -> c_int {
    if is_session != 0 {
        return 1;
    }
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(f.contains(SessionFlags::CURSOR))
}

/// Check if curswant indicates end of line (curswant == MAXCOL).
#[no_mangle]
pub extern "C" fn rs_session_curswant_is_eol(curswant: ColNr) -> c_int {
    c_int::from(curswant == MAXCOL)
}

/// Get the normal command for cursor at end of line.
#[no_mangle]
pub extern "C" fn rs_session_cursor_eol_cmd() -> *const c_char {
    static CMD: &[u8] = b"normal! $\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the format for cursor column command.
/// Format: "normal! 0%d|\n" where %d is column + 1
#[no_mangle]
pub extern "C" fn rs_session_cursor_col_fmt() -> *const c_char {
    static FMT: &[u8] = b"normal! 0%d|\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the format for setting cursor line variable.
#[no_mangle]
pub extern "C" fn rs_session_cursor_line_let_fmt() -> *const c_char {
    static FMT: &[u8] = b"let s:l = %ld\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the format for cursor line with view height calculation.
#[no_mangle]
pub extern "C" fn rs_session_cursor_line_view_fmt() -> *const c_char {
    static FMT: &[u8] = b"let s:l = %ld - ((%ld * winheight(0) + %d) / %d)\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the cursor positioning commands.
#[no_mangle]
pub extern "C" fn rs_session_cursor_pos_cmd() -> *const c_char {
    static CMD: &[u8] = b"if s:l < 1 | let s:l = 1 | endif\nkeepjumps exe s:l\nnormal! zt\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the keepjumps line command format.
#[no_mangle]
pub extern "C" fn rs_session_keepjumps_line_fmt() -> *const c_char {
    static FMT: &[u8] = b"keepjumps %ld\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the format for cursor with horizontal scroll.
#[no_mangle]
pub extern "C" fn rs_session_cursor_hscroll_fmt() -> *const c_char {
    static FMT: &[u8] = b"let s:c = %ld - ((%ld * winwidth(0) + %ld) / %ld)\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the horizontal scroll conditional.
#[no_mangle]
pub extern "C" fn rs_session_hscroll_cond_start() -> *const c_char {
    static CMD: &[u8] = b"if s:c > 0\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the horizontal scroll positioning command format.
#[no_mangle]
pub extern "C" fn rs_session_hscroll_pos_fmt() -> *const c_char {
    static FMT: &[u8] = b"  exe 'normal! ' . s:c . '|zs' . %ld . '|'\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the else branch for hscroll.
#[no_mangle]
pub extern "C" fn rs_session_hscroll_else() -> *const c_char {
    static CMD: &[u8] = b"else\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the endif for hscroll.
#[no_mangle]
pub extern "C" fn rs_session_hscroll_endif() -> *const c_char {
    static CMD: &[u8] = b"endif\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Check if horizontal scroll needs to be saved.
#[no_mangle]
pub extern "C" fn rs_session_need_hscroll(
    wrap: c_int,
    leftcol: c_int,
    width: c_int,
) -> c_int {
    c_int::from(wrap == 0 && leftcol > 0 && width > 0)
}

/// Check if cursor column needs special handling.
#[no_mangle]
pub extern "C" fn rs_session_cursor_col_zero(col: ColNr) -> c_int {
    c_int::from(col == 0)
}

/// Get the normal! 0 command.
#[no_mangle]
pub extern "C" fn rs_session_normal_0_cmd() -> *const c_char {
    static CMD: &[u8] = b"normal! 0\n\0";
    CMD.as_ptr().cast::<c_char>()
}

// =============================================================================
// Fold State Serialization
// =============================================================================

/// Check if folds flag is set in options.
#[no_mangle]
pub extern "C" fn rs_session_has_folds_flag(flags: u32) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(f.contains(SessionFlags::FOLDS))
}

/// Check if folds can be saved for a buffer.
///
/// Folds can be saved if buffer has a file name and is either normal or help.
#[no_mangle]
pub extern "C" fn rs_session_can_save_folds(
    has_ffname: c_int,
    is_normal: c_int,
    is_help: c_int,
) -> c_int {
    c_int::from(has_ffname != 0 && (is_normal != 0 || is_help != 0))
}

/// Fold method constants
pub const FOLD_MANUAL: c_int = 0;
pub const FOLD_INDENT: c_int = 1;
pub const FOLD_EXPR: c_int = 2;
pub const FOLD_MARKER: c_int = 3;
pub const FOLD_SYNTAX: c_int = 4;
pub const FOLD_DIFF: c_int = 5;

/// Check if fold method saves manual folds.
#[no_mangle]
pub extern "C" fn rs_session_foldmethod_saves_manual(method: c_int) -> c_int {
    c_int::from(method == FOLD_MANUAL || method == FOLD_MARKER)
}

/// Get the setlocal foldmethod= format.
#[no_mangle]
pub extern "C" fn rs_session_setl_foldmethod_fmt() -> *const c_char {
    static FMT: &[u8] = b"setlocal foldmethod=%s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the fold open command format.
#[no_mangle]
pub extern "C" fn rs_session_fold_open_fmt() -> *const c_char {
    static FMT: &[u8] = b"%ld,%ldfoldopen!\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the fold close command format.
#[no_mangle]
pub extern "C" fn rs_session_fold_close_fmt() -> *const c_char {
    static FMT: &[u8] = b"%ld,%ldfold\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the silent! command wrapper.
#[no_mangle]
pub extern "C" fn rs_session_silent_fmt() -> *const c_char {
    static FMT: &[u8] = b"silent! %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the setlocal foldlevel= format.
#[no_mangle]
pub extern "C" fn rs_session_setl_foldlevel_fmt() -> *const c_char {
    static FMT: &[u8] = b"setlocal foldlevel=%ld\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the setlocal foldcolumn= format.
#[no_mangle]
pub extern "C" fn rs_session_setl_foldcolumn_fmt() -> *const c_char {
    static FMT: &[u8] = b"setlocal foldcolumn=%ld\n\0";
    FMT.as_ptr().cast::<c_char>()
}

// =============================================================================
// Local Options Serialization
// =============================================================================

/// Check if local options should be saved.
#[no_mangle]
pub extern "C" fn rs_session_should_save_local_options(
    flags: u32,
    is_view: c_int,
) -> c_int {
    let f = SessionFlags::from_bits_truncate(flags);

    if f.contains(SessionFlags::OPTIONS) || f.contains(SessionFlags::LOCALOPTIONS) {
        return 1;
    }

    // For views, save fold options even without options flag
    if is_view != 0 && f.contains(SessionFlags::FOLDS) {
        return 1;
    }

    0
}

/// Check if only local values should be saved.
///
/// Returns true for :mkview or when 'options' is not in sessionoptions.
#[no_mangle]
pub extern "C" fn rs_session_only_local_values(is_view: c_int, flags: u32) -> c_int {
    if is_view != 0 {
        return 1;
    }
    let f = SessionFlags::from_bits_truncate(flags);
    c_int::from(!f.contains(SessionFlags::OPTIONS))
}

/// Get the setlocal command for string options.
#[no_mangle]
pub extern "C" fn rs_session_setl_string_fmt() -> *const c_char {
    static FMT: &[u8] = b"setlocal %s=%s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the setlocal command for number options.
#[no_mangle]
pub extern "C" fn rs_session_setl_number_fmt() -> *const c_char {
    static FMT: &[u8] = b"setlocal %s=%ld\n\0";
    FMT.as_ptr().cast::<c_char>()
}

/// Get the setlocal command for boolean options.
#[no_mangle]
pub extern "C" fn rs_session_setl_bool_fmt() -> *const c_char {
    static FMT: &[u8] = b"setlocal %s%s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

// =============================================================================
// Window-local Directory
// =============================================================================

/// Check if window-local directory should be saved.
#[no_mangle]
pub extern "C" fn rs_session_should_save_lcd(
    is_view: c_int,
    flags: u32,
    has_localdir: c_int,
) -> c_int {
    if has_localdir == 0 {
        return 0;
    }

    // For views, only save if curdir is in viewoptions
    if is_view != 0 {
        let f = SessionFlags::from_bits_truncate(flags);
        return c_int::from(f.contains(SessionFlags::CURDIR));
    }

    // For sessions, always save window-local directory
    1
}

/// Get the lcd command format.
#[no_mangle]
pub extern "C" fn rs_session_lcd_fmt() -> *const c_char {
    static FMT: &[u8] = b"lcd %s\n\0";
    FMT.as_ptr().cast::<c_char>()
}

// =============================================================================
// Scrolloff/Sidescrolloff Preservation
// =============================================================================

/// Get the save scrolloff command.
#[no_mangle]
pub extern "C" fn rs_session_save_scrolloff_cmd() -> *const c_char {
    static CMD: &[u8] = b"let s:so_save = &g:so | let s:siso_save = &g:siso | setg so=0 siso=0 | setl so=-1 siso=-1\n\0";
    CMD.as_ptr().cast::<c_char>()
}

/// Get the restore scrolloff command.
#[no_mangle]
pub extern "C" fn rs_session_restore_scrolloff_cmd() -> *const c_char {
    static CMD: &[u8] = b"let &g:so = s:so_save | let &g:siso = s:siso_save\n\0";
    CMD.as_ptr().cast::<c_char>()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_save_cursor() {
        // Session always saves cursor
        assert_eq!(rs_session_should_save_cursor(1, 0), 1);

        // View without CURSOR flag
        assert_eq!(rs_session_should_save_cursor(0, 0), 0);

        // View with CURSOR flag
        assert_eq!(
            rs_session_should_save_cursor(0, SessionFlags::CURSOR.bits()),
            1
        );
    }

    #[test]
    fn test_curswant_is_eol() {
        assert_eq!(rs_session_curswant_is_eol(MAXCOL), 1);
        assert_eq!(rs_session_curswant_is_eol(0), 0);
        assert_eq!(rs_session_curswant_is_eol(10), 0);
    }

    #[test]
    fn test_need_hscroll() {
        // No wrap, has leftcol, has width
        assert_eq!(rs_session_need_hscroll(0, 10, 80), 1);

        // With wrap
        assert_eq!(rs_session_need_hscroll(1, 10, 80), 0);

        // No leftcol
        assert_eq!(rs_session_need_hscroll(0, 0, 80), 0);

        // No width
        assert_eq!(rs_session_need_hscroll(0, 10, 0), 0);
    }

    #[test]
    fn test_has_folds_flag() {
        assert_eq!(rs_session_has_folds_flag(0), 0);
        assert_eq!(
            rs_session_has_folds_flag(SessionFlags::FOLDS.bits()),
            1
        );
    }

    #[test]
    fn test_can_save_folds() {
        // Has filename, is normal
        assert_eq!(rs_session_can_save_folds(1, 1, 0), 1);

        // Has filename, is help
        assert_eq!(rs_session_can_save_folds(1, 0, 1), 1);

        // No filename
        assert_eq!(rs_session_can_save_folds(0, 1, 0), 0);

        // Neither normal nor help
        assert_eq!(rs_session_can_save_folds(1, 0, 0), 0);
    }

    #[test]
    fn test_foldmethod_saves_manual() {
        assert_eq!(rs_session_foldmethod_saves_manual(FOLD_MANUAL), 1);
        assert_eq!(rs_session_foldmethod_saves_manual(FOLD_MARKER), 1);
        assert_eq!(rs_session_foldmethod_saves_manual(FOLD_INDENT), 0);
        assert_eq!(rs_session_foldmethod_saves_manual(FOLD_EXPR), 0);
        assert_eq!(rs_session_foldmethod_saves_manual(FOLD_SYNTAX), 0);
        assert_eq!(rs_session_foldmethod_saves_manual(FOLD_DIFF), 0);
    }

    #[test]
    fn test_should_save_local_options() {
        // No flags
        assert_eq!(rs_session_should_save_local_options(0, 0), 0);

        // OPTIONS flag
        assert_eq!(
            rs_session_should_save_local_options(SessionFlags::OPTIONS.bits(), 0),
            1
        );

        // LOCALOPTIONS flag
        assert_eq!(
            rs_session_should_save_local_options(SessionFlags::LOCALOPTIONS.bits(), 0),
            1
        );

        // View with FOLDS
        assert_eq!(
            rs_session_should_save_local_options(SessionFlags::FOLDS.bits(), 1),
            1
        );

        // Non-view with FOLDS only
        assert_eq!(
            rs_session_should_save_local_options(SessionFlags::FOLDS.bits(), 0),
            0
        );
    }

    #[test]
    fn test_only_local_values() {
        // View always returns true
        assert_eq!(rs_session_only_local_values(1, 0), 1);
        assert_eq!(
            rs_session_only_local_values(1, SessionFlags::OPTIONS.bits()),
            1
        );

        // Non-view without OPTIONS
        assert_eq!(rs_session_only_local_values(0, 0), 1);

        // Non-view with OPTIONS
        assert_eq!(
            rs_session_only_local_values(0, SessionFlags::OPTIONS.bits()),
            0
        );
    }

    #[test]
    fn test_should_save_lcd() {
        // No localdir
        assert_eq!(rs_session_should_save_lcd(0, 0, 0), 0);

        // Session with localdir
        assert_eq!(rs_session_should_save_lcd(0, 0, 1), 1);

        // View without CURDIR
        assert_eq!(rs_session_should_save_lcd(1, 0, 1), 0);

        // View with CURDIR
        assert_eq!(
            rs_session_should_save_lcd(1, SessionFlags::CURDIR.bits(), 1),
            1
        );
    }

    #[test]
    fn test_command_strings() {
        unsafe {
            let s = std::ffi::CStr::from_ptr(rs_session_cursor_eol_cmd());
            assert!(s.to_str().unwrap().contains("normal! $"));

            let s = std::ffi::CStr::from_ptr(rs_session_normal_0_cmd());
            assert!(s.to_str().unwrap().contains("normal! 0"));

            let s = std::ffi::CStr::from_ptr(rs_session_save_scrolloff_cmd());
            assert!(s.to_str().unwrap().contains("so_save"));
        }
    }
}
