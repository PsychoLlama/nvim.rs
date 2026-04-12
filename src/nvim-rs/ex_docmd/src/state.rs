//! Static state for ex_docmd migrated from C to Rust.
//!
//! This module owns the following C statics that were previously in ex_docmd.c:
//! - `quitmore`
//! - `ex_pressedreturn`
//! - `filetype_detect`, `filetype_plugin`, `filetype_indent`
//! - `dollar_command`
//! - `exmode_plus`

#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

type WinHandle = *mut c_void;
type TabHandle = *mut c_void;

extern "C" {
    // Window/tab list traversal primitives (for current_win_nr etc.)
    fn nvim_get_curwin() -> WinHandle;
    fn nvim_get_lastwin() -> WinHandle;
    fn nvim_curtab_first_win() -> WinHandle;
    fn nvim_win_get_next_in_tab(wp: WinHandle) -> WinHandle;
    fn nvim_get_curtab() -> TabHandle;
    fn nvim_ex2_get_first_tabpage() -> TabHandle;
    fn nvim_ex2_tp_next(tp: TabHandle) -> TabHandle;
}

// =============================================================================
// quitmore
// =============================================================================

static QUITMORE: AtomicI32 = AtomicI32::new(0);

/// Get the current value of quitmore.
#[no_mangle]
pub extern "C" fn nvim_docmd_get_quitmore() -> c_int {
    QUITMORE.load(Ordering::Relaxed)
}

/// Set quitmore to a specific value.
#[no_mangle]
pub extern "C" fn nvim_docmd_set_quitmore(n: c_int) {
    QUITMORE.store(n, Ordering::Relaxed);
}

/// Decrement quitmore by 1.
#[no_mangle]
pub extern "C" fn nvim_docmd_dec_quitmore() {
    QUITMORE.fetch_sub(1, Ordering::Relaxed);
}

// =============================================================================
// ex_pressedreturn
// =============================================================================

static EX_PRESSEDRETURN: AtomicBool = AtomicBool::new(false);

/// Get ex_pressedreturn as int (0 or 1).
#[no_mangle]
pub extern "C" fn nvim_get_ex_pressedreturn() -> c_int {
    c_int::from(EX_PRESSEDRETURN.load(Ordering::Relaxed))
}

/// Set ex_pressedreturn.
#[no_mangle]
pub extern "C" fn nvim_set_ex_pressedreturn(val: bool) {
    EX_PRESSEDRETURN.store(val, Ordering::Relaxed);
}

// =============================================================================
// filetype_detect, filetype_plugin, filetype_indent (TriState: -1, 0, 1)
// =============================================================================
// TriState: kNone = 0, kFalse = -1, kTrue = 1

static FILETYPE_DETECT: AtomicI32 = AtomicI32::new(0); // kNone
static FILETYPE_PLUGIN: AtomicI32 = AtomicI32::new(0); // kNone
static FILETYPE_INDENT: AtomicI32 = AtomicI32::new(0); // kNone

/// Get filetype_detect as int.
#[no_mangle]
pub extern "C" fn nvim_docmd_get_filetype_detect() -> c_int {
    FILETYPE_DETECT.load(Ordering::Relaxed)
}

/// Set filetype_detect.
#[no_mangle]
pub extern "C" fn nvim_docmd_set_filetype_detect(val: c_int) {
    FILETYPE_DETECT.store(val, Ordering::Relaxed);
}

/// Get filetype_plugin as int.
#[no_mangle]
pub extern "C" fn nvim_docmd_get_filetype_plugin() -> c_int {
    FILETYPE_PLUGIN.load(Ordering::Relaxed)
}

/// Set filetype_plugin.
#[no_mangle]
pub extern "C" fn nvim_docmd_set_filetype_plugin(val: c_int) {
    FILETYPE_PLUGIN.store(val, Ordering::Relaxed);
}

/// Get filetype_indent as int.
#[no_mangle]
pub extern "C" fn nvim_docmd_get_filetype_indent() -> c_int {
    FILETYPE_INDENT.load(Ordering::Relaxed)
}

/// Set filetype_indent.
#[no_mangle]
pub extern "C" fn nvim_docmd_set_filetype_indent(val: c_int) {
    FILETYPE_INDENT.store(val, Ordering::Relaxed);
}

// =============================================================================
// dollar_command - static char array "$\0"
// Used as a pointer returned to callers (pointer identity not critical).
// =============================================================================

static DOLLAR_COMMAND: [u8; 2] = [b'$', 0];

/// Return pointer to the dollar_command string "$".
#[no_mangle]
pub extern "C" fn nvim_docmd_get_dollar_command() -> *mut c_char {
    DOLLAR_COMMAND.as_ptr() as *mut c_char
}

/// Return pointer to the dollar_command string "$" (alias for do_ecmd_cmd context).
#[no_mangle]
pub extern "C" fn nvim_docmd_get_do_ecmd_cmd_dollar() -> *mut c_char {
    DOLLAR_COMMAND.as_ptr() as *mut c_char
}

// =============================================================================
// exmode_plus - static char array "+\0"
// Used in pointer comparison: if cmd == exmode_plus+1 we skip print.
// Both C and Rust must get the same pointer, so Rust owns it and C calls us.
// =============================================================================

static EXMODE_PLUS: [u8; 2] = [b'+', 0];

/// Return pointer to the exmode_plus string "+".
#[no_mangle]
pub extern "C" fn nvim_docmd_get_exmode_plus() -> *mut c_char {
    EXMODE_PLUS.as_ptr() as *mut c_char
}

// =============================================================================
// Window / tab number counting
// Previously static C functions current_win_nr / current_tab_nr in ex_docmd.c,
// plus their macro wrappers and C accessor functions.
// =============================================================================

/// Count the window number of `win` in curtab (1-based).
/// If `win` is NULL, returns the total number of windows.
unsafe fn current_win_nr_impl(win: WinHandle) -> c_int {
    let mut nr = 0;
    let mut wp = nvim_curtab_first_win();
    while !wp.is_null() {
        nr += 1;
        if wp == win {
            break;
        }
        wp = nvim_win_get_next_in_tab(wp);
    }
    nr
}

/// Count the tab number of `tab` (1-based).
/// If `tab` is NULL, returns the total number of tab pages.
unsafe fn current_tab_nr_impl(tab: TabHandle) -> c_int {
    let mut nr = 0;
    let mut tp = nvim_ex2_get_first_tabpage();
    while !tp.is_null() {
        nr += 1;
        if tp == tab {
            break;
        }
        tp = nvim_ex2_tp_next(tp);
    }
    nr
}

/// Return the window number of curwin in curtab (CURRENT_WIN_NR).
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_current_win_nr() -> c_int {
    current_win_nr_impl(nvim_get_curwin())
}

/// Return the total number of windows in curtab (LAST_WIN_NR).
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_last_win_nr() -> c_int {
    current_win_nr_impl(std::ptr::null_mut())
}

/// Return the tab number of curtab (CURRENT_TAB_NR).
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_current_tab_nr() -> c_int {
    current_tab_nr_impl(nvim_get_curtab())
}

/// Return the total number of tab pages (LAST_TAB_NR).
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_last_tab_nr() -> c_int {
    current_tab_nr_impl(std::ptr::null_mut())
}

/// Return the nth window in curtab (1-based), or lastwin if not found.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_nth_window(nr: c_int) -> WinHandle {
    let mut winnr = 0;
    let mut wp = nvim_curtab_first_win();
    while !wp.is_null() {
        winnr += 1;
        if winnr == nr {
            return wp;
        }
        wp = nvim_win_get_next_in_tab(wp);
    }
    nvim_get_lastwin()
}
