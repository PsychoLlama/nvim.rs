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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkMove {
    pub start_row: ::core::ffi::c_int,
    pub start_col: ::core::ffi::c_int,
    pub extent_row: ::core::ffi::c_int,
    pub extent_col: ::core::ffi::c_int,
    pub new_row: ::core::ffi::c_int,
    pub new_col: ::core::ffi::c_int,
    pub start_byte: bcount_t,
    pub extent_byte: bcount_t,
    pub new_byte: bcount_t,
}
pub type ExtmarkOp = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkSavePos {
    pub mark: uint64_t,
    pub old_row: ::core::ffi::c_int,
    pub old_col: colnr_T,
    pub invalidated: bool,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ExtmarkSplice {
    pub start_row: ::core::ffi::c_int,
    pub start_col: colnr_T,
    pub old_row: ::core::ffi::c_int,
    pub old_col: colnr_T,
    pub new_row: ::core::ffi::c_int,
    pub new_col: colnr_T,
    pub start_byte: bcount_t,
    pub old_byte: bcount_t,
    pub new_byte: bcount_t,
}
pub type ExtmarkType = ::core::ffi::c_uint;
pub type UndoObjectType = ::core::ffi::c_uint;
pub type bcount_t = ptrdiff_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct extmark_undo_vec_t {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut ExtmarkUndoObject,
}
/// One recorded extmark operation, as a buffer's undo list holds it.
///
/// Upstream is a `type` tag beside an untagged union. The tag's numbers are
/// what the undo *file* writes, so they stay reachable as
/// [`ExtmarkUndoObject::wire_type`]; nothing else needs them.
#[derive(Copy, Clone)]
pub enum ExtmarkUndoObject {
    /// A text change: every operation that moves marks except `:move`.
    Splice(ExtmarkSplice),
    /// A `:move`.
    Move(ExtmarkMove),
    /// Where one mark was before the change deleted it outright. Not
    /// written to an undo file: it names marks of a live buffer.
    SavePos(ExtmarkSavePos),
}

impl ExtmarkUndoObject {
    /// The number the undo file writes for this kind of record --
    /// upstream's `kExtmarkSplice`, `kExtmarkMove` and `kExtmarkSavePos`.
    /// These are a file format, so they are literals here rather than a
    /// name that could be renumbered.
    pub fn wire_type(&self) -> UndoObjectType {
        match self {
            ExtmarkUndoObject::Splice(_) => 0,
            ExtmarkUndoObject::Move(_) => 1,
            ExtmarkUndoObject::SavePos(_) => 3,
        }
    }
}
