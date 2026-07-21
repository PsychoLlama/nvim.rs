// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_mmbuffer {
    pub ptr: *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_mmfile {
    pub ptr: *mut ::core::ffi::c_char,
    pub size: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
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
#[derive(Copy, Clone)]
#[repr(C)]
pub struct s_xdemitconf {
    pub ctxlen: ::core::ffi::c_long,
    pub interhunkctxlen: ::core::ffi::c_long,
    pub flags: ::core::ffi::c_ulong,
    pub find_func: find_func_t,
    pub find_func_priv: *mut ::core::ffi::c_void,
    pub hunk_func: xdl_emit_hunk_consume_func_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
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
