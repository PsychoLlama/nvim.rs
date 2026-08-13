//! Setting a window's size explicitly -- `:resize`, and dragging a
//! separator.
//!
//! [`win_setheight_win`] and [`win_setwidth_win`] are the `:resize` /
//! `:vertical resize` entry points; [`frame_setheight`] and
//! [`frame_setwidth`] are the recursive half, which takes the room from the
//! frames around the one being sized, respecting the minimum sizes and the
//! `'winfix*'` pins, and grows an ancestor when the siblings cannot pay.
//! [`win_drag_status_line`] and [`win_drag_vsep_line`] are the mouse forms,
//! and [`win_comp_pos`]/[`frame_comp_pos`] recompute every window's screen
//! position afterwards.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::drawscreen::{
    UPD_NOT_VALID, UPD_SOME_VALID, UPD_VALID, redraw_all_later, redraw_later, showmode,
};
use crate::src::nvim::main::{
    Columns, Rows, cmdline_row, curwin, e_noroom, lastwin, p_ch, p_wmh, p_wmw, redraw_cmdline,
    topframe,
};
use crate::src::nvim::message::emsg;
use crate::src::nvim::os::libc::gettext;
use crate::src::nvim::types::{OptInt, frame_T, kFloatRelativeWindow, optset_T, tabpage_T, win_T};
use crate::src::nvim::winfloat::win_config_float;

pub unsafe extern "C" fn win_comp_pos() -> ::core::ffi::c_int {
    unsafe {
        let mut row: ::core::ffi::c_int = tabline_height();
        let mut col: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        frame_comp_pos(topframe.get(), &raw mut row, &raw mut col);
        let mut wp: *mut win_T = lastwin.get();
        while !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 {
            if (*wp).w_config.relative as ::core::ffi::c_uint
                == kFloatRelativeWindow as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                (*wp).w_pos_changed = true_0 != 0;
            }
            wp = (*wp).w_prev;
        }
        return row + global_stl_height();
    }
}

pub(crate) unsafe extern "C" fn frame_comp_pos(
    mut topfrp: *mut frame_T,
    mut row: *mut ::core::ffi::c_int,
    mut col: *mut ::core::ffi::c_int,
) {
    unsafe {
        let mut wp: *mut win_T = (*topfrp).fr_win;
        if !wp.is_null() {
            if (*wp).w_winrow != *row || (*wp).w_wincol != *col {
                (*wp).w_winrow = *row;
                (*wp).w_wincol = *col;
                redraw_later(wp, UPD_NOT_VALID);
                (*wp).w_redr_status = true_0 != 0;
                (*wp).w_pos_changed = true_0 != 0;
            }
            let h: ::core::ffi::c_int =
                (*wp).w_height + (*wp).w_hsep_height + (*wp).w_status_height;
            *row += if h > (*topfrp).fr_height {
                (*topfrp).fr_height
            } else {
                h
            };
            *col += (*wp).w_width + (*wp).w_vsep_width;
        } else {
            let mut startrow: ::core::ffi::c_int = *row;
            let mut startcol: ::core::ffi::c_int = *col;
            let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            frp = (*topfrp).fr_child;
            while !frp.is_null() {
                if (*topfrp).fr_layout as ::core::ffi::c_int == FR_ROW {
                    *row = startrow;
                } else {
                    *col = startcol;
                }
                frame_comp_pos(frp, row, col);
                frp = (*frp).fr_next;
            }
        };
    }
}

pub unsafe extern "C" fn win_setheight(mut height: ::core::ffi::c_int) {
    unsafe {
        win_setheight_win(height, curwin.get());
    }
}

pub unsafe extern "C" fn win_setheight_win(mut height: ::core::ffi::c_int, mut win: *mut win_T) {
    unsafe {
        height = if height
            > (if win == curwin.get() {
                if p_wmh.get() > 1 as OptInt {
                    p_wmh.get()
                } else {
                    1 as OptInt
                }
            } else {
                p_wmh.get()
            }) as ::core::ffi::c_int
                + (*win).w_winbar_height
        {
            height
        } else {
            (if win == curwin.get() {
                if p_wmh.get() > 1 as OptInt {
                    p_wmh.get()
                } else {
                    1 as OptInt
                }
            } else {
                p_wmh.get()
            }) as ::core::ffi::c_int
                + (*win).w_winbar_height
        };
        if (*win).w_floating {
            (*win).w_config.height = if height > 1 as ::core::ffi::c_int {
                height
            } else {
                1 as ::core::ffi::c_int
            };
            win_config_float(win, (*win).w_config);
            redraw_later(win, UPD_VALID);
        } else {
            frame_setheight(
                (*win).w_frame,
                // `height` came from `:resize`, which does not clamp it; the C
                // wraps here and `frame_setheight` clamps whatever comes out.
                height
                    .wrapping_add((*win).w_hsep_height)
                    .wrapping_add((*win).w_status_height),
            );
            win_comp_pos();
            win_fix_scroll(true_0 != 0);
            redraw_all_later(UPD_NOT_VALID);
            redraw_cmdline.set(true_0 != 0);
        };
    }
}

unsafe extern "C" fn frame_setheight(mut curfrp: *mut frame_T, mut height: ::core::ffi::c_int) {
    unsafe {
        if (*curfrp).fr_height == height {
            return;
        }
        if (*curfrp).fr_parent.is_null() {
            if height > 0 as ::core::ffi::c_int {
                frame_new_height(curfrp, height, false_0 != 0, false_0 != 0, true_0 != 0);
            }
        } else if (*(*curfrp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW {
            let mut h: ::core::ffi::c_int =
                frame_minheight((*curfrp).fr_parent, ::core::ptr::null_mut::<win_T>());
            height = if height > h { height } else { h };
            frame_setheight((*curfrp).fr_parent, height);
        } else {
            let mut room: ::core::ffi::c_int = 0;
            let mut room_cmdline: ::core::ffi::c_int = 0;
            let mut room_reserved: ::core::ffi::c_int = 0;
            let mut run: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while run <= 2 as ::core::ffi::c_int {
                room = 0 as ::core::ffi::c_int;
                room_reserved = 0 as ::core::ffi::c_int;
                let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
                frp = (*(*curfrp).fr_parent).fr_child;
                while !frp.is_null() {
                    if frp != curfrp
                        && !(*frp).fr_win.is_null()
                        && (*(*frp).fr_win).w_onebuf_opt.wo_wfh != 0
                    {
                        room_reserved += (*frp).fr_height;
                    }
                    room += (*frp).fr_height;
                    if frp != curfrp {
                        room -= frame_minheight(frp, ::core::ptr::null_mut::<win_T>());
                    }
                    frp = (*frp).fr_next;
                }
                if (*curfrp).fr_width != Columns.get() {
                    room_cmdline = 0 as ::core::ffi::c_int;
                } else {
                    let mut wp: *mut win_T =
                        lastwin_nofloating(::core::ptr::null_mut::<tabpage_T>());
                    room_cmdline = Rows.get()
                        - p_ch.get() as ::core::ffi::c_int
                        - global_stl_height()
                        - ((*wp).w_winrow
                            + (*wp).w_height
                            + (*wp).w_hsep_height
                            + (*wp).w_status_height);
                    room_cmdline = if room_cmdline > 0 as ::core::ffi::c_int {
                        room_cmdline
                    } else {
                        0 as ::core::ffi::c_int
                    };
                }
                if height <= room + room_cmdline {
                    break;
                }
                if run == 2 as ::core::ffi::c_int || (*curfrp).fr_width == Columns.get() {
                    height = room + room_cmdline;
                    break;
                } else {
                    frame_setheight(
                        (*curfrp).fr_parent,
                        height + frame_minheight((*curfrp).fr_parent, NOWIN)
                            - p_wmh.get() as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int,
                    );
                    run += 1;
                }
            }
            let mut take: ::core::ffi::c_int = height - (*curfrp).fr_height;
            if height > room + room_cmdline - room_reserved {
                room_reserved = room + room_cmdline - height;
            }
            if take < 0 as ::core::ffi::c_int && room - (*curfrp).fr_height <= room_reserved {
                room_reserved = 0 as ::core::ffi::c_int;
            }
            if take > 0 as ::core::ffi::c_int && room_cmdline > 0 as ::core::ffi::c_int {
                room_cmdline = if room_cmdline < take {
                    room_cmdline
                } else {
                    take
                };
                take -= room_cmdline;
                (*topframe.get()).fr_height += room_cmdline;
            }
            frame_new_height(curfrp, height, false_0 != 0, false_0 != 0, true_0 != 0);
            let mut run_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while run_0 < 2 as ::core::ffi::c_int {
                let mut frp_0: *mut frame_T = if run_0 == 0 as ::core::ffi::c_int {
                    (*curfrp).fr_next
                } else {
                    (*curfrp).fr_prev
                };
                while !frp_0.is_null() && take != 0 as ::core::ffi::c_int {
                    let mut h_0: ::core::ffi::c_int =
                        frame_minheight(frp_0, ::core::ptr::null_mut::<win_T>());
                    if room_reserved > 0 as ::core::ffi::c_int
                        && !(*frp_0).fr_win.is_null()
                        && (*(*frp_0).fr_win).w_onebuf_opt.wo_wfh != 0
                    {
                        if room_reserved >= (*frp_0).fr_height {
                            room_reserved -= (*frp_0).fr_height;
                        } else {
                            if (*frp_0).fr_height - room_reserved > take {
                                room_reserved = (*frp_0).fr_height - take;
                            }
                            take -= (*frp_0).fr_height - room_reserved;
                            frame_new_height(
                                frp_0,
                                room_reserved,
                                false_0 != 0,
                                false_0 != 0,
                                true_0 != 0,
                            );
                            room_reserved = 0 as ::core::ffi::c_int;
                        }
                    } else if (*frp_0).fr_height - take < h_0 {
                        take -= (*frp_0).fr_height - h_0;
                        frame_new_height(frp_0, h_0, false_0 != 0, false_0 != 0, true_0 != 0);
                    } else {
                        frame_new_height(
                            frp_0,
                            (*frp_0).fr_height - take,
                            false_0 != 0,
                            false_0 != 0,
                            true_0 != 0,
                        );
                        take = 0 as ::core::ffi::c_int;
                    }
                    if run_0 == 0 as ::core::ffi::c_int {
                        frp_0 = (*frp_0).fr_next;
                    } else {
                        frp_0 = (*frp_0).fr_prev;
                    }
                }
                run_0 += 1;
            }
        };
    }
}

pub unsafe extern "C" fn win_setwidth(mut width: ::core::ffi::c_int) {
    unsafe {
        win_setwidth_win(width, curwin.get());
    }
}

pub unsafe extern "C" fn win_setwidth_win(mut width: ::core::ffi::c_int, mut wp: *mut win_T) {
    unsafe {
        if wp == curwin.get() {
            width = if (if width > p_wmw.get() as ::core::ffi::c_int {
                width
            } else {
                p_wmw.get() as ::core::ffi::c_int
            }) > 1 as ::core::ffi::c_int
            {
                if width > p_wmw.get() as ::core::ffi::c_int {
                    width
                } else {
                    p_wmw.get() as ::core::ffi::c_int
                }
            } else {
                1 as ::core::ffi::c_int
            };
        } else if width < 0 as ::core::ffi::c_int {
            width = 0 as ::core::ffi::c_int;
        }
        if (*wp).w_floating {
            (*wp).w_config.width = width;
            win_config_float(wp, (*wp).w_config);
            redraw_later(wp, UPD_NOT_VALID);
        } else {
            frame_setwidth((*wp).w_frame, width + (*wp).w_vsep_width);
            win_comp_pos();
            redraw_all_later(UPD_NOT_VALID);
        };
    }
}

pub(crate) unsafe extern "C" fn frame_setwidth(
    mut curfrp: *mut frame_T,
    mut width: ::core::ffi::c_int,
) {
    unsafe {
        if (*curfrp).fr_width == width {
            return;
        }
        if (*curfrp).fr_parent.is_null() {
            return;
        }
        if (*(*curfrp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL {
            let mut w: ::core::ffi::c_int =
                frame_minwidth((*curfrp).fr_parent, ::core::ptr::null_mut::<win_T>());
            width = if width > w { width } else { w };
            frame_setwidth((*curfrp).fr_parent, width);
        } else {
            let mut room: ::core::ffi::c_int = 0;
            let mut room_reserved: ::core::ffi::c_int = 0;
            let mut run: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
            while run <= 2 as ::core::ffi::c_int {
                room = 0 as ::core::ffi::c_int;
                room_reserved = 0 as ::core::ffi::c_int;
                let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
                frp = (*(*curfrp).fr_parent).fr_child;
                while !frp.is_null() {
                    if frp != curfrp
                        && !(*frp).fr_win.is_null()
                        && (*(*frp).fr_win).w_onebuf_opt.wo_wfw != 0
                    {
                        room_reserved += (*frp).fr_width;
                    }
                    room += (*frp).fr_width;
                    if frp != curfrp {
                        room -= frame_minwidth(frp, ::core::ptr::null_mut::<win_T>());
                    }
                    frp = (*frp).fr_next;
                }
                if width <= room {
                    break;
                }
                if run == 2 as ::core::ffi::c_int
                    || (*curfrp).fr_height as OptInt
                        >= Rows.get() as OptInt
                            - p_ch.get()
                            - tabline_height() as OptInt
                            - global_stl_height() as OptInt
                {
                    width = room;
                    break;
                } else {
                    frame_setwidth(
                        (*curfrp).fr_parent,
                        width + frame_minwidth((*curfrp).fr_parent, NOWIN)
                            - p_wmw.get() as ::core::ffi::c_int
                            - 1 as ::core::ffi::c_int,
                    );
                    run += 1;
                }
            }
            let mut take: ::core::ffi::c_int = width - (*curfrp).fr_width;
            if width > room - room_reserved {
                room_reserved = room - width;
            }
            if take < 0 as ::core::ffi::c_int && room - (*curfrp).fr_width < room_reserved {
                room_reserved = 0 as ::core::ffi::c_int;
            }
            frame_new_width(curfrp, width, false_0 != 0, false_0 != 0);
            let mut run_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while run_0 < 2 as ::core::ffi::c_int {
                let mut frp_0: *mut frame_T = if run_0 == 0 as ::core::ffi::c_int {
                    (*curfrp).fr_next
                } else {
                    (*curfrp).fr_prev
                };
                while !frp_0.is_null() && take != 0 as ::core::ffi::c_int {
                    let mut w_0: ::core::ffi::c_int =
                        frame_minwidth(frp_0, ::core::ptr::null_mut::<win_T>());
                    if room_reserved > 0 as ::core::ffi::c_int
                        && !(*frp_0).fr_win.is_null()
                        && (*(*frp_0).fr_win).w_onebuf_opt.wo_wfw != 0
                    {
                        if room_reserved >= (*frp_0).fr_width {
                            room_reserved -= (*frp_0).fr_width;
                        } else {
                            if (*frp_0).fr_width - room_reserved > take {
                                room_reserved = (*frp_0).fr_width - take;
                            }
                            take -= (*frp_0).fr_width - room_reserved;
                            frame_new_width(frp_0, room_reserved, false_0 != 0, false_0 != 0);
                            room_reserved = 0 as ::core::ffi::c_int;
                        }
                    } else if (*frp_0).fr_width - take < w_0 {
                        take -= (*frp_0).fr_width - w_0;
                        frame_new_width(frp_0, w_0, false_0 != 0, false_0 != 0);
                    } else {
                        frame_new_width(
                            frp_0,
                            (*frp_0).fr_width - take,
                            false_0 != 0,
                            false_0 != 0,
                        );
                        take = 0 as ::core::ffi::c_int;
                    }
                    if run_0 == 0 as ::core::ffi::c_int {
                        frp_0 = (*frp_0).fr_next;
                    } else {
                        frp_0 = (*frp_0).fr_prev;
                    }
                }
                run_0 += 1;
            }
        };
    }
}

pub unsafe extern "C" fn did_set_winminheight(
    mut _args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut first: bool = true_0 != 0;
        while p_wmh.get() > 0 as OptInt {
            let room: ::core::ffi::c_int = Rows.get() - p_ch.get() as ::core::ffi::c_int;
            let needed: ::core::ffi::c_int = min_rows_for_all_tabpages();
            if room >= needed {
                break;
            }
            (*p_wmh.ptr()) -= 1;
            if first {
                emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
                first = false_0 != 0;
            }
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn did_set_winminwidth(
    mut _args: *mut optset_T,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut first: bool = true_0 != 0;
        while p_wmw.get() > 0 as OptInt {
            let room: ::core::ffi::c_int = Columns.get();
            let needed: ::core::ffi::c_int =
                frame_minwidth(topframe.get(), ::core::ptr::null_mut::<win_T>());
            if room >= needed {
                break;
            }
            (*p_wmw.ptr()) -= 1;
            if first {
                emsg(gettext(&raw const e_noroom as *const ::core::ffi::c_char));
                first = false_0 != 0;
            }
        }
        return ::core::ptr::null::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn win_drag_status_line(
    mut dragwin: *mut win_T,
    mut offset: ::core::ffi::c_int,
) {
    unsafe {
        let mut fr: *mut frame_T = (*dragwin).w_frame;
        let mut curfr: *mut frame_T = fr;
        if fr != topframe.get() {
            fr = (*fr).fr_parent;
            if (*fr).fr_layout as ::core::ffi::c_int != FR_COL {
                curfr = fr;
                if fr != topframe.get() {
                    fr = (*fr).fr_parent;
                }
            }
        }
        while curfr != topframe.get() && (*curfr).fr_next.is_null() {
            if fr != topframe.get() {
                fr = (*fr).fr_parent;
            }
            curfr = fr;
            if fr != topframe.get() {
                fr = (*fr).fr_parent;
            }
        }
        let mut room: ::core::ffi::c_int = 0;
        let up: bool = offset < 0 as ::core::ffi::c_int;
        if up {
            offset = -offset;
            if fr == curfr {
                room = (*fr).fr_height - frame_minheight(fr, ::core::ptr::null_mut::<win_T>());
            } else {
                room = 0 as ::core::ffi::c_int;
                fr = (*fr).fr_child;
                loop {
                    room += (*fr).fr_height - frame_minheight(fr, ::core::ptr::null_mut::<win_T>());
                    if fr == curfr {
                        break;
                    }
                    fr = (*fr).fr_next;
                }
            }
            fr = (*curfr).fr_next;
        } else {
            room = Rows.get() - cmdline_row.get();
            if !(*curfr).fr_next.is_null() {
                room -= p_ch.get() as ::core::ffi::c_int + global_stl_height();
            } else if min_set_ch.get() > 0 as OptInt {
                room -= 1;
            }
            room = if room > 0 as ::core::ffi::c_int {
                room
            } else {
                0 as ::core::ffi::c_int
            };
            fr = (*curfr).fr_next;
            while !fr.is_null() {
                room += (*fr).fr_height - frame_minheight(fr, ::core::ptr::null_mut::<win_T>());
                fr = (*fr).fr_next;
            }
            fr = curfr;
        }
        offset = if offset < room { offset } else { room };
        if offset <= 0 as ::core::ffi::c_int {
            return;
        }
        if !fr.is_null() {
            frame_new_height(fr, (*fr).fr_height + offset, up, false_0 != 0, true_0 != 0);
        }
        if up {
            fr = curfr;
        } else {
            fr = (*curfr).fr_next;
        }
        while !fr.is_null() && offset > 0 as ::core::ffi::c_int {
            let mut n: ::core::ffi::c_int = frame_minheight(fr, ::core::ptr::null_mut::<win_T>());
            if (*fr).fr_height - offset <= n {
                offset -= (*fr).fr_height - n;
                frame_new_height(fr, n, !up, false_0 != 0, true_0 != 0);
                if up {
                    fr = (*fr).fr_prev;
                } else {
                    fr = (*fr).fr_next;
                }
            } else {
                frame_new_height(fr, (*fr).fr_height - offset, !up, false_0 != 0, true_0 != 0);
                break;
            }
        }
        win_comp_pos();
        win_fix_scroll(true_0 != 0);
        redraw_all_later(UPD_SOME_VALID);
        showmode();
    }
}

pub unsafe extern "C" fn win_drag_vsep_line(
    mut dragwin: *mut win_T,
    mut offset: ::core::ffi::c_int,
) {
    unsafe {
        let mut fr: *mut frame_T = (*dragwin).w_frame;
        if fr == topframe.get() {
            return;
        }
        let mut curfr: *mut frame_T = fr;
        fr = (*fr).fr_parent;
        if (*fr).fr_layout as ::core::ffi::c_int != FR_ROW {
            if fr == topframe.get() {
                return;
            }
            curfr = fr;
            fr = (*fr).fr_parent;
        }
        while (*curfr).fr_next.is_null() {
            if fr == topframe.get() {
                break;
            }
            curfr = fr;
            fr = (*fr).fr_parent;
            if fr != topframe.get() {
                curfr = fr;
                fr = (*fr).fr_parent;
            }
        }
        let mut room: ::core::ffi::c_int = 0;
        let left: bool = offset < 0 as ::core::ffi::c_int;
        if left {
            offset = -offset;
            room = 0 as ::core::ffi::c_int;
            fr = (*fr).fr_child;
            loop {
                room += (*fr).fr_width - frame_minwidth(fr, ::core::ptr::null_mut::<win_T>());
                if fr == curfr {
                    break;
                }
                fr = (*fr).fr_next;
            }
            fr = (*curfr).fr_next;
        } else {
            room = 0 as ::core::ffi::c_int;
            fr = (*curfr).fr_next;
            while !fr.is_null() {
                room += (*fr).fr_width - frame_minwidth(fr, ::core::ptr::null_mut::<win_T>());
                fr = (*fr).fr_next;
            }
            fr = curfr;
        }
        offset = if offset < room { offset } else { room };
        if offset <= 0 as ::core::ffi::c_int {
            return;
        }
        if fr.is_null() {
            return;
        }
        frame_new_width(fr, (*fr).fr_width + offset, left, false_0 != 0);
        if left {
            fr = curfr;
        } else {
            fr = (*curfr).fr_next;
        }
        while !fr.is_null() && offset > 0 as ::core::ffi::c_int {
            let mut n: ::core::ffi::c_int = frame_minwidth(fr, ::core::ptr::null_mut::<win_T>());
            if (*fr).fr_width - offset <= n {
                offset -= (*fr).fr_width - n;
                frame_new_width(fr, n, !left, false_0 != 0);
                if left {
                    fr = (*fr).fr_prev;
                } else {
                    fr = (*fr).fr_next;
                }
            } else {
                frame_new_width(fr, (*fr).fr_width - offset, !left, false_0 != 0);
                break;
            }
        }
        win_comp_pos();
        redraw_all_later(UPD_NOT_VALID);
    }
}
