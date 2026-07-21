// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct color_name_table_T {
    pub name: *mut ::core::ffi::c_char,
    pub color: RgbValue,
}
