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

pub type find_func_t = Option<
    unsafe extern "C" fn(
        *const ::core::ffi::c_char,
        ::core::ffi::c_long,
        *mut ::core::ffi::c_char,
        ::core::ffi::c_long,
        *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_long,
>;
pub type mmbuffer_t = s_mmbuffer;
pub type mmfile_t = s_mmfile;
pub struct s_mmbuffer {
    pub ptr: *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
}
pub struct s_mmfile {
    pub ptr: *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
}
pub struct s_xdemitcb {
    pub priv_0: *mut ::core::ffi::c_void,
    pub out_hunk: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            ::core::ffi::c_long,
            ::core::ffi::c_long,
            ::core::ffi::c_long,
            ::core::ffi::c_long,
            *const ::core::ffi::c_char,
            ::core::ffi::c_long,
        ) -> ::core::ffi::c_int,
    >,
    pub out_line: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            *mut mmbuffer_t,
            ::core::ffi::c_int,
        ) -> ::core::ffi::c_int,
    >,
}
pub struct s_xdemitconf {
    pub ctxlen: ::core::ffi::c_long,
    pub interhunkctxlen: ::core::ffi::c_long,
    pub flags: ::core::ffi::c_ulong,
    pub find_func: find_func_t,
    pub find_func_priv: *mut ::core::ffi::c_void,
    pub hunk_func: xdl_emit_hunk_consume_func_t,
}
pub struct s_xpparam {
    pub flags: ::core::ffi::c_ulong,
    pub anchors: *mut *mut ::core::ffi::c_char,
    pub anchors_nr: size_t,
}
pub type xdemitcb_t = s_xdemitcb;
pub type xdemitconf_t = s_xdemitconf;
pub type xdl_emit_hunk_consume_func_t = Option<
    unsafe extern "C" fn(
        ::core::ffi::c_int,
        ::core::ffi::c_int,
        ::core::ffi::c_int,
        ::core::ffi::c_int,
        *mut ::core::ffi::c_void,
    ) -> ::core::ffi::c_int,
>;
pub type xpparam_t = s_xpparam;
