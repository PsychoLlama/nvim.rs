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

#[derive(Copy, Clone)]
pub struct WinExtmark {
    pub ns_id: NS,
    pub mark_id: uint64_t,
    pub win_row: ::core::ffi::c_int,
    pub win_col: ::core::ffi::c_int,
}
#[derive(Copy, Clone, Default)]
pub struct spellvars_T {
    pub spv_has_spell: bool,
    pub spv_unchanged: bool,
    pub spv_checked_col: ::core::ffi::c_int,
    pub spv_checked_lnum: linenr_T,
    pub spv_cap_col: ::core::ffi::c_int,
    pub spv_capcol_lnum: linenr_T,
}
