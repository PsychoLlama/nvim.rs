#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
pub struct SignItem {
    pub sh: *mut DecorSignHighlight,
    pub id: uint32_t,
}
#[derive(Copy, Clone)]
pub struct SignTextAttrs {
    pub text: [schar_T; 2],
    pub hl_id: ::core::ffi::c_int,
}
#[derive(Copy, Clone, Default)]
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
