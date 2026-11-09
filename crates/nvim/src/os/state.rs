//! What the process has learned about its environment.
//!
//! `$VIM` and `$VIMRUNTIME` are derived the first time something needs
//! them and the two `didset_*` flags record that (so a later `:let $VIM`
//! is not overwritten); `globaldir` is the directory a window-local `:lcd`
//! must return to, with `LAST_CHDIR_REASON` naming who changed it for
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

use crate::global_cell::GlobalCell;
use core::ffi::c_char;

pub(crate) static DIDSET_VIM: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static DIDSET_VIMRUNTIME: GlobalCell<bool> = GlobalCell::new(false);
/// The directory to go back to when no window or tab page has one of its
/// own: saved when the first local directory is entered.
pub(crate) static GLOBALDIR: GlobalCell<Option<crate::memory::XString>> = GlobalCell::new(None);
pub(crate) static LAST_CHDIR_REASON: GlobalCell<Option<&'static ::core::ffi::CStr>> =
    GlobalCell::new(None);
/// The Windows release `windowsversion()` reports. Nothing here writes it,
/// so it stays the empty string a non-Windows build always answered.
pub(crate) static WINDOWS_VERSION: [c_char; 20] = [0 as c_char; 20];
