//! Vim script debugger — Rust implementation of debugger.c
//!
//! This module ports the 18 functions from `src/nvim/debugger.c` to Rust.
//! It uses the opaque-handle + C-accessor pattern: all C struct access goes
//! through `nvim_dbg_*` accessor functions declared in `extern "C"`.

use std::ffi::{c_char, c_int, c_void};
use std::ptr::{self, addr_of_mut};

extern "C" {
    fn atoi(s: *const c_char) -> c_int;
}

// =============================================================================
// Constants (verified via _Static_assert in debugger.c)
// =============================================================================

const DBG_FUNC: c_int = 1;
const DBG_FILE: c_int = 2;
const DBG_EXPR: c_int = 3;

const OK: c_int = 1;
const FAIL: c_int = 0;

const K_SPECIAL: u8 = 0x80;
const KS_EXTRA: u8 = 253;
const KE_SNR: u8 = 82;

const ESTACK_NONE: c_int = 0;

#[allow(dead_code)]
const EXPAND_NOTHING: c_int = 0;

const MODE_NORMAL: c_int = 0x01;

const NUL: c_char = 0;

const RE_MAGIC: c_int = 1;
const RE_STRING: c_int = 2;

const UPD_NOT_VALID: c_int = 40;

const DOCMD_VERBOSE: c_int = 0x01;
const DOCMD_EXCRESET: c_int = 0x10;

const CMD_PROFILE: c_int = 331;
const CMD_PROFDEL: c_int = 332;
const CMD_BREAKDEL: c_int = 36;
const CMD_BREAKADD: c_int = 35;

const EXPR_IS: c_int = 9;

// Debug command constants (matching C #defines in do_debug)
const CMD_CONT: c_int = 1;
const CMD_NEXT: c_int = 2;
const CMD_STEP: c_int = 3;
const CMD_FINISH: c_int = 4;
const CMD_QUIT: c_int = 5;
const CMD_INTERRUPT: c_int = 6;
const CMD_BACKTRACE: c_int = 7;
const CMD_FRAME: c_int = 8;
const CMD_UP: c_int = 9;
const CMD_DOWN: c_int = 10;

// =============================================================================
// Opaque handles
// =============================================================================

/// Opaque handle for exarg_T (C struct)
#[repr(C)]
pub struct ExArgHandle {
    _opaque: [u8; 0],
}

/// Opaque handle for garray_T (C struct)
#[repr(C)]
pub struct GapHandle {
    _opaque: [u8; 0],
}

/// Opaque handle for typval_T (C struct)
type TypvalHandle = *mut c_void;

/// Opaque handle for regprog_T (C struct)
type RegprogHandle = *mut c_void;

/// Opaque handle for tasave_T (C struct)
type TypeaheadHandle = *mut c_void;

// =============================================================================
// Module state (replaces C static variables)
// =============================================================================

struct DebuggerModuleState {
    /// Batch mode debugging: don't save and restore typeahead
    debug_greedy: bool,
    /// Old value for debug expressions
    debug_oldval: *mut c_char,
    /// New value for debug expressions
    debug_newval: *mut c_char,
    /// Nr of last defined breakpoint
    last_breakp: c_int,
    /// Whether any expression breakpoint exists
    has_expr_breakpoint: bool,
    /// Function/file name for current breakpoint
    debug_breakpoint_name: *mut c_char,
    /// Line number for current breakpoint
    debug_breakpoint_lnum: i32,
    /// Debug was skipped
    debug_skipped: bool,
    /// Source name when debug was skipped
    debug_skipped_name: *mut c_char,
    /// Last debug command (persists across do_debug calls)
    last_cmd: c_int,
}

impl DebuggerModuleState {
    const fn new() -> Self {
        Self {
            debug_greedy: false,
            debug_oldval: ptr::null_mut(),
            debug_newval: ptr::null_mut(),
            last_breakp: 0,
            has_expr_breakpoint: false,
            debug_breakpoint_name: ptr::null_mut(),
            debug_breakpoint_lnum: 0,
            debug_skipped: false,
            debug_skipped_name: ptr::null_mut(),
            last_cmd: 0,
        }
    }
}

static mut STATE: DebuggerModuleState = DebuggerModuleState::new();

// =============================================================================
// Extern "C" — C accessor functions
// =============================================================================

extern "C" {
    // --- garray_T gap handle getters ---
    fn nvim_dbg_get_breakp_gap() -> *mut GapHandle;
    fn nvim_dbg_get_prof_gap() -> *mut GapHandle;

    // --- garray_T operations ---
    fn nvim_dbg_gap_len(gap: *mut GapHandle) -> c_int;
    fn nvim_dbg_gap_set_len(gap: *mut GapHandle, len: c_int);
    fn nvim_dbg_gap_grow(gap: *mut GapHandle, n: c_int);
    fn nvim_dbg_gap_clear(gap: *mut GapHandle);
    fn nvim_dbg_gap_is_empty(gap: *mut GapHandle) -> bool;

    // --- struct debuggy per-field accessors (by index) ---
    fn nvim_dbg_get_nr(gap: *mut GapHandle, idx: c_int) -> c_int;
    fn nvim_dbg_set_nr(gap: *mut GapHandle, idx: c_int, val: c_int);
    fn nvim_dbg_get_type(gap: *mut GapHandle, idx: c_int) -> c_int;
    fn nvim_dbg_set_type(gap: *mut GapHandle, idx: c_int, val: c_int);
    fn nvim_dbg_get_name(gap: *mut GapHandle, idx: c_int) -> *mut c_char;
    fn nvim_dbg_set_name(gap: *mut GapHandle, idx: c_int, val: *mut c_char);
    fn nvim_dbg_get_prog(gap: *mut GapHandle, idx: c_int) -> RegprogHandle;
    fn nvim_dbg_set_prog(gap: *mut GapHandle, idx: c_int, val: RegprogHandle);
    fn nvim_dbg_get_lnum(gap: *mut GapHandle, idx: c_int) -> i32;
    fn nvim_dbg_set_lnum(gap: *mut GapHandle, idx: c_int, val: i32);
    fn nvim_dbg_get_forceit(gap: *mut GapHandle, idx: c_int) -> c_int;
    fn nvim_dbg_set_forceit(gap: *mut GapHandle, idx: c_int, val: c_int);
    fn nvim_dbg_get_val(gap: *mut GapHandle, idx: c_int) -> TypvalHandle;
    fn nvim_dbg_set_val(gap: *mut GapHandle, idx: c_int, val: TypvalHandle);
    fn nvim_dbg_get_level(gap: *mut GapHandle, idx: c_int) -> c_int;
    fn nvim_dbg_set_level(gap: *mut GapHandle, idx: c_int, val: c_int);

    // --- gap entry removal (memmove helper) ---
    fn nvim_dbg_gap_remove_at(gap: *mut GapHandle, idx: c_int);

    // --- Global variable accessors ---
    fn nvim_dbg_get_sourcing_lnum() -> i64;

    // --- Buffer/window accessors ---
    fn nvim_dbg_curbuf_ffname() -> *mut c_char;
    fn nvim_dbg_curwin_cursor_lnum() -> i32;

    // --- ExArg accessors ---
    fn nvim_dbg_eap_get_arg(eap: *const ExArgHandle) -> *mut c_char;
    fn nvim_dbg_eap_get_cmd(eap: *const ExArgHandle) -> *mut c_char;
    fn nvim_dbg_eap_get_skip(eap: *const ExArgHandle) -> c_int;
    fn nvim_dbg_eap_set_skip(eap: *const ExArgHandle, val: c_int);
    fn nvim_dbg_eap_get_forceit(eap: *const ExArgHandle) -> c_int;
    fn nvim_dbg_eap_get_cmdidx(eap: *const ExArgHandle) -> c_int;
    fn nvim_dbg_eap_get_addr_count(eap: *const ExArgHandle) -> c_int;
    fn nvim_dbg_eap_get_line2(eap: *const ExArgHandle) -> i32;

    // --- Message wrappers ---
    fn nvim_dbg_msg_entering_debug();
    fn nvim_dbg_smsg_oldval(val: *const c_char);
    fn nvim_dbg_smsg_newval(val: *const c_char);
    fn nvim_dbg_smsg_line_cmd(lnum: i64, cmd: *const c_char);
    fn nvim_dbg_smsg_cmd(cmd: *const c_char);
    fn nvim_dbg_smsg_breakpoint(prefix: *const c_char, name: *const c_char, lnum: i64);
    fn nvim_dbg_smsg_frame_arrow(num: c_int, name: *const c_char);
    fn nvim_dbg_smsg_frame(num: c_int, name: *const c_char);
    fn nvim_dbg_msg_frame_zero();
    fn nvim_dbg_smsg_frame_highest(max: c_int);
    fn nvim_dbg_smsg_bp_func(nr: c_int, name: *const c_char, lnum: i64);
    fn nvim_dbg_smsg_bp_file(nr: c_int, name: *const c_char, lnum: i64);
    fn nvim_dbg_smsg_bp_expr(nr: c_int, name: *const c_char);
    fn nvim_dbg_msg_no_breakpoints();
    fn nvim_dbg_emsg_noname();
    fn nvim_dbg_semsg_invarg(arg: *const c_char);
    fn nvim_dbg_semsg_bp_not_found(arg: *const c_char);
    fn nvim_dbg_msg_str(s: *const c_char);

    // --- Eval wrappers ---
    fn nvim_dbg_eval_expr(name: *const c_char) -> TypvalHandle;
    fn nvim_dbg_typval_compare(
        tv1: TypvalHandle,
        tv2: TypvalHandle,
        ctype: c_int,
        ic: bool,
    ) -> c_int;
    fn nvim_dbg_typval_get_v_number(tv: TypvalHandle) -> i64;
    fn nvim_dbg_typval_tostring(tv: TypvalHandle) -> *mut c_char;
    fn nvim_dbg_tv_free(tv: TypvalHandle);

    // --- Command line wrappers ---
    fn nvim_dbg_getcmdline_prompt() -> *mut c_char;
    fn nvim_dbg_do_cmdline(cmd: *const c_char);
    fn nvim_dbg_do_cmdline_cmd(cmd: *const c_char);
    fn nvim_dbg_msg_starthere();

    // --- Typeahead wrappers ---
    fn nvim_dbg_save_typeahead() -> TypeaheadHandle;
    fn nvim_dbg_restore_typeahead(handle: TypeaheadHandle);

    // --- String/memory ---
    #[link_name = "xstrdup"]
    fn nvim_dbg_xstrdup(s: *const c_char) -> *mut c_char;
    #[link_name = "xfree"]
    fn nvim_dbg_xfree(p: *mut c_void);
    #[link_name = "xmalloc"]
    fn nvim_dbg_xmalloc(size: usize) -> *mut c_void;
    #[link_name = "skipwhite"]
    fn nvim_dbg_skipwhite(p: *const c_char) -> *mut c_char;
    fn nvim_dbg_getdigits_int32(pp: *mut *mut c_char) -> i32;
    fn strlen(s: *const c_char) -> usize;
    fn strcmp(a: *const c_char, b: *const c_char) -> c_int;
    fn strstr(a: *const c_char, b: *const c_char) -> *mut c_char;
    fn strncmp(a: *const c_char, b: *const c_char, n: usize) -> c_int;

    // --- Path/file wrappers ---
    fn nvim_dbg_expand_env_save(p: *const c_char) -> *mut c_char;
    fn nvim_dbg_fix_fname(p: *const c_char) -> *mut c_char;
    fn nvim_dbg_home_replace(name: *const c_char, buf: *mut c_char, buflen: c_int);
    fn nvim_dbg_file_pat_to_reg_pat(pat: *const c_char) -> *mut c_char;

    // --- Regex wrappers ---
    fn nvim_dbg_vim_regcomp(pat: *const c_char, flags: c_int) -> RegprogHandle;
    fn nvim_dbg_vim_regfree(prog: RegprogHandle);
    fn nvim_dbg_vim_regexec_prog(prog_ptr: *mut RegprogHandle, name: *const c_char) -> bool;

    // --- Screen/misc wrappers ---
    fn nvim_dbg_redraw_all_later(typ: c_int);
    fn nvim_dbg_estack_sfile(which: c_int) -> *mut c_char;

}

// =============================================================================
// Global variable extern statics (direct access, no accessor wrappers)
// =============================================================================

extern "C" {
    static mut debug_break_level: c_int;
    static mut debug_did_msg: bool;
    static mut debug_tick: c_int;
    static mut debug_backtrace_level: c_int;
    static mut debug_mode: bool;
    static mut msg_scroll: c_int;
    #[link_name = "State"]
    static mut vim_State: c_int;
    static mut did_emsg: c_int;
    static mut cmd_silent: bool;
    static mut msg_silent: c_int;
    static mut emsg_silent: c_int;
    static mut emsg_off: c_int;
    static mut redir_off: bool;
    #[link_name = "RedrawingDisabled"]
    static mut redrawing_disabled: c_int;
    static mut no_wait_return: c_int;
    static mut need_wait_return: bool;
    static mut ex_normal_busy: c_int;
    static mut ignore_script: bool;
    static mut lines_left: c_int;
    #[link_name = "Rows"]
    static mut vim_Rows: c_int;
    static mut cmdline_row: c_int;
    static mut msg_row: c_int;
    static mut ex_nesting_level: c_int;
    static mut got_int: bool;
    static mut NameBuff: [std::ffi::c_char; 4096];
}

// Compile-time constant for MAXPATHL (verified against C's MAXPATHL = 4096)
const MAXPATHL: c_int = 4096;

// =============================================================================
// Helper functions
// =============================================================================

/// Check if pointer points to NUL byte
unsafe fn is_nul(p: *const c_char) -> bool {
    unsafe { *p == NUL }
}

/// Read a byte from a char pointer
unsafe fn char_at(p: *const c_char) -> u8 {
    unsafe { *p as u8 }
}

// =============================================================================
// Phase 1: Trivial functions
// =============================================================================

/// Called when a breakpoint was encountered.
#[no_mangle]
pub unsafe extern "C" fn rs_dbg_breakpoint(name: *mut c_char, lnum: i32) {
    let state = &raw mut STATE;
    unsafe {
        (*state).debug_breakpoint_name = name;
        (*state).debug_breakpoint_lnum = lnum;
    }
}

/// ":debuggreedy" command
#[no_mangle]
pub unsafe extern "C" fn rs_ex_debuggreedy(eap: *const ExArgHandle) {
    let state = &raw mut STATE;
    unsafe {
        if nvim_dbg_eap_get_addr_count(eap) == 0 || nvim_dbg_eap_get_line2(eap) != 0 {
            (*state).debug_greedy = true;
        } else {
            (*state).debug_greedy = false;
        }
    }
}

/// ":debug" command
#[no_mangle]
pub unsafe extern "C" fn rs_ex_debug(eap: *const ExArgHandle) {
    unsafe {
        let save = debug_break_level;
        debug_break_level = 9999;
        nvim_dbg_do_cmdline_cmd(nvim_dbg_eap_get_arg(eap));
        debug_break_level = save;
    }
}

/// Count the maximum backtrace level by counting ".." in sname.
/// This is pure Rust — no FFI needed.
fn get_maxbacktrace_level(sname: *const c_char) -> c_int {
    if sname.is_null() {
        return 0;
    }
    let cstr = unsafe { std::ffi::CStr::from_ptr(sname) };
    let bytes = cstr.to_bytes();
    let mut count: c_int = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'.' && bytes[i + 1] == b'.' {
            count += 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    count
}

/// Set debug trace level from argument string.
unsafe fn do_setdebugtracelevel(arg: *const c_char) {
    unsafe {
        let level = atoi(arg);
        let first = *arg;
        if first == b'+' as c_char || level < 0 {
            let cur = debug_backtrace_level;
            debug_backtrace_level = cur + level;
        } else {
            debug_backtrace_level = level;
        }
        do_checkbacktracelevel();
    }
}

/// Check backtrace level bounds.
unsafe fn do_checkbacktracelevel() {
    unsafe {
        let bt_level = debug_backtrace_level;
        if bt_level < 0 {
            debug_backtrace_level = 0;
            nvim_dbg_msg_frame_zero();
        } else {
            let sname = nvim_dbg_estack_sfile(ESTACK_NONE);
            let max = get_maxbacktrace_level(sname);
            if bt_level > max {
                debug_backtrace_level = max;
                nvim_dbg_smsg_frame_highest(max);
            }
            nvim_dbg_xfree(sname as *mut c_void);
        }
    }
}

/// Update has_expr_breakpoint by scanning all breakpoints.
unsafe fn update_has_expr_breakpoint() {
    let state = &raw mut STATE;
    unsafe {
        (*state).has_expr_breakpoint = false;
        let gap = nvim_dbg_get_breakp_gap();
        let len = nvim_dbg_gap_len(gap);
        for i in 0..len {
            if nvim_dbg_get_type(gap, i) == DBG_EXPR {
                (*state).has_expr_breakpoint = true;
                break;
            }
        }
    }
}

/// FFI export for update_has_expr_breakpoint
#[no_mangle]
pub unsafe extern "C" fn rs_update_has_expr_breakpoint() {
    unsafe {
        update_has_expr_breakpoint();
    }
}

// =============================================================================
// Phase 2: Backtrace display
// =============================================================================

/// Show backtrace (":backtrace", ":where", ":frame" with no args)
unsafe fn do_showbacktrace(cmd: *const c_char) {
    unsafe {
        let sname = nvim_dbg_estack_sfile(ESTACK_NONE);
        let max = get_maxbacktrace_level(sname);
        if !sname.is_null() {
            let mut i: c_int = 0;
            let mut cur = sname;
            let dotdot = c"..".as_ptr();
            while !got_int {
                let next = strstr(cur, dotdot);
                if !next.is_null() {
                    // Temporarily NUL-terminate at ".."
                    *(next as *mut c_char) = NUL;
                }
                let bt_level = debug_backtrace_level;
                if i == max - bt_level {
                    nvim_dbg_smsg_frame_arrow(max - i, cur);
                } else {
                    nvim_dbg_smsg_frame(max - i, cur);
                }
                i += 1;
                if next.is_null() {
                    break;
                }
                // Restore the '.'
                *(next as *mut c_char) = b'.' as c_char;
                cur = next.add(2);
            }
            nvim_dbg_xfree(sname as *mut c_void);
        }

        let sourcing_lnum = nvim_dbg_get_sourcing_lnum();
        if sourcing_lnum != 0 {
            nvim_dbg_smsg_line_cmd(sourcing_lnum, cmd);
        } else {
            nvim_dbg_smsg_cmd(cmd);
        }
    }
}

// =============================================================================
// Phase 3: Breakpoint check
// =============================================================================

/// Go to debug mode when a breakpoint was encountered or "ex_nesting_level" is
/// at or below the break level. Called from do_one_cmd() before executing a command.
#[no_mangle]
pub unsafe extern "C" fn rs_dbg_check_breakpoint(eap: *const ExArgHandle) {
    let state = &raw mut STATE;
    unsafe {
        (*state).debug_skipped = false;
        if !(*state).debug_breakpoint_name.is_null() {
            if nvim_dbg_eap_get_skip(eap) == 0 {
                // replace K_SNR with "<SNR>"
                let name = (*state).debug_breakpoint_name;
                let prefix: *const c_char;
                let name_offset: usize;
                if char_at(name) == K_SPECIAL
                    && char_at(name.add(1)) == KS_EXTRA
                    && char_at(name.add(2)) == KE_SNR
                {
                    prefix = c"<SNR>".as_ptr();
                    name_offset = 3;
                } else {
                    prefix = c"".as_ptr();
                    name_offset = 0;
                }
                nvim_dbg_smsg_breakpoint(
                    prefix,
                    name.add(name_offset),
                    i64::from((*state).debug_breakpoint_lnum),
                );
                (*state).debug_breakpoint_name = ptr::null_mut();
                rs_do_debug(nvim_dbg_eap_get_cmd(eap));
            } else {
                (*state).debug_skipped = true;
                (*state).debug_skipped_name = (*state).debug_breakpoint_name;
                (*state).debug_breakpoint_name = ptr::null_mut();
            }
        } else if ex_nesting_level <= debug_break_level {
            if nvim_dbg_eap_get_skip(eap) == 0 {
                rs_do_debug(nvim_dbg_eap_get_cmd(eap));
            } else {
                (*state).debug_skipped = true;
                (*state).debug_skipped_name = ptr::null_mut();
            }
        }
    }
}

/// Go to debug mode if skipped by dbg_check_breakpoint() because eap->skip was set.
/// Returns true when the debug mode is entered this time.
#[no_mangle]
pub unsafe extern "C" fn rs_dbg_check_skipped(eap: *const ExArgHandle) -> bool {
    let state = &raw mut STATE;
    unsafe {
        if !(*state).debug_skipped {
            return false;
        }
        // Save the value of got_int and reset it.
        let prev_got_int = got_int;
        got_int = false;
        (*state).debug_breakpoint_name = (*state).debug_skipped_name;
        // eap->skip is true
        nvim_dbg_eap_set_skip(eap, 0);
        rs_dbg_check_breakpoint(eap);
        nvim_dbg_eap_set_skip(eap, 1);
        let cur_got_int = got_int;
        got_int = cur_got_int | prev_got_int;
        true
    }
}

// =============================================================================
// Phase 4: Argument parsing
// =============================================================================

/// Evaluate expression with error messages disabled.
unsafe fn eval_expr_no_emsg(gap: *mut GapHandle, idx: c_int) -> TypvalHandle {
    unsafe {
        let name = nvim_dbg_get_name(gap, idx);
        let save = emsg_off;
        emsg_off = save + 1;
        let tv = nvim_dbg_eval_expr(name);
        emsg_off = save;
        tv
    }
}

/// Parse the arguments of ":profile", ":breakadd" or ":breakdel".
/// Populates the entry at gap->ga_len. Returns OK or FAIL.
unsafe fn dbg_parsearg(arg: *mut c_char, gap: *mut GapHandle) -> c_int {
    unsafe {
        let mut p = arg;
        let mut here = false;
        let prof_gap = nvim_dbg_get_prof_gap();
        let is_prof = gap == prof_gap;

        nvim_dbg_gap_grow(gap, 1);

        let idx = nvim_dbg_gap_len(gap);

        // Find "func" or "file"
        if strncmp(p, c"func".as_ptr(), 4) == 0 {
            nvim_dbg_set_type(gap, idx, DBG_FUNC);
        } else if strncmp(p, c"file".as_ptr(), 4) == 0 {
            nvim_dbg_set_type(gap, idx, DBG_FILE);
        } else if !is_prof && strncmp(p, c"here".as_ptr(), 4) == 0 {
            if nvim_dbg_curbuf_ffname().is_null() {
                nvim_dbg_emsg_noname();
                return FAIL;
            }
            nvim_dbg_set_type(gap, idx, DBG_FILE);
            here = true;
        } else if !is_prof && strncmp(p, c"expr".as_ptr(), 4) == 0 {
            nvim_dbg_set_type(gap, idx, DBG_EXPR);
        } else {
            nvim_dbg_semsg_invarg(p);
            return FAIL;
        }
        p = nvim_dbg_skipwhite(p.add(4));

        // Find optional line number
        if here {
            nvim_dbg_set_lnum(gap, idx, nvim_dbg_curwin_cursor_lnum());
        } else if !is_prof && (*p as u8).is_ascii_digit() {
            let mut pp = p;
            let lnum = nvim_dbg_getdigits_int32(&mut pp);
            nvim_dbg_set_lnum(gap, idx, lnum);
            p = nvim_dbg_skipwhite(pp);
        } else {
            nvim_dbg_set_lnum(gap, idx, 0);
        }

        // Find the function or file name. Don't accept a function name with ().
        let bp_type = nvim_dbg_get_type(gap, idx);
        if (!here && is_nul(p))
            || (here && !is_nul(p))
            || (bp_type == DBG_FUNC && !strstr(p, c"()".as_ptr()).is_null())
        {
            nvim_dbg_semsg_invarg(arg);
            return FAIL;
        }

        if bp_type == DBG_FUNC {
            // Strip "g:" prefix
            let name = if strncmp(p, c"g:".as_ptr(), 2) == 0 {
                nvim_dbg_xstrdup(p.add(2))
            } else {
                nvim_dbg_xstrdup(p)
            };
            nvim_dbg_set_name(gap, idx, name);
        } else if here {
            nvim_dbg_set_name(gap, idx, nvim_dbg_xstrdup(nvim_dbg_curbuf_ffname()));
        } else if bp_type == DBG_EXPR {
            nvim_dbg_set_name(gap, idx, nvim_dbg_xstrdup(p));
            let val = eval_expr_no_emsg(gap, idx);
            nvim_dbg_set_val(gap, idx, val);
        } else {
            // Expand the file name in the same way as do_source(): twice.
            let q = nvim_dbg_expand_env_save(p);
            if q.is_null() {
                return FAIL;
            }
            let p2 = nvim_dbg_expand_env_save(q);
            nvim_dbg_xfree(q as *mut c_void);
            if p2.is_null() {
                return FAIL;
            }
            if char_at(p2) != b'*' {
                let name = nvim_dbg_fix_fname(p2);
                nvim_dbg_xfree(p2 as *mut c_void);
                nvim_dbg_set_name(gap, idx, name);
            } else {
                nvim_dbg_set_name(gap, idx, p2);
            }
        }

        if nvim_dbg_get_name(gap, idx).is_null() {
            return FAIL;
        }
        OK
    }
}

// =============================================================================
// Phase 5: Breakpoint management Ex commands
// =============================================================================

/// ":breakadd". Also used for ":profile".
#[no_mangle]
pub unsafe extern "C" fn rs_ex_breakadd(eap: *const ExArgHandle) {
    let state = &raw mut STATE;
    unsafe {
        let mut gap = nvim_dbg_get_breakp_gap();
        if nvim_dbg_eap_get_cmdidx(eap) == CMD_PROFILE {
            gap = nvim_dbg_get_prof_gap();
        }

        if dbg_parsearg(nvim_dbg_eap_get_arg(eap), gap) != OK {
            return;
        }

        let idx = nvim_dbg_gap_len(gap);
        nvim_dbg_set_forceit(gap, idx, nvim_dbg_eap_get_forceit(eap));

        if nvim_dbg_get_type(gap, idx) != DBG_EXPR {
            let name = nvim_dbg_get_name(gap, idx);
            let pat = nvim_dbg_file_pat_to_reg_pat(name);
            if !pat.is_null() {
                let prog = nvim_dbg_vim_regcomp(pat, RE_MAGIC + RE_STRING);
                nvim_dbg_set_prog(gap, idx, prog);
                nvim_dbg_xfree(pat as *mut c_void);
            }
            if pat.is_null() || nvim_dbg_get_prog(gap, idx).is_null() {
                nvim_dbg_xfree(nvim_dbg_get_name(gap, idx) as *mut c_void);
            } else {
                if nvim_dbg_get_lnum(gap, idx) == 0 {
                    nvim_dbg_set_lnum(gap, idx, 1);
                }
                if nvim_dbg_eap_get_cmdidx(eap) != CMD_PROFILE {
                    (*state).last_breakp += 1;
                    nvim_dbg_set_nr(gap, idx, (*state).last_breakp);
                    let tick = debug_tick;
                    debug_tick = tick + 1;
                }
                let len = nvim_dbg_gap_len(gap);
                nvim_dbg_gap_set_len(gap, len + 1);
            }
        } else {
            // DBG_EXPR
            (*state).last_breakp += 1;
            nvim_dbg_set_nr(gap, idx, (*state).last_breakp);
            let len = nvim_dbg_gap_len(gap);
            nvim_dbg_gap_set_len(gap, len + 1);
            let tick = debug_tick;
            debug_tick = tick + 1;
            let breakp_gap = nvim_dbg_get_breakp_gap();
            if gap == breakp_gap {
                (*state).has_expr_breakpoint = true;
            }
        }
    }
}

/// ":breakdel" and ":profdel"
#[no_mangle]
pub unsafe extern "C" fn rs_ex_breakdel(eap: *const ExArgHandle) {
    unsafe {
        let mut todel: c_int = -1;
        let mut del_all = false;
        let mut best_lnum: i32 = 0;
        let mut gap = nvim_dbg_get_breakp_gap();

        if nvim_dbg_eap_get_cmdidx(eap) == CMD_PROFDEL {
            gap = nvim_dbg_get_prof_gap();
        }

        let arg = nvim_dbg_eap_get_arg(eap);
        if (*arg as u8).is_ascii_digit() {
            // ":breakdel {nr}"
            let nr = atoi(arg);
            let len = nvim_dbg_gap_len(gap);
            for i in 0..len {
                if nvim_dbg_get_nr(gap, i) == nr {
                    todel = i;
                    break;
                }
            }
        } else if char_at(arg) == b'*' {
            todel = 0;
            del_all = true;
        } else {
            // ":breakdel {func|file|expr} [lnum] {name}"
            if dbg_parsearg(arg, gap) == FAIL {
                return;
            }
            let parse_idx = nvim_dbg_gap_len(gap);
            let len = nvim_dbg_gap_len(gap);
            for i in 0..len {
                let parse_type = nvim_dbg_get_type(gap, parse_idx);
                let i_type = nvim_dbg_get_type(gap, i);
                if parse_type == i_type
                    && strcmp(nvim_dbg_get_name(gap, parse_idx), nvim_dbg_get_name(gap, i)) == 0
                    && (nvim_dbg_get_lnum(gap, parse_idx) == nvim_dbg_get_lnum(gap, i)
                        || (nvim_dbg_get_lnum(gap, parse_idx) == 0
                            && (best_lnum == 0 || nvim_dbg_get_lnum(gap, i) < best_lnum)))
                {
                    todel = i;
                    best_lnum = nvim_dbg_get_lnum(gap, i);
                }
            }
            nvim_dbg_xfree(nvim_dbg_get_name(gap, parse_idx) as *mut c_void);
        }

        if todel < 0 {
            nvim_dbg_semsg_bp_not_found(arg);
            return;
        }

        while !nvim_dbg_gap_is_empty(gap) {
            nvim_dbg_xfree(nvim_dbg_get_name(gap, todel) as *mut c_void);
            if nvim_dbg_get_type(gap, todel) == DBG_EXPR && !nvim_dbg_get_val(gap, todel).is_null()
            {
                nvim_dbg_tv_free(nvim_dbg_get_val(gap, todel));
            }
            nvim_dbg_vim_regfree(nvim_dbg_get_prog(gap, todel));
            let len = nvim_dbg_gap_len(gap);
            nvim_dbg_gap_set_len(gap, len - 1);
            let new_len = nvim_dbg_gap_len(gap);
            if todel < new_len {
                nvim_dbg_gap_remove_at(gap, todel);
            }
            if nvim_dbg_eap_get_cmdidx(eap) == CMD_BREAKDEL {
                let tick = debug_tick;
                debug_tick = tick + 1;
            }
            if !del_all {
                break;
            }
        }

        // If all breakpoints were removed clear the array.
        if nvim_dbg_gap_is_empty(gap) {
            nvim_dbg_gap_clear(gap);
        }
        let breakp_gap = nvim_dbg_get_breakp_gap();
        if gap == breakp_gap {
            update_has_expr_breakpoint();
        }
    }
}

/// ":breaklist"
#[no_mangle]
pub unsafe extern "C" fn rs_ex_breaklist(_eap: *const ExArgHandle) {
    unsafe {
        let gap = nvim_dbg_get_breakp_gap();
        if nvim_dbg_gap_is_empty(gap) {
            nvim_dbg_msg_no_breakpoints();
            return;
        }

        let len = nvim_dbg_gap_len(gap);
        for i in 0..len {
            let bp_type = nvim_dbg_get_type(gap, i);
            if bp_type == DBG_FILE {
                nvim_dbg_home_replace(
                    nvim_dbg_get_name(gap, i),
                    addr_of_mut!(NameBuff).cast::<c_char>(),
                    MAXPATHL,
                );
            }
            if bp_type != DBG_EXPR {
                let nr = nvim_dbg_get_nr(gap, i);
                let lnum = i64::from(nvim_dbg_get_lnum(gap, i));
                if bp_type == DBG_FUNC {
                    nvim_dbg_smsg_bp_func(nr, nvim_dbg_get_name(gap, i), lnum);
                } else {
                    nvim_dbg_smsg_bp_file(nr, addr_of_mut!(NameBuff).cast::<c_char>(), lnum);
                }
            } else {
                let nr = nvim_dbg_get_nr(gap, i);
                nvim_dbg_smsg_bp_expr(nr, nvim_dbg_get_name(gap, i));
            }
        }
    }
}

// =============================================================================
// Phase 6: Breakpoint search
// =============================================================================

/// Common code for dbg_find_breakpoint() and has_profiling().
unsafe fn debuggy_find(
    file: bool,
    fname: *mut c_char,
    after: i32,
    gap: *mut GapHandle,
    fp: *mut bool,
) -> i32 {
    let state = &raw mut STATE;
    unsafe {
        let mut lnum: i32 = 0;
        let mut name: *mut c_char = fname;

        // Return quickly when there are no breakpoints.
        if nvim_dbg_gap_is_empty(gap) {
            return 0;
        }

        // Replace K_SNR in function name with "<SNR>".
        let mut name_allocated = false;
        if !file && char_at(fname) == K_SPECIAL {
            let fname_len = strlen(fname);
            let new_name = nvim_dbg_xmalloc(fname_len + 3) as *mut c_char;
            // Copy "<SNR>"
            let snr = c"<SNR>".as_ptr();
            std::ptr::copy_nonoverlapping(snr, new_name, 5);
            // Copy rest after the 3-byte K_SNR sequence
            std::ptr::copy_nonoverlapping(fname.add(3), new_name.add(5), fname_len - 3 + 1);
            name = new_name;
            name_allocated = true;
        }

        let prof_gap = nvim_dbg_get_prof_gap();
        let is_prof = gap == prof_gap;
        let len = nvim_dbg_gap_len(gap);

        for i in 0..len {
            let bp_type = nvim_dbg_get_type(gap, i);
            let bp_file = bp_type == DBG_FILE;

            if bp_file == file
                && bp_type != DBG_EXPR
                && (is_prof
                    || (nvim_dbg_get_lnum(gap, i) > after
                        && (lnum == 0 || nvim_dbg_get_lnum(gap, i) < lnum)))
            {
                // Save got_int, don't want previous interruption to cancel matching
                let prev_got_int = got_int;
                got_int = false;
                let mut prog = nvim_dbg_get_prog(gap, i);
                if nvim_dbg_vim_regexec_prog(&mut prog, name) {
                    // Update prog in case vim_regexec_prog changed it
                    nvim_dbg_set_prog(gap, i, prog);
                    lnum = nvim_dbg_get_lnum(gap, i);
                    if !fp.is_null() {
                        *fp = nvim_dbg_get_forceit(gap, i) != 0;
                    }
                } else {
                    nvim_dbg_set_prog(gap, i, prog);
                }
                let cur_got_int = got_int;
                got_int = cur_got_int | prev_got_int;
            } else if bp_type == DBG_EXPR {
                let mut line = false;

                let tv = eval_expr_no_emsg(gap, i);
                if !tv.is_null() {
                    let bp_val = nvim_dbg_get_val(gap, i);
                    if bp_val.is_null() {
                        nvim_dbg_xfree((*state).debug_oldval as *mut c_void);
                        (*state).debug_oldval = nvim_dbg_typval_tostring(ptr::null_mut());
                        nvim_dbg_set_val(gap, i, tv);
                        nvim_dbg_xfree((*state).debug_newval as *mut c_void);
                        (*state).debug_newval = nvim_dbg_typval_tostring(nvim_dbg_get_val(gap, i));
                        line = true;
                    } else {
                        if nvim_dbg_typval_compare(tv, bp_val, EXPR_IS, false) == OK
                            && nvim_dbg_typval_get_v_number(tv) == 0
                        {
                            line = true;
                            nvim_dbg_xfree((*state).debug_oldval as *mut c_void);
                            (*state).debug_oldval = nvim_dbg_typval_tostring(bp_val);
                            // Need to evaluate again, typval_compare() overwrites "tv".
                            let v = eval_expr_no_emsg(gap, i);
                            nvim_dbg_xfree((*state).debug_newval as *mut c_void);
                            (*state).debug_newval = nvim_dbg_typval_tostring(v);
                            nvim_dbg_tv_free(nvim_dbg_get_val(gap, i));
                            nvim_dbg_set_val(gap, i, v);
                        }
                        nvim_dbg_tv_free(tv);
                    }
                } else {
                    let bp_val = nvim_dbg_get_val(gap, i);
                    if !bp_val.is_null() {
                        nvim_dbg_xfree((*state).debug_oldval as *mut c_void);
                        (*state).debug_oldval = nvim_dbg_typval_tostring(bp_val);
                        nvim_dbg_xfree((*state).debug_newval as *mut c_void);
                        (*state).debug_newval = nvim_dbg_typval_tostring(ptr::null_mut());
                        nvim_dbg_tv_free(bp_val);
                        nvim_dbg_set_val(gap, i, ptr::null_mut());
                        line = true;
                    }
                }

                if line {
                    lnum = if after > 0 { after } else { 1 };
                    break;
                }
            }
        }

        if name_allocated {
            nvim_dbg_xfree(name as *mut c_void);
        }

        lnum
    }
}

/// Find a breakpoint for a function or sourced file.
/// Returns line number at which to break; zero when no matching breakpoint.
#[no_mangle]
pub unsafe extern "C" fn rs_dbg_find_breakpoint(file: bool, fname: *mut c_char, after: i32) -> i32 {
    unsafe {
        debuggy_find(
            file,
            fname,
            after,
            nvim_dbg_get_breakp_gap(),
            ptr::null_mut(),
        )
    }
}

/// Returns true if profiling is on for a function or sourced file.
#[no_mangle]
pub unsafe extern "C" fn rs_has_profiling(file: bool, fname: *mut c_char, fp: *mut bool) -> bool {
    unsafe { debuggy_find(file, fname, 0, nvim_dbg_get_prof_gap(), fp) != 0 }
}

// =============================================================================
// Phase 7: Interactive debug loop
// =============================================================================

/// Debug mode. Repeatedly get Ex commands, until told to continue normal execution.
#[no_mangle]
pub unsafe extern "C" fn rs_do_debug(cmd: *mut c_char) {
    let state = &raw mut STATE;
    unsafe {
        let save_msg_scroll = msg_scroll;
        let save_state = vim_State;
        let save_did_emsg = did_emsg;
        let save_cmd_silent = cmd_silent;
        let save_msg_silent = msg_silent;
        let save_emsg_silent = emsg_silent;
        let save_redir_off = redir_off;
        let mut typeahead_saved = false;
        let mut save_ignore_script: c_int = 0;
        let mut cmdline: *mut c_char = ptr::null_mut();
        let mut p: *mut c_char;
        let mut tail: *const c_char;

        redrawing_disabled = redrawing_disabled + 1;
        no_wait_return = no_wait_return + 1;
        did_emsg = 0;
        cmd_silent = false;
        msg_silent = 0;
        emsg_silent = 0;
        redir_off = true;

        vim_State = MODE_NORMAL;
        debug_mode = true;

        if !debug_did_msg {
            nvim_dbg_msg_entering_debug();
        }
        if !(*state).debug_oldval.is_null() {
            nvim_dbg_smsg_oldval((*state).debug_oldval);
            nvim_dbg_xfree((*state).debug_oldval as *mut c_void);
            (*state).debug_oldval = ptr::null_mut();
        }
        if !(*state).debug_newval.is_null() {
            nvim_dbg_smsg_newval((*state).debug_newval);
            nvim_dbg_xfree((*state).debug_newval as *mut c_void);
            (*state).debug_newval = ptr::null_mut();
        }
        let sname = nvim_dbg_estack_sfile(ESTACK_NONE);
        if !sname.is_null() {
            nvim_dbg_msg_str(sname);
        }
        nvim_dbg_xfree(sname as *mut c_void);
        let sourcing_lnum = nvim_dbg_get_sourcing_lnum();
        if sourcing_lnum != 0 {
            nvim_dbg_smsg_line_cmd(sourcing_lnum, cmd);
        } else {
            nvim_dbg_smsg_cmd(cmd);
        }

        // Repeat getting a command and executing it.
        let mut typeahead_handle: TypeaheadHandle = ptr::null_mut();

        loop {
            msg_scroll = 1;
            need_wait_return = false;

            let save_ex_normal_busy = ex_normal_busy;
            ex_normal_busy = 0;
            if !(*state).debug_greedy {
                typeahead_handle = nvim_dbg_save_typeahead();
                typeahead_saved = true;
                save_ignore_script = ignore_script as c_int;
                ignore_script = true;
            }

            // Don't debug any function call, e.g. from an expression mapping
            let n = debug_break_level;
            debug_break_level = -1;

            nvim_dbg_xfree(cmdline as *mut c_void);
            cmdline = nvim_dbg_getcmdline_prompt();

            debug_break_level = n;
            if typeahead_saved {
                nvim_dbg_restore_typeahead(typeahead_handle);
                ignore_script = save_ignore_script != 0;
                typeahead_saved = false;
            }
            ex_normal_busy = save_ex_normal_busy;

            cmdline_row = msg_row;
            nvim_dbg_msg_starthere();

            if !cmdline.is_null() {
                // If this is a debug command, set "last_cmd".
                // If not, reset "last_cmd".
                // For a blank line use previous command.
                p = nvim_dbg_skipwhite(cmdline);
                if !is_nul(p) {
                    let ch = char_at(p);
                    match ch {
                        b'c' => {
                            (*state).last_cmd = CMD_CONT;
                            tail = c"ont".as_ptr();
                        }
                        b'n' => {
                            (*state).last_cmd = CMD_NEXT;
                            tail = c"ext".as_ptr();
                        }
                        b's' => {
                            (*state).last_cmd = CMD_STEP;
                            tail = c"tep".as_ptr();
                        }
                        b'f' => {
                            (*state).last_cmd = 0;
                            if char_at(p.add(1)) == b'r' {
                                (*state).last_cmd = CMD_FRAME;
                                tail = c"rame".as_ptr();
                            } else {
                                (*state).last_cmd = CMD_FINISH;
                                tail = c"inish".as_ptr();
                            }
                        }
                        b'q' => {
                            (*state).last_cmd = CMD_QUIT;
                            tail = c"uit".as_ptr();
                        }
                        b'i' => {
                            (*state).last_cmd = CMD_INTERRUPT;
                            tail = c"nterrupt".as_ptr();
                        }
                        b'b' => {
                            (*state).last_cmd = CMD_BACKTRACE;
                            if char_at(p.add(1)) == b't' {
                                tail = c"t".as_ptr();
                            } else {
                                tail = c"acktrace".as_ptr();
                            }
                        }
                        b'w' => {
                            (*state).last_cmd = CMD_BACKTRACE;
                            tail = c"here".as_ptr();
                        }
                        b'u' => {
                            (*state).last_cmd = CMD_UP;
                            tail = c"p".as_ptr();
                        }
                        b'd' => {
                            (*state).last_cmd = CMD_DOWN;
                            tail = c"own".as_ptr();
                        }
                        _ => {
                            (*state).last_cmd = 0;
                            tail = c"".as_ptr(); // unused
                        }
                    }
                    if (*state).last_cmd != 0 {
                        // Check that the tail matches
                        p = p.add(1);
                        let mut t = tail;
                        while !is_nul(p) && *p == *t {
                            p = p.add(1);
                            t = t.add(1);
                        }
                        if (*p as u8).is_ascii_alphabetic() && (*state).last_cmd != CMD_FRAME {
                            (*state).last_cmd = 0;
                        }
                    }
                }

                if (*state).last_cmd != 0 {
                    // Execute debug command
                    match (*state).last_cmd {
                        CMD_CONT => {
                            debug_break_level = -1;
                        }
                        CMD_NEXT => {
                            debug_break_level = ex_nesting_level;
                        }
                        CMD_STEP => {
                            debug_break_level = 9999;
                        }
                        CMD_FINISH => {
                            debug_break_level = ex_nesting_level - 1;
                        }
                        CMD_QUIT => {
                            got_int = true;
                            debug_break_level = -1;
                        }
                        CMD_INTERRUPT => {
                            got_int = true;
                            debug_break_level = 9999;
                            // Do not repeat ">interrupt" cmd, continue stepping.
                            (*state).last_cmd = CMD_STEP;
                        }
                        CMD_BACKTRACE => {
                            do_showbacktrace(cmd);
                            continue;
                        }
                        CMD_FRAME => {
                            if is_nul(p) {
                                do_showbacktrace(cmd);
                            } else {
                                p = nvim_dbg_skipwhite(p);
                                do_setdebugtracelevel(p);
                            }
                            continue;
                        }
                        CMD_UP => {
                            let bt = debug_backtrace_level;
                            debug_backtrace_level = bt + 1;
                            do_checkbacktracelevel();
                            continue;
                        }
                        CMD_DOWN => {
                            let bt = debug_backtrace_level;
                            debug_backtrace_level = bt - 1;
                            do_checkbacktracelevel();
                            continue;
                        }
                        _ => {}
                    }
                    // Going out reset backtrace_level
                    debug_backtrace_level = 0;
                    break;
                }

                // don't debug this command
                let n = debug_break_level;
                debug_break_level = -1;
                nvim_dbg_do_cmdline(cmdline);
                debug_break_level = n;
            }
            let rows = vim_Rows;
            lines_left = rows - 1;
        }
        nvim_dbg_xfree(cmdline as *mut c_void);

        redrawing_disabled = redrawing_disabled - 1;
        no_wait_return = no_wait_return - 1;
        nvim_dbg_redraw_all_later(UPD_NOT_VALID);
        need_wait_return = false;
        msg_scroll = save_msg_scroll;
        let rows = vim_Rows;
        lines_left = rows - 1;
        vim_State = save_state;
        debug_mode = false;
        did_emsg = save_did_emsg;
        cmd_silent = save_cmd_silent;
        msg_silent = save_msg_silent;
        emsg_silent = save_emsg_silent;
        redir_off = save_redir_off;

        // Only print the message again when typing a command before coming back here.
        debug_did_msg = true;
    }
}

// =============================================================================
// Tests
