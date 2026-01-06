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

pub mod attr;
pub mod chunk;
pub mod format;
pub mod history;
pub mod output;
pub mod scrollback;

// Re-export FFI functions for the static library
pub use attr::*;
pub use chunk::*;
pub use format::*;
pub use history::*;
pub use output::*;
pub use scrollback::*;

#[cfg(test)]
mod tests {
    // Integration tests would require mocking C functions
    // Unit tests for pure Rust logic go here
}
