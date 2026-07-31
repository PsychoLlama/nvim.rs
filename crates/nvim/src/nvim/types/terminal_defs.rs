#![forbid(unsafe_code)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// One `:terminal` buffer's emulator state.
///
/// Owned by the buffer it draws into: `terminal_alloc` leaks a `Box` and
/// `terminal_destroy` reclaims it. Not `Copy` — there is exactly one of
/// these per terminal buffer, reached through `buf_T::terminal`, and
/// duplicating it would duplicate the allocations it owns.
///
/// Still `repr(C)`: `buf_T` is `repr(C)` and holds a `*mut Terminal`, so
/// the FFI-safety lint follows the pointer into this definition.
#[repr(C)]
pub struct terminal {
    pub opts: TerminalOptions,
    pub vt: *mut VTerm,
    pub vts: *mut VTermScreen,
    pub textbuf: [::core::ffi::c_char; 8191],
    pub sb_buffer: *mut *mut ScrollbackLine,
    pub sb_current: size_t,
    pub sb_size: size_t,
    pub sb_pending: ::core::ffi::c_int,
    pub sb_deleted: size_t,
    pub old_sb_deleted: size_t,
    pub old_height: ::core::ffi::c_int,
    pub title: *mut ::core::ffi::c_char,
    pub title_len: size_t,
    pub title_size: size_t,
    pub buf_handle: handle_T,
    pub in_altscreen: bool,
    pub suspended: bool,
    pub closed: bool,
    pub destroy: bool,
    pub forward_mouse: bool,
    pub invalid_start: ::core::ffi::c_int,
    pub invalid_end: ::core::ffi::c_int,
    pub cursor: TerminalCursor,
    pub pending: TerminalPending,
    pub streamed_paste: bool,
    pub theme_updates: bool,
    pub synchronized_output: bool,
    pub sync_flush_pending: bool,
    pub color_set: [bool; 16],
    pub selection_buffer: *mut ::core::ffi::c_char,
    pub selection: StringBuilder,
    pub termrequest_buffer: StringBuilder,
    pub termrequest_terminator: VTermTerminator,
    pub refcount: size_t,
}
impl terminal {
    /// A terminal with everything at rest, for `terminal_alloc` to fill in.
    ///
    /// This stands in for the `xcalloc` the transpiled code used: every
    /// field starts as the all-zeroes value C would have produced, so the
    /// initialisation that follows is the same one the C did.
    pub fn new(opts: TerminalOptions, buf_handle: handle_T) -> Self {
        Self {
            opts,
            buf_handle,
            vt: ::core::ptr::null_mut(),
            vts: ::core::ptr::null_mut(),
            textbuf: [0; 8191],
            sb_buffer: ::core::ptr::null_mut(),
            sb_current: 0,
            sb_size: 0,
            sb_pending: 0,
            sb_deleted: 0,
            old_sb_deleted: 0,
            old_height: 0,
            title: ::core::ptr::null_mut(),
            title_len: 0,
            title_size: 0,
            in_altscreen: false,
            suspended: false,
            closed: false,
            destroy: false,
            forward_mouse: false,
            invalid_start: 0,
            invalid_end: 0,
            cursor: TerminalCursor {
                row: 0,
                col: 0,
                shape: 0,
                visible: false,
                blink: false,
            },
            pending: TerminalPending {
                resize: false,
                cursor: false,
                send: ::core::ptr::null_mut(),
                events: ::core::ptr::null_mut(),
            },
            streamed_paste: false,
            theme_updates: false,
            synchronized_output: false,
            sync_flush_pending: false,
            color_set: [false; 16],
            selection_buffer: ::core::ptr::null_mut(),
            selection: StringBuilder {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut(),
            },
            termrequest_buffer: StringBuilder {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut(),
            },
            termrequest_terminator: 0,
            refcount: 0,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ScrollbackLine {
    pub cols: size_t,
    pub cells: [VTermScreenCell; 0],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminalPending {
    pub resize: bool,
    pub cursor: bool,
    pub send: *mut StringBuilder,
    pub events: *mut MultiQueue,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminalCursor {
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    pub shape: ::core::ffi::c_int,
    pub visible: bool,
    pub blink: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TerminalOptions {
    pub data: *mut ::core::ffi::c_void,
    pub width: uint16_t,
    pub height: uint16_t,
    pub read_pause_cb: terminal_read_pause_cb,
    pub write_cb: terminal_write_cb,
    pub resize_cb: terminal_resize_cb,
    pub resume_cb: terminal_resume_cb,
    pub close_cb: terminal_close_cb,
    pub force_crlf: bool,
}
pub type terminal_close_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type terminal_read_pause_cb =
    Option<unsafe extern "C" fn(bool, *mut ::core::ffi::c_void) -> ()>;
pub type terminal_resize_cb =
    Option<unsafe extern "C" fn(uint16_t, uint16_t, *mut ::core::ffi::c_void) -> ()>;
pub type terminal_resume_cb = Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> ()>;
pub type terminal_write_cb = Option<
    unsafe extern "C" fn(*const ::core::ffi::c_char, size_t, *mut ::core::ffi::c_void) -> (),
>;
