#![forbid(unsafe_code)]

//! Per-subtree counts of the decoration kinds a filtered walk cares about.
//!
//! Every node carries, for each of its children, how many keys of each kind
//! live in that child's subtree; the tree's root count lives in
//! `MarkTree::meta_root`. A walk that only wants (say) inline virtual text can
//! then skip a whole subtree whose count for that kind is zero, which is what
//! makes drawing a screenful of text cheap when a buffer holds thousands of
//! marks with no decoration.
//!
//! The counts describe *start* keys only. An end key adds nothing (the pair's
//! decoration hangs off the start), and neither does an invalidated key.

use crate::types::{MTKey, uint32_t};

use super::key::{
    MT_FLAG_DECOR_CONCEAL_LINES, MT_FLAG_DECOR_SIGNHL, MT_FLAG_DECOR_SIGNTEXT,
    MT_FLAG_DECOR_VIRT_LINES, MT_FLAG_DECOR_VIRT_TEXT_INLINE, mt_end, mt_invalid,
};

// Nested so `ffigen` does not publish a name this generic into the flat cdef
// namespace; it collects top-level consts only, and `pub use` is invisible to
// it.
mod count {
    pub const META_COUNT: usize = super::kMTMetaCount as usize;
}
use crate::decoration::kMTMetaCount;
pub use count::META_COUNT;

/// One count per meta kind, indexed by `kMTMeta*`.
pub type MetaCount = [uint32_t; META_COUNT];

/// The key flag each meta index counts. Indexing a plain `static` is safe, so
/// this deliberately is not a `GlobalCell`: it holds no pointers and is only
/// ever read.
pub static META_MAP: MetaCount = [
    MT_FLAG_DECOR_VIRT_TEXT_INLINE as uint32_t,
    MT_FLAG_DECOR_VIRT_LINES as uint32_t,
    MT_FLAG_DECOR_SIGNHL as uint32_t,
    MT_FLAG_DECOR_SIGNTEXT as uint32_t,
    MT_FLAG_DECOR_CONCEAL_LINES as uint32_t,
];

/// Add what `k` contributes to `meta`. An end key and an invalidated key
/// contribute nothing.
pub fn meta_add_key(meta: &mut MetaCount, k: MTKey) {
    if mt_end(k) || mt_invalid(k) {
        return;
    }
    let flags = k.flags as uint32_t;
    for m in 0..META_COUNT {
        meta[m] = meta[m].wrapping_add(u32::from(flags & META_MAP[m] != 0));
    }
}

/// What one key contributes, on its own.
pub fn meta_describe_key(k: MTKey) -> MetaCount {
    let mut meta = [0; META_COUNT];
    meta_add_key(&mut meta, k);
    meta
}

pub fn meta_add(dst: &mut MetaCount, src: &MetaCount) {
    for m in 0..META_COUNT {
        dst[m] = dst[m].wrapping_add(src[m]);
    }
}

pub fn meta_sub(dst: &mut MetaCount, src: &MetaCount) {
    for m in 0..META_COUNT {
        dst[m] = dst[m].wrapping_sub(src[m]);
    }
}

/// Apply the change from `old` to `new`. The counts are unsigned and the
/// difference is signed, so this wraps deliberately: a decrease shows up as a
/// large addend that wraps back round, exactly as the C's `+= new - old` did.
pub fn meta_apply_delta(dst: &mut MetaCount, new: &MetaCount, old: &MetaCount) {
    for m in 0..META_COUNT {
        dst[m] = dst[m].wrapping_add(new[m].wrapping_sub(old[m]));
    }
}

/// Does this count hold anything the filter selects?
///
/// A filter slot is [`kMTFilterSelect`](super::key::kMTFilterSelect) for a
/// selected kind and zero otherwise, so the mask-and-sum is exact rather than
/// a heuristic.
pub fn meta_has(count: &MetaCount, filter: &MetaCount) -> bool {
    let mut total: uint32_t = 0;
    for m in 0..META_COUNT {
        total = total.wrapping_add(count[m] & filter[m]);
    }
    total > 0
}

/// The key flags a filtered walk is looking for: the union of the flags the
/// filter's selected kinds count.
pub fn filtered_key_flags(filter: &MetaCount) -> uint32_t {
    let mut flags: uint32_t = 0;
    for m in 0..META_COUNT {
        flags |= META_MAP[m] & filter[m];
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::super::key::{
        DECOR_HIGHLIGHT_INLINE_INIT, MT_FLAG_END, MT_FLAG_INVALID, kMTFilterSelect,
    };
    use super::*;
    use crate::decoration::{
        kMTMetaConcealLines, kMTMetaInline, kMTMetaLines, kMTMetaSignHL, kMTMetaSignText,
    };
    use crate::types::{DecorInlineData, MTPos, uint16_t};

    fn key(flags: ::core::ffi::c_int) -> MTKey {
        MTKey {
            pos: MTPos { row: 0, col: 0 },
            ns: 0,
            id: 0,
            flags: flags as uint16_t,
            decor_data: DecorInlineData {
                hl: DECOR_HIGHLIGHT_INLINE_INIT,
            },
        }
    }

    #[test]
    fn counts_one_per_decoration_kind_the_key_carries() {
        let meta = meta_describe_key(key(MT_FLAG_DECOR_VIRT_LINES | MT_FLAG_DECOR_CONCEAL_LINES));
        assert_eq!(meta[kMTMetaLines as usize], 1);
        assert_eq!(meta[kMTMetaConcealLines as usize], 1);
        assert_eq!(meta[kMTMetaInline as usize], 0);
        assert_eq!(meta[kMTMetaSignHL as usize], 0);
        assert_eq!(meta[kMTMetaSignText as usize], 0);
    }

    #[test]
    fn an_end_key_or_an_invalid_key_counts_for_nothing() {
        let flags = MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL;
        assert_eq!(meta_describe_key(key(flags | MT_FLAG_END)), [0; META_COUNT]);
        assert_eq!(
            meta_describe_key(key(flags | MT_FLAG_INVALID)),
            [0; META_COUNT]
        );
        // ... but the start of the same pair does count.
        assert_eq!(meta_describe_key(key(flags))[kMTMetaSignText as usize], 1);
    }

    #[test]
    fn a_filter_selects_only_its_own_kinds() {
        let mut filter = [0; META_COUNT];
        filter[kMTMetaSignText as usize] = kMTFilterSelect;
        assert!(meta_has(
            &meta_describe_key(key(MT_FLAG_DECOR_SIGNTEXT)),
            &filter
        ));
        assert!(!meta_has(
            &meta_describe_key(key(MT_FLAG_DECOR_SIGNHL)),
            &filter
        ));
        assert_eq!(
            filtered_key_flags(&filter),
            MT_FLAG_DECOR_SIGNTEXT as uint32_t
        );
    }

    #[test]
    fn adding_and_subtracting_a_key_cancel() {
        let mut meta = [3, 4, 5, 6, 7];
        let k = meta_describe_key(key(MT_FLAG_DECOR_VIRT_TEXT_INLINE));
        meta_add(&mut meta, &k);
        assert_eq!(meta, [4, 4, 5, 6, 7]);
        meta_sub(&mut meta, &k);
        assert_eq!(meta, [3, 4, 5, 6, 7]);
        meta_apply_delta(&mut meta, &[0; META_COUNT], &k);
        assert_eq!(meta, [2, 4, 5, 6, 7]);
    }
}
