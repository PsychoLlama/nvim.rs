//! [`XString`]: an owned byte string that keeps a NUL, and the one place a
//! `malloc` block changes hands between C and Rust.
//!
//! The editor's strings are produced by `xmalloc`/`xstrdup`/`xmemdupz` and
//! released by `xfree`, and a caller that holds one holds a bare
//! `*mut c_char` — the length is wherever the caller last measured it and
//! the free is wherever the caller remembered to write it. `XString` is that
//! buffer with an owner: it is a [`Vec<u8>`] whose last byte is always a NUL,
//! so the same allocation answers both questions the editor asks of a string.
//!
//! * As bytes ([`Deref`]) it is the payload *without* the terminator, which
//!   is what every slice-taking reader wants.
//! * As a C string ([`as_cstr`](XString::as_cstr), [`as_ptr`](XString::as_ptr))
//!   it is the same bytes with the terminator a C consumer walks to.
//!
//! Keeping the NUL in the buffer rather than appending one on demand is what
//! makes the second form free, and it is also what the transpiled walks
//! expect: several of them deliberately read one byte past the last
//! character (the same reason `memline`'s `LineCopy` carries its
//! terminator). A bare `Vec<u8>` of the payload would hand those walks
//! the vector's uninitialised capacity.
//!
//! # Crossing the ABI
//!
//! [`into_raw`](XString::into_raw) hands the block to a C consumer that will
//! `xfree` it; [`from_raw`](XString::from_raw) adopts a block an `xmalloc`
//! family function produced. Both are legal because
//! the `allocator` module *is* libc `malloc` — see the "Adopting a foreign
//! block" rule in that module's docs, which is the contract these two rely
//! on. Nothing above `memory/` should call them: a value that is born in
//! Rust and dies in Rust never needs either.
//!
//! # Interior NULs
//!
//! Storage keeps every byte it is given, terminator included, so `len()` and
//! the [`Deref`] slice are exact. A C consumer reading
//! [`as_ptr`](XString::as_ptr) stops at the first NUL, and
//! [`as_cstr`](XString::as_cstr) answers exactly what that consumer sees.
//! [`from_bytes`](XString::from_bytes) debug-asserts that its input has no
//! interior NUL, because text that may hold one is measured text and wants
//! a `&[u8]` or a [`Vec<u8>`], not a string the editor will hand to `os_*`.
//! The assertion is a debug build's tripwire, not a release-time refusal:
//! truncating or panicking on user text would be a behaviour change, and the
//! byte-for-byte answer is the one upstream gives.

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

use core::ffi::{CStr, c_char};
use core::ops::Deref;
use core::{fmt, slice};

/// An owned, NUL-terminated byte string.
///
/// The buffer always ends with a NUL and is therefore never empty; the bytes
/// this derefs to are everything before it.
pub struct XString(Vec<u8>);

impl XString {
    /// The empty string: a buffer holding just the terminator.
    pub fn new() -> Self {
        Self(vec![0])
    }

    /// An empty string with room for `payload` bytes before it has to grow
    /// again. The terminator's byte is added on top, so a caller that knows
    /// its final payload length allocates exactly once.
    pub fn with_capacity(payload: usize) -> Self {
        let mut bytes = Vec::with_capacity(payload + 1);
        bytes.push(0);
        Self(bytes)
    }

    /// A copy of `bytes`, terminated.
    ///
    /// `bytes` is payload only: a caller holding a `&CStr` wants
    /// [`from_cstr`](Self::from_cstr), and one holding a slice that already
    /// ends in a NUL must leave that byte out or the string gains an
    /// interior one.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        debug_assert!(
            !bytes.contains(&0),
            "XString::from_bytes: interior NUL -- measured text wants a Vec<u8>"
        );
        let mut buffer = Vec::with_capacity(bytes.len() + 1);
        buffer.extend_from_slice(bytes);
        buffer.push(0);
        Self(buffer)
    }

    /// A string a C callee writes.
    ///
    /// `payload` is the room the callee is told it has; the block is that
    /// many bytes plus a terminator, zero-filled, and `fill` is handed its
    /// address. The result is measured at the terminator the callee left
    /// behind, so a callee that wrote less than it asked for does not drag
    /// the rest of the buffer along -- which is the whole reason the
    /// `vim_snprintf`-into-`xmalloc` sites could not simply become a
    /// [`Vec`].
    ///
    /// The callee may write up to `payload` bytes and its own terminator,
    /// and nothing past that; stating the obligation is what the closure's
    /// own `unsafe` block is for.
    pub fn filled(payload: usize, fill: impl FnOnce(*mut c_char)) -> Self {
        let mut buffer = vec![0u8; payload + 1];
        fill(buffer.as_mut_ptr().cast::<c_char>());
        let used = buffer
            .iter()
            .position(|&byte| byte == 0)
            .expect("the last byte is the terminator the callee could not overwrite");
        buffer.truncate(used + 1);
        Self(buffer)
    }

    /// A copy of a C string, terminator and all. The `xstrdup` of this type.
    pub fn from_cstr(string: &CStr) -> Self {
        Self(string.to_bytes_with_nul().to_vec())
    }

    /// The payload's length in bytes, terminator excluded.
    pub fn len(&self) -> usize {
        self.0.len() - 1
    }

    /// Whether the string has no payload.
    pub fn is_empty(&self) -> bool {
        self.0.len() == 1
    }

    /// The string as a C string, stopping at the first NUL.
    ///
    /// Never fails: the buffer always ends with a terminator. With an
    /// interior NUL the answer is the prefix a C consumer would read, which
    /// is what [`as_ptr`](Self::as_ptr) hands out.
    pub fn as_cstr(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.0).expect("an XString always ends with a NUL")
    }

    /// The buffer's address, for a C callee that takes a `const char *`.
    ///
    /// The pointer is valid until the string is dropped, moved out of, or
    /// grown; a callee that keeps it past that point wants
    /// [`into_raw`](Self::into_raw) instead.
    pub fn as_ptr(&self) -> *const c_char {
        self.0.as_ptr().cast::<c_char>()
    }

    /// The buffer's address for a C callee that writes through it.
    ///
    /// The callee may overwrite the payload but must leave a NUL within the
    /// buffer, and must not write past it: the length stays this string's.
    pub fn as_mut_ptr(&mut self) -> *mut c_char {
        self.0.as_mut_ptr().cast::<c_char>()
    }

    /// Append `bytes` before the terminator.
    pub fn push_bytes(&mut self, bytes: &[u8]) {
        // Off, extend, back on: the invariant is restored before the
        // method returns and the append stays a single `extend_from_slice`.
        self.0.pop();
        self.0.extend_from_slice(bytes);
        self.0.push(0);
    }

    /// Append a `&str`'s bytes. The convenience the `strcat` sites want,
    /// since the tail they append is usually a literal.
    pub fn push_str(&mut self, text: &str) {
        self.push_bytes(text.as_bytes());
    }

    /// Append a C string's payload, terminator excluded.
    pub fn push_cstr(&mut self, string: &CStr) {
        self.push_bytes(string.to_bytes());
    }

    /// Append one byte.
    pub fn push_byte(&mut self, byte: u8) {
        self.push_bytes(slice::from_ref(&byte));
    }

    /// Cut the payload down to `len` bytes, re-terminating. Longer than the
    /// payload is a no-op, as [`Vec::truncate`]'s is.
    pub fn truncate(&mut self, len: usize) {
        if len < self.len() {
            self.0.truncate(len);
            self.0.push(0);
        }
    }

    /// Give the block to a caller that releases it with `xfree`.
    ///
    /// **The block does not move.** Shrinking to `len() + 1` first would be
    /// tidier -- and is what this did -- but `Vec::into_boxed_slice` is a
    /// `realloc`, and a caller that read [`as_ptr`](Self::as_ptr) before
    /// handing the string over is then left holding the old address. The
    /// option layer does exactly that: it reads the variable, replaces it,
    /// and frees what the replace handed back. So the spare capacity travels
    /// with the block, which is the mirror of the "Adopting a foreign block"
    /// rule in the `allocator` module: `free` never took a size and
    /// `realloc` reads the block's own, so an *overstated* block is as
    /// harmless to the receiver as an understated one is to us.
    ///
    /// Safe to call: what makes the handover legal is that both sides draw
    /// from one allocator.
    pub fn into_raw(self) -> *mut c_char {
        // Never empty -- the terminator is always there -- so this is a heap
        // address, not a dangling one.
        let mut bytes = core::mem::ManuallyDrop::new(self.0);
        bytes.as_mut_ptr().cast::<c_char>()
    }

    /// Adopt a block an `xmalloc`-family function produced.
    ///
    /// # Safety
    ///
    /// `raw` is a live, NUL-terminated block from the `xmalloc` family
    /// (`xmalloc`/`xmallocz`/`xstrdup`/`xmemdupz`/`xstrnsave`, or a
    /// `Vec`/`Box`/`CString` handed over the same way), it is not aliased,
    /// and nobody else will free it.
    pub unsafe fn from_raw(raw: *mut c_char) -> Self {
        // SAFETY: the caller's NUL-terminated block.
        let size = unsafe { CStr::from_ptr(raw) }.to_bytes().len() + 1;
        // SAFETY: the caller's block, which came from `malloc` because the
        // `xmalloc` family and the global allocator are the same one. The
        // stated capacity is the string's own extent; a block `malloc` made
        // larger is still released and reallocated correctly, which is the
        // adoption rule `allocator.rs` writes down.
        Self(unsafe { Vec::from_raw_parts(raw.cast::<u8>(), size, size) })
    }
}

impl Default for XString {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for XString {
    type Target = [u8];

    /// The payload, terminator excluded.
    fn deref(&self) -> &[u8] {
        &self.0[..self.0.len() - 1]
    }
}

impl Clone for XString {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl fmt::Debug for XString {
    /// The payload, escaped, so a non-UTF-8 option value is still readable.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "XString({})",
            self.as_cstr().to_string_lossy().escape_debug()
        )
    }
}

impl PartialEq for XString {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for XString {}

impl PartialEq<[u8]> for XString {
    fn eq(&self, other: &[u8]) -> bool {
        &**self == other
    }
}

impl PartialEq<&[u8]> for XString {
    fn eq(&self, other: &&[u8]) -> bool {
        &**self == *other
    }
}

impl<const N: usize> PartialEq<&[u8; N]> for XString {
    fn eq(&self, other: &&[u8; N]) -> bool {
        &**self == other.as_slice()
    }
}

impl PartialEq<&str> for XString {
    fn eq(&self, other: &&str) -> bool {
        &**self == other.as_bytes()
    }
}

impl PartialEq<&CStr> for XString {
    fn eq(&self, other: &&CStr) -> bool {
        self.as_cstr() == *other
    }
}

impl From<&[u8]> for XString {
    fn from(bytes: &[u8]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<&CStr> for XString {
    fn from(string: &CStr) -> Self {
        Self::from_cstr(string)
    }
}

impl From<&str> for XString {
    fn from(text: &str) -> Self {
        Self::from_bytes(text.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::XString;
    use crate::memory::{xfree, xstrdup};
    use core::ffi::{CStr, c_void};

    #[test]
    fn bytes_deref_without_the_terminator_and_the_cstr_keeps_it() {
        let string = XString::from_bytes(b"shell");
        assert_eq!(&*string, b"shell");
        assert_eq!(string.len(), 5);
        assert!(!string.is_empty());
        assert_eq!(string.as_cstr(), c"shell");
        assert_eq!(string.as_cstr().to_bytes_with_nul(), b"shell\0");
    }

    #[test]
    fn the_empty_string_is_a_lone_terminator() {
        let string = XString::new();
        assert!(string.is_empty());
        assert_eq!(string.len(), 0);
        assert_eq!(&*string, b"");
        assert_eq!(string.as_cstr(), c"");
        assert_eq!(XString::default(), XString::new());
    }

    #[test]
    fn from_cstr_copies_the_payload_and_the_terminator() {
        let string = XString::from_cstr(c"/usr/bin/env");
        assert_eq!(string, c"/usr/bin/env");
        assert_eq!(&*string, b"/usr/bin/env");
    }

    #[test]
    fn pushing_past_the_capacity_moves_the_terminator_along() {
        let mut string = XString::with_capacity(4);
        string.push_str("ab");
        string.push_byte(b'/');
        string.push_cstr(c"cd");
        // Past the reserved payload, so the buffer has had to grow.
        string.push_bytes(b"efghijklmnopqrstuvwxyz");
        assert_eq!(&*string, b"ab/cdefghijklmnopqrstuvwxyz");
        assert_eq!(string.as_cstr(), c"ab/cdefghijklmnopqrstuvwxyz");
        assert_eq!(string.len(), 27);
    }

    #[test]
    fn truncate_re_terminates_and_ignores_a_longer_length() {
        let mut string = XString::from_bytes(b"autocmd");
        string.truncate(4);
        assert_eq!(string.as_cstr(), c"auto");
        string.truncate(99);
        assert_eq!(string.as_cstr(), c"auto");
        string.truncate(0);
        assert!(string.is_empty());
        assert_eq!(string.as_cstr(), c"");
    }

    #[test]
    fn a_clone_is_an_independent_allocation() {
        let original = XString::from_bytes(b"first");
        let mut copy = original.clone();
        copy.push_str("-second");
        assert_eq!(original, b"first");
        assert_eq!(copy, b"first-second");
        assert_ne!(original, copy);
    }

    #[test]
    fn equality_reaches_the_shapes_the_callers_hold() {
        let string = XString::from_bytes(b"utf-8");
        assert_eq!(string, b"utf-8");
        assert_eq!(string, "utf-8");
        assert_eq!(string, b"utf-8".as_slice());
        assert_eq!(string, c"utf-8");
        assert_eq!(format!("{string:?}"), "XString(utf-8)");
    }

    /// The option layer reads a variable's address, replaces the variable,
    /// and then frees what the replace handed back — so `into_raw` must give
    /// back the block `as_ptr` named, spare capacity and all.
    #[test]
    fn into_raw_does_not_move_the_block() {
        let mut string = XString::with_capacity(64);
        string.push_str("short");
        let before = string.as_ptr();
        let raw = string.into_raw();
        assert_eq!(before, raw.cast_const());
        // SAFETY: the block `into_raw` just handed over.
        unsafe { xfree(raw.cast::<c_void>()) };
    }

    /// The round trip the perimeter exists for: a block this type gives up
    /// is one `xfree` releases, and a block `xstrdup` made is one this type
    /// adopts. Under Miri both halves are checked against the real
    /// allocator, which is why these cases live here and not in the LuaJIT
    /// unit suite.
    #[test]
    fn into_raw_hands_the_block_to_xfree() {
        let raw = XString::from_bytes(b"handover").into_raw();
        // SAFETY: `into_raw` just gave the block up, and it is terminated.
        assert_eq!(unsafe { CStr::from_ptr(raw) }, c"handover");
        // SAFETY: the same block, released once, by the allocator that made
        // it -- which is the point of the rule in `allocator.rs`.
        unsafe { xfree(raw.cast::<c_void>()) };
    }

    #[test]
    fn from_raw_adopts_an_xstrdup_block() {
        // SAFETY: a NUL-terminated literal, duplicated by the editor's own
        // allocator, then adopted exactly once.
        let mut adopted = unsafe { XString::from_raw(xstrdup(c"adopted".as_ptr())) };
        assert_eq!(adopted, b"adopted");
        // Growing it reallocates through the same allocator.
        adopted.push_str(" and grown");
        assert_eq!(adopted, b"adopted and grown");
        drop(adopted);
    }

    /// An empty `xstrdup` block is still a block, not the null pointer, and
    /// adopting it must not confuse the terminator for a missing payload.
    #[test]
    fn an_adopted_empty_string_round_trips() {
        // SAFETY: a one-byte block holding just the terminator.
        let adopted = unsafe { XString::from_raw(xstrdup(c"".as_ptr())) };
        assert!(adopted.is_empty());
        let raw = adopted.into_raw();
        // SAFETY: the block, read once and released once.
        assert_eq!(unsafe { CStr::from_ptr(raw) }, c"");
        // SAFETY: as above.
        unsafe { xfree(raw.cast::<c_void>()) };
    }

    /// Storage is byte-for-byte: an interior NUL stays in `len()` and in the
    /// deref slice, and `as_cstr` answers what a C consumer would read. The
    /// `debug_assert` in `from_bytes` is what stops this arriving by
    /// accident, so the case is built through `push_bytes`.
    #[test]
    fn an_interior_nul_is_kept_but_ends_the_c_view() {
        let mut string = XString::from_bytes(b"before");
        string.push_bytes(b"\0after");
        assert_eq!(string.len(), 12);
        assert_eq!(&*string, b"before\0after");
        assert_eq!(string.as_cstr(), c"before");
    }
}
