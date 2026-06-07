//! Variable assignment engine for VimL.
//!
//! Phase 3: ports `before_set_vvar`, `set_var_const`, and `set_var` from
//! `src/nvim/eval/vars.c` into Rust.
//!
//! ## Architecture
//!
//! `set_var` / `set_var_const` are the primary variable-assignment entry
//! points called throughout the codebase. They:
//!
//! 1. Resolve the scope hashtab + dict via `rs_find_var_ht_dict`.
//! 2. Validate name legality and lock status.
//! 3. For existing v: variables, delegate to `before_set_vvar` for
//!    type-specific handling and watcher notification.
//! 4. For new variables, allocate a `dictitem_T` on the heap and add it to
//!    the hashtab.
//! 5. Assign the value (copy vs. move semantics).
//! 6. Notify watchers and apply `tv_item_lock` for `:const`.
//!
//! ## TypvalT layout (16 bytes on 64-bit)
//!
//! ```text
//! offset 0: v_type  (c_int, 4 bytes) — VarType enum
//! offset 4: v_lock  (c_int, 4 bytes) — VarLockStatus
//! offset 8: vval    (union, 8 bytes) — interpreted via v_type
//! ```
//!
//! `dictitem_T` has `di_tv` (typval_T, 16 bytes) at offset 0, so a
//! `DictitemHandle` pointer casts directly to `*mut TypvalT`.

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::borrow_as_ptr)]
#![allow(clippy::inline_always)]
#![allow(clippy::if_not_else)]
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// VarType constants (matching C's VarType in typval_defs.h)
// =============================================================================

const VAR_UNKNOWN: c_int = 0;
const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;
const VAR_FLOAT: c_int = 3;
// VAR_LIST, VAR_DICT, etc. — not needed for our switch

// VAR_UNLOCKED = 0 (VarLockStatus)
const VAR_UNLOCKED: c_int = 0;

// FAIL = 0, OK = 1 (return codes for hash_add)
const FAIL: c_int = 0;

// DI_FLAGS constants
const DI_FLAGS_ALLOC: u8 = 16;
const DI_FLAGS_LOCK: u8 = 8;

// UPD_SOME_VALID = 35
const UPD_SOME_VALID: c_int = 35;

// =============================================================================
// TypvalT — 16-byte mirror of C typval_T
// =============================================================================

/// Thin mirror of C's `typval_T`.
///
/// Layout must match `typval_T` in `src/nvim/eval/typval_defs.h` exactly.
/// The `vval` union is treated as a raw `u64`; callers interpret it via `v_type`.
#[repr(C)]
struct TypvalT {
    v_type: c_int, // VarType (int)
    v_lock: c_int, // VarLockStatus (int)
    vval: u64,     // union — 8 bytes; interpret via v_type
}

const _: () = assert!(std::mem::size_of::<TypvalT>() == 16);

impl TypvalT {
    /// Read `vval.v_string` (valid when `v_type == VAR_STRING`).
    unsafe fn v_string(ptr: *const Self) -> *mut c_char {
        (*ptr).vval as usize as *mut c_char
    }
    /// Write `vval.v_string`.
    unsafe fn set_v_string(ptr: *mut Self, s: *mut c_char) {
        (*ptr).vval = s as usize as u64;
    }
    /// Read `vval.v_number` (valid when `v_type == VAR_NUMBER`).
    unsafe fn v_number(ptr: *const Self) -> i64 {
        (*ptr).vval as i64
    }
    /// Write `vval.v_number`.
    unsafe fn set_v_number(ptr: *mut Self, n: i64) {
        (*ptr).vval = n as u64;
    }
}

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // --- Lookup ---
    fn rs_find_var_ht_dict(
        name: *const c_char,
        name_len: usize,
        varname: *mut *const c_char,
        d: *mut *mut c_void,
    ) -> *mut c_void; // returns hashtab_T* or NULL

    fn find_var_in_ht(
        ht: *mut c_void,
        htname: c_int,
        varname: *const c_char,
        varname_len: usize,
        no_autoload: c_int,
    ) -> *mut c_void; // returns dictitem_T* or NULL

    fn find_var_in_scoped_ht(
        name: *const c_char,
        name_len: usize,
        no_autoload: c_int,
    ) -> *mut c_void; // returns dictitem_T* or NULL

    // --- Watcher ---
    fn nvim_vars_dict_is_watched(dict: *const c_void) -> bool;
    fn tv_dict_watcher_notify(
        dict: *mut c_void,
        key: *const c_char,
        newtv: *mut c_void,
        oldtv: *mut c_void,
    );

    // --- typval_T operations ---
    fn tv_copy(from: *mut c_void, to: *mut c_void);
    fn tv_clear(tv: *mut c_void);
    fn tv_get_string(tv: *mut c_void) -> *const c_char;
    fn tv_get_number(tv: *mut c_void) -> i64;
    fn nvim_vars_tv_is_func(tv: *const c_void) -> bool;
    fn nvim_vars_tv_init(tv: *mut c_void);
    fn nvim_vars_tv_item_lock_const(tv: *mut c_void);

    // --- dictitem_T accessors ---
    fn nvim_vars_dictitem_get_flags(di: *mut c_void) -> c_int;
    fn nvim_vars_dictitem_set_flags(di: *mut c_void, flags: u8);
    fn nvim_vars_dictitem_key_ptr(di: *mut c_void) -> *mut c_char;
    fn nvim_vars_dictitem_keyoff() -> usize;

    // --- Hash table ---
    fn hash_add(ht: *mut c_void, key: *mut c_char) -> c_int;
    fn get_funccal_args_ht() -> *mut c_void;

    // --- vimvarht / vimvardict ---
    fn nvim_vars_get_vimvarht() -> *mut c_void;
    fn nvim_vars_get_vimvardict() -> *mut c_void;

    // --- Memory ---
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // --- Error messages ---
    fn semsg(fmt: *const c_char, ...) -> c_int;
    fn emsg(s: *const c_char) -> c_int;
    fn nvim_vars_e_illvar() -> *const c_char;
    fn nvim_vars_e_cannot_mod() -> *const c_char;
    fn nvim_vars_e_setting_v_wrong_type() -> *const c_char;

    // --- Searchforward / hlsearch ---
    fn set_search_direction(cdir: c_int);
    fn redraw_all_later(kind: c_int);
    fn nvim_vars_set_no_hlsearch(v: bool);
}

// These Rust functions are in the same crate (checks.rs); call them directly.
use crate::checks::{rs_valid_varname, rs_var_check_lock, rs_var_check_ro, rs_var_wrong_func_name};

// "searchforward" and "hlsearch" as byte slices for comparison
const SEARCHFORWARD: &[u8] = b"searchforward";
const HLSEARCH: &[u8] = b"hlsearch";

// =============================================================================
// before_set_vvar
// =============================================================================

/// Handle type-specific assignment for v: variables.
///
/// Exported as `before_set_vvar`, replacing the C function.
///
/// Returns `true` if the caller should proceed with normal value assignment,
/// `false` if the assignment was handled here (or rejected via `*type_error`).
///
/// # Safety
/// All pointers must be valid. `di` must point to a live `dictitem_T` with
/// the `di_tv` at offset 0. `tv` must be a valid `typval_T*`.
#[unsafe(export_name = "before_set_vvar")]
pub unsafe extern "C" fn rs_before_set_vvar(
    varname: *const c_char,
    di: *mut c_void,
    tv: *mut c_void,
    copy: bool,
    watched: bool,
    type_error: *mut bool,
) -> bool {
    let di_tv = di.cast::<TypvalT>();
    let tv_typed = tv.cast::<TypvalT>();

    let di_vtype = (*di_tv).v_type;
    let tv_vtype = (*tv_typed).v_type;

    if di_vtype == VAR_STRING {
        // Save old value for watcher before modifying.
        let mut oldtv = zeroed_typval();
        if watched {
            tv_copy(
                di.cast::<TypvalT>() as *mut c_void,
                oldtv.as_mut_ptr().cast(),
            );
        }

        // Free the existing string.
        let old_str = TypvalT::v_string(di_tv);
        xfree(old_str.cast());
        TypvalT::set_v_string(di_tv, std::ptr::null_mut());

        if copy || tv_vtype != VAR_STRING {
            // Convert to string via tv_get_string.
            let val = tv_get_string(tv);
            // Guard: if di_tv.vval.v_string is still NULL (tv_get_string may set
            // v:errmsg which triggers re-entry), only then copy.
            if TypvalT::v_string(di_tv).is_null() {
                TypvalT::set_v_string(di_tv, xstrdup(val));
            }
        } else {
            // Take over the string to avoid extra alloc/free.
            let src = TypvalT::v_string(tv_typed);
            TypvalT::set_v_string(di_tv, src);
            TypvalT::set_v_string(tv_typed, std::ptr::null_mut());
        }

        if watched {
            let vimvardict = nvim_vars_get_vimvardict();
            tv_dict_watcher_notify(
                vimvardict,
                varname,
                di_tv as *mut c_void,
                oldtv.as_mut_ptr().cast(),
            );
            tv_clear(oldtv.as_mut_ptr().cast());
        }
        return false;
    }

    if di_vtype == VAR_NUMBER {
        let mut oldtv = zeroed_typval();
        if watched {
            tv_copy(di_tv as *mut c_void, oldtv.as_mut_ptr().cast());
        }

        let num = tv_get_number(tv);
        TypvalT::set_v_number(di_tv, num);

        // Side-effects for specific v: variables.
        let varname_bytes = std::slice::from_raw_parts(varname.cast::<u8>(), {
            let mut len = 0usize;
            let mut p = varname;
            while *p != 0 {
                len += 1;
                p = p.add(1);
            }
            len
        });

        if varname_bytes == SEARCHFORWARD {
            set_search_direction(if num != 0 {
                c_int::from(b'/')
            } else {
                c_int::from(b'?')
            });
        } else if varname_bytes == HLSEARCH {
            nvim_vars_set_no_hlsearch(num == 0);
            redraw_all_later(UPD_SOME_VALID);
        }

        if watched {
            let vimvardict = nvim_vars_get_vimvardict();
            tv_dict_watcher_notify(
                vimvardict,
                varname,
                di_tv as *mut c_void,
                oldtv.as_mut_ptr().cast(),
            );
            tv_clear(oldtv.as_mut_ptr().cast());
        }
        return false;
    }

    // Type mismatch check.
    if di_vtype != tv_vtype {
        *type_error = true;
        return false;
    }

    // Proceed with normal assignment.
    true
}

// =============================================================================
// set_var_const
// =============================================================================

/// Set a VimL variable to the given value, optionally making it const.
///
/// Exported as `set_var_const`, replacing the C function.
///
/// # Safety
/// All pointers must be valid. `tv` must point to a live `typval_T`.
#[unsafe(export_name = "set_var_const")]
pub unsafe extern "C" fn rs_set_var_const(
    name: *const c_char,
    name_len: usize,
    tv: *mut c_void,
    copy: bool,
    is_const: bool,
) {
    let mut varname: *const c_char = std::ptr::null();
    let mut dict: *mut c_void = std::ptr::null_mut();

    let ht = rs_find_var_ht_dict(
        name,
        name_len,
        std::ptr::addr_of_mut!(varname),
        std::ptr::addr_of_mut!(dict),
    );

    // dict_is_watched needs the dict ptr (may be NULL for compat hashtab)
    let watched = if dict.is_null() {
        false
    } else {
        nvim_vars_dict_is_watched(dict)
    };

    if ht.is_null() || *varname == 0 {
        semsg(nvim_vars_e_illvar(), name);
        return;
    }

    let varname_len = name_len - (varname as usize - name as usize);

    // Look up existing variable.
    let mut di = find_var_in_ht(ht, 0, varname, varname_len, 1);
    if di.is_null() {
        di = find_var_in_scoped_ht(name, name_len, 1);
    }

    // Validate funcref name.
    if nvim_vars_tv_is_func(tv) && rs_var_wrong_func_name(name, di.is_null()) {
        return;
    }

    // Stack-allocated old value for watcher notification.
    let mut oldtv = zeroed_typval();

    let vimvarht = nvim_vars_get_vimvarht();

    if !di.is_null() {
        // Updating an existing variable.
        if is_const {
            emsg(nvim_vars_e_cannot_mod());
            return;
        }

        // Check in order: RO, value lock, var lock.
        let flags = nvim_vars_dictitem_get_flags(di);
        if rs_var_check_ro(flags, name, name_len)
            || value_check_lock_tv(di, name, name_len)
            || rs_var_check_lock(flags, name, name_len)
        {
            return;
        }

        // v: variable special handling.
        if ht == vimvarht {
            let mut type_error = false;
            if !rs_before_set_vvar(varname, di, tv, copy, watched, &mut type_error) {
                if type_error {
                    semsg(nvim_vars_e_setting_v_wrong_type(), varname);
                }
                return;
            }
        }

        if watched {
            tv_copy(nvim_vars_di_tv_ptr(di), oldtv.as_mut_ptr().cast());
        }
        tv_clear(nvim_vars_di_tv_ptr(di));
    } else {
        // Adding a new variable.
        // Can't add to v: or a: scopes.
        if ht == vimvarht || ht == get_funccal_args_ht() {
            semsg(nvim_vars_e_illvar(), name);
            return;
        }

        if !rs_valid_varname(varname) {
            return;
        }

        // dict must be non-NULL here (assertion equivalent).
        debug_assert!(!dict.is_null());

        // Allocate a new dictitem_T: keyoff + varname_len + 1 bytes.
        let keyoff = nvim_vars_dictitem_keyoff();
        di = xmalloc(keyoff + varname_len + 1);

        // Copy the key.
        let key_ptr = (di as *mut u8).add(keyoff).cast::<c_char>();
        std::ptr::copy_nonoverlapping(varname, key_ptr, varname_len + 1);

        if hash_add(ht, key_ptr) == FAIL {
            xfree(di);
            return;
        }

        let mut flags: u8 = DI_FLAGS_ALLOC;
        if is_const {
            flags |= DI_FLAGS_LOCK;
        }
        nvim_vars_dictitem_set_flags(di, flags);
    }

    // Assign the value: copy or move.
    let di_tv_ptr = nvim_vars_di_tv_ptr(di);
    let tv_typed = tv.cast::<TypvalT>();
    let tv_vtype = (*tv_typed).v_type;

    if copy || tv_vtype == VAR_NUMBER || tv_vtype == VAR_FLOAT {
        tv_copy(tv, di_tv_ptr);
    } else {
        // Move: copy the 16-byte typval, reset v_lock to UNLOCKED, init source.
        std::ptr::copy_nonoverlapping(tv.cast::<TypvalT>(), di_tv_ptr.cast::<TypvalT>(), 1);
        (*di_tv_ptr.cast::<TypvalT>()).v_lock = VAR_UNLOCKED;
        nvim_vars_tv_init(tv);
    }

    // Watcher notification.
    if watched {
        let key = nvim_vars_dictitem_key_ptr(di);
        tv_dict_watcher_notify(dict, key, di_tv_ptr, oldtv.as_mut_ptr().cast());
        tv_clear(oldtv.as_mut_ptr().cast());
    }

    // Lock if :const.
    if is_const {
        nvim_vars_tv_item_lock_const(di_tv_ptr);
    }
}

// =============================================================================
// set_var
// =============================================================================

/// Set a VimL variable to the given value (non-const variant).
///
/// Exported as `set_var`, replacing the C function.
///
/// # Safety
/// All pointers must be valid.
#[unsafe(export_name = "set_var")]
pub unsafe extern "C" fn rs_set_var(
    name: *const c_char,
    name_len: usize,
    tv: *mut c_void,
    copy: bool,
) {
    rs_set_var_const(name, name_len, tv, copy, false);
}

// =============================================================================
// Helpers
// =============================================================================

/// Return a zeroed 16-byte buffer aligned as TypvalT, initialized to
/// `TV_INITIAL_VALUE` (v_type=VAR_UNKNOWN=0, v_lock=VAR_UNLOCKED=0, vval=0).
///
/// Using `[u8; 16]` instead of `TypvalT` to avoid needing to initialize the
/// union field explicitly.
#[inline(always)]
fn zeroed_typval() -> [u8; 16] {
    [0u8; 16]
}

/// Get the `di_tv` pointer from a `dictitem_T*` (at offset 0).
#[inline(always)]
unsafe fn nvim_vars_di_tv_ptr(di: *mut c_void) -> *mut c_void {
    di // di_tv is at offset 0; no adjustment needed
}

/// Call `value_check_lock` for the di_tv.v_lock of a dictitem.
/// Bridges the C `value_check_lock(di->di_tv.v_lock, ...)` call.
#[inline(always)]
unsafe fn value_check_lock_tv(di: *mut c_void, name: *const c_char, name_len: usize) -> bool {
    let v_lock = (*di.cast::<TypvalT>()).v_lock;
    value_check_lock(v_lock, name, name_len)
}

extern "C" {
    fn value_check_lock(lock: c_int, name: *const c_char, name_len: usize) -> bool;
}
