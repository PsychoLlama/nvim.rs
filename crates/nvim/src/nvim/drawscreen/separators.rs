//! The lines between windows, and the glyphs where they meet.
//!
//! [`draw_vsep_win`] and [`draw_hsep_win`] draw the separator right of and below
//! one window. The interesting half is the corners: with the global statusline
//! (`'laststatus'` 3) a window boundary can be a T or a cross, and
//! [`draw_sep_connectors_win`] picks the right `'fillchars'` glyph for each of a
//! window's four corners by asking [`vsep_connected`] and [`hsep_connected`]
//! whether a neighbouring window's separator continues through it. Both walk the
//! frame tree to the neighbour at that row or column.
//!
//! [`win_redraw_signcols`] is here for a different reason: it is the one
//! per-window recomputation `win_update` does before deciding what to redraw, and
//! it answers whether the sign column changed width.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub type WindowCorner = ::core::ffi::c_uint;

pub const WC_BOTTOM_RIGHT: WindowCorner = 3;

pub const WC_BOTTOM_LEFT: WindowCorner = 2;

pub const WC_TOP_RIGHT: WindowCorner = 1;

pub const WC_TOP_LEFT: WindowCorner = 0;

pub(crate) unsafe extern "C" fn win_redraw_signcols(mut wp: *mut win_T) -> bool {
    unsafe {
        let mut buf: *mut buf_T = (*wp).w_buffer;
        if !(*buf).b_signcols.autom
            && (*(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL
                || (*wp).w_maxscwidth > 1 as ::core::ffi::c_int
                    && (*wp).w_minscwidth != (*wp).w_maxscwidth)
        {
            (*buf).b_signcols.autom = true_0 != 0;
            buf_signcols_count_range(
                buf,
                0 as ::core::ffi::c_int,
                (*buf).b_ml.ml_line_count as ::core::ffi::c_int - 1 as ::core::ffi::c_int,
                MAXLNUM as ::core::ffi::c_int,
                kFalse,
            );
        }
        while (*buf).b_signcols.max > 0 as ::core::ffi::c_int
            && (*buf).b_signcols.count[((*buf).b_signcols.max - 1 as ::core::ffi::c_int) as usize]
                == 0 as ::core::ffi::c_int
        {
            (*buf).b_signcols.max -= 1;
        }
        let mut width: ::core::ffi::c_int = if (*wp).w_maxscwidth < (*buf).b_signcols.max {
            (*wp).w_maxscwidth
        } else {
            (*buf).b_signcols.max
        };
        let mut rebuild_stc: bool = (*buf).b_signcols.max != (*buf).b_signcols.last_max
            && *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int != NUL;
        if rebuild_stc {
            (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
        } else if (*wp).w_minscwidth == 0 as ::core::ffi::c_int
            && (*wp).w_maxscwidth == 1 as ::core::ffi::c_int
        {
            width = (buf_meta_total(buf, kMTMetaSignText) > 0 as uint32_t) as ::core::ffi::c_int;
        }
        let mut scwidth: ::core::ffi::c_int = (*wp).w_scwidth;
        (*wp).w_scwidth = if (if 0 as ::core::ffi::c_int > (*wp).w_minscwidth {
            0 as ::core::ffi::c_int
        } else {
            (*wp).w_minscwidth
        }) > width
        {
            if 0 as ::core::ffi::c_int > (*wp).w_minscwidth {
                0 as ::core::ffi::c_int
            } else {
                (*wp).w_minscwidth
            }
        } else {
            width
        };
        return (*wp).w_scwidth != scwidth || rebuild_stc as ::core::ffi::c_int != 0;
    }
}

pub(crate) unsafe extern "C" fn hsep_connected(
    mut wp: *mut win_T,
    mut corner: WindowCorner,
) -> bool {
    unsafe {
        let mut before: bool = corner as ::core::ffi::c_uint
            == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
            || corner as ::core::ffi::c_uint
                == WC_BOTTOM_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint;
        let mut sep_row: ::core::ffi::c_int = if corner as ::core::ffi::c_uint
            == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
            || corner as ::core::ffi::c_uint
                == WC_TOP_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*wp).w_winrow - 1 as ::core::ffi::c_int
        } else {
            (*wp).w_winrow + (*wp).w_height
        };
        let mut fr: *mut frame_T = (*wp).w_frame;
        while !(*fr).fr_parent.is_null() {
            if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
                && !(if before as ::core::ffi::c_int != 0 {
                    (*fr).fr_prev
                } else {
                    (*fr).fr_next
                })
                .is_null()
            {
                fr = if before as ::core::ffi::c_int != 0 {
                    (*fr).fr_prev
                } else {
                    (*fr).fr_next
                };
                break;
            } else {
                fr = (*fr).fr_parent;
            }
        }
        if (*fr).fr_parent.is_null() {
            return false_0 != 0;
        }
        while (*fr).fr_layout as ::core::ffi::c_int != FR_LEAF {
            fr = (*fr).fr_child;
            if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
                && before as ::core::ffi::c_int != 0
            {
                while !(*fr).fr_next.is_null() {
                    fr = (*fr).fr_next;
                }
            } else {
                while !(*fr).fr_next.is_null()
                    && (*frame2win(fr)).w_winrow + (*fr).fr_height < sep_row
                {
                    fr = (*fr).fr_next;
                }
            }
        }
        return sep_row == (*(*fr).fr_win).w_winrow - 1 as ::core::ffi::c_int
            || sep_row == (*(*fr).fr_win).w_winrow + (*(*fr).fr_win).w_height;
    }
}

pub(crate) unsafe extern "C" fn vsep_connected(
    mut wp: *mut win_T,
    mut corner: WindowCorner,
) -> bool {
    unsafe {
        let mut before: bool = corner as ::core::ffi::c_uint
            == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
            || corner as ::core::ffi::c_uint
                == WC_TOP_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint;
        let mut sep_col: ::core::ffi::c_int = if corner as ::core::ffi::c_uint
            == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
            || corner as ::core::ffi::c_uint
                == WC_BOTTOM_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            (*wp).w_wincol - 1 as ::core::ffi::c_int
        } else {
            (*wp).w_wincol + (*wp).w_width
        };
        let mut fr: *mut frame_T = (*wp).w_frame;
        while !(*fr).fr_parent.is_null() {
            if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
                && !(if before as ::core::ffi::c_int != 0 {
                    (*fr).fr_prev
                } else {
                    (*fr).fr_next
                })
                .is_null()
            {
                fr = if before as ::core::ffi::c_int != 0 {
                    (*fr).fr_prev
                } else {
                    (*fr).fr_next
                };
                break;
            } else {
                fr = (*fr).fr_parent;
            }
        }
        if (*fr).fr_parent.is_null() {
            return false_0 != 0;
        }
        while (*fr).fr_layout as ::core::ffi::c_int != FR_LEAF {
            fr = (*fr).fr_child;
            if (*(*fr).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
                && before as ::core::ffi::c_int != 0
            {
                while !(*fr).fr_next.is_null() {
                    fr = (*fr).fr_next;
                }
            } else {
                while !(*fr).fr_next.is_null()
                    && (*frame2win(fr)).w_wincol + (*fr).fr_width < sep_col
                {
                    fr = (*fr).fr_next;
                }
            }
        }
        return sep_col == (*(*fr).fr_win).w_wincol - 1 as ::core::ffi::c_int
            || sep_col == (*(*fr).fr_win).w_wincol + (*(*fr).fr_win).w_width;
    }
}

pub(crate) unsafe extern "C" fn draw_vsep_win(mut wp: *mut win_T) {
    unsafe {
        if (*wp).w_vsep_width == 0 {
            return;
        }
        let mut row: ::core::ffi::c_int = (*wp).w_winrow;
        while row < (*wp).w_winrow + (*wp).w_height {
            grid_line_start(default_gridview.ptr(), row);
            grid_line_put_schar(
                (*wp).w_wincol + (*wp).w_width,
                (*wp).w_p_fcs_chars.vert,
                win_hl_attr(wp, HLF_C),
            );
            grid_line_flush();
            row += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn draw_hsep_win(mut wp: *mut win_T) {
    unsafe {
        if (*wp).w_hsep_height == 0 {
            return;
        }
        grid_line_start(default_gridview.ptr(), (*wp).w_winrow + (*wp).w_height);
        grid_line_fill(
            (*wp).w_wincol,
            (*wp).w_wincol + (*wp).w_width,
            (*wp).w_p_fcs_chars.horiz,
            win_hl_attr(wp, HLF_C),
        );
        grid_line_flush();
    }
}

pub(crate) unsafe extern "C" fn get_corner_sep_connector(
    mut wp: *mut win_T,
    mut corner: WindowCorner,
) -> schar_T {
    unsafe {
        if vsep_connected(wp, corner) {
            if hsep_connected(wp, corner) {
                return (*wp).w_p_fcs_chars.verthoriz;
            } else if corner as ::core::ffi::c_uint
                == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
                || corner as ::core::ffi::c_uint
                    == WC_BOTTOM_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                return (*wp).w_p_fcs_chars.vertright;
            } else {
                return (*wp).w_p_fcs_chars.vertleft;
            }
        } else if corner as ::core::ffi::c_uint
            == WC_TOP_LEFT as ::core::ffi::c_int as ::core::ffi::c_uint
            || corner as ::core::ffi::c_uint
                == WC_TOP_RIGHT as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            return (*wp).w_p_fcs_chars.horizdown;
        } else {
            return (*wp).w_p_fcs_chars.horizup;
        };
    }
}

pub(crate) unsafe extern "C" fn draw_sep_connectors_win(mut wp: *mut win_T) {
    unsafe {
        if global_stl_height() == 0 as ::core::ffi::c_int
            || !((*wp).w_hsep_height == 1 as ::core::ffi::c_int
                || (*wp).w_vsep_width == 1 as ::core::ffi::c_int)
        {
            return;
        }
        let mut hl: ::core::ffi::c_int = win_hl_attr(wp, HLF_C);
        let mut win_at_top: bool = false;
        let mut win_at_bottom: bool = (*wp).w_hsep_height == 0 as ::core::ffi::c_int;
        let mut win_at_left: bool = false;
        let mut win_at_right: bool = (*wp).w_vsep_width == 0 as ::core::ffi::c_int;
        let mut frp: *mut frame_T = ::core::ptr::null_mut::<frame_T>();
        frp = (*wp).w_frame;
        while !(*frp).fr_parent.is_null() {
            if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_COL
                && !(*frp).fr_prev.is_null()
            {
                break;
            }
            frp = (*frp).fr_parent;
        }
        win_at_top = (*frp).fr_parent.is_null();
        frp = (*wp).w_frame;
        while !(*frp).fr_parent.is_null() {
            if (*(*frp).fr_parent).fr_layout as ::core::ffi::c_int == FR_ROW
                && !(*frp).fr_prev.is_null()
            {
                break;
            }
            frp = (*frp).fr_parent;
        }
        win_at_left = (*frp).fr_parent.is_null();
        let mut top_left: bool =
            !(win_at_top as ::core::ffi::c_int != 0 || win_at_left as ::core::ffi::c_int != 0);
        let mut top_right: bool =
            !(win_at_top as ::core::ffi::c_int != 0 || win_at_right as ::core::ffi::c_int != 0);
        let mut bot_left: bool =
            !(win_at_bottom as ::core::ffi::c_int != 0 || win_at_left as ::core::ffi::c_int != 0);
        let mut bot_right: bool =
            !(win_at_bottom as ::core::ffi::c_int != 0 || win_at_right as ::core::ffi::c_int != 0);
        if top_left {
            grid_line_start(
                default_gridview.ptr(),
                (*wp).w_winrow - 1 as ::core::ffi::c_int,
            );
            grid_line_put_schar(
                (*wp).w_wincol - 1 as ::core::ffi::c_int,
                get_corner_sep_connector(wp, WC_TOP_LEFT),
                hl,
            );
            grid_line_flush();
        }
        if top_right {
            grid_line_start(
                default_gridview.ptr(),
                (*wp).w_winrow - 1 as ::core::ffi::c_int,
            );
            grid_line_put_schar(
                (*wp).w_wincol + (*wp).w_width,
                get_corner_sep_connector(wp, WC_TOP_RIGHT),
                hl,
            );
            grid_line_flush();
        }
        if bot_left {
            grid_line_start(default_gridview.ptr(), (*wp).w_winrow + (*wp).w_height);
            grid_line_put_schar(
                (*wp).w_wincol - 1 as ::core::ffi::c_int,
                get_corner_sep_connector(wp, WC_BOTTOM_LEFT),
                hl,
            );
            grid_line_flush();
        }
        if bot_right {
            grid_line_start(default_gridview.ptr(), (*wp).w_winrow + (*wp).w_height);
            grid_line_put_schar(
                (*wp).w_wincol + (*wp).w_width,
                get_corner_sep_connector(wp, WC_BOTTOM_RIGHT),
                hl,
            );
            grid_line_flush();
        }
    }
}
