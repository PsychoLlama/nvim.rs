#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct EvalFuncDef {
    pub name: *mut ::core::ffi::c_char,
    pub min_argc: uint8_t,
    pub max_argc: uint8_t,
    pub base_arg: uint8_t,
    pub fast: bool,
    pub func: VimLFunc,
    pub data: EvalFuncData,
}
pub type VimLFunc = Option<unsafe extern "C" fn(*mut typval_T, *mut typval_T, EvalFuncData) -> ()>;
