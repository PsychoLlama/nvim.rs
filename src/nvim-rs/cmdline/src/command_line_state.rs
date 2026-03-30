//! CommandLineState: Rust-owned repr(C) struct matching C `CommandLineState`.
//!
//! This struct mirrors the `CommandLineState` typedef in `ex_getln.c` exactly.
//! All accesses from Rust go through this struct directly; no C accessor
//! wrappers needed.

#![allow(unsafe_code)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::{c_char, c_int, c_void};

use nvim_cmdexpand::ExpandT;

use crate::search::IncsearchStateT;

// =============================================================================
// VimState (matches C struct vim_state)
// =============================================================================

/// Opaque type matching the C `VimState` struct (2 function pointers = 16 bytes).
#[repr(C)]
pub struct VimState {
    pub check: Option<unsafe extern "C" fn(*mut VimState) -> c_int>,
    pub execute: Option<unsafe extern "C" fn(*mut VimState, c_int) -> c_int>,
}

// =============================================================================
// CommandLineState
// =============================================================================

/// Rust mirror of C `CommandLineState` (ex_getln.c).
///
/// Layout verified by a static_assert in ex_getln.c. Fields must match
/// the C struct exactly in order and alignment.
///
/// # Safety
///
/// This struct is `repr(C)` and matches the C layout. Pointer fields may be
/// null at various points during the function's lifecycle.
#[repr(C)]
pub struct CommandLineState {
    /// VimState state machine callbacks (must be first field).
    pub state: VimState, // @0, 16 bytes
    /// First character of command line (prompt type: ':' '/' '?' '=' '@' '>').
    pub firstc: c_int, // @16
    /// Count argument (used for incremental search).
    pub count: c_int, // @20
    /// Indent for inside conditionals.
    pub indent: c_int, // @24
    /// Current key code being processed.
    pub c: c_int, // @28
    /// True when `<ESC>` just typed.
    pub gotesc: bool, // @32
    /// When true, check for abbreviations.
    pub do_abbr: bool, // @33
    _pad0: [u8; 6],
    /// String to match against history (for browsing).
    pub lookfor: *mut c_char, // @40
    /// Length of lookfor string.
    pub lookforlen: c_int, // @48
    /// Current history line in use.
    pub hiscnt: c_int, // @52
    /// History line before attempting to jump to next match.
    pub save_hiscnt: c_int, // @56
    /// History type to be used.
    pub histype: c_int, // @60
    /// Incremental search state.
    pub is_state: IncsearchStateT, // @64, 116 bytes
    /// Did wild_list() recently.
    pub did_wild_list: bool, // @180
    _pad1: [u8; 3],
    /// Index in wim_flags[].
    pub wim_index: c_int, // @184
    /// Saved msg_scroll value.
    pub save_msg_scroll: c_int, // @188
    /// Saved State value when called.
    pub save_state: c_int, // @192
    /// Previous command position (for CursorMoveD detection).
    pub prev_cmdpos: c_int, // @196
    /// Previous cmdbuff copy (for CmdlineChanged detection).
    pub prev_cmdbuff: *mut c_char, // @200
    /// Saved value of 'inccommand' option.
    pub save_p_icm: *mut c_char, // @208
    /// Skip pum redraw on next wildmenu removal.
    pub skip_pum_redraw: bool, // @216
    /// One of the keys was typed (not from mapping).
    pub some_key_typed: bool, // @217
    /// Ignore mouse drag and release events until mouse down.
    pub ignore_drag_release: bool, // @218
    /// Break on Ctrl-C even in try/catch.
    pub break_ctrl_c: bool, // @219
    _pad1b: [u8; 4], // padding before xpc (align to 8 bytes for ptr fields)
    /// Command expansion state.
    pub xpc: ExpandT, // @224, 392 bytes
    /// Pointer to b_p_iminsert or b_p_imsearch.
    pub b_im_ptr: *mut i64, // @616
    /// Buffer where b_im_ptr is valid.
    pub b_im_ptr_buf: *mut c_void, // @624
    /// Cmdline type for events (firstc or '-').
    pub cmdline_type: c_int, // @632
    /// Whether CmdlineLeavePre autocmd was triggered.
    pub event_cmdlineleavepre_triggered: bool, // @636
    /// Whether history navigation occurred.
    pub did_hist_navigate: bool, // @637
}

// Compile-time size check: must match C static_asserts.
const _: () = {
    assert!(std::mem::size_of::<CommandLineState>() == 640);
};
