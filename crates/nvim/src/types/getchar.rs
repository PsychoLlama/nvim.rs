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
use crate::getchar::KeyBuffer;

pub type RemapValues = ::core::ffi::c_int;
pub type flush_buffers_T = ::core::ffi::c_uint;
/// The redo pair a user function or autocommand set aside. Not `Copy`: each
/// field owns a block chain, and `save_redobuff` *moves* them here.
/// [`Default`] is the "nothing saved yet" state its callers declare it in --
/// `mem::zeroed` is not usable on a struct holding an enum with a niche.
#[derive(Default)]
pub struct save_redo_T {
    pub(crate) sr_redobuff: KeyBuffer,
    pub(crate) sr_old_redobuff: KeyBuffer,
}
/// All three kinds of typeahead, set aside so that a prompt has to be
/// answered by the user. Not `Copy`, and `Default` rather than zeroed, for
/// the same reasons as [`save_redo_T`].
#[derive(Default)]
pub struct tasave_T {
    pub(crate) save_typebuf: typebuf_T,
    pub(crate) typebuf_valid: bool,
    pub(crate) old_char: ::core::ffi::c_int,
    pub(crate) old_mod_mask: ::core::ffi::c_int,
    pub(crate) save_readbuf1: KeyBuffer,
    pub(crate) save_readbuf2: KeyBuffer,
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
