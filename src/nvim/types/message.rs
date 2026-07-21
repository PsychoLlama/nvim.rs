// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HlMessageChunk {
    pub text: String_0,
    pub hl_id: ::core::ffi::c_int,
}
pub type MessageData = msg_data;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct msg_data {
    pub source: String_0,
    pub percent: Integer,
    pub title: String_0,
    pub status: String_0,
    pub data: Dict,
}
