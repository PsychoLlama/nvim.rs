//! Adjusting every mark for a buffer edit.
//!
//! `marktree_splice` is given the start of the edited region, the extent that
//! was removed and the extent that replaced it, and has to move every mark at
//! or after the start. Three regions behave differently:
//!
//! * before the start: untouched;
//! * inside the deleted region: collapses to one end of it, chosen by the
//!   mark's gravity — right-gravity marks ride to the new end, left-gravity
//!   ones stay at the start;
//! * after: shifted by the difference between the old and new extents, and
//!   only the first line of the tail also moves by a column delta.
//!
//! Because a node's keys are stored relative to each other, a splice that lands
//! in the middle of a node can be applied to the node's *own* position and stop
//! there — the keys inside it never move. That is the whole reason the tree is
//! shaped this way, and it is why the code walks down looking for the deepest
//! node whose span contains the whole edit before it starts rewriting anything.
//!
//! Collapsing a range can leave two marks in the wrong order, since the
//! relative encoding cannot express a negative offset. `swap_keys` restores the
//! order and `check_damage` records the pairs whose ends crossed, so
//! `marktree_restore_pair` can put them back once the walk is done.

use super::*;

pub unsafe extern "C" fn check_damage(
    mut _b: *mut MarkTree,
    mut damage: *mut MTDamageMap,
    mut itr1: *mut MarkTreeIter,
    mut itr2: *mut MarkTreeIter,
) {
    let start_id: uint64_t = mt_lookup_key_side((*(*itr1).x).key[(*itr1).i as usize], false);
    let mut p: *mut MTDamagePair = map_put_ref_uint64_t_MTDamagePair(
        damage,
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
    debug_assert!((*me).new.is_null(), "me->new == NULL");
    *me = MTDamage {
        old: (*itr1).x,
        new: (*itr2).x,
        old_i: (*itr1).i,
        new_i: (*itr2).i,
    };
}

pub unsafe extern "C" fn swap_keys(
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
            debug_assert!(
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
        debug_assert!(
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
    // The marks inside the moved region, lifted out and re-inserted at the
    // destination once the two splices have shifted everything else.
    let mut saved: Vec<MTKey> = Vec::new();
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
        saved.push(k);
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
    for mut item in saved {
        unrelative(new, &mut item.pos);
        marktree_put_key(b, item);
        if mt_paired(item) {
            marktree_restore_pair(b, item);
        }
    }
}
