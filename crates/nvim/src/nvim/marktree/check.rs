// Not graduated yet: the parent module denies `unsafe_op_in_unsafe_fn` and the
// level is inherited, so these transpiled bodies opt back out until the
// rewrite that narrows them. Remove this when the deny goes on.
#![allow(unsafe_op_in_unsafe_fn)]

//! Whole-tree invariant checks, and the entry points the unit spec drives.
//!
//! `test/unit/marktree_spec.lua` builds trees of thousands of random marks and
//! calls [`marktree_check`] after every batch, so these are not dead debug
//! code: they are the oracle. What they assert, per node:
//!
//! * the fill bounds, and that the root is the only node allowed to be short;
//! * that the keys are in order once their relative positions are resolved,
//!   including the gravity tie-break;
//! * that `id2node` names the node each key actually lives in;
//! * that every child's `parent` and `p_idx` point back correctly;
//! * that the meta counts equal what a fresh walk of the subtree computes.
//!
//! [`marktree_check_intersections`] is the expensive one: it empties every
//! node's set, rebuilds all of them from the pairs themselves, and compares.

use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_check(mut b: *mut MarkTree) {
    if (*b).root.is_null() {
        debug_assert!((*b).n_keys == 0 as size_t, "b->n_keys == 0");
        debug_assert!((*b).n_nodes == 0 as size_t, "b->n_nodes == 0");
        assert!(
            (&raw mut (*b).id2node as *mut Map_uint64_t_ptr_t).is_null()
                || (*(&raw mut (*b).id2node as *mut Map_uint64_t_ptr_t))
                    .set
                    .h
                    .size
                    == 0 as uint32_t,
            "b->id2node == NULL || map_size(b->id2node) == 0"
        );
        return;
    }
    let mut dummy: MTPos = MTPos { row: 0, col: 0 };
    let mut last_right: bool = false;
    let mut nkeys: size_t = marktree_check_node(
        b,
        (*b).root,
        &raw mut dummy,
        &raw mut last_right,
        &(*b).meta_root,
    );
    debug_assert!((*b).n_keys == nkeys, "b->n_keys == nkeys");
    debug_assert!(
        (*b).n_keys
            == (*(&raw mut (*b).id2node as *mut Map_uint64_t_ptr_t))
                .set
                .h
                .size as size_t,
        "b->n_keys == map_size(b->id2node)"
    );
}

pub unsafe extern "C" fn marktree_check_node(
    mut b: *mut MarkTree,
    mut x: *mut MTNode,
    mut last: *mut MTPos,
    mut last_right: *mut bool,
    meta_node_ref: &MetaCount,
) -> size_t {
    debug_assert!(
        (*x).n <= 2 as int32_t * MT_BRANCH_FACTOR as ::core::ffi::c_int as int32_t - 1 as int32_t,
        "x->n <= 2 * T - 1"
    );
    assert!(
        (*x).n
            >= (if x != (*b).root {
                MT_BRANCH_FACTOR as ::core::ffi::c_int as int32_t - 1 as int32_t
            } else {
                0 as int32_t
            }),
        "x->n >= (x != b->root ? T - 1 : 0)"
    );
    let mut n_keys: size_t = (*x).n as size_t;
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i as int32_t) < (*x).n {
        if (*x).level != 0 {
            n_keys = n_keys.wrapping_add(marktree_check_node(
                b,
                (*inner(x)).i_ptr[i as usize],
                last,
                last_right,
                &(*inner(x)).i_meta[i as usize],
            ));
        } else {
            *last = MTPos {
                row: 0 as int32_t,
                col: 0 as int32_t,
            };
        }
        if i > 0 as ::core::ffi::c_int {
            unrelative(
                (*x).key[(i - 1 as ::core::ffi::c_int) as usize].pos,
                &mut *last,
            );
        }
        debug_assert!(
            pos_leq(*last, (*x).key[i as usize].pos),
            "pos_leq(*last, x->key[i].pos)"
        );
        if (*last).row == (*x).key[i as usize].pos.row
            && (*last).col == (*x).key[i as usize].pos.col
        {
            debug_assert!(
                !*last_right || mt_right((*x).key[i as usize]) as ::core::ffi::c_int != 0,
                "!*last_right || mt_right(x->key[i])"
            );
        }
        *last_right = mt_right((*x).key[i as usize]);
        debug_assert!(
            (*x).key[i as usize].pos.col >= 0 as int32_t,
            "x->key[i].pos.col >= 0"
        );
        debug_assert!(
            id2node(b, mt_lookup_key((*x).key[i as usize])) == x,
            "pmap_get(uint64_t)(b->id2node, mt_lookup_key(x->key[i])) == x"
        );
        i += 1;
    }
    if (*x).level != 0 {
        n_keys = n_keys.wrapping_add(marktree_check_node(
            b,
            (*inner(x)).i_ptr[(*x).n as usize],
            last,
            last_right,
            &(*inner(x)).i_meta[(*x).n as usize],
        ));
        unrelative((*x).key[((*x).n - 1 as int32_t) as usize].pos, &mut *last);
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i_0 as int32_t) < (*x).n + 1 as int32_t {
            debug_assert!(
                (*(*inner(x)).i_ptr[i_0 as usize]).parent == x,
                "x->ptr[i]->parent == x"
            );
            debug_assert!(
                (*(*inner(x)).i_ptr[i_0 as usize]).p_idx as ::core::ffi::c_int == i_0,
                "x->ptr[i]->p_idx == i"
            );
            assert!(
                (*(*inner(x)).i_ptr[i_0 as usize]).level as ::core::ffi::c_int
                    == (*x).level as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                "x->ptr[i]->level == x->level - 1"
            );
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < i_0 {
                debug_assert!(
                    (*inner(x)).i_ptr[i_0 as usize] != (*inner(x)).i_ptr[j as usize],
                    "x->ptr[i] != x->ptr[j]"
                );
                j += 1;
            }
            i_0 += 1;
        }
    } else if (*x).n > 0 as int32_t {
        *last = (*x).key[((*x).n - 1 as int32_t) as usize].pos;
    }
    debug_assert!(
        *meta_node_ref == meta_describe_node(x),
        "meta_node_ref[m] == meta_node[m]"
    );
    return n_keys;
}

/// Rebuild every intersection set from the pairs themselves and check it
/// against what was there.
///
/// Three steps: move each node's set aside and empty it; walk every mark and,
/// for each start of a pair, intersect the nodes between the two halves as if
/// the pair had just been inserted; then compare each node's rebuilt set
/// against the one that was moved aside.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_check_intersections(mut b: *mut MarkTree) -> bool {
    if (*b).root.is_null() {
        return true;
    }
    let mut checked: Map_ptr_t_ptr_t = Map_ptr_t_ptr_t {
        set: Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<ptr_t>(),
        },
        values: ::core::ptr::null_mut::<ptr_t>(),
    };
    mt_recurse_nodes((*b).root, &raw mut checked);
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_2 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    marktree_itr_first(b, &raw mut itr as *mut MarkTreeIter);
    loop {
        let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if mark.pos.row < 0 as int32_t {
            break;
        }
        if mt_start(mark) {
            let mut start_itr: [MarkTreeIter; 1] = [MarkTreeIter {
                pos: MTPos { row: 0, col: 0 },
                lvl: 0,
                x: ::core::ptr::null_mut::<MTNode>(),
                i: 0,
                s: [C2Rust_Unnamed_2 { oldcol: 0, i: 0 }; 20],
                intersect_idx: 0,
                intersect_pos: MTPos { row: 0, col: 0 },
                intersect_pos_x: MTPos { row: 0, col: 0 },
            }; 1];
            let mut end_itr: [MarkTreeIter; 1] = [MarkTreeIter {
                pos: MTPos { row: 0, col: 0 },
                lvl: 0,
                x: ::core::ptr::null_mut::<MTNode>(),
                i: 0,
                s: [C2Rust_Unnamed_2 { oldcol: 0, i: 0 }; 20],
                intersect_idx: 0,
                intersect_pos: MTPos { row: 0, col: 0 },
                intersect_pos_x: MTPos { row: 0, col: 0 },
            }; 1];
            let mut end_id: uint64_t = mt_lookup_id(mark.ns, mark.id, true);
            let mut k: MTKey = marktree_lookup(&mut *b, end_id, Some(&mut end_itr[0]));
            if k.pos.row >= 0 as int32_t {
                *(&raw mut start_itr as *mut MarkTreeIter) = *(&raw mut itr as *mut MarkTreeIter);
                marktree_intersect_pair(
                    &mut *b,
                    mt_lookup_key(mark),
                    &mut start_itr[0],
                    &end_itr[0],
                    false,
                );
            }
        }
        marktree_itr_next(b, &raw mut itr as *mut MarkTreeIter);
    }
    let mut status: bool = mt_recurse_nodes_compare((*b).root, &raw mut checked);
    let mut val: *mut uint64_t = ::core::ptr::null_mut::<uint64_t>();
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < checked.set.h.n_keys {
        val = *checked.values.offset(__i as isize) as *mut uint64_t;
        xfree(val as *mut ::core::ffi::c_void);
        __i = __i.wrapping_add(1);
    }
    xfree(checked.set.keys as *mut ::core::ffi::c_void);
    xfree(checked.set.h.hash as *mut ::core::ffi::c_void);
    checked.set = Set_ptr_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<ptr_t>(),
    };
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut checked.values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return status;
}

pub unsafe fn mt_recurse_nodes(x: *mut MTNode, checked: *mut Map_ptr_t_ptr_t) {
    let set = ix(x);
    if !set.is_empty() {
        // Record what this node intersects and then empty it, so the walk that
        // rebuilds the sets from scratch can be compared against the record.
        // The recorded copy is terminated with a sentinel no id can equal.
        set.push(uint64_t::MAX);
        let bytes = set.len() * size_of::<uint64_t>();
        let copy = if set.is_inline() {
            xmemdup(set.as_slice().as_ptr().cast(), bytes)
        } else {
            NULL
        };
        let heap = set.take_heap();
        let owned = if heap.is_null() { copy } else { heap };
        map_put_ptr_t_ptr_t(checked, x as ptr_t, owned);
    }
    if (*x).level != 0 {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i as int32_t) < (*x).n + 1 as int32_t {
            mt_recurse_nodes((*inner(x)).i_ptr[i as usize], checked);
            i += 1;
        }
    }
}

/// Does `x`'s rebuilt intersection set match what `mt_recurse_nodes` recorded
/// for it? Recurses over the whole subtree.
pub unsafe fn mt_recurse_nodes_compare(x: *mut MTNode, checked: *mut Map_ptr_t_ptr_t) -> bool {
    let recorded: *mut uint64_t = map_get_ptr_t_ptr_t(checked, x as ptr_t) as *mut uint64_t;
    let rebuilt = ix(x);
    if recorded.is_null() {
        if !rebuilt.is_empty() {
            return false;
        }
    } else {
        // The record is sentinel-terminated; a node with an empty set was
        // never recorded at all.
        let mut i = 0;
        loop {
            let id = *recorded.add(i);
            if id == uint64_t::MAX {
                if i != rebuilt.len() {
                    return false;
                }
                break;
            }
            if rebuilt.len() <= i || id != rebuilt.as_slice()[i] {
                return false;
            }
            i += 1;
        }
    }
    if (*x).level != 0 {
        for i in 0..=(*x).n as usize {
            if !mt_recurse_nodes_compare((*inner(x)).i_ptr[i], checked) {
                return false;
            }
        }
    }
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_put_test(
    mut b: *mut MarkTree,
    mut ns: uint32_t,
    mut id: uint32_t,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut right_gravity: bool,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut end_right: bool,
    mut meta_inline: bool,
) {
    let mut flags: uint16_t = mt_flags(right_gravity, false, false, false);
    flags = (flags as ::core::ffi::c_int
        | if meta_inline as ::core::ffi::c_int != 0 {
            MT_FLAG_DECOR_VIRT_TEXT_INLINE
        } else {
            0 as ::core::ffi::c_int
        }) as uint16_t;
    let mut key: MTKey = MTKey {
        pos: MTPos {
            row: row as int32_t,
            col: col as int32_t,
        },
        ns: ns,
        id: id,
        flags: flags,
        decor_data: DecorInlineData {
            hl: DECOR_HIGHLIGHT_INLINE_INIT,
        },
    };
    marktree_put(&mut *b, key, end_row, end_col, end_right);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn mt_right_test(mut key: MTKey) -> bool {
    return mt_right(key);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_del_pair_test(
    mut b: *mut MarkTree,
    mut ns: uint32_t,
    mut id: uint32_t,
) {
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_2 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    marktree_lookup_ns(&mut *b, ns, id, false, Some(&mut itr[0]));
    let mut other: uint64_t = marktree_del_itr(&mut *b, &mut itr[0], false);
    debug_assert!(other != 0, "other");
    marktree_lookup(&mut *b, other, Some(&mut itr[0]));
    marktree_del_itr(&mut *b, &mut itr[0], false);
}
