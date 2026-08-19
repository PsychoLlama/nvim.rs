#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

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

/// The most bytes one *character* — a base plus its composing marks — can
/// occupy in the places that only need to round-trip a single character:
/// six, one over the longest legal UTF-8 sequence `utf_char2bytes` writes.
///
/// `usize`, as `MB_MAXBYTES` is, because every use is an array length. Three
/// modules had grown a private copy.
pub const MB_MAXCHAR: usize = 6;

/// `vimconv_T::vc_type` — upstream's `ConvFlags`.
///
/// `c_int`, which is what the `vc_type` field is: c2rust typed the anonymous
/// enum `c_uint` from what the C compiler picked, and every one of the 55 use
/// sites cast it back. B15-9 deleted the casts with the retype.
pub type ConvFlags = ::core::ffi::c_int;

pub const CONV_NONE: ConvFlags = 0;
pub const CONV_TO_UTF8: ConvFlags = 1;
pub const CONV_9_TO_UTF8: ConvFlags = 2;
pub const CONV_TO_LATIN1: ConvFlags = 3;
pub const CONV_TO_LATIN9: ConvFlags = 4;
pub const CONV_ICONV: ConvFlags = 5;

/// A `vimconv_T` that converts nothing — what `convert_setup` starts from and
/// what a caller with no conversion to do passes around.
pub const CONV_NONE_INIT: vimconv_T = vimconv_T {
    vc_type: CONV_NONE,
    vc_factor: 1,
    vc_fd: ::core::ptr::null_mut(),
    vc_fail: false,
};
