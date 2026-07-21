// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct strbuf_t {
    pub buf: *mut ::core::ffi::c_char,
    pub size: size_t,
    pub length: size_t,
    pub dynamic: ::core::ffi::c_int,
    pub reallocs: ::core::ffi::c_int,
    pub debug: ::core::ffi::c_int,
}
