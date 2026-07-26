#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharBoundsOff {
    pub begin_off: int8_t,
    pub end_off: int8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharInfo {
    pub value: int32_t,
    pub len: ::core::ffi::c_int,
}
pub type GraphemeState = utf8proc_int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrCharInfo {
    pub ptr: *mut ::core::ffi::c_char,
    pub chr: CharInfo,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vimconv_T {
    pub vc_type: ::core::ffi::c_int,
    pub vc_factor: ::core::ffi::c_int,
    pub vc_fd: iconv_t,
    pub vc_fail: bool,
}
