//! Profile command handling, state management, and dump.
//!
//! Implements `:profile` ex command, command-line completion,
//! `profile_reset`, `profile_dump`, and the dump output functions.

use std::os::raw::{c_char, c_int};

use crate::types::{ExargHandle, ExpandHandle, FileHandle};
use crate::Proftime;

// Constants — verified against C headers, using accessor functions for safety
extern "C" {
    fn nvim_profile_get_prof_none() -> c_int;
    fn nvim_profile_get_prof_yes() -> c_int;
    fn nvim_profile_get_prof_paused() -> c_int;
    fn nvim_profile_get_do_profiling() -> c_int;
    fn nvim_profile_set_do_profiling(val: c_int);
    fn nvim_profile_set_vim_var_nr_profiling(val: c_int);
    fn nvim_profile_emsg_e750();
    fn nvim_profile_semsg_notopen(fname: *const c_char);
    fn nvim_profile_ex_breakadd(eap: ExargHandle);
    fn nvim_profile_xfree(ptr: *mut c_char);
    fn nvim_profile_expand_env_save_opt(src: *mut c_char) -> *mut c_char;

    // eap accessors
    fn nvim_profile_eap_get_arg(eap: ExargHandle) -> *mut c_char;

    // string helpers
    fn nvim_profile_skiptowhite(s: *const c_char) -> *mut c_char;
    fn nvim_profile_skipwhite(s: *const c_char) -> *mut c_char;

    // expand_T accessors
    fn nvim_profile_xp_set_context(xp: ExpandHandle, ctx: c_int);
    fn nvim_profile_xp_set_pattern(xp: ExpandHandle, pat: *const c_char);

    // expand context values
    fn nvim_profile_get_expand_profile() -> c_int;
    fn nvim_profile_get_expand_files() -> c_int;
    fn nvim_profile_get_expand_user_func() -> c_int;
    fn nvim_profile_get_expand_nothing() -> c_int;

    // file I/O
    fn nvim_profile_os_fopen(name: *const c_char, mode: *const c_char) -> FileHandle;
    fn nvim_profile_fclose(fd: FileHandle);

    // profile_reset
    fn nvim_profile_reset_scripts();
    fn nvim_profile_reset_funcs();

    // dump functions (remain in C, iterate hashtable/garray)
    fn nvim_profile_script_dump(fd: FileHandle);
    fn nvim_profile_func_dump(fd: FileHandle);

    // C standard
    fn strncmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

/// Profile output filename. Replaces `static char *profile_fname` in profile.c.
static mut PROFILE_FNAME: *mut c_char = std::ptr::null_mut();

/// Pause time for `:profile pause/continue`. Replaces `static proftime_T pause_time`.
static mut PAUSE_TIME: Proftime = 0;

/// `:profile` sub-command names for completion.
static PEXPAND_CMDS: [&[u8]; 8] = [
    b"continue\0",
    b"dump\0",
    b"file\0",
    b"func\0",
    b"pause\0",
    b"start\0",
    b"stop\0",
    b"\0", // sentinel (NULL equivalent)
];

/// Expansion state: which kind of completion to perform.
static mut PEXPAND_WHAT: c_int = 0; // 0 = PEXP_SUBCMD

const PEXP_SUBCMD: c_int = 0;

/// `:profile cmd args` command handler.
///
/// # Safety
///
/// `eap` must be a valid `exarg_T *`.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_profile(eap: ExargHandle) {
    let arg = nvim_profile_eap_get_arg(eap);
    let e = nvim_profile_skiptowhite(arg);
    let len = e.offset_from(arg) as c_int;
    let e = nvim_profile_skipwhite(e);

    let prof_none = nvim_profile_get_prof_none();
    let prof_yes = nvim_profile_get_prof_yes();
    let prof_paused = nvim_profile_get_prof_paused();

    if len == 5 && strncmp(arg, c"start".as_ptr(), 5) == 0 && *e != 0 {
        let fname_ptr = std::ptr::addr_of_mut!(PROFILE_FNAME);
        let old = (*fname_ptr).cast::<c_char>();
        if !old.is_null() {
            nvim_profile_xfree(old);
        }
        (*fname_ptr) = nvim_profile_expand_env_save_opt(e);
        nvim_profile_set_do_profiling(prof_yes);
        crate::timing::rs_profile_set_wait(crate::rs_profile_zero());
        nvim_profile_set_vim_var_nr_profiling(1);
    } else if nvim_profile_get_do_profiling() == prof_none {
        nvim_profile_emsg_e750();
    } else if strcmp(arg, c"stop".as_ptr()) == 0 {
        rs_profile_dump();
        nvim_profile_set_do_profiling(prof_none);
        nvim_profile_set_vim_var_nr_profiling(0);
        rs_profile_reset();
    } else if strcmp(arg, c"pause".as_ptr()) == 0 {
        if nvim_profile_get_do_profiling() == prof_yes {
            std::ptr::addr_of_mut!(PAUSE_TIME).write(crate::timing::rs_profile_start());
        }
        nvim_profile_set_do_profiling(prof_paused);
    } else if strcmp(arg, c"continue".as_ptr()) == 0 {
        if nvim_profile_get_do_profiling() == prof_paused {
            let pt = std::ptr::addr_of_mut!(PAUSE_TIME);
            let elapsed = crate::timing::rs_profile_end(pt.read());
            pt.write(elapsed);
            let new_wait =
                crate::rs_profile_add(crate::timing::rs_profile_get_wait_time(), elapsed);
            crate::timing::rs_profile_set_wait(new_wait);
        }
        nvim_profile_set_do_profiling(prof_yes);
    } else if strcmp(arg, c"dump".as_ptr()) == 0 {
        rs_profile_dump();
    } else {
        nvim_profile_ex_breakadd(eap);
    }
}

/// Function given to ExpandGeneric() to obtain profile command completion.
///
/// # Safety
///
/// `xp` is unused. `idx` must be a valid index.
#[no_mangle]
pub unsafe extern "C" fn rs_get_profile_name(_xp: ExpandHandle, idx: c_int) -> *const c_char {
    let what = std::ptr::addr_of!(PEXPAND_WHAT).read();
    if what == PEXP_SUBCMD {
        let i = idx as usize;
        if i < PEXPAND_CMDS.len() - 1 {
            return PEXPAND_CMDS[i].as_ptr().cast::<c_char>();
        }
    }
    std::ptr::null()
}

/// Handle command line completion for `:profile` command.
///
/// # Safety
///
/// `xp` must be a valid `expand_T *`. `arg` must be a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_set_context_in_profile_cmd(xp: ExpandHandle, arg: *const c_char) {
    nvim_profile_xp_set_context(xp, nvim_profile_get_expand_profile());
    std::ptr::addr_of_mut!(PEXPAND_WHAT).write(PEXP_SUBCMD);
    nvim_profile_xp_set_pattern(xp, arg);

    let end_subcmd = nvim_profile_skiptowhite(arg);
    if *end_subcmd == 0 {
        return;
    }

    let subcmd_len = end_subcmd.offset_from(arg);

    if (subcmd_len == 5 && strncmp(arg, c"start".as_ptr(), 5) == 0)
        || (subcmd_len == 4 && strncmp(arg, c"file".as_ptr(), 4) == 0)
    {
        nvim_profile_xp_set_context(xp, nvim_profile_get_expand_files());
        nvim_profile_xp_set_pattern(xp, nvim_profile_skipwhite(end_subcmd));
        return;
    } else if subcmd_len == 4 && strncmp(arg, c"func".as_ptr(), 4) == 0 {
        nvim_profile_xp_set_context(xp, nvim_profile_get_expand_user_func());
        nvim_profile_xp_set_pattern(xp, nvim_profile_skipwhite(end_subcmd));
        return;
    }

    nvim_profile_xp_set_context(xp, nvim_profile_get_expand_nothing());
}

/// Reset all profiling information.
///
/// # Safety
///
/// Accesses C global data structures via FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_profile_reset() {
    nvim_profile_reset_scripts();
    nvim_profile_reset_funcs();

    let fname_ptr = std::ptr::addr_of_mut!(PROFILE_FNAME);
    let old = (*fname_ptr).cast::<c_char>();
    if !old.is_null() {
        nvim_profile_xfree(old);
        *fname_ptr = std::ptr::null_mut();
    }
}

/// Dump the profiling info.
///
/// # Safety
///
/// Accesses profile_fname static and C file I/O.
#[no_mangle]
pub unsafe extern "C" fn rs_profile_dump() {
    let fname = std::ptr::addr_of!(PROFILE_FNAME).read();
    if fname.is_null() {
        return;
    }

    let fd = nvim_profile_os_fopen(fname, c"w".as_ptr());
    if fd.is_null() {
        nvim_profile_semsg_notopen(fname);
    } else {
        nvim_profile_script_dump(fd);
        nvim_profile_func_dump(fd);
        nvim_profile_fclose(fd);
    }
}
