// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

pub type HistoryType = ::core::ffi::c_int;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct histentry_T {
    pub hisnum: ::core::ffi::c_int,
    pub hisstr: *mut ::core::ffi::c_char,
    pub hisstrlen: size_t,
    pub timestamp: Timestamp,
    pub additional_data: *mut AdditionalData,
}
