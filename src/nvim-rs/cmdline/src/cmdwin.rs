//! Command-line window functionality
//!
//! This module provides types and utilities for the command-line window (q:, q/, q?),
//! which allows editing command history in a regular window.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]

use std::ffi::c_int;

// =============================================================================
// Command Window Type
// =============================================================================

/// Type of command-line window.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CmdwinType {
    /// Not in command-line window
    #[default]
    None = 0,
    /// Ex command history (q:)
    Ex = b':' as i32,
    /// Forward search history (q/)
    ForwardSearch = b'/' as i32,
    /// Backward search history (q?)
    BackwardSearch = b'?' as i32,
    /// Expression history (q=)
    Expression = b'=' as i32,
    /// Input history (q@)
    Input = b'@' as i32,
    /// Debug history (q>)
    Debug = b'>' as i32,
}

impl CmdwinType {
    /// Parse from character.
    #[must_use]
    pub const fn from_char(c: i32) -> Self {
        match c {
            c if c == b':' as i32 => Self::Ex,
            c if c == b'/' as i32 => Self::ForwardSearch,
            c if c == b'?' as i32 => Self::BackwardSearch,
            c if c == b'=' as i32 => Self::Expression,
            c if c == b'@' as i32 => Self::Input,
            c if c == b'>' as i32 => Self::Debug,
            _ => Self::None,
        }
    }

    /// Get character representation.
    #[must_use]
    pub const fn to_char(self) -> Option<u8> {
        match self {
            Self::None => None,
            Self::Ex => Some(b':'),
            Self::ForwardSearch => Some(b'/'),
            Self::BackwardSearch => Some(b'?'),
            Self::Expression => Some(b'='),
            Self::Input => Some(b'@'),
            Self::Debug => Some(b'>'),
        }
    }

    /// Check if this is a search type.
    #[must_use]
    pub const fn is_search(self) -> bool {
        matches!(self, Self::ForwardSearch | Self::BackwardSearch)
    }

    /// Check if command window is active.
    #[must_use]
    pub const fn is_active(self) -> bool {
        !matches!(self, Self::None)
    }
}

// =============================================================================
// Command Window State
// =============================================================================

/// State for command-line window.
#[derive(Debug, Clone, Copy, Default)]
pub struct CmdwinState {
    /// Type of command-line window.
    pub win_type: CmdwinType,
    /// Command-line level when opened.
    pub level: i32,
    /// Result when closing (key code or 0).
    pub result: i32,
}

impl CmdwinState {
    /// Create a new command window state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            win_type: CmdwinType::None,
            level: 0,
            result: 0,
        }
    }

    /// Check if command window is active.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.win_type.is_active()
    }

    /// Initialize for opening command window.
    pub fn open(&mut self, win_type: CmdwinType, level: i32) {
        self.win_type = win_type;
        self.level = level;
        self.result = 0;
    }

    /// Close command window.
    pub fn close(&mut self, result: i32) {
        self.result = result;
        self.win_type = CmdwinType::None;
        self.level = 0;
    }

    /// Reset state.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

// =============================================================================
// Result Codes
// =============================================================================

/// Result codes for command window closure.
pub mod result {
    use std::ffi::c_int;

    /// Closed normally with Enter.
    pub const ENTER: c_int = 13; // CR

    /// Closed with Ctrl-C (abort).
    pub const CTRL_C: c_int = 3;

    /// Closed with ESC (cancel).
    pub const ESC: c_int = 27;

    /// Closed with K_IGNORE (ignore).
    pub const IGNORE: c_int = -1;

    /// Check if result means execute the line.
    #[must_use]
    pub const fn should_execute(r: c_int) -> bool {
        r == ENTER
    }

    /// Check if result means cancel.
    #[must_use]
    pub const fn should_cancel(r: c_int) -> bool {
        r == CTRL_C || r == ESC
    }
}

// =============================================================================
// Open Restrictions
// =============================================================================

/// Reasons why command window cannot be opened.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdwinOpenError {
    /// Can be opened.
    Ok = 0,
    /// Already in command window.
    AlreadyInCmdwin = 1,
    /// Text or buffer is locked.
    TextLocked = 2,
    /// In secret mode (cmdline_star).
    SecretMode = 3,
    /// No room for window.
    NoRoom = 4,
}

impl CmdwinOpenError {
    /// Check if opening is allowed.
    #[must_use]
    pub const fn can_open(self) -> bool {
        matches!(self, Self::Ok)
    }
}

// =============================================================================
// History Type Mapping
// =============================================================================

/// Map command window type to history type.
#[must_use]
pub const fn cmdwin_to_hist_type(win_type: CmdwinType) -> i32 {
    // History type constants from cmdhist.h
    const HIST_CMD: i32 = 0;
    const HIST_SEARCH: i32 = 1;
    const HIST_EXPR: i32 = 2;
    const HIST_INPUT: i32 = 3;
    const HIST_DEBUG: i32 = 4;

    match win_type {
        CmdwinType::ForwardSearch | CmdwinType::BackwardSearch => HIST_SEARCH,
        CmdwinType::Expression => HIST_EXPR,
        CmdwinType::Input => HIST_INPUT,
        CmdwinType::Debug => HIST_DEBUG,
        CmdwinType::Ex | CmdwinType::None => HIST_CMD,
    }
}

// =============================================================================
// Command Window Open Validation
// =============================================================================

/// Check if command window can be opened based on current state.
///
/// Returns an error code if it cannot be opened, or Ok if it can.
#[must_use]
pub const fn can_open_cmdwin(
    cmdwin_type_active: bool,
    text_locked: bool,
    cmdline_star: i32,
) -> CmdwinOpenError {
    if cmdwin_type_active {
        return CmdwinOpenError::AlreadyInCmdwin;
    }
    if text_locked {
        return CmdwinOpenError::TextLocked;
    }
    if cmdline_star > 0 {
        return CmdwinOpenError::SecretMode;
    }
    CmdwinOpenError::Ok
}

/// Check if window split validation failed.
///
/// After win_split(), check if autocommands messed with the old window.
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub const fn cmdwin_split_invalid(
    old_curwin_valid: bool,
    curwin_is_old: bool,
    old_curbuf_valid: bool,
    buf_changed: bool,
) -> bool {
    !old_curwin_valid || curwin_is_old || !old_curbuf_valid || buf_changed
}

/// Check if buffer creation for cmdwin failed.
#[must_use]
#[allow(clippy::fn_params_excessive_bools)]
pub const fn cmdwin_buffer_invalid(
    newbuf_status_ok: bool,
    cmdwin_valid: bool,
    curwin_is_cmdwin: bool,
    old_curwin_valid: bool,
    old_curbuf_valid: bool,
    buf_changed: bool,
) -> bool {
    !newbuf_status_ok
        || !cmdwin_valid
        || !curwin_is_cmdwin
        || !old_curwin_valid
        || !old_curbuf_valid
        || buf_changed
}

// =============================================================================
// Command Window Tab Mapping
// =============================================================================

/// Check if Tab key should be mapped for completion in cmdwin.
///
/// Tab completion mapping is added for Ex and Debug command windows.
#[must_use]
pub const fn cmdwin_needs_tab_mapping(histtype: i32, p_wc: i32) -> bool {
    // TAB = 9
    const TAB: i32 = 9;
    // HIST_CMD = 0, HIST_DEBUG = 4
    const HIST_CMD: i32 = 0;
    const HIST_DEBUG: i32 = 4;

    if p_wc != TAB {
        return false;
    }
    histtype == HIST_CMD || histtype == HIST_DEBUG
}

/// Check if cmdwin should set vim filetype.
///
/// Ex and Debug command windows get vim filetype for syntax highlighting.
#[must_use]
pub const fn cmdwin_needs_vim_filetype(histtype: i32) -> bool {
    const HIST_CMD: i32 = 0;
    const HIST_DEBUG: i32 = 4;
    histtype == HIST_CMD || histtype == HIST_DEBUG
}

// =============================================================================
// Command Window Cleanup Validation
// =============================================================================

/// Check if command window cleanup detected an error (window/buffer changed).
#[must_use]
pub const fn cmdwin_cleanup_had_error(
    old_curwin_valid: bool,
    old_curbuf_valid: bool,
    buf_changed: bool,
) -> bool {
    !old_curwin_valid || !old_curbuf_valid || buf_changed
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Check if command window can be opened (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_can_open(
    cmdwin_type_active: c_int,
    text_locked: c_int,
    cmdline_star: c_int,
) -> c_int {
    can_open_cmdwin(cmdwin_type_active != 0, text_locked != 0, cmdline_star) as c_int
}

/// Check if split validation failed (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_split_invalid(
    old_curwin_valid: c_int,
    curwin_is_old: c_int,
    old_curbuf_valid: c_int,
    buf_changed: c_int,
) -> c_int {
    c_int::from(cmdwin_split_invalid(
        old_curwin_valid != 0,
        curwin_is_old != 0,
        old_curbuf_valid != 0,
        buf_changed != 0,
    ))
}

/// Check if buffer creation validation failed (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_buffer_invalid(
    newbuf_status_ok: c_int,
    cmdwin_valid: c_int,
    curwin_is_cmdwin: c_int,
    old_curwin_valid: c_int,
    old_curbuf_valid: c_int,
    buf_changed: c_int,
) -> c_int {
    c_int::from(cmdwin_buffer_invalid(
        newbuf_status_ok != 0,
        cmdwin_valid != 0,
        curwin_is_cmdwin != 0,
        old_curwin_valid != 0,
        old_curbuf_valid != 0,
        buf_changed != 0,
    ))
}

/// Check if Tab mapping is needed for cmdwin (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_needs_tab_mapping(histtype: c_int, p_wc: c_int) -> c_int {
    c_int::from(cmdwin_needs_tab_mapping(histtype, p_wc))
}

/// Check if vim filetype is needed for cmdwin (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_needs_vim_filetype(histtype: c_int) -> c_int {
    c_int::from(cmdwin_needs_vim_filetype(histtype))
}

/// Check if cleanup detected an error (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_cleanup_had_error(
    old_curwin_valid: c_int,
    old_curbuf_valid: c_int,
    buf_changed: c_int,
) -> c_int {
    c_int::from(cmdwin_cleanup_had_error(
        old_curwin_valid != 0,
        old_curbuf_valid != 0,
        buf_changed != 0,
    ))
}

/// Get command window type from char (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_type_from_char_v2(c: c_int) -> c_int {
    CmdwinType::from_char(c) as c_int
}

/// Check if command window type is active (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_is_active(win_type: c_int) -> c_int {
    c_int::from(CmdwinType::from_char(win_type).is_active())
}

/// Check if command window type is search (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_is_search(win_type: c_int) -> c_int {
    c_int::from(CmdwinType::from_char(win_type).is_search())
}

/// Get history type for command window type (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_to_hist_type(win_type: c_int) -> c_int {
    cmdwin_to_hist_type(CmdwinType::from_char(win_type))
}

/// Check if result means execute (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_result_should_execute(result: c_int) -> c_int {
    c_int::from(result::should_execute(result))
}

/// Check if result means cancel (FFI).
#[no_mangle]
pub extern "C" fn rs_cmdwin_result_should_cancel(result: c_int) -> c_int {
    c_int::from(result::should_cancel(result))
}

// =============================================================================
// nvim_open_cmdwin: Full Rust port of the command-line window lifecycle
// =============================================================================

use std::ffi::{c_char, c_uint};

// Opaque C pointer types used via FFI
type WinHandle = *mut std::ffi::c_void;
type BufHandle = *mut std::ffi::c_void;

// Key constants (matching C keycodes.h)
const CTRL_C: c_int = 3;
const CAR: c_int = 13;
const K_IGNORE: c_int = {
    // termcap2key(KS_EXTRA=2, KE_IGNORE=53): -(256 * 53 + 2) = -13570 ... let's use C-computed value
    // Actually: termcap2key = K_SPECIAL + extra*256 + ... no.
    // In nvim: K_SPECIAL = 0x80 = 128; termcap2key(a,b) = -(a + b*256)
    // KS_EXTRA = 2; KE_IGNORE = 53 => -(2 + 53*256) = -(2 + 13568) = -13570
    -(2 + 53 * 256)
};
const K_NOP: c_int = {
    // KE_NOP = 97 => -(2 + 97*256) = -(2 + 24832) = -24834
    -(2 + 97 * 256)
};
const K_XF1: c_int = {
    // KE_XF1 = 57 => -(2 + 57*256) = -(2 + 14592) = -14594
    -(2 + 57 * 256)
};
const K_XF2: c_int = {
    // KE_XF2 = 58 => -(2 + 58*256) = -(2 + 14848) = -14850
    -(2 + 58 * 256)
};

// C return codes
const OK: c_int = 1;
const FAIL: c_int = 0;

// HIST constants
const HIST_INVALID: c_int = -1;
const HIST_CMD: c_int = 0;

// MODE_NORMAL
const MODE_NORMAL: c_int = 0x01;

unsafe extern "C" {
    // ccline accessors (many already declared in state.rs/keys.rs; we re-declare here)
    fn nvim_get_ccline_level() -> c_int;
    fn nvim_get_cmdline_type() -> c_int;
    fn nvim_get_ccline_cmdbuff() -> *mut c_char;
    fn nvim_get_ccline_cmdpos() -> c_int;
    fn nvim_get_ccline_cmdlen() -> c_int;
    fn nvim_set_ccline_cmdpos(pos: c_int);
    fn nvim_set_ccline_cmdbuff(buff: *mut c_char);
    fn nvim_set_ccline_cmdspos(spos: c_int);
    fn nvim_set_ccline_redraw_state(state: c_int);

    // Global state getters/setters (from cmdwin_shim.c and other shims)
    fn nvim_get_cmdwin_type() -> c_int;
    fn nvim_set_cmdwin_type(val: c_int);
    fn nvim_set_cmdwin_level(val: c_int);
    fn nvim_set_cmdwin_win(wp: WinHandle);
    fn nvim_set_cmdwin_old_curwin(wp: WinHandle);
    fn nvim_set_cmdwin_buf(buf: BufHandle);
    fn nvim_get_cmdwin_result() -> c_int;
    fn nvim_set_cmdwin_result(val: c_int);
    fn nvim_get_restart_edit() -> c_int;
    fn nvim_set_restart_edit(val: c_int);
    fn nvim_get_State() -> c_int;
    fn nvim_set_State(val: c_int);
    fn nvim_get_exmode_active() -> c_int;
    fn nvim_set_exmode_active(val: c_int);
    fn nvim_get_cmdmsg_rl() -> c_int;
    fn nvim_set_cmdmsg_rl(val: c_int);
    fn nvim_set_got_int(val: c_int);
    fn nvim_set_need_wait_return(val: c_int);
    fn nvim_get_RedrawingDisabled() -> c_int;
    fn nvim_set_RedrawingDisabled(val: c_int);
    fn nvim_get_KeyTyped() -> bool;
    fn nvim_set_KeyTyped(val: c_int);
    fn nvim_set_skip_win_fix_cursor(val: c_int);
    fn nvim_set_cmdline_was_last_drawn(val: c_int);

    // cmdmod
    fn nvim_set_cmdmod_tab_zero();
    fn nvim_set_cmdmod_noswapfile();

    // curwin / curbuf field accessors
    fn nvim_get_curwin_ptr() -> WinHandle;
    fn nvim_curwin_set_w_p_fen(val: c_int);
    fn nvim_curwin_set_w_p_rl(val: c_int);
    fn nvim_curwin_set_w_p_cole(val: c_int);
    fn nvim_curwin_set_cursor_lnum(lnum: c_int);
    fn nvim_curwin_set_cursor_col(col: c_uint);
    fn nvim_curbuf_set_b_p_ma(v: c_int);
    fn nvim_curbuf_set_b_p_tw(v: i64);
    fn nvim_curbuf_inc_ro_locked();
    fn nvim_curbuf_dec_ro_locked();
    fn nvim_curbuf_get_ml_line_count() -> c_int;
    fn nvim_get_curbuf_ptr() -> BufHandle;

    // Window / buffer management
    fn nvim_win_split_bot(height: c_int) -> c_int;
    fn nvim_buf_open_scratch_cmdwin() -> c_int;
    fn nvim_win_close_if_valid_not_last(wp: WinHandle) -> c_int;
    fn nvim_win_close_cmdwin(wp: WinHandle);
    fn nvim_close_buffer_wipe_if_valid(bufref: *mut std::ffi::c_void, curbuf: BufHandle);
    #[link_name = "win_goto"]
    fn nvim_win_goto_cmdwin(wp: WinHandle);
    fn nvim_normal_enter_cmdwin();

    // Rust win/buf helpers
    fn rs_win_valid(win: WinHandle) -> c_int;
    fn rs_win_size_save(gap: *mut std::ffi::c_void);
    fn rs_win_size_restore(gap: *mut std::ffi::c_void);
    fn rs_clear_showcmd();

    // Heap-allocated opaque handles
    fn nvim_alloc_garray() -> *mut std::ffi::c_void;
    fn nvim_free_garray(gap: *mut std::ffi::c_void);
    fn nvim_alloc_bufref() -> *mut std::ffi::c_void;
    fn nvim_free_bufref(br: *mut std::ffi::c_void);
    fn nvim_set_bufref_cmdwin(br: *mut std::ffi::c_void, buf: BufHandle);
    fn nvim_bufref_valid_cmdwin(br: *mut std::ffi::c_void) -> c_int;
    fn nvim_bufref_get_buf_cmdwin(br: *mut std::ffi::c_void) -> BufHandle;

    // text_or_buf_locked / cmdline_star
    fn nvim_text_or_buf_locked() -> c_int;
    fn nvim_get_cmdline_star() -> c_int;

    // beep / pum
    #[link_name = "beep_flush"]
    fn nvim_beep_flush();
    fn nvim_pum_undisplay_true();

    // win_T field accessor
    fn nvim_win_get_w_buffer(wp: WinHandle) -> BufHandle;

    // Autocommands
    fn nvim_trigger_cmdwinenter();
    fn nvim_trigger_cmdwinleave();

    // Option setters
    fn nvim_set_opt_bufhidden_wipe();
    fn nvim_set_opt_filetype_vim();

    // Mapping
    fn nvim_add_tab_map_insert();
    fn nvim_add_tab_map_normal();

    // History
    fn nvim_init_history_and_get_hislen() -> c_int;
    fn nvim_get_hisidx(histtype: c_int) -> c_int;
    fn nvim_get_histentry_str(histtype: c_int, i: c_int) -> *const c_char;

    // Buffer / line r/w
    fn nvim_ml_replace_last_with_cmdbuff();
    fn nvim_ml_append_cmdwin(lnum: c_int, line: *const c_char) -> c_int;

    // UI / redraw
    #[link_name = "changed_line_abv_curs"]
    fn nvim_changed_line_abv_curs_cmdwin();
    fn nvim_invalidate_botline_curwin();
    fn nvim_redraw_later_curwin_some_valid();
    fn nvim_ui_has_cmdline() -> c_int;
    fn nvim_ui_call_cmdline_hide_ccline(do_flush: c_int);
    fn nvim_cmd_screencol_cmdpos() -> c_int;
    #[link_name = "redrawcmd"]
    fn nvim_redrawcmd();

    // Getchar / typeahead
    #[link_name = "stuffcharReadbuff"]
    fn nvim_stuffcharReadbuff_cmdwin(c: c_int);

    // Clipboard batch count
    #[link_name = "save_batch_count"]
    fn nvim_save_batch_count() -> c_int;
    #[link_name = "restore_batch_count"]
    fn nvim_restore_batch_count(save_count: c_int);

    // cmdline buffer management
    #[link_name = "dealloc_cmdbuff"]
    fn nvim_dealloc_cmdbuff();

    // Misc
    fn nvim_aborting() -> c_int;
    fn nvim_set_ccline_cmdpos_from_cursor();
    fn nvim_emsg_cmdwin_changed();
    fn may_trigger_modechanged();
    fn setmouse();
    #[link_name = "setcursor"]
    fn nvim_setcursor();

    // Result extraction helpers
    fn nvim_set_ccline_cmdbuff_qa(with_bang: c_int);
    fn nvim_stuff_qa_into_readbuff(with_bang: c_int);
    fn nvim_set_ccline_cmdbuff_empty();
    fn nvim_set_ccline_cmdbuff_from_cursor();

    // p_cwh / p_wc
    fn nvim_get_p_cwh() -> c_int;
    fn nvim_cmdexpand_get_p_wc() -> c_int;
}

/// Open a command-line window for the current command-line type.
/// Rust port of `nvim_open_cmdwin` from cmdwin.c.
///
/// # Safety
///
/// Calls many C FFI functions that manipulate global editor state.
/// Must only be called from the main editor thread.
#[allow(clippy::must_use_candidate)]
#[allow(clippy::too_many_lines)]
#[unsafe(export_name = "nvim_open_cmdwin")]
pub unsafe extern "C" fn rs_nvim_open_cmdwin() -> c_int {
    // Early exit: can't open while already in cmdwin, text locked, or in password mode.
    if rs_cmdwin_can_open(
        c_int::from(nvim_get_cmdwin_type() != 0),
        nvim_text_or_buf_locked(),
        nvim_get_cmdline_star(),
    ) != 0
    {
        nvim_beep_flush();
        return K_IGNORE;
    }

    // Remember old_curwin and old_curbuf.
    let old_curwin = nvim_get_curwin_ptr();
    let old_curbuf_ref = nvim_alloc_bufref();
    nvim_set_bufref_cmdwin(old_curbuf_ref, nvim_get_curbuf_ptr());

    // Save current window sizes.
    let winsizes = nvim_alloc_garray();
    rs_win_size_save(winsizes);

    // Dismiss popup menu.
    nvim_pum_undisplay_true();

    // Don't use a new tab page.
    nvim_set_cmdmod_tab_zero();
    nvim_set_cmdmod_noswapfile();

    // Save state that we restore later.
    let save_restart_edit = nvim_get_restart_edit();
    let save_state = nvim_get_State();
    let save_exmode = nvim_get_exmode_active();
    let save_cmdmsg_rl = nvim_get_cmdmsg_rl();

    // Create a split window at the bottom.
    if nvim_win_split_bot(nvim_get_p_cwh()) == FAIL {
        nvim_beep_flush();
        nvim_free_garray(winsizes);
        nvim_free_bufref(old_curbuf_ref);
        return K_IGNORE;
    }

    // win_split() autocommands may have invalidated old_curwin / old_curbuf.
    let old_curbuf_valid = nvim_bufref_valid_cmdwin(old_curbuf_ref);
    let old_curbuf_buf = nvim_bufref_get_buf_cmdwin(old_curbuf_ref);
    let old_curwin_buf = if rs_win_valid(old_curwin) != 0 {
        nvim_win_get_w_buffer(old_curwin)
    } else {
        std::ptr::null_mut()
    };
    let curwin_is_old = nvim_get_curwin_ptr() == old_curwin;

    if rs_cmdwin_split_invalid(
        rs_win_valid(old_curwin),
        c_int::from(curwin_is_old),
        old_curbuf_valid,
        c_int::from(!old_curwin_buf.is_null() && old_curwin_buf != old_curbuf_buf),
    ) != 0
    {
        nvim_beep_flush();
        nvim_free_garray(winsizes);
        nvim_free_bufref(old_curbuf_ref);
        return CTRL_C;
    }

    // Don't let quitting the More prompt abort this.
    nvim_set_got_int(0);

    // Set cmdwin variables before any autocommands.
    nvim_set_cmdwin_type(nvim_get_cmdline_type());
    nvim_set_cmdwin_level(nvim_get_ccline_level());
    nvim_set_cmdwin_win(nvim_get_curwin_ptr());
    nvim_set_cmdwin_old_curwin(old_curwin);

    // Create empty scratch buffer. Be especially cautious of BufLeave autocommands.
    let newbuf_status = nvim_buf_open_scratch_cmdwin();
    let cmdwin_win = nvim_get_curwin_ptr(); // snapshot: may differ after autocommands
    let cmdwin_valid = rs_win_valid(cmdwin_win);
    let curwin_after = nvim_get_curwin_ptr();
    let curwin_is_cmdwin = c_int::from(curwin_after == cmdwin_win);

    // Re-check after buf_open_scratch autocommands.
    let old_curbuf_valid2 = nvim_bufref_valid_cmdwin(old_curbuf_ref);
    let old_curbuf_buf2 = nvim_bufref_get_buf_cmdwin(old_curbuf_ref);
    let old_curwin_buf2 = if rs_win_valid(old_curwin) != 0 {
        nvim_win_get_w_buffer(old_curwin)
    } else {
        std::ptr::null_mut()
    };

    if rs_cmdwin_buffer_invalid(
        c_int::from(newbuf_status == OK),
        cmdwin_valid,
        curwin_is_cmdwin,
        rs_win_valid(old_curwin),
        old_curbuf_valid2,
        c_int::from(!old_curwin_buf2.is_null() && old_curwin_buf2 != old_curbuf_buf2),
    ) != 0
    {
        // Set bufref for the new buffer (if created) for cleanup.
        let new_buf_ref = if newbuf_status == OK {
            let br = nvim_alloc_bufref();
            nvim_set_bufref_cmdwin(br, nvim_get_curbuf_ptr());
            br
        } else {
            std::ptr::null_mut()
        };

        // Close cmdwin window if still valid.
        nvim_win_close_if_valid_not_last(cmdwin_win);

        // Close newly-created buffer if it survived win_close().
        if !new_buf_ref.is_null() {
            nvim_close_buffer_wipe_if_valid(new_buf_ref, nvim_get_curbuf_ptr());
            nvim_free_bufref(new_buf_ref);
        }

        // Reset cmdwin state.
        nvim_set_cmdwin_type(0);
        nvim_set_cmdwin_level(0);
        nvim_set_cmdwin_win(std::ptr::null_mut());
        nvim_set_cmdwin_old_curwin(std::ptr::null_mut());

        nvim_beep_flush();
        nvim_free_garray(winsizes);
        nvim_free_bufref(old_curbuf_ref);
        return CTRL_C;
    }

    // Point cmdwin_buf to the new buffer.
    nvim_set_cmdwin_buf(nvim_get_curbuf_ptr());

    // Set buffer options.
    nvim_set_opt_bufhidden_wipe();
    nvim_curbuf_set_b_p_ma(1);
    nvim_curwin_set_w_p_fen(0);
    // C: curwin->w_p_rl = cmdmsg_rl; cmdmsg_rl = false;
    // save_cmdmsg_rl holds the saved cmdmsg_rl value (1 = true, 0 = false)
    nvim_curwin_set_w_p_rl(c_int::from(save_cmdmsg_rl != 0));
    nvim_set_cmdmsg_rl(0);

    // Don't allow switching to another buffer.
    nvim_curbuf_inc_ro_locked();

    // Reset need_wait_return.
    nvim_set_need_wait_return(0);

    // History type and tab mapping.
    let histtype = rs_cmdwin_to_hist_type(nvim_get_cmdwin_type());
    if rs_cmdwin_needs_tab_mapping(histtype, nvim_cmdexpand_get_p_wc()) != 0 {
        nvim_add_tab_map_insert();
        nvim_add_tab_map_normal();
    }
    if rs_cmdwin_needs_vim_filetype(histtype) != 0 {
        nvim_set_opt_filetype_vim();
    }
    nvim_curbuf_dec_ro_locked();

    // Reset 'textwidth' (vim filetype plugin sets it to 78).
    nvim_curbuf_set_b_p_tw(0);

    // Fill the buffer with history.
    let hislen = nvim_init_history_and_get_hislen();
    if hislen > 0 && histtype != HIST_INVALID {
        let hisidx = nvim_get_hisidx(histtype);
        if hisidx >= 0 {
            let mut i = hisidx;
            let mut lnum: c_int = 0;
            loop {
                i += 1;
                if i == hislen {
                    i = 0;
                }
                let s = nvim_get_histentry_str(histtype, i);
                if !s.is_null() {
                    nvim_ml_append_cmdwin(lnum, s);
                    lnum += 1;
                }
                if i == hisidx {
                    break;
                }
            }
        }
    }

    // Replace the empty last line with the current command-line.
    nvim_ml_replace_last_with_cmdbuff();
    nvim_curwin_set_cursor_lnum(nvim_curbuf_get_ml_line_count());
    nvim_curwin_set_cursor_col(nvim_get_ccline_cmdpos() as c_uint);
    nvim_changed_line_abv_curs_cmdwin();
    nvim_invalidate_botline_curwin();

    // Handle UI cmdline hiding.
    if nvim_ui_has_cmdline() != 0 {
        nvim_set_cmdline_was_last_drawn(0);
        nvim_set_ccline_redraw_state(0); // kCmdRedrawNone
        nvim_ui_call_cmdline_hide_ccline(0);
    }
    nvim_redraw_later_curwin_some_valid();

    // No Ex mode in cmdwin.
    nvim_set_exmode_active(0);

    nvim_set_State(MODE_NORMAL);
    setmouse();
    rs_clear_showcmd();

    // Reset result (can be set by CmdwinEnter autocmd).
    nvim_set_cmdwin_result(0);

    // Trigger CmdwinEnter.
    nvim_trigger_cmdwinenter();
    if nvim_get_restart_edit() != 0 {
        // autocmd with :startinsert
        nvim_stuffcharReadbuff_cmdwin(K_NOP);
    }

    // Save/clear RedrawingDisabled and batch count.
    let saved_redrawing = nvim_get_RedrawingDisabled();
    nvim_set_RedrawingDisabled(0);
    let save_count = nvim_save_batch_count();

    // Main loop: blocks until <CR> or CTRL-C.
    nvim_normal_enter_cmdwin();

    nvim_set_RedrawingDisabled(saved_redrawing);
    nvim_restore_batch_count(save_count);

    let save_key_typed = nvim_get_KeyTyped();

    // Trigger CmdwinLeave.
    nvim_trigger_cmdwinleave();

    // Restore KeyTyped in case autocommands modified it.
    nvim_set_KeyTyped(c_int::from(save_key_typed));

    // Clear cmdwin state.
    nvim_set_cmdwin_type(0);
    nvim_set_cmdwin_level(0);
    nvim_set_cmdwin_buf(std::ptr::null_mut());
    nvim_set_cmdwin_win(std::ptr::null_mut());
    nvim_set_cmdwin_old_curwin(std::ptr::null_mut());

    nvim_set_exmode_active(save_exmode);

    // Safety check: old window or buffer must still be valid.
    let old_curbuf_valid3 = nvim_bufref_valid_cmdwin(old_curbuf_ref);
    let old_curbuf_buf3 = nvim_bufref_get_buf_cmdwin(old_curbuf_ref);
    let old_curwin_buf3 = if rs_win_valid(old_curwin) != 0 {
        nvim_win_get_w_buffer(old_curwin)
    } else {
        std::ptr::null_mut()
    };

    if rs_cmdwin_cleanup_had_error(
        rs_win_valid(old_curwin),
        old_curbuf_valid3,
        c_int::from(!old_curwin_buf3.is_null() && old_curwin_buf3 != old_curbuf_buf3),
    ) != 0
    {
        nvim_set_cmdwin_result(CTRL_C);
        nvim_emsg_cmdwin_changed();
    } else {
        // autocmds may abort script processing
        if nvim_aborting() != 0 && nvim_get_cmdwin_result() != K_IGNORE {
            nvim_set_cmdwin_result(CTRL_C);
        }

        // Set the new command line from the cmdwin buffer.
        nvim_dealloc_cmdbuff();

        let result = nvim_get_cmdwin_result();
        if result == K_XF1 || result == K_XF2 {
            // :qa[!] typed
            let with_bang = c_int::from(result == K_XF1);
            if histtype == HIST_CMD {
                // Execute the command directly.
                nvim_set_ccline_cmdbuff_qa(with_bang);
                nvim_set_cmdwin_result(CAR);
            } else {
                // Cancel what we were doing, then stuff the command.
                nvim_stuff_qa_into_readbuff(with_bang);
            }
        } else if result == CTRL_C {
            // :q or :close -- don't execute, clear cmdbuff.
            nvim_set_ccline_cmdbuff(std::ptr::null_mut());
        } else {
            nvim_set_ccline_cmdbuff_from_cursor();
        }

        if nvim_get_ccline_cmdbuff().is_null() {
            nvim_set_ccline_cmdbuff_empty();
            nvim_set_cmdwin_result(CTRL_C);
        } else {
            nvim_set_ccline_cmdpos_from_cursor();
            // If cursor is at last char or beyond, set cmdpos to cmdlen.
            let cmdpos = nvim_get_ccline_cmdpos();
            let cmdlen = nvim_get_ccline_cmdlen();
            if cmdpos == cmdlen - 1 || cmdpos > cmdlen {
                nvim_set_ccline_cmdpos(cmdlen);
            }
            if nvim_get_cmdwin_result() == K_IGNORE {
                nvim_set_ccline_cmdspos(nvim_cmd_screencol_cmdpos());
                nvim_redrawcmd();
            }
        }

        // Avoid command-line window first character being concealed.
        nvim_curwin_set_w_p_cole(0);

        // Go back to the original window.
        let wp = nvim_get_curwin_ptr();
        let bufref = nvim_alloc_bufref();
        nvim_set_bufref_cmdwin(bufref, nvim_get_curbuf_ptr());

        nvim_set_skip_win_fix_cursor(1);
        nvim_win_goto_cmdwin(old_curwin);

        // win_goto() may trigger an autocommand that already closes cmdwin.
        if rs_win_valid(wp) != 0 && wp != nvim_get_curwin_ptr() {
            nvim_win_close_cmdwin(wp);
        }

        // win_close() may have already wiped the buffer when bh=wipe.
        nvim_close_buffer_wipe_if_valid(bufref, nvim_get_curbuf_ptr());
        nvim_free_bufref(bufref);

        // Restore window sizes.
        rs_win_size_restore(winsizes);
        nvim_set_skip_win_fix_cursor(0);
    }

    nvim_free_garray(winsizes);
    nvim_free_bufref(old_curbuf_ref);

    nvim_set_restart_edit(save_restart_edit);
    nvim_set_cmdmsg_rl(save_cmdmsg_rl);
    nvim_set_State(save_state);
    may_trigger_modechanged();
    setmouse();
    nvim_setcursor();

    nvim_get_cmdwin_result()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmdwin_type() {
        assert_eq!(CmdwinType::from_char(i32::from(b':')), CmdwinType::Ex);
        assert_eq!(
            CmdwinType::from_char(i32::from(b'/')),
            CmdwinType::ForwardSearch
        );
        assert_eq!(CmdwinType::from_char(0), CmdwinType::None);

        assert!(CmdwinType::Ex.is_active());
        assert!(!CmdwinType::None.is_active());

        assert!(CmdwinType::ForwardSearch.is_search());
        assert!(!CmdwinType::Ex.is_search());

        assert_eq!(CmdwinType::Ex.to_char(), Some(b':'));
        assert_eq!(CmdwinType::None.to_char(), None);
    }

    #[test]
    fn test_cmdwin_state() {
        let mut state = CmdwinState::new();
        assert!(!state.is_active());

        state.open(CmdwinType::Ex, 1);
        assert!(state.is_active());
        assert_eq!(state.level, 1);

        state.close(result::ENTER);
        assert!(!state.is_active());
        assert_eq!(state.result, result::ENTER);
    }

    #[test]
    fn test_result_codes() {
        assert!(result::should_execute(result::ENTER));
        assert!(!result::should_execute(result::ESC));

        assert!(result::should_cancel(result::CTRL_C));
        assert!(result::should_cancel(result::ESC));
        assert!(!result::should_cancel(result::ENTER));
    }

    #[test]
    fn test_history_mapping() {
        assert_eq!(cmdwin_to_hist_type(CmdwinType::Ex), 0); // HIST_CMD
        assert_eq!(cmdwin_to_hist_type(CmdwinType::ForwardSearch), 1); // HIST_SEARCH
        assert_eq!(cmdwin_to_hist_type(CmdwinType::Expression), 2); // HIST_EXPR
    }

    #[test]
    fn test_open_error() {
        assert!(CmdwinOpenError::Ok.can_open());
        assert!(!CmdwinOpenError::AlreadyInCmdwin.can_open());
        assert!(!CmdwinOpenError::TextLocked.can_open());
    }

    #[test]
    fn test_can_open_cmdwin() {
        // Normal case - can open
        assert_eq!(can_open_cmdwin(false, false, 0), CmdwinOpenError::Ok);

        // Already in cmdwin
        assert_eq!(
            can_open_cmdwin(true, false, 0),
            CmdwinOpenError::AlreadyInCmdwin
        );

        // Text locked
        assert_eq!(can_open_cmdwin(false, true, 0), CmdwinOpenError::TextLocked);

        // Secret mode (password)
        assert_eq!(
            can_open_cmdwin(false, false, 1),
            CmdwinOpenError::SecretMode
        );
    }

    #[test]
    fn test_cmdwin_split_invalid() {
        // All valid
        assert!(!cmdwin_split_invalid(true, false, true, false));

        // Old curwin not valid
        assert!(cmdwin_split_invalid(false, false, true, false));

        // Curwin is old (didn't create new window)
        assert!(cmdwin_split_invalid(true, true, true, false));

        // Old curbuf not valid
        assert!(cmdwin_split_invalid(true, false, false, false));

        // Buffer changed
        assert!(cmdwin_split_invalid(true, false, true, true));
    }

    #[test]
    fn test_cmdwin_needs_tab_mapping() {
        const TAB: i32 = 9;
        const HIST_CMD: i32 = 0;
        const HIST_DEBUG: i32 = 4;
        const HIST_SEARCH: i32 = 1;

        // Tab wildchar with Ex history
        assert!(cmdwin_needs_tab_mapping(HIST_CMD, TAB));

        // Tab wildchar with Debug history
        assert!(cmdwin_needs_tab_mapping(HIST_DEBUG, TAB));

        // Tab wildchar with Search history - no mapping
        assert!(!cmdwin_needs_tab_mapping(HIST_SEARCH, TAB));

        // Non-tab wildchar - no mapping
        assert!(!cmdwin_needs_tab_mapping(HIST_CMD, b'%' as i32));
    }

    #[test]
    fn test_cmdwin_needs_vim_filetype() {
        const HIST_CMD: i32 = 0;
        const HIST_DEBUG: i32 = 4;
        const HIST_SEARCH: i32 = 1;

        assert!(cmdwin_needs_vim_filetype(HIST_CMD));
        assert!(cmdwin_needs_vim_filetype(HIST_DEBUG));
        assert!(!cmdwin_needs_vim_filetype(HIST_SEARCH));
    }

    #[test]
    fn test_cmdwin_cleanup_had_error() {
        // All valid - no error
        assert!(!cmdwin_cleanup_had_error(true, true, false));

        // Window invalid
        assert!(cmdwin_cleanup_had_error(false, true, false));

        // Buffer invalid
        assert!(cmdwin_cleanup_had_error(true, false, false));

        // Buffer changed
        assert!(cmdwin_cleanup_had_error(true, true, true));
    }
}
