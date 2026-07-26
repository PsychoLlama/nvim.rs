#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SearchOffset {
    pub dir: ::core::ffi::c_char,
    pub line: bool,
    pub end: bool,
    pub off: int64_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SearchPattern {
    pub pat: *mut ::core::ffi::c_char,
    pub patlen: size_t,
    pub magic: bool,
    pub no_scs: bool,
    pub timestamp: Timestamp,
    pub off: SearchOffset,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct searchit_arg_T {
    pub sa_stop_lnum: linenr_T,
    pub sa_tm: *mut proftime_T,
    pub sa_timed_out: ::core::ffi::c_int,
    pub sa_wrapped: ::core::ffi::c_int,
}
