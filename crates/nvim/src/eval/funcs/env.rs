//! The environment and the paths around it: `environ()`, `expand()`,
//! `stdpath()` and the swap-file queries.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::wrappers::{
    arg_number_chk, arg_string, arg_string_chk, dict_alloc_ret, list_alloc_ret, list_set_ret,
};
use super::{
    ENV_SEPCHAR, kXDGCacheHome, kXDGConfigDirs, kXDGConfigHome, kXDGDataDirs, kXDGDataHome,
    kXDGRuntimeDir, kXDGStateHome, tv_get_buf,
};
use crate::cmdexpand::{WildMode, WildOpts, expand_cleanup, expand_init, expand_one};
use crate::eval::typval::{
    NumBuf, tv_dict_add_str, tv_dict_find, tv_dict_get_bool, tv_list_alloc,
    tv_list_append_allocated_string, tv_list_append_string, tv_list_ref,
};
use crate::ex_cmds::check_secure;
use crate::ex_docmd::{eval_vars, expand_filename};
use crate::guard::Suppress;
use crate::main::{e_invarg2, p_verbose, p_wic};
use crate::memfile::mf_fname;
use crate::memline::{recover_names, swapfile_dict};
use crate::memory::{xfree, xmalloc, xmemdupz, xstrdup};
use crate::message::emsg;
use crate::os::cshim::{gettext, strchr};
use crate::os::env::{
    os_copy_fullenv, os_free_fullenv, os_get_fullenv_size, vim_env_iter, vim_getenv,
    vim_setenv_ext, vim_unsetenv_ext,
};
use crate::os::fs::os_setperm;
use crate::os::stdpaths::{get_appname, get_xdg_home, stdpaths_get_xdg_var};
use crate::path::concat_fnames_realloc;
use crate::semsg_c;
use crate::types::{
    CMD_USER, CmdAddr, EvalFuncData, ExArgt, ExpandContext, FAIL, NUL, OK, OptInt, VAR_DICT,
    VAR_LIST, VAR_SPECIAL, VAR_STRING, XDGVarType, exarg_T, expand_T, kBoolVarFalse,
    kListLenShouldKnow, kListLenUnknown, kSpecialVarNull, list_T, typval_T, varnumber_T,
};
use ::libc::strlen;
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// `environ()` — the process environment as a Dictionary.
pub unsafe fn f_environ(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (_args, rettv) = frame!(_argvars, rettv);
    // SAFETY: `env` is an array of `env_size` strings plus a NULL, filled by
    // `os_copy_fullenv` and released by `os_free_fullenv`. Every string is
    // NUL-terminated and writable — the split below writes a NUL into one
    // and puts the original byte back.
    dict_alloc_ret(rettv);
    let env_size = os_get_fullenv_size();
    let env = unsafe { xmalloc(size_of::<*mut c_char>() * (env_size + 1)) } as *mut *mut c_char;
    unsafe { *env.add(env_size) = ptr::null_mut() };
    unsafe { os_copy_fullenv(env, env_size) };
    // Walked backwards, so that when a name appears twice the *first*
    // entry is the one that survives the duplicate check below.
    for i in (0..env_size).rev() {
        let entry = unsafe { *env.add(i) };
        // A leading '=' is part of the name on the platforms that allow
        // it, so the separator search starts past it.
        let skip = usize::from(unsafe { *entry } == b'=' as c_char);
        let end = unsafe { strchr(entry.add(skip), b'=' as c_int) };
        debug_assert!(!end.is_null());
        let len = unsafe { end.offset_from(entry) };
        debug_assert!(len > 0);
        let value = unsafe { entry.offset(len).add(1) };

        let saved = unsafe { *entry.offset(len) };
        unsafe { *entry.offset(len) = NUL as c_char };
        let key = unsafe { xstrdup(entry) };
        unsafe { *entry.offset(len) = saved };
        if unsafe { tv_dict_find(rettv.vval.v_dict, key, len) }.is_null() {
            unsafe { tv_dict_add_str(rettv.vval.v_dict, key, len as usize, value) };
        }
        unsafe { xfree(key as *mut c_void) };
    }
    unsafe { os_free_fullenv(env) };
}

/// `getenv({name})` — the variable's value, or `v:null` when it is unset.
pub unsafe fn f_getenv(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: `vim_getenv` returns an owned string or null.
    let p = unsafe { vim_getenv(arg_string(&mut numbuf, args.get(0))) };
    if p.is_null() {
        rettv.v_type = VAR_SPECIAL;
        rettv.vval.v_special = kSpecialVarNull;
    } else {
        rettv.vval.v_string = p;
        rettv.v_type = VAR_STRING;
    }
}

/// `expand({string} [, {nosuf} [, {list}]])`.
pub unsafe fn f_expand(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    let mut options = WildOpts::SILENT | WildOpts::USE_NL | WildOpts::LIST_NOTFOUND;
    let mut error = false;
    rettv.v_type = VAR_STRING;
    // SAFETY: `s` points into the argument, which outlives every call here;
    // `xpc` is cleared by `expand_init` before use and cleaned up after.
    // The `{list}` argument is only honoured when `{nosuf}` was given
    // too, because it is the third.
    if args.has(1) && args.has(2) && arg_number_chk(args.get(2), Some(&mut error)) != 0 && !error {
        list_set_ret(rettv, ptr::null_mut::<list_T>());
    }
    let s = arg_string(&mut numbuf, args.get(0));
    if matches!(unsafe { *s } as u8, b'%' | b'#' | b'<') {
        // A `%`/`#`/`<` item is resolved by the Ex-command machinery,
        // whose own errors are suppressed unless 'verbose' is set.
        let quiet = p_verbose.get() == 0 as OptInt;
        let no_emsg = quiet.then(Suppress::emsg);
        let mut len: usize = 0;
        let mut errormsg: *const c_char = ptr::null();
        let result = unsafe {
            eval_vars(
                s as *mut c_char,
                s,
                &raw mut len,
                ptr::null_mut(),
                &raw mut errormsg,
                ptr::null_mut(),
                false,
            )
        };
        drop(no_emsg);
        if !quiet && !errormsg.is_null() {
            unsafe { emsg(errormsg) };
        }
        if rettv.v_type == VAR_LIST {
            list_alloc_ret(rettv, isize::from(!result.is_null()));
            if !result.is_null() {
                unsafe { tv_list_append_string(rettv.vval.v_list, result, -1) };
            }
            unsafe { xfree(result as *mut c_void) };
        } else {
            rettv.vval.v_string = result;
        }
        return;
    }
    if args.has(1) && arg_number_chk(args.get(1), Some(&mut error)) != 0 {
        options |= WildOpts::KEEP_ALL;
    }
    if error {
        rettv.vval.v_string = ptr::null_mut();
        return;
    }
    let mut xpc: expand_T = unsafe { core::mem::zeroed() };
    unsafe { expand_init(&raw mut xpc) };
    xpc.xp_context = ExpandContext::Files;
    if p_wic.get() != 0 {
        options |= WildOpts::ICASE;
    }
    if rettv.v_type == VAR_STRING {
        rettv.vval.v_string = unsafe {
            expand_one(
                &raw mut xpc,
                s as *mut c_char,
                ptr::null_mut(),
                options,
                WildMode::All,
            )
        };
    } else {
        unsafe {
            expand_one(
                &raw mut xpc,
                s as *mut c_char,
                ptr::null_mut(),
                options,
                WildMode::AllKeep,
            )
        };
        list_alloc_ret(rettv, xpc.xp_numfiles as isize);
        for i in 0..xpc.xp_numfiles {
            unsafe {
                tv_list_append_string(rettv.vval.v_list, *xpc.xp_files.offset(i as isize), -1)
            };
        }
        unsafe { expand_cleanup(&raw mut xpc) };
    }
}

/// `expandcmd({string} [, {options}])` — expand the `%`, `#` and wildcard
/// items in a command line.
pub unsafe fn f_expandcmd(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    // SAFETY: `cmdstr` is owned here and handed to the return value;
    // `expand_filename` may replace it with another owned string.
    // {'errmsg': v:true} asks for the expansion's own error instead of
    // silence.
    let quiet = !(args.ty(1) == VAR_DICT
        && unsafe {
            tv_dict_get_bool(
                args.get(1).vval.v_dict,
                c"errmsg".as_ptr(),
                kBoolVarFalse as c_int,
            )
        } != 0);
    let mut cmdstr = unsafe { xstrdup(arg_string(&mut numbuf, args.get(0))) };
    let mut eap: exarg_T = unsafe { core::mem::zeroed() };
    eap.arg = cmdstr;
    eap.cmd = cmdstr;
    eap.cmdidx = CMD_USER;
    eap.addr_type = CmdAddr::Lines;
    eap.argt = ExArgt::NOSPC;
    let mut errormsg = None;
    let _no_emsg = quiet.then(Suppress::emsg);
    if unsafe { expand_filename(&raw mut eap, &raw mut cmdstr, &mut errormsg) } == FAIL
        && !quiet
        && let Some(msg) = &errormsg
        && !msg.is_empty()
    {
        unsafe { emsg(msg.as_ptr()) };
    }
    rettv.vval.v_string = cmdstr;
}

/// `setenv({name}, {val})` — `v:null` unsets.
pub unsafe fn f_setenv(argvars: *mut typval_T, _rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY: the two scratch buffers outlive the strings coerced into them.
    let mut namebuf = NumBuf::new();
    let mut valbuf = NumBuf::new();
    // Coerced before the sandbox check, as upstream has it: the
    // coercion can report an error of its own.
    let name = arg_string(&mut namebuf, args.get(0));
    if check_secure() {
        return;
    }
    if args.ty(1) == VAR_SPECIAL && unsafe { args.get(1).vval.v_special } == kSpecialVarNull {
        unsafe { vim_unsetenv_ext(name) };
    } else {
        unsafe { vim_setenv_ext(name, arg_string(&mut valbuf, args.get(1))) };
    }
}

/// `setfperm({fname}, {mode})` — `{mode}` is nine "rwxrwxrwx" characters,
/// any of which is "off" only when it is a `-`.
pub unsafe fn f_setfperm(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = 0;
    // SAFETY: both strings are coerced from the frame and NUL-terminated;
    // the nine bytes read below are covered by the length check.
    let fname = arg_string_chk(&mut numbuf, args.get(0));
    if fname.is_null() {
        return;
    }
    let mut modebuf = NumBuf::new();
    let mode_str = arg_string_chk(&mut modebuf, args.get(1));
    if mode_str.is_null() {
        return;
    }
    if unsafe { strlen(mode_str) } != 9 {
        semsg_c!(unsafe { gettext(e_invarg2.as_ptr()) }, mode_str);
        return;
    }
    let mut mode: c_int = 0;
    for i in (0..9).rev() {
        if unsafe { *mode_str.offset(i) } != b'-' as c_char {
            mode |= 1 << (8 - i);
        }
    }
    rettv.vval.v_number = (unsafe { os_setperm(fname, mode) } == OK) as varnumber_T;
}

/// The `config_dirs`/`data_dirs` answer: every directory in the XDG search
/// path, each with the application name appended.
///
/// # Safety
/// `rettv` is the cleared return value.
unsafe fn get_xdg_var_list(xdg: XDGVarType, rettv: &mut typval_T) {
    let appname = get_appname(false);
    // SAFETY: the caller's obligation. `dirs` is owned here; `vim_env_iter`
    // hands back slices of it and null when the walk is done.
    let list = unsafe { tv_list_alloc(kListLenShouldKnow as isize) };
    rettv.v_type = VAR_LIST;
    rettv.vval.v_list = list;
    unsafe { tv_list_ref(list) };
    let dirs = stdpaths_get_xdg_var(xdg);
    if dirs.is_null() {
        return;
    }
    let mut iter: *const c_void = ptr::null();
    loop {
        let mut dir_len: usize = 0;
        let mut dir: *const c_char = ptr::null();
        iter = unsafe {
            vim_env_iter(
                ENV_SEPCHAR as c_char,
                dirs,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            )
        };
        if !dir.is_null() && dir_len > 0 {
            let dir = unsafe { xmemdupz(dir as *const c_void, dir_len) } as *mut c_char;
            let path = unsafe { concat_fnames_realloc(dir, appname.as_ptr(), true) };
            unsafe { tv_list_append_allocated_string(list, path) };
        }
        if iter.is_null() {
            break;
        }
    }
    unsafe { xfree(dirs as *mut c_void) };
}

/// `stdpath({what})`.
pub unsafe fn f_stdpath(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: `p` is coerced from the frame and NUL-terminated once the
    // null check has passed.
    let p = arg_string_chk(&mut numbuf, args.get(0));
    if p.is_null() {
        return;
    }
    rettv.vval.v_string = match unsafe { CStr::from_ptr(p) }.to_bytes() {
        b"config" => get_xdg_home(kXDGConfigHome),
        b"data" => get_xdg_home(kXDGDataHome),
        b"cache" => get_xdg_home(kXDGCacheHome),
        // "log" is deliberately the state directory: the log file lives
        // there and there is no XDG log home.
        b"state" | b"log" => get_xdg_home(kXDGStateHome),
        b"run" => stdpaths_get_xdg_var(kXDGRuntimeDir),
        b"config_dirs" => return unsafe { get_xdg_var_list(kXDGConfigDirs, rettv) },
        b"data_dirs" => return unsafe { get_xdg_var_list(kXDGDataDirs, rettv) },
        _ => {
            // The name is arbitrary user bytes, so this keeps the
            // variadic call.
            semsg_c!(
                unsafe { gettext(c"E6100: \"%s\" is not a valid stdpath".as_ptr()) },
                p
            );
            return;
        }
    };
}

/// `swapfilelist()` — every swap file in 'directory'.
pub unsafe fn f_swapfilelist(_argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (_args, rettv) = frame!(_argvars, rettv);
    // SAFETY: `recover_names` appends to the list just allocated.
    list_alloc_ret(rettv, kListLenUnknown as isize);
    unsafe {
        recover_names(
            ptr::null_mut(),
            false,
            rettv.vval.v_list,
            0,
            ptr::null_mut(),
        )
    };
}

/// `swapinfo({fname})` — what a swap file says about its buffer.
pub unsafe fn f_swapinfo(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let mut numbuf = NumBuf::new();
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the dict is allocated into the return value first, so
    // `swapfile_dict` has somewhere to write.
    dict_alloc_ret(rettv);
    unsafe { swapfile_dict(arg_string(&mut numbuf, args.get(0)), rettv.vval.v_dict) };
}

/// `swapname({buf})` — the swap file a buffer is using, if any.
pub unsafe fn f_swapname(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    // SAFETY: the buffer comes from the buffer list; the memfile and its
    // name are checked before either is read.
    let buf = unsafe { tv_get_buf(args.ptr(0), 0) };
    rettv.vval.v_string = if buf.is_null()
        || unsafe { (*buf).b_ml.ml_mfp }.is_null()
        || unsafe { mf_fname((*buf).b_ml.ml_mfp) }.is_null()
    {
        ptr::null_mut()
    } else {
        unsafe { xstrdup(mf_fname((*buf).b_ml.ml_mfp)) }
    };
}
