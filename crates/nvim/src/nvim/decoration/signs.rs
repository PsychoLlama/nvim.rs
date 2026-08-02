//! Signs placed by extmarks: which ones a row shows, and how wide
//! the sign column has to be.
//!
//! [`decor_redraw_signs`] collects the signs overlapping one row, sorts
//! them by priority ([`sign_item_cmp`]) and hands the drawing code the
//! first few plus the winning line/number/cursorline highlights.
//! [`buf_signcols_count_range`] keeps `b_signcols`, the per-row histogram
//! `'signcolumn'`'s `auto:N` reads, in step as signs are added and
//! removed.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn may_force_numberwidth_recompute(
    mut buf: *mut buf_T,
    mut unplace: bool,
) {
    unsafe {
        let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !tp.is_null() {
            let mut wp: *mut win_T = if tp == curtab.get() {
                firstwin.get()
            } else {
                (*tp).tp_firstwin
            };
            while !wp.is_null() {
                if (*wp).w_buffer == buf
                    && (*wp).w_minscwidth == SCL_NUM
                    && ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                    && (unplace as ::core::ffi::c_int != 0
                        || (*wp).w_nrwidth_width < 2 as ::core::ffi::c_int)
                {
                    (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
                }
                wp = (*wp).w_next;
            }
            tp = (*tp).tp_next as *mut tabpage_T;
        }
    }
}

pub(crate) static sign_add_id: GlobalCell<::core::ffi::c_int> =
    GlobalCell::new(0 as ::core::ffi::c_int);

pub unsafe extern "C" fn buf_put_decor_sh(
    mut buf: *mut buf_T,
    mut sh: *mut DecorSignHighlight,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
) {
    unsafe {
        if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
            let c2rust_fresh1 = sign_add_id.get();
            sign_add_id.set(sign_add_id.get() + 1);
            (*sh).sign_add_id = c2rust_fresh1;
            if (*sh).text[0 as ::core::ffi::c_int as usize] != 0 {
                buf_signcols_count_range(buf, row1, row2, 1 as ::core::ffi::c_int, kFalse);
                may_force_numberwidth_recompute(buf, false_0 != 0);
            }
        }
    }
}

pub unsafe extern "C" fn buf_remove_decor_sh(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut sh: *mut DecorSignHighlight,
) {
    unsafe {
        if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
            if (*sh).text[0 as ::core::ffi::c_int as usize] != 0 {
                if buf_meta_total(buf, kMTMetaSignText) != 0 {
                    buf_signcols_count_range(buf, row1, row2, -1 as ::core::ffi::c_int, kFalse);
                } else {
                    may_force_numberwidth_recompute(buf, true_0 != 0);
                    (*buf).b_signcols.count[0 as ::core::ffi::c_int as usize] =
                        0 as ::core::ffi::c_int;
                    (*buf).b_signcols.max = 0 as ::core::ffi::c_int;
                }
            }
        }
    }
}

pub unsafe extern "C" fn sign_item_cmp(
    mut p1: *const ::core::ffi::c_void,
    mut p2: *const ::core::ffi::c_void,
) -> ::core::ffi::c_int {
    unsafe {
        let mut s1: *const SignItem = p1 as *mut SignItem;
        let mut s2: *const SignItem = p2 as *mut SignItem;
        if (*(*s1).sh).priority as ::core::ffi::c_int != (*(*s2).sh).priority as ::core::ffi::c_int
        {
            return if ((*(*s1).sh).priority as ::core::ffi::c_int)
                < (*(*s2).sh).priority as ::core::ffi::c_int
            {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        }
        if (*s1).id != (*s2).id {
            return if (*s1).id < (*s2).id {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        }
        if (*(*s1).sh).sign_add_id != (*(*s2).sh).sign_add_id {
            return if (*(*s1).sh).sign_add_id < (*(*s2).sh).sign_add_id {
                1 as ::core::ffi::c_int
            } else {
                -1 as ::core::ffi::c_int
            };
        }
        return 0 as ::core::ffi::c_int;
    }
}

pub(crate) static sign_filter: GlobalCell<[uint32_t; 5]> =
    GlobalCell::new([0, 0, kMTFilterSelect, kMTFilterSelect, 0]);

pub unsafe extern "C" fn decor_redraw_signs(
    mut wp: *mut win_T,
    mut buf: *mut buf_T,
    mut row: ::core::ffi::c_int,
    mut sattrs: *mut SignTextAttrs,
    mut line_id: *mut ::core::ffi::c_int,
    mut cul_id: *mut ::core::ffi::c_int,
    mut num_id: *mut ::core::ffi::c_int,
) {
    unsafe {
        if !buf_has_signs(buf) {
            return;
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
        let mut num_text: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1];
        let mut signs: C2Rust_Unnamed_27 = C2Rust_Unnamed_27 {
            size: 0 as size_t,
            capacity: 0 as size_t,
            items: ::core::ptr::null_mut::<SignItem>(),
        };
        marktree_itr_get_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            row,
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
        while marktree_itr_step_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            &raw mut pair,
        ) {
            if !mt_invalid(pair.start)
                && mt_decor_sign(pair.start) as ::core::ffi::c_int != 0
                && ns_in_win(pair.start.ns, wp) as ::core::ffi::c_int != 0
            {
                let mut sh: *mut DecorSignHighlight = decor_find_sign(mt_decor(pair.start));
                num_text += ((*sh).text[0 as ::core::ffi::c_int as usize] != NUL as schar_T)
                    as ::core::ffi::c_int;
                if signs.size == signs.capacity {
                    signs.capacity = if signs.capacity != 0 {
                        signs.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    signs.items = xrealloc(
                        signs.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<SignItem>().wrapping_mul(signs.capacity),
                    ) as *mut SignItem;
                } else {
                };
                let c2rust_fresh5 = signs.size;
                signs.size = signs.size.wrapping_add(1);
                *signs.items.offset(c2rust_fresh5 as isize) = SignItem {
                    sh: sh,
                    id: pair.start.id,
                };
            }
        }
        marktree_itr_step_out_filter(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            (sign_filter.ptr() as *const _) as MetaFilter,
        );
        while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            if mark.pos.row != row as int32_t {
                break;
            }
            if !mt_invalid(mark)
                && !mt_end(mark)
                && mt_decor_sign(mark) as ::core::ffi::c_int != 0
                && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0
            {
                let mut sh_0: *mut DecorSignHighlight = decor_find_sign(mt_decor(mark));
                num_text += ((*sh_0).text[0 as ::core::ffi::c_int as usize] != NUL as schar_T)
                    as ::core::ffi::c_int;
                if signs.size == signs.capacity {
                    signs.capacity = if signs.capacity != 0 {
                        signs.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    signs.items = xrealloc(
                        signs.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<SignItem>().wrapping_mul(signs.capacity),
                    ) as *mut SignItem;
                } else {
                };
                let c2rust_fresh6 = signs.size;
                signs.size = signs.size.wrapping_add(1);
                *signs.items.offset(c2rust_fresh6 as isize) = SignItem {
                    sh: sh_0,
                    id: mark.id,
                };
            }
            marktree_itr_next_filter(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
                row + 1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (sign_filter.ptr() as *const _) as MetaFilter,
            );
        }
        if signs.size != 0 {
            let mut width: ::core::ffi::c_int = if (*wp).w_minscwidth == SCL_NUM {
                1 as ::core::ffi::c_int
            } else {
                (*wp).w_scwidth
            };
            let mut len: ::core::ffi::c_int = if width < num_text { width } else { num_text };
            let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            qsort(
                signs.items.offset(0 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_void,
                signs.size,
                ::core::mem::size_of::<SignItem>(),
                Some(
                    sign_item_cmp
                        as unsafe extern "C" fn(
                            *const ::core::ffi::c_void,
                            *const ::core::ffi::c_void,
                        ) -> ::core::ffi::c_int,
                ),
            );
            let mut i: size_t = 0 as size_t;
            while i < signs.size {
                let mut sh_1: *mut DecorSignHighlight = (*signs.items.offset(i as isize)).sh;
                if !sattrs.is_null()
                    && idx < len
                    && (*sh_1).text[0 as ::core::ffi::c_int as usize] != 0
                {
                    memcpy(
                        &raw mut (*sattrs.offset(idx as isize)).text as *mut schar_T
                            as *mut ::core::ffi::c_void,
                        &raw mut (*sh_1).text as *mut schar_T as *const ::core::ffi::c_void,
                        (SIGN_WIDTH as ::core::ffi::c_int as size_t)
                            .wrapping_mul(::core::mem::size_of::<sattr_T>()),
                    );
                    let c2rust_fresh7 = idx;
                    idx = idx + 1;
                    (*sattrs.offset(c2rust_fresh7 as isize)).hl_id = (*sh_1).hl_id;
                }
                if !num_id.is_null() && *num_id <= 0 as ::core::ffi::c_int {
                    *num_id = (*sh_1).number_hl_id;
                }
                if !line_id.is_null() && *line_id <= 0 as ::core::ffi::c_int {
                    *line_id = (*sh_1).line_hl_id;
                }
                if !cul_id.is_null() && *cul_id <= 0 as ::core::ffi::c_int {
                    *cul_id = (*sh_1).cursorline_hl_id;
                }
                i = i.wrapping_add(1);
            }
            xfree(signs.items as *mut ::core::ffi::c_void);
            signs.capacity = 0 as size_t;
            signs.size = signs.capacity;
            signs.items = ::core::ptr::null_mut::<SignItem>();
        }
    }
}

pub unsafe extern "C" fn decor_find_sign(mut decor: DecorInline) -> *mut DecorSignHighlight {
    unsafe {
        if !decor.ext {
            return ::core::ptr::null_mut::<DecorSignHighlight>();
        }
        let mut decor_id: uint32_t = decor.data.ext.sh_idx;
        loop {
            if decor_id == DECOR_ID_INVALID as uint32_t {
                return ::core::ptr::null_mut::<DecorSignHighlight>();
            }
            let mut sh: *mut DecorSignHighlight = decor_item(decor_id);
            if (*sh).flags as ::core::ffi::c_int & kSHIsSign as ::core::ffi::c_int != 0 {
                return sh;
            }
            decor_id = (*sh).next;
        }
    }
}

pub(crate) static signtext_filter: GlobalCell<[uint32_t; 5]> =
    GlobalCell::new([0, 0, 0, kMTFilterSelect, 0]);

pub unsafe extern "C" fn buf_signcols_count_range(
    mut buf: *mut buf_T,
    mut row1: ::core::ffi::c_int,
    mut row2: ::core::ffi::c_int,
    mut add: ::core::ffi::c_int,
    mut clear: TriState,
) {
    unsafe {
        if !(*buf).b_signcols.autom || row2 < row1 || buf_meta_total(buf, kMTMetaSignText) == 0 {
            return;
        }
        let mut count: *mut ::core::ffi::c_int = xcalloc(
            (row2 + 1 as ::core::ffi::c_int - row1) as size_t,
            ::core::mem::size_of::<::core::ffi::c_int>(),
        ) as *mut ::core::ffi::c_int;
        let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos { row: 0, col: 0 },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }; 1];
        let mut pair: MTPair = MTPair {
            start: MTKey {
                pos: MTPos {
                    row: 0 as int32_t,
                    col: 0,
                },
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
        marktree_itr_get_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            row1,
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
        while marktree_itr_step_overlap(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            &raw mut pair,
        ) {
            if pair.start.flags as ::core::ffi::c_int & MT_FLAG_DECOR_SIGNTEXT != 0
                && !mt_invalid(pair.start)
            {
                let mut i: ::core::ffi::c_int = row1;
                while i as int32_t
                    <= (if (row2 as int32_t) < pair.end_pos.row {
                        row2 as int32_t
                    } else {
                        pair.end_pos.row
                    })
                {
                    *count.offset((i - row1) as isize) += 1;
                    i += 1;
                }
            }
        }
        marktree_itr_step_out_filter(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            (signtext_filter.ptr() as *const _) as MetaFilter,
        );
        while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            if mark.pos.row > row2 as int32_t {
                break;
            }
            if mark.flags as ::core::ffi::c_int & MT_FLAG_DECOR_SIGNTEXT != 0
                && !mt_invalid(mark)
                && !mt_end(mark)
            {
                let mut end: MTPos = marktree_get_altpos(
                    &raw mut (*buf).b_marktree as *mut MarkTree,
                    mark,
                    ::core::ptr::null_mut::<MarkTreeIter>(),
                );
                let mut i_0: ::core::ffi::c_int = mark.pos.row as ::core::ffi::c_int;
                while i_0 as int32_t
                    <= (if (row2 as int32_t) < end.row {
                        row2 as int32_t
                    } else {
                        end.row
                    })
                {
                    *count.offset((i_0 - row1) as isize) += 1;
                    i_0 += 1;
                }
            }
            marktree_itr_next_filter(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
                row2 + 1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (signtext_filter.ptr() as *const _) as MetaFilter,
            );
        }
        let mut i_1: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_1 < row2 + 1 as ::core::ffi::c_int - row1 {
            let mut prevwidth: ::core::ffi::c_int =
                if (SIGN_SHOW_MAX as ::core::ffi::c_int) < *count.offset(i_1 as isize) - add {
                    SIGN_SHOW_MAX as ::core::ffi::c_int
                } else {
                    *count.offset(i_1 as isize) - add
                };
            if clear as ::core::ffi::c_int != kNone as ::core::ffi::c_int
                && prevwidth > 0 as ::core::ffi::c_int
            {
                (*buf).b_signcols.count[(prevwidth - 1 as ::core::ffi::c_int) as usize] -= 1;
                '_c2rust_label: {
                    if (*buf).b_signcols.count[(prevwidth - 1 as ::core::ffi::c_int) as usize]
                        >= 0 as ::core::ffi::c_int
                    {
                    } else {
                        __assert_fail(
                            b"buf->b_signcols.count[prevwidth - 1] >= 0\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            1078 as ::core::ffi::c_uint,
                            b"void buf_signcols_count_range(buf_T *, int, int, int, TriState)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
            }
            let mut width: ::core::ffi::c_int =
                if (SIGN_SHOW_MAX as ::core::ffi::c_int) < *count.offset(i_1 as isize) {
                    SIGN_SHOW_MAX as ::core::ffi::c_int
                } else {
                    *count.offset(i_1 as isize)
                };
            if clear as ::core::ffi::c_int != kTrue as ::core::ffi::c_int
                && width > 0 as ::core::ffi::c_int
            {
                (*buf).b_signcols.count[(width - 1 as ::core::ffi::c_int) as usize] += 1;
                if width > (*buf).b_signcols.max {
                    (*buf).b_signcols.max = width;
                }
            }
            i_1 += 1;
        }
        xfree(count as *mut ::core::ffi::c_void);
    }
}
