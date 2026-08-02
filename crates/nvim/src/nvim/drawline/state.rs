//! `winlinevars_T` — the state `win_line` passes to everything else.
//!
//! One screen line is drawn by `win_line`, but almost none of the work is done
//! there: the fold column, the sign column, the number column, `'statuscolumn'`,
//! `'breakindent'`, `'showbreak'` and the virtual texts are all separate
//! functions that take `(win_T *, winlinevars_T *)` and advance the same cursor
//! through the line buffer. This module owns that struct and the small
//! operations on it that are not tied to one column kind: starting a screen line
//! ([`win_line_start`]), undoing the fake columns concealment inserts
//! ([`fix_for_boguscols`]), walking the `'colorcolumn'` list
//! ([`advance_color_col`]) and answering how far right anything needs to be drawn
//! ([`get_rightmost_vcol`]).

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct winlinevars_T {
    pub lnum: linenr_T,
    pub foldinfo: foldinfo_T,
    pub startrow: ::core::ffi::c_int,
    pub row: ::core::ffi::c_int,
    pub vcol: colnr_T,
    pub col: ::core::ffi::c_int,
    pub boguscols: ::core::ffi::c_int,
    pub old_boguscols: ::core::ffi::c_int,
    pub vcol_off_co: ::core::ffi::c_int,
    pub off: ::core::ffi::c_int,
    pub cul_attr: ::core::ffi::c_int,
    pub line_attr: ::core::ffi::c_int,
    pub line_attr_lowprio: ::core::ffi::c_int,
    pub sign_num_attr: ::core::ffi::c_int,
    pub prev_num_attr: ::core::ffi::c_int,
    pub sign_cul_attr: ::core::ffi::c_int,
    pub fromcol: ::core::ffi::c_int,
    pub tocol: ::core::ffi::c_int,
    pub vcol_sbr: colnr_T,
    pub need_showbreak: bool,
    pub char_attr: ::core::ffi::c_int,
    pub n_extra: ::core::ffi::c_int,
    pub n_attr: ::core::ffi::c_int,
    pub p_extra: *mut ::core::ffi::c_char,
    pub extra_attr: ::core::ffi::c_int,
    pub sc_extra: schar_T,
    pub sc_final: schar_T,
    pub extra_for_extmark: bool,
    pub extra: [::core::ffi::c_char; 11],
    pub diff_hlf: hlf_T,
    pub n_virt_lines: ::core::ffi::c_int,
    pub n_virt_below: ::core::ffi::c_int,
    pub filler_lines: ::core::ffi::c_int,
    pub filler_todo: ::core::ffi::c_int,
    pub sattrs: [SignTextAttrs; 9],
    pub need_lbr: bool,
    pub virt_inline: VirtText,
    pub virt_inline_i: size_t,
    pub virt_inline_hl_mode: HlMode,
    pub reset_extra_attr: bool,
    pub skip_cells: ::core::ffi::c_int,
    pub skipped_cells: ::core::ffi::c_int,
    pub color_cols: *mut ::core::ffi::c_int,
}

pub(crate) static extra_buf: GlobalCell<*mut ::core::ffi::c_char> =
    GlobalCell::new(::core::ptr::null_mut::<::core::ffi::c_char>());

pub(crate) static extra_buf_size: GlobalCell<size_t> = GlobalCell::new(0 as size_t);

pub(crate) unsafe extern "C" fn get_extra_buf(mut size: size_t) -> *mut ::core::ffi::c_char {
    unsafe {
        size = if size > 64 as size_t {
            size
        } else {
            64 as size_t
        };
        if extra_buf_size.get() < size {
            xfree(extra_buf.get() as *mut ::core::ffi::c_void);
            extra_buf.set(xmalloc(size) as *mut ::core::ffi::c_char);
            extra_buf_size.set(size);
        }
        return extra_buf.get();
    }
}

pub(crate) unsafe extern "C" fn get_lcs_ext(mut wp: *mut win_T) -> schar_T {
    unsafe {
        if (*wp).w_onebuf_opt.wo_wrap != 0 {
            return NUL as schar_T;
        }
        if (*wp).w_onebuf_opt.wo_wrap_flags & kOptFlagInsecure as ::core::ffi::c_int as uint32_t
            != 0
        {
            return '>' as ::core::ffi::c_int as schar_T;
        }
        return if (*wp).w_onebuf_opt.wo_list != 0 {
            (*wp).w_p_lcs_chars.ext
        } else {
            NUL as schar_T
        };
    }
}

pub(crate) unsafe extern "C" fn advance_color_col(
    mut wlv: *mut winlinevars_T,
    mut vcol: ::core::ffi::c_int,
) {
    unsafe {
        if !(*wlv).color_cols.is_null() {
            while *(*wlv).color_cols >= 0 as ::core::ffi::c_int && vcol > *(*wlv).color_cols {
                (*wlv).color_cols = (*wlv).color_cols.offset(1);
            }
            if *(*wlv).color_cols < 0 as ::core::ffi::c_int {
                (*wlv).color_cols = ::core::ptr::null_mut::<::core::ffi::c_int>();
            }
        }
    }
}

pub(crate) unsafe extern "C" fn margin_columns_win(
    mut wp: *mut win_T,
    mut left_col: *mut ::core::ffi::c_int,
    mut right_col: *mut ::core::ffi::c_int,
) {
    unsafe {
        static saved_w_virtcol: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
        static prev_wp: GlobalCell<*mut win_T> = GlobalCell::new(::core::ptr::null_mut::<win_T>());
        static prev_width1: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
        static prev_width2: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
        static prev_left_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
        static prev_right_col: GlobalCell<::core::ffi::c_int> = GlobalCell::new(0);
        let mut cur_col_off: ::core::ffi::c_int = win_col_off(wp);
        let mut width1: ::core::ffi::c_int = (*wp).w_view_width - cur_col_off;
        let mut width2: ::core::ffi::c_int = width1 + win_col_off2(wp);
        if saved_w_virtcol.get() == (*wp).w_virtcol
            && prev_wp.get() == wp
            && prev_width1.get() == width1
            && prev_width2.get() == width2
        {
            *right_col = prev_right_col.get();
            *left_col = prev_left_col.get();
            return;
        }
        *left_col = 0 as ::core::ffi::c_int;
        *right_col = width1;
        if (*wp).w_virtcol >= width1 && width2 > 0 as ::core::ffi::c_int {
            *right_col = width1
                + (((*wp).w_virtcol as ::core::ffi::c_int - width1) / width2
                    + 1 as ::core::ffi::c_int)
                    * width2;
        }
        if (*wp).w_virtcol >= width1 && width2 > 0 as ::core::ffi::c_int {
            *left_col = ((*wp).w_virtcol as ::core::ffi::c_int - width1) / width2 * width2 + width1;
        }
        prev_left_col.set(*left_col);
        prev_right_col.set(*right_col);
        prev_wp.set(wp);
        prev_width1.set(width1);
        prev_width2.set(width2);
        saved_w_virtcol.set((*wp).w_virtcol as ::core::ffi::c_int);
    }
}

pub(crate) unsafe extern "C" fn apply_cursorline_highlight(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
) {
    unsafe {
        (*wlv).cul_attr = win_hl_attr(wp, HLF_CUL);
        let mut ae: HlAttrs = syn_attr2entry((*wlv).cul_attr);
        if ae.rgb_fg_color == -1 as RgbValue
            && ae.cterm_fg_color as ::core::ffi::c_int == 0 as ::core::ffi::c_int
        {
            (*wlv).line_attr_lowprio = (*wlv).cul_attr;
        } else if State.get() & MODE_INSERT == 0
            && bt_quickfix((*wp).w_buffer) as ::core::ffi::c_int != 0
            && qf_current_entry(wp) == (*wlv).lnum
        {
            (*wlv).line_attr = hl_combine_attr((*wlv).cul_attr, (*wlv).line_attr);
        } else {
            (*wlv).line_attr = (*wlv).cul_attr;
        };
    }
}

pub(crate) unsafe extern "C" fn set_line_attr_for_diff(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
) {
    unsafe {
        (*wlv).line_attr = win_hl_attr(wp, (*wlv).diff_hlf as ::core::ffi::c_int);
        if (*wlv).cul_attr != 0 {
            (*wlv).line_attr = if 0 as ::core::ffi::c_int != (*wlv).line_attr_lowprio {
                hl_combine_attr(
                    hl_combine_attr((*wlv).cul_attr, (*wlv).line_attr),
                    hl_get_underline(),
                )
            } else {
                hl_combine_attr((*wlv).line_attr, (*wlv).cul_attr)
            };
        }
    }
}

pub(crate) unsafe extern "C" fn win_line_start(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) {
    unsafe {
        (*wlv).col = 0 as ::core::ffi::c_int;
        (*wlv).off = 0 as ::core::ffi::c_int;
        (*wlv).need_lbr = false_0 != 0;
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < (*wp).w_view_width {
            *(*linebuf_char.ptr()).offset(i as isize) = ' ' as ::core::ffi::c_int as schar_T;
            *(*linebuf_attr.ptr()).offset(i as isize) = 0 as ::core::ffi::c_int as sattr_T;
            *(*linebuf_vcol.ptr()).offset(i as isize) = -1 as ::core::ffi::c_int as colnr_T;
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn fix_for_boguscols(mut wlv: *mut winlinevars_T) {
    unsafe {
        (*wlv).n_extra += (*wlv).vcol_off_co;
        (*wlv).vcol -= (*wlv).vcol_off_co;
        (*wlv).vcol_off_co = 0 as ::core::ffi::c_int;
        (*wlv).col -= (*wlv).boguscols;
        (*wlv).old_boguscols = (*wlv).boguscols;
        (*wlv).boguscols = 0 as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn get_rightmost_vcol(
    mut wp: *mut win_T,
    mut color_cols: *const ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ret: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*wp).w_onebuf_opt.wo_cuc != 0 {
            ret = (*wp).w_virtcol as ::core::ffi::c_int;
        }
        if !color_cols.is_null() {
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while *color_cols.offset(i as isize) >= 0 as ::core::ffi::c_int {
                ret = if ret > *color_cols.offset(i as isize) {
                    ret
                } else {
                    *color_cols.offset(i as isize)
                };
                i += 1;
            }
        }
        return ret;
    }
}
