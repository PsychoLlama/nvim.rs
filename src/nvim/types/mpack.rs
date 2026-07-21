// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
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
#[derive(Copy, Clone)]
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
