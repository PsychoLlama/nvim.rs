//! do_cmdline: command-line execution loop and helpers.
//!
//! This module contains:
//! - `getline_equal` and `getline_cookie` (Phase 0)
//! - Rust types matching C structs: `WcmdT`, `LoopCookie`, `DbgStuff` (Phase 1)
//! - `store_loop_line`, `get_loop_line` (Phase 2)
//! - `save_dbg_stuff`, `restore_dbg_stuff` (Phase 3)
//! - `do_cmdline` main body (Phase 4)

// Phase 1 scaffolding: many items here will be used in later phases
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void};

use crate::do_one_cmd::LineGetter;

pub type LinenrT = i32;

// =============================================================================
// Rust types matching C structs (Phase 1)
// =============================================================================

/// Matches C `garray_T` layout (also matches collections::GArray).
/// Used to access ga_len and ga_data from loop_cookie.
#[repr(C)]
pub struct GArrayRepr {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

/// Matches C `wcmd_T`: a command line stored for `:while`/`:for` loop replay.
#[repr(C)]
pub struct WcmdT {
    pub line: *mut c_char,
    pub lnum: LinenrT,
}

/// Matches C `struct loop_cookie`.
#[repr(C)]
pub struct LoopCookie {
    pub lines_gap: *mut GArrayRepr,
    pub current_line: c_int,
    pub repeating: c_int,
    pub lc_getline: LineGetter,
    pub cookie: *mut c_void,
}

/// Matches C `struct dbg_stuff` -- fields saved/restored for debug mode.
#[repr(C)]
pub struct DbgStuff {
    pub trylevel: c_int,
    pub force_abort: c_int,
    pub caught_stack: *mut c_void,
    pub vv_exception: *mut c_char,
    pub vv_throwpoint: *mut c_char,
    pub did_emsg: c_int,
    pub got_int: c_int,
    pub did_throw: c_int,
    pub need_rethrow: c_int,
    pub check_cstack: c_int,
    pub current_exception: *mut c_void,
}

// =============================================================================
// C globals accessed directly (Phase 1 / Phase 4)
// =============================================================================

extern "C" {
    // Exception / error state
    pub(crate) static mut force_abort: bool;
    pub(crate) static mut did_emsg: c_int;
    pub(crate) static mut got_int: bool;
    pub(crate) static mut did_throw: bool;
    pub(crate) static mut trylevel: c_int;
    pub(crate) static mut suppress_errthrow: bool;
    pub(crate) static mut current_exception: *mut c_void;
    pub(crate) static mut need_rethrow: bool;
    pub(crate) static mut check_cstack: bool;
    pub(crate) static mut caught_stack: *mut c_void;
    pub(crate) static mut did_emsg_syntax: bool;
    pub(crate) static mut did_endif: bool;

    // Message/display state
    pub(crate) static mut msg_didout: bool;
    pub(crate) static mut msg_didany: bool;
    pub(crate) static mut msg_scroll: c_int;
    pub(crate) static mut no_wait_return: c_int;
    pub(crate) static mut need_wait_return: bool;
    pub(crate) static mut RedrawingDisabled: c_int;

    // Keyboard
    pub(crate) static mut KeyTyped: bool;

    // Command history
    pub(crate) static mut last_cmdline: *mut c_char;
    pub(crate) static mut new_last_cmdline: *mut c_char;
    pub(crate) static mut repeat_cmdline: *mut c_char;

    // Verbose / profiling
    pub(crate) static mut p_verbose: i64;
    pub(crate) static mut do_profiling: c_int;

    // Debug
    pub(crate) static mut debug_tick: c_int;
    pub(crate) static mut debug_break_level: c_int;

    // Nesting
    pub(crate) static mut ex_nesting_level: c_int;
}

// =============================================================================
// C accessor functions (Phase 1)
// =============================================================================

extern "C" {
    // Function pointer accessors
    fn nvim_docmd_get_loop_line_ptr() -> LineGetter;
    fn nvim_docmd_get_getsourceline_ptr() -> LineGetter;
    fn nvim_docmd_get_getexline_ptr() -> LineGetter;
    fn nvim_docmd_get_func_line_ptr() -> LineGetter;

    // Loop cookie field accessors
    fn nvim_docmd_loop_cookie_get_lc_getline(lc: *mut c_void) -> LineGetter;
    fn nvim_docmd_loop_cookie_get_cookie(lc: *mut c_void) -> *mut c_void;

    // userfunc helpers
    fn nvim_docmd_func_name(cookie: *mut c_void) -> *mut c_char;
    fn nvim_docmd_func_breakpoint(cookie: *mut c_void) -> *mut LinenrT;
    fn nvim_docmd_func_dbg_tick(cookie: *mut c_void) -> *mut c_int;
    fn nvim_docmd_func_has_abort(cookie: *mut c_void) -> c_int;
    fn nvim_docmd_func_has_ended(cookie: *mut c_void) -> c_int;
    fn nvim_docmd_func_level(cookie: *mut c_void) -> c_int;
    fn nvim_docmd_func_line_start(cookie: *mut c_void);
    fn nvim_docmd_func_line_end(cookie: *mut c_void);

    // runtime/source helpers
    fn nvim_docmd_c_source_finished(fgetline: LineGetter, cookie: *mut c_void) -> c_int;
    fn nvim_docmd_source_breakpoint(cookie: *mut c_void) -> *mut LinenrT;
    fn nvim_docmd_source_dbg_tick(cookie: *mut c_void) -> *mut c_int;
    fn nvim_docmd_source_level(cookie: *mut c_void) -> c_int;
    fn nvim_docmd_script_line_start();
    fn nvim_docmd_script_line_end();

    // has_loop_cmd
    fn nvim_docmd_has_loop_cmd(p: *const c_char) -> c_int;

    // UI helpers
    fn nvim_docmd_ui_has_cmdline() -> c_int;
    fn nvim_docmd_ui_ext_cmdline_block_append(indent: usize, s: *const c_char);
    fn nvim_docmd_ui_ext_cmdline_block_leave();

    // Message helpers
    fn nvim_docmd_msg_verbose_cmd(lnum: LinenrT, s: *mut c_char);
    fn nvim_docmd_msg_start();
    fn nvim_docmd_wait_return(redraw: c_int);

    // Debug helpers
    fn nvim_docmd_dbg_find_breakpoint(file: bool, fname: *mut c_char, after: LinenrT) -> LinenrT;
    fn nvim_docmd_dbg_breakpoint(name: *mut c_char, lnum: LinenrT);
    fn nvim_docmd_do_debug(cmd: *mut c_char);

    // Exception helpers
    fn nvim_docmd_c_do_errthrow(cstack: *mut c_void, cmdname: *const c_char);
    fn nvim_docmd_do_intthrow(cstack: *mut c_void) -> bool;
    fn nvim_docmd_report_make_pending(pending: c_int, value: *mut c_void);
    fn nvim_docmd_cleanup_conditionals(
        cstack: *mut c_void,
        searched_cond: c_int,
        inclusive: c_int,
    ) -> c_int;
    fn nvim_docmd_rewind_conditionals(
        cstack: *mut c_void,
        idx: c_int,
        cond_type: c_int,
        cond_level: *mut c_int,
    );

    // Misc helpers
    fn nvim_docmd_line_breakcheck();
    fn nvim_docmd_getcmdline_colon(firstc: c_int, indent: c_int, do_concat: bool) -> *mut c_char;
    fn nvim_docmd_set_sourcing_lnum(lnum: LinenrT);
    fn nvim_get_sourcing_lnum_direct() -> LinenrT;
    fn nvim_get_sourcing_name() -> *const c_char;

    // v_exception / v_throwpoint
    fn nvim_docmd_v_exception(newval: *mut c_char) -> *mut c_char;
    fn nvim_docmd_v_throwpoint(newval: *mut c_char) -> *mut c_char;

    // do_cmdline_start / do_cmdline_end wrappers
    fn nvim_do_cmdline_start() -> c_int;
    fn nvim_do_cmdline_end();

    // Memory
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // GArray helpers (ga_append_via_ptr already exported from collections crate)
    fn ga_append_via_ptr(gap: *mut c_void, item_size: usize) -> *mut c_void;

    // Error messages
    fn gettext(s: *const c_char) -> *const c_char;
}

// =============================================================================
// Helper: compare two `LineGetter` values by raw address
// =============================================================================

fn linegetter_eq(a: LineGetter, b: LineGetter) -> bool {
    match (a, b) {
        (Some(fa), Some(fb)) => std::ptr::fn_addr_eq(fa, fb),
        (None, None) => true,
        _ => false,
    }
}

// =============================================================================
// getline_equal / getline_cookie
// =============================================================================

/// If `fgetline` is `get_loop_line()`, return true if the getline it uses
/// equals `func`. Otherwise return true when `fgetline` equals `func`.
///
/// # Safety
///
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn getline_equal(
    fgetline: LineGetter,
    cookie: *mut c_void,
    func: LineGetter,
) -> bool {
    let loop_line_ptr = unsafe { nvim_docmd_get_loop_line_ptr() };
    let mut gp = fgetline;
    let mut cp = cookie;

    while linegetter_eq(gp, loop_line_ptr) {
        let new_gp = unsafe { nvim_docmd_loop_cookie_get_lc_getline(cp) };
        let new_cp = unsafe { nvim_docmd_loop_cookie_get_cookie(cp) };
        gp = new_gp;
        cp = new_cp;
    }
    linegetter_eq(gp, func)
}

/// If `fgetline` is `get_loop_line()`, return the cookie used by the original
/// getline function. Otherwise return `cookie`.
///
/// # Safety
///
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn getline_cookie(fgetline: LineGetter, cookie: *mut c_void) -> *mut c_void {
    let loop_line_ptr = unsafe { nvim_docmd_get_loop_line_ptr() };
    let mut gp = fgetline;
    let mut cp = cookie;

    while linegetter_eq(gp, loop_line_ptr) {
        let new_gp = unsafe { nvim_docmd_loop_cookie_get_lc_getline(cp) };
        let new_cp = unsafe { nvim_docmd_loop_cookie_get_cookie(cp) };
        gp = new_gp;
        cp = new_cp;
    }
    cp
}

// =============================================================================
// Phase 2: store_loop_line and get_loop_line
// =============================================================================

/// Store a line in `gap` so that a `:while` loop can execute it again.
/// Exported as `store_loop_line` so the C `do_cmdline` body can call it (Phase 2/4).
///
/// # Safety
///
/// `gap` must be a valid `garray_T *`. `line` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn store_loop_line(gap: *mut GArrayRepr, line: *const c_char) {
    // GA_APPEND_VIA_PTR(wcmd_T, gap)
    let p = unsafe { ga_append_via_ptr(gap.cast(), std::mem::size_of::<WcmdT>()) as *mut WcmdT };
    unsafe {
        (*p).line = xstrdup(line);
        (*p).lnum = nvim_get_sourcing_lnum_direct();
    }
}

/// Obtain a line when inside a `:while` or `:for` loop.
///
/// Exported as `get_loop_line` so the C `getline_equal` comparisons work.
///
/// # Safety
///
/// `cookie` must be a valid `*mut LoopCookie`.
#[export_name = "get_loop_line"]
pub unsafe extern "C" fn rs_get_loop_line(
    c: c_int,
    cookie: *mut c_void,
    indent: c_int,
    do_concat: bool,
) -> *mut c_char {
    let cp = cookie as *mut LoopCookie;

    // If we are at or past the last line in lines_gap, we need a new line.
    if unsafe { (*cp).current_line + 1 >= (*(*cp).lines_gap).ga_len } {
        if unsafe { (*cp).repeating } != 0 {
            // Trying to read past ":endwhile"/":endfor"
            return std::ptr::null_mut();
        }

        // First time inside the ":while"/":for": get line normally.
        let line = if unsafe { (*cp).lc_getline }.is_none() {
            unsafe { nvim_docmd_getcmdline_colon(c, indent, do_concat) }
        } else {
            let f = unsafe { (*cp).lc_getline.unwrap() };
            unsafe { f(c, (*cp).cookie, indent, do_concat) }
        };

        if !line.is_null() {
            unsafe { store_loop_line((*cp).lines_gap, line) };
            unsafe { (*cp).current_line += 1 };
        }

        return line;
    }

    // Replaying: return stored line.
    unsafe { KeyTyped = false };
    unsafe { (*cp).current_line += 1 };
    let wp = unsafe { ((*(*cp).lines_gap).ga_data as *mut WcmdT).add((*cp).current_line as usize) };
    unsafe {
        nvim_docmd_set_sourcing_lnum((*wp).lnum);
        xstrdup((*wp).line)
    }
}
