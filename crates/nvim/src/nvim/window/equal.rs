//! `win_equal()` -- redistributing the space between windows.
//!
//! The CTRL-W = operation and the `'equalalways'` implementation:
//! [`win_equal_rec`] walks the frame tree handing each child its share of the
//! room, honouring `'eadirection'`, the `'winfix{height,width}'` pins, the
//! minimum sizes and the status lines and separators that are not text, and
//! recursing into rows and columns until every leaf has been given a
//! size.
//!
//! Original: `src/nvim/window.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::autocmd::is_aucmd_win;
use crate::src::nvim::drawscreen::{UPD_NOT_VALID, redraw_all_later};
use crate::src::nvim::main::{
    Columns, cmdline_row, curwin, p_ead, p_ls, p_wh, p_wiw, p_wmh, p_wmw, topframe,
};
use crate::src::nvim::types::{OptInt, frame_T, win_T};

pub unsafe extern "C" fn win_equal(
    mut next_curwin: *mut win_T,
    mut current: bool,
    mut dir: ::core::ffi::c_int,
) {
    unsafe {
        if dir == 0 as ::core::ffi::c_int {
            dir = *p_ead.get() as ::core::ffi::c_uchar as ::core::ffi::c_int;
        }
        win_equal_rec(
            if next_curwin.is_null() {
                curwin.get()
            } else {
                next_curwin
            },
            current,
            topframe.get(),
            dir,
            0 as ::core::ffi::c_int,
            tabline_height(),
            Columns.get(),
            (*topframe.get()).fr_height,
        );
        if !is_aucmd_win(next_curwin) {
            win_fix_scroll(true_0 != 0);
        }
    }
}

unsafe extern "C" fn win_equal_rec(
    mut next_curwin: *mut win_T,
    mut current: bool,
    mut topfr: *mut frame_T,
    mut dir: ::core::ffi::c_int,
    mut col: ::core::ffi::c_int,
    mut row: ::core::ffi::c_int,
    mut width: ::core::ffi::c_int,
    mut height: ::core::ffi::c_int,
) {
    unsafe {
        let mut extra_sep: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut totwincount: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut next_curwin_size: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut room: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut has_next_curwin: bool = false_0 != 0;
        if (*topfr).fr_layout as ::core::ffi::c_int == FR_LEAF {
            if (*topfr).fr_height != height
                || (*(*topfr).fr_win).w_winrow != row
                || (*topfr).fr_width != width
                || (*(*topfr).fr_win).w_wincol != col
            {
                (*(*topfr).fr_win).w_winrow = row;
                frame_new_height(topfr, height, false_0 != 0, false_0 != 0, false_0 != 0);
                (*(*topfr).fr_win).w_wincol = col;
                frame_new_width(topfr, width, false_0 != 0, false_0 != 0);
                redraw_all_later(UPD_NOT_VALID);
            }
        } else if (*topfr).fr_layout as ::core::ffi::c_int == FR_ROW {
            (*topfr).fr_width = width;
            (*topfr).fr_height = height;
            if dir != 'v' as ::core::ffi::c_int {
                let mut n: ::core::ffi::c_int = frame_minwidth(topfr, NOWIN);
                if col + width == Columns.get() {
                    extra_sep = 1 as ::core::ffi::c_int;
                } else {
                    extra_sep = 0 as ::core::ffi::c_int;
                }
                totwincount =
                    (n + extra_sep) / (p_wmw.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int);
                has_next_curwin = frame_has_win(topfr, next_curwin);
                let mut m: ::core::ffi::c_int = frame_minwidth(topfr, next_curwin);
                room = width - m;
                if room < 0 as ::core::ffi::c_int {
                    next_curwin_size = p_wiw.get() as ::core::ffi::c_int + room;
                    room = 0 as ::core::ffi::c_int;
                } else {
                    next_curwin_size = -1 as ::core::ffi::c_int;
                    let mut fr: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
                    fr = (*topfr).fr_child;
                    while !fr.is_null() {
                        if frame_fixed_width(fr) {
                            n = frame_minwidth(fr, NOWIN);
                            let mut new_size: ::core::ffi::c_int = (*fr).fr_width;
                            if frame_has_win(fr, next_curwin) {
                                room += p_wiw.get() as ::core::ffi::c_int
                                    - p_wmw.get() as ::core::ffi::c_int;
                                next_curwin_size = 0 as ::core::ffi::c_int;
                                new_size = if new_size > p_wiw.get() as ::core::ffi::c_int {
                                    new_size
                                } else {
                                    p_wiw.get() as ::core::ffi::c_int
                                };
                            } else {
                                totwincount -= (n
                                    + (if (*fr).fr_next.is_null() {
                                        extra_sep
                                    } else {
                                        0 as ::core::ffi::c_int
                                    }))
                                    / (p_wmw.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int);
                            }
                            room -= new_size - n;
                            if room < 0 as ::core::ffi::c_int {
                                new_size += room;
                                room = 0 as ::core::ffi::c_int;
                            }
                            (*fr).fr_newwidth = new_size;
                        }
                        fr = (*fr).fr_next;
                    }
                    if next_curwin_size == -1 as ::core::ffi::c_int {
                        if !has_next_curwin {
                            next_curwin_size = 0 as ::core::ffi::c_int;
                        } else if totwincount > 1 as ::core::ffi::c_int
                            && ((room + (totwincount - 2 as ::core::ffi::c_int))
                                / (totwincount - 1 as ::core::ffi::c_int))
                                as OptInt
                                > p_wiw.get()
                        {
                            next_curwin_size = (room as OptInt
                                + p_wiw.get()
                                + (totwincount - 1 as ::core::ffi::c_int) as OptInt * p_wmw.get()
                                + (totwincount - 1 as ::core::ffi::c_int) as OptInt)
                                as ::core::ffi::c_int
                                / totwincount;
                            room -= next_curwin_size - p_wiw.get() as ::core::ffi::c_int;
                        } else {
                            next_curwin_size = p_wiw.get() as ::core::ffi::c_int;
                        }
                    }
                }
                if has_next_curwin {
                    totwincount -= 1;
                }
            }
            let mut fr_0: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            fr_0 = (*topfr).fr_child;
            while !fr_0.is_null() {
                let mut wincount: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                let mut new_size_0: ::core::ffi::c_int = 0;
                if (*fr_0).fr_next.is_null() {
                    new_size_0 = width;
                } else if dir == 'v' as ::core::ffi::c_int {
                    new_size_0 = (*fr_0).fr_width;
                } else if frame_fixed_width(fr_0) {
                    new_size_0 = (*fr_0).fr_newwidth;
                    wincount = 0 as ::core::ffi::c_int;
                } else {
                    let mut n_0: ::core::ffi::c_int = frame_minwidth(fr_0, NOWIN);
                    wincount =
                        (n_0 + (if (*fr_0).fr_next.is_null() {
                            extra_sep
                        } else {
                            0 as ::core::ffi::c_int
                        })) / (p_wmw.get() as ::core::ffi::c_int + 1 as ::core::ffi::c_int);
                    let mut m_0: ::core::ffi::c_int = frame_minwidth(fr_0, next_curwin);
                    let mut hnc: bool = has_next_curwin as ::core::ffi::c_int != 0
                        && frame_has_win(fr_0, next_curwin) as ::core::ffi::c_int != 0;
                    if hnc {
                        wincount -= 1;
                    }
                    if totwincount == 0 as ::core::ffi::c_int {
                        new_size_0 = room;
                    } else {
                        new_size_0 =
                            (wincount * room + totwincount / 2 as ::core::ffi::c_int) / totwincount;
                    }
                    if hnc {
                        next_curwin_size -= p_wiw.get() as ::core::ffi::c_int - (m_0 - n_0);
                        next_curwin_size = if next_curwin_size > 0 as ::core::ffi::c_int {
                            next_curwin_size
                        } else {
                            0 as ::core::ffi::c_int
                        };
                        new_size_0 += next_curwin_size;
                        room -= new_size_0 - next_curwin_size;
                    } else {
                        room -= new_size_0;
                    }
                    new_size_0 += n_0;
                }
                if !current
                    || dir != 'v' as ::core::ffi::c_int
                    || !(*topfr).fr_parent.is_null()
                    || new_size_0 != (*fr_0).fr_width
                    || frame_has_win(fr_0, next_curwin) as ::core::ffi::c_int != 0
                {
                    win_equal_rec(
                        next_curwin,
                        current,
                        fr_0,
                        dir,
                        col,
                        row,
                        new_size_0,
                        height,
                    );
                }
                col += new_size_0;
                width -= new_size_0;
                totwincount -= wincount;
                fr_0 = (*fr_0).fr_next;
            }
        } else {
            (*topfr).fr_width = width;
            (*topfr).fr_height = height;
            if dir != 'h' as ::core::ffi::c_int {
                let mut n_1: ::core::ffi::c_int = frame_minheight(topfr, NOWIN);
                if row + height >= cmdline_row.get() && p_ls.get() == 0 as OptInt {
                    extra_sep = STATUS_HEIGHT as ::core::ffi::c_int;
                } else if global_stl_height() > 0 as ::core::ffi::c_int {
                    extra_sep = 1 as ::core::ffi::c_int;
                } else {
                    extra_sep = 0 as ::core::ffi::c_int;
                }
                totwincount = get_maximum_wincount(topfr, n_1 + extra_sep);
                has_next_curwin = frame_has_win(topfr, next_curwin);
                let mut m_1: ::core::ffi::c_int = frame_minheight(topfr, next_curwin);
                room = height - m_1;
                if room < 0 as ::core::ffi::c_int {
                    next_curwin_size = p_wh.get() as ::core::ffi::c_int + room;
                    room = 0 as ::core::ffi::c_int;
                } else {
                    next_curwin_size = -1 as ::core::ffi::c_int;
                    let mut fr_1: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
                    fr_1 = (*topfr).fr_child;
                    while !fr_1.is_null() {
                        if frame_fixed_height(fr_1) {
                            n_1 = frame_minheight(fr_1, NOWIN);
                            let mut new_size_1: ::core::ffi::c_int = (*fr_1).fr_height;
                            if frame_has_win(fr_1, next_curwin) {
                                room += p_wh.get() as ::core::ffi::c_int
                                    - p_wmh.get() as ::core::ffi::c_int;
                                next_curwin_size = 0 as ::core::ffi::c_int;
                                new_size_1 = if new_size_1 > p_wh.get() as ::core::ffi::c_int {
                                    new_size_1
                                } else {
                                    p_wh.get() as ::core::ffi::c_int
                                };
                            } else {
                                totwincount -= get_maximum_wincount(
                                    fr_1,
                                    n_1 + (if (*fr_1).fr_next.is_null() {
                                        extra_sep
                                    } else {
                                        0 as ::core::ffi::c_int
                                    }),
                                );
                            }
                            room -= new_size_1 - n_1;
                            if room < 0 as ::core::ffi::c_int {
                                new_size_1 += room;
                                room = 0 as ::core::ffi::c_int;
                            }
                            (*fr_1).fr_newheight = new_size_1;
                        }
                        fr_1 = (*fr_1).fr_next;
                    }
                    if next_curwin_size == -1 as ::core::ffi::c_int {
                        if !has_next_curwin {
                            next_curwin_size = 0 as ::core::ffi::c_int;
                        } else if totwincount > 1 as ::core::ffi::c_int
                            && ((room + (totwincount - 2 as ::core::ffi::c_int))
                                / (totwincount - 1 as ::core::ffi::c_int))
                                as OptInt
                                > p_wh.get()
                        {
                            next_curwin_size = (room as OptInt
                                + p_wh.get()
                                + (totwincount - 1 as ::core::ffi::c_int) as OptInt * p_wmh.get()
                                + (totwincount - 1 as ::core::ffi::c_int) as OptInt)
                                as ::core::ffi::c_int
                                / totwincount;
                            room -= next_curwin_size - p_wh.get() as ::core::ffi::c_int;
                        } else {
                            next_curwin_size = p_wh.get() as ::core::ffi::c_int;
                        }
                    }
                }
                if has_next_curwin {
                    totwincount -= 1;
                }
            }
            let mut fr_2: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
            fr_2 = (*topfr).fr_child;
            while !fr_2.is_null() {
                let mut new_size_2: ::core::ffi::c_int = 0;
                let mut wincount_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                if (*fr_2).fr_next.is_null() {
                    new_size_2 = height;
                } else if dir == 'h' as ::core::ffi::c_int {
                    new_size_2 = (*fr_2).fr_height;
                } else if frame_fixed_height(fr_2) {
                    new_size_2 = (*fr_2).fr_newheight;
                    wincount_0 = 0 as ::core::ffi::c_int;
                } else {
                    let mut n_2: ::core::ffi::c_int = frame_minheight(fr_2, NOWIN);
                    wincount_0 = get_maximum_wincount(
                        fr_2,
                        n_2 + (if (*fr_2).fr_next.is_null() {
                            extra_sep
                        } else {
                            0 as ::core::ffi::c_int
                        }),
                    );
                    let mut m_2: ::core::ffi::c_int = frame_minheight(fr_2, next_curwin);
                    let mut hnc_0: bool = has_next_curwin as ::core::ffi::c_int != 0
                        && frame_has_win(fr_2, next_curwin) as ::core::ffi::c_int != 0;
                    if hnc_0 {
                        wincount_0 -= 1;
                    }
                    if totwincount == 0 as ::core::ffi::c_int {
                        new_size_2 = room;
                    } else {
                        new_size_2 = (wincount_0 * room + totwincount / 2 as ::core::ffi::c_int)
                            / totwincount;
                    }
                    if hnc_0 {
                        next_curwin_size -= p_wh.get() as ::core::ffi::c_int - (m_2 - n_2);
                        new_size_2 += next_curwin_size;
                        room -= new_size_2 - next_curwin_size;
                    } else {
                        room -= new_size_2;
                    }
                    new_size_2 += n_2;
                }
                if !current
                    || dir != 'h' as ::core::ffi::c_int
                    || !(*topfr).fr_parent.is_null()
                    || new_size_2 != (*fr_2).fr_height
                    || frame_has_win(fr_2, next_curwin) as ::core::ffi::c_int != 0
                {
                    win_equal_rec(next_curwin, current, fr_2, dir, col, row, width, new_size_2);
                }
                row += new_size_2;
                height -= new_size_2;
                totwincount -= wincount_0;
                fr_2 = (*fr_2).fr_next;
            }
        };
    }
}
