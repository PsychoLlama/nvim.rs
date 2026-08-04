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

use crate::src::nvim::types::{
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

/// Set on every key that is a real mark, as opposed to the `(row, col, 0)`
/// pseudo-key that means "the space before (row, col)".
pub const MT_FLAG_REAL: ::core::ffi::c_int = 1 << 0;
/// This key is the end of a range; the matching start shares its `(ns, id)`.
pub const MT_FLAG_END: ::core::ffi::c_int = 1 << 1;
/// This key is one half of a range.
pub const MT_FLAG_PAIRED: ::core::ffi::c_int = 1 << 2;
/// The other half of the pair is gone.
pub const MT_FLAG_ORPHANED: ::core::ffi::c_int = 1 << 3;
pub const MT_FLAG_NO_UNDO: ::core::ffi::c_int = 1 << 4;
pub const MT_FLAG_INVALIDATE: ::core::ffi::c_int = 1 << 5;
pub const MT_FLAG_INVALID: ::core::ffi::c_int = 1 << 6;
/// The decoration payload is a pointer to an out-of-line `DecorExt`, not the
/// inline highlight.
pub const MT_FLAG_DECOR_EXT: ::core::ffi::c_int = 1 << 7;
pub const MT_FLAG_DECOR_HL: ::core::ffi::c_int = 1 << 8;
pub const MT_FLAG_DECOR_SIGNTEXT: ::core::ffi::c_int = 1 << 9;
pub const MT_FLAG_DECOR_SIGNHL: ::core::ffi::c_int = 1 << 10;
pub const MT_FLAG_DECOR_VIRT_LINES: ::core::ffi::c_int = 1 << 11;
pub const MT_FLAG_DECOR_VIRT_TEXT_INLINE: ::core::ffi::c_int = 1 << 12;
pub const MT_FLAG_DECOR_CONCEAL_LINES: ::core::ffi::c_int = 1 << 13;
pub const MT_FLAG_RIGHT_GRAVITY: ::core::ffi::c_int = 1 << 14;
/// Set on the pseudo-key that sorts after every real key at a position.
pub const MT_FLAG_LAST: ::core::ffi::c_int = 1 << 15;

pub const MT_FLAG_DECOR_MASK: ::core::ffi::c_int = MT_FLAG_DECOR_EXT
    | MT_FLAG_DECOR_HL
    | MT_FLAG_DECOR_SIGNTEXT
    | MT_FLAG_DECOR_SIGNHL
    | MT_FLAG_DECOR_VIRT_LINES
    | MT_FLAG_DECOR_VIRT_TEXT_INLINE;

/// The flags a consumer outside the tree may modify in place.
pub const MT_FLAG_EXTERNAL_MASK: ::core::ffi::c_int = MT_FLAG_DECOR_MASK
    | MT_FLAG_NO_UNDO
    | MT_FLAG_INVALIDATE
    | MT_FLAG_INVALID
    | MT_FLAG_DECOR_CONCEAL_LINES;

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
    flags: 0,
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
    key.flags as ::core::ffi::c_int & MT_FLAG_PAIRED != 0
}

pub fn mt_end(key: MTKey) -> bool {
    key.flags as ::core::ffi::c_int & MT_FLAG_END != 0
}

pub fn mt_start(key: MTKey) -> bool {
    mt_paired(key) && !mt_end(key)
}

pub fn mt_right(key: MTKey) -> bool {
    key.flags as ::core::ffi::c_int & MT_FLAG_RIGHT_GRAVITY != 0
}

pub fn mt_no_undo(key: MTKey) -> bool {
    key.flags as ::core::ffi::c_int & MT_FLAG_NO_UNDO != 0
}

pub fn mt_invalidate(key: MTKey) -> bool {
    key.flags as ::core::ffi::c_int & MT_FLAG_INVALIDATE != 0
}

pub fn mt_invalid(key: MTKey) -> bool {
    key.flags as ::core::ffi::c_int & MT_FLAG_INVALID != 0
}

pub fn mt_decor_any(key: MTKey) -> bool {
    key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_MASK != 0
}

pub fn mt_decor_sign(key: MTKey) -> bool {
    key.flags as ::core::ffi::c_int & (MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL) != 0
}

pub fn mt_conceal_lines(key: MTKey) -> bool {
    key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_CONCEAL_LINES != 0
}

pub fn mt_decor(key: MTKey) -> DecorInline {
    DecorInline {
        ext: key.flags as ::core::ffi::c_int & MT_FLAG_DECOR_EXT != 0,
        data: key.decor_data,
    }
}

pub fn mt_flags(right_gravity: bool, no_undo: bool, invalidate: bool, decor_ext: bool) -> uint16_t {
    let mut flags = 0;
    if right_gravity {
        flags |= MT_FLAG_RIGHT_GRAVITY;
    }
    if no_undo {
        flags |= MT_FLAG_NO_UNDO;
    }
    if invalidate {
        flags |= MT_FLAG_INVALIDATE;
    }
    if decor_ext {
        flags |= MT_FLAG_DECOR_EXT;
    }
    flags as uint16_t
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
    assert!(pos_leq(base, *val), "pos_leq(base, *val)");
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
    let mask = MT_FLAG_RIGHT_GRAVITY | MT_FLAG_END | MT_FLAG_REAL | MT_FLAG_LAST;
    let a = a.flags as ::core::ffi::c_int & mask;
    let b = b.flags as ::core::ffi::c_int & mask;
    (b < a) as ::core::ffi::c_int - (a < b) as ::core::ffi::c_int
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(row: i32, col: i32) -> MTPos {
        MTPos { row, col }
    }

    fn key(row: i32, col: i32, flags: ::core::ffi::c_int) -> MTKey {
        MTKey {
            pos: pos(row, col),
            ns: 0,
            id: 0,
            flags: flags as uint16_t,
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
        let start = key(0, 0, MT_FLAG_PAIRED);
        let end = key(0, 0, MT_FLAG_PAIRED | MT_FLAG_END);
        assert!(mt_start(start) && !mt_end(start));
        assert!(!mt_start(end) && mt_end(end));
        assert!(!mt_start(key(0, 0, 0)));
    }

    #[test]
    fn orders_by_position_then_by_the_comparison_flags() {
        assert!(key_cmp(key(1, 0, 0), key(2, 0, 0)) < 0);
        assert!(key_cmp(key(1, 5, 0), key(1, 4, 0)) > 0);
        // Only the four masked flags matter: two marks differing in a decor
        // flag compare equal.
        assert_eq!(key_cmp(key(1, 1, MT_FLAG_DECOR_HL), key(1, 1, 0)), 0);
        // A right-gravity mark sorts after a left-gravity one at the same spot,
        // and the "last" pseudo-key sorts after everything.
        assert!(key_cmp(key(1, 1, MT_FLAG_RIGHT_GRAVITY), key(1, 1, 0)) > 0);
        assert!(key_cmp(key(1, 1, MT_FLAG_LAST), key(1, 1, MT_FLAG_REAL)) > 0);
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
        assert_eq!(mt_flags(false, false, false, false), 0);
        assert_eq!(
            mt_flags(true, true, true, true) as ::core::ffi::c_int,
            MT_FLAG_RIGHT_GRAVITY | MT_FLAG_NO_UNDO | MT_FLAG_INVALIDATE | MT_FLAG_DECOR_EXT
        );
    }

    #[test]
    fn the_external_mask_covers_every_flag_a_consumer_may_set() {
        // The tree's own bookkeeping bits must stay out of it.
        for own in [
            MT_FLAG_REAL,
            MT_FLAG_END,
            MT_FLAG_PAIRED,
            MT_FLAG_ORPHANED,
            MT_FLAG_RIGHT_GRAVITY,
            MT_FLAG_LAST,
        ] {
            assert_eq!(MT_FLAG_EXTERNAL_MASK & own, 0);
        }
    }
}
