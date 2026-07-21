// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct UCell {
    pub data: schar_T,
    pub attr: sattr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UGrid {
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub cells: *mut *mut UCell,
}
