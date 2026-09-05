//! The attribute numbers the drawing code paints with.
//!
//! `highlight_attr` is the resolved attribute for each `HLF_*` slot, which
//! every `win_line` decision indexes; `highlight_user` and
//! `highlight_stlnc` are the same for the nine `User1`..`User9` groups and
//! their not-current-window variants. The `normal_*`/`cterm_normal_*` pair
//! is what `Normal` resolved to, which the UI needs separately because it is
//! the screen's background rather than a run's attribute.
//!
//! The `ns_hl_*` cells say which highlight namespace is in force —
//! globally, for the window being drawn, and for the fast path a decoration
//! provider runs on — and `need_highlight_changed` is the "recompute all of
//! the above" flag `:highlight` and `:colorscheme` raise.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::global_cell::GlobalCell;
use crate::types::{NS, RgbValue};
use core::ffi::c_int;

pub(crate) static include_none: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static include_default: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static include_link: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static need_highlight_changed: GlobalCell<bool> = GlobalCell::new(true);
pub(crate) static highlight_attr: GlobalCell<[c_int; 76]> = GlobalCell::new([0; 76]);
pub(crate) static highlight_attr_last: GlobalCell<[c_int; 76]> = GlobalCell::new([0; 76]);
pub(crate) static highlight_user: GlobalCell<[c_int; 9]> = GlobalCell::new([0; 9]);
pub(crate) static highlight_stlnc: GlobalCell<[c_int; 9]> = GlobalCell::new([0; 9]);
pub(crate) static cterm_normal_fg_color: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static cterm_normal_bg_color: GlobalCell<c_int> = GlobalCell::new(0 as c_int);
pub(crate) static normal_fg: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub(crate) static normal_bg: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub(crate) static normal_sp: GlobalCell<RgbValue> = GlobalCell::new(-1 as RgbValue);
pub(crate) static ns_hl_global: GlobalCell<NS> = GlobalCell::new(0 as NS);
pub(crate) static ns_hl_win: GlobalCell<NS> = GlobalCell::new(-1 as NS);
pub(crate) static ns_hl_fast: GlobalCell<NS> = GlobalCell::new(-1 as NS);
pub(crate) static ns_hl_active: GlobalCell<NS> = GlobalCell::new(0 as NS);
pub(crate) static hl_attr_active: GlobalCell<*mut c_int> =
    GlobalCell::new((highlight_attr.as_raw() as *const _) as *mut c_int);
