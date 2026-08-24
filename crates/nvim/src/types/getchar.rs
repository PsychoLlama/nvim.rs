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

pub type RemapValues = ::core::ffi::c_int;
/// One block of a [`buffheader_T`]'s byte string.
///
/// The layout is pinned, and load-bearing: `b_str` is a flexible array member
/// — the type declares one byte, the allocation holds as many as the block was
/// sized for, and `getchar::buffers::add_buff` sizes it as
/// `offset_of!(buffblock_T, b_str) + len + 1`. That arithmetic only describes
/// the allocation when `b_str` is the *last* field, so every append past the
/// first byte would otherwise land on another field. `#[repr(C)]` is what
/// guarantees declaration order here; `add_buff` carries the matching
/// compile-time assertion.
///
/// `Copy` stays, and it is narrower than it looks: the only block ever
/// copied by value is [`buffheader_T::bh_first`], the inline head sentinel,
/// whose flexible tail is the one declared byte and is never written. Every
/// block that carries text is reached as `*mut buffblock_T` and lives for
/// exactly as long as its allocation.
#[derive(Copy, Clone)]
#[repr(C)]
pub struct buffblock {
    pub b_next: *mut buffblock,
    pub b_strlen: size_t,
    pub b_str: [::core::ffi::c_char; 1],
}
pub type buffblock_T = buffblock;
#[derive(Copy, Clone)]
pub struct buffheader_T {
    pub bh_first: buffblock_T,
    pub bh_curr: *mut buffblock_T,
    pub bh_index: size_t,
    pub bh_space: size_t,
    pub bh_create_newblock: bool,
}

impl buffheader_T {
    /// The state all five buffers start in, and the one `GlobalCell::take`
    /// leaves behind: no blocks, so nothing to free. A `const` as well as a
    /// [`Default`], because the cells themselves are `static`s.
    pub const EMPTY: Self = buffheader_T {
        bh_first: buffblock_T {
            b_next: ::core::ptr::null_mut(),
            b_strlen: 0,
            b_str: [0],
        },
        bh_curr: ::core::ptr::null_mut(),
        bh_index: 0,
        bh_space: 0,
        bh_create_newblock: false,
    };
}

impl Default for buffheader_T {
    fn default() -> Self {
        buffheader_T::EMPTY
    }
}
pub type flush_buffers_T = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
pub struct save_redo_T {
    pub sr_redobuff: buffheader_T,
    pub sr_old_redobuff: buffheader_T,
}
#[derive(Copy, Clone)]
pub struct tasave_T {
    pub save_typebuf: typebuf_T,
    pub typebuf_valid: bool,
    pub old_char: ::core::ffi::c_int,
    pub old_mod_mask: ::core::ffi::c_int,
    pub save_readbuf1: buffheader_T,
    pub save_readbuf2: buffheader_T,
    pub save_inputbuf: String_0,
}
#[derive(Copy, Clone)]
pub struct typebuf_T {
    pub tb_buf: *mut uint8_t,
    pub tb_noremap: *mut uint8_t,
    pub tb_buflen: ::core::ffi::c_int,
    pub tb_off: ::core::ffi::c_int,
    pub tb_len: ::core::ffi::c_int,
    pub tb_maplen: ::core::ffi::c_int,
    pub tb_silent: ::core::ffi::c_int,
    pub tb_no_abbr_cnt: ::core::ffi::c_int,
    pub tb_change_cnt: ::core::ffi::c_int,
}

impl typebuf_T {
    /// A typeahead buffer with no storage at all: what `init_typebuf` looks
    /// for when it decides to hand out the static initial buffers, and what
    /// `GlobalCell::take` leaves behind.
    pub const EMPTY: Self = typebuf_T {
        tb_buf: ::core::ptr::null_mut(),
        tb_noremap: ::core::ptr::null_mut(),
        tb_buflen: 0,
        tb_off: 0,
        tb_len: 0,
        tb_maplen: 0,
        tb_silent: 0,
        tb_no_abbr_cnt: 0,
        tb_change_cnt: 0,
    };
}

impl Default for typebuf_T {
    fn default() -> Self {
        typebuf_T::EMPTY
    }
}
