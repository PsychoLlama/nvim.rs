#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
use crate::types::{ColNr, Handle, LineNr, Pos, Timestamp};

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
#[derive(Clone, Debug)]
pub struct FileMark {
    pub mark: Pos,
    /// The buffer the mark is in, `Handle::NONE` for none: read it as
    /// [`FileMark::fnum`]. Not an `Option<Handle>`, which has no niche and
    /// would add a word to every mark of every store (+0.26 % Ir on
    /// undobench, in the copies and `mark_adjust`'s per-mark test).
    pub(crate) in_buffer: Handle,
    pub timestamp: Timestamp,
    pub view: FileMarkView,
    /// The ShaDa entry's keys this Nvim has no field for, written back as
    /// read. Boxed: a buffer is a zeroed allocation holding a hundred and
    /// thirty of these, and only a `Sized` box's `None` is promised to be
    /// all-zero bytes -- which also keeps the record five words.
    pub(crate) extra: Option<Box<crate::shada::ShadaExtra>>,
}
#[derive(Copy, Clone, Debug, Default)]
pub struct FileMarkView {
    pub topline_offset: i32,
    pub skipcol: ColNr,
}

impl FileMarkView {
    /// The view an unset mark carries: `topline_offset` at `MAXLNUM` means
    /// "remember nothing", so `mark_view_restore` computes a topline far
    /// below line 1 and gives up.
    pub const NONE: Self = Self {
        topline_offset: crate::pos::MAXLNUM.get(),
        skipcol: ColNr::ZERO,
    };
}

impl FileMark {
    /// The buffer the mark is in; `None` for a buffer-local mark, which is
    /// in the buffer that keeps it.
    #[inline]
    pub fn fnum(&self) -> Option<Handle> {
        Handle::from_number(self.in_buffer.get())
    }

    /// Point the mark at `fnum`'s buffer, or at none.
    #[inline]
    pub fn set_fnum(&mut self, fnum: Option<Handle>) {
        self.in_buffer = fnum.unwrap_or(Handle::NONE);
    }

    /// Whether the mark is in `buffer`: [`FileMark::fnum`]` == Some(buffer)`
    /// as one comparison.
    #[inline]
    pub fn is_in(&self, buffer: Handle) -> bool {
        self.in_buffer == buffer
    }

    /// A mark that is not set.
    pub const UNSET: Self = Self {
        mark: Pos {
            lnum: LineNr::ZERO,
            col: ColNr::ZERO,
            coladd: 0,
        },
        in_buffer: Handle::NONE,
        timestamp: 0,
        view: FileMarkView::NONE,
        extra: None,
    };

    /// Whether the mark names a line. Line 0 is how every store spells "not
    /// set", and what `mark_check` turns into `E20`.
    #[inline]
    pub fn is_set(&self) -> bool {
        self.mark.lnum != LineNr::ZERO
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

/// A [`FileMark`] without its ShaDa extra data: what an undo header keeps of
/// a named mark, to put back when the change is undone. `Copy`, because a
/// header is saved per change and a deep copy of twenty-six marks each time
/// costs more than the save.
#[derive(Copy, Clone, Debug, Default)]
pub struct SavedMark {
    pub mark: Pos,
    /// [`FileMark::fnum`] with `Handle::NONE` for `None`: an
    /// `Option<Handle>` has no niche and would add a word to each of the
    /// twenty-six marks every undo header saves (+0.5 % Ir on undobench).
    pub(crate) fnum: Handle,
    pub timestamp: Timestamp,
    pub view: FileMarkView,
}

impl SavedMark {
    /// A mark that is not set, with every field zero.
    pub const ZERO: Self = Self {
        mark: Pos {
            lnum: LineNr::ZERO,
            col: ColNr::ZERO,
            coladd: 0,
        },
        fnum: Handle::NONE,
        timestamp: 0,
        view: FileMarkView {
            topline_offset: 0,
            skipcol: ColNr::ZERO,
        },
    };

    /// What `fm` holds, but its extra data.
    #[inline]
    pub fn of(fm: &FileMark) -> Self {
        Self {
            mark: fm.mark,
            fnum: fm.in_buffer,
            timestamp: fm.timestamp,
            view: fm.view,
        }
    }

    /// The mark it saved, carrying no extra data.
    #[inline]
    pub fn to_mark(self) -> FileMark {
        FileMark {
            mark: self.mark,
            in_buffer: self.fnum,
            timestamp: self.timestamp,
            view: self.view,
            extra: None,
        }
    }
}

/// A [`FileMark`] that may name a file instead of a buffer: the global marks
/// and the jump list, whose entries read out of a ShaDa file carry the file
/// name until a buffer for it is loaded and the name becomes a number.
///
/// Not `Copy`, for `fname` on top of [`FileMark`]'s own; `Clone` is deep.
/// The name is one word wide and its `None` is all-zero bytes: a window is a
/// zeroed allocation holding a hundred of these.
#[derive(Clone, Debug)]
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
