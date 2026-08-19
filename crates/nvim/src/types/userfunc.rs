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

pub type ArgvFunc = Option<
    unsafe fn(
        ::core::ffi::c_int,
        *mut typval_T,
        ::core::ffi::c_int,
        *mut ufunc_T,
    ) -> ::core::ffi::c_int,
>;
#[derive(Copy, Clone)]
pub struct funccal_entry {
    pub top_funccal: *mut ::core::ffi::c_void,
    pub next: *mut funccal_entry_T,
}
pub type funccal_entry_T = funccal_entry;
#[derive(Copy, Clone)]
pub struct funcdict_T {
    pub fd_dict: *mut dict_T,
    pub fd_newkey: *mut ::core::ffi::c_char,
    pub fd_di: *mut dictitem_T,
}
#[derive(Copy, Clone)]
pub struct funcexe_T {
    pub fe_argv_func: ArgvFunc,
    pub fe_firstline: linenr_T,
    pub fe_lastline: linenr_T,
    pub fe_doesrange: *mut bool,
    pub fe_evaluate: bool,
    pub fe_partial: *mut partial_T,
    pub fe_selfdict: *mut dict_T,
    pub fe_basetv: *mut typval_T,
    pub fe_found_var: bool,
}
