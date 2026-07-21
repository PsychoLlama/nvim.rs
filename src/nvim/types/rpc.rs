// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
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
