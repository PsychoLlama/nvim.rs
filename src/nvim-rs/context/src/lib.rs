//! Context stack handling for Neovim
//!
//! Provides Rust implementations of context stack functions.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(unsafe_code)]

extern "C" {
    /// Get the size of the context stack
    fn nvim_get_ctx_stack_size() -> usize;
}

/// Returns the size of the context stack.
///
/// # Safety
/// Calls C accessor function for `ctx_stack`.
#[no_mangle]
pub unsafe extern "C" fn rs_ctx_size() -> usize {
    nvim_get_ctx_stack_size()
}
