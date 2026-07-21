// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SignItem {
    pub sh: *mut DecorSignHighlight,
    pub id: uint32_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SignTextAttrs {
    pub text: [schar_T; 2],
    pub hl_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct sign_T {
    pub sn_name: *mut ::core::ffi::c_char,
    pub sn_icon: *mut ::core::ffi::c_char,
    pub sn_text: [schar_T; 2],
    pub sn_line_hl: ::core::ffi::c_int,
    pub sn_text_hl: ::core::ffi::c_int,
    pub sn_cul_hl: ::core::ffi::c_int,
    pub sn_num_hl: ::core::ffi::c_int,
    pub sn_priority: ::core::ffi::c_int,
}
