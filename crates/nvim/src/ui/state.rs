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

use crate::global_cell::{ConstTable, GlobalCell};
use crate::types::{MultiQueue, uint32_t, uint64_t};
use core::ffi::{c_char, c_int};

pub(crate) static ui_ext_names: ConstTable<[*const c_char; 10]> = ConstTable::new([
    c"ext_cmdline".as_ptr(),
    c"ext_popupmenu".as_ptr(),
    c"ext_tabline".as_ptr(),
    c"ext_wildmenu".as_ptr(),
    c"ext_messages".as_ptr(),
    c"ext_linegrid".as_ptr(),
    c"ext_multigrid".as_ptr(),
    c"ext_hlstate".as_ptr(),
    c"ext_termcolors".as_ptr(),
    c"_debug_float".as_ptr(),
]);
pub(crate) static Rows: GlobalCell<c_int> = GlobalCell::new(24 as c_int);
pub(crate) static Columns: GlobalCell<c_int> = GlobalCell::new(80 as c_int);
pub(crate) static called_vim_beep: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static current_ui: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub(crate) static t_colors: GlobalCell<c_int> = GlobalCell::new(256 as c_int);
pub(crate) static ui_event_ns_id: GlobalCell<uint32_t> = GlobalCell::new(0 as uint32_t);
pub(crate) static resize_events: GlobalCell<*mut MultiQueue> =
    GlobalCell::new(::core::ptr::null_mut::<MultiQueue>());
pub(crate) static ui_refresh_cmdheight: GlobalCell<bool> = GlobalCell::new(true);
