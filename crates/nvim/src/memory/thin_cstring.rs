//! [`ThinCString`]: an owned, NUL-terminated string one pointer wide.
//!
//! [`XString`] is the editor's owned string, and it is three words: a
//! `Vec<u8>` knows its length and its capacity. That is the right shape for
//! a string being built, and the wrong one for a string *stored* in a value
//! the interpreter copies everywhere: a Vimscript value is sixteen bytes --
//! a tag and one word of payload -- and every argument frame, return slot,
//! list item and dictionary entry is sized by it. A fat string payload makes
//! it twenty-four.
//!
//! So this is the stored form: the address of an `xmalloc` block that ends
//! in a NUL, owned. The length is not kept; it is the distance to the
//! terminator, which is what the C this replaces measured too.
//!
//! * `Drop` frees the block, `Clone` duplicates it.
//! * Readers: [`as_cstr`](ThinCString::as_cstr) and
//!   [`as_bytes`](ThinCString::as_bytes) measure; [`first`](ThinCString::first)
//!   and [`is_empty`](ThinCString::is_empty) read one byte and do not;
//!   [`as_ptr`](ThinCString::as_ptr) hands the address to a callee that still
//!   walks to the NUL itself.
//! * `Option<ThinCString>` is one word as well: the address is never null,
//!   so `None` takes the null pointer -- which is how `v:_null_string` is
//!   spelled.
//!
//! # Relation to `XString`
//!
//! A sibling, not a wrapper and not the same type: a newtype over
//! `XString` cannot be one word, and making `XString` thin would cost every
//! builder a `strlen` per append. Both are blocks from the one allocator
//! (see the "Adopting a foreign block" rule in `allocator.rs`), so moving
//! between them never copies: [`From<XString>`] keeps the block and its
//! spare capacity, and [`into_xstring`](ThinCString::into_xstring) adopts it
//! back, measuring once.
//!
//! # Crossing the ABI
//!
//! [`into_raw`](ThinCString::into_raw) and [`from_raw`](ThinCString::from_raw)
//! are the hand-over to and from code that still holds a bare `*mut c_char`
//! it will `xfree`. As with `XString`, they are for the boundary with such a
//! producer or consumer; a string born and dropped in Rust needs neither.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `memory/` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::{CStr, c_char, c_void};
use core::fmt;
use core::ops::Deref;
use core::ptr::NonNull;

use super::{XString, xfree, xmemdupz, xrealloc};

/// An owned NUL-terminated string in one `xmalloc` block, one pointer wide.
///
/// See the module documentation for when to prefer it over [`XString`].
#[repr(transparent)]
pub struct ThinCString(NonNull<c_char>);

impl ThinCString {
    /// A copy of `bytes`, terminated.
    ///
    /// Byte-for-byte, as `xmemdupz` was: an interior NUL is kept, and every
    /// reader then stops at it, which is what the C reader of the copy did.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        // SAFETY: `bytes` is readable for its length; `xmemdupz` answers a
        // fresh, terminated block or does not return.
        let block = unsafe { xmemdupz(bytes.as_ptr().cast::<c_void>(), bytes.len()) };
        Self(NonNull::new(block.cast::<c_char>()).expect("xmemdupz never answers null"))
    }

    /// A copy of a C string.
    #[inline]
    pub fn from_cstr(string: &CStr) -> Self {
        Self::from_bytes(string.to_bytes())
    }

    /// The empty string: a block holding just the terminator.
    #[inline]
    pub fn empty() -> Self {
        Self::from_bytes(b"")
    }

    /// `bytes` as a string, keeping its allocation: the terminator is
    /// appended (which may grow the vector) and the block changes hands.
    pub fn from_vec(mut bytes: Vec<u8>) -> Self {
        bytes.push(0);
        // A vector's block is an `xmalloc` block, because the global
        // allocator is `malloc`; the spare capacity travels with it, which
        // `free` never asks about.
        let raw = core::mem::ManuallyDrop::new(bytes).as_mut_ptr();
        Self(NonNull::new(raw.cast::<c_char>()).expect("a vector holding a NUL is allocated"))
    }

    /// The string's address, for a callee that takes a `const char *`.
    ///
    /// Valid until the string is dropped or given away; a callee that keeps
    /// it longer wants [`into_raw`](Self::into_raw).
    #[inline(always)]
    pub fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr()
    }

    /// The string's address for a callee that writes through it. It may
    /// overwrite the payload, and must leave a NUL inside the block.
    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut c_char {
        self.0.as_ptr()
    }

    /// The string, measured: everything up to the terminator.
    #[inline]
    pub fn as_cstr(&self) -> &CStr {
        // SAFETY: the block is this value's own and ends in a NUL; the
        // borrow is tied to `self`, which frees it only on drop.
        unsafe { CStr::from_ptr(self.0.as_ptr()) }
    }

    /// The payload, measured, terminator excluded.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.as_cstr().to_bytes()
    }

    /// The payload, writable in place: the length cannot change, but a
    /// byte may become a NUL, after which the string reads shorter.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let len = self.as_bytes().len();
        // SAFETY: `len` bytes before the terminator, in this value's own
        // block, borrowed exclusively through `&mut self`.
        unsafe { core::slice::from_raw_parts_mut(self.0.as_ptr().cast::<u8>(), len) }
    }

    /// The first byte -- the terminator for the empty string. Reads one
    /// byte and measures nothing.
    #[inline(always)]
    pub fn first(&self) -> u8 {
        // SAFETY: a block always holds at least its terminator.
        unsafe { *self.0.as_ptr().cast::<u8>() }
    }

    /// Whether the payload is empty, without measuring it.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.first() == 0
    }

    /// Append `tail`, growing the block in place where `realloc` can.
    pub fn push_bytes(&mut self, tail: &[u8]) {
        let len = self.as_bytes().len();
        // SAFETY: the block is this value's own, from the `malloc` family,
        // and is handed to `xrealloc` once, which answers a block of
        // `len + tail.len() + 1` bytes holding the old `len` (or does not
        // return); `tail` cannot overlap it, since `self` is exclusive.
        // The terminator goes after the copied tail. Nothing here unwinds.
        unsafe {
            let block =
                xrealloc(self.0.as_ptr().cast::<c_void>(), len + tail.len() + 1).cast::<u8>();
            core::ptr::copy_nonoverlapping(tail.as_ptr(), block.add(len), tail.len());
            *block.add(len + tail.len()) = 0;
            self.0 = NonNull::new(block.cast::<c_char>()).expect("xrealloc never answers null");
        }
    }

    /// This string as an [`XString`], keeping the block. Measures once.
    pub fn into_xstring(self) -> XString {
        // SAFETY: an `xmalloc` block ending in a NUL, given up here.
        unsafe { XString::from_raw(self.into_raw()) }
    }

    /// Give the block to a caller that releases it with `xfree`.
    #[inline]
    pub fn into_raw(self) -> *mut c_char {
        core::mem::ManuallyDrop::new(self).0.as_ptr()
    }

    /// Adopt a block an `xmalloc`-family function produced; the null
    /// pointer, which such a producer answers for "no string", is `None`.
    ///
    /// # Safety
    ///
    /// A non-null `raw` is a live NUL-terminated block from the `xmalloc`
    /// family (or a `Vec`/`Box`/`CString` handed over the same way), not
    /// aliased, and nobody else will free it.
    #[inline]
    pub unsafe fn from_raw(raw: *mut c_char) -> Option<Self> {
        NonNull::new(raw).map(Self)
    }
}

impl Drop for ThinCString {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: the block this value owns, released once.
        unsafe { xfree(self.0.as_ptr().cast::<c_void>()) };
    }
}

impl Clone for ThinCString {
    /// A fresh block holding the same bytes: `xstrdup`.
    #[inline]
    fn clone(&self) -> Self {
        Self::from_bytes(self.as_bytes())
    }
}

impl Deref for ThinCString {
    type Target = CStr;

    /// The string as a `CStr` -- which measures it.
    #[inline]
    fn deref(&self) -> &CStr {
        self.as_cstr()
    }
}

impl fmt::Debug for ThinCString {
    /// The payload, escaped, so non-UTF-8 text is still readable.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ThinCString({})",
            self.as_cstr().to_string_lossy().escape_debug()
        )
    }
}

impl PartialEq for ThinCString {
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for ThinCString {}

impl PartialEq<[u8]> for ThinCString {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_bytes() == other
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for ThinCString {
    fn eq(&self, other: &&[u8; N]) -> bool {
        self.as_bytes() == other.as_slice()
    }
}

impl PartialEq<&CStr> for ThinCString {
    fn eq(&self, other: &&CStr) -> bool {
        self.as_cstr() == *other
    }
}

impl From<&[u8]> for ThinCString {
    fn from(bytes: &[u8]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<&CStr> for ThinCString {
    fn from(string: &CStr) -> Self {
        Self::from_cstr(string)
    }
}

impl From<&str> for ThinCString {
    fn from(text: &str) -> Self {
        Self::from_bytes(text.as_bytes())
    }
}

impl From<XString> for ThinCString {
    /// The same block, spare capacity and all: nothing is copied.
    fn from(string: XString) -> Self {
        let raw = string.into_raw();
        Self(NonNull::new(raw).expect("an XString is never a null block"))
    }
}

impl From<Vec<u8>> for ThinCString {
    fn from(bytes: Vec<u8>) -> Self {
        Self::from_vec(bytes)
    }
}

impl From<std::ffi::CString> for ThinCString {
    /// The same block: a `CString` is a `malloc` block ending in a NUL.
    fn from(string: std::ffi::CString) -> Self {
        let raw = string.into_raw();
        Self(NonNull::new(raw).expect("a CString is never a null block"))
    }
}

#[cfg(test)]
mod tests {
    use super::ThinCString;
    use crate::memory::{XString, xstrdup};
    use core::ffi::CStr;
    use core::mem::size_of;

    #[test]
    fn one_word_and_none_is_the_null_pointer() {
        assert_eq!(size_of::<ThinCString>(), size_of::<usize>());
        assert_eq!(size_of::<Option<ThinCString>>(), size_of::<usize>());
    }

    #[test]
    fn bytes_and_cstr_read_without_the_terminator() {
        let string = ThinCString::from_bytes(b"shell");
        assert_eq!(string.as_bytes(), b"shell");
        assert_eq!(string.as_cstr(), c"shell");
        assert_eq!(&*string, c"shell");
        assert_eq!(string.first(), b's');
        assert!(!string.is_empty());
    }

    #[test]
    fn the_empty_string_is_a_block_not_none() {
        let string = ThinCString::empty();
        assert!(string.is_empty());
        assert_eq!(string.first(), 0);
        assert_eq!(string.as_bytes(), b"");
        assert_eq!(string, ThinCString::from_cstr(c""));
    }

    #[test]
    fn a_clone_is_an_independent_block() {
        let original = ThinCString::from_bytes(b"first");
        let mut copy = original.clone();
        assert_ne!(original.as_ptr(), copy.as_ptr());
        copy.push_bytes(b"-second");
        assert_eq!(original, b"first");
        assert_eq!(copy, b"first-second");
    }

    #[test]
    fn writing_in_place_keeps_the_length_and_a_nul_shortens_it() {
        let mut string = ThinCString::from_bytes(b"abc");
        string.as_bytes_mut()[0] = b'x';
        assert_eq!(string, b"xbc");
        string.as_bytes_mut()[1] = 0;
        assert_eq!(string, b"x");
    }

    #[test]
    fn push_grows_past_the_original_block() {
        let mut string = ThinCString::from_bytes(b"ab");
        string.push_bytes(b"");
        string.push_bytes(b"cdefghijklmnopqrstuvwxyz0123456789");
        assert_eq!(string, b"abcdefghijklmnopqrstuvwxyz0123456789");
    }

    #[test]
    fn an_interior_nul_ends_every_reading() {
        let string = ThinCString::from_bytes(b"before\0after");
        assert_eq!(string.as_bytes(), b"before");
        // And a copy is of what a reader sees.
        assert_eq!(string.clone(), b"before");
    }

    #[test]
    fn moving_to_and_from_xstring_keeps_the_block() {
        let mut built = XString::with_capacity(32);
        built.push_str("built");
        let at = built.as_ptr();
        let thin = ThinCString::from(built);
        assert_eq!(thin.as_ptr(), at);
        assert_eq!(thin, b"built");
        let back = thin.into_xstring();
        assert_eq!(back.as_ptr(), at);
        assert_eq!(back, b"built");
    }

    #[test]
    fn a_vector_is_terminated_and_kept() {
        assert_eq!(ThinCString::from(b"vec".to_vec()), b"vec");
        assert_eq!(ThinCString::from(Vec::new()), b"");
        assert_eq!(ThinCString::from(Vec::with_capacity(8)), b"");
        let cstring = std::ffi::CString::new("owned").expect("no NUL");
        assert_eq!(ThinCString::from(cstring), b"owned");
    }

    /// The perimeter round trip, checked against the real allocator under
    /// Miri: a block `xstrdup` made is adopted and freed by `Drop`, and a
    /// block given up is one `ThinCString::from_raw` takes back.
    #[test]
    fn raw_blocks_round_trip_through_the_allocator() {
        // SAFETY: a literal duplicated by the editor's allocator, adopted
        // once.
        let adopted = unsafe { ThinCString::from_raw(xstrdup(c"adopted".as_ptr())) };
        let adopted = adopted.expect("xstrdup answers a block");
        assert_eq!(adopted, b"adopted");
        let raw = adopted.into_raw();
        // SAFETY: the block `into_raw` just gave up, read and re-adopted.
        assert_eq!(unsafe { CStr::from_ptr(raw) }, c"adopted");
        // SAFETY: as above; dropped once at the end of the statement.
        drop(unsafe { ThinCString::from_raw(raw) });
        // SAFETY: the null pointer is no block at all.
        assert!(unsafe { ThinCString::from_raw(core::ptr::null_mut()) }.is_none());
    }
}
