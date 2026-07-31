#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

pub type LineFlags = ::core::ffi::c_int;
/// What a `grid_line` event says about the line it carries.
pub const kLineFlagWrap: LineFlags = 1;
pub const kLineFlagInvalid: LineFlags = 2;
#[derive(Copy, Clone)]
#[repr(C)]
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
    pub term_name: *mut ::core::ffi::c_char,
    pub term_colors: ::core::ffi::c_int,
    pub stdin_tty: bool,
    pub stdout_tty: bool,
    pub channel_id: uint64_t,
    pub packer: PackerBuffer,
    pub cur_event: *const ::core::ffi::c_char,
    pub nevents_pos: *mut ::core::ffi::c_char,
    pub ncalls_pos: *mut ::core::ffi::c_char,
    pub nevents: uint32_t,
    pub ncalls: uint32_t,
    pub flushed_events: bool,
    pub incomplete_event: bool,
    pub ncells_pending: size_t,
    pub hl_id: ::core::ffi::c_int,
    pub cursor_row: Integer,
    pub cursor_col: Integer,
    pub client_row: Integer,
    pub client_col: Integer,
    pub wildmenu_active: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct UIClientHandler {
    pub name: *const ::core::ffi::c_char,
    pub fn_0: Option<unsafe extern "C" fn(Array) -> ()>,
}
pub type UIExtension = ::core::ffi::c_uint;
/// The `ext_*` UI capabilities, indexing `ui_ext` and the `ui_options`
/// dict. `kUIExtCount` is the number of them, not one of them.
pub const kUICmdline: UIExtension = 0;
pub const kUIPopupmenu: UIExtension = 1;
pub const kUITabline: UIExtension = 2;
pub const kUIWildmenu: UIExtension = 3;
pub const kUIMessages: UIExtension = 4;
pub const kUILinegrid: UIExtension = 5;
pub const kUIMultigrid: UIExtension = 6;
pub const kUIHlState: UIExtension = 7;
pub const kUITermColors: UIExtension = 8;
pub const kUIFloatDebug: UIExtension = 9;
pub const kUIExtCount: UIExtension = 10;
