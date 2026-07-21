// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdchange {
    pub next: *mut s_xdchange,
    pub i1: ::core::ffi::c_long,
    pub i2: ::core::ffi::c_long,
    pub chg1: ::core::ffi::c_long,
    pub chg2: ::core::ffi::c_long,
    pub ignore: ::core::ffi::c_int,
}
pub type xdchange_t = s_xdchange;
