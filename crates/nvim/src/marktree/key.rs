#![forbid(unsafe_code)]

//! Key and position vocabulary for the extmark B-tree.
//!
//! A mark is an `MTKey`: a position, the namespace/id pair that names it, a
//! bitmask of flags and an inline decoration payload. Ranges are stored as two
//! keys (a start and an end) sharing the same `(ns, id)`; `MT_FLAG_END` tells
//! them apart and `mt_lookup_key` folds the triple into the 64-bit handle the
//! tree's id-to-node map is keyed by.
//!
//! Positions inside the tree are stored *relative* to the enclosing node's
//! position, which is what `relative`/`unrelative`/`compose` translate between.

use crate::types::{
    DecorHighlightInline, DecorInline, DecorInlineData, DecorPriority, MTKey, MTPair, MTPos,
    schar_T, uint16_t, uint32_t, uint64_t,
};

/// Minimum fill of a node. The maximum branch is twice this.
pub const MT_BRANCH_FACTOR: ::core::ffi::c_uint = 10;
/// `ceil(log2(2 * MT_BRANCH_FACTOR + 1))` — one extra index encodes
/// "right before this node", which is what `pseudo_index` needs.
pub const MT_LOG2_BRANCH: ::core::ffi::c_uint = 5;

/// A filter slot set to this selects that meta kind; zero skips it.
pub const kMTFilterSelect: uint32_t = 4294967295;

crate::flag_set! {
    /// What an [`MTKey`] is and what decoration it carries -- upstream's
    /// `MT_FLAG_*`, the bits `MTKey::flags` holds.
    ///
    /// The word is a `uint16_t` and every bit of it is spoken for, so the
    /// newtype is declared over the same integer: it is a field of a
    /// `#[repr(C)]` struct that the tree walks by the million.
    pub struct MtFlags: uint16_t;

    /// Set on every key that is a real mark, as opposed to the
    /// `(row, col, 0)` pseudo-key that means "the space before (row, col)".
    const REAL = 1 << 0;
    /// This key is the end of a range; the matching start shares its
    /// `(ns, id)`.
    const END = 1 << 1;
    /// This key is one half of a range.
    const PAIRED = 1 << 2;
    /// The other half of the pair is gone.
    const ORPHANED = 1 << 3;
    /// Undo does not restore this mark's position.
    const NO_UNDO = 1 << 4;
    /// Deleting the text the mark spans marks it invalid rather than
    /// collapsing it.
    const INVALIDATE = 1 << 5;
    /// [`Self::INVALIDATE`] has fired: the mark is hidden but still stored.
    const INVALID = 1 << 6;
    /// The decoration payload is a pointer to an out-of-line `DecorExt`, not
    /// the inline highlight.
    const DECOR_EXT = 1 << 7;
    const DECOR_HL = 1 << 8;
    const DECOR_SIGNTEXT = 1 << 9;
    const DECOR_SIGNHL = 1 << 10;
    const DECOR_VIRT_LINES = 1 << 11;
    const DECOR_VIRT_TEXT_INLINE = 1 << 12;
    const DECOR_CONCEAL_LINES = 1 << 13;
    const RIGHT_GRAVITY = 1 << 14;
    /// Set on the pseudo-key that sorts after every real key at a position.
    const LAST = 1 << 15;

    /// Any decoration at all -- the test the meta index and the redraw path
    /// ask first. `DECOR_CONCEAL_LINES` is deliberately *not* in it: it is
    /// tracked per line rather than per key.
    const DECOR_MASK = Self::DECOR_EXT.bits()
        | Self::DECOR_HL.bits()
        | Self::DECOR_SIGNTEXT.bits()
        | Self::DECOR_SIGNHL.bits()
        | Self::DECOR_VIRT_LINES.bits()
        | Self::DECOR_VIRT_TEXT_INLINE.bits();

    /// The flags a consumer outside the tree may modify in place.
    const EXTERNAL_MASK = Self::DECOR_MASK.bits()
        | Self::NO_UNDO.bits()
        | Self::INVALIDATE.bits()
        | Self::INVALID.bits()
        | Self::DECOR_CONCEAL_LINES.bits();

    /// The bits [`key_cmp`] looks at: everything else about two marks at one
    /// position leaves them equal, and the tree keeps those in insertion
    /// order.
    const ORDER_MASK = Self::RIGHT_GRAVITY.bits()
        | Self::END.bits()
        | Self::REAL.bits()
        | Self::LAST.bits();
}

/// Low bit of a lookup handle: set for the end half of a pair.
pub const MARKTREE_END_FLAG: uint64_t = 1;

pub const DECOR_PRIORITY_BASE: ::core::ffi::c_int = 0x1000;

pub const DECOR_HIGHLIGHT_INLINE_INIT: DecorHighlightInline = DecorHighlightInline {
    flags: 0,
    priority: DECOR_PRIORITY_BASE as DecorPriority,
    hl_id: 0,
    conceal_char: 0 as schar_T,
};

/// What `marktree_itr_current` answers for an exhausted iterator, and what a
/// failed lookup returns. Not a valid position, so callers can test `pos.row`.
pub const MT_INVALID_KEY: MTKey = MTKey {
    pos: MTPos { row: -1, col: -1 },
    ns: 0,
    id: 0,
    flags: MtFlags::NONE,
    decor_data: DecorInlineData {
        hl: DECOR_HIGHLIGHT_INLINE_INIT,
    },
};

/// The 64-bit handle the `id2node` map is keyed by. The namespace occupies the
/// high bits, the mark id the middle, and the end flag the low bit — so the two
/// halves of a pair are adjacent and the start sorts first.
pub fn mt_lookup_id(ns: uint32_t, id: uint32_t, enda: bool) -> uint64_t {
    (ns as uint64_t) << 33 | ((id << 1) as uint64_t) | if enda { MARKTREE_END_FLAG } else { 0 }
}

/// The handle for the named side of `key`'s pair, whichever side `key` is.
pub fn mt_lookup_key_side(key: MTKey, end: bool) -> uint64_t {
    mt_lookup_id(key.ns, key.id, end)
}

pub fn mt_lookup_key(key: MTKey) -> uint64_t {
    mt_lookup_id(key.ns, key.id, mt_end(key))
}

pub fn mt_paired(key: MTKey) -> bool {
    key.flags.has(MtFlags::PAIRED)
}

pub fn mt_end(key: MTKey) -> bool {
    key.flags.has(MtFlags::END)
}

pub fn mt_start(key: MTKey) -> bool {
    mt_paired(key) && !mt_end(key)
}

pub fn mt_right(key: MTKey) -> bool {
    key.flags.has(MtFlags::RIGHT_GRAVITY)
}

pub fn mt_no_undo(key: MTKey) -> bool {
    key.flags.has(MtFlags::NO_UNDO)
}

pub fn mt_invalidate(key: MTKey) -> bool {
    key.flags.has(MtFlags::INVALIDATE)
}

pub fn mt_invalid(key: MTKey) -> bool {
    key.flags.has(MtFlags::INVALID)
}

pub fn mt_decor_any(key: MTKey) -> bool {
    key.flags.has(MtFlags::DECOR_MASK)
}

pub fn mt_decor_sign(key: MTKey) -> bool {
    key.flags
        .has(MtFlags::DECOR_SIGNTEXT.or(MtFlags::DECOR_SIGNHL))
}

pub fn mt_conceal_lines(key: MTKey) -> bool {
    key.flags.has(MtFlags::DECOR_CONCEAL_LINES)
}

pub fn mt_decor(key: MTKey) -> DecorInline {
    DecorInline {
        ext: key.flags.has(MtFlags::DECOR_EXT),
        data: key.decor_data,
    }
}

pub fn mt_flags(right_gravity: bool, no_undo: bool, invalidate: bool, decor_ext: bool) -> MtFlags {
    MtFlags::RIGHT_GRAVITY.when(right_gravity)
        | MtFlags::NO_UNDO.when(no_undo)
        | MtFlags::INVALIDATE.when(invalidate)
        | MtFlags::DECOR_EXT.when(decor_ext)
}

pub fn mtpair_from(start: MTKey, end: MTKey) -> MTPair {
    MTPair {
        start,
        end_pos: end.pos,
        end_right_gravity: mt_right(end),
    }
}

pub fn pos_leq(a: MTPos, b: MTPos) -> bool {
    a.row < b.row || (a.row == b.row && a.col <= b.col)
}

pub fn pos_less(a: MTPos, b: MTPos) -> bool {
    !pos_leq(b, a)
}

/// Rebase `val` onto `base`, i.e. make it relative to it. A position on the
/// same row keeps a column delta and drops to row zero; a later row keeps its
/// absolute column, because a row delta already says the column restarts.
pub fn relative(base: MTPos, val: &mut MTPos) {
    debug_assert!(pos_leq(base, *val), "pos_leq(base, *val)");
    if val.row == base.row {
        val.row = 0;
        val.col -= base.col;
    } else {
        val.row -= base.row;
    }
}

/// Inverse of [`relative`].
pub fn unrelative(base: MTPos, val: &mut MTPos) {
    if val.row == 0 {
        val.row = base.row;
        val.col += base.col;
    } else {
        val.row += base.row;
    }
}

/// Advance `base` by the relative position `val`, in place.
pub fn compose(base: &mut MTPos, val: MTPos) {
    if val.row == 0 {
        base.col += val.col;
    } else {
        base.row += val.row;
        base.col = val.col;
    }
}

/// Total order over keys: position first, then the flags that decide which of
/// several marks at one position comes first. Only gravity, end-ness, realness
/// and the "sorts last" pseudo-key flag participate — two real marks that agree
/// on those compare equal and the tree keeps them in insertion order.
pub fn key_cmp(a: MTKey, b: MTKey) -> ::core::ffi::c_int {
    let cmp = (b.pos.row < a.pos.row) as ::core::ffi::c_int
        - (a.pos.row < b.pos.row) as ::core::ffi::c_int;
    if cmp != 0 {
        return cmp;
    }
    let cmp = (b.pos.col < a.pos.col) as ::core::ffi::c_int
        - (a.pos.col < b.pos.col) as ::core::ffi::c_int;
    if cmp != 0 {
        return cmp;
    }
    let a = a.flags.masked(MtFlags::ORDER_MASK).bits();
    let b = b.flags.masked(MtFlags::ORDER_MASK).bits();
    (b < a) as ::core::ffi::c_int - (a < b) as ::core::ffi::c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(row: i32, col: i32) -> MTPos {
        MTPos { row, col }
    }

    fn key(row: i32, col: i32, flags: MtFlags) -> MTKey {
        MTKey {
            pos: pos(row, col),
            ns: 0,
            id: 0,
            flags,
            decor_data: DecorInlineData {
                hl: DECOR_HIGHLIGHT_INLINE_INIT,
            },
        }
    }

    #[test]
    fn packs_the_namespace_id_and_side_into_one_handle() {
        assert_eq!(mt_lookup_id(0, 0, false), 0);
        assert_eq!(mt_lookup_id(0, 1, false), 2);
        assert_eq!(mt_lookup_id(0, 1, true), 3);
        assert_eq!(mt_lookup_id(1, 0, false), 1 << 33);
        // The two halves of a pair are adjacent, start first.
        assert_eq!(mt_lookup_id(7, 9, true) - mt_lookup_id(7, 9, false), 1);
    }

    #[test]
    fn drops_the_top_bit_of_a_mark_id() {
        // `id << 1` is computed in 32 bits before the widening, so an id at or
        // above 2^31 aliases a smaller one. Upstream behaviour; ids come from a
        // counter that never gets there in practice.
        assert_eq!(mt_lookup_id(0, 1 << 31, false), mt_lookup_id(0, 0, false));
    }

    #[test]
    fn a_start_is_paired_and_not_an_end() {
        let start = key(0, 0, MtFlags::PAIRED);
        let end = key(0, 0, MtFlags::PAIRED | MtFlags::END);
        assert!(mt_start(start) && !mt_end(start));
        assert!(!mt_start(end) && mt_end(end));
        assert!(!mt_start(key(0, 0, MtFlags::NONE)));
    }

    #[test]
    fn orders_by_position_then_by_the_comparison_flags() {
        assert!(key_cmp(key(1, 0, MtFlags::NONE), key(2, 0, MtFlags::NONE)) < 0);
        assert!(key_cmp(key(1, 5, MtFlags::NONE), key(1, 4, MtFlags::NONE)) > 0);
        // Only the four masked flags matter: two marks differing in a decor
        // flag compare equal.
        assert_eq!(
            key_cmp(key(1, 1, MtFlags::DECOR_HL), key(1, 1, MtFlags::NONE)),
            0
        );
        // A right-gravity mark sorts after a left-gravity one at the same spot,
        // and the "last" pseudo-key sorts after everything.
        assert!(key_cmp(key(1, 1, MtFlags::RIGHT_GRAVITY), key(1, 1, MtFlags::NONE)) > 0);
        assert!(key_cmp(key(1, 1, MtFlags::LAST), key(1, 1, MtFlags::REAL)) > 0);
    }

    #[test]
    fn relative_and_unrelative_round_trip() {
        for base in [pos(0, 0), pos(3, 7), pos(3, 0)] {
            for val in [pos(3, 9), pos(4, 2), pos(9, 0)] {
                if !pos_leq(base, val) {
                    continue;
                }
                let mut rel = val;
                relative(base, &mut rel);
                unrelative(base, &mut rel);
                assert_eq!((rel.row, rel.col), (val.row, val.col));
            }
        }
    }

    #[test]
    fn a_relative_position_on_the_base_row_keeps_only_a_column_delta() {
        let mut val = pos(3, 9);
        relative(pos(3, 7), &mut val);
        assert_eq!((val.row, val.col), (0, 2));

        let mut val = pos(5, 2);
        relative(pos(3, 7), &mut val);
        assert_eq!((val.row, val.col), (2, 2));
    }

    #[test]
    fn compose_resets_the_column_only_when_the_row_moves() {
        let mut base = pos(3, 7);
        compose(&mut base, pos(0, 2));
        assert_eq!((base.row, base.col), (3, 9));
        compose(&mut base, pos(1, 4));
        assert_eq!((base.row, base.col), (4, 4));
    }

    #[test]
    fn the_flag_builder_sets_exactly_the_four_requested_bits() {
        assert_eq!(mt_flags(false, false, false, false), MtFlags::NONE);
        assert_eq!(
            mt_flags(true, true, true, true),
            MtFlags::RIGHT_GRAVITY | MtFlags::NO_UNDO | MtFlags::INVALIDATE | MtFlags::DECOR_EXT
        );
    }

    #[test]
    fn the_external_mask_covers_every_flag_a_consumer_may_set() {
        // The tree's own bookkeeping bits must stay out of it.
        for own in [
            MtFlags::REAL,
            MtFlags::END,
            MtFlags::PAIRED,
            MtFlags::ORPHANED,
            MtFlags::RIGHT_GRAVITY,
            MtFlags::LAST,
        ] {
            assert!(!MtFlags::EXTERNAL_MASK.has(own));
        }
    }
}
