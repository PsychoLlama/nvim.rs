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

    /// Where a capture slot nothing has filled in stands: NULL in a string
    /// match, and line `-1` column `-1` in a buffer one — the C original
    /// memset the whole slot to `0xff`.
    #[inline(always)]
    pub(crate) fn unset(kind: PosKind) -> MatchPos {
        match kind {
            PosKind::Str => MatchPos::NOWHERE,
            PosKind::Buf => MatchPos::from_pos(lpos_T { lnum: -1, col: -1 }),
        }
    }

    /// Has a capture slot holding this been filled in?
    #[inline(always)]
    pub(crate) fn is_set(self, kind: PosKind) -> bool {
        match kind {
            PosKind::Str => !self.as_ptr().is_null(),
            PosKind::Buf => self.as_pos().lnum >= 0,
        }
    }

    /// Make a capture slot holding this read as unset, *without* disturbing a
    /// buffer match's column.
    ///
    /// Not the same as assigning [`MatchPos::unset`]: the walk uses this for
    /// the slots it stepped over on its way to a later group, and upstream
    /// wrote only the line numbers there. An unset capture is recognised by
    /// its line alone, so the column left lying beside it is never compared —
    /// but it *is* handed back to the caller, so writing one would be a
    /// change in what a match reports.
    #[inline(always)]
    pub(crate) fn mark_unset(&mut self, kind: PosKind) {
        match kind {
            PosKind::Str => self.set_ptr(core::ptr::null_mut()),
            PosKind::Buf => self.pos_mut().lnum = -1,
        }
    }

    /// Is this position strictly before `other`? Both must come from the
    /// same match, so that `kind` describes both.
    #[inline(always)]
    pub(crate) fn is_before(self, other: MatchPos, kind: PosKind) -> bool {
        match kind {
            PosKind::Str => self.as_ptr() < other.as_ptr(),
            PosKind::Buf => {
                let (a, b) = (self.as_pos(), other.as_pos());
                a.lnum < b.lnum || (a.lnum == b.lnum && a.col < b.col)
            }
        }
    }

    /// Do two capture positions describe the same place?
    ///
    /// As [`MatchPos::same`], except that a buffer match's unset position
    /// compares equal whatever column happens to sit beside it — see
    /// [`MatchPos::mark_unset`] for where those stale columns come from.
    #[inline(always)]
    pub(crate) fn same_capture(self, other: MatchPos, kind: PosKind) -> bool {
        match kind {
            PosKind::Str => self.as_ptr() == other.as_ptr(),
            PosKind::Buf => {
                let (a, b) = (self.as_pos(), other.as_pos());
                a.lnum == b.lnum && (a.lnum < 0 || a.col == b.col)
            }
        }
    }
}

/// What one capture group matched: where it started and where it ended.
///
/// Upstream wrote this twice, as `multipos` (four `int`s: two line numbers
/// and two columns) and `linepos` (two pointers), and unioned arrays of the
/// two into `regsub_T.list`. Both arms are a *pair of positions* and
/// [`MatchPos`] is what a position is, so there is nothing left to union: the
/// two shapes are one type, sixteen bytes either way, and the arm is still
/// picked once per match by [`super::rex::Rex::pos_kind`].
///
/// That the size did not move is load-bearing. A capture set rides in every
/// `nfa_thread_T`, twice over, and 'maxmempattern' is charged in threads —
/// so a tag on the two shapes would have cost eight bytes a set, thirty-two a
/// thread, and moved the depth at which E363 is reported by about four per
/// cent.
#[derive(Clone, Copy)]
pub(crate) struct Capture {
    /// Where the group started, or unset.
    pub(crate) start: MatchPos,
    /// Where it ended, or unset.
    pub(crate) end: MatchPos,
}

impl Capture {
    /// A capture slot the match has never reached — see [`MatchPos::unset`].
    #[inline(always)]
    pub(crate) fn unset(kind: PosKind) -> Capture {
        let nowhere = MatchPos::unset(kind);
        Capture {
            start: nowhere,
            end: nowhere,
        }
    }

    /// Make both ends read as unset, leaving a buffer match's columns alone —
    /// see [`MatchPos::mark_unset`].
    #[inline(always)]
    pub(crate) fn mark_unset(&mut self, kind: PosKind) {
        self.start.mark_unset(kind);
        self.end.mark_unset(kind);
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

    /// The measurement the whole design rests on: a capture is two positions
    /// and no discriminant, so an array of them is the size the C union was.
    /// A tagged pair would be twenty-four bytes, and 'maxmempattern' is
    /// charged in the threads that carry ten of them.
    #[test]
    fn a_capture_is_two_positions_and_no_tag() {
        assert_eq!(size_of::<Capture>(), 2 * size_of::<*mut uint8_t>());
        assert_eq!(size_of::<Capture>(), 2 * size_of::<lpos_T>());
        assert_eq!(align_of::<Capture>(), align_of::<*mut uint8_t>());
    }

    #[test]
    fn an_unset_capture_reads_unset_in_either_shape() {
        for kind in [PosKind::Str, PosKind::Buf] {
            let capture = Capture::unset(kind);
            assert!(!capture.start.is_set(kind));
            assert!(!capture.end.is_set(kind));
        }
        let mut byte = 0u8;
        assert!(MatchPos::from_ptr(&raw mut byte).is_set(PosKind::Str));
        assert!(MatchPos::from_pos(lpos_T { lnum: 0, col: 0 }).is_set(PosKind::Buf));
    }

    /// The slots a walk steps over are marked unset by their line alone, and
    /// the column beside it is left as it lies — see `MatchPos::mark_unset`.
    #[test]
    fn marking_a_buffer_position_unset_leaves_its_column() {
        let mut capture = Capture {
            start: MatchPos::from_pos(lpos_T { lnum: 3, col: 11 }),
            end: MatchPos::from_pos(lpos_T { lnum: 4, col: 12 }),
        };
        capture.mark_unset(PosKind::Buf);
        assert_eq!(capture.start.as_pos().lnum, -1);
        assert_eq!(capture.start.as_pos().col, 11);
        assert_eq!(capture.end.as_pos().col, 12);

        let mut line = *b"ab\0";
        let mut capture = Capture {
            start: MatchPos::from_ptr(&raw mut line[0]),
            end: MatchPos::from_ptr(&raw mut line[1]),
        };
        capture.mark_unset(PosKind::Str);
        assert!(capture.start.as_ptr().is_null());
        assert!(capture.end.as_ptr().is_null());
    }

    /// Which is why comparing two capture positions has to ignore the column
    /// of an unset one, where comparing two *input* positions does not.
    #[test]
    fn same_capture_ignores_an_unset_position_s_column() {
        let (a, b) = (
            MatchPos::from_pos(lpos_T { lnum: -1, col: 5 }),
            MatchPos::from_pos(lpos_T { lnum: -1, col: 9 }),
        );
        assert!(a.same_capture(b, PosKind::Buf));
        assert!(!a.same(b, PosKind::Buf));

        let set = MatchPos::from_pos(lpos_T { lnum: 2, col: 5 });
        assert!(!set.same_capture(MatchPos::from_pos(lpos_T { lnum: 2, col: 9 }), PosKind::Buf));
        assert!(set.same_capture(set, PosKind::Buf));
    }
}
