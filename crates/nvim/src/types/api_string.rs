//! `api/private/defs.h`'s `String`: the API's byte string.
//!
//! The rest of that header's types are plain records and live in
//! [`super::api`]. This one has behaviour -- owning, reading and releasing
//! its bytes -- so it gets its own file.
//!
//! **The string owns its bytes.** It is the [`NULL` string](String_0::NULL)
//! -- a value in its own right, which encodes as msgpack `nil` -- or a boxed
//! run of `len() + 1` bytes whose last is a NUL, which the string frees when
//! it drops. There is no borrowing spelling: a view of somebody else's bytes
//! is a `&[u8]` or a `&CStr`, and the functions that used to answer a
//! borrowing `String` copy.
//!
//! The trailing NUL is not decoration. The msgpack packer and the Lua push
//! take [`as_bytes`](String_0::as_bytes), but the editor's own consumers --
//! the option table, the hashtables a dict key ends up in, `os_*` -- read the
//! string as a C string, so every string this type makes terminates.
//! Interior NULs are still allowed and still counted in `len()`.
//!
//! The layout is a boxed slice, two words, and not C's `{ char *data; size_t
//! size; }`: nothing outside the crate reads one. The two test exports that
//! take a `String` from LuaJIT by value declare their own C-shaped argument
//! (`crate::harness`).
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::size_t;
use core::ffi::{CStr, c_char};
use core::{fmt, ptr};

/// A byte string as the API layer passes one: bytes it owns, NUL-terminated.
#[allow(
    non_camel_case_types,
    reason = "c2rust's spelling of C's `String`, which `repr(C)` used to exempt; the rename is a sweep of its own"
)]
#[derive(Default, PartialEq, Eq)]
pub struct String_0 {
    /// The bytes and the NUL after them; `None` for the null string.
    bytes: Option<Box<[u8]>>,
}

impl String_0 {
    /// The null string -- upstream's `STRING_INIT`.
    ///
    /// It is *not* the same as a zero-length string;
    /// [`is_null`](Self::is_null) tells them apart, and the API's msgpack
    /// codec answers `nil` for the first and `""` for the second.
    pub const NULL: Self = Self { bytes: None };

    /// The bytes as the `char *` a C callee takes: null for
    /// [`NULL`](Self::NULL), otherwise the string's own NUL-terminated run,
    /// valid while the string is and not to be written through.
    pub fn data(&self) -> *mut c_char {
        self.bytes
            .as_ref()
            .map_or(ptr::null_mut(), |bytes| bytes.as_ptr().cast_mut().cast())
    }

    /// The byte count. Interior NULs are included and the terminator is
    /// not: this is a byte string that happens to also be a C string.
    pub fn len(&self) -> size_t {
        self.bytes.as_ref().map_or(0, |bytes| bytes.len() - 1)
    }

    /// Whether the string has no bytes. Says nothing about nullness --
    /// [`NULL`](Self::NULL) and the empty string are both empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Whether this is the null string rather than a string of no bytes.
    pub const fn is_null(&self) -> bool {
        self.bytes.is_none()
    }

    /// The bytes, not counting the terminator. [`NULL`](Self::NULL) answers
    /// the empty slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        match &self.bytes {
            Some(bytes) => &bytes[..bytes.len() - 1],
            None => &[],
        }
    }

    /// The string as a C string, stopping at its first NUL -- the terminator
    /// this type always writes, or an interior one, as every C consumer of
    /// the bytes sees it. [`NULL`](Self::NULL) answers `c""`.
    pub fn as_cstr(&self) -> &CStr {
        match &self.bytes {
            Some(bytes) => CStr::from_bytes_until_nul(bytes).expect("a String ends in a NUL"),
            None => c"",
        }
    }

    /// A copy of `bytes`, NUL-terminated, owned by the answer.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut owned = Vec::with_capacity(bytes.len() + 1);
        owned.extend_from_slice(bytes);
        owned.push(0);
        Self {
            bytes: Some(owned.into_boxed_slice()),
        }
    }

    /// `bytes` as a string, terminated in place.
    pub fn from_vec(mut bytes: Vec<u8>) -> Self {
        bytes.push(0);
        Self {
            bytes: Some(bytes.into_boxed_slice()),
        }
    }

    /// A copy of `str`'s bytes, stopping at its terminator.
    pub fn from_cstr(str: &CStr) -> Self {
        Self {
            bytes: Some(str.to_bytes_with_nul().into()),
        }
    }

    /// Take over an [`XString`](crate::memory::XString)'s buffer: the two
    /// hold the same bytes, terminator included.
    pub fn from_xstring(string: crate::memory::XString) -> Self {
        Self {
            bytes: Some(string.into_vec().into_boxed_slice()),
        }
    }

    /// Give the bytes up as an allocation a C consumer releases with
    /// `xfree` (the Rust allocator is `malloc`'s); null for
    /// [`NULL`](Self::NULL).
    pub fn into_raw(self) -> *mut c_char {
        self.bytes.map_or(ptr::null_mut(), |bytes| {
            Box::into_raw(bytes).cast::<c_char>()
        })
    }
}

impl Clone for String_0 {
    /// A copy with its own allocation. [`String_0::NULL`] stays null rather
    /// than becoming the empty string.
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
        }
    }
}

impl From<&CStr> for String_0 {
    /// [`from_cstr`](String_0::from_cstr): a copy of `s`'s bytes.
    fn from(s: &CStr) -> Self {
        Self::from_cstr(s)
    }
}

impl From<&[u8]> for String_0 {
    fn from(bytes: &[u8]) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<&str> for String_0 {
    fn from(text: &str) -> Self {
        Self::from_bytes(text.as_bytes())
    }
}

impl fmt::Debug for String_0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.bytes {
            None => f.write_str("String_0(NULL)"),
            Some(_) => write!(f, "String_0({:?})", self.as_cstr()),
        }
    }
}
