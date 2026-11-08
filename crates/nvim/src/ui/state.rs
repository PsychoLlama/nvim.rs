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

use crate::event_queue::ChildQueue;
use crate::global_cell::GlobalCell;
use crate::msgpack_rpc::packer::PackerBuffer;
use crate::os::libc::{size_t, uint32_t, uint64_t};
use crate::types::{ApiString, Array, Integer};
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

pub(crate) type LineFlags = ::core::ffi::c_int;
/// What a `grid_line` event says about the line it carries.
pub(crate) const kLineFlagWrap: LineFlags = 1;
pub(crate) const kLineFlagInvalid: LineFlags = 2;
/// One attached UI: what it negotiated, and its outgoing batch.
///
/// Owned by the attach table in [`crate::ui`] and reached by its channel id;
/// nothing keeps its address.
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
#[derive(Copy, Clone)]
pub struct UIClientHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: Option<fn(Array)>,
}
pub(crate) type UIExtension = ::core::ffi::c_uint;
/// The `ext_*` UI capabilities, indexing `ui_ext` and the `ui_options`
/// dict. `kUIExtCount` is the number of them, not one of them.
pub(crate) const kUICmdline: UIExtension = 0;
pub(crate) const kUIPopupmenu: UIExtension = 1;
pub(crate) const kUITabline: UIExtension = 2;
pub(crate) const kUIWildmenu: UIExtension = 3;
pub(crate) const kUIMessages: UIExtension = 4;
pub(crate) const kUILinegrid: UIExtension = 5;
pub(crate) const kUIMultigrid: UIExtension = 6;
pub(crate) const kUIHlState: UIExtension = 7;
pub(crate) const kUITermColors: UIExtension = 8;
pub(crate) const kUIFloatDebug: UIExtension = 9;
pub(crate) const kUIExtCount: UIExtension = 10;
