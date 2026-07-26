#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TPVAR {
    pub num: ::core::ffi::c_long,
    pub string: *mut ::core::ffi::c_char,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminfoEntry {
    pub bce: bool,
    pub has_Tc_or_RGB: bool,
    pub Su: bool,
    pub max_colors: ::core::ffi::c_int,
    pub lines: ::core::ffi::c_int,
    pub columns: ::core::ffi::c_int,
    pub defs: [*const ::core::ffi::c_char; 49],
    pub keys: [[*const ::core::ffi::c_char; 2]; 16],
    pub f_keys: [*const ::core::ffi::c_char; 63],
}
