//! `Blob`: a reference-counted byte vector, and the builtins over it.
//!
//! [`tv_blob_alloc`] answers a [`BlobRef`], whose drop is the release.
//! [`blob_slice_or_index`] is the subscript, [`set_range`] and
//! [`Blob::set_or_append`] the two ways an assignment writes into one, and
//! [`blob_remove`] is `remove()`.  [`f_blob2list`] and [`f_list2blob`]
//! convert to and from a list of byte numbers.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::message::emsg;
use crate::semsg;
use crate::types::Failed;

/// The byte vector a [`Blob`] is.
impl Blob {
    /// The blob's bytes.
    #[inline]
    pub fn bytes(&self) -> &[u8] {
        &self.bv_data
    }

    /// The blob's bytes, writable.
    #[inline]
    pub(crate) fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bv_data
    }

    /// How many bytes the blob holds.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.bv_data.len()
    }

    /// How many bytes the blob holds, as the `int` upstream counted in.
    #[inline]
    pub(crate) fn len_int(&self) -> ::core::ffi::c_int {
        ::core::ffi::c_int::try_from(self.len()).expect("a blob shorter than 2 GiB")
    }

    /// Whether the blob holds no bytes at all.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.bv_data.is_empty()
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
        let len = self.len_int();
        if idx > len {
            return;
        }
        if idx == len {
            self.claim(1);
        }
        self.set_byte(idx, byte);
    }

    /// Append `byte` to the blob.
    #[inline]
    pub fn push(&mut self, byte: u8) {
        self.bv_data.push(byte);
    }

    /// Append `bytes` to the blob.
    #[inline]
    pub(crate) fn extend(&mut self, bytes: &[u8]) {
        self.bv_data.extend_from_slice(bytes);
    }

    /// Make room for `n` more bytes, declare them live and answer the run
    /// just claimed. The run is zeroed; every caller overwrites it.
    pub(crate) fn claim(&mut self, n: usize) -> &mut [u8] {
        let was = self.len();
        self.bv_data.resize(was + n, 0);
        &mut self.bytes_mut()[was..]
    }

    /// Take the bytes `first..=last` out, closing the gap.
    pub(crate) fn drain(&mut self, first: usize, last: usize) {
        self.bv_data.drain(first..=last);
    }

    /// Drop every byte, leaving the blob empty and its storage released.
    #[inline]
    pub(crate) fn clear(&mut self) {
        self.bv_data = Vec::new();
    }
}

/// Length of `b`'s data in bytes; a NULL blob is empty.
#[inline]
pub(crate) fn blob_len(b: Option<&Blob>) -> ::core::ffi::c_int {
    b.map_or(0, Blob::len_int)
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
    /// The blob this value holds, borrowed -- `None` for every other kind
    /// and for `v:_null_blob`.
    ///
    /// The one the `blob_*` family reads its argument in: the borrow lasts
    /// as long as the value does.
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

    /// Another reference to the blob this value holds; see
    /// [`TypVal::list_handle`].
    #[inline(always)]
    pub(crate) fn blob_handle(&self) -> Option<BlobRef> {
        match self {
            TypVal::Blob(blob) => (**blob).clone(),
            _ => None,
        }
    }

    /// The handle this value holds, borrowed; see [`TypVal::list_shared`].
    #[inline(always)]
    pub(crate) fn blob_shared(&self) -> Option<&BlobRef> {
        match self {
            TypVal::Blob(blob) => blob.as_ref(),
            _ => None,
        }
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
/// `src` may hold the very same blob: `:let b[0 : len(b) - 1] = b` reaches
/// here with one blob as both operands. The length check forces such a range
/// to span the whole blob, so the copy is the identity and this answers
/// before it borrows `dest` at all.
pub(crate) fn set_range(
    dest: &BlobRef,
    n1: VarNumber,
    n2: VarNumber,
    src: &TypVal,
) -> Result<(), Failed> {
    if n2 - n1 + 1 != VarNumber::from(blob_len(src.blob_ref())) {
        let msg = gettext(c"E972: Blob value does not have the right number of bytes");
        emsg(msg);
        return Err(Failed);
    }
    if src.blob_shared().is_some_and(|from| from.ptr_eq(dest)) {
        return Ok(());
    }
    let bytes = blob_bytes(src.blob_ref());
    let first = usize::try_from(n1).expect("a byte of the blob");
    // Two different blobs, so the borrows name different allocations.
    dest.edit().bytes_mut()[first..first + bytes.len()].copy_from_slice(bytes);
    Ok(())
}

/// `remove()` over a blob: take out one byte, or the range `[idx, end]`, and
/// store what was removed in `result`. `arg_errmsg` is the message literal
/// a locked blob reports, translated.
pub fn blob_remove(
    blob: Option<&mut Blob>,
    args: &[TypVal],
    result: &mut TypVal,
    arg_errmsg: &'static ::core::ffi::CStr,
) {
    let lock = blob.as_ref().map_or(VarLock::Unlocked, |b| b.bv_lock);
    if value_check_lock(lock, LockName::Translate(arg_errmsg)) {
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
    let list = tv_list_alloc_ret(result, kListLenMayKnow as ptrdiff_t);
    if tv_check_for_blob_arg(args, 0).is_err() {
        return;
    }
    for &byte in blob_bytes(args[0].blob_ref()) {
        list.push_number(VarNumber::from(byte));
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
    for li in list_iter(args[0].list_ref()) {
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
    use crate::global_cell::editor_state_lock;

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
        let _held = editor_state_lock();
        let mut b = tv_blob_alloc();
        assert_eq!(b.bv_data.capacity(), 0);
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
        let _held = editor_state_lock();
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
        let _held = editor_state_lock();
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
        let _held = editor_state_lock();
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
        let _held = editor_state_lock();
        let held = blob_of(b"abcd");
        let src = TypVal::blob(Some(held.clone()));
        let mut dest = TypVal::Unknown;
        dest.write_blob(Some(held.clone()));

        assert_eq!(set_range(&held, 0, 3, &src), Ok(()));
        assert_eq!(blob_bytes(dest.blob_ref()), b"abcd");

        // A shorter range of the same blob never gets here: the length
        // check refuses it first, which is what keeps the identity the only
        // aliased case. That path raises `E972`, so it belongs in the
        // differential rather than here.

        tv_clear(&mut dest);
        drop(src);
        drop(held);
    }

    /// A copy is a blob of its own, holding the same bytes.
    #[test]
    fn copying_a_blob_answers_a_blob_of_its_own() {
        let _held = editor_state_lock();
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

    /// `tv_copy` of a blob shares it: one more reference, the same bytes,
    /// and a write through either value is seen through the other.
    #[test]
    fn a_copied_blob_value_shares_the_blob() {
        let _held = editor_state_lock();
        let b = blob_of(b"ab");
        let mut one = TypVal::blob(Some(b.clone()));
        let mut two = TypVal::Unknown;
        tv_copy(&one, &mut two);
        assert!(two.blob_shared().is_some_and(|two| two.ptr_eq(&b)));
        assert_eq!(b.bv_refcount.get(), 3);
        two.blob_mut().expect("a blob").push(b'c');
        assert_eq!(blob_bytes(one.blob_ref()), b"abc");
        tv_clear(&mut one);
        assert_eq!(b.bv_refcount.get(), 2);
        tv_clear(&mut two);
        assert_eq!(b.bv_refcount.get(), 1);
        assert_eq!(b.bytes(), b"abc");
    }

    /// The handle is the count: a clone retains, a drop releases, and the
    /// last drop frees (under Miri, a missed one is a leak).
    #[test]
    fn a_blob_handle_counts_its_references() {
        let _held = editor_state_lock();
        let b = tv_blob_alloc();
        assert_eq!(b.bv_refcount.get(), 1);
        let c = b.clone();
        assert_eq!(c.bv_refcount.get(), 2);
        drop(b);
        assert_eq!(c.bv_refcount.get(), 1);
        let d = c.clone();
        assert_eq!(d.bv_refcount.get(), 2);
        drop(c);
        drop(d);
    }
}
