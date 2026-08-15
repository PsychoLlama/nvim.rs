// Not graduated yet: the parent module denies `unsafe_op_in_unsafe_fn` and the
// level is inherited, so these transpiled bodies opt back out until the
// rewrite that narrows them. Remove this when the deny goes on.
#![allow(unsafe_op_in_unsafe_fn)]

//! Keeping the tree balanced across an insertion or a deletion.
//!
//! A node holds between `MT_BRANCH_FACTOR - 1` and `2 * MT_BRANCH_FACTOR - 1`
//! keys. Insertion splits a full child on the way down; deletion borrows from a
//! sibling ([`pivot_left`]/[`pivot_right`]) or merges with one
//! ([`merge_node`]) on the way back up.
//!
//! What makes this more than a textbook B-tree is that every key position is
//! stored relative to the key before it, and every node carries meta counts and
//! a set of covering ranges. So each of these operations has three jobs at
//! once: move the keys, rebase the positions of everything on either side of
//! the boundary that moved, and re-home the meta counts and the intersection
//! sets. Getting any one of the three wrong shows up as a corrupt tree several
//! operations later, which is what `marktree_check` exists to catch.

use super::*;

pub unsafe extern "C" fn split_node(
    mut b: *mut MarkTree,
    mut x: *mut MTNode,
    i: ::core::ffi::c_int,
    mut next: MTKey,
) {
    let mut y: *mut MTNode = (*inner(x)).i_ptr[i as usize];
    let mut z: *mut MTNode = marktree_alloc_node(b, (*y).level != 0);
    (*z).level = (*y).level;
    (*z).n = (MT_BRANCH_FACTOR as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int32_t;
    let mut last_start: uint64_t = if mt_end(next) as ::core::ffi::c_int != 0 {
        mt_lookup_id(next.ns, next.id, false)
    } else {
        MARKTREE_END_FLAG
    };
    // z inherits everything y intersected: the split does not change which
    // ranges cover either half.
    ix(z).clear();
    ix(z).extend_from_slice(ix(y).as_slice());
    if (*y).level == 0 {
        let mut pi: uint64_t = pseudo_index(y, 0 as ::core::ffi::c_int);
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < MT_BRANCH_FACTOR as ::core::ffi::c_int {
            let mut k: MTKey = (*y).key[j as usize];
            let mut pi_end = pseudo_index_for_id(&mut *b, mt_lookup_id(k.ns, k.id, true), true);
            if mt_start(k) as ::core::ffi::c_int != 0
                && pi_end > pi
                && mt_lookup_key(k) != last_start
            {
                intersect_node(Node::new(z), mt_lookup_id(k.ns, k.id, false));
            }
            j += 1;
        }
        let mut j_0: ::core::ffi::c_int =
            MT_BRANCH_FACTOR as ::core::ffi::c_int - 1 as ::core::ffi::c_int;
        while j_0
            < MT_BRANCH_FACTOR as ::core::ffi::c_int * 2 as ::core::ffi::c_int
                - 1 as ::core::ffi::c_int
        {
            let mut k_0: MTKey = (*y).key[j_0 as usize];
            let mut pi_start =
                pseudo_index_for_id(&mut *b, mt_lookup_id(k_0.ns, k_0.id, false), true);
            if mt_end(k_0) as ::core::ffi::c_int != 0 && pi_start > 0 as uint64_t && pi_start < pi {
                intersect_node(Node::new(y), mt_lookup_id(k_0.ns, k_0.id, false));
            }
            j_0 += 1;
        }
    }
    memcpy(
        &raw mut (*z).key as *mut MTKey as *mut ::core::ffi::c_void,
        (&raw mut (*y).key as *mut MTKey).offset(MT_BRANCH_FACTOR as ::core::ffi::c_int as isize)
            as *const ::core::ffi::c_void,
        ::core::mem::size_of::<MTKey>().wrapping_mul(
            (MT_BRANCH_FACTOR as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as size_t,
        ),
    );
    let mut j_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while j_1 < MT_BRANCH_FACTOR as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        refkey(b, z, j_1);
        j_1 += 1;
    }
    if (*y).level != 0 {
        memcpy(
            &raw mut (*inner(z)).i_ptr as *mut *mut MTNode as *mut ::core::ffi::c_void,
            (&raw mut (*inner(y)).i_ptr as *mut *mut MTNode)
                .offset(MT_BRANCH_FACTOR as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<*mut MTNode>()
                .wrapping_mul(MT_BRANCH_FACTOR as ::core::ffi::c_int as size_t),
        );
        memcpy(
            &raw mut (*inner(z)).i_meta as *mut [uint32_t; 5] as *mut ::core::ffi::c_void,
            (&raw mut (*inner(y)).i_meta as *mut [uint32_t; 5])
                .offset(MT_BRANCH_FACTOR as ::core::ffi::c_int as isize)
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[uint32_t; 5]>()
                .wrapping_mul(MT_BRANCH_FACTOR as ::core::ffi::c_int as size_t),
        );
        let mut j_2: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j_2 < MT_BRANCH_FACTOR as ::core::ffi::c_int {
            (*(*inner(z)).i_ptr[j_2 as usize]).parent = z;
            (*(*inner(z)).i_ptr[j_2 as usize]).p_idx = j_2 as int16_t;
            j_2 += 1;
        }
    }
    (*y).n = (MT_BRANCH_FACTOR as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as int32_t;
    memmove(
        (&raw mut (*inner(x)).i_ptr as *mut *mut MTNode)
            .offset((i + 2 as ::core::ffi::c_int) as isize) as *mut ::core::ffi::c_void,
        (&raw mut (*inner(x)).i_ptr as *mut *mut MTNode)
            .offset((i + 1 as ::core::ffi::c_int) as isize) as *const ::core::ffi::c_void,
        ::core::mem::size_of::<*mut MTNode>().wrapping_mul(((*x).n - i as int32_t) as size_t),
    );
    memmove(
        (&raw mut (*inner(x)).i_meta as *mut [uint32_t; 5])
            .offset((i + 2 as ::core::ffi::c_int) as isize) as *mut ::core::ffi::c_void,
        (&raw mut (*inner(x)).i_meta as *mut [uint32_t; 5])
            .offset((i + 1 as ::core::ffi::c_int) as isize) as *const ::core::ffi::c_void,
        ::core::mem::size_of::<[uint32_t; 5]>().wrapping_mul(((*x).n - i as int32_t) as size_t),
    );
    (*inner(x)).i_ptr[(i + 1 as ::core::ffi::c_int) as usize] = z;
    (*inner(x)).i_meta[(i + 1 as ::core::ffi::c_int) as usize] = meta_describe_node(z);
    (*z).parent = x;
    let mut j_3: ::core::ffi::c_int = i + 1 as ::core::ffi::c_int;
    while (j_3 as int32_t) < (*x).n + 2 as int32_t {
        (*(*inner(x)).i_ptr[j_3 as usize]).p_idx = j_3 as int16_t;
        j_3 += 1;
    }
    memmove(
        (&raw mut (*x).key as *mut MTKey).offset((i + 1 as ::core::ffi::c_int) as isize)
            as *mut ::core::ffi::c_void,
        (&raw mut (*x).key as *mut MTKey).offset(i as isize) as *const ::core::ffi::c_void,
        ::core::mem::size_of::<MTKey>().wrapping_mul(((*x).n - i as int32_t) as size_t),
    );
    (*x).key[i as usize] =
        (*y).key[(MT_BRANCH_FACTOR as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as usize];
    refkey(b, x, i);
    (*x).n += 1;
    let mut meta_inc = meta_describe_key((*x).key[i as usize]);
    let moved = (*inner(x)).i_meta[(i + 1) as usize];
    meta_sub(&mut (*inner(x)).i_meta[i as usize], &moved);
    meta_sub(&mut (*inner(x)).i_meta[i as usize], &meta_inc);
    let mut j_4: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while j_4 < MT_BRANCH_FACTOR as ::core::ffi::c_int - 1 as ::core::ffi::c_int {
        relative(
            (*x).key[i as usize].pos,
            &mut (*(&mut (*z).key as *mut MTKey).offset(j_4 as isize)).pos,
        );
        j_4 += 1;
    }
    if i > 0 as ::core::ffi::c_int {
        unrelative(
            (*x).key[(i - 1 as ::core::ffi::c_int) as usize].pos,
            &mut (*(&mut (*x).key as *mut MTKey).offset(i as isize)).pos,
        );
    }
    if (*y).level != 0 {
        bubble_up(y);
        bubble_up(z);
    }
}

pub unsafe extern "C" fn merge_node(
    mut b: *mut MarkTree,
    mut p: *mut MTNode,
    mut i: ::core::ffi::c_int,
) -> *mut MTNode {
    let mut x: *mut MTNode = (*inner(p)).i_ptr[i as usize];
    let mut y: *mut MTNode = (*inner(p)).i_ptr[(i + 1 as ::core::ffi::c_int) as usize];
    // What x and y both intersected becomes the merged node's own set; what
    // only one of them did stays on that half's keys.
    let mut merged = Intersection {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut(),
        init_array: [0; 4],
    };
    let merged = IdSet::new(&raw mut merged);
    merged.init();
    intersect_merge(&merged, &ix(x), &ix(y));
    (*x).key[(*x).n as usize] = (*p).key[i as usize];
    refkey(b, x, (*x).n as ::core::ffi::c_int);
    if i > 0 as ::core::ffi::c_int {
        relative(
            (*p).key[(i - 1 as ::core::ffi::c_int) as usize].pos,
            &mut (*(&mut (*x).key as *mut MTKey).offset((*x).n as isize)).pos,
        );
    }
    let mut meta_inc = meta_describe_key((*x).key[(*x).n as usize]);
    memmove(
        (&raw mut (*x).key as *mut MTKey).offset(((*x).n + 1 as int32_t) as isize)
            as *mut ::core::ffi::c_void,
        &raw mut (*y).key as *mut MTKey as *const ::core::ffi::c_void,
        ((*y).n as size_t).wrapping_mul(::core::mem::size_of::<MTKey>()),
    );
    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (k as int32_t) < (*y).n {
        refkey(
            b,
            x,
            (*x).n as ::core::ffi::c_int + 1 as ::core::ffi::c_int + k,
        );
        unrelative(
            (*x).key[(*x).n as usize].pos,
            &mut (*(&mut (*x).key as *mut MTKey)
                .offset(((*x).n + 1 as int32_t + k as int32_t) as isize))
            .pos,
        );
        k += 1;
    }
    if (*x).level != 0 {
        memmove(
            (&raw mut (*inner(x)).i_ptr as *mut *mut MTNode)
                .offset(((*x).n + 1 as int32_t) as isize) as *mut ::core::ffi::c_void,
            &raw mut (*inner(y)).i_ptr as *mut *mut MTNode as *const ::core::ffi::c_void,
            ((*y).n as size_t)
                .wrapping_add(1 as size_t)
                .wrapping_mul(::core::mem::size_of::<*mut MTNode>()),
        );
        memmove(
            (&raw mut (*inner(x)).i_meta as *mut [uint32_t; 5])
                .offset(((*x).n + 1 as int32_t) as isize) as *mut ::core::ffi::c_void,
            &raw mut (*inner(y)).i_meta as *mut [uint32_t; 5] as *const ::core::ffi::c_void,
            ((*y).n as size_t)
                .wrapping_add(1 as size_t)
                .wrapping_mul(::core::mem::size_of::<[uint32_t; 5]>()),
        );
        let mut k_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (k_0 as int32_t) < (*x).n + 1 as int32_t {
            for &id in ix(x).as_slice() {
                intersect_node(Node::new((*inner(x)).i_ptr[k_0 as usize]), id);
            }
            k_0 += 1;
        }
        let mut ky: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (ky as int32_t) < (*y).n + 1 as int32_t {
            let mut k_1: ::core::ffi::c_int =
                (*x).n as ::core::ffi::c_int + ky + 1 as ::core::ffi::c_int;
            (*(*inner(x)).i_ptr[k_1 as usize]).parent = x;
            (*(*inner(x)).i_ptr[k_1 as usize]).p_idx = k_1 as int16_t;
            for &id in ix(y).as_slice() {
                intersect_node(Node::new((*inner(x)).i_ptr[k_1 as usize]), id);
            }
            ky += 1;
        }
    }
    (*x).n =
        ((*x).n as ::core::ffi::c_int + ((*y).n + 1 as int32_t) as ::core::ffi::c_int) as int32_t;
    let absorbed = (*inner(p)).i_meta[(i + 1) as usize];
    meta_add(&mut (*inner(p)).i_meta[i as usize], &absorbed);
    meta_add(&mut (*inner(p)).i_meta[i as usize], &meta_inc);
    memmove(
        (&raw mut (*p).key as *mut MTKey).offset(i as isize) as *mut ::core::ffi::c_void,
        (&raw mut (*p).key as *mut MTKey).offset((i + 1 as ::core::ffi::c_int) as isize)
            as *const ::core::ffi::c_void,
        (((*p).n - i as int32_t - 1 as int32_t) as size_t)
            .wrapping_mul(::core::mem::size_of::<MTKey>()),
    );
    memmove(
        (&raw mut (*inner(p)).i_ptr as *mut *mut MTNode)
            .offset((i + 1 as ::core::ffi::c_int) as isize) as *mut ::core::ffi::c_void,
        (&raw mut (*inner(p)).i_ptr as *mut *mut MTNode)
            .offset((i + 2 as ::core::ffi::c_int) as isize) as *const ::core::ffi::c_void,
        (((*p).n - i as int32_t - 1 as int32_t) as size_t)
            .wrapping_mul(::core::mem::size_of::<*mut MTKey>()),
    );
    memmove(
        (&raw mut (*inner(p)).i_meta as *mut [uint32_t; 5])
            .offset((i + 1 as ::core::ffi::c_int) as isize) as *mut ::core::ffi::c_void,
        (&raw mut (*inner(p)).i_meta as *mut [uint32_t; 5])
            .offset((i + 2 as ::core::ffi::c_int) as isize) as *const ::core::ffi::c_void,
        (((*p).n - i as int32_t - 1 as int32_t) as size_t)
            .wrapping_mul(::core::mem::size_of::<[uint32_t; 5]>()),
    );
    let mut j: ::core::ffi::c_int = i + 1 as ::core::ffi::c_int;
    while (j as int32_t) < (*p).n {
        (*(*inner(p)).i_ptr[j as usize]).p_idx = j as int16_t;
        j += 1;
    }
    (*p).n -= 1;
    marktree_free_node(b, y);
    xfree(ix(x).take_heap());
    ix(x).move_from(&merged);
    return x;
}

pub unsafe extern "C" fn pivot_right(
    mut b: *mut MarkTree,
    mut _p_pos: MTPos,
    mut p: *mut MTNode,
    i: ::core::ffi::c_int,
) {
    let mut x: *mut MTNode = (*inner(p)).i_ptr[i as usize];
    let mut y: *mut MTNode = (*inner(p)).i_ptr[(i + 1 as ::core::ffi::c_int) as usize];
    memmove(
        (&raw mut (*y).key as *mut MTKey).offset(1 as ::core::ffi::c_int as isize)
            as *mut ::core::ffi::c_void,
        &raw mut (*y).key as *mut MTKey as *const ::core::ffi::c_void,
        ((*y).n as size_t).wrapping_mul(::core::mem::size_of::<MTKey>()),
    );
    if (*y).level != 0 {
        memmove(
            (&raw mut (*inner(y)).i_ptr as *mut *mut MTNode)
                .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            &raw mut (*inner(y)).i_ptr as *mut *mut MTNode as *const ::core::ffi::c_void,
            ((*y).n as size_t)
                .wrapping_add(1 as size_t)
                .wrapping_mul(::core::mem::size_of::<*mut MTNode>()),
        );
        memmove(
            (&raw mut (*inner(y)).i_meta as *mut [uint32_t; 5])
                .offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            &raw mut (*inner(y)).i_meta as *mut [uint32_t; 5] as *const ::core::ffi::c_void,
            ((*y).n as size_t)
                .wrapping_add(1 as size_t)
                .wrapping_mul(::core::mem::size_of::<[uint32_t; 5]>()),
        );
        let mut j: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        while (j as int32_t) < (*y).n + 2 as int32_t {
            (*(*inner(y)).i_ptr[j as usize]).p_idx = j as int16_t;
            j += 1;
        }
    }
    (*y).key[0 as ::core::ffi::c_int as usize] = (*p).key[i as usize];
    refkey(b, y, 0 as ::core::ffi::c_int);
    (*p).key[i as usize] = (*x).key[((*x).n - 1 as int32_t) as usize];
    refkey(b, p, i);
    let mut meta_inc_y = meta_describe_key((*y).key[0 as ::core::ffi::c_int as usize]);
    let mut meta_inc_x = meta_describe_key((*p).key[i as usize]);
    meta_add(&mut (*inner(p)).i_meta[(i + 1) as usize], &meta_inc_y);
    meta_sub(&mut (*inner(p)).i_meta[i as usize], &meta_inc_x);
    if (*x).level != 0 {
        (*inner(y)).i_ptr[0 as ::core::ffi::c_int as usize] = (*inner(x)).i_ptr[(*x).n as usize];
        memcpy(
            &raw mut *(&raw mut (*inner(y)).i_meta as *mut [uint32_t; 5])
                .offset(0 as ::core::ffi::c_int as isize) as *mut uint32_t
                as *mut ::core::ffi::c_void,
            &raw mut *(&raw mut (*inner(x)).i_meta as *mut [uint32_t; 5]).offset((*x).n as isize)
                as *mut uint32_t as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[uint32_t; 5]>(),
        );
        let moved = (*inner(y)).i_meta[0];
        meta_add(&mut (*inner(p)).i_meta[(i + 1) as usize], &moved);
        meta_sub(&mut (*inner(p)).i_meta[i as usize], &moved);
        (*(*inner(y)).i_ptr[0 as ::core::ffi::c_int as usize]).parent = y;
        (*(*inner(y)).i_ptr[0 as ::core::ffi::c_int as usize]).p_idx = 0 as int16_t;
    }
    (*x).n -= 1;
    (*y).n += 1;
    if i > 0 as ::core::ffi::c_int {
        unrelative(
            (*p).key[(i - 1 as ::core::ffi::c_int) as usize].pos,
            &mut (*(&mut (*p).key as *mut MTKey).offset(i as isize)).pos,
        );
    }
    relative(
        (*p).key[i as usize].pos,
        &mut (*(&mut (*y).key as *mut MTKey).offset(0 as ::core::ffi::c_int as isize)).pos,
    );
    let mut k: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while (k as int32_t) < (*y).n {
        unrelative(
            (*y).key[0 as ::core::ffi::c_int as usize].pos,
            &mut (*(&mut (*y).key as *mut MTKey).offset(k as isize)).pos,
        );
        k += 1;
    }
    if (*x).level != 0 {
        // Ids y's other children have to take on themselves, now that the
        // moved child no longer shares them.
        let mut demoted = Intersection {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut(),
            init_array: [0; 4],
        };
        let demoted = IdSet::new(&raw mut demoted);
        demoted.init();
        intersect_mov(&ix(x), &ix(y), &ix((*inner(y)).i_ptr[0]), &demoted);
        if !demoted.is_empty() {
            for yi in 1..=(*y).n as usize {
                intersect_add(&ix((*inner(y)).i_ptr[yi]), &demoted);
            }
        }
        xfree(demoted.take_heap());
        bubble_up(x);
    } else {
        if mt_end((*p).key[i as usize]) {
            let mut pi: uint64_t = pseudo_index(x, 0 as ::core::ffi::c_int);
            let mut start_id: uint64_t = mt_lookup_key_side((*p).key[i as usize], false);
            let mut pi_start = pseudo_index_for_id(&mut *b, start_id, true);
            if pi_start > 0 as uint64_t && pi_start < pi {
                intersect_node(Node::new(x), start_id);
            }
        }
        if mt_start((*y).key[0 as ::core::ffi::c_int as usize]) {
            unintersect_node(
                Node::new(y),
                mt_lookup_key((*y).key[0 as ::core::ffi::c_int as usize]),
                false,
            );
        }
    };
}

pub unsafe extern "C" fn pivot_left(
    mut b: *mut MarkTree,
    mut _p_pos: MTPos,
    mut p: *mut MTNode,
    mut i: ::core::ffi::c_int,
) {
    let mut x: *mut MTNode = (*inner(p)).i_ptr[i as usize];
    let mut y: *mut MTNode = (*inner(p)).i_ptr[(i + 1 as ::core::ffi::c_int) as usize];
    let mut k: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    while (k as int32_t) < (*y).n {
        relative(
            (*y).key[0 as ::core::ffi::c_int as usize].pos,
            &mut (*(&mut (*y).key as *mut MTKey).offset(k as isize)).pos,
        );
        k += 1;
    }
    unrelative(
        (*p).key[i as usize].pos,
        &mut (*(&mut (*y).key as *mut MTKey).offset(0 as ::core::ffi::c_int as isize)).pos,
    );
    if i > 0 as ::core::ffi::c_int {
        relative(
            (*p).key[(i - 1 as ::core::ffi::c_int) as usize].pos,
            &mut (*(&mut (*p).key as *mut MTKey).offset(i as isize)).pos,
        );
    }
    (*x).key[(*x).n as usize] = (*p).key[i as usize];
    refkey(b, x, (*x).n as ::core::ffi::c_int);
    (*p).key[i as usize] = (*y).key[0 as ::core::ffi::c_int as usize];
    refkey(b, p, i);
    let mut meta_inc_x = meta_describe_key((*x).key[(*x).n as usize]);
    let mut meta_inc_y = meta_describe_key((*p).key[i as usize]);
    meta_add(&mut (*inner(p)).i_meta[i as usize], &meta_inc_x);
    meta_sub(&mut (*inner(p)).i_meta[(i + 1) as usize], &meta_inc_y);
    if (*x).level != 0 {
        (*inner(x)).i_ptr[((*x).n + 1 as int32_t) as usize] =
            (*inner(y)).i_ptr[0 as ::core::ffi::c_int as usize];
        memcpy(
            &raw mut *(&raw mut (*inner(x)).i_meta as *mut [uint32_t; 5])
                .offset(((*x).n + 1 as int32_t) as isize) as *mut uint32_t
                as *mut ::core::ffi::c_void,
            &raw mut *(&raw mut (*inner(y)).i_meta as *mut [uint32_t; 5])
                .offset(0 as ::core::ffi::c_int as isize) as *mut uint32_t
                as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[uint32_t; 5]>(),
        );
        let moved = (*inner(y)).i_meta[0];
        meta_sub(&mut (*inner(p)).i_meta[(i + 1) as usize], &moved);
        meta_add(&mut (*inner(p)).i_meta[i as usize], &moved);
        (*(*inner(x)).i_ptr[((*x).n + 1 as int32_t) as usize]).parent = x;
        (*(*inner(x)).i_ptr[((*x).n + 1 as int32_t) as usize]).p_idx =
            ((*x).n + 1 as int32_t) as int16_t;
    }
    memmove(
        &raw mut (*y).key as *mut MTKey as *mut ::core::ffi::c_void,
        (&raw mut (*y).key as *mut MTKey).offset(1 as ::core::ffi::c_int as isize)
            as *const ::core::ffi::c_void,
        (((*y).n - 1 as int32_t) as size_t).wrapping_mul(::core::mem::size_of::<MTKey>()),
    );
    if (*y).level != 0 {
        memmove(
            &raw mut (*inner(y)).i_ptr as *mut *mut MTNode as *mut ::core::ffi::c_void,
            (&raw mut (*inner(y)).i_ptr as *mut *mut MTNode)
                .offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            ((*y).n as size_t).wrapping_mul(::core::mem::size_of::<*mut MTNode>()),
        );
        memmove(
            &raw mut (*inner(y)).i_meta as *mut [uint32_t; 5] as *mut ::core::ffi::c_void,
            (&raw mut (*inner(y)).i_meta as *mut [uint32_t; 5])
                .offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
            ((*y).n as size_t).wrapping_mul(::core::mem::size_of::<[uint32_t; 5]>()),
        );
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (j as int32_t) < (*y).n {
            (*(*inner(y)).i_ptr[j as usize]).p_idx = j as int16_t;
            j += 1;
        }
    }
    (*x).n += 1;
    (*y).n -= 1;
    if (*x).level != 0 {
        // Ids y's other children have to take on themselves, now that the
        // moved child no longer shares them.
        let mut demoted = Intersection {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut(),
            init_array: [0; 4],
        };
        let demoted = IdSet::new(&raw mut demoted);
        demoted.init();
        intersect_mov(
            &ix(y),
            &ix(x),
            &ix((*inner(x)).i_ptr[(*x).n as usize]),
            &demoted,
        );
        if !demoted.is_empty() {
            for xi in 0..(*x).n as usize {
                intersect_add(&ix((*inner(x)).i_ptr[xi]), &demoted);
            }
        }
        xfree(demoted.take_heap());
        bubble_up(y);
    } else {
        if mt_start((*p).key[i as usize]) {
            let mut pi: uint64_t = pseudo_index(y, 0 as ::core::ffi::c_int);
            let mut end_id: uint64_t = mt_lookup_key_side((*p).key[i as usize], true);
            let mut pi_end = pseudo_index_for_id(&mut *b, end_id, true);
            if pi_end > pi {
                intersect_node(Node::new(y), mt_lookup_key((*p).key[i as usize]));
            }
        }
        if mt_end((*x).key[((*x).n - 1 as int32_t) as usize]) {
            unintersect_node(
                Node::new(x),
                mt_lookup_key_side((*x).key[((*x).n - 1 as int32_t) as usize], false),
                false,
            );
        }
    };
}

/// `x` shrank, or is one half of a split. Ranges that used to cover every one
/// of its children now cover `x` itself, so hoist them one level.
fn bubble_up(x: *mut MTNode) {
    let mut common = Intersection {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut(),
        init_array: [0; 4],
    };
    let common = unsafe { IdSet::new(&raw mut common) };
    common.init();
    unsafe {
        let first = ix((*inner(x)).i_ptr[0]);
        let last = ix((*inner(x)).i_ptr[(*x).n as usize]);
        intersect_common(&common, &first, &last);
        if !common.is_empty() {
            for i in 0..=(*x).n as usize {
                intersect_sub(&ix((*inner(x)).i_ptr[i]), &common);
            }
            intersect_add(&ix(x), &common);
        }
        xfree(common.take_heap());
    }
}

#[inline]
pub unsafe extern "C" fn marktree_putp_aux(
    mut b: *mut MarkTree,
    mut x: *mut MTNode,
    mut k: MTKey,
    meta_inc: &MetaCount,
) {
    let mut i: ::core::ffi::c_int = find_key(node_keys(x), k).0 + 1 as ::core::ffi::c_int;
    if (*x).level as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
        if i as int32_t != (*x).n {
            memmove(
                (&raw mut (*x).key as *mut MTKey).offset((i + 1 as ::core::ffi::c_int) as isize)
                    as *mut ::core::ffi::c_void,
                (&raw mut (*x).key as *mut MTKey).offset(i as isize) as *const ::core::ffi::c_void,
                (((*x).n - i as int32_t) as size_t).wrapping_mul(::core::mem::size_of::<MTKey>()),
            );
        }
        (*x).key[i as usize] = k;
        refkey(b, x, i);
        (*x).n += 1;
    } else {
        if (*(*inner(x)).i_ptr[i as usize]).n
            == 2 as int32_t * MT_BRANCH_FACTOR as ::core::ffi::c_int as int32_t - 1 as int32_t
        {
            split_node(b, x, i, k);
            if key_cmp(k, (*x).key[i as usize]) > 0 as ::core::ffi::c_int {
                i += 1;
            }
        }
        if i > 0 as ::core::ffi::c_int {
            relative(
                (*x).key[(i - 1 as ::core::ffi::c_int) as usize].pos,
                &mut k.pos,
            );
        }
        marktree_putp_aux(b, (*inner(x)).i_ptr[i as usize], k, meta_inc);
        meta_add(&mut (*inner(x)).i_meta[i as usize], meta_inc);
    };
}
