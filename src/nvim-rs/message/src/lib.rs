//! Message display and history for Neovim
//!
//! This crate provides Rust implementations for the message system,
//! which handles all user-facing message display: error messages,
//! warnings, info messages, prompts, confirmations, and message history.
//!
//! # Modules
//!
//! - [`history`]: Message history linked list management
//! - [`chunk`]: Scrollback buffer message chunks
//! - [`format`]: Message formatting utilities
//! - [`output`]: Message output state management
//! - [`attr`]: Message attribute handling
//! - [`scrollback`]: Scrollback buffer management
//! - [`dialog`]: Dialog and confirmation handling
//! - [`error`]: Error and warning message state
//!
//! # Note
//!
//! Some message-related functions (`rs_msg_use_grid`, `rs_msg_scrollsize`,
//! `rs_msg_do_throttle`, `rs_redirecting`, `rs_msg_use_printf`) are implemented
//! in the `grid` crate since they integrate closely with grid management.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(unsafe_code)]

use std::ffi::c_int;

pub mod attr;
pub mod chunk;
pub mod dialog;
pub mod error;
pub mod format;
pub mod history;
pub mod output;
pub mod output_core;
pub mod scrollback;

// Re-export FFI functions for the static library
pub use attr::*;
pub use chunk::*;
pub use dialog::*;
pub use error::*;
pub use format::*;
pub use history::*;
pub use output::*;
pub use output_core::*;
pub use scrollback::*;

// ============================================================================
// Message Kind Constants
// ============================================================================
//
// These constants define the different kinds of messages that can be displayed.
// They correspond to the string values used by msg_ext_set_kind() in C.

/// Message kind: Error message (emsg)
pub const MSG_KIND_EMSG: &[u8] = b"emsg\0";

/// Message kind: Echo message (echo)
pub const MSG_KIND_ECHO: &[u8] = b"echo\0";

/// Message kind: Echo message (echomsg)
pub const MSG_KIND_ECHOMSG: &[u8] = b"echomsg\0";

/// Message kind: Warning message (wmsg)
pub const MSG_KIND_WMSG: &[u8] = b"wmsg\0";

/// Message kind: Confirm dialog
pub const MSG_KIND_CONFIRM: &[u8] = b"confirm\0";

/// Message kind: Command listing
pub const MSG_KIND_LIST_CMD: &[u8] = b"list_cmd\0";

/// Message kind: Shell command output
pub const MSG_KIND_SHELL_CMD: &[u8] = b"shell_cmd\0";

/// Message kind: Buffer write
pub const MSG_KIND_BUFWRITE: &[u8] = b"bufwrite\0";

/// Message kind: Wildmenu list
pub const MSG_KIND_WILDLIST: &[u8] = b"wildlist\0";

/// Message kind: Completion
pub const MSG_KIND_COMPLETION: &[u8] = b"completion\0";

/// Message kind: Verbose message
pub const MSG_KIND_VERBOSE: &[u8] = b"verbose\0";

// ============================================================================
// Message State Accessors
// ============================================================================

extern "C" {
    /// Get `msg_scrolled` global
    fn nvim_get_msg_scrolled() -> c_int;
    /// Get `msg_scroll` flag
    fn nvim_get_msg_scroll() -> c_int;
    /// Set `msg_scroll` flag
    fn nvim_set_msg_scroll(val: c_int);
    /// Get `msg_hist_off` flag
    fn nvim_get_msg_hist_off() -> c_int;
    /// Set `msg_hist_off` flag
    fn nvim_set_msg_hist_off(val: c_int);
    /// Get `keep_msg_more` flag
    fn nvim_get_keep_msg_more() -> c_int;
}

/// Get the message scrolled count.
///
/// This is the number of screen lines that messages have scrolled.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_scrolled() -> c_int {
    nvim_get_msg_scrolled()
}

/// Check if message scrolling is enabled.
///
/// When true, msg_start() will cause scrolling.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_scroll() -> c_int {
    nvim_get_msg_scroll()
}

/// Set the message scroll flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_scroll(val: c_int) {
    nvim_set_msg_scroll(val);
}

/// Check if message history recording is disabled.
///
/// When true, messages are not added to history.
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_msg_hist_off() -> c_int {
    nvim_get_msg_hist_off()
}

/// Set the message history off flag.
///
/// # Safety
/// Calls C mutator function.
#[no_mangle]
pub unsafe extern "C" fn rs_set_msg_hist_off(val: c_int) {
    nvim_set_msg_hist_off(val);
}

/// Check if keep_msg was set by msgmore().
///
/// # Safety
/// Calls C accessor function.
#[no_mangle]
pub unsafe extern "C" fn rs_keep_msg_more() -> c_int {
    nvim_get_keep_msg_more()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_kind_constants() {
        // Verify constants are null-terminated
        assert_eq!(MSG_KIND_EMSG.last(), Some(&0));
        assert_eq!(MSG_KIND_ECHO.last(), Some(&0));
        assert_eq!(MSG_KIND_ECHOMSG.last(), Some(&0));
        assert_eq!(MSG_KIND_WMSG.last(), Some(&0));
        assert_eq!(MSG_KIND_CONFIRM.last(), Some(&0));
        assert_eq!(MSG_KIND_LIST_CMD.last(), Some(&0));
        assert_eq!(MSG_KIND_SHELL_CMD.last(), Some(&0));
        assert_eq!(MSG_KIND_BUFWRITE.last(), Some(&0));
        assert_eq!(MSG_KIND_WILDLIST.last(), Some(&0));
        assert_eq!(MSG_KIND_COMPLETION.last(), Some(&0));
        assert_eq!(MSG_KIND_VERBOSE.last(), Some(&0));
    }
}
