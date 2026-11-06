#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

//! The bundled `lua-cjson` JSON codec.

pub mod fpconv;
pub mod lua_cjson;
