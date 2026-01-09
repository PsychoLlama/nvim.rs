//! Type inspection functions for VimL.
//!
//! This module implements type functions from `src/nvim/eval/funcs.c`:
//! - `type()` - returns the type of a value
//! - `typename()` - returns the type name as a string
//! - `isnumber()` - check if value is a number
//! - `islist()` - check if value is a list
//! - `isdict()` - check if value is a dictionary
//! - `isfloat()` - check if value is a float
//! - `isstring()` - check if value is a string

use std::ffi::c_void;

use super::dispatch::{
    argvar_at, rettv_set_bool, rettv_set_number, tv_get_type, TypevalPtrMut, VarType,
};

// =============================================================================
// VAR_TYPE constants (from typval_defs.h)
// =============================================================================

/// VAR_TYPE_NUMBER = 0
const VAR_TYPE_NUMBER: i64 = 0;
/// VAR_TYPE_STRING = 1
const VAR_TYPE_STRING: i64 = 1;
/// VAR_TYPE_FUNC = 2
const VAR_TYPE_FUNC: i64 = 2;
/// VAR_TYPE_LIST = 3
const VAR_TYPE_LIST: i64 = 3;
/// VAR_TYPE_DICT = 4
const VAR_TYPE_DICT: i64 = 4;
/// VAR_TYPE_FLOAT = 5
const VAR_TYPE_FLOAT: i64 = 5;
/// VAR_TYPE_BOOL = 6
const VAR_TYPE_BOOL: i64 = 6;
/// VAR_TYPE_SPECIAL = 7
const VAR_TYPE_SPECIAL: i64 = 7;
/// VAR_TYPE_BLOB = 10
const VAR_TYPE_BLOB: i64 = 10;

// =============================================================================
// type() function
// =============================================================================

/// "type()" function - returns the type of a value
///
/// Returns:
/// - 0 for Number
/// - 1 for String
/// - 2 for Funcref
/// - 3 for List
/// - 4 for Dictionary
/// - 5 for Float
/// - 6 for Boolean
/// - 7 for Special (null)
/// - 10 for Blob
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_type(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);

    let vtype = tv_get_type(arg0);
    let n = match vtype {
        VarType::Number => VAR_TYPE_NUMBER,
        VarType::String => VAR_TYPE_STRING,
        VarType::Func | VarType::Partial => VAR_TYPE_FUNC,
        VarType::List => VAR_TYPE_LIST,
        VarType::Dict => VAR_TYPE_DICT,
        VarType::Float => VAR_TYPE_FLOAT,
        VarType::Bool => VAR_TYPE_BOOL,
        VarType::Special => VAR_TYPE_SPECIAL,
        VarType::Blob => VAR_TYPE_BLOB,
        VarType::Unknown => -1, // This should not happen in normal code
    };
    rettv_set_number(rettv, n);
}

// =============================================================================
// Type check functions (isXXX)
// =============================================================================

/// "isnumber()" function - check if value is a number.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isnumber(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Number));
}

/// "isfloat()" function - check if value is a float.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isfloat(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Float));
}

/// "isstring()" function - check if value is a string.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isstring(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::String));
}

/// "islist()" function - check if value is a list.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_islist(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::List));
}

/// "isdict()" function - check if value is a dictionary.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isdict(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Dict));
}

/// "isfunc()" function - check if value is a funcref.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isfunc(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Func | VarType::Partial));
}

/// "isblob()" function - check if value is a blob.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isblob(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Blob));
}

/// "isbool()" function - check if value is a boolean.
///
/// # Safety
/// Caller must provide valid pointers to typval_T arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_f_isbool(argvars: *const c_void, rettv: *mut c_void) {
    let rettv = TypevalPtrMut::from_raw(rettv);
    let arg0 = argvar_at(argvars, 0);
    let vtype = tv_get_type(arg0);
    rettv_set_bool(rettv, matches!(vtype, VarType::Bool));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_type_constants() {
        // Verify constants match C definitions
        assert_eq!(VAR_TYPE_NUMBER, 0);
        assert_eq!(VAR_TYPE_STRING, 1);
        assert_eq!(VAR_TYPE_FUNC, 2);
        assert_eq!(VAR_TYPE_LIST, 3);
        assert_eq!(VAR_TYPE_DICT, 4);
        assert_eq!(VAR_TYPE_FLOAT, 5);
        assert_eq!(VAR_TYPE_BOOL, 6);
        assert_eq!(VAR_TYPE_SPECIAL, 7);
        assert_eq!(VAR_TYPE_BLOB, 10);
    }
}
