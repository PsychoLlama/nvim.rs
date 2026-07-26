#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
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
