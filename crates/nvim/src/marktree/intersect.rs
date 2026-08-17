#![deny(unsafe_op_in_unsafe_fn)]

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
//! Every set is an `Intersection`, klib's `kvec_withinit_t`: four ids inline in
//! the node itself, moving to the heap beyond that. [`IdSet`] holds the raw
//! pointer to one rather than a `&mut Intersection`, because while the set is
//! inline the `items` field points at the very bytes such a reference would
//! cover, and reborrowing them invalidates it.

use core::ffi::c_void;
use core::{ptr, slice};

use crate::memory::{xmalloc, xrealloc};
use crate::types::{Intersection, uint64_t};

use super::node::INTERSECT_INLINE;

/// A borrowed view of one node's set of covering mark ids.
#[derive(Copy, Clone)]
pub struct IdSet {
    set: *mut Intersection,
}

impl IdSet {
    /// # Safety
    /// `set` must name a live `Intersection` that outlives the view, and no
    /// other view of the same set may be used while this one is.
    #[inline]
    pub unsafe fn new(set: *mut Intersection) -> Self {
        IdSet { set }
    }

    /// `size`, `capacity` and `items` in one read.
    fn parts(&self) -> (usize, usize, *mut uint64_t) {
        // SAFETY: `self.set` names a live `Intersection` per `IdSet::new`, so
        // its three header fields are initialised.
        unsafe { ((*self.set).size, (*self.set).capacity, (*self.set).items) }
    }

    /// The struct's own inline array, freshly derived.
    fn inline_array(&self) -> *mut uint64_t {
        // SAFETY: `self.set` is live, so `init_array` is a field of it; this
        // only takes its address.
        unsafe { (&raw mut (*self.set).init_array).cast() }
    }

    /// Where the ids actually live.
    ///
    /// While the set is inline, `items` names the containing struct's own
    /// `init_array`; re-derive that address rather than trusting the stored
    /// pointer, which is what keeps this sound however the node was reached.
    fn base(&self) -> *mut uint64_t {
        let (_, _, items) = self.parts();
        let inline = self.inline_array();
        if items == inline { inline } else { items }
    }

    fn set_len(&self, len: usize) {
        // SAFETY: `self.set` is live and no other view of it is in use.
        unsafe { (*self.set).size = len };
    }

    /// Point the set at `items`, which holds room for `capacity` ids.
    fn set_storage(&self, items: *mut uint64_t, capacity: usize) {
        // SAFETY: `self.set` is live and no other view of it is in use.
        unsafe { (*self.set).items = items };
        // SAFETY: as above.
        unsafe { (*self.set).capacity = capacity };
    }

    /// `kvi_init`: empty the set and point it at its own inline array. Only
    /// valid where the containing struct will stay put.
    pub fn init(&self) {
        self.set_storage(self.inline_array(), INTERSECT_INLINE);
        self.set_len(0);
    }

    /// Still living in the containing struct's own array, not on the heap.
    pub fn is_inline(&self) -> bool {
        let (_, _, items) = self.parts();
        items == self.inline_array()
    }

    pub fn len(&self) -> usize {
        self.parts().0
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_slice(&self) -> &[uint64_t] {
        let len = self.len();
        if len == 0 {
            return &[];
        }
        // SAFETY: `base` names `len` initialised, contiguous ids — either the
        // live struct's own inline array or the heap buffer `reserve` grew.
        unsafe { slice::from_raw_parts(self.base(), len) }
    }

    #[allow(clippy::mut_from_ref)]
    fn as_mut_slice(&self) -> &mut [uint64_t] {
        let len = self.len();
        if len == 0 {
            return &mut [];
        }
        // SAFETY: as `as_slice`, and `IdSet::new` promises no other view of the
        // same set is in use, so this is the only slice over those ids.
        unsafe { slice::from_raw_parts_mut(self.base(), len) }
    }

    /// Make room for `extra` more ids, doubling onto the heap if the inline
    /// array or the current buffer is too small.
    fn reserve(&self, extra: usize) {
        let (len, capacity, items) = self.parts();
        let needed = len + extra;
        if needed <= capacity {
            return;
        }
        let mut grown = capacity.max(INTERSECT_INLINE);
        while grown < needed {
            grown <<= 1;
        }
        let bytes = grown * size_of::<uint64_t>();
        let base = self.base();
        let moved: *mut uint64_t = if items == self.inline_array() {
            // SAFETY: `xmalloc` hands back `bytes` of suitably aligned storage
            // and never returns null.
            let heap = unsafe { xmalloc(bytes) }.cast::<uint64_t>();
            // SAFETY: `base` holds `len` ids and the fresh buffer is at least
            // `needed >= len` ids wide; the two cannot overlap.
            unsafe { ptr::copy_nonoverlapping(base, heap, len) };
            heap
        } else {
            // SAFETY: `base` is this set's own heap buffer, so `xrealloc` may
            // grow it in place or move it, and it never returns null.
            unsafe { xrealloc(base.cast(), bytes) }.cast()
        };
        self.set_storage(moved, grown);
    }

    /// Append, without regard for the ordering.
    pub fn push(&self, id: uint64_t) {
        self.reserve(1);
        let len = self.len();
        // SAFETY: `reserve` left room for one more id past the `len` in use.
        unsafe { self.base().add(len).write(id) };
        self.set_len(len + 1);
    }

    pub fn extend_from_slice(&self, ids: &[uint64_t]) {
        if ids.is_empty() {
            return;
        }
        self.reserve(ids.len());
        let end = self.base().wrapping_add(self.len());
        // SAFETY: `reserve` left room for `ids.len()` more ids at `end`; `ids`
        // is a live slice and every caller passes another set's ids, so the
        // two ranges do not overlap.
        unsafe { ptr::copy_nonoverlapping(ids.as_ptr(), end, ids.len()) };
        self.set_len(self.len() + ids.len());
    }

    /// Insert at `at`, shifting the tail up.
    pub fn insert(&self, at: usize, id: uint64_t) {
        self.push(id);
        let items = self.as_mut_slice();
        items[at..].rotate_right(1);
    }

    pub fn truncate(&self, len: usize) {
        debug_assert!(len <= self.len());
        self.set_len(len);
    }

    pub fn clear(&self) {
        self.set_len(0);
    }

    /// Hand back the heap buffer, if the set ever left its inline array, and
    /// leave the set empty and inline. The caller frees it.
    #[must_use = "the returned buffer must be freed"]
    pub fn take_heap(&self) -> *mut c_void {
        let (_, _, items) = self.parts();
        let heap = if items == self.inline_array() || items.is_null() {
            ptr::null_mut()
        } else {
            items.cast()
        };
        self.init();
        heap
    }

    /// Take `src`'s contents over, leaving `src` alone. The inline case has to
    /// copy: `src`'s `items` names `src`'s own bytes, which the caller is about
    /// to stop using.
    pub fn move_from(&self, src: &IdSet) {
        let (len, capacity, items) = src.parts();
        let dest = self.inline_array();
        let items = if items == src.inline_array() {
            // SAFETY: `src`'s ids live in its own inline array, which holds at
            // most `INTERSECT_INLINE` of them — so does this set's, and the two
            // structs are distinct, so the ranges do not overlap.
            unsafe { ptr::copy_nonoverlapping(src.base(), dest, len) };
            dest
        } else {
            items
        };
        self.set_storage(items, capacity);
        self.set_len(len);
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

    /// Add `id`, keeping the set sorted. Optimised for the common case of the
    /// new id being the largest.
    pub fn insert_sorted(&self, id: uint64_t) {
        self.push(id);
        let items = self.as_mut_slice();
        let mut i = items.len() - 1;
        loop {
            if i > 0 && items[i - 1] > id {
                items[i] = items[i - 1];
                i -= 1;
            } else {
                items[i] = id;
                return;
            }
        }
    }

    /// Remove `id` if present. `strict` asserts that it was.
    pub fn remove(&self, id: uint64_t, strict: bool) {
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
            items.copy_within(at + 1.., at);
            self.set_len(items.len() - 1);
        }
    }
}

/// Move what `x` and `y` have in common into `m`, leaving each of them with
/// only what the other lacked. All three stay sorted.
pub fn intersect_merge(m: &IdSet, x: &IdSet, y: &IdSet) {
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
fn compact_tail(set: &IdSet, to: usize, from: usize) -> usize {
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
pub fn intersect_mov(x: &IdSet, y: &IdSet, w: &IdSet, d: &IdSet) {
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
pub fn intersect_common(i: &IdSet, x: &IdSet, y: &IdSet) {
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
pub fn intersect_add(x: &IdSet, y: &IdSet) {
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
pub fn intersect_sub(x: &IdSet, y: &IdSet) {
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
    use crate::memory::xfree;

    /// An `Intersection` that stays put for the length of a test. `init` points
    /// `items` at the struct's own array, so it must not be moved afterwards.
    struct Fixture(Box<Intersection>);

    impl Fixture {
        fn new(ids: &[uint64_t]) -> Self {
            let f = Fixture(Box::new(Intersection {
                size: 0,
                capacity: 0,
                items: ptr::null_mut(),
                init_array: [0; INTERSECT_INLINE],
            }));
            let set = f.set();
            set.init();
            set.extend_from_slice(ids);
            f
        }

        fn set(&self) -> IdSet {
            unsafe { IdSet::new(&raw const *self.0 as *mut Intersection) }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            unsafe { xfree(self.set().take_heap()) };
        }
    }

    #[test]
    fn grows_off_the_inline_array_and_keeps_the_order() {
        let f = Fixture::new(&[]);
        let set = f.set();
        for id in (0..20).rev() {
            set.insert_sorted(id * 2);
        }
        assert_eq!(set.len(), 20);
        assert_eq!(
            set.as_slice(),
            &(0..20).map(|i| i * 2).collect::<Vec<_>>()[..]
        );
        assert!(set.contains(38));
        assert!(!set.contains(39));
    }

    #[test]
    fn removing_the_only_id_empties_the_set() {
        let f = Fixture::new(&[4]);
        f.set().remove(4, true);
        assert!(f.set().is_empty());
        // Removing an absent id is a no-op when not strict.
        f.set().remove(4, false);
        assert!(f.set().is_empty());
    }

    #[test]
    fn removing_from_the_middle_closes_the_gap() {
        let f = Fixture::new(&[2, 4, 6, 8]);
        f.set().remove(4, true);
        assert_eq!(f.set().as_slice(), &[2, 6, 8]);
    }

    #[test]
    #[should_panic(expected = "seen")]
    fn a_strict_removal_of_an_absent_id_is_a_bug() {
        let f = Fixture::new(&[2, 6]);
        f.set().remove(4, true);
    }

    #[test]
    fn merge_moves_the_shared_ids_out_of_both() {
        let m = Fixture::new(&[]);
        let x = Fixture::new(&[1, 2, 3, 5]);
        let y = Fixture::new(&[2, 3, 4]);
        intersect_merge(&m.set(), &x.set(), &y.set());
        assert_eq!(m.set().as_slice(), &[2, 3]);
        assert_eq!(x.set().as_slice(), &[1, 5]);
        assert_eq!(y.set().as_slice(), &[4]);
    }

    #[test]
    fn common_leaves_both_inputs_alone() {
        let i = Fixture::new(&[]);
        let x = Fixture::new(&[1, 2, 3]);
        let y = Fixture::new(&[2, 3, 9]);
        intersect_common(&i.set(), &x.set(), &y.set());
        assert_eq!(i.set().as_slice(), &[2, 3]);
        assert_eq!(x.set().as_slice(), &[1, 2, 3]);
        assert_eq!(y.set().as_slice(), &[2, 3, 9]);
    }

    #[test]
    fn union_and_difference_keep_the_set_sorted() {
        let x = Fixture::new(&[2, 6]);
        let y = Fixture::new(&[1, 2, 4, 8, 9, 10, 11]);
        intersect_add(&x.set(), &y.set());
        assert_eq!(x.set().as_slice(), &[1, 2, 4, 6, 8, 9, 10, 11]);
        intersect_sub(&x.set(), &y.set());
        assert_eq!(x.set().as_slice(), &[6]);
    }

    #[test]
    fn difference_with_nothing_in_common_keeps_everything() {
        let x = Fixture::new(&[1, 3, 5]);
        let y = Fixture::new(&[2, 4]);
        intersect_sub(&x.set(), &y.set());
        assert_eq!(x.set().as_slice(), &[1, 3, 5]);
    }

    #[test]
    fn a_moved_child_takes_on_what_its_old_parent_covered() {
        // `x` (the old parent) covered 1 and 3; `y` (the new one) covers 3 and
        // 5. The child `w` must end up covering everything the old parent did
        // that the new one does not, and `d` collects what `y`'s other children
        // now have to carry themselves.
        let x = Fixture::new(&[1, 3]);
        let y = Fixture::new(&[3, 5]);
        let w = Fixture::new(&[7]);
        let d = Fixture::new(&[]);
        intersect_mov(&x.set(), &y.set(), &w.set(), &d.set());
        assert_eq!(w.set().as_slice(), &[1, 7]);
        assert_eq!(y.set().as_slice(), &[3]);
        assert_eq!(d.set().as_slice(), &[5]);
    }

    #[test]
    fn moving_a_set_off_the_inline_array_copies_it() {
        let src = Fixture::new(&[1, 2, 3]);
        let dest = Fixture::new(&[]);
        dest.set().move_from(&src.set());
        assert_eq!(dest.set().as_slice(), &[1, 2, 3]);
        // The copy is dest's own; src's bytes are no longer referenced.
        src.set().clear();
        assert_eq!(dest.set().as_slice(), &[1, 2, 3]);
    }
}
