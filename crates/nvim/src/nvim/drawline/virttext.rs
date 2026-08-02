//! Virtual text — `nvim_buf_set_extmark`'s `virt_text`.
//!
//! Two shapes, and they reach the line buffer by different routes.
//! [`draw_virt_text`] runs once after the text of a screen line has been laid
//! out and paints the texts that are positioned relative to the *line*: at the
//! end of it, right-aligned in the window, or at a fixed window column. Inline
//! virtual text is instead fed to `win_line`'s character loop as if it were
//! buffer text, which is what [`handle_inline_virtual_text`] does — including
//! dropping or partially skipping a chunk that starts left of the first visible
//! column.
//!
//! [`draw_virt_text_item`] paints one text through the `'hl_mode'` blend, and
//! [`line_putchar`] is the single-character primitive under both of them.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn line_putchar(
    mut buf: *mut buf_T,
    mut pp: *mut *const ::core::ffi::c_char,
    mut dest: *mut schar_T,
    mut maxcells: ::core::ffi::c_int,
    mut vcol: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        '_c2rust_label: {
            if *dest.offset(0 as ::core::ffi::c_int as isize) != 0 as schar_T {
            } else {
                __assert_fail(
                    b"dest[0] != 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    235 as ::core::ffi::c_uint,
                    b"int line_putchar(buf_T *, const char **, schar_T *, int, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        let mut p: *const ::core::ffi::c_char = *pp;
        let mut cells: ::core::ffi::c_int = utf_ptr2cells(p);
        let mut c_len: ::core::ffi::c_int = utfc_ptr2len(p);
        '_c2rust_label_0: {
            if maxcells > 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                    b"maxcells > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    240 as ::core::ffi::c_uint,
                    b"int line_putchar(buf_T *, const char **, schar_T *, int, int)\0".as_ptr()
                        as *const ::core::ffi::c_char,
                );
            }
        };
        if cells > maxcells {
            *dest.offset(0 as ::core::ffi::c_int as isize) = ' ' as ::core::ffi::c_int as schar_T;
            return 1 as ::core::ffi::c_int;
        }
        if *p as ::core::ffi::c_int == TAB {
            cells = tabstop_padding(vcol as colnr_T, (*buf).b_p_ts, (*buf).b_p_vts_array);
            cells = if cells < maxcells { cells } else { maxcells };
        }
        if cells < maxcells && *dest.offset(cells as isize) == 0 as schar_T {
            *dest.offset(cells as isize) = ' ' as ::core::ffi::c_int as schar_T;
        }
        if *p as ::core::ffi::c_int == TAB {
            let mut c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while c < cells {
                *dest.offset(c as isize) = ' ' as ::core::ffi::c_int as schar_T;
                c += 1;
            }
        } else {
            let mut u8c: ::core::ffi::c_int = 0;
            *dest.offset(0 as ::core::ffi::c_int as isize) = utfc_ptr2schar(p, &raw mut u8c);
            if cells > 1 as ::core::ffi::c_int {
                *dest.offset(1 as ::core::ffi::c_int as isize) = 0 as schar_T;
            }
        }
        *pp = (*pp).offset(c_len as isize);
        return cells;
    }
}

pub(crate) unsafe extern "C" fn draw_virt_text(
    mut wp: *mut win_T,
    mut buf: *mut buf_T,
    mut col_off: ::core::ffi::c_int,
    mut end_col: *mut ::core::ffi::c_int,
    mut win_row: ::core::ffi::c_int,
) {
    unsafe {
        let state: *mut DecorState = decor_state.ptr();
        let max_col: ::core::ffi::c_int = (*wp).w_view_width;
        let mut right_pos: ::core::ffi::c_int = max_col;
        let do_eol: bool = (*state).eol_col > -1 as ::core::ffi::c_int;
        let end: ::core::ffi::c_int = (*state).current_end;
        let mut totalWidthOfEolRightAlignedVirtText: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < end {
            let mut item: *mut DecorRange = decor_range_at(state, i);
            if (*item).start_row == (*state).row && decor_virt_pos(item) as ::core::ffi::c_int != 0
            {
                let mut vt: *mut DecorVirtText = ::core::ptr::null_mut::<DecorVirtText>();
                if (*item).kind as ::core::ffi::c_int == kDecorKindVirtText as ::core::ffi::c_int {
                    '_c2rust_label: {
                        if !(*item).data.vt.is_null() {
                        } else {
                            __assert_fail(
                                b"item->data.vt\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                293 as ::core::ffi::c_uint,
                                b"void draw_virt_text(win_T *, buf_T *, int, int *, int)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    vt = (*item).data.vt;
                }
                if decor_virt_pos(item) as ::core::ffi::c_int != 0
                    && (*item).draw_col == -1 as ::core::ffi::c_int
                {
                    let mut updated: bool = true_0 != 0;
                    let mut pos: VirtTextPos = decor_virt_pos_kind(item);
                    if do_eol as ::core::ffi::c_int != 0
                        && pos as ::core::ffi::c_uint
                            == kVPosEndOfLineRightAlign as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        let mut eolOffset: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        if totalWidthOfEolRightAlignedVirtText == 0 as ::core::ffi::c_int {
                            let mut j: ::core::ffi::c_int = i;
                            while j < end {
                                let mut lookaheadItem: *mut DecorRange = decor_range_at(state, j);
                                if !((*lookaheadItem).start_row != (*state).row
                                    || !decor_virt_pos(lookaheadItem)
                                    || (*lookaheadItem).draw_col != -1 as ::core::ffi::c_int)
                                {
                                    let mut lookaheadVt: *mut DecorVirtText =
                                        ::core::ptr::null_mut::<DecorVirtText>();
                                    if (*lookaheadItem).kind as ::core::ffi::c_int
                                        == kDecorKindVirtText as ::core::ffi::c_int
                                    {
                                        '_c2rust_label_0: {
                                            if !(*lookaheadItem).data.vt.is_null() {
                                            } else {
                                                __assert_fail(
                                                b"lookaheadItem->data.vt\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"src/nvim/drawline.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                317 as ::core::ffi::c_uint,
                                                b"void draw_virt_text(win_T *, buf_T *, int, int *, int)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                            }
                                        };
                                        lookaheadVt = (*lookaheadItem).data.vt;
                                    }
                                    if decor_virt_pos_kind(lookaheadItem) as ::core::ffi::c_uint
                                        == kVPosEndOfLineRightAlign as ::core::ffi::c_int
                                            as ::core::ffi::c_uint
                                    {
                                        totalWidthOfEolRightAlignedVirtText +=
                                            (*lookaheadVt).width + 1 as ::core::ffi::c_int;
                                    }
                                }
                                j += 1;
                            }
                            totalWidthOfEolRightAlignedVirtText -= 1;
                            if totalWidthOfEolRightAlignedVirtText <= right_pos - (*state).eol_col {
                                eolOffset = right_pos
                                    - totalWidthOfEolRightAlignedVirtText
                                    - (*state).eol_col;
                            }
                        }
                        (*item).draw_col = (*state).eol_col + eolOffset;
                    } else if pos as ::core::ffi::c_uint
                        == kVPosRightAlign as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        right_pos -= (*vt).width;
                        (*item).draw_col = right_pos;
                    } else if pos as ::core::ffi::c_uint
                        == kVPosEndOfLine as ::core::ffi::c_int as ::core::ffi::c_uint
                        && do_eol as ::core::ffi::c_int != 0
                    {
                        (*item).draw_col = (*state).eol_col;
                    } else if pos as ::core::ffi::c_uint
                        == kVPosWinCol as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        (*item).draw_col = if col_off + (*vt).col > 0 as ::core::ffi::c_int {
                            col_off + (*vt).col
                        } else {
                            0 as ::core::ffi::c_int
                        };
                    } else {
                        updated = false_0 != 0;
                    }
                    if updated as ::core::ffi::c_int != 0
                        && ((*item).draw_col < 0 as ::core::ffi::c_int
                            || (*item).draw_col >= (*wp).w_view_width)
                    {
                        (*item).draw_col = INT_MIN;
                    }
                }
                if (*item).draw_col >= 0 as ::core::ffi::c_int {
                    if (*item).kind as ::core::ffi::c_int
                        == kDecorKindUIWatched as ::core::ffi::c_int
                    {
                        let mut m: WinExtmark = WinExtmark {
                            ns_id: (*item).data.ui.ns_id as NS,
                            mark_id: (*item).data.ui.mark_id as uint64_t,
                            win_row: win_row,
                            win_col: (*item).draw_col,
                        };
                        if (*win_extmark_arr.ptr()).size == (*win_extmark_arr.ptr()).capacity {
                            (*win_extmark_arr.ptr()).capacity =
                                if (*win_extmark_arr.ptr()).capacity != 0 {
                                    (*win_extmark_arr.ptr()).capacity << 1 as ::core::ffi::c_int
                                } else {
                                    8 as size_t
                                };
                            (*win_extmark_arr.ptr()).items = xrealloc(
                                (*win_extmark_arr.ptr()).items as *mut ::core::ffi::c_void,
                                ::core::mem::size_of::<WinExtmark>()
                                    .wrapping_mul((*win_extmark_arr.ptr()).capacity),
                            )
                                as *mut WinExtmark;
                        } else {
                        };
                        let c2rust_fresh4 = (*win_extmark_arr.ptr()).size;
                        (*win_extmark_arr.ptr()).size =
                            (*win_extmark_arr.ptr()).size.wrapping_add(1);
                        *(*win_extmark_arr.ptr())
                            .items
                            .offset(c2rust_fresh4 as isize) = m;
                    }
                    if !vt.is_null() {
                        let mut vcol: ::core::ffi::c_int = (*item).draw_col - col_off;
                        let mut col: ::core::ffi::c_int = draw_virt_text_item(
                            buf,
                            (*item).draw_col,
                            (*vt).data.virt_text,
                            (*vt).hl_mode as HlMode,
                            max_col,
                            vcol,
                            0 as ::core::ffi::c_int,
                        );
                        if do_eol as ::core::ffi::c_int != 0
                            && ((*vt).pos as ::core::ffi::c_uint
                                == kVPosEndOfLine as ::core::ffi::c_int as ::core::ffi::c_uint
                                || (*vt).pos as ::core::ffi::c_uint
                                    == kVPosEndOfLineRightAlign as ::core::ffi::c_int
                                        as ::core::ffi::c_uint)
                        {
                            (*state).eol_col = col + 1 as ::core::ffi::c_int;
                        }
                        *end_col = if *end_col > col { *end_col } else { col };
                    }
                    if vt.is_null()
                        || (*vt).flags as ::core::ffi::c_int
                            & kVTRepeatLinebreak as ::core::ffi::c_int
                            == 0
                    {
                        (*item).draw_col = INT_MIN;
                    }
                }
            }
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn draw_virt_text_item(
    mut buf: *mut buf_T,
    mut col: ::core::ffi::c_int,
    mut vt: VirtText,
    mut hl_mode: HlMode,
    mut max_col: ::core::ffi::c_int,
    mut vcol: ::core::ffi::c_int,
    mut skip_cells: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut virt_str: *const ::core::ffi::c_char = b"\0".as_ptr() as *const ::core::ffi::c_char;
        let mut virt_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut virt_pos: size_t = 0 as size_t;
        while col < max_col {
            if skip_cells >= 0 as ::core::ffi::c_int && *virt_str as ::core::ffi::c_int == NUL {
                if virt_pos >= vt.size {
                    break;
                }
                virt_attr = 0 as ::core::ffi::c_int;
                virt_str = next_virt_text_chunk(vt, &raw mut virt_pos, &raw mut virt_attr);
                if virt_str.is_null() {
                    break;
                }
            }
            while skip_cells > 0 as ::core::ffi::c_int && *virt_str as ::core::ffi::c_int != NUL {
                let mut c_len: ::core::ffi::c_int = utfc_ptr2len(virt_str);
                let mut cells: ::core::ffi::c_int = if *virt_str as ::core::ffi::c_int == TAB {
                    tabstop_padding(vcol as colnr_T, (*buf).b_p_ts, (*buf).b_p_vts_array)
                } else {
                    utf_ptr2cells(virt_str)
                };
                skip_cells -= cells;
                vcol += cells;
                virt_str = virt_str.offset(c_len as isize);
            }
            let mut draw_str: *const ::core::ffi::c_char = if skip_cells < 0 as ::core::ffi::c_int {
                b" \0".as_ptr() as *const ::core::ffi::c_char
            } else {
                virt_str
            };
            if *draw_str as ::core::ffi::c_int == NUL {
                continue;
            }
            '_c2rust_label: {
                if skip_cells <= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"skip_cells <= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        407 as ::core::ffi::c_uint,
                        b"int draw_virt_text_item(buf_T *, int, VirtText, HlMode, int, int, int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut attr: ::core::ffi::c_int = 0;
            let mut through: bool = false_0 != 0;
            if hl_mode as ::core::ffi::c_uint
                == kHlModeCombine as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                attr = hl_combine_attr(
                    *(*linebuf_attr.ptr()).offset(col as isize) as ::core::ffi::c_int,
                    virt_attr,
                );
            } else if hl_mode as ::core::ffi::c_uint
                == kHlModeBlend as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                through = *draw_str as ::core::ffi::c_int == ' ' as ::core::ffi::c_int;
                attr = hl_blend_attrs(
                    *(*linebuf_attr.ptr()).offset(col as isize) as ::core::ffi::c_int,
                    virt_attr,
                    &mut through,
                );
            } else {
                attr = virt_attr;
            }
            let mut dummy: [schar_T; 2] = [
                ' ' as ::core::ffi::c_int as schar_T,
                ' ' as ::core::ffi::c_int as schar_T,
            ];
            let mut maxcells: ::core::ffi::c_int = max_col - col;
            if !through && *(*linebuf_char.ptr()).offset(col as isize) == 0 as schar_T {
                '_c2rust_label_0: {
                    if col > 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                        b"col > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        422 as ::core::ffi::c_uint,
                        b"int draw_virt_text_item(buf_T *, int, VirtText, HlMode, int, int, int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                *(*linebuf_char.ptr()).offset((col - 1 as ::core::ffi::c_int) as isize) =
                    ' ' as ::core::ffi::c_int as schar_T;
                *(*linebuf_char.ptr()).offset(col as isize) = ' ' as ::core::ffi::c_int as schar_T;
            }
            let mut cells_0: ::core::ffi::c_int = line_putchar(
                buf,
                &raw mut draw_str,
                if through as ::core::ffi::c_int != 0 {
                    &raw mut dummy as *mut schar_T
                } else {
                    (*linebuf_char.ptr()).offset(col as isize)
                },
                maxcells,
                vcol,
            );
            let mut c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while c < cells_0 {
                *(*linebuf_attr.ptr()).offset(col as isize) = attr as sattr_T;
                col += 1;
                c += 1;
            }
            if skip_cells < 0 as ::core::ffi::c_int {
                skip_cells += 1;
            } else {
                vcol += cells_0;
                virt_str = draw_str;
            }
        }
        return col;
    }
}

pub(crate) unsafe extern "C" fn has_more_inline_virt(
    mut wlv: *mut winlinevars_T,
    mut v: ptrdiff_t,
) -> bool {
    unsafe {
        if (*wlv).virt_inline_i < (*wlv).virt_inline.size {
            return true_0 != 0;
        }
        let count: ::core::ffi::c_int = decor_range_count(decor_state.ptr());
        let cur_end: ::core::ffi::c_int = (*decor_state.ptr()).current_end;
        let fut_beg: ::core::ffi::c_int = (*decor_state.ptr()).future_begin;
        let beg_pos: [::core::ffi::c_int; 2] = [0 as ::core::ffi::c_int, fut_beg];
        let end_pos: [::core::ffi::c_int; 2] = [cur_end, count];
        let mut pos_i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while pos_i < 2 as ::core::ffi::c_int {
            let mut i: ::core::ffi::c_int = beg_pos[pos_i as usize];
            while i < end_pos[pos_i as usize] {
                let mut item: *mut DecorRange = decor_range_at(decor_state.ptr(), i);
                if !((*item).start_row != (*decor_state.ptr()).row
                    || (*item).kind as ::core::ffi::c_int
                        != kDecorKindVirtText as ::core::ffi::c_int
                    || (*(*item).data.vt).pos as ::core::ffi::c_uint
                        != kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                    || (*(*item).data.vt).width == 0 as ::core::ffi::c_int)
                {
                    if (*item).draw_col >= -1 as ::core::ffi::c_int
                        && (*item).start_col as ptrdiff_t >= v
                    {
                        return true_0 != 0;
                    }
                }
                i += 1;
            }
            pos_i += 1;
        }
        return false_0 != 0;
    }
}

pub(crate) unsafe extern "C" fn handle_inline_virtual_text(
    mut _wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
    mut v: ptrdiff_t,
    mut selected: bool,
) {
    unsafe {
        while (*wlv).n_extra == 0 as ::core::ffi::c_int {
            if (*wlv).virt_inline_i >= (*wlv).virt_inline.size {
                (*wlv).virt_inline = VIRTTEXT_EMPTY;
                (*wlv).virt_inline_i = 0 as size_t;
                let mut state: *mut DecorState = decor_state.ptr();
                let end: ::core::ffi::c_int = (*state).current_end;
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < end {
                    let mut item: *mut DecorRange = decor_range_at(state, i);
                    if (*item).draw_col == -3 as ::core::ffi::c_int {
                        decor_init_draw_col((*wlv).off, selected, item);
                    }
                    if !((*item).start_row != (*state).row
                        || (*item).kind as ::core::ffi::c_int
                            != kDecorKindVirtText as ::core::ffi::c_int
                        || (*(*item).data.vt).pos as ::core::ffi::c_uint
                            != kVPosInline as ::core::ffi::c_int as ::core::ffi::c_uint
                        || (*(*item).data.vt).width == 0 as ::core::ffi::c_int)
                    {
                        if (*item).draw_col >= -1 as ::core::ffi::c_int
                            && (*item).start_col as ptrdiff_t == v
                        {
                            (*wlv).virt_inline = (*(*item).data.vt).data.virt_text;
                            (*wlv).virt_inline_hl_mode = (*(*item).data.vt).hl_mode as HlMode;
                            (*item).draw_col = INT_MIN;
                            break;
                        }
                    }
                    i += 1;
                }
                if (*wlv).virt_inline.size == 0 {
                    break;
                }
            } else {
                let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut text: *mut ::core::ffi::c_char = next_virt_text_chunk(
                    (*wlv).virt_inline,
                    &raw mut (*wlv).virt_inline_i,
                    &raw mut attr,
                );
                if text.is_null() {
                    continue;
                }
                (*wlv).p_extra = text;
                (*wlv).n_extra = strlen(text) as ::core::ffi::c_int;
                if (*wlv).n_extra == 0 as ::core::ffi::c_int {
                    continue;
                }
                (*wlv).sc_extra = NUL as schar_T;
                (*wlv).sc_final = NUL as schar_T;
                (*wlv).extra_attr = attr;
                (*wlv).n_attr = mb_charlen(text);
                if (*wlv).skip_cells > 0 as ::core::ffi::c_int {
                    let mut virt_text_width: ::core::ffi::c_int =
                        mb_string2cells((*wlv).p_extra) as ::core::ffi::c_int;
                    if virt_text_width > (*wlv).skip_cells {
                        let mut skip_cells_remaining: ::core::ffi::c_int = (*wlv).skip_cells;
                        while skip_cells_remaining > 0 as ::core::ffi::c_int {
                            let mut cells: ::core::ffi::c_int = utf_ptr2cells((*wlv).p_extra);
                            if cells > skip_cells_remaining {
                                break;
                            }
                            let mut c_len: ::core::ffi::c_int = utfc_ptr2len((*wlv).p_extra);
                            skip_cells_remaining -= cells;
                            (*wlv).p_extra = (*wlv).p_extra.offset(c_len as isize);
                            (*wlv).n_extra -= c_len;
                            (*wlv).n_attr -= 1;
                        }
                        (*wlv).skipped_cells += (*wlv).skip_cells - skip_cells_remaining;
                        (*wlv).skip_cells = skip_cells_remaining;
                    } else {
                        (*wlv).skip_cells -= virt_text_width;
                        (*wlv).skipped_cells += virt_text_width;
                        (*wlv).n_attr = 0 as ::core::ffi::c_int;
                        (*wlv).n_extra = 0 as ::core::ffi::c_int;
                        continue;
                    }
                }
                '_c2rust_label: {
                    if (*wlv).n_extra > 0 as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                        b"wlv->n_extra > 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/drawline.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        1017 as ::core::ffi::c_uint,
                        b"void handle_inline_virtual_text(win_T *, winlinevars_T *, ptrdiff_t, _Bool)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                (*wlv).extra_for_extmark = true_0 != 0;
            }
        }
    }
}
