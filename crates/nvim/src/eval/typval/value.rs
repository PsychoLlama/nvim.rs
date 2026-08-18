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
use crate::semsg_c;

/// Release whatever `tv` holds and leave `VAR_UNKNOWN` behind.
///
/// The work is done by the `nothing` sink, the seventh instantiation of
/// `typval_encode.c.h`: it walks the value iteratively, so a container that
/// refers to itself is deep-freed without recursing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_clear(tv: *mut typval_T) {
    unsafe {
        if tv.is_null() || (*tv).v_type == VAR_UNKNOWN {
            return;
        }

        // WARNING: do not translate the string here, gettext is slow and this
        // function is used *very* often. At the current state
        // `encode_vim_to_nothing` does not error out and does not use the
        // argument anywhere.
        //
        // If that changes and the argument starts being used, translate it
        // where it is used.
        let evn_ret = encode_vim_to_nothing(tv, c"tv_clear() argument".as_ptr());
        debug_assert!(evn_ret);
    }
}

/// Release what `tv` holds and free the `typval_T` itself.
///
/// Unlike [`tv_clear`] this does not recurse into a container: it drops one
/// reference and frees the box.
pub unsafe fn tv_free(tv: *mut typval_T) {
    unsafe {
        if tv.is_null() {
            return;
        }

        match (*tv).v_type {
            VAR_PARTIAL => partial_unref((*tv).vval.v_partial),
            // FALLTHROUGH from VAR_FUNC into VAR_STRING: a funcref owns both a
            // reference to the function and the name string.
            VAR_FUNC | VAR_STRING => {
                if (*tv).v_type == VAR_FUNC {
                    func_unref((*tv).vval.v_string);
                }
                xfree((*tv).vval.v_string.cast());
            }
            VAR_BLOB => tv_blob_unref((*tv).vval.v_blob),
            VAR_LIST => tv_list_unref((*tv).vval.v_list),
            VAR_DICT => tv_dict_unref((*tv).vval.v_dict),
            _ => {}
        }
        xfree(tv.cast());
    }
}

/// Copy `from` into `to`, taking a reference to whatever it holds.
///
/// The copy is shallow and always unlocked; `deepcopy()` goes through
/// `var_item_copy`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_copy(from: *const typval_T, to: *mut typval_T) {
    unsafe {
        (*to).v_type = (*from).v_type;
        (*to).v_lock = VAR_UNLOCKED;
        (*to).vval = (*from).vval;

        match (*from).v_type {
            VAR_STRING | VAR_FUNC => {
                if !(*from).vval.v_string.is_null() {
                    (*to).vval.v_string = xstrdup((*from).vval.v_string);
                    if (*from).v_type == VAR_FUNC {
                        func_ref((*to).vval.v_string);
                    }
                }
            }
            VAR_PARTIAL => {
                if let Some(pt) = (*to).vval.v_partial.as_mut() {
                    pt.pt_refcount += 1;
                }
            }
            VAR_BLOB => {
                if !(*from).vval.v_blob.is_null() {
                    (*(*to).vval.v_blob).bv_refcount += 1;
                }
            }
            VAR_LIST => tv_list_ref((*to).vval.v_list),
            VAR_DICT => {
                if !(*from).vval.v_dict.is_null() {
                    (*(*to).vval.v_dict).dv_refcount += 1;
                }
            }
            VAR_UNKNOWN => {
                semsg_c!(
                    gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                    c"tv_copy(UNKNOWN)".as_ptr(),
                );
            }
            _ => {}
        }
    }
}

/// Upstream's `CHANGE_LOCK`: what a lock status becomes under `:lockvar` /
/// `:unlockvar`.
///
/// `VAR_FIXED` never changes — that is a slot that cannot be unlocked at all,
/// not merely one that is locked.  c2rust renders the macro's designated-index
/// array literal at each of its four use sites.
#[inline]
fn change_lock(lock: bool, var: VarLockStatus) -> VarLockStatus {
    match var {
        VAR_FIXED => VAR_FIXED,
        _ if lock => VAR_LOCKED,
        _ => VAR_UNLOCKED,
    }
}

/// `:lockvar` / `:unlockvar` over `tv`, descending `deep` levels into
/// containers (negative meaning all the way down).
///
/// With `check_refcount`, a container held by more than one reference is left
/// alone — that is what keeps `:lockvar` on a function argument from locking
/// the caller's value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_item_lock(
    tv: *mut typval_T,
    deep: ::core::ffi::c_int,
    lock: bool,
    check_refcount: bool,
) {
    unsafe {
        // TODO(ZyX-I): Make this not recursive
        static recurse: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

        if recurse.get() >= DICT_MAXNEST {
            emsg(gettext(e_variable_nested_too_deep_for_unlock.as_ptr()));
            return;
        }
        if deep == 0 {
            return;
        }
        (*recurse.ptr()) += 1;

        // lock/unlock the item itself
        (*tv).v_lock = change_lock(lock, (*tv).v_lock);

        match (*tv).v_type {
            VAR_BLOB => {
                let b = (*tv).vval.v_blob;
                if !b.is_null() && !(check_refcount && (*b).bv_refcount > 1) {
                    (*b).bv_lock = change_lock(lock, (*b).bv_lock);
                }
            }
            VAR_LIST => {
                let l = (*tv).vval.v_list;
                if !l.is_null() && !(check_refcount && (*l).lv_refcount > 1) {
                    (*l).lv_lock = change_lock(lock, (*l).lv_lock);
                    if deep < 0 || deep > 1 {
                        // Recursive: lock/unlock the items the List contains.
                        for li in tv_list_iter(l.as_ref()) {
                            tv_item_lock(&raw mut (*li).li_tv, deep - 1, lock, check_refcount);
                        }
                    }
                }
            }
            VAR_DICT => {
                let d = (*tv).vval.v_dict;
                if !d.is_null() && !(check_refcount && (*d).dv_refcount > 1) {
                    (*d).dv_lock = change_lock(lock, (*d).dv_lock);
                    if deep < 0 || deep > 1 {
                        // recursive: lock/unlock the items the Dict contains
                        for hi in tv_dict_iter(&*d) {
                            let di = tv_dict_hi2di(hi);
                            tv_item_lock(&raw mut (*di).di_tv, deep - 1, lock, check_refcount);
                        }
                    }
                }
            }
            VAR_UNKNOWN => abort(),
            _ => {}
        }

        (*recurse.ptr()) -= 1;
    }
}

/// Whether `tv` is locked, either itself or as the container it names.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_islocked(tv: *const typval_T) -> bool {
    unsafe {
        (*tv).v_lock == VAR_LOCKED
            || ((*tv).v_type == VAR_LIST && tv_list_locked((*tv).vval.v_list) == VAR_LOCKED)
            || ((*tv).v_type == VAR_DICT
                && !(*tv).vval.v_dict.is_null()
                && (*(*tv).vval.v_dict).dv_lock == VAR_LOCKED)
    }
}

/// Whether `tv` may not be changed, raising the matching error if so.
///
/// `name` is what the error names; `name_len` may be `TV_TRANSLATE` or
/// `TV_CSTRING` instead of a real length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_check_lock(
    tv: *const typval_T,
    name: *const ::core::ffi::c_char,
    name_len: size_t,
) -> bool {
    unsafe {
        let lock = match (*tv).v_type {
            VAR_BLOB => (*tv)
                .vval
                .v_blob
                .as_ref()
                .map_or(VAR_UNLOCKED, |b| b.bv_lock),
            VAR_LIST => (*tv)
                .vval
                .v_list
                .as_ref()
                .map_or(VAR_UNLOCKED, |l| l.lv_lock),
            VAR_DICT => (*tv)
                .vval
                .v_dict
                .as_ref()
                .map_or(VAR_UNLOCKED, |d| d.dv_lock),
            _ => VAR_UNLOCKED,
        };
        value_check_lock((*tv).v_lock, name, name_len)
            || (lock != VAR_UNLOCKED && value_check_lock(lock, name, name_len))
    }
}

/// Whether `lock` forbids a change, raising the matching error if so.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn value_check_lock(
    lock: VarLockStatus,
    mut name: *const ::core::ffi::c_char,
    mut name_len: size_t,
) -> bool {
    unsafe {
        // The two `_` arms are `VAR_FIXED`, the only status left. Upstream
        // asserts the message was set, which is vacuous once the arms are
        // exhaustive.
        let error_message = match (lock, name.is_null()) {
            (VAR_UNLOCKED, _) => return false,
            (VAR_LOCKED, true) => &raw const e_value_is_locked as *const ::core::ffi::c_char,
            (VAR_LOCKED, false) => &raw const e_value_is_locked_str as *const ::core::ffi::c_char,
            (_, true) => &raw const e_cannot_change_value as *const ::core::ffi::c_char,
            (_, false) => &raw const e_cannot_change_value_of_str as *const ::core::ffi::c_char,
        };

        if name.is_null() {
            emsg(gettext(error_message));
        } else {
            if name_len == TV_TRANSLATE as size_t {
                name = gettext(name);
                name_len = strlen(name);
            } else if name_len == TV_CSTRING as size_t {
                name_len = strlen(name);
            }
            semsg_c!(gettext(error_message), name_len as ::core::ffi::c_int, name);
        }

        true
    }
}

/// Whether `tv1` and `tv2` are equal, `ic` ignoring case in strings.
///
/// Containers are compared structurally.  Two values of different types are
/// never equal, except that a funcref and a partial may be.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_equal(tv1: *mut typval_T, tv2: *mut typval_T, ic: bool) -> bool {
    unsafe {
        // TODO(ZyX-I): Make this not recursive
        static recursive_cnt: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

        if !(tv_is_func(*tv1) && tv_is_func(*tv2)) && (*tv1).v_type != (*tv2).v_type {
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
            (*tv_equal_recurse_limit.ptr()) -= 1;
            return true;
        }

        // The three container arms bracket their call with the depth counter.
        // Written out rather than folded into a helper taking a closure: this
        // runs once per item of a list or dict comparison, and a `dyn FnMut`
        // there would be an indirect call on a measured phase.
        match (*tv1).v_type {
            VAR_LIST => {
                (*recursive_cnt.ptr()) += 1;
                let r = tv_list_equal((*tv1).vval.v_list, (*tv2).vval.v_list, ic);
                (*recursive_cnt.ptr()) -= 1;
                r
            }
            VAR_DICT => {
                (*recursive_cnt.ptr()) += 1;
                let r = tv_dict_equal((*tv1).vval.v_dict, (*tv2).vval.v_dict, ic);
                (*recursive_cnt.ptr()) -= 1;
                r
            }
            VAR_PARTIAL | VAR_FUNC => {
                if ((*tv1).v_type == VAR_PARTIAL && (*tv1).vval.v_partial.is_null())
                    || ((*tv2).v_type == VAR_PARTIAL && (*tv2).vval.v_partial.is_null())
                {
                    return false;
                }
                (*recursive_cnt.ptr()) += 1;
                let r = func_equal(tv1, tv2, ic);
                (*recursive_cnt.ptr()) -= 1;
                r
            }
            VAR_BLOB => tv_blob_equal((*tv1).vval.v_blob, (*tv2).vval.v_blob),
            VAR_NUMBER => (*tv1).vval.v_number == (*tv2).vval.v_number,
            VAR_FLOAT => (*tv1).vval.v_float == (*tv2).vval.v_float,
            VAR_STRING => {
                let mut buf1: [::core::ffi::c_char; 65] = [0; 65];
                let mut buf2: [::core::ffi::c_char; 65] = [0; 65];
                let s1 = tv_get_string_buf(tv1, buf1.as_mut_ptr());
                let s2 = tv_get_string_buf(tv2, buf2.as_mut_ptr());
                mb_strcmp_ic(ic, s1, s2) == 0
            }
            VAR_BOOL => (*tv1).vval.v_bool == (*tv2).vval.v_bool,
            VAR_SPECIAL => (*tv1).vval.v_special == (*tv2).vval.v_special,
            // VAR_UNKNOWN can be the result of an invalid expression, let's say
            // it does not equal anything, not even self.
            VAR_UNKNOWN => false,
            _ => abort(),
        }
    }
}
