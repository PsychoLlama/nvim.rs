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
pub struct u_entry {
    pub ue_next: *mut u_entry_T,
    pub ue_top: linenr_T,
    pub ue_bot: linenr_T,
    pub ue_lcount: linenr_T,
    pub ue_array: *mut *mut ::core::ffi::c_char,
    pub ue_size: linenr_T,
}
pub type u_entry_T = u_entry;

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
#[repr(C)]
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

#[derive(Copy, Clone)]
pub struct u_header {
    pub uh_next: UndoLink,
    pub uh_prev: UndoLink,
    pub uh_alt_next: UndoLink,
    pub uh_alt_prev: UndoLink,
    pub uh_seq: ::core::ffi::c_int,
    pub uh_walk: ::core::ffi::c_int,
    pub uh_entry: *mut u_entry_T,
    pub uh_getbot_entry: *mut u_entry_T,
    pub uh_cursor: pos_T,
    pub uh_cursor_vcol: colnr_T,
    pub uh_flags: ::core::ffi::c_int,
    pub uh_namedm: [fmark_T; 26],
    pub uh_extmark: extmark_undo_vec_t,
    pub uh_visual: visualinfo_T,
    pub uh_time: time_t,
    pub uh_save_nr: ::core::ffi::c_int,
}
pub type u_header_T = u_header;

impl Default for u_header {
    /// A header linked to nothing, with every other field zero — what
    /// `xmalloc` plus a `memset` left behind, spelled out so that getting
    /// one does not mean writing zeroes through a pointer.
    fn default() -> Self {
        let unset_mark = fmark_T {
            mark: pos_T::default(),
            fnum: 0,
            timestamp: 0,
            view: fmarkv_T {
                topline_offset: 0,
                skipcol: 0,
            },
            additional_data: ::core::ptr::null_mut(),
        };
        Self {
            uh_next: UndoLink::NONE,
            uh_prev: UndoLink::NONE,
            uh_alt_next: UndoLink::NONE,
            uh_alt_prev: UndoLink::NONE,
            uh_seq: 0,
            uh_walk: 0,
            uh_entry: ::core::ptr::null_mut(),
            uh_getbot_entry: ::core::ptr::null_mut(),
            uh_cursor: pos_T::default(),
            uh_cursor_vcol: 0,
            uh_flags: 0,
            uh_namedm: [unset_mark; 26],
            uh_extmark: extmark_undo_vec_t {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut(),
            },
            uh_visual: visualinfo_T {
                vi_start: pos_T::default(),
                vi_end: pos_T::default(),
                vi_mode: 0,
                vi_curswant: 0,
            },
            uh_time: 0,
            uh_save_nr: 0,
        }
    }
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct visualinfo_T {
    pub vi_start: pos_T,
    pub vi_end: pos_T,
    pub vi_mode: ::core::ffi::c_int,
    pub vi_curswant: colnr_T,
}
