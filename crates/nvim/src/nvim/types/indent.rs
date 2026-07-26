#![deny(unsafe_op_in_unsafe_fn)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

pub type IndentGetter = Option<unsafe extern "C" fn() -> ::core::ffi::c_int>;
pub type Indenter = Option<unsafe extern "C" fn() -> ::core::ffi::c_int>;
