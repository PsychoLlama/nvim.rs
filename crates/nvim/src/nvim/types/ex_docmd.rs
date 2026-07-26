#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct save_state_T {
    pub save_msg_scroll: ::core::ffi::c_int,
    pub save_restart_edit: ::core::ffi::c_int,
    pub save_msg_didout: bool,
    pub save_State: ::core::ffi::c_int,
    pub save_finish_op: bool,
    pub save_opcount: ::core::ffi::c_int,
    pub save_reg_executing: ::core::ffi::c_int,
    pub save_pending_end_reg_executing: bool,
    pub tabuf: tasave_T,
}
