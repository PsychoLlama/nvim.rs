//! [`MatchPos`], the one shape a saved position comes in.
//!
//! A match runs over either a string or a range of buffer lines, and the two
//! record a position differently: a string match holds a pointer into the one
//! string it was handed, a buffer match a line/column pair. Upstream wrote
//! that choice out three separate times — `save_se_T.se_u`, `regsave_T.rs_u`
//! and `nfa_pim_T.end` — as three anonymous unions with the same two arms.
//! This is the one type all three became.
//!
//! ## Why there is no tag in here
//!
//! Which arm is live is not a property of the value. It is a property of the
//! *run*: `rex.reg_match` is null for a buffer match and non-null for a
//! string match, it is fixed before the first frame is pushed, and it decides
//! the shape of every position the match will ever save. A tagged enum would
//! store that one global answer once per saved position — and the values are
//! not scarce. The backtracker pushes a [`SavedInput`] per decision and a
//! `regbehind_T` carries twenty [`MatchPos`]es, so a tag would cost a
//! quarter of the frame stack and half of every look-behind snapshot, and
//! `'maxmempattern'` is charged in bytes of that stack: growing the frame
//! moves the depth at which E363 fires, which is user-visible behaviour.
//!
//! So the tag stays where the answer already lives — on the [`Rex`] handle,
//! as [`Rex::pos_kind`] — and this type is eight bytes of storage with an
//! arm-explicit API. Nothing here is `unsafe` to call: both arms are eight
//! initialised bytes with no invalid values, so reading the arm the run does
//! not use is meaningless rather than undefined, and the meaning is what the
//! `kind` argument and the accessor names carry.
//!
//! ## What that leaves of `regitem_T`'s union
//!
//! Once the three spellings are one type, `regitem_T`'s `rs_un` — upstream's
//! union of `save_se_T` against `regsave_T` — has two arms that differ only
//! by a trailing `int`. There is no pun left to express: a frame saves a
//! position, and the frames that save the *input* position additionally save
//! how much of `backpos` belonged to it. That is [`SavedInput`], a plain
//! struct, and the states that only wanted the position read [`SavedInput::
//! pos`] and leave [`SavedInput::backpos_len`] alone.

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use crate::types::{lpos_T, uint8_t};

/// Which shape a match records positions in. Fixed for the whole of one
/// match; see [`super::rex::Rex::pos_kind`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PosKind {
    /// A string match: a position is a pointer into the string.
    Str,
    /// A buffer match: a position is a line number and a column.
    Buf,
}

/// The two arms. Private, and the only reason this file has an `unsafe` in
/// it: `MatchPos` is what the rest of the tree sees.
#[derive(Clone, Copy)]
union Arms {
    ptr: *mut uint8_t,
    pos: lpos_T,
}

/// A position a match saved, in whichever shape [`PosKind`] the run uses.
#[derive(Clone, Copy)]
pub(crate) struct MatchPos(Arms);

impl MatchPos {
    /// No position at all: a null pointer, which reads as line 0 column 0 in
    /// the other arm. What a blank frame and an untouched capture slot hold.
    pub(crate) const NOWHERE: MatchPos = MatchPos::from_ptr(core::ptr::null_mut());

    /// A string match's position.
    pub(crate) const fn from_ptr(ptr: *mut uint8_t) -> MatchPos {
        MatchPos(Arms { ptr })
    }

    /// A buffer match's position.
    pub(crate) const fn from_pos(pos: lpos_T) -> MatchPos {
        MatchPos(Arms { pos })
    }

    /// The pointer a string match saved.
    #[inline(always)]
    pub(crate) fn as_ptr(self) -> *mut uint8_t {
        // SAFETY: both arms are eight initialised bytes. This one is the
        // string match's; a buffer match never asks for it.
        unsafe { self.0.ptr }
    }

    /// The line and column a buffer match saved.
    #[inline(always)]
    pub(crate) fn as_pos(self) -> lpos_T {
        // SAFETY: as `as_ptr`, for the buffer match's arm.
        unsafe { self.0.pos }
    }

    /// The line and column a buffer match saved, to edit in place — the
    /// look-behind walks its start position backwards through it.
    #[inline(always)]
    pub(crate) fn pos_mut(&mut self) -> &mut lpos_T {
        // SAFETY: as `as_pos`. `lpos_T` is a pair of plain integers, so the
        // reference can neither observe nor create an invalid value.
        unsafe { &mut self.0.pos }
    }

    /// Overwrite a string match's position.
    #[inline(always)]
    pub(crate) fn set_ptr(&mut self, ptr: *mut uint8_t) {
        self.0.ptr = ptr;
    }

    /// Are these the same position? Both must come from the same match, so
    /// that `kind` describes both.
    #[inline(always)]
    pub(crate) fn same(self, other: MatchPos, kind: PosKind) -> bool {
        match kind {
            PosKind::Str => self.as_ptr() == other.as_ptr(),
            PosKind::Buf => {
                let (a, b) = (self.as_pos(), other.as_pos());
                a.lnum == b.lnum && a.col == b.col
            }
        }
    }
}

/// Where the input was, and how much of `backpos` belonged to that.
///
/// `backpos` records where each loop back-edge has already been, so undoing a
/// decision has to forget the positions discovered after it: truncating to
/// `backpos_len` is that.
#[derive(Clone, Copy)]
pub(crate) struct SavedInput {
    pub(crate) pos: MatchPos,
    pub(crate) backpos_len: c_int,
}

impl SavedInput {
    /// A blank frame's, before the pusher fills it in.
    pub(crate) const NOWHERE: SavedInput = SavedInput {
        pos: MatchPos::NOWHERE,
        backpos_len: 0,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The whole reason the arms are untagged: a saved position is one
    /// pointer wide, in a stack whose size 'maxmempattern' is charged for.
    #[test]
    fn a_position_is_pointer_sized() {
        assert_eq!(size_of::<MatchPos>(), size_of::<*mut uint8_t>());
        assert_eq!(align_of::<MatchPos>(), align_of::<*mut uint8_t>());
    }

    #[test]
    fn a_string_match_gets_its_pointer_back() {
        let mut line = *b"hello\0";
        let at = &raw mut line[2];
        let saved = MatchPos::from_ptr(at);
        assert_eq!(saved.as_ptr(), at);
        // Provenance survives the round trip, so the pointer is still
        // dereferenceable -- which is what the engines do with it.
        // SAFETY: `at` points into `line`, which outlives this.
        assert_eq!(unsafe { *saved.as_ptr() }, b'l');
    }

    #[test]
    fn a_buffer_match_gets_its_line_and_column_back() {
        let saved = MatchPos::from_pos(lpos_T { lnum: 7, col: 13 });
        assert_eq!(saved.as_pos().lnum, 7);
        assert_eq!(saved.as_pos().col, 13);
    }

    #[test]
    fn nowhere_is_null_in_either_arm() {
        assert!(MatchPos::NOWHERE.as_ptr().is_null());
        assert_eq!(MatchPos::NOWHERE.as_pos().lnum, 0);
        assert_eq!(MatchPos::NOWHERE.as_pos().col, 0);
    }

    #[test]
    fn the_look_behind_walks_a_column_back_in_place() {
        let mut saved = MatchPos::from_pos(lpos_T { lnum: 4, col: 9 });
        saved.pos_mut().col -= 3;
        saved.pos_mut().lnum -= 1;
        assert_eq!(saved.as_pos().lnum, 3);
        assert_eq!(saved.as_pos().col, 6);
    }

    #[test]
    fn a_string_position_can_be_overwritten() {
        let mut line = *b"hello\0";
        let mut saved = MatchPos::from_ptr(&raw mut line[4]);
        saved.set_ptr(&raw mut line[1]);
        // SAFETY: `line` outlives the pointer saved into it.
        assert_eq!(unsafe { *saved.as_ptr() }, b'e');
    }

    #[test]
    fn same_compares_the_arm_the_run_uses() {
        let mut line = *b"ab\0";
        let (one, two) = (&raw mut line[0], &raw mut line[1]);
        assert!(MatchPos::from_ptr(one).same(MatchPos::from_ptr(one), PosKind::Str));
        assert!(!MatchPos::from_ptr(one).same(MatchPos::from_ptr(two), PosKind::Str));

        let here = lpos_T { lnum: 2, col: 5 };
        assert!(MatchPos::from_pos(here).same(MatchPos::from_pos(here), PosKind::Buf));
        assert!(
            !MatchPos::from_pos(here)
                .same(MatchPos::from_pos(lpos_T { lnum: 2, col: 6 }), PosKind::Buf)
        );
        assert!(
            !MatchPos::from_pos(here)
                .same(MatchPos::from_pos(lpos_T { lnum: 3, col: 5 }), PosKind::Buf)
        );
    }

    /// A frame is a position plus one `int`, and that is all the union
    /// `regitem_T.rs_un` was hiding.
    #[test]
    fn a_blank_input_save_is_nowhere_with_no_backpos() {
        assert!(SavedInput::NOWHERE.pos.as_ptr().is_null());
        assert_eq!(SavedInput::NOWHERE.backpos_len, 0);
    }
}
