// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct WinExtmark {
    pub ns_id: NS,
    pub mark_id: uint64_t,
    pub win_row: ::core::ffi::c_int,
    pub win_col: ::core::ffi::c_int,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct spellvars_T {
    pub spv_has_spell: bool,
    pub spv_unchanged: bool,
    pub spv_checked_col: ::core::ffi::c_int,
    pub spv_checked_lnum: linenr_T,
    pub spv_cap_col: ::core::ffi::c_int,
    pub spv_capcol_lnum: linenr_T,
}
