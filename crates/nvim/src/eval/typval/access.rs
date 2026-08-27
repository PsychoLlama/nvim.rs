//! The one-line accessors every other module reaches a `typval_T` through.
//!
//! Upstream declares these `static inline` in `typval.h`, so they are the part
//! of this file that is compiled into its callers rather than called; they keep
//! the `#[inline]` the transpile gave them for the same reason.  The four
//! `QUEUE_*` helpers are the intrusive-list macros `dv_watchers` is threaded
//! on.
//!
//! Every accessor takes the raw pointer its callers already hold — 500-odd call
//! sites across the tree pass `*mut list_T`/`*mut dict_T` around, and the
//! `typval_T` family's layout is frozen by the LuaJIT unit specs.  What they
//! buy the rest of the family is that *nothing else* has to spell a field walk:
//! the children below reach a list through `tv_list_first`/`tv_list_last`/
//! `tv_list_len`, never through `(*l).lv_first`.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::winlayer::Live;

/// The `Copy` handles over the four objects this module manipulates through
/// raw pointers, plus the two it reaches through them.
///
/// See [`Live`](crate::winlayer::Live): construction is the one `unsafe` step
/// and records the caller's promise that the pointee stays live; every
/// `(*p).field` after it is ordinary checked code. Writing a `typval_T`'s
/// union member through one is checked too — it is only *reading* a union
/// that stays unsafe — which is why the `tv_*_set`/`_alloc` families lose
/// almost all of their regions to these.
///
/// A handle is never built from a pointer the code has not already committed
/// to dereferencing: the null-tolerant entry points (`tv_list_len`,
/// `tv_list_ref`, …) keep their `as_ref()` guard and take no handle.
pub(crate) type Tv = Live<typval_T>;
/// A live `list_T`; see [`Tv`].
pub(crate) type Ls = Live<list_T>;
/// A live `listitem_T`; see [`Tv`].
pub(crate) type Li = Live<listitem_T>;
/// A live `dict_T`; see [`Tv`].
pub(crate) type Dt = Live<dict_T>;
/// A live `dictitem_T`; see [`Tv`].
pub(crate) type Di = Live<dictitem_T>;
/// A live `blob_T`; see [`Tv`].
pub(crate) type Bl = Live<blob_T>;
/// A live `garray_T`; see [`Tv`].
pub(crate) type Ga = Live<garray_T>;
/// A live `partial_T`; see [`Tv`].
pub(crate) type Pt = Live<partial_T>;
/// A live `Callback`; see [`Tv`].
pub(crate) type Cb = Live<Callback>;
/// A live `DictWatcher`; see [`Tv`].
pub(crate) type Dw = Live<DictWatcher>;
/// A live `listwatch_T`; see [`Tv`].
pub(crate) type Lw = Live<listwatch_T>;
/// A live `sortinfo_T`; see [`Tv`].
pub(crate) type Si = Live<sortinfo_T>;

/// The address of a field of `*p`, **computed rather than read**.
///
/// `&raw mut (*p).field` still requires an `unsafe` block today even though
/// nothing is dereferenced; `wrapping_byte_add` is the same arithmetic
/// spelled with a safe method, and it is defined for *every* pointer, null
/// and dangling included. The obligation the field's address carries belongs
/// to whoever dereferences it, and it is paid there.
///
/// This is [`Live::field_ptr`] without the handle, for the many places that
/// hold a bare pointer and only want to name one of its fields. The named
/// wrappers below spell the offsets, so no call site writes `offset_of!`.
#[inline(always)]
pub(crate) fn field_of<T, F>(p: *mut T, offset: usize) -> *mut F {
    p.wrapping_byte_add(offset).cast()
}

/// The address of a dictionary item's value; see [`field_of`].
#[inline(always)]
pub(crate) fn di_tv(di: *mut dictitem_T) -> *mut typval_T {
    field_of(di, ::core::mem::offset_of!(dictitem_T, di_tv))
}

/// The address of a list item's value; see [`field_of`].
#[inline(always)]
pub(crate) fn li_tv(li: *mut listitem_T) -> *mut typval_T {
    field_of(li, ::core::mem::offset_of!(listitem_T, li_tv))
}

/// The address of a dictionary's hash table; see [`field_of`].
#[inline(always)]
pub(crate) fn dv_hashtab(d: *mut dict_T) -> *mut hashtab_T {
    field_of(d, ::core::mem::offset_of!(dict_T, dv_hashtab))
}

/// The address of a dictionary's copy mark; see [`field_of`].
#[inline(always)]
pub(crate) fn dv_copyid(d: *mut dict_T) -> *mut ::core::ffi::c_int {
    field_of(d, ::core::mem::offset_of!(dict_T, dv_copyID))
}

/// The address of a dictionary's watcher queue; see [`field_of`].
#[inline(always)]
pub(crate) fn dv_watchers(d: *mut dict_T) -> *mut QUEUE {
    field_of(d, ::core::mem::offset_of!(dict_T, watchers))
}

/// The address of a list's copy mark; see [`field_of`].
#[inline(always)]
pub(crate) fn lv_copyid(l: *mut list_T) -> *mut ::core::ffi::c_int {
    field_of(l, ::core::mem::offset_of!(list_T, lv_copyID))
}

/// The address of a list's watcher chain head; see [`field_of`].
#[inline(always)]
pub(crate) fn lv_watch(l: *mut list_T) -> *mut *mut listwatch_T {
    field_of(l, ::core::mem::offset_of!(list_T, lv_watch))
}

/// The address of a blob's byte array; see [`field_of`].
#[inline(always)]
pub(crate) fn bv_ga(b: *mut blob_T) -> *mut garray_T {
    field_of(b, ::core::mem::offset_of!(blob_T, bv_ga))
}

/// The union members of a live `typval_T`, read as ordinary safe code.
///
/// Reading a union field is `unsafe` in Rust because a member the union does
/// not currently hold may have an invalid bit pattern for its type.
/// `typval_vval_union` has none: its nine members are a `varnumber_T`, two
/// `c_uint` tags, an `f64` and five raw pointers, and *every* bit pattern is
/// a valid value of each. So for an initialised `typval_T` — which is what
/// [`Live::new`] promises — reading any member is defined however the
/// `v_type` tag reads, and these are safe projections rather than an escape
/// hatch.
///
/// The tag still says which member is *meaningful*; that is the caller's
/// business, exactly as it was when the read spelled itself `(*tv).vval.v_x`.
impl Tv {
    /// `vval.v_number`.
    #[inline(always)]
    pub(crate) fn number(self) -> varnumber_T {
        unsafe { self.vval.v_number }
    }

    /// `vval.v_bool`.
    #[inline(always)]
    pub(crate) fn boolean(self) -> BoolVarValue {
        unsafe { self.vval.v_bool }
    }

    /// `vval.v_special`.
    #[inline(always)]
    pub(crate) fn special(self) -> SpecialVarValue {
        unsafe { self.vval.v_special }
    }

    /// `vval.v_float`.
    #[inline(always)]
    pub(crate) fn float(self) -> float_T {
        unsafe { self.vval.v_float }
    }

    /// `vval.v_string`.
    #[inline(always)]
    pub(crate) fn string(self) -> *mut ::core::ffi::c_char {
        unsafe { self.vval.v_string }
    }

    /// `vval.v_list`.
    #[inline(always)]
    pub(crate) fn list(self) -> *mut list_T {
        unsafe { self.vval.v_list }
    }

    /// `vval.v_dict`.
    #[inline(always)]
    pub(crate) fn dict(self) -> *mut dict_T {
        unsafe { self.vval.v_dict }
    }

    /// `vval.v_partial`.
    #[inline(always)]
    pub(crate) fn partial(self) -> *mut partial_T {
        unsafe { self.vval.v_partial }
    }

    /// `vval.v_blob`.
    #[inline(always)]
    pub(crate) fn blob(self) -> *mut blob_T {
        unsafe { self.vval.v_blob }
    }

    /// The *address* of `vval.v_dict`, for the sinks that are handed a
    /// `*mut *mut dict_T` so they can clear the slot; see [`field_of`].
    #[inline(always)]
    pub(crate) fn dict_ptr(self) -> *mut *mut dict_T {
        self.field_ptr(::core::mem::offset_of!(typval_T, vval))
    }
}

impl Li {
    /// The item's value, as a handle: it lives exactly as long as the item.
    #[inline(always)]
    pub(crate) fn tv(self) -> Tv {
        // SAFETY: `li_tv` is a field of the live item this handle names, and
        // `field_ptr` computes its address without borrowing the item.
        unsafe { Tv::new(self.field_ptr(::core::mem::offset_of!(listitem_T, li_tv))) }
    }

    /// `li_tv.v_type`.
    #[inline(always)]
    pub(crate) fn v_type(self) -> crate::types::VarType {
        self.li_tv.v_type
    }

    /// `li_tv.vval.v_number`; see [`Tv::number`].
    #[inline(always)]
    pub(crate) fn number(self) -> varnumber_T {
        self.tv().number()
    }

    /// `li_tv.vval.v_list`; see [`Tv::list`].
    #[inline(always)]
    pub(crate) fn list(self) -> *mut list_T {
        self.tv().list()
    }
}

/// `_()`: the translation of a message, which is always a literal here.
///
/// Safe by construction rather than by promise: the argument is a `&CStr`,
/// so the NUL `gettext` looks for is part of the type.
#[inline(always)]
pub(crate) fn tr(msg: &'static ::core::ffi::CStr) -> *const ::core::ffi::c_char {
    // SAFETY: `gettext` only reads the NUL-terminated string it is handed.
    unsafe { gettext(msg.as_ptr()) }
}

/// `_()` on one of the `c_bytes` error statics.
///
/// Those are `[c_char; N]` arrays rather than `CStr`s, so the NUL is the
/// literal's last byte rather than the type's promise — which is checked here
/// with a `debug_assert` rather than left to a comment.
#[inline(always)]
pub(crate) fn tr_bytes(msg: &'static [::core::ffi::c_char]) -> *const ::core::ffi::c_char {
    debug_assert!(msg.last() == Some(&0));
    // SAFETY: as `tr`, with the NUL checked just above.
    unsafe { gettext(msg.as_ptr()) }
}

/// The unlocked scalars and container handles, spelled once.
///
/// c2rust wrote the designated initialiser out in full at every site — three
/// fields, one of them a union literal, over six to nine lines. Every one of
/// them is the same shape: a `v_type` tag, `VarLock::Unlocked`, and the one
/// union member that tag selects. Writing a union field is safe, so these are
/// safe `const fn`s, and the sites that used to spell them inside an `unsafe`
/// region no longer put the literal there.
///
/// Deliberately not a constructor for `VAR_UNKNOWN`: that one is
/// [`TV_INITIAL_VALUE`], because it is a value rather than a conversion.
impl typval_T {
    /// A `VAR_NUMBER`.
    #[inline(always)]
    pub(crate) const fn number(v_number: varnumber_T) -> Self {
        Self {
            v_type: VAR_NUMBER,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_number },
        }
    }

    /// A `VAR_BOOL`.
    #[inline(always)]
    pub(crate) const fn boolean(v_bool: BoolVarValue) -> Self {
        Self {
            v_type: VAR_BOOL,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_bool },
        }
    }

    /// A `VAR_SPECIAL`.
    #[inline(always)]
    pub(crate) const fn special(v_special: SpecialVarValue) -> Self {
        Self {
            v_type: VAR_SPECIAL,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_special },
        }
    }

    /// A `VAR_FLOAT`.
    #[inline(always)]
    pub(crate) const fn float(v_float: float_T) -> Self {
        Self {
            v_type: VAR_FLOAT,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_float },
        }
    }

    /// A `VAR_STRING` owning `v_string`.
    #[inline(always)]
    pub(crate) const fn string(v_string: *mut ::core::ffi::c_char) -> Self {
        Self {
            v_type: VAR_STRING,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_string },
        }
    }

    /// A `VAR_LIST`.  Takes no reference; the caller still owes `tv_list_ref`.
    #[inline(always)]
    pub(crate) const fn list(v_list: *mut list_T) -> Self {
        Self {
            v_type: VAR_LIST,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_list },
        }
    }

    /// A `VAR_DICT`.  Takes no reference; the caller still owes a `retain`.
    #[inline(always)]
    pub(crate) const fn dict(v_dict: *mut dict_T) -> Self {
        Self {
            v_type: VAR_DICT,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_dict },
        }
    }
}

/// True when an intrusive queue head has no entries.
///
/// # Safety
/// `q` must point at an initialised queue node — one that has been through
/// [`queue_init`] or spliced onto a queue that has.
#[inline(always)]
pub unsafe fn queue_empty(q: *const QUEUE) -> bool {
    unsafe { q == (*q).next }
}

/// Make `q` an empty queue head, pointing at itself both ways.
///
/// # Safety
/// `q` must point at writable `QUEUE`-sized storage. Anything it was
/// already linked into is left pointing at it, so only initialise a node
/// that is on no queue.
#[inline(always)]
pub unsafe fn queue_init(q: *mut QUEUE) {
    unsafe { (*q).next = q };
    unsafe { (*q).prev = q };
}

/// Splice `q` in as the last entry of the queue headed by `h`.
///
/// # Safety
/// `h` must be an initialised queue head and `q` a node that is on no
/// queue. Both must outlive the link.
#[inline(always)]
pub(crate) unsafe fn queue_insert_tail(h: *mut QUEUE, q: *mut QUEUE) {
    unsafe { (*q).next = h };
    unsafe { (*q).prev = (*h).prev };
    unsafe { (*(*q).prev).next = q };
    unsafe { (*h).prev = q };
}

/// Unlink `q` from whatever queue it is on.
///
/// # Safety
/// `q` must be a node currently on a queue, and its neighbours must still
/// be live. The node's own links are left stale, so it has to be
/// re-initialised before it is used as a head again.
#[inline(always)]
pub(crate) unsafe fn queue_remove(q: *mut QUEUE) {
    unsafe { (*(*q).prev).next = (*q).next };
    unsafe { (*(*q).next).prev = (*q).prev };
}

/// Increase the reference count of `l`; does nothing for a NULL list.
///
/// # Safety
/// `l` is null or points at a live list. The caller gains a reference and
/// owes a matching `tv_list_unref`.
#[inline(always)]
pub unsafe fn tv_list_ref(l: *mut list_T) {
    if let Some(l) = unsafe { l.as_mut() } {
        l.lv_refcount.retain();
    }
}

/// Store `l` in `tv` as the return value, taking a reference to it.
///
/// # Safety
/// `tv` must point at a writable `typval_T` holding no value yet — the old
/// contents are overwritten, not cleared — and `l` is null or a live list.
#[inline(always)]
pub unsafe fn tv_list_set_ret(tv: *mut typval_T, l: *mut list_T) {
    // SAFETY: the caller's promise: a writable typval.
    let mut val = unsafe { Tv::new(tv) };
    val.v_type = VAR_LIST;
    val.vval.v_list = l;
    unsafe { tv_list_ref(l) };
}

/// Lock status of `l`; a NULL list reads as `VarLock::Fixed`.
///
/// # Safety
/// `l` is null or points at a live list.
#[inline]
pub unsafe fn tv_list_locked(l: *const list_T) -> VarLock {
    unsafe { l.as_ref() }.map_or(VarLock::Fixed, |l| l.lv_lock)
}

/// Set the lock status of `l`.  A NULL list may only be "set" to `VarLock::Fixed`.
///
/// # Safety
/// `l` is null or points at a live list. A null list can only be "set" to
/// `VarLock::Fixed`, which is what a `debug_assert` here checks.
#[inline]
pub unsafe fn tv_list_set_lock(l: *mut list_T, lock: VarLock) {
    match unsafe { l.as_mut() } {
        Some(l) => l.lv_lock = lock,
        None => debug_assert!(lock == VarLock::Fixed),
    }
}

/// Set the copyID of `l`.  Does not expect a NULL list, be careful.
///
/// # Safety
/// `l` must point at a live list — **not** null, unlike its neighbours. The
/// `copyid` must be one the caller reserved from `get_copyID`.
#[inline]
pub unsafe fn tv_list_set_copyid(l: *mut list_T, copyid: ::core::ffi::c_int) {
    unsafe { (*l).lv_copyID = copyid };
}

/// Number of items in `l`; a NULL list is empty.
///
/// # Safety
/// `l` is null or points at a live list.
#[inline]
pub unsafe fn tv_list_len(l: *const list_T) -> ::core::ffi::c_int {
    unsafe { l.as_ref() }.map_or(0, |l| l.lv_len)
}

/// The copyID of `l`.  Does not expect a NULL list, be careful.
///
/// # Safety
/// `l` must point at a live list — **not** null, unlike its neighbours.
#[inline]
pub unsafe fn tv_list_copyid(l: *const list_T) -> ::core::ffi::c_int {
    unsafe { (*l).lv_copyID }
}

/// Normalise a possibly negative list index against `l`'s length.
///
/// Returns an index in `0..tv_list_len(l)`, or -1 when it is out of range.
///
/// # Safety
/// `l` is null or points at a live list.
#[inline]
pub unsafe fn tv_list_uidx(l: *const list_T, n: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let len = unsafe { tv_list_len(l) };
    // A negative index counts back from the end.
    let n = if n < 0 { n + len } else { n };
    if n < 0 || n >= len { -1 } else { n }
}

/// First item of `l`, or NULL when it is empty or NULL.
///
/// # Safety
/// `l` is null or points at a live list. The item borrows the list, so it
/// is only valid while the list is.
#[inline]
pub unsafe fn tv_list_first(l: *const list_T) -> *mut listitem_T {
    unsafe { l.as_ref() }.map_or(::core::ptr::null_mut(), |l| l.lv_first)
}

/// Last item of `l`, or NULL when it is empty or NULL.
///
/// # Safety
/// `l` is null or points at a live list. The item borrows the list, so it
/// is only valid while the list is.
#[inline]
pub unsafe fn tv_list_last(l: *const list_T) -> *mut listitem_T {
    unsafe { l.as_ref() }.map_or(::core::ptr::null_mut(), |l| l.lv_last)
}

/// A walk over a list's items.  See [`tv_list_iter`].
pub(crate) struct ListIter {
    li: *mut listitem_T,
}

impl Iterator for ListIter {
    type Item = *mut listitem_T;

    #[inline]
    fn next(&mut self) -> Option<*mut listitem_T> {
        let li = self.li;
        if li.is_null() {
            return None;
        }
        self.li = unsafe { (*li).li_next };
        Some(li)
    }
}

/// Walk `l`'s items: upstream's `TV_LIST_ITER_CONST`.
///
/// It is **not** `TV_LIST_ITER`.  That macro re-reads `li_next` *after* the
/// body has run, so a body that frees or relinks the item it is standing on
/// still advances correctly; this reads the link first.  The two agree only
/// where the body leaves the walked list alone — which is every use in this
/// family, but check before reaching for it.
///
/// Takes an `Option` rather than a pointer because the macro's NULL handling is
/// half of what it does, and because that makes this one safe: `l.as_ref()` at
/// the call site costs the caller nothing, its `unsafe` block being already
/// open.
#[inline]
pub(crate) fn tv_list_iter(l: Option<&list_T>) -> ListIter {
    ListIter {
        li: l.map_or(::core::ptr::null_mut(), |l| l.lv_first),
    }
}

/// Store `d` in `tv` as the return value, taking a reference to it.
///
/// # Safety
/// `tv` must point at a writable `typval_T` holding no value yet — the old
/// contents are overwritten, not cleared — and `d` is null or a live
/// dictionary.
#[inline(always)]
pub unsafe fn tv_dict_set_ret(tv: *mut typval_T, d: *mut dict_T) {
    // SAFETY: the caller's promise: a writable typval.
    let mut val = unsafe { Tv::new(tv) };
    val.v_type = VAR_DICT;
    val.vval.v_dict = d;
    if let Some(d) = unsafe { d.as_mut() } {
        d.dv_refcount.retain();
    }
}

/// Number of items in `d`; a NULL dictionary is empty.
///
/// # Safety
/// `d` is null or points at a live dictionary.
#[inline]
pub unsafe fn tv_dict_len(d: *const dict_T) -> ::core::ffi::c_long {
    unsafe { d.as_ref() }.map_or(0, |d| d.dv_hashtab.ht_used as ::core::ffi::c_long)
}

/// Whether at least one watcher is registered on `d`.
///
/// # Safety
/// `d` is null or points at a live dictionary whose watcher queue has been
/// initialised (every dictionary from `tv_dict_alloc` has).
#[inline]
pub unsafe fn tv_dict_is_watched(d: *const dict_T) -> bool {
    unsafe { d.as_ref() }.is_some_and(|d| !unsafe { queue_empty(&raw const d.watchers) })
}

/// The key of `di`, which upstream reads as the plain `di->di_key`.
///
/// `di_key` is a flexible array member: `tv_dict_item_alloc_len` over-allocates
/// the `dictitem_T` so the NUL-terminated key sits in the tail.  The field
/// itself covers zero bytes, so the pointer has to be formed with `&raw`, not
/// by autoreffing the array.
///
/// Safe: this is arithmetic, not a read.  The key's address is the item's
/// plus a constant, and [`field_of`] computes it with `wrapping_byte_add`,
/// which is defined for every pointer.  *Reading* what comes back still needs
/// an item allocated by
/// [`tv_dict_item_alloc_len`](super::tv_dict_item_alloc_len) or embedded in a
/// fixed-variable array — the key lives in the allocation's tail, so an item
/// that was not over-allocated for its key has none — but that obligation
/// belongs to the dereference, which is where it is now paid.
#[inline(always)]
pub(crate) fn tv_dict_item_key(di: *const dictitem_T) -> *mut ::core::ffi::c_char {
    field_of(di.cast_mut(), ::core::mem::offset_of!(dictitem_T, di_key))
}

/// The `dictitem_T` a hashtab item's key points into: upstream's
/// `TV_DICT_HI2DI`.
///
/// A dictionary's hashtab does not store a pointer to its item; `hi_key` points
/// *at* the item's own `di_key`, so the item is that many bytes back.
///
/// # Safety
/// `hi` must be an *occupied* slot of a dictionary's hashtab. The item is
/// found by subtracting an offset from `hi_key`, so an empty or removed
/// slot yields a wild pointer rather than null.
#[inline(always)]
pub(crate) unsafe fn tv_dict_hi2di(hi: *const hashitem_T) -> *mut dictitem_T {
    unsafe {
        (*hi)
            .hi_key
            .sub(::core::mem::offset_of!(dictitem_T, di_key))
            .cast::<dictitem_T>()
    }
}

/// A walk over the occupied slots of a dictionary's hashtab.
///
/// See [`tv_dict_iter`].
pub(crate) struct DictIter {
    hi: *mut hashitem_T,
    todo: size_t,
}

impl Iterator for DictIter {
    type Item = *mut hashitem_T;

    #[inline]
    fn next(&mut self) -> Option<*mut hashitem_T> {
        while self.todo != 0 {
            let hi = self.hi;
            self.hi = unsafe { self.hi.add(1) };
            if unsafe { (*hi).is_kept() } {
                self.todo -= 1;
                return Some(hi);
            }
        }
        None
    }
}

/// Walk the occupied slots of `d`'s hashtab: upstream's `TV_DICT_ITER`, which
/// is `HASHTAB_ITER` plus a `TV_DICT_HI2DI`.
///
/// The item is yielded as the `hashitem_T`, not the `dictitem_T`, because the
/// bodies that remove entries need it for `hash_remove`; [`tv_dict_hi2di`] is
/// the other half.
///
/// The live-item count is snapshotted before the first step, exactly as the
/// macro does.  That is what lets a body remove entries as it goes — but only
/// with the hashtab locked, since an unlocked `hash_remove` may rehash and
/// invalidate `ht_array` underneath the walk.
///
/// The borrow is momentary — it reads the two hashtab header fields and is
/// gone before the first step, so a body may write through the caller's raw
/// pointer as upstream's does.  That is also what makes this one safe.
#[inline]
pub(crate) fn tv_dict_iter(d: &dict_T) -> DictIter {
    tv_ht_iter(&d.dv_hashtab)
}

/// [`tv_dict_iter`] over a bare hashtab: upstream's `HASHTAB_ITER`.
///
/// The variable scopes are reached both ways -- as a `dict_T` and as the
/// `hashtab_T` inside it -- so both spellings exist. The contract is the
/// same one.
#[inline]
pub(crate) fn tv_ht_iter(ht: &hashtab_T) -> DictIter {
    DictIter {
        hi: ht.ht_array,
        todo: ht.ht_used,
    }
}

/// Store `b` in `tv` as the return value, taking a reference to it.
///
/// # Safety
/// `tv` must point at a writable `typval_T` holding no value yet — the old
/// contents are overwritten, not cleared — and `b` is null or a live blob.
#[inline(always)]
pub unsafe fn tv_blob_set_ret(tv: *mut typval_T, b: *mut blob_T) {
    // SAFETY: the caller's promise: a writable typval.
    let mut val = unsafe { Tv::new(tv) };
    val.v_type = VAR_BLOB;
    val.vval.v_blob = b;
    if let Some(b) = unsafe { b.as_mut() } {
        b.bv_refcount.retain();
    }
}

/// Length of `b`'s data in bytes; a NULL blob is empty.
///
/// # Safety
/// `b` is null or points at a live blob.
#[inline]
pub unsafe fn tv_blob_len(b: *const blob_T) -> ::core::ffi::c_int {
    unsafe { b.as_ref() }.map_or(0, |b| b.bv_ga.ga_len)
}

/// The byte at `idx` in `b`.  `b` must be non-NULL and `idx` in range.
///
/// # Safety
/// `b` must point at a live blob and `idx` must be in `0..tv_blob_len(b)`.
/// Neither is checked.
#[inline(always)]
pub unsafe fn tv_blob_get(b: *const blob_T, idx: ::core::ffi::c_int) -> uint8_t {
    unsafe { *(*b).bv_ga.ga_data.cast::<uint8_t>().offset(idx as isize) }
}

/// Store `c` at `idx` in `blob`.  `blob` must be non-NULL and `idx` in range.
///
/// # Safety
/// `blob` must point at a live blob and `idx` must be in
/// `0..tv_blob_len(blob)`. Neither is checked.
#[inline(always)]
pub unsafe fn tv_blob_set(blob: *mut blob_T, idx: ::core::ffi::c_int, c: uint8_t) {
    unsafe { *(*blob).bv_ga.ga_data.cast::<uint8_t>().offset(idx as isize) = c };
}

/// The `DictWatcher` a queue node is embedded in (upstream's `QUEUE_DATA`).
///
/// Upstream spells this out as a function rather than the macro purely so it
/// can carry `FUNC_ATTR_NO_SANITIZE_ADDRESS`: ASan does not follow the pointer
/// arithmetic back out of the struct.
///
/// # Safety
/// `q` must be the `node` field of a live `DictWatcher` — a node from any
/// other queue yields a wild pointer, since the watcher is found by
/// subtracting an offset.
#[inline(always)]
pub unsafe fn tv_dict_watcher_node_data(q: *mut QUEUE) -> *mut DictWatcher {
    unsafe {
        q.cast::<::core::ffi::c_char>()
            .sub(::core::mem::offset_of!(DictWatcher, node))
    }
    .cast::<DictWatcher>()
}

/// Whether `tv` holds a function: either `VAR_FUNC` or `VAR_PARTIAL`.
#[inline(always)]
pub fn tv_is_func(tv: typval_T) -> bool {
    tv.v_type == VAR_FUNC || tv.v_type == VAR_PARTIAL
}
