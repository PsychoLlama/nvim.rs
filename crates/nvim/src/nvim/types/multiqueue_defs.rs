#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiqueue {
    pub parent: *mut MultiQueue,
    pub headtail: QUEUE,
    pub on_put: PutCallback,
    pub data: *mut ::core::ffi::c_void,
    pub size: size_t,
}
