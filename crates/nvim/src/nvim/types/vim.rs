#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

pub type CdCause = ::core::ffi::c_int;
pub type CdScope = ::core::ffi::c_int;
pub type Direction = ::core::ffi::c_int;
