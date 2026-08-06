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

/// The most bytes one multi-byte character can occupy: a 16-bit character of
/// up to three bytes plus six composing characters of three bytes each, or a
/// 32-bit character of up to six.
///
/// `usize` because most of its uses are array lengths; the transpiled sites
/// spell the cast out.
pub const MB_MAXBYTES: usize = 21;

/// `vimconv_T::vc_type` — upstream's `ConvFlags`.
///
/// c2rust typed this `c_uint` while the field it is compared against is a
/// `c_int`, so every use site casts. Retyping belongs to the slice that
/// deletes those casts, not here.
pub type ConvFlags = ::core::ffi::c_uint;

pub const CONV_NONE: ConvFlags = 0;
pub const CONV_TO_UTF8: ConvFlags = 1;
pub const CONV_9_TO_UTF8: ConvFlags = 2;
pub const CONV_TO_LATIN1: ConvFlags = 3;
pub const CONV_TO_LATIN9: ConvFlags = 4;
pub const CONV_ICONV: ConvFlags = 5;
