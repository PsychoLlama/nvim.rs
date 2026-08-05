//! The environment and the paths around it: `environ()`, `expand()`,
//! `stdpath()` and the swap-file queries.
#![deny(unsafe_op_in_unsafe_fn)]

use super::args::frame;
use super::{
    ADDR_LINES, CMD_USER, ENV_SEPCHAR, EX_NOSPC, EXPAND_FILES, FAIL, NUL, OK, WILD_ALL,
    WILD_ALL_KEEP, WILD_ICASE, WILD_KEEP_ALL, WILD_LIST_NOTFOUND, WILD_SILENT, WILD_USE_NL,
    kXDGCacheHome, kXDGConfigDirs, kXDGConfigHome, kXDGDataDirs, kXDGDataHome, kXDGRuntimeDir,
    kXDGStateHome, tv_get_buf,
};
use crate::src::nvim::cmdexpand::{ExpandCleanup, ExpandInit, ExpandOne};
use crate::src::nvim::eval::typval::{
    tv_dict_add_str, tv_dict_alloc_ret, tv_dict_find, tv_dict_get_bool, tv_get_number_chk,
    tv_get_string, tv_get_string_buf, tv_get_string_buf_chk, tv_get_string_chk, tv_list_alloc,
    tv_list_alloc_ret, tv_list_append_allocated_string, tv_list_append_string, tv_list_ref,
    tv_list_set_ret,
};
use crate::src::nvim::ex_cmds::check_secure;
use crate::src::nvim::ex_docmd::{eval_vars, expand_filename};
use crate::src::nvim::main::{e_invarg2, emsg_off, p_verbose, p_wic};
use crate::src::nvim::memfile::mf_fname;
use crate::src::nvim::memline::{recover_names, swapfile_dict};
use crate::src::nvim::memory::{xfree, xmalloc, xmemdupz, xstrdup};
use crate::src::nvim::message::{emsg, semsg};
use crate::src::nvim::os::env::{
    os_copy_fullenv, os_free_fullenv, os_get_fullenv_size, vim_env_iter, vim_getenv,
    vim_setenv_ext, vim_unsetenv_ext,
};
use crate::src::nvim::os::fs::os_setperm;
use crate::src::nvim::os::libc::{gettext, strchr, strlen};
use crate::src::nvim::os::stdpaths::{get_appname, get_xdg_home, stdpaths_get_xdg_var};
use crate::src::nvim::path::concat_fnames_realloc;
use crate::src::nvim::types::{
    EvalFuncData, OptInt, VAR_DICT, VAR_LIST, VAR_SPECIAL, VAR_STRING, XDGVarType, exarg_T,
    expand_T, kBoolVarFalse, kListLenShouldKnow, kListLenUnknown, kSpecialVarNull, list_T,
    typval_T, varnumber_T,
};
use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

/// `environ()` — the process environment as a Dictionary.
pub unsafe extern "C" fn f_environ(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (_args, rettv) = frame!(_argvars, rettv);
    // SAFETY: `env` is an array of `env_size` strings plus a NULL, filled by
    // `os_copy_fullenv` and released by `os_free_fullenv`. Every string is
    // NUL-terminated and writable — the split below writes a NUL into one
    // and puts the original byte back.
    unsafe {
        tv_dict_alloc_ret(rettv);
        let env_size = os_get_fullenv_size();
        let env = xmalloc(size_of::<*mut c_char>() * (env_size + 1)) as *mut *mut c_char;
        *env.add(env_size) = ptr::null_mut();
        os_copy_fullenv(env, env_size);
        // Walked backwards, so that when a name appears twice the *first*
        // entry is the one that survives the duplicate check below.
        for i in (0..env_size).rev() {
            let entry = *env.add(i);
            // A leading '=' is part of the name on the platforms that allow
            // it, so the separator search starts past it.
            let skip = usize::from(*entry == b'=' as c_char);
            let end = strchr(entry.add(skip), b'=' as c_int);
            debug_assert!(!end.is_null());
            let len = end.offset_from(entry);
            debug_assert!(len > 0);
            let value = entry.offset(len).add(1);

            let saved = *entry.offset(len);
            *entry.offset(len) = NUL as c_char;
            let key = xstrdup(entry);
            *entry.offset(len) = saved;
            if tv_dict_find(rettv.vval.v_dict, key, len).is_null() {
                tv_dict_add_str(rettv.vval.v_dict, key, len as usize, value);
            }
            xfree(key as *mut c_void);
        }
        os_free_fullenv(env);
    }
}

/// `getenv({name})` — the variable's value, or `v:null` when it is unset.
pub unsafe extern "C" fn f_getenv(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: `vim_getenv` returns an owned string or null.
    let p = unsafe { vim_getenv(tv_get_string(args.ptr(0))) };
    if p.is_null() {
        rettv.v_type = VAR_SPECIAL;
        rettv.vval.v_special = kSpecialVarNull;
    } else {
        rettv.vval.v_string = p;
        rettv.v_type = VAR_STRING;
    }
}

/// `expand({string} [, {nosuf} [, {list}]])`.
pub unsafe extern "C" fn f_expand(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    let mut options = (WILD_SILENT | WILD_USE_NL | WILD_LIST_NOTFOUND) as c_int;
    let mut error = false;
    rettv.v_type = VAR_STRING;
    // SAFETY: `s` points into the argument, which outlives every call here;
    // `xpc` is cleared by `ExpandInit` before use and cleaned up after.
    unsafe {
        // The `{list}` argument is only honoured when `{nosuf}` was given
        // too, because it is the third.
        if args.has(1)
            && args.has(2)
            && tv_get_number_chk(args.ptr(2), &raw mut error) != 0
            && !error
        {
            tv_list_set_ret(rettv, ptr::null_mut::<list_T>());
        }
        let s = tv_get_string(args.ptr(0));
        if matches!(*s as u8, b'%' | b'#' | b'<') {
            // A `%`/`#`/`<` item is resolved by the Ex-command machinery,
            // whose own errors are suppressed unless 'verbose' is set.
            let quiet = p_verbose.get() == 0 as OptInt;
            if quiet {
                *emsg_off.ptr() += 1;
            }
            let mut len: usize = 0;
            let mut errormsg: *const c_char = ptr::null();
            let result = eval_vars(
                s as *mut c_char,
                s,
                &raw mut len,
                ptr::null_mut(),
                &raw mut errormsg,
                ptr::null_mut(),
                false,
            );
            if quiet {
                *emsg_off.ptr() -= 1;
            } else if !errormsg.is_null() {
                emsg(errormsg);
            }
            if rettv.v_type == VAR_LIST {
                tv_list_alloc_ret(rettv, isize::from(!result.is_null()));
                if !result.is_null() {
                    tv_list_append_string(rettv.vval.v_list, result, -1);
                }
                xfree(result as *mut c_void);
            } else {
                rettv.vval.v_string = result;
            }
            return;
        }
        if args.has(1) && tv_get_number_chk(args.ptr(1), &raw mut error) != 0 {
            options |= WILD_KEEP_ALL as c_int;
        }
        if error {
            rettv.vval.v_string = ptr::null_mut();
            return;
        }
        let mut xpc: expand_T = core::mem::zeroed();
        ExpandInit(&raw mut xpc);
        xpc.xp_context = EXPAND_FILES as c_int;
        if p_wic.get() != 0 {
            // `+=`, as upstream has it; WILD_ICASE is not already set, so it
            // is the same as `|=`.
            options += WILD_ICASE as c_int;
        }
        if rettv.v_type == VAR_STRING {
            rettv.vval.v_string = ExpandOne(
                &raw mut xpc,
                s as *mut c_char,
                ptr::null_mut(),
                options,
                WILD_ALL as c_int,
            );
        } else {
            ExpandOne(
                &raw mut xpc,
                s as *mut c_char,
                ptr::null_mut(),
                options,
                WILD_ALL_KEEP as c_int,
            );
            tv_list_alloc_ret(rettv, xpc.xp_numfiles as isize);
            for i in 0..xpc.xp_numfiles {
                tv_list_append_string(rettv.vval.v_list, *xpc.xp_files.offset(i as isize), -1);
            }
            ExpandCleanup(&raw mut xpc);
        }
    }
}

/// `expandcmd({string} [, {options}])` — expand the `%`, `#` and wildcard
/// items in a command line.
pub unsafe extern "C" fn f_expandcmd(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    // SAFETY: `cmdstr` is owned here and handed to the return value;
    // `expand_filename` may replace it with another owned string.
    unsafe {
        // {'errmsg': v:true} asks for the expansion's own error instead of
        // silence.
        let quiet = !(args.ty(1) == VAR_DICT
            && tv_dict_get_bool(
                args.get(1).vval.v_dict,
                c"errmsg".as_ptr(),
                kBoolVarFalse as c_int,
            ) != 0);
        let mut cmdstr = xstrdup(tv_get_string(args.ptr(0)));
        let mut eap: exarg_T = core::mem::zeroed();
        eap.arg = cmdstr;
        eap.cmd = cmdstr;
        eap.cmdidx = CMD_USER;
        eap.addr_type = ADDR_LINES;
        eap.argt = EX_NOSPC;
        let mut errormsg: *const c_char = ptr::null();
        if quiet {
            *emsg_off.ptr() += 1;
        }
        if expand_filename(&raw mut eap, &raw mut cmdstr, &raw mut errormsg) == FAIL
            && !quiet
            && !errormsg.is_null()
            && *errormsg as c_int != NUL
        {
            emsg(errormsg);
        }
        if quiet {
            *emsg_off.ptr() -= 1;
        }
        rettv.vval.v_string = cmdstr;
    }
}

/// `setenv({name}, {val})` — `v:null` unsets.
pub unsafe extern "C" fn f_setenv(
    argvars: *mut typval_T,
    _rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, _rettv) = frame!(argvars, _rettv);
    // SAFETY: the two scratch buffers outlive the strings coerced into them.
    unsafe {
        let mut namebuf: [c_char; 65] = [0; 65];
        let mut valbuf: [c_char; 65] = [0; 65];
        // Coerced before the sandbox check, as upstream has it: the
        // coercion can report an error of its own.
        let name = tv_get_string_buf(args.ptr(0), namebuf.as_mut_ptr());
        if check_secure() {
            return;
        }
        if args.ty(1) == VAR_SPECIAL && args.get(1).vval.v_special == kSpecialVarNull {
            vim_unsetenv_ext(name);
        } else {
            vim_setenv_ext(name, tv_get_string_buf(args.ptr(1), valbuf.as_mut_ptr()));
        }
    }
}

/// `setfperm({fname}, {mode})` — `{mode}` is nine "rwxrwxrwx" characters,
/// any of which is "off" only when it is a `-`.
pub unsafe extern "C" fn f_setfperm(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.vval.v_number = 0;
    // SAFETY: both strings are coerced from the frame and NUL-terminated;
    // the nine bytes read below are covered by the length check.
    unsafe {
        let fname = tv_get_string_chk(args.ptr(0));
        if fname.is_null() {
            return;
        }
        let mut modebuf: [c_char; 65] = [0; 65];
        let mode_str = tv_get_string_buf_chk(args.ptr(1), modebuf.as_mut_ptr());
        if mode_str.is_null() {
            return;
        }
        if strlen(mode_str) != 9 {
            semsg(gettext(e_invarg2.ptr() as *const c_char), mode_str);
            return;
        }
        let mut mode: c_int = 0;
        for i in (0..9).rev() {
            if *mode_str.offset(i) != b'-' as c_char {
                mode |= 1 << (8 - i);
            }
        }
        rettv.vval.v_number = (os_setperm(fname, mode) == OK) as varnumber_T;
    }
}

/// The `config_dirs`/`data_dirs` answer: every directory in the XDG search
/// path, each with the application name appended.
///
/// # Safety
/// `rettv` is the cleared return value.
unsafe fn get_xdg_var_list(xdg: XDGVarType, rettv: &mut typval_T) {
    // SAFETY: the caller's obligation. `dirs` is owned here; `vim_env_iter`
    // hands back slices of it and null when the walk is done.
    unsafe {
        let list = tv_list_alloc(kListLenShouldKnow as isize);
        rettv.v_type = VAR_LIST;
        rettv.vval.v_list = list;
        tv_list_ref(list);
        let dirs = stdpaths_get_xdg_var(xdg);
        if dirs.is_null() {
            return;
        }
        let appname = get_appname(false);
        let mut iter: *const c_void = ptr::null();
        loop {
            let mut dir_len: usize = 0;
            let mut dir: *const c_char = ptr::null();
            iter = vim_env_iter(
                ENV_SEPCHAR as c_char,
                dirs,
                iter,
                &raw mut dir,
                &raw mut dir_len,
            );
            if !dir.is_null() && dir_len > 0 {
                let dir = xmemdupz(dir as *const c_void, dir_len) as *mut c_char;
                tv_list_append_allocated_string(list, concat_fnames_realloc(dir, appname, true));
            }
            if iter.is_null() {
                break;
            }
        }
        xfree(dirs as *mut c_void);
    }
}

/// `stdpath({what})`.
pub unsafe extern "C" fn f_stdpath(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    rettv.vval.v_string = ptr::null_mut();
    // SAFETY: `p` is coerced from the frame and NUL-terminated once the
    // null check has passed.
    unsafe {
        let p = tv_get_string_chk(args.ptr(0));
        if p.is_null() {
            return;
        }
        rettv.vval.v_string = match CStr::from_ptr(p).to_bytes() {
            b"config" => get_xdg_home(kXDGConfigHome),
            b"data" => get_xdg_home(kXDGDataHome),
            b"cache" => get_xdg_home(kXDGCacheHome),
            // "log" is deliberately the state directory: the log file lives
            // there and there is no XDG log home.
            b"state" | b"log" => get_xdg_home(kXDGStateHome),
            b"run" => stdpaths_get_xdg_var(kXDGRuntimeDir),
            b"config_dirs" => return get_xdg_var_list(kXDGConfigDirs, rettv),
            b"data_dirs" => return get_xdg_var_list(kXDGDataDirs, rettv),
            _ => {
                // The name is arbitrary user bytes, so this keeps the
                // variadic call.
                semsg(gettext(c"E6100: \"%s\" is not a valid stdpath".as_ptr()), p);
                return;
            }
        };
    }
}

/// `swapfilelist()` — every swap file in 'directory'.
pub unsafe extern "C" fn f_swapfilelist(
    _argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (_args, rettv) = frame!(_argvars, rettv);
    // SAFETY: `recover_names` appends to the list just allocated.
    unsafe {
        tv_list_alloc_ret(rettv, kListLenUnknown as isize);
        recover_names(
            ptr::null_mut(),
            false,
            rettv.vval.v_list,
            0,
            ptr::null_mut(),
        );
    }
}

/// `swapinfo({fname})` — what a swap file says about its buffer.
pub unsafe extern "C" fn f_swapinfo(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    // SAFETY: the dict is allocated into the return value first, so
    // `swapfile_dict` has somewhere to write.
    unsafe {
        tv_dict_alloc_ret(rettv);
        swapfile_dict(tv_get_string(args.ptr(0)), rettv.vval.v_dict);
    }
}

/// `swapname({buf})` — the swap file a buffer is using, if any.
pub unsafe extern "C" fn f_swapname(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    _fptr: EvalFuncData,
) {
    let (args, rettv) = frame!(argvars, rettv);
    rettv.v_type = VAR_STRING;
    // SAFETY: the buffer comes from the buffer list; the memfile and its
    // name are checked before either is read.
    unsafe {
        let buf = tv_get_buf(args.ptr(0), 0);
        rettv.vval.v_string = if buf.is_null()
            || (*buf).b_ml.ml_mfp.is_null()
            || mf_fname((*buf).b_ml.ml_mfp).is_null()
        {
            ptr::null_mut()
        } else {
            xstrdup(mf_fname((*buf).b_ml.ml_mfp))
        };
    }
}
