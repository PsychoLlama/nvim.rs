//! Scrollback buffer management
//!
//! Implements scrollback storage for the "more" and "hit-enter" prompts.
//! Text is stored in a linked list of chunks that can be displayed when
//! scrolling back through message history.

use std::ffi::c_int;
use std::ptr;

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

use crate::chunk::MsgChunk;

// C accessor declarations
extern "C" {
    /// xfree wrapper
    fn xfree(ptr: *mut std::ffi::c_void);
}

// ============================================================================
// Rust-owned statics (previously file-local in message.c)
// ============================================================================

/// Last message chunk in scrollback buffer (replaces C static last_msgchunk)
#[no_mangle]
pub static mut last_msgchunk: *mut MsgChunk = std::ptr::null_mut();

/// Scrollback clear state (replaces C static do_clear_sb_text, 0 = SB_CLEAR_NONE)
#[no_mangle]
pub static mut do_clear_sb_text: c_int = 0;

/// Whether to clear temporary history entries (replaces C static do_clear_hist_temp)
#[no_mangle]
pub static mut do_clear_hist_temp: bool = true;

/// Scrollback clear state (mirrors sb_clear_T in C)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SbClearState(pub c_int);

impl SbClearState {
    /// Don't clear anything
    pub const NONE: Self = Self(0);
    /// Clear all scrollback text
    pub const ALL: Self = Self(1);
    /// Command line is busy, don't clear yet
    pub const CMDLINE_BUSY: Self = Self(2);
    /// Command line done, clear on next message
    pub const CMDLINE_DONE: Self = Self(3);
}

/// Get the current scrollback clear state.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_clear_state() -> c_int {
    do_clear_sb_text
}

/// Set the scrollback clear state.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_sb_clear_state(state: c_int) {
    do_clear_sb_text = state;
}

/// Check if scrollback should be cleared on next message.
///
/// Returns true if clear state is ALL or CMDLINE_DONE.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_should_clear() -> c_int {
    let state = do_clear_sb_text;
    c_int::from(state == SbClearState::ALL.0 || state == SbClearState::CMDLINE_DONE.0)
}

/// Check if in command line busy state.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_cmdline_busy() -> c_int {
    c_int::from(do_clear_sb_text == SbClearState::CMDLINE_BUSY.0)
}

/// Get the start of a screen line in the scrollback buffer.
///
/// Walks backwards through chunks until finding one where the previous
/// chunk has sb_eol set (or there is no previous chunk).
///
/// # Safety
/// Direct field access on repr(C) struct.
unsafe fn msg_sb_start(mps: *mut MsgChunk) -> *mut MsgChunk {
    if mps.is_null() {
        return ptr::null_mut();
    }

    let mut mp = mps;
    loop {
        let prev = (*mp).sb_prev;
        if prev.is_null() {
            break;
        }
        if (*prev).sb_eol != 0 {
            break;
        }
        mp = prev;
    }
    mp
}

/// Mark the end of a line in scrollback.
///
/// # Safety
/// Direct field access on repr(C) struct.
#[export_name = "msg_sb_eol"]
pub unsafe extern "C" fn rs_sb_mark_eol() {
    let last = last_msgchunk;
    if !last.is_null() {
        (*last).sb_eol = 1;
    }
}

/// Start entering a command line - prepare scrollback state.
///
/// If already in busy state (nested command line), clears the last
/// unfinished line. Otherwise marks current position and sets busy state.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[export_name = "sb_text_start_cmdline"]
pub unsafe extern "C" fn rs_sb_start_cmdline() {
    let state = do_clear_sb_text;

    if state == SbClearState::CMDLINE_BUSY.0 {
        // Recursive command line: clear last unfinished line
        rs_sb_restart_cmdline();
    } else {
        rs_sb_mark_eol();
        do_clear_sb_text = SbClearState::CMDLINE_BUSY.0;
    }
}

/// Restart command line - clear last unfinished line.
///
/// Called when redrawing the command line.
///
/// # Safety
/// Direct field access on repr(C) struct, frees memory.
#[export_name = "sb_text_restart_cmdline"]
pub unsafe extern "C" fn rs_sb_restart_cmdline() {
    do_clear_sb_text = SbClearState::CMDLINE_BUSY.0;

    let last = last_msgchunk;
    if last.is_null() || (*last).sb_eol != 0 {
        // No unfinished line
        return;
    }

    // Find start of the unfinished line and free it
    let tofree = msg_sb_start(last);
    if !tofree.is_null() {
        let prev = (*tofree).sb_prev;
        last_msgchunk = prev;

        if !prev.is_null() {
            (*prev).sb_next = ptr::null_mut();
        }

        // Free all chunks in this line
        let mut current = tofree;
        while !current.is_null() {
            let next = (*current).sb_next;
            xfree(current.cast());
            current = next;
        }
    }
}

/// End command line - schedule cleanup of old lines.
///
/// # Safety
/// Calls C mutator function.
#[export_name = "sb_text_end_cmdline"]
pub unsafe extern "C" fn rs_sb_end_cmdline() {
    do_clear_sb_text = SbClearState::CMDLINE_DONE.0;
}

/// Schedule clearing all scrollback text.
///
/// Called when done showing messages and screen will be redrawn.
/// Also sets do_clear_hist_temp to clear temporary history entries.
///
/// # Safety
/// Calls C mutator functions.
#[export_name = "may_clear_sb_text"]
pub unsafe extern "C" fn rs_sb_schedule_clear() {
    do_clear_sb_text = SbClearState::ALL.0;
    do_clear_hist_temp = true;
}

/// Clear scrollback text.
///
/// If `all` is true (non-zero), clears all text.
/// If `all` is false (zero), keeps the last line.
///
/// # Safety
/// Calls C accessor and mutator functions, frees memory.
#[export_name = "clear_sb_text"]
pub unsafe extern "C" fn rs_sb_clear(all: c_int) {
    let last = last_msgchunk;

    if all != 0 {
        // Clear everything
        let mut current = last;
        while !current.is_null() {
            let prev = (*current).sb_prev;
            xfree(current.cast());
            current = prev;
        }
        last_msgchunk = ptr::null_mut();
    } else {
        // Keep the last line, clear everything before it
        if last.is_null() {
            return;
        }

        let line_start = msg_sb_start(last);
        if line_start.is_null() {
            return;
        }

        let before_start = (*line_start).sb_prev;
        if before_start.is_null() {
            // Only one line, nothing to clear
            return;
        }

        // Clear everything before the last line
        let mut current = before_start;
        while !current.is_null() {
            let prev = (*current).sb_prev;
            xfree(current.cast());
            current = prev;
        }

        // Note: line_start's prev is now invalid, but we don't need to update it
        // since we iterate from last_msgchunk going backwards
    }
}

/// Count the number of lines in scrollback.
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_line_count() -> c_int {
    let last = last_msgchunk;
    if last.is_null() {
        return 0;
    }

    let mut count = 1;
    let mut mp = msg_sb_start(last);

    while !mp.is_null() {
        let prev = (*mp).sb_prev;
        if prev.is_null() {
            break;
        }
        mp = msg_sb_start(prev);
        count += 1;
    }

    count
}

/// Check if there is content in scrollback that can be shown.
///
/// Returns true if there is more than one line (single line looks weird).
///
/// # Safety
/// Calls C accessor functions.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_has_content() -> c_int {
    let last = last_msgchunk;
    if last.is_null() {
        return 0;
    }

    let start = msg_sb_start(last);
    if start.is_null() {
        return 0;
    }

    let prev = (*start).sb_prev;
    c_int::from(!prev.is_null())
}

/// Check if scrollback buffer is empty.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_empty() -> c_int {
    c_int::from(last_msgchunk.is_null())
}

/// Get the SB_CLEAR_NONE constant.
#[no_mangle]
pub const extern "C" fn rs_sb_clear_none() -> c_int {
    SbClearState::NONE.0
}

/// Get the SB_CLEAR_ALL constant.
#[no_mangle]
pub const extern "C" fn rs_sb_clear_all() -> c_int {
    SbClearState::ALL.0
}

/// Get the SB_CLEAR_CMDLINE_BUSY constant.
#[no_mangle]
pub const extern "C" fn rs_sb_clear_cmdline_busy() -> c_int {
    SbClearState::CMDLINE_BUSY.0
}

/// Get the SB_CLEAR_CMDLINE_DONE constant.
#[no_mangle]
pub const extern "C" fn rs_sb_clear_cmdline_done() -> c_int {
    SbClearState::CMDLINE_DONE.0
}

/// Reset scrollback clear state to NONE.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_sb_reset_clear_state() {
    do_clear_sb_text = SbClearState::NONE.0;
}

// ============================================================================
// Phase 426: Additional Scroll Operations
// ============================================================================

extern "C" {
    static mut msg_scrolled: c_int;
    static mut msg_did_scroll: bool;
    fn ui_has(ext: c_int) -> bool;

    // For inlined nvim_msg_reset_scroll_grid
    static mut msg_grid: crate::ScreenGrid;
    static mut msg_scrolled_at_flush: c_int;
    static mut msg_grid_scroll_discount: c_int;
    static mut clear_cmdline: bool;
    static mut p_ch: i64;
    static Rows: c_int;
    fn msg_grid_set_pos(row: c_int, scrolled: bool);
    fn grid_clear_line(grid: *mut crate::ScreenGrid, off: usize, width: c_int, valid: bool);
    fn msg_scrollsize() -> c_int;
}

/// Scroll message display up (normal case).
///
/// Convenience wrapper without throttling or zerocmd handling.
///
/// # Safety
/// Calls Rust msg_scroll_up implementation.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_scroll_up_simple() {
    crate::output_core::rs_msg_scroll_up(0, 0);
}

// Note: rs_msg_scrolled() is defined in lib.rs

/// Set the msg_scrolled counter.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_scrolled(val: c_int) {
    msg_scrolled = val;
}

/// Increment the msg_scrolled counter.
///
/// # Safety
/// Calls C accessor/mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_inc_msg_scrolled() {
    let val = msg_scrolled;
    msg_scrolled = val + 1;
}

/// Check if message display has scrolled.
///
/// Returns true if msg_scrolled > 0.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_has_msg_scrolled() -> c_int {
    c_int::from(msg_scrolled > 0)
}

/// Check if msg_did_scroll flag is set.
///
/// Returns true if scrolling occurred during current message.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_did_scroll() -> c_int {
    c_int::from(msg_did_scroll)
}

/// Set the msg_did_scroll flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_did_scroll(val: c_int) {
    msg_did_scroll = val != 0;
}

/// Reset scroll state and message grid position.
///
/// Called when the message grid should be collapsed (e.g., after redraw).
///
/// # Safety
/// Modifies grid state globals; calls msg_grid_set_pos and grid_clear_line.
#[export_name = "msg_reset_scroll"]
pub unsafe extern "C" fn rs_msg_reset_scroll() {
    if ui_has(K_UI_MESSAGES) {
        return;
    }
    // Inlined from nvim_msg_reset_scroll_grid:
    msg_grid.throttled = false;
    let new_pos = Rows - c_int::try_from(p_ch).unwrap_or(c_int::MAX);
    msg_grid_set_pos(new_pos, false);
    clear_cmdline = true;
    if !msg_grid.chars.is_null() {
        let limit = msg_scrollsize().min(msg_grid.rows);
        for i in 0..limit {
            #[allow(clippy::cast_sign_loss)]
            let off = *msg_grid.line_offset.add(i as usize);
            grid_clear_line(std::ptr::addr_of_mut!(msg_grid), off, msg_grid.cols, false);
        }
    }
    msg_scrolled = 0;
    msg_scrolled_at_flush = 0;
    msg_grid_scroll_discount = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sb_clear_state_constants() {
        assert_eq!(SbClearState::NONE.0, 0);
        assert_eq!(SbClearState::ALL.0, 1);
        assert_eq!(SbClearState::CMDLINE_BUSY.0, 2);
        assert_eq!(SbClearState::CMDLINE_DONE.0, 3);
    }

    #[test]
    fn test_sb_clear_state_exports() {
        assert_eq!(rs_sb_clear_none(), 0);
        assert_eq!(rs_sb_clear_all(), 1);
        assert_eq!(rs_sb_clear_cmdline_busy(), 2);
        assert_eq!(rs_sb_clear_cmdline_done(), 3);
    }
}
