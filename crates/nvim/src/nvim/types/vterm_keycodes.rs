#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

pub type VTermKey = ::core::ffi::c_uint;
pub type VTermModifier = ::core::ffi::c_uint;
