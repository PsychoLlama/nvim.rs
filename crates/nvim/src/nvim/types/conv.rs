#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

pub type mpack_sintmax_t = ::core::ffi::c_longlong;
pub type mpack_uintmax_t = ::core::ffi::c_ulonglong;
