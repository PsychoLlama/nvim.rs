#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

pub type IndentGetter = Option<unsafe fn() -> ::core::ffi::c_int>;
pub type Indenter = Option<unsafe fn() -> ::core::ffi::c_int>;
