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
use crate::types::Failed;

pub struct switchwin_T {
    pub sw_curwin: *mut win_T,
    pub sw_curtab: *mut tabpage_T,
    pub sw_same_win: bool,
    pub sw_visual_active: bool,
}
pub struct win_execute_T {
    pub wp: *mut win_T,
    pub curpos: pos_T,
    pub cwd: [::core::ffi::c_char; 4096],
    pub cwd_status: Result<(), Failed>,
    pub apply_acd: bool,
    pub save_sfname: *mut ::core::ffi::c_char,
    pub switchwin: switchwin_T,
}

impl Default for switchwin_T {
    /// The zeroed state a caller declares before handing it to `switch_win`,
    /// which fills every field. Nothing reads one of these before that.
    fn default() -> Self {
        switchwin_T {
            sw_curwin: ::core::ptr::null_mut(),
            sw_curtab: ::core::ptr::null_mut(),
            sw_same_win: false,
            sw_visual_active: false,
        }
    }
}

impl Default for win_execute_T {
    /// The zeroed state a caller declares before handing it to
    /// `win_execute_before`, which fills what it needs and leaves the rest --
    /// `cwd` in particular is only written when 'autochdir' is on.
    fn default() -> Self {
        win_execute_T {
            wp: ::core::ptr::null_mut(),
            curpos: pos_T {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cwd: [0; 4096],
            cwd_status: Err(Failed),
            apply_acd: false,
            save_sfname: ::core::ptr::null_mut(),
            switchwin: switchwin_T::default(),
        }
    }
}
