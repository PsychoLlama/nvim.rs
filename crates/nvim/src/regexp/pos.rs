//! [`MatchPos`], the one shape a saved position comes in.
//!
//! A match runs over either a string or a range of buffer lines. Upstream
//! recorded the two differently — a pointer into the string, a line/column
//! pair in the buffer — and wrote that choice out three separate times
//! (`save_se_T.se_u`, `regsave_T.rs_u`, `NfaPim.end`) as anonymous unions.
//! With the cursor an offset into its line, a string match's position is a
//! column too, on line 0, so a position is a line and a column whichever
//! kind of match saved it, and there is no union left.
//!
//! What still differs between the two kinds is what an *unset* capture
//! looks like, and [`PosKind`] — fixed for a whole match, and carried by the
//! [`Rex`] handle rather than by every position — is how the few places
//! that care are told:
//!
//! - upstream's string match nulled a pointer, which is line -1 column -1
//!   here; its buffer match wrote only the line number, leaving the column
//!   beside it to be handed back to the caller. [`MatchPos::mark_unset`]
//!   keeps both behaviours.
//!
//! A position is eight bytes, as the pointer and the `lpos_T` were, and that
//! matters: 'maxmempattern' is charged in bytes of the backtracker's stack
//! and of the NFA's thread lists, which hold these.
//!
//! ## What that leaves of `RegItem`'s union
//!
//! Upstream's `rs_un` — `save_se_T` against `regsave_T` — has two arms that
//! differ only by a trailing `int`. A frame saves a position, and the frames
//! that save the *input* position additionally save how much of `backpos`
//! belonged to it. That is [`SavedInput`], a plain struct, and the states
//! that only wanted the position read [`SavedInput::pos`] and leave
//! [`SavedInput::backpos_len`] alone.
//!
//! [`Rex`]: super::Rex

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::types::LPos;

/// Which kind of match a position belongs to. Fixed for the whole of one
/// match; see [`super::rex::Rex::pos_kind`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PosKind {
    /// A string match: every position is on line 0.
    Str,
    /// A buffer match.
    Buf,
}

/// A position a match saved: a line, counted from the match's first, and a
/// byte column in it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct MatchPos(LPos);

impl MatchPos {
    /// Line 0 column 0: what a blank frame holds before the pusher fills it
    /// in. Never read as a position.
    pub(crate) const NOWHERE: MatchPos = MatchPos(LPos { lnum: 0, col: 0 });

    /// A capture slot nothing has filled in.
    const UNSET: MatchPos = MatchPos(LPos { lnum: -1, col: -1 });

    #[inline(always)]
    pub(crate) const fn from_pos(pos: LPos) -> MatchPos {
        MatchPos(pos)
    }

    #[inline(always)]
    pub(crate) fn as_pos(self) -> LPos {
        self.0
    }

    /// The line and column, to edit in place — the look-behind walks its
    /// start position backwards through it.
    #[inline(always)]
    pub(crate) fn pos_mut(&mut self) -> &mut LPos {
        &mut self.0
    }

    /// Are these the same position?
    #[inline(always)]
    pub(crate) fn same(self, other: MatchPos, _kind: PosKind) -> bool {
        self.0.lnum == other.0.lnum && self.0.col == other.0.col
    }

    /// Where a capture slot nothing has filled in stands: line `-1` column
    /// `-1` — what upstream's NULL pointer and its `0xff`-filled `lpos_T`
    /// both become.
    #[inline(always)]
    pub(crate) fn unset(_kind: PosKind) -> MatchPos {
        MatchPos::UNSET
    }

    /// Has a capture slot holding this been filled in?
    #[inline(always)]
    pub(crate) fn is_set(self, _kind: PosKind) -> bool {
        self.0.lnum >= 0
    }

    /// Make a capture slot holding this read as unset.
    ///
    /// Not the same as assigning [`MatchPos::unset`] in a buffer match: the
    /// walk uses this for the slots it stepped over on its way to a later
    /// group, and upstream wrote only the line numbers there. An unset
    /// capture is recognised by its line alone, so the column left lying
    /// beside it is never compared — but it *is* handed back to the caller,
    /// so writing one would be a change in what a match reports. A string
    /// match nulled the whole pointer.
    #[inline(always)]
    pub(crate) fn mark_unset(&mut self, kind: PosKind) {
        match kind {
            PosKind::Str => *self = MatchPos::UNSET,
            PosKind::Buf => self.0.lnum = -1,
        }
    }

    /// Is this position strictly before `other`?
    #[inline(always)]
    pub(crate) fn is_before(self, other: MatchPos, _kind: PosKind) -> bool {
        let (a, b) = (self.0, other.0);
        a.lnum < b.lnum || (a.lnum == b.lnum && a.col < b.col)
    }

    /// Do two capture positions describe the same place?
    ///
    /// As [`MatchPos::same`], except that an unset position compares equal
    /// whatever column happens to sit beside it — see
    /// [`MatchPos::mark_unset`] for where those stale columns come from.
    #[inline(always)]
    pub(crate) fn same_capture(self, other: MatchPos, _kind: PosKind) -> bool {
        let (a, b) = (self.0, other.0);
        a.lnum == b.lnum && (a.lnum < 0 || a.col == b.col)
    }
}

/// What one capture group matched: where it started and where it ended.
///
/// Upstream wrote this twice, as `multipos` (four `int`s) and `linepos` (two
/// pointers), and unioned arrays of the two into `RegSub.list`. Both are a
/// pair of positions, sixteen bytes, and that the size did not move is
/// load-bearing: a capture set rides in every `NfaThread`, twice over, and
/// 'maxmempattern' is charged in threads.
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
    pub(crate) backpos_len: usize,
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

    /// A saved position is eight bytes, in stacks whose size
    /// 'maxmempattern' is charged for.
    #[test]
    fn a_position_is_eight_bytes() {
        assert_eq!(size_of::<MatchPos>(), 8);
        assert_eq!(size_of::<Capture>(), 16);
        assert_eq!(size_of::<SavedInput>(), 16);
    }

    #[test]
    fn the_look_behind_walks_a_column_back_in_place() {
        let mut saved = MatchPos::from_pos(LPos { lnum: 4, col: 9 });
        saved.pos_mut().col -= 3;
        saved.pos_mut().lnum -= 1;
        assert_eq!(saved.as_pos().lnum, 3);
        assert_eq!(saved.as_pos().col, 6);
    }

    #[test]
    fn an_unset_capture_reads_unset_in_either_kind() {
        for kind in [PosKind::Str, PosKind::Buf] {
            let capture = Capture::unset(kind);
            assert!(!capture.start.is_set(kind));
            assert!(!capture.end.is_set(kind));
        }
        assert!(MatchPos::from_pos(LPos { lnum: 0, col: 0 }).is_set(PosKind::Str));
        assert!(MatchPos::from_pos(LPos { lnum: 0, col: 0 }).is_set(PosKind::Buf));
    }

    /// The slots a buffer match's walk steps over are marked unset by their
    /// line alone, and the column beside it is left as it lies; a string
    /// match's become wholly unset, as upstream's NULL pointer did.
    #[test]
    fn marking_a_position_unset_follows_the_kind() {
        let mut capture = Capture {
            start: MatchPos::from_pos(LPos { lnum: 3, col: 11 }),
            end: MatchPos::from_pos(LPos { lnum: 4, col: 12 }),
        };
        capture.mark_unset(PosKind::Buf);
        assert_eq!(capture.start.as_pos(), LPos { lnum: -1, col: 11 });
        assert_eq!(capture.end.as_pos(), LPos { lnum: -1, col: 12 });

        let mut capture = Capture {
            start: MatchPos::from_pos(LPos { lnum: 0, col: 1 }),
            end: MatchPos::from_pos(LPos { lnum: 0, col: 2 }),
        };
        capture.mark_unset(PosKind::Str);
        assert_eq!(capture.start, MatchPos::unset(PosKind::Str));
        assert_eq!(capture.end, MatchPos::unset(PosKind::Str));
    }

    /// Which is why comparing two capture positions has to ignore the column
    /// of an unset one, where comparing two *input* positions does not.
    #[test]
    fn same_capture_ignores_an_unset_position_s_column() {
        let (a, b) = (
            MatchPos::from_pos(LPos { lnum: -1, col: 5 }),
            MatchPos::from_pos(LPos { lnum: -1, col: 9 }),
        );
        assert!(a.same_capture(b, PosKind::Buf));
        assert!(!a.same(b, PosKind::Buf));

        let set = MatchPos::from_pos(LPos { lnum: 2, col: 5 });
        assert!(!set.same_capture(MatchPos::from_pos(LPos { lnum: 2, col: 9 }), PosKind::Buf));
        assert!(set.same_capture(set, PosKind::Buf));
    }

    /// Positions order by line, then column; an unset one sorts first, as
    /// upstream's NULL did against a pointer into the string.
    #[test]
    fn positions_order_by_line_then_column() {
        let at = |lnum, col| MatchPos::from_pos(LPos { lnum, col });
        assert!(at(0, 3).is_before(at(0, 4), PosKind::Str));
        assert!(!at(0, 4).is_before(at(0, 4), PosKind::Str));
        assert!(at(1, 9).is_before(at(2, 0), PosKind::Buf));
        assert!(MatchPos::unset(PosKind::Str).is_before(at(0, 0), PosKind::Str));
    }
}
