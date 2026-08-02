//! Everything drawn to the left of the text.
//!
//! In window order: the fold column ([`draw_foldcolumn`], [`fill_foldcolumn`]),
//! the sign column ([`draw_sign`]), the number column ([`draw_lnum_col`]) — or,
//! when `'statuscolumn'` is set, one expression replacing all three
//! ([`draw_statuscol`]) — followed by the `'breakindent'` and `'showbreak'`
//! padding a wrapped line's continuation rows start with
//! ([`handle_breakindent`], [`handle_showbreak_and_filler`]).
//!
//! [`draw_col_buf`] and [`draw_col_fill`] are the two primitives all of them
//! emit through: one copies a string into the line buffer a character at a time,
//! the other repeats one character.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn draw_col_buf(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
    mut text: *const ::core::ffi::c_char,
    mut len: size_t,
    mut attr: ::core::ffi::c_int,
    mut fold_vcol: *const colnr_T,
    mut inc_vcol: bool,
) {
    unsafe {
        let mut ptr: *const ::core::ffi::c_char = text;
        while ptr < text.offset(len as isize) && (*wlv).off < (*wp).w_view_width {
            let mut cells: ::core::ffi::c_int = line_putchar(
                (*wp).w_buffer,
                &raw mut ptr,
                (*linebuf_char.ptr()).offset((*wlv).off as isize),
                (*wp).w_view_width - (*wlv).off,
                (*wlv).off,
            );
            let mut myattr: ::core::ffi::c_int = attr;
            if inc_vcol {
                advance_color_col(wlv, (*wlv).vcol as ::core::ffi::c_int);
                if !(*wlv).color_cols.is_null() && (*wlv).vcol == *(*wlv).color_cols {
                    myattr = hl_combine_attr(win_hl_attr(wp, HLF_MC), myattr);
                }
            }
            let mut c: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while c < cells {
                *(*linebuf_attr.ptr()).offset((*wlv).off as isize) = myattr as sattr_T;
                *(*linebuf_vcol.ptr()).offset((*wlv).off as isize) =
                    (if inc_vcol as ::core::ffi::c_int != 0 {
                        let c2rust_fresh6 = (*wlv).vcol;
                        (*wlv).vcol = (*wlv).vcol + 1;
                        c2rust_fresh6
                    } else if !fold_vcol.is_null() {
                        let c2rust_fresh7 = fold_vcol;
                        fold_vcol = fold_vcol.offset(1);
                        *c2rust_fresh7 as ::core::ffi::c_int
                    } else {
                        -1 as ::core::ffi::c_int
                    }) as colnr_T;
                (*wlv).off += 1;
                c += 1;
            }
        }
    }
}

pub(crate) unsafe extern "C" fn draw_col_fill(
    mut wlv: *mut winlinevars_T,
    mut fillchar: schar_T,
    mut width: ::core::ffi::c_int,
    mut attr: ::core::ffi::c_int,
) {
    unsafe {
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < width {
            *(*linebuf_char.ptr()).offset((*wlv).off as isize) = fillchar;
            *(*linebuf_attr.ptr()).offset((*wlv).off as isize) = attr as sattr_T;
            (*wlv).off += 1;
            i += 1;
        }
    }
}

pub unsafe extern "C" fn use_cursor_line_highlight(mut wp: *mut win_T, mut lnum: linenr_T) -> bool {
    unsafe {
        return (*wp).w_onebuf_opt.wo_cul != 0
            && lnum == (*wp).w_cursorline
            && (*wp).w_p_culopt_flags as ::core::ffi::c_int
                & kOptCuloptFlagNumber as ::core::ffi::c_int
                != 0;
    }
}

pub(crate) unsafe extern "C" fn draw_foldcolumn(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) {
    unsafe {
        let mut fdc: ::core::ffi::c_int = compute_foldcolumn(wp, 0 as ::core::ffi::c_int);
        if fdc > 0 as ::core::ffi::c_int {
            let mut attr: ::core::ffi::c_int = win_hl_attr(
                wp,
                if use_cursor_line_highlight(wp, (*wlv).lnum) as ::core::ffi::c_int != 0 {
                    HLF_CLF
                } else {
                    HLF_FC
                },
            );
            let mut is_virt: bool = (*wlv).filler_todo > 0 as ::core::ffi::c_int;
            fill_foldcolumn(
                wp,
                (*wlv).foldinfo,
                (*wlv).lnum,
                attr,
                fdc,
                is_virt,
                &raw mut (*wlv).off,
                ::core::ptr::null_mut::<colnr_T>(),
                ::core::ptr::null_mut::<schar_T>(),
            );
        }
    }
}

#[inline]
pub(crate) unsafe extern "C" fn foldcolumn_sep_char(
    mut first_level: ::core::ffi::c_int,
    mut i: ::core::ffi::c_int,
    mut wp: *mut win_T,
) -> schar_T {
    unsafe {
        if first_level == 1 as ::core::ffi::c_int {
            return (*wp).w_p_fcs_chars.foldsep;
        } else if (*wp).w_p_fcs_chars.foldinner != NUL as schar_T {
            return (*wp).w_p_fcs_chars.foldinner;
        } else if first_level + i <= 9 as ::core::ffi::c_int {
            return ('0' as ::core::ffi::c_int + first_level + i) as schar_T;
        } else {
            return '>' as ::core::ffi::c_int as schar_T;
        };
    }
}

pub unsafe extern "C" fn fill_foldcolumn(
    mut wp: *mut win_T,
    mut foldinfo: foldinfo_T,
    mut lnum: linenr_T,
    mut attr: ::core::ffi::c_int,
    mut fdc: ::core::ffi::c_int,
    mut is_virt: bool,
    mut wlv_off: *mut ::core::ffi::c_int,
    mut out_vcol: *mut colnr_T,
    mut out_buffer: *mut schar_T,
) {
    unsafe {
        let mut closed: bool =
            foldinfo.fi_level != 0 as ::core::ffi::c_int && foldinfo.fi_lines > 0 as linenr_T;
        let mut level: ::core::ffi::c_int = foldinfo.fi_level;
        let mut first_level: ::core::ffi::c_int = if level - fdc - closed as ::core::ffi::c_int
            + 1 as ::core::ffi::c_int
            > 1 as ::core::ffi::c_int
        {
            level - fdc - closed as ::core::ffi::c_int + 1 as ::core::ffi::c_int
        } else {
            1 as ::core::ffi::c_int
        };
        let mut closedcol: ::core::ffi::c_int = if fdc < level { fdc } else { level };
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while i < fdc {
            let mut symbol: schar_T = 0 as schar_T;
            if i >= level {
                symbol = ' ' as ::core::ffi::c_int as schar_T;
            } else if i == closedcol - 1 as ::core::ffi::c_int && closed as ::core::ffi::c_int != 0
            {
                symbol = (*wp).w_p_fcs_chars.foldclosed;
            } else if foldinfo.fi_lnum == lnum && first_level + i >= foldinfo.fi_low_level {
                symbol = (*wp).w_p_fcs_chars.foldopen;
            } else {
                symbol = foldcolumn_sep_char(first_level, i, wp);
            }
            if is_virt as ::core::ffi::c_int != 0
                && foldinfo.fi_level != 0 as ::core::ffi::c_int
                && foldinfo.fi_lnum == lnum
            {
                let mut outer_level: ::core::ffi::c_int =
                    if foldinfo.fi_low_level - 1 as ::core::ffi::c_int > 0 as ::core::ffi::c_int {
                        foldinfo.fi_low_level - 1 as ::core::ffi::c_int
                    } else {
                        0 as ::core::ffi::c_int
                    };
                let mut outer_first_level: ::core::ffi::c_int =
                    if outer_level - fdc + 1 as ::core::ffi::c_int > 1 as ::core::ffi::c_int {
                        outer_level - fdc + 1 as ::core::ffi::c_int
                    } else {
                        1 as ::core::ffi::c_int
                    };
                if i >= outer_level {
                    symbol = ' ' as ::core::ffi::c_int as schar_T;
                } else {
                    symbol = foldcolumn_sep_char(outer_first_level, i, wp);
                }
            }
            let mut vcol: ::core::ffi::c_int = if i >= level {
                -1 as ::core::ffi::c_int
            } else if i == closedcol - 1 as ::core::ffi::c_int && closed as ::core::ffi::c_int != 0
            {
                -2 as ::core::ffi::c_int
            } else {
                -3 as ::core::ffi::c_int
            };
            if !out_buffer.is_null() {
                *out_vcol.offset(i as isize) = vcol as colnr_T;
                *out_buffer.offset(i as isize) = symbol;
            } else {
                *(*linebuf_vcol.ptr()).offset(*wlv_off as isize) = vcol as colnr_T;
                *(*linebuf_attr.ptr()).offset(*wlv_off as isize) = attr as sattr_T;
                let c2rust_fresh0 = *wlv_off;
                *wlv_off = *wlv_off + 1;
                *(*linebuf_char.ptr()).offset(c2rust_fresh0 as isize) = symbol;
            }
            i += 1;
        }
    }
}

pub(crate) unsafe extern "C" fn draw_sign(
    mut nrcol: bool,
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
    mut sign_idx: ::core::ffi::c_int,
) {
    unsafe {
        let mut sattr: SignTextAttrs = (*wlv).sattrs[sign_idx as usize];
        let mut scl_attr: ::core::ffi::c_int = win_hl_attr(
            wp,
            if use_cursor_line_highlight(wp, (*wlv).lnum) as ::core::ffi::c_int != 0 {
                HLF_CLS
            } else {
                HLF_SC
            },
        );
        if sattr.text[0 as ::core::ffi::c_int as usize] != 0
            && (*wlv).row == (*wlv).startrow + (*wlv).filler_lines
            && (*wlv).filler_todo <= 0 as ::core::ffi::c_int
        {
            let mut fill: ::core::ffi::c_int = if nrcol as ::core::ffi::c_int != 0 {
                number_width(wp) + 1 as ::core::ffi::c_int
            } else {
                SIGN_WIDTH as ::core::ffi::c_int
            };
            let mut attr: ::core::ffi::c_int = if (*wlv).sign_cul_attr != 0 {
                (*wlv).sign_cul_attr
            } else if sattr.hl_id != 0 {
                syn_id2attr(sattr.hl_id)
            } else {
                0 as ::core::ffi::c_int
            };
            attr = hl_combine_attr(scl_attr, attr);
            draw_col_fill(wlv, ' ' as ::core::ffi::c_int as schar_T, fill, attr);
            let mut sign_pos: ::core::ffi::c_int =
                (*wlv).off - SIGN_WIDTH as ::core::ffi::c_int - nrcol as ::core::ffi::c_int;
            '_c2rust_label: {
                if sign_pos >= 0 as ::core::ffi::c_int {
                } else {
                    __assert_fail(
                        b"sign_pos >= 0\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        580 as ::core::ffi::c_uint,
                        b"void draw_sign(_Bool, win_T *, winlinevars_T *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            *(*linebuf_char.ptr()).offset(sign_pos as isize) =
                sattr.text[0 as ::core::ffi::c_int as usize];
            *(*linebuf_char.ptr()).offset((sign_pos + 1 as ::core::ffi::c_int) as isize) =
                sattr.text[1 as ::core::ffi::c_int as usize];
        } else {
            '_c2rust_label_0: {
                if !nrcol {
                } else {
                    __assert_fail(
                        b"!nrcol\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/drawline.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        584 as ::core::ffi::c_uint,
                        b"void draw_sign(_Bool, win_T *, winlinevars_T *, int)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            draw_col_fill(
                wlv,
                ' ' as ::core::ffi::c_int as schar_T,
                SIGN_WIDTH as ::core::ffi::c_int,
                scl_attr,
            );
        };
    }
}

#[inline]
pub(crate) unsafe extern "C" fn get_line_number_str(
    mut wp: *mut win_T,
    mut lnum: linenr_T,
    mut buf: *mut ::core::ffi::c_char,
    mut buf_len: size_t,
) {
    unsafe {
        let mut num: linenr_T = 0;
        let mut fmt: *mut ::core::ffi::c_char =
            b"%*d \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        if (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu == 0 {
            num = lnum;
        } else {
            num = abs(get_cursor_rel_lnum(wp, lnum) as ::core::ffi::c_int) as linenr_T;
            if num == 0 as linenr_T
                && (*wp).w_onebuf_opt.wo_nu != 0
                && (*wp).w_onebuf_opt.wo_rnu != 0
            {
                num = lnum;
                fmt = b"%-*d \0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            }
        }
        snprintf(buf, buf_len, fmt, number_width(wp), num);
    }
}

pub(crate) unsafe extern "C" fn use_cursor_line_nr(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
) -> bool {
    unsafe {
        return (*wp).w_onebuf_opt.wo_cul != 0
            && (*wlv).lnum == (*wp).w_cursorline
            && (*wp).w_p_culopt_flags as ::core::ffi::c_int
                & kOptCuloptFlagNumber as ::core::ffi::c_int
                != 0
            && ((*wlv).row == (*wlv).startrow + (*wlv).filler_lines
                || (*wlv).row > (*wlv).startrow + (*wlv).filler_lines
                    && (*wp).w_p_culopt_flags as ::core::ffi::c_int
                        & kOptCuloptFlagLine as ::core::ffi::c_int
                        != 0);
    }
}

pub(crate) unsafe extern "C" fn get_line_number_attr(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut numhl_attr: ::core::ffi::c_int = (*wlv).sign_num_attr;
        if (*wlv).n_virt_lines - (*wlv).filler_todo < (*wlv).n_virt_below {
            if (*wlv).prev_num_attr == -1 as ::core::ffi::c_int {
                decor_redraw_signs(
                    wp,
                    (*wp).w_buffer,
                    (*wlv).lnum as ::core::ffi::c_int - 2 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<SignTextAttrs>(),
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    ::core::ptr::null_mut::<::core::ffi::c_int>(),
                    &raw mut (*wlv).prev_num_attr,
                );
                if (*wlv).prev_num_attr > 0 as ::core::ffi::c_int {
                    (*wlv).prev_num_attr = syn_id2attr((*wlv).prev_num_attr);
                }
            }
            numhl_attr = (*wlv).prev_num_attr;
        }
        if use_cursor_line_nr(wp, wlv) {
            return hl_combine_attr(win_hl_attr(wp, HLF_CLN), numhl_attr);
        }
        if (*wp).w_onebuf_opt.wo_rnu != 0 {
            if (*wlv).lnum < (*wp).w_cursor.lnum {
                return hl_combine_attr(win_hl_attr(wp, HLF_LNA), numhl_attr);
            }
            if (*wlv).lnum > (*wp).w_cursor.lnum {
                return hl_combine_attr(win_hl_attr(wp, HLF_LNB), numhl_attr);
            }
        }
        return hl_combine_attr(win_hl_attr(wp, HLF_N), numhl_attr);
    }
}

pub(crate) unsafe extern "C" fn draw_lnum_col(mut wp: *mut win_T, mut wlv: *mut winlinevars_T) {
    unsafe {
        let mut has_cpo_n: bool = !vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null();
        if ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
            && ((*wlv).row == (*wlv).startrow + (*wlv).filler_lines || !has_cpo_n)
            && !(has_cpo_n as ::core::ffi::c_int != 0
                && (*wp).w_onebuf_opt.wo_bri == 0
                && (*wp).w_skipcol > 0 as ::core::ffi::c_int
                && (*wlv).lnum == (*wp).w_topline)
        {
            if (*wp).w_minscwidth == SCL_NUM
                && (*wlv).sattrs[0 as ::core::ffi::c_int as usize].text
                    [0 as ::core::ffi::c_int as usize]
                    != 0
                && (*wlv).row == (*wlv).startrow + (*wlv).filler_lines
                && (*wlv).filler_todo <= 0 as ::core::ffi::c_int
            {
                draw_sign(true_0 != 0, wp, wlv, 0 as ::core::ffi::c_int);
            } else {
                let mut width: ::core::ffi::c_int = number_width(wp) + 1 as ::core::ffi::c_int;
                let mut attr: ::core::ffi::c_int = get_line_number_attr(wp, wlv);
                if (*wlv).row == (*wlv).startrow + (*wlv).filler_lines
                    && ((*wp).w_skipcol == 0 as ::core::ffi::c_int
                        || (*wlv).row > 0 as ::core::ffi::c_int
                        || (*wp).w_onebuf_opt.wo_nu != 0 && (*wp).w_onebuf_opt.wo_rnu != 0)
                {
                    let mut buf: [::core::ffi::c_char; 32] = [0; 32];
                    get_line_number_str(
                        wp,
                        (*wlv).lnum,
                        &raw mut buf as *mut ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
                    );
                    if (*wp).w_skipcol > 0 as ::core::ffi::c_int
                        && (*wlv).startrow == 0 as ::core::ffi::c_int
                    {
                        let mut c: *mut ::core::ffi::c_char =
                            &raw mut buf as *mut ::core::ffi::c_char;
                        while *c as ::core::ffi::c_int == ' ' as ::core::ffi::c_int {
                            *c = '-' as ::core::ffi::c_char;
                            c = c.offset(1);
                        }
                    }
                    if (*wp).w_onebuf_opt.wo_rl != 0 {
                        let mut num: *mut ::core::ffi::c_char =
                            skipwhite(&raw mut buf as *mut ::core::ffi::c_char);
                        rl_mirror_ascii(num, skiptowhite(num));
                    }
                    draw_col_buf(
                        wp,
                        wlv,
                        &raw mut buf as *mut ::core::ffi::c_char,
                        width as size_t,
                        attr,
                        ::core::ptr::null::<colnr_T>(),
                        false_0 != 0,
                    );
                } else {
                    draw_col_fill(wlv, ' ' as ::core::ffi::c_int as schar_T, width, attr);
                }
            }
        }
    }
}

pub(crate) unsafe extern "C" fn draw_statuscol(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
    mut virtnum: ::core::ffi::c_int,
    mut col_rows: ::core::ffi::c_int,
    mut stcp: *mut statuscol_T,
) {
    unsafe {
        let mut lnum: linenr_T = (*wlv).lnum
            - ((*wlv).n_virt_lines - (*wlv).filler_todo < (*wlv).n_virt_below)
                as ::core::ffi::c_int;
        let mut relnum: linenr_T = if virtnum == -(*wlv).filler_lines
            || virtnum == 0 as ::core::ffi::c_int
            || virtnum == (*wlv).n_virt_below - (*wlv).filler_lines
        {
            abs(get_cursor_rel_lnum(wp, lnum) as ::core::ffi::c_int) as linenr_T
        } else {
            -1 as linenr_T
        };
        let mut buf: [::core::ffi::c_char; 4096] = [0; 4096];
        if (*wp).w_statuscol_line_count != (*wp).w_nrwidth_line_count {
            (*wp).w_statuscol_line_count = (*wp).w_nrwidth_line_count;
            set_vim_var_nr(VV_VIRTNUM, 0 as varnumber_T);
            let mut width: ::core::ffi::c_int = build_statuscol_str(
                wp,
                (*wp).w_nrwidth_line_count,
                (*wp).w_nrwidth_line_count,
                &raw mut buf as *mut ::core::ffi::c_char,
                stcp,
            );
            if width > (*stcp).width {
                let mut addwidth: ::core::ffi::c_int = if width - (*stcp).width
                    < 20 as ::core::ffi::c_int
                        + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                        + 9 as ::core::ffi::c_int
                        - (*stcp).width
                {
                    width - (*stcp).width
                } else {
                    20 as ::core::ffi::c_int
                        + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                        + 9 as ::core::ffi::c_int
                        - (*stcp).width
                };
                (*wp).w_nrwidth += addwidth;
                (*wp).w_nrwidth_width = (*wp).w_nrwidth;
                if col_rows > 0 as ::core::ffi::c_int {
                    (*wp).w_redr_statuscol = true_0 != 0;
                    return;
                }
                (*stcp).width += addwidth;
                (*wp).w_valid &= !VALID_WCOL;
            }
        }
        set_vim_var_nr(VV_VIRTNUM, virtnum as varnumber_T);
        let mut width_0: ::core::ffi::c_int = build_statuscol_str(
            wp,
            lnum,
            relnum,
            &raw mut buf as *mut ::core::ffi::c_char,
            stcp,
        );
        if *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int == NUL
            || width_0 > (*stcp).width
                && (*stcp).width
                    < MAX_NUMBERWIDTH
                        + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                        + 9 as ::core::ffi::c_int
        {
            if *(*wp).w_onebuf_opt.wo_stc as ::core::ffi::c_int == NUL {
                (*wp).w_nrwidth_line_count = 0 as ::core::ffi::c_int as linenr_T;
                (*wp).w_nrwidth = ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                    as ::core::ffi::c_int
                    * number_width(wp);
            } else {
                (*wp).w_nrwidth += if width_0 - (*stcp).width
                    < 20 as ::core::ffi::c_int
                        + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                        + 9 as ::core::ffi::c_int
                        - (*stcp).width
                {
                    width_0 - (*stcp).width
                } else {
                    20 as ::core::ffi::c_int
                        + SIGN_SHOW_MAX as ::core::ffi::c_int * SIGN_WIDTH as ::core::ffi::c_int
                        + 9 as ::core::ffi::c_int
                        - (*stcp).width
                };
                (*wp).w_nrwidth_width = (*wp).w_nrwidth;
            }
            (*wp).w_redr_statuscol = true_0 != 0;
            return;
        }
        let mut p: *mut ::core::ffi::c_char = &raw mut buf as *mut ::core::ffi::c_char;
        let mut transbuf: [::core::ffi::c_char; 4096] = [0; 4096];
        let mut fold_vcol: *mut colnr_T = ::core::ptr::null_mut::<colnr_T>();
        let mut len: size_t = strlen(&raw mut buf as *mut ::core::ffi::c_char);
        let mut scl_attr: ::core::ffi::c_int = win_hl_attr(
            wp,
            if use_cursor_line_highlight(wp, (*wlv).lnum) as ::core::ffi::c_int != 0 {
                HLF_CLS
            } else {
                HLF_SC
            },
        );
        let mut num_attr: ::core::ffi::c_int = get_line_number_attr(wp, wlv);
        let mut cur_attr: ::core::ffi::c_int = num_attr;
        let mut sp: *mut stl_hlrec_t = (*stcp).hlrec;
        while !(*sp).start.is_null() {
            let mut textlen: ptrdiff_t = (*sp).start.offset_from(p);
            let mut translen: size_t = transstr_buf(
                p,
                textlen as ssize_t,
                &raw mut transbuf as *mut ::core::ffi::c_char,
                MAXPATHL as size_t,
                true_0 != 0,
            );
            draw_col_buf(
                wp,
                wlv,
                &raw mut transbuf as *mut ::core::ffi::c_char,
                translen,
                cur_attr,
                fold_vcol,
                false_0 != 0,
            );
            let mut attr: ::core::ffi::c_int = if (*sp).item as ::core::ffi::c_uint
                == STL_SIGNCOL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                scl_attr
            } else if (*sp).item as ::core::ffi::c_uint
                == STL_FOLDCOL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                0 as ::core::ffi::c_int
            } else {
                num_attr
            };
            cur_attr = hl_combine_attr(
                attr,
                if (*sp).userhl < 0 as ::core::ffi::c_int {
                    syn_id2attr(-(*sp).userhl)
                } else {
                    0 as ::core::ffi::c_int
                },
            );
            fold_vcol = if (*sp).item as ::core::ffi::c_uint
                == STL_FOLDCOL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                &raw mut (*stcp).fold_vcol as *mut colnr_T
            } else {
                ::core::ptr::null_mut::<colnr_T>()
            };
            p = (*sp).start;
            sp = sp.offset(1);
        }
        let mut translen_0: size_t = transstr_buf(
            p,
            (&raw mut buf as *mut ::core::ffi::c_char)
                .offset(len as isize)
                .offset_from(p) as ssize_t,
            &raw mut transbuf as *mut ::core::ffi::c_char,
            MAXPATHL as size_t,
            true_0 != 0,
        );
        draw_col_buf(
            wp,
            wlv,
            &raw mut transbuf as *mut ::core::ffi::c_char,
            translen_0,
            cur_attr,
            fold_vcol,
            false_0 != 0,
        );
        draw_col_fill(
            wlv,
            ' ' as ::core::ffi::c_int as schar_T,
            (*stcp).width - width_0,
            cur_attr,
        );
    }
}

pub(crate) unsafe extern "C" fn handle_breakindent(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
) {
    unsafe {
        if (*wp).w_onebuf_opt.wo_bri != 0
            && ((*wlv).row > (*wlv).startrow + (*wlv).filler_lines
                || (*wlv).need_showbreak as ::core::ffi::c_int != 0)
        {
            let mut attr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            if (*wlv).diff_hlf as ::core::ffi::c_uint != HLF_NONE as ::core::ffi::c_uint {
                attr = win_hl_attr(wp, (*wlv).diff_hlf as ::core::ffi::c_int);
            }
            let mut num: ::core::ffi::c_int =
                get_breakindent_win(wp, ml_get_buf((*wp).w_buffer, (*wlv).lnum));
            if (*wlv).row == (*wlv).startrow {
                num -= win_col_off2(wp);
                if (*wlv).n_extra < 0 as ::core::ffi::c_int {
                    num = 0 as ::core::ffi::c_int;
                }
            }
            let mut vcol_before: colnr_T = (*wlv).vcol;
            let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while i < num {
                *(*linebuf_char.ptr()).offset((*wlv).off as isize) =
                    ' ' as ::core::ffi::c_int as schar_T;
                advance_color_col(wlv, (*wlv).vcol as ::core::ffi::c_int);
                let mut myattr: ::core::ffi::c_int = attr;
                if !(*wlv).color_cols.is_null() && (*wlv).vcol == *(*wlv).color_cols {
                    myattr = hl_combine_attr(win_hl_attr(wp, HLF_MC), myattr);
                }
                *(*linebuf_attr.ptr()).offset((*wlv).off as isize) = myattr as sattr_T;
                let c2rust_fresh5 = (*wlv).vcol;
                (*wlv).vcol = (*wlv).vcol + 1;
                *(*linebuf_vcol.ptr()).offset((*wlv).off as isize) = c2rust_fresh5;
                (*wlv).off += 1;
                i += 1;
            }
            if (*wlv).fromcol >= vcol_before && (*wlv).fromcol < (*wlv).vcol {
                (*wlv).fromcol = (*wlv).vcol as ::core::ffi::c_int;
            }
            if (*wlv).tocol == vcol_before {
                (*wlv).tocol = (*wlv).vcol as ::core::ffi::c_int;
            }
        }
        if (*wp).w_skipcol > 0 as ::core::ffi::c_int
            && (*wlv).startrow == 0 as ::core::ffi::c_int
            && (*wp).w_onebuf_opt.wo_wrap != 0
            && (*wp).w_briopt_sbr as ::core::ffi::c_int != 0
        {
            (*wlv).need_showbreak = false_0 != 0;
        }
    }
}

pub(crate) unsafe extern "C" fn handle_showbreak_and_filler(
    mut wp: *mut win_T,
    mut wlv: *mut winlinevars_T,
) {
    unsafe {
        let mut remaining: ::core::ffi::c_int = (*wp).w_view_width - (*wlv).off;
        if (*wlv).filler_todo > (*wlv).filler_lines - (*wlv).n_virt_lines {
            draw_col_fill(
                wlv,
                ' ' as ::core::ffi::c_int as schar_T,
                remaining,
                0 as ::core::ffi::c_int,
            );
        } else if (*wlv).filler_todo > 0 as ::core::ffi::c_int {
            let mut c: schar_T = (*wp).w_p_fcs_chars.diff;
            draw_col_fill(wlv, c, remaining, win_hl_attr(wp, HLF_DED));
        }
        let sbr: *mut ::core::ffi::c_char = get_showbreak_value(wp);
        if *sbr as ::core::ffi::c_int != NUL && (*wlv).need_showbreak as ::core::ffi::c_int != 0 {
            let mut attr: ::core::ffi::c_int =
                hl_combine_attr((*wlv).cul_attr, win_hl_attr(wp, HLF_AT));
            let mut vcol_before: colnr_T = (*wlv).vcol;
            draw_col_buf(
                wp,
                wlv,
                sbr,
                strlen(sbr),
                attr,
                ::core::ptr::null::<colnr_T>(),
                true_0 != 0,
            );
            (*wlv).vcol_sbr = (*wlv).vcol;
            if (*wlv).fromcol >= vcol_before && (*wlv).fromcol < (*wlv).vcol {
                (*wlv).fromcol = (*wlv).vcol as ::core::ffi::c_int;
            }
            if (*wlv).tocol == vcol_before {
                (*wlv).tocol = (*wlv).vcol as ::core::ffi::c_int;
            }
        }
        if (*wp).w_skipcol == 0 as ::core::ffi::c_int
            || (*wlv).startrow > 0 as ::core::ffi::c_int
            || (*wp).w_onebuf_opt.wo_wrap == 0
            || !(*wp).w_briopt_sbr
        {
            (*wlv).need_showbreak = false_0 != 0;
        }
    }
}
