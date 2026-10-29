#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// libmpack's token typedefs, named as the codec names them.
#![allow(non_camel_case_types)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
pub type mpack_token_type_t = ::core::ffi::c_uint;
pub type mpack_uint32_t = ::core::ffi::c_uint;
