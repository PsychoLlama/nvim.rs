//! An atom with its multi, the branches around it, and the program
//! the whole pattern compiles to.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn regpiece(mut flagp: *mut ::core::ffi::c_int) -> *mut uint8_t {
    let mut ret: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut op: ::core::ffi::c_int = 0;
    let mut next: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut flags: ::core::ffi::c_int = 0;
    let mut minval: ::core::ffi::c_int = 0;
    let mut maxval: ::core::ffi::c_int = 0;
    ret = regatom(&raw mut flags);
    if ret.is_null() {
        return ::core::ptr::null_mut::<uint8_t>();
    }
    op = peekchr();
    if re_multi_type(op) == NOT_MULTI {
        *flagp = flags;
        return ret;
    }
    *flagp = WORST | SPSTART | flags & (HASNL | HASLOOKBH);
    skipchr();
    match op {
        -214 => {
            if flags & SIMPLE != 0 {
                reginsert(STAR, ret);
            } else {
                reginsert(BRANCH, ret);
                regoptail(ret, regnode(BACK));
                regoptail(ret, ret);
                regtail(ret, regnode(BRANCH));
                regtail(ret, regnode(NOTHING));
            }
        }
        -213 => {
            if flags & SIMPLE != 0 {
                reginsert(PLUS, ret);
            } else {
                next = regnode(BRANCH);
                regtail(ret, next);
                regtail(regnode(BACK), ret);
                regtail(next, regnode(BRANCH));
                regtail(ret, regnode(NOTHING));
            }
            *flagp = WORST | HASWIDTH | flags & (HASNL | HASLOOKBH);
        }
        -192 => {
            let mut lop: ::core::ffi::c_int = END;
            let mut nr: int64_t = getdecchrs();
            match no_Magic(getchr()) {
                61 => {
                    lop = MATCH;
                }
                33 => {
                    lop = NOMATCH;
                }
                62 => {
                    lop = SUBPAT;
                }
                60 => match no_Magic(getchr()) {
                    61 => {
                        lop = BEHIND;
                    }
                    33 => {
                        lop = NOBEHIND;
                    }
                    _ => {}
                },
                _ => {}
            }
            if lop == END {
                semsg(
                    gettext(
                        (e_invalid_character_after_str_at.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    if reg_magic.get() as ::core::ffi::c_uint
                        == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        b"\0".as_ptr() as *const ::core::ffi::c_char
                    } else {
                        b"\\\0".as_ptr() as *const ::core::ffi::c_char
                    },
                );
                rc_did_emsg.set(true_0 != 0);
                return NULL_0 as *mut uint8_t;
            }
            if lop == BEHIND || lop == NOBEHIND {
                regtail(ret, regnode(BHPOS));
                *flagp |= HASLOOKBH;
            }
            regtail(ret, regnode(END));
            if lop == BEHIND || lop == NOBEHIND {
                if nr < 0 as int64_t {
                    nr = 0 as int64_t;
                }
                reginsert_nr(lop, nr as uint32_t as int64_t, ret);
            } else {
                reginsert(lop, ret);
            }
        }
        -193 | -195 => {
            reginsert(BRANCH, ret);
            regtail(ret, regnode(BRANCH));
            next = regnode(NOTHING);
            regtail(ret, next);
            regoptail(ret, next);
        }
        -133 => {
            if read_limits(&raw mut minval, &raw mut maxval) == 0 {
                return ::core::ptr::null_mut::<uint8_t>();
            }
            if flags & SIMPLE != 0 {
                reginsert(BRACE_SIMPLE, ret);
                reginsert_limits(BRACE_LIMITS, minval as int64_t, maxval as int64_t, ret);
            } else {
                if num_complex_braces.get() >= 10 as ::core::ffi::c_int {
                    semsg(
                        gettext(b"E60: Too many complex %s{...}s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        if reg_magic.get() as ::core::ffi::c_uint
                            == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            b"\0".as_ptr() as *const ::core::ffi::c_char
                        } else {
                            b"\\\0".as_ptr() as *const ::core::ffi::c_char
                        },
                    );
                    rc_did_emsg.set(true_0 != 0);
                    return NULL_0 as *mut uint8_t;
                }
                reginsert(BRACE_COMPLEX + num_complex_braces.get(), ret);
                regoptail(ret, regnode(BACK));
                regoptail(ret, ret);
                reginsert_limits(BRACE_LIMITS, minval as int64_t, maxval as int64_t, ret);
                (*num_complex_braces.ptr()) += 1;
            }
            if minval > 0 as ::core::ffi::c_int && maxval > 0 as ::core::ffi::c_int {
                *flagp = HASWIDTH | flags & (HASNL | HASLOOKBH);
            }
        }
        _ => {}
    }
    if re_multi_type(peekchr()) != NOT_MULTI {
        if peekchr() == '*' as ::core::ffi::c_int - 256 as ::core::ffi::c_int {
            semsg(
                gettext(b"E61: Nested %s*\0".as_ptr() as *const ::core::ffi::c_char),
                if reg_magic.get() as ::core::ffi::c_uint
                    >= MAGIC_ON as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            rc_did_emsg.set(true_0 != 0);
            return NULL_0 as *mut uint8_t;
        }
        semsg(
            gettext(b"E62: Nested %s%c\0".as_ptr() as *const ::core::ffi::c_char),
            if reg_magic.get() as ::core::ffi::c_uint
                == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                b"\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\\\0".as_ptr() as *const ::core::ffi::c_char
            },
            no_Magic(peekchr()),
        );
        rc_did_emsg.set(true_0 != 0);
        return NULL_0 as *mut uint8_t;
    }
    return ret;
}
pub(crate) unsafe extern "C" fn regconcat(mut flagp: *mut ::core::ffi::c_int) -> *mut uint8_t {
    let mut first: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut chain: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut latest: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut flags: ::core::ffi::c_int = 0;
    let mut cont: ::core::ffi::c_int = true_0;
    *flagp = WORST;
    while cont != 0 {
        match peekchr() {
            NUL | -132 | -218 | -215 => {
                cont = false_0;
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
                latest = regpiece(&raw mut flags);
                if latest.is_null() || reg_toolong.get() != 0 {
                    return ::core::ptr::null_mut::<uint8_t>();
                }
                *flagp |= flags & (HASWIDTH | HASNL | HASLOOKBH);
                if chain.is_null() {
                    *flagp |= flags & SPSTART;
                } else {
                    regtail(chain, latest);
                }
                chain = latest;
                if first.is_null() {
                    first = latest;
                }
            }
        }
    }
    if first.is_null() {
        first = regnode(NOTHING);
    }
    return first;
}
pub(crate) unsafe extern "C" fn regbranch(mut flagp: *mut ::core::ffi::c_int) -> *mut uint8_t {
    let mut ret: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut chain: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut latest: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut flags: ::core::ffi::c_int = 0;
    *flagp = WORST | HASNL;
    ret = regnode(BRANCH);
    loop {
        latest = regconcat(&raw mut flags);
        if latest.is_null() {
            return ::core::ptr::null_mut::<uint8_t>();
        }
        *flagp |= flags & (HASWIDTH | SPSTART | HASLOOKBH);
        *flagp &= !HASNL | flags & HASNL;
        if !chain.is_null() {
            regtail(chain, latest);
        }
        if peekchr() != '&' as ::core::ffi::c_int - 256 as ::core::ffi::c_int {
            break;
        }
        skipchr();
        regtail(latest, regnode(END));
        if reg_toolong.get() != 0 {
            break;
        }
        reginsert(MATCH, latest);
        chain = latest;
    }
    return ret;
}
pub(crate) unsafe extern "C" fn reg(
    mut paren: ::core::ffi::c_int,
    mut flagp: *mut ::core::ffi::c_int,
) -> *mut uint8_t {
    let mut ret: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut br: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut ender: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut parno: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut flags: ::core::ffi::c_int = 0;
    *flagp = HASWIDTH;
    if paren == REG_ZPAREN {
        if regnzpar.get() >= NSUBEXP as ::core::ffi::c_int {
            emsg(gettext(
                b"E50: Too many \\z(\0".as_ptr() as *const ::core::ffi::c_char
            ));
            rc_did_emsg.set(true_0 != 0);
            return NULL_0 as *mut uint8_t;
        }
        parno = regnzpar.get();
        (*regnzpar.ptr()) += 1;
        ret = regnode(ZOPEN + parno);
    } else if paren == REG_PAREN {
        if regnpar.get() >= NSUBEXP as ::core::ffi::c_int {
            semsg(
                gettext(b"E51: Too many %s(\0".as_ptr() as *const ::core::ffi::c_char),
                if reg_magic.get() as ::core::ffi::c_uint
                    == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            rc_did_emsg.set(true_0 != 0);
            return NULL_0 as *mut uint8_t;
        }
        parno = regnpar.get();
        (*regnpar.ptr()) += 1;
        ret = regnode(MOPEN + parno);
    } else if paren == REG_NPAREN {
        ret = regnode(NOPEN);
    } else {
        ret = ::core::ptr::null_mut::<uint8_t>();
    }
    br = regbranch(&raw mut flags);
    if br.is_null() {
        return ::core::ptr::null_mut::<uint8_t>();
    }
    if !ret.is_null() {
        regtail(ret, br);
    } else {
        ret = br;
    }
    if flags & HASWIDTH == 0 {
        *flagp &= !HASWIDTH;
    }
    *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);
    while peekchr() == '|' as ::core::ffi::c_int - 256 as ::core::ffi::c_int {
        skipchr();
        br = regbranch(&raw mut flags);
        if br.is_null() || reg_toolong.get() != 0 {
            return ::core::ptr::null_mut::<uint8_t>();
        }
        regtail(ret, br);
        if flags & HASWIDTH == 0 {
            *flagp &= !HASWIDTH;
        }
        *flagp |= flags & (SPSTART | HASNL | HASLOOKBH);
    }
    ender = regnode(if paren == REG_ZPAREN {
        ZCLOSE + parno
    } else if paren == REG_PAREN {
        MCLOSE + parno
    } else if paren == REG_NPAREN {
        NCLOSE
    } else {
        END
    });
    regtail(ret, ender);
    br = ret;
    while !br.is_null() {
        regoptail(br, ender);
        br = regnext(br);
    }
    if paren != REG_NOPAREN && getchr() != ')' as ::core::ffi::c_int - 256 as ::core::ffi::c_int {
        if paren == REG_ZPAREN {
            emsg(gettext(
                b"E52: Unmatched \\z(\0".as_ptr() as *const ::core::ffi::c_char
            ));
            rc_did_emsg.set(true_0 != 0);
            return NULL_0 as *mut uint8_t;
        } else if paren == REG_NPAREN {
            semsg(
                gettext((e_unmatchedpp.ptr() as *const _) as *const ::core::ffi::c_char),
                if reg_magic.get() as ::core::ffi::c_uint
                    == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            rc_did_emsg.set(true_0 != 0);
            return NULL_0 as *mut uint8_t;
        } else {
            semsg(
                gettext((e_unmatchedp.ptr() as *const _) as *const ::core::ffi::c_char),
                if reg_magic.get() as ::core::ffi::c_uint
                    == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            rc_did_emsg.set(true_0 != 0);
            return NULL_0 as *mut uint8_t;
        }
    } else if paren == REG_NOPAREN && peekchr() != NUL {
        if curchr.get() == ')' as ::core::ffi::c_int - 256 as ::core::ffi::c_int {
            semsg(
                gettext((e_unmatchedpar.ptr() as *const _) as *const ::core::ffi::c_char),
                if reg_magic.get() as ::core::ffi::c_uint
                    == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    b"\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
            rc_did_emsg.set(true_0 != 0);
            return NULL_0 as *mut uint8_t;
        } else {
            emsg(gettext(&raw const e_trailing as *const ::core::ffi::c_char));
            rc_did_emsg.set(true_0 != 0);
            return NULL_0 as *mut uint8_t;
        }
    }
    if paren == REG_PAREN {
        (*had_endbrace.ptr())[parno as usize] = true_0 as uint8_t;
    }
    return ret;
}
pub(crate) unsafe extern "C" fn bt_regcomp(
    mut expr: *mut uint8_t,
    mut re_flags: ::core::ffi::c_int,
) -> *mut regprog_T {
    let mut scan: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut longest: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut len: ::core::ffi::c_int = 0;
    let mut flags: ::core::ffi::c_int = 0;
    if expr.is_null() {
        iemsg(gettext(&raw const e_null as *const ::core::ffi::c_char));
        rc_did_emsg.set(true_0 != 0);
        return NULL_0 as *mut regprog_T;
    }
    init_class_tab();
    regcomp_start(expr, re_flags);
    regcode.set(JUST_CALC_SIZE);
    regc(REGMAGIC);
    if reg(REG_NOPAREN, &raw mut flags).is_null() {
        return ::core::ptr::null_mut::<regprog_T>();
    }
    let mut r: *mut bt_regprog_T =
        xmalloc((45 as size_t).wrapping_add(regsize.get() as size_t)) as *mut bt_regprog_T;
    (*r).re_in_use = false_0 != 0;
    regcomp_start(expr, re_flags);
    regcode.set(&raw mut (*r).program as *mut uint8_t);
    regc(REGMAGIC);
    if reg(REG_NOPAREN, &raw mut flags).is_null() || reg_toolong.get() != 0 {
        xfree(r as *mut ::core::ffi::c_void);
        if reg_toolong.get() != 0 {
            emsg(gettext(
                b"E339: Pattern too long\0".as_ptr() as *const ::core::ffi::c_char
            ));
            rc_did_emsg.set(true_0 != 0);
            return NULL_0 as *mut regprog_T;
        }
        return ::core::ptr::null_mut::<regprog_T>();
    }
    (*r).regstart = NUL;
    (*r).reganch = 0 as uint8_t;
    (*r).regmust = ::core::ptr::null_mut::<uint8_t>();
    (*r).regmlen = 0 as ::core::ffi::c_int;
    (*r).regflags = regflags.get();
    if flags & HASNL != 0 {
        (*r).regflags |= RF_HASNL as ::core::ffi::c_uint;
    }
    if flags & HASLOOKBH != 0 {
        (*r).regflags |= RF_LOOKBH as ::core::ffi::c_uint;
    }
    (*r).reghasz = re_has_z.get() as uint8_t;
    scan = (&raw mut (*r).program as *mut uint8_t).offset(1 as ::core::ffi::c_int as isize);
    if *regnext(scan) as ::core::ffi::c_int == END {
        scan = scan.offset(3 as ::core::ffi::c_int as isize);
        if *scan as ::core::ffi::c_int == BOL || *scan as ::core::ffi::c_int == RE_BOF {
            (*r).reganch = (*r).reganch.wrapping_add(1);
            scan = regnext(scan);
        }
        if *scan as ::core::ffi::c_int == EXACTLY {
            (*r).regstart = utf_ptr2char(
                scan.offset(3 as ::core::ffi::c_int as isize) as *mut ::core::ffi::c_char
            );
        } else if *scan as ::core::ffi::c_int == BOW
            || *scan as ::core::ffi::c_int == EOW
            || *scan as ::core::ffi::c_int == NOTHING
            || *scan as ::core::ffi::c_int == MOPEN + 0 as ::core::ffi::c_int
            || *scan as ::core::ffi::c_int == NOPEN
            || *scan as ::core::ffi::c_int == MCLOSE + 0 as ::core::ffi::c_int
            || *scan as ::core::ffi::c_int == NCLOSE
        {
            let mut regnext_scan: *mut uint8_t = regnext(scan);
            if *regnext_scan as ::core::ffi::c_int == EXACTLY {
                (*r).regstart = utf_ptr2char(regnext_scan.offset(3 as ::core::ffi::c_int as isize)
                    as *mut ::core::ffi::c_char);
            }
        }
        if (flags & SPSTART != 0
            || *scan as ::core::ffi::c_int == BOW
            || *scan as ::core::ffi::c_int == EOW)
            && flags & HASNL == 0
        {
            longest = ::core::ptr::null_mut::<uint8_t>();
            len = 0 as ::core::ffi::c_int;
            while !scan.is_null() {
                if *scan as ::core::ffi::c_int == EXACTLY {
                    let mut scanlen: size_t =
                        strlen(scan.offset(3 as ::core::ffi::c_int as isize)
                            as *mut ::core::ffi::c_char);
                    if scanlen >= len as size_t {
                        longest = scan.offset(3 as ::core::ffi::c_int as isize);
                        len = scanlen as ::core::ffi::c_int;
                    }
                }
                scan = regnext(scan);
            }
            (*r).regmust = longest;
            (*r).regmlen = len;
        }
    }
    (*r).engine = bt_regengine.ptr();
    return r as *mut regprog_T;
}
pub unsafe extern "C" fn vim_regcomp_had_eol() -> ::core::ffi::c_int {
    return had_eol.get();
}
pub(crate) unsafe extern "C" fn coll_get_char() -> ::core::ffi::c_int {
    let mut nr: int64_t = -1 as int64_t;
    let c2rust_fresh131 = regparse.get();
    regparse.set((*regparse.ptr()).offset(1));
    match *c2rust_fresh131 as ::core::ffi::c_int {
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
        _ => {}
    }
    if nr < 0 as int64_t {
        regparse.set((*regparse.ptr()).offset(-1));
        nr = '\\' as int64_t;
    }
    if nr > INT_MAX as int64_t {
        nr = INT_MAX as int64_t;
    }
    return nr as ::core::ffi::c_int;
}
pub(crate) unsafe extern "C" fn bt_regfree(mut prog: *mut regprog_T) {
    xfree(prog as *mut ::core::ffi::c_void);
}
