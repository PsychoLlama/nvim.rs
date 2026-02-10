//! Edit mode for Neovim
//!
//! This crate provides Rust implementations of edit-related functions
//! from `src/nvim/edit.c`. It handles insert mode state management,
//! cursor position tracking, and mode-specific operations.

#![allow(unsafe_code)] // FFI requires unsafe

pub mod abbrev;
pub mod completion;
pub mod helpers;
pub mod insert;
pub mod key_handlers;
pub mod keys;
pub mod mode;
pub mod movement;
pub mod replace;
pub mod state;
pub mod undo;

use std::ffi::{c_char, c_int};

/// Line number type (matches `linenr_T` in Neovim).
type LinenrT = i32;

/// Column number type (matches `colnr_T` in Neovim).
type ColnrT = i32;

// C accessor functions for edit state.
// These are defined in edit.c and provide safe access to static variables.
extern "C" {
    /// Get the `ins_need_undo` static variable.
    fn nvim_get_ins_need_undo() -> c_int;
    /// Set the `ins_need_undo` static variable.
    fn nvim_set_ins_need_undo(val: c_int);
    /// Get the `can_cindent` static variable.
    fn nvim_get_can_cindent() -> c_int;
    /// Set the `can_cindent` static variable.
    fn nvim_set_can_cindent(val: c_int);
    /// Get the `revins_on` static variable.
    fn nvim_get_revins_on() -> c_int;
    /// Get the `did_restart_edit` static variable.
    fn nvim_get_did_restart_edit() -> c_int;
    /// Get buf->b_prompt_text for a buffer.
    fn nvim_buf_get_b_prompt_text(buf: *const std::ffi::c_void) -> *const c_char;
    /// Get curbuf handle.
    fn nvim_get_curbuf() -> *const std::ffi::c_void;
    /// Get curwin->w_cursor.lnum.
    fn nvim_curwin_get_cursor_lnum() -> LinenrT;
    /// Get curwin->w_cursor.col.
    fn nvim_curwin_get_cursor_col() -> ColnrT;
    /// Get curbuf->b_prompt_start.mark.lnum.
    fn nvim_curbuf_get_b_prompt_start_lnum() -> LinenrT;
}

/// Check if undo is needed for insert mode.
///
/// Returns the value of the static `ins_need_undo` variable.
#[inline]
fn ins_need_undo_get_impl() -> bool {
    // SAFETY: nvim_get_ins_need_undo is a simple global accessor
    unsafe { nvim_get_ins_need_undo() != 0 }
}

/// FFI wrapper for `ins_need_undo_get`.
#[no_mangle]
pub extern "C" fn rs_ins_need_undo_get() -> c_int {
    c_int::from(ins_need_undo_get_impl())
}

/// Get whether cindenting may be done on this line.
///
/// # Safety
/// Calls C accessor function for `can_cindent` static.
#[no_mangle]
pub unsafe extern "C" fn rs_get_can_cindent() -> c_int {
    nvim_get_can_cindent()
}

/// Set whether cindenting may be done on this line.
///
/// # Safety
/// Calls C accessor function for `can_cindent` static.
#[no_mangle]
pub unsafe extern "C" fn rs_set_can_cindent(val: c_int) {
    nvim_set_can_cindent(val);
}

/// Set whether undo is needed for the next insert.
///
/// # Safety
/// Calls C accessor function for `ins_need_undo` static.
#[no_mangle]
pub unsafe extern "C" fn rs_set_ins_need_undo(val: c_int) {
    nvim_set_ins_need_undo(val);
}

/// Check if reverse insert mode is on.
///
/// Returns true if `revins_on` is true (reverse insert mode).
///
/// # Safety
/// Calls C accessor function for `revins_on` static.
#[no_mangle]
pub unsafe extern "C" fn rs_get_revins_on() -> c_int {
    nvim_get_revins_on()
}

/// Check if edit was restarted.
///
/// Returns the value of `did_restart_edit`.
///
/// # Safety
/// Calls C accessor function for `did_restart_edit` static.
#[no_mangle]
pub unsafe extern "C" fn rs_get_did_restart_edit() -> c_int {
    nvim_get_did_restart_edit()
}

/// Default prompt text used when buffer has no custom prompt.
const DEFAULT_PROMPT: &[u8] = b"% \0";

/// Get the effective prompt text for the specified buffer.
///
/// Returns the buffer's `b_prompt_text` if set, otherwise returns "% ".
///
/// # Safety
/// The buf pointer must be a valid `buf_T` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_prompt_text(buf: *const std::ffi::c_void) -> *const c_char {
    let prompt = nvim_buf_get_b_prompt_text(buf);
    if prompt.is_null() {
        DEFAULT_PROMPT.as_ptr().cast()
    } else {
        prompt
    }
}

/// Get the effective prompt for the current buffer.
///
/// # Safety
/// Accesses curbuf global.
#[no_mangle]
pub unsafe extern "C" fn rs_prompt_text() -> *const c_char {
    rs_buf_prompt_text(nvim_get_curbuf())
}

/// Check if the cursor is in the editable position of the prompt line.
///
/// Returns true if the cursor is past the prompt text on the prompt line.
///
/// # Safety
/// Accesses curwin and curbuf globals via accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_prompt_curpos_editable() -> bool {
    let cursor_lnum = nvim_curwin_get_cursor_lnum();
    let prompt_start_lnum = nvim_curbuf_get_b_prompt_start_lnum();

    if cursor_lnum > prompt_start_lnum {
        return true;
    }

    if cursor_lnum == prompt_start_lnum {
        let cursor_col = nvim_curwin_get_cursor_col();
        let prompt = rs_prompt_text();
        // strlen of the prompt text
        let prompt_len = if prompt.is_null() {
            0
        } else {
            let mut len = 0usize;
            while *prompt.add(len) != 0 {
                len += 1;
            }
            // Safe: prompt strings are always short (well under i32::MAX)
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            {
                len as ColnrT
            }
        };
        return cursor_col >= prompt_len;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_prompt_format() {
        // Verify the default prompt is "% " with null terminator
        assert_eq!(DEFAULT_PROMPT, b"% \0");
        assert_eq!(DEFAULT_PROMPT.len(), 3);
    }

    #[test]
    fn test_default_prompt_null_terminated() {
        // Ensure the prompt is properly null-terminated for C interop
        assert_eq!(DEFAULT_PROMPT[DEFAULT_PROMPT.len() - 1], 0);
    }

    #[test]
    fn test_default_prompt_starts_with_percent() {
        // Prompt should start with '%'
        assert_eq!(DEFAULT_PROMPT[0], b'%');
    }

    #[test]
    fn test_type_aliases_sizes() {
        // LinenrT and ColnrT should be i32
        assert_eq!(std::mem::size_of::<LinenrT>(), 4);
        assert_eq!(std::mem::size_of::<ColnrT>(), 4);
    }

    #[test]
    fn test_default_prompt_is_valid_cstr() {
        // Should be usable as a C string (non-empty, null-terminated)
        let prompt = DEFAULT_PROMPT;
        // Has content before the null terminator
        assert!(prompt.len() >= 2, "Prompt must have content plus NUL");
        // No embedded NULs except at end
        let without_terminator = &prompt[..prompt.len() - 1];
        assert!(!without_terminator.contains(&0));
    }

    #[test]
    fn test_linenr_range() {
        // LinenrT should be able to hold reasonable line numbers
        let max_lines: LinenrT = 1_000_000;
        assert!(max_lines > 0);
        let min_lines: LinenrT = 1;
        assert_eq!(min_lines, 1);
    }

    #[test]
    fn test_colnr_range() {
        // ColnrT should be able to hold reasonable column numbers
        let max_cols: ColnrT = 100_000;
        assert!(max_cols > 0);
    }
}
