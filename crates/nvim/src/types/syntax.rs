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
use crate::syntax::SynFlags;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct bufstate_T {
    pub bs_idx: ::core::ffi::c_int,
    pub bs_flags: SynFlags,
    pub bs_seqnr: ::core::ffi::c_int,
    pub bs_cchar: ::core::ffi::c_int,
    pub bs_extmatch: *mut reg_extmatch_T,
}
#[derive(Copy, Clone)]
pub struct syn_state {
    pub sst_next: *mut synstate_T,
    pub sst_lnum: linenr_T,
    pub sst_union: syn_state_sst_union,
    pub sst_next_flags: SynFlags,
    pub sst_stacksize: ::core::ffi::c_int,
    pub sst_next_list: *mut int16_t,
    pub sst_tick: disptick_T,
    pub sst_change_lnum: linenr_T,
}
/// A cached entry's state stack: either inline in the entry, or on the heap
/// when there are more than `SST_FIX_STATES` items.
///
/// Discriminated by `syn_state::sst_stacksize`, which is also the *length* of
/// both arms -- so the heap arm needs no length of its own, and upstream's
/// growarray here carried one that was always a copy of it. `syntax::stack`'s
/// `entry_states` is the one place the discrimination is written down.
#[derive(Copy, Clone)]
#[repr(C)]
pub union syn_state_sst_union {
    pub sst_stack: [bufstate_T; 7],
    /// A `Box<[bufstate_T]>` of `sst_stacksize` items taken apart: a union
    /// field may not have a destructor, so `clear_syn_state` puts it back
    /// together to release it.
    pub sst_heap: *mut bufstate_T,
}
