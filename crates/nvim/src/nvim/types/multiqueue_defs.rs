#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiqueue {
    pub parent: *mut MultiQueue,
    pub on_put: PutCallback,
    pub data: *mut ::core::ffi::c_void,
    pub size: size_t,
    /// The events and links this queue holds, in order. Owned; a
    /// `Box<ItemList>` in `event::multiqueue`'s terms, opaque here so the
    /// struct keeps a C-visible layout — several `extern` blocks declare
    /// functions taking types that embed a `MultiQueue *`, and the FFI-safety
    /// lint follows a pointer into the type it points at.
    pub items: *mut ::core::ffi::c_void,
}
