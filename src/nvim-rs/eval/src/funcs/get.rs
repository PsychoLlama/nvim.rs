//! Rust implementation of the VimL `get()` built-in function.
//!
//! Ported from `f_get` in `src/nvim/eval/funcs_shim.c` (Phase 30).
//! The C body has been deleted; this file is the authoritative implementation.

#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::ptr_cast_constness)]
#![allow(clashing_extern_declarations)]

use std::ffi::{c_char, c_int, c_void};

use crate::typval::PartialT;
use crate::typval::TypvalT;

// ─── Type constants ───────────────────────────────────────────────────────────

const VAR_UNKNOWN: c_int = 0;
const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;
const VAR_FUNC: c_int = 3;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;
const VAR_PARTIAL: c_int = 9;
const VAR_BLOB: c_int = 10;

/// `kBoolVarFalse = 0`, `kBoolVarTrue = 1` (BoolVarValue enum)
const K_BOOL_VAR_FALSE: c_int = 0;
const K_BOOL_VAR_TRUE: c_int = 1;

/// `EvalFuncData` — opaque union passed by value
type EvalFuncData = *mut c_void;

// ─── FFI imports ─────────────────────────────────────────────────────────────

extern "C" {
    // Typval accessors
    fn tv_get_string(tv: *const c_void) -> *const c_char;
    fn tv_get_number_chk(tv: *const c_void, error: *mut bool) -> i64;

    // Blob accessors (via wrappers for static-inline C functions)
    fn nvim_eval_blob_len(b: *const c_void) -> c_int;
    fn nvim_eval_blob_get(b: *const c_void, idx: c_int) -> u8;

    // List accessors
    fn tv_list_find(l: *mut c_void, n: c_int) -> *mut c_void;
    fn tv_list_alloc_ret(rettv: *mut c_void, len: isize) -> *mut c_void;
    fn tv_list_append_tv(l: *mut c_void, tv: *const c_void);
    // TV_LIST_ITEM_TV accessor (wraps macro in C)
    fn nvim_tv_list_item_tv(li: *mut c_void) -> *mut c_void;

    // Dict accessors
    fn tv_dict_find(d: *const c_void, key: *const c_char, len: isize) -> *mut c_void;
    fn nvim_eval_tv_dict_set_ret(tv: *mut c_void, d: *mut c_void);
    fn tv_dict_alloc_ret(rettv: *mut c_void);
    fn tv_dict_add_nr(d: *mut c_void, key: *const c_char, key_len: usize, nr: i64) -> c_int;
    fn tv_dict_add_bool(d: *mut c_void, key: *const c_char, key_len: usize, val: c_int) -> c_int;

    // nvim_eval_tv_is_func: returns 1 if v_type is VAR_FUNC or VAR_PARTIAL, else 0.
    // Existing function at funcs_shim.c:635.
    fn nvim_eval_tv_is_func(tv: *const c_void) -> c_int;

    // Funcref helpers
    fn rs_partial_name(pt: *const c_void) -> *mut c_char;
    fn func_ref(name: *const c_char);
    fn printable_func_name(fp: *mut c_void) -> *const c_char;
    fn get_func_arity(
        name: *const c_char,
        required: *mut c_int,
        optional: *mut c_int,
        varargs: *mut bool,
    ) -> c_int;
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // Error messages
    fn semsg(fmt: *const c_char, ...);
    static e_invarg2: [c_char; 0];
    static e_listdictblobarg: [c_char; 0];

    // tv_copy: deep-copy a typval
    fn tv_copy(from: *const c_void, to: *mut c_void);

    // gettext (macro _() in C)
    fn gettext(msg: *const c_char) -> *const c_char;
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Return a pointer to `argvars[i]` (each typval_T is 16 bytes).
///
/// # Safety
///
/// `argvars` must be valid for at least `i+1` elements.
#[inline]
unsafe fn argvar(argvars: *const c_void, i: usize) -> *const c_void {
    argvars.cast::<u8>().add(i * 16).cast::<c_void>()
}

/// Compare a C string at `ptr` against a Rust `&str` literal.
///
/// # Safety
///
/// `ptr` must be a valid NUL-terminated C string.
#[inline]
unsafe fn c_str_eq(ptr: *const c_char, s: &[u8]) -> bool {
    let mut p = ptr.cast::<u8>();
    let mut i = 0;
    loop {
        if i == s.len() {
            return *p == 0;
        }
        if *p != s[i] {
            return false;
        }
        p = p.add(1);
        i += 1;
    }
}

// ─── f_get ───────────────────────────────────────────────────────────────────

/// `get()` VimL function — polymorphic list/dict/blob/funcref introspection.
///
/// # Safety
///
/// `argvars` and `rettv` must be valid `typval_T *`.
#[export_name = "f_get"]
pub unsafe extern "C" fn rs_f_get(argvars: *const c_void, rettv: *mut c_void, _fptr: EvalFuncData) {
    // tv: non-null means we have a value to copy at the end
    let mut tv: *const c_void = std::ptr::null();
    let mut what_is_dict = false;

    let arg0 = argvar(argvars, 0);
    let arg0_type = (*arg0.cast::<TypvalT>()).v_type;

    if arg0_type == VAR_BLOB {
        let mut error = false;
        let arg1 = argvar(argvars, 1);
        let mut idx = tv_get_number_chk(arg1, &raw mut error) as c_int;

        if !error {
            (*rettv.cast::<TypvalT>()).v_type = VAR_NUMBER;
            let blob = (*arg0.cast::<TypvalT>()).vval.v_blob;
            if idx < 0 {
                idx += nvim_eval_blob_len(blob);
            }
            if idx < 0 || idx >= nvim_eval_blob_len(blob) {
                (*rettv.cast::<TypvalT>()).vval.v_number = -1;
            } else {
                (*rettv.cast::<TypvalT>()).vval.v_number = i64::from(nvim_eval_blob_get(blob, idx));
                tv = rettv.cast_const();
            }
        }
    } else if arg0_type == VAR_LIST {
        let l = (*arg0.cast::<TypvalT>()).vval.v_list;
        if !l.is_null() {
            let mut error = false;
            let arg1 = argvar(argvars, 1);
            let idx = tv_get_number_chk(arg1, &raw mut error) as c_int;
            if !error {
                let li = tv_list_find(l, idx);
                if !li.is_null() {
                    tv = nvim_tv_list_item_tv(li).cast_const();
                }
            }
        }
    } else if arg0_type == VAR_DICT {
        let d = (*arg0.cast::<TypvalT>()).vval.v_dict;
        if !d.is_null() {
            let arg1 = argvar(argvars, 1);
            let key = tv_get_string(arg1);
            let di = tv_dict_find(d.cast_const(), key, -1);
            if !di.is_null() {
                // di_tv is at offset 0 of dictitem_T
                tv = di.cast_const();
            }
        }
    } else if nvim_eval_tv_is_func(arg0) != 0 {
        // Build a partial pointer — either from the VAR_PARTIAL, or a synthetic
        // stack-local partial for VAR_FUNC (zero-init with only pt_name set).
        let mut fref_pt: PartialT = std::mem::zeroed();
        let pt: *mut PartialT = if arg0_type == VAR_PARTIAL {
            (*arg0.cast::<TypvalT>()).vval.v_partial.cast::<PartialT>()
        } else {
            fref_pt.pt_name = (*arg0.cast::<TypvalT>()).vval.v_string;
            &raw mut fref_pt
        };

        if !pt.is_null() {
            let arg1 = argvar(argvars, 1);
            let what = tv_get_string(arg1);

            if c_str_eq(what, b"func") || c_str_eq(what, b"name") {
                let name = rs_partial_name(pt.cast::<c_void>());
                // what[0] == 'f' → VAR_FUNC, else → VAR_STRING
                (*rettv.cast::<TypvalT>()).v_type = if *what.cast::<u8>() == b'f' {
                    VAR_FUNC
                } else {
                    VAR_STRING
                };
                // name is never NULL per rs_partial_name contract
                if (*rettv.cast::<TypvalT>()).v_type == VAR_FUNC {
                    func_ref(name);
                }
                // For "name" with pt_name==NULL and pt_func!=NULL, use <SNR> form
                let final_name: *const c_char = if *what.cast::<u8>() == b'n'
                    && (*pt).pt_name.is_null()
                    && !(*pt).pt_func.is_null()
                {
                    printable_func_name((*pt).pt_func)
                } else {
                    name
                };
                (*rettv.cast::<TypvalT>()).vval.v_string = xstrdup(final_name);
            } else if c_str_eq(what, b"dict") {
                what_is_dict = true;
                if !(*pt).pt_dict.is_null() {
                    nvim_eval_tv_dict_set_ret(rettv, (*pt).pt_dict);
                }
            } else if c_str_eq(what, b"args") {
                (*rettv.cast::<TypvalT>()).v_type = VAR_LIST;
                let retlist = tv_list_alloc_ret(rettv, (*pt).pt_argc as isize);
                let partial_argc = (*pt).pt_argc;
                for i in 0..partial_argc {
                    let item_tv = (*pt).pt_argv.add(i as usize);
                    tv_list_append_tv(retlist, item_tv.cast::<c_void>());
                }
            } else if c_str_eq(what, b"arity") {
                let mut required: c_int = 0;
                let mut optional: c_int = 0;
                let mut varargs = false;
                let arity_name = rs_partial_name(pt.cast::<c_void>());

                get_func_arity(
                    arity_name,
                    &raw mut required,
                    &raw mut optional,
                    &raw mut varargs,
                );

                (*rettv.cast::<TypvalT>()).v_type = VAR_DICT;
                tv_dict_alloc_ret(rettv);
                let dict = (*rettv.cast::<TypvalT>()).vval.v_dict;

                // Adjust for partial's pre-supplied arguments.
                let pt_argc = (*pt).pt_argc;
                if pt_argc >= required + optional {
                    required = 0;
                    optional = 0;
                } else if pt_argc > required {
                    optional -= pt_argc - required;
                    required = 0;
                } else {
                    required -= pt_argc;
                }

                tv_dict_add_nr(dict, c"required".as_ptr(), 8, i64::from(required));
                tv_dict_add_nr(dict, c"optional".as_ptr(), 8, i64::from(optional));
                tv_dict_add_bool(
                    dict,
                    c"varargs".as_ptr(),
                    7,
                    if varargs {
                        K_BOOL_VAR_TRUE
                    } else {
                        K_BOOL_VAR_FALSE
                    },
                );
            } else {
                semsg(gettext(e_invarg2.as_ptr()), what);
            }

            // When {what} == "dict" and pt->pt_dict == NULL, fall through to
            // evaluate the third argument (default value). For all other what
            // values, return immediately (no default copy).
            if !what_is_dict {
                return;
            }
        }
    } else {
        semsg(gettext(e_listdictblobarg.as_ptr()), c"get()".as_ptr());
    }

    // Default copy fallthrough
    let arg2 = argvar(argvars, 2);
    if tv.is_null() {
        if (*arg2.cast::<TypvalT>()).v_type != VAR_UNKNOWN {
            tv_copy(arg2, rettv);
        }
    } else {
        tv_copy(tv, rettv);
    }
}
