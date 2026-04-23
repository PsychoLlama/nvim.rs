//! Terminal emulator utilities for Neovim
//!
//! This crate provides Rust implementations for terminal-related functions,
//! primarily working with the libvterm-based terminal emulator.
//!
//! Re-exports vterm types for terminal emulation.

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_int, c_long, c_uint};
use std::os::raw::c_void;

use nvim_buffer::buf_struct::BufStruct;
use nvim_vterm::{VTermScreenCellAttrs, VTermStringFragment, VTermValue};
use nvim_window::win_struct::WinStruct;

// =============================================================================
// C Global Variable Statics (Phase 1: eliminate C accessor functions)
// =============================================================================

extern "C" {
    /// `exiting` global from globals.h (bool).
    #[link_name = "exiting"]
    static c_exiting: bool;
    /// `entered_free_all_mem` from `memory.h` -- accessed via the C wrapper in `buffer_shim.c`.
    #[link_name = "nvim_get_entered_free_all_mem"]
    fn c_entered_free_all_mem_fn() -> c_int;
    /// `must_redraw` global from `globals.h` (int).
    #[link_name = "must_redraw"]
    static c_must_redraw: c_int;
    /// `clear_cmdline` global from `globals.h` (bool).
    #[link_name = "clear_cmdline"]
    static c_clear_cmdline: bool;
    /// `redraw_cmdline` global from `globals.h` (bool).
    #[link_name = "redraw_cmdline"]
    static c_redraw_cmdline: bool;
    /// `redraw_mode` global from `globals.h` (bool).
    #[link_name = "redraw_mode"]
    static c_redraw_mode: bool;
    /// `got_int` global from `globals.h` (bool).
    #[link_name = "got_int"]
    static mut c_got_int: bool;
    /// `vgetc_mod_mask` global from `globals.h` (int).
    #[link_name = "vgetc_mod_mask"]
    static c_vgetc_mod_mask: c_int;
    /// `KeyTyped` global from `globals.h` (bool).
    #[link_name = "KeyTyped"]
    static c_key_typed: bool;
    /// `tpf_flags` global from `option_vars.h` (unsigned int).
    #[link_name = "tpf_flags"]
    static c_tpf_flags: c_uint;
    /// `p_bg` option from `option_vars.h` (`*const char`).
    #[link_name = "p_bg"]
    static c_p_bg: *const i8;
    /// `curwin` global from `globals.h` (`*mut win_T`, opaque).
    #[link_name = "curwin"]
    static c_curwin: *mut c_void;
    /// `State` global from `globals.h` (int, current editor mode bitmask).
    #[link_name = "State"]
    static c_nvim_state: c_int;
}

/// `refresh_pending` -- owned by Rust (moved from C file-static).
///
/// Previously a file-static `bool` in `terminal_shim.c`; now a Rust `static mut`
/// exported so `refresh_timer_cb` (still in C) can read it directly.
#[no_mangle]
pub static mut refresh_pending: bool = false;

/// Access `WinStruct` fields from a raw `win_T` pointer.
#[allow(clippy::missing_const_for_fn)]
#[inline]
unsafe fn win_ref_raw<'a>(wp: *mut c_void) -> &'a WinStruct {
    nvim_window::win_struct::win_ref(nvim_window::WinHandle::from_ptr(wp))
}
/// Mutable access to `WinStruct` fields from a raw `win_T` pointer.
#[inline]
unsafe fn win_mut_raw<'a>(wp: *mut c_void) -> &'a mut WinStruct {
    nvim_window::win_struct::win_mut(nvim_window::WinHandle::from_ptr(wp))
}

/// Access `BufStruct` fields from a raw `buf_T` pointer.
#[inline]
unsafe fn bref_raw(buf: *mut c_void) -> &'static BufStruct {
    &*(buf.cast::<BufStruct>())
}

// =============================================================================
// Submodules
// =============================================================================

pub mod buffer;
pub mod input;
pub mod mode;
pub mod output;
pub mod pty;
pub mod scrollback;

// Re-export commonly used types from submodules
// Note: Some types (InvalidRegion, TPF_*, TERMINAL_SB_MAX) already exist in lib.rs,
// so we don't re-export those to avoid conflicts.
pub use buffer::{
    get_terminal_buffer_state, is_screen_line, is_scrollback_line, linenr_to_row, row_to_linenr,
    validate_terminal_buffer, BufferValidation, TerminalBufferState,
};
pub use input::{
    is_valid_input_char, is_valid_key_code, InputEvent, InputEventType, InputModifiers,
    InputValidation, TerminalKey, INPUT_MOD_ALT, INPUT_MOD_CTRL, INPUT_MOD_SHIFT,
};
pub use mode::{
    get_terminal_cursor, get_terminal_state, validate_mode_transition, FocusChange, FocusState,
    ModeTransitionResult, TerminalCursor, TerminalMode, TerminalState,
};
pub use output::{
    BellFlags, DamageRegion, OutputEvent, OutputEventType, TitleUpdateResult, BELL_AUDIO,
    BELL_URGENT, BELL_VISUAL, MAX_TITLE_LEN,
};
pub use pty::{
    convert_nvim_modifier, is_pty_ready, KeySendResult, PtyState, SendResult, TermModifiers,
    TermPasteFlags,
};
pub use scrollback::{
    calculate_adjustment, calculate_pop, calculate_push, calculate_scrollback_size,
    calculate_trim_count, get_scrollback_state, is_valid_scrollback_index, ScrollbackAdjustment,
    ScrollbackLine, ScrollbackPop, ScrollbackPush, ScrollbackResult, ScrollbackState,
};

// Re-export vterm types that don't conflict with existing definitions
// The terminal crate already has its own VTermRect, VTermPos, and modifier constants
pub mod vterm {
    pub use nvim_vterm::{
        decode_dec_drawing,
        decode_usascii,
        encode_button,
        encode_key,
        encode_move,
        encode_unichar,
        // Pen colors
        lookup_colour,
        lookup_colour_palette,
        lookup_keycode,
        parse_sgr_param,
        // Parser types
        CsiParserState,
        // Encoding
        Encoding,
        EncodingType,
        // Keyboard encoding
        KeyOutput,
        // Mouse encoding
        MouseOutput,
        // State types (use prefixed names to avoid conflicts)
        MouseProtocol as VTermMouseProtocol,
        MouseState,
        OscParserState,
        ParserState,
        Pen,
        SavedModes,
        // Screen buffer types
        Screen,
        ScreenCell,
        ScreenPen,
        SelectionState,
        TerminalModes,
        Utf8Decoder,
        VTermKey,
        // Mouse flags
        MOUSE_WANT_CLICK,
        MOUSE_WANT_DRAG,
        MOUSE_WANT_MOVE,
        UNICODE_INVALID,
    };
}

// =============================================================================
// Opaque Handles
// =============================================================================

/// Opaque handle to Terminal struct (terminal.c struct terminal)
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalHandle(*mut c_void);

impl TerminalHandle {
    /// Create a handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `Terminal*` or null.
    #[inline]
    pub const unsafe fn from_ptr(ptr: *mut c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    pub const fn as_ptr(self) -> *mut c_void {
        self.0
    }

    /// Check if the handle is null.
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

// =============================================================================
// repr(C) Terminal struct -- matches `struct terminal` in terminal.c exactly.
//
// Layout verified with offsetof() on 2026-03-09:
//   sizeof(struct terminal) == 8472
//   All field offsets match those computed in terminal.c via static_assert.
// =============================================================================

/// `TerminalOptions` from terminal.h (48 bytes on 64-bit).
#[repr(C)]
pub struct CTerminalOptions {
    pub data: *mut c_void,      // offset 0
    pub width: u16,             // offset 8
    pub height: u16,            // offset 10
    _pad0: [u8; 4],             // offset 12, padding to 16
    pub write_cb: *mut c_void,  // offset 16 (fn ptr, opaque)
    pub resize_cb: *mut c_void, // offset 24 (fn ptr, opaque)
    pub close_cb: *mut c_void,  // offset 32 (fn ptr, opaque)
    pub force_crlf: bool,       // offset 40
    _pad1: [u8; 7],             // offset 41, padding to 48
}

/// Cursor sub-struct (16 bytes).
#[repr(C)]
pub struct CTerminalCursor {
    pub row: c_int,    // offset 0
    pub col: c_int,    // offset 4
    pub shape: c_int,  // offset 8
    pub visible: bool, // offset 12
    pub blink: bool,   // offset 13
    _pad: [u8; 2],     // offset 14, padding to 16
}

/// Pending sub-struct (24 bytes).
#[repr(C)]
pub struct CTerminalPending {
    pub resize: bool,        // offset 0
    pub cursor: bool,        // offset 1
    _pad: [u8; 6],           // offset 2, padding to 8
    pub send: *mut c_void,   // offset 8 (StringBuilder*)
    pub events: *mut c_void, // offset 16 (MultiQueue*)
}

/// `StringBuilder = kvec_t(char)` -- 3 pointer-sized fields (24 bytes).
#[repr(C)]
pub struct CStringBuilder {
    pub size: usize,
    pub capacity: usize,
    pub items: *mut i8,
}

// =============================================================================
// CStringBuilder helpers (Phase 3: replaces nvim_term_sb_* C wrappers)
// =============================================================================

/// Cast a raw `*mut c_void` to a `&mut CStringBuilder`.
///
/// # Safety
/// `sb` must be a valid, non-null pointer to a `CStringBuilder`.
#[inline]
unsafe fn sb_mut<'a>(sb: *mut c_void) -> &'a mut CStringBuilder {
    unsafe { &mut *sb.cast::<CStringBuilder>() }
}

/// Cast a raw `*const c_void` to a `&CStringBuilder`.
///
/// # Safety
/// `sb` must be a valid, non-null pointer to a `CStringBuilder`.
#[inline]
unsafe fn sb_ref<'a>(sb: *const c_void) -> &'a CStringBuilder {
    unsafe { &*sb.cast::<CStringBuilder>() }
}

/// Append `len` bytes from `data` to the `StringBuilder`. Grows by doubling if needed.
///
/// Equivalent to `kv_concat_len(*(StringBuilder*)sb, data, len)`.
///
/// # Safety
/// `sb` must be a valid `*mut CStringBuilder`. `data` must point to `len` bytes.
unsafe fn sb_concat_len(sb: *mut c_void, data: *const i8, len: usize) {
    if len == 0 {
        return;
    }
    let s = unsafe { sb_mut(sb) };
    // Grow if needed: new_capacity = max(capacity * 2, size + len, 8)
    if s.size + len > s.capacity {
        let new_cap = (s.capacity * 2).max(s.size + len).max(8);
        s.items = unsafe { nvim_term_xrealloc(s.items.cast::<c_void>(), new_cap).cast::<i8>() };
        s.capacity = new_cap;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(data, s.items.add(s.size), len);
    }
    s.size += len;
}

/// Push a single char onto the `StringBuilder`. Grows by doubling if needed.
///
/// Equivalent to `kv_push(*(StringBuilder*)sb, c)`.
///
/// # Safety
/// `sb` must be a valid `*mut CStringBuilder`.
unsafe fn sb_push_char(sb: *mut c_void, c: i8) {
    let s = unsafe { sb_mut(sb) };
    if s.size == s.capacity {
        let new_cap = (s.capacity * 2).max(8);
        s.items = unsafe { nvim_term_xrealloc(s.items.cast::<c_void>(), new_cap).cast::<i8>() };
        s.capacity = new_cap;
    }
    unsafe { s.items.add(s.size).write(c) };
    s.size += 1;
}

/// Free the items buffer and zero the fields.
///
/// Equivalent to `kv_destroy(*(StringBuilder*)sb)`.
///
/// # Safety
/// `sb` must be a valid `*mut CStringBuilder`.
unsafe fn sb_destroy(sb: *mut c_void) {
    let s = unsafe { sb_mut(sb) };
    unsafe { xfree(s.items.cast::<c_void>()) };
    s.size = 0;
    s.capacity = 0;
    s.items = std::ptr::null_mut();
}

/// Allocate and zero-initialize a heap `CStringBuilder`.
///
/// Equivalent to `nvim_term_sb_alloc_init()`.
///
/// # Safety
/// Caller must eventually free with `sb_destroy` then `xfree`.
unsafe fn sb_alloc_init() -> *mut c_void {
    unsafe { xcalloc(1, std::mem::size_of::<CStringBuilder>()) }
}

// =============================================================================
// ScrollbackLine helpers (Phase 3: replaces nvim_scrollback_line_* C wrappers)
// =============================================================================

/// Size of a single `VTermScreenCell` (matches C `sizeof(VTermScreenCell)`).
const VTERM_SCREEN_CELL_SIZE: usize = std::mem::size_of::<nvim_vterm::VTermScreenCell>();

/// Byte size of a `ScrollbackLine` with `cols` cells.
///
/// Equivalent to `nvim_scrollback_line_size(cols)`.
#[inline]
fn scrollback_line_size(cols: usize) -> usize {
    std::mem::size_of::<usize>() + cols * VTERM_SCREEN_CELL_SIZE
}

/// Read the `cols` field from a `ScrollbackLine*`.
///
/// Equivalent to `nvim_scrollback_line_cols(sbrow)`.
///
/// # Safety
/// `sbrow` must be a valid `ScrollbackLine*`.
#[inline]
unsafe fn scrollback_line_cols(sbrow: *const c_void) -> usize {
    unsafe { *sbrow.cast::<usize>() }
}

/// Get a const pointer to the cells array inside a `ScrollbackLine*`.
///
/// Equivalent to `nvim_scrollback_line_cells(sbrow)`.
///
/// # Safety
/// `sbrow` must be a valid `ScrollbackLine*`.
#[inline]
unsafe fn scrollback_line_cells(sbrow: *const c_void) -> *const c_void {
    unsafe {
        sbrow
            .cast::<u8>()
            .add(std::mem::size_of::<usize>())
            .cast::<c_void>()
    }
}

/// Get a mutable pointer to the cells array inside a `ScrollbackLine*`.
///
/// Equivalent to `nvim_scrollback_line_cells_mut(sbrow)`.
///
/// # Safety
/// `sbrow` must be a valid `ScrollbackLine*`.
#[inline]
unsafe fn scrollback_line_cells_mut(sbrow: *mut c_void) -> *mut c_void {
    unsafe {
        sbrow
            .cast::<u8>()
            .add(std::mem::size_of::<usize>())
            .cast::<c_void>()
    }
}

/// Zero a `VTermScreenCell` at `cell_ptr` (schar=0, width=1).
///
/// Equivalent to `nvim_vterm_cell_zero(cell_ptr)`.
///
/// # Safety
/// `cell_ptr` must be a valid `*mut VTermScreenCell`.
#[inline]
unsafe fn vterm_cell_zero(cell_ptr: *mut c_void) {
    let cell = unsafe { &mut *cell_ptr.cast::<nvim_vterm::VTermScreenCell>() };
    cell.schar = 0;
    cell.width = 1;
}

/// `struct terminal` from terminal.c -- repr(C), layout-verified.
///
/// # Safety
/// This struct must match the C layout exactly. Layout assertions are added
/// via `_Static_assert` in terminal.c and Rust compile-time checks below.
#[repr(C)]
pub struct CTerminal {
    pub opts: CTerminalOptions,             // offset 0,    size 48
    pub vt: *mut c_void,                    // offset 48,   VTerm*
    pub vts: *mut c_void,                   // offset 56,   VTermScreen*
    pub textbuf: [i8; 0x1fff],              // offset 64,   8191 bytes
    _pad_textbuf: u8,                       // offset 8255, 1 byte padding
    pub sb_buffer: *mut *mut c_void,        // offset 8256, ScrollbackLine**
    pub sb_current: usize,                  // offset 8264
    pub sb_size: usize,                     // offset 8272
    pub sb_pending: c_int,                  // offset 8280
    _pad_sb_pending: [u8; 4],               // offset 8284, padding to 8288
    pub sb_deleted: usize,                  // offset 8288
    pub sb_deleted_last: usize,             // offset 8296
    pub title: *mut i8,                     // offset 8304
    pub title_len: usize,                   // offset 8312
    pub title_size: usize,                  // offset 8320
    pub buf_handle: c_int,                  // offset 8328
    pub closed: bool,                       // offset 8332
    pub destroy: bool,                      // offset 8333
    pub forward_mouse: bool,                // offset 8334
    _pad_flags: u8,                         // offset 8335, padding
    pub invalid_start: c_int,               // offset 8336
    pub invalid_end: c_int,                 // offset 8340
    pub cursor: CTerminalCursor,            // offset 8344, size 16
    pub pending: CTerminalPending,          // offset 8360, size 24
    pub theme_updates: bool,                // offset 8384
    pub color_set: [bool; 16],              // offset 8385
    _pad_color: [u8; 7],                    // offset 8401, padding to 8408
    pub selection_buffer: *mut i8,          // offset 8408
    pub selection: CStringBuilder,          // offset 8416
    pub termrequest_buffer: CStringBuilder, // offset 8440
    pub refcount: usize,                    // offset 8464
}

// Compile-time layout assertions (must match offsetof output from C).
const _: () = {
    use std::mem::{offset_of, size_of};
    assert!(size_of::<CTerminal>() == 8472);
    assert!(offset_of!(CTerminal, vt) == 48);
    assert!(offset_of!(CTerminal, vts) == 56);
    assert!(offset_of!(CTerminal, textbuf) == 64);
    assert!(offset_of!(CTerminal, sb_buffer) == 8256);
    assert!(offset_of!(CTerminal, sb_current) == 8264);
    assert!(offset_of!(CTerminal, sb_size) == 8272);
    assert!(offset_of!(CTerminal, sb_pending) == 8280);
    assert!(offset_of!(CTerminal, sb_deleted) == 8288);
    assert!(offset_of!(CTerminal, sb_deleted_last) == 8296);
    assert!(offset_of!(CTerminal, title) == 8304);
    assert!(offset_of!(CTerminal, title_len) == 8312);
    assert!(offset_of!(CTerminal, title_size) == 8320);
    assert!(offset_of!(CTerminal, buf_handle) == 8328);
    assert!(offset_of!(CTerminal, closed) == 8332);
    assert!(offset_of!(CTerminal, destroy) == 8333);
    assert!(offset_of!(CTerminal, forward_mouse) == 8334);
    assert!(offset_of!(CTerminal, invalid_start) == 8336);
    assert!(offset_of!(CTerminal, invalid_end) == 8340);
    assert!(offset_of!(CTerminal, cursor) == 8344);
    assert!(offset_of!(CTerminal, pending) == 8360);
    assert!(offset_of!(CTerminal, theme_updates) == 8384);
    assert!(offset_of!(CTerminal, color_set) == 8385);
    assert!(offset_of!(CTerminal, selection_buffer) == 8408);
    assert!(offset_of!(CTerminal, selection) == 8416);
    assert!(offset_of!(CTerminal, termrequest_buffer) == 8440);
    assert!(offset_of!(CTerminal, refcount) == 8464);
};

impl TerminalHandle {
    /// Get a shared reference to the underlying `CTerminal`.
    ///
    /// # Safety
    /// The handle must be non-null and point to a valid `CTerminal`.
    #[inline]
    pub const unsafe fn as_ref(self) -> &'static CTerminal {
        unsafe { &*(self.0 as *const CTerminal) }
    }

    /// Get a mutable reference to the underlying `CTerminal`.
    ///
    /// # Safety
    /// The handle must be non-null and point to a valid `CTerminal`.
    #[inline]
    pub unsafe fn as_mut(self) -> &'static mut CTerminal {
        unsafe { &mut *(self.0.cast::<CTerminal>()) }
    }
}

// =============================================================================
// TerminalState: C-compatible layout for the terminal mode state machine
// =============================================================================

/// Function pointer type for `VimState` check callback: `int (*)(VimState *)`
type StateCheckFn = Option<unsafe extern "C" fn(*mut c_void) -> c_int>;
/// Function pointer type for `VimState` execute callback: `int (*)(VimState *, int)`
type StateExecFn = Option<unsafe extern "C" fn(*mut c_void, c_int) -> c_int>;

/// `VimState` from `state_defs.h` -- two function pointers.
#[repr(C)]
struct VimStateRust {
    check: StateCheckFn,
    execute: StateExecFn,
}

/// `TerminalState` from `terminal_shim.c` -- layout-verified below.
#[repr(C)]
struct TerminalStateRust {
    state: VimStateRust,       // offset 0,  size 16
    term: *mut c_void,         // offset 16, Terminal *
    save_rd: c_int,            // offset 24, RedrawingDisabled
    close: bool,               // offset 28
    got_bsl: bool,             // offset 29
    got_bsl_o: bool,           // offset 30
    cursor_visible: bool,      // offset 31
    save_curwin_handle: c_int, // offset 32, handle_T
    save_w_p_cul: bool,        // offset 36
    _pad1: [u8; 3],            // offset 37–39
    save_w_p_culopt: *mut i8,  // offset 40, char *
    save_w_p_culopt_flags: u8, // offset 48
    _pad2: [u8; 3],            // offset 49–51
    save_w_p_cuc: c_int,       // offset 52
    save_w_p_so: i64,          // offset 56, OptInt
    save_w_p_siso: i64,        // offset 64, OptInt
}

const _: () = {
    use std::mem::{offset_of, size_of};
    assert!(size_of::<TerminalStateRust>() == 72);
    assert!(offset_of!(TerminalStateRust, term) == 16);
    assert!(offset_of!(TerminalStateRust, save_rd) == 24);
    assert!(offset_of!(TerminalStateRust, close) == 28);
    assert!(offset_of!(TerminalStateRust, got_bsl) == 29);
    assert!(offset_of!(TerminalStateRust, got_bsl_o) == 30);
    assert!(offset_of!(TerminalStateRust, cursor_visible) == 31);
    assert!(offset_of!(TerminalStateRust, save_curwin_handle) == 32);
    assert!(offset_of!(TerminalStateRust, save_w_p_cul) == 36);
    assert!(offset_of!(TerminalStateRust, save_w_p_culopt) == 40);
    assert!(offset_of!(TerminalStateRust, save_w_p_culopt_flags) == 48);
    assert!(offset_of!(TerminalStateRust, save_w_p_cuc) == 52);
    assert!(offset_of!(TerminalStateRust, save_w_p_so) == 56);
    assert!(offset_of!(TerminalStateRust, save_w_p_siso) == 64);
};

// =============================================================================
// FFI for vterm state focus and global state
// =============================================================================

/// `MODE_TERMINAL` constant from `state_defs.h`.
const MODE_TERMINAL: c_int = 0x80;

extern "C" {
    /// Obtain the `VTermState` pointer from a `VTerm`.
    fn vterm_obtain_state(vt: *mut c_void) -> *mut c_void;
    /// Signal focus gained to the `VTerm` state machine.
    fn vterm_state_focus_in(state: *mut c_void);
    /// Signal focus lost to the `VTerm` state machine.
    fn vterm_state_focus_out(state: *mut c_void);
    /// Get the current Neovim `State` global.
    fn nvim_get_state() -> c_int;
}

// =============================================================================
// Terminal Status Functions
// =============================================================================

/// Check if a terminal is running (not closed).
///
/// This is the Rust equivalent of `terminal_running()` in terminal.c.
#[inline]
fn terminal_running_impl(term: TerminalHandle) -> bool {
    if term.is_null() {
        return false;
    }
    !unsafe { term.as_ref().closed }
}

/// FFI wrapper for `terminal_running`.
///
/// Returns 1 if the terminal is running, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_terminal_running(term: TerminalHandle) -> c_int {
    c_int::from(terminal_running_impl(term))
}

/// Direct replacement for C `terminal_running`.
///
/// Returns C `bool` (`_Bool`) matching the C signature.
#[export_name = "terminal_running"]
pub extern "C" fn terminal_running_export(term: TerminalHandle) -> bool {
    terminal_running_impl(term)
}

// =============================================================================
// Terminal Buffer Functions
// =============================================================================

/// Get the buffer handle associated with a terminal.
///
/// This is the Rust equivalent of `terminal_buf()` in terminal.c.
#[no_mangle]
pub extern "C" fn rs_terminal_buf(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().buf_handle }
}

/// Direct replacement for C `terminal_buf`.
///
/// Returns `Buffer` (i32) matching the C signature.
#[export_name = "terminal_buf"]
pub extern "C" fn terminal_buf_export(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().buf_handle }
}

// =============================================================================
// Terminal Cursor Functions
// =============================================================================

/// FFI wrapper for getting terminal cursor row.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_row(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().cursor.row }
}

/// FFI wrapper for getting terminal cursor column.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_col(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().cursor.col }
}

/// FFI wrapper for checking if terminal cursor is visible.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_visible(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(unsafe { term.as_ref().cursor.visible })
}

/// FFI wrapper for getting terminal cursor shape.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_shape(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().cursor.shape }
}

/// FFI wrapper for checking if terminal cursor should blink.
#[no_mangle]
pub extern "C" fn rs_terminal_cursor_blink(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(unsafe { term.as_ref().cursor.blink })
}

// =============================================================================
// Terminal Property Functions
// =============================================================================

/// FFI wrapper for checking if terminal forwards mouse.
#[no_mangle]
pub extern "C" fn rs_terminal_forward_mouse(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(unsafe { term.as_ref().forward_mouse })
}

/// FFI wrapper for checking if terminal wants theme updates.
#[no_mangle]
pub extern "C" fn rs_terminal_theme_updates(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(unsafe { term.as_ref().theme_updates })
}

// =============================================================================
// VTerm Handle Accessors
// =============================================================================

/// Get the `VTerm` handle from a Terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_get_vterm(term: TerminalHandle) -> *mut c_void {
    if term.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { term.as_ref().vt }
}

/// Get the `VTermScreen` handle from a Terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_get_vterm_screen(term: TerminalHandle) -> *mut c_void {
    if term.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { term.as_ref().vts }
}

// =============================================================================
// Scrollback Accessors
// =============================================================================

/// Get the scrollback current count.
#[no_mangle]
pub extern "C" fn rs_terminal_get_sb_current(term: TerminalHandle) -> usize {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().sb_current }
}

/// Get the scrollback size (capacity).
#[no_mangle]
pub extern "C" fn rs_terminal_get_sb_size(term: TerminalHandle) -> usize {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().sb_size }
}

/// Get the scrollback pending count.
#[no_mangle]
pub extern "C" fn rs_terminal_get_sb_pending(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().sb_pending }
}

/// Get the scrollback deleted count.
#[no_mangle]
pub extern "C" fn rs_terminal_get_sb_deleted(term: TerminalHandle) -> usize {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().sb_deleted }
}

// =============================================================================
// Invalid Region Accessors
// =============================================================================

/// Get the invalid start row.
#[no_mangle]
pub extern "C" fn rs_terminal_get_invalid_start(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().invalid_start }
}

/// Get the invalid end row.
#[no_mangle]
pub extern "C" fn rs_terminal_get_invalid_end(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().invalid_end }
}

// =============================================================================
// Pending State Accessors
// =============================================================================

/// Check if terminal has pending resize.
#[no_mangle]
pub extern "C" fn rs_terminal_get_pending_resize(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(unsafe { term.as_ref().pending.resize })
}

/// Check if terminal has pending cursor update.
#[no_mangle]
pub extern "C" fn rs_terminal_get_pending_cursor(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(unsafe { term.as_ref().pending.cursor })
}

// =============================================================================
// Lifecycle State Accessors
// =============================================================================

/// Check if terminal is marked for destruction.
#[no_mangle]
pub extern "C" fn rs_terminal_get_destroy(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(unsafe { term.as_ref().destroy })
}

/// Get the terminal refcount.
#[no_mangle]
pub extern "C" fn rs_terminal_get_refcount(term: TerminalHandle) -> usize {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().refcount }
}

// =============================================================================
// Terminal Mutators
// =============================================================================

/// Set the terminal closed flag.
#[no_mangle]
pub extern "C" fn rs_terminal_set_closed(term: TerminalHandle, closed: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().closed = closed != 0 }
    }
}

/// Set the terminal `forward_mouse` flag.
#[no_mangle]
pub extern "C" fn rs_terminal_set_forward_mouse(term: TerminalHandle, forward: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().forward_mouse = forward != 0 }
    }
}

/// Set the cursor row.
#[no_mangle]
pub extern "C" fn rs_terminal_set_cursor_row(term: TerminalHandle, row: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().cursor.row = row }
    }
}

/// Set the cursor column.
#[no_mangle]
pub extern "C" fn rs_terminal_set_cursor_col(term: TerminalHandle, col: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().cursor.col = col }
    }
}

/// Set the cursor visible flag.
#[no_mangle]
pub extern "C" fn rs_terminal_set_cursor_visible(term: TerminalHandle, visible: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().cursor.visible = visible != 0 }
    }
}

/// Set the cursor shape.
#[no_mangle]
pub extern "C" fn rs_terminal_set_cursor_shape(term: TerminalHandle, shape: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().cursor.shape = shape }
    }
}

/// Set the cursor blink flag.
#[no_mangle]
pub extern "C" fn rs_terminal_set_cursor_blink(term: TerminalHandle, blink: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().cursor.blink = blink != 0 }
    }
}

/// Set the invalid region.
#[no_mangle]
pub extern "C" fn rs_terminal_set_invalid_region(term: TerminalHandle, start: c_int, end: c_int) {
    if !term.is_null() {
        let t = unsafe { term.as_mut() };
        t.invalid_start = start;
        t.invalid_end = end;
    }
}

/// Set the pending resize flag.
#[no_mangle]
pub extern "C" fn rs_terminal_set_pending_resize(term: TerminalHandle, resize: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().pending.resize = resize != 0 }
    }
}

/// Set the pending cursor flag.
#[no_mangle]
pub extern "C" fn rs_terminal_set_pending_cursor(term: TerminalHandle, cursor: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().pending.cursor = cursor != 0 }
    }
}

/// Set the scrollback pending count.
#[no_mangle]
pub extern "C" fn rs_terminal_set_sb_pending(term: TerminalHandle, pending: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().sb_pending = pending }
    }
}

/// Increment the terminal refcount.
#[no_mangle]
pub extern "C" fn rs_terminal_inc_refcount(term: TerminalHandle) {
    if !term.is_null() {
        unsafe { term.as_mut().refcount += 1 }
    }
}

/// Decrement the terminal refcount.
#[no_mangle]
pub extern "C" fn rs_terminal_dec_refcount(term: TerminalHandle) {
    if !term.is_null() {
        let t = unsafe { term.as_mut() };
        if t.refcount > 0 {
            t.refcount -= 1;
        }
    }
}

// =============================================================================
// Terminal Lifecycle Functions
// =============================================================================

/// Set the buffer handle on a terminal.
///
/// Used to associate or disassociate a terminal with a buffer.
/// Set to 0 to clear the association.
#[no_mangle]
pub extern "C" fn rs_terminal_set_buf_handle(term: TerminalHandle, buf_handle: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().buf_handle = buf_handle }
    }
}

/// Mark a terminal for destruction.
#[no_mangle]
pub extern "C" fn rs_terminal_set_destroy(term: TerminalHandle, destroy: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().destroy = destroy != 0 }
    }
}

/// Check if a terminal handle is valid.
///
/// A terminal is valid if the pointer is non-null and has a `VTerm` instance.
/// Returns 1 if valid, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_terminal_is_valid(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(!unsafe { term.as_ref().vt.is_null() })
}

/// Check if a terminal can be destroyed.
///
/// A terminal can be destroyed when its refcount is 0.
/// Returns 1 if it can be destroyed, 0 otherwise.
#[no_mangle]
pub extern "C" fn rs_terminal_can_destroy(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(unsafe { term.as_ref().refcount == 0 })
}

/// Get the opts.data pointer from a terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_get_opts_data(term: TerminalHandle) -> *mut c_void {
    if term.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { term.as_ref().opts.data }
}

/// Get the configured width from terminal options.
#[no_mangle]
pub extern "C" fn rs_terminal_get_opts_width(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(unsafe { term.as_ref().opts.width })
}

/// Get the configured height from terminal options.
#[no_mangle]
pub extern "C" fn rs_terminal_get_opts_height(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    c_int::from(unsafe { term.as_ref().opts.height })
}

/// Prepare a terminal for close.
///
/// Sets `forward_mouse` to false and marks the terminal as closed.
#[no_mangle]
pub extern "C" fn rs_terminal_prepare_close(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_mut() };
    t.forward_mouse = false;
    t.closed = true;
}

/// Clear buffer association from a terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_clear_buf_handle(term: TerminalHandle) {
    if !term.is_null() {
        unsafe { term.as_mut().buf_handle = 0 }
    }
}

/// Mark a terminal for destruction and clear buffer association.
#[no_mangle]
pub extern "C" fn rs_terminal_mark_for_destruction(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_mut() };
    t.buf_handle = 0;
    t.destroy = true;
}

// =============================================================================
// Display and Refresh Functions
// =============================================================================

/// Get the `sb_deleted_last` field from a terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_get_sb_deleted_last(term: TerminalHandle) -> usize {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().sb_deleted_last }
}

/// Set the `sb_deleted_last` field on a terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_set_sb_deleted_last(term: TerminalHandle, value: usize) {
    if !term.is_null() {
        unsafe { term.as_mut().sb_deleted_last = value }
    }
}

/// Set the `sb_current` field on a terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_set_sb_current(term: TerminalHandle, value: usize) {
    if !term.is_null() {
        unsafe { term.as_mut().sb_current = value }
    }
}

/// Get the title from a terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_get_title(term: TerminalHandle) -> *const i8 {
    if term.is_null() {
        return std::ptr::null();
    }
    unsafe { term.as_ref().title }
}

/// Get the title length from a terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_get_title_len(term: TerminalHandle) -> usize {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().title_len }
}

/// Get the textbuf pointer from a terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_get_textbuf(term: TerminalHandle) -> *mut i8 {
    if term.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { term.as_mut().textbuf.as_mut_ptr() }
}

/// Get the textbuf size constant (0x1fff).
#[no_mangle]
pub extern "C" fn rs_terminal_get_textbuf_size() -> usize {
    0x1fff
}

/// Invalidate a region of the terminal screen.
///
/// Use -1, -1 to invalidate the entire screen without changing the region.
#[no_mangle]
pub extern "C" fn rs_terminal_invalidate_region(term: TerminalHandle, start: c_int, end: c_int) {
    if term.is_null() {
        return;
    }
    if start != -1 && end != -1 {
        let t = unsafe { term.as_mut() };
        let new_start = start.min(t.invalid_start);
        let new_end = end.max(t.invalid_end);
        t.invalid_start = new_start;
        t.invalid_end = new_end;
    }
}

/// Reset the invalid region to cover the full screen height.
#[no_mangle]
pub extern "C" fn rs_terminal_invalidate_all(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    let vt = unsafe { term.as_ref().vt };
    if !vt.is_null() {
        let size = unsafe { rs_vterm_get_size(vt) };
        let t = unsafe { term.as_mut() };
        t.invalid_start = 0;
        t.invalid_end = size.rows;
    }
}

/// Sync the `sb_deleted_last` field with `sb_deleted`.
#[no_mangle]
pub extern "C" fn rs_terminal_sync_sb_deleted(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_mut() };
    t.sb_deleted_last = t.sb_deleted;
}

// =============================================================================
// Mode Integration Functions
// =============================================================================

/// Set terminal focus state (calls `vterm_state_focus_in/out`).
#[no_mangle]
pub extern "C" fn rs_terminal_set_focus(term: TerminalHandle, focus: c_int) {
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_ref() };
    if t.vt.is_null() {
        return;
    }
    let state = unsafe { vterm_obtain_state(t.vt) };
    if state.is_null() {
        return;
    }
    if focus != 0 {
        unsafe { vterm_state_focus_in(state) };
    } else {
        unsafe { vterm_state_focus_out(state) };
    }
}

/// Check if the terminal should be closed.
///
/// Returns 1 if the terminal's closed flag is set and `buf_handle` is 0.
#[no_mangle]
pub extern "C" fn rs_terminal_should_close(term: TerminalHandle) -> c_int {
    if term.is_null() {
        return 0;
    }
    let t = unsafe { term.as_ref() };
    c_int::from(t.closed && t.buf_handle == 0)
}

/// Get the `MODE_TERMINAL` constant.
#[no_mangle]
pub extern "C" fn rs_get_mode_terminal() -> c_int {
    MODE_TERMINAL
}

/// Check if currently in terminal mode.
#[no_mangle]
pub extern "C" fn rs_is_terminal_mode() -> c_int {
    c_int::from((unsafe { nvim_get_state() } & MODE_TERMINAL) != 0)
}

/// Convenience function to handle terminal focus gain.
#[no_mangle]
pub extern "C" fn rs_terminal_focus_gain(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    rs_terminal_set_focus(term, 1);
    unsafe { term.as_mut().pending.cursor = true };
}

/// Convenience function to handle terminal focus loss.
#[no_mangle]
pub extern "C" fn rs_terminal_focus_lose(term: TerminalHandle) {
    rs_terminal_set_focus(term, 0);
}

// =============================================================================
// VTerm Callback Implementations
// =============================================================================

extern "C" {
    /// `vim_beep` from `ui.h`.
    fn vim_beep(flag: c_uint);
}
/// `kOptBoFlagTerm` from `option_vars.generated.h`.
const OPT_BO_FLAG_TERM: c_uint = 0x40000;

/// Terminal bell callback -- ring the system bell.
///
/// Replaces the body of `term_bell` in `terminal_shim.c`.
#[no_mangle]
pub extern "C" fn rs_terminal_bell() -> c_int {
    unsafe { vim_beep(OPT_BO_FLAG_TERM) };
    1
}

/// Terminal theme query callback -- report whether the background is dark.
///
/// Replaces the body of `term_theme` in `terminal_shim.c`.
///
/// # Safety
/// `dark` must be a valid non-null pointer to a `bool`.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_theme_query(dark: *mut bool) -> c_int {
    if dark.is_null() {
        return 0;
    }
    let bg = unsafe { *c_p_bg };
    unsafe { *dark = bg == 0x64i8 }; // 0x64 == b'd' (dark)
    1
}

// =============================================================================
// Callback Helper Functions
// =============================================================================

/// HL_ flag constants for underline attributes (must match `highlight_defs.h`).
const HL_UNDERLINE: c_int = 0x08;
const HL_UNDERCURL: c_int = 0x10;
const HL_UNDERDOUBLE: c_int = 0x18;

/// Map a `VTermScreenCellAttrs` underline style to the corresponding `HL_*` flag.
///
/// Returns `HL_UNDERLINE`, `HL_UNDERDOUBLE`, `HL_UNDERCURL`, or `0`.
/// Replaces the C `get_underline_hl_flag` in `terminal_shim.c`.
#[no_mangle]
pub extern "C" fn rs_terminal_underline_hl_flag(attrs: VTermScreenCellAttrs) -> c_int {
    match attrs.underline() {
        0 => 0,
        2 => HL_UNDERDOUBLE,
        3 => HL_UNDERCURL,
        _ => HL_UNDERLINE, // 1 (single) and unknown values
    }
}

/// Parse an OSC 8 hyperlink sequence and return the URL attribute id.
///
/// The `str` pointer must be nul-terminated and point to the content after
/// the `\x1b]8;` prefix (i.e. `id;uri`).  Returns 1 on success (writing the
/// `hl_add_url` result into `*attr_out`) and 0 if the sequence is invalid.
///
/// Replaces the C `parse_osc8` in `terminal_shim.c`.
///
/// # Safety
/// `str_ptr` must be a valid nul-terminated C string.  `attr_out` must be a
/// valid, non-null pointer to a `c_int`.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_parse_osc8(
    str_ptr: *const std::ffi::c_char,
    attr_out: *mut c_int,
) -> c_int {
    extern "C" {
        fn hl_add_url(attr: c_int, url: *const std::ffi::c_char) -> c_int;
    }

    if str_ptr.is_null() || attr_out.is_null() {
        return 0;
    }

    // Find the semicolon separating id from uri.
    let mut i = 0usize;
    loop {
        let ch: std::ffi::c_char = unsafe { *str_ptr.add(i) };
        if ch == 0 {
            // No semicolon found -- invalid sequence.
            return 0;
        }
        // 0x3B == b';' (ASCII semicolon)
        if ch == 0x3B {
            break;
        }
        i += 1;
    }

    // Move past the semicolon.
    i += 1;

    let ch: std::ffi::c_char = unsafe { *str_ptr.add(i) };
    if ch == 0 {
        // Empty URI -- clear the URL attribute.
        unsafe { *attr_out = 0 };
        return 1;
    }

    // Pass the URI part to hl_add_url.
    let uri_ptr = unsafe { str_ptr.add(i) };
    unsafe { *attr_out = hl_add_url(0, uri_ptr) };
    1
}

/// Set the cursor position on a terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_set_cursor_pos(term: TerminalHandle, row: c_int, col: c_int) {
    if !term.is_null() {
        let t = unsafe { term.as_mut() };
        t.cursor.row = row;
        t.cursor.col = col;
    }
}

/// Set the cursor visibility flag.
#[no_mangle]
pub extern "C" fn rs_terminal_set_cursor_vis(term: TerminalHandle, visible: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().cursor.visible = visible != 0 }
    }
}

/// Get the scrollback buffer pointer.
#[no_mangle]
pub extern "C" fn rs_terminal_get_sb_buffer(term: TerminalHandle) -> *mut *mut c_void {
    if term.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { term.as_ref().sb_buffer }
}

/// Increment the scrollback current count.
#[no_mangle]
pub extern "C" fn rs_terminal_inc_sb_current(term: TerminalHandle) {
    if !term.is_null() {
        let t = unsafe { term.as_mut() };
        if t.sb_current < t.sb_size {
            t.sb_current += 1;
        }
    }
}

/// Decrement the scrollback current count.
#[no_mangle]
pub extern "C" fn rs_terminal_dec_sb_current(term: TerminalHandle) {
    if !term.is_null() {
        let t = unsafe { term.as_mut() };
        if t.sb_current > 0 {
            t.sb_current -= 1;
        }
    }
}

/// Increment the scrollback deleted count.
#[no_mangle]
pub extern "C" fn rs_terminal_inc_sb_deleted(term: TerminalHandle) {
    if !term.is_null() {
        unsafe { term.as_mut().sb_deleted += 1 }
    }
}

/// Get the scrollback deleted count.
#[no_mangle]
pub extern "C" fn rs_terminal_get_sb_deleted_val(term: TerminalHandle) -> usize {
    if term.is_null() {
        return 0;
    }
    unsafe { term.as_ref().sb_deleted }
}

/// Handle cursor move callback.
#[no_mangle]
pub extern "C" fn rs_terminal_on_cursor_move(term: TerminalHandle, row: c_int, col: c_int) {
    if !term.is_null() {
        let t = unsafe { term.as_mut() };
        t.cursor.row = row;
        t.cursor.col = col;
    }
}

/// Handle cursor visibility change.
#[no_mangle]
pub extern "C" fn rs_terminal_on_cursor_visible(term: TerminalHandle, visible: c_int) {
    if !term.is_null() {
        unsafe { term.as_mut().cursor.visible = visible != 0 }
    }
}

// =============================================================================
// VTerm Size Types
// =============================================================================

/// Size of a `VTerm` instance (rows and columns).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VTermSize {
    /// Number of rows
    pub rows: c_int,
    /// Number of columns
    pub cols: c_int,
}

// =============================================================================
// Terminal I/O Operations
// =============================================================================

// External C functions for I/O operations
extern "C" {
    // VTerm I/O functions (defined in vterm crate)
    fn rs_vterm_input_write(vt: *mut c_void, data: *const i8, len: usize) -> usize;
    fn rs_vterm_keyboard_key(vt: *mut c_void, key: c_int, mods: c_int);
    fn rs_vterm_keyboard_unichar(vt: *mut c_void, ch: u32, mods: c_int);
    fn rs_vterm_keyboard_start_paste(vt: *mut c_void);
    fn rs_vterm_keyboard_end_paste(vt: *mut c_void);
    fn rs_vterm_mouse_move(vt: *mut c_void, row: c_int, col: c_int, mods: c_int);
    fn rs_vterm_mouse_button(vt: *mut c_void, button: c_int, pressed: c_int, mods: c_int);
    fn rs_vterm_screen_flush_damage(vts: *mut c_void);

    // VTerm size function (defined in vterm crate)
    fn rs_vterm_get_size(vt: *mut c_void) -> VTermSize;
    fn rs_vterm_set_size(vt: *mut c_void, rows: c_int, cols: c_int) -> c_int;
    // Phase 12: terminal_check_size helper
    fn nvim_terminal_find_size(term: *mut c_void, out_width: *mut u16, out_height: *mut u16);
    // Phase 14: terminal_enter state machine helpers
    // nvim_get_state already declared above
    fn nvim_set_state(s: c_int);
    fn nvim_get_RedrawingDisabled() -> c_int;
    fn nvim_set_RedrawingDisabled(v: c_int);
    static mut mapped_ctrl_c: c_int;
    fn nvim_get_stop_insert_mode() -> c_int;
    fn nvim_set_stop_insert_mode(v: c_int);
    fn nvim_get_restart_edit() -> c_int;
    fn nvim_set_restart_edit(v: c_int);
    fn showmode();
    #[link_name = "unshowmode"]
    fn nvim_unshowmode(force: bool);
    #[link_name = "ui_cursor_shape"]
    fn nvim_ui_cursor_shape();
    #[link_name = "setcursor"]
    fn nvim_setcursor();
    #[link_name = "parse_shape_opt"]
    fn nvim_parse_shape_opt(scope: c_int) -> *const i8;
    #[link_name = "show_cursor_info_later"]
    fn nvim_show_cursor_info_later(force: bool);
    // rs_refresh_cursor is defined in this Rust crate -- no extern needed
    #[link_name = "validate_cursor"]
    fn nvim_validate_cursor_cw(wp: *mut c_void);
    #[link_name = "update_screen"]
    fn nvim_update_screen_c();
    #[link_name = "redraw_statuslines"]
    fn nvim_redraw_statuslines();
    #[link_name = "ui_flush"]
    fn nvim_ui_flush();
    // apply_autocmds(event, fname, fname_io, force, buf)
    fn apply_autocmds(
        event: c_int,
        fname: *mut i8,
        fname_io: *mut i8,
        force: bool,
        buf: *mut c_void,
    ) -> bool;
    // has_event(event) -> int
    fn has_event(event: c_int) -> c_int;
    fn may_trigger_modechanged();
    #[link_name = "may_trigger_win_scrolled_resized"]
    fn nvim_may_trigger_win_scrolled_resized();
    fn nvim_term_buf_get_changedtick(buf: *const c_void) -> i64;
    #[link_name = "state_enter"]
    fn nvim_state_enter_c(state: *mut c_void);
    #[link_name = "merge_modifiers"]
    fn nvim_merge_modifiers_c(key: c_int, tmp_mod_mask: *mut c_int) -> c_int;
    #[link_name = "paste_repeat"]
    fn nvim_paste_repeat_c(count: c_int);
    fn state_handle_k_event();
    // do_cmdline(cmdline, fgetline, cookie, flags)
    fn do_cmdline(
        cmdline: *mut i8,
        fgetline: Option<unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut i8>,
        cookie: *mut c_void,
        flags: c_int,
    );
    fn getcmdkeycmd(promptc: c_int, cookie: *mut c_void, indent: c_int, do_concat: bool)
        -> *mut i8;
    // map_execute_lua(may_repeat, discard) -> bool
    #[link_name = "map_execute_lua"]
    fn nvim_map_execute_lua_c(may_repeat: bool, discard: bool) -> bool;
    #[link_name = "rs_set_terminal_winopts"]
    fn nvim_terminal_set_winopts(s: *mut c_void);
    #[link_name = "rs_unset_terminal_winopts"]
    fn nvim_terminal_unset_winopts(s: *mut c_void);
    // rs_terminal_check_cursor is defined in this crate -- call directly
    // Mouse event accessors (Phase 3)
    static mouse_row: c_int;
    static mouse_col: c_int;
    static mouse_grid: c_int;
    // nvim_wf_mouse_find_win_inner is the same function in window_shim.c
    #[link_name = "nvim_wf_mouse_find_win_inner"]
    fn nvim_mouse_find_win_inner(grid: *mut c_int, row: *mut c_int, col: *mut c_int)
        -> *mut c_void;
    #[link_name = "win_col_off"]
    fn nvim_win_col_off(wp: *mut c_void) -> c_int;
    fn nvim_get_vgetc_char() -> c_int;
    #[link_name = "ins_char_typebuf"]
    fn nvim_ins_char_typebuf_c(c: c_int, mod_mask_val: c_int, on_key_ignore: bool) -> c_int;
    #[link_name = "ungetchars"]
    fn nvim_ungetchars(len: c_int);
    fn nvim_do_mousescroll_c(term: *mut c_void, mouse_win: *mut c_void, c: c_int) -> c_int;
    // do_buffer_ext(action, start, dir, count, flags)
    // DOBUF_WIPE=4, DOBUF_FIRST=1, FORWARD=1, DOBUF_FORCEIT=1
    #[link_name = "do_buffer_ext"]
    fn nvim_do_buffer_wipe(action: c_int, start: c_int, dir: c_int, count: c_int, flags: c_int);
    // Phase 13: terminal_close helpers
    #[link_name = "block_autocmds"]
    fn c_block_autocmds();
    #[link_name = "unblock_autocmds"]
    fn c_unblock_autocmds();
    fn nvim_terminal_opts_is_internal(term: *mut c_void) -> c_int;
    fn nvim_terminal_apply_termclose_event(buf: *mut c_void, status: c_int);
    static mut mod_mask: c_int;
}

/// Write input data to a terminal's `VTerm` instance.
///
/// This combines getting the `VTerm` handle and calling `vterm_input_write`.
/// Returns the number of bytes written.
///
/// # Safety
/// The data pointer must be valid and point to at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_input_write(
    term: TerminalHandle,
    data: *const i8,
    len: usize,
) -> usize {
    if term.is_null() || data.is_null() {
        return 0;
    }
    let vt = unsafe { term.as_ref().vt };
    if vt.is_null() {
        return 0;
    }
    rs_vterm_input_write(vt, data, len)
}

/// Flush screen damage on a terminal's `VTermScreen`.
#[no_mangle]
pub extern "C" fn rs_terminal_flush_damage(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    let vts = unsafe { term.as_ref().vts };
    if !vts.is_null() {
        unsafe { rs_vterm_screen_flush_damage(vts) }
    }
}

/// Write input data to a terminal and flush damage.
///
/// # Safety
/// The data pointer must be valid and point to at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_receive(
    term: TerminalHandle,
    data: *const i8,
    len: usize,
) -> usize {
    if term.is_null() || data.is_null() {
        return 0;
    }
    let t = unsafe { term.as_ref() };
    if t.vt.is_null() {
        return 0;
    }
    let written = rs_vterm_input_write(t.vt, data, len);
    if !t.vts.is_null() {
        rs_vterm_screen_flush_damage(t.vts);
    }
    written
}

/// Send a keyboard key to a terminal's `VTerm` instance.
#[no_mangle]
pub extern "C" fn rs_terminal_send_key(term: TerminalHandle, key: c_int, mods: c_int) {
    if term.is_null() {
        return;
    }
    let vt = unsafe { term.as_ref().vt };
    if !vt.is_null() {
        unsafe { rs_vterm_keyboard_key(vt, key, mods) }
    }
}

/// Send a Unicode character to a terminal's `VTerm` instance.
#[no_mangle]
pub extern "C" fn rs_terminal_send_unichar(term: TerminalHandle, ch: u32, mods: c_int) {
    if term.is_null() {
        return;
    }
    let vt = unsafe { term.as_ref().vt };
    if !vt.is_null() {
        unsafe { rs_vterm_keyboard_unichar(vt, ch, mods) }
    }
}

/// Start a paste operation on a terminal's `VTerm` instance.
#[no_mangle]
pub extern "C" fn rs_terminal_start_paste(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    let vt = unsafe { term.as_ref().vt };
    if !vt.is_null() {
        unsafe { rs_vterm_keyboard_start_paste(vt) }
    }
}

/// End a paste operation on a terminal's `VTerm` instance.
#[no_mangle]
pub extern "C" fn rs_terminal_end_paste(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    let vt = unsafe { term.as_ref().vt };
    if !vt.is_null() {
        unsafe { rs_vterm_keyboard_end_paste(vt) }
    }
}

// =============================================================================
// VTerm Key Constants (from vterm_keycodes.h)
// =============================================================================

/// No key.
pub const VTERM_KEY_NONE: c_int = 0;
/// Enter key.
pub const VTERM_KEY_ENTER: c_int = 1;
/// Tab key.
pub const VTERM_KEY_TAB: c_int = 2;
/// Backspace key.
pub const VTERM_KEY_BACKSPACE: c_int = 3;
/// Escape key.
pub const VTERM_KEY_ESCAPE: c_int = 4;
/// Up arrow key.
pub const VTERM_KEY_UP: c_int = 5;
/// Down arrow key.
pub const VTERM_KEY_DOWN: c_int = 6;
/// Left arrow key.
pub const VTERM_KEY_LEFT: c_int = 7;
/// Right arrow key.
pub const VTERM_KEY_RIGHT: c_int = 8;
/// Insert key.
pub const VTERM_KEY_INS: c_int = 9;
/// Delete key.
pub const VTERM_KEY_DEL: c_int = 10;
/// Home key.
pub const VTERM_KEY_HOME: c_int = 11;
/// End key.
pub const VTERM_KEY_END: c_int = 12;
/// Page Up key.
pub const VTERM_KEY_PAGEUP: c_int = 13;
/// Page Down key.
pub const VTERM_KEY_PAGEDOWN: c_int = 14;
/// Keypad 0.
pub const VTERM_KEY_KP_0: c_int = 16;
/// Keypad 1.
pub const VTERM_KEY_KP_1: c_int = 17;
/// Keypad 2.
pub const VTERM_KEY_KP_2: c_int = 18;
/// Keypad 3.
pub const VTERM_KEY_KP_3: c_int = 19;
/// Keypad 4.
pub const VTERM_KEY_KP_4: c_int = 20;
/// Keypad 5.
pub const VTERM_KEY_KP_5: c_int = 21;
/// Keypad 6.
pub const VTERM_KEY_KP_6: c_int = 22;
/// Keypad 7.
pub const VTERM_KEY_KP_7: c_int = 23;
/// Keypad 8.
pub const VTERM_KEY_KP_8: c_int = 24;
/// Keypad 9.
pub const VTERM_KEY_KP_9: c_int = 25;
/// Keypad multiply (*).
pub const VTERM_KEY_KP_MULT: c_int = 26;
/// Keypad plus (+).
pub const VTERM_KEY_KP_PLUS: c_int = 27;
/// Keypad comma (,).
pub const VTERM_KEY_KP_COMMA: c_int = 28;
/// Keypad minus (-).
pub const VTERM_KEY_KP_MINUS: c_int = 29;
/// Keypad period (.).
pub const VTERM_KEY_KP_PERIOD: c_int = 30;
/// Keypad divide (/).
pub const VTERM_KEY_KP_DIVIDE: c_int = 31;
/// Keypad Enter.
pub const VTERM_KEY_KP_ENTER: c_int = 32;
/// Keypad equal (=).
pub const VTERM_KEY_KP_EQUAL: c_int = 33;
/// Maximum keypad key (sentinel for function keys).
pub const VTERM_KEY_MAX: c_int = VTERM_KEY_KP_EQUAL;
/// Number of function keys supported.
pub const VTERM_KEY_FUNCTION_MAX: c_int = 66;

/// Generate function key code F1-F66.
#[inline]
pub const fn vterm_key_function(n: c_int) -> c_int {
    if n < 1 || n > VTERM_KEY_FUNCTION_MAX {
        VTERM_KEY_NONE
    } else {
        VTERM_KEY_MAX + n
    }
}

// =============================================================================
// VTerm Modifier Constants
// =============================================================================

/// No modifier.
pub const VTERM_MOD_NONE: c_int = 0;
/// Shift modifier.
pub const VTERM_MOD_SHIFT: c_int = 1;
/// Alt modifier.
pub const VTERM_MOD_ALT: c_int = 2;
/// Ctrl modifier.
pub const VTERM_MOD_CTRL: c_int = 4;

// =============================================================================
// Neovim Key Constants (from keycodes.h)
// =============================================================================

/// Convert termcap codes to internal key representation
const fn termcap2key(a: c_int, b: c_int) -> c_int {
    -((a) + (b << 8))
}

// KS_EXTRA for special keys
const KS_EXTRA: c_int = 253;

// Neovim special key codes
const K_BS: c_int = termcap2key(b'k' as c_int, b'b' as c_int);
const K_UP: c_int = termcap2key(b'k' as c_int, b'u' as c_int);
const K_DOWN: c_int = termcap2key(b'k' as c_int, b'd' as c_int);
const K_LEFT: c_int = termcap2key(b'k' as c_int, b'l' as c_int);
const K_RIGHT: c_int = termcap2key(b'k' as c_int, b'r' as c_int);
const K_HOME: c_int = termcap2key(b'k' as c_int, b'h' as c_int);
const K_END: c_int = termcap2key(b'@' as c_int, b'7' as c_int);
const K_PAGEUP: c_int = termcap2key(b'k' as c_int, b'P' as c_int);
const K_PAGEDOWN: c_int = termcap2key(b'k' as c_int, b'N' as c_int);
const K_INS: c_int = termcap2key(b'k' as c_int, b'I' as c_int);
const K_DEL: c_int = termcap2key(b'k' as c_int, b'D' as c_int);
const K_S_TAB: c_int = termcap2key(b'k' as c_int, b'B' as c_int);

// Keypad keys
const K_KUP: c_int = termcap2key(b'K' as c_int, b'u' as c_int);
const K_KDOWN: c_int = termcap2key(b'K' as c_int, b'd' as c_int);
const K_KLEFT: c_int = termcap2key(b'K' as c_int, b'l' as c_int);
const K_KRIGHT: c_int = termcap2key(b'K' as c_int, b'r' as c_int);
const K_KHOME: c_int = termcap2key(b'K' as c_int, b'1' as c_int);
const K_KEND: c_int = termcap2key(b'K' as c_int, b'4' as c_int);
const K_KPAGEUP: c_int = termcap2key(b'K' as c_int, b'3' as c_int);
const K_KPAGEDOWN: c_int = termcap2key(b'K' as c_int, b'5' as c_int);
const K_KORIGIN: c_int = termcap2key(b'K' as c_int, b'7' as c_int);
const K_K0: c_int = termcap2key(b'K' as c_int, b'0' as c_int);
const K_K1: c_int = termcap2key(b'K' as c_int, b'a' as c_int);
const K_K2: c_int = termcap2key(b'K' as c_int, b'b' as c_int);
const K_K3: c_int = termcap2key(b'K' as c_int, b'c' as c_int);
const K_K4: c_int = termcap2key(b'K' as c_int, b'e' as c_int);
const K_K5: c_int = termcap2key(b'K' as c_int, b'f' as c_int);
const K_K6: c_int = termcap2key(b'K' as c_int, b'g' as c_int);
const K_K7: c_int = termcap2key(b'K' as c_int, b'h' as c_int);
const K_K8: c_int = termcap2key(b'K' as c_int, b'i' as c_int);
const K_K9: c_int = termcap2key(b'K' as c_int, b'j' as c_int);
const K_KINS: c_int = termcap2key(b'K' as c_int, b'I' as c_int);
const K_KDEL: c_int = termcap2key(b'K' as c_int, b'D' as c_int);
const K_KPOINT: c_int = termcap2key(b'K' as c_int, b'.' as c_int);
const K_KENTER: c_int = termcap2key(b'K' as c_int, b'\r' as c_int);
const K_KPLUS: c_int = termcap2key(b'K' as c_int, b'+' as c_int);
const K_KMINUS: c_int = termcap2key(b'K' as c_int, b'-' as c_int);
const K_KMULTIPLY: c_int = termcap2key(b'K' as c_int, b'*' as c_int);
const K_KDIVIDE: c_int = termcap2key(b'K' as c_int, b'/' as c_int);

// Function keys F1-F12
const K_F1: c_int = termcap2key(b'k' as c_int, b'1' as c_int);
const K_F2: c_int = termcap2key(b'k' as c_int, b'2' as c_int);
const K_F3: c_int = termcap2key(b'k' as c_int, b'3' as c_int);
const K_F4: c_int = termcap2key(b'k' as c_int, b'4' as c_int);
const K_F5: c_int = termcap2key(b'k' as c_int, b'5' as c_int);
const K_F6: c_int = termcap2key(b'k' as c_int, b'6' as c_int);
const K_F7: c_int = termcap2key(b'k' as c_int, b'7' as c_int);
const K_F8: c_int = termcap2key(b'k' as c_int, b'8' as c_int);
const K_F9: c_int = termcap2key(b'k' as c_int, b'9' as c_int);
const K_F10: c_int = termcap2key(b'k' as c_int, b';' as c_int);
const K_F11: c_int = termcap2key(b'F' as c_int, b'1' as c_int);
const K_F12: c_int = termcap2key(b'F' as c_int, b'2' as c_int);

// Function keys F13-F63 (TERMCAP2KEY('F', char))
const K_F13: c_int = termcap2key(b'F' as c_int, b'3' as c_int);
const K_F14: c_int = termcap2key(b'F' as c_int, b'4' as c_int);
const K_F15: c_int = termcap2key(b'F' as c_int, b'5' as c_int);
const K_F16: c_int = termcap2key(b'F' as c_int, b'6' as c_int);
const K_F17: c_int = termcap2key(b'F' as c_int, b'7' as c_int);
const K_F18: c_int = termcap2key(b'F' as c_int, b'8' as c_int);
const K_F19: c_int = termcap2key(b'F' as c_int, b'9' as c_int);
const K_F20: c_int = termcap2key(b'F' as c_int, b'A' as c_int);
const K_F21: c_int = termcap2key(b'F' as c_int, b'B' as c_int);
const K_F22: c_int = termcap2key(b'F' as c_int, b'C' as c_int);
const K_F23: c_int = termcap2key(b'F' as c_int, b'D' as c_int);
const K_F24: c_int = termcap2key(b'F' as c_int, b'E' as c_int);
const K_F25: c_int = termcap2key(b'F' as c_int, b'F' as c_int);
const K_F26: c_int = termcap2key(b'F' as c_int, b'G' as c_int);
const K_F27: c_int = termcap2key(b'F' as c_int, b'H' as c_int);
const K_F28: c_int = termcap2key(b'F' as c_int, b'I' as c_int);
const K_F29: c_int = termcap2key(b'F' as c_int, b'J' as c_int);
const K_F30: c_int = termcap2key(b'F' as c_int, b'K' as c_int);
const K_F31: c_int = termcap2key(b'F' as c_int, b'L' as c_int);
const K_F32: c_int = termcap2key(b'F' as c_int, b'M' as c_int);
const K_F33: c_int = termcap2key(b'F' as c_int, b'N' as c_int);
const K_F34: c_int = termcap2key(b'F' as c_int, b'O' as c_int);
const K_F35: c_int = termcap2key(b'F' as c_int, b'P' as c_int);
const K_F36: c_int = termcap2key(b'F' as c_int, b'Q' as c_int);
const K_F37: c_int = termcap2key(b'F' as c_int, b'R' as c_int);
const K_F38: c_int = termcap2key(b'F' as c_int, b'S' as c_int);
const K_F39: c_int = termcap2key(b'F' as c_int, b'T' as c_int);
const K_F40: c_int = termcap2key(b'F' as c_int, b'U' as c_int);
const K_F41: c_int = termcap2key(b'F' as c_int, b'V' as c_int);
const K_F42: c_int = termcap2key(b'F' as c_int, b'W' as c_int);
const K_F43: c_int = termcap2key(b'F' as c_int, b'X' as c_int);
const K_F44: c_int = termcap2key(b'F' as c_int, b'Y' as c_int);
const K_F45: c_int = termcap2key(b'F' as c_int, b'Z' as c_int);
const K_F46: c_int = termcap2key(b'F' as c_int, b'a' as c_int);
const K_F47: c_int = termcap2key(b'F' as c_int, b'b' as c_int);
const K_F48: c_int = termcap2key(b'F' as c_int, b'c' as c_int);
const K_F49: c_int = termcap2key(b'F' as c_int, b'd' as c_int);
const K_F50: c_int = termcap2key(b'F' as c_int, b'e' as c_int);
const K_F51: c_int = termcap2key(b'F' as c_int, b'f' as c_int);
const K_F52: c_int = termcap2key(b'F' as c_int, b'g' as c_int);
const K_F53: c_int = termcap2key(b'F' as c_int, b'h' as c_int);
const K_F54: c_int = termcap2key(b'F' as c_int, b'i' as c_int);
const K_F55: c_int = termcap2key(b'F' as c_int, b'j' as c_int);
const K_F56: c_int = termcap2key(b'F' as c_int, b'k' as c_int);
const K_F57: c_int = termcap2key(b'F' as c_int, b'l' as c_int);
const K_F58: c_int = termcap2key(b'F' as c_int, b'm' as c_int);
const K_F59: c_int = termcap2key(b'F' as c_int, b'n' as c_int);
const K_F60: c_int = termcap2key(b'F' as c_int, b'o' as c_int);
const K_F61: c_int = termcap2key(b'F' as c_int, b'p' as c_int);
const K_F62: c_int = termcap2key(b'F' as c_int, b'q' as c_int);
const K_F63: c_int = termcap2key(b'F' as c_int, b'r' as c_int);

// Shifted function keys (KS_EXTRA variants)
const KE_S_UP: c_int = 4;
const KE_S_DOWN: c_int = 5;
const KE_S_F1: c_int = 6;
const KE_S_F2: c_int = 7;
const KE_S_F3: c_int = 8;
const KE_S_F4: c_int = 9;
const KE_S_F5: c_int = 10;
const KE_S_F6: c_int = 11;
const KE_S_F7: c_int = 12;
const KE_S_F8: c_int = 13;
const KE_S_F9: c_int = 14;
const KE_S_F10: c_int = 15;
const KE_S_F11: c_int = 16;
const KE_S_F12: c_int = 17;
const KE_C_LEFT: c_int = 85;
const KE_C_RIGHT: c_int = 86;
const KE_C_HOME: c_int = 87;
const KE_C_END: c_int = 88;

const K_S_UP: c_int = termcap2key(KS_EXTRA, KE_S_UP);
const K_S_DOWN: c_int = termcap2key(KS_EXTRA, KE_S_DOWN);
const K_S_LEFT: c_int = termcap2key(b'#' as c_int, b'4' as c_int);
const K_S_RIGHT: c_int = termcap2key(b'%' as c_int, b'i' as c_int);
const K_S_HOME: c_int = termcap2key(b'#' as c_int, b'2' as c_int);
const K_S_END: c_int = termcap2key(b'*' as c_int, b'7' as c_int);
const K_S_F1: c_int = termcap2key(KS_EXTRA, KE_S_F1);
const K_S_F2: c_int = termcap2key(KS_EXTRA, KE_S_F2);
const K_S_F3: c_int = termcap2key(KS_EXTRA, KE_S_F3);
const K_S_F4: c_int = termcap2key(KS_EXTRA, KE_S_F4);
const K_S_F5: c_int = termcap2key(KS_EXTRA, KE_S_F5);
const K_S_F6: c_int = termcap2key(KS_EXTRA, KE_S_F6);
const K_S_F7: c_int = termcap2key(KS_EXTRA, KE_S_F7);
const K_S_F8: c_int = termcap2key(KS_EXTRA, KE_S_F8);
const K_S_F9: c_int = termcap2key(KS_EXTRA, KE_S_F9);
const K_S_F10: c_int = termcap2key(KS_EXTRA, KE_S_F10);
const K_S_F11: c_int = termcap2key(KS_EXTRA, KE_S_F11);
const K_S_F12: c_int = termcap2key(KS_EXTRA, KE_S_F12);

const K_C_LEFT: c_int = termcap2key(KS_EXTRA, KE_C_LEFT);
const K_C_RIGHT: c_int = termcap2key(KS_EXTRA, KE_C_RIGHT);
const K_C_HOME: c_int = termcap2key(KS_EXTRA, KE_C_HOME);
const K_C_END: c_int = termcap2key(KS_EXTRA, KE_C_END);

// Modifier mask constants (from keycodes.h)
const MOD_MASK_SHIFT: c_int = 0x02;
const MOD_MASK_CTRL: c_int = 0x04;
const MOD_MASK_ALT: c_int = 0x08;

// ASCII constants
const TAB: c_int = 0x09;
const ESC: c_int = 0x1B;
const CTRL_M: c_int = 0x0D;

// =============================================================================
// Key Conversion Result
// =============================================================================

/// Result of converting a Neovim key to `VTerm` key with modifiers.
#[repr(C)]
pub struct VTermKeyResult {
    /// The `VTerm` key code (`VTERM_KEY_*` or `vterm_key_function(n)`).
    /// Returns `VTERM_KEY_NONE` if not a special key (send as character).
    pub key: c_int,
    /// The `VTerm` modifier flags (`VTERM_MOD_*`).
    pub modifiers: c_int,
}

// =============================================================================
// Key Conversion Functions
// =============================================================================

/// Convert Neovim modifier mask to `VTerm` modifier mask.
///
/// This handles the modifier conversion and updates the key if Ctrl is pressed
/// with an uppercase letter (vterm expects lowercase with Ctrl).
const fn convert_modifiers(key: c_int, nvim_mod_mask: c_int) -> (c_int, c_int) {
    let mut vterm_mod: c_int = 0;
    let mut result_key = key;

    if (nvim_mod_mask & MOD_MASK_SHIFT) != 0 {
        vterm_mod |= VTERM_MOD_SHIFT;
    }
    if (nvim_mod_mask & MOD_MASK_CTRL) != 0 {
        vterm_mod |= VTERM_MOD_CTRL;
        // vterm interprets CTRL+A as SHIFT+CTRL, change to CTRL+a
        if (nvim_mod_mask & MOD_MASK_SHIFT) == 0
            && result_key >= b'A' as c_int
            && result_key <= b'Z' as c_int
        {
            result_key += b'a' as c_int - b'A' as c_int;
        }
    }
    if (nvim_mod_mask & MOD_MASK_ALT) != 0 {
        vterm_mod |= VTERM_MOD_ALT;
    }

    (result_key, vterm_mod)
}

/// Convert a Neovim key code to a `VTerm` key code.
///
/// Takes a Neovim key code and converts it to the corresponding `VTerm` key.
/// Also converts modifier keys that are embedded in the key code (like `K_S_UP`).
///
/// # Arguments
/// * `key` - The Neovim key code
/// * `nvim_mod_mask` - The Neovim modifier mask (`MOD_MASK_*`)
///
/// # Returns
/// A `VTermKeyResult` with the `VTerm` key code and modifiers.
/// If key is `VTERM_KEY_NONE`, the key should be sent as a character instead.
#[no_mangle]
#[allow(clippy::too_many_lines)]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_convert_key(key: c_int, nvim_mod_mask: c_int) -> VTermKeyResult {
    let (result_key, mut vterm_mod) = convert_modifiers(key, nvim_mod_mask);

    // Handle keys that have shift/ctrl embedded in the key code
    match result_key {
        K_S_TAB | K_S_UP | K_S_DOWN | K_S_LEFT | K_S_RIGHT | K_S_HOME | K_S_END | K_S_F1
        | K_S_F2 | K_S_F3 | K_S_F4 | K_S_F5 | K_S_F6 | K_S_F7 | K_S_F8 | K_S_F9 | K_S_F10
        | K_S_F11 | K_S_F12 => {
            vterm_mod |= VTERM_MOD_SHIFT;
        }
        K_C_LEFT | K_C_RIGHT | K_C_HOME | K_C_END => {
            vterm_mod |= VTERM_MOD_CTRL;
        }
        _ => {}
    }

    // Convert the key to VTerm key code
    let vterm_key = match result_key {
        K_BS => VTERM_KEY_BACKSPACE,
        K_S_TAB | TAB => VTERM_KEY_TAB,
        CTRL_M => VTERM_KEY_ENTER,
        ESC => VTERM_KEY_ESCAPE,

        K_S_UP | K_UP => VTERM_KEY_UP,
        K_S_DOWN | K_DOWN => VTERM_KEY_DOWN,
        K_S_LEFT | K_C_LEFT | K_LEFT => VTERM_KEY_LEFT,
        K_S_RIGHT | K_C_RIGHT | K_RIGHT => VTERM_KEY_RIGHT,

        K_INS => VTERM_KEY_INS,
        K_DEL => VTERM_KEY_DEL,
        K_S_HOME | K_C_HOME | K_HOME => VTERM_KEY_HOME,
        K_S_END | K_C_END | K_END => VTERM_KEY_END,
        K_PAGEUP => VTERM_KEY_PAGEUP,
        K_PAGEDOWN => VTERM_KEY_PAGEDOWN,

        // Keypad keys
        K_K0 | K_KINS => VTERM_KEY_KP_0,
        K_K1 | K_KEND => VTERM_KEY_KP_1,
        K_K2 | K_KDOWN => VTERM_KEY_KP_2,
        K_K3 | K_KPAGEDOWN => VTERM_KEY_KP_3,
        K_K4 | K_KLEFT => VTERM_KEY_KP_4,
        K_K5 | K_KORIGIN => VTERM_KEY_KP_5,
        K_K6 | K_KRIGHT => VTERM_KEY_KP_6,
        K_K7 | K_KHOME => VTERM_KEY_KP_7,
        K_K8 | K_KUP => VTERM_KEY_KP_8,
        K_K9 | K_KPAGEUP => VTERM_KEY_KP_9,
        K_KDEL | K_KPOINT => VTERM_KEY_KP_PERIOD,
        K_KENTER => VTERM_KEY_KP_ENTER,
        K_KPLUS => VTERM_KEY_KP_PLUS,
        K_KMINUS => VTERM_KEY_KP_MINUS,
        K_KMULTIPLY => VTERM_KEY_KP_MULT,
        K_KDIVIDE => VTERM_KEY_KP_DIVIDE,

        // Function keys F1-F12 (with shift variants)
        K_S_F1 | K_F1 => vterm_key_function(1),
        K_S_F2 | K_F2 => vterm_key_function(2),
        K_S_F3 | K_F3 => vterm_key_function(3),
        K_S_F4 | K_F4 => vterm_key_function(4),
        K_S_F5 | K_F5 => vterm_key_function(5),
        K_S_F6 | K_F6 => vterm_key_function(6),
        K_S_F7 | K_F7 => vterm_key_function(7),
        K_S_F8 | K_F8 => vterm_key_function(8),
        K_S_F9 | K_F9 => vterm_key_function(9),
        K_S_F10 | K_F10 => vterm_key_function(10),
        K_S_F11 | K_F11 => vterm_key_function(11),
        K_S_F12 | K_F12 => vterm_key_function(12),

        // Function keys F13-F63 (no shift variants)
        K_F13 => vterm_key_function(13),
        K_F14 => vterm_key_function(14),
        K_F15 => vterm_key_function(15),
        K_F16 => vterm_key_function(16),
        K_F17 => vterm_key_function(17),
        K_F18 => vterm_key_function(18),
        K_F19 => vterm_key_function(19),
        K_F20 => vterm_key_function(20),
        K_F21 => vterm_key_function(21),
        K_F22 => vterm_key_function(22),
        K_F23 => vterm_key_function(23),
        K_F24 => vterm_key_function(24),
        K_F25 => vterm_key_function(25),
        K_F26 => vterm_key_function(26),
        K_F27 => vterm_key_function(27),
        K_F28 => vterm_key_function(28),
        K_F29 => vterm_key_function(29),
        K_F30 => vterm_key_function(30),
        K_F31 => vterm_key_function(31),
        K_F32 => vterm_key_function(32),
        K_F33 => vterm_key_function(33),
        K_F34 => vterm_key_function(34),
        K_F35 => vterm_key_function(35),
        K_F36 => vterm_key_function(36),
        K_F37 => vterm_key_function(37),
        K_F38 => vterm_key_function(38),
        K_F39 => vterm_key_function(39),
        K_F40 => vterm_key_function(40),
        K_F41 => vterm_key_function(41),
        K_F42 => vterm_key_function(42),
        K_F43 => vterm_key_function(43),
        K_F44 => vterm_key_function(44),
        K_F45 => vterm_key_function(45),
        K_F46 => vterm_key_function(46),
        K_F47 => vterm_key_function(47),
        K_F48 => vterm_key_function(48),
        K_F49 => vterm_key_function(49),
        K_F50 => vterm_key_function(50),
        K_F51 => vterm_key_function(51),
        K_F52 => vterm_key_function(52),
        K_F53 => vterm_key_function(53),
        K_F54 => vterm_key_function(54),
        K_F55 => vterm_key_function(55),
        K_F56 => vterm_key_function(56),
        K_F57 => vterm_key_function(57),
        K_F58 => vterm_key_function(58),
        K_F59 => vterm_key_function(59),
        K_F60 => vterm_key_function(60),
        K_F61 => vterm_key_function(61),
        K_F62 => vterm_key_function(62),
        K_F63 => vterm_key_function(63),

        // Not a special key - return VTERM_KEY_NONE to indicate
        // the key should be sent as a character
        _ => VTERM_KEY_NONE,
    };

    VTermKeyResult {
        key: vterm_key,
        modifiers: vterm_mod,
    }
}

/// Check if a character should be filtered when sending to terminal.
///
/// Some characters like NUL shouldn't be sent to the terminal.
#[no_mangle]
pub extern "C" fn rs_terminal_is_filter_char(c: c_int) -> c_int {
    // Filter out NUL bytes and certain control characters
    c_int::from(c == 0)
}

// =============================================================================
// VTerm Cursor Shape Constants (from vterm.h)
// =============================================================================

/// Block cursor shape.
pub const VTERM_PROP_CURSORSHAPE_BLOCK: c_int = 1;
/// Underline cursor shape.
pub const VTERM_PROP_CURSORSHAPE_UNDERLINE: c_int = 2;
/// Vertical bar cursor shape (left side).
pub const VTERM_PROP_CURSORSHAPE_BAR_LEFT: c_int = 3;

// =============================================================================
// Screen Damage and Invalidation Helpers
// =============================================================================

/// Result of an invalid region calculation.
#[repr(C)]
pub struct InvalidRegion {
    /// Start row of the invalid region.
    pub start_row: c_int,
    /// End row of the invalid region (exclusive).
    pub end_row: c_int,
}

/// Calculate the updated invalid region when damage occurs.
///
/// This computes the union of the current invalid region and the new damage.
/// Pass -1 for both current values to indicate the entire screen is invalid.
///
/// # Arguments
/// * `current_start` - Current invalid start row (-1 for full invalidation)
/// * `current_end` - Current invalid end row (-1 for full invalidation)
/// * `damage_start` - New damage start row
/// * `damage_end` - New damage end row
///
/// # Returns
/// The updated invalid region.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_update_invalid_region(
    current_start: c_int,
    current_end: c_int,
    damage_start: c_int,
    damage_end: c_int,
) -> InvalidRegion {
    // If requesting full invalidation
    if damage_start == -1 && damage_end == -1 {
        return InvalidRegion {
            start_row: current_start,
            end_row: current_end,
        };
    }

    // Compute union of regions
    let start = if current_start == -1 || damage_start < current_start {
        damage_start
    } else {
        current_start
    };

    let end = if current_end == -1 || damage_end > current_end {
        damage_end
    } else {
        current_end
    };

    InvalidRegion {
        start_row: start,
        end_row: end,
    }
}

/// Reset invalid region to indicate no pending damage.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_reset_invalid_region() -> InvalidRegion {
    InvalidRegion {
        start_row: i32::MAX,
        end_row: -1,
    }
}

// =============================================================================
// Resize Calculation Helpers
// =============================================================================

/// Result of terminal resize dimension calculation.
#[repr(C)]
pub struct ResizeDimensions {
    /// Calculated width (0 if no resize needed or invalid).
    pub width: u16,
    /// Calculated height (0 if no resize needed or invalid).
    pub height: u16,
    /// Whether a resize is needed.
    pub needs_resize: c_int,
}

/// Calculate terminal dimensions by taking the maximum of current and new values.
///
/// This is used when determining terminal size across multiple windows.
///
/// # Arguments
/// * `current_width` - Current accumulated width
/// * `current_height` - Current accumulated height
/// * `new_width` - Width of the new window
/// * `new_height` - Height of the new window
///
/// # Returns
/// The maximum dimensions.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_max_dimensions(
    current_width: u16,
    current_height: u16,
    new_width: u16,
    new_height: u16,
) -> ResizeDimensions {
    ResizeDimensions {
        width: current_width.max(new_width),
        height: current_height.max(new_height),
        needs_resize: 0, // Not used in this context
    }
}

/// Check if terminal needs resize based on current and target dimensions.
///
/// # Arguments
/// * `cur_width` - Current terminal width
/// * `cur_height` - Current terminal height
/// * `target_width` - Target width
/// * `target_height` - Target height
///
/// # Returns
/// `ResizeDimensions` with `needs_resize` set to 1 if resize is needed.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_check_resize(
    cur_width: c_int,
    cur_height: c_int,
    target_width: u16,
    target_height: u16,
) -> ResizeDimensions {
    // No resize needed if dimensions match or target is zero
    if target_width == 0
        || target_height == 0
        || (cur_width == c_int::from(target_width) && cur_height == c_int::from(target_height))
    {
        return ResizeDimensions {
            width: 0,
            height: 0,
            needs_resize: 0,
        };
    }

    ResizeDimensions {
        width: target_width,
        height: target_height,
        needs_resize: 1,
    }
}

// =============================================================================
// Scrollback Calculation Helpers
// =============================================================================

/// Maximum scrollback size constant.
pub const TERMINAL_SB_MAX: usize = 100_000;

/// Calculate the effective scrollback size.
///
/// If the provided size is less than 1 (typically -1 for "unlimited"),
/// returns the maximum scrollback size.
///
/// # Arguments
/// * `scrollback` - The requested scrollback size
///
/// # Returns
/// The effective scrollback size.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_effective_scrollback(scrollback: i64) -> usize {
    if scrollback < 1 {
        TERMINAL_SB_MAX
    } else {
        // Safe: we've verified scrollback >= 1 above
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let size = scrollback as usize;
        size
    }
}

/// Calculate how many scrollback lines to delete when reducing scrollback size.
///
/// # Arguments
/// * `current_sb` - Current number of scrollback lines
/// * `new_size` - New scrollback size limit
///
/// # Returns
/// Number of lines to delete (0 if none needed).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_scrollback_lines_to_delete(
    current_sb: usize,
    new_size: usize,
) -> usize {
    current_sb.saturating_sub(new_size)
}

/// Check if scrollback buffer is full and needs to wrap.
///
/// # Arguments
/// * `current_sb` - Current number of scrollback lines
/// * `sb_size` - Maximum scrollback size
///
/// # Returns
/// 1 if scrollback is full, 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_scrollback_is_full(current_sb: usize, sb_size: usize) -> c_int {
    c_int::from(current_sb == sb_size)
}

/// Calculate the buffer index for inserting a scrollback line.
///
/// # Arguments
/// * `line_count` - Total lines in buffer
/// * `height` - Terminal height
///
/// # Returns
/// The buffer index where the scrollback line should be inserted.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_scrollback_insert_index(line_count: c_int, height: c_int) -> c_int {
    line_count - height
}

// =============================================================================
// VTerm Callback Helper Types
// =============================================================================

/// `VTerm` rectangle structure (matches `VTermRect` from libvterm).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VTermRect {
    /// Start row (inclusive).
    pub start_row: c_int,
    /// End row (exclusive).
    pub end_row: c_int,
    /// Start column (inclusive).
    pub start_col: c_int,
    /// End column (exclusive).
    pub end_col: c_int,
}

/// `VTerm` position structure (matches `VTermPos` from libvterm).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct VTermPos {
    /// Row position.
    pub row: c_int,
    /// Column position.
    pub col: c_int,
}

/// `VTerm` property constants (matching `VTermProp` enum in `vterm_defs.h`).
pub const VTERM_PROP_CURSORVISIBLE: c_int = 1;
pub const VTERM_PROP_CURSORBLINK: c_int = 2;
pub const VTERM_PROP_ALTSCREEN: c_int = 3;
pub const VTERM_PROP_TITLE: c_int = 4;
pub const VTERM_PROP_ICONNAME: c_int = 5;
pub const VTERM_PROP_REVERSE: c_int = 6;
pub const VTERM_PROP_CURSORSHAPE: c_int = 7;
pub const VTERM_PROP_MOUSE: c_int = 8;
pub const VTERM_PROP_FOCUSREPORT: c_int = 9;
pub const VTERM_PROP_THEMEUPDATES: c_int = 10;

// =============================================================================
// VTerm Callback Helpers
// =============================================================================

/// Calculate combined damage region from two rectangles.
///
/// Used in `term_moverect` callback to compute the union of source and
/// destination rectangles.
///
/// # Arguments
/// * `dest` - Destination rectangle
/// * `src` - Source rectangle
///
/// # Returns
/// The combined damage region (`start_row`, `end_row`).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_moverect_damage(dest: VTermRect, src: VTermRect) -> InvalidRegion {
    let start = dest.start_row.min(src.start_row);
    let end = dest.end_row.max(src.end_row);
    InvalidRegion {
        start_row: start,
        end_row: end,
    }
}

/// Result of processing a `VTerm` property change.
#[repr(C)]
pub struct VTermPropResult {
    /// Whether the property was handled.
    pub handled: c_int,
    /// Whether the terminal should be invalidated.
    pub invalidate: c_int,
    /// Whether cursor pending flag should be set.
    pub cursor_pending: c_int,
}

/// Check if a `VTerm` property change requires terminal invalidation.
///
/// # Arguments
/// * `prop` - The `VTerm` property that changed
///
/// # Returns
/// A `VTermPropResult` indicating how to handle the property change.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_prop_needs_invalidate(prop: c_int) -> VTermPropResult {
    match prop {
        // Properties that are handled but don't need invalidation
        VTERM_PROP_ALTSCREEN | VTERM_PROP_TITLE | VTERM_PROP_ICONNAME | VTERM_PROP_MOUSE => {
            VTermPropResult {
                handled: 1,
                invalidate: 0,
                cursor_pending: 0,
            }
        }
        // Cursor visibility needs invalidation but no cursor pending
        VTERM_PROP_CURSORVISIBLE => VTermPropResult {
            handled: 1,
            invalidate: 1,
            cursor_pending: 0,
        },
        // Cursor shape/blink needs both invalidation and cursor pending
        VTERM_PROP_CURSORBLINK | VTERM_PROP_CURSORSHAPE => VTermPropResult {
            handled: 1,
            invalidate: 1,
            cursor_pending: 1,
        },
        // Unknown properties
        _ => VTermPropResult {
            handled: 0,
            invalidate: 0,
            cursor_pending: 0,
        },
    }
}

/// Process a cursor move event from `VTerm`.
///
/// # Arguments
/// * `new_row` - New cursor row
/// * `new_col` - New cursor column
/// * `old_row` - Previous cursor row
/// * `old_col` - Previous cursor column
///
/// # Returns
/// 1 to indicate the callback was handled.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_movecursor_handled(
    _new_row: c_int,
    _new_col: c_int,
    _old_row: c_int,
    _old_col: c_int,
) -> c_int {
    // The actual cursor update is done in C by writing to term->cursor.row/col
    // This function just provides a hook point for potential future logic
    1
}

/// Calculate the number of columns to copy when popping scrollback.
///
/// # Arguments
/// * `requested_cols` - Number of columns requested
/// * `available_cols` - Number of columns available in scrollback row
///
/// # Returns
/// The number of columns to actually copy.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_sb_pop_cols(requested_cols: usize, available_cols: usize) -> usize {
    requested_cols.min(available_cols)
}

/// Check if dark theme should be reported to `VTerm`.
///
/// # Arguments
/// * `bg_char` - The background option character ('d' for dark, 'l' for light)
///
/// # Returns
/// 1 if dark theme, 0 if light theme.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
pub extern "C" fn rs_terminal_is_dark_theme(bg_char: u8) -> c_int {
    c_int::from(bg_char == b'd')
}

// =============================================================================
// Row/Line Number Conversion
// =============================================================================

/// Convert a terminal row number to a buffer line number.
///
/// Formula: `linenr = row + sb_current + 1`
///
/// The terminal has a scrollback buffer at the top of the nvim buffer.
/// Row 0 of the terminal is at line `sb_current + 1` in the buffer.
///
/// # Arguments
/// * `row` - Terminal row (0-based, can be negative for scrollback)
/// * `sb_current` - Current scrollback buffer size
///
/// # Returns
/// Buffer line number (1-based).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_terminal_row_to_linenr(row: c_int, sb_current: usize) -> c_int {
    if row == i32::MAX {
        return i32::MAX;
    }
    // Safe cast: sb_current is typically small (max ~100000)
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    let sb = sb_current as c_int;
    row + sb + 1
}

/// Convert a buffer line number to a terminal row number.
///
/// Formula: `row = linenr - sb_current - 1`
///
/// # Arguments
/// * `linenr` - Buffer line number (1-based)
/// * `sb_current` - Current scrollback buffer size
///
/// # Returns
/// Terminal row (0-based, negative for scrollback lines).
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_terminal_linenr_to_row(linenr: c_int, sb_current: usize) -> c_int {
    #[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
    let sb = sb_current as c_int;
    linenr - sb - 1
}

// =============================================================================
// Mouse Button Conversion
// =============================================================================

// Mouse key constants (from keycodes.h)
const K_LEFTMOUSE: c_int = termcap2key(KS_EXTRA, 39);
const K_LEFTDRAG: c_int = termcap2key(KS_EXTRA, 40);
const K_LEFTRELEASE: c_int = termcap2key(KS_EXTRA, 41);
const K_MIDDLEMOUSE: c_int = termcap2key(KS_EXTRA, 42);
const K_MIDDLEDRAG: c_int = termcap2key(KS_EXTRA, 43);
const K_MIDDLERELEASE: c_int = termcap2key(KS_EXTRA, 44);
const K_RIGHTMOUSE: c_int = termcap2key(KS_EXTRA, 45);
const K_RIGHTDRAG: c_int = termcap2key(KS_EXTRA, 46);
const K_RIGHTRELEASE: c_int = termcap2key(KS_EXTRA, 47);
const K_MOUSEDOWN: c_int = termcap2key(KS_EXTRA, 54);
const K_MOUSEUP: c_int = termcap2key(KS_EXTRA, 55);
const K_MOUSELEFT: c_int = termcap2key(KS_EXTRA, 56);
const K_MOUSERIGHT: c_int = termcap2key(KS_EXTRA, 57);
const K_MOUSEMOVE: c_int = termcap2key(KS_EXTRA, 74);
const K_X1MOUSE: c_int = termcap2key(KS_EXTRA, 75);
const K_X1DRAG: c_int = termcap2key(KS_EXTRA, 76);
const K_X1RELEASE: c_int = termcap2key(KS_EXTRA, 77);
const K_X2MOUSE: c_int = termcap2key(KS_EXTRA, 78);
const K_X2DRAG: c_int = termcap2key(KS_EXTRA, 79);
const K_X2RELEASE: c_int = termcap2key(KS_EXTRA, 80);

/// Result of mouse button conversion.
#[repr(C)]
pub struct MouseButtonResult {
    /// Button number (1=left, 2=middle, 3=right, 4=scroll down, 5=scroll up, etc.)
    /// -1 if unknown key
    pub button: c_int,
    /// 1 if pressed/dragging, 0 if released
    pub pressed: c_int,
}

/// Convert a Neovim mouse key code to a `VTerm` button number.
///
/// This handles the conversion of mouse events for forwarding to the terminal.
///
/// # Arguments
/// * `key` - Neovim key code (`K_LEFTMOUSE`, `K_RIGHTDRAG`, etc.)
///
/// # Returns
/// `MouseButtonResult` with button number and pressed state.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_terminal_convert_mouse_button(key: c_int) -> MouseButtonResult {
    match key {
        K_LEFTDRAG | K_LEFTMOUSE => MouseButtonResult {
            button: 1,
            pressed: 1,
        },
        K_LEFTRELEASE => MouseButtonResult {
            button: 1,
            pressed: 0,
        },
        K_MIDDLEDRAG | K_MIDDLEMOUSE => MouseButtonResult {
            button: 2,
            pressed: 1,
        },
        K_MIDDLERELEASE => MouseButtonResult {
            button: 2,
            pressed: 0,
        },
        K_RIGHTDRAG | K_RIGHTMOUSE => MouseButtonResult {
            button: 3,
            pressed: 1,
        },
        K_RIGHTRELEASE => MouseButtonResult {
            button: 3,
            pressed: 0,
        },
        K_X1DRAG | K_X1MOUSE => MouseButtonResult {
            button: 8,
            pressed: 1,
        },
        K_X1RELEASE => MouseButtonResult {
            button: 8,
            pressed: 0,
        },
        K_X2DRAG | K_X2MOUSE => MouseButtonResult {
            button: 9,
            pressed: 1,
        },
        K_X2RELEASE => MouseButtonResult {
            button: 9,
            pressed: 0,
        },
        K_MOUSEDOWN => MouseButtonResult {
            button: 4,
            pressed: 1,
        },
        K_MOUSEUP => MouseButtonResult {
            button: 5,
            pressed: 1,
        },
        K_MOUSELEFT => MouseButtonResult {
            button: 7,
            pressed: 1,
        },
        K_MOUSERIGHT => MouseButtonResult {
            button: 6,
            pressed: 1,
        },
        K_MOUSEMOVE => MouseButtonResult {
            button: 0,
            pressed: 0,
        },
        _ => MouseButtonResult {
            button: -1,
            pressed: 0,
        },
    }
}

// =============================================================================
// Mouse Event Handling
// =============================================================================

/// Process a mouse event while the terminal is focused.
///
/// Returns 1 if the terminal should lose focus, 0 otherwise.
///
/// Replaces `send_mouse_event` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_send_mouse_event(term: TerminalHandle, c: c_int) -> c_int {
    let mut row = unsafe { mouse_row };
    let mut col = unsafe { mouse_col };
    let mut grid = unsafe { mouse_grid };
    let mouse_win = unsafe { nvim_mouse_find_win_inner(&raw mut grid, &raw mut row, &raw mut col) };

    if !mouse_win.is_null() {
        let t = unsafe { term.as_ref() };
        let win_buf = unsafe { win_ref_raw(mouse_win).w_buffer };
        let buf_term = unsafe { bref_raw(win_buf).terminal };
        let winbar_height = unsafe { win_ref_raw(mouse_win).w_winbar_height };
        let win_height = unsafe { win_ref_raw(mouse_win).w_height };
        let win_width = unsafe { win_ref_raw(mouse_win).w_width };
        let offset = unsafe { nvim_win_col_off(mouse_win) };

        if t.forward_mouse
            && buf_term == term.as_ptr()
            && row >= 0
            && (grid > 1 || row + winbar_height < win_height)
            && col >= offset
            && (grid > 1 || col < win_width)
        {
            // Event in terminal window with mouse forwarding enabled.
            let mb = rs_terminal_convert_mouse_button(c);
            if mb.button < 0 {
                return 0;
            }
            let mouse_result = rs_terminal_convert_key(c, unsafe { mod_mask });
            let mouse_mod = mouse_result.modifiers;
            unsafe { rs_vterm_mouse_move(t.vt, row, col - offset, mouse_mod) };
            if mb.button > 0 {
                unsafe { rs_vterm_mouse_button(t.vt, mb.button, mb.pressed, mouse_mod) };
            }
            return 0;
        }

        if c == K_MOUSEUP || c == K_MOUSEDOWN || c == K_MOUSELEFT || c == K_MOUSERIGHT {
            return unsafe { nvim_do_mousescroll_c(term.as_ptr(), mouse_win, c) };
        }
    }

    // end: label logic
    let win_matches = if mouse_win.is_null() {
        false
    } else {
        let win_buf = unsafe { win_ref_raw(mouse_win).w_buffer };
        let buf_term = unsafe { bref_raw(win_buf).terminal };
        buf_term == term.as_ptr()
    };

    if (c == K_LEFTRELEASE && win_matches) || c == K_MOUSEMOVE {
        return 0;
    }

    let vgetc_char = unsafe { nvim_get_vgetc_char() };
    let vgetc_mod = unsafe { c_vgetc_mod_mask };
    let len = unsafe { nvim_ins_char_typebuf_c(vgetc_char, vgetc_mod, true) };
    if unsafe { c_key_typed } {
        unsafe { nvim_ungetchars(len) };
    }
    1
}

// =============================================================================
// Terminal Paste Filter (TPF) Flags
// =============================================================================

/// Filter backspace characters (0x08)
pub const TPF_BS: c_int = 0x001;
/// Filter horizontal tab characters (0x09)
pub const TPF_HT: c_int = 0x002;
/// Filter form feed characters (0x0C)
pub const TPF_FF: c_int = 0x004;
/// Filter escape characters (0x1B)
pub const TPF_ESC: c_int = 0x008;
/// Filter DEL characters (0x7F)
pub const TPF_DEL: c_int = 0x010;
/// Filter C0 control characters (0x01-0x1F, except specific ones)
pub const TPF_C0: c_int = 0x020;
/// Filter C1 control characters (0x80-0x9F)
pub const TPF_C1: c_int = 0x040;

/// Check if a character should be filtered when pasting to terminal.
///
/// This implements the 'termpastefilter' option logic. Certain control
/// characters can be filtered out when pasting to prevent security issues.
///
/// # Arguments
/// * `c` - Character to check
/// * `tpf_flags` - Current 'termpastefilter' flag settings
///
/// # Returns
/// 1 if the character should be filtered, 0 otherwise.
#[no_mangle]
#[allow(clippy::missing_const_for_fn)]
pub extern "C" fn rs_terminal_should_filter_char(c: c_int, tpf_flags: c_int) -> c_int {
    let flag = match c {
        0x08 => TPF_BS,
        0x09 => TPF_HT,
        // 0x0A (LF) and 0x0D (CR) are never filtered
        0x0A | 0x0D => return 0,
        0x0C => TPF_FF,
        0x1B => TPF_ESC,
        0x7F => TPF_DEL,
        _ if c > 0 && c < 0x20 => TPF_C0,
        _ if (0x80..=0x9F).contains(&c) => TPF_C1,
        _ => return 0,
    };
    c_int::from((tpf_flags & flag) != 0)
}

// =============================================================================
// Phase 1: VTerm Callback Implementations (migrated from terminal_shim.c)
// =============================================================================

/// `VTerm` damage callback -- invalidate the damaged rows.
///
/// Replaces `term_damage` in `terminal_shim.c`.
///
/// # Safety
/// `data` must be a valid `Terminal *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_term_damage(rect: VTermRect, data: *mut c_void) -> c_int {
    unsafe { rs_invalidate_terminal(TerminalHandle::from_ptr(data), rect.start_row, rect.end_row) };
    1
}

/// `VTerm` moverect callback -- invalidate the union of source and dest rows.
///
/// Replaces `term_moverect` in `terminal_shim.c`.
///
/// # Safety
/// `data` must be a valid `Terminal *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_term_moverect(
    dest: VTermRect,
    src: VTermRect,
    data: *mut c_void,
) -> c_int {
    let start = dest.start_row.min(src.start_row);
    let end = dest.end_row.max(src.end_row);
    unsafe { rs_invalidate_terminal(TerminalHandle::from_ptr(data), start, end) };
    1
}

/// `VTerm` movecursor callback -- update cursor pos and invalidate.
///
/// Replaces `term_movecursor` in `terminal_shim.c`.
///
/// # Safety
/// `data` must be a valid `Terminal *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_term_movecursor(
    new_pos: VTermPos,
    _old_pos: VTermPos,
    _visible: c_int,
    data: *mut c_void,
) -> c_int {
    let term = unsafe { TerminalHandle::from_ptr(data) };
    if !term.is_null() {
        let t = unsafe { term.as_mut() };
        t.cursor.row = new_pos.row;
        t.cursor.col = new_pos.col;
    }
    unsafe { rs_invalidate_terminal(TerminalHandle::from_ptr(data), -1, -1) };
    1
}

/// `VTerm` bell callback -- matches `int (*bell)(void *user)` signature.
///
/// Replaces the `term_bell` thin wrapper in `terminal_shim.c`.
/// The `_user` parameter is ignored (bell doesn't need terminal context).
#[no_mangle]
pub extern "C" fn rs_term_bell_cb(_user: *mut c_void) -> c_int {
    unsafe { vim_beep(OPT_BO_FLAG_TERM) };
    1
}

/// `VTerm` theme callback -- matches `int (*theme)(bool *dark, void *user)` signature.
///
/// Replaces the `term_theme` thin wrapper in `terminal_shim.c`.
/// The `_user` parameter is ignored.
///
/// # Safety
/// `dark` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_term_theme_cb(dark: *mut bool, _user: *mut c_void) -> c_int {
    if dark.is_null() {
        return 0;
    }
    let bg = unsafe { *c_p_bg };
    unsafe { *dark = bg == 0x64i8 }; // 0x64 == b'd' (dark)
    1
}

/// `VTerm` output callback -- send output data to the terminal process.
///
/// Replaces `term_output_callback` in `terminal_shim.c`.
///
/// # Safety
/// `user_data` must be a valid `Terminal *` pointer.
/// `s` must be a valid pointer to `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_term_output_callback(s: *const i8, len: usize, user_data: *mut c_void) {
    unsafe { rs_terminal_do_send(TerminalHandle::from_ptr(user_data), s, len) };
}

// =============================================================================
// Phase 3: Migrate terminal_send_key and terminal_notify_theme
// =============================================================================

// =============================================================================
// Phase 2: Migrate term_sb_push and term_sb_pop (scrollback callbacks)
// =============================================================================

extern "C" {
    /// Neovim memory allocator (wraps malloc, never returns null).
    fn xmalloc(size: usize) -> *mut c_void;
    /// Neovim memory deallocator.
    fn xfree(ptr: *mut c_void);

    /// Accessor: add terminal to invalidated set without starting timer.
    fn nvim_terminal_set_put(term: *mut c_void);
}

/// `VTerm` scrollback push callback -- store a line going offscreen.
///
/// Replaces `term_sb_push` in `terminal_shim.c`.
/// Called just before a line goes offscreen; stores it in the scrollback buffer.
///
/// # Safety
/// - `cells` must be a valid pointer to `cols` `VTermScreenCell` structs.
/// - `data` must be a valid `Terminal *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_term_sb_push(
    cols: c_int,
    cells: *const c_void,
    data: *mut c_void,
) -> c_int {
    let term = unsafe { TerminalHandle::from_ptr(data) };
    if term.is_null() {
        return 0;
    }
    let t = unsafe { term.as_mut() };

    if t.sb_size == 0 {
        return 0;
    }

    // cols >= 0 because it comes from vterm (number of columns)
    #[allow(clippy::cast_sign_loss)]
    let c = cols as usize;
    let cell_size = VTERM_SCREEN_CELL_SIZE;
    // sb_buffer is *mut *mut c_void (array of ScrollbackLine* pointers)
    let sb_buf = t.sb_buffer;

    let mut sbrow: *mut c_void = std::ptr::null_mut();

    if t.sb_current == t.sb_size {
        // Buffer is full. Recycle the last row if it has the right column count.
        let last = unsafe { *sb_buf.add(t.sb_current - 1) };
        if unsafe { scrollback_line_cols(last) } == c {
            sbrow = last;
        } else {
            unsafe { xfree(last) };
        }
        t.sb_deleted += 1;

        // Shift the buffer right by one to make room at the front.
        // memmove(sb_buffer + 1, sb_buffer, sizeof(*sb_buffer) * (sb_current - 1))
        unsafe {
            std::ptr::copy(sb_buf, sb_buf.add(1), t.sb_current - 1);
        }
    } else if t.sb_current > 0 {
        // Make room at the front.
        // memmove(sb_buffer + 1, sb_buffer, sizeof(*sb_buffer) * sb_current)
        unsafe {
            std::ptr::copy(sb_buf, sb_buf.add(1), t.sb_current);
        }
    }

    if sbrow.is_null() {
        let row_size = scrollback_line_size(c);
        sbrow = unsafe { xmalloc(row_size) };
        // Write cols field (first usize in ScrollbackLine)
        unsafe { sbrow.cast::<usize>().write(c) };
    }

    // sb_buffer[0] = sbrow
    unsafe { *sb_buf = sbrow };

    if t.sb_current < t.sb_size {
        t.sb_current += 1;
    }

    // sb_pending is c_int; check against sb_size (usize)
    #[allow(clippy::cast_sign_loss)]
    if (t.sb_pending as usize) < t.sb_size {
        t.sb_pending += 1;
    }

    // Copy cells into the row: memcpy(sbrow->cells, cells, cell_size * c)
    let dest = unsafe { scrollback_line_cells_mut(sbrow) };
    unsafe { std::ptr::copy_nonoverlapping(cells.cast::<u8>(), dest.cast::<u8>(), cell_size * c) };

    unsafe { nvim_terminal_set_put(data) };

    1
}

/// `VTerm` scrollback pop callback -- restore a line from the scrollback buffer.
///
/// Replaces `term_sb_pop` in `terminal_shim.c`.
/// Called when the screen height increases and a previously-pushed line is restored.
///
/// # Safety
/// - `cells` must be a valid pointer to `cols` `VTermScreenCell` structs.
/// - `data` must be a valid `Terminal *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_term_sb_pop(
    cols: c_int,
    cells: *mut c_void,
    data: *mut c_void,
) -> c_int {
    let term = unsafe { TerminalHandle::from_ptr(data) };
    if term.is_null() {
        return 0;
    }
    let t = unsafe { term.as_mut() };

    if t.sb_current == 0 {
        return 0;
    }

    if t.sb_pending > 0 {
        t.sb_pending -= 1;
    }

    let sb_buf = t.sb_buffer;
    let sbrow = unsafe { *sb_buf };
    t.sb_current -= 1;

    // Shift buffer left: memmove(sb_buffer, sb_buffer + 1, sizeof(*) * sb_current)
    unsafe { std::ptr::copy(sb_buf.add(1), sb_buf, t.sb_current) };

    // cols >= 0 because it comes from vterm
    #[allow(clippy::cast_sign_loss)]
    let c = cols as usize;
    let cell_size = VTERM_SCREEN_CELL_SIZE;
    let sbrow_cols = unsafe { scrollback_line_cols(sbrow) };
    let cols_to_copy = c.min(sbrow_cols);

    // Copy stored cells to the output buffer
    let src = unsafe { scrollback_line_cells(sbrow) };
    unsafe {
        std::ptr::copy_nonoverlapping(
            src.cast::<u8>(),
            cells.cast::<u8>(),
            cell_size * cols_to_copy,
        );
    }

    // Zero-fill any remaining columns
    for col in cols_to_copy..c {
        let cell_ptr = unsafe { cells.cast::<u8>().add(cell_size * col).cast::<c_void>() };
        unsafe { vterm_cell_zero(cell_ptr) };
    }

    unsafe { xfree(sbrow) };

    1
}

// =============================================================================
// Phase 4: Migrate refresh_size
// =============================================================================

/// Handle pending resize for a terminal.
///
/// Replaces `refresh_size` in `terminal_shim.c`.
/// Called from `refresh_terminal` in C.
/// The `_buf` parameter is the associated buffer but unused here.
///
/// # Safety
/// `term` must be a valid `Terminal *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_refresh_size(term: TerminalHandle, _buf: *mut c_void) {
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_mut() };
    if !t.pending.resize || t.closed {
        return;
    }
    t.pending.resize = false;

    let size = unsafe { rs_vterm_get_size(t.vt) };
    t.invalid_start = 0;
    t.invalid_end = size.rows;

    // Call resize_cb: void (*)(uint16_t width, uint16_t height, void *data)
    if !t.opts.resize_cb.is_null() {
        // SAFETY: resize_cb is a valid fn pointer stored by terminal_open.
        let resize_fn: unsafe extern "C" fn(u16, u16, *mut c_void) =
            unsafe { std::mem::transmute(t.opts.resize_cb) };
        // rows/cols from VTermSize are guaranteed >= 0 and fit in u16
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        unsafe {
            resize_fn(size.cols as u16, size.rows as u16, t.opts.data);
        }
    }
}

// K_ZERO = TERMCAP2KEY(KS_ZERO=255, KE_FILLER='X') = -(255 + ('X' << 8)) = -22783
const K_ZERO: c_int = termcap2key(255, b'X' as c_int);
// Ctrl-@ = ASCII NUL (0)
const CTRL_AT: c_int = 0;

/// Send a key to a terminal, handling special key codes.
///
/// Replaces `terminal_send_key` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_send_key_impl(term: TerminalHandle, c: c_int) {
    if term.is_null() {
        return;
    }
    let vt = unsafe { term.as_ref().vt };
    if vt.is_null() {
        return;
    }

    // Convert K_ZERO back to ASCII NUL (Ctrl-@)
    let c = if c == K_ZERO { CTRL_AT } else { c };

    let mods = unsafe { mod_mask };
    let result = rs_terminal_convert_key(c, mods);

    if result.key != VTERM_KEY_NONE {
        unsafe { rs_vterm_keyboard_key(vt, result.key, result.modifiers) };
    } else if c >= 0 {
        // IS_SPECIAL(c) is (c < 0); only send non-special chars as unichar
        // c >= 0 is checked above, so this cast is safe
        #[allow(clippy::cast_sign_loss)]
        unsafe {
            rs_vterm_keyboard_unichar(vt, c as u32, result.modifiers);
        };
    }
}

/// Send a theme notification to a terminal (OSC 997 sequence).
///
/// Replaces `terminal_notify_theme` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_notify_theme_impl(term: TerminalHandle, dark: c_int) {
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_ref() };
    if !t.theme_updates {
        return;
    }
    // Format: ESC [ 997 ; {1|2} n  (max 10 bytes including NUL)
    let ch = if dark != 0 { b'1' } else { b'2' };
    // The sequence is always exactly 9 bytes: \x1b[997;Xn
    let buf: [u8; 9] = [0x1b, b'[', b'9', b'9', b'7', b';', ch, b'n', 0];
    let len = 8usize; // without the NUL terminator
    unsafe { rs_terminal_do_send(term, buf.as_ptr().cast::<i8>(), len) };
}

// =============================================================================
// Phase 5: Migrate terminal_receive, terminal_send, invalidate_terminal,
//          and term_settermprop
// =============================================================================

extern "C" {
    fn nvim_terminal_timer_start();
    fn nvim_terminal_buf_set_title(buf: *mut c_void, title: *const i8, len: usize);
    #[link_name = "xrealloc"]
    fn nvim_term_xrealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    // Phase 6: termrequest / OSC / DCS / APC
    // has_event is declared globally above
    // vterm_state_set_penattr replaces nvim_term_set_osc8_attr
    // VTERM_ATTR_URI=13, VTERM_VALUETYPE_INT=2
    fn vterm_state_set_penattr(
        state: *mut c_void,
        attr: c_int,
        vtype: c_int,
        val: *mut c_void,
    ) -> c_int;
    // Phase 16: term_selection_set / term_clipboard_set migration
    fn nvim_terminal_clipboard_queue(mask: c_long, data: *mut i8);
    fn rs_eval_call_provider(
        provider: *const i8,
        method: *const i8,
        args: *mut c_void,
        discard: bool,
        rettv: *mut c_void,
    );
    fn tv_list_alloc(count_hint: isize) -> *mut c_void;
    fn tv_list_append_allocated_string(l: *mut c_void, val: *mut i8);
    fn tv_list_append_list(l: *mut c_void, val: *mut c_void);
    fn tv_list_append_string(l: *mut c_void, val: *const i8, len: isize);
    // Phase 15: emit_termrequest / schedule_termrequest migration
    fn nvim_terminal_set_vim_var_termrequest(seq: *const i8, seqlen: usize);
    fn nvim_terminal_apply_termrequest_autocmd(
        buf: *mut c_void,
        row: i64,
        col: i64,
        seq: *const i8,
        seqlen: usize,
    );
    fn nvim_terminal_pending_put_termrequest(
        term: *mut c_void,
        fn_ptr: unsafe extern "C" fn(*mut *mut c_void),
        sequence: *mut i8,
        seqlen: usize,
        pending_send: *mut c_void,
        row: isize,
        col: isize,
        sb_deleted: isize,
    );
    fn nvim_terminal_main_put_termrequest(
        fn_ptr: unsafe extern "C" fn(*mut *mut c_void),
        term: *mut c_void,
        sequence: *mut i8,
        seqlen: usize,
        pending_send: *mut c_void,
        row: isize,
        col: isize,
        sb_deleted: isize,
    );
    fn xmemdup(src: *const c_void, len: usize) -> *mut c_void;
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_void;
    // Phase 7: refresh pipeline
    fn rs_buf_valid(buf: *mut c_void) -> c_int;
    // nvim_handle_get_buffer is in buffer_shim.c - same as nvim_terminal_get_buffer
    #[link_name = "nvim_handle_get_buffer"]
    fn nvim_terminal_get_buffer(buf_handle: c_int) -> *mut c_void;
    // ml_append_buf(buf, lnum, line, len, newfile) -- len=0 means NUL-terminated
    #[link_name = "ml_append_buf"]
    fn nvim_ml_append_buf_term(
        buf: *mut c_void,
        lnum: c_int,
        line: *mut i8,
        len: c_int,
        newfile: bool,
    ) -> c_int;
    // ml_replace_buf(buf, lnum, line, copy, noalloc)
    #[link_name = "ml_replace_buf"]
    fn nvim_ml_replace_buf_term(
        buf: *mut c_void,
        lnum: c_int,
        line: *mut i8,
        copy: bool,
        noalloc: bool,
    ) -> c_int;
    // ml_delete_buf(buf, lnum, message)
    #[link_name = "ml_delete_buf"]
    fn nvim_ml_delete_buf_term(buf: *mut c_void, lnum: c_int, message: bool) -> c_int;
    // mark_adjust_buf(buf, line1, line2, amount, amount_after, adjust_folds, mode, op)
    // kMarkAdjustTerm=2, kExtmarkUndo=1
    #[link_name = "mark_adjust_buf"]
    fn nvim_mark_adjust_buf_term(
        buf: *mut c_void,
        line1: c_int,
        line2: c_int,
        amount: c_int,
        amount_after: c_int,
        adjust_folds: bool,
        mode: c_int,
        op: c_int,
    );
    #[link_name = "appended_lines_buf"]
    fn nvim_appended_lines_buf_term(buf: *mut c_void, lnum: c_int, count: c_int);
    #[link_name = "deleted_lines_buf"]
    fn nvim_deleted_lines_buf_term(buf: *mut c_void, lnum: c_int, count: c_int);
    // changed_lines(buf, lnum, col, lnume, xtra, do_buf_event)
    #[link_name = "changed_lines"]
    fn nvim_changed_lines_term(
        buf: *mut c_void,
        lnum: c_int,
        col: c_int,
        lnume: c_int,
        xtra: c_int,
        do_buf_event: bool,
    );
    // For replacing nvim_multiqueue_move_events_term
    fn nvim_get_main_loop() -> *mut c_void;
    #[link_name = "rs_loop_get_events"]
    fn nvim_term_loop_get_events(lp: *mut c_void) -> *mut c_void;
    fn multiqueue_move_events(dest: *mut c_void, src: *mut c_void);
    #[link_name = "ui_busy_start"]
    fn nvim_ui_busy_start();
    #[link_name = "ui_busy_stop"]
    fn nvim_ui_busy_stop();
    #[link_name = "ui_mode_info_set"]
    fn nvim_term_ui_mode_info_set();
    fn nvim_shape_table_set_cursor(blink: c_int, shape: c_int, percentage: c_int);
    fn nvim_terminal_foreach_invalidated(
        fn_ptr: unsafe extern "C" fn(*mut c_void, *mut c_void),
        ctx: *mut c_void,
    );
    // Phase 9: terminal_paste helpers
    #[link_name = "utf_ptr2len"]
    fn nvim_term_utf_ptr2len(s: *const i8) -> c_int;
    #[link_name = "utf_ptr2char"]
    fn nvim_term_utf_ptr2char(s: *const i8) -> c_int;
    // Phase 10: terminal_destroy helpers
    fn nvim_terminal_invalidated_check_del(term: *mut c_void) -> c_int;
    #[link_name = "vterm_free"]
    fn nvim_vterm_free(vt: *mut c_void);
    #[link_name = "multiqueue_free"]
    fn nvim_multiqueue_free(q: *mut c_void);
    // Phase 4 (terminal_shim cleanup): fetch_cell/fetch_row migration
    // vterm_screen_get_cell(screen, pos, cell) - pos is VTermPos by value
    fn vterm_screen_get_cell(
        vts: *mut c_void,
        pos: nvim_vterm::VTermPos,
        cell: *mut c_void,
    ) -> c_int;
    fn rs_vterm_state_convert_color_to_rgb(state: *mut c_void, col: *mut nvim_vterm::VTermColor);
    fn schar_get_adv(buf_out: *mut *mut i8, sc: nvim_vterm::SChar) -> usize;
    fn hl_get_term_attr(attrs: *const HlAttrsLocal) -> c_int;
    fn hl_combine_attr(char_attr: c_int, prim_attr: c_int) -> c_int;
    /// linenr -> row conversion (from buffer.rs)
    #[link_name = "rs_terminal_linenr_to_row_term"]
    fn rs_terminal_linenr_to_row_term_ffi(term: *mut c_void, linenr: c_int) -> c_int;
}

/// Fetch a single cell from either scrollback buffer (`row < 0`) or live `VTerm` screen (`row >= 0`).
///
/// Returns 1 if the cell was found, 0 if out of bounds (cell is zeroed in that case).
/// Replaces `fetch_cell` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *`, `cell` must be a valid `*mut VTermScreenCell`.
#[no_mangle]
pub unsafe extern "C" fn rs_fetch_cell(
    term: TerminalHandle,
    row: c_int,
    col: c_int,
    cell: *mut c_void,
) -> c_int {
    let t = unsafe { term.as_ref() };
    if row < 0 {
        // row < 0 means scrollback; -row-1 is the scrollback index.
        #[allow(clippy::cast_sign_loss)]
        let sb_idx = (-row - 1) as usize;
        let sbrow = unsafe { *t.sb_buffer.add(sb_idx) };
        if sbrow.is_null() {
            return 0;
        }
        let cols = unsafe { scrollback_line_cols(sbrow.cast()) };
        #[allow(clippy::cast_sign_loss)]
        let col_idx = col as usize;
        if col_idx < cols {
            let cells_ptr = unsafe { scrollback_line_cells(sbrow.cast()) };
            let cell_size = VTERM_SCREEN_CELL_SIZE;
            let src = unsafe { cells_ptr.cast::<u8>().add(col_idx * cell_size) };
            unsafe { std::ptr::copy_nonoverlapping(src, cell.cast::<u8>(), cell_size) };
        } else {
            // Out of bounds: write empty cell
            unsafe { vterm_cell_zero(cell) };
            return 0;
        }
    } else {
        unsafe { vterm_screen_get_cell(t.vts, nvim_vterm::VTermPos::new(row, col), cell) };
    }
    1
}

/// Fill `term->textbuf` with the UTF-8 text of a screen row from col 0 to `end_col`.
///
/// Replaces `fetch_row` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *`.
#[no_mangle]
pub unsafe extern "C" fn rs_fetch_row(term: TerminalHandle, row: c_int, end_col: c_int) {
    let t = unsafe { term.as_mut() };
    let textbuf_ptr = t.textbuf.as_mut_ptr();
    let mut col: c_int = 0;
    let mut line_len: usize = 0;
    let mut ptr: *mut i8 = textbuf_ptr;

    while col < end_col {
        let mut cell = nvim_vterm::VTermScreenCell::default();
        unsafe { rs_fetch_cell(term, row, col, (&raw mut cell).cast()) };
        if cell.schar != 0 {
            unsafe { schar_get_adv(&raw mut ptr, cell.schar) };
            // SAFETY: ptr always stays within textbuf bounds (caller guarantees end_col fits)
            #[allow(clippy::cast_sign_loss)]
            {
                line_len = unsafe { ptr.offset_from(textbuf_ptr) } as usize;
            }
        } else {
            #[allow(clippy::cast_possible_wrap)]
            let space = b' ' as i8;
            unsafe { *ptr = space };
            ptr = unsafe { ptr.add(1) };
        }
        col += c_int::from(cell.width);
    }

    // NUL-terminate at the end of the last non-space character
    unsafe { *textbuf_ptr.add(line_len) = 0 };
}

/// Receive data from PTY, optionally inserting `\r` before bare `\n` chars.
///
/// Replaces `terminal_receive` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *`, `data` must point to `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_receive_impl(
    term: TerminalHandle,
    data: *const i8,
    len: usize,
) {
    if data.is_null() {
        return;
    }
    let t = unsafe { term.as_ref() };
    if t.opts.force_crlf {
        let slice = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), len) };
        let mut buf: Vec<u8> = Vec::with_capacity(len + 16);
        let mut prev = b' ';
        for &c in slice {
            if c == b'\n' && prev != b'\r' {
                buf.push(b'\r');
            }
            buf.push(c);
            prev = c;
        }
        unsafe { rs_vterm_input_write(t.vt, buf.as_ptr().cast::<i8>(), buf.len()) };
    } else {
        unsafe { rs_vterm_input_write(t.vt, data, len) };
    }
    unsafe { rs_vterm_screen_flush_damage(t.vts) };
}

/// Send data to terminal process, buffering if a pending `TermRequest` is active.
///
/// Replaces `terminal_send` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *`, `data` must point to `size` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_do_send(term: TerminalHandle, data: *const i8, size: usize) {
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_ref() };
    if t.closed {
        return;
    }
    let pending_send = unsafe { term.as_ref().pending.send };
    if !pending_send.is_null() {
        unsafe { sb_concat_len(pending_send, data, size) };
        return;
    }
    // Call write_cb via transmute (fn ptr stored as *mut c_void in opts)
    let write_cb: unsafe extern "C" fn(*const i8, usize, *mut c_void) =
        unsafe { std::mem::transmute(term.as_ref().opts.write_cb) };
    unsafe { write_cb(data, size, term.as_ref().opts.data) };
}

/// `HlAttrs` layout (matches `highlight_defs.h`). Local copy to avoid cross-crate dependency.
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct HlAttrsLocal {
    rgb_ae_attr: i16,
    cterm_ae_attr: i16,
    rgb_fg_color: i32,
    rgb_bg_color: i32,
    rgb_sp_color: i32,
    cterm_fg_color: i16,
    cterm_bg_color: i16,
    hl_blend: i32,
    url: i32,
}

/// Nvim API `String` type (matches C `typedef struct { char *data; size_t size; } String`).
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NvimStr {
    /// Pointer to string data (not necessarily NUL-terminated)
    pub data: *const i8,
    /// Length in bytes
    pub size: usize,
}

/// Paste yank array into the terminal, filtering out characters in `tpf_flags`.
///
/// Replaces `terminal_paste` in `terminal_shim.c`.
///
/// # Safety
/// `y_array` must point to `y_size` valid `NvimStr` values.
/// The current buffer must have a terminal (`curbuf->terminal != NULL`).
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_paste(count: c_int, y_array: *const NvimStr, y_size: usize) {
    if y_size == 0 {
        return;
    }
    let term = unsafe { bref_raw(curbuf).terminal };
    let vt = unsafe { TerminalHandle::from_ptr(term).as_ref().vt };
    #[allow(clippy::cast_possible_wrap)]
    let tpf = unsafe { c_tpf_flags } as c_int;

    unsafe { rs_vterm_keyboard_start_paste(vt) };

    let items = unsafe { std::slice::from_raw_parts(y_array, y_size) };
    // Allocate an initial work buffer.
    let mut buff: Vec<u8> = Vec::with_capacity(items[0].size.max(16));

    for _ in 0..count {
        for (j, item) in items.iter().enumerate() {
            if j > 0 {
                // Terminate the previous line.
                #[cfg(target_os = "windows")]
                unsafe {
                    rs_terminal_do_send(TerminalHandle::from_ptr(term), b"\r\n".as_ptr().cast(), 2);
                };
                #[cfg(not(target_os = "windows"))]
                unsafe {
                    rs_terminal_do_send(TerminalHandle::from_ptr(term), b"\n".as_ptr().cast(), 1);
                };
            }
            let src_len = item.size;
            buff.clear();
            if src_len == 0 {
                continue;
            }
            let mut src = item.data;
            let end = unsafe { src.add(src_len) };
            while src < end {
                // NUL byte terminates
                if unsafe { *src } == 0 {
                    break;
                }
                #[allow(clippy::cast_sign_loss)]
                let char_len = unsafe { nvim_term_utf_ptr2len(src) } as usize;
                let c = unsafe { nvim_term_utf_ptr2char(src) };
                if crate::pty::rs_terminal_is_filter_char_flags(c, tpf) == 0 {
                    let bytes = unsafe { std::slice::from_raw_parts(src.cast::<u8>(), char_len) };
                    buff.extend_from_slice(bytes);
                }
                src = unsafe { src.add(char_len) };
            }
            if !buff.is_empty() {
                unsafe {
                    rs_terminal_do_send(
                        TerminalHandle::from_ptr(term),
                        buff.as_ptr().cast(),
                        buff.len(),
                    );
                };
            }
        }
    }

    unsafe { rs_vterm_keyboard_end_paste(vt) };
}

/// Destroy a Terminal instance, freeing all associated memory.
///
/// Replaces `terminal_destroy` in `terminal_shim.c`.
///
/// # Safety
/// `termpp` must be a valid `Terminal **`. After return, `*termpp` is NULL if freed.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_destroy(termpp: *mut *mut c_void) {
    if termpp.is_null() {
        return;
    }
    let term_ptr = unsafe { *termpp };
    if term_ptr.is_null() {
        return;
    }
    let term = unsafe { TerminalHandle::from_ptr(term_ptr) };
    let t = unsafe { term.as_mut() };

    let buf = unsafe { nvim_terminal_get_buffer(t.buf_handle) };
    if !buf.is_null() {
        t.buf_handle = 0;
        unsafe { (*buf.cast::<BufStruct>()).terminal = std::ptr::null_mut() };
    }

    if t.refcount == 0 {
        // Flush pending changes if this terminal was in the invalidated set.
        unsafe { nvim_terminal_invalidated_check_del(term_ptr) };

        // Free scrollback lines
        for i in 0..t.sb_current {
            let sbrow = unsafe { *t.sb_buffer.add(i) };
            unsafe { xfree(sbrow) };
        }
        unsafe { xfree(t.sb_buffer.cast::<c_void>()) };
        unsafe { xfree(t.title.cast::<c_void>()) };
        unsafe { xfree(t.selection_buffer.cast::<c_void>()) };
        unsafe {
            sb_destroy((&raw mut t.selection).cast::<c_void>());
        };
        unsafe {
            sb_destroy((&raw mut t.termrequest_buffer).cast::<c_void>());
        };
        unsafe { nvim_vterm_free(t.vt) };
        unsafe { nvim_multiqueue_free(t.pending.events) };
        unsafe { xfree(term_ptr) };
        unsafe { *termpp = std::ptr::null_mut() };
    }
}

/// Maximum number of terminal columns for `terminal_get_line_attributes`.
const TERM_ATTRS_MAX: usize = 1024;

/// Convert a `VTermColor` to a packed RGB integer (same as C `get_rgb`).
///
/// # Safety
/// `state` must be a valid `VTermState *`.
#[inline]
unsafe fn get_rgb_impl(state: *mut c_void, mut color: nvim_vterm::VTermColor) -> c_int {
    unsafe { rs_vterm_state_convert_color_to_rgb(state, &raw mut color) };
    let rgb = unsafe { color.rgb };
    (c_int::from(rgb.red) << 16) | (c_int::from(rgb.green) << 8) | c_int::from(rgb.blue)
}

/// Compute highlight attributes for each cell in a terminal line.
///
/// Replaces `terminal_get_line_attributes` in `terminal_shim.c`.
///
/// # Safety
/// `term`, `wp` (unused), and `term_attrs` must all be valid. `term_attrs` must point to
/// at least `TERM_ATTRS_MAX` ints.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_get_line_attributes(
    term: TerminalHandle,
    _wp: *mut c_void,
    linenr: c_int,
    term_attrs: *mut c_int,
) {
    // HL_* attribute bit constants (from highlight_defs.h)
    const HL_INVERSE: i16 = 0x01;
    const HL_BOLD: i16 = 0x02;
    const HL_ITALIC: i16 = 0x04;
    const HL_STRIKETHROUGH: i16 = 0x0080;
    const HL_FG_INDEXED: i16 = 0x1000;
    const HL_BG_INDEXED: i16 = 0x0800;
    use nvim_vterm::VTermScreenCell;
    let t = unsafe { term.as_ref() };
    let size = unsafe { rs_vterm_get_size(t.vt) };
    let height = size.rows;
    let width = size.cols;
    let state = unsafe { vterm_obtain_state(t.vt) };

    if linenr == 0 {
        return;
    }
    let row = unsafe { rs_terminal_linenr_to_row_term_ffi(term.as_ptr(), linenr) };
    if row >= height {
        // Terminal height decreased but not yet reflected in buffer
        return;
    }

    let col_limit = width.min(c_int::try_from(TERM_ATTRS_MAX).unwrap_or(c_int::MAX));

    for col in 0..col_limit {
        let mut cell = VTermScreenCell::default();
        let color_valid = unsafe { rs_fetch_cell(term, row, col, (&raw mut cell).cast()) } != 0;

        let fg_default = !color_valid || cell.fg.is_default_fg();
        let bg_default = !color_valid || cell.bg.is_default_bg();

        let foreground_color = if fg_default {
            -1
        } else {
            unsafe { get_rgb_impl(state, cell.fg) }
        };
        let background_color = if bg_default {
            -1
        } else {
            unsafe { get_rgb_impl(state, cell.bg) }
        };

        let fg_indexed = cell.fg.is_indexed();
        let bg_indexed = cell.bg.is_indexed();

        // +1: nvim uses 1-based color indices (0 = no color)
        let fg_idx: i16 = if !fg_default && fg_indexed {
            i16::from(unsafe { cell.fg.indexed.idx }) + 1
        } else {
            0
        };
        let bg_idx: i16 = if !bg_default && bg_indexed {
            i16::from(unsafe { cell.bg.indexed.idx }) + 1
        } else {
            0
        };

        #[allow(clippy::cast_sign_loss)]
        let fg_set = fg_idx > 0 && fg_idx <= 16 && t.color_set[(fg_idx - 1) as usize];
        #[allow(clippy::cast_sign_loss)]
        let bg_set = bg_idx > 0 && bg_idx <= 16 && t.color_set[(bg_idx - 1) as usize];

        #[allow(clippy::cast_possible_truncation)]
        let underline_flag = rs_terminal_underline_hl_flag(cell.attrs) as i16;
        let hl_attrs: i16 = if cell.attrs.bold() { HL_BOLD } else { 0 }
            | if cell.attrs.italic() { HL_ITALIC } else { 0 }
            | if cell.attrs.reverse() { HL_INVERSE } else { 0 }
            | underline_flag
            | if cell.attrs.strike() {
                HL_STRIKETHROUGH
            } else {
                0
            }
            | if fg_indexed && !fg_set {
                HL_FG_INDEXED
            } else {
                0
            }
            | if bg_indexed && !bg_set {
                HL_BG_INDEXED
            } else {
                0
            };

        let attr_id: c_int = if hl_attrs != 0 || !fg_default || !bg_default {
            let attrs = HlAttrsLocal {
                rgb_ae_attr: hl_attrs,
                cterm_ae_attr: hl_attrs,
                rgb_fg_color: foreground_color,
                rgb_bg_color: background_color,
                rgb_sp_color: -1,
                cterm_fg_color: fg_idx,
                cterm_bg_color: bg_idx,
                hl_blend: -1,
                url: -1,
            };
            unsafe { hl_get_term_attr(&raw const attrs) }
        } else {
            0
        };

        let attr_id = if cell.uri > 0 {
            unsafe { hl_combine_attr(attr_id, cell.uri) }
        } else {
            attr_id
        };

        #[allow(clippy::cast_sign_loss)]
        unsafe {
            *term_attrs.add(col as usize) = attr_id;
        };
    }
}

/// Queue a terminal instance for refresh (starts the refresh timer if needed).
///
/// Replaces `invalidate_terminal` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *`.
#[no_mangle]
pub unsafe extern "C" fn rs_invalidate_terminal(
    term: TerminalHandle,
    start_row: c_int,
    end_row: c_int,
) {
    if term.is_null() {
        return;
    }
    if start_row != -1 && end_row != -1 {
        let t = unsafe { term.as_mut() };
        t.invalid_start = t.invalid_start.min(start_row);
        t.invalid_end = t.invalid_end.max(end_row);
    }
    unsafe { nvim_terminal_set_put(term.as_ptr()) };
    if !unsafe { refresh_pending } {
        unsafe { nvim_terminal_timer_start() };
        unsafe { refresh_pending = true };
    }
}

/// Handle a `VTerm` property change (settermprop callback).
///
/// Replaces `term_settermprop` in `terminal_shim.c`.
///
/// # Safety
/// `data` must be a valid `Terminal *`, `val` must be a valid `VTermValue *`.
#[no_mangle]
pub unsafe extern "C" fn rs_term_settermprop(
    prop: c_int,
    val: *const c_void,
    data: *mut c_void,
) -> c_int {
    let term = unsafe { TerminalHandle::from_ptr(data) };
    if term.is_null() || val.is_null() {
        return 0;
    }
    let t = unsafe { term.as_mut() };

    let prop_val = unsafe { &*val.cast::<VTermValue>() };
    match prop {
        VTERM_PROP_ALTSCREEN => {} // no-op

        VTERM_PROP_CURSORVISIBLE => {
            t.cursor.visible = unsafe { prop_val.boolean } != 0;
            unsafe { rs_invalidate_terminal(term, -1, -1) };
        }

        VTERM_PROP_TITLE => {
            let buf = unsafe { nvim_terminal_get_buffer(t.buf_handle) };
            let frag: &VTermStringFragment = unsafe { &prop_val.string };
            let frag_str = frag.str_ptr;
            let frag_len = frag.len();
            let is_initial = frag.is_initial();
            let is_final = frag.is_final();

            if is_initial && is_final {
                if !buf.is_null() {
                    unsafe { nvim_terminal_buf_set_title(buf, frag_str, frag_len) };
                }
            } else {
                if is_initial {
                    t.title_len = 0;
                    t.title_size = frag_len.max(1024);
                    t.title = unsafe { xmalloc(t.title_size).cast::<i8>() };
                } else if t.title_len + frag_len > t.title_size {
                    t.title_size *= 2;
                    t.title = unsafe {
                        nvim_term_xrealloc(t.title.cast::<c_void>(), t.title_size).cast::<i8>()
                    };
                }
                if !t.title.is_null() && !frag_str.is_null() {
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            frag_str.cast::<u8>(),
                            t.title.add(t.title_len).cast::<u8>(),
                            frag_len,
                        );
                    }
                    t.title_len += frag_len;
                }
                if is_final {
                    if !buf.is_null() {
                        unsafe { nvim_terminal_buf_set_title(buf, t.title, t.title_len) };
                    }
                    unsafe { xfree(t.title.cast::<c_void>()) };
                    t.title = std::ptr::null_mut();
                }
            }
        }

        VTERM_PROP_MOUSE => {
            t.forward_mouse = unsafe { prop_val.number } != 0;
        }

        VTERM_PROP_CURSORBLINK => {
            t.cursor.blink = unsafe { prop_val.boolean } != 0;
            t.pending.cursor = true;
            unsafe { rs_invalidate_terminal(term, -1, -1) };
        }

        VTERM_PROP_CURSORSHAPE => {
            t.cursor.shape = unsafe { prop_val.number };
            t.pending.cursor = true;
            unsafe { rs_invalidate_terminal(term, -1, -1) };
        }

        VTERM_PROP_THEMEUPDATES => {
            t.theme_updates = unsafe { prop_val.boolean } != 0;
        }

        _ => return 0,
    }

    1
}

// =============================================================================
// Phase 6: Migrate on_osc, on_dcs, on_apc (VTerm fallback callbacks)
// =============================================================================

/// `VTerm` OSC (Operating System Command) fallback callback.
///
/// Replaces `on_osc` in `terminal_shim.c`.
///
/// # Safety
/// `user` must be a valid `Terminal *`, `str` must point to `len` bytes (or be null).
#[no_mangle]
pub unsafe extern "C" fn rs_on_osc(
    command: c_int,
    str_ptr: *const i8,
    len: usize,
    initial: c_int,
    is_final: c_int,
    user: *mut c_void,
) -> c_int {
    if str_ptr.is_null() || len == 0 {
        return 0;
    }
    if command != 8 && unsafe { has_event(119) } == 0 {
        return 1;
    }
    let term = unsafe { TerminalHandle::from_ptr(user) };
    if term.is_null() {
        return 0;
    }
    let t = unsafe { term.as_mut() };
    let treq_buf = (&raw mut t.termrequest_buffer).cast::<c_void>();

    if initial != 0 {
        unsafe { sb_mut(treq_buf).size = 0 };
        // kv_printf(treqbuf, "\x1b]%d;", command) - write OSC prefix
        let osc_prefix = format!("\x1b]{command};");
        unsafe { sb_concat_len(treq_buf, osc_prefix.as_ptr().cast(), osc_prefix.len()) };
    }
    unsafe { sb_concat_len(treq_buf, str_ptr, len) };

    if is_final != 0 {
        if unsafe { has_event(119) } != 0 {
            unsafe { rs_schedule_termrequest(user) };
        }
        if command == 8 {
            unsafe { sb_push_char(treq_buf, 0) };
            // Offset past "\x1b]8;" (4 bytes)
            let osc8_start = unsafe { t.termrequest_buffer.items.add(4) };
            let mut attr: c_int = 0;
            if unsafe { rs_terminal_parse_osc8(osc8_start, &raw mut attr) } != 0 {
                // vterm_state_set_penattr(state, VTERM_ATTR_URI=13, VTERM_VALUETYPE_INT=2, &val)
                let mut v = nvim_vterm::VTermValue { number: attr };
                let state = unsafe { vterm_obtain_state(t.vt) };
                unsafe { vterm_state_set_penattr(state, 13, 2, std::ptr::addr_of_mut!(v).cast()) };
            }
        }
    }
    1
}

/// `VTerm` DCS (Device Control String) fallback callback.
///
/// Replaces `on_dcs` in `terminal_shim.c`.
///
/// # Safety
/// `user` must be a valid `Terminal *`.
#[no_mangle]
pub unsafe extern "C" fn rs_on_dcs(
    command: *const i8,
    commandlen: usize,
    str_ptr: *const i8,
    len: usize,
    initial: c_int,
    is_final: c_int,
    user: *mut c_void,
) -> c_int {
    if command.is_null() || str_ptr.is_null() {
        return 0;
    }
    if unsafe { has_event(119) } == 0 {
        return 1;
    }
    let term = unsafe { TerminalHandle::from_ptr(user) };
    if term.is_null() {
        return 0;
    }
    let t = unsafe { term.as_mut() };
    let treq_buf = (&raw mut t.termrequest_buffer).cast::<c_void>();

    if initial != 0 {
        unsafe { sb_mut(treq_buf).size = 0 };
        // kv_printf(treqbuf, "\x1bP%*s", cmdlen, command) -- write DCS prefix + command bytes
        unsafe { sb_concat_len(treq_buf, b"\x1bP".as_ptr().cast(), 2) };
        unsafe { sb_concat_len(treq_buf, command, commandlen) };
    }
    unsafe { sb_concat_len(treq_buf, str_ptr, len) };
    if is_final != 0 {
        unsafe { rs_schedule_termrequest(user) };
    }
    1
}

/// `VTerm` APC (Application Program Command) fallback callback.
///
/// Replaces `on_apc` in `terminal_shim.c`.
///
/// # Safety
/// `user` must be a valid `Terminal *`.
#[no_mangle]
pub unsafe extern "C" fn rs_on_apc(
    str_ptr: *const i8,
    len: usize,
    initial: c_int,
    is_final: c_int,
    user: *mut c_void,
) -> c_int {
    if str_ptr.is_null() || len == 0 {
        return 0;
    }
    if unsafe { has_event(119) } == 0 {
        return 1;
    }
    let term = unsafe { TerminalHandle::from_ptr(user) };
    if term.is_null() {
        return 0;
    }
    let t = unsafe { term.as_mut() };
    let treq_buf = (&raw mut t.termrequest_buffer).cast::<c_void>();

    if initial != 0 {
        unsafe { sb_mut(treq_buf).size = 0 };
        // kv_printf(treqbuf, "\x1b_") -- write APC prefix
        unsafe { sb_concat_len(treq_buf, b"\x1b_".as_ptr().cast(), 2) };
    }
    unsafe { sb_concat_len(treq_buf, str_ptr, len) };
    if is_final != 0 {
        unsafe { rs_schedule_termrequest(user) };
    }
    1
}

// =============================================================================
// Phase 16: Migrate term_selection_set / term_clipboard_set
// =============================================================================

/// Set clipboard from terminal selection data.
///
/// Replaces `term_clipboard_set` in `terminal_shim.c`.
/// Called as a `void **argv` multiqueue callback.
///
/// # Safety
/// `argv` must be a valid pointer to at least 2 elements.
#[no_mangle]
pub unsafe extern "C" fn rs_term_clipboard_set(argv: *mut *mut c_void) {
    let mask = unsafe { *argv } as c_long;
    let data = unsafe { (*argv.add(1)).cast::<i8>() };

    // VTERM_SELECTION_CLIPBOARD = 1, VTERM_SELECTION_PRIMARY = 2
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let is_primary = (mask as u32) == 2;
    let regname = if is_primary { b'*' } else { b'+' };

    let lines = unsafe { tv_list_alloc(1) };
    unsafe { tv_list_append_allocated_string(lines, data) };

    let argv_list = unsafe { tv_list_alloc(3) };
    unsafe { tv_list_append_list(argv_list, lines) };

    let regtype = b'v';
    unsafe {
        tv_list_append_string(argv_list, std::ptr::addr_of!(regtype).cast::<i8>(), 1);
        tv_list_append_string(argv_list, std::ptr::addr_of!(regname).cast::<i8>(), 1);
    }

    let mut rettv = std::mem::MaybeUninit::<c_void>::uninit();
    unsafe {
        rs_eval_call_provider(
            c"clipboard".as_ptr(),
            c"set".as_ptr(),
            argv_list,
            true,
            rettv.as_mut_ptr(),
        );
    }
}

/// Accumulate `VTerm` selection data and queue clipboard set when final.
///
/// Replaces `term_selection_set` in `terminal_shim.c`.
///
/// # Safety
/// `user` must be a valid `Terminal` pointer, `str_ptr` must point to `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_term_selection_set(
    mask: c_int,
    str_ptr: *const i8,
    len: usize,
    initial: c_int,
    is_final: c_int,
    user: *mut c_void,
) -> c_int {
    let term = unsafe { TerminalHandle::from_ptr(user) };
    let t = unsafe { term.as_mut() };

    if initial != 0 {
        t.selection.size = 0;
    }

    if len > 0 {
        unsafe { sb_concat_len((&raw mut t.selection).cast(), str_ptr, len) };
    }

    if is_final != 0 {
        // Equivalent to: kv_size(t->selection) + xmemdupz(t->selection.items, len)
        let data_len = t.selection.size;
        let data = unsafe { xmemdupz(t.selection.items.cast::<c_void>(), data_len).cast::<i8>() };
        unsafe { nvim_terminal_clipboard_queue(c_long::from(mask), data) };
    }

    1
}

// =============================================================================
// Phase 15: Migrate emit_termrequest / schedule_termrequest
// =============================================================================

/// Process a `TermRequest` autocmd event.
///
/// Replaces `emit_termrequest` in `terminal_shim.c`.
/// This is called as a `void **argv` callback via `multiqueue_put`.
///
/// # Safety
/// `argv` must be a valid pointer to at least 7 elements.
#[no_mangle]
pub unsafe extern "C" fn rs_emit_termrequest(argv: *mut *mut c_void) {
    let term_ptr = unsafe { *argv };
    let sequence = unsafe { (*argv.add(1)).cast::<i8>() };
    let sequence_length = unsafe { *argv.add(2) } as usize;
    let pending_send = unsafe { *argv.add(3) };
    let row = unsafe { *argv.add(4) } as isize;
    let col = unsafe { *argv.add(5) } as isize;
    let sb_deleted = unsafe { *argv.add(6) } as usize;

    let term = unsafe { TerminalHandle::from_ptr(term_ptr) };
    let t = unsafe { term.as_mut() };

    if t.sb_pending > 0 {
        // Pending scrollback: re-queue onto pending.events.
        #[allow(clippy::cast_possible_wrap)]
        unsafe {
            nvim_terminal_pending_put_termrequest(
                term_ptr,
                rs_emit_termrequest,
                sequence,
                sequence_length,
                pending_send,
                row,
                col,
                sb_deleted as isize,
            );
        }
        return;
    }

    unsafe { nvim_terminal_set_vim_var_termrequest(sequence, sequence_length) };

    let buf = unsafe { nvim_terminal_get_buffer(t.buf_handle) };
    #[allow(clippy::cast_possible_wrap)]
    let row_adj = row - (t.sb_deleted as isize - sb_deleted as isize);
    unsafe {
        nvim_terminal_apply_termrequest_autocmd(
            buf,
            row_adj as i64,
            col as i64,
            sequence,
            sequence_length,
        );
    }
    unsafe { xfree(sequence.cast::<c_void>()) };

    let term_pending_send = t.pending.send;
    t.pending.send = std::ptr::null_mut();
    let sb_size = unsafe { sb_ref(pending_send).size };
    if sb_size > 0 {
        let sb_items = unsafe { sb_ref(pending_send).items };
        unsafe { rs_terminal_do_send(term, sb_items, sb_size) };
        unsafe { sb_destroy(pending_send) };
    }
    if term_pending_send != pending_send {
        t.pending.send = term_pending_send;
    }
    unsafe { xfree(pending_send) };
}

/// Schedule a `TermRequest` event to be emitted after the next terminal refresh.
///
/// Replaces `schedule_termrequest` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *`.
#[no_mangle]
pub unsafe extern "C" fn rs_schedule_termrequest(term_ptr: *mut c_void) {
    let term = unsafe { TerminalHandle::from_ptr(term_ptr) };
    let t = unsafe { term.as_mut() };

    t.pending.send = unsafe { sb_alloc_init() }.cast();

    let line = rs_terminal_row_to_linenr(t.cursor.row, t.sb_current);
    let seq_data = t.termrequest_buffer.items;
    let seq_len = t.termrequest_buffer.size;
    let sequence = unsafe { xmemdup(seq_data.cast::<c_void>(), seq_len) }.cast::<i8>();

    #[allow(clippy::cast_possible_wrap)]
    unsafe {
        nvim_terminal_main_put_termrequest(
            rs_emit_termrequest,
            term_ptr,
            sequence,
            seq_len,
            t.pending.send.cast(),
            line as isize,
            t.cursor.col as isize,
            t.sb_deleted as isize,
        );
    }
}

// =============================================================================
// Phase 7: Migrate refresh pipeline functions
// =============================================================================

// Cursor shape constants (from cursor_shape.h)
const SHAPE_BLOCK: c_int = 0;
const SHAPE_HOR: c_int = 1;
const SHAPE_VER: c_int = 2;

/// Sync the visible terminal rows with the nvim buffer.
///
/// Replaces `refresh_screen` in `terminal_shim.c`.
///
/// # Safety
/// `term` and `buf` must be valid pointers.
unsafe fn rs_refresh_screen(term: TerminalHandle, buf: *mut c_void) {
    let t = unsafe { term.as_mut() };
    let size = unsafe { rs_vterm_get_size(t.vt) };
    let height = size.rows;
    let width = size.cols;

    // Terminal height may have decreased before invalid_end reflects it
    t.invalid_end = t.invalid_end.min(height);

    if t.invalid_start >= t.invalid_end {
        t.invalid_start = c_int::MAX;
        t.invalid_end = -1;
        return;
    }

    let mut changed = 0;
    let mut added = 0;
    let ml_line_count = unsafe { bref_raw(buf).ml_line_count };
    let row_start = t.invalid_start;
    let row_end = t.invalid_end;

    for r in row_start..row_end {
        let linenr = rs_terminal_row_to_linenr(r, t.sb_current);
        unsafe { rs_fetch_row(term, r, width) };
        let textbuf = t.textbuf.as_mut_ptr();
        if linenr <= ml_line_count {
            unsafe { nvim_ml_replace_buf_term(buf, linenr, textbuf, true, false) };
            changed += 1;
        } else {
            unsafe { nvim_ml_append_buf_term(buf, linenr - 1, textbuf, 0, false) };
            added += 1;
        }
    }

    let change_start = rs_terminal_row_to_linenr(row_start, t.sb_current);
    let change_end = change_start + changed;
    // changed_lines(buf, lnum, col=0, lnume, xtra, do_buf_event=true)
    unsafe { nvim_changed_lines_term(buf, change_start, 0, change_end, added, true) };
    t.invalid_start = c_int::MAX;
    t.invalid_end = -1;
}

/// Adjust scrollback storage size, deleting lines exceeding new `scrollback` limit.
///
/// Replaces `adjust_scrollback` in `terminal_shim.c`.
///
/// # Safety
/// `term` and `buf` must be valid pointers.
unsafe fn rs_adjust_scrollback(term: TerminalHandle, buf: *mut c_void) {
    let t = unsafe { term.as_mut() };
    let mut scbk = unsafe { bref_raw(buf).b_p_scbk };
    if scbk < 1 {
        #[allow(clippy::cast_possible_wrap)]
        let sb_max = TERMINAL_SB_MAX as i64;
        scbk = sb_max;
        unsafe { (*buf.cast::<BufStruct>()).b_p_scbk = scbk };
    }
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let scbk = scbk as usize;
    assert!(t.sb_current < usize::MAX);
    assert!(
        t.sb_pending == 0,
        "sb_pending must be 0 before adjust_scrollback"
    );

    // Delete lines exceeding the new scrollback limit
    if scbk < t.sb_current {
        let diff = t.sb_current - scbk;
        for _ in 0..diff {
            unsafe { nvim_ml_delete_buf_term(buf, 1, false) };
            t.sb_current -= 1;
            let sbrow = unsafe { *t.sb_buffer.add(t.sb_current) };
            unsafe { xfree(sbrow) };
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        unsafe {
            // mark_adjust_buf(buf, 1, diff, MAXLNUM, -diff, adjust_folds=true, kMarkAdjustTerm=2, kExtmarkUndo=1)
            nvim_mark_adjust_buf_term(
                buf,
                1,
                diff as c_int,
                i32::MAX,
                -(diff as c_int),
                true,
                2,
                1,
            );
            nvim_deleted_lines_buf_term(buf, 1, diff as c_int);
        }
    }

    // Resize the scrollback storage
    if scbk != t.sb_size {
        unsafe {
            t.sb_buffer = nvim_term_xrealloc(
                t.sb_buffer.cast::<c_void>(),
                std::mem::size_of::<*mut c_void>() * scbk,
            )
            .cast::<*mut c_void>();
        }
        t.sb_size = scbk;
    }
}

/// Sync scrollback buffer with nvim buffer lines.
///
/// Replaces `refresh_scrollback` in `terminal_shim.c`.
///
/// # Safety
/// `term` and `buf` must be valid pointers.
unsafe fn rs_refresh_scrollback(term: TerminalHandle, buf: *mut c_void) {
    let t = unsafe { term.as_mut() };
    #[allow(clippy::cast_sign_loss)]
    let deleted =
        (t.sb_deleted - t.sb_deleted_last).min(unsafe { bref_raw(buf).ml_line_count } as usize);
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    unsafe {
        // mark_adjust_buf(buf, 1, deleted, MAXLNUM, -deleted, adjust_folds=true, kMarkAdjustTerm=2, kExtmarkUndo=1)
        nvim_mark_adjust_buf_term(
            buf,
            1,
            deleted as c_int,
            i32::MAX,
            -(deleted as c_int),
            true,
            2,
            1,
        );
    }
    t.sb_deleted_last = t.sb_deleted;

    let size = unsafe { rs_vterm_get_size(t.vt) };
    let height = size.rows;
    let width = size.cols;

    // May still have pending scrollback after increase in terminal height
    let row_offset = t.sb_pending;
    while t.sb_pending > 0 && unsafe { bref_raw(buf).ml_line_count } < height {
        unsafe { rs_fetch_row(term, t.sb_pending - row_offset - 1, width) };
        unsafe { nvim_ml_append_buf_term(buf, 0, t.textbuf.as_mut_ptr(), 0, false) };
        unsafe { nvim_appended_lines_buf_term(buf, 0, 1) };
        t.sb_pending -= 1;
    }

    let row_offset = row_offset - t.sb_pending;
    while t.sb_pending > 0 {
        let ml_line_count = unsafe { bref_raw(buf).ml_line_count };
        #[allow(clippy::cast_sign_loss)]
        if (ml_line_count - height) as usize >= t.sb_size {
            // scrollback full, delete lines at the top
            unsafe { nvim_ml_delete_buf_term(buf, 1, false) };
            unsafe { nvim_deleted_lines_buf_term(buf, 1, 1) };
        }
        unsafe { rs_fetch_row(term, -t.sb_pending - row_offset, width) };
        let buf_index = unsafe { bref_raw(buf).ml_line_count } - height;
        unsafe { nvim_ml_append_buf_term(buf, buf_index, t.textbuf.as_mut_ptr(), 0, false) };
        unsafe { nvim_appended_lines_buf_term(buf, buf_index, 1) };
        t.sb_pending -= 1;
    }

    // Remove extra lines at the bottom
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let max_line_count = t.sb_current as c_int + height;
    while unsafe { bref_raw(buf).ml_line_count } > max_line_count {
        let last = unsafe { bref_raw(buf).ml_line_count };
        unsafe { nvim_ml_delete_buf_term(buf, last, false) };
        unsafe { nvim_deleted_lines_buf_term(buf, last, 1) };
    }

    unsafe { rs_adjust_scrollback(term, buf) };
}

/// Update cursor shape/blink in `shape_table` and call `ui_mode_info_set`.
///
/// Replaces `refresh_cursor` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *`.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_cursor(term: TerminalHandle, cursor_visible: *mut bool) {
    if term.is_null() || cursor_visible.is_null() {
        return;
    }
    // Equivalent to C: (State & MODE_TERMINAL) && curbuf->terminal == term
    let is_active = unsafe {
        (c_nvim_state & MODE_TERMINAL) != 0 && bref_raw(curbuf).terminal == term.as_ptr()
    };
    if !is_active {
        return;
    }
    let t = unsafe { term.as_ref() };
    let cv = unsafe { &mut *cursor_visible };

    if t.cursor.visible != *cv {
        *cv = t.cursor.visible;
        if *cv {
            unsafe { nvim_ui_busy_stop() };
        } else {
            unsafe { nvim_ui_busy_start() };
        }
    }

    if !unsafe { term.as_ref() }.pending.cursor {
        return;
    }
    unsafe { term.as_mut() }.pending.cursor = false;

    let (shape, percentage) = match t.cursor.shape {
        VTERM_PROP_CURSORSHAPE_UNDERLINE => (SHAPE_HOR, 20),
        VTERM_PROP_CURSORSHAPE_BAR_LEFT => (SHAPE_VER, 25),
        _ => (SHAPE_BLOCK, 0), // BLOCK or unknown
    };
    unsafe { nvim_shape_table_set_cursor(c_int::from(t.cursor.blink), shape, percentage) };
    unsafe { nvim_term_ui_mode_info_set() };
}

/// Refresh a terminal: handle resize, scrollback, screen, topline cursor.
///
/// Replaces `refresh_terminal` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *`.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_terminal(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_ref() };
    let buf = unsafe { nvim_terminal_get_buffer(t.buf_handle) };
    if buf.is_null() || unsafe { rs_buf_valid(buf) } == 0 {
        if !buf.is_null() {
            // buf was destroyed by close_buffer
            unsafe { term.as_mut() }.buf_handle = 0;
        }
        return;
    }

    let ml_before = unsafe { bref_raw(buf).ml_line_count };
    unsafe { rs_terminal_refresh_size(term, buf) };
    unsafe { rs_refresh_scrollback(term, buf) };
    unsafe { rs_refresh_screen(term, buf) };
    let ml_added = unsafe { bref_raw(buf).ml_line_count } - ml_before;

    // Adjust topline and cursor - stays in C (uses FOR_ALL_TAB_WINDOWS macro)
    unsafe { rs_adjust_topline_cursor(term.as_ptr(), buf, ml_added) };

    // Copy pending events back to the main event queue
    unsafe {
        let main_loop = nvim_get_main_loop();
        let main_events = nvim_term_loop_get_events(main_loop);
        let t = term.as_mut();
        multiqueue_move_events(main_events, t.pending.events);
    };
}

/// Public C-callable wrapper for `rs_refresh_screen`.
///
/// # Safety
/// `term` and `buf` must be valid pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_screen_pub(term: TerminalHandle, buf: *mut c_void) {
    unsafe { rs_refresh_screen(term, buf) };
}

extern "C" {
    /// C `adjust_topline_cursor` (uses `FOR_ALL_TAB_WINDOWS` macro, stays in C).
    fn rs_adjust_topline_cursor(term: *mut c_void, buf: *mut c_void, added: c_int);
}

/// Timer callback: refresh all invalidated terminals.
///
/// Replaces `refresh_timer_cb` in `terminal_shim.c`.
///
/// # Safety
/// Called by libuv timer; all invalidated terminals must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_refresh_timer_cb(_watcher: *mut c_void, _data: *mut c_void) {
    unsafe { refresh_pending = false };
    if unsafe { c_exiting } {
        return;
    }
    // Iterate and refresh all invalidated terminals, then clear the set.
    // block/unblock_autocmds is done inside nvim_terminal_foreach_invalidated.
    #[allow(clippy::items_after_statements)]
    unsafe extern "C" fn refresh_one(term: *mut c_void, _ctx: *mut c_void) {
        let handle = unsafe { TerminalHandle::from_ptr(term) };
        unsafe { rs_refresh_terminal(handle) };
    }
    unsafe { nvim_terminal_foreach_invalidated(refresh_one, std::ptr::null_mut()) };
}

/// Handle `scrollback` option change on a terminal buffer.
///
/// Replaces `on_scrollback_option_changed` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *`.
#[no_mangle]
pub unsafe extern "C" fn rs_on_scrollback_option_changed(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_ref() };
    if !t.sb_buffer.is_null() {
        unsafe { rs_refresh_terminal(term) };
    }
}

// Phase 13: Migrate terminal_close

/// Close the terminal buffer.
///
/// Replaces `terminal_close` in `terminal_shim.c`.
///
/// # Safety
/// `termpp` must point to a valid `Terminal *`.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_close(termpp: *mut *mut c_void, status: c_int) {
    if termpp.is_null() {
        return;
    }
    let term = unsafe { TerminalHandle::from_ptr(*termpp) };
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_ref() };
    if t.destroy {
        return;
    }

    // If called inside free_all_mem(), the main loop has already been freed;
    // just destroy the terminal struct directly.
    if unsafe { c_entered_free_all_mem_fn() } != 0 {
        unsafe { rs_terminal_destroy(termpp) };
        return;
    }

    let only_destroy = if t.closed {
        // Process already exited: only clean up the terminal object.
        true
    } else {
        let t = unsafe { term.as_mut() };
        t.forward_mouse = false;
        if !unsafe { c_exiting } {
            // Equivalent to nvim_terminal_refresh_blocking: block_autocmds, refresh, unblock
            unsafe { c_block_autocmds() };
            unsafe { rs_refresh_terminal(term) };
            unsafe { c_unblock_autocmds() };
        }
        t.closed = true;
        false
    };

    let t = unsafe { term.as_ref() };
    let buf = unsafe { nvim_terminal_get_buffer(t.buf_handle) };

    if status == -1 || unsafe { c_exiting } {
        // Called by close_buffer() or while exiting: disconnect buffer from terminal.
        let t = unsafe { term.as_mut() };
        t.buf_handle = 0;
        if !buf.is_null() {
            unsafe { (*buf.cast::<BufStruct>()).terminal = std::ptr::null_mut() };
        }
        if t.refcount == 0 {
            // Not inside Terminal mode event handling: destroy immediately.
            t.destroy = true;
            // Equivalent to C: ((void (*)(void *))t->opts.close_cb)(t->opts.data)
            let close_cb: unsafe extern "C" fn(*mut c_void) =
                unsafe { std::mem::transmute(t.opts.close_cb) };
            unsafe { close_cb(t.opts.data) };
        }
    } else if !only_destroy {
        // Channel closed, editor not exiting: display exit message, wait for keypress.
        let is_internal = unsafe { nvim_terminal_opts_is_internal(term.as_ptr()) } != 0;
        let msg: std::ffi::CString = if is_internal {
            std::ffi::CString::new("\r\n[Terminal closed]").unwrap_or_default()
        } else {
            let s = format!("\r\n[Process exited {status}]");
            std::ffi::CString::new(s).unwrap_or_default()
        };
        let msg_bytes = msg.to_bytes();
        unsafe {
            rs_terminal_receive_impl(term, msg_bytes.as_ptr().cast(), msg_bytes.len());
        }
    }

    if only_destroy {
        return;
    }

    unsafe { nvim_terminal_apply_termclose_event(buf, status) };
}

// Phase 12: Migrate terminal_check_size

/// Resize the terminal to fit all windows that display it.
///
/// Replaces `terminal_check_size` in `terminal_shim.c`.
///
/// # Safety
/// `term` must be a valid `Terminal *`.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_check_size(term: TerminalHandle) {
    if term.is_null() {
        return;
    }
    let t = unsafe { term.as_ref() };
    if t.closed {
        return;
    }

    let cur_size = unsafe { rs_vterm_get_size(t.vt) };
    let curheight = cur_size.rows;
    let curwidth = cur_size.cols;

    let mut width: u16 = 0;
    let mut height: u16 = 0;
    // Check all windows that display this terminal and find the max size.
    // Skip the autocommand window which isn't actually displayed.
    unsafe {
        nvim_terminal_find_size(term.as_ptr(), &raw mut width, &raw mut height);
    }

    // If no window displays the terminal, or all are zero-height, don't resize.
    #[allow(clippy::cast_lossless)]
    if (curheight == height as c_int && curwidth == width as c_int) || height == 0 || width == 0 {
        return;
    }

    let t = unsafe { term.as_mut() };
    unsafe { rs_vterm_set_size(t.vt, c_int::from(height), c_int::from(width)) };
    unsafe { rs_vterm_screen_flush_damage(t.vts) };
    t.pending.resize = true;
    unsafe { rs_invalidate_terminal(term, -1, -1) };
}

// Phase 17: Migrate set_terminal_winopts / unset_terminal_winopts

extern "C" {
    // nvim_curwin_ptr: use c_curwin static instead
    #[link_name = "redraw_later"]
    fn nvim_win_redraw_later(wp: *mut c_void, kind: c_int);
    // UPD_SOME_VALID = 2, UPD_VALID = 3, UPD_NOT_VALID = 4
    #[link_name = "free_string_option"]
    fn nvim_free_string_option(str: *mut i8);
    fn nvim_win_set_p_culopt(wp: *mut c_void, s: *mut i8);
    #[link_name = "xstrdup"]
    fn nvim_xstrdup(s: *const i8) -> *mut i8;
    // string option — no WinStruct accessor, keep extern
    fn nvim_win_get_p_culopt(wp: *mut c_void) -> *const i8;
    fn nvim_handle_get_window(handle: c_int) -> *mut c_void;
    fn rs_win_valid(wp: *mut c_void) -> c_int;
}

// kOptCuloptFlagNumber = 0x04 (from option_vars.generated.h)
const K_OPT_CULOPT_FLAG_NUMBER: u8 = 0x04;

/// Save current window options and apply terminal-mode overrides.
///
/// Replaces `set_terminal_winopts` in `terminal_shim.c`.
///
/// # Safety
/// `s` must be a valid `*mut TerminalStateRust`.
///
/// # Panics
/// Panics if `s.save_curwin_handle != 0` (window opts already saved).
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::similar_names
)]
pub unsafe extern "C" fn rs_set_terminal_winopts(s: *mut c_void) {
    let s = unsafe { &mut *s.cast::<TerminalStateRust>() };
    assert!(s.save_curwin_handle == 0);

    let curwin = unsafe { c_curwin };
    // Disable these options in terminal-mode. They are nonsense because cursor is
    // placed at end of buffer to "follow" output. #11072
    s.save_curwin_handle = unsafe { win_ref_raw(curwin).handle };
    s.save_w_p_cul = unsafe { win_ref_raw(curwin).w_p_cul() } != 0;
    s.save_w_p_culopt = std::ptr::null_mut();
    s.save_w_p_culopt_flags = unsafe { win_ref_raw(curwin).w_p_culopt_flags };
    s.save_w_p_cuc = unsafe { win_ref_raw(curwin).w_p_cuc() };
    s.save_w_p_so = unsafe { win_ref_raw(curwin).w_p_so() };
    s.save_w_p_siso = unsafe { win_ref_raw(curwin).w_p_siso() };

    let culopt_flags = s.save_w_p_culopt_flags;
    if s.save_w_p_cul && (culopt_flags & K_OPT_CULOPT_FLAG_NUMBER != 0) {
        // Compare curwin->w_p_culopt to "number"
        let culopt_c = unsafe { nvim_win_get_p_culopt(curwin) };
        let is_number = !culopt_c.is_null() && {
            // Safety: culopt_c is a valid C string owned by the option system
            let culopt_str = unsafe { std::ffi::CStr::from_ptr(culopt_c) };
            culopt_str == c"number"
        };
        if !is_number {
            // Save old culopt and replace with "number"
            s.save_w_p_culopt = culopt_c.cast_mut();
            let new_culopt = unsafe { nvim_xstrdup(c"number".as_ptr()) };
            unsafe { nvim_win_set_p_culopt(curwin, new_culopt) };
        }
        unsafe { win_mut_raw(curwin).w_p_culopt_flags = K_OPT_CULOPT_FLAG_NUMBER };
    } else {
        unsafe { win_mut_raw(curwin).set_w_p_cul(0) };
    }
    unsafe { win_mut_raw(curwin).set_w_p_cuc(0) };
    unsafe { win_mut_raw(curwin).set_w_p_so(0) };
    unsafe { win_mut_raw(curwin).set_w_p_siso(0) };

    let result_cuc = unsafe { win_ref_raw(curwin).w_p_cuc() } != 0;
    let result_cul = unsafe { win_ref_raw(curwin).w_p_cul() } != 0;
    let result_culopt_flags = unsafe { win_ref_raw(curwin).w_p_culopt_flags };
    if result_cuc != (s.save_w_p_cuc != 0) {
        unsafe { nvim_win_redraw_later(curwin, 35) };
    } else if result_cul != s.save_w_p_cul
        || (result_cul && result_culopt_flags != s.save_w_p_culopt_flags)
    {
        unsafe { nvim_win_redraw_later(curwin, 10) };
    }
}

/// Restore window options after leaving terminal mode.
///
/// Replaces `unset_terminal_winopts` in `terminal_shim.c`.
///
/// # Safety
/// `s` must be a valid `*mut TerminalStateRust`.
///
/// # Panics
/// Panics if `s.save_curwin_handle == 0` (window opts not saved).
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::similar_names
)]
pub unsafe extern "C" fn rs_unset_terminal_winopts(s: *mut c_void) {
    let s = unsafe { &mut *s.cast::<TerminalStateRust>() };
    assert!(s.save_curwin_handle != 0);

    let wp = unsafe { nvim_handle_get_window(s.save_curwin_handle) };
    if wp.is_null() {
        unsafe { nvim_free_string_option(s.save_w_p_culopt) };
        s.save_curwin_handle = 0;
        return;
    }

    if unsafe { rs_win_valid(wp) } != 0 {
        // No need to redraw if window not in curtab.
        let wp_has_cuc = unsafe { win_ref_raw(wp).w_p_cuc() } != 0;
        let wp_has_cul = unsafe { win_ref_raw(wp).w_p_cul() } != 0;
        let wp_culopt_flags = unsafe { win_ref_raw(wp).w_p_culopt_flags };
        if (s.save_w_p_cuc != 0) != wp_has_cuc {
            unsafe { nvim_win_redraw_later(wp, 35) };
        } else if s.save_w_p_cul != wp_has_cul
            || (s.save_w_p_cul && s.save_w_p_culopt_flags != wp_culopt_flags)
        {
            unsafe { nvim_win_redraw_later(wp, 10) };
        }
    }

    unsafe { win_mut_raw(wp).set_w_p_cul(c_int::from(s.save_w_p_cul)) };
    if !s.save_w_p_culopt.is_null() {
        let old_culopt = unsafe { nvim_win_get_p_culopt(wp) };
        unsafe { nvim_free_string_option(old_culopt.cast_mut()) };
        unsafe { nvim_win_set_p_culopt(wp, s.save_w_p_culopt) };
    }
    unsafe { win_mut_raw(wp).w_p_culopt_flags = s.save_w_p_culopt_flags };
    unsafe { win_mut_raw(wp).set_w_p_cuc(c_int::from(s.save_w_p_cuc != 0)) };
    unsafe { win_mut_raw(wp).set_w_p_so(s.save_w_p_so) };
    unsafe { win_mut_raw(wp).set_w_p_siso(s.save_w_p_siso) };
    s.save_curwin_handle = 0;
}

// Phase 14: Migrate terminal_enter state machine

// Constants for terminal_execute
const K_EVENT: c_int = -26365; // TERMCAP2KEY(KS_EXTRA, KE_EVENT=102)
const K_COMMAND: c_int = -26877; // TERMCAP2KEY(KS_EXTRA, KE_COMMAND=104)
const K_LUA: c_int = -26621; // TERMCAP2KEY(KS_EXTRA, KE_LUA=103)
const K_PASTE_START: c_int = -21328; // TERMCAP2KEY('P','S')
const CTRL_BSL: c_int = 28; // Ctrl_BSL from ascii_defs.h
const CTRL_N: c_int = 14; // Ctrl_N
const CTRL_O: c_int = 15; // Ctrl_O
const CTRL_C: c_int = 3; // Ctrl_C
const SHAPE_CURSOR: c_int = 2; // SHAPE_CURSOR from cursor_shape.h

/// Process one char of terminal-mode input.
///
/// Replaces `terminal_execute` in `terminal_shim.c`.
/// Called via the `VimState.execute` function pointer.
///
/// # Safety
/// `state` must be a valid `*mut TerminalStateRust`.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_execute(state: *mut c_void, key: c_int) -> c_int {
    if state.is_null() {
        return 0;
    }
    let s = unsafe { &mut *state.cast::<TerminalStateRust>() };

    let mut tmp_mod_mask = unsafe { mod_mask };
    let mod_key = unsafe { nvim_merge_modifiers_c(key, &raw mut tmp_mod_mask) };

    // Mouse events
    let is_mouse = matches!(
        mod_key,
        _ if mod_key == K_LEFTMOUSE
            || mod_key == K_LEFTDRAG
            || mod_key == K_LEFTRELEASE
            || mod_key == K_MIDDLEMOUSE
            || mod_key == K_MIDDLEDRAG
            || mod_key == K_MIDDLERELEASE
            || mod_key == K_RIGHTMOUSE
            || mod_key == K_RIGHTDRAG
            || mod_key == K_RIGHTRELEASE
            || mod_key == K_X1MOUSE
            || mod_key == K_X1DRAG
            || mod_key == K_X1RELEASE
            || mod_key == K_X2MOUSE
            || mod_key == K_X2DRAG
            || mod_key == K_X2RELEASE
            || mod_key == K_MOUSEDOWN
            || mod_key == K_MOUSEUP
            || mod_key == K_MOUSELEFT
            || mod_key == K_MOUSERIGHT
            || mod_key == K_MOUSEMOVE
    );

    if is_mouse {
        if unsafe { rs_send_mouse_event(TerminalHandle::from_ptr(s.term), key) } != 0 {
            return 0;
        }
        return 1;
    }

    if mod_key == K_PASTE_START {
        unsafe { nvim_paste_repeat_c(1) };
        return 1;
    }

    if mod_key == K_EVENT {
        // Don't free the terminal yet; it is still needed.
        let term = unsafe { TerminalHandle::from_ptr(s.term) };
        unsafe { term.as_mut().refcount += 1 };
        unsafe { state_handle_k_event() };
        unsafe { term.as_mut().refcount -= 1 };
        if unsafe { term.as_ref().buf_handle } == 0 {
            s.close = true;
            return 0;
        }
        return 1;
    }

    if mod_key == K_COMMAND {
        unsafe {
            do_cmdline(
                std::ptr::null_mut(),
                Some(getcmdkeycmd),
                std::ptr::null_mut(),
                0,
            );
        };
        return 1;
    }

    if mod_key == K_LUA {
        unsafe { nvim_map_execute_lua_c(false, false) };
        return 1;
    }

    if mod_key == CTRL_N && s.got_bsl {
        return 0;
    }

    if mod_key == CTRL_O && s.got_bsl {
        s.got_bsl_o = true;
        unsafe { nvim_set_restart_edit(c_int::from(b'I')) };
        return 0;
    }

    // Default / fallthrough
    if mod_key == CTRL_C {
        // Always map CTRL-C to avoid interrupt.
        unsafe { c_got_int = false };
    }

    if mod_key == CTRL_BSL && !s.got_bsl {
        s.got_bsl = true;
        return 1;
    }

    let term = unsafe { TerminalHandle::from_ptr(s.term) };
    if unsafe { term.as_ref().closed } {
        s.close = true;
        return 0;
    }

    s.got_bsl = false;
    // terminal_send_key is rs_terminal_send_key_impl
    unsafe { rs_terminal_send_key_impl(term, key) };

    1
}

/// Check function called before each iteration of terminal mode.
///
/// Replaces `terminal_check` in `terminal_shim.c`.
/// Called via the `VimState.check` function pointer.
///
/// # Safety
/// `state` must be a valid `*mut TerminalStateRust`.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_check(state: *mut c_void) -> c_int {
    if state.is_null() {
        return 0;
    }
    let s = unsafe { &mut *state.cast::<TerminalStateRust>() };

    if unsafe { nvim_get_stop_insert_mode() } != 0 {
        return 0;
    }

    // Check focus; returns false if we should exit the state machine.
    if !rs_terminal_check_focus_impl(s) {
        return 0;
    }

    // Validate topline and cursor position for autocommands.
    unsafe { rs_terminal_check_cursor() };
    unsafe { nvim_validate_cursor_cw(c_curwin) };

    // Don't let autocommands free the terminal from under our fingers.
    let term = unsafe { TerminalHandle::from_ptr(s.term) };
    unsafe { term.as_mut().refcount += 1 };

    if unsafe { has_event(124) } != 0 {
        let buf_ref = unsafe { bref_raw(curbuf) };
        let cur_tick = unsafe { nvim_term_buf_get_changedtick(curbuf.cast()) };
        if buf_ref.b_last_changedtick_i != cur_tick {
            // EVENT_TEXTCHANGEDT = 124
            unsafe {
                apply_autocmds(
                    124,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    false,
                    curbuf,
                )
            };
            unsafe { (*curbuf.cast::<BufStruct>()).b_last_changedtick_i = cur_tick };
        }
    }
    unsafe { nvim_may_trigger_win_scrolled_resized() };
    unsafe { term.as_mut().refcount -= 1 };

    if unsafe { term.as_ref().buf_handle } == 0 {
        s.close = true;
        return 0;
    }

    if !rs_terminal_check_focus_impl(s) {
        return 0;
    }
    unsafe { rs_terminal_check_cursor() };
    unsafe { nvim_validate_cursor_cw(c_curwin) };

    unsafe { nvim_show_cursor_info_later(false) };
    if unsafe { c_must_redraw } != 0 {
        unsafe { nvim_update_screen_c() };
    } else {
        unsafe { nvim_redraw_statuslines() };
        if unsafe { c_clear_cmdline } || unsafe { c_redraw_cmdline } || unsafe { c_redraw_mode } {
            unsafe { showmode() };
        }
    }

    unsafe { nvim_setcursor() };
    let mut cv = s.cursor_visible;
    unsafe { rs_refresh_cursor(TerminalHandle(s.term), &raw mut cv) };
    s.cursor_visible = cv;
    unsafe { nvim_ui_flush() };
    1
}

/// Check focus state and update `TerminalStateRust` accordingly.
/// Returns `true` if the state machine should continue, `false` to exit.
fn rs_terminal_check_focus_impl(s: &mut TerminalStateRust) -> bool {
    let cur_terminal = unsafe { bref_raw(curbuf).terminal };
    if cur_terminal.is_null() {
        return false;
    }

    let curwin_handle = unsafe { win_ref_raw(c_curwin).handle };
    if s.save_curwin_handle != curwin_handle {
        // Terminal window changed, update window options.
        unsafe { nvim_terminal_unset_winopts(std::ptr::addr_of_mut!(*s).cast()) };
        unsafe { nvim_terminal_set_winopts(std::ptr::addr_of_mut!(*s).cast()) };
    }

    if s.term != cur_terminal {
        // Active terminal buffer changed.
        let old_term = unsafe { TerminalHandle::from_ptr(s.term) };
        rs_terminal_focus_lose(old_term);

        s.term = cur_terminal;
        let new_term = unsafe { TerminalHandle::from_ptr(s.term) };
        unsafe { new_term.as_mut().pending.cursor = true };
        unsafe { rs_invalidate_terminal(new_term, -1, -1) };
        rs_terminal_focus_gain(new_term);
    }
    true
}

/// Implements `MODE_TERMINAL` state. :help Terminal-mode.
///
/// Replaces `terminal_enter` in `terminal_shim.c`.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub extern "C" fn rs_terminal_enter() -> bool {
    let cur_terminal = unsafe { bref_raw(curbuf).terminal };
    if cur_terminal.is_null() {
        return false;
    }

    let mut s = TerminalStateRust {
        state: VimStateRust {
            check: Some(rs_terminal_check),
            execute: Some(rs_terminal_execute),
        },
        term: cur_terminal,
        cursor_visible: true, // Assume visible; may change via refresh_cursor later.
        save_rd: 0,
        close: false,
        got_bsl: false,
        got_bsl_o: false,
        save_curwin_handle: 0,
        save_w_p_cul: false,
        _pad1: [0; 3],
        save_w_p_culopt: std::ptr::null_mut(),
        save_w_p_culopt_flags: 0,
        _pad2: [0; 3],
        save_w_p_cuc: 0,
        save_w_p_so: 0,
        save_w_p_siso: 0,
    };

    // Ensure the terminal is properly sized.
    let term = unsafe { TerminalHandle::from_ptr(s.term) };
    unsafe { rs_terminal_check_size(term) };

    let save_state = unsafe { nvim_get_state() };
    s.save_rd = unsafe { nvim_get_RedrawingDisabled() };
    unsafe { nvim_set_state(MODE_TERMINAL) };
    // Always map CTRL-C to avoid interrupt.
    unsafe { mapped_ctrl_c |= MODE_TERMINAL };
    unsafe { nvim_set_RedrawingDisabled(0) };
    unsafe { nvim_set_stop_insert_mode(0) };

    unsafe { nvim_terminal_set_winopts(std::ptr::addr_of_mut!(s).cast()) };

    // Update the cursor shape and scroll to end.
    unsafe { term.as_mut().pending.cursor = true };
    let buf = unsafe { nvim_terminal_get_buffer(term.as_ref().buf_handle) };
    unsafe { rs_adjust_topline_cursor(s.term, buf, 0) };
    unsafe { showmode() };
    unsafe { nvim_ui_cursor_shape() };

    // Tell the terminal it has focus.
    rs_terminal_focus_gain(term);
    // Don't fire TextChangedT from changes in Normal mode.
    unsafe {
        (*curbuf.cast::<BufStruct>()).b_last_changedtick_i =
            nvim_term_buf_get_changedtick(curbuf.cast());
    };

    // EVENT_TERMENTER = 116
    unsafe {
        apply_autocmds(
            116,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            false,
            curbuf,
        )
    };
    unsafe { may_trigger_modechanged() };

    // Run the state machine.
    unsafe { nvim_state_enter_c(std::ptr::addr_of_mut!(s.state).cast()) };

    let got_bsl_o = s.got_bsl_o;
    if !got_bsl_o {
        unsafe { nvim_set_restart_edit(0) };
    }
    unsafe { nvim_set_state(save_state) };
    unsafe { nvim_set_RedrawingDisabled(s.save_rd) };

    if !s.cursor_visible {
        // If cursor was hidden, show it again.
        unsafe { nvim_ui_busy_stop() };
    }

    // Restore the terminal cursor to what is set in 'guicursor'.
    let _ = unsafe { nvim_parse_shape_opt(SHAPE_CURSOR) };

    unsafe { nvim_terminal_unset_winopts(std::ptr::addr_of_mut!(s).cast()) };

    // Tell the terminal it lost focus.
    let term = unsafe { TerminalHandle::from_ptr(s.term) };
    rs_terminal_focus_lose(term);
    // Don't fire TextChanged from changes in terminal mode.
    unsafe {
        (*curbuf.cast::<BufStruct>()).b_last_changedtick =
            nvim_term_buf_get_changedtick(curbuf.cast());
    };

    let cur_terminal = unsafe { bref_raw(curbuf).terminal };
    let same_term = cur_terminal == s.term;
    if same_term && !s.close {
        unsafe { rs_terminal_check_cursor() };
    }
    if unsafe { nvim_get_restart_edit() } != 0 {
        unsafe { showmode() };
    } else {
        unsafe { nvim_unshowmode(true) };
    }
    unsafe { nvim_ui_cursor_shape() };

    // If we're to close the terminal, don't let TermLeave autocmds free it first!
    if s.close {
        let term = unsafe { TerminalHandle::from_ptr(s.term) };
        unsafe { term.as_mut().refcount += 1 };
    }
    // EVENT_TERMLEAVE = 117
    unsafe {
        apply_autocmds(
            117,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            false,
            curbuf,
        )
    };
    if s.close {
        let term = unsafe { TerminalHandle::from_ptr(s.term) };
        unsafe { term.as_mut().refcount -= 1 };
        let buf_handle = unsafe { term.as_ref().buf_handle };
        unsafe { term.as_mut().destroy = true };
        {
            let t = unsafe { term.as_ref() };
            let close_cb: unsafe extern "C" fn(*mut c_void) =
                unsafe { std::mem::transmute(t.opts.close_cb) };
            unsafe { close_cb(t.opts.data) };
        }
        if buf_handle != 0 {
            // do_buffer_ext(DOBUF_WIPE=4, DOBUF_FIRST=1, FORWARD=1, count, DOBUF_FORCEIT=1)
            unsafe { nvim_do_buffer_wipe(4, 1, 1, buf_handle, 1) };
        }
    }

    got_bsl_o
}

// =============================================================================
// Phase 5: Migrate terminal_open
// =============================================================================

extern "C" {
    fn xcalloc(count: usize, size: usize) -> *mut c_void;
    fn vterm_new(rows: c_int, cols: c_int) -> *mut c_void;
    fn vterm_set_utf8(vt: *mut c_void, is_utf8: c_int);
    fn vterm_obtain_screen(vt: *mut c_void) -> *mut c_void;
    fn vterm_screen_enable_altscreen(vts: *mut c_void, altscreen: c_int);
    fn vterm_screen_enable_reflow(vts: *mut c_void, reflow: bool);
    fn vterm_screen_set_damage_merge(vts: *mut c_void, size: c_int);
    fn vterm_screen_reset(vts: *mut c_void, hard: c_int);
    fn vterm_output_set_callback(
        vt: *mut c_void,
        cb: unsafe extern "C" fn(*const i8, usize, *mut c_void),
        user: *mut c_void,
    );
    fn vterm_state_set_termprop(state: *mut c_void, prop: c_int, val: *mut c_void) -> c_int;
    fn nvim_terminal_vterm_set_callbacks(term: *mut c_void);
    // nvim_get_shape_table_* from cursor_shape shim -- pass SHAPE_IDX_TERM=17
    #[link_name = "nvim_get_shape_table_shape"]
    fn nvim_shape_table_get_shape(idx: c_int) -> c_int;
    #[link_name = "nvim_get_shape_table_blinkon"]
    fn nvim_shape_table_get_blinkon(idx: c_int) -> c_int;
    #[link_name = "nvim_get_shape_table_blinkoff"]
    fn nvim_shape_table_get_blinkoff(idx: c_int) -> c_int;
    fn nvim_set_option_value_buftype_terminal();
    fn nvim_aucmd_prepbuf_alloc(buf: *mut c_void) -> *mut c_void;
    // aucmd_restbuf + xfree replaces nvim_aucmd_restbuf_free
    #[link_name = "aucmd_restbuf"]
    fn nvim_aucmd_restbuf(aco: *mut c_void);
    // apply_autocmds declared above; EVENT_TERMOPEN = 118
    fn nvim_reset_binding_curwin();

    // multiqueue_new(on_put, data) - pass NULL for standalone queue
    #[link_name = "multiqueue_new"]
    fn nvim_multiqueue_new_standalone(on_put: *const c_void, data: *mut c_void) -> *mut c_void;
    fn nvim_get_config_string(key: *const i8) -> *mut i8;
    // name_to_color(name, idx) - idx is an out param we discard
    #[link_name = "name_to_color"]
    fn nvim_name_to_color_int(name: *const i8, idx: *mut c_int) -> c_int;
    fn nvim_terminal_vterm_set_palette(state: *mut c_void, i: c_int, r: c_int, g: c_int, b: c_int);
}

/// `VTERM_DAMAGE_SCROLL` constant from vterm.h.
const VTERM_DAMAGE_SCROLL: c_int = 2;
/// `SELECTIONBUF_SIZE` from `terminal_shim.c`.
const SELECTIONBUF_SIZE: usize = 0x0400;

/// Initialize vterm, register callbacks, and configure the initial cursor shape.
///
/// # Safety
/// `term` and `term_ptr` must point to the same valid `CTerminal`. `height`/`width` from opts.
unsafe fn rs_terminal_open_init_vterm(
    term: &mut CTerminal,
    term_ptr: *mut CTerminal,
    height: u16,
    width: u16,
) {
    use nvim_vterm::VTermValue;
    term.vt = unsafe { vterm_new(c_int::from(height), c_int::from(width)) };
    unsafe { vterm_set_utf8(term.vt, 1) };
    term.vts = unsafe { vterm_obtain_screen(term.vt) };
    unsafe { vterm_screen_enable_altscreen(term.vts, 1) };
    unsafe { vterm_screen_enable_reflow(term.vts, true) };
    term.selection_buffer = unsafe { xcalloc(SELECTIONBUF_SIZE, 1) }.cast();
    unsafe { nvim_terminal_vterm_set_callbacks(term_ptr.cast()) };
    unsafe { vterm_screen_set_damage_merge(term.vts, VTERM_DAMAGE_SCROLL) };
    unsafe { vterm_screen_reset(term.vts, 1) };
    unsafe {
        vterm_output_set_callback(term.vt, rs_term_output_callback_trampoline, term_ptr.cast());
    };
    // SHAPE_IDX_TERM = 17
    let shape = unsafe { nvim_shape_table_get_shape(17) };
    let cursor_shape_num = match shape {
        SHAPE_BLOCK => VTERM_PROP_CURSORSHAPE_BLOCK,
        SHAPE_HOR => VTERM_PROP_CURSORSHAPE_UNDERLINE,
        _ => VTERM_PROP_CURSORSHAPE_BAR_LEFT,
    };
    let state = unsafe { vterm_obtain_state(term.vt) };
    let mut val_shape = VTermValue {
        number: cursor_shape_num,
    };
    unsafe {
        vterm_state_set_termprop(state, VTERM_PROP_CURSORSHAPE, (&raw mut val_shape).cast());
    };
    let blinkon = unsafe { nvim_shape_table_get_blinkon(17) };
    let blinkoff = unsafe { nvim_shape_table_get_blinkoff(17) };
    let mut val_blink = VTermValue {
        boolean: c_int::from(blinkon != 0 && blinkoff != 0),
    };
    unsafe {
        vterm_state_set_termprop(state, VTERM_PROP_CURSORBLINK, (&raw mut val_blink).cast());
    };
}

/// Initialize a new terminal.
///
/// Replaces `terminal_open` in `terminal_shim.c`.
///
/// # Safety
/// `termpp` must be a valid `Terminal **`, `buf` and `opts` must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_open(
    termpp: *mut *mut c_void,
    buf: *mut c_void,
    opts: CTerminalOptions,
) {
    // Allocate and initialize the terminal struct.
    let term_ptr = unsafe { xcalloc(1, std::mem::size_of::<CTerminal>()) }.cast::<CTerminal>();
    unsafe { *termpp = term_ptr.cast() };
    let term = unsafe { &mut *term_ptr };

    // Save height/width before moving opts into term.
    let height = opts.height;
    let width = opts.width;

    // Fill in basic fields.
    term.opts = opts;
    // buf_handle = buf->handle (direct BufStruct field access)
    let buf_handle = unsafe { bref_raw(buf).handle };
    term.buf_handle = buf_handle;
    // buf->terminal = term
    unsafe { (*buf.cast::<BufStruct>()).terminal = term_ptr.cast() };

    // Initialize vterm and set up callback tables and cursor shape.
    unsafe { rs_terminal_open_init_vterm(term, term_ptr, height, width) };

    // Force initial refresh of the screen.
    term.invalid_start = 0;
    term.invalid_end = c_int::from(height);

    // Pending events queue: deferred until next terminal refresh (#32753).
    term.pending.events =
        unsafe { nvim_multiqueue_new_standalone(std::ptr::null(), std::ptr::null_mut()) };

    // Run TermOpen autocmd in the context of the terminal buffer.
    let aco = unsafe { nvim_aucmd_prepbuf_alloc(buf) };
    let term_handle = unsafe { TerminalHandle::from_ptr(term_ptr.cast()) };
    unsafe { rs_refresh_screen_pub(term_handle, buf) };
    unsafe { nvim_set_option_value_buftype_terminal() };
    let ffname = unsafe { bref_raw(buf).b_ffname };
    if !ffname.is_null() {
        let len = unsafe { std::ffi::CStr::from_ptr(ffname).to_bytes().len() };
        unsafe { nvim_terminal_buf_set_title(buf, ffname, len) };
    }
    unsafe { nvim_reset_binding_curwin() };
    // Set curwin->w_cursor to {.lnum=1, .col=0, .coladd=0}
    unsafe {
        win_mut_raw(c_curwin).w_cursor = nvim_window::win_struct::PosT {
            lnum: 1,
            col: 0,
            coladd: 0,
        }
    };
    term.sb_buffer = std::ptr::null_mut(); // check if TermOpen autocmd allocates it
                                           // EVENT_TERMOPEN = 118
    unsafe { apply_autocmds(118, std::ptr::null_mut(), std::ptr::null_mut(), false, buf) };
    // aucmd_restbuf + xfree (was: nvim_aucmd_restbuf_free)
    unsafe { nvim_aucmd_restbuf(aco) };
    unsafe { xfree(aco) };

    // Check if terminal was already destroyed during TermOpen autocmd.
    if unsafe { *termpp }.is_null() {
        return;
    }

    // Allocate scrollback buffer if TermOpen autocmd didn't already.
    if term.sb_buffer.is_null() {
        let scbk = unsafe { bref_raw(buf).b_p_scbk };
        #[allow(clippy::cast_possible_wrap)]
        let sb_max_i64 = TERMINAL_SB_MAX as i64;
        let scbk = if scbk < 1 {
            unsafe { (*buf.cast::<BufStruct>()).b_p_scbk = sb_max_i64 };
            sb_max_i64
        } else {
            scbk
        };
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        let sb_size = scbk as usize;
        term.sb_size = sb_size;
        term.sb_buffer =
            unsafe { xmalloc(sb_size * std::mem::size_of::<*mut c_void>()) }.cast::<*mut c_void>();
    }

    // Configure color palette from b:terminal_color_{N} / g:terminal_color_{N}.
    let state = unsafe { vterm_obtain_state(term.vt) };
    for i in 0..16_i32 {
        let key = format!("terminal_color_{i}\0");
        let name = unsafe { nvim_get_config_string(key.as_ptr().cast()) };
        if name.is_null() {
            continue;
        }
        let color_val = unsafe {
            let mut dummy: c_int = 0;
            nvim_name_to_color_int(name, &raw mut dummy)
        };
        if color_val == -1 {
            continue;
        }
        #[allow(clippy::cast_sign_loss)]
        let (r, g, b) = (
            (color_val >> 16) & 0xFF,
            (color_val >> 8) & 0xFF,
            color_val & 0xFF,
        );
        unsafe { nvim_terminal_vterm_set_palette(state, i, r, g, b) };
        #[allow(clippy::cast_sign_loss)]
        {
            term.color_set[i as usize] = true;
        }
    }
}

// Phase 6: terminal_init, terminal_teardown, terminal_check_cursor
extern "C" {
    fn nvim_terminal_init_timer();
    fn nvim_terminal_teardown_timer();
    // set_topline(wp, lnum) - use c_curwin static
    #[link_name = "set_topline"]
    fn nvim_set_topline_curwin(wp: *mut c_void, lnum: c_int);
    static mut curbuf: *mut std::ffi::c_void;
    fn nvim_curwin_get_view_height() -> c_int;
    fn nvim_curwin_get_w_p_rl() -> c_int;
    fn nvim_curwin_set_cursor_lnum(lnum: c_int);
    fn nvim_coladvance(col: c_int);
}

/// Initialize the terminal subsystem timer and event queue.
///
/// Replaces `terminal_init` in `terminal_shim.c`.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_init() {
    unsafe { nvim_terminal_init_timer() };
}

/// Tear down the terminal subsystem timer and free resources.
///
/// Replaces `terminal_teardown` in `terminal_shim.c`.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_teardown() {
    unsafe { nvim_terminal_teardown_timer() };
}

/// Reposition the cursor to the terminal's current cursor position.
///
/// Replaces `terminal_check_cursor` in `terminal_shim.c`.
///
/// # Safety
/// Must be called with a valid current buffer and window that have a terminal attached.
#[no_mangle]
pub unsafe extern "C" fn rs_terminal_check_cursor() {
    let term_ptr = unsafe { bref_raw(curbuf).terminal }.cast::<CTerminal>();
    if term_ptr.is_null() {
        return;
    }
    let t = unsafe { &*term_ptr };
    let ml_line_count =
        nvim_buffer::buf_struct::buf_ref(nvim_buffer::BufHandle::from_ptr(curbuf)).ml_line_count;
    let linenr = rs_terminal_row_to_linenr(t.cursor.row, t.sb_current);
    let lnum = ml_line_count.min(linenr);
    unsafe { nvim_curwin_set_cursor_lnum(lnum) };
    let view_height = unsafe { nvim_curwin_get_view_height() };
    let topline = (ml_line_count - view_height + 1).max(1);
    if topline != unsafe { win_ref_raw(c_curwin).w_topline } {
        unsafe { nvim_set_topline_curwin(c_curwin, topline) };
    }
    let state = unsafe { nvim_get_state() };
    let off = if (state & MODE_TERMINAL) != 0
        && unsafe { bref_raw(curbuf).terminal } == term_ptr.cast()
    {
        0
    } else if unsafe { nvim_curwin_get_w_p_rl() } != 0 {
        1
    } else {
        -1
    };
    let col = (t.cursor.col + off).max(0);
    unsafe { nvim_coladvance(col) };
}

/// The output callback that vterm calls when it has data to send.
///
/// This is a thin trampoline to [`rs_term_output_callback`] for use with `vterm_output_set_callback`.
///
/// # Safety
/// Must be called from vterm with valid `s` and `user_data` pointers.
unsafe extern "C" fn rs_term_output_callback_trampoline(
    s: *const i8,
    len: usize,
    user_data: *mut c_void,
) {
    unsafe { rs_term_output_callback(s, len, user_data) };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_handle_null() {
        let handle = unsafe { TerminalHandle::from_ptr(std::ptr::null_mut()) };
        assert!(handle.is_null());
        assert!(!terminal_running_impl(handle));
        assert_eq!(rs_terminal_forward_mouse(handle), 0);
        assert_eq!(rs_terminal_theme_updates(handle), 0);
    }

    #[test]
    fn test_terminal_handle_non_null() {
        let fake_ptr = 0x1000 as *mut c_void;
        let handle = unsafe { TerminalHandle::from_ptr(fake_ptr) };
        assert!(!handle.is_null());
        assert_eq!(handle.as_ptr(), fake_ptr);
    }

    #[test]
    fn test_vterm_key_function() {
        // Valid function keys
        assert_eq!(vterm_key_function(1), VTERM_KEY_MAX + 1);
        assert_eq!(vterm_key_function(12), VTERM_KEY_MAX + 12);
        assert_eq!(vterm_key_function(66), VTERM_KEY_MAX + 66);

        // Invalid function keys
        assert_eq!(vterm_key_function(0), VTERM_KEY_NONE);
        assert_eq!(vterm_key_function(67), VTERM_KEY_NONE);
        assert_eq!(vterm_key_function(-1), VTERM_KEY_NONE);
    }

    #[test]
    fn test_convert_key_arrow_keys() {
        let result = rs_terminal_convert_key(K_UP, 0);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_NONE);

        let result = rs_terminal_convert_key(K_DOWN, 0);
        assert_eq!(result.key, VTERM_KEY_DOWN);

        let result = rs_terminal_convert_key(K_LEFT, 0);
        assert_eq!(result.key, VTERM_KEY_LEFT);

        let result = rs_terminal_convert_key(K_RIGHT, 0);
        assert_eq!(result.key, VTERM_KEY_RIGHT);
    }

    #[test]
    fn test_convert_key_shifted_arrows() {
        let result = rs_terminal_convert_key(K_S_UP, 0);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_SHIFT);

        let result = rs_terminal_convert_key(K_S_DOWN, 0);
        assert_eq!(result.key, VTERM_KEY_DOWN);
        assert_eq!(result.modifiers, VTERM_MOD_SHIFT);
    }

    #[test]
    fn test_convert_key_ctrl_arrows() {
        let result = rs_terminal_convert_key(K_C_LEFT, 0);
        assert_eq!(result.key, VTERM_KEY_LEFT);
        assert_eq!(result.modifiers, VTERM_MOD_CTRL);

        let result = rs_terminal_convert_key(K_C_RIGHT, 0);
        assert_eq!(result.key, VTERM_KEY_RIGHT);
        assert_eq!(result.modifiers, VTERM_MOD_CTRL);
    }

    #[test]
    fn test_convert_key_function_keys() {
        let result = rs_terminal_convert_key(K_F1, 0);
        assert_eq!(result.key, vterm_key_function(1));

        let result = rs_terminal_convert_key(K_F12, 0);
        assert_eq!(result.key, vterm_key_function(12));

        // Shifted function key
        let result = rs_terminal_convert_key(K_S_F1, 0);
        assert_eq!(result.key, vterm_key_function(1));
        assert_eq!(result.modifiers, VTERM_MOD_SHIFT);
    }

    #[test]
    fn test_convert_key_keypad() {
        let result = rs_terminal_convert_key(K_K0, 0);
        assert_eq!(result.key, VTERM_KEY_KP_0);

        let result = rs_terminal_convert_key(K_KENTER, 0);
        assert_eq!(result.key, VTERM_KEY_KP_ENTER);
    }

    #[test]
    fn test_convert_key_with_modifiers() {
        // Shift modifier
        let result = rs_terminal_convert_key(K_UP, MOD_MASK_SHIFT);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_SHIFT);

        // Ctrl modifier
        let result = rs_terminal_convert_key(K_UP, MOD_MASK_CTRL);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_CTRL);

        // Alt modifier
        let result = rs_terminal_convert_key(K_UP, MOD_MASK_ALT);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_ALT);

        // Multiple modifiers
        let result = rs_terminal_convert_key(K_UP, MOD_MASK_CTRL | MOD_MASK_ALT);
        assert_eq!(result.key, VTERM_KEY_UP);
        assert_eq!(result.modifiers, VTERM_MOD_CTRL | VTERM_MOD_ALT);
    }

    #[test]
    fn test_convert_key_regular_char() {
        // Regular ASCII character returns VTERM_KEY_NONE
        let result = rs_terminal_convert_key(c_int::from(b'a'), 0);
        assert_eq!(result.key, VTERM_KEY_NONE);
    }

    #[test]
    fn test_filter_char() {
        assert_eq!(rs_terminal_is_filter_char(0), 1);
        assert_eq!(rs_terminal_is_filter_char(c_int::from(b'a')), 0);
        assert_eq!(rs_terminal_is_filter_char(c_int::from(b' ')), 0);
    }

    #[test]
    fn test_vterm_constants() {
        // Verify constant values are as expected
        assert_eq!(VTERM_KEY_NONE, 0);
        assert_eq!(VTERM_KEY_ENTER, 1);
        assert_eq!(VTERM_KEY_TAB, 2);
        assert_eq!(VTERM_KEY_KP_0, 16);
        assert_eq!(VTERM_KEY_KP_9, 25);
        assert_eq!(VTERM_KEY_MAX, 36);
    }

    #[test]
    fn test_cursor_shape_constants() {
        assert_eq!(VTERM_PROP_CURSORSHAPE_BLOCK, 1);
        assert_eq!(VTERM_PROP_CURSORSHAPE_UNDERLINE, 2);
        assert_eq!(VTERM_PROP_CURSORSHAPE_BAR_LEFT, 3);
    }

    #[test]
    fn test_update_invalid_region() {
        // First damage - use large initial values
        let region = rs_terminal_update_invalid_region(i32::MAX, -1, 5, 10);
        assert_eq!(region.start_row, 5);
        assert_eq!(region.end_row, 10);

        // Extend region with larger damage
        let region = rs_terminal_update_invalid_region(5, 10, 3, 15);
        assert_eq!(region.start_row, 3);
        assert_eq!(region.end_row, 15);

        // Damage within existing region
        let region = rs_terminal_update_invalid_region(3, 15, 5, 10);
        assert_eq!(region.start_row, 3);
        assert_eq!(region.end_row, 15);

        // Full invalidation request
        let region = rs_terminal_update_invalid_region(3, 15, -1, -1);
        assert_eq!(region.start_row, 3);
        assert_eq!(region.end_row, 15);
    }

    #[test]
    fn test_reset_invalid_region() {
        let region = rs_terminal_reset_invalid_region();
        assert_eq!(region.start_row, i32::MAX);
        assert_eq!(region.end_row, -1);
    }

    #[test]
    fn test_max_dimensions() {
        let dims = rs_terminal_max_dimensions(80, 24, 120, 30);
        assert_eq!(dims.width, 120);
        assert_eq!(dims.height, 30);

        let dims = rs_terminal_max_dimensions(120, 30, 80, 24);
        assert_eq!(dims.width, 120);
        assert_eq!(dims.height, 30);

        let dims = rs_terminal_max_dimensions(0, 0, 80, 24);
        assert_eq!(dims.width, 80);
        assert_eq!(dims.height, 24);
    }

    #[test]
    fn test_check_resize() {
        // Need resize - different dimensions
        let dims = rs_terminal_check_resize(80, 24, 120, 30);
        assert_eq!(dims.needs_resize, 1);
        assert_eq!(dims.width, 120);
        assert_eq!(dims.height, 30);

        // No resize needed - same dimensions
        let dims = rs_terminal_check_resize(80, 24, 80, 24);
        assert_eq!(dims.needs_resize, 0);

        // No resize - zero target
        let dims = rs_terminal_check_resize(80, 24, 0, 30);
        assert_eq!(dims.needs_resize, 0);

        let dims = rs_terminal_check_resize(80, 24, 80, 0);
        assert_eq!(dims.needs_resize, 0);
    }

    #[test]
    fn test_effective_scrollback() {
        assert_eq!(rs_terminal_effective_scrollback(-1), TERMINAL_SB_MAX);
        assert_eq!(rs_terminal_effective_scrollback(0), TERMINAL_SB_MAX);
        assert_eq!(rs_terminal_effective_scrollback(1000), 1000);
        assert_eq!(rs_terminal_effective_scrollback(50000), 50000);
    }

    #[test]
    fn test_scrollback_lines_to_delete() {
        // Need to delete lines
        assert_eq!(rs_terminal_scrollback_lines_to_delete(1000, 500), 500);
        assert_eq!(rs_terminal_scrollback_lines_to_delete(100, 50), 50);

        // No deletion needed
        assert_eq!(rs_terminal_scrollback_lines_to_delete(500, 1000), 0);
        assert_eq!(rs_terminal_scrollback_lines_to_delete(500, 500), 0);
    }

    #[test]
    fn test_scrollback_is_full() {
        assert_eq!(rs_terminal_scrollback_is_full(1000, 1000), 1);
        assert_eq!(rs_terminal_scrollback_is_full(500, 1000), 0);
        assert_eq!(rs_terminal_scrollback_is_full(0, 1000), 0);
    }

    #[test]
    fn test_scrollback_insert_index() {
        assert_eq!(rs_terminal_scrollback_insert_index(100, 24), 76);
        assert_eq!(rs_terminal_scrollback_insert_index(50, 24), 26);
        assert_eq!(rs_terminal_scrollback_insert_index(24, 24), 0);
    }

    #[test]
    fn test_vterm_rect_default() {
        let rect = VTermRect::default();
        assert_eq!(rect.start_row, 0);
        assert_eq!(rect.end_row, 0);
        assert_eq!(rect.start_col, 0);
        assert_eq!(rect.end_col, 0);
    }

    #[test]
    fn test_vterm_pos_default() {
        let pos = VTermPos::default();
        assert_eq!(pos.row, 0);
        assert_eq!(pos.col, 0);
    }

    #[test]
    fn test_moverect_damage() {
        // Destination before source
        let dest = VTermRect {
            start_row: 0,
            end_row: 5,
            start_col: 0,
            end_col: 80,
        };
        let src = VTermRect {
            start_row: 5,
            end_row: 10,
            start_col: 0,
            end_col: 80,
        };
        let region = rs_terminal_moverect_damage(dest, src);
        assert_eq!(region.start_row, 0);
        assert_eq!(region.end_row, 10);

        // Source before destination
        let dest = VTermRect {
            start_row: 10,
            end_row: 20,
            start_col: 0,
            end_col: 80,
        };
        let src = VTermRect {
            start_row: 5,
            end_row: 15,
            start_col: 0,
            end_col: 80,
        };
        let region = rs_terminal_moverect_damage(dest, src);
        assert_eq!(region.start_row, 5);
        assert_eq!(region.end_row, 20);
    }

    #[test]
    fn test_prop_needs_invalidate() {
        // Altscreen - handled but no invalidation
        let result = rs_terminal_prop_needs_invalidate(VTERM_PROP_ALTSCREEN);
        assert_eq!(result.handled, 1);
        assert_eq!(result.invalidate, 0);
        assert_eq!(result.cursor_pending, 0);

        // Cursor visible - invalidates
        let result = rs_terminal_prop_needs_invalidate(VTERM_PROP_CURSORVISIBLE);
        assert_eq!(result.handled, 1);
        assert_eq!(result.invalidate, 1);
        assert_eq!(result.cursor_pending, 0);

        // Cursor blink - invalidates and sets cursor pending
        let result = rs_terminal_prop_needs_invalidate(VTERM_PROP_CURSORBLINK);
        assert_eq!(result.handled, 1);
        assert_eq!(result.invalidate, 1);
        assert_eq!(result.cursor_pending, 1);

        // Cursor shape - invalidates and sets cursor pending
        let result = rs_terminal_prop_needs_invalidate(VTERM_PROP_CURSORSHAPE);
        assert_eq!(result.handled, 1);
        assert_eq!(result.invalidate, 1);
        assert_eq!(result.cursor_pending, 1);

        // Title - handled but no invalidation
        let result = rs_terminal_prop_needs_invalidate(VTERM_PROP_TITLE);
        assert_eq!(result.handled, 1);
        assert_eq!(result.invalidate, 0);

        // Unknown property
        let result = rs_terminal_prop_needs_invalidate(99);
        assert_eq!(result.handled, 0);
        assert_eq!(result.invalidate, 0);
    }

    #[test]
    fn test_movecursor_handled() {
        assert_eq!(rs_terminal_movecursor_handled(5, 10, 0, 0), 1);
        assert_eq!(rs_terminal_movecursor_handled(0, 0, 5, 10), 1);
    }

    #[test]
    fn test_sb_pop_cols() {
        assert_eq!(rs_terminal_sb_pop_cols(80, 100), 80);
        assert_eq!(rs_terminal_sb_pop_cols(100, 80), 80);
        assert_eq!(rs_terminal_sb_pop_cols(80, 80), 80);
    }

    #[test]
    fn test_is_dark_theme() {
        assert_eq!(rs_terminal_is_dark_theme(b'd'), 1);
        assert_eq!(rs_terminal_is_dark_theme(b'l'), 0);
        assert_eq!(rs_terminal_is_dark_theme(b'x'), 0);
    }

    #[test]
    fn test_vterm_prop_constants() {
        // Values match `VTermProp` enum in vterm_defs.h
        assert_eq!(VTERM_PROP_CURSORVISIBLE, 1);
        assert_eq!(VTERM_PROP_CURSORBLINK, 2);
        assert_eq!(VTERM_PROP_ALTSCREEN, 3);
        assert_eq!(VTERM_PROP_TITLE, 4);
        assert_eq!(VTERM_PROP_CURSORSHAPE, 7);
        assert_eq!(VTERM_PROP_MOUSE, 8);
    }

    #[test]
    fn test_row_to_linenr() {
        // sb_current = 10, row = 0 => linenr = 11
        assert_eq!(rs_terminal_row_to_linenr(0, 10), 11);
        // sb_current = 0, row = 0 => linenr = 1
        assert_eq!(rs_terminal_row_to_linenr(0, 0), 1);
        // sb_current = 5, row = 3 => linenr = 9
        assert_eq!(rs_terminal_row_to_linenr(3, 5), 9);
        // INT_MAX stays INT_MAX
        assert_eq!(rs_terminal_row_to_linenr(i32::MAX, 10), i32::MAX);
    }

    #[test]
    fn test_linenr_to_row() {
        // sb_current = 10, linenr = 11 => row = 0
        assert_eq!(rs_terminal_linenr_to_row(11, 10), 0);
        // sb_current = 0, linenr = 1 => row = 0
        assert_eq!(rs_terminal_linenr_to_row(1, 0), 0);
        // sb_current = 5, linenr = 9 => row = 3
        assert_eq!(rs_terminal_linenr_to_row(9, 5), 3);
    }

    #[test]
    fn test_mouse_button_conversion() {
        // Left mouse
        let result = rs_terminal_convert_mouse_button(K_LEFTMOUSE);
        assert_eq!(result.button, 1);
        assert_eq!(result.pressed, 1);

        let result = rs_terminal_convert_mouse_button(K_LEFTRELEASE);
        assert_eq!(result.button, 1);
        assert_eq!(result.pressed, 0);

        let result = rs_terminal_convert_mouse_button(K_LEFTDRAG);
        assert_eq!(result.button, 1);
        assert_eq!(result.pressed, 1);

        // Middle mouse
        let result = rs_terminal_convert_mouse_button(K_MIDDLEMOUSE);
        assert_eq!(result.button, 2);
        assert_eq!(result.pressed, 1);

        // Right mouse
        let result = rs_terminal_convert_mouse_button(K_RIGHTMOUSE);
        assert_eq!(result.button, 3);
        assert_eq!(result.pressed, 1);

        // Scroll
        let result = rs_terminal_convert_mouse_button(K_MOUSEDOWN);
        assert_eq!(result.button, 4);
        assert_eq!(result.pressed, 1);

        let result = rs_terminal_convert_mouse_button(K_MOUSEUP);
        assert_eq!(result.button, 5);
        assert_eq!(result.pressed, 1);

        // Mouse move
        let result = rs_terminal_convert_mouse_button(K_MOUSEMOVE);
        assert_eq!(result.button, 0);
        assert_eq!(result.pressed, 0);

        // Unknown key
        let result = rs_terminal_convert_mouse_button(0);
        assert_eq!(result.button, -1);
    }

    #[test]
    fn test_filter_char_detailed() {
        // Test various filter flags
        assert_eq!(rs_terminal_should_filter_char(0x08, TPF_BS), 1); // Backspace
        assert_eq!(rs_terminal_should_filter_char(0x08, 0), 0); // Backspace without flag
        assert_eq!(rs_terminal_should_filter_char(0x09, TPF_HT), 1); // Tab
        assert_eq!(rs_terminal_should_filter_char(0x0C, TPF_FF), 1); // Form feed
        assert_eq!(rs_terminal_should_filter_char(0x1B, TPF_ESC), 1); // Escape
        assert_eq!(rs_terminal_should_filter_char(0x7F, TPF_DEL), 1); // DEL

        // C0 control characters (0x01-0x1F excluding specific ones)
        assert_eq!(rs_terminal_should_filter_char(0x01, TPF_C0), 1);
        assert_eq!(rs_terminal_should_filter_char(0x1F, TPF_C0), 1);

        // C1 control characters (0x80-0x9F)
        assert_eq!(rs_terminal_should_filter_char(0x80, TPF_C1), 1);
        assert_eq!(rs_terminal_should_filter_char(0x9F, TPF_C1), 1);

        // Normal characters shouldn't be filtered
        assert_eq!(rs_terminal_should_filter_char(c_int::from(b'a'), 0xFFFF), 0);
        assert_eq!(rs_terminal_should_filter_char(c_int::from(b' '), 0xFFFF), 0);

        // Newline and carriage return are never filtered
        assert_eq!(rs_terminal_should_filter_char(0x0A, 0xFFFF), 0);
        assert_eq!(rs_terminal_should_filter_char(0x0D, 0xFFFF), 0);
    }
}
