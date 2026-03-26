//! do_cmdline: command-line execution loop and helpers.
//!
//! This module contains:
//! - `getline_equal` and `getline_cookie`
//! - Rust types: `WcmdT`, `LoopCookie`, `DbgStuff`
//! - `store_loop_line`, `get_loop_line` (exported for C symbol compatibility)
//! - `save_dbg_stuff`, `restore_dbg_stuff` (exported for C symbol compatibility)
//! - `do_cmdline` (the main command execution loop)

// Phase 1 scaffolding: many items here will be used in later phases
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void};

use crate::do_one_cmd::LineGetter;
use nvim_ex_eval::CstackT;

// =============================================================================
// DOCMD_ flag constants (from ex_docmd.h)
// =============================================================================

const DOCMD_NOWAIT: c_int = 0x02;
const DOCMD_REPEAT: c_int = 0x04;
const DOCMD_KEYTYPED: c_int = 0x08;
const DOCMD_EXCRESET: c_int = 0x10;
const DOCMD_KEEPLINE: c_int = 0x20;

// =============================================================================
// cstack flag constants (from ex_eval_defs.h)
// =============================================================================

const CSF_ACTIVE: c_int = 0x0002;
const CSF_WHILE: c_int = 0x0008;
const CSF_FOR: c_int = 0x0010;
const CSF_TRY: c_int = 0x0100;
const CSF_FINALLY: c_int = 0x0200;

const CSL_HAD_LOOP: c_int = 1;
const CSL_HAD_ENDLOOP: c_int = 2;
const CSL_HAD_CONT: c_int = 4;
const CSL_HAD_FINA: c_int = 8;

const CSTP_ERROR: c_int = 1;
const CSTP_INTERRUPT: c_int = 2;
const CSTP_THROW: c_int = 4;

// OK/FAIL constants
const OK: c_int = 1;
const FAIL: c_int = 0;

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
///
/// IMPORTANT: `did_throw` is C `bool` (1 byte). The `int need_rethrow` after it
/// will be at offset +3 due to alignment. This repr(C) struct matches the C layout.
#[repr(C)]
pub struct DbgStuff {
    pub trylevel: c_int,
    pub force_abort: c_int,
    pub caught_stack: *mut c_void,
    pub vv_exception: *mut c_char,
    pub vv_throwpoint: *mut c_char,
    pub did_emsg: c_int,
    pub got_int: c_int,
    pub did_throw: bool, // C `bool` is 1 byte; followed by 3 bytes padding before need_rethrow
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

    // msg_list: pointer to the linked list of messages to be converted to exceptions
    pub(crate) static mut msg_list: *mut *mut c_void;
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

    // has_loop_cmd (Rust export from ex_eval crate)
    fn has_loop_cmd(p: *mut c_char) -> bool;

    // UI helpers
    fn nvim_docmd_ui_has_cmdline() -> c_int;
    fn nvim_docmd_ui_ext_cmdline_block_append(indent: usize, s: *const c_char);
    fn nvim_docmd_ui_ext_cmdline_block_leave();

    // Message helpers
    fn msg_verbose_cmd(lnum: LinenrT, s: *const c_char);
    fn msg_start();
    fn wait_return(redraw: c_int);

    // Debug helpers
    fn nvim_docmd_dbg_find_breakpoint(file: bool, fname: *mut c_char, after: LinenrT) -> LinenrT;
    fn nvim_docmd_dbg_breakpoint(name: *mut c_char, lnum: LinenrT);
    fn nvim_docmd_do_debug(cmd: *mut c_char);

    // Exception helpers
    fn nvim_docmd_c_do_errthrow(cstack: *mut c_void, cmdname: *const c_char);
    fn do_intthrow(cstack: *mut c_void) -> bool;
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
    fn line_breakcheck();
    fn nvim_docmd_getcmdline_colon(firstc: c_int, indent: c_int, do_concat: bool) -> *mut c_char;
    fn nvim_docmd_set_sourcing_lnum(lnum: LinenrT);
    fn nvim_get_sourcing_lnum_direct() -> LinenrT;
    fn nvim_get_sourcing_name() -> *const c_char;

    // v_exception / v_throwpoint
    fn nvim_docmd_v_exception(newval: *mut c_char) -> *mut c_char;
    fn nvim_docmd_v_throwpoint(newval: *mut c_char) -> *mut c_char;

    // do_cmdline_start / do_cmdline_end wrappers

    // Memory
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // GArray helpers (ga_append_via_ptr already exported from collections crate)
    fn ga_append_via_ptr(gap: *mut c_void, item_size: usize) -> *mut c_void;

    // Error messages for do_cmdline
    fn gettext(s: *const c_char) -> *const c_char;
    fn emsg(s: *const c_char);
    fn aborting() -> bool;
    fn nvim_docmd_PROF_YES() -> c_int;
    fn nvim_docmd_end_of_sourced_file_msg() -> *const c_char;
    fn nvim_docmd_end_of_function_msg() -> *const c_char;

    // do_one_cmd (already in Rust, same crate -- called via extern for clarity)
    fn do_one_cmd(
        cmdlinep: *mut *mut c_char,
        flags: c_int,
        cstack: *mut c_void,
        fgetline: LineGetter,
        cookie: *mut c_void,
    ) -> *mut c_char;

    // handle_did_throw (already in Rust, ex_docmd crate)
    fn handle_did_throw();

    // ga_init for GArray initialization
    fn ga_init(gap: *mut c_void, itemsize: c_int, growsize: c_int);
    fn ga_clear(gap: *mut c_void);

    // lines_ga deep clear helper
    fn nvim_docmd_ga_deep_clear_lines(gap: *mut c_void);
    fn nvim_docmd_strmove(dst: *mut c_char, src: *const c_char);
    fn nvim_docmd_free_emsg_silent_list(cstack: *mut c_void);
    fn nvim_docmd_get_sourcing_name_raw() -> *const c_char;
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
// Phase 3: save_dbg_stuff and restore_dbg_stuff
// =============================================================================

/// Save global exception/debug state into `dsp`, zeroing the globals.
/// Exported as `save_dbg_stuff` so the C `do_cmdline` can still call it.
///
/// # Safety
///
/// `dsp` must be a valid pointer to a `DbgStuff` struct.
#[no_mangle]
pub unsafe extern "C" fn save_dbg_stuff(dsp: *mut DbgStuff) {
    unsafe {
        (*dsp).trylevel = trylevel;
        trylevel = 0;
        (*dsp).force_abort = force_abort as c_int;
        force_abort = false;
        (*dsp).caught_stack = caught_stack;
        caught_stack = std::ptr::null_mut();
        (*dsp).vv_exception = nvim_docmd_v_exception(std::ptr::null_mut());
        (*dsp).vv_throwpoint = nvim_docmd_v_throwpoint(std::ptr::null_mut());

        // Necessary for debugging an inactive ":catch", ":finally", ":endtry".
        (*dsp).did_emsg = did_emsg;
        did_emsg = 0;
        (*dsp).got_int = got_int as c_int;
        got_int = false;
        (*dsp).did_throw = did_throw;
        did_throw = false;
        (*dsp).need_rethrow = need_rethrow as c_int;
        need_rethrow = false;
        (*dsp).check_cstack = check_cstack as c_int;
        check_cstack = false;
        (*dsp).current_exception = current_exception;
        current_exception = std::ptr::null_mut();
    }
}

/// Restore global exception/debug state from `dsp`.
/// Exported as `restore_dbg_stuff` so the C `do_cmdline` can still call it.
///
/// # Safety
///
/// `dsp` must be a valid pointer to a `DbgStuff` struct previously filled by
/// `save_dbg_stuff`.
#[no_mangle]
pub unsafe extern "C" fn restore_dbg_stuff(dsp: *mut DbgStuff) {
    unsafe {
        suppress_errthrow = false;
        trylevel = (*dsp).trylevel;
        force_abort = (*dsp).force_abort != 0;
        caught_stack = (*dsp).caught_stack;
        nvim_docmd_v_exception((*dsp).vv_exception);
        nvim_docmd_v_throwpoint((*dsp).vv_throwpoint);
        did_emsg = (*dsp).did_emsg;
        got_int = (*dsp).got_int != 0;
        did_throw = (*dsp).did_throw;
        need_rethrow = (*dsp).need_rethrow != 0;
        check_cstack = (*dsp).check_cstack != 0;
        current_exception = (*dsp).current_exception;
    }
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

// =============================================================================
// Phase 4: do_cmdline main body
// =============================================================================

/// Static recursion counter (Neovim is single-threaded, no mutex needed).
static mut RECURSIVE: c_int = 0;

/// Execute one Ex command line.
///
/// Exported as `do_cmdline` to replace the C implementation.
///
/// # Safety
///
/// All raw pointer arguments must be valid for their lifetime.
#[export_name = "do_cmdline"]
pub unsafe extern "C" fn rs_do_cmdline(
    cmdline: *mut c_char,
    fgetline: LineGetter,
    cookie: *mut c_void,
    flags: c_int,
) -> c_int {
    // local variables
    let mut next_cmdline: *mut c_char; // next cmd to execute
    let mut cmdline_copy: *mut c_char = std::ptr::null_mut(); // copy of cmd line
    let mut used_getline = false; // used "fgetline" to obtain command
    let mut msg_didout_before_start = false;
    let mut count: c_int = 0; // line number count
    let mut did_inc = false; // incremented RedrawingDisabled
    let mut did_block = false; // emitted cmdline_block event
    let mut retval: c_int = OK;

    // Conditional stack (stack-allocated, zeroed, cs_idx = -1)
    let mut cstack: CstackT = unsafe { std::mem::zeroed() };
    cstack.cs_idx = -1;

    // GArray for storing lines in while/for loops
    let mut lines_ga: GArrayRepr = GArrayRepr {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 1,
        ga_data: std::ptr::null_mut(),
    };

    let mut current_line: c_int = 0; // active line in lines_ga
    let mut fname: *mut c_char = std::ptr::null_mut(); // function or script name
    let mut breakpoint: *mut LinenrT = std::ptr::null_mut(); // ptr to breakpoint field
    let mut dbg_tick: *mut c_int = std::ptr::null_mut(); // ptr to dbg_tick field

    let mut debug_saved: DbgStuff = unsafe { std::mem::zeroed() };

    // "fgetline" and "cookie" passed to do_one_cmd()
    let mut cmd_getline: LineGetter;
    let mut cmd_cookie: *mut c_void;
    let mut cmd_loop_cookie: LoopCookie = LoopCookie {
        lines_gap: std::ptr::null_mut(),
        current_line: 0,
        repeating: 0,
        lc_getline: None,
        cookie: std::ptr::null_mut(),
    };

    // For every pair of do_cmdline()/do_one_cmd() calls, use an extra memory
    // location for storing error messages to be converted to an exception.
    let mut private_msg_list: *mut c_void = std::ptr::null_mut();
    let saved_msg_list = unsafe { msg_list };
    unsafe {
        msg_list = std::ptr::addr_of_mut!(private_msg_list).cast();
    }

    if unsafe { crate::execute::rs_do_cmdline_start() } == FAIL {
        unsafe {
            emsg(gettext(crate::E_COMMAND_TOO_RECURSIVE_STR.as_ptr()));
            nvim_docmd_c_do_errthrow(std::ptr::null_mut(), std::ptr::null());
            msg_list = saved_msg_list;
        }
        return FAIL;
    }

    // Initialize GArray for loop lines
    unsafe {
        ga_init(
            std::ptr::addr_of_mut!(lines_ga).cast(),
            std::mem::size_of::<WcmdT>() as c_int,
            10,
        );
    }

    let real_cookie = unsafe { getline_cookie(fgetline, cookie) };

    // Inside a function use a higher nesting level.
    let mut getline_is_func =
        unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_func_line_ptr()) };
    if getline_is_func && unsafe { ex_nesting_level == nvim_docmd_func_level(real_cookie) } {
        unsafe { ex_nesting_level += 1 };
    }

    // Get the function or script name and debug breakpoint info.
    if getline_is_func {
        unsafe {
            fname = nvim_docmd_func_name(real_cookie);
            breakpoint = nvim_docmd_func_breakpoint(real_cookie);
            dbg_tick = nvim_docmd_func_dbg_tick(real_cookie);
        }
    } else if unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_getsourceline_ptr()) } {
        unsafe {
            fname = nvim_get_sourcing_name() as *mut c_char;
            breakpoint = nvim_docmd_source_breakpoint(real_cookie);
            dbg_tick = nvim_docmd_source_dbg_tick(real_cookie);
        }
    }

    // Initialize "force_abort" and "suppress_errthrow" at the top level.
    if unsafe { RECURSIVE == 0 } {
        unsafe {
            force_abort = false;
            suppress_errthrow = false;
        }
    }

    // If requested, store and reset the global values controlling the
    // exception handling (used when debugging).
    if flags & DOCMD_EXCRESET != 0 {
        unsafe { save_dbg_stuff(std::ptr::addr_of_mut!(debug_saved)) };
    } else {
        // CLEAR_FIELD: zero out debug_saved
        debug_saved = unsafe { std::mem::zeroed() };
    }

    let initial_trylevel = unsafe { trylevel };

    // "did_throw" will be set to true when an exception is being thrown.
    unsafe { did_throw = false };
    // "did_emsg" will be set to true when emsg() is used.
    unsafe { did_emsg = 0 };

    // KeyTyped is only set when calling vgetc(). Reset it here when not
    // calling vgetc() (sourced command lines).
    if flags & DOCMD_KEYTYPED == 0
        && !unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_getexline_ptr()) }
    {
        unsafe { KeyTyped = false };
    }

    // Continue executing command lines.
    next_cmdline = cmdline;
    loop {
        getline_is_func =
            unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_func_line_ptr()) };

        // stop skipping cmds for an error msg after all endif/while/for
        if next_cmdline.is_null()
            && !unsafe { force_abort }
            && cstack.cs_idx < 0
            && !(getline_is_func && unsafe { nvim_docmd_func_has_abort(real_cookie) } != 0)
        {
            unsafe { did_emsg = 0 };
        }

        // 1. If repeating a line in a loop, get a line from lines_ga.
        if cstack.cs_looplevel > 0 && current_line < lines_ga.ga_len {
            // Each '|' separated command is stored separately in lines_ga.
            // XFREE_CLEAR(cmdline_copy)
            unsafe {
                xfree(cmdline_copy.cast());
                cmdline_copy = std::ptr::null_mut();
            }

            // Check if a function has returned or aborted.
            if getline_is_func {
                if unsafe { do_profiling == nvim_docmd_PROF_YES() } {
                    unsafe { nvim_docmd_func_line_end(real_cookie) };
                }
                if unsafe { nvim_docmd_func_has_ended(real_cookie) } != 0 {
                    retval = FAIL;
                    break;
                }
            } else if unsafe { do_profiling == nvim_docmd_PROF_YES() }
                && unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_getsourceline_ptr()) }
            {
                unsafe { nvim_docmd_script_line_end() };
            }

            // Check if a sourced file hit a ":finish" command.
            if unsafe { nvim_docmd_c_source_finished(fgetline, cookie) } != 0 {
                retval = FAIL;
                break;
            }

            // If breakpoints have been added/deleted need to check for it.
            if !breakpoint.is_null() && !dbg_tick.is_null() && unsafe { *dbg_tick != debug_tick } {
                unsafe {
                    *breakpoint = nvim_docmd_dbg_find_breakpoint(
                        getline_equal(fgetline, cookie, nvim_docmd_get_getsourceline_ptr()),
                        fname,
                        nvim_get_sourcing_lnum_direct(),
                    );
                    *dbg_tick = debug_tick;
                }
            }

            // Get next_cmdline and SOURCING_LNUM from lines_ga.
            unsafe {
                let wcmd_ptr = (lines_ga.ga_data as *mut WcmdT).add(current_line as usize);
                next_cmdline = (*wcmd_ptr).line;
                nvim_docmd_set_sourcing_lnum((*wcmd_ptr).lnum);
            }

            // Did we encounter a breakpoint?
            if !breakpoint.is_null()
                && unsafe { *breakpoint != 0 }
                && unsafe { *breakpoint <= nvim_get_sourcing_lnum_direct() }
            {
                unsafe {
                    nvim_docmd_dbg_breakpoint(fname, nvim_get_sourcing_lnum_direct());
                    // Find next breakpoint.
                    *breakpoint = nvim_docmd_dbg_find_breakpoint(
                        getline_equal(fgetline, cookie, nvim_docmd_get_getsourceline_ptr()),
                        fname,
                        nvim_get_sourcing_lnum_direct(),
                    );
                    *dbg_tick = debug_tick;
                }
            }
            if unsafe { do_profiling == nvim_docmd_PROF_YES() } {
                if getline_is_func {
                    unsafe { nvim_docmd_func_line_start(real_cookie) };
                } else if unsafe {
                    getline_equal(fgetline, cookie, nvim_docmd_get_getsourceline_ptr())
                } {
                    unsafe { nvim_docmd_script_line_start() };
                }
            }
        }

        // 2. If no line given, get an allocated line with fgetline().
        if next_cmdline.is_null() {
            let indent: c_int = if cstack.cs_idx < 0 {
                0
            } else {
                (cstack.cs_idx + 1) * 2
            };

            if count == 1
                && unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_getexline_ptr()) }
            {
                if unsafe { nvim_docmd_ui_has_cmdline() } != 0 {
                    // Emit cmdline_block event for loop/conditional block.
                    unsafe {
                        nvim_docmd_ui_ext_cmdline_block_append(0, last_cmdline);
                    }
                    did_block = true;
                }
                // Need to set msg_didout for the first line after an ":if".
                unsafe { msg_didout = true };
            }

            if let Some(f) = fgetline {
                next_cmdline = unsafe { f(b':' as c_int, cookie, indent, true) };
            } else {
                next_cmdline = std::ptr::null_mut();
            }
            if next_cmdline.is_null() {
                // Don't call wait_return() for aborted command line.
                if unsafe { KeyTyped } && flags & DOCMD_REPEAT == 0 {
                    unsafe { need_wait_return = false };
                }
                retval = FAIL;
                break;
            }
            used_getline = true;

            // Emit all but the first cmdline_block event immediately.
            if unsafe { nvim_docmd_ui_has_cmdline() } != 0
                && count > 0
                && unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_getexline_ptr()) }
            {
                unsafe {
                    nvim_docmd_ui_ext_cmdline_block_append(indent as usize, next_cmdline);
                }
            }

            // Keep the first typed line. Clear it when more lines are typed.
            if flags & DOCMD_KEEPLINE != 0 {
                unsafe {
                    xfree(repeat_cmdline.cast());
                    if count == 0 {
                        repeat_cmdline = xstrdup(next_cmdline);
                    } else {
                        repeat_cmdline = std::ptr::null_mut();
                    }
                }
            }
        } else if cmdline_copy.is_null() {
            // 3. Make a copy of the command so we can mess with it.
            next_cmdline = unsafe { xstrdup(next_cmdline) };
        }
        cmdline_copy = next_cmdline;

        let mut current_line_before: c_int = 0;

        // Inside a while/for loop, or when the command looks like a ":while"/":for",
        // the line is stored.
        if cstack.cs_looplevel > 0 || unsafe { has_loop_cmd(next_cmdline as *mut c_char) } {
            // Set up get_loop_line
            cmd_getline = unsafe { nvim_docmd_get_loop_line_ptr() };
            cmd_cookie = std::ptr::addr_of_mut!(cmd_loop_cookie).cast();
            cmd_loop_cookie.lines_gap = std::ptr::addr_of_mut!(lines_ga);
            cmd_loop_cookie.current_line = current_line;
            cmd_loop_cookie.lc_getline = fgetline;
            cmd_loop_cookie.cookie = cookie;
            cmd_loop_cookie.repeating = (current_line < lines_ga.ga_len) as c_int;

            // Save the current line when encountering it the first time.
            if current_line == lines_ga.ga_len {
                unsafe {
                    store_loop_line(std::ptr::addr_of_mut!(lines_ga), next_cmdline);
                }
            }
            current_line_before = current_line;
        } else {
            cmd_getline = fgetline;
            cmd_cookie = cookie;
        }

        unsafe { did_endif = false };

        if count == 0 {
            // All output from the commands is put below each other.
            if flags & DOCMD_NOWAIT == 0 && unsafe { RECURSIVE } == 0 {
                unsafe {
                    msg_didout_before_start = msg_didout;
                    msg_didany = false; // no output yet
                    msg_start();
                    msg_scroll = 1; // put messages below each other (true)
                    no_wait_return += 1; // don't wait for return until finished
                    RedrawingDisabled += 1;
                }
                did_inc = true;
            }
        }
        count += 1;

        if (unsafe { p_verbose } >= 15 && !unsafe { nvim_docmd_get_sourcing_name_raw() }.is_null())
            || unsafe { p_verbose } >= 16
        {
            unsafe {
                msg_verbose_cmd(nvim_get_sourcing_lnum_direct(), cmdline_copy);
            }
        }

        // Execute one '|' separated command.
        unsafe { RECURSIVE += 1 };
        next_cmdline = unsafe {
            do_one_cmd(
                std::ptr::addr_of_mut!(cmdline_copy),
                flags,
                std::ptr::addr_of_mut!(cstack).cast(),
                cmd_getline,
                cmd_cookie,
            )
        };
        unsafe { RECURSIVE -= 1 };

        if std::ptr::eq(cmd_cookie, std::ptr::addr_of!(cmd_loop_cookie).cast()) {
            // Use "current_line" from "cmd_loop_cookie".
            current_line = cmd_loop_cookie.current_line;
        }

        if next_cmdline.is_null() {
            // XFREE_CLEAR(cmdline_copy)
            unsafe {
                xfree(cmdline_copy.cast());
                cmdline_copy = std::ptr::null_mut();
            }

            // If the command was typed, remember it for the ':' register.
            if unsafe {
                getline_equal(fgetline, cookie, nvim_docmd_get_getexline_ptr())
                    && !new_last_cmdline.is_null()
            } {
                unsafe {
                    xfree(last_cmdline.cast());
                    last_cmdline = new_last_cmdline;
                    new_last_cmdline = std::ptr::null_mut();
                }
            }
        } else {
            // need to copy the command after the '|' to cmdline_copy
            // STRMOVE(cmdline_copy, next_cmdline)
            unsafe { nvim_docmd_strmove(cmdline_copy, next_cmdline) };
            next_cmdline = cmdline_copy;
        }

        // reset did_emsg for a function that is not aborted by an error
        if unsafe { did_emsg } != 0
            && !unsafe { force_abort }
            && unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_func_line_ptr()) }
            && unsafe { nvim_docmd_func_has_abort(real_cookie) } == 0
        {
            unsafe { did_emsg = 0 };
        }

        if cstack.cs_looplevel > 0 {
            current_line += 1;

            // An ":endwhile", ":endfor" and ":continue" is handled here.
            if cstack.cs_lflags & (CSL_HAD_CONT | CSL_HAD_ENDLOOP) != 0 {
                cstack.cs_lflags &= !(CSL_HAD_CONT | CSL_HAD_ENDLOOP);

                // Jump back to the matching ":while" or ":for".
                if unsafe { did_emsg } == 0
                    && !unsafe { got_int }
                    && !unsafe { did_throw }
                    && cstack.cs_idx >= 0
                    && (cstack.cs_flags[cstack.cs_idx as usize] & (CSF_WHILE | CSF_FOR)) != 0
                    && cstack.cs_line[cstack.cs_idx as usize] >= 0
                    && (cstack.cs_flags[cstack.cs_idx as usize] & CSF_ACTIVE) != 0
                {
                    current_line = cstack.cs_line[cstack.cs_idx as usize];
                    // remember we jumped there
                    cstack.cs_lflags |= CSL_HAD_LOOP;
                    unsafe { line_breakcheck() }; // check if CTRL-C typed

                    // Check for the next breakpoint at or after the ":while"/":for".
                    if !breakpoint.is_null() && lines_ga.ga_len > current_line {
                        unsafe {
                            *breakpoint = nvim_docmd_dbg_find_breakpoint(
                                getline_equal(fgetline, cookie, nvim_docmd_get_getsourceline_ptr()),
                                fname,
                                (*(lines_ga.ga_data as *mut WcmdT).add(current_line as usize)).lnum
                                    - 1,
                            );
                            *dbg_tick = debug_tick;
                        }
                    }
                } else {
                    // can only get here with ":endwhile" or ":endfor"
                    if cstack.cs_idx >= 0 {
                        unsafe {
                            nvim_docmd_rewind_conditionals(
                                std::ptr::addr_of_mut!(cstack).cast(),
                                cstack.cs_idx - 1,
                                CSF_WHILE | CSF_FOR,
                                std::ptr::addr_of_mut!(cstack.cs_looplevel),
                            );
                        }
                    }
                }
            } else if cstack.cs_lflags & CSL_HAD_LOOP != 0 {
                // For a ":while" or ":for" we need to remember the line number.
                cstack.cs_lflags &= !CSL_HAD_LOOP;
                cstack.cs_line[cstack.cs_idx as usize] = current_line_before;
            }
        }

        // When not inside any ":while" loop, clear remembered lines.
        if cstack.cs_looplevel == 0 {
            if lines_ga.ga_len != 0 {
                unsafe {
                    let wcmd_last =
                        (lines_ga.ga_data as *mut WcmdT).add((lines_ga.ga_len - 1) as usize);
                    nvim_docmd_set_sourcing_lnum((*wcmd_last).lnum);
                    nvim_docmd_ga_deep_clear_lines(std::ptr::addr_of_mut!(lines_ga).cast());
                }
            }
            current_line = 0;
        }

        // A ":finally" makes did_emsg, got_int and did_throw pending for
        // being restored at the ":endtry".
        if cstack.cs_lflags & CSL_HAD_FINA != 0 {
            cstack.cs_lflags &= !CSL_HAD_FINA;
            let pending = cstack.cs_pending[cstack.cs_idx as usize] as c_int;
            let exc_val = if unsafe { did_throw } {
                unsafe { current_exception }
            } else {
                std::ptr::null_mut()
            };
            unsafe {
                nvim_docmd_report_make_pending(
                    pending & (CSTP_ERROR | CSTP_INTERRUPT | CSTP_THROW),
                    exc_val,
                );
                did_emsg = 0;
                got_int = false;
                did_throw = false;
            }
            cstack.cs_flags[cstack.cs_idx as usize] |= CSF_ACTIVE | CSF_FINALLY;
        }

        // Update global "trylevel" for recursive calls to do_cmdline().
        unsafe { trylevel = initial_trylevel + cstack.cs_trylevel };

        // If the outermost try conditional is done normally, reset force_abort.
        if unsafe { trylevel == 0 && did_emsg == 0 && !got_int && !did_throw } {
            unsafe { force_abort = false };
        }

        // Convert an interrupt to an exception if appropriate.
        unsafe {
            do_intthrow(std::ptr::addr_of_mut!(cstack).cast());
        }

        // Continue condition (C: do { ... } while (!(...)) )
        // Loop while:
        //   NOT (abort condition)
        //   AND NOT (error + getexline)
        //   AND (more cmds || cstack active || repeating)
        let abort_cond = (unsafe { got_int }
            || (unsafe { did_emsg } != 0 && unsafe { force_abort })
            || unsafe { did_throw })
            && cstack.cs_trylevel == 0;
        let getexline_err = unsafe { did_emsg } != 0
            && (cstack.cs_trylevel == 0 || unsafe { did_emsg_syntax })
            && used_getline
            && unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_getexline_ptr()) };
        let has_more = !next_cmdline.is_null() || cstack.cs_idx >= 0 || flags & DOCMD_REPEAT != 0;

        if abort_cond || getexline_err || !has_more {
            break;
        }
    } // end main loop

    // Post-loop cleanup
    unsafe {
        xfree(cmdline_copy.cast());
        did_emsg_syntax = false;
        nvim_docmd_ga_deep_clear_lines(std::ptr::addr_of_mut!(lines_ga).cast());
    }

    if cstack.cs_idx >= 0 {
        // If a sourced file or executed function ran to its end,
        // report the unclosed conditional.
        if !unsafe { got_int }
            && !unsafe { did_throw }
            && !unsafe { aborting() }
            && ((unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_getsourceline_ptr()) }
                && unsafe { nvim_docmd_c_source_finished(fgetline, cookie) } == 0)
                || (unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_func_line_ptr()) }
                    && unsafe { nvim_docmd_func_has_ended(real_cookie) } == 0))
        {
            if cstack.cs_flags[cstack.cs_idx as usize] & CSF_TRY != 0 {
                unsafe { emsg(gettext(crate::E_ENDTRY_STR.as_ptr())) };
            } else if cstack.cs_flags[cstack.cs_idx as usize] & CSF_WHILE != 0 {
                unsafe { emsg(gettext(crate::E_ENDWHILE_STR.as_ptr())) };
            } else if cstack.cs_flags[cstack.cs_idx as usize] & CSF_FOR != 0 {
                unsafe { emsg(gettext(crate::E_ENDFOR_STR.as_ptr())) };
            } else {
                unsafe { emsg(gettext(crate::E_ENDIF_STR.as_ptr())) };
            }
        }

        // Reset "trylevel" in case of a ":finish" or ":return" or a missing
        // ":endtry" in a sourced file or executed function.
        loop {
            let idx = unsafe {
                nvim_docmd_cleanup_conditionals(std::ptr::addr_of_mut!(cstack).cast(), 0, 1)
            };
            let idx_adj = if idx >= 0 { idx - 1 } else { idx };
            unsafe {
                nvim_docmd_rewind_conditionals(
                    std::ptr::addr_of_mut!(cstack).cast(),
                    idx_adj,
                    CSF_WHILE | CSF_FOR,
                    std::ptr::addr_of_mut!(cstack.cs_looplevel),
                );
            }
            if cstack.cs_idx < 0 {
                break;
            }
        }
        unsafe { trylevel = initial_trylevel };
    }

    // do_errthrow after rewinding the cstack.
    let errthrow_cmd = if unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_func_line_ptr()) }
    {
        c"endfunction".as_ptr()
    } else {
        std::ptr::null()
    };
    unsafe {
        nvim_docmd_c_do_errthrow(std::ptr::addr_of_mut!(cstack).cast(), errthrow_cmd);
    }

    if unsafe { trylevel } == 0 {
        if unsafe { did_throw } {
            unsafe { handle_did_throw() };
        } else if unsafe { got_int } || (unsafe { did_emsg } != 0 && unsafe { force_abort }) {
            unsafe { suppress_errthrow = true };
        }
    }

    // Set need_rethrow if an exception is being thrown.
    if unsafe { did_throw } {
        unsafe { need_rethrow = true };
    }

    if (unsafe {
        getline_equal(fgetline, cookie, nvim_docmd_get_getsourceline_ptr())
            && ex_nesting_level > nvim_docmd_source_level(real_cookie)
    }) || (unsafe {
        getline_equal(fgetline, cookie, nvim_docmd_get_func_line_ptr())
            && ex_nesting_level > nvim_docmd_func_level(real_cookie) + 1
    }) {
        if !unsafe { did_throw } {
            unsafe { check_cstack = true };
        }
    } else {
        // When leaving a function, reduce nesting level.
        if unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_func_line_ptr()) } {
            unsafe { ex_nesting_level -= 1 };
        }
        // Go to debug mode when returning from a function in which we are
        // single-stepping.
        if (unsafe {
            getline_equal(fgetline, cookie, nvim_docmd_get_getsourceline_ptr())
                || getline_equal(fgetline, cookie, nvim_docmd_get_func_line_ptr())
        }) && unsafe { ex_nesting_level < debug_break_level }
        {
            let msg =
                if unsafe { getline_equal(fgetline, cookie, nvim_docmd_get_getsourceline_ptr()) } {
                    unsafe { nvim_docmd_end_of_sourced_file_msg() }
                } else {
                    unsafe { nvim_docmd_end_of_function_msg() }
                };
            unsafe { nvim_docmd_do_debug(msg as *mut c_char) };
        }
    }

    // Restore the exception environment (done after returning from the debugger).
    if flags & DOCMD_EXCRESET != 0 {
        unsafe { restore_dbg_stuff(std::ptr::addr_of_mut!(debug_saved)) };
    }

    unsafe { msg_list = saved_msg_list };

    // Cleanup if "cs_emsg_silent_list" remains.
    unsafe {
        nvim_docmd_free_emsg_silent_list(std::ptr::addr_of_mut!(cstack).cast());
    }

    // If there was too much output to fit on the command line, ask the user to
    // hit return before redrawing the screen.
    if did_inc {
        unsafe {
            RedrawingDisabled -= 1;
            no_wait_return -= 1;
            msg_scroll = 0; // false
        }

        if retval == FAIL || (unsafe { did_endif && KeyTyped && did_emsg == 0 }) {
            unsafe {
                need_wait_return = false;
                msg_didany = false; // don't wait when restarting edit
            }
        } else if unsafe { need_wait_return } {
            unsafe {
                msg_didout |= msg_didout_before_start;
                wait_return(0); // false
            }
        }
    }

    if did_block {
        unsafe { nvim_docmd_ui_ext_cmdline_block_leave() };
    }

    unsafe { did_endif = false }; // in case do_cmdline used recursively

    unsafe { crate::execute::rs_do_cmdline_end() };
    retval
}
