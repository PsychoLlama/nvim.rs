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

#[repr(C)]
pub struct mpack_one_parser_t {
    pub data: mpack_data_t,
    pub size: mpack_uint32_t,
    pub capacity: mpack_uint32_t,
    pub status: ::core::ffi::c_int,
    pub exiting: ::core::ffi::c_int,
    pub tokbuf: mpack_tokbuf_t,
    pub items: [mpack_node_t; 1],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct mpack_parser_t {
    pub data: mpack_data_t,
    pub size: mpack_uint32_t,
    pub capacity: mpack_uint32_t,
    pub status: ::core::ffi::c_int,
    pub exiting: ::core::ffi::c_int,
    pub tokbuf: mpack_tokbuf_t,
    pub items: [mpack_node_t; 33],
}
#[repr(C)]
pub struct mpack_rpc_one_session_t {
    pub reader: mpack_tokbuf_t,
    pub writer: mpack_tokbuf_t,
    pub receive: mpack_rpc_header_t,
    pub send: mpack_rpc_header_t,
    pub request_id: mpack_uint32_t,
    pub capacity: mpack_uint32_t,
    pub slots: [mpack_rpc_slot_s; 1],
}
#[repr(C)]
pub struct mpack_rpc_session_t {
    pub reader: mpack_tokbuf_t,
    pub writer: mpack_tokbuf_t,
    pub receive: mpack_rpc_header_t,
    pub send: mpack_rpc_header_t,
    pub request_id: mpack_uint32_t,
    pub capacity: mpack_uint32_t,
    pub slots: [mpack_rpc_slot_s; 32],
}
