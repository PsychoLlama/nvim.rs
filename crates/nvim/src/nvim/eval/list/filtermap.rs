//! Walking a container and applying an expression to every item --
//! `filter()`, `map()`, `mapnew()` and `foreach()`.
//!
//! All four are one `filter_map` with a `filtermap_T` saying what to do with
//! each result: drop the item, replace it, collect it into a fresh container,
//! or throw it away.  `filter_map_one` is the per-item half -- it sets `v:key`
//! and `v:val`, evaluates the expression or calls the Funcref, and reports
//! whether the walk should keep going -- and [`containers`] is the four
//! per-container walks it is driven from.  The whole family re-enters the
//! evaluator on every item, so a callback may lock, unlock, extend or free the
//! very container being walked; the checks that make that survivable are the
//! reason these functions are as long as they are.
//!
//! Original: `src/nvim/eval/list.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{
    FAIL, FILTERMAP_FILTER, FILTERMAP_FOREACH, FILTERMAP_MAP, FILTERMAP_MAPNEW, OK,
    e_argument_of_str_must_be_list_string_dictionary_or_blob, false_0, filtermap_T,
};
use crate::semsg_c;
use crate::src::nvim::eval::eval_expr_typval;
use crate::src::nvim::eval::typval::{tv_clear, tv_copy, tv_get_number_chk, tv_get_string};
use crate::src::nvim::eval::vars::{get_vim_var_tv, prepare_vimvar, restore_vimvar};
use crate::src::nvim::ex_docmd::do_cmdline_cmd;
use crate::src::nvim::main::did_emsg;
use crate::src::nvim::types::{
    EvalFuncData, VAR_BLOB, VAR_DICT, VAR_LIST, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, VV_KEY,
    VV_VAL, typval_T, typval_vval_union, varnumber_T,
};

// The carve of the transpiled module; see each child's docs.
mod containers;

pub(crate) use self::containers::*;

unsafe extern "C" fn filter_map_one(
    mut tv: *mut typval_T,
    mut expr: *mut typval_T,
    filtermap: filtermap_T,
    mut newtv: *mut typval_T,
    mut remp: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut argv: [typval_T; 3] = [typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        }; 3];
        let mut retval: ::core::ffi::c_int = FAIL;
        tv_copy(tv, get_vim_var_tv(VV_VAL));
        (*newtv).v_type = VAR_UNKNOWN;
        '_theend: {
            if filtermap as ::core::ffi::c_uint
                == FILTERMAP_FOREACH as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*expr).v_type as ::core::ffi::c_uint
                    == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                do_cmdline_cmd((*expr).vval.v_string);
                if did_emsg.get() == 0 {
                    retval = OK;
                }
            } else {
                argv[0 as ::core::ffi::c_int as usize] = *get_vim_var_tv(VV_KEY);
                argv[1 as ::core::ffi::c_int as usize] = *get_vim_var_tv(VV_VAL);
                if eval_expr_typval(
                    expr,
                    false_0 != 0,
                    &raw mut argv as *mut typval_T,
                    2 as ::core::ffi::c_int,
                    newtv,
                ) != FAIL
                {
                    if filtermap as ::core::ffi::c_uint
                        == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let mut error: bool = false_0 != 0;
                        *remp = tv_get_number_chk(newtv, &raw mut error) == 0 as varnumber_T;
                        tv_clear(newtv);
                        if error {
                            break '_theend;
                        }
                    } else if filtermap as ::core::ffi::c_uint
                        == FILTERMAP_FOREACH as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        tv_clear(newtv);
                    }
                    retval = OK;
                }
            }
        }
        tv_clear(get_vim_var_tv(VV_VAL));
        return retval;
    }
}

unsafe extern "C" fn filter_map(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut filtermap: filtermap_T,
) {
    unsafe {
        let func_name: *const ::core::ffi::c_char = if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            c"map()".as_ptr()
        } else if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            c"mapnew()".as_ptr()
        } else if filtermap as ::core::ffi::c_uint
            == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            c"filter()".as_ptr()
        } else {
            c"foreach()".as_ptr()
        };
        let arg_errmsg: *const ::core::ffi::c_char = if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAP as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            c"map() argument".as_ptr()
        } else if filtermap as ::core::ffi::c_uint
            == FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            c"mapnew() argument".as_ptr()
        } else if filtermap as ::core::ffi::c_uint
            == FILTERMAP_FILTER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            c"filter() argument".as_ptr()
        } else {
            c"foreach() argument".as_ptr()
        };
        if filtermap as ::core::ffi::c_uint
            != FILTERMAP_MAPNEW as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            tv_copy(argvars.offset(0 as ::core::ffi::c_int as isize), rettv);
        }
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            semsg_c!(
                e_argument_of_str_must_be_list_string_dictionary_or_blob.as_ptr()
                    as *mut ::core::ffi::c_char,
                func_name,
            );
            return;
        }
        let mut expr: *mut typval_T = argvars.offset(1 as ::core::ffi::c_int as isize);
        if (*expr).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return;
        }
        let mut save_val: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut save_key: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        prepare_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
        prepare_vimvar(VV_KEY as ::core::ffi::c_int, &raw mut save_key);
        let mut save_did_emsg: ::core::ffi::c_int = did_emsg.get();
        did_emsg.set(false_0);
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_DICT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            filter_map_dict(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_dict,
                filtermap,
                func_name,
                arg_errmsg,
                expr,
                rettv,
            );
        } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_BLOB as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            filter_map_blob(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_blob,
                filtermap,
                expr,
                arg_errmsg,
                rettv,
            );
        } else if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            filter_map_string(
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
                filtermap,
                expr,
                rettv,
            );
        } else {
            debug_assert!(
                (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint,
                "argvars[0].v_type == VAR_LIST"
            );
            filter_map_list(
                (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list,
                filtermap,
                func_name,
                arg_errmsg,
                expr,
                rettv,
            );
        }
        restore_vimvar(VV_KEY as ::core::ffi::c_int, &raw mut save_key);
        restore_vimvar(VV_VAL as ::core::ffi::c_int, &raw mut save_val);
        (*did_emsg.ptr()) |= save_did_emsg;
    }
}

pub unsafe extern "C" fn f_filter(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        filter_map(argvars, rettv, FILTERMAP_FILTER);
    }
}

pub unsafe extern "C" fn f_map(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        filter_map(argvars, rettv, FILTERMAP_MAP);
    }
}

pub unsafe extern "C" fn f_mapnew(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        filter_map(argvars, rettv, FILTERMAP_MAPNEW);
    }
}

pub unsafe extern "C" fn f_foreach(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        filter_map(argvars, rettv, FILTERMAP_FOREACH);
    }
}
