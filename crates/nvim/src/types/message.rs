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

/// Not `Copy`: `text` is an owned string.
#[derive(Clone)]
pub struct HlMessageChunk {
    pub text: String_0,
    pub hl_id: ::core::ffi::c_int,
}
pub type MessageData = msg_data;
#[derive(Copy, Clone)]
pub struct msg_data {
    pub source: String_0,
    pub percent: Integer,
    pub title: String_0,
    pub status: String_0,
    pub data: Dict,
}
