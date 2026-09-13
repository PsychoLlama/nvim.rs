//! `Blob`: a reference-counted byte vector, and the builtins over it.
//!
//! [`tv_blob_alloc`] / [`blob_unref`] are the lifetime pair.
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
        unsafe { blob_unref(self.as_ptr()) };
    }
}

impl ::core::ops::Deref for BlobRef {
    type Target = Blob;

    #[inline(always)]
    fn deref(&self) -> &Blob {
        // SAFETY: the handle holds a reference, so the blob is live; the
        // borrow lasts only as long as the access that asked for it.
        unsafe { self.0.as_ref() }
    }
}

impl ::core::ops::DerefMut for BlobRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Blob {
        // SAFETY: as [`BlobRef::deref`].
        unsafe { self.0.as_mut() }
    }
}

/// The byte vector a [`Blob`] is.
///
/// The bytes live in a [`GArray`](crate::types::GArray) because the C
/// reached them through one; an array that never grew has a null
/// `ga_data`, which is the empty slice.
impl Blob {
    /// The blob's bytes.
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        if self.bv_ga.ga_data.is_null() {
            return &[];
        }
        let len = self.len();
        // SAFETY: the blob's own byte array, `ga_len` bytes long -- the
        // invariant of every `Blob` the allocator hands out.
        unsafe { ::core::slice::from_raw_parts(self.bv_ga.ga_data.cast::<u8>(), len) }
    }

    /// The blob's bytes, writable.
    #[inline]
    pub(crate) fn bytes_mut(&mut self) -> &mut [u8] {
        if self.bv_ga.ga_data.is_null() {
            return &mut [];
        }
        let len = self.len();
        // SAFETY: as [`Blob::bytes`], and `&mut self` is the exclusive
        // borrow the slice needs.
        unsafe { ::core::slice::from_raw_parts_mut(self.bv_ga.ga_data.cast::<u8>(), len) }
    }

    /// How many bytes the blob holds.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        usize::try_from(self.bv_ga.ga_len).unwrap_or(0)
    }

    /// Whether the blob holds no bytes at all.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.bv_ga.ga_len <= 0
    }

    /// The byte at `idx`, which must name one.
    #[inline]
    pub(crate) fn byte(&self, idx: ::core::ffi::c_int) -> u8 {
        self.bytes()[usize::try_from(idx).expect("a byte of the blob")]
    }

    /// Store `byte` at `idx`, which must name a byte of the blob.
    #[inline]
    pub(crate) fn set_byte(&mut self, idx: ::core::ffi::c_int, byte: u8) {
        let at = usize::try_from(idx).expect("a byte of the blob");
        self.bytes_mut()[at] = byte;
    }

    /// `blob[idx] = byte`, growing the blob by one when `idx` is the slot
    /// just past the end.  Anything further out is silently ignored, which
    /// is upstream's `tv_blob_set_append`.
    pub(crate) fn set_or_append(&mut self, idx: ::core::ffi::c_int, byte: u8) {
        if idx > self.bv_ga.ga_len {
            return;
        }
        if idx == self.bv_ga.ga_len {
            self.claim(1);
        }
        self.set_byte(idx, byte);
    }

    /// Append `byte` to the blob.
    #[inline]
    pub(crate) fn push(&mut self, byte: u8) {
        // SAFETY: the blob's own byte array, which this grows by one.
        unsafe { ga_append(&raw mut self.bv_ga, byte) };
    }

    /// Make room for `n` more bytes, declare them live and answer the run
    /// just claimed.
    ///
    /// The bytes are whatever the allocator left there, so every caller
    /// overwrites the whole run.
    pub(crate) fn claim(&mut self, n: usize) -> &mut [u8] {
        let was = self.len();
        let n = ::core::ffi::c_int::try_from(n).expect("a short blob");
        // SAFETY: the blob's own byte array.
        unsafe { ga_grow(&raw mut self.bv_ga, n) };
        self.bv_ga.ga_len += n;
        &mut self.bytes_mut()[was..]
    }

    /// Take the bytes `first..=last` out, closing the gap.
    pub(crate) fn drain(&mut self, first: usize, last: usize) {
        let taken = ::core::ffi::c_int::try_from(last - first + 1).expect("a short blob");
        self.bytes_mut().copy_within(last + 1.., first);
        self.bv_ga.ga_len -= taken;
    }

    /// Drop every byte, leaving the blob empty and its storage released.
    #[inline]
    pub(crate) fn clear(&mut self) {
        // SAFETY: the blob's own byte array.
        unsafe { ga_clear(&raw mut self.bv_ga) };
    }
}

/// Length of `b`'s data in bytes; a NULL blob is empty.
#[inline]
pub(crate) fn blob_len(b: Option<&Blob>) -> ::core::ffi::c_int {
    b.map_or(0, |b| b.bv_ga.ga_len)
}

/// The bytes of `b`; a NULL blob is empty.
#[inline]
pub(crate) fn blob_bytes(b: Option<&Blob>) -> &[u8] {
    b.map_or(&[], Blob::bytes)
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

    /// The blob this value holds, borrowed -- `None` for every other kind
    /// and for `v:_null_blob`.
    ///
    /// The safe spelling of [`TypVal::blob_or_null`], and the one the
    /// `blob_*` family reads its argument in: the borrow lasts as long as
    /// the value does, which is what the pointer never said.
    #[inline(always)]
    pub(crate) fn blob_ref(&self) -> Option<&Blob> {
        match self {
            TypVal::Blob(blob) => blob.as_deref(),
            _ => None,
        }
    }

    /// The blob this value holds, borrowed for writing.
    ///
    /// The exclusive borrow is the whole point: a caller holding one cannot
    /// also be reading the blob through the value, which the raw pointer
    /// let it do.
    #[inline(always)]
    pub(crate) fn blob_mut(&mut self) -> Option<&mut Blob> {
        match self {
            TypVal::Blob(blob) => blob.as_deref_mut(),
            _ => None,
        }
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
pub unsafe fn blob_free(b: *mut Blob) {
    // SAFETY: the caller's promise: a live, unaliased blob.
    unsafe { &mut *b }.clear();
    unsafe { xfree(b.cast()) };
}

/// Drop a reference to `b`, freeing it when the last one goes.
///
/// # Safety
///
/// `b` must point at a live blob, unaliased for the call.
pub unsafe fn blob_unref(b: *mut Blob) {
    if let Some(blob) = unsafe { b.as_mut() }
        && blob.bv_refcount.release() <= 0
    {
        unsafe { blob_free(b) };
    }
}

/// Whether `b1` and `b2` hold the same bytes.  An empty blob and a NULL one
/// are equal.
pub fn blob_equal(b1: Option<&Blob>, b2: Option<&Blob>) -> bool {
    blob_bytes(b1) == blob_bytes(b2)
}

/// `blob[n1 : n2]`: store the sub-blob in `result`.
///
/// `result` holds the blob being subscripted on the way in.  Indexes out of
/// range give an empty result rather than an error.
pub(crate) fn blob_slice(
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
        let mut new_blob = tv_blob_alloc();
        let from = usize::try_from(n1).expect("a byte of the blob");
        let to = usize::try_from(n2).expect("a byte of the blob");
        new_blob
            .claim(to - from + 1)
            .copy_from_slice(&blob_bytes(result.blob_ref())[from..=to]);
        tv_clear(result);
        tv_blob_set_ret(result, Some(new_blob));
    }

    Ok(())
}

/// `blob[idx]`: store the byte in `result`.
///
/// `result` holds the blob being subscripted on the way in.  An index out of
/// range raises `E979`.
pub(crate) fn blob_index(
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

    let at = usize::try_from(idx).expect("a byte of the blob");
    let v = blob_bytes(result.blob_ref())[at];
    tv_clear(result);
    result.write_number(VarNumber::from(v));
    Ok(())
}

/// `blob[n1]` or `blob[n1 : n2]`, whichever `is_range` says.
///
/// `result` holds the blob being subscripted on the way in, which is the
/// only blob either half reads -- upstream passed it a second time and the
/// argument went unread.
pub fn blob_slice_or_index(
    is_range: bool,
    n1: VarNumber,
    n2: VarNumber,
    exclusive: bool,
    result: &mut TypVal,
) -> Result<(), Failed> {
    let len = blob_len(result.blob_ref());
    if is_range {
        blob_slice(len, n1, n2, exclusive, result)
    } else {
        blob_index(len, n1, result)
    }
}

/// Whether `n1` names a byte of a `bloblen`-byte blob, or the slot just past
/// the end (which an assignment may append to).
pub fn blob_check_index(
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
pub fn blob_check_range(
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
/// `dest` is a **pointer** rather than a borrow because `src` may name the
/// very same blob: `:let b[0 : len(b) - 1] = b` reaches here with one blob
/// as both operands, and a `&mut` to it while the source is being read is
/// undefined where the pointer was merely delicate. The length check forces
/// such a range to span the whole blob, so the copy is the identity and
/// this answers before it takes the borrow at all.
///
/// # Safety
///
/// `dest` must point at a live blob with no other borrow of it live for the
/// call; `src` may hold that same blob.
pub unsafe fn blob_set_range(
    dest: *mut Blob,
    n1: VarNumber,
    n2: VarNumber,
    src: &TypVal,
) -> Result<(), Failed> {
    // The source is kept as a pointer as well as a borrow: `dest` is
    // compared against the pointer, because a second *borrow* of what it
    // names is the aliasing this exists to avoid.
    let at = src.blob_or_null();
    // SAFETY: the value's own blob.
    let from = unsafe { at.as_ref() };
    if n2 - n1 + 1 != VarNumber::from(blob_len(from)) {
        let msg = tr(c"E972: Blob value does not have the right number of bytes");
        // SAFETY: a NUL-terminated message from the translation table.
        unsafe { emsg_ptr(msg) };
        return Err(Failed);
    }
    if ::core::ptr::eq(at, dest) {
        return Ok(());
    }
    let bytes = blob_bytes(from);
    let first = usize::try_from(n1).expect("a byte of the blob");
    // SAFETY: the caller's live blob, which the test above says `from` is
    // not -- so the two borrows name different allocations.
    unsafe { (*dest).bytes_mut()[first..first + bytes.len()].copy_from_slice(bytes) };
    Ok(())
}

/// `remove()` over a blob: take out one byte, or the range `[idx, end]`, and
/// store what was removed in `result`.
///
/// # Safety
///
/// `arg_errmsg` must point at the NUL-terminated message to raise when the
/// blob is locked.
pub unsafe fn blob_remove(
    blob: Option<&mut Blob>,
    args: &[TypVal],
    result: &mut TypVal,
    arg_errmsg: *const ::core::ffi::c_char,
) {
    let lock = blob.as_ref().map_or(VarLock::Unlocked, |b| b.bv_lock);
    // SAFETY: the caller's promise: a NUL-terminated message.
    if unsafe { value_check_lock(lock, arg_errmsg, TV_TRANSLATE as size_t) } {
        return;
    }

    let Ok(mut idx) = tv_get_number_chk(&args[1]) else {
        return;
    };

    let len = int64_t::from(blob_len(blob.as_deref()));
    if idx < 0 {
        // count from the end
        idx += len;
    }
    if idx < 0 || idx >= len {
        semsg!("E979: Blob index out of range: {}", idx);
        return;
    }
    // Past the range check the length is at least one, which a NULL blob
    // cannot be.
    let blob = blob.expect("a blob with a byte in it");
    let first = usize::try_from(idx).expect("a byte of the blob");

    if args.len() <= 2 {
        // Remove one item, return its value.
        result.write_number(VarNumber::from(blob.bytes()[first]));
        blob.drain(first, first);
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
    let last = usize::try_from(end).expect("a byte of the blob");

    let mut taken_held = tv_blob_alloc();
    taken_held
        .claim(last - first + 1)
        .copy_from_slice(&blob.bytes()[first..=last]);
    tv_blob_set_ret(result, Some(taken_held));
    blob.drain(first, last);
}

/// `blob2list()`: the blob's bytes as a list of numbers.
pub fn f_blob2list(args: &[TypVal], result: &mut TypVal, _fptr: EvalFuncData) {
    tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    if tv_check_for_blob_arg(args, 0).is_err() {
        return;
    }
    let l = result.list_or_null();
    for &byte in blob_bytes(args[0].blob_ref()) {
        // SAFETY: the list just stored in the return slot.
        unsafe { (*l).push_number(VarNumber::from(byte)) };
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
    // SAFETY: the argument's own list, borrowed for the walk.
    for li in list_iter(unsafe { args[0].list_or_null().as_ref() }) {
        let read = tv_get_number_chk(&li.li_tv);
        let n = read.unwrap_or(0);
        if read.is_err() || !(0..=255).contains(&n) {
            if read.is_ok() {
                // As in `eval/lval.rs`: upstream's text has no conversion in
                // it, so `n` has never reached the message.
                semsg!("E1239: Invalid value for blob: 0xlX");
            }
            blob.clear();
            return;
        }
        blob.push(u8::try_from(n).expect("a byte, just checked"));
    }
}

/// Allocate an empty blob and store it in `ret_tv` as the return value.
///
/// The answer is a **borrow** of the slot's blob, for the callers that go on
/// filling it; the slot owns the reference.
pub fn tv_blob_alloc_ret(ret_tv: &mut TypVal) -> &mut Blob {
    tv_blob_set_ret(ret_tv, Some(tv_blob_alloc()));
    ret_tv.blob_mut().expect("the blob just stored")
}

/// Store a copy of `from` in `to`.  A NULL blob copies as a NULL blob.
///
/// `to` is overwritten, not cleared: it holds no value yet.
pub fn blob_copy(from: Option<&Blob>, to: &mut TypVal) {
    let Some(from) = from else {
        to.write_blob(None);
        return;
    };
    let mut copy = tv_blob_alloc();
    copy.claim(from.len()).copy_from_slice(from.bytes());
    to.write_blob(Some(copy));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exclusive use of the collector's registries, which allocating a value
    /// into a typval reaches. See [`crate::eval::gc::serial`].
    fn serial() -> crate::eval::gc::serial::Held {
        crate::eval::gc::serial::lock()
    }

    /// A blob holding `bytes`, owned by the handle it answers.
    fn blob_of(bytes: &[u8]) -> BlobRef {
        let mut b = tv_blob_alloc();
        b.claim(bytes.len()).copy_from_slice(bytes);
        b
    }

    /// A blob the allocator has never grown has a **null** `ga_data`, and
    /// `from_raw_parts` refuses a null base even for an empty slice -- which
    /// is why [`Blob::bytes`] tests it rather than trusting the length.
    #[test]
    fn an_untouched_blob_is_the_empty_slice() {
        let _held = serial();
        let mut b = tv_blob_alloc();
        assert!(b.bv_ga.ga_data.is_null());
        assert_eq!(b.bytes(), b"");
        assert_eq!(b.bytes_mut(), b"");
        assert_eq!(b.len(), 0);
        assert!(b.is_empty());
        assert_eq!(blob_len(Some(&b)), 0);
        assert_eq!(blob_len(None), 0);
        assert_eq!(blob_bytes(None), b"");
    }

    /// `claim` hands back exactly the run it added, leaving what was there.
    #[test]
    fn claiming_room_answers_only_the_new_bytes() {
        let _held = serial();
        let mut b = blob_of(b"ab");
        let room = b.claim(3);
        assert_eq!(room.len(), 3);
        room.copy_from_slice(b"cde");
        assert_eq!(b.bytes(), b"abcde");
        assert_eq!(b.claim(0).len(), 0);
        assert_eq!(b.bytes(), b"abcde");
    }

    /// `drain` closes the gap and shortens the blob; the run may be the
    /// whole of it, which is `remove(b, 0, len(b) - 1)`.
    #[test]
    fn draining_a_run_closes_the_gap() {
        let _held = serial();
        let mut b = blob_of(b"abcdef");
        b.drain(1, 2);
        assert_eq!(b.bytes(), b"adef");
        b.drain(3, 3);
        assert_eq!(b.bytes(), b"ade");
        b.drain(0, 2);
        assert_eq!(b.bytes(), b"");
        assert!(b.is_empty());
    }

    /// `blob[idx] = byte` grows the blob by one at the slot just past the
    /// end and ignores anything further out -- upstream's silence, kept.
    #[test]
    fn setting_the_slot_past_the_end_appends_and_no_further() {
        let _held = serial();
        let mut b = blob_of(b"ab");
        b.set_or_append(0, b'z');
        assert_eq!(b.bytes(), b"zb");
        b.set_or_append(2, b'c');
        assert_eq!(b.bytes(), b"zbc");
        b.set_or_append(9, b'!');
        assert_eq!(b.bytes(), b"zbc");
    }

    /// `:let b[0 : len(b) - 1] = b` names one blob twice.  The length check
    /// forces such a range to be the whole blob, so the copy is the
    /// identity -- and the borrow is never taken twice.
    #[test]
    fn assigning_a_blob_over_the_whole_of_itself_changes_nothing() {
        let _held = serial();
        let held = blob_of(b"abcd");
        let at = held.as_ptr();
        let src = TypVal::blob(Some(held.clone()));
        let mut dest = TypVal::Unknown;
        dest.write_blob(Some(held));

        // SAFETY: the blob both values hold, unborrowed for the call.
        assert_eq!(unsafe { blob_set_range(at, 0, 3, &src) }, Ok(()));
        assert_eq!(blob_bytes(dest.blob_ref()), b"abcd");

        // A shorter range of the same blob never gets here: the length
        // check refuses it first, which is what keeps the identity the only
        // aliased case. That path raises `E972`, so it belongs in the
        // differential rather than here.

        tv_clear(&mut dest);
        drop(src);
    }

    /// A copy is a blob of its own, holding the same bytes.
    #[test]
    fn copying_a_blob_answers_a_blob_of_its_own() {
        let _held = serial();
        let from = blob_of(b"xyz");
        let mut to = TypVal::Unknown;
        blob_copy(Some(&from), &mut to);
        let copy = to.blob_ref().expect("the copy");
        assert_eq!(copy.bytes(), b"xyz");
        assert!(!::core::ptr::eq(copy, &*from));
        assert!(blob_equal(Some(&from), Some(copy)));

        let mut nothing = TypVal::Unknown;
        blob_copy(None, &mut nothing);
        assert_eq!(nothing.blob_ref().map(Blob::len), None);
        assert!(blob_equal(None, nothing.blob_ref()));

        tv_clear(&mut to);
    }
}
