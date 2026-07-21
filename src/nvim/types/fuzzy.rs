// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct fuzmatch_str_T {
    pub idx: ::core::ffi::c_int,
    pub str: *mut ::core::ffi::c_char,
    pub score: ::core::ffi::c_int,
}
