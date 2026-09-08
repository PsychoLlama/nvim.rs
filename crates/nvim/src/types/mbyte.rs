#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// How far the character under a byte starts before it and ends after it.
///
/// `Copy`: two offsets, and nothing else.
pub struct CharBoundsOff {
    pub begin_off: int8_t,
    pub end_off: int8_t,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct CharInfo {
    pub value: int32_t,
    pub len: ::core::ffi::c_int,
}
pub type GraphemeState = utf8proc_int32_t;
/// A character of some text, paired with the text from it onward.
///
/// The cursor a `MB_PTR_ADV` walk carries. Stepping it is subslicing `rest`,
/// so "how far in are we" is `line.len() - rest.len()` rather than pointer
/// arithmetic, and the walk cannot leave the text it was given.
#[derive(Copy, Clone)]
pub struct StrChar<'a> {
    /// The text from this character to the end.
    pub rest: &'a [u8],
    /// The codepoint, or a **negative** number for a byte that is not a
    /// character. Zero at the end of the text.
    pub value: int32_t,
    /// How many bytes of [`rest`](Self::rest) the character occupies -- one
    /// for a byte that is not a character, so a walk always advances.
    pub len: usize,
}

impl StrChar<'_> {
    /// Whether the walk has run out of text.
    #[inline(always)]
    pub fn at_end(self) -> bool {
        self.rest.is_empty()
    }

    /// The address this character starts at, for the measures that still
    /// take a pointer into a NUL-terminated line.
    ///
    /// **Read-only.** The `*mut` is what those measures' transpiled
    /// signatures ask for, not permission: the address is derived from a
    /// shared borrow, so writing through it is undefined.
    #[inline(always)]
    pub fn address(self) -> *mut ::core::ffi::c_char {
        self.rest.as_ptr().cast::<::core::ffi::c_char>().cast_mut()
    }
}
/// Not `Copy`: `vc_fd` is an iconv descriptor that has to be closed once.
#[derive(Clone)]
pub struct VimConv {
    pub vc_type: ::core::ffi::c_int,
    pub vc_factor: ::core::ffi::c_int,
    pub vc_fd: iconv_t,
    pub vc_fail: bool,
}

/// The most bytes one multi-byte character can occupy: a 16-bit character of
/// up to three bytes plus six composing characters of three bytes each, or a
/// 32-bit character of up to six.
///
/// `usize` because most of its uses are array lengths; the transpiled sites
/// spell the cast out.
pub const MB_MAXBYTES: usize = 21;

/// The most bytes one *character* — a base plus its composing marks — can
/// occupy in the places that only need to round-trip a single character:
/// six, one over the longest legal UTF-8 sequence `utf_char2bytes` writes.
///
/// `usize`, as `MB_MAXBYTES` is, because every use is an array length. Three
/// modules had grown a private copy.
pub const MB_MAXCHAR: usize = 6;

/// `VimConv::vc_type` — upstream's `ConvFlags`.
///
/// `c_int`, which is what the `vc_type` field is: c2rust typed the anonymous
/// enum `c_uint` from what the C compiler picked, and every one of the 55 use
/// sites cast it back. B15-9 deleted the casts with the retype.
pub type ConvFlags = ::core::ffi::c_int;

pub const CONV_NONE: ConvFlags = 0;
pub const CONV_TO_UTF8: ConvFlags = 1;
pub const CONV_9_TO_UTF8: ConvFlags = 2;
pub const CONV_TO_LATIN1: ConvFlags = 3;
pub const CONV_TO_LATIN9: ConvFlags = 4;
pub const CONV_ICONV: ConvFlags = 5;

/// A `VimConv` that converts nothing — what `convert_setup` starts from and
/// what a caller with no conversion to do passes around.
pub const CONV_NONE_INIT: VimConv = VimConv {
    vc_type: CONV_NONE,
    vc_factor: 1,
    vc_fd: ::core::ptr::null_mut(),
    vc_fail: false,
};
