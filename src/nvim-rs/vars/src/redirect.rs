//! Variable existence check and output-redirection functions for VimL.
//!
//! Phase 6: Migrated from `src/nvim/eval/vars.c`.
//!
//! Functions:
//! - `rs_var_exists`: Check if a variable exists
//! - `rs_var_redir_start`: Start redirecting command output to a variable
//! - `rs_var_redir_stop`: Stop redirecting and assign the collected value

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::manual_c_str_literals)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_char, c_int, c_void};

// Typval size in bytes (must match sizeof(typval_T) = 24).
const TYPVAL_SIZE: usize = 24;

// OK / FAIL return codes
const OK: c_int = 1;
const FAIL: c_int = 0;

// VAR_STRING typval type constant (matches C VarType::VAR_STRING = 1)
const VAR_STRING: c_int = 1;

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // --- name lookup ---
    fn get_name_len(
        arg: *mut *const c_char,
        tofree: *mut *mut c_char,
        evaluate: bool,
        verbose: bool,
    ) -> c_int;

    // --- variable evaluation ---
    /// eval_variable wrapper: evaluates name[..len] into tv. dip=NULL, verbose=false, no_autoload=true.
    fn nvim_vars_eval_variable(name: *const c_char, len: c_int, tv: *mut c_void) -> c_int;

    /// handle_subscript wrapper: processes subscripts/calls on tv using EVALARG_EVALUATE, verbose=false.
    fn nvim_vars_handle_subscript_check(arg: *mut *const c_char, tv: *mut c_void) -> c_int;

    // --- typval ops ---
    fn tv_clear(tv: *mut c_void);

    // --- lval ops ---
    fn nvim_vars_get_lval(name: *mut c_char, lv: *mut c_void) -> *mut c_char;
    fn nvim_vars_set_var_lval(
        lv: *mut c_void,
        endp: *mut c_char,
        tv: *mut c_void,
        copy: bool,
        is_const: bool,
        op: *const c_char,
    );
    fn nvim_vars_clear_lval(lv: *mut c_void);
    fn nvim_lval_get_name(lv: *const c_void) -> *const c_char;
    fn nvim_vars_alloc_lval() -> *mut c_void;

    // --- redir state accessors ---
    fn nvim_get_redir_lval() -> *mut c_void;
    fn nvim_get_redir_varname() -> *mut c_char;
    fn nvim_vars_set_redir_lval(lv: *mut c_void);
    fn nvim_vars_set_redir_varname(n: *mut c_char);
    fn nvim_vars_set_redir_endp(e: *mut c_char);
    fn nvim_vars_get_redir_endp() -> *mut c_char;
    fn nvim_vars_redir_ga_init();
    fn nvim_vars_redir_ga_append_nul();
    fn nvim_vars_redir_ga_data() -> *mut c_void;
    fn nvim_vars_redir_ga_data_clear();
    fn nvim_vars_redir_lval_free();

    // --- emsg globals ---
    fn nvim_vars_get_called_emsg() -> c_int;
    fn nvim_vars_set_did_emsg(v: c_int);

    // --- char/name helpers ---
    fn rs_eval_isnamec1(c: c_int) -> bool;

    // --- memory ---
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);

    // --- error messages ---
    fn emsg(msg: *const c_char) -> c_int;
    fn semsg(fmt: *const c_char, ...) -> c_int;
}

/// Check if a variable `var` exists.
///
/// Matches C `var_exists`. Returns true if the variable exists and is valid.
///
/// # Safety
/// `var` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_var_exists(var: *const c_char) -> bool {
    let mut arg = var;
    let mut tofree: *mut c_char = std::ptr::null_mut();

    // get_name_len() handles curly-brace expansion.
    let len = get_name_len(&mut arg, &mut tofree, true, false);
    let mut found = false;

    if len > 0 {
        // Allocate tv on heap (24 bytes = sizeof(typval_T))
        let mut tv_buf = vec![0u8; TYPVAL_SIZE];
        let tv = tv_buf.as_mut_ptr() as *mut c_void;

        let name = if tofree.is_null() { var } else { tofree };
        if nvim_vars_eval_variable(name, len, tv) == OK {
            // Handle d.key, l[idx], f(expr).
            if nvim_vars_handle_subscript_check(&mut arg, tv) == OK {
                found = true;
                tv_clear(tv);
            }
        }
    }

    // If there are trailing characters, the variable doesn't "exist" as a clean name.
    if !found || *arg != 0 {
        found = false;
    }

    xfree(tofree as *mut c_void);
    found
}

/// Start recording command output to a variable.
///
/// Matches C `var_redir_start`. Returns OK (1) or FAIL (0).
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_var_redir_start(name: *mut c_char, append: bool) -> c_int {
    // Catch a bad name early.
    if !rs_eval_isnamec1(*name as c_int) {
        emsg(b"E474: Invalid argument\0".as_ptr() as *const c_char);
        return FAIL;
    }

    // Make a copy of the name; it's used until redirection ends.
    let varname_copy = xstrdup(name);
    nvim_vars_set_redir_varname(varname_copy);

    // Allocate lval_T.
    let lv = nvim_vars_alloc_lval();
    nvim_vars_set_redir_lval(lv);

    // Initialize the growing-array to collect output.
    nvim_vars_redir_ga_init();

    // Parse the variable name (may be a dict/list entry).
    let endp = nvim_vars_get_lval(varname_copy, lv);
    nvim_vars_set_redir_endp(endp);

    if endp.is_null() || nvim_lval_get_name(lv).is_null() || *endp != 0 {
        nvim_vars_clear_lval(lv);
        let redir_endp_val = nvim_vars_get_redir_endp();
        if !redir_endp_val.is_null() && *redir_endp_val != 0 {
            semsg(
                b"E488: Trailing characters: %s\0".as_ptr() as *const c_char,
                redir_endp_val,
            );
        } else {
            semsg(
                b"E475: Invalid argument: %s\0".as_ptr() as *const c_char,
                name,
            );
        }
        nvim_vars_set_redir_endp(std::ptr::null_mut());
        rs_var_redir_stop();
        return FAIL;
    }

    // Check we can write to the variable: set to empty string.
    let called_emsg_before = nvim_vars_get_called_emsg();
    nvim_vars_set_did_emsg(0);

    // Build a VAR_STRING typval with vval.v_string = "".
    let mut tv_buf = vec![0u8; TYPVAL_SIZE];
    let tv = tv_buf.as_mut_ptr() as *mut c_void;
    // Set v_type (first field, offset 0) = VAR_STRING (1)
    let vtype_ptr = tv as *mut c_int;
    *vtype_ptr = VAR_STRING;
    // Set vval.v_string (at offset 8) = pointer to empty C string
    let vstring_ptr = tv.add(8) as *mut *const c_char;
    *vstring_ptr = b"\0".as_ptr() as *const c_char;

    let op = if append {
        b".\0".as_ptr() as *const c_char
    } else {
        b"=\0".as_ptr() as *const c_char
    };
    nvim_vars_set_var_lval(lv, endp, tv, true, false, op);
    nvim_vars_clear_lval(lv);

    if nvim_vars_get_called_emsg() > called_emsg_before {
        nvim_vars_set_redir_endp(std::ptr::null_mut());
        rs_var_redir_stop();
        return FAIL;
    }

    OK
}

/// Stop redirecting command output to a variable.
///
/// Matches C `var_redir_stop`. Assigns collected output and frees all state.
///
/// # Safety
/// No parameters; operates on C-side statics via accessors.
#[no_mangle]
pub unsafe extern "C" fn rs_var_redir_stop() {
    let redir_lval = nvim_get_redir_lval();
    if !redir_lval.is_null() {
        let redir_endp = nvim_vars_get_redir_endp();
        if !redir_endp.is_null() {
            // Append trailing NUL to collected output.
            nvim_vars_redir_ga_append_nul();

            // Build VAR_STRING typval pointing to the collected data.
            let mut tv_buf = vec![0u8; TYPVAL_SIZE];
            let tv = tv_buf.as_mut_ptr() as *mut c_void;
            let vtype_ptr = tv as *mut c_int;
            *vtype_ptr = VAR_STRING;
            let ga_data = nvim_vars_redir_ga_data();
            let vstring_ptr = tv.add(8) as *mut *mut c_void;
            *vstring_ptr = ga_data;

            // Re-parse the variable name (dict/list item may have moved).
            let redir_varname = nvim_get_redir_varname();
            let new_endp = nvim_vars_get_lval(redir_varname, redir_lval);
            nvim_vars_set_redir_endp(new_endp);
            let redir_endp = nvim_vars_get_redir_endp();
            if !redir_endp.is_null() && !nvim_lval_get_name(redir_lval).is_null() {
                nvim_vars_set_var_lval(
                    redir_lval,
                    redir_endp,
                    tv,
                    false,
                    false,
                    b".\0".as_ptr() as *const c_char,
                );
            }
            nvim_vars_clear_lval(redir_lval);
        }

        // Free collected output.
        nvim_vars_redir_ga_data_clear();

        // Free lval.
        nvim_vars_redir_lval_free();
    }

    // Free varname.
    let varname = nvim_get_redir_varname();
    if !varname.is_null() {
        xfree(varname as *mut c_void);
        nvim_vars_set_redir_varname(std::ptr::null_mut());
    }
}
