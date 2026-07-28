//! The match-time context (`rex`): the line the engines read, the
//! submatch bookkeeping, the Visual-area and back-reference tests.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn reg_breakcheck() {
    if !(*rex.ptr()).reg_nobreak {
        fast_breakcheck();
    }
}
pub(crate) unsafe extern "C" fn reg_iswordc(mut c: ::core::ffi::c_int) -> bool {
    return vim_iswordc_buf(c, (*rex.ptr()).reg_buf);
}
pub(crate) unsafe extern "C" fn reg_getline_common(
    mut lnum: linenr_T,
    mut flags: reg_getline_flags_T,
    mut line: *mut *mut ::core::ffi::c_char,
    mut length: *mut colnr_T,
) {
    let mut get_line: bool =
        flags as ::core::ffi::c_uint & RGLF_LINE as ::core::ffi::c_int as ::core::ffi::c_uint != 0;
    let mut get_length: bool = flags as ::core::ffi::c_uint
        & RGLF_LENGTH as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0;
    let mut firstlnum: linenr_T = 0;
    let mut maxline: linenr_T = 0;
    if flags as ::core::ffi::c_uint & RGLF_SUBMATCH as ::core::ffi::c_int as ::core::ffi::c_uint
        != 0
    {
        firstlnum = (*rsm.ptr()).sm_firstlnum + lnum;
        maxline = (*rsm.ptr()).sm_maxline;
    } else {
        firstlnum = (*rex.ptr()).reg_firstlnum + lnum;
        maxline = (*rex.ptr()).reg_maxline;
    }
    if firstlnum < 1 as linenr_T {
        if get_line {
            *line = ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if get_length {
            *length = 0 as ::core::ffi::c_int as colnr_T;
        }
        return;
    }
    if lnum > maxline {
        if get_line {
            *line = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        }
        if get_length {
            *length = 0 as ::core::ffi::c_int as colnr_T;
        }
        return;
    }
    if get_line {
        *line = ml_get_buf((*rex.ptr()).reg_buf, firstlnum);
    }
    if get_length {
        *length = ml_get_buf_len((*rex.ptr()).reg_buf, firstlnum);
    }
}
pub(crate) unsafe extern "C" fn reg_getline(mut lnum: linenr_T) -> *mut ::core::ffi::c_char {
    let mut line: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    reg_getline_common(
        lnum,
        RGLF_LINE,
        &raw mut line,
        ::core::ptr::null_mut::<colnr_T>(),
    );
    return line;
}
pub(crate) unsafe extern "C" fn reg_getline_len(mut lnum: linenr_T) -> colnr_T {
    let mut length: colnr_T = 0;
    reg_getline_common(
        lnum,
        RGLF_LENGTH,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        &raw mut length,
    );
    return length;
}
pub(crate) unsafe extern "C" fn make_extmatch() -> *mut reg_extmatch_T {
    let mut em: *mut reg_extmatch_T =
        xcalloc(1 as size_t, ::core::mem::size_of::<reg_extmatch_T>()) as *mut reg_extmatch_T;
    (*em).refcnt = 1 as int16_t;
    return em;
}
pub unsafe extern "C" fn ref_extmatch(mut em: *mut reg_extmatch_T) -> *mut reg_extmatch_T {
    if !em.is_null() {
        (*em).refcnt += 1;
    }
    return em;
}
pub unsafe extern "C" fn unref_extmatch(mut em: *mut reg_extmatch_T) {
    let mut i: ::core::ffi::c_int = 0;
    if !em.is_null() && {
        (*em).refcnt -= 1;
        (*em).refcnt as ::core::ffi::c_int <= 0 as ::core::ffi::c_int
    } {
        i = 0 as ::core::ffi::c_int;
        while i < NSUBEXP as ::core::ffi::c_int {
            xfree((*em).matches[i as usize] as *mut ::core::ffi::c_void);
            i += 1;
        }
        xfree(em as *mut ::core::ffi::c_void);
    }
}
pub(crate) unsafe extern "C" fn reg_prev_class() -> ::core::ffi::c_int {
    if (*rex.ptr()).input > (*rex.ptr()).line {
        return mb_get_class_tab(
            ((*rex.ptr()).input as *mut ::core::ffi::c_char)
                .offset(-(1 as ::core::ffi::c_int as isize))
                .offset(
                    -(utf_head_off(
                        (*rex.ptr()).line as *mut ::core::ffi::c_char,
                        ((*rex.ptr()).input as *mut ::core::ffi::c_char)
                            .offset(-(1 as ::core::ffi::c_int as isize)),
                    ) as isize),
                ),
            &raw mut (*(*rex.ptr()).reg_buf).b_chartab as *mut uint64_t,
        );
    }
    return -1 as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn reg_match_visual() -> bool {
    let mut top: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut bot: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut lnum: linenr_T = 0;
    let mut col: colnr_T = 0;
    let mut wp: *mut win_T = if (*rex.ptr()).reg_win.is_null() {
        curwin.get()
    } else {
        (*rex.ptr()).reg_win
    };
    let mut mode: ::core::ffi::c_int = 0;
    let mut start: colnr_T = 0;
    let mut end: colnr_T = 0;
    let mut start2: colnr_T = 0;
    let mut end2: colnr_T = 0;
    let mut curswant: colnr_T = 0;
    if (*rex.ptr()).reg_buf != curbuf.get()
        || (*VIsual.ptr()).lnum == 0 as linenr_T
        || !(*rex.ptr()).reg_match.is_null()
    {
        return false_0 != 0;
    }
    if VIsual_active.get() {
        if lt(VIsual.get(), (*wp).w_cursor) {
            top = VIsual.get();
            bot = (*wp).w_cursor;
        } else {
            top = (*wp).w_cursor;
            bot = VIsual.get();
        }
        mode = VIsual_mode.get();
        curswant = (*wp).w_curswant;
    } else {
        if lt(
            (*curbuf.get()).b_visual.vi_start,
            (*curbuf.get()).b_visual.vi_end,
        ) {
            top = (*curbuf.get()).b_visual.vi_start;
            bot = (*curbuf.get()).b_visual.vi_end;
        } else {
            top = (*curbuf.get()).b_visual.vi_end;
            bot = (*curbuf.get()).b_visual.vi_start;
        }
        if bot.lnum > (*curbuf.get()).b_ml.ml_line_count {
            bot.lnum = (*curbuf.get()).b_ml.ml_line_count;
        }
        mode = (*curbuf.get()).b_visual.vi_mode;
        curswant = (*curbuf.get()).b_visual.vi_curswant;
    }
    lnum = (*rex.ptr()).lnum + (*rex.ptr()).reg_firstlnum;
    if lnum < top.lnum || lnum > bot.lnum {
        return false_0 != 0;
    }
    col = (*rex.ptr()).input.offset_from((*rex.ptr()).line) as colnr_T;
    if mode == 'v' as ::core::ffi::c_int {
        if lnum == top.lnum && col < top.col
            || lnum == bot.lnum
                && col
                    >= bot.col as ::core::ffi::c_int
                        + (*p_sel.get() as ::core::ffi::c_int != 'e' as ::core::ffi::c_int)
                            as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
    } else if mode == Ctrl_V {
        getvvcol(
            wp,
            &raw mut top,
            &raw mut start,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut end,
        );
        getvvcol(
            wp,
            &raw mut bot,
            &raw mut start2,
            ::core::ptr::null_mut::<colnr_T>(),
            &raw mut end2,
        );
        if start2 < start {
            start = start2;
        }
        if end2 > end {
            end = end2;
        }
        if top.col == MAXCOL as ::core::ffi::c_int
            || bot.col == MAXCOL as ::core::ffi::c_int
            || curswant == MAXCOL as ::core::ffi::c_int
        {
            end = MAXCOL as ::core::ffi::c_int as colnr_T;
        }
        (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
        (*rex.ptr()).input = (*rex.ptr()).line.offset(col as isize);
        let mut cols: colnr_T = win_linetabsize(
            wp,
            (*rex.ptr()).reg_firstlnum + (*rex.ptr()).lnum,
            (*rex.ptr()).line as *mut ::core::ffi::c_char,
            col,
        );
        if cols < start
            || cols
                > end as ::core::ffi::c_int
                    - (*p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int)
                        as ::core::ffi::c_int
        {
            return false_0 != 0;
        }
    }
    return true_0 != 0;
}
pub(crate) unsafe extern "C" fn prog_magic_wrong() -> ::core::ffi::c_int {
    let mut prog: *mut regprog_T = ::core::ptr::null_mut::<regprog_T>();
    prog = if (*rex.ptr()).reg_match.is_null() {
        (*(*rex.ptr()).reg_mmatch).regprog
    } else {
        (*(*rex.ptr()).reg_match).regprog
    };
    if (*prog).engine == nfa_regengine.ptr() {
        return false_0;
    }
    if *(&raw mut (*(prog as *mut bt_regprog_T)).program as *mut uint8_t) as ::core::ffi::c_int
        != REGMAGIC
    {
        emsg(gettext(&raw const e_re_corr as *const ::core::ffi::c_char));
        return true_0;
    }
    return false_0;
}
pub(crate) unsafe extern "C" fn cleanup_subexpr() {
    if (*rex.ptr()).need_clear_subexpr == 0 {
        return;
    }
    if (*rex.ptr()).reg_match.is_null() {
        memset(
            (*rex.ptr()).reg_startpos as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<lpos_T>().wrapping_mul(NSUBEXP as ::core::ffi::c_int as size_t),
        );
        memset(
            (*rex.ptr()).reg_endpos as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<lpos_T>().wrapping_mul(NSUBEXP as ::core::ffi::c_int as size_t),
        );
    } else {
        memset(
            (*rex.ptr()).reg_startp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                .wrapping_mul(NSUBEXP as ::core::ffi::c_int as size_t),
        );
        memset(
            (*rex.ptr()).reg_endp as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                .wrapping_mul(NSUBEXP as ::core::ffi::c_int as size_t),
        );
    }
    (*rex.ptr()).need_clear_subexpr = false_0;
}
pub(crate) unsafe extern "C" fn cleanup_zsubexpr() {
    if (*rex.ptr()).need_clear_zsubexpr == 0 {
        return;
    }
    if (*rex.ptr()).reg_match.is_null() {
        memset(
            reg_startzpos.ptr() as *mut lpos_T as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<lpos_T>().wrapping_mul(NSUBEXP as ::core::ffi::c_int as size_t),
        );
        memset(
            reg_endzpos.ptr() as *mut lpos_T as *mut ::core::ffi::c_void,
            0xff as ::core::ffi::c_int,
            ::core::mem::size_of::<lpos_T>().wrapping_mul(NSUBEXP as ::core::ffi::c_int as size_t),
        );
    } else {
        memset(
            reg_startzp.ptr() as *mut *mut uint8_t as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                .wrapping_mul(NSUBEXP as ::core::ffi::c_int as size_t),
        );
        memset(
            reg_endzp.ptr() as *mut *mut uint8_t as *mut ::core::ffi::c_void,
            0 as ::core::ffi::c_int,
            ::core::mem::size_of::<*mut ::core::ffi::c_char>()
                .wrapping_mul(NSUBEXP as ::core::ffi::c_int as size_t),
        );
    }
    (*rex.ptr()).need_clear_zsubexpr = false_0;
}
pub(crate) unsafe extern "C" fn reg_nextline() {
    (*rex.ptr()).lnum += 1;
    (*rex.ptr()).line = reg_getline((*rex.ptr()).lnum) as *mut uint8_t;
    (*rex.ptr()).input = (*rex.ptr()).line;
    reg_breakcheck();
}
pub(crate) unsafe extern "C" fn match_with_backref(
    mut start_lnum: linenr_T,
    mut start_col: colnr_T,
    mut end_lnum: linenr_T,
    mut end_col: colnr_T,
    mut bytelen: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut clnum: linenr_T = start_lnum;
    let mut ccol: colnr_T = start_col;
    let mut len: ::core::ffi::c_int = 0;
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if !bytelen.is_null() {
        *bytelen = 0 as ::core::ffi::c_int;
    }
    loop {
        if (*rex.ptr()).line != reg_tofree.get() {
            len = strlen((*rex.ptr()).line as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
            if (*reg_tofree.ptr()).is_null() || len >= reg_tofreelen.get() as ::core::ffi::c_int {
                len += 50 as ::core::ffi::c_int;
                xfree(reg_tofree.get() as *mut ::core::ffi::c_void);
                reg_tofree.set(xmalloc(len as size_t) as *mut uint8_t);
                reg_tofreelen.set(len as ::core::ffi::c_uint);
            }
            strcpy(
                reg_tofree.get() as *mut ::core::ffi::c_char,
                (*rex.ptr()).line as *mut ::core::ffi::c_char,
            );
            (*rex.ptr()).input = (*reg_tofree.ptr())
                .offset((*rex.ptr()).input.offset_from((*rex.ptr()).line) as isize);
            (*rex.ptr()).line = reg_tofree.get();
        }
        p = reg_getline(clnum);
        assert!(!p.is_null(), "p");
        if clnum == end_lnum {
            len = (end_col - ccol) as ::core::ffi::c_int;
        } else {
            len = (reg_getline_len(clnum) - ccol) as ::core::ffi::c_int;
        }
        if !(*rex.ptr()).reg_ic
            && cstrncmp(
                p.offset(ccol as isize),
                (*rex.ptr()).input as *mut ::core::ffi::c_char,
                &raw mut len,
            ) != 0 as ::core::ffi::c_int
            || (*rex.ptr()).reg_ic as ::core::ffi::c_int != 0
                && mb_strnicmp(
                    p.offset(ccol as isize),
                    (*rex.ptr()).input as *mut ::core::ffi::c_char,
                    len as size_t,
                ) != 0 as ::core::ffi::c_int
        {
            return RA_NOMATCH;
        }
        if !bytelen.is_null() {
            *bytelen += len;
        }
        if clnum == end_lnum {
            break;
        }
        if (*rex.ptr()).lnum >= (*rex.ptr()).reg_maxline {
            return RA_NOMATCH;
        }
        reg_nextline();
        if !bytelen.is_null() {
            *bytelen = 0 as ::core::ffi::c_int;
        }
        clnum += 1;
        ccol = 0 as ::core::ffi::c_int as colnr_T;
        if got_int.get() {
            return RA_FAIL;
        }
    }
    return RA_MATCH;
}
pub(crate) unsafe extern "C" fn re_mult_next(mut what: *mut ::core::ffi::c_char) -> bool {
    if re_multi_type(peekchr()) == MULTI_MULT {
        semsg(
            gettext(b"E888: (NFA regexp) cannot repeat %s\0".as_ptr() as *const ::core::ffi::c_char),
            what,
        );
        rc_did_emsg.set(true_0 != 0);
        return false_0 != 0;
    }
    return true_0 != 0;
}
pub(crate) unsafe extern "C" fn init_regexec_multi(
    mut rmp: *mut regmmatch_T,
    mut win: *mut win_T,
    mut buf: *mut buf_T,
    mut lnum: linenr_T,
) {
    (*rex.ptr()).reg_match = ::core::ptr::null_mut::<regmatch_T>();
    (*rex.ptr()).reg_mmatch = rmp;
    (*rex.ptr()).reg_buf = buf;
    (*rex.ptr()).reg_win = win;
    (*rex.ptr()).reg_firstlnum = lnum;
    (*rex.ptr()).reg_maxline = (*(*rex.ptr()).reg_buf).b_ml.ml_line_count - lnum;
    (*rex.ptr()).reg_line_lbr = false_0 != 0;
    (*rex.ptr()).reg_ic = (*rmp).rmm_ic != 0;
    (*rex.ptr()).reg_icombine = false_0 != 0;
    (*rex.ptr()).reg_nobreak = (*(*rmp).regprog).re_flags & RE_NOBREAK as ::core::ffi::c_uint != 0;
    (*rex.ptr()).reg_maxcol = (*rmp).rmm_maxcol;
}
