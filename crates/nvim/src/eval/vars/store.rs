//! Assigning to a name, and the four checks that can refuse.
//!
//! [`set_var_const`] is the single entry point every assignment reaches; the
//! `var_check_*` trio reads `di_flags` and produces E46 / E795 / E1122, and
//! [`var_wrong_func_name`] and [`valid_varname`] reject the name itself.
//!
//! The three checks each resolve `name_len` the same way before reporting,
//! because it may be one of the two sentinels `TV_TRANSLATE` and
//! `TV_CSTRING` rather than a length: a caller with no length to hand passes
//! one of those so that `strlen` and `gettext` are only paid for on the path
//! that actually reports.  The repetition is upstream's, and keeping it is
//! what keeps each check a single `unsafe` block.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::semsg_c;
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::types::{FAIL, NUL};

// ---------------------------------------------------------------------
// Reporting, and the one place a value this frame owns is freed.
//
// These live beside the `var_check_*` trio rather than in `mod.rs` because
// this is the file whose job is refusing an assignment and saying why; every
// other file of the family reaches them through the `pub use self::store::*`
// re-export.

/// The translation of one of the editor's message strings, which are held as
/// NUL-terminated `static` byte arrays.
///
/// Safe because the whole array is passed rather than a pointer into it: its
/// last byte is the terminator, so `gettext` is handed a NUL-terminated
/// string, and what it answers is either that `static` or one of its own --
/// both of which outlive the report it is passed to.
pub(crate) fn translate<const N: usize>(msg: &'static [c_char; N]) -> *const c_char {
    debug_assert_eq!(msg[N - 1], 0, "an editor message is NUL-terminated");
    // SAFETY: a NUL-terminated `static`, as the assertion above says.
    unsafe { gettext(msg.as_ptr()) }
}

/// [`translate`] for a message written as a literal, where the terminator is
/// the type's own guarantee.
pub(crate) fn translate_lit(msg: &'static CStr) -> *const c_char {
    // SAFETY: a `CStr` is NUL-terminated by construction.
    unsafe { gettext(msg.as_ptr()) }
}

/// Report one of the editor's `static` messages, translated.
pub(crate) fn emsg_static<const N: usize>(msg: &'static [c_char; N]) {
    // SAFETY: [`translate`]'s answer is a live NUL-terminated string.
    unsafe { emsg(translate(msg)) };
}

/// [`emsg_static`] for a message written as a literal.
pub(crate) fn emsg_lit(msg: &'static CStr) {
    // SAFETY: as [`emsg_static`].
    unsafe { emsg(translate_lit(msg)) };
}

/// Clear a value this frame owns, freeing whatever it holds.
///
/// Safe: `tv_clear`'s only precondition is a live, writable value, which an
/// exclusive borrow of the caller's own local is. Nothing it frees runs user
/// code, so the borrow cannot be re-entered through.
pub(crate) fn clear_local(tv: &mut typval_T) {
    // SAFETY: an exclusive borrow of a live local.
    unsafe { tv_clear(&raw mut *tv) };
}

/// Store `tv` in the variable `name`.
///
/// # Safety
/// `name` points at `name_len` readable bytes and is NUL-terminated there;
/// `tv` is a live value.
pub unsafe fn set_var(name: *const c_char, name_len: size_t, tv: *mut typval_T, copy: bool) {
    unsafe { set_var_const(name, name_len, tv, copy, false) }
}

/// Store `tv` in the variable `name`, creating it if it does not exist.
///
/// `copy` asks for a copy of the value; without it `tv` is moved out of and
/// left `VAR_UNKNOWN`, except for the two scalar types, which are copied
/// either way.  `is_const` is `:const`: the variable is created locked, and
/// an *existing* one is refused outright.
///
/// # Safety
/// As [`set_var`].
pub unsafe fn set_var_const(
    name: *const c_char,
    name_len: size_t,
    tv: *mut typval_T,
    copy: bool,
    is_const: bool,
) {
    let mut varname: *const c_char = ptr::null();
    let mut dict: *mut dict_T = ptr::null_mut();
    let ht = unsafe { find_var_ht_dict(name, name_len, &raw mut varname, &raw mut dict) };
    let watched = unsafe { tv_dict_is_watched(dict) };

    if ht.is_null() || unsafe { *varname } == NUL as c_char {
        semsg_c!(translate(&e_illvar), name);
        return;
    }
    // `varname` is `name` itself or `name + 2`, so this cannot go
    // negative; a name that is nothing but a scope prefix was caught by
    // the NUL test above.
    let varname_len = name_len - unsafe { varname.offset_from(name) } as size_t;

    let mut di = unsafe { find_var_in_ht(ht, 0, varname, varname_len, true) };
    if di.is_null() {
        // Search the parent scope, which a lambda can reference.
        di = unsafe { find_var_in_scoped_ht(name, name_len, true as c_int) };
    }

    // SAFETY: the caller's obligation -- a live value.
    let tvh = unsafe { Tv::new(tv) };
    if tv_is_func(*tvh) && unsafe { var_wrong_func_name(name, di.is_null()) } {
        return;
    }

    let mut oldtv = TV_INITIAL_VALUE;
    if !di.is_null() {
        if is_const {
            emsg_static(&e_cannot_mod);
            return;
        }

        // The order is upstream's and is kept for backwards
        // compatibility: read-only first, then the value's lock, then
        // the variable's.
        // SAFETY: `find_var_in_ht` answers a live item of a live scope.
        let item = unsafe { Di::new(di) };
        let (flags, lock) = (item.di_flags as c_int, item.di_tv.v_lock);
        if unsafe { var_check_ro(flags, name, name_len) }
            || unsafe { value_check_lock(lock, name, name_len) }
            || unsafe { var_check_lock(flags, name, name_len) }
        {
            return;
        }

        // A `v:` variable keeps its declared type, and two of them have
        // a side effect on assignment; `before_set_vvar` is both, and it
        // answers false when it has already done the store itself.
        let mut type_error = false;
        let err = &raw mut type_error;
        if ht == get_vimvar_ht() && !unsafe { before_set_vvar(varname, di, tv, copy, watched, err) }
        {
            if type_error {
                semsg_c!(
                    translate_lit(e_setting_v_str_to_value_with_wrong_type),
                    varname,
                );
            }
            return;
        }

        let cur = item.field_ptr(offset_of!(dictitem_T, di_tv));
        if watched {
            unsafe { tv_copy(cur, &raw mut oldtv) };
        }
        unsafe { tv_clear(cur) };
    } else {
        // A new variable. `v:` and `a:` do not take one.
        if ht == get_vimvar_ht() || ht == unsafe { get_funccal_args_ht() } {
            semsg_c!(translate(&e_illvar), name);
            return;
        }
        if !unsafe { valid_varname(varname) } {
            return;
        }
        debug_assert!(!dict.is_null());

        // Upstream allocates and copies the key by hand here, taking
        // `varname_len + 1` bytes so that the NUL comes along.  That is
        // what `tv_dict_item_alloc_len` does, and it is where every
        // other item in the tree comes from; `valid_varname` has just
        // walked `varname` to its NUL, so the two agree on the length.
        di = unsafe { tv_dict_item_alloc_len(varname, varname_len) };
        if unsafe { hash_add(ht, tv_dict_item_key(di)) } == FAIL {
            unsafe { xfree(di.cast()) };
            return;
        }
        // SAFETY: the item just allocated.
        let mut item = unsafe { Di::new(di) };
        item.di_flags = DI_FLAGS_ALLOC as uint8_t;
        if is_const {
            item.di_flags |= DI_FLAGS_LOCK as uint8_t;
        }
    }

    // SAFETY: `di` is the item found or the one just added, and `tv` the
    // caller's live value.
    let mut item = unsafe { Di::new(di) };
    let cur = item.field_ptr(offset_of!(dictitem_T, di_tv));
    if copy || tvh.v_type == VAR_NUMBER || tvh.v_type == VAR_FLOAT {
        unsafe { tv_copy(tv, cur) };
    } else {
        item.di_tv = *tvh;
        item.di_tv.v_lock = VarLock::Unlocked;
        unsafe { tv_init(tv) };
    }

    if watched {
        let key = unsafe { tv_dict_item_key(di) };
        unsafe { tv_dict_watcher_notify(dict, key, cur, &raw mut oldtv) };
        clear_local(&mut oldtv);
    }

    if is_const {
        // Like `:lockvar! name`: lock the value and what it contains,
        // but only where the reference count is one, so that only
        // literal values are locked.
        unsafe { tv_item_lock(cur, DICT_MAXNEST, true, true) };
    }
}

/// Whether `flags` says the variable may not be written, reporting E46 or
/// E794 if so.
///
/// `name_len` may be `TV_TRANSLATE` or `TV_CSTRING` rather than a length.
///
/// # Safety
/// `name` is NUL-terminated, or `name_len` bytes long.
pub unsafe fn var_check_ro(flags: c_int, mut name: *const c_char, mut name_len: size_t) -> bool {
    let error_message = if flags & DI_FLAGS_RO as c_int != 0 {
        &raw const e_cannot_change_readonly_variable_str as *const c_char
    } else if flags & DI_FLAGS_RO_SBX as c_int != 0 && sandbox.get() != 0 {
        &raw const e_cannot_set_variable_in_sandbox_str as *const c_char
    } else {
        return false;
    };

    if name_len == TV_TRANSLATE as size_t {
        name = unsafe { gettext(name) };
        name_len = unsafe { strlen(name) };
    } else if name_len == TV_CSTRING as size_t {
        name_len = unsafe { strlen(name) };
    }
    semsg_c!(unsafe { gettext(error_message) }, name_len as c_int, name);
    true
}

/// Whether `flags` says the variable is locked, reporting E1122 if so.
///
/// # Safety
/// As [`var_check_ro`].
pub unsafe fn var_check_lock(flags: c_int, mut name: *const c_char, mut name_len: size_t) -> bool {
    if flags & DI_FLAGS_LOCK as c_int == 0 {
        return false;
    }
    if name_len == TV_TRANSLATE as size_t {
        name = unsafe { gettext(name) };
        name_len = unsafe { strlen(name) };
    } else if name_len == TV_CSTRING as size_t {
        name_len = unsafe { strlen(name) };
    }
    semsg_c!(
        translate_lit(c"E1122: Variable is locked: %.*s"),
        name_len as c_int,
        name,
    );
    true
}

/// Whether `flags` says the variable may not be deleted, reporting E795 if
/// so.
///
/// # Safety
/// As [`var_check_ro`].
pub unsafe fn var_check_fixed(flags: c_int, mut name: *const c_char, mut name_len: size_t) -> bool {
    if flags & DI_FLAGS_FIX as c_int == 0 {
        return false;
    }
    if name_len == TV_TRANSLATE as size_t {
        name = unsafe { gettext(name) };
        name_len = unsafe { strlen(name) };
    } else if name_len == TV_CSTRING as size_t {
        name_len = unsafe { strlen(name) };
    }
    semsg_c!(
        translate(&e_cannot_delete_variable_str),
        name_len as c_int,
        name,
    );
    true
}

/// Whether `name` may not hold a Funcref, reporting E704 or E705 if so.
///
/// A Funcref has to look like a function name -- capitalised, or scoped to
/// `w:`/`b:`/`s:`/`t:`, or autoloaded -- and `new_var` additionally forbids
/// shadowing a function that already exists.
///
/// # Safety
/// `name` is a NUL-terminated string.
pub unsafe fn var_wrong_func_name(name: *const c_char, new_var: bool) -> bool {
    // SAFETY: the caller's obligation -- a NUL-terminated name, so the
    // second byte is only read once the first has proved not to be the NUL.
    let lead = unsafe { *name } as u8;
    let has_scope = lead != NUL as u8 && unsafe { *name.add(1) } == b':' as c_char;
    // The character the capital is wanted at: past a scope prefix, if
    // there is one.
    let first = match has_scope {
        true => (unsafe { *name.add(2) }).cast_unsigned(),
        false => lead,
    };
    let scoped = unsafe { vim_strchr(c"wbst".as_ptr(), lead.into()) };
    let func_scope = has_scope && !scoped.is_null();

    if !func_scope
        && !first.is_ascii_uppercase()
        && unsafe { vim_strchr(name, b'#' as c_int) }.is_null()
    {
        let msg = c"E704: Funcref variable name must start with a capital: %s";
        semsg_c!(translate_lit(msg), name);
        return true;
    }
    // Don't allow hiding a function. With an existing variable this may
    // be assigning another function to the same one, whose type the
    // caller checks.
    if new_var && unsafe { function_exists(name, false) } {
        let msg = c"E705: Variable name conflicts with existing function: %s";
        semsg_c!(translate_lit(msg), name);
        return true;
    }
    false
}

/// Whether `varname` is spellable as a variable name, reporting E461 if not.
///
/// # Safety
/// `varname` is a NUL-terminated string.
pub unsafe fn valid_varname(varname: *const c_char) -> bool {
    let mut p = varname;
    // SAFETY: the caller's obligation -- a NUL-terminated name, which the
    // walk stops at.
    while unsafe { *p } != NUL as c_char {
        let c = unsafe { *p };
        if !eval_isnamec1(c_int::from(c.cast_unsigned()))
            && (p == varname || !ascii_isdigit(c_int::from(c)))
            && c != AUTOLOAD_CHAR as c_char
        {
            semsg_c!(translate(&e_illvar), varname);
            return false;
        }
        p = unsafe { p.add(1) };
    }
    true
}
