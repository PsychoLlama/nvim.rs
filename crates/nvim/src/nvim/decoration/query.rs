//! Asking the marktree about a row without drawing it.
//!
//! The three questions the layout code needs answered before it knows how
//! tall a line is: does it have virtual text ([`decor_find_virttext`]), is
//! it concealed ([`decor_conceal_line`]), and how many virtual lines does
//! it carry ([`decor_virt_lines`]).

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn next_virt_text_chunk(
    mut vt: VirtText,
    mut pos: *mut size_t,
    mut attr: *mut ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut text: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        while text.is_null() && *pos < vt.size {
            text = (*vt.items.offset(*pos as isize)).text;
            let mut hl_id: ::core::ffi::c_int = (*vt.items.offset(*pos as isize)).hl_id;
            if hl_id >= 0 as ::core::ffi::c_int {
                *attr = if *attr > 0 as ::core::ffi::c_int {
                    *attr
                } else {
                    0 as ::core::ffi::c_int
                };
                if hl_id > 0 as ::core::ffi::c_int {
                    *attr = hl_combine_attr(*attr, syn_id2attr(hl_id));
                }
            }
            *pos = (*pos).wrapping_add(1);
        }
        return text;
    }
}

pub unsafe extern "C" fn decor_find_virttext(
    mut buf: *mut buf_T,
    mut row: ::core::ffi::c_int,
    mut ns_id: uint64_t,
) -> *mut DecorVirtText {
    unsafe {
        let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos {
                row: 0 as int32_t,
                col: 0,
            },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }];
        marktree_itr_get(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            row as int32_t,
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
        let mut decor: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
        loop {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            if mark.pos.row < 0 as int32_t || mark.pos.row > row as int32_t {
                break;
            }
            if !mt_invalid(mark) {
                decor = mt_decor_virt(mark);
                while !decor.is_null()
                    && (*decor).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0
                {
                    decor = (*decor).next;
                }
                if (ns_id == 0 as uint64_t || ns_id == mark.ns as uint64_t) && !decor.is_null() {
                    return decor;
                }
            }
            marktree_itr_next(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
            );
        }
        return ::core::ptr::null_mut::<DecorVirtText>();
    }
}

pub(crate) static conceal_filter: GlobalCell<[uint32_t; 5]> =
    GlobalCell::new([0, 0, 0, 0, kMTFilterSelect]);

pub unsafe extern "C" fn decor_conceal_line(
    mut wp: *mut win_T,
    mut row: ::core::ffi::c_int,
    mut check_cursor: bool,
) -> bool {
    unsafe {
        if row < 0 as ::core::ffi::c_int
            || (*wp).w_onebuf_opt.wo_cole < 2 as OptInt
            || !check_cursor
                && wp == curwin.get()
                && row as linenr_T + 1 as linenr_T == (*wp).w_cursor.lnum
                && !conceal_cursor_line(wp)
        {
            return false_0 != 0;
        }
        if buf_meta_total((*wp).w_buffer, kMTMetaConcealLines) == 0 {
            return decor_providers_invoke_conceal_line(wp, row);
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
        marktree_itr_get_overlap(
            &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
            row,
            0 as ::core::ffi::c_int,
            &raw mut itr as *mut MarkTreeIter,
        );
        while marktree_itr_step_overlap(
            &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            &raw mut pair,
        ) {
            if mt_conceal_lines(pair.start) as ::core::ffi::c_int != 0
                && ns_in_win(pair.start.ns, wp) as ::core::ffi::c_int != 0
            {
                return true_0 != 0;
            }
        }
        marktree_itr_step_out_filter(
            &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
            &raw mut itr as *mut MarkTreeIter,
            (conceal_filter.ptr() as *const _) as MetaFilter,
        );
        while !(*(&raw mut itr as *mut MarkTreeIter)).x.is_null() {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            if mark.pos.row > row as int32_t {
                break;
            }
            if mt_conceal_lines(mark) as ::core::ffi::c_int != 0
                && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0
            {
                return true_0 != 0;
            }
            marktree_itr_next_filter(
                &raw mut (*(*wp).w_buffer).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
                row + 1 as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                (conceal_filter.ptr() as *const _) as MetaFilter,
            );
        }
        return decor_providers_invoke_conceal_line(wp, row);
    }
}

pub unsafe extern "C" fn win_lines_concealed(mut wp: *mut win_T) -> bool {
    unsafe {
        return hasAnyFolding(wp) != 0 || (*wp).w_onebuf_opt.wo_cole >= 2 as OptInt;
    }
}

pub(crate) static lines_filter: GlobalCell<[uint32_t; 5]> =
    GlobalCell::new([0, kMTFilterSelect, 0, 0, 0]);

pub unsafe extern "C" fn decor_virt_lines(
    mut wp: *mut win_T,
    mut start_row: ::core::ffi::c_int,
    mut end_row: ::core::ffi::c_int,
    mut num_below: *mut ::core::ffi::c_int,
    mut lines: *mut VirtLines,
    mut apply_folds: bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut buf: *mut buf_T = (*wp).w_buffer;
        if buf_meta_total(buf, kMTMetaLines) == 0 {
            return 0 as ::core::ffi::c_int;
        }
        let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
            pos: MTPos {
                row: 0 as int32_t,
                col: 0,
            },
            lvl: 0,
            x: ::core::ptr::null_mut::<MTNode>(),
            i: 0,
            s: [C2Rust_Unnamed_19 { oldcol: 0, i: 0 }; 20],
            intersect_idx: 0,
            intersect_pos: MTPos { row: 0, col: 0 },
            intersect_pos_x: MTPos { row: 0, col: 0 },
        }];
        if !marktree_itr_get_filter(
            &raw mut (*buf).b_marktree as *mut MarkTree,
            if start_row - 1 as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                start_row as int32_t - 1 as int32_t
            } else {
                0 as int32_t
            },
            0 as ::core::ffi::c_int,
            end_row,
            0 as ::core::ffi::c_int,
            (lines_filter.ptr() as *const _) as MetaFilter,
            &raw mut itr as *mut MarkTreeIter,
        ) {
            return 0 as ::core::ffi::c_int;
        }
        '_c2rust_label: {
            if start_row >= 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"start_row >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/decoration.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    1138 as ::core::ffi::c_uint,
                    b"int decor_virt_lines(win_T *, int, int, int *, VirtLines *, _Bool)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut virt_lines: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        loop {
            let mut mark: MTKey = marktree_itr_current(&raw mut itr as *mut MarkTreeIter);
            let mut vt: *mut DecorVirtText = mt_decor_virt(mark);
            if !mt_invalid(mark) && ns_in_win(mark.ns, wp) as ::core::ffi::c_int != 0 {
                while !vt.is_null() {
                    if (*vt).flags as ::core::ffi::c_int & kVTIsLines as ::core::ffi::c_int != 0 {
                        let mut above: bool = (*vt).flags as ::core::ffi::c_int
                            & kVTLinesAbove as ::core::ffi::c_int
                            != 0;
                        let mut mrow: ::core::ffi::c_int = mark.pos.row as ::core::ffi::c_int;
                        let mut draw_row: ::core::ffi::c_int = mrow
                            + (if above as ::core::ffi::c_int != 0 {
                                0 as ::core::ffi::c_int
                            } else {
                                1 as ::core::ffi::c_int
                            });
                        if draw_row >= start_row
                            && draw_row < end_row
                            && (!apply_folds
                                || !(hasFolding(
                                    wp,
                                    mrow as linenr_T + 1 as linenr_T,
                                    ::core::ptr::null_mut::<linenr_T>(),
                                    ::core::ptr::null_mut::<linenr_T>(),
                                ) as ::core::ffi::c_int
                                    != 0
                                    || decor_conceal_line(wp, mrow, false_0 != 0)
                                        as ::core::ffi::c_int
                                        != 0))
                        {
                            virt_lines += (*vt).data.virt_lines.size as ::core::ffi::c_int;
                            if !lines.is_null() {
                                if (*vt).data.virt_lines.size > 0 as size_t {
                                    if (*lines).capacity
                                        < (*lines).size.wrapping_add((*vt).data.virt_lines.size)
                                    {
                                        (*lines).capacity =
                                            (*lines).size.wrapping_add((*vt).data.virt_lines.size);
                                        (*lines).capacity = (*lines).capacity.wrapping_sub(1);
                                        (*lines).capacity |=
                                            (*lines).capacity >> 1 as ::core::ffi::c_int;
                                        (*lines).capacity |=
                                            (*lines).capacity >> 2 as ::core::ffi::c_int;
                                        (*lines).capacity |=
                                            (*lines).capacity >> 4 as ::core::ffi::c_int;
                                        (*lines).capacity |=
                                            (*lines).capacity >> 8 as ::core::ffi::c_int;
                                        (*lines).capacity |=
                                            (*lines).capacity >> 16 as ::core::ffi::c_int;
                                        (*lines).capacity = (*lines).capacity.wrapping_add(1);
                                        (*lines).items = xrealloc(
                                            (*lines).items as *mut ::core::ffi::c_void,
                                            ::core::mem::size_of::<virt_line>()
                                                .wrapping_mul((*lines).capacity),
                                        )
                                            as *mut virt_line;
                                    }
                                    '_c2rust_label_0: {
                                        if !(*lines).items.is_null() {
                                        } else {
                                            __assert_fail(
                                            b"(*lines).items\0".as_ptr() as *const ::core::ffi::c_char,
                                            b"src/nvim/decoration.rs\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                            1155 as ::core::ffi::c_uint,
                                            b"int decor_virt_lines(win_T *, int, int, int *, VirtLines *, _Bool)\0"
                                                .as_ptr() as *const ::core::ffi::c_char,
                                        );
                                        }
                                    };
                                    memcpy(
                                        (*lines).items.offset((*lines).size as isize)
                                            as *mut ::core::ffi::c_void,
                                        (*vt).data.virt_lines.items as *const ::core::ffi::c_void,
                                        ::core::mem::size_of::<virt_line>()
                                            .wrapping_mul((*vt).data.virt_lines.size),
                                    );
                                    (*lines).size =
                                        (*lines).size.wrapping_add((*vt).data.virt_lines.size);
                                }
                            }
                            if !num_below.is_null() && !above {
                                *num_below += (*vt).data.virt_lines.size as ::core::ffi::c_int;
                            }
                        }
                    }
                    vt = (*vt).next;
                }
            }
            if !marktree_itr_next_filter(
                &raw mut (*buf).b_marktree as *mut MarkTree,
                &raw mut itr as *mut MarkTreeIter,
                end_row,
                0 as ::core::ffi::c_int,
                (lines_filter.ptr() as *const _) as MetaFilter,
            ) {
                break;
            }
        }
        return virt_lines;
    }
}
