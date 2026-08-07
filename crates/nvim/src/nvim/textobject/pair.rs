//! The objects delimited by a matched pair: `i(`/`a{`/... and `it`/`at`.
//!
//! Both halves answer "where does the region enclosing the cursor start and
//! end", and differ only in how the pair is found: [`current_block`] hands
//! the bracket to `findmatch`, [`current_tagblock`] hands a generated
//! start/end pattern to `do_searchpair`.  The retry loops in both are
//! Visual-mode extension -- an object already selected whole grows outwards.

use super::*;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::cursor::{
    dec_cursor, gchar_cursor, get_cursor_line_ptr, get_cursor_pos_ptr, inc_cursor,
};
use crate::src::nvim::drawscreen::{UPD_INVERTED, redraw_curbuf_later, showmode};
use crate::src::nvim::eval::funcs::do_searchpair;
use crate::src::nvim::indent::inindent;
use crate::src::nvim::main::{VIsual, VIsual_active, VIsual_mode, curwin, p_cpo, p_sel, p_ws};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::mbyte::{utf_head_off, utfc_ptr2len};
use crate::src::nvim::memline::{decl, inc, incl, ml_get_pos};
use crate::src::nvim::memory::{xfree, xmalloc};
use crate::src::nvim::os::libc::snprintf;
use crate::src::nvim::pos::{equalpos, lt, ltoreq};
use crate::src::nvim::search::{BACKWARD, FORWARD, findmatch, findmatchlimit};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{
    colnr_T, int64_t, linenr_T, oparg_T, pos_T, size_t, typval_T, uint8_t,
};

pub unsafe extern "C" fn current_block(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut what: ::core::ffi::c_int,
    mut other: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut start_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut end_pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
    let mut sol: bool = false;
    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
    let mut old_end: pos_T = (*curwin.get()).w_cursor;
    let mut old_start: pos_T = old_end;
    if !VIsual_active.get()
        || equalpos(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
    {
        setpcmark();
        if what == '{' as ::core::ffi::c_int {
            while inindent(1 as ::core::ffi::c_int) {
                if inc_cursor() != 0 as ::core::ffi::c_int {
                    break;
                }
            }
        }
        if gchar_cursor() == what {
            (*curwin.get()).w_cursor.col += 1;
        }
    } else if lt(VIsual.get(), (*curwin.get()).w_cursor) {
        old_start = VIsual.get();
        (*curwin.get()).w_cursor = VIsual.get();
    } else {
        old_end = VIsual.get();
    }
    let mut save_cpo: *mut ::core::ffi::c_char = p_cpo.get();
    p_cpo.set(
        (if !vim_strchr(p_cpo.get(), CPO_MATCHBSL).is_null() {
            c"%M".as_ptr()
        } else {
            c"%".as_ptr()
        }) as *mut ::core::ffi::c_char,
    );
    pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
    if !pos.is_null() {
        loop {
            let c2rust_fresh4 = count;
            count = count - 1;
            if c2rust_fresh4 <= 0 as ::core::ffi::c_int {
                break;
            }
            pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
            if pos.is_null() {
                break;
            }
            (*curwin.get()).w_cursor = *pos;
            start_pos = *pos;
        }
    } else {
        loop {
            let c2rust_fresh5 = count;
            count = count - 1;
            if c2rust_fresh5 <= 0 as ::core::ffi::c_int {
                break;
            }
            pos = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                what,
                FM_FORWARD as ::core::ffi::c_int,
                0 as int64_t,
            );
            if pos.is_null() {
                break;
            }
            (*curwin.get()).w_cursor = *pos;
            start_pos = *pos;
        }
    }
    p_cpo.set(save_cpo);
    if pos.is_null() || {
        end_pos = findmatch(::core::ptr::null_mut::<oparg_T>(), other);
        end_pos.is_null()
    } {
        (*curwin.get()).w_cursor = old_pos;
        return FAIL;
    }
    (*curwin.get()).w_cursor = *end_pos;
    // Upstream's `if (!include)` retry loop; exits via break or return.
    #[allow(clippy::while_immutable_condition)]
    while !include {
        incl(&raw mut start_pos);
        sol = (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int;
        decl(&raw mut (*curwin.get()).w_cursor);
        while inindent(1 as ::core::ffi::c_int) {
            sol = true;
            if decl(&raw mut (*curwin.get()).w_cursor) != 0 as ::core::ffi::c_int {
                break;
            }
        }
        if equalpos(start_pos, *end_pos) as ::core::ffi::c_int != 0
            && VIsual_active.get() as ::core::ffi::c_int != 0
        {
            (*curwin.get()).w_cursor = old_pos;
            return FAIL;
        }
        if !(!lt(start_pos, old_start)
            && !lt(old_end, (*curwin.get()).w_cursor)
            && !equalpos(start_pos, (*curwin.get()).w_cursor)
            && VIsual_active.get() as ::core::ffi::c_int != 0)
        {
            break;
        }
        (*curwin.get()).w_cursor = old_start;
        decl(&raw mut (*curwin.get()).w_cursor);
        pos = findmatch(::core::ptr::null_mut::<oparg_T>(), what);
        if pos.is_null() {
            (*curwin.get()).w_cursor = old_pos;
            return FAIL;
        }
        start_pos = *pos;
        (*curwin.get()).w_cursor = *pos;
        end_pos = findmatch(::core::ptr::null_mut::<oparg_T>(), other);
        if end_pos.is_null() {
            (*curwin.get()).w_cursor = old_pos;
            return FAIL;
        }
        (*curwin.get()).w_cursor = *end_pos;
    }
    if VIsual_active.get() {
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            inc(&raw mut (*curwin.get()).w_cursor);
        }
        if sol as ::core::ffi::c_int != 0 && gchar_cursor() != NUL {
            inc(&raw mut (*curwin.get()).w_cursor);
        }
        VIsual.set(start_pos);
        VIsual_mode.set('v' as ::core::ffi::c_int);
        redraw_curbuf_later(UPD_INVERTED);
        showmode();
    } else {
        (*oap).start = start_pos;
        (*oap).motion_type = kMTCharWise;
        (*oap).inclusive = false;
        if sol {
            incl(&raw mut (*curwin.get()).w_cursor);
        } else if ltoreq(start_pos, (*curwin.get()).w_cursor) {
            (*oap).inclusive = true;
        } else {
            (*curwin.get()).w_cursor = start_pos;
        }
    }
    return OK;
}
unsafe extern "C" fn in_html_tag(mut end_tag: bool) -> bool {
    let mut line: *mut ::core::ffi::c_char = get_cursor_line_ptr();
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut lc: ::core::ffi::c_int = NUL;
    let mut pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    p = line.offset((*curwin.get()).w_cursor.col as isize);
    while p > line {
        if *p as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
            break;
        }
        p = p.offset(
            -((utf_head_off(line, p.offset(-(1 as ::core::ffi::c_int as isize)))
                + 1 as ::core::ffi::c_int) as isize),
        );
        if *p as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
            break;
        }
    }
    if *p as ::core::ffi::c_int != '<' as ::core::ffi::c_int {
        return false;
    }
    pos.lnum = (*curwin.get()).w_cursor.lnum;
    pos.col = p.offset_from(line) as colnr_T;
    p = p.offset(utfc_ptr2len(p) as isize);
    if end_tag {
        return *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int;
    }
    if *p as ::core::ffi::c_int == '/' as ::core::ffi::c_int {
        return false;
    }
    loop {
        if inc(&raw mut pos) < 0 as ::core::ffi::c_int {
            return false;
        }
        let mut c: ::core::ffi::c_int = *ml_get_pos(&raw mut pos) as uint8_t as ::core::ffi::c_int;
        if c == '>' as ::core::ffi::c_int {
            break;
        }
        lc = c;
    }
    return lc != '/' as ::core::ffi::c_int;
}
pub unsafe extern "C" fn current_tagblock(
    mut oap: *mut oparg_T,
    mut count_arg: ::core::ffi::c_int,
    mut include: bool,
) -> ::core::ffi::c_int {
    let mut start_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut len: ::core::ffi::c_int = 0;
    let mut spat_len: size_t = 0;
    let mut spat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut epat_len: size_t = 0;
    let mut epat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut r: ::core::ffi::c_int = 0;
    let mut end_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut count: ::core::ffi::c_int = count_arg;
    let mut cp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut do_include: bool = include;
    let mut save_p_ws: bool = p_ws.get() != 0;
    let mut retval: ::core::ffi::c_int = FAIL;
    let mut is_inclusive: bool = true;
    p_ws.set(false_0);
    let mut old_pos: pos_T = (*curwin.get()).w_cursor;
    let mut old_end: pos_T = (*curwin.get()).w_cursor;
    let mut old_start: pos_T = old_end;
    if !VIsual_active.get() || *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
        decl(&raw mut old_end);
    }
    if !VIsual_active.get()
        || equalpos(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
    {
        setpcmark();
        while inindent(1 as ::core::ffi::c_int) {
            if inc_cursor() != 0 as ::core::ffi::c_int {
                break;
            }
        }
        if in_html_tag(false) {
            while *get_cursor_pos_ptr() as ::core::ffi::c_int != '>' as ::core::ffi::c_int {
                if inc_cursor() < 0 as ::core::ffi::c_int {
                    break;
                }
            }
        } else if in_html_tag(true) {
            while *get_cursor_pos_ptr() as ::core::ffi::c_int != '<' as ::core::ffi::c_int {
                if dec_cursor() < 0 as ::core::ffi::c_int {
                    break;
                }
            }
            dec_cursor();
            old_end = (*curwin.get()).w_cursor;
        }
    } else if lt(VIsual.get(), (*curwin.get()).w_cursor) {
        old_start = VIsual.get();
        (*curwin.get()).w_cursor = VIsual.get();
    } else {
        old_end = VIsual.get();
    }
    '_theend: {
        loop {
            let mut n: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while n < count {
                if do_searchpair(
                    c"<[^ \t>/!]\\+\\%(\\_s\\_[^>]\\{-}[^/]>\\|$\\|\\_s\\=>\\)".as_ptr(),
                    c"".as_ptr(),
                    c"</[^>]*>".as_ptr(),
                    BACKWARD as ::core::ffi::c_int,
                    ::core::ptr::null::<typval_T>(),
                    0 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<pos_T>(),
                    0 as linenr_T,
                    0 as int64_t,
                ) <= 0 as ::core::ffi::c_int
                {
                    (*curwin.get()).w_cursor = old_pos;
                    break '_theend;
                } else {
                    n += 1;
                }
            }
            start_pos = (*curwin.get()).w_cursor;
            inc_cursor();
            p = get_cursor_pos_ptr();
            cp = p;
            while *cp as ::core::ffi::c_int != NUL
                && *cp as ::core::ffi::c_int != '>' as ::core::ffi::c_int
                && !ascii_iswhite(*cp as ::core::ffi::c_int)
            {
                cp = cp.offset(utfc_ptr2len(cp) as isize);
            }
            len = cp.offset_from(p) as ::core::ffi::c_int;
            if len == 0 as ::core::ffi::c_int {
                (*curwin.get()).w_cursor = old_pos;
                break '_theend;
            } else {
                spat_len = (len as size_t).wrapping_add(39 as size_t);
                spat = xmalloc(spat_len) as *mut ::core::ffi::c_char;
                epat_len = (len as size_t).wrapping_add(9 as size_t);
                epat = xmalloc(epat_len) as *mut ::core::ffi::c_char;
                snprintf(
                    spat,
                    spat_len,
                    c"<%.*s\\>\\%%(\\_s\\_[^>]\\{-}\\_[^/]>\\|\\_s\\?>\\)\\c".as_ptr(),
                    len,
                    p,
                );
                snprintf(epat, epat_len, c"</%.*s>\\c".as_ptr(), len, p);
                r = do_searchpair(
                    spat,
                    c"".as_ptr(),
                    epat,
                    FORWARD as ::core::ffi::c_int,
                    ::core::ptr::null::<typval_T>(),
                    0 as ::core::ffi::c_int,
                    ::core::ptr::null_mut::<pos_T>(),
                    0 as linenr_T,
                    0 as int64_t,
                );
                xfree(spat as *mut ::core::ffi::c_void);
                xfree(epat as *mut ::core::ffi::c_void);
                if r < 1 as ::core::ffi::c_int
                    || lt((*curwin.get()).w_cursor, old_end) as ::core::ffi::c_int != 0
                {
                    count = 1 as ::core::ffi::c_int;
                    (*curwin.get()).w_cursor = start_pos;
                } else {
                    if do_include {
                        while *get_cursor_pos_ptr() as ::core::ffi::c_int
                            != '>' as ::core::ffi::c_int
                        {
                            if inc_cursor() < 0 as ::core::ffi::c_int {
                                break;
                            }
                        }
                    } else {
                        let mut c: *mut ::core::ffi::c_char = get_cursor_pos_ptr();
                        if *c as ::core::ffi::c_int == '<' as ::core::ffi::c_int
                            && !VIsual_active.get()
                            && (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                        {
                            is_inclusive = false;
                        } else if *c as ::core::ffi::c_int == '<' as ::core::ffi::c_int {
                            dec_cursor();
                        }
                    }
                    end_pos = (*curwin.get()).w_cursor;
                    if do_include {
                        break;
                    }
                    let mut in_quotes: bool = false;
                    (*curwin.get()).w_cursor = start_pos;
                    while inc_cursor() >= 0 as ::core::ffi::c_int {
                        p = get_cursor_pos_ptr();
                        if *p as ::core::ffi::c_int == '>' as ::core::ffi::c_int && !in_quotes {
                            inc_cursor();
                            start_pos = (*curwin.get()).w_cursor;
                            break;
                        } else if *p as ::core::ffi::c_int == '"' as ::core::ffi::c_int
                            || *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                        {
                            in_quotes = !in_quotes;
                        }
                    }
                    (*curwin.get()).w_cursor = end_pos;
                    if !(VIsual_active.get() as ::core::ffi::c_int != 0
                        && equalpos(start_pos, old_start) as ::core::ffi::c_int != 0
                        && equalpos(end_pos, old_end) as ::core::ffi::c_int != 0)
                    {
                        break;
                    }
                    do_include = true;
                    (*curwin.get()).w_cursor = old_start;
                    count = count_arg;
                }
            }
        }
        if VIsual_active.get() {
            if lt(end_pos, start_pos) {
                (*curwin.get()).w_cursor = start_pos;
            } else if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                inc_cursor();
            }
            VIsual.set(start_pos);
            VIsual_mode.set('v' as ::core::ffi::c_int);
            redraw_curbuf_later(UPD_INVERTED);
            showmode();
        } else {
            (*oap).start = start_pos;
            (*oap).motion_type = kMTCharWise;
            if lt(end_pos, start_pos) {
                (*curwin.get()).w_cursor = start_pos;
                (*oap).inclusive = false;
            } else {
                (*oap).inclusive = is_inclusive;
            }
        }
        retval = OK;
    }
    p_ws.set(save_p_ws as ::core::ffi::c_int);
    return retval;
}
