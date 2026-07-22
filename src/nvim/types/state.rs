// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.

pub type VimState = vim_state;
pub type state_check_callback = Option<unsafe extern "C" fn(*mut VimState) -> ::core::ffi::c_int>;
pub type state_execute_callback =
    Option<unsafe extern "C" fn(*mut VimState, ::core::ffi::c_int) -> ::core::ffi::c_int>;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct vim_state {
    pub check: state_check_callback,
    pub execute: state_execute_callback,
}
