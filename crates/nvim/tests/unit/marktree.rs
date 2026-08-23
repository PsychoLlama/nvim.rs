//! The extmark B-tree, driven through the same entry points the editor uses.
//!
//! This is a port of the shape of `test/unit/marktree_spec.lua`: build a tree
//! and a plain-`Vec` shadow of where every mark ought to be, then after each
//! batch of operations check that walking the tree in order visits exactly the
//! shadow's marks, in the shadow's order, and that `marktree_check` — which
//! verifies the fill bounds, the relative-position encoding, the `id2node`
//! index, the parent back-pointers and the meta counts — still passes.
//!
//! The Lua spec does thousands of random operations. This one runs a small
//! deterministic set under Miri and a larger one otherwise: the point of
//! having it here at all is that Miri gets to watch the pointer work, which
//! nothing else in the tree does. Deletion order is shuffled by a fixed LCG so
//! the rebalancing paths (borrow from a sibling, merge with one, shrink the
//! root) are all reached without depending on the platform's hasher.

use std::ffi::{c_int, c_uint};
use std::ptr;

use c2rust_neovim::marktree::check::{
    MarkEnd, marktree_check, marktree_check_intersections, marktree_del_pair_test,
    marktree_put_test, mt_right_test,
};
use c2rust_neovim::marktree::iter::{
    marktree_itr_current, marktree_itr_first, marktree_itr_get, marktree_itr_get_filter,
    marktree_itr_get_overlap, marktree_itr_next, marktree_itr_next_filter,
    marktree_itr_step_overlap,
};
use c2rust_neovim::marktree::key::mt_end;
use c2rust_neovim::marktree::meta::META_COUNT;
use c2rust_neovim::marktree::splice::marktree_splice;
use c2rust_neovim::marktree::{
    marktree_clear, marktree_del_itr, marktree_lookup_ns, marktree_move,
};
use c2rust_neovim::types::{
    MTKey, MTNode, MTPair, MTPos, Map_uint64_t_ptr_t, MarkTree, MarkTreeIter, MarkTreeIter_s,
};

/// The namespace every mark in this file lives in.
const NS: u32 = 10;

/// One count per meta kind; a filter selects a kind by holding a non-zero mask
/// for it. The walks below only ever ask for the first kind.
type MetaFilter = [u32; META_COUNT];

/// One end of a mark, spelled out.
fn at(row: i32, col: i32, right_gravity: bool) -> MarkEnd {
    MarkEnd {
        row,
        col,
        right_gravity,
    }
}

/// Select inline virtual text and nothing else.
fn inline_filter() -> MetaFilter {
    let mut filter: MetaFilter = [0; META_COUNT];
    filter[0] = u32::MAX;
    filter
}

/// `MarkTree` and `MarkTreeIter` are both valid all-zero — the Lua spec relies
/// on it too (`ffi.new` zero-initializes) — and `marktree_clear` restores that
/// state, so a tree can be dropped by clearing it.
fn zeroed_tree() -> MarkTree {
    MarkTree {
        root: ptr::null_mut::<MTNode>(),
        meta_root: [0; 5],
        n_keys: 0,
        n_nodes: 0,
        id2node: [unsafe { std::mem::zeroed::<Map_uint64_t_ptr_t>() }; 1],
    }
}

fn zeroed_iter() -> MarkTreeIter {
    MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ptr::null_mut::<MTNode>(),
        i: 0,
        s: [MarkTreeIter_s { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }
}

/// Where one mark is expected to be.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Shadow {
    id: u32,
    row: i32,
    col: i32,
    right: bool,
}

/// A tree plus the model of what it should hold.
struct Tree {
    tree: Box<MarkTree>,
    shadow: Vec<Shadow>,
    next_id: u32,
}

impl Tree {
    fn new() -> Self {
        Tree {
            tree: Box::new(zeroed_tree()),
            shadow: Vec::new(),
            next_id: 0,
        }
    }

    fn put(&mut self, row: i32, col: i32, right: bool) -> u32 {
        self.next_id += 1;
        let id = self.next_id;
        unsafe {
            marktree_put_test(&mut self.tree, NS, id, at(row, col, right), None, false);
        }
        self.shadow.push(Shadow {
            id,
            row,
            col,
            right,
        });
        id
    }

    /// A paired mark: one range from (row, col) to (end_row, end_col). Both
    /// halves land in the shadow, since the walk visits both.
    fn put_pair(&mut self, row: i32, col: i32, end_row: i32, end_col: i32) -> u32 {
        self.next_id += 1;
        let id = self.next_id;
        unsafe {
            marktree_put_test(
                &mut self.tree,
                NS,
                id,
                at(row, col, false),
                Some(at(end_row, end_col, true)),
                false,
            );
        }
        self.shadow.push(Shadow {
            id,
            row,
            col,
            right: false,
        });
        self.shadow.push(Shadow {
            id,
            row: end_row,
            col: end_col,
            right: true,
        });
        id
    }

    /// A range whose two halves each carry their own gravity, and which need
    /// not run forwards -- the tree accepts an end that sorts before its start,
    /// and `marktree_check_intersections` has an opinion about what that means.
    fn put_pair_gravity(&mut self, start: MarkEnd, stop: MarkEnd) -> u32 {
        self.next_id += 1;
        let id = self.next_id;
        unsafe { marktree_put_test(&mut self.tree, NS, id, start, Some(stop), false) };
        for half in [start, stop] {
            self.shadow.push(Shadow {
                id,
                row: half.row,
                col: half.col,
                right: half.right_gravity,
            });
        }
        id
    }

    /// A mark carrying the first meta kind (inline virtual text), which is what
    /// the filtered walk below selects on.
    fn put_meta(&mut self, row: i32, col: i32, right: bool, meta: bool) -> u32 {
        self.next_id += 1;
        let id = self.next_id;
        unsafe {
            marktree_put_test(&mut self.tree, NS, id, at(row, col, right), None, meta);
        }
        self.shadow.push(Shadow {
            id,
            row,
            col,
            right,
        });
        id
    }

    /// Move one half of a mark to a new position, the way the decoration layer
    /// does when a provider revises a mark in place.
    fn move_half(&mut self, id: u32, end: bool, row: i32, col: i32) {
        let mut itr = zeroed_iter();
        unsafe {
            marktree_lookup_ns(&mut self.tree, NS, id, end, Some(&mut itr));
            assert!(!itr.x.is_null(), "id {id} not found");
            marktree_move(&mut self.tree, &mut itr, row, col);
        }
        let half = self
            .shadow
            .iter_mut()
            .filter(|s| s.id == id)
            .nth(usize::from(end))
            .expect("id in shadow");
        half.row = row;
        half.col = col;
    }

    /// The tree's own invariants, without the shadow walk -- for the cases
    /// whose marks the shadow model cannot describe.
    fn check_tree(&mut self) {
        unsafe { marktree_check(&mut self.tree) };
        assert!(unsafe { marktree_check_intersections(&mut self.tree) });
    }

    /// Every range covering (row, col), by id, sorted.
    fn overlapping(&mut self, row: i32, col: i32) -> Vec<u32> {
        let mut itr = zeroed_iter();
        let mut pair: MTPair = unsafe { std::mem::zeroed() };
        let mut ids = Vec::new();
        if unsafe { marktree_itr_get_overlap(&mut self.tree, row, col, &mut itr) } {
            while unsafe { marktree_itr_step_overlap(&mut self.tree, &mut itr, &mut pair) } {
                ids.push(pair.start.id);
            }
        }
        ids.sort_unstable();
        ids
    }

    /// Walk the whole tree with a meta filter, answering (id, row, col) for
    /// every key it stops on.
    fn filtered(&mut self, filter: &MetaFilter) -> Vec<(u32, i32, i32)> {
        let mut itr = zeroed_iter();
        let mut seen = Vec::new();
        let stop = (i32::MAX, 0);
        if !unsafe {
            marktree_itr_get_filter(
                &mut self.tree,
                0,
                0,
                stop.0,
                stop.1,
                filter.as_ptr(),
                &mut itr,
            )
        } {
            return seen;
        }
        loop {
            let k: MTKey = unsafe { marktree_itr_current(&mut itr) };
            seen.push((k.id, k.pos.row, k.pos.col));
            if !unsafe {
                marktree_itr_next_filter(&mut self.tree, &mut itr, stop.0, stop.1, filter.as_ptr())
            } {
                break;
            }
        }
        seen
    }

    /// Delete the mark with this id, through a lookup by namespace and id.
    fn del(&mut self, id: u32) {
        let mut itr = zeroed_iter();
        unsafe {
            marktree_lookup_ns(&mut self.tree, NS, id, false, Some(&mut itr));
            marktree_del_itr(&mut self.tree, &mut itr, false);
        }
        let at = self.shadow.iter().position(|s| s.id == id).unwrap();
        self.shadow.remove(at);
    }

    fn del_pair(&mut self, id: u32) {
        unsafe { marktree_del_pair_test(&mut self.tree, NS, id) };
        self.shadow.retain(|s| s.id != id);
    }

    /// The edit `marktree_splice` models: at `start`, `old` was replaced by
    /// `new`, both extents being (rows, cols) relative to the start.
    fn splice(&mut self, start: (i32, i32), old: (i32, i32), new: (i32, i32)) {
        unsafe {
            marktree_splice(&mut self.tree, start.0, start.1, old.0, old.1, new.0, new.1);
        }
        shadow_splice(&mut self.shadow, start, old, new);
    }

    /// Assert the tree's own invariants and that an in-order walk matches the
    /// shadow. Answers the ids in tree order.
    fn check(&mut self) -> Vec<u32> {
        unsafe { marktree_check(&mut self.tree) };
        assert!(unsafe { marktree_check_intersections(&mut self.tree) });

        let mut expected = self.shadow.clone();
        expected.sort_by_key(|s| (s.row, s.col, s.right));

        let mut itr = zeroed_iter();
        let mut seen: Vec<Shadow> = Vec::new();
        let mut order: Vec<u32> = Vec::new();
        if unsafe { marktree_itr_first(&mut self.tree, &mut itr) } {
            loop {
                let k: MTKey = unsafe { marktree_itr_current(&mut itr) };
                seen.push(Shadow {
                    id: k.id,
                    row: k.pos.row,
                    col: k.pos.col,
                    right: unsafe { mt_right_test(k) },
                });
                order.push(k.id);
                if !unsafe { marktree_itr_next(&mut self.tree, &mut itr) } {
                    break;
                }
            }
        }

        assert_eq!(seen.len(), expected.len(), "mark count");
        assert_eq!(self.tree.n_keys, expected.len(), "n_keys");
        for (got, want) in seen.iter().zip(expected.iter()) {
            assert_eq!(
                (got.row, got.col, got.right),
                (want.row, want.col, want.right),
                "mark {} misplaced",
                got.id
            );
        }
        order
    }

    /// Every mark is findable by its id, and reports the position the shadow
    /// says it has. Ranges have two halves under one id; this checks the start.
    fn check_lookups(&mut self) {
        let mut itr = zeroed_iter();
        for want in self.shadow.clone() {
            if want.right {
                continue;
            }
            let k =
                unsafe { marktree_lookup_ns(&mut self.tree, NS, want.id, false, Some(&mut itr)) };
            assert_eq!(
                (k.pos.row, k.pos.col),
                (want.row, want.col),
                "id {}",
                want.id
            );
            assert!(!mt_end(k));
        }
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        unsafe { marktree_clear(&mut self.tree) };
    }
}

/// The model `marktree_splice` is checked against, straight off the Lua spec's
/// `shadowsplice`. A mark inside the deleted region collapses to one end of it,
/// chosen by gravity; a mark after it shifts by the difference in extent, and
/// only a mark on the *last* line of the old extent also moves by a column.
fn shadow_splice(shadow: &mut [Shadow], start: (i32, i32), old: (i32, i32), new: (i32, i32)) {
    let leq = |a: (i32, i32), b: (i32, i32)| a.0 < b.0 || (a.0 == b.0 && a.1 <= b.1);
    let old_end = (
        start.0 + old.0,
        (if old.0 == 0 { start.1 } else { 0 }) + old.1,
    );
    let new_end = (
        start.0 + new.0,
        (if new.0 == 0 { start.1 } else { 0 }) + new.1,
    );
    let delta = (new_end.0 - old_end.0, new_end.1 - old_end.1);
    for mark in shadow {
        let pos = (mark.row, mark.col);
        if !leq(start, pos) {
            continue;
        }
        if leq(pos, old_end) {
            let to = if mark.right { new_end } else { start };
            mark.row = to.0;
            mark.col = to.1;
        } else {
            if mark.row == old_end.0 {
                mark.col += delta.1;
            }
            mark.row += delta.0;
        }
    }
}

/// A fixed linear congruential generator, so a failure is reproducible and
/// nothing depends on the platform's hasher.
struct Rng(u64);

impl Rng {
    fn next(&mut self, bound: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.0 >> 33) as usize) % bound
    }
}

/// Miri interprets every operation, so the sizes are picked to still force a
/// tree several levels deep without taking minutes.
const fn scale(miri: usize, native: usize) -> usize {
    if cfg!(miri) { miri } else { native }
}

#[test]
fn keys_come_back_in_order_from_a_multi_level_tree() {
    // 2 * MT_BRANCH_FACTOR - 1 == 19 keys per node, so this is deep enough to
    // have split the root at least twice.
    let rows = scale(6, 30);
    let cols = scale(9, 40);
    let mut t = Tree::new();
    for row in 1..=rows as i32 {
        for col in 1..=cols as i32 {
            t.put(row, col, col % 2 == 0);
        }
    }
    assert!(unsafe { (*t.tree.root).level } >= 1);
    let order = t.check();
    // In-order iteration visits (1,1), (1,2), ... which is insertion order here.
    assert_eq!(order.len(), rows * cols);
    assert!(order.windows(2).all(|w| w[0] < w[1]));
    t.check_lookups();
}

#[test]
fn deleting_in_a_scrambled_order_rebalances_without_losing_a_mark() {
    let n = scale(60, 700);
    let mut t = Tree::new();
    for i in 0..n as i32 {
        t.put(i / 7, i % 7, i % 3 == 0);
    }
    t.check();

    let mut rng = Rng(0x5eed);
    let mut live: Vec<u32> = t.shadow.iter().map(|s| s.id).collect();
    let mut steps = 0;
    while !live.is_empty() {
        let at = rng.next(live.len());
        t.del(live.swap_remove(at));
        steps += 1;
        // Checking after every delete is too slow; every so often is enough to
        // localise a fault to a handful of operations.
        if steps % scale(7, 61) == 0 {
            t.check();
        }
    }
    let order = t.check();
    assert!(order.is_empty());
    assert_eq!(t.tree.n_keys, 0);
}

#[test]
fn a_splice_moves_every_mark_the_way_the_model_says() {
    let rows = scale(8, 40);
    let mut t = Tree::new();
    for row in 0..rows as i32 {
        for col in [0, 2, 5, 9] {
            t.put(row, col, col == 2 || col == 9);
        }
    }
    t.check();

    // An insertion within one line, an insertion of whole lines, a deletion
    // within one line, and a deletion spanning lines — which is the case that
    // collapses marks onto an endpoint by gravity.
    t.splice((1, 3), (0, 0), (0, 4));
    t.check();
    t.splice((2, 0), (0, 0), (3, 1));
    t.check();
    t.splice((0, 1), (0, 3), (0, 0));
    t.check();
    t.splice((4, 2), (2, 4), (0, 1));
    t.check();
    t.check_lookups();
}

/// Deleting everything after a splice has collapsed marks on top of each other
/// exercises the path where two keys share a position and the relative encoding
/// has to keep them apart.
#[test]
fn marks_collapsed_onto_one_position_stay_distinct() {
    let n = scale(20, 200);
    let mut t = Tree::new();
    for i in 0..n as i32 {
        t.put(1, i, i % 2 == 0);
    }
    t.splice((1, 0), (0, n as i32), (0, 0));
    let order = t.check();
    assert_eq!(order.len(), n);
    // Every mark is now at (1, 0): the left-gravity ones stayed at the start of
    // the deleted region and the right-gravity ones rode its end, which is the
    // same place because the replacement is empty.
    for s in &t.shadow {
        assert_eq!((s.row, s.col), (1, 0));
    }
    t.check_lookups();
}

#[test]
fn ranges_keep_their_two_halves_paired_across_a_rebalance() {
    let n = scale(12, 120);
    let mut t = Tree::new();
    // Nested ranges, so that the outermost covers whole subtrees and lands in
    // their intersection sets rather than being stored on every node.
    for i in 0..n as i32 {
        t.put_pair(0, i, (2 * n) as i32 - i, 0);
    }
    t.check();

    let mut rng = Rng(0xf00d);
    let mut live: Vec<u32> = (1..=n as u32).collect();
    while !live.is_empty() {
        let at = rng.next(live.len());
        t.del_pair(live.swap_remove(at));
        t.check();
    }
    assert_eq!(t.tree.n_keys, 0);
}

#[test]
fn an_iterator_positioned_by_coordinates_lands_on_the_first_mark_at_or_after_it() {
    let mut t = Tree::new();
    for row in 0..scale(4, 20) as i32 {
        t.put(row * 2, 5, false);
    }
    t.check();

    let mut itr = zeroed_iter();
    unsafe { marktree_itr_get(&mut t.tree, 0, 0, &mut itr) };
    let k = unsafe { marktree_itr_current(&mut itr) };
    assert_eq!((k.pos.row, k.pos.col), (0, 5));

    // Between two marks: lands on the later one.
    unsafe { marktree_itr_get(&mut t.tree, 1, 0, &mut itr) };
    let k = unsafe { marktree_itr_current(&mut itr) };
    assert_eq!((k.pos.row, k.pos.col), (2, 5));

    // Past the end: the iterator is exhausted.
    unsafe { marktree_itr_get(&mut t.tree, 1 << 20, 0, &mut itr) };
    assert!(itr.x.is_null());
}

#[test]
fn an_empty_tree_has_nothing_to_walk() {
    let mut t = Tree::new();
    let mut itr = zeroed_iter();
    assert!(!unsafe { marktree_itr_first(&mut t.tree, &mut itr) });
    assert_eq!(t.check(), Vec::<u32>::new());
}

/// `marktree_clear` has to free every node, not just the root's subtree, and
/// leave the tree usable again.
#[test]
fn clearing_a_tree_leaves_it_reusable() {
    let mut t = Tree::new();
    for i in 0..scale(30, 300) as i32 {
        t.put(i, i, false);
    }
    unsafe { marktree_clear(&mut t.tree) };
    t.shadow.clear();
    assert_eq!(t.tree.n_keys, 0);
    assert_eq!(t.tree.n_nodes, 0);
    assert!(t.tree.root.is_null());

    t.put(1, 1, false);
    t.check();
}

/// Guards the `find_key` boundary: a node holding exactly the maximum number
/// of keys splits on the next insertion, and the key that triggers the split
/// can be the smallest, the largest, or land in the middle.
#[test]
fn splitting_a_full_node_keeps_the_order_whichever_key_triggers_it() {
    // Columns cannot be negative — `marktree_check` asserts it — so the
    // "smallest" case is a column below the first key rather than below zero.
    for extra in [0_i32, 100, 21] {
        let mut t = Tree::new();
        for col in 0..19 {
            t.put(0, 2 + col * 2, false);
        }
        t.put(0, extra, false);
        let order = t.check();
        assert_eq!(order.len(), 20);
    }
}

#[test]
fn a_splice_that_touches_nothing_leaves_every_mark_alone() {
    let mut t = Tree::new();
    for i in 0..scale(10, 100) as i32 {
        t.put(i, 3, i % 2 == 0);
    }
    let before = t.check();
    // Entirely after the last mark.
    t.splice((1 << 16, 0), (0, 1), (0, 2));
    assert_eq!(t.check(), before);
}

/// `c_int`/`c_uint` are what the entry points take; this fails to compile if
/// the platform ever disagrees with the `i32`/`u32` used throughout.
const _: () = {
    assert!(size_of::<c_int>() == size_of::<i32>());
    assert!(size_of::<c_uint>() == size_of::<u32>());
};

/// `marktree_move` re-inserts a key at a new position rather than editing it in
/// place, so the case that matters is the one where the new slot looks like the
/// old one: the key has to be taken out and put back anyway, because its stored
/// position is relative to its node.
#[test]
fn moving_a_mark_onto_a_neighbouring_slot_keeps_the_order() {
    let mut t = Tree::new();
    t.put_pair_gravity(at(1, 1, false), at(1, 3, false));
    t.put_pair_gravity(at(1, 3, false), at(1, 3, false));
    let third = t.put_pair_gravity(at(1, 3, false), at(1, 3, false));
    t.put_pair_gravity(at(1, 3, false), at(1, 3, false));

    t.move_half(third, false, 1, 2);
    t.check_tree();
    t.check();
}

/// Moving one half of every range in a tree deep enough to have intersection
/// sets on its internal nodes: each move retracts the old covering records and
/// establishes new ones, and `marktree_check_intersections` recomputes them
/// from scratch to compare.
#[test]
fn moving_halves_of_overlapping_ranges_rebuilds_the_covering_records() {
    let n = scale(20, 300) as i32;
    let mut t = Tree::new();
    let ids: Vec<u32> = (1..=n)
        .map(|i| t.put_pair_gravity(at(1, i, false), at(2, n - i, false)))
        .collect();
    t.check_tree();

    for (i, &id) in ids.iter().enumerate() {
        // Alternate which half moves, so both the start's and the end's
        // re-insertion paths run.
        let end = i % 2 == 1;
        t.move_half(id, end, 1 + i32::from(end), n / 2 + i as i32);
        if i % 10 == 0 {
            t.check_tree();
        }
    }
    t.check_tree();
    t.check();
}

/// The overlap iterator answers "which ranges cover this position" without
/// walking them: the ranges covering a whole subtree are recorded on its root
/// node, and the leaf scan picks up the ones that only partly cover their leaf.
/// The model here is the same one the Lua spec kept -- a range from row1 to
/// row2 covers column 0 of every row strictly after row1 and up to row2,
/// because its start sits at a positive column.
#[test]
fn the_overlap_iterator_finds_every_range_covering_a_row() {
    let size = scale(45, 600);
    let mut t = Tree::new();
    let mut at_row: Vec<Vec<u32>> = vec![Vec::new(); 11];

    let mut k = 1i32;
    'fill: loop {
        for row1 in 1..=9 {
            for row2 in row1..=10 {
                if k > size as i32 {
                    break 'fill;
                }
                let id = t.put_pair_gravity(at(row1, k, false), at(row2, size as i32 - k, false));
                for row in at_row
                    .iter_mut()
                    .take(row2 as usize + 1)
                    .skip(row1 as usize + 1)
                {
                    row.push(id);
                }
                k += 1;
            }
        }
    }
    assert_eq!(t.tree.n_keys, 2 * size);
    assert!(unsafe { (*t.tree.root).level } >= scale(1, 2) as i16);
    t.check_tree();

    for row in 1..=10 {
        let mut want = at_row[row as usize].clone();
        want.sort_unstable();
        assert_eq!(t.overlapping(row, 0), want, "row {row}");
    }

    // A position no range reaches: past every end.
    assert_eq!(t.overlapping(11, 0), Vec::<u32>::new());
}

/// A splice over the *middle* of a stack of ranges. Every range here starts on
/// row 1 and ends on row 2, so deleting columns from row 0 leaves the covering
/// records alone while moving both halves -- which is the case that used to
/// desynchronise the two.
#[test]
fn splicing_across_a_stack_of_ranges_keeps_the_covering_records() {
    let n = scale(20, 400) as i32;
    let mut t = Tree::new();
    for i in 1..=n {
        t.put_pair_gravity(at(1, i, false), at(2, n - i, false));
    }
    t.check_tree();

    for _ in 0..scale(2, 10) {
        t.splice((0, 0), (0, 100), (0, 0));
        t.check_tree();
    }
    t.check();
}

/// The same, on a tree several levels deep whose ranges start and end on many
/// different rows, spliced at a row in the middle of them.
#[test]
fn splicing_inside_a_deep_tree_of_ranges_keeps_the_covering_records() {
    let size = scale(45, 900);
    let mut t = Tree::new();
    let mut k = 1i32;
    'fill: loop {
        for row1 in 1..=9 {
            for row2 in row1..=10 {
                if k > size as i32 {
                    break 'fill;
                }
                t.put_pair_gravity(at(row1, k, false), at(row2, size as i32 - k, false));
                k += 1;
            }
        }
    }
    assert_eq!(t.tree.n_keys, 2 * size);
    assert!(unsafe { (*t.tree.root).level } >= scale(1, 2) as i16);
    t.check_tree();

    for _ in 0..scale(1, 4) {
        for row in 3..=8 {
            t.splice((row, 0), (0, 200), (0, 0));
            t.check_tree();
        }
    }
    t.check();
}

/// The meta counts let a walk skip a subtree that holds none of the kind it
/// wants. This checks both halves of that: the filtered walk visits exactly the
/// marked keys, and starting it anywhere before the next marked key still lands
/// on that key rather than on the first key of some skipped subtree.
#[test]
fn a_filtered_walk_visits_only_the_marked_keys() {
    let rows = scale(12, 120) as i32;
    let cols = scale(8, 60) as i32;
    let mut t = Tree::new();
    // Sparse enough that most subtrees hold nothing the filter wants.
    let marked = |row: i32, col: i32| col == 0 && row % (rows / 4) == 3;

    let mut want: Vec<(u32, i32, i32)> = Vec::new();
    for row in 0..rows {
        for col in 0..cols {
            let id = t.put_meta(row, col, row % 2 == 0, marked(row, col));
            if marked(row, col) {
                want.push((id, row, col));
            }
        }
    }
    t.check();
    assert!(want.len() >= 3, "the fixture has to mark something");

    let filter = inline_filter();
    assert_eq!(t.filtered(&filter), want);

    // Subtree skipping: from any row at or before it, the first marked key is
    // the same one.
    let (_, first_row, first_col) = want[0];
    for row in 0..=first_row {
        let mut itr = zeroed_iter();
        assert!(
            unsafe {
                marktree_itr_get_filter(&mut t.tree, row, 0, rows, 0, filter.as_ptr(), &mut itr)
            },
            "no filtered mark at or after row {row}"
        );
        let k = unsafe { marktree_itr_current(&mut itr) };
        assert_eq!((k.pos.row, k.pos.col), (first_row, first_col), "from {row}");
    }

    // A filter for a kind nothing carries walks nothing at all.
    let mut other: MetaFilter = [0; META_COUNT];
    other[1] = u32::MAX;
    assert_eq!(t.filtered(&other), Vec::new());

    // The counts survive an edit: deleting every marked key empties the walk.
    for &(id, _, _) in &want {
        t.del(id);
    }
    t.check();
    assert_eq!(t.filtered(&filter), Vec::new());
}

/// Deleting through the *ends* of ranges, in a strided order rather than a
/// random one, so that a whole run of neighbouring keys leaves at once and the
/// rebalancer has to merge repeatedly.
#[test]
fn deleting_ranges_in_a_strided_order_keeps_the_covering_records() {
    let n = scale(24, 320);
    let mut t = Tree::new();
    let ids: Vec<u32> = (1..=n as i32)
        .map(|i| t.put_pair_gravity(at(1, i, false), at(2, n as i32 - i, false)))
        .collect();
    t.check_tree();

    let stride = scale(4, 8);
    let mut steps = 0;
    for start in 0..stride {
        let mut at = start;
        while at < n {
            t.del_pair(ids[at]);
            at += stride;
            steps += 1;
            if steps % scale(3, 17) == 0 {
                t.check_tree();
            }
        }
    }
    assert_eq!(t.tree.n_keys, 0);
    t.check_tree();
}

/// Upstream #37867: a splice that deletes a span containing both halves of
/// several ranges, some of which run *backwards* -- their end sorts before
/// their start. The covering records of an inverted range are empty, and the
/// splice has to leave them that way rather than deriving them from a span it
/// reads as negative.
#[test]
fn splicing_over_ranges_whose_ends_precede_their_starts() {
    let mut t = Tree::new();
    t.put(190, 48, false);
    t.put(48, 48, true);
    t.put(190, 48, false);
    for &(row, right, end_row) in &[
        (166, false, 166),
        (48, true, 48),
        (48, true, 48),
        (48, true, 255),
        (131, false, 48),
        (131, false, 48),
        (48, true, 131),
        (48, false, 216),
        (172, false, 51),
        (131, false, 131),
        (156, false, 131),
        (135, false, 166),
        (172, false, 250),
        (48, false, 143),
    ] {
        t.put_pair_gravity(at(row, 48, right), at(end_row, 48, false));
    }
    assert!(unsafe { marktree_check_intersections(&mut t.tree) });
    t.splice((48, 0), (139, 0), (0, 0));
    assert!(unsafe { marktree_check_intersections(&mut t.tree) });
}
