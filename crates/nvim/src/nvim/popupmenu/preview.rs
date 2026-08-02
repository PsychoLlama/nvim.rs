//! The info window and the selection that feeds it.
//!
//! [`pum_set_selected`] scrolls the menu to the new selection and, when
//! `'completeopt'` asks for it, fills a preview window (a split, or a
//! float under `popup`) with the item's `info` text.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn pum_preview_set_text(
    mut win: *mut win_T,
    mut info: *mut ::core::ffi::c_char,
    mut lnum: *mut linenr_T,
    mut max_width: *mut ::core::ffi::c_int,
) {
    unsafe {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut arena: Arena = ARENA_EMPTY;
        let mut replacement: Array = ARRAY_DICT_INIT;
        let mut buf: *mut buf_T = (*win).w_buffer;
        (*buf).b_p_ma = true_0;
        let mut curr: *mut ::core::ffi::c_char = info;
        let mut next: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        while !curr.is_null() {
            next = strchr(curr, '\n' as ::core::ffi::c_int);
            if !next.is_null() {
                *next = NUL as ::core::ffi::c_char;
            }
            if *curr as ::core::ffi::c_int == NUL && next.is_null() {
                break;
            }
            let mut save_wrap: bool = (*win).w_onebuf_opt.wo_wrap != 0;
            (*win).w_onebuf_opt.wo_wrap = false_0;
            let mut line_width: ::core::ffi::c_int =
                win_linetabsize(win, 0 as linenr_T, curr, MAXCOL as ::core::ffi::c_int);
            (*win).w_onebuf_opt.wo_wrap = save_wrap as ::core::ffi::c_int;
            *max_width = if *max_width > line_width {
                *max_width
            } else {
                line_width
            };
            if replacement.size == replacement.capacity {
                replacement.capacity = if replacement.capacity != 0 {
                    replacement.capacity << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
                replacement.items = xrealloc(
                    replacement.items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<Object>().wrapping_mul(replacement.capacity),
                ) as *mut Object;
            } else {
            };
            let c2rust_fresh5 = replacement.size;
            replacement.size = replacement.size.wrapping_add(1);
            *replacement.items.offset(c2rust_fresh5 as isize) = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed_12 {
                    string: cstr_to_string(curr),
                },
            };
            *lnum += 1;
            if !next.is_null() {
                *next = '\n' as ::core::ffi::c_char;
            }
            curr = if !next.is_null() {
                next.offset(1 as ::core::ffi::c_int as isize)
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            };
        }
        let mut original_textlock: ::core::ffi::c_int = textlock.get();
        textlock.set(0 as ::core::ffi::c_int);
        nvim_buf_set_lines(
            0 as uint64_t,
            (*buf).handle as Buffer,
            0 as Integer,
            -1 as Integer,
            false_0 != 0,
            replacement,
            &raw mut arena,
            &raw mut err,
        );
        textlock.set(original_textlock);
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            emsg(err.msg);
            api_clear_error(&raw mut err);
        }
        arena_mem_free(arena_finish(&raw mut arena));
        api_free_array(replacement);
        (*buf).b_p_ma = false_0;
    }
}

pub(crate) unsafe extern "C" fn pum_adjust_info_position(
    mut wp: *mut win_T,
    mut width: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut border_width: ::core::ffi::c_int = pum_border_width();
        let mut col: ::core::ffi::c_int = pum_col.get()
            + pum_width.get()
            + 1 as ::core::ffi::c_int
            + (if border_width > pum_scrollbar.get() {
                border_width
            } else {
                pum_scrollbar.get()
            });
        let mut right_extra: ::core::ffi::c_int = Columns.get() - col;
        let mut left_extra: ::core::ffi::c_int = pum_col.get() - 2 as ::core::ffi::c_int;
        let mut max_extra: ::core::ffi::c_int = if right_extra > left_extra {
            right_extra
        } else {
            left_extra
        };
        if max_extra < 10 as ::core::ffi::c_int {
            (*wp).w_config.hide = true_0 != 0;
            return false_0 != 0;
        }
        if right_extra > width {
            (*wp).w_config.width = width;
            (*wp).w_config.col = (col - 1 as ::core::ffi::c_int) as ::core::ffi::c_double;
        } else if left_extra > width {
            (*wp).w_config.width = width;
            (*wp).w_config.col = (pum_col.get() - (*wp).w_config.width - 1 as ::core::ffi::c_int)
                as ::core::ffi::c_double;
        } else {
            let place_in_right: bool = right_extra > left_extra;
            (*wp).w_config.width = max_extra;
            (*wp).w_config.col = (if place_in_right as ::core::ffi::c_int != 0 {
                col - 1 as ::core::ffi::c_int
            } else {
                pum_col.get() - (*wp).w_config.width - 1 as ::core::ffi::c_int
            }) as ::core::ffi::c_double;
        }
        (*wp).w_config.anchor = 0 as ::core::ffi::c_int as FloatAnchor;
        let mut count: linenr_T = (*(*wp).w_buffer).b_ml.ml_line_count;
        (*wp).w_view_width = (*wp).w_config.width;
        (*wp).w_config.height = plines_m_win(wp, (*wp).w_topline, count, Rows.get());
        (*wp).w_config.row = pum_row.get() as ::core::ffi::c_double;
        (*wp).w_config.hide = false_0 != 0;
        win_config_float(wp, (*wp).w_config);
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn pum_set_info(
    mut selected: ::core::ffi::c_int,
    mut info: *mut ::core::ffi::c_char,
) -> *mut win_T {
    unsafe {
        if !pum_is_visible.get() || !compl_match_curr_select(selected) {
            return ::core::ptr::null_mut::<win_T>();
        }
        block_autocmds();
        (*RedrawingDisabled.ptr()) += 1;
        (*no_u_sync.ptr()) += 1;
        let mut wp: *mut win_T = win_float_find_preview();
        if wp.is_null() {
            wp = win_float_create_preview(false_0 != 0, true_0 != 0);
            if wp.is_null() {
                return ::core::ptr::null_mut::<win_T>();
            }
            (*wp).w_topline = 1 as ::core::ffi::c_int as linenr_T;
            (*wp).w_onebuf_opt.wo_wfb = true_0;
        }
        let mut lnum: linenr_T = 0 as linenr_T;
        let mut max_info_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        pum_preview_set_text(wp, info, &raw mut lnum, &raw mut max_info_width);
        (*no_u_sync.ptr()) -= 1;
        (*RedrawingDisabled.ptr()) -= 1;
        redraw_later(wp, UPD_NOT_VALID);
        if !pum_adjust_info_position(wp, max_info_width) {
            wp = ::core::ptr::null_mut::<win_T>();
        }
        unblock_autocmds();
        return wp;
    }
}

pub(crate) unsafe extern "C" fn pum_set_selected(
    mut n: ::core::ffi::c_int,
    mut repeat: ::core::ffi::c_int,
) -> bool {
    unsafe {
        let mut resized: bool = false_0 != 0;
        let mut context: ::core::ffi::c_int = pum_height.get() / 2 as ::core::ffi::c_int;
        let mut prev_selected: ::core::ffi::c_int = pum_selected.get();
        pum_selected.set(n);
        let mut scroll_offset: ::core::ffi::c_int = pum_selected.get() - pum_height.get();
        let mut cur_cot_flags: ::core::ffi::c_uint = get_cot_flags();
        let mut use_float: bool = cur_cot_flags
            & kOptCotFlagPopup as ::core::ffi::c_int as ::core::ffi::c_uint
            != 0 as ::core::ffi::c_uint;
        if use_float as ::core::ffi::c_int != 0
            && (pum_selected.get() < 0 as ::core::ffi::c_int
                || (*(*pum_array.ptr()).offset(pum_selected.get() as isize))
                    .pum_info
                    .is_null())
        {
            let mut wp: *mut win_T = win_float_find_preview();
            if !wp.is_null() {
                (*wp).w_config.hide = true_0 != 0;
                win_config_float(wp, (*wp).w_config);
            }
        }
        if pum_selected.get() >= 0 as ::core::ffi::c_int && pum_selected.get() < pum_size.get() {
            if pum_first.get() > pum_selected.get() - 4 as ::core::ffi::c_int {
                if pum_first.get() > pum_selected.get() - 2 as ::core::ffi::c_int {
                    (*pum_first.ptr()) -= pum_height.get() - 2 as ::core::ffi::c_int;
                    if pum_first.get() < 0 as ::core::ffi::c_int {
                        pum_first.set(0 as ::core::ffi::c_int);
                    } else if pum_first.get() > pum_selected.get() {
                        pum_first.set(pum_selected.get());
                    }
                } else {
                    pum_first.set(pum_selected.get());
                }
            } else if pum_first.get() < scroll_offset + 5 as ::core::ffi::c_int {
                if pum_first.get() < scroll_offset + 3 as ::core::ffi::c_int {
                    pum_first.set(
                        if pum_first.get() + pum_height.get() - 2 as ::core::ffi::c_int
                            > scroll_offset + 1 as ::core::ffi::c_int
                        {
                            pum_first.get() + pum_height.get() - 2 as ::core::ffi::c_int
                        } else {
                            scroll_offset + 1 as ::core::ffi::c_int
                        },
                    );
                } else {
                    pum_first.set(scroll_offset + 1 as ::core::ffi::c_int);
                }
            }
            context = if context < 3 as ::core::ffi::c_int {
                context
            } else {
                3 as ::core::ffi::c_int
            };
            if pum_height.get() > 2 as ::core::ffi::c_int {
                if pum_first.get() > pum_selected.get() - context {
                    pum_first.set(if pum_selected.get() - context > 0 as ::core::ffi::c_int {
                        pum_selected.get() - context
                    } else {
                        0 as ::core::ffi::c_int
                    });
                } else if pum_first.get()
                    < pum_selected.get() + context - pum_height.get() + 1 as ::core::ffi::c_int
                {
                    pum_first.set(
                        pum_selected.get() + context - pum_height.get() + 1 as ::core::ffi::c_int,
                    );
                }
            }
            pum_first.set(if pum_first.get() < pum_size.get() - pum_height.get() {
                pum_first.get()
            } else {
                pum_size.get() - pum_height.get()
            });
            if !(*(*pum_array.ptr()).offset(pum_selected.get() as isize))
                .pum_info
                .is_null()
                && Rows.get() > 10 as ::core::ffi::c_int
                && repeat <= 1 as ::core::ffi::c_int
                && cur_cot_flags
                    & (kOptCotFlagPreview as ::core::ffi::c_int
                        | kOptCotFlagPopup as ::core::ffi::c_int)
                        as ::core::ffi::c_uint
                    != 0
                && !(cur_cot_flags
                    & kOptCotFlagPreview as ::core::ffi::c_int as ::core::ffi::c_uint
                    != 0
                    && cmdwin_type.get() != 0 as ::core::ffi::c_int)
            {
                let mut curwin_save: *mut win_T = curwin.get();
                let mut curtab_save: *mut tabpage_T = curtab.get();
                if use_float {
                    block_autocmds();
                }
                g_do_tagpreview.set(3 as ::core::ffi::c_int);
                if p_pvh.get() > 0 as OptInt && p_pvh.get() < g_do_tagpreview.get() as OptInt {
                    g_do_tagpreview.set(p_pvh.get() as ::core::ffi::c_int);
                }
                (*RedrawingDisabled.ptr()) += 1;
                (*no_u_sync.ptr()) += 1;
                if !use_float {
                    resized = prepare_tagpreview(false_0 != 0);
                } else {
                    let mut wp_0: *mut win_T = win_float_find_preview();
                    if !wp_0.is_null() {
                        win_enter(wp_0, false_0 != 0);
                    } else {
                        wp_0 = win_float_create_preview(true_0 != 0, true_0 != 0);
                        if !wp_0.is_null() {
                            resized = true_0 != 0;
                        }
                    }
                }
                (*no_u_sync.ptr()) -= 1;
                (*RedrawingDisabled.ptr()) -= 1;
                g_do_tagpreview.set(0 as ::core::ffi::c_int);
                if (*curwin.get()).w_onebuf_opt.wo_pvw != 0
                    || (*curwin.get()).w_float_is_info as ::core::ffi::c_int != 0
                {
                    let mut res: ::core::ffi::c_int = OK;
                    if !resized
                        && (*curbuf.get()).b_nwindows == 1 as ::core::ffi::c_int
                        && (*curbuf.get()).b_fname.is_null()
                        && bt_nofile(curbuf.get()) as ::core::ffi::c_int != 0
                        && *(*curbuf.get())
                            .b_p_bh
                            .offset(0 as ::core::ffi::c_int as isize)
                            as ::core::ffi::c_int
                            == 'w' as ::core::ffi::c_int
                    {
                        buf_clear();
                    } else {
                        (*no_u_sync.ptr()) += 1;
                        res = do_ecmd(
                            0 as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<::core::ffi::c_char>(),
                            ::core::ptr::null_mut::<exarg_T>(),
                            ECMD_ONE as ::core::ffi::c_int as linenr_T,
                            0 as ::core::ffi::c_int,
                            ::core::ptr::null_mut::<win_T>(),
                        );
                        (*no_u_sync.ptr()) -= 1;
                        if res == OK {
                            set_option_value_give_err(
                                kOptSwapfile,
                                OptVal {
                                    type_0: kOptValTypeBoolean,
                                    data: OptValData { boolean: kFalse },
                                },
                                OPT_LOCAL as ::core::ffi::c_int,
                            );
                            set_option_value_give_err(
                                kOptBuflisted,
                                OptVal {
                                    type_0: kOptValTypeBoolean,
                                    data: OptValData { boolean: kFalse },
                                },
                                OPT_LOCAL as ::core::ffi::c_int,
                            );
                            set_option_value_give_err(
                                kOptBuftype,
                                OptVal {
                                    type_0: kOptValTypeString,
                                    data: OptValData {
                                        string: String_0 {
                                            data: b"nofile\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char,
                                            size: ::core::mem::size_of::<[::core::ffi::c_char; 7]>(
                                            )
                                            .wrapping_sub(1 as size_t),
                                        },
                                    },
                                },
                                OPT_LOCAL as ::core::ffi::c_int,
                            );
                            set_option_value_give_err(
                                kOptBufhidden,
                                OptVal {
                                    type_0: kOptValTypeString,
                                    data: OptValData {
                                        string: String_0 {
                                            data: b"wipe\0".as_ptr() as *const ::core::ffi::c_char
                                                as *mut ::core::ffi::c_char,
                                            size: ::core::mem::size_of::<[::core::ffi::c_char; 5]>(
                                            )
                                            .wrapping_sub(1 as size_t),
                                        },
                                    },
                                },
                                OPT_LOCAL as ::core::ffi::c_int,
                            );
                            set_option_value_give_err(
                                kOptDiff,
                                OptVal {
                                    type_0: kOptValTypeBoolean,
                                    data: OptValData { boolean: kFalse },
                                },
                                OPT_LOCAL as ::core::ffi::c_int,
                            );
                        }
                    }
                    if res == OK {
                        let mut lnum: linenr_T = 0 as linenr_T;
                        let mut max_info_width: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        pum_preview_set_text(
                            curwin.get(),
                            (*(*pum_array.ptr()).offset(pum_selected.get() as isize)).pum_info,
                            &raw mut lnum,
                            &raw mut max_info_width,
                        );
                        if repeat == 0 as ::core::ffi::c_int && !use_float {
                            lnum = if lnum < p_pvh.get() as linenr_T {
                                lnum
                            } else {
                                p_pvh.get() as linenr_T
                            };
                            if ((*curwin.get()).w_height as linenr_T) < lnum {
                                win_setheight(lnum as ::core::ffi::c_int);
                                resized = true_0 != 0;
                            }
                        }
                        (*curbuf.get()).b_changed = false_0;
                        (*curbuf.get()).b_p_ma = false_0;
                        if pum_selected.get() != prev_selected {
                            (*curwin.get()).w_topline = 1 as ::core::ffi::c_int as linenr_T;
                        } else if (*curwin.get()).w_topline > (*curbuf.get()).b_ml.ml_line_count {
                            (*curwin.get()).w_topline = (*curbuf.get()).b_ml.ml_line_count;
                        }
                        (*curwin.get()).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
                        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                        if use_float {
                            if !pum_adjust_info_position(curwin.get(), max_info_width)
                                && win_valid(curwin_save) as ::core::ffi::c_int != 0
                            {
                                win_enter(curwin_save, false_0 != 0);
                            }
                        }
                        if curwin.get() != curwin_save
                            && win_valid(curwin_save) as ::core::ffi::c_int != 0
                            || curtab.get() != curtab_save
                                && valid_tabpage(curtab_save) as ::core::ffi::c_int != 0
                        {
                            if curtab.get() != curtab_save
                                && valid_tabpage(curtab_save) as ::core::ffi::c_int != 0
                            {
                                goto_tabpage_tp(curtab_save, false_0 != 0, false_0 != 0);
                            }
                            if ins_compl_active() as ::core::ffi::c_int != 0 && !resized {
                                (*curwin.get()).w_redr_status = false_0 != 0;
                            }
                            validate_cursor(curwin.get());
                            redraw_later(curwin.get(), UPD_SOME_VALID);
                            if resized as ::core::ffi::c_int != 0
                                && win_valid(curwin_save) as ::core::ffi::c_int != 0
                            {
                                (*no_u_sync.ptr()) += 1;
                                win_enter(curwin_save, true_0 != 0);
                                (*no_u_sync.ptr()) -= 1;
                                update_topline(curwin.get());
                            }
                            pum_is_visible.set(false_0 != 0);
                            update_screen();
                            pum_is_visible.set(true_0 != 0);
                            if !resized && win_valid(curwin_save) as ::core::ffi::c_int != 0 {
                                (*no_u_sync.ptr()) += 1;
                                win_enter(curwin_save, true_0 != 0);
                                (*no_u_sync.ptr()) -= 1;
                            }
                            pum_is_visible.set(false_0 != 0);
                            update_screen();
                            pum_is_visible.set(true_0 != 0);
                        }
                    }
                }
                if use_float {
                    unblock_autocmds();
                }
            }
        }
        return resized;
    }
}
