//! `api/private/defs.h`'s `String`: the API's byte string.
//!
//! The rest of that header's types are plain records and live in
//! [`super::api`], which forbids `unsafe`. This one has behaviour --
//! reading and releasing the bytes it points at -- so it gets its own file.
//!
//! **The layout is two words and stays two words.** `String` is embedded by
//! value in [`Object`](super::Object)'s payload, `KeyValuePair`,
//! `HlMessageChunk`, `StringArray` and every `KeyDict_*`; it is what the
//! msgpack-RPC codec and the Lua converter serialise; and `tools/ffigen`
//! emits it into `unit-cdefs.h` as `struct String { char *data; size_t
//! size; }`. So the pair stays `#[repr(C)]`, in that order.
//!
//! **The string owns its bytes.** `data` is null -- the [`NULL`
//! string](String_0::NULL), which is a value in its own right and encodes as
//! msgpack `nil` -- or an `xmalloc`ed block of `size + 1` bytes with a NUL
//! at `data[size]`, which the string frees when it drops. There is no
//! borrowing spelling: a view of somebody else's bytes is a `&[u8]` or a
//! `&CStr`, and the functions that used to answer a borrowing `String` now
//! copy.
//!
//! The trailing NUL is not decoration. The msgpack packer and the Lua push
//! take `size` bytes, but the editor's own consumers -- the option table,
//! the hashtables a dict key ends up in, `os_*` -- read `data` as a C
//! string, so every string this type makes terminates. Interior NULs are
//! still allowed and still counted in `size`.
//!
//! `types/` forbids `unsafe`, so the halves that have to touch the pointer
//! -- the constructors that allocate, [`Drop`], [`Clone`] and the byte
//! accessors -- live in a second `impl` block in
//! [`crate::api::private::helpers`], next to the functions that build these
//! strings.
#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::size_t;
use core::ffi::c_char;
use core::{fmt, ptr};

/// A byte string as the API layer passes one: a pointer to bytes it owns,
/// and their length.
#[repr(C)]
pub struct String_0 {
    data: *mut c_char,
    size: size_t,
}

impl String_0 {
    /// The null string -- upstream's `STRING_INIT`, which this tree had
    /// re-declared in five modules.
    ///
    /// Its pointer is null, so it is *not* the same as a zero-length string
    /// with a real buffer; [`is_null`](Self::is_null) tells them apart, and
    /// the API's msgpack codec answers `nil` for the first and `""` for the
    /// second.
    pub const NULL: Self = Self {
        data: ptr::null_mut(),
        size: 0,
    };

    /// The pointer. Null for [`NULL`](Self::NULL); otherwise the string's
    /// own allocation, readable for `len() + 1` bytes.
    pub const fn data(&self) -> *mut c_char {
        self.data
    }

    /// The byte count. Interior NULs are included and the terminator is
    /// not: this is a byte string that happens to also be a C string.
    pub const fn len(&self) -> size_t {
        self.size
    }

    /// Whether the string has no bytes. Says nothing about the pointer --
    /// [`NULL`](Self::NULL) and a zero-length buffer are both empty.
    pub const fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// Whether this is the null string rather than a string of no bytes.
    pub const fn is_null(&self) -> bool {
        self.data.is_null()
    }

    /// Take the allocation out of the string, leaving it
    /// [`NULL`](Self::NULL). The caller frees it with `xfree`.
    ///
    /// For the handful of edges that hand a buffer to a C consumer which
    /// takes it over (`ui_call_set_title`'s spec double, the option table).
    pub fn into_raw(self) -> *mut c_char {
        // `Self` has a destructor, so it cannot be destructured; the husk
        // is what keeps the pointer from being freed on the way out.
        core::mem::ManuallyDrop::new(self).data
    }

    /// The two words at once, for the C callees that take a `char **` and a
    /// `size_t *` and fill them together (`encode_vim_list_to_buf`,
    /// `luaL_checklstring`).
    ///
    /// Producing the addresses is safe; what the callee stores through them
    /// must be an `xmalloc`ed block of `*size + 1` bytes, NUL-terminated,
    /// which the string then owns -- so the caller takes the old block out
    /// first, or leaks it.
    pub const fn parts_mut(&mut self) -> (*mut *mut c_char, *mut size_t) {
        (&raw mut self.data, &raw mut self.size)
    }
}

impl Default for String_0 {
    /// [`NULL`](String_0::NULL).
    fn default() -> Self {
        Self::NULL
    }
}

impl fmt::Debug for String_0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.data.is_null() {
            return f.write_str("String_0(NULL)");
        }
        // The bytes are not read: a `String` may point at a partially built
        // buffer, and a debug print must not be the thing that touches it.
        write!(f, "String_0({:p}, {})", self.data, self.size)
    }
}
