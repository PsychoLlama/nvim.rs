#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_rpc_header_s {
    pub toks: [mpack_token_t; 3],
    pub index: ::core::ffi::c_int,
}
pub type mpack_rpc_header_t = mpack_rpc_header_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_rpc_message_s {
    pub id: mpack_uint32_t,
    pub data: mpack_data_t,
}
pub type mpack_rpc_message_t = mpack_rpc_message_s;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_rpc_slot_s {
    pub used: ::core::ffi::c_int,
    pub msg: mpack_rpc_message_t,
}
