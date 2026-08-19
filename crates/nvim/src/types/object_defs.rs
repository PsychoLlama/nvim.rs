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
pub union mpack_data_t {
    pub p: *mut ::core::ffi::c_void,
    pub u: mpack_uintmax_t,
    pub i: mpack_sintmax_t,
    pub d: ::core::ffi::c_double,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_node_s {
    pub tok: mpack_token_t,
    pub pos: size_t,
    pub key_visited: ::core::ffi::c_int,
    pub data: [mpack_data_t; 2],
}
pub type mpack_node_t = mpack_node_s;
pub type mpack_walk_cb =
    Option<unsafe extern "C-unwind" fn(*mut mpack_parser_t, *mut mpack_node_t) -> ()>;
