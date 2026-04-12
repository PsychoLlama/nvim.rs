//! Phase 1 _impl function bodies migrated from ex_docmd.c.
//!
//! Contains Rust implementations of the medium-sized `_impl` functions
//! that were previously in C:
//! - check_more, tabpage_new_impl, expand_argopt_impl
//! - do_exedit_handle_exmode, do_exedit_split_fail_cleanup, do_exedit_split_fallback
//! - ex_read_impl, ex_terminal_impl, ex_restart_impl
//! - ex_detach_impl, ex_connect_impl, ex_checkhealth_impl
//! - did_set_findfunc_impl

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::ExArgHandle;

// Type aliases
type LinenrT = i32;
type BufHandle = *mut c_void;

/// Matches C `String` (api/private/defs.h): `{ char *data; size_t size }`.
#[repr(C)]
struct NvimString {
    data: *mut c_char,
    size: usize,
}

/// Local mirror of C `Error` struct (api_defs.h).
/// `err_type == -1` (kErrorTypeNone) means no error.
#[repr(C)]
struct CError {
    err_type: c_int,
    msg: *mut c_char,
}

impl CError {
    const fn init() -> Self {
        CError {
            err_type: -1,
            msg: std::ptr::null_mut(),
        }
    }
    fn is_set(&self) -> bool {
        self.err_type != -1
    }
}

// ============================================================================
// Constants
// ============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;

/// OPT_LOCAL from option.h
const OPT_LOCAL: c_int = 0x02;
/// OPT_GLOBAL from option.h
const OPT_GLOBAL: c_int = 0x01;

/// VIM_YES response value
const VIM_YES: c_int = 2;

/// CPO_ALTREAD: cpoptions flag for `:read` setting alt filename ('a')
const CPO_ALTREAD: c_int = b'a' as c_int;

/// UPD_VALID redraw type
const UPD_VALID: c_int = 10;
/// UPD_NOT_VALID redraw type
const UPD_NOT_VALID: c_int = 40;

/// MAXLNUM: maximum line number
const MAXLNUM: LinenrT = 0x7FFF_FFFF;

// ============================================================================
// FFI declarations
// ============================================================================

extern "C" {
    // eap accessors

    // message functions
    fn emsg(s: *const c_char);
    fn semsg(fmt: *const c_char, ...);

    // --- check_more helpers ---
    fn nvim_docmd_check_more_semsg(n: c_int);
    fn vim_dialog_yesno(
        type_: c_int,
        title: *const c_char,
        message: *const c_char,
        dflt: c_int,
    ) -> c_int;
    fn ngettext(s1: *const c_char, s2: *const c_char, n: std::ffi::c_ulong) -> *const c_char;
    fn nvim_get_argcount() -> c_int;
    fn nvim_win_get_arg_idx(wp: *mut c_void) -> c_int;
    fn nvim_al_get_arg_had_last() -> c_int;
    fn rs_only_one_window() -> c_int;
    fn nvim_get_p_confirm() -> c_int;
    fn nvim_get_cmdmod_confirm() -> c_int;

    // --- tabpage_new helper ---
    fn nvim_docmd_tabpage_new_body();

    // --- do_exedit_handle_exmode helpers ---
    static mut exmode_active: bool;
    fn nvim_get_global_busy() -> bool;
    fn stuffReadbuff(s: *const c_char);
    static mut RedrawingDisabled: c_int;
    static mut no_wait_return: c_int;
    static mut need_wait_return: bool;
    static mut msg_scroll: c_int;
    fn redraw_all_later(type_: c_int);
    static mut pending_exmode_active: bool;
    fn normal_enter(cmdwin: bool, noexmode: bool);
    // CMD_visual and CMD_view use crate::cmd_idx::{CMD_visual, CMD_view}.

    // --- do_exedit_split_fail_cleanup helpers ---
    fn curbufIsChanged() -> bool;
    fn nvim_buf_get_nwindows(buf: BufHandle) -> c_int;
    fn buf_hide(buf: BufHandle) -> bool;
    fn nvim_get_curbuf() -> BufHandle;
    fn nvim_get_curwin() -> *mut c_void;
    fn enter_cleanup(cs: *mut c_void);
    fn leave_cleanup(cs: *mut c_void);
    fn win_close(win: *mut c_void, free_buf: bool, force: bool) -> c_int;

    // --- do_exedit_split_fallback helpers ---
    fn do_cmdline_cmd(cmd: *const c_char) -> c_int;
    fn nvim_win_get_arg_idx_invalid(wp: *mut c_void) -> c_int;
    fn nvim_al_check_arg_idx(wp: *mut c_void);
    fn maketitle();

    // --- ex_read helpers ---
    fn nvim_buf_get_ml_empty(buf: BufHandle) -> bool;
    fn nvim_docmd_do_bang_read(eap: ExArgHandle);
    fn u_save(top: LinenrT, bot: LinenrT) -> c_int;
    fn check_fname() -> c_int;
    fn readfile(
        fname: *const c_char,
        sfname: *const c_char,
        from: LinenrT,
        lines_to_skip: LinenrT,
        lines_to_read: LinenrT,
        eap: ExArgHandle,
        flags: c_int,
        silent: bool,
    ) -> c_int;
    fn nvim_docmd_curbuf_b_ffname() -> *const c_char;
    fn nvim_docmd_curbuf_b_fname() -> *const c_char;
    fn nvim_get_p_cpo() -> *const c_char;
    fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    fn setaltfname(ffname: *const c_char, fname: *const c_char, lnum: LinenrT);
    fn aborting() -> bool;
    fn nvim_docmd_e_notopen_str() -> *const c_char;
    fn nvim_buf_get_line_count(buf: BufHandle) -> LinenrT;
    fn ml_get(lnum: LinenrT) -> *const c_char;
    fn u_savedel(lnum: LinenrT, nlines: LinenrT) -> c_int;
    fn ml_delete(lnum: LinenrT) -> c_int;
    fn nvim_docmd_curwin_cursor_lnum_maybe_dec(lnum: LinenrT);
    fn deleted_lines_mark(lnum: LinenrT, count: LinenrT);
    fn redraw_curbuf_later(type_: c_int);

    // --- ex_terminal helpers ---
    // nvim_docmd_add_win_cmd_modifiers_global: now implemented in Rust below
    fn add_win_cmd_modifiers(buf: *mut c_char, cmod: *mut c_void, multi_mods: *mut bool) -> usize;
    fn nvim_docmd_p_sh_is_empty() -> c_int;
    fn nvim_docmd_e_shellempty_str() -> *const c_char;
    fn nvim_docmd_terminal_get_shell_argv_str(buf: *mut c_char, buflen: usize);
    fn vim_strsave_escaped(s: *const c_char, esc: *const c_char) -> *mut c_char;
    fn xfree(p: *mut c_void);
    fn nvim_docmd_snprintf_terminal_suffix(
        buf: *mut c_char,
        buflen: usize,
        name: *const c_char,
    ) -> c_int;
    fn nvim_docmd_snprintf_terminal_shell(
        buf: *mut c_char,
        buflen: usize,
        shell_argv: *const c_char,
    ) -> c_int;

    // --- ex_restart helpers ---
    fn nvim_docmd_restart_patch_argv(arg: *const c_char);
    static mut restarting: bool;
    // nvim_docmd_run_quit_cmd: now implemented in Rust below
    fn concat_str(a: *const c_char, b: *const c_char) -> *mut c_char;
    fn nvim_docmd_get_cmod_confirm_prefix() -> *const c_char;
    // nvim_command for ex_quit_impl
    fn nvim_command(cmd: NvimString, err: *mut CError);

    // --- ex_detach helpers ---
    fn nvim_docmd_get_current_ui() -> u64;
    // find_channel is inline in C; use the accessor that finds + sets detach + returns chan->id (-1 on fail).
    fn nvim_channel_find_and_set_detach(id: u64) -> c_int;
    fn remote_ui_disconnect(channel_id: u64, err: *mut CError, send_error_exit: bool);
    fn channel_close(id: u64, part: c_int, error: *mut *const c_char) -> bool;
    fn api_clear_error(err: *mut CError);

    // --- ex_connect helpers ---
    fn nvim_docmd_ui_active_count() -> c_int;
    fn remote_ui_connect(channel_id: u64, server_addr: *mut c_char, err: *mut CError);
    fn getout(exitval: c_int);

    // --- ex_checkhealth helpers ---
    fn nvim_docmd_checkhealth_exec_lua(
        mods: *const c_char,
        mlen: usize,
        arg: *const c_char,
        err_msg_out: *mut *mut c_char,
    ) -> c_int;
    fn nvim_docmd_get_vimruntime() -> *const c_char;
    fn nvim_docmd_get_p_rtp() -> *const c_char;
    fn nvim_docmd_semsg_multiline_emsg(msg: *const c_char);
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *mut c_char;

    // --- expand_argopt helpers ---
    fn nvim_docmd_get_bad_name(xp: *mut c_void, idx: c_int) -> *mut c_char;
    fn get_fileformat_name(xp: *mut c_void, idx: c_int) -> *mut c_char;
    fn get_encoding_name(xp: *mut c_void, idx: c_int) -> *mut c_char;
    fn ExpandGeneric(
        pat: *const c_char,
        xp: *mut c_void,
        rmp: *mut c_void,
        matches: *mut *mut *mut c_char,
        num_matches: *mut c_int,
        cb: Option<unsafe extern "C" fn(*mut c_void, c_int) -> *mut c_char>,
        escaped: bool,
    );
    fn nvim_docmd_get_argopt_name(idx: c_int) -> *mut c_char;
    fn xmalloc(size: usize) -> *mut c_void;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    // expand_T field accessors (to avoid pulling in cmdexpand crate)
    fn nvim_xp_get_pattern(xp: *mut c_void) -> *mut c_char;
    fn nvim_xp_get_line(xp: *mut c_void) -> *mut c_char;
    fn nvim_xp_get_pattern_len(xp: *mut c_void) -> usize;

    // --- did_set_findfunc helpers ---
    fn nvim_optset_get_buf(args: *const c_void) -> BufHandle;
    fn nvim_optset_get_flags(args: *const c_void) -> c_int;
    fn nvim_docmd_findfunc_set_global() -> c_int;
    fn nvim_docmd_findfunc_set_local(buf: BufHandle) -> c_int;
    fn nvim_docmd_findfunc_free_local_cb(buf: BufHandle);
    fn nvim_docmd_optset_varp_deref(args: *mut c_void) -> *mut c_char;
    fn nvim_docmd_optset_varp_set(args: *mut c_void, name: *mut c_char);
    fn get_scriptlocal_funcname(name: *const c_char) -> *mut c_char;
    fn free_string_option(p: *mut c_char);
}

// ============================================================================
// nvim_docmd_add_win_cmd_modifiers_global (migrated from C to Rust)
// ============================================================================

/// Build the window command modifiers string into `buf` from the global cmdmod.
///
/// Previously `nvim_docmd_add_win_cmd_modifiers_global` in ex_docmd.c.
/// Calls `add_win_cmd_modifiers` (Rust, from usercmd crate).
///
/// # Safety
/// `buf` must have at least `bufsize` bytes.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_add_win_cmd_modifiers_global(
    buf: *mut c_char,
    bufsize: usize,
) -> usize {
    let mut multi_mods = false;
    *buf = 0; // buf[0] = NUL
    let len = add_win_cmd_modifiers(
        buf,
        std::ptr::addr_of_mut!(crate::cmdmod).cast::<c_void>(),
        &raw mut multi_mods,
    );
    debug_assert!(len < bufsize);
    len
}

// ============================================================================
// nvim_docmd_run_quit_cmd (migrated from C to Rust)
// ============================================================================

/// Execute a quit command via the nvim_command API.
///
/// Returns 1 on success, 0 on error (and emits the error message).
/// Previously `nvim_docmd_run_quit_cmd` in ex_docmd.c.
///
/// # Safety
/// Calls C API functions. `cmd` must be a valid null-terminated string.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_run_quit_cmd(cmd: *const c_char) -> c_int {
    let len = libc_strlen(cmd);
    let mut err = CError::init();
    nvim_command(
        NvimString {
            data: cmd as *mut c_char,
            size: len,
        },
        &raw mut err,
    );
    if err.is_set() {
        emsg(err.msg);
        api_clear_error(&raw mut err);
        return 0;
    }
    1
}

/// Compute strlen of a C string (used in nvim_docmd_run_quit_cmd).
unsafe fn libc_strlen(s: *const c_char) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    p.offset_from(s) as usize
}

// ============================================================================
// Rust replacement for nvim_docmd_check_more_dialog (Phase 5)
// ============================================================================

/// VIM_QUESTION = 4 (from message.h)
const VIM_QUESTION: c_int = 4;

/// Build the "more files" dialog message and call vim_dialog_yesno.
/// Replaces C `nvim_docmd_check_more_dialog`.
///
/// ngettext selects the translated format string; we substitute `n` for `%d` in Rust.
unsafe fn rs_nvim_docmd_check_more_dialog(n: c_int) -> c_int {
    // ngettext picks the singular or plural translated format string (with "%d").
    let fmt_ptr = ngettext(
        c"%d more file to edit.  Quit anyway?".as_ptr(),
        c"%d more files to edit.  Quit anyway?".as_ptr(),
        n as std::ffi::c_ulong,
    );
    // Convert the C format string to a Rust str and replace "%d" with the number.
    let fmt_cstr = std::ffi::CStr::from_ptr(fmt_ptr);
    let fmt_str = fmt_cstr.to_string_lossy();
    let msg = format!("{}\0", fmt_str.replacen("%d", &n.to_string(), 1));
    let c_msg = msg.as_ptr() as *const c_char;
    vim_dialog_yesno(VIM_QUESTION, std::ptr::null(), c_msg, 1)
}

// ============================================================================
// nvim_docmd_check_more
// ============================================================================

/// Check if more files remain in the argument list.
///
/// Returns OK (1) if no more files or forceit is set, FAIL (0) otherwise.
///
/// # Safety
/// Accesses global state (ARGCOUNT, curwin, quitmore, cmdmod).
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_check_more(message: c_int, forceit: c_int) -> c_int {
    let n = nvim_get_argcount() - nvim_win_get_arg_idx(nvim_get_curwin()) - 1;

    if forceit == 0
        && rs_only_one_window() != 0
        && nvim_get_argcount() > 1
        && nvim_al_get_arg_had_last() == 0
        && n > 0
        && crate::state::nvim_docmd_get_quitmore() == 0
    {
        if message != 0 {
            if nvim_get_p_confirm() != 0 || nvim_get_cmdmod_confirm() != 0 {
                let response = rs_nvim_docmd_check_more_dialog(n);
                if response == VIM_YES {
                    return OK;
                }
                return FAIL;
            }
            nvim_docmd_check_more_semsg(n);
            crate::state::nvim_docmd_set_quitmore(2); // next try to quit is allowed
        }
        return FAIL;
    }
    OK
}

// ============================================================================
// nvim_docmd_tabpage_new_impl
// ============================================================================

/// Open a new tab page.
///
/// # Safety
/// Accesses global state.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_tabpage_new_impl() {
    nvim_docmd_tabpage_new_body();
}

// ============================================================================
// nvim_docmd_do_exedit_handle_exmode
// ============================================================================

/// Handle the exmode_active early-return branch of do_exedit_impl.
/// Returns 1 if the caller should return early, 0 otherwise.
///
/// # Safety
/// Accesses and modifies global state (exmode_active, ex_pressedreturn, etc.).
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_do_exedit_handle_exmode(eap: ExArgHandle) -> c_int {
    let cmd_visual = crate::cmd_idx::CMD_visual;
    let cmd_view = crate::cmd_idx::CMD_view;
    let cmdidx = (*eap).cmdidx;

    if crate::exmode_active && (cmdidx == cmd_visual || cmdidx == cmd_view) {
        exmode_active = false;
        crate::state::nvim_set_ex_pressedreturn(false);

        let arg = (*eap).arg;
        if arg.is_null() || *arg as u8 == 0 {
            if nvim_get_global_busy() {
                let nextcmd = (*eap).nextcmd;
                if !nextcmd.is_null() {
                    stuffReadbuff(nextcmd);
                    (*eap).nextcmd = ptr::null_mut();
                }
                let save_rd = RedrawingDisabled;
                RedrawingDisabled = 0;
                let save_nwr = no_wait_return;
                no_wait_return = 0;
                need_wait_return = false;
                let save_ms = msg_scroll;
                msg_scroll = 0;
                redraw_all_later(UPD_NOT_VALID);
                pending_exmode_active = true;
                normal_enter(false, true);
                pending_exmode_active = false;
                RedrawingDisabled = save_rd;
                no_wait_return = save_nwr;
                msg_scroll = save_ms;
            }
            return 1;
        }
    }
    0
}

// ============================================================================
// nvim_docmd_do_exedit_split_fail_cleanup
// ============================================================================

/// Close curwin with cleanup on do_ecmd failure (for split windows).
///
/// # Safety
/// Accesses and modifies global window state.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_do_exedit_split_fail_cleanup() {
    let curbuf = nvim_get_curbuf();
    let need_hide = curbufIsChanged() && nvim_buf_get_nwindows(nvim_get_curbuf()) <= 1;
    if !need_hide || buf_hide(curbuf) {
        // cleanup_T is an opaque C struct; 64 bytes is sufficient on all platforms.
        let mut cs = [0u8; 64];
        let cs_ptr = cs.as_mut_ptr() as *mut c_void;
        enter_cleanup(cs_ptr);
        let free_buf = !need_hide && !buf_hide(curbuf);
        win_close(nvim_get_curwin(), free_buf, false);
        leave_cleanup(cs_ptr);
    }
}

// ============================================================================
// nvim_docmd_do_exedit_split_fallback
// ============================================================================

/// Handle the split-with-no-arg fallback branch of do_exedit_impl.
///
/// # Safety
/// Accesses global window state.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_do_exedit_split_fallback(eap: ExArgHandle) {
    let do_ecmd_cmd = (*eap).do_ecmd_cmd;
    if !do_ecmd_cmd.is_null() {
        do_cmdline_cmd(do_ecmd_cmd);
    }
    let n = nvim_win_get_arg_idx_invalid(nvim_get_curwin());
    nvim_al_check_arg_idx(nvim_get_curwin());
    if n != nvim_win_get_arg_idx_invalid(nvim_get_curwin()) {
        maketitle();
    }
}

// ============================================================================
// nvim_docmd_ex_read_impl
// ============================================================================

/// `:read` implementation called by Rust ex_read.
///
/// # Safety
/// Accesses global buffer and window state.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_ex_read_impl(eap: ExArgHandle) {
    let empty = nvim_buf_get_ml_empty(nvim_get_curbuf()) as c_int;

    if (*eap).usefilter != 0 {
        // :r!cmd
        nvim_docmd_do_bang_read(eap);
        return;
    }

    let line2 = (*eap).line2;
    if u_save(line2, line2 + 1) == FAIL {
        return;
    }

    let arg = (*eap).arg;
    let i = if arg.is_null() || *arg as u8 == 0 {
        if check_fname() == FAIL {
            return;
        }
        readfile(
            nvim_docmd_curbuf_b_ffname(),
            nvim_docmd_curbuf_b_fname(),
            line2,
            0,
            MAXLNUM,
            eap,
            0,
            false,
        )
    } else {
        if !vim_strchr(nvim_get_p_cpo(), CPO_ALTREAD).is_null() {
            setaltfname(arg, arg, 1);
        }
        readfile(arg, ptr::null(), line2, 0, MAXLNUM, eap, 0, false)
    };

    if i != OK {
        if !aborting() {
            semsg(nvim_docmd_e_notopen_str(), arg);
        }
    } else {
        if empty != 0 && crate::exmode_active {
            // Delete the empty line that remains (ex behavior, not vi).
            let lnum = if line2 == 0 {
                nvim_buf_get_line_count(nvim_get_curbuf())
            } else {
                1
            };
            if *ml_get(lnum) as u8 == 0 && u_savedel(lnum, 1) == OK {
                ml_delete(lnum);
                nvim_docmd_curwin_cursor_lnum_maybe_dec(lnum);
                deleted_lines_mark(lnum, 1);
            }
        }
        redraw_curbuf_later(UPD_VALID);
    }
}

// ============================================================================
// nvim_docmd_ex_checkhealth_impl
// ============================================================================

/// `:checkhealth [plugins]` implementation called by Rust ex_checkhealth.
///
/// # Safety
/// Calls Lua executor and accesses global option state.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_ex_checkhealth_impl(eap: ExArgHandle) {
    // Build modifier string
    let mut mods = [0u8; 1024];
    let mods_ptr = mods.as_mut_ptr() as *mut c_char;
    let mut mods_len: usize = 0;

    let cmod_tab = crate::cmdmod.cmod_tab;
    let cmod_split = crate::cmdmod.cmod_split;
    if cmod_tab > 0 || cmod_split != 0 {
        mods_len = nvim_docmd_add_win_cmd_modifiers_global(mods_ptr, mods.len());
    }

    let arg = (*eap).arg;
    let mut err_msg: *mut c_char = ptr::null_mut();
    let ok = nvim_docmd_checkhealth_exec_lua(mods_ptr, mods_len, arg, &mut err_msg);
    if ok != 0 {
        return;
    }

    // Execution failed -- show error
    let vimruntime = nvim_docmd_get_vimruntime();
    if vimruntime.is_null() {
        emsg(c"E5009: $VIMRUNTIME is empty or unset".as_ptr());
    } else {
        let rtp = nvim_docmd_get_p_rtp();
        let rtp_ok = !rtp.is_null() && !strstr(rtp, vimruntime).is_null();
        if rtp_ok {
            semsg(c"E5009: Invalid $VIMRUNTIME: %s".as_ptr(), vimruntime);
        } else {
            emsg(c"E5009: Invalid 'runtimepath'".as_ptr());
        }
    }
    if !err_msg.is_null() {
        nvim_docmd_semsg_multiline_emsg(err_msg);
        xfree(err_msg as *mut c_void);
    }
}

// ============================================================================
// nvim_docmd_ex_terminal_impl
// ============================================================================

/// `:terminal` implementation called by Rust ex_terminal.
///
/// # Safety
/// Calls do_cmdline_cmd and accesses global shell option state.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_ex_terminal_impl(eap: ExArgHandle) {
    let mut ex_cmd = [0u8; 1024];
    let ex_cmd_ptr = ex_cmd.as_mut_ptr() as *mut c_char;

    let cmod_tab = crate::cmdmod.cmod_tab;
    let cmod_split = crate::cmdmod.cmod_split;

    let len: usize;
    if cmod_tab > 0 || cmod_split != 0 {
        *ex_cmd_ptr = 0;
        let mods_len = nvim_docmd_add_win_cmd_modifiers_global(ex_cmd_ptr, ex_cmd.len());
        // Append " new"
        let new_suffix = b" new\0";
        if mods_len + new_suffix.len() <= ex_cmd.len() {
            ptr::copy_nonoverlapping(
                new_suffix.as_ptr(),
                ex_cmd.as_mut_ptr().add(mods_len),
                new_suffix.len(),
            );
            len = mods_len + 4; // " new" without null
        } else {
            len = mods_len;
        }
    } else {
        let forceit = (*eap).forceit != 0;
        let base: &[u8] = if forceit { b"enew!\0" } else { b"enew\0" };
        ptr::copy_nonoverlapping(base.as_ptr(), ex_cmd.as_mut_ptr(), base.len());
        len = base.len() - 1;
    }

    let arg = (*eap).arg;
    if !arg.is_null() && *arg as u8 != 0 {
        // Run {cmd} in 'shell'.
        let name = vim_strsave_escaped(arg, c"\"\\".as_ptr());
        let written =
            nvim_docmd_snprintf_terminal_suffix(ex_cmd_ptr.add(len), ex_cmd.len() - len, name);
        xfree(name as *mut c_void);
        let _ = written;
    } else {
        // No {cmd}: run the job with tokenized 'shell'.
        if nvim_docmd_p_sh_is_empty() != 0 {
            emsg(nvim_docmd_e_shellempty_str());
            return;
        }

        let mut shell_argv = [0u8; 512];
        nvim_docmd_terminal_get_shell_argv_str(
            shell_argv.as_mut_ptr() as *mut c_char,
            shell_argv.len(),
        );
        // shell_argv starts with ",\"sh\",..."; skip the leading ','
        let shell_argv_start = shell_argv.as_ptr().add(1) as *const c_char;
        let written = nvim_docmd_snprintf_terminal_shell(
            ex_cmd_ptr.add(len),
            ex_cmd.len() - len,
            shell_argv_start,
        );
        let _ = written;
    }

    do_cmdline_cmd(ex_cmd_ptr);
}

// ============================================================================
// nvim_docmd_ex_restart_impl
// ============================================================================

/// `:restart` implementation called by Rust ex_restart.
///
/// # Safety
/// Modifies global restart/exit state.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_ex_restart_impl(eap: ExArgHandle) {
    // Patch v:argv to include "-c <arg>" when it restarts.
    let arg = (*eap).arg;
    if !arg.is_null() && *arg as u8 != 0 {
        nvim_docmd_restart_patch_argv(arg);
    }

    let do_ecmd_cmd = (*eap).do_ecmd_cmd;
    let quit_cmd_base: *const c_char = if !do_ecmd_cmd.is_null() {
        do_ecmd_cmd
    } else {
        c"qall".as_ptr()
    };

    // Prepend "confirm " if :confirm modifier is active
    let confirm_prefix = nvim_docmd_get_cmod_confirm_prefix();
    let quit_cmd_copy: *mut c_char;
    let quit_cmd: *const c_char;
    if !confirm_prefix.is_null() {
        quit_cmd_copy = concat_str(confirm_prefix, quit_cmd_base);
        quit_cmd = quit_cmd_copy;
    } else {
        quit_cmd_copy = ptr::null_mut();
        quit_cmd = quit_cmd_base;
    }

    restarting = true;
    let ok = nvim_docmd_run_quit_cmd(quit_cmd);

    if !quit_cmd_copy.is_null() {
        xfree(quit_cmd_copy as *mut c_void);
    }

    if ok == 0 {
        restarting = false;
        return;
    }

    if !crate::exiting {
        emsg(c"restart failed: +cmd did not quit the server".as_ptr());
        restarting = false;
    }
}

// ============================================================================
// nvim_docmd_ex_detach_impl
// ============================================================================

/// `:detach` implementation called by Rust ex_detach.
///
/// # Safety
/// Modifies global UI/channel state.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_ex_detach_impl(eap: ExArgHandle) {
    if !eap.is_null() && (*eap).forceit != 0 {
        emsg(c"bang (!) not supported yet".as_ptr());
        return;
    }

    let current_ui = nvim_docmd_get_current_ui();
    if current_ui == 0 {
        emsg(c"UI not attached".as_ptr());
        return;
    }

    // find_channel and set detach flag (returns -1 on failure, already emitted error)
    let chan_id = nvim_channel_find_and_set_detach(current_ui);
    if chan_id < 0 {
        return;
    }

    // Server-side UI detach
    let mut err = CError::init();
    remote_ui_disconnect(current_ui, &mut err, true);
    if err.is_set() {
        emsg(err.msg);
        api_clear_error(&mut err);
        return;
    }

    // Server-side channel close
    let mut close_err: *const c_char = std::ptr::null();
    // kChannelPartAll = 4
    let rv = channel_close(current_ui, 4, &mut close_err);
    if !rv && !close_err.is_null() {
        emsg(close_err);
    }
}

// ============================================================================
// nvim_docmd_ex_connect_impl
// ============================================================================

/// `:connect` implementation called by Rust ex_connect.
///
/// # Safety
/// Modifies global UI/channel state.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_ex_connect_impl(eap: ExArgHandle) {
    let forceit = (*eap).forceit != 0;
    let stop_server = forceit && nvim_docmd_ui_active_count() == 1;

    let current_ui = nvim_docmd_get_current_ui();
    let arg = (*eap).arg;

    let mut err = CError::init();
    remote_ui_connect(current_ui, arg, &mut err);
    if err.is_set() {
        emsg(err.msg);
        api_clear_error(&mut err);
        return;
    }

    nvim_docmd_ex_detach_impl(ptr::null_mut());

    if stop_server {
        crate::exiting = true;
        getout(0);
    }
}

// ============================================================================
// nvim_docmd_expand_argopt_impl
// ============================================================================

/// Command-line expansion for ++opt=name.
///
/// # Safety
/// Accesses expand_T fields and calls ExpandGeneric.
#[export_name = "expand_argopt"]
pub unsafe extern "C" fn nvim_docmd_expand_argopt_impl(
    pat: *const c_char,
    xp: *mut c_void,
    rmp: *mut c_void,
    matches: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) -> c_int {
    let xp_pattern = nvim_xp_get_pattern(xp);
    let xp_line = nvim_xp_get_line(xp);

    if !xp_pattern.is_null() && xp_pattern > xp_line && *xp_pattern.offset(-1) as u8 == b'=' {
        let name_end = xp_pattern.offset(-1);
        let off = name_end.offset_from(xp_line) as usize;

        // Match suffix of the text before '='
        #[allow(clippy::if_same_then_else)]
        let cb: Option<unsafe extern "C" fn(*mut c_void, c_int) -> *mut c_char> =
            if off >= 2 && ends_with_bytes(name_end, b"ff") {
                Some(get_fileformat_name)
            } else if off >= 10 && ends_with_bytes(name_end, b"fileformat") {
                Some(get_fileformat_name)
            } else if off >= 3 && ends_with_bytes(name_end, b"enc") {
                Some(get_encoding_name)
            } else if off >= 8 && ends_with_bytes(name_end, b"encoding") {
                Some(get_encoding_name)
            } else if off >= 3 && ends_with_bytes(name_end, b"bad") {
                Some(nvim_docmd_get_bad_name)
            } else {
                None
            };

        if let Some(cb_fn) = cb {
            ExpandGeneric(pat, xp, rmp, matches, num_matches, Some(cb_fn), false);
            return OK;
        }
        return FAIL;
    }

    // Special handling of "ff" as short form of "fileformat"
    let pat_len = nvim_xp_get_pattern_len(xp);
    if pat_len == 2 {
        let p = nvim_xp_get_pattern(xp);
        if !p.is_null() && *p as u8 == b'f' && *p.add(1) as u8 == b'f' {
            let s = xmalloc(std::mem::size_of::<*mut c_char>()) as *mut *mut c_char;
            *matches = s;
            *num_matches = 1;
            *s = xstrdup(c"fileformat=".as_ptr());
            return OK;
        }
    }

    // General argopt completion
    ExpandGeneric(
        pat,
        xp,
        rmp,
        matches,
        num_matches,
        Some(rs_get_argopt_name_callback),
        false,
    );
    OK
}

/// Callback wrapper for the Rust-exported get_argopt_name function.
///
/// # Safety
/// `xp` may be null (unused). `idx` is a non-negative index.
unsafe extern "C" fn rs_get_argopt_name_callback(xp: *mut c_void, idx: c_int) -> *mut c_char {
    let _ = xp;
    nvim_docmd_get_argopt_name(idx)
}

/// Check whether the `len` bytes ending at `ptr` (exclusive) match `expected`.
///
/// `ptr` points just past the last byte to check (i.e., to `=`).
/// `expected` is the pattern we look for immediately before `ptr`.
///
/// # Safety
/// `ptr` must be valid and have at least `expected.len()` bytes before it.
#[inline]
unsafe fn ends_with_bytes(ptr: *const c_char, expected: &[u8]) -> bool {
    let start = ptr.sub(expected.len());
    for (i, &b) in expected.iter().enumerate() {
        if *start.add(i) as u8 != b {
            return false;
        }
    }
    true
}

// ============================================================================
// nvim_docmd_did_set_findfunc_impl
// ============================================================================

/// Process the 'findfunc' option value.
/// Returns NULL on success and an error message string on failure.
///
/// # Safety
/// Accesses global option state.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_did_set_findfunc_impl(args: *mut c_void) -> *const c_char {
    let buf = nvim_optset_get_buf(args as *const c_void);
    let flags = nvim_optset_get_flags(args as *const c_void);

    let retval = if (flags & OPT_LOCAL) != 0 {
        nvim_docmd_findfunc_set_local(buf)
    } else {
        let r = nvim_docmd_findfunc_set_global();
        // When using :set (not :setglobal), free the local callback too.
        if (flags & OPT_GLOBAL) == 0 {
            nvim_docmd_findfunc_free_local_cb(buf);
        }
        r
    };

    if retval == FAIL {
        return crate::errors::E_INVARG_STR.as_ptr();
    }

    // Replace <SID>/s: prefix with the script identifier if needed.
    let varp = nvim_docmd_optset_varp_deref(args);
    let name = get_scriptlocal_funcname(varp);
    if !name.is_null() {
        free_string_option(varp);
        nvim_docmd_optset_varp_set(args, name);
    }

    ptr::null()
}
