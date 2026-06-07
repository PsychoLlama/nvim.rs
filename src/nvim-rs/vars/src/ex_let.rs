//! Port of `ex_let` from `src/nvim/eval/vars.c`.
//!
//! Phase 4: `ex_let` handles `:let` / `:const` commands.
//!
//! Control flow:
//! 1. Parse the variable list with `rs_skip_var_list`.
//! 2. If no assignment operator is present → list variables.
//! 3. `=<<` heredoc path → call `rs_heredoc_get` + `rs_ex_let_vars`.
//! 4. Regular assignment path → evaluate the RHS via `nvim_vars_eval_let_expr`
//!    then assign via `rs_ex_let_vars`.

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::if_not_else)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::ptr_cast_constness)]
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void};

// Constants
const FAIL: c_int = 0;
const NUL: u8 = 0;

// Typval size (must match C's sizeof(typval_T))
const TYPVAL_SIZE: usize = 24;

// TypvalT v_type initial value (VAR_UNKNOWN)
const VAR_UNKNOWN: c_int = 0;

extern "C" {
    // --- skip_var_list (Rust export, already exported as rs_skip_var_list) ---
    fn rs_skip_var_list(
        arg: *const c_char,
        var_count: *mut c_int,
        semicolon: *mut c_int,
        silent: bool,
    ) -> *const c_char;

    // --- skipwhite ---
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // --- vim_strchr ---
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;

    // --- ends_excmd (via accessor) ---
    fn nvim_ends_excmd_char(c: c_int) -> bool;

    // --- emsg ---
    fn emsg(msg: *const c_char) -> c_int;

    // --- tv_clear ---
    fn tv_clear(tv: *mut c_void);

    // --- rs_list_arg_vars ---
    fn rs_list_arg_vars(eap: *const c_void, arg: *const c_char, first: *mut c_int)
        -> *const c_char;

    // --- rs_list_hashtable_vars ---
    fn rs_list_hashtable_vars(
        ht: *mut c_void,
        prefix: *const c_char,
        empty: c_int,
        first: *mut c_int,
    );

    // --- globvarht pointer accessor ---
    fn nvim_get_globvarht_ptr() -> *mut c_void;

    // --- per-scope listing helpers ---
    fn nvim_list_buf_vars(first: *mut c_int);
    fn nvim_list_win_vars(first: *mut c_int);
    fn nvim_list_tab_vars(first: *mut c_int);
    fn nvim_list_vim_vars(first: *mut c_int);
    fn nvim_list_script_vars(first: *mut c_int);
    fn nvim_list_func_vars(first: *mut c_int);

    // --- check_nextcmd / eap nextcmd setter ---
    fn nvim_check_nextcmd(arg: *mut c_char) -> *mut c_char;
    fn nvim_eap_set_nextcmd(eap: *mut c_void, val: *mut c_char);

    // --- eap accessors ---
    fn nvim_eap_get_arg(eap: *const c_void) -> *mut c_char;
    fn nvim_eap_get_skip_val(eap: *const c_void) -> c_int;
    fn nvim_eap_get_cmdidx(eap: *const c_void) -> c_int;

    // --- CMD_const constant ---
    fn nvim_cmd_const() -> c_int;

    // --- e_invarg string ---
    fn nvim_vars_e_invarg() -> *const c_char;

    // --- heredoc_get (Rust export rs_heredoc_get) ---
    fn rs_heredoc_get(eap: *mut c_void, cmd: *mut c_char, script_get: c_int) -> *mut c_void;

    // --- tv_list_set_ret accessor ---
    fn nvim_vars_tv_list_set_ret(tv: *mut c_void, l: *mut c_void);

    // --- ex_let_vars (Rust export rs_ex_let_vars) ---
    fn rs_ex_let_vars(
        arg_start: *mut c_char,
        tv: *mut c_void,
        copy: c_int,
        semicolon: c_int,
        var_count: c_int,
        is_const: c_int,
        op: *mut c_char,
    ) -> c_int;

    // --- eval_let_expr: wraps fill_evalarg+eval0+clear_evalarg ---
    fn nvim_vars_eval_let_expr(eap: *mut c_void, expr: *mut c_char, rettv: *mut c_void) -> c_int;
}

/// Port of C `ex_let`.
///
/// Handles `:let` and `:const` commands.
///
/// # Safety
/// `eap` must be a valid `exarg_T *`.
#[unsafe(export_name = "ex_let")]
pub unsafe extern "C" fn rs_ex_let(eap: *mut c_void) {
    let is_const = nvim_eap_get_cmdidx(eap) == nvim_cmd_const();
    let arg = nvim_eap_get_arg(eap);
    let mut arg: *mut c_char = arg;

    let mut var_count: c_int = 0;
    let mut semicolon: c_int = 0;

    let argend = rs_skip_var_list(arg, &raw mut var_count, &raw mut semicolon, false);
    if argend.is_null() {
        return;
    }

    let mut expr = skipwhite(argend);

    // Detect assignment operator
    let concat =
        *expr == b'.' as c_char && *expr.add(1) == b'.' as c_char && *expr.add(2) == b'=' as c_char;
    let has_assign = *expr == b'=' as c_char
        || (!vim_strchr(
            b"+-*/%.\0".as_ptr() as *const c_char,
            c_int::from(*expr as u8),
        )
        .is_null()
            && *expr.add(1) == b'=' as c_char);

    if !has_assign && !concat {
        // `:let` without `=`: list variables
        if *arg == b'[' as c_char {
            emsg(nvim_vars_e_invarg());
        } else if !nvim_ends_excmd_char(c_int::from(*arg as u8)) {
            // `:let var1 var2`
            arg = rs_list_arg_vars(eap, arg, std::ptr::null_mut()) as *mut c_char;
        } else if nvim_eap_get_skip_val(eap) == 0 {
            // `:let`
            let mut first: c_int = 1;
            rs_list_hashtable_vars(
                nvim_get_globvarht_ptr(),
                b"\0".as_ptr() as *const c_char,
                1,
                &raw mut first,
            );
            nvim_list_buf_vars(&raw mut first);
            nvim_list_win_vars(&raw mut first);
            nvim_list_tab_vars(&raw mut first);
            nvim_list_script_vars(&raw mut first);
            nvim_list_func_vars(&raw mut first);
            nvim_list_vim_vars(&raw mut first);
        }
        let next = nvim_check_nextcmd(arg);
        nvim_eap_set_nextcmd(eap, next);
        return;
    }

    // HERE document: `=<<`
    if *expr == b'=' as c_char && *expr.add(1) == b'<' as c_char && *expr.add(2) == b'<' as c_char {
        let l = rs_heredoc_get(eap, expr.add(3), 0);
        if !l.is_null() {
            let mut rettv_buf = [0u8; TYPVAL_SIZE];
            let rettv = rettv_buf.as_mut_ptr() as *mut c_void;
            nvim_vars_tv_list_set_ret(rettv, l);
            if nvim_eap_get_skip_val(eap) == 0 {
                let mut op = [b'=', NUL];
                rs_ex_let_vars(
                    nvim_eap_get_arg(eap),
                    rettv,
                    0,
                    semicolon,
                    var_count,
                    c_int::from(is_const),
                    op.as_mut_ptr() as *mut c_char,
                );
            }
            tv_clear(rettv);
        }
        return;
    }

    // Regular assignment: parse operator
    let mut op = [NUL, NUL, NUL]; // max 2 chars + NUL
    op[0] = b'=';
    op[1] = NUL;

    if *expr != b'=' as c_char {
        if !vim_strchr(
            b"+-*/%.\0".as_ptr() as *const c_char,
            c_int::from(*expr as u8),
        )
        .is_null()
        {
            op[0] = *expr as u8;
            if *expr == b'.' as c_char && *expr.add(1) == b'.' as c_char {
                // `..=`
                expr = expr.add(1);
            }
        }
        expr = expr.add(2);
    } else {
        expr = expr.add(1);
    }

    expr = skipwhite(expr);

    // Allocate rettv on the stack (all-zero = VAR_UNKNOWN)
    let mut rettv_buf = [0u8; TYPVAL_SIZE];
    let rettv = rettv_buf.as_mut_ptr() as *mut c_void;

    // Set v_type = VAR_UNKNOWN explicitly (redundant since zeroed, but matches C)
    let vtype_ptr = rettv as *mut c_int;
    *vtype_ptr = VAR_UNKNOWN;

    let eval_res = nvim_vars_eval_let_expr(eap, expr, rettv);

    if nvim_eap_get_skip_val(eap) == 0 && eval_res != FAIL {
        rs_ex_let_vars(
            nvim_eap_get_arg(eap),
            rettv,
            0,
            semicolon,
            var_count,
            c_int::from(is_const),
            op.as_mut_ptr() as *mut c_char,
        );
    }
    if eval_res != FAIL {
        tv_clear(rettv);
    }
}
