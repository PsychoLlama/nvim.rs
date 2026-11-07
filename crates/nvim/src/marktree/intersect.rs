#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! The intersection sets that make range queries local.
//!
//! A node records the ids of every paired mark whose range covers the *whole*
//! of that node's span without covering its parent's. So a query for "which
//! ranges cover this position" reads one set per level on the way down instead
//! of scanning every range in the buffer, and a range that spans a million
//! lines is stored on a handful of nodes rather than smeared over all of them.
//!
//! Ids in a set are start-side handles (`MARKTREE_END_FLAG` clear) and are kept
//! sorted, so the set algebra the rebalancing needs — union, difference,
//! intersection, and the three-way shuffle when a child changes parent — is a
//! linear merge.
//!
//! Every set is an [`IdSet`]: up to four ids inline in the node, a `Vec`
//! beyond that — klib's `kvec_withinit_t`, minus its pointer into itself.

use crate::os::libc::uint64_t;

/// Nested to keep the name out of the flat cdef namespace `ffigen` builds.
mod sizes {
    /// How many ids a set holds before it goes to the heap.
    pub(super) const INTERSECT_INLINE: usize = 4;
}
use sizes::INTERSECT_INLINE;

/// One node's set of covering mark ids.
#[derive(Clone, Debug)]
pub enum IdSet {
    /// Short sets, which is nearly all of them.
    Inline {
        len: u8,
        ids: [uint64_t; INTERSECT_INLINE],
    },
    /// A set that once outgrew the inline array. It stays on the heap when it
    /// shrinks again, as the kvec did.
    Heap(Vec<uint64_t>),
}

impl Default for IdSet {
    fn default() -> Self {
        IdSet::EMPTY
    }
}

impl IdSet {
    /// An empty inline set.
    pub const EMPTY: IdSet = IdSet::Inline {
        len: 0,
        ids: [0; INTERSECT_INLINE],
    };

    /// A set holding `ids`, in the order given.
    #[cfg(test)]
    fn from_slice(ids: &[uint64_t]) -> IdSet {
        let mut set = IdSet::EMPTY;
        set.extend_from_slice(ids);
        set
    }

    /// Still living in the node's own array, not on the heap.
    pub fn is_inline(&self) -> bool {
        matches!(self, IdSet::Inline { .. })
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            IdSet::Inline { len, .. } => usize::from(*len),
            IdSet::Heap(ids) => ids.len(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn as_slice(&self) -> &[uint64_t] {
        match self {
            IdSet::Inline { len, ids } => &ids[..usize::from(*len)],
            IdSet::Heap(ids) => ids,
        }
    }

    #[inline]
    fn as_mut_slice(&mut self) -> &mut [uint64_t] {
        match self {
            IdSet::Inline { len, ids } => &mut ids[..usize::from(*len)],
            IdSet::Heap(ids) => ids,
        }
    }

    /// Append, without regard for the ordering.
    pub fn push(&mut self, id: uint64_t) {
        match self {
            IdSet::Inline { len, ids } if usize::from(*len) < INTERSECT_INLINE => {
                ids[usize::from(*len)] = id;
                *len += 1;
            }
            IdSet::Inline { ids, .. } => {
                let mut heap = Vec::with_capacity(2 * INTERSECT_INLINE);
                heap.extend_from_slice(ids);
                heap.push(id);
                *self = IdSet::Heap(heap);
            }
            IdSet::Heap(ids) => ids.push(id),
        }
    }

    pub fn extend_from_slice(&mut self, ids: &[uint64_t]) {
        for &id in ids {
            self.push(id);
        }
    }

    /// Insert at `at`, shifting the tail up.
    pub fn insert(&mut self, at: usize, id: uint64_t) {
        if let IdSet::Heap(ids) = self {
            ids.insert(at, id);
        } else {
            self.push(id);
            self.as_mut_slice()[at..].rotate_right(1);
        }
    }

    pub fn truncate(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.len());
        match self {
            IdSet::Inline { len, .. } => {
                *len = u8::try_from(new_len).expect("an inline set holds a handful of ids");
            }
            IdSet::Heap(ids) => ids.truncate(new_len),
        }
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }

    /// Is `id` in the set? Bails at the first larger id, as the C did — the
    /// sets are short enough that the scan beats a binary search.
    pub fn contains(&self, id: uint64_t) -> bool {
        for &have in self.as_slice() {
            if have == id {
                return true;
            } else if have >= id {
                return false;
            }
        }
        false
    }

    /// Add `id`, keeping the set sorted: after every id not greater than it,
    /// as upstream's shift from the end placed it. A set covering a long
    /// stretch of a big buffer holds hundreds of ids, so the place is found by
    /// bisection and the tail moves in one copy.
    pub fn insert_sorted(&mut self, id: uint64_t) {
        let at = self.as_slice().partition_point(|&have| have <= id);
        self.insert(at, id);
    }

    /// Remove `id` if present. `strict` asserts that it was.
    pub fn remove(&mut self, id: uint64_t, strict: bool) {
        let items = self.as_mut_slice();
        let mut at = items.len();
        let mut seen = false;
        for (i, &have) in items.iter().enumerate() {
            if have < id {
                continue;
            }
            at = i;
            seen = have == id;
            break;
        }
        if strict {
            debug_assert!(seen, "seen");
        }
        if seen {
            let len = items.len();
            items.copy_within(at + 1.., at);
            self.truncate(len - 1);
        }
    }
}

/// Move what `x` and `y` have in common into `m`, leaving each of them with
/// only what the other lacked. All three stay sorted.
pub fn intersect_merge(m: &mut IdSet, x: &mut IdSet, y: &mut IdSet) {
    let (mut xi, mut yi, mut xn, mut yn) = (0, 0, 0, 0);
    while xi < x.len() && yi < y.len() {
        let (a, b) = (x.as_slice()[xi], y.as_slice()[yi]);
        if a == b {
            m.push(a);
            xi += 1;
            yi += 1;
        } else if a < b {
            x.as_mut_slice()[xn] = a;
            xn += 1;
            xi += 1;
        } else {
            y.as_mut_slice()[yn] = b;
            yn += 1;
            yi += 1;
        }
    }
    xn += compact_tail(x, xn, xi);
    yn += compact_tail(y, yn, yi);
    x.truncate(xn);
    y.truncate(yn);
}

/// Shift `set[from..]` down to `to` and answer how many were moved.
fn compact_tail(set: &mut IdSet, to: usize, from: usize) -> usize {
    let items = set.as_mut_slice();
    if from >= items.len() {
        return 0;
    }
    let moved = items.len() - from;
    items.copy_within(from.., to);
    moved
}

/// `w` used to be a child of `x` and is now a child of `y`; adjust the sets so
/// that `w` covers what it inherits from its new parent's ancestors and no
/// longer relies on its old one.
///
/// `d` collects the ids that `y`'s *other* children have to take on, because
/// `y` can only keep an id that every one of its children now shares.
pub fn intersect_mov(x: &IdSet, y: &mut IdSet, w: &mut IdSet, d: &mut IdSet) {
    let (mut wi, mut yi, mut wn, mut yn, mut xi) = (0, 0, 0, 0, 0);
    while wi < w.len() || xi < x.len() {
        let take_w = wi < w.len() && (xi >= x.len() || x.as_slice()[xi] >= w.as_slice()[wi]);
        if take_w {
            let id = w.as_slice()[wi];
            if xi < x.len() && x.as_slice()[xi] == id {
                xi += 1;
            }
            // Now w's id is strictly below x's.
            while yi < y.len() && y.as_slice()[yi] < id {
                d.push(y.as_slice()[yi]);
                yi += 1;
            }
            if yi < y.len() && y.as_slice()[yi] == id {
                y.as_mut_slice()[yn] = id;
                yn += 1;
                yi += 1;
                wi += 1;
            } else {
                w.as_mut_slice()[wn] = id;
                wn += 1;
                wi += 1;
            }
        } else {
            let id = x.as_slice()[xi];
            while yi < y.len() && y.as_slice()[yi] < id {
                d.push(y.as_slice()[yi]);
                yi += 1;
            }
            if yi < y.len() && y.as_slice()[yi] == id {
                y.as_mut_slice()[yn] = id;
                yn += 1;
                yi += 1;
                xi += 1;
            } else if wi == wn {
                // Nothing has been consumed from `w` yet, so there is no hole
                // to write into: make one, and skip past the id just added.
                w.insert(wn, id);
                wn += 1;
                wi += 1;
                xi += 1;
            } else {
                debug_assert!(wn < wi, "wn < wi");
                w.as_mut_slice()[wn] = id;
                wn += 1;
                xi += 1;
            }
        }
    }
    if yi < y.len() {
        d.extend_from_slice(&y.as_slice()[yi..]);
    }
    w.truncate(wn);
    y.truncate(yn);
}

/// `i = x & y`, appended to whatever `i` already holds.
pub fn intersect_common(i: &mut IdSet, x: &IdSet, y: &IdSet) {
    let (mut xi, mut yi) = (0, 0);
    while xi < x.len() && yi < y.len() {
        let (a, b) = (x.as_slice()[xi], y.as_slice()[yi]);
        if a == b {
            i.push(a);
            xi += 1;
            yi += 1;
        } else if a < b {
            xi += 1;
        } else {
            yi += 1;
        }
    }
}

/// `x |= y`, in place and in order.
pub fn intersect_add(x: &mut IdSet, y: &IdSet) {
    let (mut xi, mut yi) = (0, 0);
    while xi < x.len() && yi < y.len() {
        let (a, b) = (x.as_slice()[xi], y.as_slice()[yi]);
        if a == b {
            xi += 1;
            yi += 1;
        } else if b < a {
            x.insert(xi, b);
            xi += 1;
            yi += 1;
        } else {
            xi += 1;
        }
    }
    if yi < y.len() {
        x.extend_from_slice(&y.as_slice()[yi..]);
    }
}

/// `x &= ~y`, in place.
pub fn intersect_sub(x: &mut IdSet, y: &IdSet) {
    let (mut xi, mut yi, mut xn) = (0, 0, 0);
    while xi < x.len() && yi < y.len() {
        let (a, b) = (x.as_slice()[xi], y.as_slice()[yi]);
        if a == b {
            xi += 1;
            yi += 1;
        } else if a < b {
            x.as_mut_slice()[xn] = a;
            xn += 1;
            xi += 1;
        } else {
            yi += 1;
        }
    }
    xn += compact_tail(x, xn, xi);
    x.truncate(xn);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(ids: &[uint64_t]) -> IdSet {
        IdSet::from_slice(ids)
    }

    #[test]
    fn grows_off_the_inline_array_and_keeps_the_order() {
        let mut set = set(&[]);
        for id in (0..20).rev() {
            set.insert_sorted(id * 2);
        }
        assert!(!set.is_inline());
        assert_eq!(set.len(), 20);
        assert_eq!(
            set.as_slice(),
            &(0..20).map(|i| i * 2).collect::<Vec<_>>()[..]
        );
        assert!(set.contains(38));
        assert!(!set.contains(39));
    }

    #[test]
    fn an_insert_in_the_middle_of_a_full_inline_set_spills_in_order() {
        let mut set = set(&[1, 2, 4, 5]);
        assert!(set.is_inline());
        set.insert(2, 3);
        assert!(!set.is_inline());
        assert_eq!(set.as_slice(), &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn removing_the_only_id_empties_the_set() {
        let mut f = set(&[4]);
        f.remove(4, true);
        assert!(f.is_empty());
        // Removing an absent id is a no-op when not strict.
        f.remove(4, false);
        assert!(f.is_empty());
    }

    #[test]
    fn removing_from_the_middle_closes_the_gap() {
        let mut f = set(&[2, 4, 6, 8]);
        f.remove(4, true);
        assert_eq!(f.as_slice(), &[2, 6, 8]);
    }

    #[test]
    #[should_panic(expected = "seen")]
    fn a_strict_removal_of_an_absent_id_is_a_bug() {
        let mut f = set(&[2, 6]);
        f.remove(4, true);
    }

    #[test]
    fn merge_moves_the_shared_ids_out_of_both() {
        let mut m = set(&[]);
        let mut x = set(&[1, 2, 3, 5]);
        let mut y = set(&[2, 3, 4]);
        intersect_merge(&mut m, &mut x, &mut y);
        assert_eq!(m.as_slice(), &[2, 3]);
        assert_eq!(x.as_slice(), &[1, 5]);
        assert_eq!(y.as_slice(), &[4]);
    }

    #[test]
    fn common_leaves_both_inputs_alone() {
        let mut i = set(&[]);
        let x = set(&[1, 2, 3]);
        let y = set(&[2, 3, 9]);
        intersect_common(&mut i, &x, &y);
        assert_eq!(i.as_slice(), &[2, 3]);
        assert_eq!(x.as_slice(), &[1, 2, 3]);
        assert_eq!(y.as_slice(), &[2, 3, 9]);
    }

    #[test]
    fn union_and_difference_keep_the_set_sorted() {
        let mut x = set(&[2, 6]);
        let y = set(&[1, 2, 4, 8, 9, 10, 11]);
        intersect_add(&mut x, &y);
        assert_eq!(x.as_slice(), &[1, 2, 4, 6, 8, 9, 10, 11]);
        intersect_sub(&mut x, &y);
        assert_eq!(x.as_slice(), &[6]);
    }

    #[test]
    fn difference_with_nothing_in_common_keeps_everything() {
        let mut x = set(&[1, 3, 5]);
        let y = set(&[2, 4]);
        intersect_sub(&mut x, &y);
        assert_eq!(x.as_slice(), &[1, 3, 5]);
    }

    #[test]
    fn a_moved_child_takes_on_what_its_old_parent_covered() {
        // `x` (the old parent) covered 1 and 3; `y` (the new one) covers 3 and
        // 5. The child `w` must end up covering everything the old parent did
        // that the new one does not, and `d` collects what `y`'s other children
        // now have to carry themselves.
        let x = set(&[1, 3]);
        let mut y = set(&[3, 5]);
        let mut w = set(&[7]);
        let mut d = set(&[]);
        intersect_mov(&x, &mut y, &mut w, &mut d);
        assert_eq!(w.as_slice(), &[1, 7]);
        assert_eq!(y.as_slice(), &[3]);
        assert_eq!(d.as_slice(), &[5]);
    }

    /// The table `marktree_spec.lua` drove through an `intersect_mov_test`
    /// shim that existed only to reach this function from Lua. Each row is
    /// (old parent's set, new parent's set, the child's own set) and the answer
    /// is (the child's set afterwards, what the new parent's other children
    /// have to take on).
    #[test]
    fn a_move_redistributes_what_each_side_no_longer_covers() {
        #[track_caller]
        fn mov(x: &[uint64_t], y: &[uint64_t], w: &[uint64_t]) -> (Vec<uint64_t>, Vec<uint64_t>) {
            let (x, mut y, mut w, mut d) = (set(x), set(y), set(w), set(&[]));
            intersect_mov(&x, &mut y, &mut w, &mut d);
            (w.as_slice().to_vec(), d.as_slice().to_vec())
        }
        let none: Vec<uint64_t> = Vec::new();

        // Nothing to inherit and nothing to shed.
        assert_eq!(mov(&[], &[2, 3], &[2, 3]), (none.clone(), none.clone()));
        // The child keeps what neither parent covers.
        assert_eq!(mov(&[], &[], &[2, 3]), (vec![2, 3], none.clone()));
        // What the old parent covered and the new one does not moves down.
        assert_eq!(mov(&[2, 3], &[], &[]), (vec![2, 3], none.clone()));
        // What the new parent covers and the child does not moves out to the
        // new parent's other children.
        assert_eq!(mov(&[], &[2, 3], &[]), (none.clone(), vec![2, 3]));

        // Overlapping sets: only the difference each way moves.
        assert_eq!(mov(&[1, 2, 5], &[2, 3], &[3]), (vec![1, 5], none.clone()));
        assert_eq!(mov(&[1, 2, 5], &[5, 10], &[10]), (vec![1, 2], none.clone()));
        assert_eq!(mov(&[1, 2], &[5, 10], &[10]), (vec![1, 2], vec![5]));

        // Interleaved, so every merge step alternates sides.
        let (x, y) = ([1, 3, 5, 7, 9], [2, 4, 6, 8, 10]);
        assert_eq!(
            mov(&x, &y, &[]),
            (vec![1, 3, 5, 7, 9], vec![2, 4, 6, 8, 10])
        );
        assert_eq!(mov(&x, &y, &[4, 8]), (vec![1, 3, 5, 7, 9], vec![2, 6, 10]));

        // Sets sharing some ids: 3, 6 and 9 are covered both before and after,
        // so they neither move down nor move out.
        let (x, y) = ([1, 3, 4, 6, 7, 9], [2, 3, 5, 6, 8, 9]);
        assert_eq!(mov(&x, &y, &[]), (vec![1, 4, 7], vec![2, 5, 8]));
        assert_eq!(mov(&x, &y, &[2, 5, 8]), (vec![1, 4, 7], none.clone()));
        // And ids the child already carries that neither parent mentions stay.
        assert_eq!(
            mov(&x, &y, &[0, 2, 5, 8, 10]),
            (vec![0, 1, 4, 7, 10], none.clone())
        );
    }
}
