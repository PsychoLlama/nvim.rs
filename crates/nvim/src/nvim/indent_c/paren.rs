//! Finding the enclosing bracket, and the matching keyword.
//!
//! `find_match_paren`/`find_match_char` search backwards for an unclosed
//! `(`/`[`, `find_start_brace` for an unclosed `{` that is not inside a comment
//! or a paren, both bounded by 'cinoptions' `)N` (`b_ind_maxparen`).
//! `find_last_paren` puts the cursor on the rightmost unmatched bracket of a
//! line first, which is what makes the backwards search start in the right
//! place.  `find_match` is the other kind of matching: the `if` an `else`
//! belongs to, or the `do` a `while` closes.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn cin_skip2pos(mut trypos: *mut pos_T) -> ::core::ffi::c_int {
    unsafe {
        let mut line: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut new_p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        line = ml_get((*trypos).lnum);
        p = line;
        while *p as ::core::ffi::c_int != 0 && (p.offset_from(line) as colnr_T) < (*trypos).col {
            if cin_iscomment(p) != 0 {
                p = cin_skipcomment(p);
            } else {
                new_p = skip_string(p);
                if new_p == p {
                    p = p.offset(1);
                } else {
                    p = new_p;
                }
            }
        }
        return p.offset_from(line) as ::core::ffi::c_int;
    }
}

pub(crate) unsafe extern "C" fn find_start_brace() -> *mut pos_T {
    unsafe {
        let mut cursor_save: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut pos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        static pos_copy: GlobalCell<pos_T> = GlobalCell::new(pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        });
        cursor_save = (*curwin.get()).w_cursor;
        loop {
            trypos = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                '{' as ::core::ffi::c_int,
                FM_BLOCKSTOP as ::core::ffi::c_int,
                0 as int64_t,
            );
            if trypos.is_null() {
                break;
            }
            pos_copy.set(*trypos);
            trypos = pos_copy.ptr();
            (*curwin.get()).w_cursor = *trypos;
            pos = ::core::ptr::null_mut::<pos_T>();
            if cin_skip2pos(trypos) == (*trypos).col && {
                pos = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
                pos.is_null()
            } {
                break;
            }
            if !pos.is_null() {
                (*curwin.get()).w_cursor = *pos;
            }
        }
        (*curwin.get()).w_cursor = cursor_save;
        return trypos;
    }
}

pub(crate) unsafe extern "C" fn find_match_paren(
    mut ind_maxparen: ::core::ffi::c_int,
) -> *mut pos_T {
    unsafe {
        return find_match_char('(' as ::core::ffi::c_char, ind_maxparen);
    }
}

pub(crate) unsafe extern "C" fn find_match_char(
    mut c: ::core::ffi::c_char,
    mut ind_maxparen: ::core::ffi::c_int,
) -> *mut pos_T {
    unsafe {
        let mut cursor_save: pos_T = pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        };
        let mut trypos: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        static pos_copy: GlobalCell<pos_T> = GlobalCell::new(pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        });
        let mut ind_maxp_wk: ::core::ffi::c_int = 0;
        cursor_save = (*curwin.get()).w_cursor;
        ind_maxp_wk = ind_maxparen;
        loop {
            trypos = findmatchlimit(
                ::core::ptr::null_mut::<oparg_T>(),
                c as uint8_t as ::core::ffi::c_int,
                0 as ::core::ffi::c_int,
                ind_maxp_wk as int64_t,
            );
            if trypos.is_null() {
                break;
            }
            if cin_skip2pos(trypos) > (*trypos).col {
                ind_maxp_wk = (ind_maxparen as linenr_T - (cursor_save.lnum - (*trypos).lnum))
                    as ::core::ffi::c_int;
                if ind_maxp_wk > 0 as ::core::ffi::c_int {
                    (*curwin.get()).w_cursor = *trypos;
                    (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
                } else {
                    trypos = ::core::ptr::null_mut::<pos_T>();
                    break;
                }
            } else {
                let mut trypos_wk: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
                pos_copy.set(*trypos);
                trypos = pos_copy.ptr();
                (*curwin.get()).w_cursor = *trypos;
                trypos_wk = ind_find_start_CORS(::core::ptr::null_mut::<linenr_T>());
                if trypos_wk.is_null() {
                    break;
                }
                ind_maxp_wk = (ind_maxparen as linenr_T - (cursor_save.lnum - (*trypos_wk).lnum))
                    as ::core::ffi::c_int;
                if ind_maxp_wk > 0 as ::core::ffi::c_int {
                    (*curwin.get()).w_cursor = *trypos_wk;
                } else {
                    trypos = ::core::ptr::null_mut::<pos_T>();
                    break;
                }
            }
        }
        (*curwin.get()).w_cursor = cursor_save;
        return trypos;
    }
}

pub(crate) unsafe extern "C" fn find_match_paren_after_brace(
    mut ind_maxparen: ::core::ffi::c_int,
) -> *mut pos_T {
    unsafe {
        let mut trypos: *mut pos_T = find_match_paren(ind_maxparen);
        if trypos.is_null() {
            return ::core::ptr::null_mut::<pos_T>();
        }
        let mut tryposBrace: *mut pos_T = find_start_brace();
        if !tryposBrace.is_null()
            && (if (*trypos).lnum != (*tryposBrace).lnum {
                ((*trypos).lnum < (*tryposBrace).lnum) as ::core::ffi::c_int
            } else {
                ((*trypos).col < (*tryposBrace).col) as ::core::ffi::c_int
            }) != 0
        {
            trypos = ::core::ptr::null_mut::<pos_T>();
        }
        return trypos;
    }
}

pub(crate) unsafe extern "C" fn corr_ind_maxparen(mut startpos: *mut pos_T) -> ::core::ffi::c_int {
    unsafe {
        let mut n: ::core::ffi::c_int = (*startpos).lnum as ::core::ffi::c_int
            - (*curwin.get()).w_cursor.lnum as ::core::ffi::c_int;
        if n > 0 as ::core::ffi::c_int
            && n < (*curbuf.get()).b_ind_maxparen / 2 as ::core::ffi::c_int
        {
            return (*curbuf.get()).b_ind_maxparen - n;
        }
        return (*curbuf.get()).b_ind_maxparen;
    }
}

pub(crate) unsafe extern "C" fn find_last_paren(
    mut l: *const ::core::ffi::c_char,
    mut start: ::core::ffi::c_char,
    mut end: ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut i: ::core::ffi::c_int = 0;
        let mut retval: ::core::ffi::c_int = false_0;
        let mut open_count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        i = 0 as ::core::ffi::c_int;
        while *l.offset(i as isize) as ::core::ffi::c_int != NUL {
            i = cin_skipcomment(l.offset(i as isize)).offset_from(l) as ::core::ffi::c_int;
            i = skip_string(l.offset(i as isize)).offset_from(l) as ::core::ffi::c_int;
            if *l.offset(i as isize) as ::core::ffi::c_int == start as ::core::ffi::c_int {
                open_count += 1;
            } else if *l.offset(i as isize) as ::core::ffi::c_int == end as ::core::ffi::c_int {
                if open_count > 0 as ::core::ffi::c_int {
                    open_count -= 1;
                } else {
                    (*curwin.get()).w_cursor.col = i as colnr_T;
                    retval = true_0;
                }
            }
            i += 1;
        }
        return retval;
    }
}

pub(crate) unsafe extern "C" fn find_match(
    mut lookfor: ::core::ffi::c_int,
    mut ourscope: linenr_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut look: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut theirscope: *mut pos_T = ::core::ptr::null_mut::<pos_T>();
        let mut mightbeif: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut elselevel: ::core::ffi::c_int = 0;
        let mut whilelevel: ::core::ffi::c_int = 0;
        if lookfor == LOOKFOR_IF {
            elselevel = 1 as ::core::ffi::c_int;
            whilelevel = 0 as ::core::ffi::c_int;
        } else {
            elselevel = 0 as ::core::ffi::c_int;
            whilelevel = 1 as ::core::ffi::c_int;
        }
        (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
        while (*curwin.get()).w_cursor.lnum > ourscope + 1 as linenr_T {
            (*curwin.get()).w_cursor.lnum -= 1;
            (*curwin.get()).w_cursor.col = 0 as ::core::ffi::c_int as colnr_T;
            look = cin_skipcomment(get_cursor_line_ptr());
            if cin_iselse(look) == 0
                && cin_isif(look) == 0
                && cin_isdo(look) == 0
                && cin_iswhileofdo(look, (*curwin.get()).w_cursor.lnum) == 0
            {
                continue;
            }
            theirscope = find_start_brace();
            if theirscope.is_null() {
                break;
            }
            if (*theirscope).lnum < ourscope {
                break;
            }
            if (*theirscope).lnum > ourscope {
                continue;
            }
            look = cin_skipcomment(get_cursor_line_ptr());
            if !(lookfor == LOOKFOR_IF && whilelevel != 0) {
                if cin_iselse(look) != 0 {
                    mightbeif = cin_skipcomment(look.offset(4 as ::core::ffi::c_int as isize));
                    if cin_isif(mightbeif) == 0 {
                        elselevel += 1;
                    }
                    continue;
                } else if cin_isif(look) != 0 {
                    elselevel -= 1;
                    if elselevel == 0 as ::core::ffi::c_int && lookfor == LOOKFOR_IF {
                        whilelevel = 0 as ::core::ffi::c_int;
                    }
                }
            }
            if cin_iswhileofdo(look, (*curwin.get()).w_cursor.lnum) != 0 {
                whilelevel += 1;
            } else {
                if cin_isdo(look) != 0 {
                    whilelevel -= 1;
                }
                if elselevel <= 0 as ::core::ffi::c_int && whilelevel <= 0 as ::core::ffi::c_int {
                    return OK;
                }
            }
        }
        return FAIL;
    }
}
