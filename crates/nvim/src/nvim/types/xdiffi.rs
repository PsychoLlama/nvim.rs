#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

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
