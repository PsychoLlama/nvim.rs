// Canonical type definitions extracted by tools/unify (phase 5a).
// One definition per logical type; every module re-exports from here.
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct switchwin_T {
    pub sw_curwin: *mut win_T,
    pub sw_curtab: *mut tabpage_T,
    pub sw_same_win: bool,
    pub sw_visual_active: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct win_execute_T {
    pub wp: *mut win_T,
    pub curpos: pos_T,
    pub cwd: [::core::ffi::c_char; 4096],
    pub cwd_status: ::core::ffi::c_int,
    pub apply_acd: bool,
    pub save_sfname: *mut ::core::ffi::c_char,
    pub switchwin: switchwin_T,
}
