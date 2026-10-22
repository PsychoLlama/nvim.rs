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

pub type MarkAdjustMode = ::core::ffi::c_uint;
pub type MarkGet = ::core::ffi::c_uint;
pub type MarkMove = ::core::ffi::c_uint;
pub type MarkMoveRes = ::core::ffi::c_uint;
/// A position a mark store keeps: where, in which buffer, when it was set,
/// the view around it, and what the ShaDa file said about it that this Nvim
/// has no field for.
///
/// Not `Copy`, because `extra` is owned: `Clone` is deep and dropping a
/// record releases it. A store's record is replaced by assignment; nothing
/// frees one by hand.
#[derive(Clone)]
pub struct FileMark {
    pub mark: Pos,
    pub fnum: ::core::ffi::c_int,
    pub timestamp: Timestamp,
    pub view: FileMarkView,
    /// The ShaDa entry's keys this Nvim has no field for, written back as
    /// read. Boxed: a buffer is a zeroed allocation holding a hundred and
    /// thirty of these, and only a `Sized` box's `None` is promised to be
    /// all-zero bytes -- which also keeps the record five words.
    pub(crate) extra: Option<Box<crate::shada::ShadaExtra>>,
}
#[derive(Copy, Clone)]
pub struct FileMarkView {
    pub topline_offset: LineNr,
    pub skipcol: ColNr,
}

impl FileMarkView {
    /// The view an unset mark carries: `topline_offset` at `MAXLNUM` means
    /// "remember nothing", so `mark_view_restore` computes a topline far
    /// below line 1 and gives up.
    pub const NONE: Self = Self {
        topline_offset: crate::pos::MAXLNUM,
        skipcol: 0,
    };
}

impl FileMark {
    /// A mark that is not set.
    pub const UNSET: Self = Self {
        mark: Pos {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        fnum: 0,
        timestamp: 0,
        view: FileMarkView::NONE,
        extra: None,
    };

    /// Whether the mark names a line. Line 0 is how every store spells "not
    /// set", and what `mark_check` turns into `E20`.
    #[inline]
    pub fn is_set(&self) -> bool {
        self.mark.lnum != 0
    }

    /// Release what the record holds and leave it unset, stamped
    /// `timestamp` so a later ShaDa merge can tell the clearing from the
    /// records it is merging: upstream's `clear_fmark`.
    pub fn clear(&mut self, timestamp: Timestamp) {
        *self = Self {
            timestamp,
            ..Self::UNSET
        };
    }
}

/// A [`FileMark`] that may name a file instead of a buffer: the global marks
/// and the jump list, whose entries read out of a ShaDa file carry the file
/// name until a buffer for it is loaded and the name becomes a number.
///
/// Not `Copy`, for `fname` on top of [`FileMark`]'s own; `Clone` is deep.
/// The name is one word wide and its `None` is all-zero bytes: a window is a
/// zeroed allocation holding a hundred of these.
#[derive(Clone)]
pub struct XFileMark {
    pub fmark: FileMark,
    pub fname: Option<crate::memory::ThinCString>,
}

impl XFileMark {
    /// An entry that is not set.
    pub const UNSET: Self = Self {
        fmark: FileMark::UNSET,
        fname: None,
    };

    /// The file name a mark read out of the shada file carries until its
    /// buffer is loaded, if it carries one.
    #[inline]
    pub fn file_name(&self) -> Option<&::core::ffi::CStr> {
        self.fname.as_ref().map(crate::memory::ThinCString::as_cstr)
    }
}
