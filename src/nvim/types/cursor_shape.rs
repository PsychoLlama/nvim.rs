// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

pub type CursorShape = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct cursorentry_T {
    pub full_name: *mut ::core::ffi::c_char,
    pub shape: CursorShape,
    pub mshape: ::core::ffi::c_int,
    pub percentage: ::core::ffi::c_int,
    pub blinkwait: ::core::ffi::c_int,
    pub blinkon: ::core::ffi::c_int,
    pub blinkoff: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
    pub id_lm: ::core::ffi::c_int,
    pub name: *mut ::core::ffi::c_char,
    pub used_for: ::core::ffi::c_char,
}
