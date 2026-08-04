//! The Vimscript face: `getcompletion()`, `getcompletiontype()`,
//! `cmdcomplete_info()`.
//!
//! [`f_getcompletion`] runs the whole classify-then-expand pipeline against a
//! string instead of the real command line, which is what makes it the
//! completion layer's differential oracle.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn f_getcompletion(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
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
        let mut filtered: bool = false_0 != 0;
        let mut options: ::core::ffi::c_int =
            WILD_SILENT | WILD_USE_NL | WILD_ADD_SLASH | WILD_NO_BEEP | WILD_HOME_REPLACE;
        if tv_check_for_string_arg(argvars, 1 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let type_0: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(1 as ::core::ffi::c_int as isize));
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            filtered = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<bool>(),
            ) != 0;
        }
        if p_wic.get() != 0 {
            options |= WILD_ICASE;
        }
        if !filtered {
            options |= WILD_KEEP_ALL;
        }
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        let pattern: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut pattern_start: *const ::core::ffi::c_char = pattern;
        if strcmp(type_0, b"cmdline\0".as_ptr() as *const ::core::ffi::c_char)
            == 0 as ::core::ffi::c_int
        {
            let cmdline_len: ::core::ffi::c_int = strlen(pattern) as ::core::ffi::c_int;
            set_cmd_context(
                &raw mut xpc,
                pattern as *mut ::core::ffi::c_char,
                cmdline_len,
                cmdline_len,
                false_0,
            );
            pattern_start = xpc.xp_pattern;
            xpc.xp_pattern_len = strlen(xpc.xp_pattern);
            xpc.xp_col = cmdline_len;
        } else {
            ExpandInit(&raw mut xpc);
            xpc.xp_pattern = pattern as *mut ::core::ffi::c_char;
            xpc.xp_pattern_len = strlen(xpc.xp_pattern);
            xpc.xp_line = pattern as *mut ::core::ffi::c_char;
            xpc.xp_context = cmdcomplete_str_to_type(type_0);
            match xpc.xp_context {
                0 => {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        type_0,
                    );
                    return;
                }
                30 => {
                    if strncmp(
                        type_0,
                        b"custom,\0".as_ptr() as *const ::core::ffi::c_char,
                        7 as size_t,
                    ) != 0 as ::core::ffi::c_int
                    {
                        semsg(
                            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                            type_0,
                        );
                        return;
                    }
                    xpc.xp_arg =
                        type_0.offset(7 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char;
                }
                31 => {
                    if strncmp(
                        type_0,
                        b"customlist,\0".as_ptr() as *const ::core::ffi::c_char,
                        11 as size_t,
                    ) != 0 as ::core::ffi::c_int
                    {
                        semsg(
                            gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                            type_0,
                        );
                        return;
                    }
                    xpc.xp_arg = type_0.offset(11 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_char;
                }
                11 => {
                    set_context_in_menu_cmd(
                        &raw mut xpc,
                        b"menu\0".as_ptr() as *const ::core::ffi::c_char,
                        xpc.xp_pattern,
                        false_0 != 0,
                    );
                    xpc.xp_pattern_len = xpc
                        .xp_pattern_len
                        .wrapping_sub(xpc.xp_pattern.offset_from(pattern_start) as size_t);
                }
                34 => {
                    set_context_in_sign_cmd(&raw mut xpc, xpc.xp_pattern);
                    xpc.xp_pattern_len = xpc
                        .xp_pattern_len
                        .wrapping_sub(xpc.xp_pattern.offset_from(pattern_start) as size_t);
                }
                51 => {
                    set_context_in_runtime_cmd(&raw mut xpc, xpc.xp_pattern);
                    xpc.xp_pattern_len = xpc
                        .xp_pattern_len
                        .wrapping_sub(xpc.xp_pattern.offset_from(pattern_start) as size_t);
                }
                57 => {
                    let mut context: ::core::ffi::c_int = EXPAND_SHELLCMDLINE;
                    set_context_for_wildcard_arg(
                        ::core::ptr::null_mut::<exarg_T>(),
                        xpc.xp_pattern,
                        false_0 != 0,
                        &raw mut xpc,
                        &raw mut context,
                    );
                    xpc.xp_pattern_len = xpc
                        .xp_pattern_len
                        .wrapping_sub(xpc.xp_pattern.offset_from(pattern_start) as size_t);
                }
                59 => {
                    filetype_expand_what.set(EXP_FILETYPECMD_ALL);
                }
                _ => {}
            }
        }
        if xpc.xp_context == EXPAND_LUA {
            xpc.xp_col = strlen(xpc.xp_line) as ::core::ffi::c_int;
            nlua_expand_pat(&raw mut xpc);
            xpc.xp_pattern_len = xpc
                .xp_pattern_len
                .wrapping_sub(xpc.xp_pattern.offset_from(pattern_start) as size_t);
        }
        let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if cmdline_fuzzy_completion_supported(&raw mut xpc) {
            pat = xmemdupz(
                xpc.xp_pattern as *const ::core::ffi::c_void,
                xpc.xp_pattern_len,
            ) as *mut ::core::ffi::c_char;
        } else {
            pat = addstar(xpc.xp_pattern, xpc.xp_pattern_len, xpc.xp_context);
        }
        ExpandOne(
            &raw mut xpc,
            pat,
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            options,
            WILD_ALL_KEEP,
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
        xfree(pat as *mut ::core::ffi::c_void);
        ExpandCleanup(&raw mut xpc);
    }
}

pub unsafe extern "C" fn f_getcompletiontype(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
            return;
        }
        let mut pat: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
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
        let mut cmdline_len: ::core::ffi::c_int = strlen(pat) as ::core::ffi::c_int;
        set_cmd_context(
            &raw mut xpc,
            pat as *mut ::core::ffi::c_char,
            cmdline_len,
            cmdline_len,
            false_0,
        );
        (*rettv).vval.v_string = cmdcomplete_type_to_str(xpc.xp_context, xpc.xp_arg);
        ExpandCleanup(&raw mut xpc);
    }
}

pub unsafe extern "C" fn f_cmdcomplete_info(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut ccline: *mut CmdlineInfo = get_cmdline_info();
        tv_dict_alloc_ret(rettv);
        if ccline.is_null() || (*ccline).xpc.is_null() || (*(*ccline).xpc).xp_files.is_null() {
            return;
        }
        let mut retdict: *mut dict_T = (*rettv).vval.v_dict;
        let mut ret: ::core::ffi::c_int = tv_dict_add_str(
            retdict,
            b"cmdline_orig\0".as_ptr() as *const ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 13]>().wrapping_sub(1 as size_t),
            cmdline_orig.get(),
        );
        if ret == OK {
            ret = tv_dict_add_nr(
                retdict,
                b"pum_visible\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
                pum_visible() as varnumber_T,
            );
        }
        if ret == OK {
            ret = tv_dict_add_nr(
                retdict,
                b"selected\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 9]>().wrapping_sub(1 as size_t),
                (*(*ccline).xpc).xp_selected as varnumber_T,
            );
        }
        if ret == OK {
            let mut li: *mut list_T = tv_list_alloc((*(*ccline).xpc).xp_numfiles as ptrdiff_t);
            ret = tv_dict_add_list(
                retdict,
                b"matches\0".as_ptr() as *const ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
                li,
            );
            let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while ret == OK && idx < (*(*ccline).xpc).xp_numfiles {
                tv_list_append_string(
                    li,
                    *(*(*ccline).xpc).xp_files.offset(idx as isize),
                    -1 as ssize_t,
                );
                idx += 1;
            }
        }
    }
}
