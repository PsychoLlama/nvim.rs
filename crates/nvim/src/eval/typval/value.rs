//! Whole-`TypVal` operations: clear, copy, compare, lock.
//!
//! [`tv_clear`] releases whatever a value holds and leaves `VAR_UNKNOWN`
//! behind; it hands a self-referencing container to the deep-free walk in
//! [`super::nothing`] rather than recursing.  [`tv_copy`] is the shallow
//! copy, [`tv_equal`] the recursion-limited structural comparison, and
//! [`tv_item_lock`] is `:lockvar`, which walks into containers to the depth
//! it is given.

#![forbid(unsafe_code)]
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
use crate::eval::userfunc::func_ref_name;
use crate::guard::Depth;
use crate::mbyte::strnicmp_in;
use crate::message_fmt::{emsg_text, msg_bytes};
use crate::semsg;
use crate::tr_plural;
use ::core::ffi::CStr;

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
            TypVal::String(ref text) => TypVal::string((**text).clone()),
            TypVal::Func(ref name) if name.is_some() => {
                let copy = (**name).clone();
                if let Some(copy) = &copy {
                    // A funcref owns a reference to the function as well as
                    // the text.
                    func_ref_name(copy.as_cstr());
                }
                TypVal::func(copy)
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
            // A scalar, and a Funcref over NULL: nothing to duplicate and
            // no reference to take.
            TypVal::Func(_) => TypVal::func(None),
            TypVal::Number(n) => TypVal::Number(n),
            TypVal::Float(f) => TypVal::Float(f),
            TypVal::Bool(b) => TypVal::Bool(b),
            TypVal::Special(s) => TypVal::Special(s),
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
    // The old value is the caller's: overwritten, not released.
    to.overwrite(from.clone());
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
pub fn tv_item_lock(
    slot_lock: &mut VarLock,
    tv: &TypVal,
    deep: ::core::ffi::c_int,
    lock: bool,
    check_refcount: bool,
) {
    let Some(_depth) = item_lock_depth(deep) else {
        return;
    };
    if let Some(held) = lock_slot(slot_lock, tv, lock, check_refcount) {
        held.lock(deep, lock, check_refcount);
    }
}

/// The recursion counter of [`tv_item_lock`], one level deeper; `None` --
/// having reported `E743` past the limit -- when the descent stops here.
fn item_lock_depth(deep: ::core::ffi::c_int) -> Option<crate::guard::Bump> {
    // TODO(ZyX-I): Make this not recursive
    static recurse: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);

    if recurse.get() >= DICT_MAXNEST {
        emsg(gettext(e_variable_nested_too_deep_for_unlock));
        return None;
    }
    if deep == 0 {
        return None;
    }
    Some(Depth::of(&recurse))
}

/// A container [`tv_item_lock`] descends into, held for the descent.
enum Held {
    Blob(BlobRef),
    List(ListRef),
    Dict(DictRef),
}

/// Lock or unlock the slot itself, and answer the container in it that the
/// descent goes on into -- unless `check_refcount` says it is shared.
///
/// The container is answered as a handle of its own, so that the descent
/// holds no borrow of the slot, which may be an item of a container the
/// descent reaches again.
fn lock_slot(
    slot_lock: &mut VarLock,
    tv: &TypVal,
    lock: bool,
    check_refcount: bool,
) -> Option<Held> {
    *slot_lock = slot_lock.changed(lock);
    let skip = |count: &crate::types::Refcount| check_refcount && count.is_shared();
    match tv {
        TypVal::Blob(blob) => blob
            .as_ref()
            .filter(|b| !skip(&b.bv_refcount))
            .map(|b| Held::Blob(b.clone())),
        TypVal::List(list) => list
            .as_ref()
            .filter(|l| !skip(&l.lv_refcount))
            .map(|l| Held::List(l.clone())),
        TypVal::Dict(dict) => dict
            .as_ref()
            .filter(|d| !skip(&d.dv_refcount))
            .map(|d| Held::Dict(d.clone())),
        TypVal::Unknown => ::std::process::abort(),
        _ => None,
    }
}

impl Held {
    /// Lock or unlock the container, and its items below `deep`.
    fn lock(self, deep: ::core::ffi::c_int, lock: bool, check_refcount: bool) {
        let descend = !(0..=1).contains(&deep);
        match self {
            Held::Blob(blob) => {
                let blob = blob.edit();
                blob.bv_lock = blob.bv_lock.changed(lock);
            }
            Held::List(list) => {
                let this = list.edit();
                this.lv_lock = this.lv_lock.changed(lock);
                // Recursive: lock/unlock the items the List contains.
                for at in 0..if descend { list.len() } else { 0 } {
                    let Some(_depth) = item_lock_depth(deep - 1) else {
                        continue;
                    };
                    let item = &mut list.edit().items_mut()[at];
                    let held = lock_slot(&mut item.li_lock, &item.li_tv, lock, check_refcount);
                    if let Some(held) = held {
                        held.lock(deep - 1, lock, check_refcount);
                    }
                }
            }
            Held::Dict(dict) => {
                let this = dict.edit();
                this.dv_lock = this.dv_lock.changed(lock);
                if !descend {
                    return;
                }
                // Recursive: lock/unlock the items the Dict contains.
                let mut cursor = DictCursor::new(&dict);
                while let Some(slot) = cursor.next(&dict) {
                    let Some(_depth) = item_lock_depth(deep - 1) else {
                        continue;
                    };
                    let item = dict.edit().item_at_mut(slot).expect("a kept slot");
                    let held = lock_slot(&mut item.di_lock, &item.di_tv, lock, check_refcount);
                    if let Some(held) = held {
                        held.lock(deep - 1, lock, check_refcount);
                    }
                }
            }
        }
    }
}

/// Whether the slot `slot_lock`/`tv` names is locked, either as a slot or as
/// the container it holds.
pub fn tv_islocked(slot_lock: VarLock, tv: &TypVal) -> bool {
    let val = tv;
    let container_lock = match val.v_type() {
        VAR_LIST => list_locked((*tv).list_ref()),
        VAR_DICT => tv.dict_ref().map_or(VarLock::Unlocked, |d| d.dv_lock),
        _ => VarLock::Unlocked,
    };
    slot_lock == VarLock::Locked || container_lock == VarLock::Locked
}

/// What a lock error names: upstream's `name`/`name_len` pair, whose
/// `TV_TRANSLATE` and `TV_CSTRING` sentinel lengths become variants.
#[derive(Clone, Copy, Debug)]
pub enum LockName<'a> {
    /// No name: "E741: Value is locked".
    None,
    /// A message literal, translated before it is shown.
    Translate(&'static CStr),
    /// These bytes, as they are.
    Bytes(&'a [u8]),
}

impl TypVal {
    /// The lock of the container this value holds; a scalar or NULL
    /// container has none.
    fn container_lock(&self) -> VarLock {
        match self {
            TypVal::Blob(blob) => blob.as_ref().map_or(VarLock::Unlocked, |b| b.bv_lock),
            TypVal::List(list) => list.as_ref().map_or(VarLock::Unlocked, |l| l.lv_lock),
            TypVal::Dict(dict) => dict.as_ref().map_or(VarLock::Unlocked, |d| d.dv_lock),
            _ => VarLock::Unlocked,
        }
    }
}

/// Whether the slot `slot_lock`/`tv` names may not be changed, raising the
/// matching error if so: the slot's own lock first, then the container's.
pub fn tv_check_lock(slot_lock: VarLock, tv: &TypVal, name: LockName<'_>) -> bool {
    let lock = tv.container_lock();
    value_check_lock(slot_lock, name) || (lock.is_locked() && value_check_lock(lock, name))
}

/// Whether `lock` forbids a change, raising the matching error if so.
pub fn value_check_lock(lock: VarLock, name: LockName<'_>) -> bool {
    // Upstream asserts the message was set; with `VarLock` an enum the
    // match is exhaustive over three named states and the assertion is
    // the compiler's.
    let unnamed = matches!(name, LockName::None);
    let error_message = match (lock, unnamed) {
        (VarLock::Unlocked, _) => return false,
        (VarLock::Locked, true) => e_value_is_locked,
        (VarLock::Locked, false) => e_value_is_locked_str,
        (VarLock::Fixed, true) => e_cannot_change_value,
        (VarLock::Fixed, false) => e_cannot_change_value_of_str,
    };
    let error_message = gettext(error_message);
    let name = match name {
        LockName::None => {
            emsg(error_message);
            return true;
        }
        LockName::Translate(literal) => gettext(literal).to_bytes(),
        LockName::Bytes(bytes) => bytes,
    };
    emsg_text(tr_plural!(
        error_message,
        crate::narrow::len_as_int(name.len()),
        msg_bytes(name)
    ));
    true
}

/// [`value_check_lock`] naming the value with `name`, measured.
pub(crate) fn value_check_lock_named(lock: VarLock, name: &[u8]) -> bool {
    value_check_lock(lock, LockName::Bytes(name))
}

/// [`tv_check_lock`] naming the value with `name`, measured.
pub(crate) fn tv_check_lock_named(slot_lock: VarLock, tv: &TypVal, name: &[u8]) -> bool {
    tv_check_lock(slot_lock, tv, LockName::Bytes(name))
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
            dict_equal((*tv1).dict_ref(), (*tv2).dict_ref(), ic)
        }
        VAR_PARTIAL | VAR_FUNC => {
            if matches!(a, TypVal::Partial(pt) if pt.is_none())
                || matches!(b, TypVal::Partial(pt) if pt.is_none())
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
            let (s1, s2) = (tv1.string_bytes(), tv2.string_bytes());
            if ic {
                strnicmp_in(s1, s2) == 0
            } else {
                s1 == s2
            }
        }
        VAR_BOOL => a.as_bool() == b.as_bool(),
        VAR_SPECIAL => a.as_special() == b.as_special(),
        // VAR_UNKNOWN can be the result of an invalid expression, let's say
        // it does not equal anything, not even self.
        VAR_UNKNOWN => false,
        _ => ::std::process::abort(),
    }
}
