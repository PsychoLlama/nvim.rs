//! Command execution and quickfix helpers
//!
//! Implements `rs_exe_pre_commands`, `rs_exe_commands`, `rs_handle_quickfix`,
//! `rs_handle_tag`, `rs_edit_stdin` replacing static C functions in main.c.

use crate::setup::MparmT;
use std::ffi::{c_char, c_int};

// Constants from C headers

// SID_* (from globals.h)
const SID_CMDARG: c_int = -2;
const SID_CARG: c_int = -3;

// ETYPE_ARGS value: ETYPE_TOP=0,...,ARGS=6 (from runtime_defs.h)
const ETYPE_ARGS: c_int = 6;

// EDIT_QF = 4, EDIT_STDIN = 2 (from main.c local enums)
const EDIT_STDIN: c_int = 2;
const EDIT_QF: c_int = 4;

// IOSIZE = 1025 (from globals.h)
const IOSIZE: usize = 1025;

// sctx_T mirror (same as config.rs)
#[repr(C)]
#[derive(Clone, Copy)]
struct SctxT {
    sc_sid: c_int,
    sc_seq: c_int,
    sc_lnum: i32,
    sc_chan: u64,
}

// Extern C declarations
unsafe extern "C" {
    fn nvim_get_curwin() -> *mut std::ffi::c_void; // win_T* (opaque)
    fn nvim_win_get_cursor_lnum(wp: *mut std::ffi::c_void) -> i32;
    fn nvim_win_set_cursor_lnum(wp: *mut std::ffi::c_void, lnum: i32);
    fn estack_push(etype: c_int, name: *const c_char, lnum: i32);
    fn estack_pop();
    fn do_cmdline_cmd(cmd: *const c_char) -> c_int;
    fn xfree(p: *mut std::ffi::c_void);
    fn rs_qf_jump_newwin(
        wp: *mut std::ffi::c_void,
        dir: c_int,
        errornr: c_int,
        forceit: bool,
        newwin: bool,
    );
    fn vim_snprintf(buf: *mut c_char, len: usize, fmt: *const c_char, ...) -> c_int;
    fn qf_init(
        wp: *mut std::ffi::c_void,
        efile: *const c_char,
        errorformat: *mut c_char,
        newlist: c_int,
        qf_title: *const c_char,
        enc: *mut c_char,
    ) -> c_int;
    fn msg_putchar(c: c_int);
    fn os_exit(r: c_int) -> !;
    fn ui_call_error_exit(status: c_int);
    fn getout(exitval: c_int) -> !;
    static mut swap_exists_did_quit: bool;
    /// Thin C helper: set 'errorfile' option to val
    fn nvim_set_errorfile_opt(val: *const c_char);

    static mut current_sctx: SctxT;
    static mut msg_scroll: c_int;
    static mut exmode_active: bool;
    static mut headless_mode: bool;
    static mut embedded_mode: bool;
    static mut stdin_isatty: bool;
    static mut stdin_fd: c_int;

    // Option globals
    static mut p_ef: *mut c_char; // 'errorfile'
    static mut p_efm: *mut c_char; // 'errorformat'
    static mut p_menc: *mut c_char; // 'makeencoding'

    // IObuff global buffer
    static mut IObuff: [c_char; IOSIZE];
}

/// Execute commands from --cmd arguments.
///
/// # Safety
/// `parmp` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_exe_pre_commands(parmp: *mut MparmT) {
    let p = &*parmp;
    let cnt = p.n_pre_commands;

    if cnt <= 0 {
        return;
    }

    let curwin = nvim_get_curwin();
    nvim_win_set_cursor_lnum(curwin, 0);

    estack_push(ETYPE_ARGS, c"pre-vimrc command line".as_ptr(), 0);
    current_sctx.sc_sid = SID_CMDARG;

    for i in 0..cnt as usize {
        do_cmdline_cmd(p.pre_commands[i]);
    }

    estack_pop();
    current_sctx.sc_sid = 0;
    // TIME_MSG("--cmd commands") is a no-op in Rust
}

/// Execute "+", "-c" and "-S" arguments.
///
/// # Safety
/// `parmp` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_exe_commands(parmp: *mut MparmT) {
    let p = &mut *parmp;
    let curwin = nvim_get_curwin();

    msg_scroll = 1; // true
    if p.tagname.is_null() && nvim_win_get_cursor_lnum(curwin) <= 1 {
        nvim_win_set_cursor_lnum(curwin, 0);
    }

    estack_push(ETYPE_ARGS, c"command line".as_ptr(), 0);
    current_sctx.sc_sid = SID_CARG;
    current_sctx.sc_seq = 0;

    for i in 0..p.n_commands as usize {
        do_cmdline_cmd(p.commands[i]);
        if p.cmds_tofree[i] != 0 {
            xfree(p.commands[i] as *mut std::ffi::c_void);
        }
    }

    estack_pop();
    current_sctx.sc_sid = 0;

    if nvim_win_get_cursor_lnum(curwin) == 0 {
        nvim_win_set_cursor_lnum(curwin, 1);
    }

    if !exmode_active {
        msg_scroll = 0; // false
    }

    // When started with "-q errorfile" jump to first error again.
    if p.edit_type == EDIT_QF {
        rs_qf_jump_newwin(std::ptr::null_mut(), 0, 0, false, false);
    }
    // TIME_MSG("executing command arguments") is a no-op in Rust
}

/// Load the error file for "-q".
///
/// # Safety
/// `paramp` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_handle_quickfix(paramp: *mut MparmT) {
    let p = &*paramp;

    if p.edit_type != EDIT_QF {
        return;
    }

    if !p.use_ef.is_null() {
        nvim_set_errorfile_opt(p.use_ef);
    }

    // Build "cfile <p_ef>" in IObuff
    let iobuf = std::ptr::addr_of_mut!(IObuff) as *mut c_char;
    vim_snprintf(iobuf, IOSIZE, c"cfile %s".as_ptr(), p_ef);

    let result = qf_init(
        std::ptr::null_mut(),
        p_ef,
        p_efm,
        1, // newlist = true
        iobuf as *const c_char,
        p_menc,
    );
    if result < 0 {
        msg_putchar(b'\n' as c_int);
        os_exit(3);
    }
    // TIME_MSG("reading errorfile") is a no-op in Rust
}

/// Jump to tag for "-t".
///
/// # Safety
/// `tagname` may be null (no-op) or a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_handle_tag(tagname: *mut c_char) {
    if tagname.is_null() {
        return;
    }

    swap_exists_did_quit = false;

    let iobuf = std::ptr::addr_of_mut!(IObuff) as *mut c_char;
    vim_snprintf(iobuf, IOSIZE, c"ta %s".as_ptr(), tagname);
    do_cmdline_cmd(iobuf as *const c_char);
    // TIME_MSG("jumping to tag") is a no-op in Rust

    if swap_exists_did_quit {
        ui_call_error_exit(1);
        getout(1);
    }
}

/// Decides whether text will be read from stdin.
///
/// # Safety
/// `parmp` must be a valid pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_edit_stdin(parmp: *const MparmT) -> bool {
    let p = &*parmp;

    #[allow(clippy::nonminimal_bool)]
    let implicit = !headless_mode
        && !(embedded_mode && stdin_fd <= 0)
        && (!exmode_active || p.input_istext)
        && !stdin_isatty
        && p.edit_type <= EDIT_STDIN
        && p.scriptin.is_null();

    p.had_stdin_file || implicit
}
