pub mod key;
pub mod meta;
pub mod node;

use crate::src::nvim::api::private::helpers::ga_take_string;
use crate::src::nvim::garray::{ga_concat, ga_init};
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::map::{
    map_del_uint64_t_ptr_t, map_put_ref_ptr_t_ptr_t, map_put_ref_uint64_t_MTDamagePair,
    mh_get_ptr_t,
};
use crate::src::nvim::marktree::key::*;
use crate::src::nvim::marktree::meta::*;
use crate::src::nvim::marktree::node::*;
use crate::src::nvim::memory::{xfree, xmalloc, xmemdup, xrealloc};
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
#[inline(always)]
unsafe extern "C" fn _memcpy_free(
    dest: *mut ::core::ffi::c_void,
    src: *mut ::core::ffi::c_void,
    size: size_t,
) -> *mut ::core::ffi::c_void {
    memcpy(dest, src, size);
    let mut ptr_: *mut *mut ::core::ffi::c_void = &raw const src as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return dest;
}
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
#[inline]
unsafe extern "C" fn split_node(
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
    if (*z).intersect.capacity < (*y).intersect.size {
        (*z).intersect.capacity = if (*y).intersect.size
            > ::core::mem::size_of::<[uint64_t; 4]>()
                .wrapping_div(::core::mem::size_of::<uint64_t>())
                .wrapping_div(
                    (::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_rem(::core::mem::size_of::<uint64_t>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            (*y).intersect.size
        } else {
            ::core::mem::size_of::<[uint64_t; 4]>()
                .wrapping_div(::core::mem::size_of::<uint64_t>())
                .wrapping_div(
                    (::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_rem(::core::mem::size_of::<uint64_t>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        (*z).intersect.items = (if (*z).intersect.capacity
            == ::core::mem::size_of::<[uint64_t; 4]>()
                .wrapping_div(::core::mem::size_of::<uint64_t>())
                .wrapping_div(
                    (::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_rem(::core::mem::size_of::<uint64_t>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if (*z).intersect.items == &raw mut (*z).intersect.init_array as *mut uint64_t {
                (*z).intersect.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut (*z).intersect.init_array as *mut uint64_t as *mut ::core::ffi::c_void,
                    (*z).intersect.items as *mut ::core::ffi::c_void,
                    (*z).intersect
                        .size
                        .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                )
            }
        } else {
            if (*z).intersect.items == &raw mut (*z).intersect.init_array as *mut uint64_t {
                memcpy(
                    xmalloc(
                        (*z).intersect
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    ),
                    (*z).intersect.items as *const ::core::ffi::c_void,
                    (*z).intersect
                        .size
                        .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                )
            } else {
                xrealloc(
                    (*z).intersect.items as *mut ::core::ffi::c_void,
                    (*z).intersect
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                )
            }
        }) as *mut uint64_t;
    }
    (*z).intersect.size = (*y).intersect.size;
    memcpy(
        (*z).intersect.items as *mut ::core::ffi::c_void,
        (*y).intersect.items as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>().wrapping_mul((*y).intersect.size),
    );
    if (*y).level == 0 {
        let mut pi: uint64_t = pseudo_index(y, 0 as ::core::ffi::c_int);
        let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while j < MT_BRANCH_FACTOR as ::core::ffi::c_int {
            let mut k: MTKey = (*y).key[j as usize];
            let mut pi_end: uint64_t = pseudo_index_for_id(b, mt_lookup_id(k.ns, k.id, true), true);
            if mt_start(k) as ::core::ffi::c_int != 0
                && pi_end > pi
                && mt_lookup_key(k) != last_start
            {
                intersect_node(b, z, mt_lookup_id(k.ns, k.id, false));
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
            let mut pi_start: uint64_t =
                pseudo_index_for_id(b, mt_lookup_id(k_0.ns, k_0.id, false), true);
            if mt_end(k_0) as ::core::ffi::c_int != 0 && pi_start > 0 as uint64_t && pi_start < pi {
                intersect_node(b, y, mt_lookup_id(k_0.ns, k_0.id, false));
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
#[inline]
unsafe extern "C" fn marktree_putp_aux(
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
unsafe extern "C" fn intersection_has(mut x: *mut Intersection, mut id: uint64_t) -> bool {
    let mut i: size_t = 0 as size_t;
    while i < (*x).size {
        if *(*x).items.offset(i as isize) == id {
            return true;
        } else if *(*x).items.offset(i as isize) >= id {
            return false;
        }
        i = i.wrapping_add(1);
    }
    return false;
}
unsafe extern "C" fn intersect_node(mut _b: *mut MarkTree, mut x: *mut MTNode, mut id: uint64_t) {
    assert!(
        id & 1 as ::core::ffi::c_int as uint64_t == 0,
        "!(id & MARKTREE_END_FLAG)"
    );
    if (*x).intersect.size == (*x).intersect.capacity {
        (*x).intersect.capacity = if (*x).intersect.capacity << 1 as ::core::ffi::c_int
            > ::core::mem::size_of::<[uint64_t; 4]>()
                .wrapping_div(::core::mem::size_of::<uint64_t>())
                .wrapping_div(
                    (::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_rem(::core::mem::size_of::<uint64_t>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            (*x).intersect.capacity << 1 as ::core::ffi::c_int
        } else {
            ::core::mem::size_of::<[uint64_t; 4]>()
                .wrapping_div(::core::mem::size_of::<uint64_t>())
                .wrapping_div(
                    (::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_rem(::core::mem::size_of::<uint64_t>())
                        == 0) as ::core::ffi::c_int as size_t,
                )
        };
        (*x).intersect.items = (if (*x).intersect.capacity
            == ::core::mem::size_of::<[uint64_t; 4]>()
                .wrapping_div(::core::mem::size_of::<uint64_t>())
                .wrapping_div(
                    (::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_rem(::core::mem::size_of::<uint64_t>())
                        == 0) as ::core::ffi::c_int as usize,
                ) {
            if (*x).intersect.items == &raw mut (*x).intersect.init_array as *mut uint64_t {
                (*x).intersect.items as *mut ::core::ffi::c_void
            } else {
                _memcpy_free(
                    &raw mut (*x).intersect.init_array as *mut uint64_t as *mut ::core::ffi::c_void,
                    (*x).intersect.items as *mut ::core::ffi::c_void,
                    (*x).intersect
                        .size
                        .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                )
            }
        } else {
            if (*x).intersect.items == &raw mut (*x).intersect.init_array as *mut uint64_t {
                memcpy(
                    xmalloc(
                        (*x).intersect
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    ),
                    (*x).intersect.items as *const ::core::ffi::c_void,
                    (*x).intersect
                        .size
                        .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                )
            } else {
                xrealloc(
                    (*x).intersect.items as *mut ::core::ffi::c_void,
                    (*x).intersect
                        .capacity
                        .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                )
            }
        }) as *mut uint64_t;
    } else {
    };
    (*x).intersect.size = (*x).intersect.size.wrapping_add(1);
    let mut i: ssize_t = (*x).intersect.size as ssize_t - 1 as ssize_t;
    while i >= 0 as ssize_t {
        if i > 0 as ssize_t && *(*x).intersect.items.offset((i - 1 as ssize_t) as isize) > id {
            *(*x).intersect.items.offset(i as isize) =
                *(*x).intersect.items.offset((i - 1 as ssize_t) as isize);
            i -= 1;
        } else {
            *(*x).intersect.items.offset(i as isize) = id;
            break;
        }
    }
}
unsafe extern "C" fn unintersect_node(
    mut _b: *mut MarkTree,
    mut x: *mut MTNode,
    mut id: uint64_t,
    mut strict: bool,
) {
    assert!(
        id & 1 as ::core::ffi::c_int as uint64_t == 0,
        "!(id & MARKTREE_END_FLAG)"
    );
    let mut seen: bool = false;
    let mut i: size_t = 0;
    i = 0 as size_t;
    while i < (*x).intersect.size {
        if *(*x).intersect.items.offset(i as isize) < id {
            i = i.wrapping_add(1);
        } else {
            if *(*x).intersect.items.offset(i as isize) != id {
                break;
            }
            seen = true;
            break;
        }
    }
    if strict {
        assert!(seen, "seen");
    }
    if seen {
        if i < (*x).intersect.size.wrapping_sub(1 as size_t) {
            memmove(
                (*x).intersect.items.offset(i as isize) as *mut ::core::ffi::c_void,
                (*x).intersect
                    .items
                    .offset(i.wrapping_add(1 as size_t) as isize)
                    as *const ::core::ffi::c_void,
                (*x).intersect
                    .size
                    .wrapping_sub(i)
                    .wrapping_sub(1 as size_t)
                    .wrapping_mul(::core::mem::size_of::<uint64_t>()),
            );
        }
        (*x).intersect.size = (*x).intersect.size.wrapping_sub(1);
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
                    unintersect_node(b, x, id, true);
                } else {
                    intersect_node(b, x, id);
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
                if intersection_has(
                    &raw mut (**(&raw mut (*inner(p)).i_ptr as *mut *mut MTNode)
                        .offset(0 as ::core::ffi::c_int as isize))
                    .intersect,
                    start_id,
                ) {
                    let mut last: ::core::ffi::c_int = if lnode != x {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                    let mut k: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while (k as int32_t) < (*p).n + last as int32_t {
                        unintersect_node(b, (*inner(p)).i_ptr[k as usize], start_id, true);
                        k += 1;
                    }
                    intersect_node(b, p, start_id);
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
                intersect_node(b, x, start_id);
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
unsafe extern "C" fn intersect_merge(
    mut m: *mut Intersection,
    mut x: *mut Intersection,
    mut y: *mut Intersection,
) {
    let mut xi: size_t = 0 as size_t;
    let mut yi: size_t = 0 as size_t;
    let mut xn: size_t = 0 as size_t;
    let mut yn: size_t = 0 as size_t;
    while xi < (*x).size && yi < (*y).size {
        if *(*x).items.offset(xi as isize) == *(*y).items.offset(yi as isize) {
            if (*m).size == (*m).capacity {
                (*m).capacity = if (*m).capacity << 1 as ::core::ffi::c_int
                    > ::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_div(::core::mem::size_of::<uint64_t>())
                        .wrapping_div(
                            (::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    (*m).capacity << 1 as ::core::ffi::c_int
                } else {
                    ::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_div(::core::mem::size_of::<uint64_t>())
                        .wrapping_div(
                            (::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                == 0) as ::core::ffi::c_int as size_t,
                        )
                };
                (*m).items = (if (*m).capacity
                    == ::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_div(::core::mem::size_of::<uint64_t>())
                        .wrapping_div(
                            (::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    if (*m).items == &raw mut (*m).init_array as *mut uint64_t {
                        (*m).items as *mut ::core::ffi::c_void
                    } else {
                        _memcpy_free(
                            &raw mut (*m).init_array as *mut uint64_t as *mut ::core::ffi::c_void,
                            (*m).items as *mut ::core::ffi::c_void,
                            (*m).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        )
                    }
                } else {
                    if (*m).items == &raw mut (*m).init_array as *mut uint64_t {
                        memcpy(
                            xmalloc(
                                (*m).capacity
                                    .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                            ),
                            (*m).items as *const ::core::ffi::c_void,
                            (*m).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        )
                    } else {
                        xrealloc(
                            (*m).items as *mut ::core::ffi::c_void,
                            (*m).capacity
                                .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        )
                    }
                }) as *mut uint64_t;
            } else {
            };
            let c2rust_fresh3 = (*m).size;
            (*m).size = (*m).size.wrapping_add(1);
            *(*m).items.offset(c2rust_fresh3 as isize) = *(*x).items.offset(xi as isize);
            xi = xi.wrapping_add(1);
            yi = yi.wrapping_add(1);
        } else if *(*x).items.offset(xi as isize) < *(*y).items.offset(yi as isize) {
            let c2rust_fresh4 = xi;
            xi = xi.wrapping_add(1);
            let c2rust_fresh5 = xn;
            xn = xn.wrapping_add(1);
            *(*x).items.offset(c2rust_fresh5 as isize) = *(*x).items.offset(c2rust_fresh4 as isize);
        } else {
            let c2rust_fresh6 = yi;
            yi = yi.wrapping_add(1);
            let c2rust_fresh7 = yn;
            yn = yn.wrapping_add(1);
            *(*y).items.offset(c2rust_fresh7 as isize) = *(*y).items.offset(c2rust_fresh6 as isize);
        }
    }
    if xi < (*x).size {
        memmove(
            (*x).items.offset(xn as isize) as *mut ::core::ffi::c_void,
            (*x).items.offset(xi as isize) as *const ::core::ffi::c_void,
            ::core::mem::size_of::<uint64_t>().wrapping_mul((*x).size.wrapping_sub(xi)),
        );
        xn = xn.wrapping_add((*x).size.wrapping_sub(xi));
    }
    if yi < (*y).size {
        memmove(
            (*y).items.offset(yn as isize) as *mut ::core::ffi::c_void,
            (*y).items.offset(yi as isize) as *const ::core::ffi::c_void,
            ::core::mem::size_of::<uint64_t>().wrapping_mul((*y).size.wrapping_sub(yi)),
        );
        yn = yn.wrapping_add((*y).size.wrapping_sub(yi));
    }
    (*x).size = xn;
    (*y).size = yn;
}
unsafe extern "C" fn intersect_mov(
    mut x: *mut Intersection,
    mut y: *mut Intersection,
    mut w: *mut Intersection,
    mut d: *mut Intersection,
) {
    let mut wi: size_t = 0 as size_t;
    let mut yi: size_t = 0 as size_t;
    let mut wn: size_t = 0 as size_t;
    let mut yn: size_t = 0 as size_t;
    let mut xi: size_t = 0 as size_t;
    while wi < (*w).size || xi < (*x).size {
        if wi < (*w).size
            && (xi >= (*x).size
                || *(*x).items.offset(xi as isize) >= *(*w).items.offset(wi as isize))
        {
            if xi < (*x).size && *(*x).items.offset(xi as isize) == *(*w).items.offset(wi as isize)
            {
                xi = xi.wrapping_add(1);
            }
            while yi < (*y).size
                && *(*y).items.offset(yi as isize) < *(*w).items.offset(wi as isize)
            {
                if (*d).size == (*d).capacity {
                    (*d).capacity = if (*d).capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_div(::core::mem::size_of::<uint64_t>())
                            .wrapping_div(
                                (::core::mem::size_of::<[uint64_t; 4]>()
                                    .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        (*d).capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_div(::core::mem::size_of::<uint64_t>())
                            .wrapping_div(
                                (::core::mem::size_of::<[uint64_t; 4]>()
                                    .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    (*d).items = (if (*d).capacity
                        == ::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_div(::core::mem::size_of::<uint64_t>())
                            .wrapping_div(
                                (::core::mem::size_of::<[uint64_t; 4]>()
                                    .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if (*d).items == &raw mut (*d).init_array as *mut uint64_t {
                            (*d).items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut (*d).init_array as *mut uint64_t
                                    as *mut ::core::ffi::c_void,
                                (*d).items as *mut ::core::ffi::c_void,
                                (*d).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                            )
                        }
                    } else {
                        if (*d).items == &raw mut (*d).init_array as *mut uint64_t {
                            memcpy(
                                xmalloc(
                                    (*d).capacity
                                        .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                                ),
                                (*d).items as *const ::core::ffi::c_void,
                                (*d).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                            )
                        } else {
                            xrealloc(
                                (*d).items as *mut ::core::ffi::c_void,
                                (*d).capacity
                                    .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                            )
                        }
                    }) as *mut uint64_t;
                } else {
                };
                let c2rust_fresh8 = (*d).size;
                (*d).size = (*d).size.wrapping_add(1);
                *(*d).items.offset(c2rust_fresh8 as isize) = *(*y).items.offset(yi as isize);
                yi = yi.wrapping_add(1);
            }
            if yi < (*y).size && *(*y).items.offset(yi as isize) == *(*w).items.offset(wi as isize)
            {
                let c2rust_fresh9 = yi;
                yi = yi.wrapping_add(1);
                let c2rust_fresh10 = yn;
                yn = yn.wrapping_add(1);
                *(*y).items.offset(c2rust_fresh10 as isize) =
                    *(*y).items.offset(c2rust_fresh9 as isize);
                wi = wi.wrapping_add(1);
            } else {
                let c2rust_fresh11 = wi;
                wi = wi.wrapping_add(1);
                let c2rust_fresh12 = wn;
                wn = wn.wrapping_add(1);
                *(*w).items.offset(c2rust_fresh12 as isize) =
                    *(*w).items.offset(c2rust_fresh11 as isize);
            }
        } else {
            while yi < (*y).size
                && *(*y).items.offset(yi as isize) < *(*x).items.offset(xi as isize)
            {
                if (*d).size == (*d).capacity {
                    (*d).capacity = if (*d).capacity << 1 as ::core::ffi::c_int
                        > ::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_div(::core::mem::size_of::<uint64_t>())
                            .wrapping_div(
                                (::core::mem::size_of::<[uint64_t; 4]>()
                                    .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        (*d).capacity << 1 as ::core::ffi::c_int
                    } else {
                        ::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_div(::core::mem::size_of::<uint64_t>())
                            .wrapping_div(
                                (::core::mem::size_of::<[uint64_t; 4]>()
                                    .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                    == 0) as ::core::ffi::c_int
                                    as size_t,
                            )
                    };
                    (*d).items = (if (*d).capacity
                        == ::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_div(::core::mem::size_of::<uint64_t>())
                            .wrapping_div(
                                (::core::mem::size_of::<[uint64_t; 4]>()
                                    .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                    == 0) as ::core::ffi::c_int
                                    as usize,
                            ) {
                        if (*d).items == &raw mut (*d).init_array as *mut uint64_t {
                            (*d).items as *mut ::core::ffi::c_void
                        } else {
                            _memcpy_free(
                                &raw mut (*d).init_array as *mut uint64_t
                                    as *mut ::core::ffi::c_void,
                                (*d).items as *mut ::core::ffi::c_void,
                                (*d).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                            )
                        }
                    } else {
                        if (*d).items == &raw mut (*d).init_array as *mut uint64_t {
                            memcpy(
                                xmalloc(
                                    (*d).capacity
                                        .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                                ),
                                (*d).items as *const ::core::ffi::c_void,
                                (*d).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                            )
                        } else {
                            xrealloc(
                                (*d).items as *mut ::core::ffi::c_void,
                                (*d).capacity
                                    .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                            )
                        }
                    }) as *mut uint64_t;
                } else {
                };
                let c2rust_fresh13 = (*d).size;
                (*d).size = (*d).size.wrapping_add(1);
                *(*d).items.offset(c2rust_fresh13 as isize) = *(*y).items.offset(yi as isize);
                yi = yi.wrapping_add(1);
            }
            if yi < (*y).size && *(*y).items.offset(yi as isize) == *(*x).items.offset(xi as isize)
            {
                let c2rust_fresh14 = yi;
                yi = yi.wrapping_add(1);
                let c2rust_fresh15 = yn;
                yn = yn.wrapping_add(1);
                *(*y).items.offset(c2rust_fresh15 as isize) =
                    *(*y).items.offset(c2rust_fresh14 as isize);
                xi = xi.wrapping_add(1);
            } else {
                if wi == wn {
                    let mut n: size_t = (*w).size.wrapping_sub(wn);
                    if (*w).size == (*w).capacity {
                        (*w).capacity = if (*w).capacity << 1 as ::core::ffi::c_int
                            > ::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_div(::core::mem::size_of::<uint64_t>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[uint64_t; 4]>()
                                        .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*w).capacity << 1 as ::core::ffi::c_int
                        } else {
                            ::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_div(::core::mem::size_of::<uint64_t>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[uint64_t; 4]>()
                                        .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*w).items = (if (*w).capacity
                            == ::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_div(::core::mem::size_of::<uint64_t>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[uint64_t; 4]>()
                                        .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*w).items == &raw mut (*w).init_array as *mut uint64_t {
                                (*w).items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                    &raw mut (*w).init_array as *mut uint64_t
                                        as *mut ::core::ffi::c_void,
                                    (*w).items as *mut ::core::ffi::c_void,
                                    (*w).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                                )
                            }
                        } else {
                            if (*w).items == &raw mut (*w).init_array as *mut uint64_t {
                                memcpy(
                                    xmalloc(
                                        (*w).capacity
                                            .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                                    ),
                                    (*w).items as *const ::core::ffi::c_void,
                                    (*w).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                                )
                            } else {
                                xrealloc(
                                    (*w).items as *mut ::core::ffi::c_void,
                                    (*w).capacity
                                        .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                                )
                            }
                        }) as *mut uint64_t;
                    } else {
                    };
                    let _ = *w;
                    (*w).size = (*w).size.wrapping_add(1);
                    if n > 0 as size_t {
                        memmove(
                            (*w).items.offset(wn.wrapping_add(1 as size_t) as isize)
                                as *mut ::core::ffi::c_void,
                            (*w).items.offset(wn as isize) as *const ::core::ffi::c_void,
                            n.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        );
                    }
                    *(*w).items.offset(wi as isize) = *(*x).items.offset(xi as isize);
                    wn = wn.wrapping_add(1);
                    wi = wi.wrapping_add(1);
                } else {
                    assert!(wn < wi, "wn < wi");
                    let c2rust_fresh16 = wn;
                    wn = wn.wrapping_add(1);
                    *(*w).items.offset(c2rust_fresh16 as isize) = *(*x).items.offset(xi as isize);
                }
                xi = xi.wrapping_add(1);
            }
        }
    }
    if yi < (*y).size {
        let mut n_0: size_t = (*y).size.wrapping_sub(yi);
        if (*d).capacity < (*d).size.wrapping_add(n_0) {
            (*d).capacity = (*d).size.wrapping_add(n_0);
            (*d).capacity = (*d).capacity.wrapping_sub(1);
            (*d).capacity |= (*d).capacity >> 1 as ::core::ffi::c_int;
            (*d).capacity |= (*d).capacity >> 2 as ::core::ffi::c_int;
            (*d).capacity |= (*d).capacity >> 4 as ::core::ffi::c_int;
            (*d).capacity |= (*d).capacity >> 8 as ::core::ffi::c_int;
            (*d).capacity |= (*d).capacity >> 16 as ::core::ffi::c_int;
            (*d).capacity = (*d).capacity.wrapping_add(1);
            (*d).capacity = if (*d).capacity
                > ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*d).capacity
            } else {
                ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*d).items = (if (*d).capacity
                == ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*d).items == &raw mut (*d).init_array as *mut uint64_t {
                    (*d).items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*d).init_array as *mut uint64_t as *mut ::core::ffi::c_void,
                        (*d).items as *mut ::core::ffi::c_void,
                        (*d).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                }
            } else {
                if (*d).items == &raw mut (*d).init_array as *mut uint64_t {
                    memcpy(
                        xmalloc(
                            (*d).capacity
                                .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        ),
                        (*d).items as *const ::core::ffi::c_void,
                        (*d).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                } else {
                    xrealloc(
                        (*d).items as *mut ::core::ffi::c_void,
                        (*d).capacity
                            .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                }
            }) as *mut uint64_t;
        }
        memcpy(
            (*d).items.offset((*d).size as isize) as *mut ::core::ffi::c_void,
            (*y).items.offset(yi as isize) as *const ::core::ffi::c_void,
            n_0.wrapping_mul(::core::mem::size_of::<uint64_t>()),
        );
        (*d).size = (*d).size.wrapping_add(n_0);
    }
    (*w).size = wn;
    (*y).size = yn;
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
    let mut xi: Intersection = Intersection {
        size: nx,
        capacity: 0,
        items: x as *mut uint64_t,
        init_array: [0; 4],
    };
    let mut yi: Intersection = Intersection {
        size: ny,
        capacity: 0,
        items: y as *mut uint64_t,
        init_array: [0; 4],
    };
    let mut w: Intersection = Intersection {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<uint64_t>(),
        init_array: [0; 4],
    };
    w.capacity = ::core::mem::size_of::<[uint64_t; 4]>()
        .wrapping_div(::core::mem::size_of::<uint64_t>())
        .wrapping_div(
            (::core::mem::size_of::<[uint64_t; 4]>()
                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    w.size = 0 as size_t;
    w.items = &raw mut w.init_array as *mut uint64_t;
    let mut i: size_t = 0 as size_t;
    while i < nwin {
        if w.size == w.capacity {
            w.capacity = if w.capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                w.capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            w.items = (if w.capacity
                == ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if w.items == &raw mut w.init_array as *mut uint64_t {
                    w.items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut w.init_array as *mut uint64_t as *mut ::core::ffi::c_void,
                        w.items as *mut ::core::ffi::c_void,
                        w.size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                }
            } else {
                if w.items == &raw mut w.init_array as *mut uint64_t {
                    memcpy(
                        xmalloc(w.capacity.wrapping_mul(::core::mem::size_of::<uint64_t>())),
                        w.items as *const ::core::ffi::c_void,
                        w.size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                } else {
                    xrealloc(
                        w.items as *mut ::core::ffi::c_void,
                        w.capacity.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                }
            }) as *mut uint64_t;
        } else {
        };
        let c2rust_fresh17 = w.size;
        w.size = w.size.wrapping_add(1);
        *w.items.offset(c2rust_fresh17 as isize) = *win.offset(i as isize);
        i = i.wrapping_add(1);
    }
    let mut d: Intersection = Intersection {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<uint64_t>(),
        init_array: [0; 4],
    };
    d.capacity = ::core::mem::size_of::<[uint64_t; 4]>()
        .wrapping_div(::core::mem::size_of::<uint64_t>())
        .wrapping_div(
            (::core::mem::size_of::<[uint64_t; 4]>()
                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    d.size = 0 as size_t;
    d.items = &raw mut d.init_array as *mut uint64_t;
    intersect_mov(&raw mut xi, &raw mut yi, &raw mut w, &raw mut d);
    if w.size > *nwout || d.size > *ndout {
        return false;
    }
    memcpy(
        wout as *mut ::core::ffi::c_void,
        w.items as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>().wrapping_mul(w.size),
    );
    *nwout = w.size;
    memcpy(
        dout as *mut ::core::ffi::c_void,
        d.items as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>().wrapping_mul(d.size),
    );
    *ndout = d.size;
    return true;
}
unsafe extern "C" fn intersect_common(
    mut i: *mut Intersection,
    mut x: *mut Intersection,
    mut y: *mut Intersection,
) {
    let mut xi: size_t = 0 as size_t;
    let mut yi: size_t = 0 as size_t;
    while xi < (*x).size && yi < (*y).size {
        if *(*x).items.offset(xi as isize) == *(*y).items.offset(yi as isize) {
            if (*i).size == (*i).capacity {
                (*i).capacity = if (*i).capacity << 1 as ::core::ffi::c_int
                    > ::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_div(::core::mem::size_of::<uint64_t>())
                        .wrapping_div(
                            (::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    (*i).capacity << 1 as ::core::ffi::c_int
                } else {
                    ::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_div(::core::mem::size_of::<uint64_t>())
                        .wrapping_div(
                            (::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                == 0) as ::core::ffi::c_int as size_t,
                        )
                };
                (*i).items = (if (*i).capacity
                    == ::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_div(::core::mem::size_of::<uint64_t>())
                        .wrapping_div(
                            (::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    if (*i).items == &raw mut (*i).init_array as *mut uint64_t {
                        (*i).items as *mut ::core::ffi::c_void
                    } else {
                        _memcpy_free(
                            &raw mut (*i).init_array as *mut uint64_t as *mut ::core::ffi::c_void,
                            (*i).items as *mut ::core::ffi::c_void,
                            (*i).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        )
                    }
                } else {
                    if (*i).items == &raw mut (*i).init_array as *mut uint64_t {
                        memcpy(
                            xmalloc(
                                (*i).capacity
                                    .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                            ),
                            (*i).items as *const ::core::ffi::c_void,
                            (*i).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        )
                    } else {
                        xrealloc(
                            (*i).items as *mut ::core::ffi::c_void,
                            (*i).capacity
                                .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        )
                    }
                }) as *mut uint64_t;
            } else {
            };
            let c2rust_fresh2 = (*i).size;
            (*i).size = (*i).size.wrapping_add(1);
            *(*i).items.offset(c2rust_fresh2 as isize) = *(*x).items.offset(xi as isize);
            xi = xi.wrapping_add(1);
            yi = yi.wrapping_add(1);
        } else if *(*x).items.offset(xi as isize) < *(*y).items.offset(yi as isize) {
            xi = xi.wrapping_add(1);
        } else {
            yi = yi.wrapping_add(1);
        }
    }
}
unsafe extern "C" fn intersect_add(mut x: *mut Intersection, mut y: *mut Intersection) {
    let mut xi: size_t = 0 as size_t;
    let mut yi: size_t = 0 as size_t;
    while xi < (*x).size && yi < (*y).size {
        if *(*x).items.offset(xi as isize) == *(*y).items.offset(yi as isize) {
            xi = xi.wrapping_add(1);
            yi = yi.wrapping_add(1);
        } else if *(*y).items.offset(yi as isize) < *(*x).items.offset(xi as isize) {
            let mut n: size_t = (*x).size.wrapping_sub(xi);
            if (*x).size == (*x).capacity {
                (*x).capacity = if (*x).capacity << 1 as ::core::ffi::c_int
                    > ::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_div(::core::mem::size_of::<uint64_t>())
                        .wrapping_div(
                            (::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    (*x).capacity << 1 as ::core::ffi::c_int
                } else {
                    ::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_div(::core::mem::size_of::<uint64_t>())
                        .wrapping_div(
                            (::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                == 0) as ::core::ffi::c_int as size_t,
                        )
                };
                (*x).items = (if (*x).capacity
                    == ::core::mem::size_of::<[uint64_t; 4]>()
                        .wrapping_div(::core::mem::size_of::<uint64_t>())
                        .wrapping_div(
                            (::core::mem::size_of::<[uint64_t; 4]>()
                                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                                == 0) as ::core::ffi::c_int as usize,
                        ) {
                    if (*x).items == &raw mut (*x).init_array as *mut uint64_t {
                        (*x).items as *mut ::core::ffi::c_void
                    } else {
                        _memcpy_free(
                            &raw mut (*x).init_array as *mut uint64_t as *mut ::core::ffi::c_void,
                            (*x).items as *mut ::core::ffi::c_void,
                            (*x).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        )
                    }
                } else {
                    if (*x).items == &raw mut (*x).init_array as *mut uint64_t {
                        memcpy(
                            xmalloc(
                                (*x).capacity
                                    .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                            ),
                            (*x).items as *const ::core::ffi::c_void,
                            (*x).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        )
                    } else {
                        xrealloc(
                            (*x).items as *mut ::core::ffi::c_void,
                            (*x).capacity
                                .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        )
                    }
                }) as *mut uint64_t;
            } else {
            };
            let _ = *x;
            (*x).size = (*x).size.wrapping_add(1);
            memmove(
                (*x).items.offset(xi.wrapping_add(1 as size_t) as isize)
                    as *mut ::core::ffi::c_void,
                (*x).items.offset(xi as isize) as *const ::core::ffi::c_void,
                n.wrapping_mul(::core::mem::size_of::<uint64_t>()),
            );
            *(*x).items.offset(xi as isize) = *(*y).items.offset(yi as isize);
            xi = xi.wrapping_add(1);
            yi = yi.wrapping_add(1);
        } else {
            xi = xi.wrapping_add(1);
        }
    }
    if yi < (*y).size {
        let mut n_0: size_t = (*y).size.wrapping_sub(yi);
        if (*x).capacity < (*x).size.wrapping_add(n_0) {
            (*x).capacity = (*x).size.wrapping_add(n_0);
            (*x).capacity = (*x).capacity.wrapping_sub(1);
            (*x).capacity |= (*x).capacity >> 1 as ::core::ffi::c_int;
            (*x).capacity |= (*x).capacity >> 2 as ::core::ffi::c_int;
            (*x).capacity |= (*x).capacity >> 4 as ::core::ffi::c_int;
            (*x).capacity |= (*x).capacity >> 8 as ::core::ffi::c_int;
            (*x).capacity |= (*x).capacity >> 16 as ::core::ffi::c_int;
            (*x).capacity = (*x).capacity.wrapping_add(1);
            (*x).capacity = if (*x).capacity
                > ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*x).capacity
            } else {
                ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*x).items = (if (*x).capacity
                == ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*x).items == &raw mut (*x).init_array as *mut uint64_t {
                    (*x).items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*x).init_array as *mut uint64_t as *mut ::core::ffi::c_void,
                        (*x).items as *mut ::core::ffi::c_void,
                        (*x).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                }
            } else {
                if (*x).items == &raw mut (*x).init_array as *mut uint64_t {
                    memcpy(
                        xmalloc(
                            (*x).capacity
                                .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        ),
                        (*x).items as *const ::core::ffi::c_void,
                        (*x).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                } else {
                    xrealloc(
                        (*x).items as *mut ::core::ffi::c_void,
                        (*x).capacity
                            .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                }
            }) as *mut uint64_t;
        }
        memcpy(
            (*x).items.offset((*x).size as isize) as *mut ::core::ffi::c_void,
            (*y).items.offset(yi as isize) as *const ::core::ffi::c_void,
            n_0.wrapping_mul(::core::mem::size_of::<uint64_t>()),
        );
        (*x).size = (*x).size.wrapping_add(n_0);
    }
}
unsafe extern "C" fn intersect_sub(mut x: *mut Intersection, mut y: *mut Intersection) {
    let mut xi: size_t = 0 as size_t;
    let mut yi: size_t = 0 as size_t;
    let mut xn: size_t = 0 as size_t;
    while xi < (*x).size && yi < (*y).size {
        if *(*x).items.offset(xi as isize) == *(*y).items.offset(yi as isize) {
            xi = xi.wrapping_add(1);
            yi = yi.wrapping_add(1);
        } else if *(*x).items.offset(xi as isize) < *(*y).items.offset(yi as isize) {
            let c2rust_fresh0 = xi;
            xi = xi.wrapping_add(1);
            let c2rust_fresh1 = xn;
            xn = xn.wrapping_add(1);
            *(*x).items.offset(c2rust_fresh1 as isize) = *(*x).items.offset(c2rust_fresh0 as isize);
        } else {
            yi = yi.wrapping_add(1);
        }
    }
    if xi < (*x).size {
        let mut n: size_t = (*x).size.wrapping_sub(xi);
        if xn < xi {
            memmove(
                (*x).items.offset(xn as isize) as *mut ::core::ffi::c_void,
                (*x).items.offset(xi as isize) as *const ::core::ffi::c_void,
                n.wrapping_mul(::core::mem::size_of::<uint64_t>()),
            );
        }
        xn = xn.wrapping_add(n);
    }
    (*x).size = xn;
}
unsafe extern "C" fn bubble_up(mut x: *mut MTNode) {
    let mut xi: Intersection = Intersection {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<uint64_t>(),
        init_array: [0; 4],
    };
    xi.capacity = ::core::mem::size_of::<[uint64_t; 4]>()
        .wrapping_div(::core::mem::size_of::<uint64_t>())
        .wrapping_div(
            (::core::mem::size_of::<[uint64_t; 4]>()
                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    xi.size = 0 as size_t;
    xi.items = &raw mut xi.init_array as *mut uint64_t;
    intersect_common(
        &raw mut xi,
        &raw mut (**(&raw mut (*inner(x)).i_ptr as *mut *mut MTNode)
            .offset(0 as ::core::ffi::c_int as isize))
        .intersect,
        &raw mut (**(&raw mut (*inner(x)).i_ptr as *mut *mut MTNode).offset((*x).n as isize))
            .intersect,
    );
    if xi.size != 0 {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i as int32_t) < (*x).n + 1 as int32_t {
            intersect_sub(
                &raw mut (**(&raw mut (*inner(x)).i_ptr as *mut *mut MTNode).offset(i as isize))
                    .intersect,
                &raw mut xi,
            );
            i += 1;
        }
        intersect_add(&raw mut (*x).intersect, &raw mut xi);
    }
    if xi.items != &raw mut xi.init_array as *mut uint64_t {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut xi.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
}
unsafe extern "C" fn merge_node(
    mut b: *mut MarkTree,
    mut p: *mut MTNode,
    mut i: ::core::ffi::c_int,
) -> *mut MTNode {
    let mut x: *mut MTNode = (*inner(p)).i_ptr[i as usize];
    let mut y: *mut MTNode = (*inner(p)).i_ptr[(i + 1 as ::core::ffi::c_int) as usize];
    let mut mi: Intersection = Intersection {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut::<uint64_t>(),
        init_array: [0; 4],
    };
    mi.capacity = ::core::mem::size_of::<[uint64_t; 4]>()
        .wrapping_div(::core::mem::size_of::<uint64_t>())
        .wrapping_div(
            (::core::mem::size_of::<[uint64_t; 4]>()
                .wrapping_rem(::core::mem::size_of::<uint64_t>())
                == 0) as ::core::ffi::c_int as usize,
        ) as size_t;
    mi.size = 0 as size_t;
    mi.items = &raw mut mi.init_array as *mut uint64_t;
    intersect_merge(
        &raw mut mi,
        &raw mut (*x).intersect,
        &raw mut (*y).intersect,
    );
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
            let mut idx: size_t = 0 as size_t;
            while idx < (*x).intersect.size {
                intersect_node(
                    b,
                    (*inner(x)).i_ptr[k_0 as usize],
                    *(*x).intersect.items.offset(idx as isize),
                );
                idx = idx.wrapping_add(1);
            }
            k_0 += 1;
        }
        let mut ky: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (ky as int32_t) < (*y).n + 1 as int32_t {
            let mut k_1: ::core::ffi::c_int =
                (*x).n as ::core::ffi::c_int + ky + 1 as ::core::ffi::c_int;
            (*(*inner(x)).i_ptr[k_1 as usize]).parent = x;
            (*(*inner(x)).i_ptr[k_1 as usize]).p_idx = k_1 as int16_t;
            let mut idx_0: size_t = 0 as size_t;
            while idx_0 < (*y).intersect.size {
                intersect_node(
                    b,
                    (*inner(x)).i_ptr[k_1 as usize],
                    *(*y).intersect.items.offset(idx_0 as isize),
                );
                idx_0 = idx_0.wrapping_add(1);
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
    if (*x).intersect.items != &raw mut (*x).intersect.init_array as *mut uint64_t {
        let mut ptr_: *mut *mut ::core::ffi::c_void =
            &raw mut (*x).intersect.items as *mut *mut ::core::ffi::c_void;
        xfree(*ptr_);
        *ptr_ = NULL;
        let _ = *ptr_;
    }
    kvi_move(&raw mut (*x).intersect, &raw mut mi);
    return x;
}
pub unsafe extern "C" fn kvi_move(mut dest: *mut Intersection, mut src: *mut Intersection) {
    (*dest).size = (*src).size;
    (*dest).capacity = (*src).capacity;
    if (*src).items == &raw mut (*src).init_array as *mut uint64_t {
        memcpy(
            &raw mut (*dest).init_array as *mut uint64_t as *mut ::core::ffi::c_void,
            &raw mut (*src).init_array as *mut uint64_t as *const ::core::ffi::c_void,
            (*src).size.wrapping_mul(::core::mem::size_of::<uint64_t>()),
        );
        (*dest).items = &raw mut (*dest).init_array as *mut uint64_t;
    } else {
        (*dest).items = (*src).items;
    };
}
unsafe extern "C" fn pivot_right(
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
        let mut d: Intersection = Intersection {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<uint64_t>(),
            init_array: [0; 4],
        };
        d.capacity = ::core::mem::size_of::<[uint64_t; 4]>()
            .wrapping_div(::core::mem::size_of::<uint64_t>())
            .wrapping_div(
                (::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_rem(::core::mem::size_of::<uint64_t>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as size_t;
        d.size = 0 as size_t;
        d.items = &raw mut d.init_array as *mut uint64_t;
        intersect_mov(
            &raw mut (*x).intersect,
            &raw mut (*y).intersect,
            &raw mut (**(&raw mut (*inner(y)).i_ptr as *mut *mut MTNode)
                .offset(0 as ::core::ffi::c_int as isize))
            .intersect,
            &raw mut d,
        );
        if d.size != 0 {
            let mut yi: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while (yi as int32_t) < (*y).n + 1 as int32_t {
                intersect_add(
                    &raw mut (**(&raw mut (*inner(y)).i_ptr as *mut *mut MTNode)
                        .offset(yi as isize))
                    .intersect,
                    &raw mut d,
                );
                yi += 1;
            }
        }
        if d.items != &raw mut d.init_array as *mut uint64_t {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut d.items as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
        bubble_up(x);
    } else {
        if mt_end((*p).key[i as usize]) {
            let mut pi: uint64_t = pseudo_index(x, 0 as ::core::ffi::c_int);
            let mut start_id: uint64_t = mt_lookup_key_side((*p).key[i as usize], false);
            let mut pi_start: uint64_t = pseudo_index_for_id(b, start_id, true);
            if pi_start > 0 as uint64_t && pi_start < pi {
                intersect_node(b, x, start_id);
            }
        }
        if mt_start((*y).key[0 as ::core::ffi::c_int as usize]) {
            unintersect_node(
                b,
                y,
                mt_lookup_key((*y).key[0 as ::core::ffi::c_int as usize]),
                false,
            );
        }
    };
}
unsafe extern "C" fn pivot_left(
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
        let mut d: Intersection = Intersection {
            size: 0,
            capacity: 0,
            items: ::core::ptr::null_mut::<uint64_t>(),
            init_array: [0; 4],
        };
        d.capacity = ::core::mem::size_of::<[uint64_t; 4]>()
            .wrapping_div(::core::mem::size_of::<uint64_t>())
            .wrapping_div(
                (::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_rem(::core::mem::size_of::<uint64_t>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as size_t;
        d.size = 0 as size_t;
        d.items = &raw mut d.init_array as *mut uint64_t;
        intersect_mov(
            &raw mut (*y).intersect,
            &raw mut (*x).intersect,
            &raw mut (**(&raw mut (*inner(x)).i_ptr as *mut *mut MTNode).offset((*x).n as isize))
                .intersect,
            &raw mut d,
        );
        if d.size != 0 {
            let mut xi: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while (xi as int32_t) < (*x).n {
                intersect_add(
                    &raw mut (**(&raw mut (*inner(x)).i_ptr as *mut *mut MTNode)
                        .offset(xi as isize))
                    .intersect,
                    &raw mut d,
                );
                xi += 1;
            }
        }
        if d.items != &raw mut d.init_array as *mut uint64_t {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut d.items as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL;
            let _ = *ptr_;
        }
        bubble_up(y);
    } else {
        if mt_start((*p).key[i as usize]) {
            let mut pi: uint64_t = pseudo_index(y, 0 as ::core::ffi::c_int);
            let mut end_id: uint64_t = mt_lookup_key_side((*p).key[i as usize], true);
            let mut pi_end: uint64_t = pseudo_index_for_id(b, end_id, true);
            if pi_end > pi {
                intersect_node(b, y, mt_lookup_key((*p).key[i as usize]));
            }
        }
        if mt_end((*x).key[((*x).n - 1 as int32_t) as usize]) {
            unintersect_node(
                b,
                x,
                mt_lookup_key_side((*x).key[((*x).n - 1 as int32_t) as usize], false),
                false,
            );
        }
    };
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
unsafe extern "C" fn marktree_itr_next_skip(
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
unsafe extern "C" fn marktree_itr_check_filter(
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
unsafe extern "C" fn itr_eq(mut itr1: *mut MarkTreeIter, mut itr2: *mut MarkTreeIter) -> bool {
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
        if (*itr).intersect_idx < (*(*itr).x).intersect.size {
            let c2rust_fresh18 = (*itr).intersect_idx;
            (*itr).intersect_idx = (*itr).intersect_idx.wrapping_add(1);
            let mut id: uint64_t = *(*(*itr).x).intersect.items.offset(c2rust_fresh18 as isize);
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
unsafe extern "C" fn check_damage(
    mut _b: *mut MarkTree,
    mut damage: *mut MTDamageMap,
    mut itr1: *mut MarkTreeIter,
    mut itr2: *mut MarkTreeIter,
) {
    let start_id: uint64_t = mt_lookup_key_side((*(*itr1).x).key[(*itr1).i as usize], false);
    let mut p: *mut MTDamagePair = map_put_ref_uint64_t_MTDamagePair(
        damage as *mut Map_uint64_t_MTDamagePair,
        start_id,
        ::core::ptr::null_mut::<*mut uint64_t>(),
        ::core::ptr::null_mut::<bool>(),
    );
    let mut me: *mut MTDamage =
        if mt_end((*(*itr1).x).key[(*itr1).i as usize]) as ::core::ffi::c_int != 0 {
            &raw mut (*p).end
        } else {
            &raw mut (*p).start
        };
    assert!((*me).new.is_null(), "me->new == NULL");
    *me = MTDamage {
        old: (*itr1).x,
        new: (*itr2).x,
        old_i: (*itr1).i,
        new_i: (*itr2).i,
    };
}
unsafe extern "C" fn swap_keys(
    mut b: *mut MarkTree,
    mut itr1: *mut MarkTreeIter,
    mut itr2: *mut MarkTreeIter,
    mut damage: *mut MTDamageMap,
) {
    if (*(*itr1).x).level as ::core::ffi::c_int != 0 || (*itr1).x != (*itr2).x {
        if mt_paired((*(*itr1).x).key[(*itr1).i as usize]) {
            check_damage(b, damage, itr1, itr2);
        }
        if mt_paired((*(*itr2).x).key[(*itr2).i as usize]) {
            check_damage(b, damage, itr2, itr1);
        }
    }
    if (*itr1).x != (*itr2).x {
        let mut meta_inc_1 = meta_describe_key((*(*itr1).x).key[(*itr1).i as usize]);
        let mut meta_inc_2 = meta_describe_key((*(*itr2).x).key[(*itr2).i as usize]);
        if memcmp(
            &raw mut meta_inc_1 as *mut uint32_t as *const ::core::ffi::c_void,
            &raw mut meta_inc_2 as *mut uint32_t as *const ::core::ffi::c_void,
            ::core::mem::size_of::<[uint32_t; 5]>(),
        ) != 0 as ::core::ffi::c_int
        {
            let mut x1: *mut MTNode = (*itr1).x;
            let mut x2: *mut MTNode = (*itr2).x;
            while x1 != x2 {
                if (*x1).level as ::core::ffi::c_int <= (*x2).level as ::core::ffi::c_int {
                    meta_apply_delta(
                        &mut (*inner((*x1).parent)).i_meta[(*x1).p_idx as usize],
                        &meta_inc_2,
                        &meta_inc_1,
                    );
                    x1 = (*x1).parent;
                }
                if ((*x2).level as ::core::ffi::c_int) < (*x1).level as ::core::ffi::c_int {
                    meta_apply_delta(
                        &mut (*inner((*x2).parent)).i_meta[(*x2).p_idx as usize],
                        &meta_inc_1,
                        &meta_inc_2,
                    );
                    x2 = (*x2).parent;
                }
            }
        }
    }
    let mut key1: MTKey = (*(*itr1).x).key[(*itr1).i as usize];
    let mut key2: MTKey = (*(*itr2).x).key[(*itr2).i as usize];
    (*(*itr1).x).key[(*itr1).i as usize] = key2;
    (*(*itr1).x).key[(*itr1).i as usize].pos = key1.pos;
    (*(*itr2).x).key[(*itr2).i as usize] = key1;
    (*(*itr2).x).key[(*itr2).i as usize].pos = key2.pos;
    refkey(b, (*itr1).x, (*itr1).i);
    refkey(b, (*itr2).x, (*itr2).i);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_splice(
    mut b: *mut MarkTree,
    mut start_line: int32_t,
    mut start_col: ::core::ffi::c_int,
    mut old_extent_line: ::core::ffi::c_int,
    mut old_extent_col: ::core::ffi::c_int,
    mut new_extent_line: ::core::ffi::c_int,
    mut new_extent_col: ::core::ffi::c_int,
) -> bool {
    let mut start: MTPos = MTPos {
        row: start_line,
        col: start_col as int32_t,
    };
    let mut old_extent: MTPos = MTPos {
        row: old_extent_line as int32_t,
        col: old_extent_col as int32_t,
    };
    let mut new_extent: MTPos = MTPos {
        row: new_extent_line as int32_t,
        col: new_extent_col as int32_t,
    };
    let mut may_delete: bool = old_extent.row != 0 as int32_t || old_extent.col != 0 as int32_t;
    let mut same_line: bool = old_extent.row == 0 as int32_t && new_extent.row == 0 as int32_t;
    unrelative(start, &mut old_extent);
    unrelative(start, &mut new_extent);
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
    let mut enditr: [MarkTreeIter; 1] = [MarkTreeIter {
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
    let mut oldbase: [MTPos; 20] = [
        MTPos {
            row: 0 as int32_t,
            col: 0,
        },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
        MTPos { row: 0, col: 0 },
    ];
    marktree_itr_get_ext(
        b,
        start,
        &raw mut itr as *mut MarkTreeIter,
        false,
        true,
        &raw mut oldbase as *mut MTPos,
        ::core::ptr::null::<uint32_t>(),
    );
    if (*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        return false;
    }
    let mut delta: MTPos = MTPos {
        row: new_extent.row - old_extent.row,
        col: new_extent.col - old_extent.col,
    };
    if may_delete {
        let mut ipos: MTPos = marktree_itr_pos(&raw mut itr as *mut MarkTreeIter);
        if !pos_leq(old_extent, ipos)
            || old_extent.row == ipos.row
                && old_extent.col == ipos.col
                && !mt_right(
                    (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize],
                )
        {
            marktree_itr_get_ext(
                b,
                old_extent,
                &raw mut enditr as *mut MarkTreeIter,
                true,
                true,
                ::core::ptr::null_mut::<MTPos>(),
                ::core::ptr::null::<uint32_t>(),
            );
            assert!(
                !(*(&raw mut enditr as *mut MarkTreeIter)).x.is_null(),
                "enditr->x"
            );
        } else {
            may_delete = false;
        }
    }
    let mut past_right: bool = false;
    let mut moved: bool = false;
    let mut damage: MTDamageMap = Map_uint64_t_MTDamagePair {
        set: Set_uint64_t {
            h: MAPHASH_INIT,
            keys: ::core::ptr::null_mut::<uint64_t>(),
        },
        values: ::core::ptr::null_mut::<MTDamagePair>(),
    };
    if may_delete {
        's_214: while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() && !past_right {
            let mut loc_start: MTPos = start;
            let mut loc_old: MTPos = old_extent;
            relative((*(&mut itr as *mut MarkTreeIter)).pos, &mut loc_start);
            relative(
                oldbase[(*(&mut itr as *mut MarkTreeIter)).lvl as usize],
                &mut loc_old,
            );
            loop {
                if !pos_leq(
                    (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                        .pos,
                    loc_old,
                ) {
                    break 's_214;
                }
                if mt_right(
                    (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize],
                ) {
                    while !itr_eq(
                        &raw mut itr as *mut MarkTreeIter,
                        &raw mut enditr as *mut MarkTreeIter,
                    ) && mt_right(
                        (*(*(&raw mut enditr as *mut MarkTreeIter)).x).key
                            [(*(&raw mut enditr as *mut MarkTreeIter)).i as usize],
                    ) as ::core::ffi::c_int
                        != 0
                    {
                        marktree_itr_prev(b, &raw mut enditr as *mut MarkTreeIter);
                    }
                    if !mt_right(
                        (*(*(&raw mut enditr as *mut MarkTreeIter)).x).key
                            [(*(&raw mut enditr as *mut MarkTreeIter)).i as usize],
                    ) {
                        swap_keys(
                            b,
                            &raw mut itr as *mut MarkTreeIter,
                            &raw mut enditr as *mut MarkTreeIter,
                            &raw mut damage,
                        );
                    } else {
                        past_right = true;
                        break 's_214;
                    }
                }
                if itr_eq(
                    &raw mut itr as *mut MarkTreeIter,
                    &raw mut enditr as *mut MarkTreeIter,
                ) {
                    past_right = true;
                }
                moved = true;
                if (*(*(&raw mut itr as *mut MarkTreeIter)).x).level != 0 {
                    oldbase[((*(&raw mut itr as *mut MarkTreeIter)).lvl + 1 as ::core::ffi::c_int)
                        as usize] = (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                        .pos;
                    unrelative(
                        oldbase[(*(&mut itr as *mut MarkTreeIter)).lvl as usize],
                        &mut oldbase[((*(&mut itr as *mut MarkTreeIter)).lvl + 1) as usize],
                    );
                    (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                        .pos = loc_start;
                    marktree_itr_next_skip(
                        b,
                        &raw mut itr as *mut MarkTreeIter,
                        false,
                        false,
                        &raw mut oldbase as *mut MTPos,
                        ::core::ptr::null::<uint32_t>(),
                    );
                    break;
                } else {
                    (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                        .pos = loc_start;
                    if ((*(&raw mut itr as *mut MarkTreeIter)).i as int32_t)
                        < (*(*(&raw mut itr as *mut MarkTreeIter)).x).n - 1 as int32_t
                    {
                        (*(&raw mut itr as *mut MarkTreeIter)).i += 1;
                        if past_right {
                            break;
                        }
                    } else {
                        marktree_itr_next(b, &raw mut itr as *mut MarkTreeIter);
                        break;
                    }
                }
            }
        }
        's_289: while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
            let mut loc_new: MTPos = new_extent;
            relative((*(&mut itr as *mut MarkTreeIter)).pos, &mut loc_new);
            let mut limit: MTPos = old_extent;
            relative(
                oldbase[(*(&mut itr as *mut MarkTreeIter)).lvl as usize],
                &mut limit,
            );
            loop {
                if pos_leq(
                    limit,
                    (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                        .pos,
                ) {
                    break 's_289;
                }
                let mut oldpos: MTPos = (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                    [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                    .pos;
                (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                    [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                    .pos = loc_new;
                moved = true;
                if (*(*(&raw mut itr as *mut MarkTreeIter)).x).level != 0 {
                    oldbase[((*(&raw mut itr as *mut MarkTreeIter)).lvl + 1 as ::core::ffi::c_int)
                        as usize] = oldpos;
                    unrelative(
                        oldbase[(*(&mut itr as *mut MarkTreeIter)).lvl as usize],
                        &mut oldbase[((*(&mut itr as *mut MarkTreeIter)).lvl + 1) as usize],
                    );
                    marktree_itr_next_skip(
                        b,
                        &raw mut itr as *mut MarkTreeIter,
                        false,
                        false,
                        &raw mut oldbase as *mut MTPos,
                        ::core::ptr::null::<uint32_t>(),
                    );
                    break;
                } else if ((*(&raw mut itr as *mut MarkTreeIter)).i as int32_t)
                    < (*(*(&raw mut itr as *mut MarkTreeIter)).x).n - 1 as int32_t
                {
                    (*(&raw mut itr as *mut MarkTreeIter)).i += 1;
                } else {
                    marktree_itr_next(b, &raw mut itr as *mut MarkTreeIter);
                    break;
                }
            }
        }
    }
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        unrelative(
            oldbase[(*(&mut itr as *mut MarkTreeIter)).lvl as usize],
            &mut (*(&mut (*(*(&mut itr as *mut MarkTreeIter)).x).key as *mut MTKey)
                .offset((*(&mut itr as *mut MarkTreeIter)).i as isize))
            .pos,
        );
        let mut realrow: ::core::ffi::c_int = (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
            [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
            .pos
            .row as ::core::ffi::c_int;
        assert!(
            realrow as int32_t >= old_extent.row,
            "realrow >= old_extent.row"
        );
        let mut done: bool = false;
        if realrow as int32_t == old_extent.row {
            if delta.col != 0 {
                (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                    [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                    .pos
                    .col += delta.col;
            }
        } else if same_line {
            done = true;
        }
        if delta.row != 0 {
            (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
                [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
                .pos
                .row += delta.row;
            moved = true;
        }
        relative(
            (*(&mut itr as *mut MarkTreeIter)).pos,
            &mut (*(&mut (*(*(&mut itr as *mut MarkTreeIter)).x).key as *mut MTKey)
                .offset((*(&mut itr as *mut MarkTreeIter)).i as isize))
            .pos,
        );
        if done {
            break;
        }
        marktree_itr_next_skip(
            b,
            &raw mut itr as *mut MarkTreeIter,
            true,
            false,
            ::core::ptr::null_mut::<MTPos>(),
            ::core::ptr::null::<uint32_t>(),
        );
    }
    let mut start_id: uint64_t = 0;
    let mut d: MTDamagePair = MTDamagePair {
        start: MTDamage {
            old: ::core::ptr::null_mut::<MTNode>(),
            new: ::core::ptr::null_mut::<MTNode>(),
            old_i: 0,
            new_i: 0,
        },
        end: MTDamage {
            old: ::core::ptr::null_mut::<MTNode>(),
            new: ::core::ptr::null_mut::<MTNode>(),
            old_i: 0,
            new_i: 0,
        },
    };
    let mut __i: uint32_t = 0;
    __i = 0 as uint32_t;
    while __i < damage.set.h.n_keys {
        start_id = *damage.set.keys.offset(__i as isize);
        d = *damage.values.offset(__i as isize);
        if !d.start.old.is_null() && !d.end.old.is_null() {
            marktree_itr_set_node(
                b,
                &raw mut itr as *mut MarkTreeIter,
                d.start.old,
                d.start.old_i,
            );
            marktree_itr_set_node(
                b,
                &raw mut enditr as *mut MarkTreeIter,
                d.end.old,
                d.end.old_i,
            );
            marktree_intersect_pair(
                b,
                start_id,
                &raw mut itr as *mut MarkTreeIter,
                &raw mut enditr as *mut MarkTreeIter,
                true,
            );
            marktree_itr_set_node(
                b,
                &raw mut itr as *mut MarkTreeIter,
                d.start.new,
                d.start.new_i,
            );
            marktree_itr_set_node(
                b,
                &raw mut enditr as *mut MarkTreeIter,
                d.end.new,
                d.end.new_i,
            );
            marktree_intersect_pair(
                b,
                start_id,
                &raw mut itr as *mut MarkTreeIter,
                &raw mut enditr as *mut MarkTreeIter,
                false,
            );
        } else if !d.start.old.is_null() {
            let mut endpos: [MarkTreeIter; 1] = [MarkTreeIter {
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
                start_id | 1 as ::core::ffi::c_int as uint64_t,
                &raw mut endpos as *mut MarkTreeIter,
            );
            if !(*(&raw mut endpos as *mut MarkTreeIter)).x.is_null() {
                marktree_itr_set_node(
                    b,
                    &raw mut itr as *mut MarkTreeIter,
                    d.start.old,
                    d.start.old_i,
                );
                *(&raw mut enditr as *mut MarkTreeIter) = *(&raw mut endpos as *mut MarkTreeIter);
                marktree_intersect_pair(
                    b,
                    start_id,
                    &raw mut itr as *mut MarkTreeIter,
                    &raw mut enditr as *mut MarkTreeIter,
                    true,
                );
                marktree_itr_set_node(
                    b,
                    &raw mut itr as *mut MarkTreeIter,
                    d.start.new,
                    d.start.new_i,
                );
                *(&raw mut enditr as *mut MarkTreeIter) = *(&raw mut endpos as *mut MarkTreeIter);
                marktree_intersect_pair(
                    b,
                    start_id,
                    &raw mut itr as *mut MarkTreeIter,
                    &raw mut enditr as *mut MarkTreeIter,
                    false,
                );
            }
        } else if !d.end.old.is_null() {
            let mut startpos: [MarkTreeIter; 1] = [MarkTreeIter {
                pos: MTPos { row: 0, col: 0 },
                lvl: 0,
                x: ::core::ptr::null_mut::<MTNode>(),
                i: 0,
                s: [C2Rust_Unnamed_2 { oldcol: 0, i: 0 }; 20],
                intersect_idx: 0,
                intersect_pos: MTPos { row: 0, col: 0 },
                intersect_pos_x: MTPos { row: 0, col: 0 },
            }; 1];
            marktree_lookup(b, start_id, &raw mut startpos as *mut MarkTreeIter);
            if !(*(&raw mut startpos as *mut MarkTreeIter)).x.is_null() {
                *(&raw mut itr as *mut MarkTreeIter) = *(&raw mut startpos as *mut MarkTreeIter);
                marktree_itr_set_node(
                    b,
                    &raw mut enditr as *mut MarkTreeIter,
                    d.end.old,
                    d.end.old_i,
                );
                marktree_intersect_pair(
                    b,
                    start_id,
                    &raw mut itr as *mut MarkTreeIter,
                    &raw mut enditr as *mut MarkTreeIter,
                    true,
                );
                *(&raw mut itr as *mut MarkTreeIter) = *(&raw mut startpos as *mut MarkTreeIter);
                marktree_itr_set_node(
                    b,
                    &raw mut enditr as *mut MarkTreeIter,
                    d.end.new,
                    d.end.new_i,
                );
                marktree_intersect_pair(
                    b,
                    start_id,
                    &raw mut itr as *mut MarkTreeIter,
                    &raw mut enditr as *mut MarkTreeIter,
                    false,
                );
            }
        }
        __i = __i.wrapping_add(1);
    }
    xfree(damage.set.keys as *mut ::core::ffi::c_void);
    xfree(damage.set.h.hash as *mut ::core::ffi::c_void);
    damage.set = Set_uint64_t {
        h: MAPHASH_INIT,
        keys: ::core::ptr::null_mut::<uint64_t>(),
    };
    let mut ptr_: *mut *mut ::core::ffi::c_void =
        &raw mut damage.values as *mut *mut ::core::ffi::c_void;
    xfree(*ptr_);
    *ptr_ = NULL;
    let _ = *ptr_;
    return moved;
}
pub unsafe extern "C" fn marktree_move_region(
    mut b: *mut MarkTree,
    mut start_row: ::core::ffi::c_int,
    mut start_col: colnr_T,
    mut extent_row: ::core::ffi::c_int,
    mut extent_col: colnr_T,
    mut new_row: ::core::ffi::c_int,
    mut new_col: colnr_T,
) {
    let mut start: MTPos = MTPos {
        row: start_row as int32_t,
        col: start_col as int32_t,
    };
    let mut size: MTPos = MTPos {
        row: extent_row as int32_t,
        col: extent_col as int32_t,
    };
    let mut end: MTPos = size;
    unrelative(start, &mut end);
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
    marktree_itr_get_ext(
        b,
        start,
        &raw mut itr as *mut MarkTreeIter,
        false,
        true,
        ::core::ptr::null_mut::<MTPos>(),
        ::core::ptr::null::<uint32_t>(),
    );
    let mut saved: C2Rust_Unnamed_3 = KV_INITIAL_VALUE;
    while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
        let mut k: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
        if !pos_leq(k.pos, end)
            || k.pos.row == end.row
                && k.pos.col == end.col
                && mt_right(k) as ::core::ffi::c_int != 0
        {
            break;
        }
        relative(start, &mut k.pos);
        if saved.size == saved.capacity {
            saved.capacity = if saved.capacity != 0 {
                saved.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            saved.items = xrealloc(
                saved.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<MTKey>().wrapping_mul(saved.capacity),
            ) as *mut MTKey;
        } else {
        };
        let c2rust_fresh21 = saved.size;
        saved.size = saved.size.wrapping_add(1);
        *saved.items.offset(c2rust_fresh21 as isize) = k;
        marktree_del_itr(b, &raw mut itr as *mut MarkTreeIter, false);
    }
    marktree_splice(
        b,
        start.row,
        start.col as ::core::ffi::c_int,
        size.row as ::core::ffi::c_int,
        size.col as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
    );
    let mut new: MTPos = MTPos {
        row: new_row as int32_t,
        col: new_col as int32_t,
    };
    marktree_splice(
        b,
        new.row,
        new.col as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        0 as ::core::ffi::c_int,
        size.row as ::core::ffi::c_int,
        size.col as ::core::ffi::c_int,
    );
    let mut i: size_t = 0 as size_t;
    while i < saved.size {
        let mut item: MTKey = *saved.items.offset(i as isize);
        unrelative(new, &mut item.pos);
        marktree_put_key(b, item);
        if mt_paired(item) {
            marktree_restore_pair(b, item);
        }
        i = i.wrapping_add(1);
    }
    xfree(saved.items as *mut ::core::ffi::c_void);
    saved.capacity = 0 as size_t;
    saved.size = saved.capacity;
    saved.items = ::core::ptr::null_mut::<MTKey>();
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
unsafe extern "C" fn marktree_itr_fix_pos(mut b: *mut MarkTree, mut itr: *mut MarkTreeIter) {
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
    marktree_put(b, key, end_row, end_col, end_right);
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
    marktree_lookup_ns(b, ns, id, false, &raw mut itr as *mut MarkTreeIter);
    let mut other: uint64_t = marktree_del_itr(b, &raw mut itr as *mut MarkTreeIter, false);
    assert!(other != 0, "other");
    marktree_lookup(b, other, &raw mut itr as *mut MarkTreeIter);
    marktree_del_itr(b, &raw mut itr as *mut MarkTreeIter, false);
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_check(mut b: *mut MarkTree) {
    if (*b).root.is_null() {
        assert!((*b).n_keys == 0 as size_t, "b->n_keys == 0");
        assert!((*b).n_nodes == 0 as size_t, "b->n_nodes == 0");
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
    assert!((*b).n_keys == nkeys, "b->n_keys == nkeys");
    assert!(
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
    assert!(
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
        assert!(
            pos_leq(*last, (*x).key[i as usize].pos),
            "pos_leq(*last, x->key[i].pos)"
        );
        if (*last).row == (*x).key[i as usize].pos.row
            && (*last).col == (*x).key[i as usize].pos.col
        {
            assert!(
                !*last_right || mt_right((*x).key[i as usize]) as ::core::ffi::c_int != 0,
                "!*last_right || mt_right(x->key[i])"
            );
        }
        *last_right = mt_right((*x).key[i as usize]);
        assert!(
            (*x).key[i as usize].pos.col >= 0 as int32_t,
            "x->key[i].pos.col >= 0"
        );
        assert!(
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
            assert!(
                (*(*inner(x)).i_ptr[i_0 as usize]).parent == x,
                "x->ptr[i]->parent == x"
            );
            assert!(
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
                assert!(
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
    assert!(
        *meta_node_ref == meta_describe_node(x),
        "meta_node_ref[m] == meta_node[m]"
    );
    return n_keys;
}
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
            let mut k: MTKey = marktree_lookup(b, end_id, &raw mut end_itr as *mut MarkTreeIter);
            if k.pos.row >= 0 as int32_t {
                *(&raw mut start_itr as *mut MarkTreeIter) = *(&raw mut itr as *mut MarkTreeIter);
                marktree_intersect_pair(
                    b,
                    mt_lookup_key(mark),
                    &raw mut start_itr as *mut MarkTreeIter,
                    &raw mut end_itr as *mut MarkTreeIter,
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
pub unsafe extern "C" fn mt_recurse_nodes(mut x: *mut MTNode, mut checked: *mut Map_ptr_t_ptr_t) {
    if (*x).intersect.size != 0 {
        if (*x).intersect.size == (*x).intersect.capacity {
            (*x).intersect.capacity = if (*x).intersect.capacity << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*x).intersect.capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*x).intersect.items = (if (*x).intersect.capacity
                == ::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_div(::core::mem::size_of::<uint64_t>())
                    .wrapping_div(
                        (::core::mem::size_of::<[uint64_t; 4]>()
                            .wrapping_rem(::core::mem::size_of::<uint64_t>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*x).intersect.items == &raw mut (*x).intersect.init_array as *mut uint64_t {
                    (*x).intersect.items as *mut ::core::ffi::c_void
                } else {
                    _memcpy_free(
                        &raw mut (*x).intersect.init_array as *mut uint64_t
                            as *mut ::core::ffi::c_void,
                        (*x).intersect.items as *mut ::core::ffi::c_void,
                        (*x).intersect
                            .size
                            .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                }
            } else {
                if (*x).intersect.items == &raw mut (*x).intersect.init_array as *mut uint64_t {
                    memcpy(
                        xmalloc(
                            (*x).intersect
                                .capacity
                                .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                        ),
                        (*x).intersect.items as *const ::core::ffi::c_void,
                        (*x).intersect
                            .size
                            .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                } else {
                    xrealloc(
                        (*x).intersect.items as *mut ::core::ffi::c_void,
                        (*x).intersect
                            .capacity
                            .wrapping_mul(::core::mem::size_of::<uint64_t>()),
                    )
                }
            }) as *mut uint64_t;
        } else {
        };
        let c2rust_fresh22 = (*x).intersect.size;
        (*x).intersect.size = (*x).intersect.size.wrapping_add(1);
        *(*x).intersect.items.offset(c2rust_fresh22 as isize) =
            -1 as ::core::ffi::c_int as uint64_t;
        let mut val: *mut uint64_t = ::core::ptr::null_mut::<uint64_t>();
        if (*x).intersect.items == &raw mut (*x).intersect.init_array as *mut uint64_t {
            val = xmemdup(
                (*x).intersect.items as *const ::core::ffi::c_void,
                (*x).intersect
                    .size
                    .wrapping_mul(::core::mem::size_of::<uint64_t>()),
            ) as *mut uint64_t;
        } else {
            val = (*x).intersect.items;
        }
        map_put_ptr_t_ptr_t(checked, x as ptr_t, val as ptr_t);
        (*x).intersect.capacity = ::core::mem::size_of::<[uint64_t; 4]>()
            .wrapping_div(::core::mem::size_of::<uint64_t>())
            .wrapping_div(
                (::core::mem::size_of::<[uint64_t; 4]>()
                    .wrapping_rem(::core::mem::size_of::<uint64_t>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as size_t;
        (*x).intersect.size = 0 as size_t;
        (*x).intersect.items = &raw mut (*x).intersect.init_array as *mut uint64_t;
    }
    if (*x).level != 0 {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i as int32_t) < (*x).n + 1 as int32_t {
            mt_recurse_nodes((*inner(x)).i_ptr[i as usize], checked);
            i += 1;
        }
    }
}
pub unsafe extern "C" fn mt_recurse_nodes_compare(
    mut x: *mut MTNode,
    mut checked: *mut Map_ptr_t_ptr_t,
) -> bool {
    let mut ref_0: *mut uint64_t = map_get_ptr_t_ptr_t(checked, x as ptr_t) as *mut uint64_t;
    if !ref_0.is_null() {
        let mut i: size_t = 0 as size_t;
        loop {
            if *ref_0.offset(i as isize) == -1 as ::core::ffi::c_int as uint64_t {
                if i != (*x).intersect.size {
                    return false;
                }
                break;
            } else {
                if (*x).intersect.size <= i
                    || *ref_0.offset(i as isize) != *(*x).intersect.items.offset(i as isize)
                {
                    return false;
                }
                i = i.wrapping_add(1);
            }
        }
    } else if (*x).intersect.size != 0 {
        return false;
    }
    if (*x).level != 0 {
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while (i_0 as int32_t) < (*x).n + 1 as int32_t {
            if !mt_recurse_nodes_compare((*inner(x)).i_ptr[i_0 as usize], checked) {
                return false;
            }
            i_0 += 1;
        }
    }
    return true;
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mt_inspect(
    mut b: *mut MarkTree,
    mut keys: bool,
    mut dot: bool,
) -> String_0 {
    let mut ga: [garray_T; 1] = [garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 0,
        ga_growsize: 0,
        ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
    }; 1];
    ga_init(
        &raw mut ga as *mut garray_T,
        ::core::mem::size_of::<::core::ffi::c_char>() as ::core::ffi::c_int,
        80 as ::core::ffi::c_int,
    );
    let mut p: MTPos = MTPos {
        row: 0 as int32_t,
        col: 0 as int32_t,
    };
    if !(*b).root.is_null() {
        if dot {
            ga_concat(
                &raw mut ga as *mut garray_T,
                b"digraph D {\n\n\0".as_ptr() as *const ::core::ffi::c_char
                    as *mut ::core::ffi::c_char,
            );
            mt_inspect_dotfile_node(
                b,
                &raw mut ga as *mut garray_T,
                (*b).root,
                p,
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
            );
            ga_concat(
                &raw mut ga as *mut garray_T,
                b"\n}\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        } else {
            mt_inspect_node(b, &raw mut ga as *mut garray_T, keys, (*b).root, p);
        }
    }
    return ga_take_string(&raw mut ga as *mut garray_T);
}
#[inline]
unsafe extern "C" fn mt_dbg_id(mut id: uint64_t) -> uint64_t {
    return id >> 1 as ::core::ffi::c_int & 0xffffffff as uint64_t;
}
unsafe extern "C" fn mt_inspect_node(
    mut b: *mut MarkTree,
    mut ga: *mut garray_T,
    mut keys: bool,
    mut n: *mut MTNode,
    mut off: MTPos,
) {
    static buf: GlobalCell<[::core::ffi::c_char; 1024]> = GlobalCell::new([0; 1024]);
    ga_concat(
        ga,
        b"[\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    if keys as ::core::ffi::c_int != 0 && (*n).intersect.size != 0 {
        let mut i: size_t = 0 as size_t;
        while i < (*n).intersect.size {
            ga_concat(
                ga,
                (if i == 0 as size_t {
                    b"{\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b";\0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char,
            );
            snprintf(
                buf.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
                b"%lu\0".as_ptr() as *const ::core::ffi::c_char,
                mt_dbg_id(*(*n).intersect.items.offset(i as isize)),
            );
            ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
            i = i.wrapping_add(1);
        }
        ga_concat(
            ga,
            b"},\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
    }
    if (*n).level != 0 {
        mt_inspect_node(
            b,
            ga,
            keys,
            (*inner(n)).i_ptr[0 as ::core::ffi::c_int as usize],
            off,
        );
    }
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i_0 as int32_t) < (*n).n {
        let mut p: MTPos = (*n).key[i_0 as usize].pos;
        unrelative(off, &mut p);
        snprintf(
            buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
            b"%d/%d\0".as_ptr() as *const ::core::ffi::c_char,
            p.row,
            p.col,
        );
        ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
        if keys {
            let mut key: MTKey = (*n).key[i_0 as usize];
            ga_concat(
                ga,
                b":\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
            if mt_start(key) {
                ga_concat(
                    ga,
                    b"<\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            }
            snprintf(
                buf.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
                b"%u\0".as_ptr() as *const ::core::ffi::c_char,
                key.id,
            );
            ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
            if mt_end(key) {
                ga_concat(
                    ga,
                    b">\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            }
        }
        if (*n).level != 0 {
            mt_inspect_node(
                b,
                ga,
                keys,
                (*inner(n)).i_ptr[(i_0 + 1 as ::core::ffi::c_int) as usize],
                p,
            );
        } else {
            ga_concat(ga, b",\0".as_ptr() as *const ::core::ffi::c_char);
        }
        i_0 += 1;
    }
    ga_concat(ga, b"]\0".as_ptr() as *const ::core::ffi::c_char);
}
unsafe extern "C" fn mt_inspect_dotfile_node(
    mut b: *mut MarkTree,
    mut ga: *mut garray_T,
    mut n: *mut MTNode,
    mut off: MTPos,
    mut parent: *mut ::core::ffi::c_char,
) {
    static buf: GlobalCell<[::core::ffi::c_char; 1024]> = GlobalCell::new([0; 1024]);
    let mut namebuf: [::core::ffi::c_char; 64] = [0; 64];
    if !parent.is_null() {
        snprintf(
            &raw mut namebuf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>(),
            b"%s_%c%d\0".as_ptr() as *const ::core::ffi::c_char,
            parent,
            'a' as ::core::ffi::c_int + (*n).level as ::core::ffi::c_int,
            (*n).p_idx as ::core::ffi::c_int,
        );
    } else {
        snprintf(
            &raw mut namebuf as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 64]>(),
            b"MTNode\0".as_ptr() as *const ::core::ffi::c_char,
        );
    }
    snprintf(
        buf.ptr() as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
        b"  %s[shape=plaintext, label=<\n\0".as_ptr() as *const ::core::ffi::c_char,
        &raw mut namebuf as *mut ::core::ffi::c_char,
    );
    ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
    ga_concat(
        ga,
        b"    <table border='0' cellborder='1' cellspacing='0'>\n\0".as_ptr()
            as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    if (*n).intersect.size != 0 {
        ga_concat(
            ga,
            b"    <tr><td>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
        let mut i: size_t = 0 as size_t;
        while i < (*n).intersect.size {
            if i > 0 as size_t {
                ga_concat(
                    ga,
                    b", \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
            }
            snprintf(
                buf.ptr() as *mut ::core::ffi::c_char,
                ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
                b"%lu\0".as_ptr() as *const ::core::ffi::c_char,
                mt_dbg_id(*(*n).intersect.items.offset(i as isize)),
            );
            ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
            i = i.wrapping_add(1);
        }
        ga_concat(
            ga,
            b"</td></tr>\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
        );
    }
    ga_concat(
        ga,
        b"    <tr><td>\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i_0 as int32_t) < (*n).n {
        let mut k: MTKey = (*n).key[i_0 as usize];
        if i_0 > 0 as ::core::ffi::c_int {
            ga_concat(
                ga,
                b", \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
            );
        }
        snprintf(
            buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
            b"%d\0".as_ptr() as *const ::core::ffi::c_char,
            k.id,
        );
        ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
        if mt_paired(k) {
            ga_concat(
                ga,
                (if mt_end(k) as ::core::ffi::c_int != 0 {
                    b"e\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"s\0".as_ptr() as *const ::core::ffi::c_char
                }) as *mut ::core::ffi::c_char,
            );
        }
        i_0 += 1;
    }
    ga_concat(
        ga,
        b"</td></tr>\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    ga_concat(
        ga,
        b"    </table>\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    ga_concat(
        ga,
        b">];\n\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
    );
    if !parent.is_null() {
        snprintf(
            buf.ptr() as *mut ::core::ffi::c_char,
            ::core::mem::size_of::<[::core::ffi::c_char; 1024]>(),
            b"  %s -> %s\n\0".as_ptr() as *const ::core::ffi::c_char,
            parent,
            &raw mut namebuf as *mut ::core::ffi::c_char,
        );
        ga_concat(ga, buf.ptr() as *mut ::core::ffi::c_char);
    }
    if (*n).level != 0 {
        mt_inspect_dotfile_node(
            b,
            ga,
            (*inner(n)).i_ptr[0 as ::core::ffi::c_int as usize],
            off,
            &raw mut namebuf as *mut ::core::ffi::c_char,
        );
    }
    let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    while (i_1 as int32_t) < (*n).n {
        let mut p: MTPos = (*n).key[i_1 as usize].pos;
        unrelative(off, &mut p);
        if (*n).level != 0 {
            mt_inspect_dotfile_node(
                b,
                ga,
                (*inner(n)).i_ptr[(i_1 + 1 as ::core::ffi::c_int) as usize],
                p,
                &raw mut namebuf as *mut ::core::ffi::c_char,
            );
        }
        i_1 += 1;
    }
}
