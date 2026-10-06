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
#![allow(unsafe_code)]

use crate::cstr;
use crate::eval::typval::DictRef;
use crate::semsg;
use crate::strings::has_char;
use crate::strings::vim_strchr;
use crate::tr_plural;
use core::ffi::{c_char, c_int};
use core::mem::offset_of;
use core::ptr;

use super::*;
use crate::eval::typval::{DictEntry, tv_dict_item_free};
use crate::message::emsg;
use crate::message_fmt::{c_str, c_str_len, emsg_text, msg_bytes};
use crate::os::cshim::gettext_ptr;
use crate::types::NUL;

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
/// Safe by construction: a `CStr` carries its terminator in the type, and
/// what `gettext` answers is either that `static` or one of its own -- both
/// of which outlive the report it is passed to.
pub(crate) fn translate(msg: &'static CStr) -> &'static CStr {
    gettext(msg)
}

/// Report one of the editor's `static` messages, translated.
pub(crate) fn emsg_static(msg: &'static CStr) {
    emsg(translate(msg));
}

/// Clear a value this frame owns, freeing whatever it holds.
///
/// Safe: `tv_clear`'s only precondition is a live, writable value, which an
/// exclusive borrow of the caller's own local is. Nothing it frees runs user
/// code, so the borrow cannot be re-entered through.
pub(crate) fn clear_local(tv: &mut TypVal) {
    tv_clear(&mut *tv);
}

/// Store `tv` in the variable `name`.
///
/// # Safety
/// `name` points at `name_len` readable bytes, which need not be followed by
/// a NUL; `tv` is a live value.
pub unsafe fn set_var(name: *const c_char, name_len: size_t, tv: &mut TypVal, copy: bool) {
    unsafe { set_var_const(name, name_len, tv, copy, false) }
}

/// [`set_var`] of the variable `name` names.
pub(crate) fn set_var_named(name: &CStr, tv: &mut TypVal, copy: bool) {
    // SAFETY: `name` is NUL-terminated at its length.
    unsafe { set_var(name.as_ptr(), name.count_bytes(), tv, copy) }
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
    tv: &mut TypVal,
    copy: bool,
    is_const: bool,
) {
    let mut dict: *mut Dict = ptr::null_mut();
    let (ht, varname) = unsafe { find_var_ht_dict(name, name_len, &raw mut dict) };
    let varname = name.wrapping_add(varname);
    let watched = dict_is_watched(unsafe { (dict).as_ref() });

    // The name, measured: nothing below reads past its length, so a caller
    // may hand a slice of a longer line.
    // SAFETY: the caller's `name_len` readable bytes.
    let name_bytes = unsafe { cstr::slice_at(name, name_len) };
    // `varname` is `name` itself or `name + 2`, inside the name.
    let varname_len = if ht.is_null() {
        0
    } else {
        name_len - unsafe { varname.offset_from(name) } as size_t
    };
    if ht.is_null() || varname_len == 0 {
        let name = msg_bytes(name_bytes);
        semsg!("E461: Illegal variable name: {name}");
        return;
    }
    let varname_bytes = &name_bytes[name_len - varname_len..];

    let mut di = unsafe { find_var_in_ht(ht, 0, varname, varname_len, true) };
    if di.is_null() {
        // Search the parent scope, which a lambda can reference.
        di = unsafe { find_var_in_scoped_ht(name, name_len, true as c_int) };
    }

    // SAFETY: the caller's obligation -- a live value.
    let mut tvh = unsafe { Tv::new(tv) };
    if tvh.is_func() && var_wrong_func_name_named(name_bytes, di.is_null()) {
        return;
    }

    let mut oldtv = TV_INITIAL_VALUE;
    if !di.is_null() {
        if is_const {
            emsg_static(e_cannot_mod);
            return;
        }

        // The order is upstream's and is kept for backwards
        // compatibility: read-only first, then the value's lock, then
        // the variable's.
        // SAFETY: `find_var_in_ht` answers a live item of a live scope.
        let item = unsafe { Di::new(di) };
        let (flags, lock) = (item.di_flags as c_int, item.di_lock);
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
        // The item's own key is the name without the `v:`, terminated.
        let key = unsafe { (*di).di_key.as_ptr() };
        if ht == get_vimvar_ht() && !unsafe { before_set_vvar(key, di, tv, copy, watched, err) } {
            if type_error {
                // SAFETY: the item's NUL-terminated key.
                let varname = unsafe { c_str(key) };
                semsg!("E963: Setting v:{varname} to value with wrong type");
            }
            return;
        }

        let cur = item.field_ptr::<TypVal>(offset_of!(DictItem, di_tv));
        if watched {
            unsafe { tv_copy(&*cur, &mut oldtv) };
        }
        unsafe { tv_clear(&mut *cur) };
    } else {
        // A new variable. `v:` and `a:` do not take one.
        if ht == get_vimvar_ht() || ht == get_funccal_args_ht() {
            let name = msg_bytes(name_bytes);
            semsg!("E461: Illegal variable name: {name}");
            return;
        }
        if !valid_varname_named(varname_bytes) {
            return;
        }
        debug_assert!(!dict.is_null());

        // Upstream allocates and copies the key by hand here, taking
        // `varname_len + 1` bytes so that the NUL comes along.  That is
        // what `tv_dict_item_alloc_len` does, and it is where every
        // other item in the tree comes from; `valid_varname` has just
        // walked `varname` to its NUL, so the two agree on the length.
        // SAFETY: `varname_len` readable bytes; the table frees the item by
        // its `DI_FLAGS_ALLOC`.
        di = Box::into_raw(DictItem::boxed(unsafe {
            cstr::slice_at(varname, varname_len)
        }));
        if unsafe { hash_add(ht, DictEntry::new(di)) }.is_err() {
            unsafe { tv_dict_item_free(di) };
            return;
        }
        // SAFETY: the item just allocated, which arrives `DI_FLAGS_ALLOC`.
        if is_const {
            unsafe { Di::new(di) }.di_flags |= DI_FLAGS_LOCK as uint8_t;
        }
    }

    // SAFETY: `di` is the item found or the one just added, and `tv` the
    // caller's live value.
    // The store goes through the item's *value*: `cur` points into the item,
    // so writing a field through a borrow of the whole item would invalidate
    // the pointer the watcher notification and the `:const` lock below are
    // handed. See [`Live`]'s module docs.
    let cur: *mut TypVal = unsafe { Di::new(di) }.field_ptr(offset_of!(DictItem, di_tv));
    if copy || tvh.v_type() == VAR_NUMBER || tvh.v_type() == VAR_FLOAT {
        unsafe { tv_copy(tv, &mut *cur) };
    } else {
        let mut into = unsafe { Tv::new(cur) };
        *into = tvh.take();
    }
    // Upstream leaves the stored value `VAR_UNLOCKED`, through `tv_copy` on
    // one branch and by hand on the other; the lock is the item's, so the
    // store clears it here.  Written through the field's address rather than
    // a borrow of the item, for the reason `cur` is.
    unsafe { *di_lock(di) = VarLock::Unlocked };

    if watched {
        let key = unsafe { (*di).di_key.as_ptr() };
        unsafe {
            dict_watcher_notify(
                &::core::mem::ManuallyDrop::new(
                    DictRef::owning(dict).expect("a watched dictionary"),
                ),
                ::core::ffi::CStr::from_ptr(key),
                Some(&*cur),
                Some(&oldtv),
            )
        };
        clear_local(&mut oldtv);
    }

    if is_const {
        // Like `:lockvar! name`: lock the value and what it contains,
        // but only where the reference count is one, so that only
        // literal values are locked.
        unsafe { tv_item_lock(di_lock(di), &mut *cur, DICT_MAXNEST, true, true) };
    }
}

/// [`set_var_const`] of the variable `name` spells, which need not be
/// terminated where it ends.
pub(crate) fn set_var_const_named(name: &[u8], tv: &mut TypVal, copy: bool, is_const: bool) {
    // SAFETY: `name` names its own bytes, which is all the callee reads.
    unsafe { set_var_const(name.as_ptr().cast(), name.len(), tv, copy, is_const) }
}

/// [`var_check_ro`] naming the variable `name`, measured.
pub(crate) fn var_check_ro_named(flags: c_int, name: &[u8]) -> bool {
    let error_message = if flags & DI_FLAGS_RO as c_int != 0 {
        e_cannot_change_readonly_variable_str
    } else if flags & DI_FLAGS_RO_SBX as c_int != 0 && sandbox.get() != 0 {
        e_cannot_set_variable_in_sandbox_str
    } else {
        return false;
    };
    let len = crate::narrow::len_as_int(name.len());
    emsg_text(tr_plural!(gettext(error_message), len, msg_bytes(name)));
    true
}

/// [`var_check_fixed`] naming the variable `name`, measured.
pub(crate) fn var_check_fixed_named(flags: c_int, name: &[u8]) -> bool {
    if flags & DI_FLAGS_FIX as c_int == 0 {
        return false;
    }
    let name = msg_bytes(name);
    semsg!("E795: Cannot delete variable {name}");
    true
}

/// [`var_check_lock`] naming the variable `name`, measured.
pub(crate) fn var_check_lock_named(flags: c_int, name: &[u8]) -> bool {
    if flags & DI_FLAGS_LOCK as c_int == 0 {
        return false;
    }
    let name = msg_bytes(name);
    semsg!("E1122: Variable is locked: {name}");
    true
}

/// [`var_wrong_func_name`] of the name `name` spells.
pub(crate) fn var_wrong_func_name_named(name: &[u8], new_var: bool) -> bool {
    // SAFETY: `c` is NUL-terminated.
    cstr::with_terminated(name, |c| unsafe {
        var_wrong_func_name(c.as_ptr(), new_var)
    })
}

/// [`valid_varname`] of the name `name` spells.
pub(crate) fn valid_varname_named(name: &[u8]) -> bool {
    // SAFETY: `c` is NUL-terminated.
    cstr::with_terminated(name, |c| unsafe { valid_varname(c.as_ptr()) })
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
        e_cannot_change_readonly_variable_str.as_ptr()
    } else if flags & DI_FLAGS_RO_SBX as c_int != 0 && sandbox.get() != 0 {
        e_cannot_set_variable_in_sandbox_str.as_ptr()
    } else {
        return false;
    };

    if name_len == TV_TRANSLATE as size_t {
        name = unsafe { gettext_ptr(name).as_ptr() };
        name_len = unsafe { cstr::bytes_at(name) }.len();
    } else if name_len == TV_CSTRING as size_t {
        name_len = unsafe { cstr::bytes_at(name) }.len();
    }
    // SAFETY: `error_message` is one of the module's statics, and `name` is
    // readable for `name_len` bytes.
    let (error_message, shown) = unsafe { (gettext_ptr(error_message), c_str_len(name, name_len)) };
    emsg_text(tr_plural!(error_message, name_len as c_int, shown));
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
        name = unsafe { gettext_ptr(name).as_ptr() };
        name_len = unsafe { cstr::bytes_at(name) }.len();
    } else if name_len == TV_CSTRING as size_t {
        name_len = unsafe { cstr::bytes_at(name) }.len();
    }
    // SAFETY: a message argument the caller holds as a NUL-terminated string.
    let name = unsafe { c_str_len(name, name_len) };
    semsg!("E1122: Variable is locked: {name}");
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
        name = unsafe { gettext_ptr(name).as_ptr() };
        name_len = unsafe { cstr::bytes_at(name) }.len();
    } else if name_len == TV_CSTRING as size_t {
        name_len = unsafe { cstr::bytes_at(name) }.len();
    }
    // SAFETY: a message argument the caller holds as a NUL-terminated string.
    let name = unsafe { c_str_len(name, name_len) };
    semsg!("E795: Cannot delete variable {name}");
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
        && !has_char(unsafe { cstr::at(name) }, b'#' as c_int)
    {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let name = unsafe { c_str(name) };
        semsg!("E704: Funcref variable name must start with a capital: {name}");
        return true;
    }
    // Don't allow hiding a function. With an existing variable this may
    // be assigning another function to the same one, whose type the
    // caller checks.
    // SAFETY: the caller's NUL-terminated name.
    if new_var && function_exists(unsafe { cstr::bytes_at(name) }, false) {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let name = unsafe { c_str(name) };
        semsg!("E705: Variable name conflicts with existing function: {name}");
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
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let varname = unsafe { c_str(varname) };
            semsg!("E461: Illegal variable name: {varname}");
            return false;
        }
        p = unsafe { p.add(1) };
    }
    true
}
