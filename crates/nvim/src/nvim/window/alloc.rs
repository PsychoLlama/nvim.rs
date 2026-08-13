//! Allocating and freeing windows and frames, and the lists they live on.
//!
//! [`win_alloc`] creates a `win_T` with its option and variable dictionaries;
//! [`win_free`] tears one down, including the `wininfo_T` remembered positions
//! and the autocommand bookkeeping.  [`win_append`]/[`win_remove`] and
//! [`frame_append`]/[`frame_insert`]/[`frame_remove`] are the linked-list
//! splices for the window list and the frame tree.  The `win_alloc_first*`
//! group builds the very first window and frame at startup, and
//! [`win_alloc_aucmd_win`] the invisible window autocommands execute in.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::arglist::alist_unlink;
use crate::src::nvim::autocmd::{block_autocmds, unblock_autocmds};
use crate::src::nvim::buffer::buflist_new;
use crate::src::nvim::decoration::clear_virttext;
use crate::src::nvim::eval::typval::tv_dict_alloc;
use crate::src::nvim::eval::vars::{init_var_dict, unref_var_dict, vars_clear};
use crate::src::nvim::fold::{clearFolding, deleteFoldRecurse, foldInitWin};
use crate::src::nvim::grid::{grid_assign_handle, grid_free};
use crate::src::nvim::hashtab::hash_init;
use crate::src::nvim::main::aucmd_win_vec;
use crate::src::nvim::main::{
    Columns, Rows, au_pending_free_win, autocmd_busy, curbuf, curtab, curwin, first_tabpage,
    firstbuf, firstwin, global_alist, lastwin, p_ch, prevwin, topframe, window_handles,
};
use crate::src::nvim::map::map_del_int_ptr_t;
use crate::src::nvim::mark::free_jumplist;
use crate::src::nvim::r#match::clear_matches;
use crate::src::nvim::memory::{xcalloc, xfree};
use crate::src::nvim::option::clear_winopt;
use crate::src::nvim::os::libc::{abort, memmove, memset};
use crate::src::nvim::quickfix::qf_free_all;
use crate::src::nvim::statusline::stl_clear_click_defs;
use crate::src::nvim::tag::tagstack_clear_entry;
use crate::src::nvim::types::ui::kUIMultigrid;
use crate::src::nvim::types::{
    Error, FloatAnchor, Integer, OptInt, ScreenGrid, Set_uint32_t, VAR_SCOPE, VirtText,
    VirtTextChunk, WinConfig, WinInfo, buf_T, colnr_T, frame_T, handle_T, kErrorTypeNone,
    kFloatRelativeEditor, linenr_T, lpos_T, ptr_t, size_t, tabpage_T, win_T,
};
use crate::src::nvim::ui::{ui_call_grid_destroy, ui_has};
use crate::src::nvim::winfloat::win_new_float;

pub unsafe extern "C" fn win_alloc_first() {
    unsafe {
        if win_alloc_firstwin(::core::ptr::null_mut::<win_T>()) == FAIL {
            abort();
        }
        first_tabpage.set(alloc_tabpage());
        curtab.set(first_tabpage.get());
        unuse_tabpage(first_tabpage.get());
    }
}

pub unsafe extern "C" fn win_alloc_aucmd_win(mut idx: ::core::ffi::c_int) {
    unsafe {
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
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
        fconfig.width = Columns.get();
        fconfig.height = 5 as ::core::ffi::c_int;
        fconfig.focusable = false_0 != 0;
        fconfig.mouse = false_0 != 0;
        (*(*aucmd_win_vec.ptr()).items.offset(idx as isize)).auc_win = win_new_float(
            ::core::ptr::null_mut::<win_T>(),
            true_0 != 0,
            fconfig,
            &raw mut err,
        );
        (*(*(*(*aucmd_win_vec.ptr()).items.offset(idx as isize)).auc_win).w_buffer).b_nwindows -= 1;
        (*(*(*aucmd_win_vec.ptr()).items.offset(idx as isize)).auc_win)
            .w_onebuf_opt
            .wo_scb = false_0;
        (*(*(*aucmd_win_vec.ptr()).items.offset(idx as isize)).auc_win)
            .w_onebuf_opt
            .wo_crb = false_0;
    }
}

pub(crate) unsafe extern "C" fn win_alloc_firstwin(mut oldwin: *mut win_T) -> ::core::ffi::c_int {
    unsafe {
        curwin.set(win_alloc(::core::ptr::null_mut::<win_T>(), false_0 != 0));
        if oldwin.is_null() {
            curbuf.set(buflist_new(
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                ::core::ptr::null_mut::<::core::ffi::c_char>(),
                1 as linenr_T,
                BLN_LISTED as ::core::ffi::c_int,
            ));
            if (*curbuf.ptr()).is_null() {
                return FAIL;
            }
            (*curwin.get()).w_buffer = curbuf.get();
            (*curwin.get()).w_s = &raw mut (*curbuf.get()).b_s;
            (*curbuf.get()).b_nwindows = 1 as ::core::ffi::c_int;
            (*curwin.get()).w_alist = global_alist.ptr();
            curwin_init();
        } else {
            win_init(curwin.get(), oldwin, 0 as ::core::ffi::c_int);
            (*curwin.get()).w_onebuf_opt.wo_scb = false_0;
            (*curwin.get()).w_onebuf_opt.wo_crb = false_0;
        }
        new_frame(curwin.get());
        topframe.set((*curwin.get()).w_frame);
        (*topframe.get()).fr_width = Columns.get();
        (*topframe.get()).fr_height =
            Rows.get() - p_ch.get() as ::core::ffi::c_int - global_stl_height();
        return OK;
    }
}

pub(crate) unsafe extern "C" fn new_frame(mut wp: *mut win_T) {
    unsafe {
        let mut frp: *mut frame_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<frame_T>()) as *mut frame_T;
        (*wp).w_frame = frp;
        (*frp).fr_layout = FR_LEAF as ::core::ffi::c_char;
        (*frp).fr_win = wp;
    }
}

pub unsafe extern "C" fn win_init_size() {
    unsafe {
        (*firstwin.get()).w_height = (Rows.get() as OptInt
            - p_ch.get()
            - tabline_height() as OptInt
            - global_stl_height() as OptInt)
            as ::core::ffi::c_int;
        (*firstwin.get()).w_prev_height = (Rows.get() as OptInt
            - p_ch.get()
            - tabline_height() as OptInt
            - global_stl_height() as OptInt)
            as ::core::ffi::c_int;
        (*firstwin.get()).w_view_height =
            (*firstwin.get()).w_height - (*firstwin.get()).w_winbar_height;
        (*firstwin.get()).w_height_outer = (*firstwin.get()).w_height;
        (*firstwin.get()).w_winrow_off = (*firstwin.get()).w_winbar_height;
        (*topframe.get()).fr_height = (Rows.get() as OptInt
            - p_ch.get()
            - tabline_height() as OptInt
            - global_stl_height() as OptInt)
            as ::core::ffi::c_int;
        (*firstwin.get()).w_width = Columns.get();
        (*firstwin.get()).w_view_width = (*firstwin.get()).w_width;
        (*firstwin.get()).w_width_outer = (*firstwin.get()).w_width;
        (*topframe.get()).fr_width = Columns.get();
    }
}

pub unsafe extern "C" fn win_alloc(mut after: *mut win_T, mut hidden: bool) -> *mut win_T {
    unsafe {
        let mut new_wp: *mut win_T =
            xcalloc(1 as size_t, ::core::mem::size_of::<win_T>()) as *mut win_T;
        (*last_win_id.ptr()) += 1;
        (*new_wp).handle = last_win_id.get() as handle_T;
        map_put_int_ptr_t(
            window_handles.ptr(),
            (*new_wp).handle as ::core::ffi::c_int,
            new_wp as ptr_t,
        );
        (*new_wp).w_grid_alloc.mouse_enabled = true_0 != 0;
        grid_assign_handle(&raw mut (*new_wp).w_grid_alloc);
        (*new_wp).w_vars = tv_dict_alloc();
        init_var_dict((*new_wp).w_vars, &raw mut (*new_wp).w_winvar, VAR_SCOPE);
        block_autocmds();
        if !hidden {
            let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
            if !after.is_null() {
                tp = win_find_tabpage(after);
                if tp == curtab.get() {
                    tp = ::core::ptr::null_mut::<tabpage_T>();
                }
            }
            win_append(after, new_wp, tp);
        }
        (*new_wp).w_wincol = 0 as ::core::ffi::c_int;
        (*new_wp).w_width = Columns.get();
        (*new_wp).w_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*new_wp).w_topfill = 0 as ::core::ffi::c_int;
        (*new_wp).w_botline = 2 as ::core::ffi::c_int as linenr_T;
        (*new_wp).w_cursor.lnum = 1 as ::core::ffi::c_int as linenr_T;
        (*new_wp).w_scbind_pos = 1 as ::core::ffi::c_int;
        (*new_wp).w_floating = false;
        (*new_wp).w_config = WinConfig {
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
        (*new_wp).w_viewport_invalid = true_0 != 0;
        (*new_wp).w_viewport_last_topline = 1 as ::core::ffi::c_int as linenr_T;
        (*new_wp).w_ns_hl = -1 as ::core::ffi::c_int;
        let mut ns_set: Set_uint32_t = SET_INIT;
        (*new_wp).w_ns_set = ns_set;
        (*new_wp).w_onebuf_opt.wo_so = -1 as OptInt;
        (*new_wp).w_allbuf_opt.wo_so = (*new_wp).w_onebuf_opt.wo_so;
        (*new_wp).w_onebuf_opt.wo_siso = -1 as OptInt;
        (*new_wp).w_allbuf_opt.wo_siso = (*new_wp).w_onebuf_opt.wo_siso;
        (*new_wp).w_fraction = 0 as ::core::ffi::c_int;
        (*new_wp).w_prev_fraction_row = -1 as ::core::ffi::c_int;
        foldInitWin(new_wp);
        unblock_autocmds();
        (*new_wp).w_next_match_id = 1000 as ::core::ffi::c_int;
        return new_wp;
    }
}

pub unsafe extern "C" fn free_wininfo(mut wip: *mut WinInfo) {
    unsafe {
        if (*wip).wi_optset {
            clear_winopt(&raw mut (*wip).wi_opt);
            deleteFoldRecurse(&raw mut (*wip).wi_folds);
        }
        xfree(wip as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn win_free(mut wp: *mut win_T, mut tp: *mut tabpage_T) {
    unsafe {
        map_del_int_ptr_t(
            window_handles.ptr(),
            (*wp).handle as ::core::ffi::c_int,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
        );
        clearFolding(wp);
        alist_unlink((*wp).w_alist);
        block_autocmds();
        xfree((*wp).w_ns_set.keys as *mut ::core::ffi::c_void);
        xfree((*wp).w_ns_set.h.hash as *mut ::core::ffi::c_void);
        (*wp).w_ns_set = SET_INIT;
        clear_winopt(&raw mut (*wp).w_onebuf_opt);
        clear_winopt(&raw mut (*wp).w_allbuf_opt);
        xfree((*wp).w_p_lcs_chars.multispace as *mut ::core::ffi::c_void);
        xfree((*wp).w_p_lcs_chars.leadmultispace as *mut ::core::ffi::c_void);
        vars_clear(&raw mut (*(*wp).w_vars).dv_hashtab);
        hash_init(&raw mut (*(*wp).w_vars).dv_hashtab);
        unref_var_dict((*wp).w_vars);
        if prevwin.get() == wp {
            prevwin.set(::core::ptr::null_mut::<win_T>());
        }
        let mut ttp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
        while !ttp.is_null() {
            if (*ttp).tp_prevwin == wp {
                (*ttp).tp_prevwin = ::core::ptr::null_mut::<win_T>();
            }
            ttp = (*ttp).tp_next as *mut tabpage_T;
        }
        xfree((*wp).w_lines as *mut ::core::ffi::c_void);
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*wp).w_tagstacklen {
            tagstack_clear_entry(&mut (*wp).w_tagstack[i as usize]);
            i += 1;
        }
        xfree((*wp).w_localdir as *mut ::core::ffi::c_void);
        xfree((*wp).w_prevdir as *mut ::core::ffi::c_void);
        stl_clear_click_defs((*wp).w_status_click_defs, (*wp).w_status_click_defs_size);
        xfree((*wp).w_status_click_defs as *mut ::core::ffi::c_void);
        stl_clear_click_defs((*wp).w_winbar_click_defs, (*wp).w_winbar_click_defs_size);
        xfree((*wp).w_winbar_click_defs as *mut ::core::ffi::c_void);
        stl_clear_click_defs(
            (*wp).w_statuscol_click_defs,
            (*wp).w_statuscol_click_defs_size,
        );
        xfree((*wp).w_statuscol_click_defs as *mut ::core::ffi::c_void);
        let mut buf: *mut buf_T = firstbuf.get();
        while !buf.is_null() {
            let mut wip_wp: *mut WinInfo = ::core::ptr::null_mut::<WinInfo>();
            let mut pos_wip: size_t = (*buf).b_wininfo.size;
            let mut pos_null: size_t = (*buf).b_wininfo.size;
            let mut i_0: size_t = 0 as size_t;
            while i_0 < (*buf).b_wininfo.size {
                let mut wip: *mut WinInfo = *(*buf).b_wininfo.items.add(i_0);
                if (*wip).wi_win == wp {
                    wip_wp = wip;
                    pos_wip = i_0;
                } else if (*wip).wi_win.is_null() {
                    pos_null = i_0;
                }
                i_0 = i_0.wrapping_add(1);
            }
            if !wip_wp.is_null() {
                (*wip_wp).wi_win = ::core::ptr::null_mut::<win_T>();
                if (*wp).w_config.style as ::core::ffi::c_uint
                    == kWinStyleMinimal as ::core::ffi::c_int as ::core::ffi::c_uint
                    && (*wip_wp).wi_optset as ::core::ffi::c_int != 0
                {
                    clear_winopt(&raw mut (*wip_wp).wi_opt);
                    deleteFoldRecurse(&raw mut (*wip_wp).wi_folds);
                    (*wip_wp).wi_optset = false_0 != 0;
                }
                if pos_null < (*buf).b_wininfo.size {
                    let mut pos_delete: size_t = if pos_null > pos_wip {
                        pos_null
                    } else {
                        pos_wip
                    };
                    free_wininfo(*(*buf).b_wininfo.items.add(pos_delete));
                    (*buf).b_wininfo.size = (*buf).b_wininfo.size.wrapping_sub(1 as size_t);
                    (pos_delete < (*buf).b_wininfo.size
                        && !memmove(
                            (*buf).b_wininfo.items.add(pos_delete) as *mut ::core::ffi::c_void,
                            (*buf)
                                .b_wininfo
                                .items
                                .add(pos_delete.wrapping_add(1 as size_t))
                                as *const ::core::ffi::c_void,
                            (*buf)
                                .b_wininfo
                                .size
                                .wrapping_sub(pos_delete)
                                .wrapping_mul(::core::mem::size_of::<*mut WinInfo>()),
                        )
                        .is_null()) as ::core::ffi::c_int;
                }
            }
            buf = (*buf).b_next;
        }
        clear_virttext(&raw mut (*wp).w_config.title_chunks);
        clear_virttext(&raw mut (*wp).w_config.footer_chunks);
        clear_matches(wp);
        free_jumplist(wp);
        qf_free_all(wp);
        xfree((*wp).w_p_cc_cols as *mut ::core::ffi::c_void);
        win_free_grid(wp, false_0 != 0);
        if win_valid_any_tab(wp) {
            win_remove(wp, tp);
        }
        if autocmd_busy.get() {
            (*wp).w_next = au_pending_free_win.get();
            au_pending_free_win.set(wp);
        } else {
            xfree(wp as *mut ::core::ffi::c_void);
        }
        unblock_autocmds();
    }
}

pub unsafe extern "C" fn win_free_grid(mut wp: *mut win_T, mut reinit: bool) {
    unsafe {
        if (*wp).w_grid_alloc.handle != 0 as ::core::ffi::c_int
            && ui_has(kUIMultigrid) as ::core::ffi::c_int != 0
        {
            ui_call_grid_destroy((*wp).w_grid_alloc.handle as Integer);
        }
        grid_free(&raw mut (*wp).w_grid_alloc);
        if reinit {
            memset(
                &raw mut (*wp).w_grid_alloc as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<ScreenGrid>(),
            );
        }
    }
}

pub unsafe extern "C" fn win_append(
    mut after: *mut win_T,
    mut wp: *mut win_T,
    mut tp: *mut tabpage_T,
) {
    unsafe {
        debug_assert!(
            tp.is_null() || tp != curtab.get(),
            "tp == NULL || tp != curtab"
        );
        let mut first: *mut *mut win_T = if tp.is_null() {
            firstwin.ptr()
        } else {
            &raw mut (*tp).tp_firstwin
        };
        let mut last: *mut *mut win_T = if tp.is_null() {
            lastwin.ptr()
        } else {
            &raw mut (*tp).tp_lastwin
        };
        let mut before: *mut win_T = if after.is_null() {
            *first
        } else {
            (*after).w_next
        };
        (*wp).w_next = before;
        (*wp).w_prev = after;
        if after.is_null() {
            *first = wp;
        } else {
            (*after).w_next = wp;
        }
        if before.is_null() {
            *last = wp;
        } else {
            (*before).w_prev = wp;
        };
    }
}

pub unsafe extern "C" fn win_remove(mut wp: *mut win_T, mut tp: *mut tabpage_T) {
    unsafe {
        debug_assert!(
            tp.is_null() || tp != curtab.get(),
            "tp == NULL || tp != curtab"
        );
        if !(*wp).w_prev.is_null() {
            (*(*wp).w_prev).w_next = (*wp).w_next;
        } else if tp.is_null() {
            (*curtab.get()).tp_firstwin = (*wp).w_next;
            firstwin.set((*curtab.get()).tp_firstwin);
        } else {
            (*tp).tp_firstwin = (*wp).w_next;
        }
        if !(*wp).w_next.is_null() {
            (*(*wp).w_next).w_prev = (*wp).w_prev;
        } else if tp.is_null() {
            (*curtab.get()).tp_lastwin = (*wp).w_prev;
            lastwin.set((*curtab.get()).tp_lastwin);
        } else {
            (*tp).tp_lastwin = (*wp).w_prev;
        };
    }
}

pub(crate) unsafe extern "C" fn frame_append(mut after: *mut frame_T, mut frp: *mut frame_T) {
    unsafe {
        (*frp).fr_next = (*after).fr_next;
        (*after).fr_next = frp;
        if !(*frp).fr_next.is_null() {
            (*(*frp).fr_next).fr_prev = frp;
        }
        (*frp).fr_prev = after;
    }
}

pub(crate) unsafe extern "C" fn frame_insert(mut before: *mut frame_T, mut frp: *mut frame_T) {
    unsafe {
        (*frp).fr_next = before;
        (*frp).fr_prev = (*before).fr_prev;
        (*before).fr_prev = frp;
        if !(*frp).fr_prev.is_null() {
            (*(*frp).fr_prev).fr_next = frp;
        } else {
            (*(*frp).fr_parent).fr_child = frp;
        };
    }
}

pub(crate) unsafe extern "C" fn frame_remove(mut frp: *mut frame_T) {
    unsafe {
        if !(*frp).fr_prev.is_null() {
            (*(*frp).fr_prev).fr_next = (*frp).fr_next;
        } else {
            (*(*frp).fr_parent).fr_child = (*frp).fr_next;
        }
        if !(*frp).fr_next.is_null() {
            (*(*frp).fr_next).fr_prev = (*frp).fr_prev;
        }
    }
}
