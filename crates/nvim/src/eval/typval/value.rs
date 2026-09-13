//! Whole-`TypVal` operations: clear, copy, compare, lock.
//!
//! [`tv_clear`] releases whatever a value holds and leaves `VAR_UNKNOWN`
//! behind; it hands a self-referencing container to the deep-free walk in
//! [`super::nothing`] rather than recursing.  [`tv_copy`] is the shallow
//! copy, [`tv_equal`] the recursion-limited structural comparison, and
//! [`tv_item_lock`] is `:lockvar`, which walks into containers to the depth
//! it is given.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
// The globals here keep upstream's spelling; upper-casing them is a per-module rewrite.
#![allow(non_upper_case_globals)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::cstr;
use crate::guard::Depth;
use crate::message_fmt::{c_str_len, emsg_text};
use crate::os::cshim::gettext_ptr;
use crate::semsg;
use crate::tr_plural;

/// `TV_TRANSLATE`: the `name_len` sentinel that asks a lock error to run the
/// name through `gettext` and measure it itself.
const TV_TRANSLATE: size_t = size_t::MAX;
/// `TV_CSTRING`: the `name_len` sentinel that asks it to measure the name.
const TV_CSTRING: size_t = size_t::MAX - 1;

/// Release whatever `tv` holds, leaving the **empty value of its own kind**.
///
/// The work is done by the `nothing` sink, the seventh instantiation of
/// `typval_encode.c.h`: it walks the value iteratively, so a container that
/// refers to itself is deep-freed without recursing.
///
/// The kind survives, as upstream's does: a cleared String is a
/// `VAR_STRING` over NULL, a cleared List a `VAR_LIST` over NULL. Several
/// places read the kind back afterwards -- `filter()` checks the callback's
/// answer *after* clearing it, `1.234 - 8` decides on float arithmetic from
/// the tag the clear left behind -- and none of them is visible to a static
/// check.
///
/// Clearing an already-cleared value is free: [`TypVal::is_empty`] is the
/// fast path, and it is the one that matters, because with `Drop` live an
/// explicit `tv_clear` followed by the value leaving scope is the ordinary
/// case.
pub fn tv_clear(tv: &mut TypVal) {
    if tv.is_empty() {
        return;
    }

    // WARNING: do not translate the string here, gettext is slow and this
    // function is used *very* often. At the current state
    // `encode_vim_to_nothing` does not error out and does not use the
    // argument anywhere.
    //
    // If that changes and the argument starts being used, translate it
    // where it is used.
    let evn_ret = encode_vim_to_nothing(tv, c"tv_clear() argument");
    debug_assert!(evn_ret);
}

/// Release what `tv` holds and free the `TypVal` itself.
///
/// Unlike [`tv_clear`] this does not recurse into a container: it drops one
/// reference and frees the box.
///
/// `None` is a no-op, which is what the callers that free the answer of a
/// failed evaluation need.
///
/// # Safety
///
/// `tv` must be an initialized typval, unaliased for the call, in an
/// allocation of its own.
pub unsafe fn tv_free(tv: Option<&mut TypVal>) {
    let Some(tv) = tv else { return };
    match tv.v_type() {
        // SAFETY, for every arm: the caller's promise -- a live typval, so
        // the member the kind names is its own.
        VAR_PARTIAL => drop(tv.take_partial()),
        // FALLTHROUGH from VAR_FUNC into VAR_STRING: a funcref owns both a
        // reference to the function and the name string.
        VAR_FUNC | VAR_STRING => {
            if tv.v_type() == VAR_FUNC {
                unsafe { func_unref(tv.func_name_or_null()) };
            }
            unsafe { xfree(tv.string_or_func_name().cast()) };
        }
        VAR_BLOB => drop(tv.take_blob()),
        VAR_LIST => drop(tv.take_list()),
        VAR_DICT => drop(tv.take_dict()),
        _ => {}
    }
    // SAFETY: the caller's promise -- the box is theirs to free.
    unsafe { xfree(::core::ptr::from_mut(tv).cast()) };
}

impl Clone for TypVal {
    /// A shallow copy: the string is duplicated, and a container gains a
    /// reference rather than being copied.
    ///
    /// This *is* `tv_copy`. `deepcopy()` is `var_item_copy`, which walks.
    ///
    /// The lock does not come along, because there is none to come: a lock
    /// belongs to the slot a value sits in, and the copy is going somewhere
    /// else.
    fn clone(&self) -> TypVal {
        match *self {
            TypVal::String(text) if !text.is_null() => {
                // SAFETY: the variant says the payload is a live
                // NUL-terminated string.
                TypVal::String(unsafe { xstrdup(text) })
            }
            TypVal::Func(name) if !name.is_null() => {
                // SAFETY: as above -- a funcref's payload is its name.
                let copy = unsafe { xstrdup(name) };
                // SAFETY: the name just copied; a funcref owns a reference
                // to the function as well as the text.
                unsafe { func_ref(copy) };
                TypVal::Func(copy)
            }
            // As `List`: the handle's own `Clone` is the reference.
            TypVal::Partial(ref pt) => TypVal::partial((**pt).clone()),
            // As `List`: the handle's own `Clone` is the reference.
            TypVal::Blob(ref blob) => TypVal::blob((**blob).clone()),
            // The handle's own `Clone` is the reference: one more owner of
            // the same list, and `v:_null_list` counts nothing.
            TypVal::List(ref list) => TypVal::list((**list).clone()),
            // As `List`: the handle's own `Clone` is the reference.
            TypVal::Dict(ref dict) => TypVal::dict((**dict).clone()),
            TypVal::Unknown => {
                let arg0 = "tv_copy(UNKNOWN)";
                semsg!("E685: Internal error: {arg0}");
                TypVal::Unknown
            }
            // A scalar, and the two container variants over NULL: nothing
            // to duplicate and no reference to take.
            // SAFETY: the arms above cover everything that owns anything,
            // so what is left holds no reference this could duplicate.
            ref other => unsafe { other.bit_copy() },
        }
    }
}

impl Drop for TypVal {
    /// Release whatever this value holds; this *is* `tv_clear`.
    ///
    /// The refcount is the ownership now: dropping a `TypVal::List` gives up
    /// one reference to the list, and the last one frees it. What it is not
    /// is a *deep* free by recursion -- see [`tv_clear`].
    fn drop(&mut self) {
        // The fast path is *here* rather than only inside `tv_clear`: an
        // implicit drop is the commonest operation the interpreter has, and
        // most of them are of a scalar or of a slot something already took
        // the value out of. Testing before the call is what keeps this the
        // whole of the glue -- a jump table, a compare and a tail call --
        // and small enough that most of the sites that drop a value inline
        // it instead of calling it (796 out-of-line calls became 255).
        if !self.is_empty() {
            tv_clear(&mut *self);
        }
        // And nothing after it: the container payloads are held in a
        // `ManuallyDrop`, so the compiler appends no field glue to this and
        // the clear above is the only release path. See [`TypVal`].
    }
}

/// Copy `from` into `to`, taking a reference to whatever it holds.
///
/// The raw-pointer spelling of [`Clone`]: the destination is **overwritten**,
/// not assigned, because half the callers hand this a fresh `xmalloc`'d list
/// item and the other half have just cleared the slot.
pub fn tv_copy(from: &TypVal, to: &mut TypVal) {
    // SAFETY: the caller's promise: a live source and writable storage that
    // owes nothing, so the old bits are overwritten rather than released.
    let copy = (*from).clone();
    unsafe { ::core::ptr::write(to, copy) };
}

/// `:lockvar` / `:unlockvar` over the slot `slot_lock`/`tv` name, descending
/// `deep` levels into containers (negative meaning all the way down).
///
/// `slot_lock` is the lock of the place the value sits in — a list item's,
/// a dictionary item's — because that is what `:lockvar l[0]` locks: the
/// slot, not whatever value is in it today.  The containers below it carry
/// their own (`lv_lock`, `dv_lock`, `bv_lock`), which is what the descent
/// writes.
///
/// With `check_refcount`, a container held by more than one reference is left
/// alone — that is what keeps `:lockvar` on a function argument from locking
/// the caller's value.
///
/// # Safety
///
/// `tv` must point at an initialized typval and `slot_lock` at the lock of
/// the slot holding it; both unaliased for the call.
pub unsafe fn tv_item_lock(
    slot_lock: *mut VarLock,
    tv: &mut TypVal,
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

    // lock/unlock the slot itself
    unsafe { *slot_lock = (*slot_lock).changed(lock) };

    // SAFETY: the caller's promise: a live typval.
    let val = unsafe { Tv::new(tv) };
    match val.v_type() {
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
                    for li in list_iter_mut(unsafe { l.as_mut() }) {
                        let (lock_of, value) = (&raw mut li.li_lock, &raw mut li.li_tv);
                        unsafe {
                            tv_item_lock(lock_of, &mut *value, deep - 1, lock, check_refcount)
                        };
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
                        let di = tv_dict_hi2di(hi);
                        let (lock_of, value) = (di_lock(di), di_tv(di));
                        unsafe {
                            tv_item_lock(lock_of, &mut *value, deep - 1, lock, check_refcount)
                        };
                    }
                }
            }
        }
        VAR_UNKNOWN => unsafe { abort() },
        _ => {}
    }
}

/// Whether the slot `slot_lock`/`tv` names is locked, either as a slot or as
/// the container it holds.
pub fn tv_islocked(slot_lock: VarLock, tv: &TypVal) -> bool {
    let val = tv;
    let container_lock = match val.v_type() {
        VAR_LIST => list_locked((*tv).list_ref()),
        VAR_DICT => {
            unsafe { (*tv).dict_or_null().as_ref() }.map_or(VarLock::Unlocked, |d| d.dv_lock)
        }
        _ => VarLock::Unlocked,
    };
    slot_lock == VarLock::Locked || container_lock == VarLock::Locked
}

/// Whether the slot `slot_lock`/`tv` names may not be changed, raising the
/// matching error if so.
///
/// `name` is what the error names; `name_len` may be `TV_TRANSLATE` or
/// `TV_CSTRING` instead of a real length.
///
/// # Safety
///
/// `tv` must point at an initialized typval, and `slot_lock` be the lock of
/// the slot holding it. `name` must be null, or point at
/// the name the error reports — NUL-terminated for
/// `TV_CSTRING`/`TV_TRANSLATE`, otherwise `name_len` readable bytes.
pub unsafe extern "C" fn tv_check_lock(
    slot_lock: VarLock,
    tv: &TypVal,
    name: *const ::core::ffi::c_char,
    name_len: size_t,
) -> bool {
    let val = tv;
    let lock = match val.v_type() {
        // SAFETY (all three arms): the caller's live typval, whose kind says
        // which container it holds.
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
    (unsafe { value_check_lock(slot_lock, name, name_len) })
        || (lock.is_locked() && unsafe { value_check_lock(lock, name, name_len) })
}

/// Whether `lock` forbids a change, raising the matching error if so.
///
/// # Safety
///
/// `name` must be null, or point at the name the error reports — NUL-
/// terminated for `TV_CSTRING`/`TV_TRANSLATE`, otherwise `name_len` readable
/// bytes.
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
        if name_len == TV_TRANSLATE {
            name = unsafe { gettext_ptr(name) }.as_ptr();
            name_len = unsafe { cstr::bytes_at(name) }.len();
        } else if name_len == TV_CSTRING {
            name_len = unsafe { cstr::bytes_at(name) }.len();
        }
        // SAFETY: `name` is readable for `name_len` bytes.
        let shown = unsafe { c_str_len(name, name_len) };
        emsg_text(tr_plural!(
            error_message,
            crate::narrow::len_as_int(name_len),
            shown
        ));
    }

    true
}

/// Whether `tv1` and `tv2` are equal, `ic` ignoring case in strings.
///
/// Containers are compared structurally.  Two values of different types are
/// never equal, except that a funcref and a partial may be.
pub fn tv_equal(tv1: &TypVal, tv2: &TypVal, ic: bool) -> bool {
    // TODO(ZyX-I): Make this not recursive
    static recursive_cnt: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

    if !((*tv1).is_func() && (*tv2).is_func()) && (*tv1).v_type() != (*tv2).v_type() {
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
    let (a, b) = (tv1, tv2);
    match a.v_type() {
        VAR_LIST => {
            let _recursing = Depth::of(&recursive_cnt);
            list_equal((*tv1).list_ref(), (*tv2).list_ref(), ic)
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
            func_equal(tv1, tv2, ic)
        }
        VAR_BLOB => blob_equal(tv1.blob_ref(), tv2.blob_ref()),
        VAR_NUMBER => a.as_number() == b.as_number(),
        VAR_FLOAT => a.as_float() == b.as_float(),
        VAR_STRING => {
            let mut buf1 = NumBuf::new();
            let mut buf2 = NumBuf::new();
            let s1 = buf1.string_ptr(tv1);
            let s2 = buf2.string_ptr(tv2);
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
