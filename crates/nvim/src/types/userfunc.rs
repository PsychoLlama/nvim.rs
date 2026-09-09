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

pub type ArgvFunc = Option<unsafe fn(&[TypVal], usize, *mut UserFunc) -> usize>;
pub struct FuncCallEntry {
    pub top_funccal: *mut ::core::ffi::c_void,
    pub next: *mut FuncCallEntry,
}
pub struct FuncDict {
    pub fd_dict: *mut Dict,
    pub fd_newkey: *mut ::core::ffi::c_char,
    pub fd_di: *mut DictItem,
}
#[derive(Copy, Clone)]
pub struct FuncExe {
    pub fe_argv_func: ArgvFunc,
    pub fe_firstline: LineNr,
    pub fe_lastline: LineNr,
    pub fe_doesrange: *mut bool,
    pub fe_evaluate: bool,
    pub fe_partial: *mut Partial,
    pub fe_selfdict: *mut Dict,
    pub fe_basetv: *mut TypVal,
    pub fe_found_var: bool,
}
