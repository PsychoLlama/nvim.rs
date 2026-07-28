//! The environment and the paths around it: `environ()`, `expand()`,
//! `stdpath()` and the swap-file queries.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_environ(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let mut env_size: size_t = os_get_fullenv_size();
    let mut env: *mut *mut ::core::ffi::c_char = xmalloc(
        ::core::mem::size_of::<*mut ::core::ffi::c_char>()
            .wrapping_mul(env_size.wrapping_add(1 as size_t)),
    ) as *mut *mut ::core::ffi::c_char;
    *env.offset(env_size as isize) = ::core::ptr::null_mut::<::core::ffi::c_char>();
    os_copy_fullenv(env, env_size);
    let mut i: ssize_t = env_size as ssize_t - 1 as ssize_t;
    while i >= 0 as ssize_t {
        let mut str: *const ::core::ffi::c_char = *env.offset(i as isize);
        let end: *const ::core::ffi::c_char = strchr(
            str.offset(
                (if *str.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '=' as ::core::ffi::c_int
                {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as isize,
            ),
            '=' as ::core::ffi::c_int,
        );
        '_c2rust_label: {
            if !end.is_null() {
            } else {
                __assert_fail(
                    b"end != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1177 as ::core::ffi::c_uint,
                    b"void f_environ(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut len: ptrdiff_t = end.offset_from(str);
        '_c2rust_label_0: {
            if len > 0 as ptrdiff_t {
            } else {
                __assert_fail(
                    b"len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/eval/funcs.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1179 as ::core::ffi::c_uint,
                    b"void f_environ(typval_T *, typval_T *, EvalFuncData)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut value: *const ::core::ffi::c_char = str
            .offset(len as isize)
            .offset(1 as ::core::ffi::c_int as isize);
        let mut c: ::core::ffi::c_char = *(*env.offset(i as isize)).offset(len as isize);
        *(*env.offset(i as isize)).offset(len as isize) = NUL as ::core::ffi::c_char;
        let key: *mut ::core::ffi::c_char = xstrdup(str);
        *(*env.offset(i as isize)).offset(len as isize) = c;
        if !tv_dict_find((*rettv).vval.v_dict, key, len).is_null() {
            xfree(key as *mut ::core::ffi::c_void);
        } else {
            tv_dict_add_str((*rettv).vval.v_dict, key, len as size_t, value);
            xfree(key as *mut ::core::ffi::c_void);
        }
        i -= 1;
    }
    os_free_fullenv(env);
}
pub unsafe extern "C" fn f_getenv(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut p: *mut ::core::ffi::c_char = vim_getenv(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    if p.is_null() {
        (*rettv).v_type = VAR_SPECIAL;
        (*rettv).vval.v_special = kSpecialVarNull;
        return;
    }
    (*rettv).vval.v_string = p;
    (*rettv).v_type = VAR_STRING;
}
pub unsafe extern "C" fn f_expand(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut options: ::core::ffi::c_int = WILD_SILENT as ::core::ffi::c_int
        | WILD_USE_NL as ::core::ffi::c_int
        | WILD_LIST_NOTFOUND as ::core::ffi::c_int;
    let mut error: bool = false_0 != 0;
    (*rettv).v_type = VAR_STRING;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_number_chk(
            argvars.offset(2 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) != 0
        && !error
    {
        tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
    }
    let mut s: *const ::core::ffi::c_char =
        tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
    if *s as ::core::ffi::c_int == '%' as ::core::ffi::c_int
        || *s as ::core::ffi::c_int == '#' as ::core::ffi::c_int
        || *s as ::core::ffi::c_int == '<' as ::core::ffi::c_int
    {
        if p_verbose.get() == 0 as OptInt {
            (*emsg_off.ptr()) += 1;
        }
        let mut len: size_t = 0;
        let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut result: *mut ::core::ffi::c_char = eval_vars(
            s as *mut ::core::ffi::c_char,
            s,
            &raw mut len,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut errormsg,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            false_0 != 0,
        );
        if p_verbose.get() == 0 as OptInt {
            (*emsg_off.ptr()) -= 1;
        } else if !errormsg.is_null() {
            emsg(errormsg);
        }
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_list_alloc_ret(rettv, !result.is_null() as ::core::ffi::c_int as ptrdiff_t);
            if !result.is_null() {
                tv_list_append_string((*rettv).vval.v_list, result, -1 as ssize_t);
            }
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut result as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        } else {
            (*rettv).vval.v_string = result;
        }
    } else {
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && tv_get_number_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0
        {
            options |= WILD_KEEP_ALL as ::core::ffi::c_int;
        }
        if !error {
            let mut xpc: expand_T = expand_T {
                xp_pattern: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_context: 0,
                xp_pattern_len: 0,
                xp_prefix: XP_PREFIX_NONE,
                xp_arg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_luaref: 0,
                xp_script_ctx: sctx_T {
                    sc_sid: 0,
                    sc_seq: 0,
                    sc_lnum: 0,
                    sc_chan: 0,
                },
                xp_backslash: 0,
                xp_shell: false,
                xp_numfiles: 0,
                xp_col: 0,
                xp_selected: 0,
                xp_orig: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_files: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
                xp_line: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                xp_buf: [0; 256],
                xp_search_dir: kDirectionNotSet,
                xp_pre_incsearch_pos: pos_T {
                    lnum: 0,
                    col: 0,
                    coladd: 0,
                },
            };
            ExpandInit(&raw mut xpc);
            xpc.xp_context = EXPAND_FILES as ::core::ffi::c_int;
            if p_wic.get() != 0 {
                options += WILD_ICASE as ::core::ffi::c_int;
            }
            if (*rettv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*rettv).vval.v_string = ExpandOne(
                    &raw mut xpc,
                    s as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    options,
                    WILD_ALL as ::core::ffi::c_int,
                );
            } else {
                ExpandOne(
                    &raw mut xpc,
                    s as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    options,
                    WILD_ALL_KEEP as ::core::ffi::c_int,
                );
                tv_list_alloc_ret(rettv, xpc.xp_numfiles as ptrdiff_t);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < xpc.xp_numfiles {
                    tv_list_append_string(
                        (*rettv).vval.v_list,
                        *xpc.xp_files.offset(i as isize),
                        -1 as ssize_t,
                    );
                    i += 1;
                }
                ExpandCleanup(&raw mut xpc);
            }
        } else {
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
    };
}
pub unsafe extern "C" fn f_expandcmd(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut errormsg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    let mut emsgoff: bool = true_0 != 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_dict_get_bool(
            (*argvars.offset(1 as ::core::ffi::c_int as isize))
                .vval
                .v_dict,
            b"errmsg\0".as_ptr() as *const ::core::ffi::c_char,
            kBoolVarFalse as ::core::ffi::c_int,
        ) != 0
    {
        emsgoff = false_0 != 0;
    }
    (*rettv).v_type = VAR_STRING;
    let mut cmdstr: *mut ::core::ffi::c_char = xstrdup(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    let mut eap: exarg_T = exarg {
        arg: cmdstr,
        args: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        arglens: ::core::ptr::null_mut::<size_t>(),
        argc: 0,
        nextcmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmd: cmdstr,
        cmdlinep: ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        cmdline_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        cmdidx: CMD_USER,
        argt: 0,
        skip: 0,
        forceit: 0,
        addr_count: 0,
        line1: 0,
        line2: 0,
        addr_type: ADDR_LINES,
        flags: 0,
        do_ecmd_cmd: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        do_ecmd_lnum: 0,
        append: 0,
        usefilter: false_0,
        amount: 0,
        regname: 0,
        force_bin: 0,
        read_edit: 0,
        mkdir_p: 0,
        force_ff: 0,
        force_enc: 0,
        bad_char: 0,
        useridx: 0,
        errmsg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ea_getline: None,
        cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        cstack: ::core::ptr::null_mut::<cstack_T>(),
    };
    eap.argt = (eap.argt as ::core::ffi::c_uint | EX_NOSPC) as uint32_t;
    if emsgoff {
        (*emsg_off.ptr()) += 1;
    }
    if expand_filename(&raw mut eap, &raw mut cmdstr, &raw mut errormsg) == FAIL {
        if !emsgoff && !errormsg.is_null() && *errormsg as ::core::ffi::c_int != NUL {
            emsg(errormsg);
        }
    }
    if emsgoff {
        (*emsg_off.ptr()) -= 1;
    }
    (*rettv).vval.v_string = cmdstr;
}
pub unsafe extern "C" fn f_setenv(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut namebuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut valbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut name: *const ::core::ffi::c_char = tv_get_string_buf(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut namebuf as *mut ::core::ffi::c_char,
    );
    if check_secure() {
        return;
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_SPECIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && (*argvars.offset(1 as ::core::ffi::c_int as isize))
            .vval
            .v_special as ::core::ffi::c_uint
            == kSpecialVarNull as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        vim_unsetenv_ext(name);
    } else {
        vim_setenv_ext(
            name,
            tv_get_string_buf(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut valbuf as *mut ::core::ffi::c_char,
            ),
        );
    };
}
pub unsafe extern "C" fn f_setfperm(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = 0 as varnumber_T;
    let fname: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if fname.is_null() {
        return;
    }
    let mut modebuf: [::core::ffi::c_char; 65] = [0; 65];
    let mode_str: *const ::core::ffi::c_char = tv_get_string_buf_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut modebuf as *mut ::core::ffi::c_char,
    );
    if mode_str.is_null() {
        return;
    }
    if strlen(mode_str) != 9 as size_t {
        semsg(
            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
            mode_str,
        );
        return;
    }
    let mut mask: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut mode: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut i: ::core::ffi::c_int = 8 as ::core::ffi::c_int;
    while i >= 0 as ::core::ffi::c_int {
        if *mode_str.offset(i as isize) as ::core::ffi::c_int != '-' as ::core::ffi::c_int {
            mode |= mask;
        }
        mask = mask << 1 as ::core::ffi::c_int;
        i -= 1;
    }
    (*rettv).vval.v_number = (os_setperm(fname, mode) == OK) as ::core::ffi::c_int as varnumber_T;
}
unsafe extern "C" fn get_xdg_var_list(xdg: XDGVarType, mut rettv: *mut typval_T) {
    let list: *mut list_T = tv_list_alloc(kListLenShouldKnow as ::core::ffi::c_int as ptrdiff_t);
    (*rettv).v_type = VAR_LIST;
    (*rettv).vval.v_list = list;
    tv_list_ref(list);
    let dirs: *mut ::core::ffi::c_char = stdpaths_get_xdg_var(xdg);
    if dirs.is_null() {
        return;
    }
    let mut iter: *const ::core::ffi::c_void = ::core::ptr::null::<::core::ffi::c_void>();
    let mut appname: *const ::core::ffi::c_char = get_appname(false_0 != 0);
    loop {
        let mut dir_len: size_t = 0;
        let mut dir: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        iter = vim_env_iter(
            ENV_SEPCHAR as ::core::ffi::c_char,
            dirs,
            iter,
            &raw mut dir,
            &raw mut dir_len,
        );
        if !dir.is_null() && dir_len > 0 as size_t {
            let mut dir_with_nvim: *mut ::core::ffi::c_char =
                xmemdupz(dir as *const ::core::ffi::c_void, dir_len) as *mut ::core::ffi::c_char;
            dir_with_nvim = concat_fnames_realloc(dir_with_nvim, appname, true_0 != 0);
            tv_list_append_allocated_string(list, dir_with_nvim);
        }
        if iter.is_null() {
            break;
        }
    }
    xfree(dirs as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn f_stdpath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let p: *const ::core::ffi::c_char =
        tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
    if p.is_null() {
        return;
    }
    if strequal(p, b"config\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = get_xdg_home(kXDGConfigHome);
    } else if strequal(p, b"data\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = get_xdg_home(kXDGDataHome);
    } else if strequal(p, b"cache\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = get_xdg_home(kXDGCacheHome);
    } else if strequal(p, b"state\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = get_xdg_home(kXDGStateHome);
    } else if strequal(p, b"log\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = get_xdg_home(kXDGStateHome);
    } else if strequal(p, b"run\0".as_ptr() as *const ::core::ffi::c_char) {
        (*rettv).vval.v_string = stdpaths_get_xdg_var(kXDGRuntimeDir);
    } else if strequal(p, b"config_dirs\0".as_ptr() as *const ::core::ffi::c_char) {
        get_xdg_var_list(kXDGConfigDirs, rettv);
    } else if strequal(p, b"data_dirs\0".as_ptr() as *const ::core::ffi::c_char) {
        get_xdg_var_list(kXDGDataDirs, rettv);
    } else {
        semsg(
            gettext(
                b"E6100: \"%s\" is not a valid stdpath\0".as_ptr() as *const ::core::ffi::c_char
            ),
            p,
        );
    };
}
pub unsafe extern "C" fn f_swapfilelist(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
    recover_names(
        ::core::ptr::null_mut::<::core::ffi::c_char>(),
        false_0 != 0,
        (*rettv).vval.v_list,
        0 as ::core::ffi::c_int,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
    );
}
pub unsafe extern "C" fn f_swapinfo(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    swapfile_dict(tv_get_string(argvars), (*rettv).vval.v_dict);
}
pub unsafe extern "C" fn f_swapname(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).v_type = VAR_STRING;
    let mut buf: *mut buf_T = tv_get_buf(argvars.offset(0 as ::core::ffi::c_int as isize), false_0);
    if buf.is_null() || (*buf).b_ml.ml_mfp.is_null() || (*(*buf).b_ml.ml_mfp).mf_fname.is_null() {
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        (*rettv).vval.v_string = xstrdup((*(*buf).b_ml.ml_mfp).mf_fname);
    };
}
