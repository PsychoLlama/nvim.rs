// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

pub type GRegFlags = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct block_def {
    pub startspaces: ::core::ffi::c_int,
    pub endspaces: ::core::ffi::c_int,
    pub textlen: ::core::ffi::c_int,
    pub textstart: *mut ::core::ffi::c_char,
    pub textcol: colnr_T,
    pub start_vcol: colnr_T,
    pub end_vcol: colnr_T,
    pub is_short: ::core::ffi::c_int,
    pub is_MAX: ::core::ffi::c_int,
    pub is_oneChar: ::core::ffi::c_int,
    pub pre_whitesp: ::core::ffi::c_int,
    pub pre_whitesp_c: ::core::ffi::c_int,
    pub end_char_vcols: colnr_T,
    pub start_char_vcols: colnr_T,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct yankreg_T {
    pub y_array: *mut String_0,
    pub y_size: size_t,
    pub y_type: MotionType,
    pub y_width: colnr_T,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
