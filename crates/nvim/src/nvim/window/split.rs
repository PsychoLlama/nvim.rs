//! Splitting a window -- `win_split()` and `win_split_ins()`.
//!
//! [`win_split_ins`] is the whole operation: decide the new window's size
//! from `'winheight'`/`'winwidth'` and the flags, insert a frame beside or
//! around the existing one, allocate the window, copy or inherit the options
//! and the buffer, redistribute the space, and enter the new window unless
//! `WSP_NOENTER` said not to.  [`win_split`] is the thin `:split` entry point
//! over it, and [`win_init`] copies one window's state onto another.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::is_aucmd_win;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, comp_col, redraw_later, status_redraw_all};
use crate::src::nvim::fold::copyFoldingState;
use crate::src::nvim::main::{
    Columns, Rows, cmdmod, curtab, curwin, e_noroom, first_tabpage, firstwin, msg_col, msg_row,
    p_ch, p_ea, p_ead, p_ls, p_sb, p_spk, p_spr, p_wh, p_wiw, p_wmh, p_wmw, sc_col, topframe,
};
use crate::src::nvim::mark::copy_jumplist;
use crate::src::nvim::memory::{xcalloc, xstrdup};
use crate::src::nvim::message::{emsg, msg_clr_eos_force};
use crate::src::nvim::option::win_copy_options;
use crate::src::nvim::os::libc::{gettext, memset};
use crate::src::nvim::quickfix::copy_loclist_stack;
use crate::src::nvim::types::ui::kUIMultigrid;
use crate::src::nvim::types::{
    Integer, OptInt, frame_T, qf_info_T, size_t, tabpage_T, taggy_T, win_T,
};
use crate::src::nvim::ui::{ui_call_win_hide, ui_has};
use crate::src::nvim::ui_compositor::ui_comp_remove_grid;
use crate::src::nvim::winfloat::win_float_anchor_laststatus;

pub unsafe extern "C" fn win_split(
    mut size: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if check_split_disallowed(curwin.get()) == FAIL {
            return FAIL;
        }
        if may_open_tabpage() == OK {
            return OK;
        }
        flags |= (*cmdmod.ptr()).cmod_split;
        if flags & WSP_TOP as ::core::ffi::c_int != 0 && flags & WSP_BOT as ::core::ffi::c_int != 0
        {
            emsg(gettext(
                c"E442: Can't split topleft and botright at the same time".as_ptr(),
            ));
            return FAIL;
        }
        if flags & WSP_HELP as ::core::ffi::c_int != 0 {
            make_snapshot(SNAP_HELP_IDX);
        } else {
            clear_snapshot(curtab.get(), SNAP_HELP_IDX);
        }
        if flags & WSP_QUICKFIX as ::core::ffi::c_int != 0 {
            make_snapshot(SNAP_QUICKFIX_IDX);
        } else {
            clear_snapshot(curtab.get(), SNAP_QUICKFIX_IDX);
        }
        return if win_split_ins(
            size,
            flags,
            ::core::ptr::null_mut::<win_T>(),
            0 as ::core::ffi::c_int,
            ::core::ptr::null_mut::<frame_T>(),
        )
        .is_null()
        {
            FAIL
        } else {
            OK
        };
    }
}

pub unsafe extern "C" fn win_split_ins(
    mut size: ::core::ffi::c_int,
    mut flags: ::core::ffi::c_int,
    mut new_wp: *mut win_T,
    mut dir: ::core::ffi::c_int,
    mut to_flatten: *mut frame_T,
) -> *mut win_T {
    unsafe {
        let mut wp: *mut win_T = new_wp;
        if !new_wp.is_null() && is_aucmd_win(new_wp) as ::core::ffi::c_int != 0 {
            return ::core::ptr::null_mut::<win_T>();
        }
        if new_wp.is_null() {
            trigger_winnewpre();
        }
        let mut oldwin: *mut win_T = ::core::ptr::null_mut::<win_T>();
        if flags & WSP_TOP as ::core::ffi::c_int != 0 {
            oldwin = firstwin.get();
        } else if flags & WSP_BOT as ::core::ffi::c_int != 0
            || (*curwin.get()).w_floating as ::core::ffi::c_int != 0
        {
            oldwin = lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>());
        } else {
            oldwin = curwin.get();
        }
        let mut need_status: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut new_size: ::core::ffi::c_int = size;
        let mut vertical: bool = flags & WSP_VERT as ::core::ffi::c_int != 0;
        let mut toplevel: bool =
            flags & (WSP_TOP as ::core::ffi::c_int | WSP_BOT as ::core::ffi::c_int) != 0;
        if one_window(firstwin.get(), ::core::ptr::null_mut::<tabpage_T>()) as ::core::ffi::c_int
            != 0
            && p_ls.get() == 1 as OptInt
            && (*oldwin).w_status_height == 0 as ::core::ffi::c_int
        {
            if (*oldwin).w_height as OptInt <= p_wmh.get() {
                emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
                return ::core::ptr::null_mut::<win_T>();
            }
            need_status = STATUS_HEIGHT as ::core::ffi::c_int;
            win_float_anchor_laststatus();
        }
        let mut do_equal: bool = false_0 != 0;
        let mut oldwin_height: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let layout: ::core::ffi::c_int = if vertical as ::core::ffi::c_int != 0 {
            FR_ROW
        } else {
            FR_COL
        };
        let mut did_set_fraction: bool = false_0 != 0;
        if vertical {
            let mut wmw1: ::core::ffi::c_int = if p_wmw.get() == 0 as OptInt {
                1 as ::core::ffi::c_int
            } else {
                p_wmw.get() as ::core::ffi::c_int
            };
            let mut needed: ::core::ffi::c_int = wmw1 + 1 as ::core::ffi::c_int;
            if flags & WSP_ROOM as ::core::ffi::c_int != 0 {
                needed += p_wiw.get() as ::core::ffi::c_int - wmw1;
            }
            let mut minwidth: ::core::ffi::c_int = 0;
            let mut available: ::core::ffi::c_int = 0;
            if toplevel {
                minwidth = frame_minwidth(topframe.get(), NOWIN);
                available = (*topframe.get()).fr_width;
                needed += minwidth;
            } else if p_ea.get() != 0 {
                minwidth = frame_minwidth((*oldwin).w_frame, NOWIN);
                let mut prevfrp: *mut frame_T = (*oldwin).w_frame;
                let mut frp: *mut frame_T = (*(*oldwin).w_frame).fr_parent;
                while !frp.is_null() {
                    if (*frp).fr_layout as ::core::ffi::c_int == FR_ROW {
                        let mut frp2: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
                        frp2 = (*frp).fr_child;
                        while !frp2.is_null() {
                            if frp2 != prevfrp {
                                minwidth += frame_minwidth(frp2, NOWIN);
                            }
                            frp2 = (*frp2).fr_next;
                        }
                    }
                    prevfrp = frp;
                    frp = (*frp).fr_parent;
                }
                available = (*topframe.get()).fr_width;
                needed += minwidth;
            } else {
                minwidth = frame_minwidth((*oldwin).w_frame, NOWIN);
                available = (*(*oldwin).w_frame).fr_width;
                needed += minwidth;
            }
            if available < needed {
                emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
                return ::core::ptr::null_mut::<win_T>();
            }
            if new_size == 0 as ::core::ffi::c_int {
                new_size = (*oldwin).w_width / 2 as ::core::ffi::c_int;
            }
            new_size = if (if new_size < available - minwidth - 1 as ::core::ffi::c_int {
                new_size
            } else {
                available - minwidth - 1 as ::core::ffi::c_int
            }) > wmw1
            {
                if new_size < available - minwidth - 1 as ::core::ffi::c_int {
                    new_size
                } else {
                    available - minwidth - 1 as ::core::ffi::c_int
                }
            } else {
                wmw1
            };
            if (((*oldwin).w_width - new_size - 1 as ::core::ffi::c_int) as OptInt) < p_wmw.get() {
                do_equal = true_0 != 0;
            }
            if (*oldwin).w_onebuf_opt.wo_wfw != 0 {
                win_setwidth_win(
                    (*oldwin).w_width + new_size + 1 as ::core::ffi::c_int,
                    oldwin,
                );
            }
            if !do_equal
                && p_ea.get() != 0
                && size == 0 as ::core::ffi::c_int
                && *p_ead.get() as ::core::ffi::c_int != 'v' as ::core::ffi::c_int
                && !(*(*oldwin).w_frame).fr_parent.is_null()
            {
                let mut frp_0: *mut frame_T = (*(*(*oldwin).w_frame).fr_parent).fr_child;
                while !frp_0.is_null() {
                    if (*frp_0).fr_win != oldwin
                        && !(*frp_0).fr_win.is_null()
                        && ((*(*frp_0).fr_win).w_width > new_size
                            || (*(*frp_0).fr_win).w_width
                                > (*oldwin).w_width - new_size - 1 as ::core::ffi::c_int)
                    {
                        do_equal = true_0 != 0;
                        break;
                    } else {
                        frp_0 = (*frp_0).fr_next;
                    }
                }
            }
        } else {
            let mut wmh1: ::core::ffi::c_int =
                (if p_wmh.get() as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
                    p_wmh.get() as ::core::ffi::c_int
                } else {
                    1 as ::core::ffi::c_int
                }) + (*oldwin).w_winbar_height;
            let mut needed_0: ::core::ffi::c_int = wmh1 + STATUS_HEIGHT as ::core::ffi::c_int;
            if flags & WSP_ROOM as ::core::ffi::c_int != 0 {
                needed_0 += p_wh.get() as ::core::ffi::c_int - wmh1 + (*oldwin).w_winbar_height;
            }
            if p_ch.get() < 1 as OptInt {
                needed_0 += 1 as ::core::ffi::c_int;
            }
            let mut minheight: ::core::ffi::c_int = 0;
            let mut available_0: ::core::ffi::c_int = 0;
            if toplevel {
                minheight = frame_minheight(topframe.get(), NOWIN) + need_status;
                available_0 = (*topframe.get()).fr_height;
                needed_0 += minheight;
            } else if p_ea.get() != 0 {
                minheight = frame_minheight((*oldwin).w_frame, NOWIN) + need_status;
                let mut prevfrp_0: *mut frame_T = (*oldwin).w_frame;
                let mut frp_1: *mut frame_T = (*(*oldwin).w_frame).fr_parent;
                while !frp_1.is_null() {
                    if (*frp_1).fr_layout as ::core::ffi::c_int == FR_COL {
                        let mut frp2_0: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
                        frp2_0 = (*frp_1).fr_child;
                        while !frp2_0.is_null() {
                            if frp2_0 != prevfrp_0 {
                                minheight += frame_minheight(frp2_0, NOWIN);
                            }
                            frp2_0 = (*frp2_0).fr_next;
                        }
                    }
                    prevfrp_0 = frp_1;
                    frp_1 = (*frp_1).fr_parent;
                }
                available_0 = (*topframe.get()).fr_height;
                needed_0 += minheight;
            } else {
                minheight = frame_minheight((*oldwin).w_frame, NOWIN) + need_status;
                available_0 = (*(*oldwin).w_frame).fr_height;
                needed_0 += minheight;
            }
            if available_0 < needed_0 {
                emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
                return ::core::ptr::null_mut::<win_T>();
            }
            oldwin_height = (*oldwin).w_height;
            if need_status != 0 {
                (*oldwin).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
                oldwin_height -= STATUS_HEIGHT as ::core::ffi::c_int;
            }
            if new_size == 0 as ::core::ffi::c_int {
                new_size = oldwin_height / 2 as ::core::ffi::c_int;
            }
            new_size =
                if (if new_size < available_0 - minheight - STATUS_HEIGHT as ::core::ffi::c_int {
                    new_size
                } else {
                    available_0 - minheight - STATUS_HEIGHT as ::core::ffi::c_int
                }) > wmh1
                {
                    if new_size < available_0 - minheight - STATUS_HEIGHT as ::core::ffi::c_int {
                        new_size
                    } else {
                        available_0 - minheight - STATUS_HEIGHT as ::core::ffi::c_int
                    }
                } else {
                    wmh1
                };
            if ((oldwin_height - new_size - STATUS_HEIGHT as ::core::ffi::c_int) as OptInt)
                < p_wmh.get()
            {
                do_equal = true_0 != 0;
            }
            if (*oldwin).w_onebuf_opt.wo_wfh != 0 {
                set_fraction(oldwin);
                did_set_fraction = true_0 != 0;
                win_setheight_win(
                    (*oldwin).w_height + new_size + STATUS_HEIGHT as ::core::ffi::c_int,
                    oldwin,
                );
                oldwin_height = (*oldwin).w_height;
                if need_status != 0 {
                    oldwin_height -= STATUS_HEIGHT as ::core::ffi::c_int;
                }
            }
            if !do_equal
                && p_ea.get() != 0
                && size == 0 as ::core::ffi::c_int
                && *p_ead.get() as ::core::ffi::c_int != 'h' as ::core::ffi::c_int
                && !(*(*oldwin).w_frame).fr_parent.is_null()
            {
                let mut frp_2: *mut frame_T = (*(*(*oldwin).w_frame).fr_parent).fr_child;
                while !frp_2.is_null() {
                    if (*frp_2).fr_win != oldwin
                        && !(*frp_2).fr_win.is_null()
                        && ((*(*frp_2).fr_win).w_height > new_size
                            || (*(*frp_2).fr_win).w_height
                                > oldwin_height - new_size - STATUS_HEIGHT as ::core::ffi::c_int)
                    {
                        do_equal = true_0 != 0;
                        break;
                    } else {
                        frp_2 = (*frp_2).fr_next;
                    }
                }
            }
        }
        if flags & WSP_TOP as ::core::ffi::c_int == 0 as ::core::ffi::c_int
            && (flags & WSP_BOT as ::core::ffi::c_int != 0
                || flags & WSP_BELOW as ::core::ffi::c_int != 0
                || flags & WSP_ABOVE as ::core::ffi::c_int == 0
                    && (if vertical as ::core::ffi::c_int != 0 {
                        p_spr.get()
                    } else {
                        p_sb.get()
                    }) != 0)
        {
            if new_wp.is_null() {
                wp = win_alloc(oldwin, false_0 != 0);
            } else {
                win_append(oldwin, wp, ::core::ptr::null_mut::<tabpage_T>());
            }
        } else if new_wp.is_null() {
            wp = win_alloc((*oldwin).w_prev, false_0 != 0);
        } else {
            win_append((*oldwin).w_prev, wp, ::core::ptr::null_mut::<tabpage_T>());
        }
        if new_wp.is_null() {
            if wp.is_null() {
                return ::core::ptr::null_mut::<win_T>();
            }
            new_frame(wp);
            win_init(wp, curwin.get(), flags);
        } else if (*wp).w_floating {
            ui_comp_remove_grid(&raw mut (*wp).w_grid_alloc);
            if ui_has(kUIMultigrid) {
                (*wp).w_pos_changed = true_0 != 0;
            } else {
                ui_call_win_hide((*wp).w_grid_alloc.handle as Integer);
                win_free_grid(wp, true_0 != 0);
            }
            if (*wp).w_config.external {
                let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                while !tp.is_null() {
                    if tp != curtab.get() && (*tp).tp_curwin == wp {
                        (*tp).tp_curwin = (*tp).tp_firstwin;
                    }
                    tp = (*tp).tp_next as *mut tabpage_T;
                }
            }
            (*wp).w_floating = false_0 != 0;
            new_frame(wp);
            clear_float_config(&raw mut (*wp).w_config, true_0 != 0);
            memset(
                &raw mut (*wp).w_border_adj as *mut ::core::ffi::c_void,
                0 as ::core::ffi::c_int,
                ::core::mem::size_of::<[::core::ffi::c_int; 4]>(),
            );
        }
        if !to_flatten.is_null() {
            frame_flatten(to_flatten);
        }
        let mut before: bool = false;
        let mut curfrp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        if toplevel {
            if (*topframe.get()).fr_layout as ::core::ffi::c_int == FR_COL && !vertical
                || (*topframe.get()).fr_layout as ::core::ffi::c_int == FR_ROW
                    && vertical as ::core::ffi::c_int != 0
            {
                curfrp = (*topframe.get()).fr_child;
                if flags & WSP_BOT as ::core::ffi::c_int != 0 {
                    while !(*curfrp).fr_next.is_null() {
                        curfrp = (*curfrp).fr_next;
                    }
                }
            } else {
                curfrp = topframe.get();
            }
            before = flags & WSP_TOP as ::core::ffi::c_int != 0;
        } else {
            curfrp = (*oldwin).w_frame;
            if flags & WSP_BELOW as ::core::ffi::c_int != 0 {
                before = false_0 != 0;
            } else if flags & WSP_ABOVE as ::core::ffi::c_int != 0 {
                before = true_0 != 0;
            } else if vertical {
                before = p_spr.get() == 0;
            } else {
                before = p_sb.get() == 0;
            }
        }
        if (*curfrp).fr_parent.is_null()
            || (*(*curfrp).fr_parent).fr_layout as ::core::ffi::c_int != layout
        {
            let mut frp_3: *mut frame_T =
                xcalloc(1 as size_t, ::core::mem::size_of::<frame_T>()) as *mut frame_T;
            *frp_3 = *curfrp;
            (*curfrp).fr_layout = layout as ::core::ffi::c_char;
            (*frp_3).fr_parent = curfrp;
            (*frp_3).fr_next = ::core::ptr::null_mut::<frame_T>();
            (*frp_3).fr_prev = ::core::ptr::null_mut::<frame_T>();
            (*curfrp).fr_child = frp_3;
            (*curfrp).fr_win = ::core::ptr::null_mut::<win_T>();
            curfrp = frp_3;
            if !(*frp_3).fr_win.is_null() {
                (*oldwin).w_frame = frp_3;
            } else {
                frp_3 = (*frp_3).fr_child;
                while !frp_3.is_null() {
                    (*frp_3).fr_parent = curfrp;
                    frp_3 = (*frp_3).fr_next;
                }
            }
        }
        let mut frp_4: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        if new_wp.is_null() {
            frp_4 = (*wp).w_frame;
        } else {
            frp_4 = (*new_wp).w_frame;
        }
        (*frp_4).fr_parent = (*curfrp).fr_parent;
        if before {
            frame_insert(curfrp, frp_4);
        } else {
            frame_append(curfrp, frp_4);
        }
        if !did_set_fraction {
            set_fraction(oldwin);
        }
        (*wp).w_fraction = (*oldwin).w_fraction;
        if vertical {
            (*wp).w_onebuf_opt.wo_scr = (*curwin.get()).w_onebuf_opt.wo_scr;
            if need_status != 0 {
                win_new_height(oldwin, (*oldwin).w_height - 1 as ::core::ffi::c_int);
                (*oldwin).w_status_height = need_status;
            }
            if toplevel {
                (*wp).w_winrow = tabline_height();
                win_new_height(
                    wp,
                    (*curfrp).fr_height
                        - (p_ls.get() == 1 as OptInt || p_ls.get() == 2 as OptInt)
                            as ::core::ffi::c_int,
                );
                (*wp).w_status_height =
                    (p_ls.get() == 1 as OptInt || p_ls.get() == 2 as OptInt) as ::core::ffi::c_int;
                (*wp).w_hsep_height = 0 as ::core::ffi::c_int;
            } else {
                (*wp).w_winrow = (*oldwin).w_winrow;
                win_new_height(wp, (*oldwin).w_height);
                (*wp).w_status_height = (*oldwin).w_status_height;
                (*wp).w_hsep_height = (*oldwin).w_hsep_height;
            }
            (*frp_4).fr_height = (*curfrp).fr_height;
            win_new_width(wp, new_size);
            if before {
                (*wp).w_vsep_width = 1 as ::core::ffi::c_int;
            } else {
                (*wp).w_vsep_width = (*oldwin).w_vsep_width;
                (*oldwin).w_vsep_width = 1 as ::core::ffi::c_int;
            }
            if toplevel {
                if flags & WSP_BOT as ::core::ffi::c_int != 0 {
                    frame_set_vsep(curfrp, true_0 != 0);
                }
                frame_new_width(
                    curfrp,
                    (*curfrp).fr_width
                        - (new_size
                            + (flags & WSP_TOP as ::core::ffi::c_int != 0 as ::core::ffi::c_int)
                                as ::core::ffi::c_int),
                    flags & WSP_TOP as ::core::ffi::c_int != 0,
                    false_0 != 0,
                );
            } else {
                win_new_width(
                    oldwin,
                    (*oldwin).w_width - (new_size + 1 as ::core::ffi::c_int),
                );
            }
            if before {
                (*wp).w_wincol = (*oldwin).w_wincol;
                (*oldwin).w_wincol += new_size + 1 as ::core::ffi::c_int;
            } else {
                (*wp).w_wincol = (*oldwin).w_wincol + (*oldwin).w_width + 1 as ::core::ffi::c_int;
            }
            frame_fix_width(oldwin);
            frame_fix_width(wp);
        } else {
            let is_stl_global: bool = global_stl_height() > 0 as ::core::ffi::c_int;
            if toplevel {
                (*wp).w_wincol = 0 as ::core::ffi::c_int;
                win_new_width(wp, Columns.get());
                (*wp).w_vsep_width = 0 as ::core::ffi::c_int;
            } else {
                (*wp).w_wincol = (*oldwin).w_wincol;
                win_new_width(wp, (*oldwin).w_width);
                (*wp).w_vsep_width = (*oldwin).w_vsep_width;
            }
            (*frp_4).fr_width = (*curfrp).fr_width;
            win_new_height(wp, new_size);
            let old_status_height: ::core::ffi::c_int = (*oldwin).w_status_height;
            if before {
                (*wp).w_hsep_height = if is_stl_global as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            } else {
                (*wp).w_hsep_height = (*oldwin).w_hsep_height;
                (*oldwin).w_hsep_height = if is_stl_global as ::core::ffi::c_int != 0 {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                };
            }
            if toplevel {
                let mut new_fr_height: ::core::ffi::c_int = (*curfrp).fr_height - new_size;
                if is_stl_global {
                    if flags & WSP_BOT as ::core::ffi::c_int != 0 {
                        frame_add_hsep(curfrp);
                    } else {
                        new_fr_height -= 1 as ::core::ffi::c_int;
                    }
                } else {
                    if !(flags & WSP_BOT as ::core::ffi::c_int != 0 && p_ls.get() == 0 as OptInt) {
                        new_fr_height -= STATUS_HEIGHT as ::core::ffi::c_int;
                    }
                    if flags & WSP_BOT as ::core::ffi::c_int != 0 {
                        frame_add_statusline(curfrp);
                    }
                }
                frame_new_height(
                    curfrp,
                    new_fr_height,
                    flags & WSP_TOP as ::core::ffi::c_int != 0,
                    false_0 != 0,
                    false_0 != 0,
                );
            } else {
                win_new_height(
                    oldwin,
                    oldwin_height - (new_size + STATUS_HEIGHT as ::core::ffi::c_int),
                );
            }
            if before {
                (*wp).w_winrow = (*oldwin).w_winrow;
                if is_stl_global {
                    (*wp).w_status_height = 0 as ::core::ffi::c_int;
                    (*oldwin).w_winrow += (*wp).w_height + 1 as ::core::ffi::c_int;
                } else {
                    (*wp).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
                    (*oldwin).w_winrow += (*wp).w_height + STATUS_HEIGHT as ::core::ffi::c_int;
                }
            } else if is_stl_global {
                (*wp).w_winrow = (*oldwin).w_winrow + (*oldwin).w_height + 1 as ::core::ffi::c_int;
                (*wp).w_status_height = 0 as ::core::ffi::c_int;
            } else {
                (*wp).w_winrow =
                    (*oldwin).w_winrow + (*oldwin).w_height + STATUS_HEIGHT as ::core::ffi::c_int;
                (*wp).w_status_height = old_status_height;
                if flags & WSP_BOT as ::core::ffi::c_int == 0 {
                    (*oldwin).w_status_height = STATUS_HEIGHT as ::core::ffi::c_int;
                }
            }
            frame_fix_height(wp);
            frame_fix_height(oldwin);
        }
        if toplevel {
            win_comp_pos();
        }
        redraw_later(wp, UPD_NOT_VALID);
        redraw_later(oldwin, UPD_NOT_VALID);
        status_redraw_all();
        if need_status != 0 {
            msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
            msg_col.set(sc_col.get());
            msg_clr_eos_force();
            comp_col();
            msg_row.set(Rows.get() - 1 as ::core::ffi::c_int);
            msg_col.set(0 as ::core::ffi::c_int);
        }
        if do_equal as ::core::ffi::c_int != 0 || dir != 0 as ::core::ffi::c_int {
            win_equal(
                wp,
                true_0 != 0,
                if vertical as ::core::ffi::c_int != 0 {
                    if dir == 'v' as ::core::ffi::c_int {
                        'b' as ::core::ffi::c_int
                    } else {
                        'h' as ::core::ffi::c_int
                    }
                } else if dir == 'h' as ::core::ffi::c_int {
                    'b' as ::core::ffi::c_int
                } else {
                    'v' as ::core::ffi::c_int
                },
            );
        } else if !is_aucmd_win(wp) {
            win_fix_scroll(false_0 != 0);
        }
        let mut i: ::core::ffi::c_int = 0;
        if flags & WSP_VERT as ::core::ffi::c_int != 0 {
            i = p_wiw.get() as ::core::ffi::c_int;
            if size != 0 as ::core::ffi::c_int {
                p_wiw.set(size as OptInt);
            }
        } else {
            i = p_wh.get() as ::core::ffi::c_int;
            if size != 0 as ::core::ffi::c_int {
                p_wh.set(size as OptInt);
            }
        }
        if flags & WSP_NOENTER as ::core::ffi::c_int == 0 {
            win_enter_ext(
                wp,
                (if new_wp.is_null() {
                    WEE_TRIGGER_NEW_AUTOCMDS as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) | WEE_TRIGGER_ENTER_AUTOCMDS as ::core::ffi::c_int
                    | WEE_TRIGGER_LEAVE_AUTOCMDS as ::core::ffi::c_int,
            );
        }
        if vertical {
            p_wiw.set(i as OptInt);
        } else {
            p_wh.set(i as OptInt);
        }
        if win_valid(oldwin) {
            (*oldwin).w_pos_changed = true_0 != 0;
        }
        return wp;
    }
}

pub unsafe extern "C" fn win_init(
    mut newp: *mut win_T,
    mut oldp: *mut win_T,
    mut flags: ::core::ffi::c_int,
) {
    unsafe {
        (*newp).w_buffer = (*oldp).w_buffer;
        (*newp).w_s = &raw mut (*(*oldp).w_buffer).b_s;
        (*(*oldp).w_buffer).b_nwindows += 1;
        (*newp).w_cursor = (*oldp).w_cursor;
        (*newp).w_valid = 0 as ::core::ffi::c_int;
        (*newp).w_curswant = (*oldp).w_curswant;
        (*newp).w_set_curswant = (*oldp).w_set_curswant;
        (*newp).w_topline = (*oldp).w_topline;
        (*newp).w_topfill = (*oldp).w_topfill;
        (*newp).w_leftcol = (*oldp).w_leftcol;
        (*newp).w_pcmark = (*oldp).w_pcmark;
        (*newp).w_prev_pcmark = (*oldp).w_prev_pcmark;
        (*newp).w_alt_fnum = (*oldp).w_alt_fnum;
        (*newp).w_wrow = (*oldp).w_wrow;
        (*newp).w_fraction = (*oldp).w_fraction;
        (*newp).w_prev_fraction_row = (*oldp).w_prev_fraction_row;
        copy_jumplist(oldp, newp);
        if flags & WSP_NEWLOC as ::core::ffi::c_int != 0 {
            (*newp).w_llist = ::core::ptr::null_mut::<qf_info_T>();
            (*newp).w_llist_ref = ::core::ptr::null_mut::<qf_info_T>();
        } else {
            copy_loclist_stack(oldp, newp);
        }
        (*newp).w_localdir = if (*oldp).w_localdir.is_null() {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            xstrdup((*oldp).w_localdir)
        };
        (*newp).w_prevdir = if (*oldp).w_prevdir.is_null() {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            xstrdup((*oldp).w_prevdir)
        };
        if *p_spk.get() as ::core::ffi::c_int != 'c' as ::core::ffi::c_int {
            if *p_spk.get() as ::core::ffi::c_int == 't' as ::core::ffi::c_int {
                (*newp).w_skipcol = (*oldp).w_skipcol;
            }
            (*newp).w_botline = (*oldp).w_botline;
            (*newp).w_prev_height = (*oldp).w_height;
            (*newp).w_prev_winrow = (*oldp).w_winrow;
        }
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*oldp).w_tagstacklen {
            let mut tag: *mut taggy_T =
                (&raw mut (*newp).w_tagstack as *mut taggy_T).offset(i as isize);
            *tag = (*oldp).w_tagstack[i as usize];
            if !(*tag).tagname.is_null() {
                (*tag).tagname = xstrdup((*tag).tagname);
            }
            if !(*tag).user_data.is_null() {
                (*tag).user_data = xstrdup((*tag).user_data);
            }
            i += 1;
        }
        (*newp).w_tagstackidx = (*oldp).w_tagstackidx;
        (*newp).w_tagstacklen = (*oldp).w_tagstacklen;
        (*newp).w_changelistidx = (*oldp).w_changelistidx;
        copyFoldingState(oldp, newp);
        win_init_some(newp, oldp);
        (*newp).w_winbar_height = (*oldp).w_winbar_height;
    }
}

unsafe extern "C" fn win_init_some(mut newp: *mut win_T, mut oldp: *mut win_T) {
    unsafe {
        (*newp).w_alist = (*oldp).w_alist;
        (*(*newp).w_alist).al_refcount += 1;
        (*newp).w_arg_idx = (*oldp).w_arg_idx;
        win_copy_options(oldp, newp);
    }
}
