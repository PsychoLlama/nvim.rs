//! Whole-`typval_T` operations: clear, copy, compare, lock.
//!
//! [`tv_clear`] releases whatever a value holds and leaves `VAR_UNKNOWN`
//! behind; it hands a self-referencing container to the deep-free walk in
//! [`super::nothing`] rather than recursing.  [`tv_copy`] is the shallow
//! copy, [`tv_equal`] the recursion-limited structural comparison, and
//! [`tv_item_lock`] is `:lockvar`, which walks into containers to the depth
//! it is given.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::guard::Depth;
use crate::message_fmt::{c_str_len, emsg_text};
use crate::os::cshim::gettext_ptr;
use crate::semsg;
use crate::tr_plural;

/// Release whatever `tv` holds and leave `VAR_UNKNOWN` behind.
///
/// The work is done by the `nothing` sink, the seventh instantiation of
/// `typval_encode.c.h`: it walks the value iteratively, so a container that
/// refers to itself is deep-freed without recursing.
pub unsafe fn tv_clear(tv: *mut typval_T) {
    if tv.is_null() || unsafe { (*tv).v_type } == VAR_UNKNOWN {
        return;
    }

    // WARNING: do not translate the string here, gettext is slow and this
    // function is used *very* often. At the current state
    // `encode_vim_to_nothing` does not error out and does not use the
    // argument anywhere.
    //
    // If that changes and the argument starts being used, translate it
    // where it is used.
    let evn_ret = unsafe { encode_vim_to_nothing(tv, c"tv_clear() argument".as_ptr()) };
    debug_assert!(evn_ret);
}

/// Release what `tv` holds and free the `typval_T` itself.
///
/// Unlike [`tv_clear`] this does not recurse into a container: it drops one
/// reference and frees the box.
pub unsafe fn tv_free(tv: *mut typval_T) {
    if tv.is_null() {
        return;
    }

    // SAFETY: the caller's promise: a live typval.
    let val = unsafe { Tv::new(tv) };
    match val.v_type {
        VAR_PARTIAL => unsafe { partial_unref((*tv).partial_or_null()) },
        // FALLTHROUGH from VAR_FUNC into VAR_STRING: a funcref owns both a
        // reference to the function and the name string.
        VAR_FUNC | VAR_STRING => {
            if val.v_type == VAR_FUNC {
                unsafe { func_unref((*tv).func_name_or_null()) };
            }
            unsafe { xfree((*tv).string_or_func_name().cast()) };
        }
        VAR_BLOB => unsafe { tv_blob_unref((*tv).blob_or_null()) },
        VAR_LIST => unsafe { tv_list_unref((*tv).list_or_null()) },
        VAR_DICT => unsafe { tv_dict_unref((*tv).dict_or_null()) },
        _ => {}
    }
    unsafe { xfree(tv.cast()) };
}

/// Copy `from` into `to`, taking a reference to whatever it holds.
///
/// The copy is shallow and always unlocked; `deepcopy()` goes through
/// `var_item_copy`.
pub unsafe fn tv_copy(from: *const typval_T, to: *mut typval_T) {
    unsafe { (*to).v_type = (*from).v_type };
    // SAFETY: the caller's promise: a writable typval.
    let mut dst = unsafe { Tv::new(to) };
    dst.v_lock = VarLock::Unlocked;
    unsafe { (*to).vval = (*from).vval };

    // SAFETY: the caller's promise: a live source typval.
    let src = unsafe { Tv::new(from.cast_mut()) };
    match src.v_type {
        VAR_STRING | VAR_FUNC => {
            if !src.string_or_func_name().is_null() {
                unsafe { (*to).vval.v_string = xstrdup((*from).string_or_func_name()) };
                if src.v_type == VAR_FUNC {
                    unsafe { func_ref((*to).string_or_func_name()) };
                }
            }
        }
        VAR_PARTIAL => {
            if let Some(pt) = unsafe { (*to).partial_or_null().as_mut() } {
                pt.pt_refcount.retain();
            }
        }
        VAR_BLOB => {
            if !src.blob_or_null().is_null() {
                unsafe { (*(*to).blob_or_null()).bv_refcount.retain() };
            }
        }
        VAR_LIST => unsafe { tv_list_ref((*to).list_or_null()) },
        VAR_DICT => {
            if !src.dict_or_null().is_null() {
                unsafe { (*(*to).dict_or_null()).dv_refcount.retain() };
            }
        }
        VAR_UNKNOWN => {
            let arg0 = "tv_copy(UNKNOWN)";
            semsg!("E685: Internal error: {arg0}");
        }
        _ => {}
    }
}

/// `:lockvar` / `:unlockvar` over `tv`, descending `deep` levels into
/// containers (negative meaning all the way down).
///
/// With `check_refcount`, a container held by more than one reference is left
/// alone — that is what keeps `:lockvar` on a function argument from locking
/// the caller's value.
pub unsafe fn tv_item_lock(
    tv: *mut typval_T,
    deep: ::core::ffi::c_int,
    lock: bool,
    check_refcount: bool,
) {
    // TODO(ZyX-I): Make this not recursive
    static recurse: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

    if recurse.get() >= DICT_MAXNEST {
        emsg(gettext(e_variable_nested_too_deep_for_unlock));
        return;
    }
    if deep == 0 {
        return;
    }
    let _recurse = Depth::of(&recurse);

    // lock/unlock the item itself
    unsafe { (*tv).v_lock = (*tv).v_lock.changed(lock) };

    // SAFETY: the caller's promise: a live typval.
    let val = unsafe { Tv::new(tv) };
    match val.v_type {
        VAR_BLOB => {
            let b = val.blob_or_null();
            // SAFETY: the typval's own blob.
            let bl = unsafe { Bl::new(b) };
            if !b.is_null() && !(check_refcount && bl.bv_refcount.is_shared()) {
                unsafe { (*b).bv_lock = (*b).bv_lock.changed(lock) };
            }
        }
        VAR_LIST => {
            let l = val.list_or_null();
            // SAFETY: the typval's own list.
            let ls = unsafe { Ls::new(l) };
            if !l.is_null() && !(check_refcount && ls.lv_refcount.is_shared()) {
                unsafe { (*l).lv_lock = (*l).lv_lock.changed(lock) };
                if !(0..=1).contains(&deep) {
                    // Recursive: lock/unlock the items the List contains.
                    for li in tv_list_iter(unsafe { l.as_ref() }) {
                        let item = li_tv(li);
                        unsafe { tv_item_lock(item, deep - 1, lock, check_refcount) };
                    }
                }
            }
        }
        VAR_DICT => {
            let d = val.dict_or_null();
            // SAFETY: the typval's own dictionary.
            let dt = unsafe { Dt::new(d) };
            if !d.is_null() && !(check_refcount && dt.dv_refcount.is_shared()) {
                unsafe { (*d).dv_lock = (*d).dv_lock.changed(lock) };
                if !(0..=1).contains(&deep) {
                    // recursive: lock/unlock the items the Dict contains
                    for hi in unsafe { tv_dict_iter(d) } {
                        let di = unsafe { tv_dict_hi2di(hi) };
                        let item = di_tv(di);
                        unsafe { tv_item_lock(item, deep - 1, lock, check_refcount) };
                    }
                }
            }
        }
        VAR_UNKNOWN => unsafe { abort() },
        _ => {}
    }
}

/// Whether `tv` is locked, either itself or as the container it names.
pub unsafe fn tv_islocked(tv: *const typval_T) -> bool {
    // SAFETY: the caller's promise: a live typval.
    let val = unsafe { Tv::new(tv.cast_mut()) };
    let (v_lock, v_type) = (val.v_lock, val.v_type);
    let container_lock = match v_type {
        VAR_LIST => unsafe { tv_list_locked((*tv).list_or_null()) },
        VAR_DICT => {
            unsafe { (*tv).dict_or_null().as_ref() }.map_or(VarLock::Unlocked, |d| d.dv_lock)
        }
        _ => VarLock::Unlocked,
    };
    v_lock == VarLock::Locked || container_lock == VarLock::Locked
}

/// Whether `tv` may not be changed, raising the matching error if so.
///
/// `name` is what the error names; `name_len` may be `TV_TRANSLATE` or
/// `TV_CSTRING` instead of a real length.
pub unsafe extern "C" fn tv_check_lock(
    tv: *const typval_T,
    name: *const ::core::ffi::c_char,
    name_len: size_t,
) -> bool {
    // SAFETY: the caller's promise: a live typval.
    let val = unsafe { Tv::new(tv.cast_mut()) };
    let lock = match val.v_type {
        // SAFETY: the caller's live typval, read through the union member
        // its own `v_type` selects.
        VAR_BLOB => {
            unsafe { (*tv).blob_or_null().as_ref() }.map_or(VarLock::Unlocked, |b| b.bv_lock)
        }
        VAR_LIST => {
            unsafe { (*tv).list_or_null().as_ref() }.map_or(VarLock::Unlocked, |l| l.lv_lock)
        }
        VAR_DICT => {
            unsafe { (*tv).dict_or_null().as_ref() }.map_or(VarLock::Unlocked, |d| d.dv_lock)
        }
        _ => VarLock::Unlocked,
    };
    (unsafe { value_check_lock((*tv).v_lock, name, name_len) })
        || (lock.is_locked() && unsafe { value_check_lock(lock, name, name_len) })
}

/// Whether `lock` forbids a change, raising the matching error if so.
pub unsafe fn value_check_lock(
    lock: VarLock,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    // Upstream asserts the message was set; with `VarLock` an enum the
    // match is exhaustive over three named states and the assertion is
    // the compiler's.
    let error_message = match (lock, name.is_null()) {
        (VarLock::Unlocked, _) => return false,
        (VarLock::Locked, true) => e_value_is_locked.as_ptr(),
        (VarLock::Locked, false) => e_value_is_locked_str.as_ptr(),
        (VarLock::Fixed, true) => e_cannot_change_value.as_ptr(),
        (VarLock::Fixed, false) => e_cannot_change_value_of_str.as_ptr(),
    };

    // SAFETY: `error_message` is one of the NUL-terminated statics chosen
    // just above.
    let error_message = unsafe { gettext_ptr(error_message) };
    if name.is_null() {
        emsg(error_message);
    } else {
        if name_len == TV_TRANSLATE as size_t {
            name = unsafe { gettext_ptr(name) }.as_ptr();
            name_len = unsafe { cstr::bytes_at(name) }.len();
        } else if name_len == TV_CSTRING as size_t {
            name_len = unsafe { cstr::bytes_at(name) }.len();
        }
        // SAFETY: `name` is readable for `name_len` bytes.
        let shown = unsafe { c_str_len(name, name_len) };
        emsg_text(tr_plural!(
            error_message,
            name_len as ::core::ffi::c_int,
            shown
        ));
    }

    true
}

/// Whether `tv1` and `tv2` are equal, `ic` ignoring case in strings.
///
/// Containers are compared structurally.  Two values of different types are
/// never equal, except that a funcref and a partial may be.
pub unsafe fn tv_equal(tv1: *mut typval_T, tv2: *mut typval_T, ic: bool) -> bool {
    // TODO(ZyX-I): Make this not recursive
    static recursive_cnt: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

    if !(tv_is_func(unsafe { *tv1 }) && tv_is_func(unsafe { *tv2 }))
        && unsafe { (*tv1).v_type } != unsafe { (*tv2).v_type }
    {
        return false;
    }

    // Catch lists and dicts that have an endless loop by limiting
    // recursiveness to a limit.  We guess they are equal then.
    // A fixed limit has the problem of still taking an awful long time.
    // Reduce the limit every time running into it. That should work fine for
    // deeply linked structures that are not recursively linked and catch
    // recursiveness quickly.
    if recursive_cnt.get() == 0 {
        tv_equal_recurse_limit.set(1000);
    }
    if recursive_cnt.get() >= tv_equal_recurse_limit.get() {
        tv_equal_recurse_limit.set(tv_equal_recurse_limit.get() - 1);
        return true;
    }

    // The three container arms bracket their call with the depth counter.
    // Written out rather than folded into a helper taking a closure: this
    // runs once per item of a list or dict comparison, and a `dyn FnMut`
    // there would be an indirect call on a measured phase. [`Depth`] costs
    // nothing extra -- it is the same two `set`s, moved onto the scope.
    // SAFETY: the caller's promise: two live typvals.
    let (a, b) = unsafe { (Tv::new(tv1), Tv::new(tv2)) };
    match a.v_type {
        VAR_LIST => {
            let _recursing = Depth::of(&recursive_cnt);
            unsafe { tv_list_equal((*tv1).list_or_null(), (*tv2).list_or_null(), ic) }
        }
        VAR_DICT => {
            let _recursing = Depth::of(&recursive_cnt);
            unsafe { tv_dict_equal((*tv1).dict_or_null(), (*tv2).dict_or_null(), ic) }
        }
        VAR_PARTIAL | VAR_FUNC => {
            if a.as_partial().is_some_and(|p| p.is_null())
                || b.as_partial().is_some_and(|p| p.is_null())
            {
                return false;
            }
            let _recursing = Depth::of(&recursive_cnt);
            unsafe { func_equal(tv1, tv2, ic) }
        }
        VAR_BLOB => unsafe { tv_blob_equal((*tv1).blob_or_null(), (*tv2).blob_or_null()) },
        VAR_NUMBER => a.as_number() == b.as_number(),
        VAR_FLOAT => a.as_float() == b.as_float(),
        VAR_STRING => {
            let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
            let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
            let s1 = unsafe { tv_get_string_buf(tv1, buf1.as_mut_ptr()) };
            let s2 = unsafe { tv_get_string_buf(tv2, buf2.as_mut_ptr()) };
            (unsafe { mb_strcmp_ic(ic, s1, s2) }) == 0
        }
        VAR_BOOL => a.as_bool() == b.as_bool(),
        VAR_SPECIAL => a.as_special() == b.as_special(),
        // VAR_UNKNOWN can be the result of an invalid expression, let's say
        // it does not equal anything, not even self.
        VAR_UNKNOWN => false,
        _ => unsafe { abort() },
    }
}
