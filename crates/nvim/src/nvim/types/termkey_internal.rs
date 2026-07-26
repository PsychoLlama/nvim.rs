#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct keyinfo {
    pub type_0: TermKeyType,
    pub sym: TermKeySym,
    pub modifier_mask: ::core::ffi::c_int,
    pub modifier_set: ::core::ffi::c_int,
}
pub type ssize_t = isize;
