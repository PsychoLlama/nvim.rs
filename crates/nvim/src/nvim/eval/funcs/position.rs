//! Positions in a buffer: the cursor, `line()`, `col()`,
//! `virtcol()`, `getpos()`/`setpos()` and the character-search state.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_byte2line(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut boff: ::core::ffi::c_int =
        tv_get_number(argvars.offset(0 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int
            - 1 as ::core::ffi::c_int;
    if boff < 0 as ::core::ffi::c_int {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number =
            ml_find_line_or_offset(curbuf.get(), 0 as linenr_T, &raw mut boff, false_0 != 0)
                as varnumber_T;
    };
}
unsafe extern "C" fn get_col(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut charcol: bool,
) {
    if tv_check_for_string_or_list_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
        || tv_check_for_opt_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
    {
        return;
    }
    let mut wp: *mut win_T = curwin.get();
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        wp = win_id2wp_tp(
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int,
            &raw mut tp,
        );
        if wp.is_null() || tp.is_null() {
            return;
        }
        check_cursor(wp);
    }
    let mut bp: *mut buf_T = (*wp).w_buffer;
    let mut col: colnr_T = 0 as colnr_T;
    let mut fnum: ::core::ffi::c_int = (*bp).handle as ::core::ffi::c_int;
    let mut fp: *mut pos_T = var2fpos(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        false_0 != 0,
        &raw mut fnum,
        charcol,
        wp,
    );
    if !fp.is_null() && fnum == (*bp).handle {
        if (*fp).col == MAXCOL as ::core::ffi::c_int {
            if (*fp).lnum <= (*bp).b_ml.ml_line_count {
                col = (ml_get_buf_len(bp, (*fp).lnum) + 1 as ::core::ffi::c_int) as colnr_T;
            } else {
                col = MAXCOL as ::core::ffi::c_int as colnr_T;
            }
        } else {
            col = ((*fp).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as colnr_T;
            if virtual_active(wp) as ::core::ffi::c_int != 0 && fp == &raw mut (*wp).w_cursor {
                let mut p: *mut ::core::ffi::c_char =
                    ml_get_buf(bp, (*wp).w_cursor.lnum).offset((*wp).w_cursor.col as isize);
                if (*wp).w_cursor.coladd
                    >= win_chartabsize(wp, p, (*wp).w_virtcol - (*wp).w_cursor.coladd)
                {
                    let mut l: ::core::ffi::c_int = 0;
                    if *p as ::core::ffi::c_int != NUL && {
                        l = utfc_ptr2len(p);
                        *p.offset(l as isize) as ::core::ffi::c_int == NUL
                    } {
                        col += l;
                    }
                }
            }
        }
    }
    (*rettv).vval.v_number = col as varnumber_T;
}
pub unsafe extern "C" fn f_charcol(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_col(argvars, rettv, true_0 != 0);
}
pub unsafe extern "C" fn f_col(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    get_col(argvars, rettv, false_0 != 0);
}
unsafe extern "C" fn set_cursorpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut charcol: bool,
) {
    let mut lnum: linenr_T = 0;
    let mut col: colnr_T = 0;
    let mut coladd: colnr_T = 0 as colnr_T;
    let mut set_curswant: bool = true_0 != 0;
    (*rettv).vval.v_number = -1 as varnumber_T;
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut pos: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut curswant: colnr_T = -1 as colnr_T;
        if list2fpos(
            argvars,
            &raw mut pos,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            &raw mut curswant,
            charcol,
        ) == FAIL
        {
            emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
            return;
        }
        lnum = pos.lnum;
        col = pos.col;
        coladd = pos.coladd;
        if curswant >= 0 as ::core::ffi::c_int {
            (*curwin.get()).w_curswant =
                (curswant as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
            set_curswant = false_0 != 0;
        }
    } else if ((*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        || (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
        && ((*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            || (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint)
    {
        lnum = tv_get_lnum(argvars);
        if lnum < 0 as linenr_T {
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            );
        } else if lnum == 0 as linenr_T {
            lnum = (*curwin.get()).w_cursor.lnum;
        }
        col = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        ) as colnr_T;
        if charcol {
            col = (buf_charidx_to_byteidx(curbuf.get(), lnum, col) + 1 as ::core::ffi::c_int)
                as colnr_T;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            coladd = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                ::core::ptr::null_mut::<bool>(),
            ) as colnr_T;
        }
    } else {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
        return;
    }
    if lnum < 0 as linenr_T || col < 0 as ::core::ffi::c_int || coladd < 0 as ::core::ffi::c_int {
        return;
    }
    if lnum > 0 as linenr_T {
        (*curwin.get()).w_cursor.lnum = lnum;
    }
    if col != MAXCOL as ::core::ffi::c_int && {
        col -= 1;
        col < 0 as ::core::ffi::c_int
    } {
        col = 0 as ::core::ffi::c_int as colnr_T;
    }
    (*curwin.get()).w_cursor.col = col;
    (*curwin.get()).w_cursor.coladd = coladd;
    check_cursor(curwin.get());
    mb_adjust_cursor();
    (*curwin.get()).w_set_curswant = set_curswant as ::core::ffi::c_int;
    (*rettv).vval.v_number = 0 as varnumber_T;
}
pub unsafe extern "C" fn f_cursor(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    set_cursorpos(argvars, rettv, false_0 != 0);
}
unsafe extern "C" fn getpos_both(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut getcurpos: bool,
    mut charcol: bool,
) {
    let mut fp: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut wp: *mut win_T = curwin.get();
    let mut fnum: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    if getcurpos {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            wp = find_win_by_nr_or_id(argvars.offset(0 as ::core::ffi::c_int as isize));
            if !wp.is_null() {
                fp = &raw mut (*wp).w_cursor;
            }
        } else {
            fp = &raw mut (*curwin.get()).w_cursor;
        }
        if !fp.is_null() && charcol as ::core::ffi::c_int != 0 {
            pos = *fp;
            pos.col =
                buf_byteidx_to_charidx((*wp).w_buffer, pos.lnum, pos.col as ::core::ffi::c_int)
                    as colnr_T;
            fp = &raw mut pos;
        }
    } else {
        fp = var2fpos(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            true_0 != 0,
            &raw mut fnum,
            charcol,
            curwin.get(),
        );
    }
    let l: *mut list_T = tv_list_alloc_ret(
        rettv,
        (4 as ::core::ffi::c_int + getcurpos as ::core::ffi::c_int) as ptrdiff_t,
    );
    tv_list_append_number(
        l,
        if fnum != -1 as ::core::ffi::c_int {
            fnum as varnumber_T
        } else {
            0 as ::core::ffi::c_int as varnumber_T
        },
    );
    tv_list_append_number(
        l,
        if !fp.is_null() {
            (*fp).lnum as varnumber_T
        } else {
            0 as ::core::ffi::c_int as varnumber_T
        },
    );
    tv_list_append_number(
        l,
        if !fp.is_null() {
            (if (*fp).col == MAXCOL as ::core::ffi::c_int {
                MAXCOL as ::core::ffi::c_int
            } else {
                (*fp).col as ::core::ffi::c_int + 1 as ::core::ffi::c_int
            }) as varnumber_T
        } else {
            0 as ::core::ffi::c_int as varnumber_T
        },
    );
    tv_list_append_number(
        l,
        if !fp.is_null() {
            (*fp).coladd as varnumber_T
        } else {
            0 as ::core::ffi::c_int as varnumber_T
        },
    );
    if getcurpos {
        let save_set_curswant: bool = (*curwin.get()).w_set_curswant != 0;
        let save_curswant: colnr_T = (*curwin.get()).w_curswant;
        let save_virtcol: colnr_T = (*curwin.get()).w_virtcol;
        if wp == curwin.get() {
            update_curswant();
        }
        tv_list_append_number(
            l,
            if wp.is_null() {
                0 as varnumber_T
            } else if (*wp).w_curswant == MAXCOL as ::core::ffi::c_int {
                MAXCOL as ::core::ffi::c_int as varnumber_T
            } else {
                (*wp).w_curswant as varnumber_T + 1 as varnumber_T
            },
        );
        if wp == curwin.get() && save_set_curswant as ::core::ffi::c_int != 0 {
            (*curwin.get()).w_set_curswant = save_set_curswant as ::core::ffi::c_int;
            (*curwin.get()).w_curswant = save_curswant;
            (*curwin.get()).w_virtcol = save_virtcol;
            (*curwin.get()).w_valid &= !VALID_VIRTCOL;
        }
    }
}
pub unsafe extern "C" fn f_getcharpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getpos_both(argvars, rettv, false_0 != 0, true_0 != 0);
}
pub unsafe extern "C" fn f_getcharsearch(
    mut _argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    tv_dict_alloc_ret(rettv);
    let mut dict: *mut dict_T = (*rettv).vval.v_dict;
    tv_dict_add_str(
        dict,
        b"char\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 5]>().wrapping_sub(1 as size_t),
        last_csearch(),
    );
    tv_dict_add_nr(
        dict,
        b"forward\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as size_t),
        last_csearch_forward() as varnumber_T,
    );
    tv_dict_add_nr(
        dict,
        b"until\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as size_t),
        last_csearch_until() as varnumber_T,
    );
}
pub unsafe extern "C" fn f_getcurpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getpos_both(argvars, rettv, true_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn f_getcursorcharpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getpos_both(argvars, rettv, true_0 != 0, true_0 != 0);
}
pub unsafe extern "C" fn f_getpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    getpos_both(argvars, rettv, false_0 != 0, false_0 != 0);
}
pub unsafe extern "C" fn f_line(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut lnum: linenr_T = 0 as linenr_T;
    let mut fp: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut fnum: ::core::ffi::c_int = 0;
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        let mut id: ::core::ffi::c_int =
            tv_get_number(argvars.offset(1 as ::core::ffi::c_int as isize)) as ::core::ffi::c_int;
        let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
        let mut wp: *mut win_T = win_id2wp_tp(id, &raw mut tp);
        if !wp.is_null() && !tp.is_null() {
            if *p_spk.get() as ::core::ffi::c_int != 'c' as ::core::ffi::c_int
                || (*wp).w_onebuf_opt.wo_diff != 0 && (*curwin.get()).w_onebuf_opt.wo_diff != 0
            {
                skip_update_topline.set(true_0 != 0);
            }
            check_cursor(wp);
            fp = var2fpos(
                argvars.offset(0 as ::core::ffi::c_int as isize),
                true_0 != 0,
                &raw mut fnum,
                false_0 != 0,
                wp,
            );
            skip_update_topline.set(false_0 != 0);
        }
    } else {
        fp = var2fpos(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            true_0 != 0,
            &raw mut fnum,
            false_0 != 0,
            curwin.get(),
        );
    }
    if !fp.is_null() {
        lnum = (*fp).lnum;
    }
    (*rettv).vval.v_number = lnum as varnumber_T;
}
pub unsafe extern "C" fn f_line2byte(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let lnum: linenr_T = tv_get_lnum(argvars);
    if lnum < 1 as linenr_T || lnum > (*curbuf.get()).b_ml.ml_line_count + 1 as linenr_T {
        (*rettv).vval.v_number = -1 as varnumber_T;
    } else {
        (*rettv).vval.v_number = ml_find_line_or_offset(
            curbuf.get(),
            lnum,
            ::core::ptr::null_mut::<::core::ffi::c_int>(),
            false_0 != 0,
        ) as varnumber_T;
    }
    if (*rettv).vval.v_number >= 0 as varnumber_T {
        (*rettv).vval.v_number += 1;
    }
}
unsafe extern "C" fn set_position(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut charpos: bool,
) {
    let mut curswant: colnr_T = -1 as colnr_T;
    (*rettv).vval.v_number = -1 as varnumber_T;
    let name: *const ::core::ffi::c_char = tv_get_string_chk(argvars);
    if name.is_null() {
        return;
    }
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut fnum: ::core::ffi::c_int = 0;
    if list2fpos(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        &raw mut pos,
        &raw mut fnum,
        &raw mut curswant,
        charpos,
    ) != OK
    {
        return;
    }
    if pos.col != MAXCOL as ::core::ffi::c_int && {
        pos.col -= 1;
        pos.col < 0 as ::core::ffi::c_int
    } {
        pos.col = 0 as ::core::ffi::c_int as colnr_T;
    }
    if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '.' as ::core::ffi::c_int
        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        (*curwin.get()).w_cursor = pos;
        if curswant >= 0 as ::core::ffi::c_int {
            (*curwin.get()).w_curswant =
                (curswant as ::core::ffi::c_int - 1 as ::core::ffi::c_int) as colnr_T;
            (*curwin.get()).w_set_curswant = false_0;
        }
        check_cursor(curwin.get());
        (*rettv).vval.v_number = 0 as varnumber_T;
    } else if *name.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
        == '\'' as ::core::ffi::c_int
        && *name.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        && *name.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
    {
        if setmark_pos(
            *name.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
            &raw mut pos,
            fnum,
            ::core::ptr::null_mut::<fmarkv_T>(),
        ) == OK
        {
            (*rettv).vval.v_number = 0 as varnumber_T;
        }
    } else {
        emsg(gettext(&raw const e_invarg as *const ::core::ffi::c_char));
    };
}
pub unsafe extern "C" fn f_setcharpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    set_position(argvars, rettv, true_0 != 0);
}
pub unsafe extern "C" fn f_setcharsearch(
    mut argvars: *mut typval_T,
    mut _rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if tv_check_for_dict_arg(argvars, 0 as ::core::ffi::c_int) == FAIL {
        return;
    }
    let mut d: *mut dict_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
        .vval
        .v_dict;
    if d.is_null() {
        return;
    }
    let csearch: *mut ::core::ffi::c_char = tv_dict_get_string(
        d,
        b"char\0".as_ptr() as *const ::core::ffi::c_char,
        false_0 != 0,
    );
    if !csearch.is_null() {
        let mut c: ::core::ffi::c_int = utf_ptr2char(csearch);
        set_last_csearch(c, csearch, utfc_ptr2len(csearch));
    }
    let mut di: *mut dictitem_T = tv_dict_find(
        d,
        b"forward\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        set_csearch_direction(
            (if tv_get_number(&raw mut (*di).di_tv) != 0 {
                FORWARD as ::core::ffi::c_int
            } else {
                BACKWARD as ::core::ffi::c_int
            }) as Direction,
        );
    }
    di = tv_dict_find(
        d,
        b"until\0".as_ptr() as *const ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 6]>().wrapping_sub(1 as usize) as ptrdiff_t,
    );
    if !di.is_null() {
        set_csearch_until((tv_get_number(&raw mut (*di).di_tv) != 0) as ::core::ffi::c_int);
    }
}
pub unsafe extern "C" fn f_setcursorcharpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    set_cursorpos(argvars, rettv, true_0 != 0);
}
pub unsafe extern "C" fn f_setpos(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    set_position(argvars, rettv, false_0 != 0);
}
pub unsafe extern "C" fn f_virtcol(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut bp: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
    let mut fnum: ::core::ffi::c_int = 0;
    let mut fp: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut vcol_start: colnr_T = 0 as colnr_T;
    let mut vcol_end: colnr_T = 0 as colnr_T;
    let mut wp: *mut win_T = curwin.get();
    '_theend: {
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut tp: *mut tabpage_T = ::core::ptr::null_mut::<tabpage_T>();
            wp = win_id2wp_tp(
                tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int,
                &raw mut tp,
            );
            if wp.is_null() || tp.is_null() {
                break '_theend;
            } else {
                check_cursor(wp);
            }
        }
        bp = (*wp).w_buffer;
        fnum = (*bp).handle as ::core::ffi::c_int;
        fp = var2fpos(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            false_0 != 0,
            &raw mut fnum,
            false_0 != 0,
            wp,
        );
        if !fp.is_null() && (*fp).lnum <= (*bp).b_ml.ml_line_count && fnum == (*bp).handle {
            if (*fp).col < 0 as ::core::ffi::c_int {
                (*fp).col = 0 as ::core::ffi::c_int as colnr_T;
            } else {
                let len: colnr_T = ml_get_buf_len(bp, (*fp).lnum);
                if (*fp).col > len {
                    (*fp).col = len;
                }
            }
            getvvcol(
                wp,
                fp,
                &raw mut vcol_start,
                ::core::ptr::null_mut::<colnr_T>(),
                &raw mut vcol_end,
            );
            vcol_start += 1;
            vcol_end += 1;
        }
    }
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        && tv_get_bool(argvars.offset(1 as ::core::ffi::c_int as isize)) != 0
    {
        tv_list_alloc_ret(rettv, 2 as ptrdiff_t);
        tv_list_append_number((*rettv).vval.v_list, vcol_start as varnumber_T);
        tv_list_append_number((*rettv).vval.v_list, vcol_end as varnumber_T);
    } else {
        (*rettv).vval.v_number = vcol_end as varnumber_T;
    };
}
