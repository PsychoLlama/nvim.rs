//! Message display functions
//!
//! Provides Rust implementations for message display operations including
//! ext_messages UI protocol handling, scrolling coordination, and
//! display state management.

use std::ffi::{c_char, c_int};

// ============================================================================
// C Function Declarations
// ============================================================================

extern "C" {
    // ext_messages protocol functions
    fn msg_ext_set_kind(msg_kind: *const c_char);
    fn msg_ext_ui_flush();
    fn msg_ext_flush_showmode();

    // Display state accessors (getters in format.rs, only setters needed here)
    fn nvim_get_msg_row() -> c_int;
    fn nvim_set_msg_row(val: c_int);
    fn nvim_set_msg_col(val: c_int);
    fn nvim_set_cmdline_row(val: c_int);

    // UI capability check
    fn nvim_ui_has_messages() -> c_int;

    // Display coordination
    fn msg_check();
    fn msg_starthere();
    fn msg_grid_validate();
    fn msg_line_flush();

    // Wait state
    fn nvim_get_did_wait_return() -> c_int;
    fn nvim_set_did_wait_return(val: c_int);

    // Overwrite state
    fn nvim_get_msg_ext_overwrite() -> c_int;
    fn nvim_set_msg_ext_overwrite(val: c_int);

    // Skip flush state
    fn nvim_get_msg_ext_skip_flush() -> c_int;
    fn nvim_set_msg_ext_skip_flush(val: c_int);

    // Clear EOS flag
    fn nvim_get_need_clr_eos() -> c_int;
    fn nvim_set_need_clr_eos(val: c_int);
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
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_set_kind(msg_kind: *const c_char) {
    msg_ext_set_kind(msg_kind);
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
    nvim_ui_has_messages()
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
    nvim_set_cmdline_row(val);
}

/// Reset message position to start of message area.
///
/// Sets msg_col to 0.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_reset_col() {
    nvim_set_msg_col(0);
}

/// Move message position to a new line.
///
/// Sets msg_col to 0 and increments msg_row.
///
/// # Safety
/// Calls C accessor and mutator functions.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_newline() {
    nvim_set_msg_col(0);
    let row = nvim_get_msg_row();
    nvim_set_msg_row(row + 1);
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
    nvim_get_did_wait_return()
}

/// Set the did_wait_return flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_did_wait_return(val: c_int) {
    nvim_set_did_wait_return(val);
}

/// Check if ext_messages should overwrite previous message.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_overwrite() -> c_int {
    nvim_get_msg_ext_overwrite()
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
    nvim_set_msg_ext_overwrite(val);
}

/// Check if ext_messages flush is skipped.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_skip_flush() -> c_int {
    nvim_get_msg_ext_skip_flush()
}

/// Set the ext_messages skip flush flag.
///
/// When true, message chunks are accumulated but not flushed.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_ext_skip_flush(val: c_int) {
    nvim_set_msg_ext_skip_flush(val);
}

/// Check if clear to end of screen is needed.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_need_clr_eos() -> c_int {
    nvim_get_need_clr_eos()
}

/// Set the need_clr_eos flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_need_clr_eos(val: c_int) {
    nvim_set_need_clr_eos(val);
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
/// Calls C function that may modify global state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_check() {
    msg_check();
}

/// Mark the start position for message display.
///
/// Sets msg_startvcol and msg_startrow to current position.
///
/// # Safety
/// Calls C function that modifies global state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_starthere() {
    msg_starthere();
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
/// Writes any accumulated grid line content to the screen.
///
/// # Safety
/// Calls C function that modifies display state.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_line_flush() {
    msg_line_flush();
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
    msg_ext_set_kind(kind);
    nvim_set_msg_ext_skip_flush(1);
}

/// End an ext_messages batch and flush.
///
/// Clears skip_flush and flushes accumulated chunks to the UI.
///
/// # Safety
/// Calls C functions that may emit UI events.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_ext_end() {
    nvim_set_msg_ext_skip_flush(0);
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
    nvim_set_msg_col(0);
    nvim_set_did_wait_return(0);
    nvim_set_need_clr_eos(0);
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
    nvim_ui_has_messages()
}

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
}
