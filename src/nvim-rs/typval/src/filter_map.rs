//! filter(), map(), mapnew(), foreach() — migrated from eval/list.c (Phase 1).
//!
//! Direct port of the C implementation.  All logic is verbatim; the only
//! changes are mechanical FFI wrapping and handle-based field access.

#![allow(clippy::too_many_arguments)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(dead_code)]
#![allow(clashing_extern_declarations)]

use std::ffi::{c_char, c_int, c_void};

use crate::{
    BlobHandle, DictHandle, DictItemHandle, GArrayT, HashItemHandle, ListHandle, ListItemHandle,
    TypevalHandle, VarType,
};

// =============================================================================
// C constants
// =============================================================================

const OK: c_int = 1;
const FAIL: c_int = 0;

/// VAR_NUMBER type constant
const VAR_NUMBER: c_int = VarType::Number as c_int;
/// VAR_BOOL type constant
const VAR_BOOL: c_int = VarType::Bool as c_int;
/// VAR_STRING type constant
const VAR_STRING: c_int = VarType::String as c_int;
/// VAR_UNKNOWN type constant
const VAR_UNKNOWN: c_int = VarType::Unknown as c_int;
/// VAR_UNLOCKED = 0 (matching C's VarLockStatus::Unlocked)
const VAR_UNLOCKED: c_int = 0;
/// VAR_LOCKED = 1 (matching C's VarLockStatus::Locked)
const VAR_LOCKED: c_int = 1;

/// TV_TRANSLATE flag (passed as name_len to value_check_lock / var_check_ro etc.)
const TV_TRANSLATE: usize = usize::MAX;

/// VV_VAL = 35 (from eval_defs.h enum, counting from VV_COUNT=0)
const VV_VAL: c_int = 35;
/// VV_KEY = 36
const VV_KEY: c_int = 36;

/// kListLenUnknown = -3
const K_LIST_LEN_UNKNOWN: isize = -3;

// =============================================================================
// FilterMap mode
// =============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum FilterMap {
    Filter,
    Map,
    MapNew,
    Foreach,
}

impl FilterMap {
    fn func_name(self) -> *const c_char {
        match self {
            Self::Filter => b"filter()\0".as_ptr().cast::<c_char>(),
            Self::Map => b"map()\0".as_ptr().cast::<c_char>(),
            Self::MapNew => b"mapnew()\0".as_ptr().cast::<c_char>(),
            Self::Foreach => b"foreach()\0".as_ptr().cast::<c_char>(),
        }
    }

    fn arg_errmsg(self) -> *const c_char {
        match self {
            Self::Filter => b"filter() argument\0".as_ptr().cast::<c_char>(),
            Self::Map => b"map() argument\0".as_ptr().cast::<c_char>(),
            Self::MapNew => b"mapnew() argument\0".as_ptr().cast::<c_char>(),
            Self::Foreach => b"foreach() argument\0".as_ptr().cast::<c_char>(),
        }
    }
}

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // typval array indexing (sizeof-free)
    fn nvim_typval_array_get(args: TypevalHandle, idx: c_int) -> TypevalHandle;

    // eval_expr_typval: evaluates expr with argv, stores result in rettv
    fn eval_expr_typval(
        expr: TypevalHandle,
        want_func: bool,
        argv: TypevalHandle,
        argc: c_int,
        rettv: TypevalHandle,
    ) -> c_int;

    // do_cmdline_cmd: run an ex command string (for foreach with string expr)
    fn do_cmdline_cmd(cmd: *const c_char) -> c_int;

    // tv_copy / tv_clear
    fn tv_copy(from: *const c_void, to: *mut c_void);
    fn tv_clear(tv: TypevalHandle);

    // v:key / v:val management
    fn prepare_vimvar(idx: c_int, save: TypevalHandle);
    fn restore_vimvar(idx: c_int, save: TypevalHandle);
    fn get_vim_var_tv(idx: c_int) -> TypevalHandle;
    fn set_vim_var_nr(idx: c_int, val: i64);
    fn set_vim_var_string(idx: c_int, val: *const c_char, len: c_int);
    fn set_vim_var_type(idx: c_int, vtype: c_int);

    // tv_get_*
    fn tv_get_number_chk(tv: TypevalHandle, error: *mut bool) -> i64;
    fn tv_get_string(tv: TypevalHandle) -> *const c_char;

    // value_check_lock / var_check_ro / var_check_fixed
    fn value_check_lock(lock: c_int, name: *const c_char, name_len: usize) -> bool;
    fn var_check_ro(flags: c_int, name: *const c_char, name_len: usize) -> bool;
    fn var_check_fixed(flags: c_int, name: *const c_char, name_len: usize) -> bool;

    // List operations (exported Rust functions via export_name)
    fn tv_list_alloc_ret(ret_tv: TypevalHandle, len: isize) -> ListHandle;
    fn tv_list_append_owned_tv(l: ListHandle, tv: *const c_void);
    fn tv_list_item_remove(l: ListHandle, item: ListItemHandle) -> ListItemHandle;

    // List accessors (real C accessor functions in typval.c)
    fn nvim_list_set_lock(l: ListHandle, lock: c_int);
    fn nvim_list_get_lock(l: ListHandle) -> c_int;
    fn nvim_list_get_first(l: ListHandle) -> ListItemHandle;

    // List item accessors
    fn nvim_listitem_get_tv(li: ListItemHandle) -> TypevalHandle;
    fn nvim_listitem_get_next(li: ListItemHandle) -> ListItemHandle;

    // List item v_lock accessor (for MAP check)
    fn nvim_listitem_get_v_lock(li: ListItemHandle) -> c_int;

    // Dict operations
    fn tv_dict_alloc_ret(ret_tv: TypevalHandle);
    fn tv_dict_item_remove(d: DictHandle, di: DictItemHandle);
    fn tv_dict_add_tv(d: DictHandle, key: *const c_char, keylen: usize, tv: TypevalHandle)
        -> c_int;
    fn nvim_tv_get_dict(tv: TypevalHandle) -> DictHandle;

    // Dict lock accessors
    fn nvim_dict_get_lock(d: DictHandle) -> c_int;
    fn nvim_dict_set_lock(d: DictHandle, lock: c_int);

    // Dict item accessors for iteration
    fn nvim_dict_get_ht_used(d: DictHandle) -> usize;
    fn nvim_dict_get_ht_array(d: DictHandle) -> HashItemHandle;
    fn nvim_hashitem_get_key(hi: HashItemHandle) -> *const c_char;
    fn nvim_hash_removed_ptr() -> *const c_char;
    fn nvim_hashitem_next(hi: HashItemHandle) -> HashItemHandle;
    fn nvim_hashitem_to_dictitem(hi: HashItemHandle) -> DictItemHandle;
    fn nvim_dictitem_get_tv(di: DictItemHandle) -> TypevalHandle;
    fn nvim_dictitem_get_key(di: DictItemHandle) -> *const c_char;
    fn nvim_dictitem_get_flags(di: DictItemHandle) -> c_int;
    fn nvim_dict_hash_lock(d: DictHandle);
    fn nvim_dict_hash_unlock(d: DictHandle);

    // Blob operations
    fn tv_blob_copy(b: TypevalHandle, rettv: TypevalHandle);
    fn nvim_tv_get_blob(tv: TypevalHandle) -> BlobHandle;
    fn nvim_blob_get_len(b: BlobHandle) -> c_int;
    fn nvim_blob_get_lock(b: BlobHandle) -> c_int;
    fn nvim_blob_set_lock(b: BlobHandle, lock: c_int);
    fn nvim_blob_get_byte(b: BlobHandle, idx: c_int) -> u8;
    fn nvim_blob_set_byte(b: BlobHandle, idx: c_int, c: u8);
    fn nvim_blob_get_ga_data(b: BlobHandle) -> *mut u8;
    fn nvim_blob_set_ga_len(b: BlobHandle, len: c_int);

    // String/garray operations
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_char;
    fn ga_init(gap: *mut GArrayT, itemsize: c_int, growsize: c_int);
    fn ga_concat(gap: *mut GArrayT, s: *const c_char);
    fn ga_append(gap: *mut GArrayT, c: u8);
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // Nvim typval field access
    fn nvim_tv_set_type(tv: TypevalHandle, vtype: c_int);
    fn nvim_tv_set_string(tv: TypevalHandle, s: *mut c_char);
    fn nvim_tv_set_list(tv: TypevalHandle, l: ListHandle);
    fn nvim_tv_set_dict(tv: TypevalHandle, d: DictHandle);
    fn nvim_tv_set_blob(tv: TypevalHandle, b: BlobHandle);
    fn nvim_tv_get_list(tv: TypevalHandle) -> ListHandle;
    fn nvim_tv_get_string_ptr(tv: TypevalHandle) -> *const c_char;
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;
    fn nvim_tv_get_number(tv: TypevalHandle) -> i64;
    fn nvim_tv_set_lock(tv: TypevalHandle, lock: c_int);

    // Error messages
    fn emsg(s: *const c_char);
    fn semsg(fmt: *const c_char, ...) -> c_int;

    // Error string globals
    static e_invalblob: [c_char; 0];
    static e_string_required: [c_char; 0];

    // did_emsg global
    static mut did_emsg: c_int;

    // gettext
    fn gettext(s: *const c_char) -> *const c_char;
}

/// E1250 error string (local to this file, same as C's static const char[]).
static E_ARGUMENT_OF_STR_MUST_BE_LIST_STRING_DICTIONARY_OR_BLOB: &[u8] =
    b"E1250: Argument of %s must be a List, String, Dictionary or Blob\0";

// =============================================================================
// Helper: get typval type
// =============================================================================

#[inline]
unsafe fn tv_type(tv: TypevalHandle) -> c_int {
    if tv.is_null() {
        VAR_UNKNOWN
    } else {
        unsafe { nvim_tv_get_type(tv) }
    }
}

// =============================================================================
// filter_map_one
// =============================================================================

/// Handle one item for map(), filter(), foreach().
///
/// Sets v:val to `tv`. Caller must set v:key.
///
/// Returns OK/FAIL.  On success, `newtv` holds the callback result (caller
/// must clear it when done).  For FILTER mode, `*remp` is set.
unsafe fn filter_map_one(
    tv: TypevalHandle,
    expr: TypevalHandle,
    filtermap: FilterMap,
    newtv: TypevalHandle,
    remp: *mut bool,
) -> c_int {
    // argv[0] = v:key copy, argv[1] = v:val copy (16 bytes each = typval_T)
    let mut argv = [0u8; 32];

    // v:val = *tv
    let vv_val = unsafe { get_vim_var_tv(VV_VAL) };
    unsafe { tv_copy(tv.as_ptr(), vv_val.as_ptr().cast_mut()) };

    // newtv->v_type = VAR_UNKNOWN
    unsafe { nvim_tv_set_type(newtv, VAR_UNKNOWN) };

    if filtermap == FilterMap::Foreach && unsafe { tv_type(expr) } == VAR_STRING {
        // foreach() with a string expression runs the string as a command
        let cmd = unsafe { nvim_tv_get_string_ptr(expr) };
        unsafe { do_cmdline_cmd(cmd) };
        if unsafe { did_emsg } == 0 {
            unsafe { tv_clear(get_vim_var_tv(VV_VAL)) };
            return OK;
        }
        unsafe { tv_clear(get_vim_var_tv(VV_VAL)) };
        return FAIL;
    }

    // argv[0] = v:key, argv[1] = v:val
    unsafe {
        let vv_key = get_vim_var_tv(VV_KEY);
        let vv_val = get_vim_var_tv(VV_VAL);
        // copy 16 bytes of v:key into argv[0..16]
        std::ptr::copy_nonoverlapping(vv_key.as_ptr().cast::<u8>(), argv.as_mut_ptr(), 16);
        // copy 16 bytes of v:val into argv[16..32]
        std::ptr::copy_nonoverlapping(vv_val.as_ptr().cast::<u8>(), argv.as_mut_ptr().add(16), 16);
    }

    let argv0 = unsafe { TypevalHandle::from_ptr(argv.as_mut_ptr().cast::<c_void>()) };
    if unsafe { eval_expr_typval(expr, false, argv0, 2, newtv) } == FAIL {
        unsafe { tv_clear(get_vim_var_tv(VV_VAL)) };
        return FAIL;
    }

    if filtermap == FilterMap::Filter {
        let mut error = false;
        let keep = unsafe { tv_get_number_chk(newtv, &raw mut error) } == 0;
        unsafe { tv_clear(newtv) };
        if error {
            unsafe { tv_clear(get_vim_var_tv(VV_VAL)) };
            return FAIL;
        }
        unsafe { *remp = keep };
    } else if filtermap == FilterMap::Foreach {
        unsafe { tv_clear(newtv) };
    }

    unsafe { tv_clear(get_vim_var_tv(VV_VAL)) };
    OK
}

// =============================================================================
// filter_map_dict
// =============================================================================

/// filter()/map()/foreach() implementation for a Dict.
unsafe fn filter_map_dict(
    d: DictHandle,
    filtermap: FilterMap,
    arg_errmsg: *const c_char,
    expr: TypevalHandle,
    rettv: TypevalHandle,
) {
    if filtermap == FilterMap::MapNew {
        unsafe { nvim_tv_set_type(rettv, VarType::Dict as c_int) };
        unsafe { nvim_tv_set_dict(rettv, DictHandle::null()) };
    }
    if d.is_null()
        || (filtermap == FilterMap::Filter
            && unsafe { value_check_lock(nvim_dict_get_lock(d), arg_errmsg, TV_TRANSLATE) })
    {
        return;
    }

    let d_ret = if filtermap == FilterMap::MapNew {
        unsafe { tv_dict_alloc_ret(rettv) };
        // get the newly allocated dict from rettv
        unsafe { nvim_tv_get_dict(rettv) }
    } else {
        DictHandle::null()
    };

    // Lock dict while iterating (mirrors C's dv_lock manipulation)
    let prev_lock = unsafe { nvim_dict_get_lock(d) };
    if prev_lock == VAR_UNLOCKED {
        unsafe { nvim_dict_set_lock(d, VAR_LOCKED) };
    }
    unsafe { nvim_dict_hash_lock(d) };

    // Iterate hashtab entries
    let ht_used = unsafe { nvim_dict_get_ht_used(d) };
    let hi_removed = unsafe { nvim_hash_removed_ptr() };
    let mut hi = unsafe { nvim_dict_get_ht_array(d) };
    let mut seen = 0usize;
    let mut should_break = false;

    while seen < ht_used {
        let key = unsafe { nvim_hashitem_get_key(hi) };
        if !key.is_null() && key != hi_removed {
            seen += 1;
            let di = unsafe { nvim_hashitem_to_dictitem(hi) };

            if filtermap == FilterMap::Map {
                let di_tv = unsafe { nvim_dictitem_get_tv(di) };
                let item_lock = crate::get_v_lock(di_tv);
                let di_flags = unsafe { nvim_dictitem_get_flags(di) };
                if unsafe {
                    value_check_lock(item_lock, arg_errmsg, TV_TRANSLATE)
                        || var_check_ro(di_flags, arg_errmsg, TV_TRANSLATE)
                } {
                    should_break = true;
                    break;
                }
            }

            unsafe { set_vim_var_string(VV_KEY, key, -1) };

            // Allocate newtv on stack (16 bytes = sizeof(typval_T))
            let mut newtv_bytes = [0u8; 16];
            let newtv =
                unsafe { TypevalHandle::from_ptr(newtv_bytes.as_mut_ptr().cast::<c_void>()) };
            let mut rem = false;
            let di_tv = unsafe { nvim_dictitem_get_tv(di) };
            let r = unsafe { filter_map_one(di_tv, expr, filtermap, newtv, &raw mut rem) };
            unsafe { tv_clear(get_vim_var_tv(VV_KEY)) };

            if r == FAIL || unsafe { did_emsg } != 0 {
                unsafe { tv_clear(newtv) };
                should_break = true;
                break;
            }

            if filtermap == FilterMap::Map {
                // Replace item value in-place
                unsafe { tv_clear(di_tv) };
                // newtv -> di->di_tv: copy 16 bytes
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        newtv_bytes.as_ptr(),
                        di_tv.as_ptr().cast::<u8>().cast_mut(),
                        16,
                    );
                    // set v_lock = VAR_UNLOCKED on the destination
                    nvim_tv_set_lock(di_tv, VAR_UNLOCKED);
                }
            } else if filtermap == FilterMap::MapNew {
                let key_ptr = unsafe { nvim_dictitem_get_key(di) };
                let key_len = unsafe { libc_strlen(key_ptr) };
                let res = unsafe { tv_dict_add_tv(d_ret, key_ptr, key_len, newtv) };
                unsafe { tv_clear(newtv) };
                if res == FAIL {
                    should_break = true;
                    break;
                }
            } else if filtermap == FilterMap::Filter && rem {
                let di_flags = unsafe { nvim_dictitem_get_flags(di) };
                if unsafe {
                    var_check_fixed(di_flags, arg_errmsg, TV_TRANSLATE)
                        || var_check_ro(di_flags, arg_errmsg, TV_TRANSLATE)
                } {
                    should_break = true;
                    break;
                }
                // Advance hi BEFORE removing (the item being removed is at hi)
                hi = unsafe { nvim_hashitem_next(hi) };
                unsafe { tv_dict_item_remove(d, di) };
                // Skip the normal hi advance at bottom of loop
                continue;
            }
        }
        hi = unsafe { nvim_hashitem_next(hi) };
    }

    unsafe { nvim_dict_hash_unlock(d) };
    unsafe { nvim_dict_set_lock(d, prev_lock) };

    let _ = should_break;
}

/// strlen using libc (not std, to avoid depending on a separate crate)
#[inline]
unsafe fn libc_strlen(s: *const c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut len = 0usize;
    while unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    len
}

// =============================================================================
// filter_map_blob
// =============================================================================

/// filter()/map()/foreach() implementation for a Blob.
unsafe fn filter_map_blob(
    blob_arg: BlobHandle,
    filtermap: FilterMap,
    expr: TypevalHandle,
    arg_errmsg: *const c_char,
    rettv: TypevalHandle,
) {
    if filtermap == FilterMap::MapNew {
        unsafe { nvim_tv_set_type(rettv, VarType::Blob as c_int) };
        unsafe { nvim_tv_set_blob(rettv, BlobHandle::null()) };
    }
    let b = blob_arg;
    if b.is_null()
        || (filtermap == FilterMap::Filter
            && unsafe { value_check_lock(nvim_blob_get_lock(b), arg_errmsg, TV_TRANSLATE) })
    {
        return;
    }

    // For mapnew: copy the blob first
    let b_ret = if filtermap == FilterMap::MapNew {
        // tv_blob_copy takes the blob from a typval_T source, not directly
        // We need to make a temporary typval for the source blob
        let mut src_tv = [0u8; 16];
        let src_handle = unsafe { TypevalHandle::from_ptr(src_tv.as_mut_ptr().cast::<c_void>()) };
        unsafe {
            nvim_tv_set_type(src_handle, VarType::Blob as c_int);
            nvim_tv_set_blob(src_handle, b);
        }
        unsafe { tv_blob_copy(src_handle, rettv) };
        unsafe { nvim_tv_get_blob(rettv) }
    } else {
        b
    };

    // set_vim_var_nr() doesn't set the type; do it explicitly
    unsafe { set_vim_var_type(VV_KEY, VAR_NUMBER) };

    let prev_lock = unsafe { nvim_blob_get_lock(b) };
    if prev_lock == 0 {
        unsafe { nvim_blob_set_lock(b, VAR_LOCKED) };
    }

    let mut i = 0i32;
    let mut idx = 0i32;
    let b_len = unsafe { nvim_blob_get_len(b) };

    while i < b_len {
        let val = unsafe { nvim_blob_get_byte(b, i) };

        // Build a VAR_NUMBER typval for the byte value
        let mut tv_bytes = [0u8; 16];
        let tv_handle = unsafe { TypevalHandle::from_ptr(tv_bytes.as_mut_ptr().cast::<c_void>()) };
        unsafe {
            nvim_tv_set_type(tv_handle, VAR_NUMBER);
            // v_number is at offset 8 in typval_T
            *(tv_bytes.as_mut_ptr().add(8).cast::<i64>()) = i64::from(val);
        }

        unsafe { set_vim_var_nr(VV_KEY, i64::from(idx)) };

        let mut newtv_bytes = [0u8; 16];
        let newtv = unsafe { TypevalHandle::from_ptr(newtv_bytes.as_mut_ptr().cast::<c_void>()) };
        let mut rem = false;

        if unsafe { filter_map_one(tv_handle, expr, filtermap, newtv, &raw mut rem) } == FAIL
            || unsafe { did_emsg } != 0
        {
            break;
        }

        if filtermap != FilterMap::Foreach {
            let new_type = unsafe { nvim_tv_get_type(newtv) };
            if new_type != VAR_NUMBER && new_type != VAR_BOOL {
                unsafe { tv_clear(newtv) };
                unsafe { emsg(gettext(e_invalblob.as_ptr())) };
                break;
            }
            if filtermap != FilterMap::Filter {
                // map/mapnew: replace byte if changed
                let new_val = unsafe { nvim_tv_get_number(newtv) };
                if new_val != i64::from(val) {
                    unsafe { nvim_blob_set_byte(b_ret, i, new_val as u8) };
                }
            } else if rem {
                // filter: remove byte by shifting
                let data = unsafe { nvim_blob_get_ga_data(blob_arg) };
                let b_len_now = unsafe { nvim_blob_get_len(blob_arg) };
                unsafe {
                    std::ptr::copy(
                        data.add(i as usize + 1),
                        data.add(i as usize),
                        (b_len_now - i - 1) as usize,
                    );
                    nvim_blob_set_ga_len(blob_arg, b_len_now - 1);
                }
                i -= 1;
            }
        }

        i += 1;
        idx += 1;
    }

    unsafe { nvim_blob_set_lock(b, prev_lock) };
}

// =============================================================================
// filter_map_string
// =============================================================================

/// filter()/map()/foreach() implementation for a String.
unsafe fn filter_map_string(
    str_ptr: *const c_char,
    filtermap: FilterMap,
    expr: TypevalHandle,
    rettv: TypevalHandle,
) {
    unsafe { nvim_tv_set_type(rettv, VAR_STRING) };
    unsafe { nvim_tv_set_string(rettv, std::ptr::null_mut()) };

    // set_vim_var_nr() doesn't set the type
    unsafe { set_vim_var_type(VV_KEY, VAR_NUMBER) };

    let mut ga = GArrayT::new();
    unsafe { ga_init(&raw mut ga, 1, 80) };

    let mut p = str_ptr;
    let mut idx = 0i32;

    while !unsafe { *p == 0 } {
        let len = unsafe { utfc_ptr2len(p) } as usize;

        // Build VAR_STRING typval for one codepoint
        let dup = unsafe { xmemdupz(p.cast::<c_void>(), len) };
        let mut tv_bytes = [0u8; 16];
        let tv_handle = unsafe { TypevalHandle::from_ptr(tv_bytes.as_mut_ptr().cast::<c_void>()) };
        unsafe {
            nvim_tv_set_type(tv_handle, VAR_STRING);
            // v_string is at offset 8 in typval_T
            *(tv_bytes.as_mut_ptr().add(8).cast::<*mut c_char>()) = dup;
        }

        unsafe { set_vim_var_nr(VV_KEY, i64::from(idx)) };

        let mut newtv_bytes = [0u8; 16];
        let newtv = unsafe { TypevalHandle::from_ptr(newtv_bytes.as_mut_ptr().cast::<c_void>()) };
        let mut rem = false;

        let r = unsafe { filter_map_one(tv_handle, expr, filtermap, newtv, &raw mut rem) };
        if r == FAIL || unsafe { did_emsg } != 0 {
            unsafe { tv_clear(newtv) };
            unsafe { tv_clear(tv_handle) };
            break;
        }

        if filtermap == FilterMap::Map || filtermap == FilterMap::MapNew {
            let new_type = unsafe { nvim_tv_get_type(newtv) };
            if new_type != VAR_STRING {
                unsafe { tv_clear(newtv) };
                unsafe { tv_clear(tv_handle) };
                unsafe { emsg(gettext(e_string_required.as_ptr())) };
                break;
            }
            let s = unsafe { nvim_tv_get_string_ptr(newtv) };
            unsafe { ga_concat(&raw mut ga, s) };
        } else if filtermap == FilterMap::Foreach || !rem {
            let s = unsafe { nvim_tv_get_string_ptr(tv_handle) };
            unsafe { ga_concat(&raw mut ga, s) };
        }

        unsafe { tv_clear(newtv) };
        unsafe { tv_clear(tv_handle) };

        p = unsafe { p.add(len) };
        idx += 1;
    }

    unsafe { ga_append(&raw mut ga, 0) }; // NUL-terminate
                                          // Transfer ownership of ga_data to rettv (do NOT ga_clear)
    unsafe { nvim_tv_set_string(rettv, ga.ga_data.cast::<c_char>()) };
}

// =============================================================================
// filter_map_list
// =============================================================================

/// filter()/map()/foreach() implementation for a List.
unsafe fn filter_map_list(
    l: ListHandle,
    filtermap: FilterMap,
    arg_errmsg: *const c_char,
    expr: TypevalHandle,
    rettv: TypevalHandle,
) {
    if filtermap == FilterMap::MapNew {
        unsafe { nvim_tv_set_type(rettv, VarType::List as c_int) };
        unsafe { nvim_tv_set_list(rettv, ListHandle::null()) };
    }
    if l.is_null()
        || (filtermap == FilterMap::Filter
            && unsafe { value_check_lock(nvim_list_get_lock(l), arg_errmsg, TV_TRANSLATE) })
    {
        return;
    }

    let l_ret = if filtermap == FilterMap::MapNew {
        unsafe { tv_list_alloc_ret(rettv, K_LIST_LEN_UNKNOWN) }
    } else {
        ListHandle::null()
    };

    // set_vim_var_nr() doesn't set the type
    unsafe { set_vim_var_type(VV_KEY, VAR_NUMBER) };

    let prev_lock = unsafe { nvim_list_get_lock(l) };
    if prev_lock == VAR_UNLOCKED {
        unsafe { nvim_list_set_lock(l, VAR_LOCKED) };
    }

    let mut li = if l.is_null() {
        ListItemHandle::null()
    } else {
        unsafe { nvim_list_get_first(l) }
    };
    let mut idx = 0i32;

    while !li.is_null() {
        if filtermap == FilterMap::Map {
            let item_tv = unsafe { nvim_listitem_get_tv(li) };
            let item_lock = crate::get_v_lock(item_tv);
            if unsafe { value_check_lock(item_lock, arg_errmsg, TV_TRANSLATE) } {
                break;
            }
        }

        unsafe { set_vim_var_nr(VV_KEY, i64::from(idx)) };

        let mut newtv_bytes = [0u8; 16];
        let newtv = unsafe { TypevalHandle::from_ptr(newtv_bytes.as_mut_ptr().cast::<c_void>()) };
        let mut rem = false;
        let item_tv = unsafe { nvim_listitem_get_tv(li) };

        if unsafe { filter_map_one(item_tv, expr, filtermap, newtv, &raw mut rem) } == FAIL {
            break;
        }
        if unsafe { did_emsg } != 0 {
            unsafe { tv_clear(newtv) };
            break;
        }

        if filtermap == FilterMap::Map {
            unsafe { tv_clear(item_tv) };
            unsafe {
                std::ptr::copy_nonoverlapping(
                    newtv_bytes.as_ptr(),
                    item_tv.as_ptr().cast::<u8>().cast_mut(),
                    16,
                );
                nvim_tv_set_lock(item_tv, VAR_UNLOCKED);
            }
        } else if filtermap == FilterMap::MapNew {
            // tv_list_append_owned_tv takes *const typval_T by pointer in the C wrapper
            unsafe { tv_list_append_owned_tv(l_ret, newtv_bytes.as_ptr().cast::<c_void>()) };
        }

        if filtermap == FilterMap::Filter && rem {
            li = unsafe { tv_list_item_remove(l, li) };
        } else {
            li = unsafe { nvim_listitem_get_next(li) };
        }

        idx += 1;
    }

    unsafe { nvim_list_set_lock(l, prev_lock) };
}

// =============================================================================
// filter_map (dispatcher)
// =============================================================================

/// Implementation of map(), filter(), foreach() and mapnew().
pub(crate) unsafe fn filter_map_impl(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    filtermap: FilterMap,
) {
    let func_name = filtermap.func_name();
    let arg_errmsg = filtermap.arg_errmsg();

    let arg0 = unsafe { nvim_typval_array_get(argvars, 0) };
    let arg0_type = unsafe { nvim_tv_get_type(arg0) };

    // map/filter/foreach return the first argument (except mapnew with non-string)
    if filtermap != FilterMap::MapNew && arg0_type != VAR_STRING {
        unsafe { tv_copy(arg0.as_ptr(), rettv.as_ptr().cast_mut()) };
    }

    let is_blob = arg0_type == VarType::Blob as c_int;
    let is_list = arg0_type == VarType::List as c_int;
    let is_dict = arg0_type == VarType::Dict as c_int;
    let is_string = arg0_type == VAR_STRING;

    if !is_blob && !is_list && !is_dict && !is_string {
        unsafe {
            semsg(
                gettext(
                    E_ARGUMENT_OF_STR_MUST_BE_LIST_STRING_DICTIONARY_OR_BLOB
                        .as_ptr()
                        .cast::<c_char>(),
                ),
                func_name,
            )
        };
        return;
    }

    let expr = unsafe { nvim_typval_array_get(argvars, 1) };
    if unsafe { nvim_tv_get_type(expr) } == VAR_UNKNOWN {
        return;
    }

    // Save v:val and v:key
    let mut save_val = [0u8; 16];
    let save_val_handle =
        unsafe { TypevalHandle::from_ptr(save_val.as_mut_ptr().cast::<c_void>()) };
    let mut save_key = [0u8; 16];
    let save_key_handle =
        unsafe { TypevalHandle::from_ptr(save_key.as_mut_ptr().cast::<c_void>()) };

    unsafe { prepare_vimvar(VV_VAL, save_val_handle) };
    unsafe { prepare_vimvar(VV_KEY, save_key_handle) };

    // Save did_emsg and reset to detect errors in the callback
    let save_did_emsg = unsafe { did_emsg };
    unsafe { did_emsg = 0 };

    if is_dict {
        let d = unsafe { nvim_tv_get_dict(arg0) };
        unsafe { filter_map_dict(d, filtermap, arg_errmsg, expr, rettv) };
    } else if is_blob {
        let b = unsafe { nvim_tv_get_blob(arg0) };
        unsafe { filter_map_blob(b, filtermap, expr, arg_errmsg, rettv) };
    } else if is_string {
        let s = unsafe { tv_get_string(arg0) };
        unsafe { filter_map_string(s, filtermap, expr, rettv) };
    } else {
        // list
        let l = unsafe { nvim_tv_get_list(arg0) };
        unsafe { filter_map_list(l, filtermap, arg_errmsg, expr, rettv) };
    }

    unsafe { restore_vimvar(VV_KEY, save_key_handle) };
    unsafe { restore_vimvar(VV_VAL, save_val_handle) };

    unsafe { did_emsg |= save_did_emsg };
}

// =============================================================================
// Exported VimL function entry points
// =============================================================================

/// "filter()" function — replaces C f_filter.
#[export_name = "f_filter"]
pub unsafe extern "C" fn rs_f_filter(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *const c_void,
) {
    unsafe { filter_map_impl(argvars, rettv, FilterMap::Filter) };
}

/// "map()" function — replaces C f_map.
#[export_name = "f_map"]
pub unsafe extern "C" fn rs_f_map(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *const c_void,
) {
    unsafe { filter_map_impl(argvars, rettv, FilterMap::Map) };
}

/// "mapnew()" function — replaces C f_mapnew.
#[export_name = "f_mapnew"]
pub unsafe extern "C" fn rs_f_mapnew(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *const c_void,
) {
    unsafe { filter_map_impl(argvars, rettv, FilterMap::MapNew) };
}

/// "foreach()" function — replaces C f_foreach.
#[export_name = "f_foreach"]
pub unsafe extern "C" fn rs_f_foreach(
    argvars: TypevalHandle,
    rettv: TypevalHandle,
    _fptr: *const c_void,
) {
    unsafe { filter_map_impl(argvars, rettv, FilterMap::Foreach) };
}
