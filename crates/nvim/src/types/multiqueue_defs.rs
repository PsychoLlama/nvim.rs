#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::multiqueue_list::ItemList;
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct multiqueue {
    pub parent: *mut MultiQueue,
    pub on_put: PutCallback,
    pub data: *mut ::core::ffi::c_void,
    pub size: size_t,
    /// The events and links this queue holds, in order. Owned: a
    /// `Box<ItemList>` that `multiqueue_new` leaves here and
    /// `multiqueue_free` takes back.
    pub items: *mut ItemList,
}
