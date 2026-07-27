//! Walking the tree.
//!
//! `MarkTreeIter` is a cursor: the node and index it currently names, plus the
//! path it took to get there (`s[lvl]`, one entry per level) and the absolute
//! position of that node, since positions in the tree are stored relative to
//! the enclosing key. Callers all over the editor hold one across an edit, so
//! the layout is fixed and the entry points keep their C signatures.
//!
//! Three walks share the machinery:
//!
//! * the plain one, [`marktree_itr_next`]/[`marktree_itr_prev`];
//! * the filtered one, which reads a node's meta counts and skips any subtree
//!   that holds none of the decoration kinds the caller asked for;
//! * the overlap walk, which answers "which ranges cover this position" by
//!   reading the intersection set on each node of the path down.

use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_get(
    mut b: *mut MarkTree,
    mut row: int32_t,
    mut col: ::core::ffi::c_int,
    mut itr: *mut MarkTreeIter,
) -> bool {
    return marktree_itr_get_ext(
        b,
        MTPos {
            row: row,
            col: col as int32_t,
        },
        itr,
        false,
        false,
        ::core::ptr::null_mut::<MTPos>(),
        ::core::ptr::null::<uint32_t>(),
    );
}

pub unsafe extern "C" fn marktree_itr_get_ext(
    mut b: *mut MarkTree,
    mut p: MTPos,
    mut itr: *mut MarkTreeIter,
    mut last: bool,
    mut gravity: bool,
    mut oldbase: *mut MTPos,
    mut meta_filter: MetaFilter,
) -> bool {
    if (*b).n_keys == 0 as size_t {
        (*itr).x = ::core::ptr::null_mut::<MTNode>();
        return false;
    }
    let mut k: MTKey = MTKey {
        pos: p,
        ns: 0,
        id: 0,
        flags: (if gravity as ::core::ffi::c_int != 0 {
            MT_FLAG_RIGHT_GRAVITY
        } else {
            0 as ::core::ffi::c_int
        }) as uint16_t,
        decor_data: DecorInlineData {
            hl: DecorHighlightInline {
                flags: 0,
                priority: 0,
                hl_id: 0,
                conceal_char: 0,
            },
        },
    };
    if last as ::core::ffi::c_int != 0 && !gravity {
        k.flags = MT_FLAG_LAST as uint16_t;
    }
    (*itr).pos = MTPos {
        row: 0 as int32_t,
        col: 0 as int32_t,
    };
    (*itr).x = (*b).root;
    (*itr).lvl = 0 as ::core::ffi::c_int;
    if !oldbase.is_null() {
        *oldbase.offset((*itr).lvl as isize) = (*itr).pos;
    }
    loop {
        (*itr).i = find_key(node_keys((*itr).x), k).0 + 1 as ::core::ffi::c_int;
        if (*(*itr).x).level as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            break;
        }
        if !meta_filter.is_null() {
            if !meta_has(
                &(*inner((*itr).x)).i_meta[((*itr).i) as usize],
                &*meta_filter.cast(),
            ) {
                break;
            }
        }
        (*itr).s[(*itr).lvl as usize].i = (*itr).i;
        (*itr).s[(*itr).lvl as usize].oldcol = (*itr).pos.col as ::core::ffi::c_int;
        if (*itr).i > 0 as ::core::ffi::c_int {
            compose(
                &mut (*itr).pos,
                (*(*itr).x).key[((*itr).i - 1 as ::core::ffi::c_int) as usize].pos,
            );
            relative(
                (*(*itr).x).key[((*itr).i - 1 as ::core::ffi::c_int) as usize].pos,
                &mut k.pos,
            );
        }
        (*itr).x = (*inner((*itr).x)).i_ptr[(*itr).i as usize];
        (*itr).lvl += 1;
        if !oldbase.is_null() {
            *oldbase.offset((*itr).lvl as isize) = (*itr).pos;
        }
    }
    if last {
        return marktree_itr_prev(b, itr);
    } else if (*itr).i as int32_t >= (*(*itr).x).n {
        return marktree_itr_next_skip(
            b,
            itr,
            true,
            false,
            ::core::ptr::null_mut::<MTPos>(),
            ::core::ptr::null::<uint32_t>(),
        );
    }
    return true;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_first(
    mut b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
) -> bool {
    if (*b).n_keys == 0 as size_t {
        (*itr).x = ::core::ptr::null_mut::<MTNode>();
        return false;
    }
    (*itr).x = (*b).root;
    (*itr).i = 0 as ::core::ffi::c_int;
    (*itr).lvl = 0 as ::core::ffi::c_int;
    (*itr).pos = MTPos {
        row: 0 as int32_t,
        col: 0 as int32_t,
    };
    while (*(*itr).x).level as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        (*itr).s[(*itr).lvl as usize].i = 0 as ::core::ffi::c_int;
        (*itr).s[(*itr).lvl as usize].oldcol = 0 as ::core::ffi::c_int;
        (*itr).lvl += 1;
        (*itr).x = (*inner((*itr).x)).i_ptr[0 as ::core::ffi::c_int as usize];
    }
    return true;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_next(
    mut b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
) -> bool {
    return marktree_itr_next_skip(
        b,
        itr,
        false,
        false,
        ::core::ptr::null_mut::<MTPos>(),
        ::core::ptr::null::<uint32_t>(),
    );
}

pub unsafe extern "C" fn marktree_itr_next_skip(
    mut _b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
    mut skip: bool,
    mut preload: bool,
    mut oldbase: *mut MTPos,
    mut meta_filter: MetaFilter,
) -> bool {
    if (*itr).x.is_null() {
        return false;
    }
    (*itr).i += 1;
    if !meta_filter.is_null() && (*(*itr).x).level as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
        if !meta_has(
            &(*inner((*itr).x)).i_meta[((*itr).i) as usize],
            &*meta_filter.cast(),
        ) {
            skip = true;
        }
    }
    if (*(*itr).x).level as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        || skip as ::core::ffi::c_int != 0
    {
        if preload as ::core::ffi::c_int != 0
            && (*(*itr).x).level as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && skip as ::core::ffi::c_int != 0
        {
            (*itr).i = (*(*itr).x).n as ::core::ffi::c_int;
        } else if ((*itr).i as int32_t) < (*(*itr).x).n {
            return true;
        }
        while ((*itr).i as int32_t) >= (*(*itr).x).n {
            (*itr).x = (*(*itr).x).parent;
            if (*itr).x.is_null() {
                return false;
            }
            (*itr).lvl -= 1;
            (*itr).i = (*itr).s[(*itr).lvl as usize].i;
            if (*itr).i > 0 as ::core::ffi::c_int {
                (*itr).pos.row -= (*(*itr).x).key[((*itr).i - 1 as ::core::ffi::c_int) as usize]
                    .pos
                    .row;
                (*itr).pos.col = (*itr).s[(*itr).lvl as usize].oldcol as int32_t;
            }
        }
    } else {
        while (*(*itr).x).level as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            if (*itr).i > 0 as ::core::ffi::c_int {
                (*itr).s[(*itr).lvl as usize].oldcol = (*itr).pos.col as ::core::ffi::c_int;
                compose(
                    &mut (*itr).pos,
                    (*(*itr).x).key[((*itr).i - 1 as ::core::ffi::c_int) as usize].pos,
                );
            }
            if !oldbase.is_null() && (*itr).i == 0 as ::core::ffi::c_int {
                *oldbase.offset(((*itr).lvl + 1 as ::core::ffi::c_int) as isize) =
                    *oldbase.offset((*itr).lvl as isize);
            }
            (*itr).s[(*itr).lvl as usize].i = (*itr).i;
            assert!(
                (*(*inner((*itr).x)).i_ptr[(*itr).i as usize]).parent == (*itr).x,
                "itr->x->ptr[itr->i]->parent == itr->x"
            );
            (*itr).lvl += 1;
            (*itr).x = (*inner((*itr).x)).i_ptr[(*itr).i as usize];
            if preload as ::core::ffi::c_int != 0 && (*(*itr).x).level as ::core::ffi::c_int != 0 {
                (*itr).i = -1 as ::core::ffi::c_int;
                break;
            } else {
                (*itr).i = 0 as ::core::ffi::c_int;
                if !(!meta_filter.is_null() && (*(*itr).x).level as ::core::ffi::c_int != 0) {
                    continue;
                }
                if !meta_has(&(*inner((*itr).x)).i_meta[0], &*meta_filter.cast()) {
                    break;
                }
            }
        }
    }
    return true;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_get_filter(
    mut b: *mut MarkTree,
    mut row: int32_t,
    mut col: ::core::ffi::c_int,
    mut stop_row: ::core::ffi::c_int,
    mut stop_col: ::core::ffi::c_int,
    mut meta_filter: MetaFilter,
    mut itr: *mut MarkTreeIter,
) -> bool {
    if !meta_has(&(*b).meta_root, &*meta_filter.cast()) {
        return false;
    }
    if !marktree_itr_get_ext(
        b,
        MTPos {
            row: row,
            col: col as int32_t,
        },
        itr,
        false,
        false,
        ::core::ptr::null_mut::<MTPos>(),
        meta_filter,
    ) {
        return false;
    }
    return marktree_itr_check_filter(b, itr, stop_row, stop_col, meta_filter);
}

pub unsafe extern "C" fn marktree_itr_step_out_filter(
    mut b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
    mut meta_filter: MetaFilter,
) -> bool {
    if !meta_has(&(*b).meta_root, &*meta_filter.cast()) {
        (*itr).x = ::core::ptr::null_mut::<MTNode>();
        return false;
    }
    while !(*itr).x.is_null() && !(*(*itr).x).parent.is_null() {
        if meta_has(
            &(*inner((*(*itr).x).parent)).i_meta[(*(*itr).x).p_idx as usize],
            &*meta_filter.cast(),
        ) {
            return true;
        }
        (*itr).i = (*(*itr).x).n as ::core::ffi::c_int;
        marktree_itr_next_skip(
            b,
            itr,
            true,
            false,
            ::core::ptr::null_mut::<MTPos>(),
            ::core::ptr::null::<uint32_t>(),
        );
    }
    return !(*itr).x.is_null();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_next_filter(
    mut b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
    mut stop_row: ::core::ffi::c_int,
    mut stop_col: ::core::ffi::c_int,
    mut meta_filter: MetaFilter,
) -> bool {
    if !marktree_itr_next_skip(
        b,
        itr,
        false,
        false,
        ::core::ptr::null_mut::<MTPos>(),
        meta_filter,
    ) {
        return false;
    }
    return marktree_itr_check_filter(b, itr, stop_row, stop_col, meta_filter);
}

pub unsafe extern "C" fn marktree_itr_check_filter(
    mut b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
    mut stop_row: ::core::ffi::c_int,
    mut stop_col: ::core::ffi::c_int,
    mut meta_filter: MetaFilter,
) -> bool {
    let mut stop_pos: MTPos = MTPos {
        row: stop_row as int32_t,
        col: stop_col as int32_t,
    };
    let key_filter = filtered_key_flags(&*meta_filter.cast());
    loop {
        if pos_leq(stop_pos, marktree_itr_pos(itr)) {
            (*itr).x = ::core::ptr::null_mut::<MTNode>();
            return false;
        }
        let mut k: MTKey = (*(*itr).x).key[(*itr).i as usize];
        if !mt_end(k) && k.flags as uint32_t & key_filter != 0 {
            return true;
        }
        if !marktree_itr_next_skip(
            b,
            itr,
            false,
            false,
            ::core::ptr::null_mut::<MTPos>(),
            meta_filter,
        ) {
            return false;
        }
    }
}

pub unsafe extern "C" fn marktree_itr_prev(
    mut _b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
) -> bool {
    if (*itr).x.is_null() {
        return false;
    }
    if (*(*itr).x).level as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        (*itr).i -= 1;
        if (*itr).i >= 0 as ::core::ffi::c_int {
            return true;
        }
        while (*itr).i < 0 as ::core::ffi::c_int {
            (*itr).x = (*(*itr).x).parent;
            if (*itr).x.is_null() {
                return false;
            }
            (*itr).lvl -= 1;
            (*itr).i = (*itr).s[(*itr).lvl as usize].i - 1 as ::core::ffi::c_int;
            if (*itr).i >= 0 as ::core::ffi::c_int {
                (*itr).pos.row -= (*(*itr).x).key[(*itr).i as usize].pos.row;
                (*itr).pos.col = (*itr).s[(*itr).lvl as usize].oldcol as int32_t;
            }
        }
    } else {
        while (*(*itr).x).level as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
            if (*itr).i > 0 as ::core::ffi::c_int {
                (*itr).s[(*itr).lvl as usize].oldcol = (*itr).pos.col as ::core::ffi::c_int;
                compose(
                    &mut (*itr).pos,
                    (*(*itr).x).key[((*itr).i - 1 as ::core::ffi::c_int) as usize].pos,
                );
            }
            (*itr).s[(*itr).lvl as usize].i = (*itr).i;
            assert!(
                (*(*inner((*itr).x)).i_ptr[(*itr).i as usize]).parent == (*itr).x,
                "itr->x->ptr[itr->i]->parent == itr->x"
            );
            (*itr).x = (*inner((*itr).x)).i_ptr[(*itr).i as usize];
            (*itr).i = (*(*itr).x).n as ::core::ffi::c_int;
            (*itr).lvl += 1;
        }
        (*itr).i -= 1;
    }
    return true;
}

pub unsafe extern "C" fn marktree_itr_pos(mut itr: *mut MarkTreeIter) -> MTPos {
    let mut pos: MTPos = (*(*itr).x).key[(*itr).i as usize].pos;
    unrelative((*itr).pos, &mut pos);
    return pos;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_current(mut itr: *mut MarkTreeIter) -> MTKey {
    if !(*itr).x.is_null() {
        let mut key: MTKey = (*(*itr).x).key[(*itr).i as usize];
        key.pos = marktree_itr_pos(itr);
        return key;
    }
    return MT_INVALID_KEY;
}

pub unsafe extern "C" fn itr_eq(mut itr1: *mut MarkTreeIter, mut itr2: *mut MarkTreeIter) -> bool {
    return (&raw mut (*(*itr1).x).key as *mut MTKey).offset((*itr1).i as isize)
        == (&raw mut (*(*itr2).x).key as *mut MTKey).offset((*itr2).i as isize);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_get_overlap(
    mut b: *mut MarkTree,
    mut row: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut itr: *mut MarkTreeIter,
) -> bool {
    if (*b).n_keys == 0 as size_t {
        (*itr).x = ::core::ptr::null_mut::<MTNode>();
        return false;
    }
    (*itr).x = (*b).root;
    (*itr).i = -1 as ::core::ffi::c_int;
    (*itr).lvl = 0 as ::core::ffi::c_int;
    (*itr).pos = MTPos {
        row: 0 as int32_t,
        col: 0 as int32_t,
    };
    (*itr).intersect_pos = MTPos {
        row: row as int32_t,
        col: col as int32_t,
    };
    (*itr).intersect_pos_x = MTPos {
        row: row as int32_t,
        col: col as int32_t,
    };
    (*itr).intersect_idx = 0 as size_t;
    return true;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_step_overlap(
    mut b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
    mut pair: *mut MTPair,
) -> bool {
    while (*itr).i == -1 as ::core::ffi::c_int {
        if (*itr).intersect_idx < ix((*itr).x).len() {
            let id = ix((*itr).x).as_slice()[(*itr).intersect_idx];
            (*itr).intersect_idx += 1;
            *pair = mtpair_from(
                marktree_lookup(b, id, ::core::ptr::null_mut::<MarkTreeIter>()),
                marktree_lookup(
                    b,
                    id | MARKTREE_END_FLAG,
                    ::core::ptr::null_mut::<MarkTreeIter>(),
                ),
            );
            return true;
        }
        if (*(*itr).x).level as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
            (*itr).i = 0 as ::core::ffi::c_int;
            (*itr).s[(*itr).lvl as usize].i = (*itr).i;
            break;
        } else {
            let mut k: MTKey = MTKey {
                pos: (*itr).intersect_pos_x,
                ns: 0,
                id: 0,
                flags: 0 as uint16_t,
                decor_data: DecorInlineData {
                    hl: DecorHighlightInline {
                        flags: 0,
                        priority: 0,
                        hl_id: 0,
                        conceal_char: 0,
                    },
                },
            };
            (*itr).i = find_key(node_keys((*itr).x), k).0 + 1 as ::core::ffi::c_int;
            (*itr).s[(*itr).lvl as usize].i = (*itr).i;
            (*itr).s[(*itr).lvl as usize].oldcol = (*itr).pos.col as ::core::ffi::c_int;
            if (*itr).i > 0 as ::core::ffi::c_int {
                compose(
                    &mut (*itr).pos,
                    (*(*itr).x).key[((*itr).i - 1 as ::core::ffi::c_int) as usize].pos,
                );
                relative(
                    (*(*itr).x).key[((*itr).i - 1 as ::core::ffi::c_int) as usize].pos,
                    &mut (*itr).intersect_pos_x,
                );
            }
            (*itr).x = (*inner((*itr).x)).i_ptr[(*itr).i as usize];
            (*itr).lvl += 1;
            (*itr).i = -1 as ::core::ffi::c_int;
            (*itr).intersect_idx = 0 as size_t;
        }
    }
    while ((*itr).i as int32_t) < (*(*itr).x).n
        && pos_less(
            (*(*itr).x).key[(*itr).i as usize].pos,
            (*itr).intersect_pos_x,
        ) as ::core::ffi::c_int
            != 0
    {
        let c2rust_fresh19 = (*itr).i;
        (*itr).i = (*itr).i + 1;
        let mut k_0: MTKey = (*(*itr).x).key[c2rust_fresh19 as usize];
        (*itr).s[(*itr).lvl as usize].i = (*itr).i;
        if !mt_start(k_0) {
            continue;
        }
        let mut end: MTKey = marktree_lookup(
            b,
            mt_lookup_id(k_0.ns, k_0.id, true),
            ::core::ptr::null_mut::<MarkTreeIter>(),
        );
        if pos_less(end.pos, (*itr).intersect_pos) {
            continue;
        }
        unrelative((*itr).pos, &mut k_0.pos);
        *pair = mtpair_from(k_0, end);
        return true;
    }
    while ((*itr).i as int32_t) < (*(*itr).x).n {
        let c2rust_fresh20 = (*itr).i;
        (*itr).i = (*itr).i + 1;
        let mut k_1: MTKey = (*(*itr).x).key[c2rust_fresh20 as usize];
        if !mt_end(k_1) {
            continue;
        }
        let mut id_0: uint64_t = mt_lookup_id(k_1.ns, k_1.id, false);
        if id2node(b, id_0) == (*itr).x {
            continue;
        }
        unrelative((*itr).pos, &mut k_1.pos);
        let mut start: MTKey = marktree_lookup(b, id_0, ::core::ptr::null_mut::<MarkTreeIter>());
        if pos_leq((*itr).intersect_pos, start.pos) {
            continue;
        }
        *pair = mtpair_from(start, k_1);
        return true;
    }
    (*itr).i = (*itr).s[(*itr).lvl as usize].i;
    assert!((*itr).i >= 0 as ::core::ffi::c_int, "itr->i >= 0");
    if (*itr).i as int32_t >= (*(*itr).x).n {
        marktree_itr_next(b, itr);
    }
    return false;
}

pub unsafe extern "C" fn marktree_itr_set_node(
    mut b: *mut MarkTree,
    mut itr: *mut MarkTreeIter,
    mut n: *mut MTNode,
    mut i: ::core::ffi::c_int,
) -> MTKey {
    let mut key: MTKey = (*n).key[i as usize];
    if !itr.is_null() {
        (*itr).i = i;
        (*itr).x = n;
        (*itr).lvl = (*(*b).root).level as ::core::ffi::c_int - (*n).level as ::core::ffi::c_int;
    }
    while !(*n).parent.is_null() {
        let mut p: *mut MTNode = (*n).parent;
        i = (*n).p_idx as ::core::ffi::c_int;
        assert!((*inner(p)).i_ptr[i as usize] == n, "p->ptr[i] == n");
        if !itr.is_null() {
            (*itr)
                .s[((*(*b).root).level as ::core::ffi::c_int
                    - (*p).level as ::core::ffi::c_int) as usize]
                .i = i;
        }
        if i > 0 as ::core::ffi::c_int {
            unrelative(
                (*p).key[(i - 1 as ::core::ffi::c_int) as usize].pos,
                &mut key.pos,
            );
        }
        n = p;
    }
    if !itr.is_null() {
        marktree_itr_fix_pos(b, itr);
    }
    return key;
}

pub unsafe extern "C" fn marktree_itr_fix_pos(mut b: *mut MarkTree, mut itr: *mut MarkTreeIter) {
    (*itr).pos = MTPos {
        row: 0 as int32_t,
        col: 0 as int32_t,
    };
    let mut x: *mut MTNode = (*b).root;
    let mut lvl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while lvl < (*itr).lvl {
        (*itr).s[lvl as usize].oldcol = (*itr).pos.col as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = (*itr).s[lvl as usize].i;
        if i > 0 as ::core::ffi::c_int {
            compose(
                &mut (*itr).pos,
                (*x).key[(i - 1 as ::core::ffi::c_int) as usize].pos,
            );
        }
        assert!((*x).level != 0, "x->level");
        x = (*inner(x)).i_ptr[i as usize];
        lvl += 1;
    }
    assert!(x == (*itr).x, "x == itr->x");
}
