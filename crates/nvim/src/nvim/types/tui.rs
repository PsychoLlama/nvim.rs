#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

pub type TermMode = ::core::ffi::c_uint;
pub type TermModeState = ::core::ffi::c_uint;
