//! v: variable accessor functions (get_vim_var_*, set_vim_var_*).
//!
//! Phase 4 & 5: Migrated from `src/nvim/eval/vars.c`.
//!
//! These functions access the `vimvars[]` static C array via the
//! `get_vim_var_tv()` C accessor function, then manipulate typval_T fields
//! through existing C accessor functions from typval.c and new ones in vars.c.

#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Type aliases for C types
// =============================================================================

/// Opaque handle to a typval_T.
type TvPtr = *mut c_void;

/// Opaque handle to a list_T.
type ListPtr = *mut c_void;

/// Opaque handle to a dict_T.
type DictPtr = *mut c_void;

/// Opaque handle to a partial_T.
type PartialPtr = *mut c_void;

/// varnumber_T (int64_t).
type Varnumber = i64;

/// VimVarIndex maps to c_int.
type VimVarIndex = c_int;

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // ── v: variable core accessor ──────────────────────────────────────────
    /// get_vim_var_tv: returns &vimvars[idx].vv_tv
    fn get_vim_var_tv(idx: VimVarIndex) -> TvPtr;

    // ── typval_T accessors from typval.c ─────────────────────────────────
    fn nvim_tv_set_type(tv: TvPtr, vtype: c_int);
    fn nvim_tv_get_number(tv: TvPtr) -> Varnumber;
    fn nvim_tv_set_number(tv: TvPtr, n: Varnumber);
    fn nvim_tv_set_bool(tv: TvPtr, val: c_int);
    fn nvim_tv_get_list(tv: TvPtr) -> ListPtr;
    fn nvim_tv_set_list(tv: TvPtr, l: ListPtr);
    fn nvim_tv_get_dict(tv: TvPtr) -> DictPtr;
    fn nvim_tv_set_dict(tv: TvPtr, d: DictPtr);
    // nvim_tv_set_string_copy handles NULL, copy (len==-1 uses strlen, >0 copies len bytes)
    fn nvim_tv_set_string_copy(tv: TvPtr, s: *const c_char, len: c_int);

    // ── typval_T accessors new in vars.c ─────────────────────────────────
    fn nvim_tv_set_special(tv: TvPtr, val: c_int);
    fn nvim_tv_get_partial(tv: TvPtr) -> PartialPtr;
    fn nvim_tv_set_partial(tv: TvPtr, p: PartialPtr);

    // ── vimvar meta-accessors (new in vars.c) ─────────────────────────────
    fn nvim_vimvar_get_name(idx: VimVarIndex) -> *mut c_char;

    // ── typval operations ─────────────────────────────────────────────────
    fn tv_clear(tv: TvPtr);
    fn tv_copy(src: TvPtr, dst: TvPtr);
    fn tv_get_string(tv: TvPtr) -> *const c_char;

    // ── list/dict helpers ──────────────────────────────────────────────────
    fn nvim_tv_list_ref(l: ListPtr);
    fn nvim_dict_incr_refcount(d: DictPtr);
    fn nvim_dict_set_keys_readonly(d: DictPtr);

    // ── misc helpers ───────────────────────────────────────────────────────
    #[link_name = "utf_char2bytes"]
    fn nvim_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;

    // ── typval_T string/type accessors (vars.c) ───────────────────────────
    fn nvim_tv_get_type(tv: TvPtr) -> c_int;
    fn nvim_tv_get_string_val(tv: TvPtr) -> *mut c_char;
    fn nvim_tv_set_string_val(tv: TvPtr, s: *mut c_char);

    // ── list helpers ──────────────────────────────────────────────────────
    fn tv_list_alloc(len: i64) -> ListPtr;
    fn tv_list_append_string(l: ListPtr, s: *const c_char, len: i64);
}

// VarType values (matching C enum)
const VAR_LIST: c_int = 4;
const VAR_BOOL: c_int = 7;
const VAR_SPECIAL: c_int = 8;

// VV_* index constants (matching C enum VimVarIndex in eval_defs.h)
const VV_COUNT: VimVarIndex = 0;
const VV_COUNT1: VimVarIndex = 1;
const VV_PREVCOUNT: VimVarIndex = 2;
const VV_EXCEPTION: VimVarIndex = 30;
const VV_THROWPOINT: VimVarIndex = 31;
const VV_REG: VimVarIndex = 32;
const VV_OPTION_NEW: VimVarIndex = 62;
const VV_OPTION_OLD: VimVarIndex = 63;
const VV_OPTION_OLDLOCAL: VimVarIndex = 64;
const VV_OPTION_OLDGLOBAL: VimVarIndex = 65;
const VV_OPTION_COMMAND: VimVarIndex = 66;
const VV_OPTION_TYPE: VimVarIndex = 67;
const VV_ERRORS: VimVarIndex = 68;
const VV_CHAR: VimVarIndex = 50;

// =============================================================================
// get_vim_var_* functions
// =============================================================================

/// Get typval_T pointer for v: variable `idx`.
///
/// Matches C `get_vim_var_tv` (thin wrapper -- the C function accesses
/// vimvars[] directly; we re-export it for Rust callers that need the ptr).
#[no_mangle]
pub unsafe extern "C" fn rs_get_vim_var_tv(idx: VimVarIndex) -> TvPtr {
    get_vim_var_tv(idx)
}

/// Get the name of v: variable `idx`.
///
/// Matches C `get_vim_var_name`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_vim_var_name(idx: VimVarIndex) -> *mut c_char {
    nvim_vimvar_get_name(idx)
}

/// Get number value of v: variable `idx`.
///
/// Matches C `get_vim_var_nr`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_vim_var_nr(idx: VimVarIndex) -> Varnumber {
    let tv = get_vim_var_tv(idx);
    nvim_tv_get_number(tv)
}

/// Get list value of v: variable `idx`.
///
/// Matches C `get_vim_var_list`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_vim_var_list(idx: VimVarIndex) -> ListPtr {
    let tv = get_vim_var_tv(idx);
    nvim_tv_get_list(tv)
}

/// Get dict value of v: variable `idx`.
///
/// Matches C `get_vim_var_dict`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_vim_var_dict(idx: VimVarIndex) -> DictPtr {
    let tv = get_vim_var_tv(idx);
    nvim_tv_get_dict(tv)
}

/// Get string value of v: variable `idx`.
///
/// Returns a C string (never NULL, may be empty). Matches C `get_vim_var_str`.
///
/// # Safety
/// The returned pointer is only valid as long as the v: variable is unchanged.
#[no_mangle]
pub unsafe extern "C" fn rs_get_vim_var_str(idx: VimVarIndex) -> *const c_char {
    let tv = get_vim_var_tv(idx);
    tv_get_string(tv)
}

/// Get partial value of v: variable `idx`.
///
/// Matches C `get_vim_var_partial`.
#[no_mangle]
pub unsafe extern "C" fn rs_get_vim_var_partial(idx: VimVarIndex) -> PartialPtr {
    let tv = get_vim_var_tv(idx);
    nvim_tv_get_partial(tv)
}

// =============================================================================
// set_vim_var_* functions
// =============================================================================

/// Set v: variable to a copy of typval.
///
/// Matches C `set_vim_var_tv`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vim_var_tv(idx: VimVarIndex, tv_in: TvPtr) {
    let tv_out = get_vim_var_tv(idx);
    tv_clear(tv_out);
    tv_copy(tv_in, tv_out);
}

/// Set the type of v: variable `idx`.
///
/// Matches C `set_vim_var_type`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vim_var_type(idx: VimVarIndex, vtype: c_int) {
    let tv = get_vim_var_tv(idx);
    nvim_tv_set_type(tv, vtype);
}

/// Set number v: variable.
///
/// Matches C `set_vim_var_nr`. Clears first, then sets type+value.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vim_var_nr(idx: VimVarIndex, val: Varnumber) {
    let tv = get_vim_var_tv(idx);
    tv_clear(tv);
    // nvim_tv_set_number also sets v_type = VAR_NUMBER
    nvim_tv_set_number(tv, val);
}

/// Set boolean v: variable.
///
/// Matches C `set_vim_var_bool`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vim_var_bool(idx: VimVarIndex, val: c_int) {
    let tv = get_vim_var_tv(idx);
    tv_clear(tv);
    nvim_tv_set_type(tv, VAR_BOOL);
    nvim_tv_set_bool(tv, val);
}

/// Set special v: variable.
///
/// Matches C `set_vim_var_special`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vim_var_special(idx: VimVarIndex, val: c_int) {
    let tv = get_vim_var_tv(idx);
    tv_clear(tv);
    nvim_tv_set_type(tv, VAR_SPECIAL);
    nvim_tv_set_special(tv, val);
}

/// Set v:char to character `c`.
///
/// Matches C `set_vim_var_char`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vim_var_char(c: c_int) {
    // MB_MAXCHAR + 1 = 6 bytes max for UTF-8 + NUL
    let mut buf = [0u8; 8];
    let len = nvim_utf_char2bytes(c, buf.as_mut_ptr().cast());
    buf[len as usize] = 0;
    rs_set_vim_var_string(VV_CHAR, buf.as_ptr().cast(), -1);
}

/// Set string v: variable. `val` is copied; `len == -1` means use strlen.
///
/// Matches C `set_vim_var_string`.
///
/// # Safety
/// `val` must be NULL or a valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vim_var_string(idx: VimVarIndex, val: *const c_char, len: i64) {
    let tv = get_vim_var_tv(idx);
    tv_clear(tv);
    // nvim_tv_set_string_copy handles NULL val, len==-1 (strlen), and positive len.
    // It also sets tv->v_type = VAR_STRING.
    nvim_tv_set_string_copy(tv, val, len as c_int);
}

/// Set list v: variable. Increments refcount.
///
/// Matches C `set_vim_var_list`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vim_var_list(idx: VimVarIndex, val: ListPtr) {
    let tv = get_vim_var_tv(idx);
    tv_clear(tv);
    // nvim_tv_set_list also sets v_type = VAR_LIST
    nvim_tv_set_list(tv, val);
    nvim_tv_list_ref(val);
}

/// Set dict v: variable. Increments refcount and makes keys read-only.
///
/// Matches C `set_vim_var_dict`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vim_var_dict(idx: VimVarIndex, val: DictPtr) {
    let tv = get_vim_var_tv(idx);
    tv_clear(tv);
    // nvim_tv_set_dict also sets v_type = VAR_DICT
    nvim_tv_set_dict(tv, val);
    if !val.is_null() {
        nvim_dict_incr_refcount(val);
        nvim_dict_set_keys_readonly(val);
    }
}

/// Set partial v: variable (type not set by this function).
///
/// Matches C `set_vim_var_partial`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vim_var_partial(idx: VimVarIndex, val: PartialPtr) {
    let tv = get_vim_var_tv(idx);
    nvim_tv_set_partial(tv, val);
}

// =============================================================================
// Phase 5: Helper and Utility Functions
// =============================================================================

/// Set v:register to register name character `c`.
///
/// Matches C `set_reg_var`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_reg_var(c: c_int) {
    let regname: c_char = if c == 0 || c == c_int::from(b' ') {
        b'"' as c_char
    } else {
        c as c_char
    };
    // Only update if necessary (avoid free/alloc when already correct).
    let tv = get_vim_var_tv(VV_REG);
    let s = nvim_tv_get_string_val(tv);
    if s.is_null() || *s != regname {
        rs_set_vim_var_string(VV_REG, &raw const regname, 1);
    }
}

/// Get or set v:exception.
///
/// If `oldval` is NULL, return the current value.
/// Otherwise, restore the value to `oldval` and return NULL.
///
/// Matches C `v_exception`.
#[no_mangle]
pub unsafe extern "C" fn rs_v_exception(oldval: *mut c_char) -> *mut c_char {
    let tv = get_vim_var_tv(VV_EXCEPTION);
    if oldval.is_null() {
        nvim_tv_get_string_val(tv)
    } else {
        nvim_tv_set_string_val(tv, oldval);
        std::ptr::null_mut()
    }
}

/// Get or set v:throwpoint.
///
/// If `oldval` is NULL, return the current value.
/// Otherwise, restore the value to `oldval` and return NULL.
///
/// Matches C `v_throwpoint`.
#[no_mangle]
pub unsafe extern "C" fn rs_v_throwpoint(oldval: *mut c_char) -> *mut c_char {
    let tv = get_vim_var_tv(VV_THROWPOINT);
    if oldval.is_null() {
        nvim_tv_get_string_val(tv)
    } else {
        nvim_tv_set_string_val(tv, oldval);
        std::ptr::null_mut()
    }
}

/// Set v:count and v:count1. If `set_prevcount` is true, first copy v:count to v:prevcount.
///
/// Matches C `set_vcount`.
#[no_mangle]
pub unsafe extern "C" fn rs_set_vcount(count: i64, count1: i64, set_prevcount: bool) {
    if set_prevcount {
        let prevcount_tv = get_vim_var_tv(VV_PREVCOUNT);
        let cur_count = nvim_tv_get_number(get_vim_var_tv(VV_COUNT));
        nvim_tv_set_number(prevcount_tv, cur_count);
    }
    nvim_tv_set_number(get_vim_var_tv(VV_COUNT), count);
    nvim_tv_set_number(get_vim_var_tv(VV_COUNT1), count1);
}

/// Reset all v:option_* variables to NULL.
///
/// Matches C `reset_v_option_vars`.
#[no_mangle]
pub unsafe extern "C" fn rs_reset_v_option_vars() {
    rs_set_vim_var_string(VV_OPTION_NEW, std::ptr::null(), -1);
    rs_set_vim_var_string(VV_OPTION_OLD, std::ptr::null(), -1);
    rs_set_vim_var_string(VV_OPTION_OLDLOCAL, std::ptr::null(), -1);
    rs_set_vim_var_string(VV_OPTION_OLDGLOBAL, std::ptr::null(), -1);
    rs_set_vim_var_string(VV_OPTION_COMMAND, std::ptr::null(), -1);
    rs_set_vim_var_string(VV_OPTION_TYPE, std::ptr::null(), -1);
}

/// Add an assert error (from a garray) to v:errors.
///
/// Matches C `assert_error`.
///
/// # Safety
/// `gap` must be a valid pointer to a garray_T with ga_data pointing to a
/// valid byte buffer of at least ga_len bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_assert_error(ga_data: *const c_char, ga_len: c_int) {
    let tv = get_vim_var_tv(VV_ERRORS);
    if nvim_tv_get_type(tv) != VAR_LIST || nvim_tv_get_list(tv).is_null() {
        // Make sure v:errors is a list.
        rs_set_vim_var_list(VV_ERRORS, tv_list_alloc(1));
    }
    tv_list_append_string(rs_get_vim_var_list(VV_ERRORS), ga_data, i64::from(ga_len));
}
