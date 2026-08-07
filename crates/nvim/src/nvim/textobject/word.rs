//! Word motions and the `iw`/`aw` objects.
//!
//! Every one of these is the same walk: step a character at a time and stop
//! when [`cls`] -- the character *class* under the cursor, folded to one
//! bucket when 'bigword' is asked for -- changes.  `w`/`b`/`e`/`ge` are the
//! four directions of it, and `current_word` composes them.

use super::*;
use crate::src::nvim::cursor::{
    coladvance, dec_cursor, gchar_cursor, get_cursor_line_ptr, inc_cursor,
};
use crate::src::nvim::drawscreen::{UPD_INVERTED, redraw_curbuf_later};
use crate::src::nvim::edit::oneleft;
use crate::src::nvim::fold::hasFolding;
use crate::src::nvim::global_cell::GlobalCell;
use crate::src::nvim::main::{
    VIsual, VIsual_active, VIsual_mode, VIsual_select_exclu_adj, curbuf, curwin, p_sel,
    redraw_cmdline,
};
use crate::src::nvim::mbyte::utf_class;
use crate::src::nvim::memline::{decl, incl, ml_get};
use crate::src::nvim::r#move::adjust_skipcol;
use crate::src::nvim::normal::unadjust_for_sel;
use crate::src::nvim::pos::{MAXCOL, clearpos, equalpos, lt, ltoreq};
use crate::src::nvim::search::{BACKWARD, FORWARD};
use crate::src::nvim::types::{colnr_T, linenr_T, oparg_T, pos_T};

static cls_bigword: GlobalCell<bool> = GlobalCell::new(false);
unsafe extern "C" fn cls() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = gchar_cursor();
    if c == ' ' as ::core::ffi::c_int || c == '\t' as ::core::ffi::c_int || c == NUL {
        return 0 as ::core::ffi::c_int;
    }
    c = utf_class(c);
    if c != 0 as ::core::ffi::c_int && cls_bigword.get() as ::core::ffi::c_int != 0 {
        return 1 as ::core::ffi::c_int;
    }
    return c;
}
pub unsafe extern "C" fn fwd_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut eol: bool,
) -> ::core::ffi::c_int {
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword.set(bigword);
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*curwin.get()).w_cursor.lnum,
        ) {
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        }
        let mut sclass: ::core::ffi::c_int = cls();
        let mut last_line: ::core::ffi::c_int = ((*curwin.get()).w_cursor.lnum
            == (*curbuf.get()).b_ml.ml_line_count)
            as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = inc_cursor();
        if i == -1 as ::core::ffi::c_int || i >= 1 as ::core::ffi::c_int && last_line != 0 {
            return FAIL;
        }
        if i >= 1 as ::core::ffi::c_int
            && eol as ::core::ffi::c_int != 0
            && count == 0 as ::core::ffi::c_int
        {
            return OK;
        }
        if sclass != 0 as ::core::ffi::c_int {
            while cls() == sclass {
                i = inc_cursor();
                if i == -1 as ::core::ffi::c_int
                    || i >= 1 as ::core::ffi::c_int
                        && eol as ::core::ffi::c_int != 0
                        && count == 0 as ::core::ffi::c_int
                {
                    return OK;
                }
            }
        }
        while cls() == 0 as ::core::ffi::c_int {
            if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                && *get_cursor_line_ptr() as ::core::ffi::c_int == NUL
            {
                break;
            }
            i = inc_cursor();
            if i == -1 as ::core::ffi::c_int
                || i >= 1 as ::core::ffi::c_int
                    && eol as ::core::ffi::c_int != 0
                    && count == 0 as ::core::ffi::c_int
            {
                return OK;
            }
        }
    }
    return OK;
}
pub unsafe extern "C" fn bck_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut stop: bool,
) -> ::core::ffi::c_int {
    let mut sclass: ::core::ffi::c_int = 0;
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword.set(bigword);
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            &raw mut (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
        ) {
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        }
        sclass = cls();
        if dec_cursor() == -1 as ::core::ffi::c_int {
            return FAIL;
        }
        '_finished: {
            if !stop || sclass == cls() || sclass == 0 as ::core::ffi::c_int {
                while cls() == 0 as ::core::ffi::c_int {
                    if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                        && *ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL
                    {
                        break '_finished;
                    }
                    if dec_cursor() == -1 as ::core::ffi::c_int {
                        return OK;
                    }
                }
                if skip_chars(cls(), BACKWARD as ::core::ffi::c_int) {
                    return OK;
                }
            }
            inc_cursor();
        }
        stop = false;
    }
    adjust_skipcol();
    return OK;
}
pub unsafe extern "C" fn end_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut stop: bool,
    mut empty: bool,
) -> ::core::ffi::c_int {
    let mut sclass: ::core::ffi::c_int = 0;
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword.set(bigword);
    if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && VIsual_active.get() as ::core::ffi::c_int != 0
        && VIsual_mode.get() == 'v' as ::core::ffi::c_int
        && VIsual_select_exclu_adj.get() as ::core::ffi::c_int != 0
    {
        unadjust_for_sel();
    }
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        if hasFolding(
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            ::core::ptr::null_mut::<linenr_T>(),
            &raw mut (*curwin.get()).w_cursor.lnum,
        ) {
            coladvance(curwin.get(), MAXCOL as ::core::ffi::c_int);
        }
        sclass = cls();
        if inc_cursor() == -1 as ::core::ffi::c_int {
            return FAIL;
        }
        '_finished: {
            if cls() == sclass && sclass != 0 as ::core::ffi::c_int {
                if skip_chars(sclass, FORWARD as ::core::ffi::c_int) {
                    return FAIL;
                }
            } else if !stop || sclass == 0 as ::core::ffi::c_int {
                while cls() == 0 as ::core::ffi::c_int {
                    if empty as ::core::ffi::c_int != 0
                        && (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                        && *ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL
                    {
                        break '_finished;
                    }
                    if inc_cursor() == -1 as ::core::ffi::c_int {
                        return FAIL;
                    }
                }
                if skip_chars(cls(), FORWARD as ::core::ffi::c_int) {
                    return FAIL;
                }
            }
            dec_cursor();
        }
        stop = false;
    }
    return OK;
}
pub unsafe extern "C" fn bckend_word(
    mut count: ::core::ffi::c_int,
    mut bigword: bool,
    mut eol: bool,
) -> ::core::ffi::c_int {
    (*curwin.get()).w_cursor.coladd = 0 as ::core::ffi::c_int as colnr_T;
    cls_bigword.set(bigword);
    loop {
        count -= 1;
        if count < 0 as ::core::ffi::c_int {
            break;
        }
        let mut i: ::core::ffi::c_int = 0;
        let mut sclass: ::core::ffi::c_int = cls();
        i = dec_cursor();
        if i == -1 as ::core::ffi::c_int {
            return FAIL;
        }
        if eol as ::core::ffi::c_int != 0 && i == 1 as ::core::ffi::c_int {
            return OK;
        }
        if sclass != 0 as ::core::ffi::c_int {
            while cls() == sclass {
                i = dec_cursor();
                if i == -1 as ::core::ffi::c_int
                    || eol as ::core::ffi::c_int != 0 && i == 1 as ::core::ffi::c_int
                {
                    return OK;
                }
            }
        }
        while cls() == 0 as ::core::ffi::c_int {
            if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int
                && *ml_get((*curwin.get()).w_cursor.lnum) as ::core::ffi::c_int == NUL
            {
                break;
            }
            i = dec_cursor();
            if i == -1 as ::core::ffi::c_int
                || eol as ::core::ffi::c_int != 0 && i == 1 as ::core::ffi::c_int
            {
                return OK;
            }
        }
    }
    adjust_skipcol();
    return OK;
}
unsafe extern "C" fn skip_chars(
    mut cclass: ::core::ffi::c_int,
    mut dir: ::core::ffi::c_int,
) -> bool {
    while cls() == cclass {
        if (if dir == FORWARD as ::core::ffi::c_int {
            inc_cursor()
        } else {
            dec_cursor()
        }) == -1 as ::core::ffi::c_int
        {
            return true;
        }
    }
    return false;
}
unsafe extern "C" fn back_in_line() {
    let mut sclass: ::core::ffi::c_int = cls();
    while (*curwin.get()).w_cursor.col != 0 as ::core::ffi::c_int {
        dec_cursor();
        if cls() == sclass {
            continue;
        }
        inc_cursor();
        break;
    }
}
pub unsafe extern "C" fn current_word(
    mut oap: *mut oparg_T,
    mut count: ::core::ffi::c_int,
    mut include: bool,
    mut bigword: bool,
) -> ::core::ffi::c_int {
    let mut start_pos: pos_T = pos_T {
        lnum: 0,
        col: 0,
        coladd: 0,
    };
    let mut inclusive: bool = true;
    let mut include_white: bool = false;
    cls_bigword.set(bigword);
    clearpos(&mut start_pos);
    if VIsual_active.get() as ::core::ffi::c_int != 0
        && *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
        && lt(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
    {
        dec_cursor();
    }
    if !VIsual_active.get()
        || equalpos((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0
    {
        back_in_line();
        start_pos = (*curwin.get()).w_cursor;
        if (cls() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int == include as ::core::ffi::c_int
        {
            if end_word(1 as ::core::ffi::c_int, bigword, true, true) == FAIL {
                return FAIL;
            }
        } else {
            fwd_word(1 as ::core::ffi::c_int, bigword, true);
            if (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int {
                decl(&raw mut (*curwin.get()).w_cursor);
            } else {
                oneleft();
            }
            if include {
                include_white = true;
            }
        }
        if VIsual_active.get() {
            VIsual.set(start_pos);
            redraw_curbuf_later(UPD_INVERTED);
        } else {
            (*oap).start = start_pos;
            (*oap).motion_type = kMTCharWise;
        }
        count -= 1;
    }
    while count > 0 as ::core::ffi::c_int {
        inclusive = true;
        if VIsual_active.get() as ::core::ffi::c_int != 0
            && lt((*curwin.get()).w_cursor, VIsual.get()) as ::core::ffi::c_int != 0
        {
            if decl(&raw mut (*curwin.get()).w_cursor) == -1 as ::core::ffi::c_int {
                return FAIL;
            }
            if include as ::core::ffi::c_int
                != (cls() != 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            {
                if bck_word(1 as ::core::ffi::c_int, bigword, true) == FAIL {
                    return FAIL;
                }
            } else {
                if bckend_word(1 as ::core::ffi::c_int, bigword, true) == FAIL {
                    return FAIL;
                }
                incl(&raw mut (*curwin.get()).w_cursor);
            }
        } else {
            if incl(&raw mut (*curwin.get()).w_cursor) == -1 as ::core::ffi::c_int {
                return FAIL;
            }
            if include as ::core::ffi::c_int
                != (cls() == 0 as ::core::ffi::c_int) as ::core::ffi::c_int
            {
                if fwd_word(1 as ::core::ffi::c_int, bigword, true) == FAIL
                    && count > 1 as ::core::ffi::c_int
                {
                    return FAIL;
                }
                if oneleft() == FAIL {
                    inclusive = false;
                }
            } else if end_word(1 as ::core::ffi::c_int, bigword, true, true) == FAIL {
                return FAIL;
            }
        }
        count -= 1;
    }
    if include_white as ::core::ffi::c_int != 0
        && (cls() != 0 as ::core::ffi::c_int
            || (*curwin.get()).w_cursor.col == 0 as ::core::ffi::c_int && !inclusive)
    {
        let mut pos: pos_T = (*curwin.get()).w_cursor;
        (*curwin.get()).w_cursor = start_pos;
        if oneleft() == OK {
            back_in_line();
            if cls() == 0 as ::core::ffi::c_int
                && (*curwin.get()).w_cursor.col > 0 as ::core::ffi::c_int
            {
                if VIsual_active.get() {
                    VIsual.set((*curwin.get()).w_cursor);
                } else {
                    (*oap).start = (*curwin.get()).w_cursor;
                }
            }
        }
        (*curwin.get()).w_cursor = pos;
    }
    if VIsual_active.get() {
        if *p_sel.get() as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
            && inclusive as ::core::ffi::c_int != 0
            && ltoreq(VIsual.get(), (*curwin.get()).w_cursor) as ::core::ffi::c_int != 0
        {
            inc_cursor();
        }
        if VIsual_mode.get() == 'V' as ::core::ffi::c_int {
            VIsual_mode.set('v' as ::core::ffi::c_int);
            redraw_cmdline.set(true);
        }
    } else {
        (*oap).inclusive = inclusive;
    }
    return OK;
}
