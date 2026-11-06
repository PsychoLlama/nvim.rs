#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![deny(unsafe_op_in_unsafe_fn)]

//! The Lua runtime: the LuaJIT FFI bindings, the `vim.*` stdlib, and the
//! bridge between Lua values and the editor's own.

pub mod api_wrappers;
pub mod base64;
pub mod converter;
pub mod executor;
pub mod ffi;
pub mod secure;
pub mod spell;
pub mod stack;
pub(crate) mod state;
pub mod stdlib;
pub mod treesitter;
pub mod xdiff;
