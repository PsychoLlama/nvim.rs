//! Looking for files -- `glob()`, `globpath()`, `finddir()`, `findfile()` and
//! `readdir()`.
//!
//! `f_glob` and `f_globpath` expand a wildcard pattern through the same
//! `ExpandOne`/`globpath` machinery the command line uses, so they answer to
//! 'wildignore', 'suffixes' and 'wildignorecase'; `findfilendir` is the shared
//! body of `finddir()`/`findfile()`, which walk 'path' looking for a name
//! rather than expanding a pattern; and `f_readdir` lists one directory,
//! optionally filtering each entry through a callback that `readdir_checkitem`
//! evaluates (so the filter re-enters the evaluator on every name).
//!
//! Original: `src/nvim/eval/fs.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    EXPAND_FILES, FAIL, FINDFILE_DIR, FINDFILE_FILE, NUL, OK, WILD_ALL, WILD_ALL_KEEP,
    WILD_ALLLINKS, WILD_ICASE, WILD_IGNORE_COMPLETESLASH, WILD_KEEP_ALL, WILD_SILENT, WILD_USE_NL,
    XP_PREFIX_NONE, false_0, kDirectionNotSet, true_0,
};
use crate::src::nvim::cmdexpand::{ExpandCleanup, ExpandInit, ExpandOne, globpath};
use crate::src::nvim::eval::eval_expr_typval;
use crate::src::nvim::eval::typval::tv_list_set_ret;
use crate::src::nvim::eval::typval::{
    tv_clear, tv_get_number_chk, tv_get_string, tv_get_string_buf_chk, tv_list_alloc_ret,
    tv_list_append_string,
};
use crate::src::nvim::eval::vars::{prepare_vimvar, restore_vimvar, set_vim_var_string};
use crate::src::nvim::file_search::{find_file_in_path_option, vim_findfile_cleanup};
use crate::src::nvim::fileio::readdir_core;
use crate::src::nvim::garray::{ga_clear_strings, ga_concat_strings, ga_init};
use crate::src::nvim::main::{curbuf, p_path, p_wic};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::os::libc::strlen;
use crate::src::nvim::types::{
    EvalFuncData, VAR_LIST, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, VV_VAL, expand_T, garray_T,
    kListLenUnknown, list_T, pos_T, ptrdiff_t, sctx_T, size_t, ssize_t, typval_T,
    typval_vval_union, varnumber_T,
};

unsafe extern "C" fn findfilendir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut find_what: ::core::ffi::c_int,
) {
    unsafe {
        let mut fresult: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut path: *mut ::core::ffi::c_char =
            if *(*curbuf.get()).b_p_path as ::core::ffi::c_int == NUL {
                p_path.get()
            } else {
                (*curbuf.get()).b_p_path
            };
        let mut count: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut first: bool = true_0 != 0;
        let mut error: bool = false_0 != 0;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        (*rettv).v_type = VAR_STRING;
        let mut fname: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut pathbuf: [::core::ffi::c_char; 65] = [0; 65];
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut p: *const ::core::ffi::c_char = tv_get_string_buf_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut pathbuf as *mut ::core::ffi::c_char,
            );
            if p.is_null() {
                error = true_0 != 0;
            } else {
                if *p as ::core::ffi::c_int != NUL {
                    path = p as *mut ::core::ffi::c_char;
                }
                if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    count = tv_get_number_chk(
                        argvars.offset(2 as ::core::ffi::c_int as isize),
                        &raw mut error,
                    ) as ::core::ffi::c_int;
                }
            }
        }
        if count < 0 as ::core::ffi::c_int {
            tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
        }
        if *fname as ::core::ffi::c_int != NUL && !error {
            let mut file_to_find: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut search_ctx: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            loop {
                if (*rettv).v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*rettv).v_type as ::core::ffi::c_uint
                        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    xfree(fresult as *mut ::core::ffi::c_void);
                }
                fresult = find_file_in_path_option(
                    if first as ::core::ffi::c_int != 0 {
                        fname as *mut ::core::ffi::c_char
                    } else {
                        ::core::ptr::null_mut::<::core::ffi::c_char>()
                    },
                    if first as ::core::ffi::c_int != 0 {
                        strlen(fname)
                    } else {
                        0 as size_t
                    },
                    0 as ::core::ffi::c_int,
                    first,
                    path,
                    find_what,
                    (*curbuf.get()).b_ffname,
                    (if find_what == FINDFILE_DIR as ::core::ffi::c_int {
                        c"".as_ptr()
                    } else {
                        (*curbuf.get()).b_p_sua as *const ::core::ffi::c_char
                    }) as *mut ::core::ffi::c_char,
                    &raw mut file_to_find,
                    &raw mut search_ctx,
                );
                first = false_0 != 0;
                if !fresult.is_null()
                    && (*rettv).v_type as ::core::ffi::c_uint
                        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    tv_list_append_string((*rettv).vval.v_list, fresult, -1 as ssize_t);
                }
                if !(((*rettv).v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
                    || {
                        count -= 1;
                        count > 0 as ::core::ffi::c_int
                    })
                    && !fresult.is_null())
                {
                    break;
                }
            }
            xfree(file_to_find as *mut ::core::ffi::c_void);
            vim_findfile_cleanup(search_ctx as *mut ::core::ffi::c_void);
        }
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*rettv).vval.v_string = fresult;
        }
    }
}

pub unsafe extern "C" fn f_finddir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        findfilendir(argvars, rettv, FINDFILE_DIR as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn f_findfile(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        findfilendir(argvars, rettv, FINDFILE_FILE as ::core::ffi::c_int);
    }
}

pub unsafe extern "C" fn f_glob(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut options: ::core::ffi::c_int =
            WILD_SILENT as ::core::ffi::c_int | WILD_USE_NL as ::core::ffi::c_int;
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
        let mut error: bool = false_0 != 0;
        (*rettv).v_type = VAR_STRING;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_get_number_chk(
                argvars.offset(1 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0
            {
                options |= WILD_KEEP_ALL as ::core::ffi::c_int;
            }
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if tv_get_number_chk(
                    argvars.offset(2 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) != 0
                {
                    tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
                }
                if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    && tv_get_number_chk(
                        argvars.offset(3 as ::core::ffi::c_int as isize),
                        &raw mut error,
                    ) != 0
                {
                    options |= WILD_ALLLINKS as ::core::ffi::c_int;
                }
            }
        }
        if !error {
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
                    tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                        as *mut ::core::ffi::c_char,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    options,
                    WILD_ALL as ::core::ffi::c_int,
                );
            } else {
                ExpandOne(
                    &raw mut xpc,
                    tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                        as *mut ::core::ffi::c_char,
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
        };
    }
}

pub unsafe extern "C" fn f_globpath(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut flags: ::core::ffi::c_int = WILD_IGNORE_COMPLETESLASH as ::core::ffi::c_int;
        let mut error: bool = false_0 != 0;
        (*rettv).v_type = VAR_STRING;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            if tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            ) != 0
            {
                flags |= WILD_KEEP_ALL as ::core::ffi::c_int;
            }
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                if tv_get_number_chk(
                    argvars.offset(3 as ::core::ffi::c_int as isize),
                    &raw mut error,
                ) != 0
                {
                    tv_list_set_ret(rettv, ::core::ptr::null_mut::<list_T>());
                }
                if (*argvars.offset(4 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                    && tv_get_number_chk(
                        argvars.offset(4 as ::core::ffi::c_int as isize),
                        &raw mut error,
                    ) != 0
                {
                    flags |= WILD_ALLLINKS as ::core::ffi::c_int;
                }
            }
        }
        let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
        let file: *const ::core::ffi::c_char = tv_get_string_buf_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut buf1 as *mut ::core::ffi::c_char,
        );
        if !file.is_null() && !error {
            let mut ga: garray_T = garray_T {
                ga_len: 0,
                ga_maxlen: 0,
                ga_itemsize: 0,
                ga_growsize: 0,
                ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
            };
            ga_init(
                &raw mut ga,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                10 as ::core::ffi::c_int,
            );
            globpath(
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize))
                    as *mut ::core::ffi::c_char,
                file as *mut ::core::ffi::c_char,
                &raw mut ga,
                flags,
                false_0 != 0,
            );
            if (*rettv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*rettv).vval.v_string = ga_concat_strings(&raw mut ga, c"\n".as_ptr());
            } else {
                tv_list_alloc_ret(rettv, ga.ga_len as ptrdiff_t);
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < ga.ga_len {
                    tv_list_append_string(
                        (*rettv).vval.v_list,
                        *(ga.ga_data as *mut *const ::core::ffi::c_char).offset(i as isize),
                        -1 as ssize_t,
                    );
                    i += 1;
                }
            }
            ga_clear_strings(&raw mut ga);
        } else {
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        };
    }
}

unsafe extern "C" fn readdir_checkitem(
    mut context: *mut ::core::ffi::c_void,
    mut name: *const ::core::ffi::c_char,
) -> varnumber_T {
    unsafe {
        let mut expr: *mut typval_T = context as *mut typval_T;
        let mut argv: [typval_T; 2] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 2];
        let mut retval: varnumber_T = 0 as varnumber_T;
        let mut error: bool = false_0 != 0;
        if (*expr).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return 1 as varnumber_T;
        }
        let mut save_val: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        prepare_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
        set_vim_var_string(VV_VAL, name, -1 as ptrdiff_t);
        argv[0 as ::core::ffi::c_int as usize].v_type = VAR_STRING;
        argv[0 as ::core::ffi::c_int as usize].vval.v_string = name as *mut ::core::ffi::c_char;
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        if eval_expr_typval(
            expr,
            false_0 != 0,
            &raw mut argv as *mut typval_T,
            1 as ::core::ffi::c_int,
            &raw mut rettv,
        ) != FAIL
        {
            retval = tv_get_number_chk(&raw mut rettv, &raw mut error);
            if error {
                retval = -1 as varnumber_T;
            }
            tv_clear(&raw mut rettv);
        }
        set_vim_var_string(
            VV_VAL,
            ::core::ptr::null::<::core::ffi::c_char>(),
            0 as ptrdiff_t,
        );
        restore_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
        return retval;
    }
}

pub unsafe extern "C" fn f_readdir(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        tv_list_alloc_ret(rettv, kListLenUnknown as ::core::ffi::c_int as ptrdiff_t);
        let mut path: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut expr: *mut typval_T = argvars.offset(1 as ::core::ffi::c_int as isize);
        let mut ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut ret: ::core::ffi::c_int = readdir_core(
            &raw mut ga,
            path,
            expr as *mut ::core::ffi::c_void,
            Some(
                readdir_checkitem
                    as unsafe extern "C" fn(
                        *mut ::core::ffi::c_void,
                        *const ::core::ffi::c_char,
                    ) -> varnumber_T,
            ),
        );
        if ret == OK && ga.ga_len > 0 as ::core::ffi::c_int {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < ga.ga_len {
                let mut p: *const ::core::ffi::c_char =
                    *(ga.ga_data as *mut *const ::core::ffi::c_char).offset(i as isize);
                tv_list_append_string((*rettv).vval.v_list, p, -1 as ssize_t);
                i += 1;
            }
        }
        ga_clear_strings(&raw mut ga);
    }
}
