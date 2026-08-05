use crate::src::nvim::ascii::{ascii_isdigit, ascii_iswhite, ascii_iswhite_nl_or_nul};
use crate::src::nvim::autocmd::{EVENT_FUNCUNDEFINED, apply_autocmds};
use crate::src::nvim::charset::{getdigits, skiptowhite, skipwhite, vim_strsize};
use crate::src::nvim::debugger::{dbg_breakpoint, dbg_find_breakpoint, has_profiling};
use crate::src::nvim::eval::encode::{encode_tv2echo, encode_tv2string};
use crate::src::nvim::eval::funcs::{
    call_internal_func, call_internal_method, check_internal_func, find_internal_func,
};
use crate::src::nvim::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_item_alloc, tv_dict_item_alloc_len,
    tv_dict_item_remove, tv_dict_unref, tv_get_number_chk, tv_list_append, tv_list_init_static,
    value_check_lock,
};
use crate::src::nvim::eval::typval::{tv_is_func, tv_list_set_lock};
use crate::src::nvim::eval::vars::{
    find_var, find_var_ht, find_var_in_ht, get_vim_var_nr, init_var_dict, list_hashtable_vars,
    skip_var_list, vars_clear, vars_clear_ext,
};
use crate::src::nvim::eval::{
    callback_call, check_luafunc_name, clear_evalarg, clear_lval, eval_isnamec, eval_isnamec1,
    eval_lavars_used, eval0, eval1, fill_evalarg_from_eap, find_name_end, garbage_collect,
    get_id_len, get_lval, handle_subscript, is_luafunc, last_set_msg, partial_name, partial_unref,
    set_ref_in_ht, set_ref_in_item, set_ref_in_list_items, skip_expr,
};
use crate::src::nvim::ex_docmd::{check_nextcmd, checkforcmd, do_cmdline, ends_excmd, skip_range};
use crate::src::nvim::ex_eval::{
    aborted_in_try, aborting, cleanup_conditionals, exception_state_clear, exception_state_restore,
    exception_state_save, report_make_pending, update_force_abort,
};
use crate::src::nvim::ex_getln::{
    getcmdline, ui_ext_cmdline_block_append, ui_ext_cmdline_block_leave,
};
use crate::src::nvim::garray::{ga_append_via_ptr, ga_clear, ga_clear_strings, ga_grow, ga_init};
use crate::src::nvim::getchar::{restoreRedobuff, saveRedobuff};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::hashtab::hash_removed;
use crate::src::nvim::hashtab::{hash_add, hash_find, hash_find_len, hash_init, hash_remove};
use crate::src::nvim::insexpand::ins_compl_active;
use crate::src::nvim::keycodes::K_SPECIAL;
use crate::src::nvim::lua::executor::{
    api_free_luaref, nlua_set_sctx, nlua_typval_call, typval_exec_lua_callable,
};
use crate::src::nvim::main::{
    EVALARG_EVALUATE, IObuff, KeyTyped, RedrawingDisabled, Rows, cmdline_row, curbuf, current_sctx,
    curwin, debug_backtrace_level, debug_tick, did_emsg, did_throw, do_profiling, e_dictkey,
    e_invarg2, e_invexpr2, e_invrange, e_missingparen, e_not_callable_type_str,
    e_str_not_inside_function, e_toofewarg, e_toomanyarg, e_trailing_arg, e_unknown_function_str,
    e_usingsid, emsg_off, emsg_severe, emsg_skip, ex_nesting_level, got_int, lines_left, msg_row,
    msg_scroll, need_wait_return, no_wait_return, p_ic, p_mfd, p_verbose, sandbox, trylevel,
    want_garbage_collect,
};
use crate::src::nvim::mbyte::mb_strnicmp;
use crate::src::nvim::memory::{
    xcalloc, xfree, xmalloc, xmallocz, xmemcpyz, xmemdupz, xmemrchr, xstrdup, xstrlcpy,
};
use crate::src::nvim::message::{
    emsg, iemsg, internal_error, message_filtered, msg_clr_eos, msg_ext_set_kind, msg_outnum,
    msg_prt_line, msg_putchar, msg_puts, msg_start, semsg, smsg, swmsg, trunc_string,
    verbose_enter_scroll, verbose_leave_scroll,
};
use crate::src::nvim::os::input::line_breakcheck;
use crate::src::nvim::os::libc::{
    __assert_fail, __ctype_b_loc, abort, gettext, memchr, memcmp, memcpy, memmove, memset,
    snprintf, strchr, strcmp, strcpy, strlen, strncmp, strstr,
};
use crate::src::nvim::path::path_fnamecmp;
use crate::src::nvim::profile::{
    func_do_profile, func_line_end, func_line_start, prof_def_func, profile_add, profile_end,
    profile_self, profile_start, profile_sub_wait, profile_zero, script_prof_restore,
    script_prof_save,
};
use crate::src::nvim::regexp::{RE_MAGIC, skip_regexp};
use crate::src::nvim::runtime::{
    autoload_name, estack_pop, estack_push_ufunc, exestack, get_sourced_lnum, script_autoload,
    script_items,
};
use crate::src::nvim::search::{restore_search_patterns, save_search_patterns};
use crate::src::nvim::strings::{concat_str, vim_strchr, xstrnsave};
use crate::src::nvim::types::ui::kUICmdline;
use crate::src::nvim::types::{
    CMD_defer, Callback, EvalFuncDef, LuaRef, OptInt, String_0, VAR_DEF_SCOPE, VAR_DICT, VAR_FIXED,
    VAR_FUNC, VAR_LIST, VAR_NUMBER, VAR_PARTIAL, VAR_SCOPE, VAR_SHORT_LEN, VAR_STRING, VAR_UNKNOWN,
    VAR_UNLOCKED, VV_TESTING, blob_T, buf_T, buffblock, buffblock_T, buffheader_T, colnr_T,
    cstack_T, dict_T, dictitem_T, estack_T, evalarg_T, exarg_T, except_T, exception_state_T,
    expand_T, funccal_entry_T, funccall_S_fc_fixvar as C2Rust_Unnamed_7, funccall_T, funcdict_T,
    funcexe_T, garray_T, hashitem_T, hashtab_T, ht_stack_T, intmax_t, key_extra, linenr_T, list_T,
    list_stack_T, listitem_T, lval_T, partial_T, proftime_T, regmatch_T, regprog_T, save_redo_T,
    sctx_T, size_t, typval_T, typval_vval_union, ufunc_T, uint8_t, varnumber_T,
};
use crate::src::nvim::ui::ui_has;
unsafe extern "C" {
    fn vim_regcomp(
        expr_arg: *const ::core::ffi::c_char,
        re_flags: ::core::ffi::c_int,
    ) -> *mut regprog_T;
    fn vim_regfree(prog: *mut regprog_T);
    fn vim_regexec(rmp: *mut regmatch_T, line: *const ::core::ffi::c_char, col: colnr_T) -> bool;
}
pub type C2Rust_Unnamed = ::core::ffi::c_uint;
pub const _ISdigit: C2Rust_Unnamed = 2048;
pub type C2Rust_Unnamed_14 = ::core::ffi::c_uint;
pub const DO_NOT_FREE_CNT: C2Rust_Unnamed_14 = 1073741823;
pub type C2Rust_Unnamed_15 = ::core::ffi::c_uint;
pub const DI_FLAGS_FIX: C2Rust_Unnamed_15 = 4;
pub const DI_FLAGS_RO_SBX: C2Rust_Unnamed_15 = 2;
pub const DI_FLAGS_RO: C2Rust_Unnamed_15 = 1;
pub type C2Rust_Unnamed_16 = ::core::ffi::c_uint;
pub const MAX_FUNC_ARGS: C2Rust_Unnamed_16 = 20;
pub type C2Rust_Unnamed_17 = ::core::ffi::c_uint;
pub type C2Rust_Unnamed_18 = ::core::ffi::c_uint;
pub const FIXVAR_CNT: C2Rust_Unnamed_18 = 12;
pub type C2Rust_Unnamed_19 = ::core::ffi::c_int;
pub const EXPAND_USER_FUNC: C2Rust_Unnamed_19 = 19;
pub type C2Rust_Unnamed_21 = ::core::ffi::c_uint;
pub const CSTP_RETURN: C2Rust_Unnamed_21 = 24;
pub type C2Rust_Unnamed_22 = ::core::ffi::c_uint;
pub const TFN_NO_DEREF: C2Rust_Unnamed_22 = 8;
pub const TFN_NO_AUTOLOAD: C2Rust_Unnamed_22 = 4;
pub const TFN_QUIET: C2Rust_Unnamed_22 = 2;
pub const TFN_INT: C2Rust_Unnamed_22 = 1;
pub type C2Rust_Unnamed_23 = ::core::ffi::c_uint;
pub const GLV_READ_ONLY: C2Rust_Unnamed_23 = 16;
pub type C2Rust_Unnamed_24 = ::core::ffi::c_uint;
pub const EVAL_EVALUATE: C2Rust_Unnamed_24 = 1;
pub type C2Rust_Unnamed_25 = ::core::ffi::c_uint;
pub const FCERR_DELETED: C2Rust_Unnamed_25 = 7;
pub const FCERR_NONE: C2Rust_Unnamed_25 = 5;
pub const FCERR_DICT: C2Rust_Unnamed_25 = 4;
pub const FCERR_SCRIPT: C2Rust_Unnamed_25 = 3;
pub const FCERR_TOOFEW: C2Rust_Unnamed_25 = 2;
pub const FCERR_TOOMANY: C2Rust_Unnamed_25 = 1;
pub const FCERR_UNKNOWN: C2Rust_Unnamed_25 = 0;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct defer_T {
    pub dr_name: *mut ::core::ffi::c_char,
    pub dr_argvars: [typval_T; 21],
    pub dr_argcount: ::core::ffi::c_int,
}
pub const DOCMD_REPEAT: C2Rust_Unnamed_27 = 4;
pub const DOCMD_VERBOSE: C2Rust_Unnamed_27 = 1;
pub const DOCMD_NOWAIT: C2Rust_Unnamed_27 = 2;
pub const KE_SNR: key_extra = 82;
pub type C2Rust_Unnamed_27 = ::core::ffi::c_uint;
pub const SIZE_MAX: ::core::ffi::c_ulong = 18446744073709551615 as ::core::ffi::c_ulong;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const LUA_NOREF: ::core::ffi::c_int = -2 as ::core::ffi::c_int;
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const GA_EMPTY_INIT_VALUE: garray_T = garray_T {
    ga_len: 0 as ::core::ffi::c_int,
    ga_maxlen: 0 as ::core::ffi::c_int,
    ga_itemsize: 0 as ::core::ffi::c_int,
    ga_growsize: 1 as ::core::ffi::c_int,
    ga_data: NULL,
};
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const NOTDONE: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FNE_INCL_BR: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FNE_CHECK_START: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const AUTOLOAD_CHAR: ::core::ffi::c_int = '#' as ::core::ffi::c_int;
pub const TV_CSTRING: ::core::ffi::c_ulong = SIZE_MAX.wrapping_sub(1 as ::core::ffi::c_ulong);
static func_hashtab: GlobalCell<hashtab_T> = GlobalCell::new(hashtab_T {
    ht_mask: 0,
    ht_used: 0,
    ht_filled: 0,
    ht_changed: 0,
    ht_locked: 0,
    ht_array: ::core::ptr::null_mut::<hashitem_T>(),
    ht_smallarray: [hashitem_T {
        hi_hash: 0,
        hi_key: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    }; 16],
});
static funcargs: GlobalCell<garray_T> = GlobalCell::new(GA_EMPTY_INIT_VALUE);
static current_funccal: GlobalCell<*mut funccall_T> =
    GlobalCell::new(::core::ptr::null_mut::<funccall_T>());
static previous_funccal: GlobalCell<*mut funccall_T> =
    GlobalCell::new(::core::ptr::null_mut::<funccall_T>());
static e_funcexts: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"E122: Function %s already exists, add ! to replace it\0".as_ptr()
        as *const ::core::ffi::c_char,
);
static e_funcdict: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"E717: Dictionary entry already exists\0".as_ptr() as *const ::core::ffi::c_char,
);
static e_funcref: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E718: Funcref required\0".as_ptr() as *const ::core::ffi::c_char);
static e_nofunc: GlobalCell<*const ::core::ffi::c_char> =
    GlobalCell::new(b"E130: Unknown function: %s\0".as_ptr() as *const ::core::ffi::c_char);
static e_function_list_was_modified: GlobalCell<[::core::ffi::c_char; 33]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
            *b"E454: Function list was modified\0",
        )
    });
static e_function_nesting_too_deep: GlobalCell<[::core::ffi::c_char; 33]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 33], [::core::ffi::c_char; 33]>(
            *b"E1058: Function nesting too deep\0",
        )
    });
static e_no_white_space_allowed_before_str_str: GlobalCell<[::core::ffi::c_char; 46]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 46], [::core::ffi::c_char; 46]>(
            *b"E1068: No white space allowed before '%s': %s\0",
        )
    });
static e_missing_heredoc_end_marker_str: GlobalCell<[::core::ffi::c_char; 38]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 38], [::core::ffi::c_char; 38]>(
            *b"E1145: Missing heredoc end marker: %s\0",
        )
    });
static e_cannot_use_partial_with_dictionary_for_defer: GlobalCell<[::core::ffi::c_char; 55]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 55], [::core::ffi::c_char; 55]>(
            *b"E1300: Cannot use a partial with dictionary for :defer\0",
        )
    });
pub unsafe extern "C" fn func_init() {
    hash_init(func_hashtab.ptr());
}
pub unsafe extern "C" fn func_tbl_get() -> *mut hashtab_T {
    return func_hashtab.ptr();
}
unsafe extern "C" fn one_function_arg(
    mut arg: *mut ::core::ffi::c_char,
    mut newargs: *mut garray_T,
    mut skip: bool,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = arg;
    while *p as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
        && *p as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        || *p as ::core::ffi::c_uint >= 'a' as ::core::ffi::c_uint
            && *p as ::core::ffi::c_uint <= 'z' as ::core::ffi::c_uint
        || ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
        || *p as ::core::ffi::c_int == '_' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    if arg == p
        || *(*__ctype_b_loc()).offset(*arg as uint8_t as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
        || p.offset_from(arg) == 9 as isize
            && strncmp(
                arg,
                b"firstline\0".as_ptr() as *const ::core::ffi::c_char,
                9 as size_t,
            ) == 0 as ::core::ffi::c_int
        || p.offset_from(arg) == 8 as isize
            && strncmp(
                arg,
                b"lastline\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
    {
        if !skip {
            semsg(
                gettext(b"E125: Illegal argument: %s\0".as_ptr() as *const ::core::ffi::c_char),
                arg,
            );
        }
        return arg;
    }
    if !newargs.is_null() {
        ga_grow(newargs, 1 as ::core::ffi::c_int);
        let mut c: uint8_t = *p as uint8_t;
        *p = NUL as ::core::ffi::c_char;
        let mut arg_copy: *mut ::core::ffi::c_char = xstrdup(arg);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*newargs).ga_len {
            if strcmp(
                *((*newargs).ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize),
                arg_copy,
            ) == 0 as ::core::ffi::c_int
            {
                semsg(
                    gettext(b"E853: Duplicate argument name: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    arg_copy,
                );
                xfree(arg_copy as *mut ::core::ffi::c_void);
                return arg;
            }
            i += 1;
        }
        *((*newargs).ga_data as *mut *mut ::core::ffi::c_char).offset((*newargs).ga_len as isize) =
            arg_copy;
        (*newargs).ga_len += 1;
        *p = c as ::core::ffi::c_char;
    }
    return p;
}
unsafe extern "C" fn get_function_args(
    mut argp: *mut *mut ::core::ffi::c_char,
    mut endchar: ::core::ffi::c_char,
    mut newargs: *mut garray_T,
    mut varargs: *mut ::core::ffi::c_int,
    mut default_args: *mut garray_T,
    mut skip: bool,
) -> ::core::ffi::c_int {
    let mut mustend: bool = false_0 != 0;
    let mut arg: *mut ::core::ffi::c_char = *argp;
    let mut p: *mut ::core::ffi::c_char = arg;
    if !newargs.is_null() {
        ga_init(
            newargs,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            3 as ::core::ffi::c_int,
        );
    }
    if !default_args.is_null() {
        ga_init(
            default_args,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
            3 as ::core::ffi::c_int,
        );
    }
    if !varargs.is_null() {
        *varargs = false_0;
    }
    let mut any_default: bool = false_0 != 0;
    '_err_ret: {
        while *p as ::core::ffi::c_int != endchar as ::core::ffi::c_int {
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '.' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
                && *p.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '.' as ::core::ffi::c_int
            {
                if !varargs.is_null() {
                    *varargs = true_0;
                }
                p = p.offset(3 as ::core::ffi::c_int as isize);
                mustend = true_0 != 0;
            } else {
                arg = p;
                p = one_function_arg(p, newargs, skip);
                if p == arg {
                    break;
                }
                if *skipwhite(p) as ::core::ffi::c_int == '=' as ::core::ffi::c_int
                    && !default_args.is_null()
                {
                    let mut rettv: typval_T = typval_T {
                        v_type: VAR_UNKNOWN,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    any_default = true_0 != 0;
                    p = skipwhite(p).offset(1 as ::core::ffi::c_int as isize);
                    p = skipwhite(p);
                    let mut expr: *mut ::core::ffi::c_char = p;
                    if eval1(
                        &raw mut p,
                        &raw mut rettv,
                        ::core::ptr::null_mut::<evalarg_T>(),
                    ) != FAIL
                    {
                        ga_grow(default_args, 1 as ::core::ffi::c_int);
                        while p > expr
                            && ascii_iswhite(
                                *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            ) as ::core::ffi::c_int
                                != 0
                        {
                            p = p.offset(-1);
                        }
                        let mut c: uint8_t = *p as uint8_t;
                        *p = NUL as ::core::ffi::c_char;
                        expr = xstrdup(expr);
                        *((*default_args).ga_data as *mut *mut ::core::ffi::c_char)
                            .offset((*default_args).ga_len as isize) = expr;
                        (*default_args).ga_len += 1;
                        *p = c as ::core::ffi::c_char;
                    } else {
                        mustend = true_0 != 0;
                    }
                } else if any_default {
                    emsg(gettext(
                        b"E989: Non-default argument follows default argument\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                    mustend = true_0 != 0;
                }
                if ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                    && *skipwhite(p) as ::core::ffi::c_int == ',' as ::core::ffi::c_int
                {
                    if !skip {
                        semsg(
                            gettext(
                                (e_no_white_space_allowed_before_str_str.ptr() as *const _)
                                    as *const ::core::ffi::c_char,
                            ),
                            b",\0".as_ptr() as *const ::core::ffi::c_char,
                            p,
                        );
                        break '_err_ret;
                    } else {
                        p = skipwhite(p);
                    }
                }
                if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                    p = p.offset(1);
                } else {
                    mustend = true_0 != 0;
                }
            }
            p = skipwhite(p);
            if !(mustend as ::core::ffi::c_int != 0
                && *p as ::core::ffi::c_int != endchar as ::core::ffi::c_int)
            {
                continue;
            }
            if !skip {
                semsg(
                    gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                    *argp,
                );
            }
            break;
        }
        if *p as ::core::ffi::c_int == endchar as ::core::ffi::c_int {
            p = p.offset(1);
            *argp = p;
            return OK;
        }
    }
    if !newargs.is_null() {
        ga_clear_strings(newargs);
    }
    if !default_args.is_null() {
        ga_clear_strings(default_args);
    }
    return FAIL;
}
unsafe extern "C" fn register_closure(mut fp: *mut ufunc_T) {
    if (*fp).uf_scoped == current_funccal.get() {
        return;
    }
    funccal_unref((*fp).uf_scoped, fp, false_0 != 0);
    (*fp).uf_scoped = current_funccal.get();
    (*current_funccal.get()).fc_refcount += 1;
    ga_grow(
        &raw mut (*current_funccal.get()).fc_ufuncs,
        1 as ::core::ffi::c_int,
    );
    let c2rust_fresh1 = (*current_funccal.get()).fc_ufuncs.ga_len;
    (*current_funccal.get()).fc_ufuncs.ga_len = (*current_funccal.get()).fc_ufuncs.ga_len + 1;
    let c2rust_lvalue_ptr = &raw mut *((*current_funccal.get()).fc_ufuncs.ga_data
        as *mut *mut ufunc_T)
        .offset(c2rust_fresh1 as isize);
    *c2rust_lvalue_ptr = fp;
}
static lambda_name: GlobalCell<[::core::ffi::c_char; 73]> = GlobalCell::new([0; 73]);
unsafe extern "C" fn get_lambda_name() -> String_0 {
    static lambda_no: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    (*lambda_no.ptr()) += 1;
    let mut n: ::core::ffi::c_int = snprintf(
        lambda_name.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 73]>(),
        b"<lambda>%d\0".as_ptr() as *const ::core::ffi::c_char,
        lambda_no.get(),
    );
    return String_0 {
        data: lambda_name.ptr() as *mut ::core::ffi::c_char,
        size: if n < 1 as ::core::ffi::c_int {
            0 as size_t
        } else {
            (if n < ::core::mem::size_of::<[::core::ffi::c_char; 73]>() as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
            {
                n
            } else {
                ::core::mem::size_of::<[::core::ffi::c_char; 73]>() as ::core::ffi::c_int
                    - 1 as ::core::ffi::c_int
            }) as size_t
        },
    };
}
unsafe extern "C" fn alloc_ufunc(
    mut name: *const ::core::ffi::c_char,
    mut namelen: size_t,
) -> *mut ufunc_T {
    let mut len: size_t = (240 as size_t)
        .wrapping_add(namelen)
        .wrapping_add(1 as size_t);
    let mut fp: *mut ufunc_T = xcalloc(1 as size_t, len) as *mut ufunc_T;
    xmemcpyz(
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char as *mut ::core::ffi::c_void,
        name as *const ::core::ffi::c_void,
        namelen,
    );
    (*fp).uf_namelen = namelen;
    if *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int == K_SPECIAL
    {
        len = namelen.wrapping_add(3 as size_t);
        (*fp).uf_name_exp = xmalloc(len) as *mut ::core::ffi::c_char;
        snprintf(
            (*fp).uf_name_exp,
            len,
            b"<SNR>%s\0".as_ptr() as *const ::core::ffi::c_char,
            (&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                .offset(3 as ::core::ffi::c_int as isize),
        );
    }
    return fp;
}
pub unsafe extern "C" fn get_lambda_tv(
    mut arg: *mut *mut ::core::ffi::c_char,
    mut rettv: *mut typval_T,
    mut evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let mut start: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let evaluate: bool =
        !evalarg.is_null() && (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int != 0;
    let mut newargs: garray_T = GA_EMPTY_INIT_VALUE;
    let mut pnewargs: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
    let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
    let mut pt: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut varargs: ::core::ffi::c_int = 0;
    let mut old_eval_lavars: *mut bool = eval_lavars_used.get();
    let mut eval_lavars: bool = false_0 != 0;
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut s: *mut ::core::ffi::c_char =
        skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
    let mut ret: ::core::ffi::c_int = get_function_args(
        &raw mut s,
        '-' as ::core::ffi::c_char,
        ::core::ptr::null_mut::<garray_T>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<garray_T>(),
        true_0 != 0,
    );
    if ret == FAIL || *s as ::core::ffi::c_int != '>' as ::core::ffi::c_int {
        return NOTDONE;
    }
    if evaluate {
        pnewargs = &raw mut newargs;
    } else {
        pnewargs = ::core::ptr::null_mut::<garray_T>();
    }
    *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
    ret = get_function_args(
        arg,
        '-' as ::core::ffi::c_char,
        pnewargs,
        &raw mut varargs,
        ::core::ptr::null_mut::<garray_T>(),
        false_0 != 0,
    );
    if !(ret == FAIL || **arg as ::core::ffi::c_int != '>' as ::core::ffi::c_int) {
        if evaluate {
            eval_lavars_used.set(&raw mut eval_lavars);
        }
        *arg = skipwhite((*arg).offset(1 as ::core::ffi::c_int as isize));
        start = *arg;
        ret = skip_expr(arg, evalarg);
        end = *arg;
        if ret != FAIL {
            if !evalarg.is_null() {
                tofree = (*evalarg).eval_tofree;
                (*evalarg).eval_tofree = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            *arg = skipwhite(*arg);
            if **arg as ::core::ffi::c_int != '}' as ::core::ffi::c_int {
                semsg(
                    gettext(b"E451: Expected }: %s\0".as_ptr() as *const ::core::ffi::c_char),
                    *arg,
                );
            } else {
                *arg = (*arg).offset(1);
                if evaluate {
                    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    let mut newlines: garray_T = garray_T {
                        ga_len: 0,
                        ga_maxlen: 0,
                        ga_itemsize: 0,
                        ga_growsize: 0,
                        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
                    };
                    let mut name: String_0 = get_lambda_name();
                    fp = alloc_ufunc(name.data, name.size);
                    pt =
                        xcalloc(1 as size_t, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
                    ga_init(
                        &raw mut newlines,
                        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                    ga_grow(&raw mut newlines, 1 as ::core::ffi::c_int);
                    let mut len: size_t = (end
                        .offset(7 as ::core::ffi::c_int as isize)
                        .offset_from(start)
                        + 1 as isize) as size_t;
                    let mut p: *mut ::core::ffi::c_char = xmalloc(len) as *mut ::core::ffi::c_char;
                    let c2rust_fresh0 = newlines.ga_len;
                    newlines.ga_len = newlines.ga_len + 1;
                    let c2rust_lvalue_ptr = &raw mut *(newlines.ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh0 as isize);
                    *c2rust_lvalue_ptr = p;
                    strcpy(
                        p,
                        b"return \0".as_ptr() as *const ::core::ffi::c_char
                            as *mut ::core::ffi::c_char,
                    );
                    xmemcpyz(
                        p.offset(7 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                        start as *const ::core::ffi::c_void,
                        end.offset_from(start) as size_t,
                    );
                    if strstr(
                        p.offset(7 as ::core::ffi::c_int as isize),
                        b"a:\0".as_ptr() as *const ::core::ffi::c_char,
                    )
                    .is_null()
                    {
                        flags |= FC_NOARGS;
                    }
                    (*fp).uf_refcount = 1 as ::core::ffi::c_int;
                    hash_add(
                        func_hashtab.ptr(),
                        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                    );
                    (*fp).uf_args = newargs;
                    ga_init(
                        &raw mut (*fp).uf_def_args,
                        ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                        1 as ::core::ffi::c_int,
                    );
                    (*fp).uf_lines = newlines;
                    if !(*current_funccal.ptr()).is_null() && eval_lavars as ::core::ffi::c_int != 0
                    {
                        flags |= FC_CLOSURE;
                        register_closure(fp);
                    } else {
                        (*fp).uf_scoped = ::core::ptr::null_mut::<funccall_T>();
                    }
                    if prof_def_func() {
                        func_do_profile(fp);
                    }
                    if sandbox.get() != 0 {
                        flags |= FC_SANDBOX;
                    }
                    (*fp).uf_varargs = true_0;
                    (*fp).uf_flags = flags;
                    (*fp).uf_calls = 0 as ::core::ffi::c_int;
                    (*fp).uf_script_ctx = current_sctx.get();
                    (*fp).uf_script_ctx.sc_lnum = ((*fp).uf_script_ctx.sc_lnum
                        as ::core::ffi::c_int
                        + ((*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum
                            - newlines.ga_len as linenr_T)
                            as ::core::ffi::c_int)
                        as linenr_T;
                    (*pt).pt_func = fp;
                    (*pt).pt_refcount = 1 as ::core::ffi::c_int;
                    (*rettv).vval.v_partial = pt;
                    (*rettv).v_type = VAR_PARTIAL;
                }
                eval_lavars_used.set(old_eval_lavars);
                if !evalarg.is_null() && (*evalarg).eval_tofree.is_null() {
                    (*evalarg).eval_tofree = tofree;
                } else {
                    xfree(tofree as *mut ::core::ffi::c_void);
                }
                return OK;
            }
        }
    }
    ga_clear_strings(&raw mut newargs);
    '_c2rust_label: {
        if fp.is_null() {
        } else {
            __assert_fail(
                b"fp == NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/userfunc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                418 as ::core::ffi::c_uint,
                b"int get_lambda_tv(char **, typval_T *, evalarg_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    xfree(pt as *mut ::core::ffi::c_void);
    if !evalarg.is_null() && (*evalarg).eval_tofree.is_null() {
        (*evalarg).eval_tofree = tofree;
    } else {
        xfree(tofree as *mut ::core::ffi::c_void);
    }
    eval_lavars_used.set(old_eval_lavars);
    return FAIL;
}
pub unsafe extern "C" fn deref_func_name(
    mut name: *const ::core::ffi::c_char,
    mut lenp: *mut ::core::ffi::c_int,
    partialp: *mut *mut partial_T,
    mut no_autoload: bool,
    mut found_var: *mut bool,
) -> *mut ::core::ffi::c_char {
    if !partialp.is_null() {
        *partialp = ::core::ptr::null_mut::<partial_T>();
    }
    let v: *mut dictitem_T = find_var(
        name,
        *lenp as size_t,
        ::core::ptr::null_mut::<*mut hashtab_T>(),
        no_autoload as ::core::ffi::c_int,
    );
    if v.is_null() {
        return name as *mut ::core::ffi::c_char;
    }
    let tv: *mut typval_T = &raw mut (*v).di_tv;
    if !found_var.is_null() {
        *found_var = true_0 != 0;
    }
    if (*tv).v_type as ::core::ffi::c_uint == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        if (*tv).vval.v_string.is_null() {
            *lenp = 0 as ::core::ffi::c_int;
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        *lenp = strlen((*tv).vval.v_string) as ::core::ffi::c_int;
        return (*tv).vval.v_string;
    }
    if (*tv).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let pt: *mut partial_T = (*tv).vval.v_partial;
        if pt.is_null() {
            *lenp = 0 as ::core::ffi::c_int;
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        if !partialp.is_null() {
            *partialp = pt;
        }
        let mut s: *mut ::core::ffi::c_char = partial_name(pt);
        *lenp = strlen(s) as ::core::ffi::c_int;
        return s;
    }
    return name as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn emsg_funcname(
    mut errmsg: *const ::core::ffi::c_char,
    mut name: *const ::core::ffi::c_char,
) {
    let mut p: *mut ::core::ffi::c_char = name as *mut ::core::ffi::c_char;
    if *name.offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int == K_SPECIAL
        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
    {
        p = concat_str(
            b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char,
            name.offset(3 as ::core::ffi::c_int as isize),
        );
    }
    semsg(gettext(errmsg), p);
    if p != name as *mut ::core::ffi::c_char {
        xfree(p as *mut ::core::ffi::c_void);
    }
}
unsafe extern "C" fn get_func_arguments(
    mut arg: *mut *mut ::core::ffi::c_char,
    evalarg: *mut evalarg_T,
    mut partial_argc: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut argcount: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut argp: *mut ::core::ffi::c_char = *arg;
    let mut ret: ::core::ffi::c_int = OK;
    while *argcount < MAX_FUNC_ARGS as ::core::ffi::c_int - partial_argc {
        argp = skipwhite(argp.offset(1 as ::core::ffi::c_int as isize));
        if *argp as ::core::ffi::c_int == ')' as ::core::ffi::c_int
            || *argp as ::core::ffi::c_int == ',' as ::core::ffi::c_int
            || *argp as ::core::ffi::c_int == NUL
        {
            break;
        }
        if eval1(&raw mut argp, argvars.offset(*argcount as isize), evalarg) == FAIL {
            ret = FAIL;
            break;
        } else {
            *argcount += 1;
            if *argp as ::core::ffi::c_int != ',' as ::core::ffi::c_int {
                break;
            }
        }
    }
    argp = skipwhite(argp);
    if *argp as ::core::ffi::c_int == ')' as ::core::ffi::c_int {
        argp = argp.offset(1);
    } else {
        ret = FAIL;
    }
    *arg = argp;
    return ret;
}
pub unsafe extern "C" fn get_func_tv(
    mut name: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
    mut arg: *mut *mut ::core::ffi::c_char,
    evalarg: *mut evalarg_T,
    mut funcexe: *mut funcexe_T,
) -> ::core::ffi::c_int {
    let mut argvars: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let evaluate: bool = if evalarg.is_null() {
        false_0
    } else {
        (*evalarg).eval_flags & EVAL_EVALUATE as ::core::ffi::c_int
    } != 0;
    let mut argp: *mut ::core::ffi::c_char = *arg;
    let mut ret: ::core::ffi::c_int = get_func_arguments(
        &raw mut argp,
        evalarg,
        if (*funcexe).fe_partial.is_null() {
            0 as ::core::ffi::c_int
        } else {
            (*(*funcexe).fe_partial).pt_argc
        },
        &raw mut argvars as *mut typval_T,
        &raw mut argcount,
    );
    '_c2rust_label: {
        if ret == 1 as ::core::ffi::c_int || ret == 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"ret == OK || ret == FAIL\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/userfunc.rs\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                565 as ::core::ffi::c_uint,
                b"int get_func_tv(const char *, int, typval_T *, char **, evalarg_T *const, funcexe_T *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
        }
    };
    if ret == OK {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if get_vim_var_nr(VV_TESTING) != 0 {
            if (*funcargs.ptr()).ga_itemsize == 0 as ::core::ffi::c_int {
                ga_init(
                    funcargs.ptr(),
                    ::core::mem::size_of::<*mut typval_T>() as ::core::ffi::c_int,
                    50 as ::core::ffi::c_int,
                );
            }
            i = 0 as ::core::ffi::c_int;
            while i < argcount {
                ga_grow(funcargs.ptr(), 1 as ::core::ffi::c_int);
                let c2rust_fresh2 = (*funcargs.ptr()).ga_len;
                (*funcargs.ptr()).ga_len = (*funcargs.ptr()).ga_len + 1;
                let c2rust_lvalue_ptr = &raw mut *((*funcargs.ptr()).ga_data as *mut *mut typval_T)
                    .offset(c2rust_fresh2 as isize);
                *c2rust_lvalue_ptr = (&raw mut argvars as *mut typval_T).offset(i as isize);
                i += 1;
            }
        }
        ret = call_func(
            name,
            len,
            rettv,
            argcount,
            &raw mut argvars as *mut typval_T,
            funcexe,
        );
        (*funcargs.ptr()).ga_len -= i;
    } else if !aborting() && evaluate as ::core::ffi::c_int != 0 {
        if argcount == MAX_FUNC_ARGS as ::core::ffi::c_int {
            emsg_funcname(
                b"E740: Too many arguments for function %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name,
            );
        } else {
            emsg_funcname(
                b"E116: Invalid arguments for function %s\0".as_ptr() as *const ::core::ffi::c_char,
                name,
            );
        }
    }
    loop {
        argcount -= 1;
        if argcount < 0 as ::core::ffi::c_int {
            break;
        }
        tv_clear((&raw mut argvars as *mut typval_T).offset(argcount as isize));
    }
    *arg = skipwhite(argp);
    return ret;
}
pub const FLEN_FIXED: ::core::ffi::c_int = 40 as ::core::ffi::c_int;
#[inline(always)]
unsafe extern "C" fn eval_fname_sid(name: *const ::core::ffi::c_char) -> bool {
    return *name as ::core::ffi::c_int == 's' as ::core::ffi::c_int
        || (if (*name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
            < 'a' as ::core::ffi::c_int
            || *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                > 'z' as ::core::ffi::c_int
        {
            *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        } else {
            *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                - ('a' as ::core::ffi::c_int - 'A' as ::core::ffi::c_int)
        }) == 'I' as ::core::ffi::c_int;
}
unsafe extern "C" fn fname_trans_sid(
    name: *const ::core::ffi::c_char,
    fname_buf: *mut ::core::ffi::c_char,
    tofree: *mut *mut ::core::ffi::c_char,
    error: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut script_name: *const ::core::ffi::c_char = name.offset(eval_fname_script(name) as isize);
    if script_name == name {
        return name as *mut ::core::ffi::c_char;
    }
    *fname_buf.offset(0 as ::core::ffi::c_int as isize) = K_SPECIAL as ::core::ffi::c_char;
    *fname_buf.offset(1 as ::core::ffi::c_int as isize) = KS_EXTRA as ::core::ffi::c_char;
    *fname_buf.offset(2 as ::core::ffi::c_int as isize) =
        KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char;
    let mut fname_buflen: size_t = 3 as size_t;
    if !eval_fname_sid(name) {
        *fname_buf.offset(fname_buflen as isize) = NUL as ::core::ffi::c_char;
    } else if (*current_sctx.ptr()).sc_sid <= 0 as ::core::ffi::c_int {
        *error = FCERR_SCRIPT as ::core::ffi::c_int;
    } else {
        fname_buflen = fname_buflen.wrapping_add(snprintf(
            fname_buf.offset(fname_buflen as isize),
            ((FLEN_FIXED + 1 as ::core::ffi::c_int) as size_t).wrapping_sub(fname_buflen),
            b"%d_\0".as_ptr() as *const ::core::ffi::c_char,
            (*current_sctx.ptr()).sc_sid,
        ) as size_t);
    }
    let mut fnamelen: size_t = fname_buflen.wrapping_add(strlen(script_name));
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if fnamelen < FLEN_FIXED as size_t {
        strcpy(
            fname_buf.offset(fname_buflen as isize),
            script_name as *mut ::core::ffi::c_char,
        );
        fname = fname_buf;
    } else {
        fname = xmalloc(fnamelen.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
        *tofree = fname;
        snprintf(
            fname,
            fnamelen.wrapping_add(1 as size_t),
            b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
            fname_buf,
            script_name,
        );
    }
    return fname;
}
pub unsafe extern "C" fn get_func_arity(
    mut name: *const ::core::ffi::c_char,
    mut required: *mut ::core::ffi::c_int,
    mut optional: *mut ::core::ffi::c_int,
    mut varargs: *mut bool,
) -> ::core::ffi::c_int {
    let mut argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut min_argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fdef: *const EvalFuncDef = find_internal_func(name);
    if !fdef.is_null() {
        argcount = (*fdef).max_argc as ::core::ffi::c_int;
        min_argcount = (*fdef).min_argc as ::core::ffi::c_int;
        *varargs = false_0 != 0;
    } else {
        let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
        let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
        let mut fname: *mut ::core::ffi::c_char = fname_trans_sid(
            name,
            &raw mut fname_buf as *mut ::core::ffi::c_char,
            &raw mut tofree,
            &raw mut error,
        );
        let mut ufunc: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
        if error == FCERR_NONE as ::core::ffi::c_int {
            ufunc = find_func(fname);
        }
        xfree(tofree as *mut ::core::ffi::c_void);
        if ufunc.is_null() {
            return FAIL;
        }
        argcount = (*ufunc).uf_args.ga_len;
        min_argcount = (*ufunc).uf_args.ga_len - (*ufunc).uf_def_args.ga_len;
        *varargs = (*ufunc).uf_varargs != 0;
    }
    *required = min_argcount;
    *optional = argcount - min_argcount;
    return OK;
}
pub unsafe extern "C" fn find_func(mut name: *const ::core::ffi::c_char) -> *mut ufunc_T {
    let mut hi: *mut hashitem_T = hash_find(func_hashtab.ptr(), name);
    if !((*hi).hi_key.is_null()
        || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
    {
        return (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
    }
    return ::core::ptr::null_mut::<ufunc_T>();
}
unsafe extern "C" fn func_is_global(mut ufunc: *const ufunc_T) -> bool {
    return *(&raw const (*ufunc).uf_name as *const ::core::ffi::c_char)
        .offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
        != K_SPECIAL;
}
unsafe extern "C" fn cat_func_name(
    mut buf: *mut ::core::ffi::c_char,
    mut bufsize: size_t,
    mut fp: *const ufunc_T,
) -> ::core::ffi::c_int {
    let mut len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut uflen: size_t = (*fp).uf_namelen;
    '_c2rust_label: {
        if uflen > 0 as size_t {
        } else {
            __assert_fail(
                b"uflen > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/userfunc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                736 as ::core::ffi::c_uint,
                b"int cat_func_name(char *, size_t, const ufunc_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if !func_is_global(fp) && uflen > 3 as size_t {
        len = snprintf(
            buf,
            bufsize,
            b"<SNR>%s\0".as_ptr() as *const ::core::ffi::c_char,
            (&raw const (*fp).uf_name as *const ::core::ffi::c_char)
                .offset(3 as ::core::ffi::c_int as isize),
        );
    } else {
        len = snprintf(
            buf,
            bufsize,
            b"%s\0".as_ptr() as *const ::core::ffi::c_char,
            &raw const (*fp).uf_name as *const ::core::ffi::c_char,
        );
    }
    '_c2rust_label_0: {
        if len > 0 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"len > 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/userfunc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                744 as ::core::ffi::c_uint,
                b"int cat_func_name(char *, size_t, const ufunc_T *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    return if len >= bufsize as ::core::ffi::c_int {
        bufsize as ::core::ffi::c_int - 1 as ::core::ffi::c_int
    } else {
        len
    };
}
unsafe extern "C" fn add_nr_var(
    mut dp: *mut dict_T,
    mut v: *mut dictitem_T,
    mut name: *mut ::core::ffi::c_char,
    mut nr: varnumber_T,
) {
    strcpy(&raw mut (*v).di_key as *mut ::core::ffi::c_char, name);
    (*v).di_flags =
        (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
    hash_add(
        &raw mut (*dp).dv_hashtab,
        &raw mut (*v).di_key as *mut ::core::ffi::c_char,
    );
    (*v).di_tv.v_type = VAR_NUMBER;
    (*v).di_tv.v_lock = VAR_FIXED;
    (*v).di_tv.vval.v_number = nr;
}
unsafe extern "C" fn free_funccal(mut fc: *mut funccall_T) {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*fc).fc_ufuncs.ga_len {
        let mut fp: *mut ufunc_T =
            *((*fc).fc_ufuncs.ga_data as *mut *mut ufunc_T).offset(i as isize);
        if !fp.is_null() && (*fp).uf_scoped == fc {
            (*fp).uf_scoped = ::core::ptr::null_mut::<funccall_T>();
        }
        i += 1;
    }
    ga_clear(&raw mut (*fc).fc_ufuncs);
    func_ptr_unref((*fc).fc_func);
    xfree(fc as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn free_funccal_contents(mut fc: *mut funccall_T) {
    vars_clear(&raw mut (*fc).fc_l_vars.dv_hashtab);
    vars_clear(&raw mut (*fc).fc_l_avars.dv_hashtab);
    let l_: *mut list_T = &raw mut (*fc).fc_l_varlist;
    if !l_.is_null() {
        let mut li: *mut listitem_T = (*l_).lv_first;
        while !li.is_null() {
            tv_clear(&raw mut (*li).li_tv);
            li = (*li).li_next;
        }
    }
    free_funccal(fc);
}
unsafe extern "C" fn cleanup_function_call(mut fc: *mut funccall_T) {
    let mut may_free_fc: bool = (*fc).fc_refcount <= 0 as ::core::ffi::c_int;
    let mut free_fc: bool = true_0 != 0;
    current_funccal.set((*fc).fc_caller);
    if may_free_fc as ::core::ffi::c_int != 0
        && (*fc).fc_l_vars.dv_refcount == DO_NOT_FREE_CNT as ::core::ffi::c_int
    {
        vars_clear(&raw mut (*fc).fc_l_vars.dv_hashtab);
    } else {
        free_fc = false_0 != 0;
    }
    if may_free_fc as ::core::ffi::c_int != 0
        && (*fc).fc_l_avars.dv_refcount == DO_NOT_FREE_CNT as ::core::ffi::c_int
    {
        vars_clear_ext(&raw mut (*fc).fc_l_avars.dv_hashtab, false_0 != 0);
    } else {
        free_fc = false_0 != 0;
        let dihi_ht_: *mut hashtab_T = &raw mut (*fc).fc_l_avars.dv_hashtab;
        let mut dihi_todo_: size_t = (*dihi_ht_).ht_used;
        let mut dihi_: *mut hashitem_T = (*dihi_ht_).ht_array;
        while dihi_todo_ != 0 {
            if !((*dihi_).hi_key.is_null()
                || (*dihi_).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                dihi_todo_ = dihi_todo_.wrapping_sub(1);
                let di: *mut dictitem_T = (*dihi_)
                    .hi_key
                    .offset(-(17 as ::core::ffi::c_ulong as isize))
                    as *mut dictitem_T;
                tv_copy(&raw mut (*di).di_tv, &raw mut (*di).di_tv);
            }
            dihi_ = dihi_.offset(1);
        }
    }
    if may_free_fc as ::core::ffi::c_int != 0
        && (*fc).fc_l_varlist.lv_refcount == DO_NOT_FREE_CNT as ::core::ffi::c_int
    {
        (*fc).fc_l_varlist.lv_first = ::core::ptr::null_mut::<listitem_T>();
    } else {
        free_fc = false_0 != 0;
        let l_: *mut list_T = &raw mut (*fc).fc_l_varlist;
        if !l_.is_null() {
            let mut li: *mut listitem_T = (*l_).lv_first;
            while !li.is_null() {
                tv_copy(&raw mut (*li).li_tv, &raw mut (*li).li_tv);
                li = (*li).li_next;
            }
        }
    }
    if free_fc {
        free_funccal(fc);
    } else {
        static made_copy: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
        (*fc).fc_caller = previous_funccal.get();
        previous_funccal.set(fc);
        if want_garbage_collect.get() {
            made_copy.set(0 as ::core::ffi::c_int);
        } else {
            (*made_copy.ptr()) += 1;
            if made_copy.get()
                >= ((4096 as ::core::ffi::c_int * 1024 as ::core::ffi::c_int) as usize)
                    .wrapping_div(::core::mem::size_of::<funccall_T>())
                    as ::core::ffi::c_int
            {
                made_copy.set(0 as ::core::ffi::c_int);
                want_garbage_collect.set(true_0 != 0);
            }
        }
    };
}
unsafe extern "C" fn funccal_unref(mut fc: *mut funccall_T, mut fp: *mut ufunc_T, mut force: bool) {
    if fc.is_null() {
        return;
    }
    (*fc).fc_refcount -= 1;
    if if force as ::core::ffi::c_int != 0 {
        ((*fc).fc_refcount <= 0 as ::core::ffi::c_int) as ::core::ffi::c_int
    } else {
        !fc_referenced(fc) as ::core::ffi::c_int
    } != 0
    {
        let mut pfc: *mut *mut funccall_T = previous_funccal.ptr();
        while !(*pfc).is_null() {
            if fc == *pfc {
                *pfc = (*fc).fc_caller;
                free_funccal_contents(fc);
                return;
            }
            pfc = &raw mut (**pfc).fc_caller;
        }
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*fc).fc_ufuncs.ga_len {
        if *((*fc).fc_ufuncs.ga_data as *mut *mut ufunc_T).offset(i as isize) == fp {
            *((*fc).fc_ufuncs.ga_data as *mut *mut ufunc_T).offset(i as isize) =
                ::core::ptr::null_mut::<ufunc_T>();
        }
        i += 1;
    }
}
unsafe extern "C" fn func_remove(mut fp: *mut ufunc_T) -> bool {
    let mut hi: *mut hashitem_T = hash_find(
        func_hashtab.ptr(),
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
    );
    if (*hi).hi_key.is_null() || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
    {
        return false_0 != 0;
    }
    hash_remove(func_hashtab.ptr(), hi);
    return true_0 != 0;
}
unsafe extern "C" fn func_clear_items(mut fp: *mut ufunc_T) {
    ga_clear_strings(&raw mut (*fp).uf_args);
    ga_clear_strings(&raw mut (*fp).uf_def_args);
    ga_clear_strings(&raw mut (*fp).uf_lines);
    if (*fp).uf_flags & FC_LUAREF != 0 {
        api_free_luaref((*fp).uf_luaref);
        (*fp).uf_luaref = LUA_NOREF as LuaRef;
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*fp).uf_tml_count as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    let mut ptr__0: *mut *mut ::core::ffi::c_void =
        &raw mut (*fp).uf_tml_total as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__0);
    *ptr__0 = NULL;
    let _ = *ptr__0;
    let mut ptr__1: *mut *mut ::core::ffi::c_void =
        &raw mut (*fp).uf_tml_self as *mut *mut ::core::ffi::c_void;
    xfree(*ptr__1);
    *ptr__1 = NULL;
    let _ = *ptr__1;
}
unsafe extern "C" fn func_clear(mut fp: *mut ufunc_T, mut force: bool) {
    if (*fp).uf_cleared {
        return;
    }
    (*fp).uf_cleared = true_0 != 0;
    func_clear_items(fp);
    funccal_unref((*fp).uf_scoped, fp, force);
}
unsafe extern "C" fn func_free(mut fp: *mut ufunc_T) {
    if (*fp).uf_flags & (FC_DELETED | FC_REMOVED) == 0 as ::core::ffi::c_int {
        func_remove(fp);
    }
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut (*fp).uf_name_exp as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    xfree(fp as *mut ::core::ffi::c_void);
}
unsafe extern "C" fn func_clear_free(mut fp: *mut ufunc_T, mut force: bool) {
    func_clear(fp, force);
    func_free(fp);
}
pub unsafe extern "C" fn create_funccal(
    mut fp: *mut ufunc_T,
    mut rettv: *mut typval_T,
) -> *mut funccall_T {
    let mut fc: *mut funccall_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<funccall_T>()) as *mut funccall_T;
    (*fc).fc_caller = current_funccal.get();
    current_funccal.set(fc);
    (*fc).fc_func = fp;
    func_ptr_ref(fp);
    (*fc).fc_rettv = rettv;
    return fc;
}
pub unsafe extern "C" fn call_user_func(
    mut fp: *mut ufunc_T,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut firstline: linenr_T,
    mut lastline: linenr_T,
    mut selfdict: *mut dict_T,
) {
    let mut using_sandbox: bool = false_0 != 0;
    static depth: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut fixvar_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut islambda: bool = false_0 != 0;
    let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut namelen: size_t = 0;
    let mut tv_to_free: [*mut typval_T; 20] = [::core::ptr::null_mut::<typval_T>(); 20];
    let mut tv_to_free_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut wait_start: proftime_T = 0;
    let mut call_start: proftime_T = 0;
    let mut started_profiling: bool = false_0 != 0;
    let mut did_save_redo: bool = false_0 != 0;
    let mut save_redo: save_redo_T = save_redo_T {
        sr_redobuff: buffheader_T {
            bh_first: buffblock_T {
                b_next: ::core::ptr::null_mut::<buffblock>(),
                b_strlen: 0,
                b_str: [0; 1],
            },
            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
            bh_index: 0,
            bh_space: 0,
            bh_create_newblock: false,
        },
        sr_old_redobuff: buffheader_T {
            bh_first: buffblock_T {
                b_next: ::core::ptr::null_mut::<buffblock>(),
                b_strlen: 0,
                b_str: [0; 1],
            },
            bh_curr: ::core::ptr::null_mut::<buffblock_T>(),
            bh_index: 0,
            bh_space: 0,
            bh_create_newblock: false,
        },
    };
    if depth.get() as OptInt >= p_mfd.get() {
        emsg(gettext(
            b"E132: Function call depth is higher than 'maxfuncdepth'\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = -1 as varnumber_T;
        return;
    }
    (*depth.ptr()) += 1;
    save_search_patterns();
    if !ins_compl_active() {
        saveRedobuff(&raw mut save_redo);
        did_save_redo = true_0 != 0;
    }
    (*fp).uf_calls += 1;
    line_breakcheck();
    let mut fc: *mut funccall_T = create_funccal(fp, rettv);
    (*fc).fc_level = ex_nesting_level.get();
    (*fc).fc_breakpoint = dbg_find_breakpoint(
        false_0 != 0,
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
        0 as linenr_T,
    );
    (*fc).fc_dbg_tick = debug_tick.get();
    ga_init(
        &raw mut (*fc).fc_ufuncs,
        ::core::mem::size_of::<*mut ufunc_T>() as ::core::ffi::c_int,
        1 as ::core::ffi::c_int,
    );
    if strncmp(
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
        b"<lambda>\0".as_ptr() as *const ::core::ffi::c_char,
        8 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        islambda = true_0 != 0;
    }
    init_var_dict(
        &raw mut (*fc).fc_l_vars,
        &raw mut (*fc).fc_l_vars_var,
        VAR_DEF_SCOPE,
    );
    if !selfdict.is_null() {
        let c2rust_fresh3 = fixvar_idx;
        fixvar_idx = fixvar_idx + 1;
        v = (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh3 as isize)
            as *mut dictitem_T;
        name = &raw mut (*v).di_key as *mut ::core::ffi::c_char;
        strcpy(
            name,
            b"self\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*v).di_flags =
            (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
        hash_add(
            &raw mut (*fc).fc_l_vars.dv_hashtab,
            &raw mut (*v).di_key as *mut ::core::ffi::c_char,
        );
        (*v).di_tv.v_type = VAR_DICT;
        (*v).di_tv.v_lock = VAR_UNLOCKED;
        (*v).di_tv.vval.v_dict = selfdict;
        (*selfdict).dv_refcount += 1;
    }
    init_var_dict(
        &raw mut (*fc).fc_l_avars,
        &raw mut (*fc).fc_l_avars_var,
        VAR_SCOPE,
    );
    if (*fp).uf_flags & FC_NOARGS == 0 as ::core::ffi::c_int {
        let c2rust_fresh4 = fixvar_idx;
        fixvar_idx = fixvar_idx + 1;
        add_nr_var(
            &raw mut (*fc).fc_l_avars,
            (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh4 as isize)
                as *mut dictitem_T,
            b"0\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            (if argcount >= (*fp).uf_args.ga_len {
                argcount - (*fp).uf_args.ga_len
            } else {
                0 as ::core::ffi::c_int
            }) as varnumber_T,
        );
    }
    (*fc).fc_l_avars.dv_lock = VAR_FIXED;
    if (*fp).uf_flags & FC_NOARGS == 0 as ::core::ffi::c_int {
        let c2rust_fresh5 = fixvar_idx;
        fixvar_idx = fixvar_idx + 1;
        v = (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh5 as isize)
            as *mut dictitem_T;
        name = &raw mut (*v).di_key as *mut ::core::ffi::c_char;
        strcpy(
            name,
            b"000\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        (*v).di_flags =
            (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
        hash_add(
            &raw mut (*fc).fc_l_avars.dv_hashtab,
            &raw mut (*v).di_key as *mut ::core::ffi::c_char,
        );
        (*v).di_tv.v_type = VAR_LIST;
        (*v).di_tv.v_lock = VAR_FIXED;
        (*v).di_tv.vval.v_list = &raw mut (*fc).fc_l_varlist;
    }
    tv_list_init_static(&raw mut (*fc).fc_l_varlist);
    tv_list_set_lock(&raw mut (*fc).fc_l_varlist, VAR_FIXED);
    if (*fp).uf_flags & FC_NOARGS == 0 as ::core::ffi::c_int {
        let c2rust_fresh6 = fixvar_idx;
        fixvar_idx = fixvar_idx + 1;
        add_nr_var(
            &raw mut (*fc).fc_l_avars,
            (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh6 as isize)
                as *mut dictitem_T,
            b"firstline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            firstline as varnumber_T,
        );
        let c2rust_fresh7 = fixvar_idx;
        fixvar_idx = fixvar_idx + 1;
        add_nr_var(
            &raw mut (*fc).fc_l_avars,
            (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh7 as isize)
                as *mut dictitem_T,
            b"lastline\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            lastline as varnumber_T,
        );
    }
    let mut default_arg_err: bool = false_0 != 0;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < argcount || i < (*fp).uf_args.ga_len {
        let mut addlocal: bool = false_0 != 0;
        let mut isdefault: bool = false_0 != 0;
        let mut def_rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut ai: ::core::ffi::c_int = i - (*fp).uf_args.ga_len;
        if ai < 0 as ::core::ffi::c_int {
            name = *((*fp).uf_args.ga_data as *mut *mut ::core::ffi::c_char).offset(i as isize);
            if islambda {
                addlocal = true_0 != 0;
            }
            isdefault = ai + (*fp).uf_def_args.ga_len >= 0 as ::core::ffi::c_int && i >= argcount;
            if isdefault {
                let mut default_expr: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                def_rettv.v_type = VAR_NUMBER;
                def_rettv.vval.v_number = -1 as varnumber_T;
                default_expr = *((*fp).uf_def_args.ga_data as *mut *mut ::core::ffi::c_char)
                    .offset((ai + (*fp).uf_def_args.ga_len) as isize);
                if eval1(
                    &raw mut default_expr,
                    &raw mut def_rettv,
                    EVALARG_EVALUATE.ptr(),
                ) == FAIL
                {
                    default_arg_err = true_0 != 0;
                    break;
                }
            }
            namelen = strlen(name);
        } else {
            if (*fp).uf_flags & FC_NOARGS != 0 as ::core::ffi::c_int {
                break;
            }
            namelen = snprintf(
                &raw mut numbuf as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                ai + 1 as ::core::ffi::c_int,
            ) as size_t;
            name = &raw mut numbuf as *mut ::core::ffi::c_char;
        }
        if fixvar_idx < FIXVAR_CNT as ::core::ffi::c_int
            && namelen <= VAR_SHORT_LEN as ::core::ffi::c_int as size_t
        {
            let c2rust_fresh8 = fixvar_idx;
            fixvar_idx = fixvar_idx + 1;
            v = (&raw mut (*fc).fc_fixvar as *mut C2Rust_Unnamed_7).offset(c2rust_fresh8 as isize)
                as *mut dictitem_T;
            (*v).di_flags =
                (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int) as uint8_t;
            strcpy(&raw mut (*v).di_key as *mut ::core::ffi::c_char, name);
        } else {
            v = tv_dict_item_alloc_len(name, namelen);
            (*v).di_flags = ((*v).di_flags as ::core::ffi::c_int
                | (DI_FLAGS_RO as ::core::ffi::c_int | DI_FLAGS_FIX as ::core::ffi::c_int))
                as uint8_t;
        }
        (*v).di_tv = if isdefault as ::core::ffi::c_int != 0 {
            def_rettv
        } else {
            *argvars.offset(i as isize)
        };
        (*v).di_tv.v_lock = VAR_FIXED;
        if isdefault {
            let c2rust_fresh9 = tv_to_free_len;
            tv_to_free_len = tv_to_free_len + 1;
            let c2rust_lvalue_ptr = &raw mut tv_to_free[c2rust_fresh9 as usize];
            *c2rust_lvalue_ptr = &raw mut (*v).di_tv;
        }
        if addlocal {
            tv_copy(&raw mut (*v).di_tv, &raw mut (*v).di_tv);
            hash_add(
                &raw mut (*fc).fc_l_vars.dv_hashtab,
                &raw mut (*v).di_key as *mut ::core::ffi::c_char,
            );
        } else {
            hash_add(
                &raw mut (*fc).fc_l_avars.dv_hashtab,
                &raw mut (*v).di_key as *mut ::core::ffi::c_char,
            );
        }
        if ai >= 0 as ::core::ffi::c_int && ai < MAX_FUNC_ARGS as ::core::ffi::c_int {
            let mut li: *mut listitem_T =
                (&raw mut (*fc).fc_l_listitems as *mut listitem_T).offset(ai as isize);
            (*li).li_tv = *argvars.offset(i as isize);
            (*li).li_tv.v_lock = VAR_FIXED;
            tv_list_append(&raw mut (*fc).fc_l_varlist, li);
        }
        i += 1;
    }
    (*RedrawingDisabled.ptr()) += 1;
    if (*fp).uf_flags & FC_SANDBOX != 0 {
        using_sandbox = true_0 != 0;
        (*sandbox.ptr()) += 1;
    }
    estack_push_ufunc(fp, 1 as linenr_T);
    if p_verbose.get() >= 12 as OptInt {
        (*no_wait_return.ptr()) += 1;
        verbose_enter_scroll();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"calling %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name,
        );
        if p_verbose.get() >= 14 as OptInt {
            msg_puts(b"(\0".as_ptr() as *const ::core::ffi::c_char);
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < argcount {
                if i_0 > 0 as ::core::ffi::c_int {
                    msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
                }
                if (*argvars.offset(i_0 as isize)).v_type as ::core::ffi::c_uint
                    == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    msg_outnum((*argvars.offset(i_0 as isize)).vval.v_number as ::core::ffi::c_int);
                } else {
                    (*emsg_off.ptr()) += 1;
                    let mut tofree: *mut ::core::ffi::c_char = encode_tv2string(
                        argvars.offset(i_0 as isize),
                        ::core::ptr::null_mut::<size_t>(),
                    );
                    (*emsg_off.ptr()) -= 1;
                    if !tofree.is_null() {
                        let mut s: *mut ::core::ffi::c_char = tofree;
                        let mut buf: [::core::ffi::c_char; 480] = [0; 480];
                        if vim_strsize(s) > MSG_BUF_CLEN {
                            trunc_string(
                                s,
                                &raw mut buf as *mut ::core::ffi::c_char,
                                MSG_BUF_CLEN,
                                ::core::mem::size_of::<[::core::ffi::c_char; 480]>()
                                    as ::core::ffi::c_int,
                            );
                            s = &raw mut buf as *mut ::core::ffi::c_char;
                        }
                        msg_puts(s);
                        xfree(tofree as *mut ::core::ffi::c_void);
                    }
                }
                i_0 += 1;
            }
            msg_puts(b")\0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        verbose_leave_scroll();
        (*no_wait_return.ptr()) -= 1;
    }
    let do_profiling_yes: bool = do_profiling.get() == PROF_YES;
    let mut func_not_yet_profiling_but_should: bool = do_profiling_yes as ::core::ffi::c_int != 0
        && (*fp).uf_profiling == 0
        && has_profiling(
            false_0 != 0,
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            ::core::ptr::null_mut::<bool>(),
        ) as ::core::ffi::c_int
            != 0;
    if func_not_yet_profiling_but_should {
        started_profiling = true_0 != 0;
        func_do_profile(fp);
    }
    let mut func_or_func_caller_profiling: bool = do_profiling_yes as ::core::ffi::c_int != 0
        && ((*fp).uf_profiling != 0
            || !(*fc).fc_caller.is_null() && (*(*(*fc).fc_caller).fc_func).uf_profiling != 0);
    if func_or_func_caller_profiling {
        (*fp).uf_tm_count += 1;
        call_start = profile_start();
        (*fp).uf_tm_children = profile_zero();
    }
    if do_profiling_yes {
        wait_start = script_prof_save();
    }
    let save_current_sctx: sctx_T = current_sctx.get();
    current_sctx.set((*fp).uf_script_ctx);
    let mut save_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(false_0);
    if default_arg_err as ::core::ffi::c_int != 0
        && ((*fp).uf_flags & FC_ABORT != 0 || trylevel.get() > 0 as ::core::ffi::c_int)
    {
        did_emsg.set(true_0);
    } else if islambda {
        let mut p: *mut ::core::ffi::c_char = (*((*fp).uf_lines.ga_data
            as *mut *mut ::core::ffi::c_char))
            .offset(7 as ::core::ffi::c_int as isize);
        (*ex_nesting_level.ptr()) += 1;
        eval1(&raw mut p, rettv, EVALARG_EVALUATE.ptr());
        (*ex_nesting_level.ptr()) -= 1;
    } else {
        do_cmdline(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            Some(
                get_func_line
                    as unsafe extern "C" fn(
                        ::core::ffi::c_int,
                        *mut ::core::ffi::c_void,
                        ::core::ffi::c_int,
                        bool,
                    ) -> *mut ::core::ffi::c_char,
            ),
            fc as *mut ::core::ffi::c_void,
            DOCMD_NOWAIT as ::core::ffi::c_int
                | DOCMD_VERBOSE as ::core::ffi::c_int
                | DOCMD_REPEAT as ::core::ffi::c_int,
        );
    }
    handle_defer_one(current_funccal.get());
    (*RedrawingDisabled.ptr()) -= 1;
    if did_emsg.get() != 0 && (*fp).uf_flags & FC_ABORT != 0
        || (*rettv).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        tv_clear(rettv);
        (*rettv).v_type = VAR_NUMBER;
        (*rettv).vval.v_number = -1 as varnumber_T;
    }
    if func_or_func_caller_profiling {
        call_start = profile_end(call_start);
        call_start = profile_sub_wait(wait_start, call_start);
        (*fp).uf_tm_total = profile_add((*fp).uf_tm_total, call_start);
        (*fp).uf_tm_self = profile_self((*fp).uf_tm_self, call_start, (*fp).uf_tm_children);
        if !(*fc).fc_caller.is_null() && (*(*(*fc).fc_caller).fc_func).uf_profiling != 0 {
            (*(*(*fc).fc_caller).fc_func).uf_tm_children =
                profile_add((*(*(*fc).fc_caller).fc_func).uf_tm_children, call_start);
            (*(*(*fc).fc_caller).fc_func).uf_tml_children =
                profile_add((*(*(*fc).fc_caller).fc_func).uf_tml_children, call_start);
        }
        if started_profiling {
            (*fp).uf_profiling = false_0;
        }
    }
    if p_verbose.get() >= 12 as OptInt {
        (*no_wait_return.ptr()) += 1;
        verbose_enter_scroll();
        if aborting() {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"%s aborted\0".as_ptr() as *const ::core::ffi::c_char),
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
            );
        } else if (*(*fc).fc_rettv).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            smsg(
                0 as ::core::ffi::c_int,
                gettext(b"%s returning #%ld\0".as_ptr() as *const ::core::ffi::c_char),
                (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_name,
                (*(*fc).fc_rettv).vval.v_number,
            );
        } else {
            let mut buf_0: [::core::ffi::c_char; 480] = [0; 480];
            (*emsg_off.ptr()) += 1;
            let mut s_0: *mut ::core::ffi::c_char =
                encode_tv2string((*fc).fc_rettv, ::core::ptr::null_mut::<size_t>());
            let mut tofree_0: *mut ::core::ffi::c_char = s_0;
            (*emsg_off.ptr()) -= 1;
            if !s_0.is_null() {
                if vim_strsize(s_0) > MSG_BUF_CLEN {
                    trunc_string(
                        s_0,
                        &raw mut buf_0 as *mut ::core::ffi::c_char,
                        MSG_BUF_CLEN,
                        MSG_BUF_LEN,
                    );
                    s_0 = &raw mut buf_0 as *mut ::core::ffi::c_char;
                }
                smsg(
                    0 as ::core::ffi::c_int,
                    gettext(b"%s returning %s\0".as_ptr() as *const ::core::ffi::c_char),
                    (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_name,
                    s_0,
                );
                xfree(tofree_0 as *mut ::core::ffi::c_void);
            }
        }
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        verbose_leave_scroll();
        (*no_wait_return.ptr()) -= 1;
    }
    estack_pop();
    current_sctx.set(save_current_sctx);
    if do_profiling_yes {
        script_prof_restore(wait_start);
    }
    if using_sandbox {
        (*sandbox.ptr()) -= 1;
    }
    if p_verbose.get() >= 12 as OptInt
        && !(*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_name
        .is_null()
    {
        (*no_wait_return.ptr()) += 1;
        verbose_enter_scroll();
        smsg(
            0 as ::core::ffi::c_int,
            gettext(b"continuing in %s\0".as_ptr() as *const ::core::ffi::c_char),
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_name,
        );
        msg_puts(b"\n\0".as_ptr() as *const ::core::ffi::c_char);
        verbose_leave_scroll();
        (*no_wait_return.ptr()) -= 1;
    }
    (*did_emsg.ptr()) |= save_did_emsg;
    (*depth.ptr()) -= 1;
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i_1 < tv_to_free_len {
        tv_clear(tv_to_free[i_1 as usize]);
        i_1 += 1;
    }
    cleanup_function_call(fc);
    (*fp).uf_calls -= 1;
    if (*fp).uf_calls <= 0 as ::core::ffi::c_int && (*fp).uf_refcount <= 0 as ::core::ffi::c_int {
        func_clear_free(fp, false_0 != 0);
    }
    if did_save_redo {
        restoreRedobuff(&raw mut save_redo);
    }
    restore_search_patterns();
}
unsafe extern "C" fn func_name_refcount(mut name: *const ::core::ffi::c_char) -> bool {
    return *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
        || *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '<' as ::core::ffi::c_int
            && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == 'l' as ::core::ffi::c_int;
}
unsafe extern "C" fn check_user_func_argcount(
    mut fp: *mut ufunc_T,
    mut argcount: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let regular_args: ::core::ffi::c_int = (*fp).uf_args.ga_len;
    if argcount < regular_args - (*fp).uf_def_args.ga_len {
        return FCERR_TOOFEW as ::core::ffi::c_int;
    } else if (*fp).uf_varargs == 0 && argcount > regular_args {
        return FCERR_TOOMANY as ::core::ffi::c_int;
    }
    return FCERR_UNKNOWN as ::core::ffi::c_int;
}
unsafe extern "C" fn call_user_func_check(
    mut fp: *mut ufunc_T,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut funcexe: *mut funcexe_T,
    mut selfdict: *mut dict_T,
) -> ::core::ffi::c_int {
    if (*fp).uf_flags & FC_LUAREF != 0 {
        return typval_exec_lua_callable((*fp).uf_luaref, argcount, argvars, rettv);
    }
    if (*fp).uf_flags & FC_RANGE != 0 && !(*funcexe).fe_doesrange.is_null() {
        *(*funcexe).fe_doesrange = true_0 != 0;
    }
    let mut error: ::core::ffi::c_int = check_user_func_argcount(fp, argcount);
    if error != FCERR_UNKNOWN as ::core::ffi::c_int {
        return error;
    }
    if (*fp).uf_flags & FC_DICT != 0 && selfdict.is_null() {
        error = FCERR_DICT as ::core::ffi::c_int;
    } else {
        call_user_func(
            fp,
            argcount,
            argvars,
            rettv,
            (*funcexe).fe_firstline,
            (*funcexe).fe_lastline,
            if (*fp).uf_flags & FC_DICT != 0 {
                selfdict
            } else {
                ::core::ptr::null_mut::<dict_T>()
            },
        );
        error = FCERR_NONE as ::core::ffi::c_int;
    }
    return error;
}
static funccal_stack: GlobalCell<*mut funccal_entry_T> =
    GlobalCell::new(::core::ptr::null_mut::<funccal_entry_T>());
pub unsafe extern "C" fn save_funccal(mut entry: *mut funccal_entry_T) {
    (*entry).top_funccal = current_funccal.get() as *mut ::core::ffi::c_void;
    (*entry).next = funccal_stack.get();
    funccal_stack.set(entry);
    current_funccal.set(::core::ptr::null_mut::<funccall_T>());
}
pub unsafe extern "C" fn restore_funccal() {
    if (*funccal_stack.ptr()).is_null() {
        iemsg(b"INTERNAL: restore_funccal()\0".as_ptr() as *const ::core::ffi::c_char);
    } else {
        current_funccal.set((*funccal_stack.get()).top_funccal as *mut funccall_T);
        funccal_stack.set((*funccal_stack.get()).next);
    };
}
pub unsafe extern "C" fn get_current_funccal() -> *mut funccall_T {
    return current_funccal.get();
}
pub unsafe extern "C" fn set_current_funccal(mut fc: *mut funccall_T) {
    current_funccal.set(fc);
}
unsafe extern "C" fn builtin_function(
    mut name: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> bool {
    if !(*name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
        >= 'a' as ::core::ffi::c_uint
        && *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_uint
            <= 'z' as ::core::ffi::c_uint)
        || *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
    {
        return false_0 != 0;
    }
    let mut p: *const ::core::ffi::c_char = (if len == -1 as ::core::ffi::c_int {
        strchr(name, AUTOLOAD_CHAR) as *mut ::core::ffi::c_void
    } else {
        memchr(
            name as *const ::core::ffi::c_void,
            AUTOLOAD_CHAR,
            len as size_t,
        )
    }) as *const ::core::ffi::c_char;
    return p.is_null();
}
pub unsafe extern "C" fn func_call(
    mut name: *mut ::core::ffi::c_char,
    mut args: *mut typval_T,
    mut partial: *mut partial_T,
    mut selfdict: *mut dict_T,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut funcexe: funcexe_T = funcexe_T {
        fe_argv_func: None,
        fe_firstline: 0,
        fe_lastline: 0,
        fe_doesrange: ::core::ptr::null_mut::<bool>(),
        fe_evaluate: false,
        fe_partial: ::core::ptr::null_mut::<partial_T>(),
        fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
        fe_basetv: ::core::ptr::null_mut::<typval_T>(),
        fe_found_var: false,
    };
    let mut argv: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut r: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let l_: *mut list_T = (*args).vval.v_list;
    '_func_call_skip_call: {
        's_51: {
            if !l_.is_null() {
                let mut item: *mut listitem_T = (*l_).lv_first;
                loop {
                    if item.is_null() {
                        break 's_51;
                    }
                    if argc
                        == MAX_FUNC_ARGS as ::core::ffi::c_int
                            - (if partial.is_null() {
                                0 as ::core::ffi::c_int
                            } else {
                                (*partial).pt_argc
                            })
                    {
                        emsg(gettext(
                            b"E699: Too many arguments\0".as_ptr() as *const ::core::ffi::c_char
                        ));
                        break '_func_call_skip_call;
                    } else {
                        let c2rust_fresh11 = argc;
                        argc = argc + 1;
                        tv_copy(
                            &raw mut (*item).li_tv,
                            (&raw mut argv as *mut typval_T).offset(c2rust_fresh11 as isize),
                        );
                        item = (*item).li_next;
                    }
                }
            }
        }
        funcexe = FUNCEXE_INIT;
        funcexe.fe_firstline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_lastline = (*curwin.get()).w_cursor.lnum;
        funcexe.fe_evaluate = true_0 != 0;
        funcexe.fe_partial = partial;
        funcexe.fe_selfdict = selfdict;
        r = call_func(
            name,
            -1 as ::core::ffi::c_int,
            rettv,
            argc,
            &raw mut argv as *mut typval_T,
            &raw mut funcexe,
        );
    }
    while argc > 0 as ::core::ffi::c_int {
        argc -= 1;
        tv_clear((&raw mut argv as *mut typval_T).offset(argc as isize));
    }
    return r;
}
pub unsafe extern "C" fn callback_call_retnr(
    mut callback: *mut Callback,
    mut argcount: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
) -> varnumber_T {
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    if !callback_call(callback, argcount, argvars, &raw mut rettv) {
        return -2 as varnumber_T;
    }
    let mut retval: varnumber_T =
        tv_get_number_chk(&raw mut rettv, ::core::ptr::null_mut::<bool>());
    tv_clear(&raw mut rettv);
    return retval;
}
unsafe extern "C" fn user_func_error(
    mut error: ::core::ffi::c_int,
    mut name: *const ::core::ffi::c_char,
    mut found_var: bool,
) {
    match error {
        0 => {
            if found_var {
                semsg(
                    gettext(&raw const e_not_callable_type_str as *const ::core::ffi::c_char),
                    name,
                );
            } else {
                emsg_funcname(
                    &raw const e_unknown_function_str as *const ::core::ffi::c_char,
                    name,
                );
            }
        }
        8 => {
            emsg_funcname(
                b"E276: Cannot use function as a method: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name,
            );
        }
        7 => {
            emsg_funcname(
                b"E933: Function was deleted: %s\0".as_ptr() as *const ::core::ffi::c_char,
                name,
            );
        }
        1 => {
            emsg_funcname(
                gettext(&raw const e_toomanyarg as *const ::core::ffi::c_char),
                name,
            );
        }
        2 => {
            emsg_funcname(
                gettext(&raw const e_toofewarg as *const ::core::ffi::c_char),
                name,
            );
        }
        3 => {
            emsg_funcname(
                b"E120: Using <SID> not in a script context: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name,
            );
        }
        4 => {
            emsg_funcname(
                b"E725: Calling dict function without Dictionary: %s\0".as_ptr()
                    as *const ::core::ffi::c_char,
                name,
            );
        }
        _ => {}
    };
}
unsafe extern "C" fn argv_add_base(
    basetv: *mut typval_T,
    argvars: *mut *mut typval_T,
    argcount: *mut ::core::ffi::c_int,
    new_argvars: *mut typval_T,
    argv_base: *mut ::core::ffi::c_int,
) {
    if !basetv.is_null() {
        memmove(
            new_argvars.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            *argvars as *const ::core::ffi::c_void,
            ::core::mem::size_of::<typval_T>().wrapping_mul(*argcount as size_t),
        );
        *new_argvars.offset(0 as ::core::ffi::c_int as isize) = *basetv;
        *argcount += 1;
        *argvars = new_argvars;
        *argv_base = 1 as ::core::ffi::c_int;
    }
}
pub unsafe extern "C" fn call_func(
    mut funcname: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut rettv: *mut typval_T,
    mut argcount_in: ::core::ffi::c_int,
    mut argvars_in: *mut typval_T,
    mut funcexe: *mut funcexe_T,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = FAIL;
    let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
    let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
    let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fname: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut argcount: ::core::ffi::c_int = argcount_in;
    let mut argvars: *mut typval_T = argvars_in;
    let mut selfdict: *mut dict_T = (*funcexe).fe_selfdict;
    let mut argv: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut argv_clear: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut argv_base: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut partial: *mut partial_T = (*funcexe).fe_partial;
    (*rettv).v_type = VAR_UNKNOWN;
    if len <= 0 as ::core::ffi::c_int {
        len = strlen(funcname) as ::core::ffi::c_int;
    }
    if !partial.is_null() {
        fp = (*partial).pt_func;
    }
    if fp.is_null() {
        name = xmemdupz(funcname as *const ::core::ffi::c_void, len as size_t)
            as *mut ::core::ffi::c_char;
        fname = fname_trans_sid(
            name,
            &raw mut fname_buf as *mut ::core::ffi::c_char,
            &raw mut tofree,
            &raw mut error,
        );
    }
    if !(*funcexe).fe_doesrange.is_null() {
        *(*funcexe).fe_doesrange = false_0 != 0;
    }
    '_theend: {
        if !partial.is_null() {
            if !(*partial).pt_dict.is_null() && (selfdict.is_null() || !(*partial).pt_auto) {
                selfdict = (*partial).pt_dict;
            }
            if error == FCERR_NONE as ::core::ffi::c_int
                && (*partial).pt_argc > 0 as ::core::ffi::c_int
            {
                argv_clear = 0 as ::core::ffi::c_int;
                while argv_clear < (*partial).pt_argc {
                    if argv_clear + argcount_in >= MAX_FUNC_ARGS as ::core::ffi::c_int {
                        error = FCERR_TOOMANY as ::core::ffi::c_int;
                        break '_theend;
                    } else {
                        tv_copy(
                            (*partial).pt_argv.offset(argv_clear as isize),
                            (&raw mut argv as *mut typval_T).offset(argv_clear as isize),
                        );
                        argv_clear += 1;
                    }
                }
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < argcount_in {
                    argv[(i + argv_clear) as usize] = *argvars_in.offset(i as isize);
                    i += 1;
                }
                argvars = &raw mut argv as *mut typval_T;
                argcount = (*partial).pt_argc + argcount_in;
            }
        }
        if error == FCERR_NONE as ::core::ffi::c_int
            && (*funcexe).fe_evaluate as ::core::ffi::c_int != 0
        {
            let mut is_global: bool = fp.is_null()
                && *fname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == 'g' as ::core::ffi::c_int
                && *fname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == ':' as ::core::ffi::c_int;
            let mut rfname: *mut ::core::ffi::c_char = if is_global as ::core::ffi::c_int != 0 {
                fname.offset(2 as ::core::ffi::c_int as isize)
            } else {
                fname
            };
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = 0 as varnumber_T;
            error = FCERR_UNKNOWN as ::core::ffi::c_int;
            if is_luafunc(partial) {
                if len > 0 as ::core::ffi::c_int {
                    error = FCERR_NONE as ::core::ffi::c_int;
                    argv_add_base(
                        (*funcexe).fe_basetv,
                        &raw mut argvars,
                        &raw mut argcount,
                        &raw mut argv as *mut typval_T,
                        &raw mut argv_base,
                    );
                    nlua_typval_call(funcname, len as size_t, argvars, argcount, rettv);
                } else {
                    let mut ptr_: *mut *mut ::core::ffi::c_void =
                        &raw mut name as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr_);
                    *ptr_ = NULL;
                    let _ = *ptr_;
                    funcname = b"v:lua\0".as_ptr() as *const ::core::ffi::c_char;
                }
            } else if !fp.is_null() || !builtin_function(rfname, -1 as ::core::ffi::c_int) {
                if fp.is_null() {
                    fp = find_func(rfname);
                }
                if fp.is_null()
                    && apply_autocmds(
                        EVENT_FUNCUNDEFINED,
                        rfname,
                        rfname,
                        true_0 != 0,
                        ::core::ptr::null_mut::<buf_T>(),
                    ) as ::core::ffi::c_int
                        != 0
                    && !aborting()
                {
                    fp = find_func(rfname);
                }
                if fp.is_null()
                    && script_autoload(rfname, strlen(rfname), true_0 != 0) as ::core::ffi::c_int
                        != 0
                    && !aborting()
                {
                    fp = find_func(rfname);
                }
                if !fp.is_null() && (*fp).uf_flags & FC_DELETED != 0 {
                    error = FCERR_DELETED as ::core::ffi::c_int;
                } else if !fp.is_null() {
                    if (*funcexe).fe_argv_func.is_some() {
                        argcount = (*funcexe).fe_argv_func.expect("non-null function pointer")(
                            argcount, argvars, argv_clear, fp,
                        );
                    }
                    argv_add_base(
                        (*funcexe).fe_basetv,
                        &raw mut argvars,
                        &raw mut argcount,
                        &raw mut argv as *mut typval_T,
                        &raw mut argv_base,
                    );
                    error = call_user_func_check(fp, argcount, argvars, rettv, funcexe, selfdict);
                }
            } else if !(*funcexe).fe_basetv.is_null() {
                error = call_internal_method(fname, argcount, argvars, rettv, (*funcexe).fe_basetv);
            } else {
                error = call_internal_func(fname, argcount, argvars, rettv);
            }
            update_force_abort();
        }
        if error == FCERR_NONE as ::core::ffi::c_int {
            ret = OK;
        }
    }
    if !aborting() {
        user_func_error(
            error,
            if !name.is_null() {
                name as *const ::core::ffi::c_char
            } else {
                funcname
            },
            (*funcexe).fe_found_var,
        );
    }
    while argv_clear > 0 as ::core::ffi::c_int {
        argv_clear -= 1;
        tv_clear((&raw mut argv as *mut typval_T).offset((argv_clear + argv_base) as isize));
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    xfree(name as *mut ::core::ffi::c_void);
    return ret;
}
pub unsafe extern "C" fn call_simple_luafunc(
    mut funcname: *const ::core::ffi::c_char,
    mut len: size_t,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    let mut argvars: [typval_T; 1] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 1];
    argvars[0 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
    nlua_typval_call(
        funcname,
        len,
        &raw mut argvars as *mut typval_T,
        0 as ::core::ffi::c_int,
        rettv,
    );
    return OK;
}
pub unsafe extern "C" fn call_simple_func(
    mut funcname: *const ::core::ffi::c_char,
    mut len: size_t,
    mut rettv: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = FAIL;
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = 0 as varnumber_T;
    let mut name: *mut ::core::ffi::c_char = xstrnsave(funcname, len);
    let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
    let mut fname: *mut ::core::ffi::c_char = fname_trans_sid(
        name,
        &raw mut fname_buf as *mut ::core::ffi::c_char,
        &raw mut tofree,
        &raw mut error,
    );
    let mut is_global: bool = *fname.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 'g' as ::core::ffi::c_int
        && *fname.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int;
    let mut rfname: *mut ::core::ffi::c_char = if is_global as ::core::ffi::c_int != 0 {
        fname.offset(2 as ::core::ffi::c_int as isize)
    } else {
        fname
    };
    let mut fp: *mut ufunc_T = find_func(rfname);
    if fp.is_null() {
        ret = NOTDONE;
    } else if !fp.is_null() && (*fp).uf_flags & FC_DELETED != 0 {
        error = FCERR_DELETED as ::core::ffi::c_int;
    } else if !fp.is_null() {
        let mut argvars: [typval_T; 1] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 1];
        argvars[0 as ::core::ffi::c_int as usize].v_type = VAR_UNKNOWN;
        let mut funcexe: funcexe_T = FUNCEXE_INIT;
        funcexe.fe_evaluate = true_0 != 0;
        error = call_user_func_check(
            fp,
            0 as ::core::ffi::c_int,
            &raw mut argvars as *mut typval_T,
            rettv,
            &raw mut funcexe,
            ::core::ptr::null_mut::<dict_T>(),
        );
        if error == FCERR_NONE as ::core::ffi::c_int {
            ret = OK;
        }
    }
    user_func_error(error, name, false_0 != 0);
    xfree(tofree as *mut ::core::ffi::c_void);
    xfree(name as *mut ::core::ffi::c_void);
    return ret;
}
pub unsafe extern "C" fn printable_func_name(mut fp: *mut ufunc_T) -> *mut ::core::ffi::c_char {
    return if !(*fp).uf_name_exp.is_null() {
        (*fp).uf_name_exp
    } else {
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char
    };
}
unsafe extern "C" fn function_list_modified(
    prev_ht_changed: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    if prev_ht_changed != (*func_hashtab.ptr()).ht_changed {
        emsg(gettext(
            (e_function_list_was_modified.ptr() as *const _) as *const ::core::ffi::c_char,
        ));
        return true_0;
    }
    return false_0;
}
unsafe extern "C" fn list_func_head(
    mut fp: *mut ufunc_T,
    mut indent: bool,
    mut force: bool,
) -> ::core::ffi::c_int {
    let prev_ht_changed: ::core::ffi::c_int = (*func_hashtab.ptr()).ht_changed;
    msg_start();
    if function_list_modified(prev_ht_changed) != 0 {
        return FAIL;
    }
    if indent {
        msg_puts(b"   \0".as_ptr() as *const ::core::ffi::c_char);
    }
    msg_puts(if force as ::core::ffi::c_int != 0 {
        b"function! \0".as_ptr() as *const ::core::ffi::c_char
    } else {
        b"function \0".as_ptr() as *const ::core::ffi::c_char
    });
    if !(*fp).uf_name_exp.is_null() {
        msg_puts((*fp).uf_name_exp);
    } else {
        msg_puts(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char);
    }
    msg_putchar('(' as ::core::ffi::c_int);
    let mut j: ::core::ffi::c_int = 0;
    j = 0 as ::core::ffi::c_int;
    while j < (*fp).uf_args.ga_len {
        if j != 0 {
            msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_puts(*((*fp).uf_args.ga_data as *mut *mut ::core::ffi::c_char).offset(j as isize));
        if j >= (*fp).uf_args.ga_len - (*fp).uf_def_args.ga_len {
            msg_puts(b" = \0".as_ptr() as *const ::core::ffi::c_char);
            msg_puts(
                *((*fp).uf_def_args.ga_data as *mut *mut ::core::ffi::c_char)
                    .offset((j - (*fp).uf_args.ga_len + (*fp).uf_def_args.ga_len) as isize),
            );
        }
        j += 1;
    }
    if (*fp).uf_varargs != 0 {
        if j != 0 {
            msg_puts(b", \0".as_ptr() as *const ::core::ffi::c_char);
        }
        msg_puts(b"...\0".as_ptr() as *const ::core::ffi::c_char);
    }
    msg_putchar(')' as ::core::ffi::c_int);
    if (*fp).uf_flags & FC_ABORT != 0 {
        msg_puts(b" abort\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if (*fp).uf_flags & FC_RANGE != 0 {
        msg_puts(b" range\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if (*fp).uf_flags & FC_DICT != 0 {
        msg_puts(b" dict\0".as_ptr() as *const ::core::ffi::c_char);
    }
    if (*fp).uf_flags & FC_CLOSURE != 0 {
        msg_puts(b" closure\0".as_ptr() as *const ::core::ffi::c_char);
    }
    msg_clr_eos();
    if p_verbose.get() > 0 as OptInt {
        last_set_msg((*fp).uf_script_ctx);
    }
    return OK;
}
pub unsafe extern "C" fn trans_function_name(
    mut pp: *mut *mut ::core::ffi::c_char,
    mut skip: bool,
    mut flags: ::core::ffi::c_int,
    mut fdp: *mut funcdict_T,
    mut partial: *mut *mut partial_T,
) -> *mut ::core::ffi::c_char {
    let mut sid_buflen: size_t = 0;
    let mut sid_buf: [::core::ffi::c_char; 20] = [0; 20];
    let mut name: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    let mut lv: lval_T = lval_T {
        ll_name: ::core::ptr::null::<::core::ffi::c_char>(),
        ll_name_len: 0,
        ll_exp_name: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ll_tv: ::core::ptr::null_mut::<typval_T>(),
        ll_li: ::core::ptr::null_mut::<listitem_T>(),
        ll_list: ::core::ptr::null_mut::<list_T>(),
        ll_range: false,
        ll_empty2: false,
        ll_n1: 0,
        ll_n2: 0,
        ll_dict: ::core::ptr::null_mut::<dict_T>(),
        ll_di: ::core::ptr::null_mut::<dictitem_T>(),
        ll_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        ll_blob: ::core::ptr::null_mut::<blob_T>(),
    };
    if !fdp.is_null() {
        memset(
            fdp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<funcdict_T>(),
        );
    }
    let mut start: *const ::core::ffi::c_char = *pp;
    if *(*pp).offset(0 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int == K_SPECIAL
        && *(*pp).offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int
            == KS_EXTRA
        && *(*pp).offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == KE_SNR as ::core::ffi::c_int
    {
        *pp = (*pp).offset(3 as ::core::ffi::c_int as isize);
        len = get_id_len(pp as *mut *const ::core::ffi::c_char) + 3 as ::core::ffi::c_int;
        return xmemdupz(start as *const ::core::ffi::c_void, len as size_t)
            as *mut ::core::ffi::c_char;
    }
    let mut lead: ::core::ffi::c_int = eval_fname_script(start);
    if lead > 2 as ::core::ffi::c_int {
        start = start.offset(lead as isize);
    }
    let mut end: *const ::core::ffi::c_char = get_lval(
        start as *mut ::core::ffi::c_char,
        ::core::ptr::null_mut::<typval_T>(),
        &raw mut lv,
        false_0 != 0,
        skip,
        flags | GLV_READ_ONLY as ::core::ffi::c_int,
        if lead > 2 as ::core::ffi::c_int {
            0 as ::core::ffi::c_int
        } else {
            FNE_CHECK_START
        },
    );
    '_theend: {
        if end == start {
            if !skip {
                emsg(gettext(
                    b"E129: Function name required\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
        } else if end.is_null()
            || !lv.ll_tv.is_null()
                && (lead > 2 as ::core::ffi::c_int || lv.ll_range as ::core::ffi::c_int != 0)
        {
            if !aborting() {
                if !end.is_null() {
                    semsg(
                        gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                        start,
                    );
                }
            } else {
                *pp = find_name_end(
                    start,
                    ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                    ::core::ptr::null_mut::<*const ::core::ffi::c_char>(),
                    FNE_INCL_BR,
                ) as *mut ::core::ffi::c_char;
            }
        } else if !lv.ll_tv.is_null() {
            if !fdp.is_null() {
                (*fdp).fd_dict = lv.ll_dict;
                (*fdp).fd_newkey = lv.ll_newkey;
                lv.ll_newkey = ::core::ptr::null_mut::<::core::ffi::c_char>();
                (*fdp).fd_di = lv.ll_di;
            }
            if (*lv.ll_tv).v_type as ::core::ffi::c_uint
                == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                && !(*lv.ll_tv).vval.v_string.is_null()
            {
                name = xstrdup((*lv.ll_tv).vval.v_string);
                *pp = end as *mut ::core::ffi::c_char;
            } else if (*lv.ll_tv).v_type as ::core::ffi::c_uint
                == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
                && !(*lv.ll_tv).vval.v_partial.is_null()
            {
                if is_luafunc((*lv.ll_tv).vval.v_partial) as ::core::ffi::c_int != 0
                    && *end as ::core::ffi::c_int == '.' as ::core::ffi::c_int
                {
                    len = check_luafunc_name(
                        end.offset(1 as ::core::ffi::c_int as isize),
                        true_0 != 0,
                    );
                    if len == 0 as ::core::ffi::c_int {
                        semsg(
                            &raw const e_invexpr2 as *const ::core::ffi::c_char,
                            b"v:lua\0".as_ptr() as *const ::core::ffi::c_char,
                        );
                        break '_theend;
                    } else {
                        name = xmallocz(len as size_t) as *mut ::core::ffi::c_char;
                        memcpy(
                            name as *mut ::core::ffi::c_void,
                            end.offset(1 as ::core::ffi::c_int as isize)
                                as *const ::core::ffi::c_void,
                            len as size_t,
                        );
                        *pp = (end as *mut ::core::ffi::c_char)
                            .offset(1 as ::core::ffi::c_int as isize)
                            .offset(len as isize);
                    }
                } else {
                    name = xstrdup(partial_name((*lv.ll_tv).vval.v_partial));
                    *pp = end as *mut ::core::ffi::c_char;
                }
                if !partial.is_null() {
                    *partial = (*lv.ll_tv).vval.v_partial;
                }
            } else {
                if !skip
                    && flags & TFN_QUIET as ::core::ffi::c_int == 0
                    && (fdp.is_null() || lv.ll_dict.is_null() || (*fdp).fd_newkey.is_null())
                {
                    emsg(gettext(e_funcref.get()));
                } else {
                    *pp = end as *mut ::core::ffi::c_char;
                }
                name = ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
        } else if lv.ll_name.is_null() {
            *pp = end as *mut ::core::ffi::c_char;
        } else {
            if !lv.ll_exp_name.is_null() {
                len = strlen(lv.ll_exp_name) as ::core::ffi::c_int;
                name = deref_func_name(
                    lv.ll_exp_name,
                    &raw mut len,
                    partial,
                    flags & TFN_NO_AUTOLOAD as ::core::ffi::c_int != 0,
                    ::core::ptr::null_mut::<bool>(),
                );
                if name == lv.ll_exp_name {
                    name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            } else if flags & TFN_NO_DEREF as ::core::ffi::c_int == 0 {
                len = end.offset_from(*pp) as ::core::ffi::c_int;
                name = deref_func_name(
                    *pp,
                    &raw mut len,
                    partial,
                    flags & TFN_NO_AUTOLOAD as ::core::ffi::c_int != 0,
                    ::core::ptr::null_mut::<bool>(),
                );
                if name == *pp {
                    name = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
            if !name.is_null() {
                name = xstrdup(name);
                *pp = end as *mut ::core::ffi::c_char;
                if strncmp(
                    name,
                    b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                    5 as size_t,
                ) == 0 as ::core::ffi::c_int
                {
                    *name.offset(0 as ::core::ffi::c_int as isize) =
                        K_SPECIAL as ::core::ffi::c_char;
                    *name.offset(1 as ::core::ffi::c_int as isize) =
                        KS_EXTRA as ::core::ffi::c_char;
                    *name.offset(2 as ::core::ffi::c_int as isize) =
                        KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char;
                    memmove(
                        name.offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                        name.offset(5 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                        strlen(name.offset(5 as ::core::ffi::c_int as isize))
                            .wrapping_add(1 as size_t),
                    );
                }
            } else {
                if !lv.ll_exp_name.is_null() {
                    len = strlen(lv.ll_exp_name) as ::core::ffi::c_int;
                    if lead <= 2 as ::core::ffi::c_int
                        && lv.ll_name == lv.ll_exp_name as *const ::core::ffi::c_char
                        && lv.ll_name_len >= 2 as size_t
                        && memcmp(
                            lv.ll_name as *const ::core::ffi::c_void,
                            b"s:\0".as_ptr() as *const ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                    {
                        lv.ll_name = lv.ll_name.offset(2 as ::core::ffi::c_int as isize);
                        lv.ll_name_len = lv.ll_name_len.wrapping_sub(2 as size_t);
                        len -= 2 as ::core::ffi::c_int;
                        lead = 2 as ::core::ffi::c_int;
                    }
                } else {
                    if lead == 2 as ::core::ffi::c_int
                        || *lv.ll_name.offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'g' as ::core::ffi::c_int
                            && *lv.ll_name.offset(1 as ::core::ffi::c_int as isize)
                                as ::core::ffi::c_int
                                == ':' as ::core::ffi::c_int
                    {
                        lv.ll_name = lv.ll_name.offset(2 as ::core::ffi::c_int as isize);
                        lv.ll_name_len = lv.ll_name_len.wrapping_sub(2 as size_t);
                    }
                    len = end.offset_from(lv.ll_name) as ::core::ffi::c_int;
                }
                sid_buflen = 0 as size_t;
                sid_buf = [0; 20];
                if skip {
                    lead = 0 as ::core::ffi::c_int;
                } else if lead > 0 as ::core::ffi::c_int {
                    lead = 3 as ::core::ffi::c_int;
                    if !lv.ll_exp_name.is_null()
                        && eval_fname_sid(lv.ll_exp_name) as ::core::ffi::c_int != 0
                        || eval_fname_sid(*pp) as ::core::ffi::c_int != 0
                    {
                        if (*current_sctx.ptr()).sc_sid <= 0 as ::core::ffi::c_int {
                            emsg(gettext(&raw const e_usingsid as *const ::core::ffi::c_char));
                            break '_theend;
                        } else {
                            sid_buflen = snprintf(
                                &raw mut sid_buf as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 20]>(),
                                b"%d_\0".as_ptr() as *const ::core::ffi::c_char,
                                (*current_sctx.ptr()).sc_sid,
                            ) as size_t;
                            lead += sid_buflen as ::core::ffi::c_int;
                        }
                    }
                } else if flags & TFN_INT as ::core::ffi::c_int == 0
                    && builtin_function(lv.ll_name, lv.ll_name_len as ::core::ffi::c_int)
                        as ::core::ffi::c_int
                        != 0
                {
                    semsg(
                        gettext(
                            b"E128: Function name must start with a capital or \"s:\": %s\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        ),
                        start,
                    );
                    break '_theend;
                }
                if !skip
                    && flags & TFN_QUIET as ::core::ffi::c_int == 0
                    && flags & TFN_NO_DEREF as ::core::ffi::c_int == 0
                {
                    let mut cp: *mut ::core::ffi::c_char = xmemrchr(
                        lv.ll_name as *const ::core::ffi::c_void,
                        ':' as uint8_t,
                        lv.ll_name_len,
                    )
                        as *mut ::core::ffi::c_char;
                    if !cp.is_null() && cp < end as *mut ::core::ffi::c_char {
                        semsg(
                            gettext(b"E884: Function name cannot contain a colon: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            start,
                        );
                        break '_theend;
                    }
                }
                name = xmalloc(
                    (len as size_t)
                        .wrapping_add(lead as size_t)
                        .wrapping_add(1 as size_t),
                ) as *mut ::core::ffi::c_char;
                if !skip && lead > 0 as ::core::ffi::c_int {
                    *name.offset(0 as ::core::ffi::c_int as isize) =
                        K_SPECIAL as ::core::ffi::c_char;
                    *name.offset(1 as ::core::ffi::c_int as isize) =
                        KS_EXTRA as ::core::ffi::c_char;
                    *name.offset(2 as ::core::ffi::c_int as isize) =
                        KE_SNR as ::core::ffi::c_int as ::core::ffi::c_char;
                    if sid_buflen > 0 as size_t {
                        memcpy(
                            name.offset(3 as ::core::ffi::c_int as isize)
                                as *mut ::core::ffi::c_void,
                            &raw mut sid_buf as *mut ::core::ffi::c_char
                                as *const ::core::ffi::c_void,
                            sid_buflen,
                        );
                    }
                }
                memmove(
                    name.offset(lead as isize) as *mut ::core::ffi::c_void,
                    lv.ll_name as *const ::core::ffi::c_void,
                    len as size_t,
                );
                *name.offset((lead + len) as isize) = NUL as ::core::ffi::c_char;
                *pp = end as *mut ::core::ffi::c_char;
            }
        }
    }
    clear_lval(&raw mut lv);
    return name;
}
pub unsafe extern "C" fn get_scriptlocal_funcname(
    mut funcname: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    if funcname.is_null() {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if strncmp(
        funcname,
        b"s:\0".as_ptr() as *const ::core::ffi::c_char,
        2 as size_t,
    ) != 0 as ::core::ffi::c_int
        && strncmp(
            funcname,
            b"<SID>\0".as_ptr() as *const ::core::ffi::c_char,
            5 as size_t,
        ) != 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    if !((*current_sctx.ptr()).sc_sid > 0 as ::core::ffi::c_int
        && (*current_sctx.ptr()).sc_sid <= (*script_items.ptr()).ga_len)
    {
        emsg(gettext(&raw const e_usingsid as *const ::core::ffi::c_char));
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    let mut sid_buf: [::core::ffi::c_char; 25] = [0; 25];
    let mut sid_buflen: size_t = snprintf(
        &raw mut sid_buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 25]>(),
        b"<SNR>%d_\0".as_ptr() as *const ::core::ffi::c_char,
        (*current_sctx.ptr()).sc_sid,
    ) as size_t;
    let off: ::core::ffi::c_int = if *funcname as ::core::ffi::c_int == 's' as ::core::ffi::c_int {
        2 as ::core::ffi::c_int
    } else {
        5 as ::core::ffi::c_int
    };
    let mut newnamesize: size_t = sid_buflen
        .wrapping_add(strlen(funcname.offset(off as isize)))
        .wrapping_add(1 as size_t);
    let mut newname: *mut ::core::ffi::c_char = xmalloc(newnamesize) as *mut ::core::ffi::c_char;
    snprintf(
        newname,
        newnamesize,
        b"%s%s\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut sid_buf as *mut ::core::ffi::c_char,
        funcname.offset(off as isize),
    );
    return newname;
}
pub unsafe extern "C" fn save_function_name(
    mut name: *mut *mut ::core::ffi::c_char,
    mut skip: bool,
    mut flags: ::core::ffi::c_int,
    mut fudi: *mut funcdict_T,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = *name;
    let mut saved: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if strncmp(
        p,
        b"<lambda>\0".as_ptr() as *const ::core::ffi::c_char,
        8 as size_t,
    ) == 0 as ::core::ffi::c_int
    {
        p = p.offset(8 as ::core::ffi::c_int as isize);
        getdigits(&raw mut p, false_0 != 0, 0 as intmax_t);
        saved = xmemdupz(
            *name as *const ::core::ffi::c_void,
            p.offset_from(*name) as size_t,
        ) as *mut ::core::ffi::c_char;
        if !fudi.is_null() {
            memset(
                fudi as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<funcdict_T>(),
            );
        }
    } else {
        saved = trans_function_name(
            &raw mut p,
            skip,
            flags,
            fudi,
            ::core::ptr::null_mut::<*mut partial_T>(),
        );
    }
    *name = p;
    return saved;
}
unsafe extern "C" fn list_functions(mut regmatch: *mut regmatch_T) {
    let prev_ht_changed: ::core::ffi::c_int = (*func_hashtab.ptr()).ht_changed;
    let mut todo: size_t = (*func_hashtab.ptr()).ht_used;
    let ht_array: *const hashitem_T = (*func_hashtab.ptr()).ht_array;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    let mut hi: *const hashitem_T = ht_array;
    while todo > 0 as size_t && !got_int.get() {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            let mut fp: *mut ufunc_T =
                (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
            todo = todo.wrapping_sub(1);
            if if regmatch.is_null() {
                (!message_filtered(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                    && !func_name_refcount(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char))
                    as ::core::ffi::c_int
            } else {
                (*(*__ctype_b_loc()).offset(
                    *(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char) as uint8_t
                        as ::core::ffi::c_int as isize,
                ) as ::core::ffi::c_int
                    & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
                    == 0
                    && vim_regexec(
                        regmatch,
                        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                        0 as colnr_T,
                    ) as ::core::ffi::c_int
                        != 0) as ::core::ffi::c_int
            } != 0
            {
                if list_func_head(fp, false_0 != 0, false_0 != 0) == FAIL {
                    return;
                }
                if function_list_modified(prev_ht_changed) != 0 {
                    return;
                }
            }
        }
        hi = hi.offset(1);
    }
}
unsafe extern "C" fn list_functions_matching_pat(
    mut eap: *mut exarg_T,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = skip_regexp(
        (*eap).arg.offset(1 as ::core::ffi::c_int as isize),
        '/' as ::core::ffi::c_int,
        true_0,
    );
    if (*eap).skip == 0 {
        let mut regmatch: regmatch_T = regmatch_T {
            regprog: ::core::ptr::null_mut::<regprog_T>(),
            startp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            endp: [::core::ptr::null_mut::<::core::ffi::c_char>(); 10],
            rm_matchcol: 0,
            rm_ic: false,
        };
        let mut c: ::core::ffi::c_char = *p;
        *p = NUL as ::core::ffi::c_char;
        regmatch.regprog = vim_regcomp(
            (*eap).arg.offset(1 as ::core::ffi::c_int as isize),
            RE_MAGIC,
        );
        *p = c;
        if !regmatch.regprog.is_null() {
            regmatch.rm_ic = p_ic.get() != 0;
            list_functions(&raw mut regmatch);
            vim_regfree(regmatch.regprog);
        }
    }
    if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
        p = p.offset(1);
    }
    return p;
}
unsafe extern "C" fn list_one_function(
    mut eap: *mut exarg_T,
    mut name: *mut ::core::ffi::c_char,
    mut p: *mut ::core::ffi::c_char,
) -> *mut ufunc_T {
    if ends_excmd(*skipwhite(p) as ::core::ffi::c_int) == 0 {
        semsg(
            gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
            p,
        );
        return ::core::ptr::null_mut::<ufunc_T>();
    }
    (*eap).nextcmd = check_nextcmd(p);
    if !(*eap).nextcmd.is_null() {
        *p = NUL as ::core::ffi::c_char;
    }
    if (*eap).skip != 0 || got_int.get() as ::core::ffi::c_int != 0 {
        return ::core::ptr::null_mut::<ufunc_T>();
    }
    let mut fp: *mut ufunc_T = find_func(name);
    if fp.is_null() {
        emsg_funcname(
            b"E123: Undefined function: %s\0".as_ptr() as *const ::core::ffi::c_char,
            name,
        );
        return ::core::ptr::null_mut::<ufunc_T>();
    }
    let prev_ht_changed: ::core::ffi::c_int = (*func_hashtab.ptr()).ht_changed;
    msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
    if list_func_head(fp, (*eap).forceit == 0, (*eap).forceit != 0) != OK {
        return fp;
    }
    let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while j < (*fp).uf_lines.ga_len && !got_int.get() {
        if !(*((*fp).uf_lines.ga_data as *mut *mut ::core::ffi::c_char).offset(j as isize))
            .is_null()
        {
            msg_putchar('\n' as ::core::ffi::c_int);
            if (*eap).forceit == 0 {
                msg_outnum(j + 1 as ::core::ffi::c_int);
                if j < 9 as ::core::ffi::c_int {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                if j < 99 as ::core::ffi::c_int {
                    msg_putchar(' ' as ::core::ffi::c_int);
                }
                if function_list_modified(prev_ht_changed) != 0 {
                    break;
                }
            }
            msg_prt_line(
                *((*fp).uf_lines.ga_data as *mut *mut ::core::ffi::c_char).offset(j as isize),
                false_0 != 0,
            );
            line_breakcheck();
        }
        j += 1;
    }
    if !got_int.get() {
        msg_putchar('\n' as ::core::ffi::c_int);
        if function_list_modified(prev_ht_changed) == 0 {
            msg_puts(if (*eap).forceit != 0 {
                b"endfunction\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"   endfunction\0".as_ptr() as *const ::core::ffi::c_char
            });
        }
    }
    return fp;
}
pub const MAX_FUNC_NESTING: ::core::ffi::c_int = 50 as ::core::ffi::c_int;
unsafe extern "C" fn get_function_body(
    mut eap: *mut exarg_T,
    mut newlines: *mut garray_T,
    mut line_arg_in: *mut ::core::ffi::c_char,
    mut line_to_free: *mut *mut ::core::ffi::c_char,
    mut show_block: bool,
) -> ::core::ffi::c_int {
    let mut saved_wait_return: bool = need_wait_return.get();
    let mut line_arg: *mut ::core::ffi::c_char = line_arg_in;
    let mut indent: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
    let mut nesting: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut skip_until: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut ret: ::core::ffi::c_int = FAIL;
    let mut is_heredoc: bool = false_0 != 0;
    let mut heredoc_trimmed: *mut ::core::ffi::c_char =
        ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut heredoc_trimmedlen: size_t = 0 as size_t;
    let mut do_concat: bool = true_0 != 0;
    '_theend: {
        loop {
            if KeyTyped.get() {
                msg_scroll.set(true_0);
                saved_wait_return = false_0 != 0;
            }
            need_wait_return.set(false_0 != 0);
            let mut theline: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            if !line_arg.is_null() {
                theline = line_arg;
                p = vim_strchr(theline, '\n' as ::core::ffi::c_int);
                if p.is_null() {
                    line_arg = line_arg.offset(strlen(line_arg) as isize);
                } else {
                    *p = NUL as ::core::ffi::c_char;
                    line_arg = p.offset(1 as ::core::ffi::c_int as isize);
                }
            } else {
                xfree(*line_to_free as *mut ::core::ffi::c_void);
                if (*eap).ea_getline.is_none() {
                    theline = getcmdline(
                        ':' as ::core::ffi::c_int,
                        0 as ::core::ffi::c_int,
                        indent,
                        do_concat,
                    );
                } else {
                    theline = (*eap).ea_getline.expect("non-null function pointer")(
                        ':' as ::core::ffi::c_int,
                        (*eap).cookie,
                        indent,
                        do_concat,
                    );
                }
                *line_to_free = theline;
            }
            if KeyTyped.get() {
                lines_left.set(Rows.get() - 1 as ::core::ffi::c_int);
            }
            if theline.is_null() {
                if !skip_until.is_null() {
                    semsg(
                        gettext(
                            (e_missing_heredoc_end_marker_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        skip_until,
                    );
                } else {
                    emsg(gettext(
                        b"E126: Missing :endfunction\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                }
                break '_theend;
            } else {
                if show_block {
                    '_c2rust_label: {
                        if indent >= 0 as ::core::ffi::c_int {
                        } else {
                            __assert_fail(
                                b"indent >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/eval/userfunc.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2419 as ::core::ffi::c_uint,
                                b"int get_function_body(exarg_T *, garray_T *, char *, char **, _Bool)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    ui_ext_cmdline_block_append(indent as size_t, theline);
                }
                let mut sourcing_lnum_off: linenr_T =
                    get_sourced_lnum((*eap).ea_getline, (*eap).cookie);
                if (*((*exestack.ptr()).ga_data as *mut estack_T)
                    .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                .es_lnum
                    < sourcing_lnum_off
                {
                    sourcing_lnum_off -= (*((*exestack.ptr()).ga_data as *mut estack_T)
                        .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                    .es_lnum;
                } else {
                    sourcing_lnum_off = 0 as ::core::ffi::c_int as linenr_T;
                }
                if !skip_until.is_null() {
                    if heredoc_trimmed.is_null()
                        || is_heredoc as ::core::ffi::c_int != 0 && skipwhite(theline) == theline
                        || strncmp(theline, heredoc_trimmed, heredoc_trimmedlen)
                            == 0 as ::core::ffi::c_int
                    {
                        if heredoc_trimmed.is_null() {
                            p = theline;
                        } else if is_heredoc {
                            p = if skipwhite(theline) == theline {
                                theline
                            } else {
                                theline.offset(heredoc_trimmedlen as isize)
                            };
                        } else {
                            p = theline.offset(heredoc_trimmedlen as isize);
                        }
                        if strcmp(p, skip_until) == 0 as ::core::ffi::c_int {
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                &raw mut skip_until as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL;
                            let _ = *ptr_;
                            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                &raw mut heredoc_trimmed as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr__0);
                            *ptr__0 = NULL;
                            let _ = *ptr__0;
                            heredoc_trimmedlen = 0 as size_t;
                            do_concat = true_0 != 0;
                            is_heredoc = false_0 != 0;
                        }
                    }
                } else {
                    p = theline;
                    while ascii_iswhite(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        || *p as ::core::ffi::c_int == ':' as ::core::ffi::c_int
                    {
                        p = p.offset(1);
                    }
                    if checkforcmd(
                        &raw mut p,
                        b"endfunction\0".as_ptr() as *const ::core::ffi::c_char,
                        4 as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                        != 0
                        && {
                            let c2rust_fresh12 = nesting;
                            nesting = nesting - 1;
                            c2rust_fresh12 == 0 as ::core::ffi::c_int
                        }
                    {
                        if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                            p = p.offset(1);
                        }
                        let mut nextcmd: *mut ::core::ffi::c_char =
                            ::core::ptr::null_mut::<::core::ffi::c_char>();
                        if *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
                            nextcmd = p.offset(1 as ::core::ffi::c_int as isize);
                        } else if !line_arg.is_null()
                            && *skipwhite(line_arg) as ::core::ffi::c_int != NUL
                        {
                            nextcmd = line_arg;
                        } else if *p as ::core::ffi::c_int != NUL
                            && *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int
                            && p_verbose.get() > 0 as OptInt
                        {
                            swmsg(
                                true_0 != 0,
                                gettext(b"W22: Text found after :endfunction: %s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                p,
                            );
                        }
                        if !nextcmd.is_null() {
                            (*eap).nextcmd = nextcmd;
                            if !(*line_to_free).is_null() {
                                xfree(*(*eap).cmdlinep as *mut ::core::ffi::c_void);
                                *(*eap).cmdlinep = *line_to_free;
                                *line_to_free = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            }
                        }
                        break;
                    } else {
                        if indent > 2 as ::core::ffi::c_int
                            && strncmp(
                                p,
                                b"end\0".as_ptr() as *const ::core::ffi::c_char,
                                3 as size_t,
                            ) == 0 as ::core::ffi::c_int
                        {
                            indent -= 2 as ::core::ffi::c_int;
                        } else if strncmp(
                            p,
                            b"if\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as size_t,
                        ) == 0 as ::core::ffi::c_int
                            || strncmp(
                                p,
                                b"wh\0".as_ptr() as *const ::core::ffi::c_char,
                                2 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            || strncmp(
                                p,
                                b"for\0".as_ptr() as *const ::core::ffi::c_char,
                                3 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            || strncmp(
                                p,
                                b"try\0".as_ptr() as *const ::core::ffi::c_char,
                                3 as size_t,
                            ) == 0 as ::core::ffi::c_int
                        {
                            indent += 2 as ::core::ffi::c_int;
                        }
                        if checkforcmd(
                            &raw mut p,
                            b"function\0".as_ptr() as *const ::core::ffi::c_char,
                            2 as ::core::ffi::c_int,
                        ) {
                            if *p as ::core::ffi::c_int == '!' as ::core::ffi::c_int {
                                p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
                            }
                            p = p.offset(eval_fname_script(p) as isize);
                            xfree(trans_function_name(
                                &raw mut p,
                                true_0 != 0,
                                0 as ::core::ffi::c_int,
                                ::core::ptr::null_mut::<funcdict_T>(),
                                ::core::ptr::null_mut::<*mut partial_T>(),
                            ) as *mut ::core::ffi::c_void);
                            if *skipwhite(p) as ::core::ffi::c_int == '(' as ::core::ffi::c_int {
                                if nesting == MAX_FUNC_NESTING - 1 as ::core::ffi::c_int {
                                    emsg(gettext(
                                        (e_function_nesting_too_deep.ptr() as *const _)
                                            as *const ::core::ffi::c_char,
                                    ));
                                } else {
                                    nesting += 1;
                                    indent += 2 as ::core::ffi::c_int;
                                }
                            }
                        }
                        p = skip_range(p, ::core::ptr::null_mut::<::core::ffi::c_int>());
                        let tp: *mut ::core::ffi::c_char = p;
                        if (checkforcmd(
                            &raw mut p,
                            b"append\0".as_ptr() as *const ::core::ffi::c_char,
                            1 as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                            != 0
                            || checkforcmd(
                                &raw mut p,
                                b"change\0".as_ptr() as *const ::core::ffi::c_char,
                                1 as ::core::ffi::c_int,
                            ) as ::core::ffi::c_int
                                != 0
                            || checkforcmd(
                                &raw mut p,
                                b"insert\0".as_ptr() as *const ::core::ffi::c_char,
                                1 as ::core::ffi::c_int,
                            ) as ::core::ffi::c_int
                                != 0)
                            && (*p as ::core::ffi::c_int == '!' as ::core::ffi::c_int
                                || *p as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                                || ascii_iswhite_nl_or_nul(*p as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                                    != 0)
                        {
                            skip_until = xmemdupz(
                                b".\0".as_ptr() as *const ::core::ffi::c_char
                                    as *const ::core::ffi::c_void,
                                1 as size_t,
                            ) as *mut ::core::ffi::c_char;
                        } else {
                            p = tp;
                        }
                        arg = skipwhite(skiptowhite(p));
                        if *arg.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '<' as ::core::ffi::c_int
                            && *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '<' as ::core::ffi::c_int
                            && (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == 'p' as ::core::ffi::c_int
                                && *p.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'y' as ::core::ffi::c_int
                                && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_uint
                                    >= 'A' as ::core::ffi::c_uint
                                    && *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        <= 'Z' as ::core::ffi::c_uint
                                    || *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'a' as ::core::ffi::c_uint
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'z' as ::core::ffi::c_uint
                                    || ascii_isdigit(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0)
                                    || *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 't' as ::core::ffi::c_int
                                    || (*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '3' as ::core::ffi::c_int
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'x' as ::core::ffi::c_int)
                                        && !(*p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'A' as ::core::ffi::c_uint
                                            && *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'Z' as ::core::ffi::c_uint
                                            || *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                >= 'a' as ::core::ffi::c_uint
                                                && *p.offset(3 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_uint
                                                    <= 'z' as ::core::ffi::c_uint))
                                || *p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'p' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'e' as ::core::ffi::c_int
                                    && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint)
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'r' as ::core::ffi::c_int)
                                || *p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 't' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'c' as ::core::ffi::c_int
                                    && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint)
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'l' as ::core::ffi::c_int)
                                || *p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'l' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'u' as ::core::ffi::c_int
                                    && *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'a' as ::core::ffi::c_int
                                    && !(*p.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint)
                                || *p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'r' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'u' as ::core::ffi::c_int
                                    && *p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'b' as ::core::ffi::c_int
                                    && (!(*p.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(3 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint)
                                        || *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'y' as ::core::ffi::c_int)
                                || *p.offset(0 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == 'm' as ::core::ffi::c_int
                                    && *p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == 'z' as ::core::ffi::c_int
                                    && (!(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_uint
                                        >= 'A' as ::core::ffi::c_uint
                                        && *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            <= 'Z' as ::core::ffi::c_uint
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_uint
                                            >= 'a' as ::core::ffi::c_uint
                                            && *p.offset(2 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_uint
                                                <= 'z' as ::core::ffi::c_uint)
                                        || *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 's' as ::core::ffi::c_int))
                        {
                            p = skipwhite(arg.offset(2 as ::core::ffi::c_int as isize));
                            if strncmp(
                                p,
                                b"trim\0".as_ptr() as *const ::core::ffi::c_char,
                                4 as size_t,
                            ) == 0 as ::core::ffi::c_int
                                && (*p.offset(4 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int
                                    == NUL
                                    || ascii_iswhite(*p.offset(4 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0)
                            {
                                p = skipwhite(p.offset(4 as ::core::ffi::c_int as isize));
                                heredoc_trimmedlen =
                                    skipwhite(theline).offset_from(theline) as size_t;
                                heredoc_trimmed = xmemdupz(
                                    theline as *const ::core::ffi::c_void,
                                    heredoc_trimmedlen,
                                )
                                    as *mut ::core::ffi::c_char;
                            }
                            if *p as ::core::ffi::c_int == NUL {
                                skip_until = xmemdupz(
                                    b".\0".as_ptr() as *const ::core::ffi::c_char
                                        as *const ::core::ffi::c_void,
                                    1 as size_t,
                                )
                                    as *mut ::core::ffi::c_char;
                            } else {
                                skip_until = xmemdupz(
                                    p as *const ::core::ffi::c_void,
                                    skiptowhite(p).offset_from(p) as size_t,
                                )
                                    as *mut ::core::ffi::c_char;
                            }
                            do_concat = false_0 != 0;
                            is_heredoc = true_0 != 0;
                        }
                        if !is_heredoc {
                            arg = p;
                            if checkforcmd(
                                &raw mut arg,
                                b"let\0".as_ptr() as *const ::core::ffi::c_char,
                                2 as ::core::ffi::c_int,
                            ) as ::core::ffi::c_int
                                != 0
                                || checkforcmd(
                                    &raw mut p,
                                    b"const\0".as_ptr() as *const ::core::ffi::c_char,
                                    5 as ::core::ffi::c_int,
                                ) as ::core::ffi::c_int
                                    != 0
                            {
                                let mut var_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                let mut semicolon: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                arg = skip_var_list(
                                    arg,
                                    &raw mut var_count,
                                    &raw mut semicolon,
                                    true_0 != 0,
                                ) as *mut ::core::ffi::c_char;
                                if !arg.is_null() {
                                    arg = skipwhite(arg);
                                }
                                if !arg.is_null()
                                    && strncmp(
                                        arg,
                                        b"=<<\0".as_ptr() as *const ::core::ffi::c_char,
                                        3 as size_t,
                                    ) == 0 as ::core::ffi::c_int
                                {
                                    p = skipwhite(arg.offset(3 as ::core::ffi::c_int as isize));
                                    let mut has_trim: bool = false_0 != 0;
                                    loop {
                                        if strncmp(
                                            p,
                                            b"trim\0".as_ptr() as *const ::core::ffi::c_char,
                                            4 as size_t,
                                        ) == 0 as ::core::ffi::c_int
                                            && (*p.offset(4 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == NUL
                                                || ascii_iswhite(
                                                    *p.offset(4 as ::core::ffi::c_int as isize)
                                                        as ::core::ffi::c_int,
                                                )
                                                    as ::core::ffi::c_int
                                                    != 0)
                                        {
                                            p = skipwhite(
                                                p.offset(4 as ::core::ffi::c_int as isize),
                                            );
                                            has_trim = true_0 != 0;
                                        } else {
                                            if !(strncmp(
                                                p,
                                                b"eval\0".as_ptr() as *const ::core::ffi::c_char,
                                                4 as size_t,
                                            ) == 0 as ::core::ffi::c_int
                                                && (*p.offset(4 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int
                                                    == NUL
                                                    || ascii_iswhite(
                                                        *p.offset(4 as ::core::ffi::c_int as isize)
                                                            as ::core::ffi::c_int,
                                                    )
                                                        as ::core::ffi::c_int
                                                        != 0))
                                            {
                                                break;
                                            }
                                            p = skipwhite(
                                                p.offset(4 as ::core::ffi::c_int as isize),
                                            );
                                        }
                                    }
                                    if has_trim {
                                        heredoc_trimmedlen =
                                            skipwhite(theline).offset_from(theline) as size_t;
                                        heredoc_trimmed = xmemdupz(
                                            theline as *const ::core::ffi::c_void,
                                            heredoc_trimmedlen,
                                        )
                                            as *mut ::core::ffi::c_char;
                                    }
                                    let mut ptr__1: *mut *mut ::core::ffi::c_void =
                                        &raw mut skip_until as *mut *mut ::core::ffi::c_void;
                                    xfree(*ptr__1);
                                    *ptr__1 = NULL;
                                    let _ = *ptr__1;
                                    skip_until = xmemdupz(
                                        p as *const ::core::ffi::c_void,
                                        skiptowhite(p).offset_from(p) as size_t,
                                    )
                                        as *mut ::core::ffi::c_char;
                                    do_concat = false_0 != 0;
                                    is_heredoc = true_0 != 0;
                                }
                            }
                        }
                    }
                }
                ga_grow(
                    newlines,
                    1 as ::core::ffi::c_int + sourcing_lnum_off as ::core::ffi::c_int,
                );
                p = xstrdup(theline);
                let c2rust_fresh13 = (*newlines).ga_len;
                (*newlines).ga_len = (*newlines).ga_len + 1;
                let c2rust_lvalue_ptr = &raw mut *((*newlines).ga_data
                    as *mut *mut ::core::ffi::c_char)
                    .offset(c2rust_fresh13 as isize);
                *c2rust_lvalue_ptr = p;
                loop {
                    let c2rust_fresh14 = sourcing_lnum_off;
                    sourcing_lnum_off = sourcing_lnum_off - 1;
                    if c2rust_fresh14 <= 0 as linenr_T {
                        break;
                    }
                    let c2rust_fresh15 = (*newlines).ga_len;
                    (*newlines).ga_len = (*newlines).ga_len + 1;
                    let c2rust_lvalue_ptr_0 = &raw mut *((*newlines).ga_data
                        as *mut *mut ::core::ffi::c_char)
                        .offset(c2rust_fresh15 as isize);
                    *c2rust_lvalue_ptr_0 = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                if !line_arg.is_null() && *line_arg as ::core::ffi::c_int == NUL {
                    line_arg = ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
            }
        }
        if did_emsg.get() == 0 {
            ret = OK;
        }
    }
    xfree(skip_until as *mut ::core::ffi::c_void);
    xfree(heredoc_trimmed as *mut ::core::ffi::c_void);
    need_wait_return.set(
        need_wait_return.get() as ::core::ffi::c_int | saved_wait_return as ::core::ffi::c_int != 0,
    );
    return ret;
}
pub unsafe fn ex_function(mut eap: *mut exarg_T) {
    let mut sourcing_lnum_top: linenr_T = 0;
    let mut namelen: size_t = 0;
    let mut line_to_free: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut line_arg: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut newargs: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut default_args: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut newlines: garray_T = garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    };
    let mut varargs: ::core::ffi::c_int = false_0;
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
    let mut free_fp: bool = false_0 != 0;
    let mut overwrite: bool = false_0 != 0;
    let mut fudi: funcdict_T = funcdict_T {
        fd_dict: ::core::ptr::null_mut::<dict_T>(),
        fd_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fd_di: ::core::ptr::null_mut::<dictitem_T>(),
    };
    static func_nr: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0 as ::core::ffi::c_int);
    let mut ht: *mut hashtab_T = ::core::ptr::null_mut::<hashtab_T>();
    let mut show_block: bool = false_0 != 0;
    if ends_excmd(*(*eap).arg as ::core::ffi::c_int) != 0 {
        if (*eap).skip == 0 {
            list_functions(::core::ptr::null_mut::<regmatch_T>());
        }
        (*eap).nextcmd = check_nextcmd((*eap).arg);
        return;
    }
    if *(*eap).arg as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
        let mut p: *mut ::core::ffi::c_char = list_functions_matching_pat(eap);
        (*eap).nextcmd = check_nextcmd(p);
        return;
    }
    let mut p_0: *mut ::core::ffi::c_char = (*eap).arg;
    let mut name: *mut ::core::ffi::c_char = save_function_name(
        &raw mut p_0,
        (*eap).skip != 0,
        TFN_NO_AUTOLOAD as ::core::ffi::c_int,
        &raw mut fudi,
    );
    let mut paren: ::core::ffi::c_int =
        !vim_strchr(p_0, '(' as ::core::ffi::c_int).is_null() as ::core::ffi::c_int;
    if name.is_null() && (fudi.fd_dict.is_null() || paren == 0) && (*eap).skip == 0 {
        if !aborting() {
            if !fudi.fd_newkey.is_null() {
                semsg(
                    gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                    fudi.fd_newkey,
                );
            }
            xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
            return;
        }
        (*eap).skip = true_0;
    }
    let saved_did_emsg: ::core::ffi::c_int = did_emsg.get();
    did_emsg.set(false_0);
    '_ret_free: {
        if paren == 0 {
            fp = list_one_function(eap, name, p_0);
        } else {
            p_0 = skipwhite(p_0);
            if *p_0 as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
                if (*eap).skip == 0 {
                    semsg(
                        gettext(b"E124: Missing '(': %s\0".as_ptr() as *const ::core::ffi::c_char),
                        (*eap).arg,
                    );
                    break '_ret_free;
                } else if !vim_strchr(p_0, '(' as ::core::ffi::c_int).is_null() {
                    p_0 = vim_strchr(p_0, '(' as ::core::ffi::c_int);
                }
            }
            p_0 = skipwhite(p_0.offset(1 as ::core::ffi::c_int as isize));
            ga_init(
                &raw mut newargs,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
            );
            ga_init(
                &raw mut newlines,
                ::core::mem::size_of::<*mut ::core::ffi::c_char>() as ::core::ffi::c_int,
                3 as ::core::ffi::c_int,
            );
            if (*eap).skip == 0 {
                if !name.is_null() {
                    arg = name;
                } else {
                    arg = fudi.fd_newkey;
                }
                if !arg.is_null() && (fudi.fd_di.is_null() || !tv_is_func((*fudi.fd_di).di_tv)) {
                    let mut name_base: *mut ::core::ffi::c_char = arg;
                    if arg != fudi.fd_newkey {
                        if *arg as uint8_t as ::core::ffi::c_int == K_SPECIAL {
                            name_base = vim_strchr(arg, '_' as ::core::ffi::c_int);
                            if name_base.is_null() {
                                name_base = arg.offset(3 as ::core::ffi::c_int as isize);
                            } else {
                                name_base = name_base.offset(1);
                            }
                        }
                        let mut i: ::core::ffi::c_int = 0;
                        i = 0 as ::core::ffi::c_int;
                        while *name_base.offset(i as isize) as ::core::ffi::c_int != NUL
                            && (if i == 0 as ::core::ffi::c_int {
                                eval_isnamec1(*name_base.offset(i as isize) as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                            } else {
                                eval_isnamec(*name_base.offset(i as isize) as ::core::ffi::c_int)
                                    as ::core::ffi::c_int
                            }) != 0
                        {
                            i += 1;
                        }
                        if *name_base.offset(i as isize) as ::core::ffi::c_int != NUL {
                            emsg_funcname(&raw const e_invarg2 as *const ::core::ffi::c_char, arg);
                            break '_ret_free;
                        }
                    }
                }
                if !fudi.fd_dict.is_null()
                    && (*fudi.fd_dict).dv_scope as ::core::ffi::c_uint
                        == VAR_DEF_SCOPE as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    emsg(gettext(
                        b"E862: Cannot use g: here\0".as_ptr() as *const ::core::ffi::c_char
                    ));
                    break '_ret_free;
                }
            }
            '_errret_keep: {
                if get_function_args(
                    &raw mut p_0,
                    ')' as ::core::ffi::c_char,
                    &raw mut newargs,
                    &raw mut varargs,
                    &raw mut default_args,
                    (*eap).skip != 0,
                ) != FAIL
                {
                    if KeyTyped.get() as ::core::ffi::c_int != 0
                        && ui_has(kUICmdline) as ::core::ffi::c_int != 0
                    {
                        show_block = true_0 != 0;
                        ui_ext_cmdline_block_append(0 as size_t, (*eap).cmd);
                    }
                    '_erret: {
                        loop {
                            p_0 = skipwhite(p_0);
                            if strncmp(
                                p_0,
                                b"range\0".as_ptr() as *const ::core::ffi::c_char,
                                5 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            {
                                flags |= FC_RANGE;
                                p_0 = p_0.offset(5 as ::core::ffi::c_int as isize);
                            } else if strncmp(
                                p_0,
                                b"dict\0".as_ptr() as *const ::core::ffi::c_char,
                                4 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            {
                                flags |= FC_DICT;
                                p_0 = p_0.offset(4 as ::core::ffi::c_int as isize);
                            } else if strncmp(
                                p_0,
                                b"abort\0".as_ptr() as *const ::core::ffi::c_char,
                                5 as size_t,
                            ) == 0 as ::core::ffi::c_int
                            {
                                flags |= FC_ABORT;
                                p_0 = p_0.offset(5 as ::core::ffi::c_int as isize);
                            } else {
                                if strncmp(
                                    p_0,
                                    b"closure\0".as_ptr() as *const ::core::ffi::c_char,
                                    7 as size_t,
                                ) != 0 as ::core::ffi::c_int
                                {
                                    break;
                                }
                                flags |= FC_CLOSURE;
                                p_0 = p_0.offset(7 as ::core::ffi::c_int as isize);
                                if !(*current_funccal.ptr()).is_null() {
                                    continue;
                                }
                                emsg_funcname(
                                    b"E932: Closure function should not be at top level: %s\0"
                                        .as_ptr()
                                        as *const ::core::ffi::c_char,
                                    if name.is_null() {
                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                    } else {
                                        name as *const ::core::ffi::c_char
                                    },
                                );
                                break '_erret;
                            }
                        }
                        if *p_0 as ::core::ffi::c_int == '\n' as ::core::ffi::c_int {
                            line_arg = p_0.offset(1 as ::core::ffi::c_int as isize);
                        } else if *p_0 as ::core::ffi::c_int != NUL
                            && *p_0 as ::core::ffi::c_int != '"' as ::core::ffi::c_int
                            && (*eap).skip == 0
                            && did_emsg.get() == 0
                        {
                            semsg(
                                gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                                p_0,
                            );
                        }
                        if KeyTyped.get() {
                            if (*eap).skip == 0 && (*eap).forceit == 0 {
                                if !fudi.fd_dict.is_null() && fudi.fd_newkey.is_null() {
                                    emsg(gettext(e_funcdict.get()));
                                } else if !name.is_null() && !find_func(name).is_null() {
                                    emsg_funcname(e_funcexts.get(), name);
                                }
                            }
                            if (*eap).skip == 0 && did_emsg.get() != 0 {
                                break '_erret;
                            } else {
                                if !ui_has(kUICmdline) {
                                    msg_putchar('\n' as ::core::ffi::c_int);
                                }
                                cmdline_row.set(msg_row.get());
                            }
                        }
                        sourcing_lnum_top = (*((*exestack.ptr()).ga_data as *mut estack_T)
                            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
                        .es_lnum;
                        if !(get_function_body(
                            eap,
                            &raw mut newlines,
                            line_arg,
                            &raw mut line_to_free,
                            show_block,
                        ) == FAIL
                            || (*eap).skip != 0)
                        {
                            namelen = 0 as size_t;
                            if fudi.fd_dict.is_null() {
                                let mut v: *mut dictitem_T =
                                    find_var(name, strlen(name), &raw mut ht, false_0);
                                if !v.is_null()
                                    && (*v).di_tv.v_type as ::core::ffi::c_uint
                                        == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    emsg_funcname(
                                        b"E707: Function name conflicts with variable: %s\0"
                                            .as_ptr()
                                            as *const ::core::ffi::c_char,
                                        name,
                                    );
                                    break '_erret;
                                } else {
                                    fp = find_func(name);
                                    if !fp.is_null() {
                                        if (*eap).forceit == 0
                                            && ((*fp).uf_script_ctx.sc_sid
                                                != (*current_sctx.ptr()).sc_sid
                                                || (*fp).uf_script_ctx.sc_seq
                                                    == (*current_sctx.ptr()).sc_seq)
                                        {
                                            emsg_funcname(e_funcexts.get(), name);
                                            break '_errret_keep;
                                        } else if (*fp).uf_calls > 0 as ::core::ffi::c_int {
                                            emsg_funcname(
                                                b"E127: Cannot redefine function %s: It is in use\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                name,
                                            );
                                            break '_errret_keep;
                                        } else if (*fp).uf_refcount > 1 as ::core::ffi::c_int {
                                            (*fp).uf_refcount -= 1;
                                            (*fp).uf_flags |= FC_REMOVED;
                                            fp = ::core::ptr::null_mut::<ufunc_T>();
                                            overwrite = true_0 != 0;
                                        } else {
                                            let mut exp_name: *mut ::core::ffi::c_char =
                                                (*fp).uf_name_exp;
                                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                                &raw mut name as *mut *mut ::core::ffi::c_void;
                                            xfree(*ptr_);
                                            *ptr_ = NULL;
                                            let _ = *ptr_;
                                            (*fp).uf_name_exp =
                                                ::core::ptr::null_mut::<::core::ffi::c_char>();
                                            func_clear_items(fp);
                                            (*fp).uf_name_exp = exp_name;
                                            (*fp).uf_profiling = false_0;
                                            (*fp).uf_prof_initialized = false_0;
                                        }
                                    }
                                }
                            } else {
                                let mut numbuf: [::core::ffi::c_char; 65] = [0; 65];
                                fp = ::core::ptr::null_mut::<ufunc_T>();
                                if fudi.fd_newkey.is_null() && (*eap).forceit == 0 {
                                    emsg(gettext(e_funcdict.get()));
                                    break '_erret;
                                } else {
                                    if fudi.fd_di.is_null() {
                                        if value_check_lock(
                                            (*fudi.fd_dict).dv_lock,
                                            (*eap).arg,
                                            TV_CSTRING as size_t,
                                        ) {
                                            break '_erret;
                                        }
                                    } else if value_check_lock(
                                        (*fudi.fd_di).di_tv.v_lock,
                                        (*eap).arg,
                                        TV_CSTRING as size_t,
                                    ) {
                                        break '_erret;
                                    }
                                    xfree(name as *mut ::core::ffi::c_void);
                                    (*func_nr.ptr()) += 1;
                                    namelen = snprintf(
                                        &raw mut numbuf as *mut ::core::ffi::c_char,
                                        ::core::mem::size_of::<[::core::ffi::c_char; 65]>(),
                                        b"%d\0".as_ptr() as *const ::core::ffi::c_char,
                                        func_nr.get(),
                                    ) as size_t;
                                    name = xmemdupz(
                                        &raw mut numbuf as *mut ::core::ffi::c_char
                                            as *const ::core::ffi::c_void,
                                        namelen,
                                    )
                                        as *mut ::core::ffi::c_char;
                                }
                            }
                            if fp.is_null() {
                                if fudi.fd_dict.is_null()
                                    && !vim_strchr(name, AUTOLOAD_CHAR).is_null()
                                {
                                    let mut j: ::core::ffi::c_int = FAIL;
                                    if !(*((*exestack.ptr()).ga_data as *mut estack_T).offset(
                                        ((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int)
                                            as isize,
                                    ))
                                    .es_name
                                    .is_null()
                                    {
                                        let mut scriptname: *mut ::core::ffi::c_char =
                                            autoload_name(name, strlen(name));
                                        p_0 = vim_strchr(scriptname, '/' as ::core::ffi::c_int);
                                        let mut plen: ::core::ffi::c_int =
                                            strlen(p_0) as ::core::ffi::c_int;
                                        let mut slen: ::core::ffi::c_int = strlen(
                                            (*((*exestack.ptr()).ga_data as *mut estack_T).offset(
                                                ((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int)
                                                    as isize,
                                            ))
                                            .es_name,
                                        )
                                            as ::core::ffi::c_int;
                                        if slen > plen
                                            && path_fnamecmp(
                                                p_0,
                                                (*((*exestack.ptr()).ga_data as *mut estack_T)
                                                    .offset(
                                                        ((*exestack.ptr()).ga_len
                                                            - 1 as ::core::ffi::c_int)
                                                            as isize,
                                                    ))
                                                .es_name
                                                .offset(slen as isize)
                                                .offset(-(plen as isize)),
                                            ) == 0 as ::core::ffi::c_int
                                        {
                                            j = OK;
                                        }
                                        xfree(scriptname as *mut ::core::ffi::c_void);
                                    }
                                    if j == FAIL {
                                        semsg(
                                            gettext(
                                                b"E746: Function name does not match script file name: %s\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            ),
                                            name,
                                        );
                                        break '_erret;
                                    }
                                }
                                if namelen == 0 as size_t {
                                    namelen = strlen(name);
                                }
                                fp = alloc_ufunc(name, namelen);
                                if !fudi.fd_dict.is_null() {
                                    if fudi.fd_di.is_null() {
                                        fudi.fd_di = tv_dict_item_alloc(fudi.fd_newkey);
                                        if tv_dict_add(fudi.fd_dict, fudi.fd_di) == FAIL {
                                            xfree(fudi.fd_di as *mut ::core::ffi::c_void);
                                            let mut ptr__0: *mut *mut ::core::ffi::c_void =
                                                &raw mut fp as *mut *mut ::core::ffi::c_void;
                                            xfree(*ptr__0);
                                            *ptr__0 = NULL;
                                            let _ = *ptr__0;
                                            break '_erret;
                                        }
                                    } else {
                                        tv_clear(&raw mut (*fudi.fd_di).di_tv);
                                    }
                                    (*fudi.fd_di).di_tv.v_type = VAR_FUNC;
                                    (*fudi.fd_di).di_tv.vval.v_string =
                                        xmemdupz(name as *const ::core::ffi::c_void, namelen)
                                            as *mut ::core::ffi::c_char;
                                    flags |= FC_DICT;
                                }
                                if overwrite {
                                    let mut hi: *mut hashitem_T =
                                        hash_find(func_hashtab.ptr(), name);
                                    (*hi).hi_key =
                                        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char;
                                } else if hash_add(
                                    func_hashtab.ptr(),
                                    &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                                ) == FAIL
                                {
                                    free_fp = true_0 != 0;
                                    break '_erret;
                                }
                                (*fp).uf_refcount = 1 as ::core::ffi::c_int;
                            }
                            (*fp).uf_args = newargs;
                            (*fp).uf_def_args = default_args;
                            (*fp).uf_lines = newlines;
                            if flags & FC_CLOSURE != 0 as ::core::ffi::c_int {
                                register_closure(fp);
                            } else {
                                (*fp).uf_scoped = ::core::ptr::null_mut::<funccall_T>();
                            }
                            if prof_def_func() {
                                func_do_profile(fp);
                            }
                            (*fp).uf_varargs = varargs;
                            if sandbox.get() != 0 {
                                flags |= FC_SANDBOX;
                            }
                            (*fp).uf_flags = flags;
                            (*fp).uf_calls = 0 as ::core::ffi::c_int;
                            (*fp).uf_script_ctx = current_sctx.get();
                            (*fp).uf_script_ctx.sc_lnum += sourcing_lnum_top;
                            nlua_set_sctx(&raw mut (*fp).uf_script_ctx);
                            break '_ret_free;
                        }
                    }
                    if !fp.is_null() {
                        ga_init(
                            &raw mut (*fp).uf_args,
                            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                                as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                        );
                        ga_init(
                            &raw mut (*fp).uf_def_args,
                            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                                as ::core::ffi::c_int,
                            1 as ::core::ffi::c_int,
                        );
                    }
                }
                if !fp.is_null() {
                    let mut ptr__1: *mut *mut ::core::ffi::c_void =
                        &raw mut (*fp).uf_name_exp as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__1);
                    *ptr__1 = NULL;
                    let _ = *ptr__1;
                }
                if free_fp {
                    let mut ptr__2: *mut *mut ::core::ffi::c_void =
                        &raw mut fp as *mut *mut ::core::ffi::c_void;
                    xfree(*ptr__2);
                    *ptr__2 = NULL;
                    let _ = *ptr__2;
                }
            }
            ga_clear_strings(&raw mut newargs);
            ga_clear_strings(&raw mut default_args);
            ga_clear_strings(&raw mut newlines);
        }
    }
    xfree(line_to_free as *mut ::core::ffi::c_void);
    xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
    xfree(name as *mut ::core::ffi::c_void);
    (*did_emsg.ptr()) |= saved_did_emsg;
    if show_block {
        ui_ext_cmdline_block_leave();
    }
}
pub unsafe extern "C" fn eval_fname_script(p: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '<' as ::core::ffi::c_int
        && (mb_strnicmp(
            p.offset(1 as ::core::ffi::c_int as isize),
            b"SID>\0".as_ptr() as *const ::core::ffi::c_char,
            4 as size_t,
        ) == 0 as ::core::ffi::c_int
            || mb_strnicmp(
                p.offset(1 as ::core::ffi::c_int as isize),
                b"SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                4 as size_t,
            ) == 0 as ::core::ffi::c_int)
    {
        return 5 as ::core::ffi::c_int;
    }
    if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == 's' as ::core::ffi::c_int
        && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == ':' as ::core::ffi::c_int
    {
        return 2 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn translated_function_exists(mut name: *const ::core::ffi::c_char) -> bool {
    if builtin_function(name, -1 as ::core::ffi::c_int) {
        return !find_internal_func(name).is_null();
    }
    return !find_func(name).is_null();
}
pub unsafe extern "C" fn function_exists(
    name: *const ::core::ffi::c_char,
    mut no_deref: bool,
) -> bool {
    let mut nm: *const ::core::ffi::c_char = name;
    let mut n: bool = false_0 != 0;
    let mut flag: ::core::ffi::c_int = TFN_INT as ::core::ffi::c_int
        | TFN_QUIET as ::core::ffi::c_int
        | TFN_NO_AUTOLOAD as ::core::ffi::c_int;
    if no_deref {
        flag |= TFN_NO_DEREF as ::core::ffi::c_int;
    }
    let p: *mut ::core::ffi::c_char = trans_function_name(
        &raw mut nm as *mut *mut ::core::ffi::c_char,
        false_0 != 0,
        flag,
        ::core::ptr::null_mut::<funcdict_T>(),
        ::core::ptr::null_mut::<*mut partial_T>(),
    );
    nm = skipwhite(nm);
    if !p.is_null()
        && (*nm as ::core::ffi::c_int == NUL
            || *nm as ::core::ffi::c_int == '(' as ::core::ffi::c_int)
    {
        n = translated_function_exists(p);
    }
    xfree(p as *mut ::core::ffi::c_void);
    return n;
}
pub unsafe extern "C" fn get_user_func_name(
    mut xp: *mut expand_T,
    mut idx: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    static done: GlobalCell<size_t> = GlobalCell::new(0);
    static changed: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
    static hi: GlobalCell<*mut hashitem_T> = GlobalCell::new(::core::ptr::null_mut::<hashitem_T>());
    if idx == 0 as ::core::ffi::c_int {
        done.set(0 as size_t);
        hi.set((*func_hashtab.ptr()).ht_array);
        changed.set((*func_hashtab.ptr()).ht_changed);
    }
    '_c2rust_label: {
        if !(*hi.ptr()).is_null() {
        } else {
            __assert_fail(
                b"hi\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/eval/userfunc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                3083 as ::core::ffi::c_uint,
                b"char *get_user_func_name(expand_T *, int)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    if changed.get() == (*func_hashtab.ptr()).ht_changed
        && done.get() < (*func_hashtab.ptr()).ht_used
    {
        let c2rust_fresh16 = done.get();
        done.set((*done.ptr()).wrapping_add(1));
        if c2rust_fresh16 > 0 as size_t {
            hi.set((*hi.ptr()).offset(1));
        }
        while (*hi.get()).hi_key.is_null()
            || (*hi.get()).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char
        {
            hi.set((*hi.ptr()).offset(1));
        }
        let mut fp: *mut ufunc_T = (*hi.get())
            .hi_key
            .offset(-(240 as ::core::ffi::c_ulong as isize))
            as *mut ufunc_T;
        if (*fp).uf_flags & FC_DICT != 0
            || strncmp(
                &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
                b"<lambda>\0".as_ptr() as *const ::core::ffi::c_char,
                8 as size_t,
            ) == 0 as ::core::ffi::c_int
        {
            return b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        if (*fp).uf_namelen.wrapping_add(4 as size_t) >= IOSIZE as size_t {
            return &raw mut (*fp).uf_name as *mut ::core::ffi::c_char;
        }
        let mut len: ::core::ffi::c_int = cat_func_name(
            IObuff.ptr() as *mut ::core::ffi::c_char,
            IOSIZE as size_t,
            fp,
        );
        if (*xp).xp_context != EXPAND_USER_FUNC as ::core::ffi::c_int {
            xstrlcpy(
                (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                b"(\0".as_ptr() as *const ::core::ffi::c_char,
                (IOSIZE as size_t).wrapping_sub(len as size_t),
            );
            if (*fp).uf_varargs == 0 && (*fp).uf_args.ga_len <= 0 as ::core::ffi::c_int {
                len += 1;
                xstrlcpy(
                    (IObuff.ptr() as *mut ::core::ffi::c_char).offset(len as isize),
                    b")\0".as_ptr() as *const ::core::ffi::c_char,
                    (IOSIZE as size_t).wrapping_sub(len as size_t),
                );
            }
        }
        return IObuff.ptr() as *mut ::core::ffi::c_char;
    }
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
pub unsafe fn ex_delfunction(mut eap: *mut exarg_T) {
    let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
    let mut fudi: funcdict_T = funcdict_T {
        fd_dict: ::core::ptr::null_mut::<dict_T>(),
        fd_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fd_di: ::core::ptr::null_mut::<dictitem_T>(),
    };
    let mut p: *mut ::core::ffi::c_char = (*eap).arg;
    let mut name: *mut ::core::ffi::c_char = trans_function_name(
        &raw mut p,
        (*eap).skip != 0,
        0 as ::core::ffi::c_int,
        &raw mut fudi,
        ::core::ptr::null_mut::<*mut partial_T>(),
    );
    xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
    if name.is_null() {
        if !fudi.fd_dict.is_null() && (*eap).skip == 0 {
            emsg(gettext(e_funcref.get()));
        }
        return;
    }
    if ends_excmd(*skipwhite(p) as ::core::ffi::c_int) == 0 {
        xfree(name as *mut ::core::ffi::c_void);
        semsg(
            gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
            p,
        );
        return;
    }
    (*eap).nextcmd = check_nextcmd(p);
    if !(*eap).nextcmd.is_null() {
        *p = NUL as ::core::ffi::c_char;
    }
    if *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
        && fudi.fd_dict.is_null()
    {
        if (*eap).skip == 0 {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                (*eap).arg,
            );
        }
        xfree(name as *mut ::core::ffi::c_void);
        return;
    }
    if (*eap).skip == 0 {
        fp = find_func(name);
    }
    xfree(name as *mut ::core::ffi::c_void);
    if (*eap).skip == 0 {
        if fp.is_null() {
            if (*eap).forceit == 0 {
                semsg(gettext(e_nofunc.get()), (*eap).arg);
            }
            return;
        }
        if (*fp).uf_calls > 0 as ::core::ffi::c_int {
            semsg(
                gettext(b"E131: Cannot delete function %s: It is in use\0".as_ptr()
                    as *const ::core::ffi::c_char),
                (*eap).arg,
            );
            return;
        }
        if (*fp).uf_refcount > 2 as ::core::ffi::c_int {
            semsg(
                gettext(
                    b"Cannot delete function %s: It is being used internally\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ),
                (*eap).arg,
            );
            return;
        }
        if !fudi.fd_dict.is_null() {
            tv_dict_item_remove(fudi.fd_dict, fudi.fd_di);
        } else if (*fp).uf_refcount
            > (if func_name_refcount(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                as ::core::ffi::c_int
                != 0
            {
                0 as ::core::ffi::c_int
            } else {
                1 as ::core::ffi::c_int
            })
        {
            if func_remove(fp) {
                (*fp).uf_refcount -= 1;
            }
            (*fp).uf_flags |= FC_DELETED;
        } else {
            func_clear_free(fp, false_0 != 0);
        }
    }
}
pub unsafe extern "C" fn func_unref(mut name: *mut ::core::ffi::c_char) {
    if name.is_null() || !func_name_refcount(name) {
        return;
    }
    let mut fp: *mut ufunc_T = find_func(name);
    if fp.is_null()
        && *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
            as ::core::ffi::c_int
            & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
            != 0
    {
        internal_error(b"func_unref()\0".as_ptr() as *const ::core::ffi::c_char);
        abort();
    }
    func_ptr_unref(fp);
}
pub unsafe extern "C" fn func_ptr_unref(mut fp: *mut ufunc_T) {
    if !fp.is_null() && {
        (*fp).uf_refcount -= 1;
        (*fp).uf_refcount <= 0 as ::core::ffi::c_int
    } {
        if (*fp).uf_calls == 0 as ::core::ffi::c_int {
            func_clear_free(fp, false_0 != 0);
        }
    }
}
pub unsafe extern "C" fn func_ref(mut name: *mut ::core::ffi::c_char) {
    if name.is_null() || !func_name_refcount(name) {
        return;
    }
    let mut fp: *mut ufunc_T = find_func(name);
    if !fp.is_null() {
        (*fp).uf_refcount += 1;
    } else if *(*__ctype_b_loc()).offset(*name as uint8_t as ::core::ffi::c_int as isize)
        as ::core::ffi::c_int
        & _ISdigit as ::core::ffi::c_int as ::core::ffi::c_ushort as ::core::ffi::c_int
        != 0
    {
        internal_error(b"func_ref()\0".as_ptr() as *const ::core::ffi::c_char);
    }
}
pub unsafe extern "C" fn func_ptr_ref(mut fp: *mut ufunc_T) {
    if !fp.is_null() {
        (*fp).uf_refcount += 1;
    }
}
#[inline(always)]
unsafe extern "C" fn fc_referenced(fc: *const funccall_T) -> bool {
    return (*fc).fc_l_varlist.lv_refcount != DO_NOT_FREE_CNT as ::core::ffi::c_int
        || (*fc).fc_l_vars.dv_refcount != DO_NOT_FREE_CNT as ::core::ffi::c_int
        || (*fc).fc_l_avars.dv_refcount != DO_NOT_FREE_CNT as ::core::ffi::c_int
        || (*fc).fc_refcount > 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn can_free_funccal(
    mut fc: *mut funccall_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    return (*fc).fc_l_varlist.lv_copyID != copyID
        && (*fc).fc_l_vars.dv_copyID != copyID
        && (*fc).fc_l_avars.dv_copyID != copyID
        && (*fc).fc_copyID != copyID;
}
pub unsafe fn ex_return(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut rettv: typval_T = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    };
    let mut returning: bool = false_0 != 0;
    if (*current_funccal.ptr()).is_null() {
        emsg(gettext(
            b"E133: :return not inside a function\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: if (*eap).skip != 0 {
            0 as ::core::ffi::c_int
        } else {
            EVAL_EVALUATE as ::core::ffi::c_int
        },
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) += 1;
    }
    (*eap).nextcmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if *arg as ::core::ffi::c_int != NUL
        && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
        && *arg as ::core::ffi::c_int != '\n' as ::core::ffi::c_int
        && eval0(arg, &raw mut rettv, eap, &raw mut evalarg) != FAIL
    {
        if (*eap).skip == 0 {
            returning = do_return(
                eap,
                false_0 != 0,
                true_0 != 0,
                &raw mut rettv as *mut ::core::ffi::c_void,
            );
        } else {
            tv_clear(&raw mut rettv);
        }
    } else if (*eap).skip == 0 {
        update_force_abort();
        if !aborting() {
            returning = do_return(eap, false_0 != 0, true_0 != 0, NULL);
        }
    }
    if returning {
        (*eap).nextcmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else if (*eap).nextcmd.is_null() {
        (*eap).nextcmd = check_nextcmd(arg);
    }
    if (*eap).skip != 0 {
        (*emsg_skip.ptr()) -= 1;
    }
    clear_evalarg(&raw mut evalarg, eap);
}
unsafe extern "C" fn ex_call_inner(
    mut eap: *mut exarg_T,
    mut name: *mut ::core::ffi::c_char,
    mut arg: *mut *mut ::core::ffi::c_char,
    mut startarg: *mut ::core::ffi::c_char,
    funcexe_init: *const funcexe_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let mut doesrange: bool = false;
    let mut failed: bool = false_0 != 0;
    let mut lnum: linenr_T = (*eap).line1;
    while lnum <= (*eap).line2 {
        if (*eap).addr_count > 0 as ::core::ffi::c_int {
            if lnum > (*curbuf.get()).b_ml.ml_line_count {
                emsg(gettext(&raw const e_invrange as *const ::core::ffi::c_char));
                break;
            } else {
                (*curwin.get()).w_cursor.lnum = lnum;
                (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
            }
        }
        *arg = startarg;
        let mut funcexe: funcexe_T = *funcexe_init;
        funcexe.fe_doesrange = &raw mut doesrange;
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        rettv.v_type = VAR_UNKNOWN;
        if get_func_tv(
            name,
            -1 as ::core::ffi::c_int,
            &raw mut rettv,
            arg,
            evalarg,
            &raw mut funcexe,
        ) == FAIL
        {
            failed = true_0 != 0;
            break;
        } else if handle_subscript(
            arg as *mut *const ::core::ffi::c_char,
            &raw mut rettv,
            EVALARG_EVALUATE.ptr(),
            true_0 != 0,
        ) == FAIL
        {
            failed = true_0 != 0;
            break;
        } else {
            tv_clear(&raw mut rettv);
            if doesrange {
                break;
            }
            if aborting() {
                break;
            }
            lnum += 1;
        }
    }
    return failed as ::core::ffi::c_int;
}
unsafe extern "C" fn ex_defer_inner(
    mut name: *mut ::core::ffi::c_char,
    mut arg: *mut *mut ::core::ffi::c_char,
    partial: *const partial_T,
    evalarg: *mut evalarg_T,
) -> ::core::ffi::c_int {
    let mut argvars: [typval_T; 21] = [typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VAR_UNLOCKED,
        vval: typval_vval_union { v_number: 0 },
    }; 21];
    let mut partial_argc: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut argcount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*current_funccal.ptr()).is_null() {
        semsg(
            gettext(&raw const e_str_not_inside_function as *const ::core::ffi::c_char),
            b"defer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return FAIL;
    }
    if !partial.is_null() {
        if !(*partial).pt_dict.is_null() {
            emsg(gettext(
                (e_cannot_use_partial_with_dictionary_for_defer.ptr() as *const _)
                    as *const ::core::ffi::c_char,
            ));
            return FAIL;
        }
        if (*partial).pt_argc > 0 as ::core::ffi::c_int {
            partial_argc = (*partial).pt_argc;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < partial_argc {
                tv_copy(
                    (*partial).pt_argv.offset(i as isize),
                    (&raw mut argvars as *mut typval_T).offset(i as isize),
                );
                i += 1;
            }
        }
    }
    let mut r: ::core::ffi::c_int = get_func_arguments(
        arg,
        evalarg,
        false_0,
        (&raw mut argvars as *mut typval_T).offset(partial_argc as isize),
        &raw mut argcount,
    );
    argcount += partial_argc;
    if r == OK {
        if builtin_function(name, -1 as ::core::ffi::c_int) {
            let fdef: *const EvalFuncDef = find_internal_func(name);
            if fdef.is_null() {
                emsg_funcname(
                    &raw const e_unknown_function_str as *const ::core::ffi::c_char,
                    name,
                );
                r = FAIL;
            } else if check_internal_func(fdef, argcount) == -1 as ::core::ffi::c_int {
                r = FAIL;
            }
        } else {
            let mut ufunc: *mut ufunc_T = find_func(name);
            if !ufunc.is_null() {
                let mut error: ::core::ffi::c_int = check_user_func_argcount(ufunc, argcount);
                if error != FCERR_UNKNOWN as ::core::ffi::c_int {
                    user_func_error(error, name, false_0 != 0);
                    r = FAIL;
                }
            }
        }
    }
    if r == FAIL {
        loop {
            argcount -= 1;
            if argcount < 0 as ::core::ffi::c_int {
                break;
            }
            tv_clear((&raw mut argvars as *mut typval_T).offset(argcount as isize));
        }
        return FAIL;
    }
    add_defer(name, argcount, &raw mut argvars as *mut typval_T);
    return OK;
}
pub unsafe extern "C" fn can_add_defer() -> bool {
    if get_current_funccal().is_null() {
        semsg(
            gettext(&raw const e_str_not_inside_function as *const ::core::ffi::c_char),
            b"defer\0".as_ptr() as *const ::core::ffi::c_char,
        );
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub unsafe extern "C" fn add_defer(
    mut name: *mut ::core::ffi::c_char,
    mut argcount_arg: ::core::ffi::c_int,
    mut argvars: *mut typval_T,
) {
    let mut saved_name: *mut ::core::ffi::c_char = xstrdup(name);
    let mut argcount: ::core::ffi::c_int = argcount_arg;
    if (*current_funccal.get()).fc_defer.ga_itemsize == 0 as ::core::ffi::c_int {
        ga_init(
            &raw mut (*current_funccal.get()).fc_defer,
            ::core::mem::size_of::<defer_T>() as ::core::ffi::c_int,
            10 as ::core::ffi::c_int,
        );
    }
    let mut dr: *mut defer_T = ga_append_via_ptr(
        &raw mut (*current_funccal.get()).fc_defer,
        ::core::mem::size_of::<defer_T>(),
    ) as *mut defer_T;
    (*dr).dr_name = saved_name;
    (*dr).dr_argcount = argcount;
    while argcount > 0 as ::core::ffi::c_int {
        argcount -= 1;
        (*dr).dr_argvars[argcount as usize] = *argvars.offset(argcount as isize);
    }
}
unsafe extern "C" fn handle_defer_one(mut funccal: *mut funccall_T) {
    let mut idx: ::core::ffi::c_int = (*funccal).fc_defer.ga_len - 1 as ::core::ffi::c_int;
    while idx >= 0 as ::core::ffi::c_int {
        let mut dr: *mut defer_T =
            ((*funccal).fc_defer.ga_data as *mut defer_T).offset(idx as isize);
        if !(*dr).dr_name.is_null() {
            let mut funcexe: funcexe_T = funcexe_T {
                fe_argv_func: None,
                fe_firstline: 0,
                fe_lastline: 0,
                fe_doesrange: ::core::ptr::null_mut::<bool>(),
                fe_evaluate: true_0 != 0,
                fe_partial: ::core::ptr::null_mut::<partial_T>(),
                fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
                fe_basetv: ::core::ptr::null_mut::<typval_T>(),
                fe_found_var: false,
            };
            let mut rettv: typval_T = typval_T {
                v_type: VAR_UNKNOWN,
                v_lock: VAR_UNLOCKED,
                vval: typval_vval_union { v_number: 0 },
            };
            rettv.v_type = VAR_UNKNOWN;
            let mut name: *mut ::core::ffi::c_char = (*dr).dr_name;
            (*dr).dr_name = ::core::ptr::null_mut::<::core::ffi::c_char>();
            let mut estate: exception_state_T = exception_state_T {
                estate_current_exception: ::core::ptr::null_mut::<except_T>(),
                estate_did_throw: false,
                estate_need_rethrow: false,
                estate_trylevel: 0,
                estate_did_emsg: 0,
            };
            exception_state_save(&raw mut estate);
            exception_state_clear();
            call_func(
                name,
                -1 as ::core::ffi::c_int,
                &raw mut rettv,
                (*dr).dr_argcount,
                &raw mut (*dr).dr_argvars as *mut typval_T,
                &raw mut funcexe,
            );
            exception_state_restore(&raw mut estate);
            tv_clear(&raw mut rettv);
            xfree(name as *mut ::core::ffi::c_void);
            let mut i: ::core::ffi::c_int = (*dr).dr_argcount - 1 as ::core::ffi::c_int;
            while i >= 0 as ::core::ffi::c_int {
                tv_clear((&raw mut (*dr).dr_argvars as *mut typval_T).offset(i as isize));
                i -= 1;
            }
        }
        idx -= 1;
    }
    ga_clear(&raw mut (*funccal).fc_defer);
}
pub unsafe extern "C" fn invoke_all_defer() {
    let mut fc: *mut funccall_T = current_funccal.get();
    while !fc.is_null() {
        handle_defer_one(fc);
        fc = (*fc).fc_caller;
    }
    let mut fce: *mut funccal_entry_T = funccal_stack.get();
    while !fce.is_null() {
        let mut fc_0: *mut funccall_T = (*fce).top_funccal as *mut funccall_T;
        while !fc_0.is_null() {
            handle_defer_one(fc_0);
            fc_0 = (*fc_0).fc_caller;
        }
        fce = (*fce).next;
    }
}
pub unsafe fn ex_call(mut eap: *mut exarg_T) {
    let mut arg: *mut ::core::ffi::c_char = (*eap).arg;
    let mut failed: bool = false_0 != 0;
    let mut fudi: funcdict_T = funcdict_T {
        fd_dict: ::core::ptr::null_mut::<dict_T>(),
        fd_newkey: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        fd_di: ::core::ptr::null_mut::<dictitem_T>(),
    };
    let mut partial: *mut partial_T = ::core::ptr::null_mut::<partial_T>();
    let mut evalarg: evalarg_T = evalarg_T {
        eval_flags: 0,
        eval_getline: None,
        eval_cookie: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        eval_tofree: ::core::ptr::null_mut::<::core::ffi::c_char>(),
    };
    fill_evalarg_from_eap(&raw mut evalarg, eap, (*eap).skip != 0);
    if (*eap).skip != 0 {
        let mut rettv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        (*emsg_skip.ptr()) += 1;
        if eval0((*eap).arg, &raw mut rettv, eap, &raw mut evalarg) != FAIL {
            tv_clear(&raw mut rettv);
        }
        (*emsg_skip.ptr()) -= 1;
        clear_evalarg(&raw mut evalarg, eap);
        return;
    }
    let mut tofree: *mut ::core::ffi::c_char = trans_function_name(
        &raw mut arg,
        false_0 != 0,
        TFN_INT as ::core::ffi::c_int,
        &raw mut fudi,
        &raw mut partial,
    );
    if !fudi.fd_newkey.is_null() {
        semsg(
            gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
            fudi.fd_newkey,
        );
        xfree(fudi.fd_newkey as *mut ::core::ffi::c_void);
    }
    if tofree.is_null() {
        return;
    }
    if !fudi.fd_dict.is_null() {
        (*fudi.fd_dict).dv_refcount += 1;
    }
    let mut len: ::core::ffi::c_int = strlen(tofree) as ::core::ffi::c_int;
    let mut found_var: bool = false_0 != 0;
    let mut name: *mut ::core::ffi::c_char = deref_func_name(
        tofree,
        &raw mut len,
        if !partial.is_null() {
            ::core::ptr::null_mut::<*mut partial_T>()
        } else {
            &raw mut partial
        },
        false_0 != 0,
        &raw mut found_var,
    );
    let mut startarg: *mut ::core::ffi::c_char = skipwhite(arg);
    if *startarg as ::core::ffi::c_int != '(' as ::core::ffi::c_int {
        semsg(
            gettext(&raw const e_missingparen as *const ::core::ffi::c_char),
            (*eap).arg,
        );
    } else {
        if (*eap).cmdidx as ::core::ffi::c_int == CMD_defer as ::core::ffi::c_int {
            arg = startarg;
            failed = ex_defer_inner(name, &raw mut arg, partial, &raw mut evalarg) == FAIL;
        } else {
            let mut funcexe: funcexe_T = FUNCEXE_INIT;
            funcexe.fe_partial = partial;
            funcexe.fe_selfdict = fudi.fd_dict;
            funcexe.fe_firstline = (*eap).line1;
            funcexe.fe_lastline = (*eap).line2;
            funcexe.fe_found_var = found_var;
            funcexe.fe_evaluate = true_0 != 0;
            failed = ex_call_inner(
                eap,
                name,
                &raw mut arg,
                startarg,
                &raw mut funcexe,
                &raw mut evalarg,
            ) != 0;
        }
        if (!aborting() || did_throw.get() as ::core::ffi::c_int != 0)
            && (!failed || (*(*eap).cstack).cs_trylevel > 0 as ::core::ffi::c_int)
        {
            if ends_excmd(*arg as ::core::ffi::c_int) == 0 {
                if !failed && !aborting() {
                    emsg_severe.set(true_0 != 0);
                    semsg(
                        gettext(&raw const e_trailing_arg as *const ::core::ffi::c_char),
                        arg,
                    );
                }
            } else {
                (*eap).nextcmd = check_nextcmd(arg);
            }
        }
        clear_evalarg(&raw mut evalarg, eap);
    }
    tv_dict_unref(fudi.fd_dict);
    xfree(tofree as *mut ::core::ffi::c_void);
}
pub unsafe extern "C" fn do_return(
    mut eap: *mut exarg_T,
    mut reanimate: bool,
    mut is_cmd: bool,
    mut rettv: *mut ::core::ffi::c_void,
) -> bool {
    let cstack: *mut cstack_T = (*eap).cstack;
    if reanimate {
        (*current_funccal.get()).fc_returned = false_0;
    }
    let mut idx: ::core::ffi::c_int =
        cleanup_conditionals((*eap).cstack, 0 as ::core::ffi::c_int, true_0);
    if idx >= 0 as ::core::ffi::c_int {
        (*cstack).cs_pending[idx as usize] =
            CSTP_RETURN as ::core::ffi::c_int as ::core::ffi::c_char;
        if !is_cmd && !reanimate {
            (*cstack).cs_pend.csp_rv[idx as usize] = rettv;
        } else {
            if reanimate {
                '_c2rust_label: {
                    if !(*current_funccal.get()).fc_rettv.is_null() {
                    } else {
                        __assert_fail(
                            b"current_funccal->fc_rettv\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/eval/userfunc.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            3664 as ::core::ffi::c_uint,
                            b"_Bool do_return(exarg_T *, _Bool, _Bool, void *)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                rettv = (*current_funccal.get()).fc_rettv as *mut ::core::ffi::c_void;
            }
            if !rettv.is_null() {
                (*cstack).cs_pend.csp_rv[idx as usize] =
                    xcalloc(1 as size_t, ::core::mem::size_of::<typval_T>());
                *((*cstack).cs_pend.csp_rv[idx as usize] as *mut typval_T) =
                    *(rettv as *mut typval_T);
            } else {
                (*cstack).cs_pend.csp_rv[idx as usize] = NULL;
            }
            if reanimate {
                (*(*current_funccal.get()).fc_rettv).v_type = VAR_NUMBER;
                (*(*current_funccal.get()).fc_rettv).vval.v_number = 0 as varnumber_T;
            }
        }
        report_make_pending(CSTP_RETURN as ::core::ffi::c_int, rettv);
    } else {
        (*current_funccal.get()).fc_returned = true_0;
        if !reanimate && !rettv.is_null() {
            tv_clear((*current_funccal.get()).fc_rettv);
            *(*current_funccal.get()).fc_rettv = *(rettv as *mut typval_T);
            if !is_cmd {
                xfree(rettv);
            }
        }
    }
    return idx < 0 as ::core::ffi::c_int;
}
pub unsafe extern "C" fn get_return_cmd(
    mut rettv: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    let mut s: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut slen: size_t = 0 as size_t;
    if !rettv.is_null() {
        s = encode_tv2echo(rettv as *mut typval_T, ::core::ptr::null_mut::<size_t>());
        tofree = s;
    }
    if s.is_null() {
        s = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
    } else {
        slen = strlen(s);
    }
    xstrlcpy(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        b":return \0".as_ptr() as *const ::core::ffi::c_char,
        IOSIZE as size_t,
    );
    xstrlcpy(
        (IObuff.ptr() as *mut ::core::ffi::c_char).offset(8 as ::core::ffi::c_int as isize),
        s,
        (IOSIZE - 8 as ::core::ffi::c_int) as size_t,
    );
    let mut IObufflen: size_t = (8 as size_t).wrapping_add(slen);
    if IObufflen >= IOSIZE as size_t {
        strcpy(
            (IObuff.ptr() as *mut ::core::ffi::c_char)
                .offset((1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as isize)
                .offset(-(4 as ::core::ffi::c_int as isize)),
            b"...\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        IObufflen = (IOSIZE - 1 as ::core::ffi::c_int) as size_t;
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    return xstrnsave(IObuff.ptr() as *mut ::core::ffi::c_char, IObufflen);
}
pub unsafe extern "C" fn get_func_line(
    mut _c: ::core::ffi::c_int,
    mut cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    let mut fcp: *mut funccall_T = cookie as *mut funccall_T;
    let mut fp: *mut ufunc_T = (*fcp).fc_func;
    let mut retval: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if (*fcp).fc_dbg_tick != debug_tick.get() {
        (*fcp).fc_breakpoint = dbg_find_breakpoint(
            false_0 != 0,
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        (*fcp).fc_dbg_tick = debug_tick.get();
    }
    if do_profiling.get() == PROF_YES {
        func_line_end(cookie);
    }
    let mut gap: *mut garray_T = &raw mut (*fp).uf_lines;
    if (*fp).uf_flags & FC_ABORT != 0 && did_emsg.get() != 0 && !aborted_in_try()
        || (*fcp).fc_returned != 0
    {
        retval = ::core::ptr::null_mut::<::core::ffi::c_char>();
    } else {
        while (*fcp).fc_linenr < (*gap).ga_len
            && (*((*gap).ga_data as *mut *mut ::core::ffi::c_char)
                .offset((*fcp).fc_linenr as isize))
            .is_null()
        {
            (*fcp).fc_linenr += 1;
        }
        if (*fcp).fc_linenr >= (*gap).ga_len {
            retval = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            let c2rust_fresh10 = (*fcp).fc_linenr;
            (*fcp).fc_linenr = (*fcp).fc_linenr + 1;
            retval = xstrdup(
                *((*gap).ga_data as *mut *mut ::core::ffi::c_char).offset(c2rust_fresh10 as isize),
            );
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum = (*fcp).fc_linenr as linenr_T;
            if do_profiling.get() == PROF_YES {
                func_line_start(cookie);
            }
        }
    }
    if (*fcp).fc_breakpoint != 0 as linenr_T
        && (*fcp).fc_breakpoint
            <= (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum
    {
        dbg_breakpoint(
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        (*fcp).fc_breakpoint = dbg_find_breakpoint(
            false_0 != 0,
            &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
            (*((*exestack.ptr()).ga_data as *mut estack_T)
                .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
            .es_lnum,
        );
        (*fcp).fc_dbg_tick = debug_tick.get();
    }
    return retval;
}
pub unsafe extern "C" fn func_has_ended(
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    let mut fcp: *mut funccall_T = cookie as *mut funccall_T;
    return ((*(*fcp).fc_func).uf_flags & FC_ABORT != 0 && did_emsg.get() != 0 && !aborted_in_try()
        || (*fcp).fc_returned != 0) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn func_has_abort(
    mut cookie: *mut ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    return (*(*(cookie as *mut funccall_T)).fc_func).uf_flags & FC_ABORT;
}
pub unsafe extern "C" fn make_partial(selfdict: *mut dict_T, rettv: *mut typval_T) {
    let mut fp: *mut ufunc_T = ::core::ptr::null_mut::<ufunc_T>();
    let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
    let mut error: ::core::ffi::c_int = 0;
    if (*rettv).v_type as ::core::ffi::c_uint
        == VAR_PARTIAL as ::core::ffi::c_int as ::core::ffi::c_uint
        && !(*rettv).vval.v_partial.is_null()
        && !(*(*rettv).vval.v_partial).pt_func.is_null()
    {
        fp = (*(*rettv).vval.v_partial).pt_func;
    } else {
        let mut fname: *mut ::core::ffi::c_char = if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*rettv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*rettv).vval.v_string
        } else if (*rettv).vval.v_partial.is_null() {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            (*(*rettv).vval.v_partial).pt_name
        };
        if fname.is_null() {
            (*rettv).v_type = VAR_FUNC;
            (*rettv).vval.v_string = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            let mut tofree: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            fname = fname_trans_sid(
                fname,
                &raw mut fname_buf as *mut ::core::ffi::c_char,
                &raw mut tofree,
                &raw mut error,
            );
            fp = find_func(fname);
            xfree(tofree as *mut ::core::ffi::c_void);
        }
    }
    if !fp.is_null() && (*fp).uf_flags & FC_DICT != 0 {
        let mut pt: *mut partial_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<partial_T>()) as *mut partial_T;
        (*pt).pt_refcount = 1 as ::core::ffi::c_int;
        (*pt).pt_dict = selfdict;
        (*selfdict).dv_refcount += 1;
        (*pt).pt_auto = true_0 != 0;
        if (*rettv).v_type as ::core::ffi::c_uint
            == VAR_FUNC as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*rettv).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*pt).pt_name = (*rettv).vval.v_string;
        } else {
            let mut ret_pt: *mut partial_T = (*rettv).vval.v_partial;
            if !(*ret_pt).pt_name.is_null() {
                (*pt).pt_name = xstrdup((*ret_pt).pt_name);
                func_ref((*pt).pt_name);
            } else {
                (*pt).pt_func = (*ret_pt).pt_func;
                func_ptr_ref((*pt).pt_func);
            }
            if (*ret_pt).pt_argc > 0 as ::core::ffi::c_int {
                let mut arg_size: size_t =
                    ::core::mem::size_of::<typval_T>().wrapping_mul((*ret_pt).pt_argc as size_t);
                (*pt).pt_argv = xmalloc(arg_size) as *mut typval_T;
                (*pt).pt_argc = (*ret_pt).pt_argc;
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*pt).pt_argc {
                    tv_copy(
                        (*ret_pt).pt_argv.offset(i as isize),
                        (*pt).pt_argv.offset(i as isize),
                    );
                    i += 1;
                }
            }
            partial_unref(ret_pt);
        }
        (*rettv).v_type = VAR_PARTIAL;
        (*rettv).vval.v_partial = pt;
    }
}
pub unsafe extern "C" fn func_name(
    mut cookie: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_char {
    return &raw mut (*(*(cookie as *mut funccall_T)).fc_func).uf_name as *mut ::core::ffi::c_char;
}
pub unsafe extern "C" fn func_breakpoint(mut cookie: *mut ::core::ffi::c_void) -> *mut linenr_T {
    return &raw mut (*(cookie as *mut funccall_T)).fc_breakpoint;
}
pub unsafe extern "C" fn func_dbg_tick(
    mut cookie: *mut ::core::ffi::c_void,
) -> *mut ::core::ffi::c_int {
    return &raw mut (*(cookie as *mut funccall_T)).fc_dbg_tick;
}
pub unsafe extern "C" fn func_level(mut cookie: *mut ::core::ffi::c_void) -> ::core::ffi::c_int {
    return (*(cookie as *mut funccall_T)).fc_level;
}
pub unsafe extern "C" fn current_func_returned() -> ::core::ffi::c_int {
    return (*current_funccal.get()).fc_returned;
}
pub unsafe extern "C" fn free_unref_funccal(
    mut copyID: ::core::ffi::c_int,
    mut testing: ::core::ffi::c_int,
) -> bool {
    let mut did_free: bool = false_0 != 0;
    let mut did_free_funccal: bool = false_0 != 0;
    let mut pfc: *mut *mut funccall_T = previous_funccal.ptr();
    while !(*pfc).is_null() {
        if can_free_funccal(*pfc, copyID) {
            let mut fc: *mut funccall_T = *pfc;
            *pfc = (*fc).fc_caller;
            free_funccal_contents(fc);
            did_free = true_0 != 0;
            did_free_funccal = true_0 != 0;
        } else {
            pfc = &raw mut (**pfc).fc_caller;
        }
    }
    if did_free_funccal {
        garbage_collect(testing != 0);
    }
    return did_free;
}
pub unsafe extern "C" fn get_funccal() -> *mut funccall_T {
    let mut funccal: *mut funccall_T = current_funccal.get();
    if debug_backtrace_level.get() > 0 as ::core::ffi::c_int {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < debug_backtrace_level.get() {
            let mut temp_funccal: *mut funccall_T = (*funccal).fc_caller;
            if !temp_funccal.is_null() {
                funccal = temp_funccal;
            } else {
                debug_backtrace_level.set(i);
            }
            i += 1;
        }
    }
    return funccal;
}
pub unsafe extern "C" fn get_funccal_local_dict() -> *mut dict_T {
    if (*current_funccal.ptr()).is_null()
        || (*current_funccal.get()).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<dict_T>();
    }
    return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)()).fc_l_vars;
}
pub unsafe extern "C" fn get_funccal_local_ht() -> *mut hashtab_T {
    let mut d: *mut dict_T = get_funccal_local_dict();
    return if !d.is_null() {
        &raw mut (*d).dv_hashtab
    } else {
        ::core::ptr::null_mut::<hashtab_T>()
    };
}
pub unsafe extern "C" fn get_funccal_local_var() -> *mut dictitem_T {
    if (*current_funccal.ptr()).is_null()
        || (*current_funccal.get()).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<dictitem_T>();
    }
    return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)()).fc_l_vars_var
        as *mut dictitem_T;
}
pub unsafe extern "C" fn get_funccal_args_dict() -> *mut dict_T {
    if (*current_funccal.ptr()).is_null()
        || (*current_funccal.get()).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<dict_T>();
    }
    return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)()).fc_l_avars;
}
pub unsafe extern "C" fn get_funccal_args_ht() -> *mut hashtab_T {
    let mut d: *mut dict_T = get_funccal_args_dict();
    return if !d.is_null() {
        &raw mut (*d).dv_hashtab
    } else {
        ::core::ptr::null_mut::<hashtab_T>()
    };
}
pub unsafe extern "C" fn get_funccal_args_var() -> *mut dictitem_T {
    if (*current_funccal.ptr()).is_null()
        || (*current_funccal.get()).fc_l_vars.dv_refcount == 0 as ::core::ffi::c_int
    {
        return ::core::ptr::null_mut::<dictitem_T>();
    }
    return &raw mut (*(get_funccal as unsafe extern "C" fn() -> *mut funccall_T)()).fc_l_avars_var
        as *mut dictitem_T;
}
pub unsafe extern "C" fn list_func_vars(mut first: *mut ::core::ffi::c_int) {
    if !(*current_funccal.ptr()).is_null()
        && (*current_funccal.get()).fc_l_vars.dv_refcount > 0 as ::core::ffi::c_int
    {
        list_hashtable_vars(
            &raw mut (*current_funccal.get()).fc_l_vars.dv_hashtab,
            b"l:\0".as_ptr() as *const ::core::ffi::c_char,
            false_0,
            first,
        );
    }
}
pub unsafe extern "C" fn get_current_funccal_dict(mut ht: *mut hashtab_T) -> *mut dict_T {
    if !(*current_funccal.ptr()).is_null()
        && ht == &raw mut (*current_funccal.get()).fc_l_vars.dv_hashtab
    {
        return &raw mut (*current_funccal.get()).fc_l_vars;
    }
    return ::core::ptr::null_mut::<dict_T>();
}
pub unsafe extern "C" fn find_hi_in_scoped_ht(
    mut name: *const ::core::ffi::c_char,
    mut pht: *mut *mut hashtab_T,
) -> *mut hashitem_T {
    if (*current_funccal.ptr()).is_null() || (*(*current_funccal.get()).fc_func).uf_scoped.is_null()
    {
        return ::core::ptr::null_mut::<hashitem_T>();
    }
    let mut old_current_funccal: *mut funccall_T = current_funccal.get();
    let mut hi: *mut hashitem_T = ::core::ptr::null_mut::<hashitem_T>();
    let namelen: size_t = strlen(name);
    let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    current_funccal.set((*(*current_funccal.get()).fc_func).uf_scoped);
    while !(*current_funccal.ptr()).is_null() {
        let mut ht: *mut hashtab_T = find_var_ht(name, namelen, &raw mut varname);
        if !ht.is_null() && *varname as ::core::ffi::c_int != NUL {
            hi = hash_find_len(
                ht,
                varname,
                namelen.wrapping_sub(varname.offset_from(name) as size_t),
            );
            if !((*hi).hi_key.is_null()
                || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
            {
                *pht = ht;
                break;
            }
        }
        if current_funccal.get() == (*(*current_funccal.get()).fc_func).uf_scoped {
            break;
        }
        current_funccal.set((*(*current_funccal.get()).fc_func).uf_scoped);
    }
    current_funccal.set(old_current_funccal);
    return hi;
}
pub unsafe extern "C" fn find_var_in_scoped_ht(
    mut name: *const ::core::ffi::c_char,
    namelen: size_t,
    mut no_autoload: ::core::ffi::c_int,
) -> *mut dictitem_T {
    if (*current_funccal.ptr()).is_null() || (*(*current_funccal.get()).fc_func).uf_scoped.is_null()
    {
        return ::core::ptr::null_mut::<dictitem_T>();
    }
    let mut v: *mut dictitem_T = ::core::ptr::null_mut::<dictitem_T>();
    let mut old_current_funccal: *mut funccall_T = current_funccal.get();
    let mut varname: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
    current_funccal.set((*(*current_funccal.get()).fc_func).uf_scoped);
    while !(*current_funccal.ptr()).is_null() {
        let mut ht: *mut hashtab_T = find_var_ht(name, namelen, &raw mut varname);
        if !ht.is_null() && *varname as ::core::ffi::c_int != NUL {
            v = find_var_in_ht(
                ht,
                *name as ::core::ffi::c_int,
                varname,
                namelen.wrapping_sub(varname.offset_from(name) as size_t),
                no_autoload,
            );
            if !v.is_null() {
                break;
            }
        }
        if current_funccal.get() == (*(*current_funccal.get()).fc_func).uf_scoped {
            break;
        }
        current_funccal.set((*(*current_funccal.get()).fc_func).uf_scoped);
    }
    current_funccal.set(old_current_funccal);
    return v;
}
pub unsafe extern "C" fn set_ref_in_previous_funccal(mut copyID: ::core::ffi::c_int) -> bool {
    let mut fc: *mut funccall_T = previous_funccal.get();
    while !fc.is_null() {
        (*fc).fc_copyID = copyID + 1 as ::core::ffi::c_int;
        if set_ref_in_ht(
            &raw mut (*fc).fc_l_vars.dv_hashtab,
            copyID + 1 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        ) as ::core::ffi::c_int
            != 0
            || set_ref_in_ht(
                &raw mut (*fc).fc_l_avars.dv_hashtab,
                copyID + 1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
            || set_ref_in_list_items(
                &raw mut (*fc).fc_l_varlist,
                copyID + 1 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
        {
            return true_0 != 0;
        }
        fc = (*fc).fc_caller;
    }
    return false_0 != 0;
}
unsafe extern "C" fn set_ref_in_funccal(
    mut fc: *mut funccall_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    if (*fc).fc_copyID != copyID {
        (*fc).fc_copyID = copyID;
        if set_ref_in_ht(
            &raw mut (*fc).fc_l_vars.dv_hashtab,
            copyID,
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        ) as ::core::ffi::c_int
            != 0
            || set_ref_in_ht(
                &raw mut (*fc).fc_l_avars.dv_hashtab,
                copyID,
                ::core::ptr::null_mut::<*mut list_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
            || set_ref_in_list_items(
                &raw mut (*fc).fc_l_varlist,
                copyID,
                ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ) as ::core::ffi::c_int
                != 0
            || set_ref_in_func(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                (*fc).fc_func,
                copyID,
            ) as ::core::ffi::c_int
                != 0
        {
            return true_0 != 0;
        }
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn set_ref_in_call_stack(mut copyID: ::core::ffi::c_int) -> bool {
    let mut fc: *mut funccall_T = current_funccal.get();
    while !fc.is_null() {
        if set_ref_in_funccal(fc, copyID) {
            return true_0 != 0;
        }
        fc = (*fc).fc_caller;
    }
    let mut entry: *mut funccal_entry_T = funccal_stack.get();
    while !entry.is_null() {
        let mut fc_0: *mut funccall_T = (*entry).top_funccal as *mut funccall_T;
        while !fc_0.is_null() {
            if set_ref_in_funccal(fc_0, copyID) {
                return true_0 != 0;
            }
            fc_0 = (*fc_0).fc_caller;
        }
        entry = (*entry).next;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn set_ref_in_functions(mut copyID: ::core::ffi::c_int) -> bool {
    let mut todo: ::core::ffi::c_int = (*func_hashtab.ptr()).ht_used as ::core::ffi::c_int;
    let mut hi: *mut hashitem_T = (*func_hashtab.ptr()).ht_array;
    while todo > 0 as ::core::ffi::c_int && !got_int.get() {
        if !((*hi).hi_key.is_null()
            || (*hi).hi_key == &raw const hash_removed as *mut ::core::ffi::c_char)
        {
            todo -= 1;
            let mut fp: *mut ufunc_T =
                (*hi).hi_key.offset(-(240 as ::core::ffi::c_ulong as isize)) as *mut ufunc_T;
            if !func_name_refcount(&raw mut (*fp).uf_name as *mut ::core::ffi::c_char)
                && set_ref_in_func(::core::ptr::null_mut::<::core::ffi::c_char>(), fp, copyID)
                    as ::core::ffi::c_int
                    != 0
            {
                return true_0 != 0;
            }
        }
        hi = hi.offset(1);
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn set_ref_in_func_args(mut copyID: ::core::ffi::c_int) -> bool {
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while i < (*funcargs.ptr()).ga_len {
        if set_ref_in_item(
            *((*funcargs.ptr()).ga_data as *mut *mut typval_T).offset(i as isize),
            copyID,
            ::core::ptr::null_mut::<*mut ht_stack_T>(),
            ::core::ptr::null_mut::<*mut list_stack_T>(),
        ) {
            return true_0 != 0;
        }
        i += 1;
    }
    return false_0 != 0;
}
pub unsafe extern "C" fn set_ref_in_func(
    mut name: *mut ::core::ffi::c_char,
    mut fp_in: *mut ufunc_T,
    mut copyID: ::core::ffi::c_int,
) -> bool {
    let mut fp: *mut ufunc_T = fp_in;
    let mut error: ::core::ffi::c_int = FCERR_NONE as ::core::ffi::c_int;
    let mut fname_buf: [::core::ffi::c_char; 41] = [0; 41];
    let mut tofree: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut abort_0: bool = false_0 != 0;
    if name.is_null() && fp_in.is_null() {
        return false_0 != 0;
    }
    if fp_in.is_null() {
        let mut fname: *mut ::core::ffi::c_char = fname_trans_sid(
            name,
            &raw mut fname_buf as *mut ::core::ffi::c_char,
            &raw mut tofree,
            &raw mut error,
        );
        fp = find_func(fname);
    }
    if !fp.is_null() {
        let mut fc: *mut funccall_T = (*fp).uf_scoped;
        while !fc.is_null() {
            abort_0 = abort_0 as ::core::ffi::c_int != 0
                || set_ref_in_funccal(fc, copyID) as ::core::ffi::c_int != 0;
            fc = (*(*fc).fc_func).uf_scoped;
        }
    }
    xfree(tofree as *mut ::core::ffi::c_void);
    return abort_0;
}
pub unsafe extern "C" fn register_luafunc(mut ref_0: LuaRef) -> *mut ::core::ffi::c_char {
    let mut name: String_0 = get_lambda_name();
    let mut fp: *mut ufunc_T = alloc_ufunc(name.data, name.size);
    (*fp).uf_refcount = 1 as ::core::ffi::c_int;
    (*fp).uf_varargs = true_0;
    (*fp).uf_flags = FC_LUAREF;
    (*fp).uf_calls = 0 as ::core::ffi::c_int;
    (*fp).uf_script_ctx = current_sctx.get();
    (*fp).uf_luaref = ref_0;
    hash_add(
        func_hashtab.ptr(),
        &raw mut (*fp).uf_name as *mut ::core::ffi::c_char,
    );
    return &raw mut (*fp).uf_name as *mut ::core::ffi::c_char;
}
pub const FC_ABORT: ::core::ffi::c_int = 0x1 as ::core::ffi::c_int;
pub const FC_RANGE: ::core::ffi::c_int = 0x2 as ::core::ffi::c_int;
pub const FC_DICT: ::core::ffi::c_int = 0x4 as ::core::ffi::c_int;
pub const FC_CLOSURE: ::core::ffi::c_int = 0x8 as ::core::ffi::c_int;
pub const FC_DELETED: ::core::ffi::c_int = 0x10 as ::core::ffi::c_int;
pub const FC_REMOVED: ::core::ffi::c_int = 0x20 as ::core::ffi::c_int;
pub const FC_SANDBOX: ::core::ffi::c_int = 0x40 as ::core::ffi::c_int;
pub const FC_NOARGS: ::core::ffi::c_int = 0x200 as ::core::ffi::c_int;
pub const FC_LUAREF: ::core::ffi::c_int = 0x800 as ::core::ffi::c_int;
pub const FUNCEXE_INIT: funcexe_T = funcexe_T {
    fe_argv_func: None,
    fe_firstline: 0 as linenr_T,
    fe_lastline: 0 as linenr_T,
    fe_doesrange: ::core::ptr::null_mut::<bool>(),
    fe_evaluate: false_0 != 0,
    fe_partial: ::core::ptr::null_mut::<partial_T>(),
    fe_selfdict: ::core::ptr::null_mut::<dict_T>(),
    fe_basetv: ::core::ptr::null_mut::<typval_T>(),
    fe_found_var: false_0 != 0,
};
pub const KS_EXTRA: ::core::ffi::c_int = 253 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const false_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub const MSG_BUF_LEN: ::core::ffi::c_int = 480 as ::core::ffi::c_int;
pub const MSG_BUF_CLEN: ::core::ffi::c_int = MSG_BUF_LEN / 6 as ::core::ffi::c_int;
pub const PROF_YES: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
