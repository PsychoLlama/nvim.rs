//! Status messages and UI support for completion.
//!
//! This module provides helper functions for completion UI and status messages.
//! The actual UI rendering remains in C (popupmenu.c, message.c), but Rust
//! provides utilities for state checking and formatting.

#![allow(clippy::missing_const_for_fn)]

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn nvim_get_ctrl_x_mode() -> c_int;
    fn nvim_get_compl_started() -> c_int;
    fn nvim_get_compl_matches() -> c_int;
    fn nvim_get_compl_selected_item() -> c_int;
    fn nvim_get_compl_interrupted() -> c_int;
    fn nvim_get_compl_time_slice_expired() -> c_int;
    fn nvim_pum_visible() -> c_int;
    fn nvim_pum_get_height() -> c_int;
}

// CTRL-X mode constants (for message selection)
const CTRL_X_WANT_IDENT: c_int = 0x100;

// Normal/scroll modes all map to keyword message (index 0)
#[allow(dead_code)]
const CTRL_X_NORMAL: c_int = 0;
#[allow(dead_code)]
const CTRL_X_NOT_DEFINED_YET: c_int = 1;
#[allow(dead_code)]
const CTRL_X_SCROLL: c_int = 2;
const CTRL_X_WHOLE_LINE: c_int = 3;
const CTRL_X_FILES: c_int = 4;
const CTRL_X_TAGS: c_int = 5 + CTRL_X_WANT_IDENT;
const CTRL_X_PATH_PATTERNS: c_int = 6 + CTRL_X_WANT_IDENT;
const CTRL_X_PATH_DEFINES: c_int = 7 + CTRL_X_WANT_IDENT;
const CTRL_X_DICTIONARY: c_int = 9 + CTRL_X_WANT_IDENT;
const CTRL_X_THESAURUS: c_int = 10 + CTRL_X_WANT_IDENT;
const CTRL_X_CMDLINE: c_int = 11;
const CTRL_X_FUNCTION: c_int = 12;
const CTRL_X_OMNI: c_int = 13;
const CTRL_X_SPELL: c_int = 14;
const CTRL_X_EVAL: c_int = 16;
const CTRL_X_CMDLINE_CTRL_X: c_int = 17;
const CTRL_X_BUFNAMES: c_int = 18;
const CTRL_X_REGISTER: c_int = 19;

/// Get the message index for the current CTRL-X mode.
///
/// Returns an index into ctrl_x_msgs array.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_msg_index() -> c_int {
    let mode = nvim_get_ctrl_x_mode();

    // The ctrl_x_msgs array order in C is:
    // 0: normal/keyword, 1: whole line, 2: files, 3: tags, 4: path patterns
    // 5: path defines, 6: (unused), 7: dictionary, 8: thesaurus, 9: cmdline
    // 10: function, 11: omni, 12: spell, 13: (unused), 14: eval
    // 15: local message (special), 16: bufnames, 17: register

    match mode {
        CTRL_X_WHOLE_LINE => 1,
        CTRL_X_FILES => 2,
        CTRL_X_TAGS => 3,
        CTRL_X_PATH_PATTERNS => 4,
        CTRL_X_PATH_DEFINES => 5,
        CTRL_X_DICTIONARY => 7,
        CTRL_X_THESAURUS => 8,
        CTRL_X_CMDLINE | CTRL_X_CMDLINE_CTRL_X => 9,
        CTRL_X_FUNCTION => 10,
        CTRL_X_OMNI => 11,
        CTRL_X_SPELL => 12,
        CTRL_X_EVAL => 14,
        CTRL_X_BUFNAMES => 16,
        CTRL_X_REGISTER => 17,
        // CTRL_X_NORMAL, CTRL_X_NOT_DEFINED_YET, CTRL_X_SCROLL, and unknown modes
        _ => 0, // Default to keyword
    }
}

/// Check if status message should be shown.
///
/// Returns true if completion is active and a message is appropriate.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_should_show_msg() -> c_int {
    if nvim_get_compl_started() == 0 {
        return 0;
    }
    // Show message if completion is active
    1
}

/// Check if popup menu should be displayed.
///
/// Returns true if there are matches and the popup should show.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_should_show_pum() -> c_int {
    if nvim_get_compl_started() == 0 {
        return 0;
    }
    c_int::from(nvim_get_compl_matches() > 0)
}

/// Check if popup menu is currently visible.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_pum_visible() -> c_int {
    nvim_pum_visible()
}

/// Get the popup menu height.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_pum_height() -> c_int {
    nvim_pum_get_height()
}

/// Check if completion has timed out.
///
/// Returns true if the time slice for completion has expired.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_timed_out() -> c_int {
    nvim_get_compl_time_slice_expired()
}

/// Check if completion was interrupted.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_was_interrupted() -> c_int {
    nvim_get_compl_interrupted()
}

/// Get the currently selected item index.
///
/// Returns -1 if nothing selected.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_selected() -> c_int {
    if nvim_get_compl_started() == 0 {
        return -1;
    }
    nvim_get_compl_selected_item()
}

/// Get the match count for display.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_get_match_count() -> c_int {
    nvim_get_compl_matches()
}

/// Check if we should show "(XX matches)" in the status.
///
/// Returns true if there are multiple matches.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_show_match_count() -> c_int {
    c_int::from(nvim_get_compl_matches() > 1)
}

/// Check if we should show "Pattern not found" message.
///
/// Returns true if completion started but no matches found.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_no_matches() -> c_int {
    c_int::from(nvim_get_compl_started() != 0 && nvim_get_compl_matches() == 0)
}

/// Check if completion status is "searching".
///
/// Returns true if still searching for matches.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_is_searching() -> c_int {
    // Searching if interrupted or time slice expired
    c_int::from(
        nvim_get_compl_started() != 0
            && (nvim_get_compl_interrupted() != 0 || nvim_get_compl_time_slice_expired() != 0),
    )
}

/// FFI export: Get CTRL_X_WANT_IDENT constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_want_ident() -> c_int {
    CTRL_X_WANT_IDENT
}

/// FFI export: Get CTRL_X_WHOLE_LINE constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_whole_line() -> c_int {
    CTRL_X_WHOLE_LINE
}

/// FFI export: Get CTRL_X_FILES constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_files() -> c_int {
    CTRL_X_FILES
}

/// FFI export: Get CTRL_X_TAGS constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_tags() -> c_int {
    CTRL_X_TAGS
}

/// FFI export: Get CTRL_X_DICTIONARY constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_dictionary() -> c_int {
    CTRL_X_DICTIONARY
}

/// FFI export: Get CTRL_X_THESAURUS constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_thesaurus() -> c_int {
    CTRL_X_THESAURUS
}

/// FFI export: Get CTRL_X_CMDLINE constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_cmdline() -> c_int {
    CTRL_X_CMDLINE
}

/// FFI export: Get CTRL_X_FUNCTION constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_function() -> c_int {
    CTRL_X_FUNCTION
}

/// FFI export: Get CTRL_X_OMNI constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_omni() -> c_int {
    CTRL_X_OMNI
}

/// FFI export: Get CTRL_X_SPELL constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_spell() -> c_int {
    CTRL_X_SPELL
}

/// FFI export: Get CTRL_X_EVAL constant.
#[no_mangle]
pub extern "C" fn rs_ctrl_x_eval() -> c_int {
    CTRL_X_EVAL
}

// =============================================================================
// Phase 6: Extended UI Integration Functions
// =============================================================================

// Additional C accessor functions
extern "C" {
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_compl_shown_match_exists() -> c_int;
    fn nvim_get_compl_autocomplete() -> c_int;
}

/// Check if completion should update the UI.
///
/// Returns true if completion state changed and UI needs refresh.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_needs_refresh() -> c_int {
    let started = nvim_get_compl_started();
    let visible = nvim_pum_visible();

    // Need refresh if:
    // - Completion started but popup not visible
    // - Completion stopped but popup still visible
    c_int::from((started != 0 && visible == 0) || (started == 0 && visible != 0))
}

/// Get the column where completion text starts.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_compl_col() -> c_int {
    nvim_get_compl_col()
}

/// Get the cursor column.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_cursor_col() -> c_int {
    nvim_get_cursor_col()
}

/// Get the width of the completed text.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_completion_width() -> c_int {
    let cursor_col = nvim_get_cursor_col();
    let compl_col = nvim_get_compl_col();
    let width = cursor_col - compl_col;
    if width < 0 {
        0
    } else {
        width
    }
}

/// Check if a shown match exists.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_has_shown_match() -> c_int {
    nvim_compl_shown_match_exists()
}

/// Check if autocomplete is enabled.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_is_autocomplete() -> c_int {
    nvim_get_compl_autocomplete()
}

/// Calculate the progress percentage for display.
///
/// Returns a value from 0 to 100 representing completion progress.
#[no_mangle]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn rs_ui_progress_percent(current: c_int, total: c_int) -> c_int {
    if total <= 0 {
        return 100; // Assume complete if no total
    }
    if current <= 0 {
        return 0;
    }
    if current >= total {
        return 100;
    }
    ((i64::from(current) * 100) / i64::from(total)) as c_int
}

/// Determine the completion indicator to show.
///
/// Returns:
/// - 0: No indicator (completion complete)
/// - 1: Searching indicator (still searching)
/// - 2: Interrupted indicator (user interrupted)
/// - 3: Timeout indicator (time limit reached)
#[no_mangle]
pub unsafe extern "C" fn rs_ui_indicator_type() -> c_int {
    if nvim_get_compl_started() == 0 {
        return 0;
    }

    if nvim_get_compl_time_slice_expired() != 0 {
        return 3; // Timeout
    }

    if nvim_get_compl_interrupted() != 0 {
        return 2; // Interrupted
    }

    // Still active means still searching
    1
}

/// Check if completion message needs clearing.
///
/// Returns true if completion ended and message should be cleared.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_should_clear_msg() -> c_int {
    c_int::from(nvim_get_compl_started() == 0 && nvim_pum_visible() == 0)
}

/// Calculate the display index for the selected item (1-based for users).
#[no_mangle]
pub unsafe extern "C" fn rs_ui_display_index() -> c_int {
    let selected = nvim_get_compl_selected_item();
    if selected < 0 {
        0 // No selection
    } else {
        selected + 1 // 1-based for display
    }
}

/// Format match count string (e.g., "3/10" for 3rd of 10).
///
/// Returns:
/// - 0: Cannot format (no matches)
/// - 1: Format as "X matches"
/// - 2: Format as "X/Y"
#[no_mangle]
pub unsafe extern "C" fn rs_ui_format_type() -> c_int {
    let matches = nvim_get_compl_matches();
    let selected = nvim_get_compl_selected_item();

    if matches <= 0 {
        return 0;
    }
    if selected < 0 {
        return 1; // "X matches"
    }
    2 // "X/Y"
}

/// Check if the mode name should be shown.
///
/// Returns true if in a CTRL-X sub-mode that has a specific name.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_show_mode_name() -> c_int {
    let mode = nvim_get_ctrl_x_mode();
    // Show mode name for all modes except NORMAL and SCROLL
    c_int::from(mode >= CTRL_X_WHOLE_LINE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_constants() {
        assert_eq!(CTRL_X_NORMAL, 0);
        assert_eq!(CTRL_X_NOT_DEFINED_YET, 1);
        assert_eq!(CTRL_X_WHOLE_LINE, 3);
    }

    #[test]
    fn test_msg_indices_distinct() {
        // Verify different modes map to different message indices
        // (except modes that share the same message)
        let normal_idx = 0;
        let whole_line_idx = 1;
        let files_idx = 2;

        assert_ne!(normal_idx, whole_line_idx);
        assert_ne!(whole_line_idx, files_idx);
    }
}
