//! `Blob`: a reference-counted byte vector, and the builtins over it.
//!
//! [`tv_blob_alloc`] / [`tv_blob_unref`] are the lifetime pair.
//! [`tv_blob_slice_or_index`] is the subscript, [`tv_blob_set_range`] and
//! [`tv_blob_set_append`] the two ways an assignment writes into one, and
//! [`tv_blob_remove`] is `remove()`.  [`f_blob2list`] and [`f_list2blob`]
//! convert to and from a list of byte numbers.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::message::emsg_ptr;
use crate::semsg;
use crate::types::Failed;
use ::core::ptr::NonNull;

/// One reference to a [`Blob`], given back when the handle goes.
///
/// The blob half of [`ListRef`]: the refcount *is* the ownership, `Clone`
/// retains and `Drop` releases, and the last one frees.  The two
/// constructors are the two things a raw pointer can mean; a handle is
/// never null, because `v:_null_blob` is `TypVal::blob(None)` and every
/// reader that wants the old spelling asks [`TypVal::blob_or_null`].
#[repr(transparent)]
pub struct BlobRef(NonNull<Blob>);

impl BlobRef {
    /// Take over a reference the caller already holds and will not release.
    ///
    /// # Safety
    ///
    /// `at` must point at a live blob, and the caller must hold a reference
    /// to it -- one this handle now owns and eventually gives back.
    #[inline(always)]
    pub unsafe fn from_owned(at: NonNull<Blob>) -> BlobRef {
        BlobRef(at)
    }

    /// Take over the caller's reference to `b`, or answer `None` for a NULL
    /// blob.  See [`BlobRef::from_owned`].
    ///
    /// # Safety
    ///
    /// As [`BlobRef::from_owned`], for a pointer that may be null.
    #[inline(always)]
    pub unsafe fn owning(b: *mut Blob) -> Option<BlobRef> {
        NonNull::new(b).map(BlobRef)
    }

    /// Take *another* reference to `b`: the caller keeps its own.
    ///
    /// `None` for a NULL blob, which is `v:_null_blob` and counts nothing.
    ///
    /// # Safety
    ///
    /// `b` is null or points at a live blob.
    #[inline(always)]
    pub unsafe fn retained(b: *mut Blob) -> Option<BlobRef> {
        let at = NonNull::new(b)?;
        // SAFETY: the caller's promise: a live blob.
        unsafe { Bl::new(b) }.bv_refcount.retain();
        Some(BlobRef(at))
    }

    /// The blob, as the pointer most of the family still takes.
    ///
    /// A **borrow**: live only while the handle is.
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut Blob {
        self.0.as_ptr()
    }
}

impl Clone for BlobRef {
    /// One more owner of the same blob.
    #[inline(always)]
    fn clone(&self) -> BlobRef {
        // SAFETY: this handle names a live blob, since it holds a reference
        // to it.
        unsafe { Bl::new(self.as_ptr()) }.bv_refcount.retain();
        BlobRef(self.0)
    }
}

impl Drop for BlobRef {
    /// Give the reference back, freeing the blob with the last one.
    #[inline(always)]
    fn drop(&mut self) {
        // SAFETY: this handle names a live blob, and is giving up the
        // reference that kept it so.
        unsafe { tv_blob_unref(self.as_ptr()) };
    }
}

/// The `TypVal` readers and writers for the blob arm.
///
/// Hand-written where the scalar arms are generated, because the payload is
/// a [`BlobRef`]: it can be borrowed but never handed out, since a copy of
/// it would be a reference nobody took.
impl TypVal {
    /// The blob, or `None` unless this is a `Blob` -- including the
    /// `v:_null_blob` case, which answers `Some(NULL)`.
    #[inline(always)]
    pub(crate) fn as_blob(&self) -> Option<*mut Blob> {
        match self {
            TypVal::Blob(blob) => Some(
                blob.as_ref()
                    .map_or(::core::ptr::null_mut(), BlobRef::as_ptr),
            ),
            _ => None,
        }
    }

    /// The blob this value holds, or NULL unless it is a blob holding one.
    ///
    /// A **borrow**; see [`TypVal::list_or_null`].
    #[inline(always)]
    pub(crate) fn blob_or_null(&self) -> *mut Blob {
        self.as_blob().unwrap_or(::core::ptr::null_mut())
    }

    /// A blob value over `blob`, which the value takes over.
    #[inline(always)]
    pub(crate) const fn blob(blob: Option<BlobRef>) -> TypVal {
        TypVal::Blob(::core::mem::ManuallyDrop::new(blob))
    }

    /// Overwrite this slot with `blob`, **releasing nothing**: see
    /// [`union_writers`](super::access).
    #[inline(always)]
    pub(crate) fn write_blob(&mut self, blob: Option<BlobRef>) {
        self.overwrite(TypVal::blob(blob));
    }

    /// Move the blob out of this slot, leaving `v:_null_blob` behind.
    #[inline(always)]
    pub(crate) fn take_blob(&mut self) -> Option<BlobRef> {
        match self {
            TypVal::Blob(blob) => blob.take(),
            _ => None,
        }
    }
}

/// Allocate an empty blob, **owned by the handle it answers**.
///
/// The blob arrives at a reference count of **one**, as
/// [`tv_list_alloc`](super::tv_list_alloc) does and where upstream answered
/// zero: a caller that stores it nowhere drops the handle, and that is the
/// free.
pub fn tv_blob_alloc() -> BlobRef {
    let blob = unsafe { xcalloc(1, ::core::mem::size_of::<Blob>()) } as *mut Blob;
    unsafe { ga_init(&raw mut (*blob).bv_ga, 1, 100) };
    // SAFETY: freshly allocated, and the count starts at the handle's one.
    unsafe { Bl::new(blob) }.bv_refcount.retain();
    // SAFETY: `xcalloc` never answers null, and the reference is this
    // handle's own.
    unsafe { BlobRef::from_owned(NonNull::new_unchecked(blob)) }
}

/// Free `b` and its bytes.
///
/// # Safety
///
/// `b` must point at a live blob, unaliased for the call.
pub unsafe fn tv_blob_free(b: *mut Blob) {
    unsafe { ga_clear(&raw mut (*b).bv_ga) };
    unsafe { xfree(b.cast()) };
}

/// Drop a reference to `b`, freeing it when the last one goes.
///
/// # Safety
///
/// `b` must point at a live blob, unaliased for the call.
pub unsafe fn tv_blob_unref(b: *mut Blob) {
    if let Some(blob) = unsafe { b.as_mut() }
        && blob.bv_refcount.release() <= 0
    {
        unsafe { tv_blob_free(b) };
    }
}

/// Whether `b1` and `b2` hold the same bytes.  An empty blob and a NULL one
/// are equal.
///
/// # Safety
///
/// `b1` must point at a live blob. `b2` must point at a live blob.
pub unsafe fn tv_blob_equal(b1: *const Blob, b2: *const Blob) -> bool {
    let len1 = unsafe { tv_blob_len(b1) };
    let len2 = unsafe { tv_blob_len(b2) };
    if len1 == 0 && len2 == 0 {
        return true;
    }
    if b1 == b2 {
        return true;
    }
    if len1 != len2 {
        return false;
    }
    let mut i = 0;
    while i < unsafe { (*b1).bv_ga.ga_len } {
        if unsafe { tv_blob_get(b1, i) } != unsafe { tv_blob_get(b2, i) } {
            return false;
        }
        i += 1;
    }
    true
}

/// `blob[n1 : n2]`: store the sub-blob in `result`.
///
/// `result` holds the blob being subscripted on the way in.  Indexes out of
/// range give an empty result rather than an error.
///
/// # Safety
///
/// `_blob` must point at a live blob. `result` must point at the caller's
/// return slot: an initialized typval it owns and will clear.
pub(crate) unsafe fn tv_blob_slice(
    _blob: *const Blob,
    len: ::core::ffi::c_int,
    mut n1: VarNumber,
    mut n2: VarNumber,
    exclusive: bool,
    result: &mut TypVal,
) -> Result<(), Failed> {
    // The resulting variable is a sub-blob.  If the indexes
    // are out of range the result is empty.
    if n1 < 0 {
        n1 += VarNumber::from(len);
        if n1 < 0 {
            n1 = 0;
        }
    }
    if n2 < 0 {
        n2 += VarNumber::from(len);
    } else if n2 >= VarNumber::from(len) {
        n2 = VarNumber::from(len - if exclusive { 0 } else { 1 });
    }
    if exclusive {
        n2 -= 1;
    }

    if n1 >= VarNumber::from(len) || n2 < 0 || n1 > n2 {
        tv_clear(result);
        result.write_blob(None);
    } else {
        let new_blob = tv_blob_alloc();
        let at = new_blob.as_ptr();
        let sublen = (n2 - n1 + 1) as ::core::ffi::c_int;
        unsafe { ga_grow(&raw mut (*at).bv_ga, sublen) };
        unsafe { (*at).bv_ga.ga_len = sublen };
        let n1 = n1 as ::core::ffi::c_int;
        let mut i = n1;
        while i <= n2 as ::core::ffi::c_int {
            unsafe { tv_blob_set(at, i - n1, tv_blob_get(result.blob_or_null(), i)) };
            i += 1;
        }
        tv_clear(result);
        tv_blob_set_ret(result, Some(new_blob));
    }

    Ok(())
}

/// `blob[idx]`: store the byte in `result`.
///
/// `result` holds the blob being subscripted on the way in.  An index out of
/// range raises `E979`.
///
/// # Safety
///
/// `_blob` must point at a live blob. `result` must point at the caller's
/// return slot: an initialized typval it owns and will clear.
pub(crate) unsafe fn tv_blob_index(
    _blob: *const Blob,
    len: ::core::ffi::c_int,
    mut idx: VarNumber,
    result: &mut TypVal,
) -> Result<(), Failed> {
    // The resulting variable is a byte value.
    // If the index is too big or negative that is an error.
    if idx < 0 {
        idx += VarNumber::from(len);
    }
    if idx >= VarNumber::from(len) || idx < 0 {
        semsg!("E979: Blob index out of range: {}", idx);
        return Err(Failed);
    }

    let v = unsafe { tv_blob_get((*result).blob_or_null(), idx as ::core::ffi::c_int) };
    tv_clear(result);
    (*result).write_number(VarNumber::from(v));
    Ok(())
}

/// `blob[n1]` or `blob[n1 : n2]`, whichever `is_range` says.
///
/// # Safety
///
/// `blob` must point at a live blob. `result` must point at the caller's
/// return slot: an initialized typval it owns and will clear.
pub unsafe fn tv_blob_slice_or_index(
    blob: *const Blob,
    is_range: bool,
    n1: VarNumber,
    n2: VarNumber,
    exclusive: bool,
    result: &mut TypVal,
) -> Result<(), Failed> {
    let len = unsafe { tv_blob_len((*result).blob_or_null()) };
    if is_range {
        unsafe { tv_blob_slice(blob, len, n1, n2, exclusive, result) }
    } else {
        unsafe { tv_blob_index(blob, len, n1, result) }
    }
}

/// Whether `n1` names a byte of a `bloblen`-byte blob, or the slot just past
/// the end (which an assignment may append to).
pub fn tv_blob_check_index(
    bloblen: ::core::ffi::c_int,
    n1: VarNumber,
    quiet: bool,
) -> Result<(), Failed> {
    if n1 < 0 || n1 > VarNumber::from(bloblen) {
        if !quiet {
            semsg!("E979: Blob index out of range: {}", n1);
        }
        return Err(Failed);
    }
    Ok(())
}

/// Whether `n1..=n2` is a range of a `bloblen`-byte blob.
pub fn tv_blob_check_range(
    bloblen: ::core::ffi::c_int,
    n1: VarNumber,
    n2: VarNumber,
    quiet: bool,
) -> Result<(), Failed> {
    if n2 < 0 || n2 >= VarNumber::from(bloblen) || n2 < n1 {
        if !quiet {
            semsg!("E979: Blob index out of range: {}", n2);
        }
        return Err(Failed);
    }
    Ok(())
}

/// `dest[n1 : n2] = src`: copy `src`'s blob over that range of `dest`.
///
/// # Safety
///
/// `dest` must point at a live blob, unaliased for the call. `src` must point
/// at an initialized typval, unaliased for the call.
pub unsafe fn tv_blob_set_range(
    dest: *mut Blob,
    n1: VarNumber,
    n2: VarNumber,
    src: &TypVal,
) -> Result<(), Failed> {
    if n2 - n1 + 1 != VarNumber::from(unsafe { tv_blob_len((*src).blob_or_null()) }) {
        let msg = tr(c"E972: Blob value does not have the right number of bytes");
        unsafe { emsg_ptr(msg) };
        return Err(Failed);
    }
    let mut il = n1 as ::core::ffi::c_int;
    let mut ir = 0;
    while il <= n2 as ::core::ffi::c_int {
        unsafe { tv_blob_set(dest, il, tv_blob_get((*src).blob_or_null(), ir)) };
        il += 1;
        ir += 1;
    }
    Ok(())
}

/// `blob[idx] = byte`, growing the blob by one when `idx` is the slot just
/// past the end.  Anything further out is silently ignored.
///
/// # Safety
///
/// `blob` must point at a live blob, unaliased for the call.
pub unsafe fn tv_blob_set_append(blob: *mut Blob, idx: ::core::ffi::c_int, byte: uint8_t) {
    let gap = bv_ga(blob);

    // Allow for appending a byte.  Setting a byte beyond
    // the end is an error otherwise.
    // SAFETY: the blob's own byte array.
    let mut ga = unsafe { Ga::new(gap) };
    if idx <= ga.ga_len {
        if idx == ga.ga_len {
            unsafe { ga_grow(gap, 1) };
            ga.ga_len += 1;
        }
        unsafe { tv_blob_set(blob, idx, byte) };
    }
}

/// `remove()` over a blob: take out one byte, or the range `[idx, end]`, and
/// store what was removed in `result`.
///
/// # Safety
///
/// `args` must be the evaluator's argument buffer (`Args::new`) and `result`
/// its live return value: the contract the two builtin dispatchers keep.
/// `arg_errmsg` must point at the NUL-terminated message to raise when the
/// blob is locked.
pub unsafe fn tv_blob_remove(
    args: &[TypVal],
    result: &mut TypVal,
    arg_errmsg: *const ::core::ffi::c_char,
) {
    let b = args[0].blob_or_null();
    if !b.is_null() && unsafe { value_check_lock((*b).bv_lock, arg_errmsg, TV_TRANSLATE as size_t) }
    {
        return;
    }

    let Ok(mut idx) = tv_get_number_chk(&args[1]) else {
        return;
    };

    let len = int64_t::from(unsafe { tv_blob_len(b) });
    if idx < 0 {
        // count from the end
        idx += len;
    }
    if idx < 0 || idx >= len {
        semsg!("E979: Blob index out of range: {}", idx);
        return;
    }
    // SAFETY: past the range check `len` is at least 1, which a NULL blob
    // cannot be, so this is the caller's live blob.
    let mut blob = unsafe { Bl::new(b) };

    if args.len() <= 2 {
        // Remove one item, return its value.
        let p = blob.bv_ga.ga_data.cast::<uint8_t>();
        unsafe { (*result).write_number(VarNumber::from(*p.offset(idx as isize))) };
        let at = unsafe { p.offset(idx as isize) };
        let after = unsafe { at.add(1) };
        let into = at.cast::<u8>();
        unsafe { into.copy_from(after.cast(), (len - idx - 1) as size_t) };
        blob.bv_ga.ga_len -= 1;
        return;
    }

    // Remove range of items, return blob with values.
    let Ok(mut end) = tv_get_number_chk(&args[2]) else {
        return;
    };
    if end < 0 {
        // count from the end
        end += len;
    }
    if end >= len || idx > end {
        semsg!("E979: Blob index out of range: {}", end);
        return;
    }

    let taken = (end - idx + 1) as ::core::ffi::c_int;
    let taken_held = tv_blob_alloc();
    let taken_raw = taken_held.as_ptr();
    // SAFETY: freshly allocated just above.
    let mut taken_blob = unsafe { Bl::new(taken_raw) };
    taken_blob.bv_ga.ga_len = taken;
    unsafe { ga_grow(&raw mut (*taken_raw).bv_ga, taken) };

    // Read `ga_data` after the allocation above, as upstream does.
    let p = blob.bv_ga.ga_data.cast::<uint8_t>();
    let dst = taken_blob.bv_ga.ga_data;
    let src = unsafe { p.offset(idx as isize) };
    unsafe { dst.cast::<u8>().copy_from(src.cast(), taken as size_t) };
    tv_blob_set_ret(result, Some(taken_held));

    if len - end - 1 > 0 {
        let at = unsafe { p.offset(idx as isize) };
        let after = unsafe { p.offset(end as isize).add(1) };
        let into = at.cast::<u8>();
        unsafe { into.copy_from(after.cast(), (len - end - 1) as size_t) };
    }
    blob.bv_ga.ga_len -= taken;
}

/// `blob2list()`: the blob's bytes as a list of numbers.
pub fn f_blob2list(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    if tv_check_for_blob_arg(args, 0).is_err() {
        return;
    }
    let blob = args[0].blob_or_null();
    let l = result.list_or_null();
    for i in 0..unsafe { tv_blob_len(blob) } {
        unsafe { tv_list_append_number(l, VarNumber::from(tv_blob_get(blob, i))) };
    }
}

/// `list2blob()`: a list of byte numbers as a blob.
///
/// A value outside `0..=255` raises `E1239` and answers the empty blob.
pub fn f_list2blob(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    let blob = tv_blob_alloc_ret(result);
    if tv_check_for_list_arg(args, 0).is_err() {
        return;
    }
    let l = args[0].list_or_null();
    if l.is_null() {
        return;
    }
    for li in tv_list_iter(unsafe { l.as_ref() }) {
        let read = tv_get_number_chk(&li.li_tv);
        let n = read.unwrap_or(0);
        if read.is_err() || !(0..=255).contains(&n) {
            if read.is_ok() {
                // As in `eval/lval.rs`: upstream's text has no conversion in
                // it, so `n` has never reached the message.
                semsg!("E1239: Invalid value for blob: 0xlX");
            }
            unsafe { ga_clear(&raw mut (*blob).bv_ga) };
            return;
        }
        unsafe { ga_append(&raw mut (*blob).bv_ga, n as uint8_t) };
    }
}

/// Allocate an empty blob and store it in `ret_tv` as the return value.
///
/// The answer is a **borrow** of the slot's blob, for the callers that go on
/// filling it; the slot owns the reference.
pub fn tv_blob_alloc_ret(ret_tv: &mut TypVal) -> *mut Blob {
    let held = tv_blob_alloc();
    let at = held.as_ptr();
    tv_blob_set_ret(ret_tv, Some(held));
    at
}

/// Store a copy of `from` in `to`.  A NULL blob copies as a NULL blob.
///
/// # Safety
///
/// `from` must point at a live blob, unaliased for the call. `to` must point
/// at an initialized typval, unaliased for the call.
pub unsafe fn tv_blob_copy(from: *mut Blob, to: &mut TypVal) {
    // SAFETY: the caller's promise: a writable typval.
    let mut dst = unsafe { Tv::new(to) };
    dst.write_empty(VAR_BLOB);
    if from.is_null() {
        to.write_blob(None);
        return;
    }

    tv_blob_alloc_ret(to);
    let len = unsafe { (*from).bv_ga.ga_len };
    let ga = bv_ga(dst.blob_or_null());
    if len > 0 {
        unsafe { (*ga).ga_data = xmemdup((*from).bv_ga.ga_data, len as size_t) };
    }
    // SAFETY: the destination blob's own byte array.
    let mut garr = unsafe { Ga::new(ga) };
    garr.ga_len = len;
    garr.ga_maxlen = len;
}
