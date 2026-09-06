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
use crate::winlayer::{TabId, WinId};

pub struct SwitchWin {
    /// The window and tab page to go back to. Handles: `switch_win` runs
    /// user code between the save and the restore, and that code can close
    /// either of them.
    pub(crate) sw_curwin: Option<WinId>,
    pub(crate) sw_curtab: Option<TabId>,
    pub sw_same_win: bool,
    pub sw_visual_active: bool,
}
pub struct WinExecute {
    /// The window the command ran in, as a handle: it may be gone by the
    /// time `win_execute_after` looks.
    pub(crate) wp: Option<WinId>,
    pub curpos: Pos,
    pub cwd: [::core::ffi::c_char; 4096],
    pub cwd_status: Result<(), Failed>,
    pub apply_acd: bool,
    pub save_sfname: *mut ::core::ffi::c_char,
    pub switchwin: SwitchWin,
}

impl Default for SwitchWin {
    /// The zeroed state a caller declares before handing it to `switch_win`,
    /// which fills every field. Nothing reads one of these before that.
    fn default() -> Self {
        SwitchWin {
            sw_curwin: None,
            sw_curtab: None,
            sw_same_win: false,
            sw_visual_active: false,
        }
    }
}

impl Default for WinExecute {
    /// The zeroed state a caller declares before handing it to
    /// `win_execute_before`, which fills what it needs and leaves the rest --
    /// `cwd` in particular is only written when 'autochdir' is on.
    fn default() -> Self {
        WinExecute {
            wp: None,
            curpos: Pos {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            cwd: [0; 4096],
            cwd_status: Err(Failed),
            apply_acd: false,
            save_sfname: ::core::ptr::null_mut(),
            switchwin: SwitchWin::default(),
        }
    }
}
