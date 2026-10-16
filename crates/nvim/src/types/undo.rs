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
use crate::memory::XString;

/// One undo entry: the lines a change replaced, and where they went.
///
/// The entry says "lines `top + 1 .. bot - 1` of the buffer used to be
/// `lines`". Applying it swaps the two, so the same entry put back moves the
/// buffer forward again.
#[derive(Clone, Default)]
pub struct UndoEntry {
    /// The line above the change; 0 for a change at the top of the buffer.
    pub top: LineNr,
    /// The line below the change, or 0 for "the end of the buffer" -- a
    /// sentinel `u_savecommon` writes when the change ran past the last line,
    /// resolved against the buffer as it stands when the entry is applied.
    pub bot: LineNr,
    /// The buffer's line count when the bottom was left to be worked out:
    /// what `u_getbot` measures the change's growth against.
    pub lcount: LineNr,
    /// The saved lines, oldest-first, each its own copy.
    pub lines: Vec<XString>,
}

impl UndoEntry {
    /// How many lines the entry holds, in the unit the line numbers use.
    pub fn size(&self) -> LineNr {
        LineNr::try_from(self.lines.len()).unwrap_or(LineNr::MAX)
    }
}

/// A link from one undo header to another, or to nothing.
///
/// The link *is* the target header's `uh_seq`, which is also exactly what the
/// undo file stores: `put_header_ptr` writes this number and
/// `unserialize_uhp` reads it straight back. In-memory the number is resolved
/// against the buffer's `UndoStore`, which is keyed by the same number, so
/// there is one representation instead of the "sequence number on disk,
/// pointer in memory" union this type replaced.
///
/// Sequence numbers a buffer hands out start at 1 (`b_u_seq_last` is
/// pre-incremented) and a header read from a file with `uh_seq <= 0` is
/// rejected as corrupt, so **zero is free to mean "no link"** and every
/// `UndoLink` this type builds is either zero or positive.
#[derive(Copy, Clone, PartialEq, Eq, Default, Debug, Hash)]
pub struct UndoLink {
    /// The target's `uh_seq`, or 0 for no target.
    seq: ::core::ffi::c_int,
}

impl UndoLink {
    /// No target: what a NULL `uh_*` pointer used to be, and the 0 the undo
    /// file carries for one.
    pub const NONE: Self = Self { seq: 0 };

    /// The link naming sequence number `seq`.
    ///
    /// Anything that is not a number a buffer could have handed out — zero
    /// and the negatives — is [`NONE`](Self::NONE), which is how a corrupt
    /// undo file's link field lands as "no link" rather than as a wild
    /// lookup.
    pub const fn to_seq(seq: ::core::ffi::c_int) -> Self {
        if seq > 0 { Self { seq } } else { Self::NONE }
    }

    /// The sequence number this link names, or 0. This is the number that
    /// goes into the undo file.
    pub const fn seq(self) -> ::core::ffi::c_int {
        self.seq
    }

    /// Whether the link names no header at all.
    pub const fn is_none(self) -> bool {
        self.seq == 0
    }

    /// Whether the link names some header.
    pub const fn is_some(self) -> bool {
        self.seq != 0
    }
}

/// One change: the entries it saved, where the cursor was, and how it sits
/// in the tree.
///
/// Owned by the buffer's [`UndoStore`](crate::undo::store::UndoStore), which
/// a [`UndoLink`] names it through. Dropping one drops everything it saved.
#[derive(Clone)]
pub struct UndoHeader {
    /// The change before this one on the same branch (older).
    pub next: UndoLink,
    /// The change after this one on the same branch (newer).
    pub prev: UndoLink,
    /// The next alternate branch forking at the same place.
    pub alt_next: UndoLink,
    /// The previous alternate branch forking at the same place.
    pub alt_prev: UndoLink,
    /// The header's name: its sequence number, which [`UndoLink`] is.
    pub seq: ::core::ffi::c_int,
    /// The stamp the last tree walk left; see `store::Marks`.
    pub walk: ::core::ffi::c_int,
    /// The entries, **newest last**: upstream's list pushed at its head, and
    /// the end of a vector is where a push costs nothing, so the list's
    /// order is this vector read backwards.
    pub entries: Vec<UndoEntry>,
    /// The index in `entries` of the one whose `bot` `u_getbot` has yet to
    /// work out, if any.
    pub getbot: Option<usize>,
    pub cursor: Pos,
    pub cursor_vcol: ColNr,
    pub flags: ::core::ffi::c_int,
    pub named_marks: [FileMark; 26],
    /// What the change did to extmarks, in the order it did it.
    pub extmarks: Vec<ExtmarkUndoObject>,
    pub visual: VisualInfo,
    pub time: time_t,
    pub save_nr: ::core::ffi::c_int,
}

impl UndoHeader {
    /// The link that names this header.
    pub fn link(&self) -> UndoLink {
        UndoLink::to_seq(self.seq)
    }

    /// The entries in upstream's list order: newest first.
    pub fn entries_newest_first(&self) -> impl Iterator<Item = &UndoEntry> {
        self.entries.iter().rev()
    }
}

impl Default for UndoHeader {
    /// A header linked to nothing, holding nothing, with every other field
    /// zero.
    fn default() -> Self {
        // A `const`, not a `let`: `FileMark` is not `Copy`, and only a
        // constant may be repeated into an array without it.
        const UNSET_MARK: FileMark = FileMark {
            mark: Pos {
                lnum: 0,
                col: 0,
                coladd: 0,
            },
            fnum: 0,
            timestamp: 0,
            view: FileMarkView {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut(),
        };
        Self {
            next: UndoLink::NONE,
            prev: UndoLink::NONE,
            alt_next: UndoLink::NONE,
            alt_prev: UndoLink::NONE,
            seq: 0,
            walk: 0,
            entries: Vec::new(),
            getbot: None,
            cursor: Pos::default(),
            cursor_vcol: 0,
            flags: 0,
            named_marks: [UNSET_MARK; 26],
            extmarks: Vec::new(),
            visual: VisualInfo {
                vi_start: Pos::default(),
                vi_end: Pos::default(),
                vi_mode: 0,
                vi_curswant: 0,
            },
            time: 0,
            save_nr: 0,
        }
    }
}
#[derive(Copy, Clone)]
pub struct VisualInfo {
    pub vi_start: Pos,
    pub vi_end: Pos,
    pub vi_mode: ::core::ffi::c_int,
    pub vi_curswant: ColNr,
}
