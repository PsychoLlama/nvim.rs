//! CTRL-W command dispatcher.
//!
//! This module implements [`rs_do_window`], the Rust replacement for the
//! `do_window()` switch in `src/nvim/window.c`.  Each case delegates to
//! a C wrapper that performs the actual work (autocmds, memory allocation,
//! complex C-only logic).  The dispatch itself is pure Rust.

// Window dimensions may need truncation when converting between types.
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]

use std::ffi::c_int;

use crate::{TabpageHandle, WinHandle};

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

// WSP flags (verified by _Static_assert in window.c)
const WSP_VERT: c_int = 0x02;
const WSP_TOP: c_int = 0x08;
const WSP_BOT: c_int = 0x10;

// =============================================================================
// External C Functions
// =============================================================================

extern "C" {
    // --- Wrappers for complex cases (keep logic in C) ---
    fn nvim_do_window_wW(nchar: c_int, prenum: c_int);
    fn nvim_do_window_P();
    fn nvim_do_window_T(prenum: c_int);
    fn nvim_do_window_hat(prenum: c_int);
    fn nvim_do_window_new(nchar: c_int, prenum: c_int);
    fn nvim_do_window_equalize();
    fn nvim_do_window_tag(nchar: c_int, prenum: c_int);
    fn nvim_do_window_goto_file(nchar: c_int, prenum1: c_int);
    fn nvim_do_window_find_in_path(nchar: c_int, prenum: c_int, prenum1: c_int);
    fn nvim_do_window_g(prenum: c_int, xchar: c_int);
    fn rs_qf_view_result(split: bool);

    // --- Simple wrappers ---
    fn nvim_emsg_e_cmdwin();
    #[link_name = "rs_reset_VIsual_and_resel"]
    fn nvim_reset_visual_wrapper();
    fn nvim_bt_quickfix_curbuf() -> c_int;
    fn nvim_win_split_wrapper(size: c_int, flags: c_int) -> c_int;
    fn nvim_cmd_with_count_exec(cmd: *const u8, prenum: i64);
    fn nvim_do_cmdline_cmd_wrapper(cmd: *const u8) -> c_int;
    fn nvim_beep_flush_wrapper();
    #[link_name = "rs_one_window_in_tab"]
    fn nvim_one_window_curwin(wp: WinHandle, tp: TabpageHandle) -> c_int;
    fn nvim_msg_onlyone();
    fn nvim_win_goto_wrapper(wp: WinHandle);
    fn nvim_get_curtab() -> TabpageHandle;
    fn rs_win_vert_neighbor(tp: TabpageHandle, wp: WinHandle, up: c_int, count: c_int)
        -> WinHandle;
    fn rs_win_horz_neighbor(
        tp: TabpageHandle,
        wp: WinHandle,
        left: c_int,
        count: c_int,
    ) -> WinHandle;
    fn nvim_win_exchange_wrapper(prenum: c_int);
    fn nvim_win_rotate_wrapper(upwards: c_int, count: c_int);
    fn nvim_win_splitmove_wrapper(wp: WinHandle, size: c_int, flags: c_int) -> c_int;
    fn nvim_win_get_w_height(wp: WinHandle) -> c_int;
    fn nvim_win_get_w_width(wp: WinHandle) -> c_int;
    fn nvim_get_min_set_ch() -> i64;
    fn nvim_get_rows() -> c_int;
    fn nvim_get_columns() -> c_int;

    // --- Accessors ---
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_firstwin() -> WinHandle;
    fn nvim_get_cmdwin_type() -> c_int;
    #[link_name = "rs_lastwin_nofloating"]
    fn nvim_lastwin_nofloating_wrapper() -> WinHandle;
    fn nvim_get_valid_prevwin() -> WinHandle;

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
    if nvim_get_cmdwin_type() != 0 {
        nvim_emsg_e_cmdwin();
        true
    } else {
        false
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
                    nvim_do_window_new(nchar, prenum);
                    return;
                }
                nvim_win_split_wrapper(prenum, 0);
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
                    nvim_do_window_new(nchar, prenum);
                    return;
                }
                nvim_win_split_wrapper(prenum, WSP_VERT);
            }

            // =================================================================
            // Split and edit alternate file: Ctrl-^, '^'
            // =================================================================
            CTRL_HAT | CH_HAT => {
                if check_cmdwin() {
                    return;
                }
                nvim_do_window_hat(prenum);
            }

            // =================================================================
            // New window: Ctrl-N, 'n'
            // =================================================================
            CTRL_N | CH_N => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                nvim_do_window_new(nchar, prenum);
            }

            // =================================================================
            // Quit: Ctrl-Q, 'q'
            // =================================================================
            CTRL_Q | CH_Q => {
                nvim_reset_visual_wrapper();
                nvim_cmd_with_count_exec(c"quit".as_ptr().cast(), i64::from(prenum));
            }

            // =================================================================
            // Close: Ctrl-C, 'c'
            // =================================================================
            CTRL_C | CH_C => {
                nvim_reset_visual_wrapper();
                nvim_cmd_with_count_exec(c"close".as_ptr().cast(), i64::from(prenum));
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
                nvim_do_window_P();
            }

            // =================================================================
            // Close all but current: Ctrl-O, 'o'
            // =================================================================
            CTRL_O | CH_O => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                nvim_cmd_with_count_exec(c"only".as_ptr().cast(), i64::from(prenum));
            }

            // =================================================================
            // Cursor to next/prev window with wrap: Ctrl-W, 'w', 'W'
            // =================================================================
            CTRL_W | CH_W | CH_W_UPPER => {
                if check_cmdwin() {
                    return;
                }
                nvim_do_window_wW(nchar, prenum);
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
                    nvim_win_goto_wrapper(win);
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
                    nvim_win_goto_wrapper(win);
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
                    nvim_win_goto_wrapper(win);
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
                    nvim_win_goto_wrapper(win);
                }
            }

            // =================================================================
            // Move window to new tab: 'T'
            // =================================================================
            CH_T_UPPER => {
                if check_cmdwin() {
                    return;
                }
                nvim_do_window_T(prenum);
            }

            // =================================================================
            // Cursor to top-left window: 't', Ctrl-T
            // =================================================================
            CH_T | CTRL_T => {
                nvim_win_goto_wrapper(nvim_get_firstwin());
            }

            // =================================================================
            // Cursor to bottom-right window: 'b', Ctrl-B
            // =================================================================
            CH_B | CTRL_B => {
                nvim_win_goto_wrapper(nvim_lastwin_nofloating_wrapper());
            }

            // =================================================================
            // Cursor to last accessed (previous) window: 'p', Ctrl-P
            // =================================================================
            CH_P | CTRL_P => {
                let pw = nvim_get_valid_prevwin();
                if pw.is_null() {
                    nvim_beep_flush_wrapper();
                } else {
                    nvim_win_goto_wrapper(pw);
                }
            }

            // =================================================================
            // Exchange: 'x', Ctrl-X
            // =================================================================
            CH_X | CTRL_X => {
                if check_cmdwin() {
                    return;
                }
                nvim_win_exchange_wrapper(prenum);
            }

            // =================================================================
            // Rotate downwards: Ctrl-R, 'r'
            // =================================================================
            CTRL_R | CH_R => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                nvim_win_rotate_wrapper(0, prenum1);
            }

            // =================================================================
            // Rotate upwards: 'R'
            // =================================================================
            CH_R_UPPER => {
                if check_cmdwin() {
                    return;
                }
                nvim_reset_visual_wrapper();
                nvim_win_rotate_wrapper(1, prenum1);
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
                    nvim_win_splitmove_wrapper(nvim_get_curwin(), prenum, dir);
                }
            }

            // =================================================================
            // Equalize: '='
            // =================================================================
            CH_EQ => {
                nvim_do_window_equalize();
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
                    rs_win_setheight(nvim_get_rows() - nvim_get_min_set_ch() as c_int);
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
                    rs_win_setwidth(nvim_get_columns());
                }
            }

            // =================================================================
            // Tag/preview: '}', ']', Ctrl-]
            // =================================================================
            CH_RBRACE | CH_RBRACKET | CTRL_RSB => {
                if check_cmdwin() {
                    return;
                }
                nvim_do_window_tag(nchar, prenum);
            }

            // =================================================================
            // Goto file: 'f', 'F', Ctrl-F
            // =================================================================
            CH_F | CH_F_UPPER | CTRL_F => {
                if check_cmdwin() {
                    return;
                }
                nvim_do_window_goto_file(nchar, prenum1);
            }

            // =================================================================
            // Find in path: 'i', Ctrl-I, 'd', Ctrl-D
            // =================================================================
            CH_I | CTRL_I | CH_D | CTRL_D => {
                if check_cmdwin() {
                    return;
                }
                nvim_do_window_find_in_path(nchar, prenum, prenum1);
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
                nvim_do_window_g(prenum, xchar);
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
