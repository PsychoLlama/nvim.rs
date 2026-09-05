//! What the process has learned about its environment.
//!
//! `$VIM` and `$VIMRUNTIME` are derived the first time something needs
//! them and the two `didset_*` flags record that (so a later `:let $VIM`
//! is not overwritten); `globaldir` is the directory a window-local `:lcd`
//! must return to, with `last_chdir_reason` naming who changed it for
//! `DirChanged`.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]

use crate::global_cell::GlobalCell;
use crate::types::{FileComparison, XDGVarType};
use core::ffi::c_char;

pub(crate) const kXDGConfigDirs: XDGVarType = 5;
pub(crate) const kEqualFiles: FileComparison = 1;
pub(crate) static didset_vim: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static didset_vimruntime: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static globaldir: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
pub(crate) static last_chdir_reason: GlobalCell<*mut c_char> =
    GlobalCell::new(::core::ptr::null_mut::<c_char>());
/// The Windows release `windowsversion()` reports. Nothing here writes it,
/// so it stays the empty string a non-Windows build always answered.
pub(crate) static windowsVersion: [c_char; 20] = [0 as c_char; 20];
