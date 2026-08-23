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
use super::multiqueue_list::ItemList;
use super::*;

#[derive(Copy, Clone)]
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
