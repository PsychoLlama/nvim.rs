#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

#[derive(Copy, Clone)]
#[repr(C)]
pub struct fuzmatch_str_T {
    pub idx: ::core::ffi::c_int,
    pub str: *mut ::core::ffi::c_char,
    pub score: ::core::ffi::c_int,
}
