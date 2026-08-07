//! Sentences: the `(`/`)` motions and the `is`/`as` objects.
//!
//! A sentence ends at `.`, `!` or `?` followed by white space or end of
//! line, with any of `)]"'` allowed in between, and 'cpoptions' `J` decides
//! whether one space is enough.  [`findsent`] is that rule; everything else
//! here is about which side of the trailing white space an object claims.

use super::*;
use crate::src::nvim::ascii::ascii_iswhite;
use crate::src::nvim::cursor::gchar_cursor;
use crate::src::nvim::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::src::nvim::main::{
    VIsual, VIsual_active, VIsual_mode, curbuf, curwin, p_cpo, p_sel, redraw_cmdline,
};
use crate::src::nvim::mark::setpcmark;
use crate::src::nvim::memline::{decl, gchar_pos, inc, incl, ml_get};
use crate::src::nvim::pos::{equalpos, lt};
use crate::src::nvim::search::{BACKWARD, FORWARD};
use crate::src::nvim::strings::vim_strchr;
use crate::src::nvim::types::{Direction, linenr_T, oparg_T, pos_T};

pub unsafe extern "C" fn findsent(
    mut dir: Direction,
    mut count: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut found_dot: bool = false;
    let mut startlnum: ::core::ffi::c_int = 0;
    let mut cpo_J: bool = false;
    let mut c: ::core::ffi::c_int = 0;
    let mut func: Option<unsafe fn(*mut pos_T) -> ::core::ffi::c_int> = None;
    let mut noskip: bool = false;
    let mut pos: pos_T = (*curwin.get()).w_cursor;
    if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
        func = Some(incl as unsafe fn(*mut pos_T) -> ::core::ffi::c_int);
    } else {
        func = Some(decl as unsafe fn(*mut pos_T) -> ::core::ffi::c_int);
    }
    loop {
        let c2rust_fresh0 = count;
        count = count - 1;
        if c2rust_fresh0 == 0 {
            break;
        }
        let prev_pos: pos_T = pos;
        '_found: {
            if gchar_pos(&raw mut pos) == NUL {
                while func.expect("non-null function pointer")(&raw mut pos)
                    != -1 as ::core::ffi::c_int
                {
                    if gchar_pos(&raw mut pos) != NUL {
                        break;
                    }
                }
                if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int {
                    break '_found;
                }
            } else if dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                && pos.col == 0 as ::core::ffi::c_int
                && startPS(pos.lnum, NUL, false) as ::core::ffi::c_int != 0
            {
                if pos.lnum == (*curbuf.get()).b_ml.ml_line_count {
                    return FAIL;
                }
                pos.lnum += 1;
                break '_found;
            } else if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int {
                decl(&raw mut pos);
            }
            found_dot = false;
            loop {
                c = gchar_pos(&raw mut pos);
                if !(ascii_iswhite(c) as ::core::ffi::c_int != 0
                    || !vim_strchr(c".!?)]\"'".as_ptr(), c).is_null())
                {
                    break;
                }
                let mut tpos: pos_T = pos;
                if decl(&raw mut tpos) == -1 as ::core::ffi::c_int
                    || *ml_get(tpos.lnum) as ::core::ffi::c_int == NUL
                        && dir as ::core::ffi::c_int == FORWARD as ::core::ffi::c_int
                {
                    break;
                }
                if found_dot {
                    break;
                }
                if !vim_strchr(c".!?".as_ptr(), c).is_null() {
                    found_dot = true;
                }
                if !vim_strchr(c")]\"'".as_ptr(), c).is_null()
                    && vim_strchr(c".!?)]\"'".as_ptr(), gchar_pos(&raw mut tpos)).is_null()
                {
                    break;
                }
                decl(&raw mut pos);
            }
            startlnum = pos.lnum as ::core::ffi::c_int;
            cpo_J = !vim_strchr(p_cpo.get(), CPO_ENDOFSENT).is_null();
            loop {
                c = gchar_pos(&raw mut pos);
                if c == NUL
                    || pos.col == 0 as ::core::ffi::c_int
                        && startPS(pos.lnum, NUL, false) as ::core::ffi::c_int != 0
                {
                    if dir as ::core::ffi::c_int == BACKWARD as ::core::ffi::c_int
                        && pos.lnum != startlnum as linenr_T
                    {
                        pos.lnum += 1;
                    }
                    break;
                } else {
                    if c == '.' as ::core::ffi::c_int
                        || c == '!' as ::core::ffi::c_int
                        || c == '?' as ::core::ffi::c_int
                    {
                        let mut tpos_0: pos_T = pos;
                        loop {
                            c = inc(&raw mut tpos_0);
                            if c == -1 as ::core::ffi::c_int {
                                break;
                            }
                            c = gchar_pos(&raw mut tpos_0);
                            if vim_strchr(c")]\"'".as_ptr(), c).is_null() {
                                break;
                            }
                        }
                        if c == -1 as ::core::ffi::c_int
                            || !cpo_J
                                && (c == ' ' as ::core::ffi::c_int
                                    || c == '\t' as ::core::ffi::c_int)
                            || c == NUL
                            || cpo_J as ::core::ffi::c_int != 0
                                && (c == ' ' as ::core::ffi::c_int
                                    && inc(&raw mut tpos_0) >= 0 as ::core::ffi::c_int
                                    && gchar_pos(&raw mut tpos_0) == ' ' as ::core::ffi::c_int)
                        {
                            pos = tpos_0;
                            if gchar_pos(&raw mut pos) == NUL {
                                inc(&raw mut pos);
                            }
                            break;
                        }
                    }
                    if func.expect("non-null function pointer")(&raw mut pos)
                        != -1 as ::core::ffi::c_int
                    {
                        continue;
                    }
                    if count != 0 {
                        return FAIL;
                    }
                    noskip = true;
                    break;
                }
            }
        }
        while !noskip && {
            c = gchar_pos(&raw mut pos);
            c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int
        } {
            if incl(&raw mut pos) == -1 as ::core::ffi::c_int {
                break;
            }
        }
        if !equalpos(prev_pos, pos) {
            continue;
        }
        if func.expect("non-null function pointer")(&raw mut pos) == -1 as ::core::ffi::c_int {
            if count != 0 {
                return FAIL;
            }
            break;
        } else {
            count += 1;
        }
    }
    setpcmark();
    (*curwin.get()).w_cursor = pos;
    return OK;
}
unsafe extern "C" fn find_first_blank(mut posp: *mut pos_T) {
    while decl(posp) != -1 as ::core::ffi::c_int {
        let mut c: ::core::ffi::c_int = gchar_pos(posp);
        if ascii_iswhite(c) {
            continue;
        }
        incl(posp);
        break;
    }
}
unsafe extern "C" fn findsent_forward(mut count: ::core::ffi::c_int, mut at_start_sent: bool) {
    loop {
        let c2rust_fresh3 = count;
        count = count - 1;
        if c2rust_fresh3 == 0 {
            break;
        }
        findsent(FORWARD, 1 as ::core::ffi::c_int);
        if at_start_sent {
            find_first_blank(&raw mut (*curwin.get()).w_cursor);
        }
        if count == 0 as ::core::ffi::c_int || at_start_sent as ::core::ffi::c_int != 0 {
            decl(&raw mut (*curwin.get()).w_cursor);
        }
        at_start_sent = !at_start_sent;
    }
}
pub unsafe extern "C" fn current_sent(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
) -> ::core::ffi::c_int {
    let mut start_blank: bool = false;
    let mut c: ::core::ffi::c_int = 0;
    let mut at_start_sent: bool = false;
    let mut ncount: ::core::ffi::c_int = 0;
    let mut start_pos: pos_T = (*curwin.get()).w_cursor;
    let mut pos: pos_T = start_pos;
    findsent(FORWARD, 1 as ::core::ffi::c_int);
    '_extend: {
        if !(VIsual_active.get() as ::core::ffi::c_int != 0 && !equalpos(start_pos, VIsual.get())) {
            loop {
                c = gchar_pos(&raw mut pos);
                if !ascii_iswhite(c) {
                    break;
                }
                incl(&raw mut pos);
            }
            if equalpos(pos, (*curwin.get()).w_cursor) {
                start_blank = true;
                find_first_blank(&raw mut start_pos);
            } else {
                start_blank = false;
                findsent(BACKWARD, 1 as ::core::ffi::c_int);
                start_pos = (*curwin.get()).w_cursor;
            }
            if include {
                ncount = count * 2 as ::core::ffi::c_int;
            } else {
                ncount = count;
                if start_blank {
                    ncount -= 1;
                }
            }
            if ncount > 0 as ::core::ffi::c_int {
                findsent_forward(ncount, true);
            } else {
                decl(&raw mut (*curwin.get()).w_cursor);
            }
            if include {
                if start_blank {
                    find_first_blank(&raw mut (*curwin.get()).w_cursor);
                    c = gchar_pos(&raw mut (*curwin.get()).w_cursor);
                    if ascii_iswhite(c) {
                        decl(&raw mut (*curwin.get()).w_cursor);
                    }
                } else {
                    c = gchar_cursor();
                    if !ascii_iswhite(c) as ::core::ffi::c_int != 0 {
                        find_first_blank(&raw mut start_pos);
                    }
                }
            }
            if VIsual_active.get() {
                if equalpos(start_pos, (*curwin.get()).w_cursor) {
                    break '_extend;
                } else {
                    if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                        (*curwin.get()).w_cursor.col += 1;
                    }
                    VIsual.set(start_pos);
                    VIsual_mode.set('v' as ::core::ffi::c_int);
                    redraw_cmdline.set(true);
                    redraw_curbuf_later(UPD_INVERTED);
                }
            } else {
                (*oap).inclusive =
                    incl(&raw mut (*curwin.get()).w_cursor) == -1 as ::core::ffi::c_int;
                (*oap).start = start_pos;
                (*oap).motion_type = kMTCharWise;
            }
            return OK;
        }
    }
    if lt(start_pos, VIsual.get()) {
        at_start_sent = true;
        decl(&raw mut pos);
        while lt(pos, (*curwin.get()).w_cursor) {
            c = gchar_pos(&raw mut pos);
            if !ascii_iswhite(c) {
                at_start_sent = false;
                break;
            } else {
                incl(&raw mut pos);
            }
        }
        if !at_start_sent {
            findsent(BACKWARD, 1 as ::core::ffi::c_int);
            if equalpos((*curwin.get()).w_cursor, start_pos) {
                at_start_sent = true;
            } else {
                findsent(FORWARD, 1 as ::core::ffi::c_int);
            }
        }
        if include {
            count *= 2 as ::core::ffi::c_int;
        }
        loop {
            let c2rust_fresh2 = count;
            count = count - 1;
            if c2rust_fresh2 == 0 {
                break;
            }
            if at_start_sent {
                find_first_blank(&raw mut (*curwin.get()).w_cursor);
            }
            c = gchar_cursor();
            if !at_start_sent || !include && !ascii_iswhite(c) {
                findsent(BACKWARD, 1 as ::core::ffi::c_int);
            }
            at_start_sent = !at_start_sent;
        }
    } else {
        incl(&raw mut pos);
        at_start_sent = true;
        if !equalpos(pos, (*curwin.get()).w_cursor) {
            at_start_sent = false;
            while lt(pos, (*curwin.get()).w_cursor) {
                c = gchar_pos(&raw mut pos);
                if !ascii_iswhite(c) {
                    at_start_sent = true;
                    break;
                } else {
                    incl(&raw mut pos);
                }
            }
            if at_start_sent {
                findsent(BACKWARD, 1 as ::core::ffi::c_int);
            } else {
                (*curwin.get()).w_cursor = start_pos;
            }
        }
        if include {
            count *= 2 as ::core::ffi::c_int;
        }
        findsent_forward(count, at_start_sent);
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
            (*curwin.get()).w_cursor.col += 1;
        }
    }
    return OK;
}
