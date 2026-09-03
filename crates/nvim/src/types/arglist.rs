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

#[derive(Copy, Clone)]
pub struct aentry_T {
    pub ae_fname: *mut ::core::ffi::c_char,
    pub ae_fnum: ::core::ffi::c_int,
}
pub struct alist_T {
    /// The entries, each owning its `ae_fname`. Not a `Drop` impl: a list is
    /// released through `alist_unlink`, which frees the names first.
    pub al_ga: Vec<aentry_T>,
    pub al_refcount: Refcount,
    pub id: ::core::ffi::c_int,
}
