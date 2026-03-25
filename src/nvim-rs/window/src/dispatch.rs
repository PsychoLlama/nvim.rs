//! CTRL-W command dispatcher.
//!
//! This module implements [`rs_do_window`], the Rust replacement for the
//! `do_window()` switch in `src/nvim/window.c`.  Each case delegates to
//! a C wrapper that performs the actual work (autocmds, memory allocation,
//! complex C-only logic).  The dispatch itself is pure Rust.

// Window dimensions may need truncation when converting between types.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::{c_char, c_int};

use crate::{BufHandle, TabpageHandle, WinHandle};

// =============================================================================
// EMSG IDs for nvim_emsg_id dispatcher
// =============================================================================

const EMSG_E441_NO_PREVIEW: c_int = 6;
const EMSG_NOALT: c_int = 7;

// =============================================================================
// Key code constants (verified by _Static_assert in window.c)
// =============================================================================

const K_UP: c_int = -30059;
const K_DOWN: c_int = -25707;
const K_LEFT: c_int = -27755;
const K_RIGHT: c_int = -29291;
const K_BS: c_int = -25195;
const K_KENTER: c_int = -16715;

// Control characters (from ascii_defs.h, verified by _Static_assert)
const CTRL_S: c_int = 19;
const CTRL_V: c_int = 22;
const CTRL_HAT: c_int = 30;
const CTRL_N: c_int = 14;
const CTRL_Q: c_int = 17;
const CTRL_C: c_int = 3;
const CTRL_Z: c_int = 26;
const CTRL_O: c_int = 15;
const CTRL_W: c_int = 23;
const CTRL_J: c_int = 10;
const CTRL_K: c_int = 11;
const CTRL_H: c_int = 8;
const CTRL_L: c_int = 12;
const CTRL_T: c_int = 20;
const CTRL_B: c_int = 2;
const CTRL_P: c_int = 16;
const CTRL_X: c_int = 24;
const CTRL_R: c_int = 18;
#[allow(non_upper_case_globals)]
const CTRL__: c_int = 31;
const CTRL_RSB: c_int = 29;
const CTRL_F: c_int = 6;
const CTRL_I: c_int = 9;
const CTRL_D: c_int = 4;
const CTRL_G: c_int = 7;
const NUL: c_int = 0;
const CAR: c_int = 13;

// ASCII character constants used in match arms.
// Rust doesn't allow `b'x' as c_int` in patterns, so we define named constants.
const CH_S_UPPER: c_int = b'S' as c_int;
const CH_S: c_int = b's' as c_int;
const CH_V: c_int = b'v' as c_int;
const CH_HAT: c_int = b'^' as c_int;
const CH_N: c_int = b'n' as c_int;
const CH_Q: c_int = b'q' as c_int;
const CH_C: c_int = b'c' as c_int;
const CH_Z: c_int = b'z' as c_int;
const CH_P_UPPER: c_int = b'P' as c_int;
const CH_O: c_int = b'o' as c_int;
const CH_W: c_int = b'w' as c_int;
const CH_W_UPPER: c_int = b'W' as c_int;
const CH_J: c_int = b'j' as c_int;
const CH_K: c_int = b'k' as c_int;
const CH_H: c_int = b'h' as c_int;
const CH_L: c_int = b'l' as c_int;
const CH_T_UPPER: c_int = b'T' as c_int;
const CH_T: c_int = b't' as c_int;
const CH_B: c_int = b'b' as c_int;
const CH_P: c_int = b'p' as c_int;
const CH_X: c_int = b'x' as c_int;
const CH_R_UPPER: c_int = b'R' as c_int;
const CH_R: c_int = b'r' as c_int;
const CH_K_UPPER: c_int = b'K' as c_int;
const CH_J_UPPER: c_int = b'J' as c_int;
const CH_H_UPPER: c_int = b'H' as c_int;
const CH_L_UPPER: c_int = b'L' as c_int;
const CH_EQ: c_int = b'=' as c_int;
const CH_PLUS: c_int = b'+' as c_int;
const CH_MINUS: c_int = b'-' as c_int;
const CH_UNDERSCORE: c_int = b'_' as c_int;
const CH_GT: c_int = b'>' as c_int;
const CH_LT: c_int = b'<' as c_int;
const CH_PIPE: c_int = b'|' as c_int;
const CH_RBRACE: c_int = b'}' as c_int;
const CH_RBRACKET: c_int = b']' as c_int;
const CH_F: c_int = b'f' as c_int;
const CH_F_UPPER: c_int = b'F' as c_int;
const CH_I: c_int = b'i' as c_int;
const CH_D: c_int = b'd' as c_int;
const CH_G: c_int = b'g' as c_int;
const CH_E: c_int = b'e' as c_int;
const TAB: c_int = 9;

// WSP flags (verified by _Static_assert in window.c)
const WSP_VERT: c_int = 0x02;
const WSP_HOR: c_int = 0x04;
const WSP_TOP: c_int = 0x08;
const WSP_BOT: c_int = 0x10;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    static Rows: c_int;
    static Columns: c_int;
    // --- Wrappers for complex cases (keep logic in C) ---
    // nvim_do_window_wW removed: replaced by rs_do_window_wW (Phase 4)
    // nvim_do_window_P removed: replaced by rs_do_window_P (Phase 4)
    // nvim_do_window_T removed: replaced by rs_do_window_T (Phase 4)
    // nvim_do_window_hat removed: replaced by rs_do_window_hat (Phase 4)
    // nvim_do_window_new removed: replaced by rs_do_window_new (Phase 4)
    // nvim_do_window_equalize removed: replaced inline by rs_do_window_equalize (Phase 8)
    // nvim_do_window_tag removed: replaced inline by rs_do_window_tag (Phase 8)
    // nvim_do_window_goto_file removed: replaced by do_window_goto_file (Phase 10)
    // nvim_do_window_find_in_path removed: replaced by do_window_find_in_path (Phase 10)
    // nvim_do_window_g_external removed: replaced by do_window_g_external (Phase 10)
    // nvim_do_window_g removed: replaced by rs_do_window_g
    fn rs_qf_view_result(split: bool);

    // --- 'g' sub-dispatch wrappers (Phase 3) ---
    // These exist in normal_shim.c / tag_shim.c / window_shim.c:
    fn nvim_inc_no_mapping(); // normal_shim.c
    fn nvim_dec_no_mapping(); // normal_shim.c
    fn nvim_inc_allow_keys(); // normal_shim.c
    fn nvim_dec_allow_keys(); // normal_shim.c
    fn nvim_plain_vgetc_wrapper() -> c_int; // normal_shim.c
    /// Applies LANGMAP_ADJUST(c, condition). Defined in normal_shim.c.
    fn nvim_langmap_adjust(c: c_int, condition: bool) -> c_int; // normal_shim.c
    fn nvim_add_to_showcmd_wrapper(c: c_int) -> bool; // normal_shim.c
    fn nvim_set_g_do_tagpreview(val: c_int); // tag_shim.c
    fn nvim_get_p_pvh() -> c_int; // window_shim.c
    fn nvim_set_postponed_split(val: c_int); // tag_shim.c
    fn nvim_do_nv_ident(prefix: c_int, xchar: c_int); // window_shim.c
    fn nvim_goto_tabpage(n: c_int); // normal_shim.c
    /// goto_tabpage_lastused(). Defined in normal_shim.c.
    fn nvim_goto_tabpage_lastused() -> bool; // normal_shim.c
    fn nvim_set_cmdmod_tab_to_curtab_idx(); // window_shim.c
    fn nvim_do_window_g_external(); // window_shim.c

    // --- Phase 10: goto_file, find_in_path, g_external accessors ---
    /// grab_file_name(count1, &lnum): returns owned char*, sets *lnum_out (as int).
    fn nvim_grab_file_name(count1: c_int, lnum_out: *mut c_int) -> *mut c_char;
    /// buflist_findname_exp wrapper.
    fn nvim_buflist_findname_exp(ptr: *const c_char) -> BufHandle;
    /// setpcmark() wrapper.
    fn nvim_setpcmark_curwin();
    /// RESET_BINDING(curwin) wrapper.
    fn nvim_reset_binding_curwin();
    /// do_ecmd(0, ptr, NULL, NULL, ECMD_LASTL, ECMD_HIDE, NULL) wrapper.
    fn nvim_do_ecmd_lastl_hide(ptr: *const c_char) -> c_int;
    /// check_cursor_lnum(curwin) wrapper.
    fn nvim_check_cursor_lnum_curwin();
    /// beginline(BL_SOL | BL_FIX) wrapper.
    fn nvim_beginline_sol_fix();
    /// curwin->w_cursor.lnum setter (linenr_T = int32_t).
    fn nvim_set_curwin_cursor_lnum(lnum: i32);
    /// swb_flags & kOptSwbFlagUseopen.
    fn nvim_swb_has_useopen() -> c_int;
    /// swb_flags & kOptSwbFlagUsetab.
    fn nvim_swb_has_usetab() -> c_int;
    /// goto_tabpage_win: call rs_goto_tabpage_win directly.
    fn rs_goto_tabpage_win(tp: TabpageHandle, wp: WinHandle);
    /// win_close: call directly.
    #[link_name = "win_close"]
    fn rs_win_close(wp: WinHandle, free_buf: c_int, force: c_int) -> c_int;
    /// cmdmod.cmod_tab getter.
    fn nvim_get_cmdmod_tab() -> c_int;
    /// rs_check_text_or_curbuf_locked(NULL) -- true if text or curbuf locked.
    fn rs_check_text_or_curbuf_locked(oap: *mut std::ffi::c_void) -> bool;
    /// rs_find_ident_under_cursor: sets *pp to pointer into buffer, returns len.
    fn nvim_find_ident_under_cursor(pp: *mut *mut c_char) -> usize;
    /// find_pattern_in_path with ACTION_SPLIT.
    /// `whole`: 1 if searching whole file (Prenum == 0).
    fn nvim_find_pattern_in_path_split(
        ptr: *const c_char,
        len: usize,
        type_: c_int,
        prenum1: c_int,
        whole: c_int,
    );
    /// curwin->w_set_curswant = true wrapper.
    fn nvim_set_curswant_curwin();
    /// nvim_xmemdupz: allocates a NUL-terminated copy of len bytes.
    fn nvim_xmemdupz(ptr: *const c_char, len: usize) -> *mut c_char;
    /// nvim_xfree: free a C-allocated pointer.
    fn nvim_xfree(ptr: *mut std::ffi::c_void);
    /// win_new_float external wrapper: -1=not applicable, 0=fail, 1=ok.
    fn nvim_win_new_float_external() -> c_int;

    // --- Simple wrappers ---
    fn nvim_emsg_e_cmdwin();
    #[link_name = "rs_reset_VIsual_and_resel"]
    fn nvim_reset_visual_wrapper();
    fn nvim_bt_quickfix_curbuf() -> c_int;
    fn rs_win_split(size: c_int, flags: c_int) -> c_int;
    // nvim_cmd_with_count_exec removed: replaced by rs_cmd_with_count_exec (Phase 4)
    fn nvim_do_cmdline_cmd_wrapper(cmd: *const u8) -> c_int;
    fn nvim_beep_flush_wrapper();
    #[link_name = "rs_one_window_in_tab"]
    fn nvim_one_window_curwin(wp: WinHandle, tp: TabpageHandle) -> c_int;
    fn nvim_msg_onlyone();
    fn rs_win_goto(wp: WinHandle);
    fn nvim_get_curtab() -> TabpageHandle;
    fn rs_win_vert_neighbor(tp: TabpageHandle, wp: WinHandle, up: c_int, count: c_int)
        -> WinHandle;
    fn rs_win_horz_neighbor(
        tp: TabpageHandle,
        wp: WinHandle,
        left: c_int,
        count: c_int,
    ) -> WinHandle;
    // nvim_win_exchange_wrapper removed: replaced by rs_win_exchange
    // nvim_win_rotate_wrapper removed: replaced by rs_win_rotate
    fn rs_win_splitmove(wp: WinHandle, size: c_int, flags: c_int) -> c_int;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_get_min_set_ch() -> i64;

    // --- Accessors ---
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_get_lastwin() -> WinHandle;
    static cmdwin_type: c_int;
    #[link_name = "rs_lastwin_nofloating"]
    fn nvim_lastwin_nofloating_wrapper() -> WinHandle;
    // nvim_get_valid_prevwin removed: replaced inline by get_valid_prevwin (Phase 8)
    fn nvim_get_prevwin() -> WinHandle; // for inline prevwin check
    fn nvim_get_cmdmod_split() -> c_int; // for inline equalize
    #[link_name = "rs_win_equal"]
    fn nvim_win_equal(next_curwin: WinHandle, current: c_int, dir: c_int); // for equalize
    #[link_name = "rs_win_valid"]
    fn nvim_win_valid(wp: WinHandle) -> c_int; // for prevwin check

    // --- Phase 4: wW/P/T/hat/new/count accessors ---
    fn nvim_win_get_floating(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_hide(wp: WinHandle) -> c_int;
    fn nvim_win_get_config_focusable(wp: WinHandle) -> c_int;
    fn nvim_win_get_next(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_prev(wp: WinHandle) -> WinHandle;
    fn nvim_win_get_pvw(wp: WinHandle) -> c_int;
    fn nvim_emsg_id(id: c_int);
    fn rs_win_new_tabpage(after: c_int, filename: *const u8) -> c_int;
    fn nvim_al_goto_tabpage_tp(tp: TabpageHandle, trigger_enter: c_int, trigger_leave: c_int);
    fn nvim_al_win_close(wp: WinHandle, free_buf: c_int, force: c_int);
    fn nvim_apply_autocmds_tabnewentered();
    fn nvim_win_get_alt_fnum(wp: WinHandle) -> c_int;
    fn nvim_curbuf_locked() -> c_int;
    fn nvim_semsg_e92_buf_not_found(nr: i64);
    #[link_name = "rs_buflist_findnr"]
    fn nvim_buflist_findnr(nr: c_int) -> BufHandle;
    fn nvim_buflist_getfile(nr: c_int, lnum: c_int, flags: c_int, setpm: c_int);

    // --- Resize ---
    fn rs_win_setheight(height: c_int);
    fn rs_win_setwidth(width: c_int);
}

// =============================================================================
// Dispatch helpers
// =============================================================================

/// Check if we're in the command-line window; if so, emit error and return true.
#[inline]
unsafe fn check_cmdwin() -> bool {
    if cmdwin_type != 0 {
        nvim_emsg_e_cmdwin();
        true
    } else {
        false
    }
}

/// Inline replacement for C `nvim_get_valid_prevwin`.
///
/// Returns prevwin if it is valid (rs_win_valid), not hidden, and focusable;
/// otherwise returns a null handle.
#[inline]
unsafe fn get_valid_prevwin() -> WinHandle {
    let prevwin = nvim_get_prevwin();
    if prevwin.is_null()
        || nvim_win_valid(prevwin) == 0
        || nvim_win_get_config_hide(prevwin) != 0
        || nvim_win_get_config_focusable(prevwin) == 0
    {
        WinHandle::null()
    } else {
        prevwin
    }
}

/// Inline replacement for C `nvim_do_window_equalize`.
///
/// Reads `cmdmod.cmod_split` to determine equalize direction and calls
/// `rs_win_equal(NULL, 0, dir)`.
#[inline]
unsafe fn do_window_equalize() {
    let mod_ = nvim_get_cmdmod_split() & (WSP_VERT | WSP_HOR);
    let dir: c_int = if mod_ == WSP_VERT {
        c_int::from(b'v')
    } else if mod_ == WSP_HOR {
        c_int::from(b'h')
    } else {
        c_int::from(b'b')
    };
    nvim_win_equal(WinHandle::null(), 0, dir);
}

/// Inline replacement for C `nvim_do_window_tag`.
///
/// Handles ']', '}', and Ctrl-] tag/preview window commands.
#[inline]
unsafe fn do_window_tag(nchar: c_int, prenum: c_int) {
    if nchar == CH_RBRACE {
        if prenum != 0 {
            nvim_set_g_do_tagpreview(prenum);
        } else {
            nvim_set_g_do_tagpreview(nvim_get_p_pvh());
        }
    }

    if prenum != 0 {
        nvim_set_postponed_split(prenum);
    } else {
        nvim_set_postponed_split(-1);
    }

    if nchar != CH_RBRACE {
        nvim_set_g_do_tagpreview(0);
    }

    nvim_do_nv_ident(CTRL_RSB, NUL);
    nvim_set_postponed_split(0);
}

// =============================================================================
// Phase 10: Goto-file, find-in-path, and external-float helpers
// =============================================================================

/// CTRL-F constants from search.h.
const FIND_ANY: c_int = 1;
const FIND_DEFINE: c_int = 2;

/// C 'F' nchar value.
const CH_F_VAL: c_int = b'F' as c_int;

/// do_ecmd return values.
const FAIL: c_int = 0;
const OK_ECMD: c_int = 1;

// (CTRL_I = 9 already defined above)

/// Rust implementation of `nvim_do_window_goto_file`.
///
/// The 'f'/'F'/Ctrl-F file-goto command: grab filename under cursor, split
/// window, open file, optionally jump to line.
///
/// # Safety
///
/// Called from dispatch; all state access via C wrappers.
unsafe fn do_window_goto_file(nchar: c_int, prenum1: c_int) {
    if rs_check_text_or_curbuf_locked(std::ptr::null_mut()) {
        return;
    }

    let mut lnum: c_int = -1;
    let ptr = nvim_grab_file_name(prenum1, std::ptr::addr_of_mut!(lnum));
    if ptr.is_null() {
        return;
    }

    let oldtab = nvim_get_curtab();
    let oldwin = nvim_get_curwin();
    nvim_setpcmark_curwin();

    let mut wp = WinHandle::null();

    // Check 'switchbuf' flags to reuse an existing window.
    if (nvim_swb_has_useopen() != 0 || nvim_swb_has_usetab() != 0) && nvim_get_cmdmod_tab() == 0 {
        let buf = nvim_buflist_findname_exp(ptr);
        if !buf.is_null() {
            wp = crate::navigate::find::rs_swbuf_goto_win_with_buf(buf);
        }
    }

    if wp.is_null() && rs_win_split(0, 0) == OK_ECMD {
        nvim_reset_binding_curwin();
        if nvim_do_ecmd_lastl_hide(ptr) == FAIL {
            rs_win_close(nvim_get_curwin(), 0, 0);
            rs_goto_tabpage_win(oldtab, oldwin);
        } else {
            wp = nvim_get_curwin();
        }
    }

    if !wp.is_null() && nchar == CH_F_VAL && lnum >= 0 {
        nvim_set_curwin_cursor_lnum(lnum as i32);
        nvim_check_cursor_lnum_curwin();
        nvim_beginline_sol_fix();
    }

    nvim_xfree(ptr.cast());
}

/// Rust implementation of `nvim_do_window_find_in_path`.
///
/// The 'i'/'d' find-in-path command: find identifier under cursor, call
/// `find_pattern_in_path` with ACTION_SPLIT.
///
/// # Safety
///
/// Called from dispatch; all state access via C wrappers.
unsafe fn do_window_find_in_path(nchar: c_int, prenum: c_int, prenum1: c_int) {
    let type_ = if nchar == CH_I || nchar == CTRL_I {
        FIND_ANY
    } else {
        FIND_DEFINE
    };

    let mut raw_ptr: *mut c_char = std::ptr::null_mut();
    let len = nvim_find_ident_under_cursor(std::ptr::addr_of_mut!(raw_ptr));
    if len == 0 {
        return;
    }

    // Make an owned copy so find_pattern_in_path can safely use it.
    let ptr = nvim_xmemdupz(raw_ptr, len);
    let whole = c_int::from(prenum == 0);
    nvim_find_pattern_in_path_split(ptr, len, type_, prenum1, whole);
    nvim_xfree(ptr.cast());
    nvim_set_curswant_curwin();
}

/// Rust implementation of `nvim_do_window_g_external`.
///
/// The 'g'+'e' command: convert current window to an external floating window.
///
/// # Safety
///
/// Called from dispatch; all state access via C wrappers.
unsafe fn do_window_g_external() {
    let result = nvim_win_new_float_external();
    if result <= 0 {
        nvim_beep_flush_wrapper();
    }
}

// =============================================================================
// Main dispatch
// =============================================================================

/// Rust dispatcher for all CTRL-W window commands.
///
/// This replaces the `do_window()` switch/case in `window.c`.
///
/// # Safety
///
/// Called from C via FFI. All pointer-based operations go through C wrappers.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub extern "C" fn rs_do_window(nchar: c_int, prenum: c_int, xchar: c_int) {
    let prenum1 = if prenum == 0 { 1 } else { prenum };

    unsafe {
        match nchar {
            // =================================================================
            // Split commands: 'S', Ctrl-S, 's'
            // =================================================================
            CH_S_UPPER | CTRL_S | CH_S => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                // When splitting the quickfix window, open a new buffer.
                if nvim_bt_quickfix_curbuf() != 0 {
                    rs_do_window_new(nchar, prenum);
                    return;
                }
                rs_win_split(prenum, 0);
            }

            // =================================================================
            // Vertical split: Ctrl-V, 'v'
            // =================================================================
            CTRL_V | CH_V => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                if nvim_bt_quickfix_curbuf() != 0 {
                    rs_do_window_new(nchar, prenum);
                    return;
                }
                rs_win_split(prenum, WSP_VERT);
            }

            // =================================================================
            // Split and edit alternate file: Ctrl-^, '^'
            // =================================================================
            CTRL_HAT | CH_HAT => {
                if check_cmdwin() {
                    return;
                }
                rs_do_window_hat(prenum);
            }

            // =================================================================
            // New window: Ctrl-N, 'n'
            // =================================================================
            CTRL_N | CH_N => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                rs_do_window_new(nchar, prenum);
            }

            // =================================================================
            // Quit: Ctrl-Q, 'q'
            // =================================================================
            CTRL_Q | CH_Q => {
                nvim_reset_visual_wrapper();
                rs_cmd_with_count_exec(c"quit".as_ptr().cast(), i64::from(prenum));
            }

            // =================================================================
            // Close: Ctrl-C, 'c'
            // =================================================================
            CTRL_C | CH_C => {
                nvim_reset_visual_wrapper();
                rs_cmd_with_count_exec(c"close".as_ptr().cast(), i64::from(prenum));
            }

            // =================================================================
            // Close preview: Ctrl-Z, 'z'
            // =================================================================
            CTRL_Z | CH_Z => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                nvim_do_cmdline_cmd_wrapper(c"pclose".as_ptr().cast());
            }

            // =================================================================
            // Cursor to preview window: 'P'
            // =================================================================
            CH_P_UPPER => {
                rs_do_window_P();
            }

            // =================================================================
            // Close all but current: Ctrl-O, 'o'
            // =================================================================
            CTRL_O | CH_O => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                rs_cmd_with_count_exec(c"only".as_ptr().cast(), i64::from(prenum));
            }

            // =================================================================
            // Cursor to next/prev window with wrap: Ctrl-W, 'w', 'W'
            // =================================================================
            CTRL_W | CH_W | CH_W_UPPER => {
                if check_cmdwin() {
                    return;
                }
                rs_do_window_wW(nchar, prenum);
            }

            // =================================================================
            // Cursor to window below: 'j', K_DOWN, Ctrl-J
            // =================================================================
            CH_J | K_DOWN | CTRL_J => {
                if check_cmdwin() {
                    return;
                }
                let win = rs_win_vert_neighbor(nvim_get_curtab(), nvim_get_curwin(), 0, prenum1);
                if !win.is_null() {
                    rs_win_goto(win);
                }
            }

            // =================================================================
            // Cursor to window above: 'k', K_UP, Ctrl-K
            // =================================================================
            CH_K | K_UP | CTRL_K => {
                if check_cmdwin() {
                    return;
                }
                let win = rs_win_vert_neighbor(nvim_get_curtab(), nvim_get_curwin(), 1, prenum1);
                if !win.is_null() {
                    rs_win_goto(win);
                }
            }

            // =================================================================
            // Cursor to left window: 'h', K_LEFT, Ctrl-H, K_BS
            // =================================================================
            CH_H | K_LEFT | CTRL_H | K_BS => {
                if check_cmdwin() {
                    return;
                }
                let win = rs_win_horz_neighbor(nvim_get_curtab(), nvim_get_curwin(), 1, prenum1);
                if !win.is_null() {
                    rs_win_goto(win);
                }
            }

            // =================================================================
            // Cursor to right window: 'l', K_RIGHT, Ctrl-L
            // =================================================================
            CH_L | K_RIGHT | CTRL_L => {
                if check_cmdwin() {
                    return;
                }
                let win = rs_win_horz_neighbor(nvim_get_curtab(), nvim_get_curwin(), 0, prenum1);
                if !win.is_null() {
                    rs_win_goto(win);
                }
            }

            // =================================================================
            // Move window to new tab: 'T'
            // =================================================================
            CH_T_UPPER => {
                if check_cmdwin() {
                    return;
                }
                rs_do_window_T(prenum);
            }

            // =================================================================
            // Cursor to top-left window: 't', Ctrl-T
            // =================================================================
            CH_T | CTRL_T => {
                rs_win_goto(nvim_get_firstwin());
            }

            // =================================================================
            // Cursor to bottom-right window: 'b', Ctrl-B
            // =================================================================
            CH_B | CTRL_B => {
                rs_win_goto(nvim_lastwin_nofloating_wrapper());
            }

            // =================================================================
            // Cursor to last accessed (previous) window: 'p', Ctrl-P
            // =================================================================
            CH_P | CTRL_P => {
                let pw = get_valid_prevwin();
                if pw.is_null() {
                    nvim_beep_flush_wrapper();
                } else {
                    rs_win_goto(pw);
                }
            }

            // =================================================================
            // Exchange: 'x', Ctrl-X
            // =================================================================
            CH_X | CTRL_X => {
                if check_cmdwin() {
                    return;
                }
                crate::exchange::rs_win_exchange(prenum);
            }

            // =================================================================
            // Rotate downwards: Ctrl-R, 'r'
            // =================================================================
            CTRL_R | CH_R => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                crate::exchange::rs_win_rotate(0, prenum1);
            }

            // =================================================================
            // Rotate upwards: 'R'
            // =================================================================
            CH_R_UPPER => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                crate::exchange::rs_win_rotate(1, prenum1);
            }

            // =================================================================
            // Move to very top/bottom/left/right: 'K', 'J', 'H', 'L'
            // =================================================================
            CH_K_UPPER | CH_J_UPPER | CH_H_UPPER | CH_L_UPPER => {
                if check_cmdwin() {
                    return;
                }
                if nvim_one_window_curwin(nvim_get_curwin(), TabpageHandle::null()) != 0 {
                    nvim_beep_flush_wrapper();
                } else {
                    let vert = if nchar == CH_H_UPPER || nchar == CH_L_UPPER {
                        WSP_VERT
                    } else {
                        0
                    };
                    let topbot = if nchar == CH_H_UPPER || nchar == CH_K_UPPER {
                        WSP_TOP
                    } else {
                        WSP_BOT
                    };
                    let dir = vert | topbot;
                    rs_win_splitmove(nvim_get_curwin(), prenum, dir);
                }
            }

            // =================================================================
            // Equalize: '='
            // =================================================================
            CH_EQ => {
                do_window_equalize();
            }

            // =================================================================
            // Increase height: '+'
            // =================================================================
            CH_PLUS => {
                let curwin = nvim_get_curwin();
                rs_win_setheight(nvim_win_get_w_height(curwin) + prenum1);
            }

            // =================================================================
            // Decrease height: '-'
            // =================================================================
            CH_MINUS => {
                let curwin = nvim_get_curwin();
                rs_win_setheight(nvim_win_get_w_height(curwin) - prenum1);
            }

            // =================================================================
            // Set height: Ctrl-_, '_'
            // =================================================================
            CTRL__ | CH_UNDERSCORE => {
                if prenum != 0 {
                    rs_win_setheight(prenum);
                } else {
                    rs_win_setheight(Rows - nvim_get_min_set_ch() as c_int);
                }
            }

            // =================================================================
            // Increase width: '>'
            // =================================================================
            CH_GT => {
                let curwin = nvim_get_curwin();
                rs_win_setwidth(nvim_win_get_w_width(curwin) + prenum1);
            }

            // =================================================================
            // Decrease width: '<'
            // =================================================================
            CH_LT => {
                let curwin = nvim_get_curwin();
                rs_win_setwidth(nvim_win_get_w_width(curwin) - prenum1);
            }

            // =================================================================
            // Set width: '|'
            // =================================================================
            CH_PIPE => {
                if prenum != 0 {
                    rs_win_setwidth(prenum);
                } else {
                    rs_win_setwidth(Columns);
                }
            }

            // =================================================================
            // Tag/preview: '}', ']', Ctrl-]
            // =================================================================
            CH_RBRACE | CH_RBRACKET | CTRL_RSB => {
                if check_cmdwin() {
                    return;
                }
                do_window_tag(nchar, prenum);
            }

            // =================================================================
            // Goto file: 'f', 'F', Ctrl-F
            // =================================================================
            CH_F | CH_F_UPPER | CTRL_F => {
                if check_cmdwin() {
                    return;
                }
                do_window_goto_file(nchar, prenum1);
            }

            // =================================================================
            // Find in path: 'i', Ctrl-I, 'd', Ctrl-D
            // =================================================================
            CH_I | CTRL_I | CH_D | CTRL_D => {
                if check_cmdwin() {
                    return;
                }
                do_window_find_in_path(nchar, prenum, prenum1);
            }

            // =================================================================
            // Quickfix: K_KENTER, CAR
            // =================================================================
            K_KENTER | CAR => {
                if nvim_bt_quickfix_curbuf() != 0 {
                    rs_qf_view_result(true);
                }
            }

            // =================================================================
            // Extended 'g' commands: 'g', Ctrl-G
            // =================================================================
            CH_G | CTRL_G => {
                if check_cmdwin() {
                    return;
                }
                rs_do_window_g(prenum, xchar);
            }

            // =================================================================
            // Default: beep
            // =================================================================
            _ => {
                nvim_beep_flush_wrapper();
            }
        }
    }
}

// =============================================================================
// 'g' sub-dispatcher
// =============================================================================

/// Rust dispatcher for CTRL-W g commands.
///
/// This replaces the C `nvim_do_window_g()` function.
///
/// # Safety
///
/// Called from C via FFI. All pointer-based operations go through C wrappers.
#[no_mangle]
#[allow(clippy::too_many_lines)]
pub extern "C" fn rs_do_window_g(prenum: c_int, mut xchar: c_int) {
    let prenum1 = if prenum == 0 { 1 } else { prenum };

    unsafe {
        nvim_inc_no_mapping();
        nvim_inc_allow_keys();
        if xchar == NUL {
            xchar = nvim_plain_vgetc_wrapper();
        }
        xchar = nvim_langmap_adjust(xchar, true);
        nvim_dec_no_mapping();
        nvim_dec_allow_keys();
        nvim_add_to_showcmd_wrapper(xchar);

        match xchar {
            // =================================================================
            // Tag preview: '}' -- set g_do_tagpreview, then handle like ']'
            // =================================================================
            CH_RBRACE => {
                // Convert to Ctrl_RSB for tag handling
                let tag_xchar = CTRL_RSB;
                if prenum != 0 {
                    nvim_set_g_do_tagpreview(prenum);
                } else {
                    nvim_set_g_do_tagpreview(nvim_get_p_pvh());
                }
                if prenum != 0 {
                    nvim_set_postponed_split(prenum);
                } else {
                    nvim_set_postponed_split(-1);
                }
                nvim_do_nv_ident(c_int::from(b'g'), tag_xchar);
                nvim_set_postponed_split(0);
            }

            // =================================================================
            // Tag jump: ']', Ctrl-]
            // =================================================================
            CH_RBRACKET | CTRL_RSB => {
                if prenum != 0 {
                    nvim_set_postponed_split(prenum);
                } else {
                    nvim_set_postponed_split(-1);
                }
                nvim_do_nv_ident(c_int::from(b'g'), xchar);
                nvim_set_postponed_split(0);
            }

            // =================================================================
            // Goto file in new tab: 'f', 'F'
            // =================================================================
            CH_F | CH_F_UPPER => {
                nvim_set_cmdmod_tab_to_curtab_idx();
                do_window_goto_file(xchar, prenum1);
            }

            // =================================================================
            // Goto tabpage: 't'
            // =================================================================
            CH_T => {
                nvim_goto_tabpage(prenum);
            }

            // =================================================================
            // Goto tabpage backwards: 'T'
            // =================================================================
            CH_T_UPPER => {
                nvim_goto_tabpage(-prenum1);
            }

            // =================================================================
            // Goto last used tabpage: Tab
            // =================================================================
            TAB => {
                if !nvim_goto_tabpage_lastused() {
                    nvim_beep_flush_wrapper();
                }
            }

            // =================================================================
            // External window: 'e'
            // =================================================================
            CH_E => {
                do_window_g_external();
            }

            // =================================================================
            // Default: beep
            // =================================================================
            _ => {
                nvim_beep_flush_wrapper();
            }
        }
    }
}

// =============================================================================
// Phase 4: CTRL-W subcommand implementations (migrated from C)
// =============================================================================

/// C return value for OK.
const OK: c_int = 1;

/// GETF_ALT flag: jumping to alternate file.
const GETF_ALT: c_int = 0x02;

/// Check if a floating window is hidden or non-focusable (skip during navigation).
#[inline]
unsafe fn is_hidden_float(wp: WinHandle) -> bool {
    nvim_win_get_floating(wp) != 0
        && (nvim_win_get_config_hide(wp) != 0 || nvim_win_get_config_focusable(wp) == 0)
}

/// Rust implementation of `nvim_do_window_wW`.
///
/// 'w'/'W' and Ctrl-W navigation: move cursor to next/prev/Nth window,
/// skipping hidden or non-focusable floating windows.
///
/// # Safety
///
/// Called from C via FFI. Accesses global state through C accessors.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_do_window_wW(nchar: c_int, prenum: c_int) {
    // ONE_WINDOW = firstwin == lastwin. With Prenum==1 we still navigate.
    let firstwin = nvim_get_firstwin();
    let lastwin = nvim_get_lastwin();
    if firstwin == lastwin && prenum != 1 {
        nvim_beep_flush_wrapper();
        return;
    }

    let curwin = nvim_get_curwin();
    let wp;

    if prenum != 0 {
        // Jump to window number `prenum`, skipping hidden/non-focusable floats.
        let mut last_focusable = firstwin;
        let mut w = firstwin;
        let mut remaining = prenum;
        loop {
            remaining -= 1;
            if remaining <= 0 {
                break;
            }
            if !is_hidden_float(w) {
                last_focusable = w;
            }
            let next = nvim_win_get_next(w);
            if next.is_null() {
                break;
            }
            w = next;
        }
        // Skip past any trailing hidden/non-focusable floats.
        while !w.is_null() && is_hidden_float(w) {
            w = nvim_win_get_next(w);
        }
        wp = if w.is_null() { last_focusable } else { w };
    } else if nchar == CH_W_UPPER {
        // 'W': move backwards (wrapping at firstwin -> lastwin).
        let mut w = nvim_win_get_prev(curwin);
        if w.is_null() {
            w = lastwin;
        }
        while !w.is_null() && is_hidden_float(w) {
            w = nvim_win_get_prev(w);
        }
        wp = w;
    } else {
        // 'w' / Ctrl-W: move forwards (wrapping at lastwin -> firstwin).
        let mut w = nvim_win_get_next(curwin);
        while !w.is_null() && is_hidden_float(w) {
            w = nvim_win_get_next(w);
        }
        wp = if w.is_null() { firstwin } else { w };
    }

    rs_win_goto(wp);
}

/// Rust implementation of `nvim_do_window_P`.
///
/// 'P': jump to the preview window.
///
/// # Safety
///
/// Called from C via FFI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_do_window_P() {
    let mut found: WinHandle = WinHandle::null();

    // Walk all windows in current tab via firstwin/w_next.
    let mut wp = nvim_get_firstwin();
    while !wp.is_null() {
        if nvim_win_get_pvw(wp) != 0 {
            found = wp;
            break;
        }
        wp = nvim_win_get_next(wp);
    }

    if found.is_null() {
        nvim_emsg_id(EMSG_E441_NO_PREVIEW);
    } else {
        rs_win_goto(found);
    }
}

/// Rust implementation of `nvim_do_window_T`.
///
/// 'T': move current window to a new tab page.
///
/// # Safety
///
/// Called from C via FFI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_do_window_T(prenum: c_int) {
    let curwin = nvim_get_curwin();
    if nvim_one_window_curwin(curwin, TabpageHandle::null()) != 0 {
        nvim_msg_onlyone();
        return;
    }

    let oldtab = nvim_get_curtab();
    let wp = curwin;

    if rs_win_new_tabpage(prenum, std::ptr::null()) == OK && crate::rs_valid_tabpage(oldtab) != 0 {
        let newtab = nvim_get_curtab();
        nvim_al_goto_tabpage_tp(oldtab, 1, 1);
        if nvim_get_curwin() == wp {
            nvim_al_win_close(nvim_get_curwin(), 0, 0);
        }
        if crate::rs_valid_tabpage(newtab) != 0 {
            nvim_al_goto_tabpage_tp(newtab, 1, 1);
            nvim_apply_autocmds_tabnewentered();
        }
    }
}

/// Rust implementation of `nvim_do_window_hat`.
///
/// '^': split window and edit alternate (or Nth) buffer.
///
/// # Safety
///
/// Called from C via FFI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_do_window_hat(prenum: c_int) {
    nvim_reset_visual_wrapper();

    let curwin = nvim_get_curwin();
    let alt_fnum = if prenum == 0 {
        nvim_win_get_alt_fnum(curwin)
    } else {
        prenum
    };

    // Check that buffer exists.
    let buf: BufHandle = nvim_buflist_findnr(alt_fnum);
    if buf.is_null() {
        if prenum == 0 {
            nvim_emsg_id(EMSG_NOALT);
        } else {
            nvim_semsg_e92_buf_not_found(i64::from(prenum));
        }
        return;
    }

    if nvim_curbuf_locked() == 0 && rs_win_split(0, 0) == OK {
        nvim_buflist_getfile(alt_fnum, 0, GETF_ALT, 0);
    }
}

/// Rust implementation of `nvim_do_window_new`.
///
/// 'n'/'N' or after quickfix: open a new window.
/// nchar selects horizontal ('n') vs vertical ('v'/'V'/Ctrl-V) split.
///
/// # Safety
///
/// Called from C via FFI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_do_window_new(nchar: c_int, prenum: c_int) {
    let mut buf = [0u8; 40];
    let mut pos = 0usize;

    // Write count prefix if non-zero.
    if prenum != 0 {
        let s = prenum.to_string();
        let bytes = s.as_bytes();
        let copy_len = bytes.len().min(buf.len() - 4); // reserve "vnew\0"
        buf[..copy_len].copy_from_slice(&bytes[..copy_len]);
        pos += copy_len;
    }

    // Vertical flag.
    if (nchar == CH_V || nchar == CTRL_V) && pos < buf.len() - 4 {
        buf[pos] = b'v';
        pos += 1;
    }

    // Append "new".
    for &b in b"new" {
        if pos < buf.len() - 1 {
            buf[pos] = b;
            pos += 1;
        }
    }
    // NUL terminator already in place (array is zero-initialized).
    nvim_do_cmdline_cmd_wrapper(buf.as_ptr());
}

/// Rust implementation of `nvim_cmd_with_count_exec`.
///
/// Build "cmd[count]" string and execute it.
///
/// # Safety
///
/// Called from C via FFI.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_cmd_with_count_exec(cmd: *const u8, prenum: i64) {
    let cmd_str = std::ffi::CStr::from_ptr(cmd.cast());
    let cmd_bytes = cmd_str.to_bytes();

    let mut buf = [0u8; 40];
    let cmd_len = cmd_bytes.len().min(buf.len() - 1);
    buf[..cmd_len].copy_from_slice(&cmd_bytes[..cmd_len]);

    if prenum > 0 && cmd_len < buf.len() - 1 {
        let suffix = prenum.to_string();
        let suffix_bytes = suffix.as_bytes();
        let copy_len = suffix_bytes.len().min(buf.len() - 1 - cmd_len);
        buf[cmd_len..cmd_len + copy_len].copy_from_slice(&suffix_bytes[..copy_len]);
    }

    nvim_do_cmdline_cmd_wrapper(buf.as_ptr());
}
