//! `DecorState`: the decorations in play while a window is drawn.
//!
//! The drawing code walks a window top to bottom and left to right, and
//! this is the machinery that keeps up with it. Marks are pulled out of the
//! marktree as their row is reached and split into `DecorRange`s — one per
//! highlight, virt text or virt-lines block — held in a slot array with two
//! index lists over it: the ranges that have started, sorted by priority,
//! and the ones still ahead, sorted by position. [`decor_redraw_col_impl`]
//! advances both as the column moves and answers the combined attribute for
//! the cell.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn decor_state_invalidate(mut buf: *mut buf_T) {
    unsafe {
        if !(*decor_state.ptr()).win.is_null() && (*(*decor_state.ptr()).win).w_buffer == buf {
            (*decor_state.ptr()).itr_valid = false_0 != 0;
        }
    }
}

pub unsafe extern "C" fn decor_state_free(mut state: *mut DecorState) {
    unsafe {
        xfree((*state).slots.items as *mut ::core::ffi::c_void);
        (*state).slots.capacity = 0 as size_t;
        (*state).slots.size = (*state).slots.capacity;
        (*state).slots.items = ::core::ptr::null_mut::<DecorRangeSlot>();
        xfree((*state).ranges_i.items as *mut ::core::ffi::c_void);
        (*state).ranges_i.capacity = 0 as size_t;
        (*state).ranges_i.size = (*state).ranges_i.capacity;
        (*state).ranges_i.items = ::core::ptr::null_mut::<::core::ffi::c_int>();
    }
}

pub unsafe extern "C" fn decor_redraw_reset(
    mut wp: *mut win_T,
    mut state: *mut DecorState,
) -> bool {
    unsafe {
        (*state).row = -1 as ::core::ffi::c_int;
        (*state).win = wp;
        let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
        let slots: *mut DecorRangeSlot = (*state).slots.items;
        let beg_pos: [::core::ffi::c_int; 2] = [0 as ::core::ffi::c_int, (*state).future_begin];
        let end_pos: [::core::ffi::c_int; 2] = [
            (*state).current_end,
            (*state).ranges_i.size as ::core::ffi::c_int,
        ];
        let mut pos_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while pos_i < 2 as ::core::ffi::c_int {
            let mut i: ::core::ffi::c_int = beg_pos[pos_i as usize];
            while i < end_pos[pos_i as usize] {
                let r: *mut DecorRange =
                    &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
                if (*r).owned as ::core::ffi::c_int != 0
                    && (*r).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int
                {
                    clear_virttext(&raw mut (*(*r).data.vt).data.virt_text);
                    xfree((*r).data.vt as *mut ::core::ffi::c_void);
                }
                i += 1;
            }
            pos_i += 1;
        }
        (*state).slots.size = 0 as size_t;
        (*state).ranges_i.size = 0 as size_t;
        (*state).free_slot_i = -1 as ::core::ffi::c_int;
        (*state).current_end = 0 as ::core::ffi::c_int;
        (*state).future_begin = 0 as ::core::ffi::c_int;
        (*state).new_range_ordering = 0 as ::core::ffi::c_int;
        return (*(&raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree)).n_keys != 0;
    }
}

pub unsafe extern "C" fn decor_virt_pos(mut decor: *const DecorRange) -> bool {
    unsafe {
        return (*decor).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int
            || (*decor).kind as ::core::ffi::c_int == kDecorKindUIWatched as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn decor_virt_pos_kind(mut decor: *const DecorRange) -> VirtTextPos {
    unsafe {
        if (*decor).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int {
            return (*(*decor).data.vt).pos;
        }
        if (*decor).kind as ::core::ffi::c_int == kDecorKindUIWatched as ::core::ffi::c_int {
            return (*decor).data.ui.pos;
        }
        return kVPosEndOfLine;
    }
}

pub unsafe extern "C" fn decor_redraw_start(
    mut wp: *mut win_T,
    mut top_row: ::core::ffi::c_int,
    mut state: *mut DecorState,
) -> bool {
    unsafe {
        let mut buf: *mut buf_T = (*wp).w_buffer;
        (*state).top_row = top_row;
        (*state).itr_valid = true_0 != 0;
        if !marktree_itr_get_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            top_row,
            0 as ::core::ffi::c_int,
            &raw mut (*state).itr as *mut MarkTreeIter,
        ) {
            return false_0 != 0;
        }
        let mut pair: MTPair = MTPair {
            start: MTKey {
                pos: MTPos { row: 0, col: 0 },
                ns: 0,
                id: 0,
                flags: 0,
                decor_data: DecorInlineData {
                    hl: DecorHighlightInline {
                        flags: 0,
                        priority: 0,
                        hl_id: 0,
                        conceal_char: 0,
                    },
                },
            },
            end_pos: MTPos { row: 0, col: 0 },
            end_right_gravity: false,
        };
        while marktree_itr_step_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut (*state).itr as *mut MarkTreeIter,
            &raw mut pair,
        ) {
            let mut m: MTKey = pair.start;
            if mt_invalid(m) as ::core::ffi::c_int != 0 || !mt_decor_any(m) {
                continue;
            }
            decor_range_add_from_inline(
                state,
                pair.start.pos.row as ::core::ffi::c_int,
                pair.start.pos.col as ::core::ffi::c_int,
                pair.end_pos.row as ::core::ffi::c_int,
                pair.end_pos.col as ::core::ffi::c_int,
                mt_decor(m),
                false_0 != 0,
                m.ns,
                m.id,
            );
        }
        return true_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn decor_state_pack(mut state: *mut DecorState) {
    unsafe {
        let mut count: ::core::ffi::c_int = (*state).ranges_i.size as ::core::ffi::c_int;
        let cur_end: ::core::ffi::c_int = (*state).current_end;
        let mut fut_beg: ::core::ffi::c_int = (*state).future_begin;
        if fut_beg == count {
            count = cur_end;
            fut_beg = count;
        } else if fut_beg != cur_end {
            let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
            memmove(
                indices.offset(cur_end as isize) as *mut ::core::ffi::c_void,
                indices.offset(fut_beg as isize) as *const ::core::ffi::c_void,
                ((count - fut_beg) as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
            );
            count = cur_end + (count - fut_beg);
            fut_beg = cur_end;
        }
        (*state).ranges_i.size = count as size_t;
        (*state).future_begin = fut_beg;
    }
}

pub unsafe extern "C" fn decor_redraw_line(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
    mut state: *mut DecorState,
) {
    unsafe {
        decor_state_pack(state);
        if (*state).row == -1 as ::core::ffi::c_int {
            decor_redraw_start(wp, row, state);
        } else if !(*state).itr_valid {
            marktree_itr_get(
                &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
                row as int32_t,
                0 as ::core::ffi::c_int,
                &raw mut (*state).itr as *mut MarkTreeIter,
            );
            (*state).itr_valid = true_0 != 0;
        }
        (*state).row = row;
        (*state).col_last = -1 as ::core::ffi::c_int;
        (*state).eol_col = -1 as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn decor_has_more_decorations(
    mut state: *mut DecorState,
    mut row: ::core::ffi::c_int,
) -> bool {
    unsafe {
        if (*state).current_end != 0 as ::core::ffi::c_int
            || (*state).future_begin != (*state).ranges_i.size as ::core::ffi::c_int
        {
            return true_0 != 0;
        }
        let mut k: MTKey = marktree_itr_current(&raw mut (*state).itr as *mut MarkTreeIter);
        return k.pos.row >= 0 as int32_t && k.pos.row <= row as int32_t;
    }
}

pub(crate) unsafe extern "C" fn decor_range_add_from_inline(
    mut state: *mut DecorState,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut decor: DecorInline,
    mut owned: bool,
    mut ns: uint32_t,
    mut mark_id: uint32_t,
) {
    unsafe {
        if decor.ext {
            let mut vt: *mut DecorVirtText = decor.data.ext.vt;
            while !vt.is_null() {
                decor_range_add_virt(state, start_row, start_col, end_row, end_col, vt, owned);
                vt = (*vt).next;
            }
            let mut idx: uint32_t = decor.data.ext.sh_idx;
            while idx != DECOR_ID_INVALID as uint32_t {
                let mut sh: *mut DecorSignHighlight =
                    (*decor_items.ptr()).items.offset(idx as isize);
                decor_range_add_sh(
                    state,
                    start_row,
                    start_col,
                    end_row,
                    end_col,
                    sh,
                    owned,
                    ns,
                    mark_id,
                    0 as DecorPriority,
                );
                idx = (*sh).next;
            }
        } else {
            let mut sh_0: DecorSignHighlight = decor_sh_from_inline(decor.data.hl);
            decor_range_add_sh(
                state,
                start_row,
                start_col,
                end_row,
                end_col,
                &raw mut sh_0,
                owned,
                ns,
                mark_id,
                0 as DecorPriority,
            );
        };
    }
}

pub(crate) unsafe extern "C" fn decor_range_insert(
    mut state: *mut DecorState,
    mut range: *mut DecorRange,
) {
    unsafe {
        let c2rust_fresh2 = (*state).new_range_ordering;
        (*state).new_range_ordering = (*state).new_range_ordering + 1;
        (*range).ordering = c2rust_fresh2;
        let mut index: ::core::ffi::c_int = 0;
        if (*state).free_slot_i >= 0 as ::core::ffi::c_int {
            index = (*state).free_slot_i;
            let mut slot: *mut DecorRangeSlot = (*state).slots.items.offset(index as isize);
            (*state).free_slot_i = (*slot).next_free_i;
            (*slot).range = *range;
        } else {
            index = (*state).slots.size as ::core::ffi::c_int;
            if (*state).slots.size == (*state).slots.capacity {
                (*state).slots.capacity = if (*state).slots.capacity != 0 {
                    (*state).slots.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                (*state).slots.items = xrealloc(
                    (*state).slots.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<DecorRangeSlot>().wrapping_mul((*state).slots.capacity),
                ) as *mut DecorRangeSlot;
            } else {
            };
            let c2rust_fresh3 = (*state).slots.size;
            (*state).slots.size = (*state).slots.size.wrapping_add(1);
            (*(*state).slots.items.offset(c2rust_fresh3 as isize)).range = *range;
        }
        let row: ::core::ffi::c_int = (*range).start_row;
        let col: ::core::ffi::c_int = (*range).start_col;
        let count: ::core::ffi::c_int = (*state).ranges_i.size as ::core::ffi::c_int;
        let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
        let slots: *mut DecorRangeSlot = (*state).slots.items;
        let mut begin: ::core::ffi::c_int = (*state).future_begin;
        let mut end: ::core::ffi::c_int = count;
        while begin < end {
            let mid: ::core::ffi::c_int = begin + (end - begin >> 1 as ::core::ffi::c_int);
            let mr: *mut DecorRange =
                &raw mut (*slots.offset(*indices.offset(mid as isize) as isize)).range;
            let mrow: ::core::ffi::c_int = (*mr).start_row;
            let mcol: ::core::ffi::c_int = (*mr).start_col;
            if mrow < row || mrow == row && mcol <= col {
                begin = mid + 1 as ::core::ffi::c_int;
                if mrow == row && mcol == col {
                    break;
                }
            } else {
                end = mid;
            }
        }
        if (*state).ranges_i.size == (*state).ranges_i.capacity {
            (*state).ranges_i.capacity = if (*state).ranges_i.capacity != 0 {
                (*state).ranges_i.capacity << 1 as ::core::ffi::c_int
            } else {
                8 as size_t
            };
            (*state).ranges_i.items = xrealloc(
                (*state).ranges_i.items as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<::core::ffi::c_int>()
                    .wrapping_mul((*state).ranges_i.capacity),
            ) as *mut ::core::ffi::c_int;
        } else {
        };
        (*state).ranges_i.size = (*state).ranges_i.size.wrapping_add(1);
        let item: *mut ::core::ffi::c_int = (*state).ranges_i.items.offset(begin as isize);
        memmove(
            item.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
            item as *const ::core::ffi::c_void,
            ((count - begin) as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
        );
        *item = index;
    }
}

pub unsafe extern "C" fn decor_range_add_virt(
    mut state: *mut DecorState,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut vt: *mut DecorVirtText,
    mut owned: bool,
) {
    unsafe {
        let mut is_lines: bool =
            (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0;
        let mut range: DecorRange = DecorRange {
            start_row: start_row,
            start_col: start_col,
            end_row: end_row,
            end_col: end_col,
            ordering: 0,
            priority_internal: ((*vt).priority as DecorPriorityInternal)
                << 16 as ::core::ffi::c_int,
            owned: owned,
            kind: (if is_lines as ::core::ffi::c_int != 0 {
                kDecorKindVirtLines as ::core::ffi::c_int
            } else {
                kDecorKindVirtText as ::core::ffi::c_int
            }) as DecorRangeKind,
            data: C2Rust_Unnamed_22 { vt: vt },
            attr_id: 0 as ::core::ffi::c_int,
            draw_col: -10 as ::core::ffi::c_int,
        };
        decor_range_insert(state, &raw mut range);
    }
}

pub unsafe extern "C" fn decor_range_add_sh(
    mut state: *mut DecorState,
    mut start_row: ::core::ffi::c_int,
    mut start_col: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut end_col: ::core::ffi::c_int,
    mut sh: *mut DecorSignHighlight,
    mut owned: bool,
    mut ns: uint32_t,
    mut mark_id: uint32_t,
    mut subpriority: DecorPriority,
) {
    unsafe {
        if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
            return;
        }
        let mut range: DecorRange = DecorRange {
            start_row: start_row,
            start_col: start_col,
            end_row: end_row,
            end_col: end_col,
            ordering: 0,
            priority_internal: (((*sh).priority as DecorPriorityInternal)
                << 16 as ::core::ffi::c_int)
                .wrapping_add(subpriority as DecorPriorityInternal),
            owned: owned,
            kind: kDecorKindHighlight as ::core::ffi::c_int as DecorRangeKind,
            data: C2Rust_Unnamed_22 { sh: *sh },
            attr_id: 0 as ::core::ffi::c_int,
            draw_col: -10 as ::core::ffi::c_int,
        };
        if (*sh).hl_id != 0
            || !(*sh).url.is_null()
            || (*sh).flags as ::core::ffi::c_int
                & (kSHConceal as ::core::ffi::c_int
                    | kSHSpellOn as ::core::ffi::c_int
                    | kSHSpellOff as ::core::ffi::c_int)
                != 0
        {
            if (*sh).hl_id != 0 {
                range.attr_id = syn_id2attr((*sh).hl_id);
            }
            decor_range_insert(state, &raw mut range);
        }
        if (*sh).flags as ::core::ffi::c_int & kSHUIWatched as ::core::ffi::c_int != 0 {
            range.kind = kDecorKindUIWatched as ::core::ffi::c_int as DecorRangeKind;
            range.data.ui.ns_id = ns;
            range.data.ui.mark_id = mark_id;
            range.data.ui.pos = (if (*sh).flags as ::core::ffi::c_int
                & kSHUIWatchedOverlay as ::core::ffi::c_int
                != 0
            {
                kVPosOverlay as ::core::ffi::c_int
            } else {
                kVPosEndOfLine as ::core::ffi::c_int
            }) as VirtTextPos;
            decor_range_insert(state, &raw mut range);
        }
    }
}

pub unsafe extern "C" fn decor_init_draw_col(
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut item: *mut DecorRange,
) {
    unsafe {
        let mut vt: *mut DecorVirtText =
            if (*item).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int {
                (*item).data.vt
            } else {
                ::core::ptr::null_mut::<DecorVirtText>()
            };
        let mut pos: VirtTextPos = decor_virt_pos_kind(item);
        if win_col < 0 as ::core::ffi::c_int
            && pos as ::core::ffi::c_uint
                != kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*item).draw_col = win_col;
        } else if pos as ::core::ffi::c_uint
            == kVPosOverlay as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*item).draw_col = if !vt.is_null()
                && (*vt).flags as ::core::ffi::c_int & kVTHide as ::core::ffi::c_int != 0
                && hidden as ::core::ffi::c_int != 0
            {
                INT_MIN
            } else {
                win_col
            };
        } else {
            (*item).draw_col = -1 as ::core::ffi::c_int;
        };
    }
}

pub unsafe extern "C" fn decor_recheck_draw_col(
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut state: *mut DecorState,
) {
    unsafe {
        let end: ::core::ffi::c_int = (*state).current_end;
        let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
        let slots: *mut DecorRangeSlot = (*state).slots.items;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < end {
            let r: *mut DecorRange =
                &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
            if (*r).draw_col == -3 as ::core::ffi::c_int {
                decor_init_draw_col(win_col, hidden, r);
            }
            i += 1;
        }
    }
}

pub unsafe extern "C" fn decor_redraw_col_impl(
    mut wp: *mut win_T,
    mut col: ::core::ffi::c_int,
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut state: *mut DecorState,
    mut max_col_last: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let buf: *mut buf_T = (*wp).w_buffer;
        let row: ::core::ffi::c_int = (*state).row;
        let mut col_last: ::core::ffi::c_int = max_col_last;
        let mut endpos: MTPos = MTPos { row: 0, col: 0 };
        loop {
            let mut mark: MTKey = marktree_itr_current(&raw mut (*state).itr as *mut MarkTreeIter);
            if mark.pos.row < 0 as int32_t || mark.pos.row > row as int32_t {
                break;
            }
            if mark.pos.row == row as int32_t && mark.pos.col > col as int32_t {
                col_last = (if (col_last as int32_t) < mark.pos.col - 1 as int32_t {
                    col_last as int32_t
                } else {
                    mark.pos.col - 1 as int32_t
                }) as ::core::ffi::c_int;
                break;
            } else {
                if !(mt_invalid(mark) as ::core::ffi::c_int != 0
                    || mt_end(mark) as ::core::ffi::c_int != 0
                    || !mt_decor_any(mark)
                    || !ns_in_win(mark.ns, wp))
                {
                    endpos = marktree_get_altpos(
                        &raw mut (*buf).b_marktree as *mut MarkTree,
                        mark,
                        ::core::ptr::null_mut::<MarkTreeIter>(),
                    );
                    decor_range_add_from_inline(
                        state,
                        mark.pos.row as ::core::ffi::c_int,
                        mark.pos.col as ::core::ffi::c_int,
                        endpos.row as ::core::ffi::c_int,
                        endpos.col as ::core::ffi::c_int,
                        mt_decor(mark),
                        false_0 != 0,
                        mark.ns,
                        mark.id,
                    );
                }
                marktree_itr_next(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    &raw mut (*state).itr as *mut MarkTreeIter,
                );
            }
        }
        let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
        let slots: *mut DecorRangeSlot = (*state).slots.items;
        let mut count: ::core::ffi::c_int = (*state).ranges_i.size as ::core::ffi::c_int;
        let mut cur_end: ::core::ffi::c_int = (*state).current_end;
        let mut fut_beg: ::core::ffi::c_int = (*state).future_begin;
        while fut_beg < count {
            let index: ::core::ffi::c_int = *indices.offset(fut_beg as isize);
            let r: *mut DecorRange = &raw mut (*slots.offset(index as isize)).range;
            if (*r).start_row > row || (*r).start_row == row && (*r).start_col > col {
                break;
            }
            let ordering: ::core::ffi::c_int = (*r).ordering;
            let priority: DecorPriorityInternal = (*r).priority_internal;
            let mut begin: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut end: ::core::ffi::c_int = cur_end;
            while begin < end {
                let mut mid: ::core::ffi::c_int = begin + (end - begin >> 1 as ::core::ffi::c_int);
                let mut mi: ::core::ffi::c_int = *indices.offset(mid as isize);
                let mut mr: *mut DecorRange = &raw mut (*slots.offset(mi as isize)).range;
                if (*mr).priority_internal < priority
                    || (*mr).priority_internal == priority && (*mr).ordering < ordering
                {
                    begin = mid + 1 as ::core::ffi::c_int;
                } else {
                    end = mid;
                }
            }
            let item: *mut ::core::ffi::c_int = indices.offset(begin as isize);
            memmove(
                item.offset(1 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                item as *const ::core::ffi::c_void,
                ((cur_end - begin) as size_t)
                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
            );
            *item = index;
            cur_end += 1;
            fut_beg += 1;
        }
        if fut_beg < count {
            let mut r_0: *mut DecorRange =
                &raw mut (*slots.offset(*indices.offset(fut_beg as isize) as isize)).range;
            if (*r_0).start_row == row {
                col_last = if col_last < (*r_0).start_col - 1 as ::core::ffi::c_int {
                    col_last
                } else {
                    (*r_0).start_col - 1 as ::core::ffi::c_int
                };
            }
        }
        let mut new_cur_end: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut conceal: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut conceal_char: schar_T = 0 as schar_T;
        let mut conceal_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut spell: TriState = kNone;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < cur_end {
            let index_0: ::core::ffi::c_int = *indices.offset(i as isize);
            let slot: *mut DecorRangeSlot = slots.offset(index_0 as isize);
            let r_1: *mut DecorRange = &raw mut (*slot).range;
            let mut keep: bool = false;
            if (*r_1).end_row < row || (*r_1).end_row == row && (*r_1).end_col <= col {
                keep = (*r_1).start_row >= row && decor_virt_pos(r_1) as ::core::ffi::c_int != 0;
            } else {
                keep = true_0 != 0;
                if (*r_1).end_row == row && (*r_1).end_col > col {
                    col_last = if col_last < (*r_1).end_col - 1 as ::core::ffi::c_int {
                        col_last
                    } else {
                        (*r_1).end_col - 1 as ::core::ffi::c_int
                    };
                }
                if (*r_1).attr_id > 0 as ::core::ffi::c_int {
                    attr = hl_combine_attr(attr, (*r_1).attr_id);
                }
                if (*r_1).kind as ::core::ffi::c_int == kDecorKindHighlight as ::core::ffi::c_int
                    && (*r_1).data.sh.flags as ::core::ffi::c_int & kSHConceal as ::core::ffi::c_int
                        != 0
                {
                    conceal = 1 as ::core::ffi::c_int;
                    if (*r_1).start_row == row && (*r_1).start_col == col {
                        let mut sh: *mut DecorSignHighlight = &raw mut (*r_1).data.sh;
                        conceal = 2 as ::core::ffi::c_int;
                        conceal_char = (*sh).text[0 as ::core::ffi::c_int as usize];
                        col_last = if col_last < (*r_1).start_col {
                            col_last
                        } else {
                            (*r_1).start_col
                        };
                        conceal_attr = (*r_1).attr_id;
                    }
                }
                if (*r_1).kind as ::core::ffi::c_int == kDecorKindHighlight as ::core::ffi::c_int {
                    if (*r_1).data.sh.flags as ::core::ffi::c_int & kSHSpellOn as ::core::ffi::c_int
                        != 0
                    {
                        spell = kTrue;
                    } else if (*r_1).data.sh.flags as ::core::ffi::c_int
                        & kSHSpellOff as ::core::ffi::c_int
                        != 0
                    {
                        spell = kFalse;
                    }
                    if !(*r_1).data.sh.url.is_null() {
                        attr = hl_add_url(attr, (*r_1).data.sh.url);
                    }
                }
            }
            if (*r_1).start_row == row
                && (*r_1).start_col <= col
                && decor_virt_pos(r_1) as ::core::ffi::c_int != 0
                && (*r_1).draw_col == -10 as ::core::ffi::c_int
            {
                decor_init_draw_col(win_col, hidden, r_1);
            }
            if keep {
                let c2rust_fresh4 = new_cur_end;
                new_cur_end = new_cur_end + 1;
                *indices.offset(c2rust_fresh4 as isize) = index_0;
            } else {
                if (*r_1).owned {
                    if (*r_1).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int
                    {
                        clear_virttext(&raw mut (*(*r_1).data.vt).data.virt_text);
                        xfree((*r_1).data.vt as *mut ::core::ffi::c_void);
                    } else if (*r_1).kind as ::core::ffi::c_int
                        == kDecorKindHighlight as ::core::ffi::c_int
                    {
                        xfree((*r_1).data.sh.url as *mut ::core::ffi::c_void);
                    }
                }
                let mut fi: *mut ::core::ffi::c_int = &raw mut (*state).free_slot_i;
                (*slot).next_free_i = *fi;
                *fi = index_0;
            }
            i += 1;
        }
        cur_end = new_cur_end;
        if fut_beg == count {
            count = cur_end;
            fut_beg = count;
        }
        (*state).ranges_i.size = count as size_t;
        (*state).future_begin = fut_beg;
        (*state).current_end = cur_end;
        (*state).col_last = col_last;
        (*state).current = attr;
        (*state).conceal = conceal;
        (*state).conceal_char = conceal_char;
        (*state).conceal_attr = conceal_attr;
        (*state).spell = spell;
        return attr;
    }
}

pub unsafe extern "C" fn decor_redraw_eol(
    mut wp: *mut win_T,
    mut state: *mut DecorState,
    mut eol_attr: *mut ::core::ffi::c_int,
    mut eol_col: ::core::ffi::c_int,
) -> bool {
    unsafe {
        decor_redraw_col(
            wp,
            MAXCOL as ::core::ffi::c_int,
            MAXCOL as ::core::ffi::c_int,
            false_0 != 0,
            state,
            MAXCOL as ::core::ffi::c_int,
        );
        (*state).eol_col = eol_col;
        let count: ::core::ffi::c_int = (*state).current_end;
        let indices: *mut ::core::ffi::c_int = (*state).ranges_i.items;
        let slots: *mut DecorRangeSlot = (*state).slots.items;
        let mut has_virt_pos: bool = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < count {
            let mut r: *mut DecorRange =
                &raw mut (*slots.offset(*indices.offset(i as isize) as isize)).range;
            has_virt_pos = has_virt_pos as ::core::ffi::c_int
                | ((*r).start_row == (*state).row && decor_virt_pos(r) as ::core::ffi::c_int != 0)
                    as ::core::ffi::c_int
                != 0;
            if (*r).kind as ::core::ffi::c_int == kDecorKindHighlight as ::core::ffi::c_int
                && (*r).data.sh.flags as ::core::ffi::c_int & kSHHlEol as ::core::ffi::c_int != 0
            {
                *eol_attr = hl_combine_attr(*eol_attr, (*r).attr_id);
            }
            i += 1;
        }
        return has_virt_pos;
    }
}

#[inline(always)]
pub unsafe fn decor_redraw_col(
    mut wp: *mut win_T,
    mut col: ::core::ffi::c_int,
    mut win_col: ::core::ffi::c_int,
    mut hidden: bool,
    mut state: *mut DecorState,
    mut max_col_last: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if col <= (*state).col_last {
            return (*state).current;
        }
        return decor_redraw_col_impl(wp, col, win_col, hidden, state, max_col_last);
    }
}
