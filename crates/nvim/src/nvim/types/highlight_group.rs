#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
pub struct color_name_table_T {
    pub name: *mut ::core::ffi::c_char,
    pub color: RgbValue,
}
