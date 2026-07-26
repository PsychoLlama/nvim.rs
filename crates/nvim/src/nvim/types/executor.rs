#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

pub type LuaRetMode = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct nlua_ref_state_t {
    pub nil_ref: LuaRef,
    pub empty_dict_ref: LuaRef,
    pub ref_count: ::core::ffi::c_int,
}
