//! Message display functions
//!
//! Provides Rust implementations for message display operations including
//! ext_messages UI protocol handling, scrolling coordination, and
//! display state management.

use std::ffi::{c_char, c_int};

// ============================================================================
// C Function Declarations
// ============================================================================

/// Message kind for ext_messages UI protocol (owned by Rust, also accessed from C).
#[no_mangle]
pub static mut msg_ext_kind: *const c_char = std::ptr::null();

/// Verbose message kind (saved/restored across verbose_enter/leave pairs).
#[no_mangle]
pub static mut verbose_kind: *const c_char = std::ptr::null();

/// Pre-verbose message kind (saved before entering verbose mode).
#[no_mangle]
pub static mut pre_verbose_kind: *const c_char = std::ptr::null();

/// UIExtension value for kUIMessages (ui_defs.h)
const K_UI_MESSAGES: c_int = 4;

extern "C" {
    static Rows: c_int;
    // ext_messages protocol functions
    fn msg_ext_ui_flush();
    fn msg_ext_flush_showmode();

    // Display state accessors (getters in format.rs, only setters needed here)
    static mut msg_row: c_int;
    static mut msg_col: c_int;
    static mut cmdline_row: c_int;

    // UI capability check
    fn ui_has(ext: c_int) -> bool;

    // Display coordination
    fn msg_grid_validate();

    // grid_line_mirror and grid_line_flush_if_valid_row are implemented in Rust (grid crate)
    fn grid_line_mirror(width: c_int);
    fn grid_line_flush_if_valid_row();
    static mut cmdmsg_rl: bool;
    static mut msg_grid: crate::ScreenGrid;

    // Position and display state
    static mut sc_col: c_int;
    fn nvim_set_redraw_cmdline(val: bool);

    // Wait state — direct access to C global
    static mut did_wait_return: bool;

    // Overwrite state — direct access to C global
    static mut msg_ext_overwrite: bool;

    // Skip flush state — direct access to C global
    static mut msg_ext_skip_flush: bool;

    // Clear EOS flag — direct access to C global
    static mut need_clr_eos: bool;
    // nvim_set_need_clr_eos kept for other crates
    static mut need_wait_return: bool;
}

// ============================================================================
// ext_messages Protocol Functions
// ============================================================================

/// Set the message kind for ext_messages UI protocol.
///
/// This sets the kind label for the next batch of messages
/// sent to external UIs.
///
/// # Arguments
/// * `msg_kind` - The message kind string (e.g., "emsg", "echo")
///
/// # Safety
/// - `msg_kind` must be a valid NUL-terminated C string or NULL
#[export_name = "msg_ext_set_kind"]
pub unsafe extern "C" fn rs_msg_ext_set_kind(msg_kind: *const c_char) {
    // Don't change the label of an existing batch:
    msg_ext_ui_flush();
    msg_ext_kind = msg_kind;
}

/// Flush pending messages to ext_messages UI.
///
/// This emits any accumulated message chunks to external UIs
/// using the `msg_show` UI event.
///
/// # Safety
/// Calls C function that may emit UI events.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_ui_flush() {
    msg_ext_ui_flush();
}

/// Flush showmode messages to ext_messages UI.
///
/// Similar to `rs_msg_ext_ui_flush()` but uses the separate
/// `msg_showmode` event which doesn't interrupt normal message flow.
///
/// # Safety
/// Calls C function that may emit UI events.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_flush_showmode() {
    msg_ext_flush_showmode();
}

/// Check if UI has ext_messages capability.
///
/// # Returns
/// * Non-zero if ext_messages UI protocol is active
/// * Zero if using traditional terminal messages
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_ui_has_messages() -> c_int {
    c_int::from(ui_has(K_UI_MESSAGES))
}

// ============================================================================
// Display Position Functions
// ============================================================================

// Note: rs_msg_row(), rs_msg_col(), rs_cmdline_row() and their setters
// are defined in format.rs to avoid duplication

/// Set the command line row.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_cmdline_row(val: c_int) {
    cmdline_row = val;
}

/// Reset message position to start of message area.
///
/// Sets msg_col to 0.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_reset_col() {
    msg_col = 0;
}

/// Move message position to a new line.
///
/// Sets msg_col to 0 and increments msg_row.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_newline() {
    msg_col = 0;
    let row = msg_row;
    msg_row = row + 1;
}

// ============================================================================
// Display State Functions
// ============================================================================

/// Check if wait_return was called.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_did_wait_return() -> c_int {
    c_int::from(did_wait_return)
}

/// Set the did_wait_return flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_did_wait_return(val: c_int) {
    did_wait_return = val != 0;
}

/// Check if ext_messages should overwrite previous message.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_overwrite() -> c_int {
    c_int::from(msg_ext_overwrite)
}

/// Set the ext_messages overwrite flag.
///
/// When true, the next message will overwrite the previous one
/// in external UIs.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_ext_overwrite(val: c_int) {
    msg_ext_overwrite = val != 0;
}

/// Check if ext_messages flush is skipped.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_skip_flush() -> c_int {
    c_int::from(msg_ext_skip_flush)
}

/// Set the ext_messages skip flush flag.
///
/// When true, message chunks are accumulated but not flushed.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_ext_skip_flush(val: c_int) {
    msg_ext_skip_flush = val != 0;
}

/// Check if clear to end of screen is needed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_need_clr_eos() -> c_int {
    c_int::from(need_clr_eos)
}

/// Set the need_clr_eos flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_need_clr_eos(val: c_int) {
    need_clr_eos = (val) != 0;
}

// ============================================================================
// Display Coordination Functions
// ============================================================================

/// Check if message display overlaps with command/ruler.
///
/// If the written message runs into the shown command or ruler,
/// sets need_wait_return and schedules a redraw.
///
/// # Safety
/// Calls C accessor functions that read and modify global state.
#[export_name = "msg_check"]
pub unsafe extern "C" fn rs_msg_check() {
    if ui_has(K_UI_MESSAGES) {
        return;
    }
    if msg_row == Rows - 1 && msg_col >= sc_col {
        need_wait_return = true;
        nvim_set_redraw_cmdline(true);
    }
}

/// Validate the message grid for output.
///
/// Ensures the message grid is properly allocated and sized
/// for message display.
///
/// # Safety
/// Calls C function that may allocate memory.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_grid_validate() {
    msg_grid_validate();
}

/// Flush pending line content to display.
///
/// For right-to-left command lines, mirrors the line first.
/// Then flushes the grid line if the row is valid.
///
/// Equivalent to the C function `msg_line_flush()`.
///
/// # Safety
/// Calls grid functions that modify display state.
#[export_name = "msg_line_flush"]
pub unsafe extern "C" fn rs_msg_line_flush() {
    if cmdmsg_rl {
        grid_line_mirror(msg_grid.cols);
    }
    grid_line_flush_if_valid_row();
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Begin an ext_messages batch with the given kind.
///
/// Sets the message kind and enables skip_flush to accumulate
/// chunks before sending.
///
/// # Arguments
/// * `kind` - Message kind string
///
/// # Safety
/// - `kind` must be a valid NUL-terminated C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_begin(kind: *const c_char) {
    rs_msg_ext_set_kind(kind);
    msg_ext_skip_flush = true;
}

/// End an ext_messages batch and flush.
///
/// Clears skip_flush and flushes accumulated chunks to the UI.
///
/// # Safety
/// Calls C functions that may emit UI events.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_end() {
    msg_ext_skip_flush = false;
    msg_ext_ui_flush();
}

/// Reset display state for new message sequence.
///
/// Clears position and state flags for a fresh start.
///
/// # Safety
/// Calls C mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_display_reset() {
    msg_col = 0;
    did_wait_return = false;
    need_clr_eos = false;
}

/// Check if displaying to external UI.
///
/// Returns true if messages go to ext_messages UI rather than
/// the internal terminal grid.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_using_ext_messages() -> c_int {
    c_int::from(ui_has(K_UI_MESSAGES))
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
