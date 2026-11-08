//! The bundled `libmpack` MessagePack codec and its Lua binding.

#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

pub mod conv;
pub mod lmpack;
pub mod mpack_core;
pub mod object;
pub mod rpc;
pub mod token;
pub mod typedefs;
