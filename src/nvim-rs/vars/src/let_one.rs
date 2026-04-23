//! :let assignment functions for VimL.
//!
//! Phase 10: Migrated from `src/nvim/eval/vars.c`.
//!
//! Functions:
//! - `rs_ex_let_one`: Set one variable from :let (handles $, &, @, name)
//! - `rs_ex_let_vars`: Assign to variable or list of variables from :let/:for

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_char, c_int, c_void};

// OK / FAIL return codes
const OK: c_int = 1;
const FAIL: c_int = 0;

// VAR_LIST typval type constant (matches C VarType::VAR_LIST)
// We fetch this at runtime via nvim_var_list() in unlet_lock.rs; here we declare it as extern.

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // --- char classification / string ops ---
    fn rs_eval_isnamec1(c: c_int) -> bool;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // --- lval_T ops ---
    fn nvim_vars_alloc_lval() -> *mut c_void;
    fn nvim_vars_clear_lval(lv: *mut c_void);
    fn nvim_lval_get_name(lv: *const c_void) -> *const c_char;
    fn nvim_vars_get_lval_with_tv(
        name: *mut c_char,
        tv: *mut c_void,
        lv: *mut c_void,
    ) -> *mut c_char;
    fn nvim_vars_set_var_lval(
        lv: *mut c_void,
        endp: *mut c_char,
        tv: *mut c_void,
        copy: bool,
        is_const: bool,
        op: *const c_char,
    );

    // --- error messages ---
    fn emsg(msg: *const c_char) -> c_int;
    fn semsg(fmt: *const c_char, ...) -> c_int;
    fn nvim_vars_emsg_letunexp();

    // --- memory ---
    fn xfree(ptr: *mut c_void);

    // --- typval type query ---
    fn nvim_tv_get_type(tv: *mut c_void) -> c_int;
    fn nvim_tv_get_list(tv: *mut c_void) -> *mut c_void;
    fn nvim_var_list() -> c_int;

    // --- list ops ---
    fn rs_list_len(list: *mut c_void) -> c_int;
    fn rs_list_first(list: *mut c_void) -> *mut c_void;
    fn rs_listitem_next(item: *mut c_void) -> *mut c_void;
    fn rs_listitem_tv(item: *mut c_void) -> *mut c_void;

    // --- rest-list construction (helper for semicolon case) ---
    fn nvim_vars_build_rest_list(item: *mut c_void, ltv: *mut c_void, rest_len: usize);

    // --- tv_clear ---
    fn tv_clear(tv: *mut c_void);

    // --- internal_error ---
    fn internal_error(where_: *const c_char);
}

// Typval size in bytes (must match sizeof(typval_T) = 24)
const TYPVAL_SIZE: usize = 24;

/// Set one item in a :let assignment.
///
/// Matches C `ex_let_one`. Returns pointer to char just after var name,
/// or NULL on error.
///
/// # Safety
/// `arg` must be a valid C string. `tv` must be a valid typval_T*.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_let_one(
    arg: *mut c_char,
    tv: *mut c_void,
    copy: bool,
    is_const: bool,
    endchars: *const c_char,
    op: *const c_char,
) -> *mut c_char {
    if *arg == b'$' as c_char {
        // ":let $VAR = expr": Set environment variable.
        crate::eval_helpers::rs_ex_let_env(arg, tv, is_const, endchars, op)
    } else if *arg == b'&' as c_char {
        // ":let &option = expr": Set option value.
        crate::option_conv::rs_ex_let_option(arg, tv, is_const, endchars, op)
    } else if *arg == b'@' as c_char {
        // ":let @r = expr": Set register contents.
        crate::eval_helpers::rs_ex_let_register(arg, tv, is_const, endchars, op)
    } else if rs_eval_isnamec1(*arg as c_int) || *arg == b'{' as c_char {
        // ":let var = expr": Set internal variable.
        let lv = nvim_vars_alloc_lval();
        let mut arg_end: *mut c_char = std::ptr::null_mut();

        let p = nvim_vars_get_lval_with_tv(arg, tv, lv);
        if !p.is_null() && !nvim_lval_get_name(lv).is_null() {
            if !endchars.is_null()
                && vim_strchr(endchars, *skipwhite(p) as c_int as u8 as c_int).is_null()
            {
                nvim_vars_emsg_letunexp();
            } else {
                nvim_vars_set_var_lval(lv, p, tv, copy, is_const, op);
                arg_end = p;
            }
        }
        nvim_vars_clear_lval(lv);
        xfree(lv);
        arg_end
    } else {
        semsg(
            b"E475: Invalid argument: %s\0".as_ptr() as *const c_char,
            arg,
        );
        std::ptr::null_mut()
    }
}

/// Assign typval "tv" to the variable or variables starting at "arg_start".
///
/// Matches C `ex_let_vars`. Returns OK or FAIL.
///
/// # Safety
/// `arg_start` must be a valid mutable C string. `tv` must be a valid typval_T*.
#[no_mangle]
pub unsafe extern "C" fn rs_ex_let_vars(
    arg_start: *mut c_char,
    tv: *mut c_void,
    copy: c_int,
    semicolon: c_int,
    var_count: c_int,
    is_const: c_int,
    op: *mut c_char,
) -> c_int {
    let mut arg = arg_start;

    if *arg != b'[' as c_char {
        // ":let var = expr" or ":for var in list"
        if rs_ex_let_one(arg, tv, copy != 0, is_const != 0, op, op).is_null() {
            return FAIL;
        }
        return OK;
    }

    // ":let [v1, v2] = list" or ":for [v1, v2] in listlist"
    let var_list_type = nvim_var_list();
    if nvim_tv_get_type(tv) != var_list_type {
        emsg(b"E714: List required\0".as_ptr() as *const c_char);
        return FAIL;
    }
    let l = nvim_tv_get_list(tv);

    let len = rs_list_len(l);
    if semicolon == 0 && var_count < len {
        emsg(b"E687: Less targets than List items\0".as_ptr() as *const c_char);
        return FAIL;
    }
    if var_count - semicolon > len {
        emsg(b"E688: More targets than List items\0".as_ptr() as *const c_char);
        return FAIL;
    }

    let mut item = rs_list_first(l);
    let mut rest_len = len as usize;

    while *arg != b']' as c_char {
        arg = skipwhite(arg.add(1));
        let item_tv = rs_listitem_tv(item);
        arg = rs_ex_let_one(
            arg,
            item_tv,
            true,
            is_const != 0,
            b",;]\0".as_ptr() as *const c_char,
            op,
        );
        if arg.is_null() {
            return FAIL;
        }
        rest_len -= 1;

        item = rs_listitem_next(item);
        arg = skipwhite(arg);
        if *arg == b';' as c_char {
            // Put the rest of the list (may be empty) in the var after ';'.
            let mut ltv_buf = [0u8; TYPVAL_SIZE];
            let ltv = ltv_buf.as_mut_ptr() as *mut c_void;
            nvim_vars_build_rest_list(item, ltv, rest_len);

            arg = rs_ex_let_one(
                skipwhite(arg.add(1)),
                ltv,
                false,
                is_const != 0,
                b"]\0".as_ptr() as *const c_char,
                op,
            );
            tv_clear(ltv);
            if arg.is_null() {
                return FAIL;
            }
            break;
        } else if *arg != b',' as c_char && *arg != b']' as c_char {
            internal_error(b"ex_let_vars()\0".as_ptr() as *const c_char);
            return FAIL;
        }
    }

    OK
}
