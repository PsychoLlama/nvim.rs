//! v: variable accessor functions (get_vim_var_*, set_vim_var_*).
//!
//! Phase 4: Migrated from `src/nvim/eval/vars.c`.
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
    fn nvim_utf_char2bytes(c: c_int, buf: *mut c_char) -> c_int;
}

// VarType values (matching C enum)
const VAR_BOOL: c_int = 7;
const VAR_SPECIAL: c_int = 8;

// VV_CHAR index (matches VimVarIndex::Char = 50)
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
