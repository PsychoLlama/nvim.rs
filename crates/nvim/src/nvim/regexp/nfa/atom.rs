//! One atom of a pattern, in postfix form.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn nfa_regatom() -> ::core::ffi::c_int {
    let mut c: ::core::ffi::c_int = 0;
    let mut charclass: ::core::ffi::c_int = 0;
    let mut equiclass: ::core::ffi::c_int = 0;
    let mut collclass: ::core::ffi::c_int = 0;
    let mut got_coll_char: ::core::ffi::c_int = 0;
    let mut p: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut endp: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut old_regparse: *mut uint8_t = regparse.get() as *mut uint8_t;
    let mut extra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut emit_range: ::core::ffi::c_int = 0;
    let mut negated: ::core::ffi::c_int = 0;
    let mut startc: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
    let mut save_prev_at_start: ::core::ffi::c_int = prev_at_start.get();
    c = getchr();
    's_3798: {
        let mut plen_0: ::core::ffi::c_int = 0;
        '_nfa_do_multibyte: {
            's_3797: {
                's_3637: {
                    '_collection: {
                        match c {
                            NUL => {
                                emsg(gettext(E_NUL_FOUND.as_ptr()));
                                rc_did_emsg.set(true_0 != 0);
                                return FAIL;
                            }
                            -162 => {
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh42 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh42 = NFA_BOL as ::core::ffi::c_int;
                                break 's_3797;
                            }
                            -220 => {
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh43 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh43 = NFA_EOL as ::core::ffi::c_int;
                                had_eol.set(true_0);
                                break 's_3797;
                            }
                            -196 => {
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh44 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh44 = NFA_BOW as ::core::ffi::c_int;
                                break 's_3797;
                            }
                            -194 => {
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh45 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh45 = NFA_EOW as ::core::ffi::c_int;
                                break 's_3797;
                            }
                            -161 => {
                                c = unmagic(getchr());
                                if c == NUL {
                                    emsg(gettext(E_NUL_FOUND.as_ptr()));
                                    rc_did_emsg.set(true_0 != 0);
                                    return FAIL;
                                }
                                if c == '^' as ::core::ffi::c_int {
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh46 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh46 = NFA_BOL as ::core::ffi::c_int;
                                    break 's_3797;
                                } else if c == '$' as ::core::ffi::c_int {
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh47 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh47 = NFA_EOL as ::core::ffi::c_int;
                                    had_eol.set(true_0);
                                    break 's_3797;
                                } else {
                                    extra = NFA_ADD_NL;
                                    if c == '[' as ::core::ffi::c_int {
                                        break '_collection;
                                    }
                                }
                            }
                            -210 | -151 | -183 | -149 | -181 | -154 | -186 | -144 | -176 | -141
                            | -173 | -156 | -188 | -136 | -168 | -145 | -177 | -137 | -169
                            | -152 | -184 | -159 | -191 | -148 | -180 | -139 | -171 => {}
                            -146 => {
                                if reg_string.get() != 0 {
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh51 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh51 = '\n' as ::core::ffi::c_int;
                                } else {
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh52 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh52 = NFA_NEWL as ::core::ffi::c_int;
                                    (*regflags.ptr()) |= RF_HASNL as ::core::ffi::c_uint;
                                }
                                break 's_3797;
                            }
                            -216 => {
                                if nfa_reg(REG_PAREN) == FAIL {
                                    return FAIL;
                                }
                                break 's_3797;
                            }
                            -132 | -218 | -215 => {
                                semsg(
                                    gettext(E_MISPLACED.as_ptr()),
                                    unmagic(c) as ::core::ffi::c_char as ::core::ffi::c_int,
                                );
                                return FAIL;
                            }
                            -195 | -193 | -213 | -192 | -214 | -133 => {
                                semsg(
                                    gettext(E_MISPLACED.as_ptr()),
                                    unmagic(c) as ::core::ffi::c_char as ::core::ffi::c_int,
                                );
                                return FAIL;
                            }
                            -130 => {
                                let mut lp: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
                                if (*reg_prev_sub.ptr()).is_null() {
                                    emsg(gettext(
                                        &raw const e_nopresub as *const ::core::ffi::c_char,
                                    ));
                                    return FAIL;
                                }
                                lp = reg_prev_sub.get() as *mut uint8_t;
                                while *lp as ::core::ffi::c_int != NUL {
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh53 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh53 = utf_ptr2char(lp as *mut ::core::ffi::c_char);
                                    if lp != reg_prev_sub.get() as *mut uint8_t {
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh54 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh54 = NFA_CONCAT as ::core::ffi::c_int;
                                    }
                                    lp = lp.offset(
                                        utf_ptr2len(lp as *mut ::core::ffi::c_char) as isize
                                    );
                                }
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh55 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh55 = NFA_NOPEN as ::core::ffi::c_int;
                                break 's_3797;
                            }
                            -207 | -206 | -205 | -204 | -203 | -202 | -201 | -200 | -199 => {
                                let mut refnum: ::core::ffi::c_int =
                                    unmagic(c) - '1' as ::core::ffi::c_int;
                                if seen_endbrace(refnum + 1 as ::core::ffi::c_int) == 0 {
                                    return FAIL;
                                }
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh56 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh56 = NFA_BACKREF1 as ::core::ffi::c_int + refnum;
                                (*rex.ptr()).nfa_has_backref = true_0;
                                break 's_3797;
                            }
                            -134 => {
                                c = unmagic(getchr());
                                match c {
                                    115 => {
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh57 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh57 = NFA_ZSTART as ::core::ffi::c_int;
                                        if !re_mult_next("\\zs") {
                                            return false_0;
                                        }
                                    }
                                    101 => {
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh58 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh58 = NFA_ZEND as ::core::ffi::c_int;
                                        (*rex.ptr()).nfa_has_zend = true_0;
                                        if !re_mult_next("\\ze") {
                                            return false_0;
                                        }
                                    }
                                    49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                                        if reg_do_extmatch.get() & REX_USE
                                            == 0 as ::core::ffi::c_int
                                        {
                                            emsg(gettext(E_Z1_NOT_ALLOWED.as_ptr()));
                                            rc_did_emsg.set(true_0 != 0);
                                            return FAIL;
                                        }
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh59 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh59 = NFA_ZREF1 as ::core::ffi::c_int
                                            + (unmagic(c) - '1' as ::core::ffi::c_int);
                                        re_has_z.set(REX_USE);
                                    }
                                    40 => {
                                        if reg_do_extmatch.get() != REX_SET {
                                            emsg(gettext(E_Z_NOT_ALLOWED.as_ptr()));
                                            rc_did_emsg.set(true_0 != 0);
                                            return FAIL;
                                        }
                                        if nfa_reg(REG_ZPAREN) == FAIL {
                                            return FAIL;
                                        }
                                        re_has_z.set(REX_SET);
                                    }
                                    _ => {
                                        semsg(
                                            gettext(
                                                b"E867: (NFA) Unknown operator '\\z%c'\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            unmagic(c),
                                        );
                                        return FAIL;
                                    }
                                }
                                break 's_3797;
                            }
                            -219 => {
                                c = unmagic(getchr());
                                match c {
                                    40 => {
                                        if nfa_reg(REG_NPAREN) == FAIL {
                                            return FAIL;
                                        }
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh60 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh60 = NFA_NOPEN as ::core::ffi::c_int;
                                    }
                                    100 | 111 | 120 | 117 | 85 => {
                                        let mut nr: int64_t = 0;
                                        match c {
                                            100 => {
                                                nr = getdecchrs();
                                            }
                                            111 => {
                                                nr = getoctchrs();
                                            }
                                            120 => {
                                                nr = gethexchrs(2 as ::core::ffi::c_int);
                                            }
                                            117 => {
                                                nr = gethexchrs(4 as ::core::ffi::c_int);
                                            }
                                            85 => {
                                                nr = gethexchrs(8 as ::core::ffi::c_int);
                                            }
                                            _ => {
                                                nr = -1 as int64_t;
                                            }
                                        }
                                        if nr < 0 as int64_t || nr > INT_MAX as int64_t {
                                            semsg(
                                                gettext(
                                                    b"E678: Invalid character after %s%%[dxouU]\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                if reg_magic.get() as ::core::ffi::c_uint
                                                    == MAGIC_ALL as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                                } else {
                                                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                                                },
                                            );
                                            rc_did_emsg.set(true_0 != 0);
                                            return FAIL;
                                        }
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh61 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh61 = if nr == 0 as int64_t {
                                            0xa as ::core::ffi::c_int
                                        } else {
                                            nr as ::core::ffi::c_int
                                        };
                                    }
                                    94 => {
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh62 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh62 = NFA_BOF as ::core::ffi::c_int;
                                    }
                                    36 => {
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh63 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh63 = NFA_EOF as ::core::ffi::c_int;
                                    }
                                    35 => {
                                        if *(*regparse.ptr())
                                            .offset(0 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '=' as ::core::ffi::c_int
                                            && *(*regparse.ptr())
                                                .offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                >= 48 as ::core::ffi::c_int
                                            && *(*regparse.ptr())
                                                .offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                <= 50 as ::core::ffi::c_int
                                        {
                                            semsg(
                                                gettext(
                                                    E_ATOM_ENGINE_MUST_BE_AT_START_OF_PATTERN
                                                        .as_ptr(),
                                                ),
                                                *(*regparse.ptr())
                                                    .offset(1 as ::core::ffi::c_int as isize)
                                                    as ::core::ffi::c_int,
                                            );
                                            return FAIL;
                                        }
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh64 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh64 = NFA_CURSOR as ::core::ffi::c_int;
                                    }
                                    86 => {
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh65 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh65 = NFA_VISUAL as ::core::ffi::c_int;
                                    }
                                    67 => {
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh66 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh66 = NFA_ANY_COMPOSING as ::core::ffi::c_int;
                                    }
                                    91 => {
                                        let mut n: ::core::ffi::c_int = 0;
                                        n = 0 as ::core::ffi::c_int;
                                        loop {
                                            c = peekchr();
                                            if c == ']' as ::core::ffi::c_int {
                                                break;
                                            }
                                            if c == NUL {
                                                semsg(
                                                    gettext(E_MISSING_SB.as_ptr()),
                                                    if reg_magic.get() as ::core::ffi::c_uint
                                                        == MAGIC_ALL as ::core::ffi::c_int
                                                            as ::core::ffi::c_uint
                                                    {
                                                        b"\0".as_ptr() as *const ::core::ffi::c_char
                                                    } else {
                                                        b"\\\0".as_ptr()
                                                            as *const ::core::ffi::c_char
                                                    },
                                                );
                                                rc_did_emsg.set(true_0 != 0);
                                                return FAIL;
                                            }
                                            if nfa_regatom() == FAIL {
                                                return FAIL;
                                            }
                                            n += 1;
                                        }
                                        getchr();
                                        if n == 0 as ::core::ffi::c_int {
                                            semsg(
                                                gettext(E_EMPTY_SB.as_ptr()),
                                                if reg_magic.get() as ::core::ffi::c_uint
                                                    == MAGIC_ALL as ::core::ffi::c_int
                                                        as ::core::ffi::c_uint
                                                {
                                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                                } else {
                                                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                                                },
                                            );
                                            rc_did_emsg.set(true_0 != 0);
                                            return FAIL;
                                        }
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh67 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh67 = NFA_OPT_CHARS as ::core::ffi::c_int;
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh68 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh68 = n;
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh69 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh69 = NFA_NOPEN as ::core::ffi::c_int;
                                    }
                                    _ => {
                                        let mut n_0: int64_t = 0 as int64_t;
                                        let cmp: ::core::ffi::c_int = c;
                                        let mut cur: bool = false_0 != 0;
                                        let mut got_digit: bool = false_0 != 0;
                                        if c == '<' as ::core::ffi::c_int
                                            || c == '>' as ::core::ffi::c_int
                                        {
                                            c = getchr();
                                        }
                                        if unmagic(c) == '.' as ::core::ffi::c_int {
                                            cur = true_0 != 0;
                                            c = getchr();
                                        }
                                        while ascii_isdigit(c) {
                                            if cur {
                                                semsg(
                                                    gettext(
                                                        E_REGEXP_NUMBER_AFTER_DOT_POS_SEARCH_CHR
                                                            .as_ptr(),
                                                    ),
                                                    unmagic(c),
                                                );
                                                return FAIL;
                                            }
                                            if n_0
                                                > ((INT32_MAX - (c - '0' as ::core::ffi::c_int))
                                                    / 10 as ::core::ffi::c_int)
                                                    as int64_t
                                            {
                                                emsg(gettext(E_VALUE_TOO_LARGE.as_ptr()));
                                                return FAIL;
                                            }
                                            n_0 = n_0 * 10 as int64_t
                                                + (c - '0' as ::core::ffi::c_int) as int64_t;
                                            c = getchr();
                                            got_digit = true_0 != 0;
                                        }
                                        if c == 'l' as ::core::ffi::c_int
                                            || c == 'c' as ::core::ffi::c_int
                                            || c == 'v' as ::core::ffi::c_int
                                        {
                                            let mut limit: int32_t = INT32_MAX as int32_t;
                                            if !cur && !got_digit {
                                                semsg(
                                                    gettext(
                                                        E_NFA_REGEXP_MISSING_VALUE_IN_CHR.as_ptr(),
                                                    ),
                                                    unmagic(c),
                                                );
                                                return FAIL;
                                            }
                                            if c == 'l' as ::core::ffi::c_int {
                                                if cur {
                                                    n_0 = (*curwin.get()).w_cursor.lnum as int64_t;
                                                }
                                                if post_ptr.get() >= post_end.get() {
                                                    realloc_post_list();
                                                }
                                                let c2rust_fresh70 = post_ptr.get();
                                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                                *c2rust_fresh70 =
                                                    if cmp == '<' as ::core::ffi::c_int {
                                                        NFA_LNUM_LT as ::core::ffi::c_int
                                                    } else if cmp == '>' as ::core::ffi::c_int {
                                                        NFA_LNUM_GT as ::core::ffi::c_int
                                                    } else {
                                                        NFA_LNUM as ::core::ffi::c_int
                                                    };
                                                if save_prev_at_start != 0 {
                                                    at_start.set(true_0);
                                                }
                                            } else if c == 'c' as ::core::ffi::c_int {
                                                if cur {
                                                    n_0 = (*curwin.get()).w_cursor.col as int64_t;
                                                    n_0 += 1;
                                                }
                                                if post_ptr.get() >= post_end.get() {
                                                    realloc_post_list();
                                                }
                                                let c2rust_fresh71 = post_ptr.get();
                                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                                *c2rust_fresh71 =
                                                    if cmp == '<' as ::core::ffi::c_int {
                                                        NFA_COL_LT as ::core::ffi::c_int
                                                    } else if cmp == '>' as ::core::ffi::c_int {
                                                        NFA_COL_GT as ::core::ffi::c_int
                                                    } else {
                                                        NFA_COL as ::core::ffi::c_int
                                                    };
                                            } else {
                                                if cur {
                                                    let mut vcol: colnr_T = 0 as colnr_T;
                                                    getvvcol(
                                                        curwin.get(),
                                                        &raw mut (*curwin.get()).w_cursor,
                                                        ::core::ptr::null_mut::<colnr_T>(),
                                                        ::core::ptr::null_mut::<colnr_T>(),
                                                        &raw mut vcol,
                                                    );
                                                    vcol += 1;
                                                    n_0 = vcol as int64_t;
                                                }
                                                if post_ptr.get() >= post_end.get() {
                                                    realloc_post_list();
                                                }
                                                let c2rust_fresh72 = post_ptr.get();
                                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                                *c2rust_fresh72 =
                                                    if cmp == '<' as ::core::ffi::c_int {
                                                        NFA_VCOL_LT as ::core::ffi::c_int
                                                    } else if cmp == '>' as ::core::ffi::c_int {
                                                        NFA_VCOL_GT as ::core::ffi::c_int
                                                    } else {
                                                        NFA_VCOL as ::core::ffi::c_int
                                                    };
                                                limit = (INT32_MAX
                                                    / MB_MAXBYTES as ::core::ffi::c_int)
                                                    as int32_t;
                                            }
                                            if n_0 >= limit as int64_t {
                                                emsg(gettext(E_VALUE_TOO_LARGE.as_ptr()));
                                                return FAIL;
                                            }
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh73 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh73 = n_0 as ::core::ffi::c_int;
                                        } else if unmagic(c) == '\'' as ::core::ffi::c_int
                                            && n_0 == 0 as int64_t
                                        {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh74 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh74 = if cmp == '<' as ::core::ffi::c_int {
                                                NFA_MARK_LT as ::core::ffi::c_int
                                            } else if cmp == '>' as ::core::ffi::c_int {
                                                NFA_MARK_GT as ::core::ffi::c_int
                                            } else {
                                                NFA_MARK as ::core::ffi::c_int
                                            };
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh75 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh75 = getchr();
                                        } else {
                                            semsg(
                                                gettext(
                                                    b"E867: (NFA) Unknown operator '\\%%%c'\0"
                                                        .as_ptr()
                                                        as *const ::core::ffi::c_char,
                                                ),
                                                unmagic(c),
                                            );
                                            return FAIL;
                                        }
                                    }
                                }
                                break 's_3797;
                            }
                            -165 => {
                                break '_collection;
                            }
                            _ => {
                                break 's_3637;
                            }
                        }
                        p = vim_strchr(classchars.get() as *mut ::core::ffi::c_char, unmagic(c))
                            as *mut uint8_t;
                        if p.is_null() {
                            if extra == NFA_ADD_NL {
                                semsg(gettext(E_ILL_CHAR_CLASS.as_ptr()), c as int64_t);
                                rc_did_emsg.set(true_0 != 0);
                                return FAIL;
                            }
                            siemsg(
                                b"INTERNAL: Unknown character class char: %d\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                c,
                            );
                            return FAIL;
                        }
                        if c == '.' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
                            && utf_iscomposing_legacy(peekchr()) as ::core::ffi::c_int != 0
                        {
                            old_regparse = regparse.get() as *mut uint8_t;
                            c = getchr();
                            break '_nfa_do_multibyte;
                        } else {
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh48 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh48 =
                                (*nfa_classcodes.ptr())[p.offset_from(classchars.get()) as usize];
                            if extra == NFA_ADD_NL {
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh49 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh49 = NFA_NEWL as ::core::ffi::c_int;
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh50 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh50 = NFA_OR as ::core::ffi::c_int;
                                (*regflags.ptr()) |= RF_HASNL as ::core::ffi::c_uint;
                            }
                            break 's_3797;
                        }
                    }
                    p = regparse.get() as *mut uint8_t;
                    endp = skip_anyof(p as *mut ::core::ffi::c_char) as *mut uint8_t;
                    if *endp as ::core::ffi::c_int == ']' as ::core::ffi::c_int {
                        let mut range_endpoint: bool = false;
                        let mut result: ::core::ffi::c_int = nfa_recognize_char_class(
                            regparse.get() as *mut uint8_t,
                            endp,
                            (extra == NFA_ADD_NL) as ::core::ffi::c_int,
                        );
                        if result != FAIL {
                            if result >= NFA_FIRST_NL as ::core::ffi::c_int
                                && result <= NFA_LAST_NL as ::core::ffi::c_int
                            {
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh76 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh76 = result - 31 as ::core::ffi::c_int;
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh77 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh77 = NFA_NEWL as ::core::ffi::c_int;
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh78 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh78 = NFA_OR as ::core::ffi::c_int;
                            } else {
                                if post_ptr.get() >= post_end.get() {
                                    realloc_post_list();
                                }
                                let c2rust_fresh79 = post_ptr.get();
                                post_ptr.set((*post_ptr.ptr()).offset(1));
                                *c2rust_fresh79 = result;
                            }
                            regparse.set(endp as *mut ::core::ffi::c_char);
                            regparse.set(
                                (*regparse.ptr()).offset(utfc_ptr2len(regparse.get()) as isize),
                            );
                            return OK;
                        }
                        negated = false_0;
                        if *regparse.get() as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
                            negated = true_0;
                            regparse.set(
                                (*regparse.ptr()).offset(utfc_ptr2len(regparse.get()) as isize),
                            );
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh80 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh80 = NFA_START_NEG_COLL as ::core::ffi::c_int;
                        } else {
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh81 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh81 = NFA_START_COLL as ::core::ffi::c_int;
                        }
                        if *regparse.get() as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                            startc = '-' as ::core::ffi::c_int;
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh82 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh82 = startc;
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh83 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh83 = NFA_CONCAT as ::core::ffi::c_int;
                            regparse.set(
                                (*regparse.ptr()).offset(utfc_ptr2len(regparse.get()) as isize),
                            );
                        }
                        emit_range = false_0;
                        while (regparse.get() as *mut uint8_t) < endp {
                            let mut oldstartc: ::core::ffi::c_int = startc;
                            range_endpoint = false_0 != 0;
                            startc = -1 as ::core::ffi::c_int;
                            got_coll_char = false_0;
                            if *regparse.get() as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
                                collclass = 0 as ::core::ffi::c_int;
                                equiclass = collclass;
                                charclass = take_char_class(&mut *regparse.ptr());
                                if charclass == CLASS_NONE as ::core::ffi::c_int {
                                    equiclass = take_bracketed(&mut *regparse.ptr(), b'=');
                                    if equiclass == 0 as ::core::ffi::c_int {
                                        collclass = take_bracketed(&mut *regparse.ptr(), b'.');
                                    }
                                }
                                if charclass != CLASS_NONE as ::core::ffi::c_int {
                                    match charclass {
                                        0 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh84 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh84 = NFA_CLASS_ALNUM as ::core::ffi::c_int;
                                        }
                                        1 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh85 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh85 = NFA_CLASS_ALPHA as ::core::ffi::c_int;
                                        }
                                        2 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh86 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh86 = NFA_CLASS_BLANK as ::core::ffi::c_int;
                                        }
                                        3 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh87 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh87 = NFA_CLASS_CNTRL as ::core::ffi::c_int;
                                        }
                                        4 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh88 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh88 = NFA_CLASS_DIGIT as ::core::ffi::c_int;
                                        }
                                        5 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh89 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh89 = NFA_CLASS_GRAPH as ::core::ffi::c_int;
                                        }
                                        6 => {
                                            wants_nfa.set(true_0 != 0);
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh90 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh90 = NFA_CLASS_LOWER as ::core::ffi::c_int;
                                        }
                                        7 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh91 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh91 = NFA_CLASS_PRINT as ::core::ffi::c_int;
                                        }
                                        8 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh92 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh92 = NFA_CLASS_PUNCT as ::core::ffi::c_int;
                                        }
                                        9 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh93 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh93 = NFA_CLASS_SPACE as ::core::ffi::c_int;
                                        }
                                        10 => {
                                            wants_nfa.set(true_0 != 0);
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh94 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh94 = NFA_CLASS_UPPER as ::core::ffi::c_int;
                                        }
                                        11 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh95 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh95 =
                                                NFA_CLASS_XDIGIT as ::core::ffi::c_int;
                                        }
                                        12 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh96 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh96 = NFA_CLASS_TAB as ::core::ffi::c_int;
                                        }
                                        13 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh97 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh97 =
                                                NFA_CLASS_RETURN as ::core::ffi::c_int;
                                        }
                                        14 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh98 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh98 =
                                                NFA_CLASS_BACKSPACE as ::core::ffi::c_int;
                                        }
                                        15 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh99 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh99 =
                                                NFA_CLASS_ESCAPE as ::core::ffi::c_int;
                                        }
                                        16 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh100 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh100 =
                                                NFA_CLASS_IDENT as ::core::ffi::c_int;
                                        }
                                        17 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh101 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh101 =
                                                NFA_CLASS_KEYWORD as ::core::ffi::c_int;
                                        }
                                        18 => {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh102 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh102 =
                                                NFA_CLASS_FNAME as ::core::ffi::c_int;
                                        }
                                        _ => {}
                                    }
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh103 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh103 = NFA_CONCAT as ::core::ffi::c_int;
                                    continue;
                                } else if equiclass != 0 as ::core::ffi::c_int {
                                    nfa_emit_equi_class(equiclass);
                                    continue;
                                } else if collclass != 0 as ::core::ffi::c_int {
                                    startc = collclass;
                                }
                            }
                            if *regparse.get() as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                                && oldstartc != -1 as ::core::ffi::c_int
                            {
                                emit_range = true_0;
                                startc = oldstartc;
                                regparse.set(
                                    (*regparse.ptr()).offset(utfc_ptr2len(regparse.get()) as isize),
                                );
                            } else {
                                if *regparse.get() as ::core::ffi::c_int
                                    == '\\' as ::core::ffi::c_int
                                    && (regparse.get() as *mut uint8_t)
                                        .offset(1 as ::core::ffi::c_int as isize)
                                        <= endp
                                    && (!vim_strchr(
                                        REGEXP_INRANGE.as_ptr(),
                                        *(*regparse.ptr()).offset(1 as ::core::ffi::c_int as isize)
                                            as uint8_t
                                            as ::core::ffi::c_int,
                                    )
                                    .is_null()
                                        || reg_cpo_lit.get() == 0
                                            && !vim_strchr(
                                                REGEXP_ABBR.as_ptr(),
                                                *(*regparse.ptr())
                                                    .offset(1 as ::core::ffi::c_int as isize)
                                                    as uint8_t
                                                    as ::core::ffi::c_int,
                                            )
                                            .is_null())
                                {
                                    regparse.set(
                                        (*regparse.ptr())
                                            .offset(utfc_ptr2len(regparse.get()) as isize),
                                    );
                                    if *regparse.get() as ::core::ffi::c_int
                                        == 'n' as ::core::ffi::c_int
                                    {
                                        startc = if reg_string.get() != 0
                                            || emit_range != 0
                                            || *(*regparse.ptr())
                                                .offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int
                                                == '-' as ::core::ffi::c_int
                                        {
                                            NL
                                        } else {
                                            NFA_NEWL as ::core::ffi::c_int
                                        };
                                    } else if *regparse.get() as ::core::ffi::c_int
                                        == 'd' as ::core::ffi::c_int
                                        || *regparse.get() as ::core::ffi::c_int
                                            == 'o' as ::core::ffi::c_int
                                        || *regparse.get() as ::core::ffi::c_int
                                            == 'x' as ::core::ffi::c_int
                                        || *regparse.get() as ::core::ffi::c_int
                                            == 'u' as ::core::ffi::c_int
                                        || *regparse.get() as ::core::ffi::c_int
                                            == 'U' as ::core::ffi::c_int
                                    {
                                        startc = coll_get_char();
                                        if startc == INT_MAX {
                                            emsg(gettext(E_UNICODE_VAL_TOO_LARGE.as_ptr()));
                                            rc_did_emsg.set(true_0 != 0);
                                            return FAIL;
                                        }
                                        got_coll_char = true_0;
                                        regparse.set(
                                            (*regparse.ptr()).offset(
                                                -((utf_head_off(
                                                    old_regparse as *mut ::core::ffi::c_char,
                                                    (*regparse.ptr()).offset(
                                                        -(1 as ::core::ffi::c_int as isize),
                                                    ),
                                                ) + 1 as ::core::ffi::c_int)
                                                    as isize),
                                            ),
                                        );
                                    } else {
                                        startc =
                                            backslash_abbr(*regparse.get() as ::core::ffi::c_int);
                                    }
                                }
                                if startc == -1 as ::core::ffi::c_int {
                                    startc = utf_ptr2char(regparse.get());
                                }
                                if emit_range != 0 {
                                    let mut endc: ::core::ffi::c_int = startc;
                                    range_endpoint = true_0 != 0;
                                    startc = oldstartc;
                                    if startc > endc {
                                        emsg(gettext(E_REVERSE_RANGE.as_ptr()));
                                        rc_did_emsg.set(true_0 != 0);
                                        return FAIL;
                                    }
                                    if endc > startc + 2 as ::core::ffi::c_int {
                                        if startc == 0 as ::core::ffi::c_int {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh104 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh104 = 1 as ::core::ffi::c_int;
                                        } else {
                                            post_ptr.set((*post_ptr.ptr()).offset(-1));
                                        }
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh105 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh105 = endc;
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh106 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh106 = NFA_RANGE as ::core::ffi::c_int;
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh107 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh107 = NFA_CONCAT as ::core::ffi::c_int;
                                    } else if utf_char2len(startc) > 1 as ::core::ffi::c_int
                                        || utf_char2len(endc) > 1 as ::core::ffi::c_int
                                    {
                                        c = startc + 1 as ::core::ffi::c_int;
                                        while c <= endc {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh108 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh108 = c;
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh109 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh109 = NFA_CONCAT as ::core::ffi::c_int;
                                            c += 1;
                                        }
                                    } else {
                                        c = startc + 1 as ::core::ffi::c_int;
                                        while c <= endc {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh110 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh110 = c;
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh111 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh111 = NFA_CONCAT as ::core::ffi::c_int;
                                            c += 1;
                                        }
                                    }
                                    emit_range = false_0;
                                    startc = -1 as ::core::ffi::c_int;
                                } else if startc == NFA_NEWL as ::core::ffi::c_int {
                                    if negated == 0 {
                                        extra = NFA_ADD_NL;
                                    }
                                } else if got_coll_char == true_0
                                    && startc == 0 as ::core::ffi::c_int
                                {
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh112 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh112 = 0xa as ::core::ffi::c_int;
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh113 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh113 = NFA_CONCAT as ::core::ffi::c_int;
                                } else {
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh114 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh114 = startc;
                                    if utf_ptr2len(regparse.get()) == utfc_ptr2len(regparse.get()) {
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh115 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh115 = NFA_CONCAT as ::core::ffi::c_int;
                                    }
                                }
                                let mut plen: ::core::ffi::c_int = 0;
                                if !range_endpoint && {
                                    plen = utfc_ptr2len(regparse.get());
                                    utf_ptr2len(regparse.get()) != plen
                                } {
                                    let mut i: ::core::ffi::c_int = utf_ptr2len(regparse.get());
                                    c = utf_ptr2char((*regparse.ptr()).offset(i as isize));
                                    loop {
                                        if c == 0 as ::core::ffi::c_int {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh116 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh116 = 1 as ::core::ffi::c_int;
                                        } else {
                                            if post_ptr.get() >= post_end.get() {
                                                realloc_post_list();
                                            }
                                            let c2rust_fresh117 = post_ptr.get();
                                            post_ptr.set((*post_ptr.ptr()).offset(1));
                                            *c2rust_fresh117 = c;
                                        }
                                        if post_ptr.get() >= post_end.get() {
                                            realloc_post_list();
                                        }
                                        let c2rust_fresh118 = post_ptr.get();
                                        post_ptr.set((*post_ptr.ptr()).offset(1));
                                        *c2rust_fresh118 = NFA_CONCAT as ::core::ffi::c_int;
                                        i += utf_char2len(c);
                                        if i >= plen {
                                            break;
                                        }
                                        c = utf_ptr2char((*regparse.ptr()).offset(i as isize));
                                    }
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh119 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh119 = NFA_COMPOSING as ::core::ffi::c_int;
                                    if post_ptr.get() >= post_end.get() {
                                        realloc_post_list();
                                    }
                                    let c2rust_fresh120 = post_ptr.get();
                                    post_ptr.set((*post_ptr.ptr()).offset(1));
                                    *c2rust_fresh120 = NFA_CONCAT as ::core::ffi::c_int;
                                }
                                regparse.set(
                                    (*regparse.ptr()).offset(utfc_ptr2len(regparse.get()) as isize),
                                );
                            }
                        }
                        regparse.set((*regparse.ptr()).offset(
                            -((utf_head_off(
                                old_regparse as *mut ::core::ffi::c_char,
                                (*regparse.ptr()).offset(-(1 as ::core::ffi::c_int as isize)),
                            ) + 1 as ::core::ffi::c_int) as isize),
                        ));
                        if *regparse.get() as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh121 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh121 = '-' as ::core::ffi::c_int;
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh122 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh122 = NFA_CONCAT as ::core::ffi::c_int;
                        }
                        regparse.set(endp as *mut ::core::ffi::c_char);
                        regparse
                            .set((*regparse.ptr()).offset(utfc_ptr2len(regparse.get()) as isize));
                        if negated == true_0 {
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh123 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh123 = NFA_END_NEG_COLL as ::core::ffi::c_int;
                        } else {
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh124 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh124 = NFA_END_COLL as ::core::ffi::c_int;
                        }
                        if extra == NFA_ADD_NL {
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh125 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh125 = if reg_string.get() != 0 {
                                '\n' as ::core::ffi::c_int
                            } else {
                                NFA_NEWL as ::core::ffi::c_int
                            };
                            if post_ptr.get() >= post_end.get() {
                                realloc_post_list();
                            }
                            let c2rust_fresh126 = post_ptr.get();
                            post_ptr.set((*post_ptr.ptr()).offset(1));
                            *c2rust_fresh126 = NFA_OR as ::core::ffi::c_int;
                        }
                        return OK;
                    }
                    if reg_strict.get() != 0 {
                        emsg(gettext(E_MISSINGBRACKET.as_ptr()));
                        rc_did_emsg.set(true_0 != 0);
                        return FAIL;
                    }
                }
                plen_0 = 0;
                break '_nfa_do_multibyte;
            }
            break 's_3798;
        }
        plen_0 = utfc_ptr2len(old_regparse as *mut ::core::ffi::c_char);
        if utf_char2len(c) != plen_0 || utf_iscomposing_legacy(c) as ::core::ffi::c_int != 0 {
            let mut i_0: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            loop {
                if post_ptr.get() >= post_end.get() {
                    realloc_post_list();
                }
                let c2rust_fresh127 = post_ptr.get();
                post_ptr.set((*post_ptr.ptr()).offset(1));
                *c2rust_fresh127 = c;
                if i_0 > 0 as ::core::ffi::c_int {
                    if post_ptr.get() >= post_end.get() {
                        realloc_post_list();
                    }
                    let c2rust_fresh128 = post_ptr.get();
                    post_ptr.set((*post_ptr.ptr()).offset(1));
                    *c2rust_fresh128 = NFA_CONCAT as ::core::ffi::c_int;
                }
                i_0 += utf_char2len(c);
                if i_0 >= plen_0 {
                    break;
                }
                c = utf_ptr2char((old_regparse as *mut ::core::ffi::c_char).offset(i_0 as isize));
            }
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh129 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh129 = NFA_COMPOSING as ::core::ffi::c_int;
            regparse.set((old_regparse as *mut ::core::ffi::c_char).offset(plen_0 as isize));
        } else {
            c = unmagic(c);
            if post_ptr.get() >= post_end.get() {
                realloc_post_list();
            }
            let c2rust_fresh130 = post_ptr.get();
            post_ptr.set((*post_ptr.ptr()).offset(1));
            *c2rust_fresh130 = c;
        }
        return OK;
    }
    return OK;
}
