pub mod check;
pub mod inspect;
pub mod intersect;
pub mod iter;
pub mod key;
pub mod meta;
pub mod node;
pub mod rebalance;
pub mod splice;

use crate::src::nvim::api::private::helpers::ga_take_string;
use crate::src::nvim::garray::{ga_concat, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::map::{
    map_del_uint64_t_ptr_t, map_put_ref_ptr_t_ptr_t, map_put_ref_uint64_t_MTDamagePair,
    mh_get_ptr_t,
};
use crate::src::nvim::marktree::intersect::*;
use crate::src::nvim::marktree::key::*;
use crate::src::nvim::marktree::meta::*;
use crate::src::nvim::marktree::node::*;
pub use crate::src::nvim::marktree::{check::*, inspect::*, iter::*, rebalance::*, splice::*};
use crate::src::nvim::memory::{xfree, xmemdup};
use crate::src::nvim::os::libc::{abort, memcmp, memcpy, memmove, snprintf};
pub use crate::src::nvim::types::{
    DecorExt, DecorHighlightInline, DecorInlineData, DecorPriority, DecorVirtText,
    DecorVirtText_data as C2Rust_Unnamed, Intersection, MTDamage, MTDamagePair, MTKey, MTNode,
    MTPair, MTPos, Map_ptr_t_ptr_t, Map_uint64_t_MTDamagePair, Map_uint64_t_ptr_t, MapHash,
    MarkTree, MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_2, MetaFilter, Set_ptr_t,
    Set_uint64_t, String_0, VirtLines, VirtText, VirtTextChunk, VirtTextPos, colnr_T, garray_T,
    int16_t, int32_t, mtnode_inner_s, mtnode_s, ptr_t, schar_T, size_t, ssize_t, uint8_t, uint16_t,
    uint32_t, uint64_t, virt_line,
};
pub type MTDamageMap = Map_uint64_t_MTDamagePair;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2Rust_Unnamed_3 {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut MTKey,
}
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const KV_INITIAL_VALUE: C2Rust_Unnamed_3 = C2Rust_Unnamed_3 {
    size: 0 as size_t,
    capacity: 0 as size_t,
    items: ::core::ptr::null_mut::<MTKey>(),
};
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;
#[inline]
unsafe extern "C" fn map_put_ptr_t_ptr_t(
    mut map: *mut Map_ptr_t_ptr_t,
    mut key: ptr_t,
    mut value: ptr_t,
) {
    let mut val: *mut ptr_t = map_put_ref_ptr_t_ptr_t(
        map,
        key,
        ::core::ptr::null_mut::<*mut ptr_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    *val = value;
}
#[inline]
unsafe extern "C" fn map_get_ptr_t_ptr_t(mut map: *mut Map_ptr_t_ptr_t, mut key: ptr_t) -> ptr_t {
    let mut k: uint32_t = mh_get_ptr_t(&raw mut (*map).set, key);
    return if k == MH_TOMBSTONE as uint32_t {
        value_init_ptr_t.get()
    } else {
        *(*map).values.offset(k as isize)
    };
}
/// The set of paired-mark ids whose ranges cover the whole of node `x`.
#[inline]
unsafe fn ix(x: *mut MTNode) -> IdSet {
    IdSet::new(&raw mut (*x).intersect)
}

/// Record that the range `id` covers the whole of `x`.
fn intersect_node(x: *mut MTNode, id: uint64_t) {
    assert!(id & MARKTREE_END_FLAG == 0, "!(id & MARKTREE_END_FLAG)");
    unsafe { ix(x) }.insert_sorted(id);
}

/// Drop that record. `strict` asserts the id was there to drop.
fn unintersect_node(x: *mut MTNode, id: uint64_t, strict: bool) {
    assert!(id & MARKTREE_END_FLAG == 0, "!(id & MARKTREE_END_FLAG)");
    unsafe { ix(x) }.remove(id, strict);
}

pub unsafe extern "C" fn marktree_put(
    mut b: *mut MarkTree,
    mut key: MTKey,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut end_right: bool,
) {
    assert!(
        key.flags as ::core::ffi::c_int
            & !((1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                << 7 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                    << 8 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                    << 9 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                    << 10 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                    << 11 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                    << 12 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                    << 4 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                    << 5 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                    << 6 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                    << 13 as ::core::ffi::c_int
                | (1 as ::core::ffi::c_int as uint16_t as ::core::ffi::c_int)
                    << 14 as ::core::ffi::c_int)
            == 0,
        "!(key.flags & ~(MT_FLAG_EXTERNAL_MASK | MT_FLAG_RIGHT_GRAVITY))"
    );
    if end_row >= 0 as ::core::ffi::c_int {
        key.flags = (key.flags as ::core::ffi::c_int | MT_FLAG_PAIRED) as uint16_t;
    }
    marktree_put_key(b, key);
    if end_row >= 0 as ::core::ffi::c_int {
        let mut end_key: MTKey = key;
        end_key.flags = ((key.flags as ::core::ffi::c_int & !MT_FLAG_RIGHT_GRAVITY) as uint16_t
            as ::core::ffi::c_int
            | MT_FLAG_END as uint16_t as ::core::ffi::c_int
            | (if end_right as ::core::ffi::c_int != 0 {
                MT_FLAG_RIGHT_GRAVITY
            } else {
                0 as ::core::ffi::c_int
            }) as uint16_t as ::core::ffi::c_int) as uint16_t;
        end_key.pos = MTPos {
            row: end_row as int32_t,
            col: end_col as int32_t,
        };
        marktree_put_key(b, end_key);
        let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos {
                row: 0 as int32_t,
                col: 0,
            },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_2 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }];
        let mut end_itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos {
                row: 0 as int32_t,
                col: 0,
            },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_2 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }];
        marktree_lookup(b, mt_lookup_key(key), &raw mut itr as *mut MarkTreeIter);
        marktree_lookup(
            b,
            mt_lookup_key(end_key),
            &raw mut end_itr as *mut MarkTreeIter,
        );
        marktree_intersect_pair(
            b,
            mt_lookup_key(key),
            &raw mut itr as *mut MarkTreeIter,
            &raw mut end_itr as *mut MarkTreeIter,
            false,
        );
    }
}
pub unsafe extern "C" fn marktree_intersect_pair(
    mut b: *mut MarkTree,
    mut id: uint64_t,
    mut itr: *mut MarkTreeIter,
    mut end_itr: *mut MarkTreeIter,
    mut delete: bool,
) {
    let mut lvl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut maxlvl: ::core::ffi::c_int = if (*itr).lvl < (*end_itr).lvl {
        (*itr).lvl
    } else {
        (*end_itr).lvl
    };
    while lvl < maxlvl {
        if (*itr).s[lvl as usize].i > (*end_itr).s[lvl as usize].i {
            return;
        } else {
            if (*itr).s[lvl as usize].i < (*end_itr).s[lvl as usize].i {
                break;
            }
            lvl += 1;
        }
    }
    if lvl == maxlvl
        && (if lvl == (*itr).lvl {
            (*itr).i + 1 as ::core::ffi::c_int
        } else {
            (*itr).s[lvl as usize].i
        }) > (if lvl == (*end_itr).lvl {
            (*end_itr).i + 0 as ::core::ffi::c_int
        } else {
            (*end_itr).s[lvl as usize].i
        })
    {
        return;
    }
    while !(*itr).x.is_null() {
        let mut skip: bool = false;
        if (*itr).x == (*end_itr).x {
            if (*(*itr).x).level as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                || (*itr).i >= (*end_itr).i
            {
                break;
            }
            skip = true;
        } else if (*itr).lvl > lvl {
            skip = true;
        } else if (if lvl == (*itr).lvl {
            (*itr).i + 1 as ::core::ffi::c_int
        } else {
            (*itr).s[lvl as usize].i
        }) < (if lvl == (*end_itr).lvl {
            (*end_itr).i + 1 as ::core::ffi::c_int
        } else {
            (*end_itr).s[lvl as usize].i
        }) {
            skip = true;
        } else {
            lvl += 1;
        }
        if skip {
            if (*(*itr).x).level != 0 {
                let mut x: *mut MTNode =
                    (*inner((*itr).x)).i_ptr[((*itr).i + 1 as ::core::ffi::c_int) as usize];
                if delete {
                    unintersect_node(x, id, true);
                } else {
                    intersect_node(x, id);
                }
            }
        }
        marktree_itr_next_skip(
            b,
            itr,
            skip,
            true,
            ::core::ptr::null_mut::<MTPos>(),
            ::core::ptr::null::<uint32_t>(),
        );
    }
}
pub unsafe extern "C" fn marktree_put_key(mut b: *mut MarkTree, mut k: MTKey) {
    k.flags = (k.flags as ::core::ffi::c_int | MT_FLAG_REAL) as uint16_t;
    if (*b).root.is_null() {
        (*b).root = marktree_alloc_node(b, true);
    }
    let mut r: *mut MTNode = (*b).root;
    if (*r).n == 2 as int32_t * MT_BRANCH_FACTOR as ::core::ffi::c_int as int32_t - 1 as int32_t {
        let mut s: *mut MTNode = marktree_alloc_node(b, true);
        (*b).root = s;
        (*s).level = ((*r).level as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as int16_t;
        (*s).n = 0 as ::core::ffi::c_int as int32_t;
        (*inner(s)).i_ptr[0 as ::core::ffi::c_int as usize] = r;
        (*inner(s)).i_meta[0] = (*b).meta_root;
        (*r).parent = s;
        (*r).p_idx = 0 as int16_t;
        split_node(b, s, 0 as ::core::ffi::c_int, k);
        r = s;
    }
    let mut meta_inc = meta_describe_key(k);
    marktree_putp_aux(b, r, k, &meta_inc);
    meta_add(&mut (*b).meta_root, &meta_inc);
    (*b).n_keys = (*b).n_keys.wrapping_add(1);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_del_itr(
    mut b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
    mut rev: bool,
) -> uint64_t {
    let mut adjustment: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut cur: *mut MTNode = (*itr).x;
    let mut curi: ::core::ffi::c_int = (*itr).i;
    let mut id: uint64_t = mt_lookup_key((*cur).key[curi as usize]);
    let mut raw: MTKey = (*(*itr).x).key[(*itr).i as usize];
    let mut other: uint64_t = 0 as uint64_t;
    if mt_paired(raw) as ::core::ffi::c_int != 0
        && raw.flags as ::core::ffi::c_int & MT_FLAG_ORPHANED == 0
    {
        other = mt_lookup_key_side(raw, !mt_end(raw));
        let mut other_itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_2 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1];
        marktree_lookup(b, other, &raw mut other_itr as *mut MarkTreeIter);
        (*(*(&raw mut other_itr as *mut MarkTreeIter)).x).key
            [(*(&raw mut other_itr as *mut MarkTreeIter)).i as usize]
            .flags = ((*(*(&raw mut other_itr as *mut MarkTreeIter)).x).key
            [(*(&raw mut other_itr as *mut MarkTreeIter)).i as usize]
            .flags as ::core::ffi::c_int
            | MT_FLAG_ORPHANED) as uint16_t;
        if mt_start(raw) {
            let mut this_itr: [MarkTreeIter; 1] = [*itr];
            marktree_intersect_pair(
                b,
                id,
                &raw mut this_itr as *mut MarkTreeIter,
                &raw mut other_itr as *mut MarkTreeIter,
                true,
            );
        } else {
            marktree_intersect_pair(b, other, &raw mut other_itr as *mut MarkTreeIter, itr, true);
        }
    }
    if (*(*itr).x).level != 0 {
        if rev {
            abort();
        } else {
            marktree_itr_prev(b, itr);
            adjustment = -1 as ::core::ffi::c_int;
        }
    }
    let mut x: *mut MTNode = (*itr).x;
    assert!(
        (*x).level as ::core::ffi::c_int == 0 as ::core::ffi::c_int,
        "x->level == 0"
    );
    let mut intkey: MTKey = (*x).key[(*itr).i as usize];
    let mut meta_inc = meta_describe_key(intkey);
    if (*x).n > (*itr).i as int32_t + 1 as int32_t {
        memmove(
            (&raw mut (*x).key as *mut MTKey).offset((*itr).i as isize) as *mut ::core::ffi::c_void,
            (&raw mut (*x).key as *mut MTKey).offset(((*itr).i + 1 as ::core::ffi::c_int) as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<MTKey>()
                .wrapping_mul(((*x).n - (*itr).i as int32_t - 1 as int32_t) as size_t),
        );
    }
    (*x).n -= 1;
    (*b).n_keys = (*b).n_keys.wrapping_sub(1);
    map_del_uint64_t_ptr_t(
        &raw mut (*b).id2node as *mut Map_uint64_t_ptr_t,
        id,
        ::core::ptr::null_mut::<uint64_t>(),
    );
    if adjustment == -1 as ::core::ffi::c_int {
        let mut ilvl: ::core::ffi::c_int = (*itr).lvl - 1 as ::core::ffi::c_int;
        let mut lnode: *mut MTNode = x;
        let mut start_id: uint64_t = 0 as uint64_t;
        let mut did_bubble: bool = false;
        if mt_end(intkey) {
            start_id = mt_lookup_key_side(intkey, false);
        }
        loop {
            let mut p: *mut MTNode = (*lnode).parent;
            if ilvl < 0 as ::core::ffi::c_int {
                abort();
            }
            let mut i: ::core::ffi::c_int = (*itr).s[ilvl as usize].i;
            assert!((*inner(p)).i_ptr[i as usize] == lnode, "p->ptr[i] == lnode");
            if i > 0 as ::core::ffi::c_int {
                unrelative(
                    (*p).key[(i - 1 as ::core::ffi::c_int) as usize].pos,
                    &mut intkey.pos,
                );
            }
            if p != cur && start_id != 0 {
                if ix((*inner(p)).i_ptr[0]).contains(start_id) {
                    let mut last: ::core::ffi::c_int = if lnode != x {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while (k as int32_t) < (*p).n + last as int32_t {
                        unintersect_node((*inner(p)).i_ptr[k as usize], start_id, true);
                        k += 1;
                    }
                    intersect_node(p, start_id);
                    did_bubble = true;
                }
            }
            meta_sub(&mut (*inner(p)).i_meta[(*lnode).p_idx as usize], &meta_inc);
            lnode = p;
            ilvl -= 1;
            if lnode == cur {
                break;
            }
        }
        let mut deleted: MTKey = (*cur).key[curi as usize];
        meta_inc = meta_describe_key(deleted);
        (*cur).key[curi as usize] = intkey;
        refkey(b, cur, curi);
        if mt_end((*cur).key[curi as usize]) as ::core::ffi::c_int != 0 && !did_bubble {
            let mut pi: uint64_t = pseudo_index(x, 0 as ::core::ffi::c_int);
            let mut pi_start: uint64_t = pseudo_index_for_id(b, start_id, true);
            if pi_start > 0 as uint64_t && pi_start < pi {
                intersect_node(x, start_id);
            }
        }
        relative(intkey.pos, &mut deleted.pos);
        let mut y: *mut MTNode = (*inner(cur)).i_ptr[(curi + 1 as ::core::ffi::c_int) as usize];
        if deleted.pos.row != 0 || deleted.pos.col != 0 {
            while !y.is_null() {
                let mut k_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while (k_0 as int32_t) < (*y).n {
                    unrelative(
                        deleted.pos,
                        &mut (*(&mut (*y).key as *mut MTKey).offset(k_0 as isize)).pos,
                    );
                    k_0 += 1;
                }
                y = if (*y).level as ::core::ffi::c_int != 0 {
                    (*inner(y)).i_ptr[0 as ::core::ffi::c_int as usize]
                } else {
                    ::core::ptr::null_mut::<MTNode>()
                };
            }
        }
        (*itr).i -= 1;
    }
    let mut lnode_0: *mut MTNode = cur;
    while !(*lnode_0).parent.is_null() {
        meta_sub(
            &mut (*inner((*lnode_0).parent)).i_meta[(*lnode_0).p_idx as usize],
            &meta_inc,
        );
        lnode_0 = (*lnode_0).parent;
    }
    for m in 0..META_COUNT {
        assert!(
            (*b).meta_root[m] >= meta_inc[m],
            "b->meta_root[m] >= meta_inc[m]"
        );
    }
    meta_sub(&mut (*b).meta_root, &meta_inc);
    let mut itr_dirty: bool = false;
    let mut rlvl: ::core::ffi::c_int = (*itr).lvl - 1 as ::core::ffi::c_int;
    let mut lasti: *mut ::core::ffi::c_int = &raw mut (*itr).i;
    let mut ppos: MTPos = (*itr).pos;
    while x != (*b).root {
        assert!(rlvl >= 0 as ::core::ffi::c_int, "rlvl >= 0");
        let mut p_0: *mut MTNode = (*x).parent;
        if (*x).n >= MT_BRANCH_FACTOR as ::core::ffi::c_int as int32_t - 1 as int32_t {
            break;
        }
        let mut pi_0: ::core::ffi::c_int = (*itr).s[rlvl as usize].i;
        assert!((*inner(p_0)).i_ptr[pi_0 as usize] == x, "p->ptr[pi] == x");
        if pi_0 > 0 as ::core::ffi::c_int {
            ppos.row -= (*p_0).key[(pi_0 - 1 as ::core::ffi::c_int) as usize]
                .pos
                .row;
            ppos.col = (*itr).s[rlvl as usize].oldcol as int32_t;
        }
        if pi_0 > 0 as ::core::ffi::c_int
            && (*(*inner(p_0)).i_ptr[(pi_0 - 1 as ::core::ffi::c_int) as usize]).n
                > MT_BRANCH_FACTOR as ::core::ffi::c_int as int32_t - 1 as int32_t
        {
            *lasti += 1 as ::core::ffi::c_int;
            itr_dirty = true;
            pivot_right(b, ppos, p_0, pi_0 - 1 as ::core::ffi::c_int);
            break;
        } else if (pi_0 as int32_t) < (*p_0).n
            && (*(*inner(p_0)).i_ptr[(pi_0 + 1 as ::core::ffi::c_int) as usize]).n
                > MT_BRANCH_FACTOR as ::core::ffi::c_int as int32_t - 1 as int32_t
        {
            pivot_left(b, ppos, p_0, pi_0);
            break;
        } else {
            if pi_0 > 0 as ::core::ffi::c_int {
                assert!(
                    (*(*inner(p_0)).i_ptr[(pi_0 - 1 as ::core::ffi::c_int) as usize]).n
                        == MT_BRANCH_FACTOR as ::core::ffi::c_int as int32_t - 1 as int32_t,
                    "p->ptr[pi - 1]->n == T - 1"
                );
                *lasti += MT_BRANCH_FACTOR as ::core::ffi::c_int;
                x = merge_node(b, p_0, pi_0 - 1 as ::core::ffi::c_int);
                if lasti == &raw mut (*itr).i {
                    (*itr).x = x;
                }
                (*itr).s[rlvl as usize].i -= 1;
                itr_dirty = true;
            } else {
                assert!(
                    (pi_0 as int32_t) < (*p_0).n
                        && (*(*inner(p_0)).i_ptr[(pi_0 + 1 as ::core::ffi::c_int) as usize]).n
                            == MT_BRANCH_FACTOR as ::core::ffi::c_int as int32_t - 1 as int32_t,
                    "pi < p->n && p->ptr[pi + 1]->n == T - 1"
                );
                merge_node(b, p_0, pi_0);
            }
            lasti =
                &raw mut (*(&raw mut (*itr).s as *mut C2Rust_Unnamed_2).offset(rlvl as isize)).i;
            rlvl -= 1;
            x = p_0;
        }
    }
    if (*(*b).root).n == 0 as int32_t {
        if (*itr).lvl > 0 as ::core::ffi::c_int {
            memmove(
                &raw mut (*itr).s as *mut C2Rust_Unnamed_2 as *mut ::core::ffi::c_void,
                (&raw mut (*itr).s as *mut C2Rust_Unnamed_2)
                    .offset(1 as ::core::ffi::c_int as isize)
                    as *const ::core::ffi::c_void,
                (((*itr).lvl - 1 as ::core::ffi::c_int) as size_t)
                    .wrapping_mul(::core::mem::size_of::<C2Rust_Unnamed_2>()),
            );
            (*itr).lvl -= 1;
        }
        if (*(*b).root).level != 0 {
            let mut oldroot: *mut MTNode = (*b).root;
            (*b).root = (*inner((*b).root)).i_ptr[0 as ::core::ffi::c_int as usize];
            assert!(
                (*b).meta_root == (*inner(oldroot)).i_meta[0],
                "b->meta_root[m] == oldroot->meta[0][m]"
            );
            (*(*b).root).parent = ::core::ptr::null_mut::<MTNode>();
            marktree_free_node(b, oldroot);
        } else {
            (*itr).x = ::core::ptr::null_mut::<MTNode>();
        }
    }
    if !(*itr).x.is_null() && itr_dirty as ::core::ffi::c_int != 0 {
        marktree_itr_fix_pos(b, itr);
    }
    if adjustment == -1 as ::core::ffi::c_int {
        marktree_itr_next(b, itr);
        marktree_itr_next(b, itr);
    } else if !(*itr).x.is_null() && (*itr).i as int32_t >= (*(*itr).x).n {
        assert!(
            (*(*itr).x).level as ::core::ffi::c_int == 0 as ::core::ffi::c_int,
            "itr->x->level == 0"
        );
        marktree_itr_next(b, itr);
    }
    return other;
}
pub unsafe extern "C" fn marktree_revise_meta(
    mut b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
    mut old_key: MTKey,
) {
    let mut meta_old: [uint32_t; 5] = [0; 5];
    let mut meta_new: [uint32_t; 5] = [0; 5];
    meta_old = meta_describe_key(old_key);
    meta_new = meta_describe_key((*(*itr).x).key[(*itr).i as usize]);
    if memcmp(
        &raw mut meta_old as *mut uint32_t as *const ::core::ffi::c_void,
        &raw mut meta_new as *mut uint32_t as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[uint32_t; 5]>(),
    ) == 0
    {
        return;
    }
    let mut lnode: *mut MTNode = (*itr).x;
    while !(*lnode).parent.is_null() {
        meta_apply_delta(
            &mut (*inner((*lnode).parent)).i_meta[(*lnode).p_idx as usize],
            &meta_new,
            &meta_old,
        );
        lnode = (*lnode).parent;
    }
    meta_apply_delta(&mut (*b).meta_root, &meta_new, &meta_old);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn intersect_mov_test(
    mut x: *const uint64_t,
    mut nx: size_t,
    mut y: *const uint64_t,
    mut ny: size_t,
    mut win: *const uint64_t,
    mut nwin: size_t,
    mut wout: *mut uint64_t,
    mut nwout: *mut size_t,
    mut dout: *mut uint64_t,
    mut ndout: *mut size_t,
) -> bool {
    // x is immutable as far as intersect_mov is concerned, and y may shrink —
    // whatever it loses shows up in d. Neither is ever grown, so borrowing the
    // caller's arrays as sets is enough.
    let mut xs = Intersection {
        size: nx,
        capacity: 0,
        items: x as *mut uint64_t,
        init_array: [0; 4],
    };
    let mut ys = Intersection {
        size: ny,
        capacity: 0,
        items: y as *mut uint64_t,
        init_array: [0; 4],
    };
    let mut ws = Intersection {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut(),
        init_array: [0; 4],
    };
    let mut ds = Intersection {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut(),
        init_array: [0; 4],
    };
    let (xs, ys) = (IdSet::new(&raw mut xs), IdSet::new(&raw mut ys));
    let (ws, ds) = (IdSet::new(&raw mut ws), IdSet::new(&raw mut ds));
    ws.init();
    ds.init();
    ws.extend_from_slice(::core::slice::from_raw_parts(win, nwin));

    intersect_mov(&xs, &ys, &ws, &ds);

    let fits = ws.len() <= *nwout && ds.len() <= *ndout;
    if fits {
        ::core::ptr::copy_nonoverlapping(ws.as_slice().as_ptr(), wout, ws.len());
        *nwout = ws.len();
        ::core::ptr::copy_nonoverlapping(ds.as_slice().as_ptr(), dout, ds.len());
        *ndout = ds.len();
    }
    xfree(ws.take_heap());
    xfree(ds.take_heap());
    return fits;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_clear(mut b: *mut MarkTree) {
    if !(*b).root.is_null() {
        marktree_free_subtree(b, (*b).root);
        (*b).root = ::core::ptr::null_mut::<MTNode>();
    }
    xfree(
        (*(&raw mut (*b).id2node as *mut Map_uint64_t_ptr_t))
            .set
            .keys as *mut ::core::ffi::c_void,
    );
    xfree(
        (*(&raw mut (*b).id2node as *mut Map_uint64_t_ptr_t))
            .set
            .h
            .hash as *mut ::core::ffi::c_void,
    );
    (*(&raw mut (*b).id2node as *mut Map_uint64_t_ptr_t)).set = Set_uint64_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<uint64_t>(),
    };
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw mut (*(&raw mut (*b).id2node
        as *mut Map_uint64_t_ptr_t))
        .values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    (*b).n_keys = 0 as size_t;
    (*b).meta_root = [0; META_COUNT];
    assert!((*b).n_nodes == 0 as size_t, "b->n_nodes == 0");
}
pub unsafe extern "C" fn marktree_free_subtree(mut b: *mut MarkTree, mut x: *mut MTNode) {
    if (*x).level != 0 {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i as int32_t) < (*x).n + 1 as int32_t {
            marktree_free_subtree(b, (*inner(x)).i_ptr[i as usize]);
            i += 1;
        }
    }
    marktree_free_node(b, x);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_move(
    mut b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
) {
    let mut key: MTKey = (*(*itr).x).key[(*itr).i as usize];
    let mut x: *mut MTNode = (*itr).x;
    if (*x).level == 0 {
        let mut internal: bool = false;
        let mut newpos: MTPos = MTPos {
            row: row as int32_t,
            col: col as int32_t,
        };
        if !(*x).parent.is_null() {
            if pos_less((*itr).pos, newpos) {
                relative((*itr).pos, &mut newpos);
                if pos_less(newpos, (*x).key[((*x).n - 1 as int32_t) as usize].pos) {
                    internal = true;
                }
            }
        } else {
            internal = true;
        }
        if internal {
            if key.pos.row == newpos.row && key.pos.col == newpos.col {
                return;
            }
            key.pos = newpos;
            let (mut new_i, match_0) = find_key(node_keys(x), key);
            if !match_0 {
                new_i += 1;
            }
            if new_i == (*itr).i {
                (*x).key[(*itr).i as usize].pos = newpos;
            } else if new_i < (*itr).i {
                memmove(
                    (&raw mut (*x).key as *mut MTKey)
                        .offset((new_i + 1 as ::core::ffi::c_int) as isize)
                        as *mut ::core::ffi::c_void,
                    (&raw mut (*x).key as *mut MTKey).offset(new_i as isize)
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<MTKey>().wrapping_mul(((*itr).i - new_i) as size_t),
                );
                (*x).key[new_i as usize] = key;
            } else if new_i > (*itr).i {
                memmove(
                    (&raw mut (*x).key as *mut MTKey).offset((*itr).i as isize)
                        as *mut ::core::ffi::c_void,
                    (&raw mut (*x).key as *mut MTKey)
                        .offset(((*itr).i + 1 as ::core::ffi::c_int) as isize)
                        as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<MTKey>()
                        .wrapping_mul((new_i - (*itr).i - 1 as ::core::ffi::c_int) as size_t),
                );
                (*x).key[(new_i - 1 as ::core::ffi::c_int) as usize] = key;
            }
            return;
        }
    }
    let mut other: uint64_t = marktree_del_itr(b, itr, false);
    key.pos = MTPos {
        row: row as int32_t,
        col: col as int32_t,
    };
    marktree_put_key(b, key);
    if other != 0 {
        marktree_restore_pair(b, key);
    }
    (*itr).x = ::core::ptr::null_mut::<MTNode>();
}
pub unsafe extern "C" fn marktree_restore_pair(mut b: *mut MarkTree, mut key: MTKey) {
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
    marktree_lookup(
        b,
        mt_lookup_key_side(key, false),
        &raw mut itr as *mut MarkTreeIter,
    );
    marktree_lookup(
        b,
        mt_lookup_key_side(key, true),
        &raw mut end_itr as *mut MarkTreeIter,
    );
    if (*(&raw mut itr as *mut MarkTreeIter)).x.is_null()
        || (*(&raw mut end_itr as *mut MarkTreeIter)).x.is_null()
    {
        return;
    }
    (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
        .flags = ((*(*(&raw mut itr as *mut MarkTreeIter)).x).key
        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
        .flags as ::core::ffi::c_int
        & !MT_FLAG_ORPHANED as uint16_t as ::core::ffi::c_int) as uint16_t;
    (*(*(&raw mut end_itr as *mut MarkTreeIter)).x).key
        [(*(&raw mut end_itr as *mut MarkTreeIter)).i as usize]
        .flags = ((*(*(&raw mut end_itr as *mut MarkTreeIter)).x).key
        [(*(&raw mut end_itr as *mut MarkTreeIter)).i as usize]
        .flags as ::core::ffi::c_int
        & !MT_FLAG_ORPHANED as uint16_t as ::core::ffi::c_int) as uint16_t;
    marktree_intersect_pair(
        b,
        mt_lookup_key_side(key, false),
        &raw mut itr as *mut MarkTreeIter,
        &raw mut end_itr as *mut MarkTreeIter,
        false,
    );
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_lookup_ns(
    mut b: *mut MarkTree,
    mut ns: uint32_t,
    mut id: uint32_t,
    mut end: bool,
    mut itr: *mut MarkTreeIter,
) -> MTKey {
    return marktree_lookup(b, mt_lookup_id(ns, id, end), itr);
}
unsafe extern "C" fn pseudo_index_for_id(
    mut b: *mut MarkTree,
    mut id: uint64_t,
    mut sloppy: bool,
) -> uint64_t {
    let mut n: *mut MTNode = id2node(b, id);
    if n.is_null() {
        return 0 as uint64_t;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*n).level as ::core::ffi::c_int != 0 || !sloppy {
        i = 0 as ::core::ffi::c_int;
        while (i as int32_t) < (*n).n {
            if mt_lookup_key((*n).key[i as usize]) == id {
                break;
            }
            i += 1;
        }
        assert!((i as int32_t) < (*n).n, "i < n->n");
        if (*n).level != 0 {
            i += 1 as ::core::ffi::c_int;
        }
    }
    return pseudo_index(n, i);
}
pub unsafe extern "C" fn marktree_lookup(
    mut b: *mut MarkTree,
    mut id: uint64_t,
    mut itr: *mut MarkTreeIter,
) -> MTKey {
    let mut n: *mut MTNode = id2node(b, id);
    if n.is_null() {
        if !itr.is_null() {
            (*itr).x = ::core::ptr::null_mut::<MTNode>();
        }
        return MT_INVALID_KEY;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    i = 0 as ::core::ffi::c_int;
    while (i as int32_t) < (*n).n {
        if mt_lookup_key((*n).key[i as usize]) == id {
            return marktree_itr_set_node(b, itr, n, i);
        }
        i += 1;
    }
    abort();
}
pub unsafe extern "C" fn marktree_get_altpos(
    mut b: *mut MarkTree,
    mut mark: MTKey,
    mut itr: *mut MarkTreeIter,
) -> MTPos {
    return marktree_get_alt(b, mark, itr).pos;
}
pub unsafe extern "C" fn marktree_get_alt(
    mut b: *mut MarkTree,
    mut mark: MTKey,
    mut itr: *mut MarkTreeIter,
) -> MTKey {
    return if mt_paired(mark) as ::core::ffi::c_int != 0 {
        marktree_lookup_ns(b, mark.ns, mark.id, !mt_end(mark), itr)
    } else {
        mark
    };
}
