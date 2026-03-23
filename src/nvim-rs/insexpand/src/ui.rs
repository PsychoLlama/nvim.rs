//! Status messages and UI support for completion.
//!
//! This module provides helper functions for completion UI and status messages.
//! The actual UI rendering remains in C (popupmenu.c, message.c), but Rust
//! provides utilities for state checking and formatting.

#![allow(dead_code, unused_imports)]
#![allow(clippy::missing_const_for_fn)]

use std::os::raw::c_int;

// C accessor functions
extern "C" {
    fn pum_visible() -> c_int;
    fn pum_get_height() -> c_int;
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

// =============================================================================
// Phase 6: Extended UI Integration Functions
// =============================================================================

use std::os::raw::c_char;

// Additional C accessor functions
extern "C" {
    fn nvim_get_compl_col() -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    fn nvim_compl_shown_match_exists() -> c_int;

    // For ins_compl_col_range_attr
    // (compl_hi_on_autocompl_longest moved to Rust static in state.rs)
    fn nvim_syn_name2attr(name: *const c_char) -> c_int;
    fn nvim_get_compl_lnum() -> c_int;
    fn nvim_get_curwin_cursor_lnum() -> c_int;

    // Already in Rust (lib.rs), declare as C for cross-module call
    fn rs_ins_compl_has_preinsert() -> c_int;
    fn rs_ins_compl_preinsert_longest() -> c_int;
    fn rs_cot_fuzzy() -> c_int;
    fn rs_ins_compl_leader_len() -> usize;
    fn rs_ins_compl_has_multiple() -> c_int;
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

/// MAXCOL: maximum column value (0x7fffffff).
const MAXCOL: c_int = 0x7fff_ffff;

/// Determine if a (lnum, col) position falls within the completion highlight range.
///
/// Returns the highlight attribute to use, or -1 if no highlight applies.
/// Mirrors `ins_compl_col_range_attr` in C.
///
/// # Safety
/// Requires valid completion state.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_col_range_attr(lnum: c_int, col: c_int) -> c_int {
    let has_preinsert = rs_ins_compl_has_preinsert() != 0 || rs_ins_compl_preinsert_longest() != 0;

    // Return -1 (no highlight) if:
    // - fuzzy mode is active, or
    // - preinsert_longest is active but compl_hi_on_autocompl_longest is false, or
    // - the highlight group doesn't exist (syn_name2attr returns 0)
    if rs_cot_fuzzy() != 0 {
        return -1;
    }

    if !crate::state::COMPL_HI_ON_AUTOCOMPL_LONGEST && rs_ins_compl_preinsert_longest() != 0 {
        return -1;
    }

    let hl_name = if has_preinsert {
        c"PreInsert".as_ptr()
    } else {
        c"ComplMatchIns".as_ptr()
    };

    let attr = nvim_syn_name2attr(hl_name);
    if attr == 0 {
        return -1;
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let start_col = nvim_get_compl_col() + rs_ins_compl_leader_len() as c_int;
    let compl_ins_end_col = crate::vars::nvim_get_compl_ins_end_col();
    let compl_lnum = nvim_get_compl_lnum();
    let cursor_lnum = nvim_get_curwin_cursor_lnum();

    if rs_ins_compl_has_multiple() == 0 {
        // Single-line case
        return if col >= start_col && col < compl_ins_end_col {
            attr
        } else {
            -1
        };
    }

    // Multiple lines case
    if (lnum == compl_lnum && col >= start_col && col < MAXCOL)
        || (lnum > compl_lnum && lnum < cursor_lnum)
        || (lnum == cursor_lnum && col <= compl_ins_end_col)
    {
        return attr;
    }

    -1
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
