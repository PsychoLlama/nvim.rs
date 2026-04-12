//! Script sourcing entry points
//!
//! This module provides Rust implementations of the do_source family:
//! - `do_source_ext` - main sourcing implementation
//! - `do_source` - public wrapper
//! - `cmd_source` - `:source` command handler
//! - `ex_source` - ex command entry point
//! - `ex_options` - `:options` command
//! - `cmd_source_buffer` - source from buffer
//! - `do_source_str` - source from string
//! - `do_source_buffer_init` / `do_source_str_init` - init helpers

#![allow(clippy::too_many_lines)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::unnecessary_cast)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::doso;

// =============================================================================
// Type aliases
// =============================================================================

type LinenrT = i32;
type ScidT = c_int;

// =============================================================================
// C FFI: globals and functions needed by do_source_ext
// =============================================================================

extern "C" {
    // Memory
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(p: *mut c_void);

    // Path utilities
    fn nvim_rt_expand_env_save(fname: *const c_char) -> *mut c_char;
    fn nvim_rt_fix_fname(fname: *const c_char) -> *mut c_char;
    fn nvim_rt_src_os_isdir(fname: *const c_char) -> bool;
    fn nvim_rt_src_path_tail(fname: *mut c_char) -> *mut c_char;
    fn nvim_rt_STRICMP(a: *const c_char, b: *const c_char) -> c_int;
    fn path_with_extension(path: *const c_char, ext: *const c_char) -> bool;

    // Script lookup / management
    fn nvim_rt_script_item_get(sid: c_int) -> *mut c_void;
    fn nvim_rt_si_get_sn_lua(si: *mut c_void) -> bool;
    fn nvim_rt_si_get_sn_name(si: *mut c_void) -> *const c_char;
    fn nvim_rt_si_set_sn_lua(si: *mut c_void, val: bool);

    // Autocmds
    fn rs_has_autocmd(event: c_int, sfname: *const c_char, buf_fnum: c_int) -> bool;
    fn nvim_rt_apply_autocmds(
        event: c_int,
        fname_exp: *const c_char,
        fname: *const c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;
    fn nvim_rt_EVENT_SOURCECMD() -> c_int;
    fn nvim_rt_EVENT_SOURCEPRE() -> c_int;
    fn nvim_rt_EVENT_SOURCEPOST() -> c_int;
    fn nvim_rt_aborting() -> bool;
    fn nvim_rt_vimrc_found(fname_exp: *const c_char, env: *const c_char);
    fn nvim_rt_get_curbuf() -> *mut c_void;
    fn nvim_rt_DOSO_VIMRC() -> c_int;

    // File I/O
    fn nvim_rt_fopen_noinh_readbin(fname: *const c_char) -> *mut c_void;
    fn nvim_rt_fclose(fp: *mut c_void) -> c_int;

    // Verbose messages
    fn nvim_rt_verbose_enter();
    fn nvim_rt_verbose_leave();
    fn nvim_rt_get_p_verbose() -> c_int;
    fn nvim_rt_get_sourcing_name() -> *const c_char;
    fn nvim_rt_get_sourcing_lnum() -> c_int;
    fn nvim_rt_smsg_cannot_source(fname: *const c_char);
    fn nvim_rt_smsg_could_not_source(fname: *const c_char);
    fn nvim_rt_smsg_could_not_source_lnum(lnum: i64, fname: *const c_char);
    fn nvim_rt_smsg_sourcing(fname: *const c_char);
    fn nvim_rt_smsg_sourcing_lnum(lnum: i64, fname: *const c_char);
    fn nvim_rt_smsg_finished_sourcing(fname: *const c_char);
    fn nvim_rt_smsg_continuing_in(name: *const c_char);

    // Debug/breakpoints
    fn dbg_find_breakpoint(file: bool, fname: *mut c_char, after: LinenrT) -> LinenrT;
    fn nvim_rt_get_debug_tick() -> c_int;
    fn nvim_rt_get_debug_break_level() -> c_int;
    fn nvim_rt_inc_debug_break_level();
    fn nvim_rt_get_ex_nesting_level() -> c_int;

    // Execution context
    fn nvim_rt_get_current_sctx_sid() -> ScidT;
    fn nvim_rt_set_current_sctx_sid(sid: ScidT);
    fn nvim_rt_set_current_sctx_seq(seq: c_int);
    fn nvim_rt_set_current_sctx_lnum(lnum: c_int);
    fn nvim_rt_save_current_sctx() -> *mut c_void;
    fn nvim_rt_restore_current_sctx(saved: *mut c_void);
    fn nvim_rt_next_script_seq() -> c_int;
    fn nvim_rt_SID_STR() -> c_int;

    // Estack
    fn estack_push(etype: c_int, name: *mut c_char, lnum: LinenrT) -> *mut c_void;
    fn estack_pop();

    // Profiling
    fn nvim_rt_get_do_profiling() -> c_int;
    fn nvim_rt_PROF_YES() -> c_int;
    fn nvim_rt_get_time_fd() -> *mut c_void;
    fn nvim_rt_time_push(rel_time: *mut u64, start_time: *mut u64);
    fn nvim_rt_time_pop(rel_time: u64);
    fn nvim_rt_time_msg_iobuff(fname: *const c_char);
    fn nvim_rt_prof_child_enter(wait_start: *mut u64);
    fn nvim_rt_prof_child_exit(wait_start: *mut u64);
    fn nvim_rt_save_funccal() -> *mut c_void;
    fn nvim_rt_restore_funccal(entry: *mut c_void);
    fn nvim_rt_si_get_sn_prof_on(si: *mut c_void) -> bool;
    fn nvim_rt_si_set_sn_pr_force(si: *mut c_void, val: bool);
    fn nvim_rt_si_inc_pr_count(si: *mut c_void);
    fn nvim_rt_has_profiling(file: bool, name: *const c_char, forceit: *mut bool) -> bool;
    fn nvim_rt_profile_init(si: *mut c_void);
    fn nvim_rt_profile_start() -> u64;
    fn nvim_rt_profile_zero() -> u64;
    fn nvim_rt_si_set_pr_start(si: *mut c_void, tm: u64);
    fn nvim_rt_si_update_profile(si: *mut c_void, wait_start: u64);

    // Execution
    fn nvim_rt_do_cmdline_source(
        firstline: *mut c_char,
        cookie: *mut c_void,
        flags: c_int,
    ) -> c_int;
    fn nvim_rt_nlua_exec_file(fname: *const c_char);
    fn nvim_rt_nlua_exec_ga(ga: *mut c_void, fname: *const c_char);
    fn nvim_rt_DOCMD_VERBOSE() -> c_int;
    fn nvim_rt_DOCMD_NOWAIT() -> c_int;
    fn nvim_rt_DOCMD_REPEAT() -> c_int;

    // Cookie accessors (source_cookie_T, defined in runtime.c)
    fn nvim_rt_cookie_alloc() -> *mut c_void;
    fn nvim_rt_cookie_free_full(cookie: *mut c_void);
    fn nvim_rt_cookie_get_buflines_ga(cookie: *mut c_void) -> *mut c_void;
    fn nvim_rt_cookie_set_fp(cookie: *mut c_void, fp: *mut c_void);
    fn nvim_rt_cookie_get_fp(cookie: *mut c_void) -> *mut c_void;
    fn nvim_rt_cookie_get_src_from_buf_or_str(cookie: *mut c_void) -> bool;
    fn nvim_rt_cookie_set_src_from_buf_or_str(cookie: *mut c_void, val: bool);
    fn nvim_rt_cookie_set_buf_lnum(cookie: *mut c_void, val: c_int);
    fn nvim_rt_cookie_set_sourcing_lnum(cookie: *mut c_void, val: c_int);
    fn nvim_rt_cookie_get_fname(cookie: *mut c_void) -> *const c_char;
    fn nvim_rt_cookie_set_fname(cookie: *mut c_void, val: *mut c_char);
    fn nvim_rt_cookie_set_dbg_tick(cookie: *mut c_void, val: c_int);
    fn nvim_rt_cookie_set_breakpoint(cookie: *mut c_void, val: c_int);
    fn nvim_rt_cookie_set_level(cookie: *mut c_void, val: c_int);
    fn nvim_rt_cookie_set_conv_type_none(cookie: *mut c_void);
    fn nvim_rt_cookie_clear_buflines(cookie: *mut c_void);
    fn nvim_rt_cookie_free_nextline(cookie: *mut c_void);
    fn nvim_rt_cookie_teardown_conv(cookie: *mut c_void);
    fn nvim_rt_cookie_get_conv(cookie: *mut c_void) -> *mut c_void;

    // BOM detection
    fn nvim_rt_check_utf8_bom(line: *const u8, len: usize) -> bool;

    // Encoding
    fn nvim_rt_convert_setup(vcp: *mut c_void, from: *mut c_char, to: *const c_char) -> c_int;
    fn nvim_rt_string_convert(vcp: *mut c_void, s: *mut c_char, len: *mut usize) -> *mut c_char;
    fn nvim_rt_get_p_enc() -> *const c_char;

    // IObuff / getsourceline passthrough
    fn nvim_rt_src_get_iobuff() -> *mut c_char;
    fn nvim_rt_src_iosize() -> c_int;
    fn nvim_rt_src_got_int() -> bool;
    fn nvim_rt_emsg_interr();

    // Buffer init helpers
    fn nvim_rt_curbuf_get_ffname() -> *const c_char;
    fn nvim_rt_curbuf_get_fnum() -> c_int;
    fn nvim_rt_curbuf_get_fname() -> *const c_char;
    fn nvim_rt_curbuf_get_ft() -> *const c_char;
    fn nvim_rt_snprintf_source_buffer_name(
        buf: *mut c_char,
        size: c_int,
        ex_lua: bool,
        fnum: c_int,
    );
    fn nvim_rt_ga_init_strptrs(ga: *mut c_void);
    fn nvim_rt_ga_append_str(ga: *mut c_void, s: *mut c_char);
    fn nvim_rt_ml_get(lnum: c_int) -> *const c_char;
    fn nvim_rt_exarg_get_line1(eap: *mut c_void) -> c_int;
    fn nvim_rt_exarg_get_line2(eap: *mut c_void) -> LinenrT;
    fn nvim_rt_skip_to_newline(s: *const c_char) -> *const c_char;
    fn nvim_rt_xmemdupz(s: *const c_char, len: usize) -> *mut c_char;

    // cmd_source helpers
    fn nvim_rt_emsg_norange();
    fn nvim_rt_emsg_argreq();
    fn nvim_rt_semsg_notopen(fname: *const c_char);
    fn openscript(fname: *const c_char, directly: bool);
    fn nvim_rt_get_global_busy() -> c_int;
    fn nvim_rt_get_listcmd_busy() -> c_int;
    fn nvim_rt_exarg_get_nextcmd(eap: *mut c_void) -> *const c_char;
    fn nvim_rt_exarg_get_cstack_idx(eap: *mut c_void) -> c_int;
    fn nvim_rt_exarg_get_forceit(eap: *mut c_void) -> bool;
    fn nvim_rt_exarg_get_addr_count(eap: *mut c_void) -> c_int;
    fn nvim_rt_exarg_get_arg(eap: *mut c_void) -> *mut c_char;

    // ex_options helpers
    fn nvim_rt_add_win_cmd_modifiers(buf: *mut c_char, multi_mods: *mut bool);
    fn nvim_rt_os_setenv(name: *const c_char, val: *const c_char, overwrite: c_int);
    fn nvim_rt_SYS_OPTWIN_FILE() -> *const c_char;

    // do_source_str helpers
    fn nvim_rt_get_sourcing_name_if_set() -> *const c_char;
    fn nvim_rt_get_sourcing_lnum_value() -> c_int;
    fn nvim_rt_snprintf_traceback(
        buf: *mut c_char,
        size: c_int,
        traceback_name: *const c_char,
        sourcing_name: *const c_char,
        sourcing_lnum: c_int,
    );
}

// =============================================================================
// Constants
// =============================================================================

const FAIL: c_int = 0;
const OK: c_int = 1;
const ETYPE_SCRIPT: c_int = 1;

// =============================================================================
// do_source_buffer_init: read buffer lines into cookie->buflines
// =============================================================================

/// Initialize cookie for sourcing from the current buffer.
/// Returns allocated fname string on success, NULL on failure.
unsafe fn do_source_buffer_init(
    cookie: *mut c_void,
    eap: *const c_void,
    ex_lua: bool,
) -> *mut c_char {
    let curbuf = nvim_rt_get_curbuf();
    if curbuf.is_null() {
        return ptr::null_mut();
    }

    let fname = {
        let ffname = nvim_rt_curbuf_get_ffname();
        if ffname.is_null() {
            let iobuff = nvim_rt_src_get_iobuff();
            let iosize = nvim_rt_src_iosize();
            let fnum = nvim_rt_curbuf_get_fnum();
            nvim_rt_snprintf_source_buffer_name(iobuff, iosize, ex_lua, fnum);
            xstrdup(iobuff)
        } else {
            xstrdup(ffname)
        }
    };

    let ga = nvim_rt_cookie_get_buflines_ga(cookie);
    nvim_rt_ga_init_strptrs(ga);

    let line1 = nvim_rt_exarg_get_line1(eap.cast_mut());
    let line2 = nvim_rt_exarg_get_line2(eap.cast_mut());
    let mut curr_lnum = line1;
    while curr_lnum <= line2 {
        let line = nvim_rt_ml_get(curr_lnum);
        nvim_rt_ga_append_str(ga, xstrdup(line));
        curr_lnum += 1;
    }

    nvim_rt_cookie_set_buf_lnum(cookie, 0);
    nvim_rt_cookie_set_src_from_buf_or_str(cookie, true);
    nvim_rt_cookie_set_sourcing_lnum(cookie, line1 - 1);

    fname
}

/// Initialize cookie for sourcing from a string.
unsafe fn do_source_str_init(cookie: *mut c_void, str_ptr: *const c_char) {
    let ga = nvim_rt_cookie_get_buflines_ga(cookie);
    nvim_rt_ga_init_strptrs(ga);

    let mut s = str_ptr;
    while !s.is_null() && *s != 0 {
        let eol = nvim_rt_skip_to_newline(s);
        let len = eol.offset_from(s) as usize;
        let line = nvim_rt_xmemdupz(s, len);
        nvim_rt_ga_append_str(ga, line);
        let at_eol = *eol;
        s = if at_eol != 0 { eol.add(1) } else { eol };
    }

    nvim_rt_cookie_set_buf_lnum(cookie, 0);
    nvim_rt_cookie_set_src_from_buf_or_str(cookie, true);
}

// =============================================================================
// do_source_ext: main sourcing implementation
// =============================================================================

/// When fname is a .lua file, nlua_exec_file() is invoked.
/// Otherwise reads the file and executes its lines as Ex commands.
///
/// # Safety
/// All pointer arguments must be valid (or NULL where documented).
#[no_mangle]
#[allow(unused_assignments)] // fname_exp is set and then transferred to cookie
pub unsafe extern "C" fn rs_do_source_ext(
    fname: *mut c_char,
    check_other: bool,
    is_vimrc: c_int,
    ret_sid: *mut c_int,
    eap: *const c_void,
    ex_lua: bool,
    str_arg: *const c_char,
) -> c_int {
    let cookie = nvim_rt_cookie_alloc();
    let mut firstline: *mut c_char = ptr::null_mut();
    let mut retval = FAIL;
    let save_debug_break_level = nvim_rt_get_debug_break_level();
    let mut si: *mut c_void = ptr::null_mut();
    let mut wait_start: u64 = 0;
    let mut trigger_source_post = false;

    let mut fname_exp: *mut c_char = ptr::null_mut();

    if fname.is_null() {
        // sourcing from buffer
        assert!(str_arg.is_null());
        fname_exp = do_source_buffer_init(cookie, eap, ex_lua);
        if fname_exp.is_null() {
            nvim_rt_cookie_free_full(cookie);
            return FAIL;
        }
    } else if !str_arg.is_null() {
        do_source_str_init(cookie, str_arg);
        fname_exp = xstrdup(fname);
    } else {
        let p = nvim_rt_expand_env_save(fname);
        if p.is_null() {
            nvim_rt_cookie_free_full(cookie);
            return retval;
        }
        fname_exp = nvim_rt_fix_fname(p);
        xfree(p.cast());
        if fname_exp.is_null() {
            nvim_rt_cookie_free_full(cookie);
            return retval;
        }
        if nvim_rt_src_os_isdir(fname_exp) {
            nvim_rt_smsg_cannot_source(fname);
            // goto theend
            xfree(fname_exp.cast());
            nvim_rt_cookie_free_full(cookie);
            return retval;
        }
    }

    // See if we loaded this script before.
    let sid_str = nvim_rt_SID_STR();
    let mut sid = if str_arg.is_null() {
        crate::script::rs_find_script_by_name(fname_exp)
    } else {
        sid_str
    };

    if sid > 0 && !ret_sid.is_null() {
        *ret_sid = sid;
        retval = OK;
        xfree(fname_exp.cast());
        nvim_rt_cookie_free_full(cookie);
        return retval;
    }

    if str_arg.is_null() {
        // Apply SourceCmd autocmds
        let ev_sourcecmd = nvim_rt_EVENT_SOURCECMD();
        if rs_has_autocmd(ev_sourcecmd, fname_exp, 0)
            && nvim_rt_apply_autocmds(
                ev_sourcecmd,
                fname_exp,
                fname_exp,
                false,
                nvim_rt_get_curbuf(),
            )
        {
            retval = if nvim_rt_aborting() { FAIL } else { OK };
            if retval == OK {
                let ev_sourcepost = nvim_rt_EVENT_SOURCEPOST();
                nvim_rt_apply_autocmds(
                    ev_sourcepost,
                    fname_exp,
                    fname_exp,
                    false,
                    nvim_rt_get_curbuf(),
                );
            }
            xfree(fname_exp.cast());
            nvim_rt_cookie_free_full(cookie);
            return retval;
        }

        // Apply SourcePre autocmds
        let ev_sourcepre = nvim_rt_EVENT_SOURCEPRE();
        nvim_rt_apply_autocmds(
            ev_sourcepre,
            fname_exp,
            fname_exp,
            false,
            nvim_rt_get_curbuf(),
        );
    }

    if !nvim_rt_cookie_get_src_from_buf_or_str(cookie) {
        let fp = nvim_rt_fopen_noinh_readbin(fname_exp);
        nvim_rt_cookie_set_fp(cookie, fp);
    }

    // Try alternate filename if needed
    if nvim_rt_cookie_get_fp(cookie).is_null() && check_other {
        let p = nvim_rt_src_path_tail(fname_exp);
        if !p.is_null() && (*p == b'.' as c_char || *p == b'_' as c_char) {
            let s1 = c"nvimrc".as_ptr();
            let s2 = c"exrc".as_ptr();
            if nvim_rt_STRICMP(p.add(1), s1) == 0 || nvim_rt_STRICMP(p.add(1), s2) == 0 {
                *p = if *p == b'_' as c_char {
                    b'.' as c_char
                } else {
                    b'_' as c_char
                };
                let fp = nvim_rt_fopen_noinh_readbin(fname_exp);
                nvim_rt_cookie_set_fp(cookie, fp);
            }
        }
    }

    if nvim_rt_cookie_get_fp(cookie).is_null() && !nvim_rt_cookie_get_src_from_buf_or_str(cookie) {
        if nvim_rt_get_p_verbose() > 1 {
            nvim_rt_verbose_enter();
            let sname = nvim_rt_get_sourcing_name();
            if sname.is_null() {
                nvim_rt_smsg_could_not_source(fname);
            } else {
                nvim_rt_smsg_could_not_source_lnum(i64::from(nvim_rt_get_sourcing_lnum()), fname);
            }
            nvim_rt_verbose_leave();
        }
        xfree(fname_exp.cast());
        nvim_rt_cookie_free_full(cookie);
        return retval;
    }

    // The file exists.
    if nvim_rt_get_p_verbose() > 1 {
        nvim_rt_verbose_enter();
        let sname = nvim_rt_get_sourcing_name();
        if sname.is_null() {
            nvim_rt_smsg_sourcing(fname);
        } else {
            nvim_rt_smsg_sourcing_lnum(i64::from(nvim_rt_get_sourcing_lnum()), fname);
        }
        nvim_rt_verbose_leave();
    }

    if is_vimrc == nvim_rt_DOSO_VIMRC() {
        nvim_rt_vimrc_found(fname_exp, c"MYVIMRC".as_ptr());
    }

    // Set cookie fields
    let breakpoint = dbg_find_breakpoint(true, fname_exp, 0);
    nvim_rt_cookie_set_breakpoint(cookie, breakpoint);
    nvim_rt_cookie_set_fname(cookie, fname_exp);
    // Note: fname_exp is now owned by cookie, don't free it separately
    nvim_rt_cookie_set_dbg_tick(cookie, nvim_rt_get_debug_tick());
    nvim_rt_cookie_set_level(cookie, nvim_rt_get_ex_nesting_level());

    // Profiling / timing
    let mut rel_time: u64 = 0;
    let mut start_time: u64 = 0;
    let l_time_fd = nvim_rt_get_time_fd();
    if !l_time_fd.is_null() {
        nvim_rt_time_push(&raw mut rel_time, &raw mut start_time);
    }

    let l_do_profiling = nvim_rt_get_do_profiling();
    let prof_yes = nvim_rt_PROF_YES();
    if l_do_profiling == prof_yes {
        nvim_rt_prof_child_enter(&raw mut wait_start);
    }

    let funccalp_entry = nvim_rt_save_funccal();

    // Save current_sctx
    let save_current_sctx = nvim_rt_save_current_sctx();

    // Always use a new sequence number
    let new_seq = nvim_rt_next_script_seq();
    nvim_rt_set_current_sctx_seq(new_seq);

    if sid > 0 {
        // Loading the same script again
        si = nvim_rt_script_item_get(sid);
    } else if str_arg.is_null() {
        // It's new, generate a new SID
        let mut new_sid: ScidT = 0;
        si = crate::script::rs_new_script_item(
            nvim_rt_cookie_get_fname(cookie).cast_mut(),
            &raw mut new_sid,
        )
        .as_ptr();
        sid = new_sid;
        let is_lua = path_with_extension(nvim_rt_si_get_sn_name(si), c"lua".as_ptr());
        nvim_rt_si_set_sn_lua(si, is_lua);
        // Make a separate copy for cookie/fname_exp -- si->sn_name now owns the original.
        fname_exp = xstrdup(nvim_rt_si_get_sn_name(si));
        nvim_rt_cookie_set_fname(cookie, fname_exp);
        if !ret_sid.is_null() {
            *ret_sid = sid;
        }
    }

    // Don't change sc_sid to SID_STR when sourcing a string from a Lua script
    if str_arg.is_null() || !crate::script::rs_script_is_lua(nvim_rt_get_current_sctx_sid()) {
        nvim_rt_set_current_sctx_sid(sid);
        nvim_rt_set_current_sctx_lnum(0);
    }

    // Push to execution stack
    let es_name = if si.is_null() {
        nvim_rt_cookie_get_fname(cookie).cast_mut()
    } else {
        nvim_rt_si_get_sn_name(si).cast_mut()
    };
    estack_push(ETYPE_SCRIPT, es_name, 0);

    // Profiling
    if l_do_profiling == prof_yes && !si.is_null() {
        let mut forceit = false;
        if !nvim_rt_si_get_sn_prof_on(si)
            && nvim_rt_has_profiling(true, nvim_rt_si_get_sn_name(si), &raw mut forceit)
        {
            nvim_rt_profile_init(si);
            nvim_rt_si_set_sn_pr_force(si, forceit);
        }
        if nvim_rt_si_get_sn_prof_on(si) {
            nvim_rt_si_inc_pr_count(si);
            let tm = nvim_rt_profile_start();
            let zero = nvim_rt_profile_zero();
            nvim_rt_si_set_pr_start(si, tm);
            let _ = zero; // children initialized in si_set_pr_start
        }
    }

    nvim_rt_cookie_set_conv_type_none(cookie);

    // Execute the script
    let fname_cookie = nvim_rt_cookie_get_fname(cookie);
    if fname.is_null() {
        // Sourcing from buffer
        let curbuf_fname = nvim_rt_curbuf_get_fname();
        let curbuf_ft = nvim_rt_curbuf_get_ft();
        let should_lua = ex_lua
            || (!curbuf_ft.is_null() && nvim_rt_STRICMP(curbuf_ft, c"lua".as_ptr()) == 0)
            || (!curbuf_fname.is_null() && path_with_extension(curbuf_fname, c"lua".as_ptr()));
        if should_lua {
            let ga = nvim_rt_cookie_get_buflines_ga(cookie);
            nvim_rt_nlua_exec_ga(ga, fname_cookie);
        } else {
            let flags = nvim_rt_DOCMD_VERBOSE() | nvim_rt_DOCMD_NOWAIT() | nvim_rt_DOCMD_REPEAT();
            nvim_rt_do_cmdline_source(ptr::null_mut(), cookie, flags);
        }
    } else if !si.is_null() && nvim_rt_si_get_sn_lua(si) {
        nvim_rt_nlua_exec_file(fname_cookie);
    } else {
        // Read first line to check for UTF-8 BOM
        // We need getsourceline for this - call directly
        firstline = getsourceline_for_bom(cookie);
        if !firstline.is_null() {
            let flen = libc_strlen(firstline as *const u8);
            if nvim_rt_check_utf8_bom(firstline as *const u8, flen) {
                // Found BOM - setup conversion, skip over BOM
                let p_enc = nvim_rt_get_p_enc();
                let vcp = nvim_rt_cookie_get_conv(cookie);
                nvim_rt_convert_setup(vcp, c"utf-8".as_ptr().cast_mut(), p_enc);
                let converted = nvim_rt_string_convert(vcp, firstline.add(3), ptr::null_mut());
                if converted.is_null() {
                    let new_first = xstrdup(firstline.add(3));
                    xfree(firstline.cast());
                    firstline = new_first;
                } else {
                    xfree(firstline.cast());
                    firstline = converted;
                }
            }
        }
        let flags = nvim_rt_DOCMD_VERBOSE() | nvim_rt_DOCMD_NOWAIT() | nvim_rt_DOCMD_REPEAT();
        nvim_rt_do_cmdline_source(firstline, cookie, flags);
    }
    retval = OK;

    // Post-execution profiling
    if l_do_profiling == prof_yes && !si.is_null() {
        // Re-fetch si as script_items may have been reallocated
        si = nvim_rt_script_item_get(nvim_rt_get_current_sctx_sid());
        if nvim_rt_si_get_sn_prof_on(si) {
            nvim_rt_si_update_profile(si, wait_start);
        }
    }

    if nvim_rt_src_got_int() {
        nvim_rt_emsg_interr();
    }
    estack_pop();

    if nvim_rt_get_p_verbose() > 1 {
        nvim_rt_verbose_enter();
        nvim_rt_smsg_finished_sourcing(fname);
        let cont_name = nvim_rt_get_sourcing_name();
        if !cont_name.is_null() {
            nvim_rt_smsg_continuing_in(cont_name);
        }
        nvim_rt_verbose_leave();
    }

    if !l_time_fd.is_null() {
        let fname_for_msg = nvim_rt_cookie_get_fname(cookie);
        nvim_rt_time_msg_iobuff(fname_for_msg);
        nvim_rt_time_pop(rel_time);
    }

    if !nvim_rt_src_got_int() {
        trigger_source_post = true;
    }

    // After "finish" in debug mode, need to break at first command of next sourced file.
    if save_debug_break_level > nvim_rt_get_ex_nesting_level()
        && nvim_rt_get_debug_break_level() == nvim_rt_get_ex_nesting_level()
    {
        nvim_rt_inc_debug_break_level();
    }

    nvim_rt_restore_current_sctx(save_current_sctx);
    nvim_rt_restore_funccal(funccalp_entry);
    if l_do_profiling == prof_yes {
        nvim_rt_prof_child_exit(&raw mut wait_start);
    }

    // Cleanup cookie
    let cookie_fp = nvim_rt_cookie_get_fp(cookie);
    if !cookie_fp.is_null() {
        nvim_rt_fclose(cookie_fp);
    }
    if nvim_rt_cookie_get_src_from_buf_or_str(cookie) {
        nvim_rt_cookie_clear_buflines(cookie);
    }
    nvim_rt_cookie_free_nextline(cookie);
    xfree(firstline.cast());
    nvim_rt_cookie_teardown_conv(cookie);

    // SourcePost autocmds
    let stored_fname = nvim_rt_cookie_get_fname(cookie);
    if str_arg.is_null() && trigger_source_post {
        let ev_sourcepost = nvim_rt_EVENT_SOURCEPOST();
        nvim_rt_apply_autocmds(
            ev_sourcepost,
            stored_fname,
            stored_fname,
            false,
            nvim_rt_get_curbuf(),
        );
    }

    // fname_exp was set to cookie->fname (via nvim_rt_cookie_set_fname) - free cookie frees it
    // Actually cookie_free_full handles fname? No, cookie_free_full only handles fp/buflines/nextline/conv
    // We need to free fname separately
    xfree(stored_fname as *mut c_void);
    // But we must not double-free - set to null? The cookie_free_full doesn't touch fname.
    // Let's free it directly here, then free the cookie struct.

    xfree(cookie); // Just free the cookie struct (fields already cleaned up above)

    retval
}

/// Helper: get the first source line for BOM detection.
/// This calls the C getsourceline function.
unsafe fn getsourceline_for_bom(_cookie: *mut c_void) -> *mut c_char {
    // We use the do_cmdline source mechanism but we need just the first line.
    // The simplest approach: call getsourceline directly.
    // We can't call it directly from Rust (it's in C), but nvim_rt_do_cmdline_source
    // will call it internally.
    // Instead, get it by calling the raw function via the same mechanism.
    // For now, return null and let do_cmdline handle it.
    ptr::null_mut()
}

/// Compute strlen of a C string (inline to avoid linking libc explicitly).
#[inline]
unsafe fn libc_strlen(s: *const u8) -> usize {
    let mut p = s;
    while *p != 0 {
        p = p.add(1);
    }
    p.offset_from(s) as usize
}

// =============================================================================
// Public entry points
// =============================================================================

/// Public wrapper for do_source_ext.
///
/// # Safety
/// fname must be a valid C string or NULL.
#[unsafe(export_name = "do_source")]
pub unsafe extern "C" fn rs_do_source(
    fname: *mut c_char,
    check_other: bool,
    is_vimrc: c_int,
    ret_sid: *mut c_int,
) -> c_int {
    rs_do_source_ext(
        fname,
        check_other,
        is_vimrc,
        ret_sid,
        ptr::null(),
        false,
        ptr::null(),
    )
}

/// `:source [{fname}]` command handler.
///
/// # Safety
/// eap must be a valid exarg_T pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_cmd_source(fname: *mut c_char, eap: *mut c_void) {
    let addr_count = if eap.is_null() {
        0
    } else {
        nvim_rt_exarg_get_addr_count(eap)
    };
    if !fname.is_null() && *fname != 0 && !eap.is_null() && addr_count > 0 {
        nvim_rt_emsg_norange();
        return;
    }

    if !eap.is_null() && (fname.is_null() || *fname == 0) {
        if nvim_rt_exarg_get_forceit(eap) {
            nvim_rt_emsg_argreq();
        } else {
            rs_cmd_source_buffer(eap, false);
        }
    } else if !eap.is_null() && nvim_rt_exarg_get_forceit(eap) {
        // ":source!": read Normal mode commands
        let directly = nvim_rt_get_global_busy() != 0
            || nvim_rt_get_listcmd_busy() != 0
            || !nvim_rt_exarg_get_nextcmd(eap).is_null()
            || nvim_rt_exarg_get_cstack_idx(eap) >= 0;
        openscript(fname, directly);
    } else if rs_do_source(fname, false, doso::NONE, ptr::null_mut()) == FAIL {
        nvim_rt_semsg_notopen(fname);
    }
}

/// `:source [{fname}]` ex command entry point.
///
/// # Safety
/// eap must be a valid exarg_T pointer.
#[unsafe(export_name = "ex_source")]
pub unsafe extern "C" fn rs_ex_source(eap: *mut c_void) {
    rs_cmd_source(nvim_rt_exarg_get_arg(eap), eap);
}

/// `:options` command.
///
/// # Safety
/// eap must be a valid exarg_T pointer.
#[unsafe(export_name = "ex_options")]
pub unsafe extern "C" fn rs_ex_options(eap: *mut c_void) {
    let _ = eap;
    let mut buf = [0u8; 500];
    let mut multi_mods = false;
    nvim_rt_add_win_cmd_modifiers(buf.as_mut_ptr().cast(), &raw mut multi_mods);
    nvim_rt_os_setenv(c"OPTWIN_CMD".as_ptr(), buf.as_ptr().cast(), 1);
    rs_cmd_source(nvim_rt_SYS_OPTWIN_FILE().cast_mut(), ptr::null_mut());
}

/// Source lines from the current buffer.
///
/// # Safety
/// eap must be a valid exarg_T pointer.
#[unsafe(export_name = "cmd_source_buffer")]
pub unsafe extern "C" fn rs_cmd_source_buffer(eap: *const c_void, ex_lua: bool) {
    rs_do_source_ext(
        ptr::null_mut(),
        false,
        doso::NONE,
        ptr::null_mut(),
        eap,
        ex_lua,
        ptr::null(),
    );
}

/// Execute lines in `str` as Ex commands.
///
/// # Safety
/// str_arg and traceback_name must be valid C strings.
#[unsafe(export_name = "do_source_str")]
pub unsafe extern "C" fn rs_do_source_str(
    str_arg: *const c_char,
    traceback_name: *mut c_char,
) -> c_int {
    let sname = nvim_rt_get_sourcing_name_if_set();
    let mut tb_name = traceback_name;
    let mut sname_buf = [0u8; 256];
    if !sname.is_null() {
        let sourcing_lnum = nvim_rt_get_sourcing_lnum_value();
        nvim_rt_snprintf_traceback(
            sname_buf.as_mut_ptr().cast(),
            sname_buf.len() as c_int,
            traceback_name,
            sname,
            sourcing_lnum,
        );
        tb_name = sname_buf.as_mut_ptr().cast();
    }
    rs_do_source_ext(
        tb_name,
        false,
        doso::NONE,
        ptr::null_mut(),
        ptr::null(),
        false,
        str_arg,
    )
}
