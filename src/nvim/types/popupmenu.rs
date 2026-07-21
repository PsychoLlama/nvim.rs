// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct pumitem_T {
    pub pum_text: *mut ::core::ffi::c_char,
    pub pum_kind: *mut ::core::ffi::c_char,
    pub pum_extra: *mut ::core::ffi::c_char,
    pub pum_info: *mut ::core::ffi::c_char,
    pub pum_cpt_source_idx: ::core::ffi::c_int,
    pub pum_user_abbr_hlattr: ::core::ffi::c_int,
    pub pum_user_kind_hlattr: ::core::ffi::c_int,
}
