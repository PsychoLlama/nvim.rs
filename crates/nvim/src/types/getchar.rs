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
use crate::getchar::{KeyBuffer, TypeAhead};

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
    pub(crate) save_typebuf: TypeAhead,
    pub(crate) typebuf_valid: bool,
    pub(crate) old_char: ::core::ffi::c_int,
    pub(crate) old_mod_mask: crate::keycodes::ModMask,
    pub(crate) save_readbuf1: KeyBuffer,
    pub(crate) save_readbuf2: KeyBuffer,
}
