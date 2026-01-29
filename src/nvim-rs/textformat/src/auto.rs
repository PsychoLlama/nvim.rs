//! Auto-formatting for text formatting.
//!
//! This module provides functions for automatic text formatting
//! triggered by insert mode text changes.

#![allow(unsafe_code)] // FFI requires unsafe

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to a window (win_T*).
pub type WinHandle = *mut c_void;

// =============================================================================
// Constants
// =============================================================================

/// NUL character
const NUL: c_int = 0;

/// Format option: auto-format ('a')
const FO_AUTO: c_int = b'a' as c_int;

/// Format option: wrap comments ('c')
const FO_WRAP_COMS: c_int = b'c' as c_int;

/// Format option: wrap text ('t')
const FO_WRAP: c_int = b't' as c_int;

/// Format option: one letter word ('1')
const FO_ONE_LETTER: c_int = b'1' as c_int;

/// Format option: trailing whitespace ('w')
const FO_WHITE_PAR: c_int = b'w' as c_int;

/// MAXCOL value for coladvance
const MAXCOL: c_int = 0x7FFFFFFF;

/// FAIL return value
const FAIL: c_int = 0;

// =============================================================================
// C Accessor Functions
// =============================================================================

extern "C" {
    // Format option check
    fn rs_has_format_option(x: c_int) -> c_int;

    // Cursor operations
    fn nvim_textfmt_get_curwin_cursor_lnum() -> c_int;
    fn nvim_textfmt_get_curwin_cursor_col() -> c_int;
    fn nvim_textfmt_set_curwin_cursor(lnum: c_int, col: c_int);
    fn nvim_textfmt_dec_cursor();
    fn nvim_textfmt_inc_cursor();
    fn nvim_textfmt_gchar_cursor() -> c_int;
    fn nvim_textfmt_get_cursor_line_ptr() -> *const c_char;
    fn nvim_textfmt_get_cursor_line_len() -> c_int;

    // WHITECHAR check (handles UTF-8 composing)
    fn nvim_textfmt_whitechar(cc: c_int) -> bool;

    // Leader detection
    fn nvim_textfmt_get_leader_len_simple(line: *const c_char) -> c_int;

    // Paragraph detection (already migrated)
    fn rs_paragraph_start(lnum: c_int) -> c_int;

    // Undo
    fn nvim_textfmt_u_save_cursor() -> c_int;

    // Formatting
    fn nvim_textfmt_format_lines(line_count: c_int, avoid_fex: bool);

    // Saved cursor management
    fn nvim_textfmt_set_saved_cursor(lnum: c_int, col: c_int);
    fn nvim_textfmt_get_saved_cursor_lnum() -> c_int;
    fn nvim_textfmt_get_saved_cursor_col() -> c_int;
    fn nvim_textfmt_clear_saved_cursor();

    // Buffer state
    fn nvim_textfmt_get_ml_line_count() -> c_int;

    // Cursor positioning
    fn nvim_textfmt_coladvance(win: WinHandle, col: c_int);
    fn nvim_textfmt_check_cursor_col(win: WinHandle);
    fn nvim_textfmt_check_cursor(win: WinHandle);
    fn nvim_textfmt_get_curwin() -> WinHandle;

    // Line replacement
    fn nvim_textfmt_ml_replace_with_space(lnum: c_int);

    // did_add_space state
    fn nvim_textfmt_get_did_add_space() -> bool;
    fn nvim_textfmt_set_did_add_space(val: bool);

    // Delete character
    fn nvim_textfmt_del_char(fixpos: bool) -> c_int;
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if format option 'x' is set.
#[inline]
fn has_format_option(x: c_int) -> bool {
    unsafe { rs_has_format_option(x) != 0 }
}

/// Check if paragraph starts at line.
#[inline]
fn paragraph_start(lnum: c_int) -> bool {
    unsafe { rs_paragraph_start(lnum) != 0 }
}

// =============================================================================
// Auto-format Functions
// =============================================================================

/// Called after inserting or deleting text: When 'formatoptions' includes the
/// 'a' flag format from the current line until the end of the paragraph.
/// Keep the cursor at the same position relative to the text.
///
/// # Arguments
/// * `trailblank` - When true also format with trailing blank
/// * `prev_line` - May start in previous line
///
/// # Safety
/// Accesses global state via C functions.
unsafe fn auto_format_impl(trailblank: bool, prev_line: bool) {
    if !has_format_option(FO_AUTO) {
        return;
    }

    let pos_lnum = nvim_textfmt_get_curwin_cursor_lnum();
    let pos_col = nvim_textfmt_get_curwin_cursor_col();
    let old = nvim_textfmt_get_cursor_line_ptr();

    // May remove added space
    check_auto_format_impl(false);

    // Don't format in Insert mode when the cursor is on a trailing blank, the
    // user might insert normal text next. Also skip formatting when "1" is
    // in 'formatoptions' and there is a single character before the cursor.
    let wasatend = pos_col == nvim_textfmt_get_cursor_line_len();
    if !old.is_null() && *old != 0 && !trailblank && wasatend {
        nvim_textfmt_dec_cursor();
        let mut cc = nvim_textfmt_gchar_cursor();
        if !nvim_textfmt_whitechar(cc)
            && nvim_textfmt_get_curwin_cursor_col() > 0
            && has_format_option(FO_ONE_LETTER)
        {
            nvim_textfmt_dec_cursor();
        }
        cc = nvim_textfmt_gchar_cursor();
        if nvim_textfmt_whitechar(cc) {
            nvim_textfmt_set_curwin_cursor(pos_lnum, pos_col);
            return;
        }
        nvim_textfmt_set_curwin_cursor(pos_lnum, pos_col);
    }

    // With the 'c' flag in 'formatoptions' and 't' missing: only format comments.
    if has_format_option(FO_WRAP_COMS)
        && !has_format_option(FO_WRAP)
        && nvim_textfmt_get_leader_len_simple(old) == 0
    {
        return;
    }

    // May start formatting in a previous line, so that after "x" a word is
    // moved to the previous line if it fits there now. Only when this is not
    // the start of a paragraph.
    let cursor_lnum = nvim_textfmt_get_curwin_cursor_lnum();
    if prev_line && !paragraph_start(cursor_lnum) {
        nvim_textfmt_set_curwin_cursor(cursor_lnum - 1, 0);
        if nvim_textfmt_u_save_cursor() == FAIL {
            return;
        }
    }

    // Do the formatting and restore the cursor position. "saved_cursor" will
    // be adjusted for the text formatting.
    nvim_textfmt_set_saved_cursor(pos_lnum, pos_col);
    nvim_textfmt_format_lines(-1, false);
    let saved_lnum = nvim_textfmt_get_saved_cursor_lnum();
    let saved_col = nvim_textfmt_get_saved_cursor_col();
    nvim_textfmt_set_curwin_cursor(saved_lnum, saved_col);
    nvim_textfmt_clear_saved_cursor();

    let ml_line_count = nvim_textfmt_get_ml_line_count();
    let new_cursor_lnum = nvim_textfmt_get_curwin_cursor_lnum();
    if new_cursor_lnum > ml_line_count {
        // "cannot happen"
        nvim_textfmt_set_curwin_cursor(ml_line_count, 0);
        nvim_textfmt_coladvance(nvim_textfmt_get_curwin(), MAXCOL);
    } else {
        nvim_textfmt_check_cursor_col(nvim_textfmt_get_curwin());
    }

    // Insert mode: If the cursor is now after the end of the line while it
    // previously wasn't, the line was broken. Because of the rule above we
    // need to add a space when 'w' is in 'formatoptions' to keep a paragraph
    // formatted.
    if !wasatend && has_format_option(FO_WHITE_PAR) {
        let len = nvim_textfmt_get_cursor_line_len();
        let cursor_col = nvim_textfmt_get_curwin_cursor_col();
        if cursor_col == len {
            let cursor_lnum = nvim_textfmt_get_curwin_cursor_lnum();
            nvim_textfmt_ml_replace_with_space(cursor_lnum);
            // Remove the space later
            nvim_textfmt_set_did_add_space(true);
        } else {
            // May remove added space
            check_auto_format_impl(false);
        }
    }

    nvim_textfmt_check_cursor(nvim_textfmt_get_curwin());
}

/// When an extra space was added to continue a paragraph for auto-formatting,
/// delete it now. The space must be under the cursor, just after the insert
/// position.
///
/// # Arguments
/// * `end_insert` - True when ending Insert mode
///
/// # Safety
/// Accesses global state via C functions.
unsafe fn check_auto_format_impl(end_insert: bool) {
    if !nvim_textfmt_get_did_add_space() {
        return;
    }

    let cc = nvim_textfmt_gchar_cursor();
    if !nvim_textfmt_whitechar(cc) {
        // Somehow the space was removed already.
        nvim_textfmt_set_did_add_space(false);
    } else {
        let mut c: c_int = b' ' as c_int;
        if !end_insert {
            nvim_textfmt_inc_cursor();
            c = nvim_textfmt_gchar_cursor();
            nvim_textfmt_dec_cursor();
        }
        if c != NUL {
            // The space is no longer at the end of the line, delete it.
            nvim_textfmt_del_char(false);
            nvim_textfmt_set_did_add_space(false);
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Auto-format after text insertion/deletion.
///
/// # Safety
/// Accesses global state via C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_auto_format(trailblank: c_int, prev_line: c_int) {
    auto_format_impl(trailblank != 0, prev_line != 0);
}

/// Check and remove auto-added space.
///
/// # Safety
/// Accesses global state via C functions.
#[no_mangle]
pub unsafe extern "C" fn rs_check_auto_format(end_insert: c_int) {
    check_auto_format_impl(end_insert != 0);
}

#[cfg(test)]
mod tests {
    // Integration testing is done via the full Neovim build.
}
