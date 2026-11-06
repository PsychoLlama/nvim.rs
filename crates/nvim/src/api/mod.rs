//! The `nvim_*` API: the functions msgpack-RPC and Lua both call into.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

pub mod autocmd;
pub mod buffer;
pub mod command;
pub mod deprecated;
pub mod events;
pub mod extmark;
pub mod options;
pub mod private;
pub mod tabpage;
pub mod ui;
pub mod vim;
pub mod vimscript;
pub mod win_config;
pub mod window;
