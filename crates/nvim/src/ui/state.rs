//! The shape of the screen, and what is attached to it.
//!
//! `ROWS` and `COLUMNS` are the size every layout computation starts from —
//! the *editor's* idea of it, which `ui_refresh` reconciles with what the
//! attached UIs report. The rest is what the UI layer keeps between calls:
//! which UI is being served (`CURRENT_UI`), how many colours the terminal
//! claims (`T_COLORS`), the `ext_*` capability names in the order the
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

use crate::event_queue::ChildQueue;
use crate::global_cell::GlobalCell;
use crate::msgpack_rpc::packer::PackerBuffer;
use crate::os::libc::{size_t, uint32_t, uint64_t};
use crate::types::{ApiString, Array, Integer};
use core::ffi::{CStr, c_int};

pub(crate) static UI_EXT_NAMES: [&CStr; 10] = [
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
pub(crate) static ROWS: GlobalCell<c_int> = GlobalCell::new(24 as c_int);
pub(crate) static COLUMNS: GlobalCell<c_int> = GlobalCell::new(80 as c_int);
pub(crate) static CALLED_VIM_BEEP: GlobalCell<bool> = GlobalCell::new(false);
pub(crate) static CURRENT_UI: GlobalCell<uint64_t> = GlobalCell::new(0 as uint64_t);
pub(crate) static T_COLORS: GlobalCell<c_int> = GlobalCell::new(256 as c_int);
pub(crate) static UI_EVENT_NS_ID: GlobalCell<uint32_t> = GlobalCell::new(0 as uint32_t);
pub(crate) static RESIZE_EVENTS: GlobalCell<Option<ChildQueue>> = GlobalCell::new(None);
pub(crate) static UI_REFRESH_CMDHEIGHT: GlobalCell<bool> = GlobalCell::new(true);

pub(crate) type LineFlags = ::core::ffi::c_int;
/// What a `grid_line` event says about the line it carries.
pub(crate) const LINE_FLAG_WRAP: LineFlags = 1;
pub(crate) const LINE_FLAG_INVALID: LineFlags = 2;
/// One attached UI: what it negotiated, and its outgoing batch.
///
/// Owned by the attach table in [`crate::ui`] and reached by its channel id;
/// nothing keeps its address.
#[derive(Debug)]
pub struct RemoteUI {
    pub rgb: bool,
    pub override_0: bool,
    pub composed: bool,
    pub ui_ext: [bool; 10],
    pub width: ::core::ffi::c_int,
    pub height: ::core::ffi::c_int,
    pub pum_nlines: ::core::ffi::c_int,
    pub pum_pos: bool,
    pub pum_row: ::core::ffi::c_double,
    pub pum_col: ::core::ffi::c_double,
    pub pum_height: ::core::ffi::c_double,
    pub pum_width: ::core::ffi::c_double,
    /// The `term_name` option, as `nvim_list_uis` reports it.
    pub term_name: ApiString,
    pub term_colors: ::core::ffi::c_int,
    pub stdin_tty: bool,
    pub stdout_tty: bool,
    pub channel_id: uint64_t,
    pub packer: PackerBuffer,
    /// The event the batch is appending calls to, merged while the name
    /// stays the same.
    pub cur_event: Option<&'static ::core::ffi::CStr>,
    /// Where the batch's event count is to be patched in, once known.
    pub nevents_pos: Option<usize>,
    /// Where the current event's call count is to be patched in.
    pub ncalls_pos: Option<usize>,
    pub nevents: uint32_t,
    pub ncalls: uint32_t,
    pub flushed_events: bool,
    pub ncells_pending: size_t,
    pub hl_id: ::core::ffi::c_int,
    pub cursor_row: Integer,
    pub cursor_col: Integer,
    pub client_row: Integer,
    pub client_col: Integer,
    pub wildmenu_active: bool,
}
/// One row of the UI-client redraw dispatch table.
///
/// `Copy`: a static name and a function pointer.
#[derive(Copy, Clone, Debug)]
pub struct UIClientHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: Option<fn(Array)>,
}
pub(crate) type UIExtension = ::core::ffi::c_uint;
/// The `ext_*` UI capabilities, indexing `ui_ext` and the `ui_options`
/// dict. `UI_EXT_COUNT` is the number of them, not one of them.
pub(crate) const UI_CMDLINE: UIExtension = 0;
pub(crate) const UI_POPUPMENU: UIExtension = 1;
pub(crate) const UI_TABLINE: UIExtension = 2;
pub(crate) const UI_WILDMENU: UIExtension = 3;
pub(crate) const UI_MESSAGES: UIExtension = 4;
pub(crate) const UI_LINEGRID: UIExtension = 5;
pub(crate) const UI_MULTIGRID: UIExtension = 6;
pub(crate) const UI_HL_STATE: UIExtension = 7;
pub(crate) const UI_TERM_COLORS: UIExtension = 8;
pub(crate) const UI_FLOAT_DEBUG: UIExtension = 9;
pub(crate) const UI_EXT_COUNT: UIExtension = 10;
