// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct keyvalue_T {
    pub key: ::core::ffi::c_int,
    pub value: *mut ::core::ffi::c_char,
    pub length: size_t,
}
