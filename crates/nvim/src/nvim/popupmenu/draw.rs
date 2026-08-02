//! Painting the menu onto its own grid.
//!
//! [`pum_redraw`] draws one row per visible item: the three item columns
//! in `'completeitemalign'` order, the truncation marker when a column
//! does not fit, and the scrollbar. Everything else here serves it --
//! [`pum_compute_text_attrs`] works out the per-cell attributes that make
//! the typed leader stand out inside a match.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn pum_compute_text_attrs(
    mut text: *mut ::core::ffi::c_char,
    mut hlf: hlf_T,
    mut user_hlattr: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_int {
    unsafe {
        if *text as ::core::ffi::c_int == NUL
            || hlf as ::core::ffi::c_uint != HLF_PSI as ::core::ffi::c_uint
                && hlf as ::core::ffi::c_uint != HLF_PNI as ::core::ffi::c_uint
            || win_hl_attr(curwin.get(), HLF_PMSI) == win_hl_attr(curwin.get(), HLF_PSI)
                && win_hl_attr(curwin.get(), HLF_PMNI) == win_hl_attr(curwin.get(), HLF_PNI)
        {
            return ::core::ptr::null_mut::<::core::ffi::c_int>();
        }
        let mut leader: *mut ::core::ffi::c_char = if State.get() & MODE_CMDLINE != 0 {
            cmdline_compl_pattern()
        } else {
            ins_compl_leader()
        };
        if leader.is_null() || *leader as ::core::ffi::c_int == NUL {
            return ::core::ptr::null_mut::<::core::ffi::c_int>();
        }
        let mut attrs: *mut ::core::ffi::c_int = xmalloc(
            ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(vim_strsize(text) as size_t),
        ) as *mut ::core::ffi::c_int;
        let mut in_fuzzy: bool = if State.get() & MODE_CMDLINE != 0 {
            cmdline_compl_is_fuzzy() as ::core::ffi::c_int
        } else {
            (get_cot_flags() & kOptCotFlagFuzzy as ::core::ffi::c_int as ::core::ffi::c_uint
                != 0 as ::core::ffi::c_uint) as ::core::ffi::c_int
        } != 0;
        let mut leader_len: size_t = strlen(leader);
        let mut ga: *mut garray_T = ::core::ptr::null_mut::<garray_T>();
        let mut matched_len: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        if in_fuzzy {
            ga = fuzzy_match_str_with_pos(text, leader);
            if ga.is_null() {
                xfree(attrs as *mut ::core::ffi::c_void);
                return ::core::ptr::null_mut::<::core::ffi::c_int>();
            }
        }
        let mut ptr: *const ::core::ffi::c_char = text;
        let mut cell_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut char_pos: uint32_t = 0 as uint32_t;
        let mut is_select: bool = hlf as ::core::ffi::c_uint == HLF_PSI as ::core::ffi::c_uint;
        while *ptr as ::core::ffi::c_int != NUL {
            let mut new_attr: ::core::ffi::c_int =
                win_hl_attr(curwin.get(), hlf as ::core::ffi::c_int);
            if !ga.is_null() {
                let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                while i < (*ga).ga_len {
                    if char_pos == *((*ga).ga_data as *mut uint32_t).offset(i as isize) {
                        new_attr = win_hl_attr(
                            curwin.get(),
                            if is_select as ::core::ffi::c_int != 0 {
                                HLF_PMSI
                            } else {
                                HLF_PMNI
                            },
                        );
                        new_attr = hl_combine_attr(win_hl_attr(curwin.get(), HLF_PMNI), new_attr);
                        new_attr = hl_combine_attr(
                            win_hl_attr(curwin.get(), hlf as ::core::ffi::c_int),
                            new_attr,
                        );
                        break;
                    } else {
                        i += 1;
                    }
                }
            } else {
                if matched_len < 0 as ::core::ffi::c_int
                    && mb_strnicmp(ptr, leader, leader_len) == 0 as ::core::ffi::c_int
                {
                    matched_len = leader_len as ::core::ffi::c_int;
                }
                if matched_len > 0 as ::core::ffi::c_int {
                    new_attr = win_hl_attr(
                        curwin.get(),
                        if is_select as ::core::ffi::c_int != 0 {
                            HLF_PMSI
                        } else {
                            HLF_PMNI
                        },
                    );
                    new_attr = hl_combine_attr(win_hl_attr(curwin.get(), HLF_PMNI), new_attr);
                    new_attr = hl_combine_attr(
                        win_hl_attr(curwin.get(), hlf as ::core::ffi::c_int),
                        new_attr,
                    );
                    matched_len -= 1;
                }
            }
            new_attr = hl_combine_attr(win_hl_attr(curwin.get(), HLF_PNI), new_attr);
            if user_hlattr > 0 as ::core::ffi::c_int {
                new_attr = hl_combine_attr(new_attr, user_hlattr);
            }
            let mut char_cells: ::core::ffi::c_int = utf_ptr2cells(ptr);
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i_0 < char_cells {
                *attrs.offset((cell_idx + i_0) as isize) = new_attr;
                i_0 += 1;
            }
            cell_idx += char_cells;
            ptr = ptr.offset(utfc_ptr2len(ptr as *mut ::core::ffi::c_char) as isize);
            char_pos = char_pos.wrapping_add(1);
        }
        if !ga.is_null() {
            ga_clear(ga);
            xfree(ga as *mut ::core::ffi::c_void);
        }
        return attrs;
    }
}

pub(crate) unsafe extern "C" fn pum_grid_puts_with_attrs(
    mut col: ::core::ffi::c_int,
    mut cells: ::core::ffi::c_int,
    mut text: *const ::core::ffi::c_char,
    mut textlen: ::core::ffi::c_int,
    mut attrs: *const ::core::ffi::c_int,
) {
    unsafe {
        let col_start: ::core::ffi::c_int = col;
        let mut ptr: *const ::core::ffi::c_char = text;
        while *ptr as ::core::ffi::c_int != NUL
            && (textlen < 0 as ::core::ffi::c_int || ptr < text.offset(textlen as isize))
        {
            let mut char_len: ::core::ffi::c_int = utfc_ptr2len(ptr);
            let mut attr: ::core::ffi::c_int = *attrs.offset(
                (if pum_rl.get() as ::core::ffi::c_int != 0 {
                    col_start + cells - col - 1 as ::core::ffi::c_int
                } else {
                    col - col_start
                }) as isize,
            );
            grid_line_puts(col, ptr, char_len, attr);
            col += utf_ptr2cells(ptr);
            ptr = ptr.offset(char_len as isize);
        }
    }
}

#[inline]
pub(crate) unsafe extern "C" fn pum_align_order(mut order: *mut ::core::ffi::c_int) {
    unsafe {
        let mut is_default: bool = cia_flags.get() == 0 as ::core::ffi::c_uint;
        *order.offset(0 as ::core::ffi::c_int as isize) = (if is_default as ::core::ffi::c_int != 0
        {
            CPT_ABBR as ::core::ffi::c_int as ::core::ffi::c_uint
        } else {
            (*cia_flags.ptr()).wrapping_div(100 as ::core::ffi::c_uint)
        }) as ::core::ffi::c_int;
        *order.offset(1 as ::core::ffi::c_int as isize) = (if is_default as ::core::ffi::c_int != 0
        {
            CPT_KIND as ::core::ffi::c_int as ::core::ffi::c_uint
        } else {
            (*cia_flags.ptr())
                .wrapping_div(10 as ::core::ffi::c_uint)
                .wrapping_rem(10 as ::core::ffi::c_uint)
        }) as ::core::ffi::c_int;
        *order.offset(2 as ::core::ffi::c_int as isize) = (if is_default as ::core::ffi::c_int != 0
        {
            CPT_MENU as ::core::ffi::c_int as ::core::ffi::c_uint
        } else {
            (*cia_flags.ptr()).wrapping_rem(10 as ::core::ffi::c_uint)
        }) as ::core::ffi::c_int;
    }
}

#[inline]
pub(crate) unsafe extern "C" fn pum_get_item(
    mut index: ::core::ffi::c_int,
    mut type_0: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        match type_0 {
            0 => return (*(*pum_array.ptr()).offset(index as isize)).pum_text,
            1 => return (*(*pum_array.ptr()).offset(index as isize)).pum_kind,
            2 => return (*(*pum_array.ptr()).offset(index as isize)).pum_extra,
            _ => {}
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

#[inline]
pub(crate) unsafe extern "C" fn pum_user_attr_combine(
    mut idx: ::core::ffi::c_int,
    mut type_0: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut user_attr: [::core::ffi::c_int; 2] = [
            (*(*pum_array.ptr()).offset(idx as isize)).pum_user_abbr_hlattr,
            (*(*pum_array.ptr()).offset(idx as isize)).pum_user_kind_hlattr,
        ];
        return if user_attr[type_0 as usize] > 0 as ::core::ffi::c_int {
            hl_combine_attr(attr, user_attr[type_0 as usize])
        } else {
            attr
        };
    }
}

pub unsafe extern "C" fn pum_redraw() {
    unsafe {
        let mut row: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut attr_scroll: ::core::ffi::c_int = win_hl_attr(curwin.get(), HLF_PSB);
        let mut attr_thumb: ::core::ffi::c_int = win_hl_attr(curwin.get(), HLF_PST);
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut thumb_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut thumb_height: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
        let mut n: ::core::ffi::c_int = 0;
        let fcs_trunc: schar_T = if pum_rl.get() as ::core::ffi::c_int != 0 {
            (*curwin.get()).w_p_fcs_chars.truncrl
        } else {
            (*curwin.get()).w_p_fcs_chars.trunc
        };
        let hlfsNorm: [hlf_T; 3] = [HLF_PNI, HLF_PNK, HLF_PNX];
        let hlfsSel: [hlf_T; 3] = [HLF_PSI, HLF_PSK, HLF_PSX];
        let mut grid_width: ::core::ffi::c_int = pum_width.get();
        let mut col_off: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut extra_space: bool = false_0 != 0;
        if pum_rl.get() {
            col_off = pum_width.get() - 1 as ::core::ffi::c_int;
            '_c2rust_label: {
                if State.get() & MODE_CMDLINE == 0 {
                } else {
                    __assert_fail(
                        b"!(State & MODE_CMDLINE)\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/popupmenu.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        584 as ::core::ffi::c_uint,
                        b"void pum_redraw(void)\0".as_ptr() as *const ::core::ffi::c_char,
                    );
                }
            };
            let mut win_end_col: ::core::ffi::c_int =
                (*curwin.get()).w_wincol + (*curwin.get()).w_width;
            if pum_col.get() < win_end_col - 1 as ::core::ffi::c_int {
                grid_width += 1 as ::core::ffi::c_int;
                extra_space = true_0 != 0;
            }
        } else {
            let mut min_col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if pum_col.get() > min_col {
                grid_width += 1 as ::core::ffi::c_int;
                col_off = 1 as ::core::ffi::c_int;
                extra_space = true_0 != 0;
            }
        }
        let mut fconfig: WinConfig = WinConfig {
            window: 0,
            bufpos: lpos_T {
                lnum: -1 as linenr_T,
                col: 0 as colnr_T,
            },
            height: 0 as ::core::ffi::c_int,
            width: 0 as ::core::ffi::c_int,
            row: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
            col: 0 as ::core::ffi::c_int as ::core::ffi::c_double,
            anchor: 0 as FloatAnchor,
            relative: kFloatRelativeEditor,
            external: false_0 != 0,
            focusable: true_0 != 0,
            mouse: true_0 != 0,
            split: kWinSplitLeft,
            zindex: kZIndexFloatDefault as ::core::ffi::c_int,
            style: kWinStyleUnused,
            border: false,
            shadow: false,
            border_chars: [[0; 32]; 8],
            border_hl_ids: [0; 8],
            border_attr: [0; 8],
            title: false,
            title_pos: kAlignLeft,
            title_chunks: VirtText {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
            title_width: 0,
            footer: false,
            footer_pos: kAlignLeft,
            footer_chunks: VirtText {
                size: 0,
                capacity: 0,
                items: ::core::ptr::null_mut::<VirtTextChunk>(),
            },
            footer_width: 0,
            noautocmd: false_0 != 0,
            fixed: false_0 != 0,
            hide: false_0 != 0,
            _cmdline_offset: INT_MAX,
        };
        let mut border_width: ::core::ffi::c_int = pum_border_width();
        let mut border_attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut border_char: schar_T = 0 as schar_T;
        let mut fill_char: schar_T = ' ' as ::core::ffi::c_int as schar_T;
        let mut has_border: bool = border_width > 0 as ::core::ffi::c_int;
        if border_width > 0 as ::core::ffi::c_int {
            let mut err: Error = Error {
                type_0: kErrorTypeNone,
                msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            };
            if !parse_winborder(&raw mut fconfig, p_pumborder.get(), &raw mut err) {
                if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
                    emsg(err.msg);
                }
                api_clear_error(&raw mut err);
                return;
            }
            if strequal(
                p_pumborder.get(),
                (*opt_winborder_values.ptr())[3 as ::core::ffi::c_int as usize]
                    as *const ::core::ffi::c_char,
            ) {
                fconfig.shadow = true_0 != 0;
                let mut blend: ::core::ffi::c_int = syn_check_group(
                    b"PmenuShadow\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 12]>().wrapping_sub(1 as size_t),
                );
                let mut through: ::core::ffi::c_int = syn_check_group(
                    b"PmenuShadowThrough\0".as_ptr() as *const ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 19]>().wrapping_sub(1 as size_t),
                );
                fconfig.border_hl_ids[2 as ::core::ffi::c_int as usize] = through;
                fconfig.border_hl_ids[3 as ::core::ffi::c_int as usize] = blend;
                fconfig.border_hl_ids[4 as ::core::ffi::c_int as usize] = blend;
                fconfig.border_hl_ids[5 as ::core::ffi::c_int as usize] = blend;
                fconfig.border_hl_ids[6 as ::core::ffi::c_int as usize] = through;
            }
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < 8 as ::core::ffi::c_int {
                let mut attr: ::core::ffi::c_int =
                    *(*hl_attr_active.ptr()).offset(HLF_PBR as isize);
                if fconfig.border_hl_ids[i as usize] != 0 {
                    attr = hl_get_ui_attr(
                        -1 as ::core::ffi::c_int,
                        HLF_PBR,
                        fconfig.border_hl_ids[i as usize],
                        false_0 != 0,
                    );
                }
                fconfig.border_attr[i as usize] = attr;
                i += 1;
            }
            api_clear_error(&raw mut err);
            if pum_scrollbar.get() != 0 {
                border_char = schar_from_str(
                    &raw mut *(&raw mut fconfig.border_chars as *mut [::core::ffi::c_char; 32])
                        .offset(3 as ::core::ffi::c_int as isize)
                        as *mut ::core::ffi::c_char,
                );
                border_attr = fconfig.border_attr[3 as ::core::ffi::c_int as usize];
            }
        }
        if pum_scrollbar.get() > 0 as ::core::ffi::c_int
            && (!fconfig.border || fconfig.shadow as ::core::ffi::c_int != 0)
        {
            grid_width += 1;
            if pum_rl.get() {
                col_off += 1;
            }
        }
        (*pum_grid.ptr()).blending =
            p_pb.get() > 0 as OptInt || fconfig.shadow as ::core::ffi::c_int != 0;
        grid_assign_handle(pum_grid.ptr());
        pum_left_col.set(pum_col.get() - col_off);
        pum_right_col.set(pum_left_col.get() + grid_width);
        let mut moved: bool = ui_comp_put_grid(
            pum_grid.ptr(),
            pum_row.get(),
            pum_left_col.get(),
            pum_height.get() + border_width,
            grid_width + border_width,
            false_0 != 0,
            true_0 != 0,
        );
        let mut invalid_grid: bool =
            moved as ::core::ffi::c_int != 0 || pum_invalid.get() as ::core::ffi::c_int != 0;
        pum_invalid.set(false_0 != 0);
        must_redraw_pum.set(false_0 != 0);
        if (*pum_grid.ptr()).chars.is_null()
            || (*pum_grid.ptr()).rows != pum_height.get() + border_width
            || (*pum_grid.ptr()).cols != grid_width + border_width
        {
            grid_alloc(
                pum_grid.ptr(),
                pum_height.get() + border_width,
                grid_width + border_width,
                !invalid_grid,
                false_0 != 0,
            );
            ui_call_grid_resize(
                (*pum_grid.ptr()).handle as Integer,
                (*pum_grid.ptr()).cols as Integer,
                (*pum_grid.ptr()).rows as Integer,
            );
        } else if invalid_grid {
            grid_invalidate(pum_grid.ptr());
        }
        if ui_has(kUIMultigrid) {
            let mut anchor: *const ::core::ffi::c_char =
                if pum_above.get() as ::core::ffi::c_int != 0 {
                    b"SW\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"NW\0".as_ptr() as *const ::core::ffi::c_char
                };
            let mut row_off: ::core::ffi::c_int = if pum_above.get() as ::core::ffi::c_int != 0 {
                -pum_height.get()
            } else {
                0 as ::core::ffi::c_int
            };
            ui_call_win_float_pos(
                (*pum_grid.ptr()).handle as Integer,
                -1 as Window,
                cstr_as_string(anchor),
                pum_anchor_grid.get() as Integer,
                (pum_row.get() - row_off - pum_win_row_offset.get()) as Float,
                (pum_left_col.get() - pum_win_col_offset.get()) as Float,
                false_0 != 0,
                (*pum_grid.ptr()).zindex as Integer,
                (*pum_grid.ptr()).comp_index as ::core::ffi::c_int as Integer,
                (*pum_grid.ptr()).comp_row as Integer,
                (*pum_grid.ptr()).comp_col as Integer,
            );
        }
        let mut scroll_range: ::core::ffi::c_int = pum_size.get() - pum_height.get();
        if fconfig.border {
            grid_draw_border(
                pum_grid.ptr(),
                &raw mut fconfig,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                0 as ::core::ffi::c_int,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
            );
            if !fconfig.shadow {
                row += 1;
                col_off += 1;
            }
        }
        pum_first.set(if pum_first.get() < scroll_range {
            pum_first.get()
        } else {
            scroll_range
        });
        if pum_scrollbar.get() != 0 {
            thumb_height = pum_height.get() * pum_height.get() / pum_size.get();
            if thumb_height == 0 as ::core::ffi::c_int {
                thumb_height = 1 as ::core::ffi::c_int;
            }
            thumb_pos = (pum_first.get() * (pum_height.get() - thumb_height)
                + scroll_range / 2 as ::core::ffi::c_int)
                / scroll_range;
        }
        let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i_0 < pum_height.get() {
            let mut idx: ::core::ffi::c_int = i_0 + pum_first.get();
            let selected: bool = idx == pum_selected.get();
            let hlfs: *const hlf_T = if selected as ::core::ffi::c_int != 0 {
                &raw const hlfsSel as *const hlf_T
            } else {
                &raw const hlfsNorm as *const hlf_T
            };
            let trunc_attr: ::core::ffi::c_int = win_hl_attr(
                curwin.get(),
                if selected as ::core::ffi::c_int != 0 {
                    HLF_PSI
                } else {
                    HLF_PNI
                },
            );
            let mut hlf: hlf_T = *hlfs.offset(0 as ::core::ffi::c_int as isize);
            let mut attr_0: ::core::ffi::c_int =
                win_hl_attr(curwin.get(), hlf as ::core::ffi::c_int);
            attr_0 = hl_combine_attr(win_hl_attr(curwin.get(), HLF_PNI), attr_0);
            screengrid_line_start(pum_grid.ptr(), row, 0 as ::core::ffi::c_int);
            if extra_space {
                if pum_rl.get() {
                    grid_line_puts(
                        col_off + 1 as ::core::ffi::c_int,
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                        attr_0,
                    );
                } else {
                    grid_line_puts(
                        col_off - 1 as ::core::ffi::c_int,
                        b" \0".as_ptr() as *const ::core::ffi::c_char,
                        1 as ::core::ffi::c_int,
                        attr_0,
                    );
                }
            }
            let mut grid_col: ::core::ffi::c_int = col_off;
            let mut totwidth: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            let mut need_fcs_trunc: bool = false_0 != 0;
            let mut order: [::core::ffi::c_int; 3] = [0; 3];
            let mut items_width_array: [::core::ffi::c_int; 3] = [
                pum_base_width.get(),
                pum_kind_width.get(),
                pum_extra_width.get(),
            ];
            pum_align_order(&raw mut order as *mut ::core::ffi::c_int);
            let mut basic_width: ::core::ffi::c_int =
                items_width_array[order[0 as ::core::ffi::c_int as usize] as usize];
            let mut last_isabbr: bool =
                order[2 as ::core::ffi::c_int as usize] == CPT_ABBR as ::core::ffi::c_int;
            let mut orig_attr: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
            let mut j: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while j < 3 as ::core::ffi::c_int {
                let mut item_type: ::core::ffi::c_int = order[j as usize];
                hlf = *hlfs.offset(item_type as isize);
                attr_0 = win_hl_attr(curwin.get(), hlf as ::core::ffi::c_int);
                attr_0 = hl_combine_attr(win_hl_attr(curwin.get(), HLF_PNI), attr_0);
                orig_attr = attr_0;
                if item_type < 2 as ::core::ffi::c_int {
                    attr_0 = pum_user_attr_combine(idx, item_type, attr_0);
                }
                let mut width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                let mut s: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                p = pum_get_item(idx, item_type);
                let next_isempty: bool = j + 1 as ::core::ffi::c_int >= 3 as ::core::ffi::c_int
                    || pum_get_item(idx, order[(j + 1 as ::core::ffi::c_int) as usize]).is_null();
                if !p.is_null() {
                    loop {
                        if s.is_null() {
                            s = p;
                        }
                        let mut w: ::core::ffi::c_int = ptr2cells(p);
                        if *p as ::core::ffi::c_int != NUL
                            && *p as ::core::ffi::c_int != TAB
                            && totwidth + w <= pum_width.get()
                        {
                            width += w;
                        } else {
                            let width_limit: ::core::ffi::c_int = pum_width.get();
                            let mut saved: ::core::ffi::c_char = *p;
                            if saved as ::core::ffi::c_int != NUL {
                                *p = NUL as ::core::ffi::c_char;
                            }
                            let mut st: *mut ::core::ffi::c_char = transstr(s, true_0 != 0);
                            if saved as ::core::ffi::c_int != NUL {
                                *p = saved;
                            }
                            let mut attrs: *mut ::core::ffi::c_int =
                                ::core::ptr::null_mut::<::core::ffi::c_int>();
                            if item_type == CPT_ABBR as ::core::ffi::c_int {
                                attrs = pum_compute_text_attrs(
                                    st,
                                    hlf,
                                    (*(*pum_array.ptr()).offset(idx as isize)).pum_user_abbr_hlattr,
                                );
                            }
                            if pum_rl.get() {
                                let mut rt: *mut ::core::ffi::c_char = reverse_text(st);
                                let mut rt_start: *mut ::core::ffi::c_char = rt;
                                let mut cells: ::core::ffi::c_int =
                                    mb_string2cells(rt) as ::core::ffi::c_int;
                                let mut pad: ::core::ffi::c_int =
                                    if next_isempty as ::core::ffi::c_int != 0 {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        2 as ::core::ffi::c_int
                                    };
                                if width_limit - totwidth < cells + pad {
                                    need_fcs_trunc = true_0 != 0;
                                }
                                if grid_col - cells < col_off - width_limit {
                                    loop {
                                        cells -= utf_ptr2cells(rt);
                                        rt = rt.offset(utfc_ptr2len(rt) as isize);
                                        if grid_col - cells >= col_off - width_limit {
                                            break;
                                        }
                                    }
                                    if grid_col - cells > col_off - width_limit {
                                        rt = rt.offset(-1);
                                        *rt = '<' as ::core::ffi::c_char;
                                        cells += 1;
                                    }
                                }
                                if attrs.is_null() {
                                    grid_line_puts(
                                        grid_col - cells + 1 as ::core::ffi::c_int,
                                        rt,
                                        -1 as ::core::ffi::c_int,
                                        attr_0,
                                    );
                                } else {
                                    pum_grid_puts_with_attrs(
                                        grid_col - cells + 1 as ::core::ffi::c_int,
                                        cells,
                                        rt,
                                        -1 as ::core::ffi::c_int,
                                        attrs,
                                    );
                                }
                                xfree(rt_start as *mut ::core::ffi::c_void);
                                xfree(st as *mut ::core::ffi::c_void);
                                grid_col -= width;
                            } else {
                                let mut cells_0: ::core::ffi::c_int =
                                    mb_string2cells(st) as ::core::ffi::c_int;
                                let mut pad_0: ::core::ffi::c_int =
                                    if next_isempty as ::core::ffi::c_int != 0 {
                                        0 as ::core::ffi::c_int
                                    } else {
                                        2 as ::core::ffi::c_int
                                    };
                                if width_limit - totwidth < cells_0 + pad_0 {
                                    need_fcs_trunc = true_0 != 0;
                                }
                                if need_fcs_trunc {
                                    let mut available_cells: ::core::ffi::c_int =
                                        width_limit - totwidth;
                                    let mut p_end: *mut ::core::ffi::c_char = st;
                                    let mut displayed: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                                    while *p_end as ::core::ffi::c_int != NUL {
                                        let mut char_cells: ::core::ffi::c_int =
                                            utf_ptr2cells(p_end);
                                        if displayed + char_cells > available_cells {
                                            break;
                                        }
                                        displayed += char_cells;
                                        p_end = p_end.offset(utfc_ptr2len(p_end) as isize);
                                    }
                                    *p_end = NUL as ::core::ffi::c_char;
                                    cells_0 = displayed;
                                    width = displayed;
                                }
                                if attrs.is_null() {
                                    grid_line_puts(grid_col, st, -1 as ::core::ffi::c_int, attr_0);
                                } else {
                                    pum_grid_puts_with_attrs(
                                        grid_col,
                                        cells_0,
                                        st,
                                        -1 as ::core::ffi::c_int,
                                        attrs,
                                    );
                                }
                                xfree(st as *mut ::core::ffi::c_void);
                                grid_col += width;
                            }
                            if !attrs.is_null() {
                                let mut ptr_: *mut *mut ::core::ffi::c_void =
                                    &raw mut attrs as *mut *mut ::core::ffi::c_void;
                                xfree(*ptr_);
                                *ptr_ = NULL;
                                let _ = *ptr_;
                            }
                            if *p as ::core::ffi::c_int != TAB {
                                break;
                            }
                            if pum_rl.get() {
                                grid_line_puts(
                                    grid_col - 1 as ::core::ffi::c_int,
                                    b"  \0".as_ptr() as *const ::core::ffi::c_char,
                                    2 as ::core::ffi::c_int,
                                    attr_0,
                                );
                                grid_col -= 2 as ::core::ffi::c_int;
                            } else {
                                grid_line_puts(
                                    grid_col,
                                    b"  \0".as_ptr() as *const ::core::ffi::c_char,
                                    2 as ::core::ffi::c_int,
                                    attr_0,
                                );
                                grid_col += 2 as ::core::ffi::c_int;
                            }
                            totwidth += 2 as ::core::ffi::c_int;
                            s = ::core::ptr::null_mut::<::core::ffi::c_char>();
                            width = 0 as ::core::ffi::c_int;
                        }
                        p = p.offset(utfc_ptr2len(p) as isize);
                    }
                }
                if j > 0 as ::core::ffi::c_int {
                    n = items_width_array[order[1 as ::core::ffi::c_int as usize] as usize]
                        + (if last_isabbr as ::core::ffi::c_int != 0 {
                            0 as ::core::ffi::c_int
                        } else {
                            1 as ::core::ffi::c_int
                        });
                } else {
                    n = if order[j as usize] == CPT_ABBR as ::core::ffi::c_int {
                        1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                }
                if j == 2 as ::core::ffi::c_int
                    || next_isempty as ::core::ffi::c_int != 0
                        && (j == 1 as ::core::ffi::c_int
                            || j == 0 as ::core::ffi::c_int
                                && pum_get_item(idx, order[(j + 2 as ::core::ffi::c_int) as usize])
                                    .is_null())
                    || basic_width + n >= pum_width.get()
                {
                    break;
                }
                if pum_rl.get() {
                    grid_line_fill(
                        col_off - basic_width - n + 1 as ::core::ffi::c_int,
                        grid_col + 1 as ::core::ffi::c_int,
                        ' ' as ::core::ffi::c_int as schar_T,
                        orig_attr,
                    );
                    grid_col = col_off - basic_width - n;
                } else {
                    grid_line_fill(
                        grid_col,
                        col_off + basic_width + n,
                        ' ' as ::core::ffi::c_int as schar_T,
                        orig_attr,
                    );
                    grid_col = col_off + basic_width + n;
                }
                totwidth = basic_width + n;
                j += 1;
            }
            if pum_rl.get() {
                let lcol: ::core::ffi::c_int = col_off - pum_width.get() + 1 as ::core::ffi::c_int;
                grid_line_fill(
                    lcol,
                    grid_col + 1 as ::core::ffi::c_int,
                    ' ' as ::core::ffi::c_int as schar_T,
                    orig_attr,
                );
                if need_fcs_trunc {
                    *(*linebuf_char.ptr()).offset(lcol as isize) = if fcs_trunc != NUL as schar_T {
                        fcs_trunc
                    } else {
                        '<' as ::core::ffi::c_int as schar_T
                    };
                    *(*linebuf_attr.ptr()).offset(lcol as isize) = trunc_attr as sattr_T;
                    if pum_width.get() > 1 as ::core::ffi::c_int
                        && *(*linebuf_char.ptr()).offset((lcol + 1 as ::core::ffi::c_int) as isize)
                            == NUL as schar_T
                    {
                        *(*linebuf_char.ptr()).offset((lcol + 1 as ::core::ffi::c_int) as isize) =
                            ' ' as ::core::ffi::c_int as schar_T;
                    }
                }
            } else {
                let rcol: ::core::ffi::c_int = col_off + pum_width.get();
                grid_line_fill(
                    grid_col,
                    rcol,
                    ' ' as ::core::ffi::c_int as schar_T,
                    orig_attr,
                );
                if need_fcs_trunc {
                    if pum_width.get() > 1 as ::core::ffi::c_int
                        && *(*linebuf_char.ptr()).offset((rcol - 1 as ::core::ffi::c_int) as isize)
                            == NUL as schar_T
                    {
                        *(*linebuf_char.ptr()).offset((rcol - 2 as ::core::ffi::c_int) as isize) =
                            ' ' as ::core::ffi::c_int as schar_T;
                    }
                    *(*linebuf_char.ptr()).offset((rcol - 1 as ::core::ffi::c_int) as isize) =
                        if fcs_trunc != NUL as schar_T {
                            fcs_trunc
                        } else {
                            '>' as ::core::ffi::c_int as schar_T
                        };
                    *(*linebuf_attr.ptr()).offset((rcol - 1 as ::core::ffi::c_int) as isize) =
                        trunc_attr as sattr_T;
                }
            }
            if pum_scrollbar.get() > 0 as ::core::ffi::c_int {
                let mut thumb: bool = i_0 >= thumb_pos && i_0 < thumb_pos + thumb_height;
                let mut scrollbar_col: ::core::ffi::c_int = col_off
                    + (if pum_rl.get() as ::core::ffi::c_int != 0 {
                        -pum_width.get()
                    } else {
                        pum_width.get()
                    });
                let mut use_border_style: bool =
                    has_border as ::core::ffi::c_int != 0 && !fconfig.shadow;
                grid_line_put_schar(
                    scrollbar_col,
                    if use_border_style as ::core::ffi::c_int != 0 && !thumb {
                        border_char
                    } else {
                        fill_char
                    },
                    if thumb as ::core::ffi::c_int != 0 {
                        attr_thumb
                    } else if use_border_style as ::core::ffi::c_int != 0 {
                        border_attr
                    } else {
                        attr_scroll
                    },
                );
            }
            grid_line_flush();
            row += 1;
            i_0 += 1;
        }
    }
}
