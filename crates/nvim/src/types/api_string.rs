//! `api/private/defs.h`'s `String`: the API's byte string.
//!
//! The rest of that header's types are plain records and live in
//! [`super::api`], which forbids `unsafe`. This one has behaviour --
//! reading the bytes it points at -- so it gets its own file.
//!
//! **The layout is pinned and cannot move.** `String` is embedded by value
//! in `Object`'s union, `KeyValuePair`, `HlMessageChunk`, `StringArray` and
//! every `KeyDict_*`; it is what the msgpack-RPC codec and the Lua
//! converter serialise; `tools/ffigen` emits it into `unit-cdefs.h` as
//! `struct String { char *data; size_t size; }`; and `test/unit/api/`
//! reads `.data`/`.size` off it through LuaJIT's FFI. So the two words stay
//! exactly where they are, `#[repr(C)]`, in that order.
//!
//! What *can* change, and did, is that they are no longer public: the pair
//! is reached through constructors and accessors, so "how do I make an
//! empty one", "how long is it" and "what are its bytes" have one answer
//! each instead of a hand-written `{ data, size }` at every site.
//!
//! **Ownership is not part of the type.** A `String` may own its bytes or
//! borrow them -- `api::private::helpers`' `*_to_string` copies and
//! `*_as_string` borrows -- and nothing here says which, exactly as
//! upstream. That is why there is no `Drop`.
//!
//! `types/` forbids `unsafe`, so the one accessor that has to dereference
//! the pointer -- [`String_0::as_bytes`] -- lives in a second `impl` block
//! in [`crate::api::private::helpers`], next to the functions that build
//! these strings and document who owns their bytes.
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

/// A byte string as the API layer passes one: a pointer and a length.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct String_0 {
    data: *mut c_char,
    size: size_t,
}

impl String_0 {
    /// The empty string, owning nothing -- upstream's `STRING_INIT`, which
    /// this tree had re-declared in five modules.
    ///
    /// Its pointer is null, so it is *not* the same as a zero-length string
    /// with a real buffer; [`is_null`](Self::is_null) tells them apart, and
    /// the API's msgpack codec answers `nil` for the first and `""` for the
    /// second.
    pub const NULL: Self = Self {
        data: ptr::null_mut(),
        size: 0,
    };

    /// The pinned pair, for the FFI edges that genuinely hold one: a
    /// pointer and the number of bytes readable at it.
    ///
    /// Prefer [`from_cstr`](Self::from_cstr) or one of
    /// `api::private::helpers`' `*_to_string`/`*_as_string` functions --
    /// they say whether the result owns its bytes.
    pub const fn from_raw_parts(data: *mut c_char, size: size_t) -> Self {
        Self { data, size }
    }

    /// A view of `s`, borrowing its bytes and stopping at its terminator.
    pub const fn from_cstr(s: &CStr) -> Self {
        Self {
            data: s.as_ptr().cast_mut(),
            size: s.count_bytes(),
        }
    }

    /// The pointer. Null for [`NULL`](Self::NULL); otherwise whatever the
    /// producer put there, which may or may not be owned.
    pub const fn data(self) -> *mut c_char {
        self.data
    }

    /// The byte count. Interior NULs are included: this is a byte string,
    /// not a C string, even where the producer also terminated it.
    pub const fn len(self) -> size_t {
        self.size
    }

    /// Whether the string has no bytes. Says nothing about the pointer --
    /// [`NULL`](Self::NULL) and a zero-length buffer are both empty.
    pub const fn is_empty(self) -> bool {
        self.size == 0
    }

    /// Whether this is the null string rather than a string of no bytes.
    pub const fn is_null(self) -> bool {
        self.data.is_null()
    }

    /// The place the length lives, for the C callees that fill a `String`'s
    /// two words separately (`lua_tolstring`, `luaL_checklstring`).
    pub const fn len_mut(&mut self) -> &mut size_t {
        &mut self.size
    }

    /// The place the pointer lives, for the C callees that fill it
    /// (`expand_wildcards`, `get_spec_reg`).
    pub const fn data_mut(&mut self) -> &mut *mut c_char {
        &mut self.data
    }

    /// Both words' addresses at once, for the C callees that take a
    /// `char **` and a `size_t *` and fill them together
    /// (`encode_vim_list_to_buf`). One `&mut` produces both, which two
    /// separate accessor calls cannot.
    pub const fn parts_mut(&mut self) -> (*mut *mut c_char, *mut size_t) {
        (&raw mut self.data, &raw mut self.size)
    }

    /// Point the string at `data`, keeping the length.
    pub const fn set_data(&mut self, data: *mut c_char) {
        self.data = data;
    }

    /// Set the length, keeping the pointer.
    pub const fn set_len(&mut self, size: size_t) {
        self.size = size;
    }
}

impl From<&CStr> for String_0 {
    /// [`from_cstr`](String_0::from_cstr): a view of `s`, borrowing its
    /// bytes.
    fn from(s: &CStr) -> Self {
        Self::from_cstr(s)
    }
}

impl fmt::Debug for String_0 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.data.is_null() {
            return f.write_str("String_0(NULL)");
        }
        // The bytes are not read: a `String` may point at a freed or
        // partially built buffer, and a debug print must not be the thing
        // that touches it.
        write!(f, "String_0({:p}, {})", self.data, self.size)
    }
}
