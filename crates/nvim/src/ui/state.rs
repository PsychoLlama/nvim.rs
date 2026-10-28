//! The shape of the screen, and what is attached to it.
//!
//! `Rows` and `Columns` are the size every layout computation starts from —
//! the *editor's* idea of it, which `ui_refresh` reconciles with what the
//! attached UIs report. The rest is what the UI layer keeps between calls:
//! which UI is being served (`current_ui`), how many colours the terminal
//! claims (`t_colors`), the `ext_*` capability names in the order the
//! protocol lists them, and the queues and namespaces the `ui_*` events run
//! through.
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

use crate::event::multiqueue::ChildQueue;
use crate::global_cell::GlobalCell;
use crate::types::{uint32_t, uint64_t};
use core::ffi::{CStr, c_int};

pub(crate) static ui_ext_names: [&CStr; 10] = [
    c"ext_cmdline",
    c"ext_popupmenu",
    c"ext_tabline",
    c"ext_wildmenu",
    c"ext_messages",
    c"ext_linegrid",
    c"ext_multigrid",
    c"ext_hlstate",
    c"ext_termcolors",
    c"_debug_float",
];
pub(crate) static Rows: GlobalCell<c_int> = GlobalCell::new(24 as c_int);
pub(crate) static Columns: GlobalCell<c_int> = GlobalCell::new(80 as c_int);
pub(crate) static called_vim_beep: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static current_ui: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub(crate) static t_colors: GlobalCell<c_int> = GlobalCell::new(256 as c_int);
pub(crate) static ui_event_ns_id: GlobalCell<uint32_t> = GlobalCell::new(0 as uint32_t);
pub(crate) static resize_events: GlobalCell<Option<ChildQueue>> = GlobalCell::new(None);
pub(crate) static ui_refresh_cmdheight: GlobalCell<bool> = GlobalCell::new(true);
