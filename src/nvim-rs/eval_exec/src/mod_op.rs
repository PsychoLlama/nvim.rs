//! Modifier-assignment operator implementation (`+=`, `-=`, `*=`, `/=`, `%=`, `.=`).
//!
//! Migrated from `src/nvim/eval/executor.c` (Phase 2 of f39b5673).
//! Implements `eexe_mod_op` and its helpers: tv_op_blob, tv_op_list,
//! tv_op_number, tv_op_string, tv_op_nr_or_string, tv_op_float.

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int, c_void};

use nvim_eval::typval::TypvalT as TypvalTRepr;

use crate::eval::TypevalHandle;

// =============================================================================
// Constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;

// VarType constants (from typval_defs.h)
const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;
const VAR_FUNC: c_int = 3;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;
const VAR_FLOAT: c_int = 6;
const VAR_BOOL: c_int = 7;
const VAR_SPECIAL: c_int = 8;
const VAR_PARTIAL: c_int = 9;
const VAR_BLOB: c_int = 10;

// Buffer size for number-to-string conversion (NUMBUFLEN in C)
const NUMBUFLEN: usize = 65;

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // Value extraction
    fn tv_get_number(tv: TypevalHandle) -> i64;
    fn tv_get_string(tv: TypevalHandle) -> *const c_char;
    fn tv_get_string_buf(tv: TypevalHandle, buf: *mut c_char) -> *const c_char;

    // Typval operations
    fn tv_clear(tv: TypevalHandle);

    // String operations
    fn concat_str(str1: *const c_char, str2: *const c_char) -> *mut c_char;
    fn vim_strchr(string: *const c_char, c: c_int) -> *mut c_char;

    // Blob accessors (nvim_* wrappers in typval.c)
    fn nvim_blob_get_len(b: *const c_void) -> c_int;
    fn nvim_blob_get_byte(b: *const c_void, idx: c_int) -> u8;
    fn nvim_blob_ga_append(b: *mut c_void, c: u8);
    fn nvim_blob_inc_refcount(b: *mut c_void);

    // List accessors
    fn nvim_list_ref(l: *mut c_void);
    fn tv_list_extend(l1: *mut c_void, l2: *mut c_void, bef: *mut c_void);

    // Error message
    fn semsg(fmt: *const c_char, ...) -> c_int;

    // e_letwrong: "E734: Wrong variable type for %s="
    static e_letwrong: c_char;

    // rs_num_divide / rs_num_modulus from the arithmetic crate
    fn rs_num_divide(n1: i64, n2: i64) -> i64;
    fn rs_num_modulus(n1: i64, n2: i64) -> i64;
}

// =============================================================================
// Field access helpers
// =============================================================================

/// Get `v_type` from a typval handle.
#[inline]
unsafe fn tv_get_type(tv: TypevalHandle) -> c_int {
    (*tv.as_ptr().cast::<TypvalTRepr>()).v_type
}

/// Get `vval.v_float` from a typval handle (VAR_FLOAT).
#[inline]
unsafe fn tv_get_vfloat(tv: TypevalHandle) -> f64 {
    (*tv.as_ptr().cast::<TypvalTRepr>()).vval.v_float
}

/// Set `v_type = VAR_FLOAT` and `vval.v_float = f`.
#[inline]
unsafe fn tv_set_float(tv: TypevalHandle, f: f64) {
    let t = &mut *tv.as_ptr().cast::<TypvalTRepr>();
    t.v_type = VAR_FLOAT;
    t.vval.v_float = f;
}

/// Set `v_type = VAR_NUMBER` and `vval.v_number = n`.
#[inline]
unsafe fn tv_set_number(tv: TypevalHandle, n: i64) {
    let t = &mut *tv.as_ptr().cast::<TypvalTRepr>();
    t.v_type = VAR_NUMBER;
    t.vval.v_number = n;
}

/// Set `v_type = VAR_STRING` and `vval.v_string = s`.
#[inline]
unsafe fn tv_set_string_owned(tv: TypevalHandle, s: *mut c_char) {
    let t = &mut *tv.as_ptr().cast::<TypvalTRepr>();
    t.v_type = VAR_STRING;
    t.vval.v_string = s;
}

/// Get `vval.v_blob` (blob_T pointer) from a typval handle (VAR_BLOB).
#[inline]
unsafe fn tv_get_vblob(tv: TypevalHandle) -> *mut c_void {
    (*tv.as_ptr().cast::<TypvalTRepr>()).vval.v_blob
}

/// Set `vval.v_blob` in a typval handle (does NOT change v_type).
#[inline]
unsafe fn tv_set_vblob(tv: TypevalHandle, b: *mut c_void) {
    (*tv.as_ptr().cast::<TypvalTRepr>()).vval.v_blob = b;
}

/// Get `vval.v_list` (list_T pointer) from a typval handle (VAR_LIST).
#[inline]
unsafe fn tv_get_vlist(tv: TypevalHandle) -> *mut c_void {
    (*tv.as_ptr().cast::<TypvalTRepr>()).vval.v_list
}

/// Set `vval.v_list` in a typval handle (does NOT change v_type).
#[inline]
unsafe fn tv_set_vlist(tv: TypevalHandle, l: *mut c_void) {
    (*tv.as_ptr().cast::<TypvalTRepr>()).vval.v_list = l;
}

// =============================================================================
// Operator implementations
// =============================================================================

/// Handle "blob1 += blob2".
/// Returns OK or FAIL.
unsafe fn tv_op_blob(tv1: TypevalHandle, tv2: TypevalHandle, op: *const c_char) -> c_int {
    if *op != b'+' as c_char || tv_get_type(tv2) != VAR_BLOB {
        return FAIL;
    }

    // Blob += Blob
    let b2 = tv_get_vblob(tv2);
    if b2.is_null() {
        return OK;
    }

    let b1 = tv_get_vblob(tv1);
    if b1.is_null() {
        // tv1->vval.v_blob = b2; b2->bv_refcount++
        tv_set_vblob(tv1, b2);
        nvim_blob_inc_refcount(b2);
        return OK;
    }

    let len = nvim_blob_get_len(b2 as *const c_void);
    for i in 0..len {
        nvim_blob_ga_append(b1, nvim_blob_get_byte(b2 as *const c_void, i));
    }

    OK
}

/// Handle "list1 += list2".
/// Returns OK or FAIL.
unsafe fn tv_op_list(tv1: TypevalHandle, tv2: TypevalHandle, op: *const c_char) -> c_int {
    if *op != b'+' as c_char || tv_get_type(tv2) != VAR_LIST {
        return FAIL;
    }

    // List += List
    let l2 = tv_get_vlist(tv2);
    if l2.is_null() {
        return OK;
    }

    let l1 = tv_get_vlist(tv1);
    if l1.is_null() {
        tv_set_vlist(tv1, l2);
        nvim_list_ref(l2);
    } else {
        tv_list_extend(l1, l2, std::ptr::null_mut());
    }

    OK
}

/// Handle number operations: nr += nr, nr -= nr, nr *= nr, nr /= nr, nr %= nr.
/// Returns OK or FAIL.
unsafe fn tv_op_number(tv1: TypevalHandle, tv2: TypevalHandle, op: *const c_char) -> c_int {
    // Load n from tv1 BEFORE clearing (tv_clear may free memory)
    let n = tv_get_number(tv1);

    if tv_get_type(tv2) == VAR_FLOAT {
        if *op == b'%' as c_char {
            return FAIL;
        }
        let mut f = n as f64;
        let f2 = tv_get_vfloat(tv2);
        match *op as u8 {
            b'+' => f += f2,
            b'-' => f -= f2,
            b'*' => f *= f2,
            b'/' => f /= f2,
            _ => {}
        }
        tv_clear(tv1);
        tv_set_float(tv1, f);
    } else {
        let n2 = tv_get_number(tv2);
        let result = match *op as u8 {
            b'+' => n.wrapping_add(n2),
            b'-' => n.wrapping_sub(n2),
            b'*' => n.wrapping_mul(n2),
            b'/' => rs_num_divide(n, n2),
            b'%' => rs_num_modulus(n, n2),
            _ => n,
        };
        tv_clear(tv1);
        tv_set_number(tv1, result);
    }

    OK
}

/// Handle "str1 .= str2"
/// Returns OK or FAIL.
unsafe fn tv_op_string(tv1: TypevalHandle, tv2: TypevalHandle, _op: *const c_char) -> c_int {
    if tv_get_type(tv2) == VAR_FLOAT {
        return FAIL;
    }

    // str .= str
    // Load tvs BEFORE clearing tv1
    let tvs = tv_get_string(tv1);
    let mut numbuf = [0i8; NUMBUFLEN];
    let s2 = tv_get_string_buf(tv2, numbuf.as_mut_ptr());
    let s = concat_str(tvs, s2);
    tv_clear(tv1);
    tv_set_string_owned(tv1, s);

    OK
}

/// Handle "tv1 += tv2", "tv1 -= tv2", "tv1 *= tv2", "tv1 /= tv2", "tv1 %= tv2"
/// and "tv1 .= tv2" for number or string types.
/// Returns OK or FAIL.
unsafe fn tv_op_nr_or_string(tv1: TypevalHandle, tv2: TypevalHandle, op: *const c_char) -> c_int {
    if tv_get_type(tv2) == VAR_LIST {
        return FAIL;
    }

    // "+-*/%" → number op; "." → string op
    let op_chars = b"+-*/%\0";
    if !vim_strchr(op_chars.as_ptr().cast::<c_char>(), c_int::from(*op as u8)).is_null() {
        return tv_op_number(tv1, tv2, op);
    }

    tv_op_string(tv1, tv2, op)
}

/// Handle "f1 += f2", "f1 -= f2", "f1 *= f2", "f1 /= f2".
/// Returns OK or FAIL.
unsafe fn tv_op_float(tv1: TypevalHandle, tv2: TypevalHandle, op: *const c_char) -> c_int {
    let t2 = tv_get_type(tv2);
    if *op == b'%' as c_char
        || *op == b'.' as c_char
        || (t2 != VAR_FLOAT && t2 != VAR_NUMBER && t2 != VAR_STRING)
    {
        return FAIL;
    }

    let f = if t2 == VAR_FLOAT {
        tv_get_vfloat(tv2)
    } else {
        tv_get_number(tv2) as f64
    };

    let t = &mut *tv1.as_ptr().cast::<TypvalTRepr>();
    match *op as u8 {
        b'+' => t.vval.v_float += f,
        b'-' => t.vval.v_float -= f,
        b'*' => t.vval.v_float *= f,
        b'/' => t.vval.v_float /= f,
        _ => {}
    }

    OK
}

// =============================================================================
// Public FFI export
// =============================================================================

/// Handle tv1 += tv2, -=, *=, /=, %=, .=
///
/// # Safety
/// `tv1`, `tv2`, and `op` must be valid non-null pointers.
#[export_name = "eexe_mod_op"]
pub unsafe extern "C" fn rs_eexe_mod_op(
    tv1: TypevalHandle,
    tv2: TypevalHandle,
    op: *const c_char,
) -> c_int {
    let t2 = tv_get_type(tv2);

    // Can't do anything with a Funcref or Dict on the right.
    // v:true and friends only work with "..=".
    if t2 == VAR_FUNC
        || t2 == VAR_DICT
        || ((t2 == VAR_BOOL || t2 == VAR_SPECIAL) && *op == b'.' as c_char)
    {
        semsg(&e_letwrong as *const c_char, op);
        return FAIL;
    }

    let retval = match tv_get_type(tv1) {
        VAR_DICT | VAR_FUNC | VAR_PARTIAL | VAR_BOOL | VAR_SPECIAL => FAIL,
        VAR_BLOB => tv_op_blob(tv1, tv2, op),
        VAR_LIST => tv_op_list(tv1, tv2, op),
        VAR_NUMBER | VAR_STRING => tv_op_nr_or_string(tv1, tv2, op),
        VAR_FLOAT => tv_op_float(tv1, tv2, op),
        _ => {
            // VAR_UNKNOWN → abort in C; panic in Rust dev, FAIL in release
            #[cfg(debug_assertions)]
            panic!("eexe_mod_op: VAR_UNKNOWN tv1");
            #[cfg(not(debug_assertions))]
            FAIL
        }
    };

    if retval != OK {
        semsg(&e_letwrong as *const c_char, op);
    }

    retval
}
