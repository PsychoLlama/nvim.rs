//! The rest of the recursive descent: an atom with its multi, a
//! concatenation, a branch, and the whole pattern.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn nfa_regpiece() -> ::core::ffi::c_int {
    let mut i: ::core::ffi::c_int = 0;
    let mut op: ::core::ffi::c_int = 0;
    let mut ret: ::core::ffi::c_int = 0;
    let mut minval: ::core::ffi::c_int = 0;
    let mut maxval: ::core::ffi::c_int = 0;
    let mut greedy: bool = true_0 != 0;
    let mut old_state: parse_state_T = parse_state_T {
        regparse: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        prevchr_len: 0,
        curchr: 0,
        prevchr: 0,
        prevprevchr: 0,
        nextchr: 0,
        at_start: 0,
        prev_at_start: 0,
        regnpar: 0,
    };
    let mut new_state: parse_state_T = parse_state_T {
        regparse: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        prevchr_len: 0,
        curchr: 0,
        prevchr: 0,
        prevprevchr: 0,
        nextchr: 0,
        at_start: 0,
        prev_at_start: 0,
        regnpar: 0,
    };
    let mut c2: int64_t = 0;
    let mut old_post_pos: ::core::ffi::c_int = 0;
    let mut my_post_start: ::core::ffi::c_int = 0;
    let mut quest: ::core::ffi::c_int = 0;
    save_parse_state(&raw mut old_state);
    my_post_start = (*post_ptr.ptr()).offset_from(post_start.get()) as ::core::ffi::c_int;
    ret = nfa_regatom();
    if ret == FAIL {
        return FAIL;
    }
    op = peekchr();
    if re_multi_type(op) == NOT_MULTI {
        return OK;
    }
    skipchr();
    match op {
        -214 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh29 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh29 = NFA_STAR as ::core::ffi::c_int;
        }
        -213 => {
            restore_parse_state(&raw mut old_state);
            curchr.set(-1 as ::core::ffi::c_int);
            if nfa_regatom() == FAIL {
                return FAIL;
            }
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh30 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh30 = NFA_STAR as ::core::ffi::c_int;
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh31 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh31 = NFA_CONCAT as ::core::ffi::c_int;
            skipchr();
        }
        -192 => {
            c2 = getdecchrs();
            op = no_Magic(getchr());
            i = 0 as ::core::ffi::c_int;
            match op {
                61 => {
                    i = NFA_PREV_ATOM_NO_WIDTH as ::core::ffi::c_int;
                }
                33 => {
                    i = NFA_PREV_ATOM_NO_WIDTH_NEG as ::core::ffi::c_int;
                }
                60 => {
                    op = no_Magic(getchr());
                    if op == '=' as ::core::ffi::c_int {
                        i = NFA_PREV_ATOM_JUST_BEFORE as ::core::ffi::c_int;
                    } else if op == '!' as ::core::ffi::c_int {
                        i = NFA_PREV_ATOM_JUST_BEFORE_NEG as ::core::ffi::c_int;
                    }
                }
                62 => {
                    i = NFA_PREV_ATOM_LIKE_PATTERN as ::core::ffi::c_int;
                }
                _ => {}
            }
            if i == 0 as ::core::ffi::c_int {
                semsg(
                    gettext(b"E869: (NFA) Unknown operator '\\@%c'\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    op,
                );
                return FAIL;
            }
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh32 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh32 = i;
            if i == NFA_PREV_ATOM_JUST_BEFORE as ::core::ffi::c_int
                || i == NFA_PREV_ATOM_JUST_BEFORE_NEG as ::core::ffi::c_int
            {
                if post_ptr.get() >= post_end.get() {
                    realloc_post_list();
                }
                let c2rust_fresh33 = post_ptr.get();
                post_ptr.set((*post_ptr.ptr()).offset(1));
                *c2rust_fresh33 = c2 as ::core::ffi::c_int;
            }
        }
        -193 | -195 => {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh34 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh34 = NFA_QUEST as ::core::ffi::c_int;
        }
        -133 => {
            greedy = true_0 != 0;
            c2 = peekchr() as int64_t;
            if c2 == '-' as int64_t
                || c2 == ('-' as ::core::ffi::c_int - 256 as ::core::ffi::c_int) as int64_t
            {
                skipchr();
                greedy = false_0 != 0;
            }
            if read_limits(&raw mut minval, &raw mut maxval) == 0 {
                emsg(gettext(
                    b"E870: (NFA regexp) Error reading repetition limits\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
                rc_did_emsg.set(true_0 != 0);
                return FAIL;
            }
            if minval == 0 as ::core::ffi::c_int && maxval == MAX_LIMIT {
                if greedy {
                    if post_ptr.get() >= post_end.get() {
                        realloc_post_list();
                    }
                    let c2rust_fresh35 = post_ptr.get();
                    post_ptr.set((*post_ptr.ptr()).offset(1));
                    *c2rust_fresh35 = NFA_STAR as ::core::ffi::c_int;
                } else {
                    if post_ptr.get() >= post_end.get() {
                        realloc_post_list();
                    }
                    let c2rust_fresh36 = post_ptr.get();
                    post_ptr.set((*post_ptr.ptr()).offset(1));
                    *c2rust_fresh36 = NFA_STAR_NONGREEDY as ::core::ffi::c_int;
                }
            } else {
                if maxval == 0 as ::core::ffi::c_int {
                    post_ptr.set((*post_start.ptr()).offset(my_post_start as isize));
                    if post_ptr.get() >= post_end.get() {
                        realloc_post_list();
                    }
                    let c2rust_fresh37 = post_ptr.get();
                    post_ptr.set((*post_ptr.ptr()).offset(1));
                    *c2rust_fresh37 = NFA_EMPTY as ::core::ffi::c_int;
                    return OK;
                }
                if nfa_re_flags.get() & RE_AUTO != 0
                    && (maxval > 500 as ::core::ffi::c_int
                        || maxval > minval + 200 as ::core::ffi::c_int)
                    && (maxval != MAX_LIMIT && minval < 200 as ::core::ffi::c_int)
                    && !wants_nfa.get()
                {
                    return FAIL;
                }
                post_ptr.set((*post_start.ptr()).offset(my_post_start as isize));
                save_parse_state(&raw mut new_state);
                quest = if greedy as ::core::ffi::c_int == true_0 {
                    NFA_QUEST as ::core::ffi::c_int
                } else {
                    NFA_QUEST_NONGREEDY as ::core::ffi::c_int
                };
                i = 0 as ::core::ffi::c_int;
                while i < maxval {
                    restore_parse_state(&raw mut old_state);
                    old_post_pos =
                        (*post_ptr.ptr()).offset_from(post_start.get()) as ::core::ffi::c_int;
                    if nfa_regatom() == FAIL {
                        return FAIL;
                    }
                    if i + 1 as ::core::ffi::c_int > minval {
                        if maxval == MAX_LIMIT {
                            if greedy {
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh38 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh38 = NFA_STAR as ::core::ffi::c_int;
                            } else {
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh39 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh39 = NFA_STAR_NONGREEDY as ::core::ffi::c_int;
                            }
                        } else {
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh40 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh40 = quest;
                        }
                    }
                    if old_post_pos != my_post_start {
                        if post_ptr.get() >= post_end.get() {
                            realloc_post_list();
                        }
                        let c2rust_fresh41 = post_ptr.get();
                        post_ptr.set((*post_ptr.ptr()).offset(1));
                        *c2rust_fresh41 = NFA_CONCAT as ::core::ffi::c_int;
                    }
                    if i + 1 as ::core::ffi::c_int > minval && maxval == MAX_LIMIT {
                        break;
                    }
                    i += 1;
                }
                restore_parse_state(&raw mut new_state);
                curchr.set(-1 as ::core::ffi::c_int);
            }
        }
        _ => {}
    }
    if re_multi_type(peekchr()) != NOT_MULTI {
        emsg(gettext(
            b"E871: (NFA regexp) Can't have a multi follow a multi\0".as_ptr()
                as *const ::core::ffi::c_char,
        ));
        rc_did_emsg.set(true_0 != 0);
        return FAIL;
    }
    return OK;
}
pub(crate) unsafe extern "C" fn nfa_regconcat() -> ::core::ffi::c_int {
    let mut cont: bool = true_0 != 0;
    let mut first: bool = true_0 != 0;
    while cont {
        match peekchr() {
            NUL | -132 | -218 | -215 => {
                cont = false_0 != 0;
            }
            -166 => {
                (*regflags.ptr()) |= RF_ICOMBINE as ::core::ffi::c_uint;
                skipchr_keepstart();
            }
            -157 => {
                (*regflags.ptr()) |= RF_ICASE as ::core::ffi::c_uint;
                skipchr_keepstart();
            }
            -189 => {
                (*regflags.ptr()) |= RF_NOICASE as ::core::ffi::c_uint;
                skipchr_keepstart();
            }
            -138 => {
                reg_magic.set(MAGIC_ALL);
                skipchr_keepstart();
                curchr.set(-1 as ::core::ffi::c_int);
            }
            -147 => {
                reg_magic.set(MAGIC_ON);
                skipchr_keepstart();
                curchr.set(-1 as ::core::ffi::c_int);
            }
            -179 => {
                reg_magic.set(MAGIC_OFF);
                skipchr_keepstart();
                curchr.set(-1 as ::core::ffi::c_int);
            }
            -170 => {
                reg_magic.set(MAGIC_NONE);
                skipchr_keepstart();
                curchr.set(-1 as ::core::ffi::c_int);
            }
            _ => {
                if nfa_regpiece() == FAIL {
                    return FAIL;
                }
                if first as ::core::ffi::c_int == false_0 {
                    if post_ptr.get() >= post_end.get() {
                        realloc_post_list();
                    }
                    let c2rust_fresh28 = post_ptr.get();
                    post_ptr.set((*post_ptr.ptr()).offset(1));
                    *c2rust_fresh28 = NFA_CONCAT as ::core::ffi::c_int;
                } else {
                    first = false_0 != 0;
                }
            }
        }
    }
    return OK;
}
pub(crate) unsafe extern "C" fn nfa_regbranch() -> ::core::ffi::c_int {
    let mut old_post_pos: ::core::ffi::c_int = 0;
    old_post_pos = (*post_ptr.ptr()).offset_from(post_start.get()) as ::core::ffi::c_int;
    if nfa_regconcat() == FAIL {
        return FAIL;
    }
    while peekchr() == '&' as ::core::ffi::c_int - 256 as ::core::ffi::c_int {
        skipchr();
        if old_post_pos == (*post_ptr.ptr()).offset_from(post_start.get()) as ::core::ffi::c_int {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh22 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh22 = NFA_EMPTY as ::core::ffi::c_int;
        }
        if post_ptr.get() >= post_end.get() {
            realloc_post_list();
        }
        let c2rust_fresh23 = post_ptr.get();
        post_ptr.set((*post_ptr.ptr()).offset(1));
        *c2rust_fresh23 = NFA_NOPEN as ::core::ffi::c_int;
        if post_ptr.get() >= post_end.get() {
            realloc_post_list();
        }
        let c2rust_fresh24 = post_ptr.get();
        post_ptr.set((*post_ptr.ptr()).offset(1));
        *c2rust_fresh24 = NFA_PREV_ATOM_NO_WIDTH as ::core::ffi::c_int;
        old_post_pos = (*post_ptr.ptr()).offset_from(post_start.get()) as ::core::ffi::c_int;
        if nfa_regconcat() == FAIL {
            return FAIL;
        }
        if old_post_pos == (*post_ptr.ptr()).offset_from(post_start.get()) as ::core::ffi::c_int {
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh25 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh25 = NFA_EMPTY as ::core::ffi::c_int;
        }
        if post_ptr.get() >= post_end.get() {
            realloc_post_list();
        }
        let c2rust_fresh26 = post_ptr.get();
        post_ptr.set((*post_ptr.ptr()).offset(1));
        *c2rust_fresh26 = NFA_CONCAT as ::core::ffi::c_int;
    }
    if old_post_pos == (*post_ptr.ptr()).offset_from(post_start.get()) as ::core::ffi::c_int {
        if post_ptr.get() >= post_end.get() {
            realloc_post_list();
        }
        let c2rust_fresh27 = post_ptr.get();
        post_ptr.set((*post_ptr.ptr()).offset(1));
        *c2rust_fresh27 = NFA_EMPTY as ::core::ffi::c_int;
    }
    return OK;
}
pub(crate) unsafe extern "C" fn nfa_reg(mut paren: ::core::ffi::c_int) -> ::core::ffi::c_int {
    let mut parno: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if paren == REG_PAREN {
        if regnpar.get() >= NSUBEXP as ::core::ffi::c_int {
            emsg(gettext(
                b"E872: (NFA regexp) Too many '('\0".as_ptr() as *const ::core::ffi::c_char
            ));
            rc_did_emsg.set(true_0 != 0);
            return FAIL;
        }
        let c2rust_fresh17 = regnpar.get();
        regnpar.set(regnpar.get() + 1);
        parno = c2rust_fresh17;
    } else if paren == REG_ZPAREN {
        if regnzpar.get() >= NSUBEXP as ::core::ffi::c_int {
            emsg(gettext(
                b"E879: (NFA regexp) Too many \\z(\0".as_ptr() as *const ::core::ffi::c_char
            ));
            rc_did_emsg.set(true_0 != 0);
            return FAIL;
        }
        let c2rust_fresh18 = regnzpar.get();
        regnzpar.set(regnzpar.get() + 1);
        parno = c2rust_fresh18;
    }
    if nfa_regbranch() == FAIL {
        return FAIL;
    }
    while peekchr() == '|' as ::core::ffi::c_int - 256 as ::core::ffi::c_int {
        skipchr();
        if nfa_regbranch() == FAIL {
            return FAIL;
        }
        if post_ptr.get() >= post_end.get() {
            realloc_post_list();
        }
        let c2rust_fresh19 = post_ptr.get();
        post_ptr.set((*post_ptr.ptr()).offset(1));
        *c2rust_fresh19 = NFA_OR as ::core::ffi::c_int;
    }
    if paren != REG_NOPAREN && getchr() != ')' as ::core::ffi::c_int - 256 as ::core::ffi::c_int {
        if paren == REG_NPAREN {
            semsg(
                gettext(E_UNMATCHEDPP.as_ptr()),
                if reg_magic.get() as ::core::ffi::c_uint
                    == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            rc_did_emsg.set(true_0 != 0);
            return FAIL;
        } else {
            semsg(
                gettext(E_UNMATCHEDP.as_ptr()),
                if reg_magic.get() as ::core::ffi::c_uint
                    == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            rc_did_emsg.set(true_0 != 0);
            return FAIL;
        }
    } else if paren == REG_NOPAREN && peekchr() != NUL {
        if peekchr() == ')' as ::core::ffi::c_int - 256 as ::core::ffi::c_int {
            semsg(
                gettext(E_UNMATCHEDPAR.as_ptr()),
                if reg_magic.get() as ::core::ffi::c_uint
                    == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            rc_did_emsg.set(true_0 != 0);
            return FAIL;
        } else {
            emsg(gettext(
                b"E873: (NFA regexp) proper termination error\0".as_ptr()
                    as *const ::core::ffi::c_char,
            ));
            rc_did_emsg.set(true_0 != 0);
            return FAIL;
        }
    }
    if paren == REG_PAREN {
        (*had_endbrace.ptr())[parno as usize] = true_0 as uint8_t;
        if post_ptr.get() >= post_end.get() {
            realloc_post_list();
        }
        let c2rust_fresh20 = post_ptr.get();
        post_ptr.set((*post_ptr.ptr()).offset(1));
        *c2rust_fresh20 = NFA_MOPEN as ::core::ffi::c_int + parno;
    } else if paren == REG_ZPAREN {
        if post_ptr.get() >= post_end.get() {
            realloc_post_list();
        }
        let c2rust_fresh21 = post_ptr.get();
        post_ptr.set((*post_ptr.ptr()).offset(1));
        *c2rust_fresh21 = NFA_ZOPEN as ::core::ffi::c_int + parno;
    }
    return OK;
}
pub(crate) unsafe extern "C" fn re2post() -> *mut ::core::ffi::c_int {
    if nfa_reg(REG_NOPAREN) == FAIL {
        return ::core::ptr::null_mut::<::core::ffi::c_int>();
    }
    if post_ptr.get() >= post_end.get() {
        realloc_post_list();
    }
    let c2rust_fresh16 = post_ptr.get();
    post_ptr.set((*post_ptr.ptr()).offset(1));
    *c2rust_fresh16 = NFA_MOPEN as ::core::ffi::c_int;
    return post_start.get();
}
