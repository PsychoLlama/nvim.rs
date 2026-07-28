//! The pattern cursor both engines parse through: `peekchr`/`skipchr`
//! and the escape, limit and skip helpers built on them.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn re_multi_type(mut c: ::core::ffi::c_int) -> ::core::ffi::c_int {
    if c == '@' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
        || c == '=' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
        || c == '?' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
    {
        return MULTI_ONE;
    }
    if c == '*' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
        || c == '+' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
        || c == '{' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
    {
        return MULTI_MULT;
    }
    return NOT_MULTI;
}
pub(crate) unsafe extern "C" fn skip_anyof(
    mut p: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    let mut l: ::core::ffi::c_int = 0;
    if *p as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
        p = p.offset(1);
    }
    if *p as ::core::ffi::c_int == ']' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
    {
        p = p.offset(1);
    }
    while *p as ::core::ffi::c_int != NUL && *p as ::core::ffi::c_int != ']' as ::core::ffi::c_int {
        l = utfc_ptr2len(p);
        if l > 1 as ::core::ffi::c_int {
            p = p.offset(l as isize);
        } else if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
            p = p.offset(1);
            if *p as ::core::ffi::c_int != ']' as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != NUL
            {
                p = p.offset(utfc_ptr2len(p) as isize);
            }
        } else if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
            && (!vim_strchr(
                REGEXP_INRANGE.as_ptr(),
                *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
                || reg_cpo_lit.get() == 0
                    && !vim_strchr(
                        REGEXP_ABBR.as_ptr(),
                        *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                            as ::core::ffi::c_int,
                    )
                    .is_null())
        {
            p = p.offset(2 as ::core::ffi::c_int as isize);
        } else if *p as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
            if get_char_class(&raw mut p) == CLASS_NONE as ::core::ffi::c_int
                && get_equi_class(&raw mut p) == 0 as ::core::ffi::c_int
                && get_coll_element(&raw mut p) == 0 as ::core::ffi::c_int
                && *p as ::core::ffi::c_int != NUL
            {
                p = p.offset(1);
            }
        } else {
            p = p.offset(1);
        }
    }
    return p;
}
pub unsafe extern "C" fn skip_regexp(
    mut startp: *mut ::core::ffi::c_char,
    mut delim: ::core::ffi::c_int,
    mut magic: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    return skip_regexp_ex(
        startp,
        delim,
        magic,
        ::core::ptr::null_mut::<*mut ::core::ffi::c_char>(),
        ::core::ptr::null_mut::<::core::ffi::c_int>(),
        ::core::ptr::null_mut::<magic_T>(),
    );
}
pub unsafe extern "C" fn skip_regexp_err(
    mut startp: *mut ::core::ffi::c_char,
    mut delim: ::core::ffi::c_int,
    mut magic: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    let mut p: *mut ::core::ffi::c_char = skip_regexp(startp, delim, magic);
    if *p as ::core::ffi::c_int != delim {
        semsg(
            gettext(E_MISSING_DELIMITER_AFTER_SEARCH_PATTERN_STR.as_ptr()),
            startp,
        );
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
    return p;
}
pub unsafe extern "C" fn skip_regexp_ex(
    mut startp: *mut ::core::ffi::c_char,
    mut dirc: ::core::ffi::c_int,
    mut magic: ::core::ffi::c_int,
    mut newp: *mut *mut ::core::ffi::c_char,
    mut dropped: *mut ::core::ffi::c_int,
    mut magic_val: *mut magic_T,
) -> *mut ::core::ffi::c_char {
    let mut mymagic: magic_T = 0 as magic_T;
    let mut p: *mut ::core::ffi::c_char = startp;
    let mut startplen: size_t = 0 as size_t;
    if magic != 0 {
        mymagic = MAGIC_ON;
    } else {
        mymagic = MAGIC_OFF;
    }
    get_cpo_flags();
    while *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == dirc {
            break;
        }
        if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '[' as ::core::ffi::c_int
            && mymagic as ::core::ffi::c_uint
                >= MAGIC_ON as ::core::ffi::c_int as ::core::ffi::c_uint
            || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '\\' as ::core::ffi::c_int
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '[' as ::core::ffi::c_int
                && mymagic as ::core::ffi::c_uint
                    <= MAGIC_OFF as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            p = skip_anyof(p.offset(1 as ::core::ffi::c_int as isize));
            if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL {
                break;
            }
        } else if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
            == '\\' as ::core::ffi::c_int
            && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            if dirc == '?' as ::core::ffi::c_int
                && !newp.is_null()
                && *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '?' as ::core::ffi::c_int
            {
                if startplen == 0 as size_t {
                    startplen = strlen(startp);
                }
                if (*newp).is_null() {
                    *newp = xstrnsave(startp, startplen);
                    p = (*newp).offset(p.offset_from(startp) as isize);
                    startp = *newp;
                }
                if !dropped.is_null() {
                    *dropped += 1;
                }
                memmove(
                    p as *mut ::core::ffi::c_void,
                    p.offset(1 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    startplen
                        .wrapping_sub(
                            p.offset(1 as ::core::ffi::c_int as isize)
                                .offset_from(startp) as size_t,
                        )
                        .wrapping_add(1 as size_t),
                );
            } else {
                p = p.offset(1);
            }
            if *p as ::core::ffi::c_int == 'v' as ::core::ffi::c_int {
                mymagic = MAGIC_ALL;
            } else if *p as ::core::ffi::c_int == 'V' as ::core::ffi::c_int {
                mymagic = MAGIC_NONE;
            }
        }
        p = p.offset(utfc_ptr2len(p) as isize);
    }
    if !magic_val.is_null() {
        *magic_val = mymagic;
    }
    return p;
}
pub(crate) unsafe extern "C" fn initchr(mut str: *mut ::core::ffi::c_char) {
    regparse.set(str);
    prevchr_len.set(0 as ::core::ffi::c_int);
    nextchr.set(-1 as ::core::ffi::c_int);
    prevchr.set(nextchr.get());
    prevprevchr.set(prevchr.get());
    curchr.set(prevprevchr.get());
    at_start.set(true_0);
    prev_at_start.set(false_0);
}
pub(crate) unsafe extern "C" fn save_parse_state(mut ps: *mut parse_state_T) {
    (*ps).regparse = regparse.get();
    (*ps).prevchr_len = prevchr_len.get();
    (*ps).curchr = curchr.get();
    (*ps).prevchr = prevchr.get();
    (*ps).prevprevchr = prevprevchr.get();
    (*ps).nextchr = nextchr.get();
    (*ps).at_start = at_start.get();
    (*ps).prev_at_start = prev_at_start.get();
    (*ps).regnpar = regnpar.get();
}
pub(crate) unsafe extern "C" fn restore_parse_state(mut ps: *mut parse_state_T) {
    regparse.set((*ps).regparse);
    prevchr_len.set((*ps).prevchr_len);
    curchr.set((*ps).curchr);
    prevchr.set((*ps).prevchr);
    prevprevchr.set((*ps).prevprevchr);
    nextchr.set((*ps).nextchr);
    at_start.set((*ps).at_start);
    prev_at_start.set((*ps).prev_at_start);
    regnpar.set((*ps).regnpar);
}
pub(crate) unsafe extern "C" fn peekchr() -> ::core::ffi::c_int {
    static after_slash: GlobalCell<::core::ffi::c_int> = GlobalCell::new(false_0);
    if curchr.get() != -1 as ::core::ffi::c_int {
        return curchr.get();
    }
    curchr.set(
        *(*regparse.ptr()).offset(0 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int,
    );
    match curchr.get() {
        46 | 91 | 126 => {
            if reg_magic.get() as ::core::ffi::c_uint
                >= MAGIC_ON as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                curchr.set(curchr.get() - 256 as ::core::ffi::c_int);
            }
        }
        40 | 41 | 123 | 37 | 43 | 61 | 63 | 64 | 33 | 38 | 124 | 60 | 62 | 35 | 34 | 39 | 44
        | 45 | 58 | 59 | 96 | 47 => {
            if reg_magic.get() as ::core::ffi::c_uint
                == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                curchr.set(curchr.get() - 256 as ::core::ffi::c_int);
            }
        }
        42 => {
            if reg_magic.get() as ::core::ffi::c_uint
                >= MAGIC_ON as ::core::ffi::c_int as ::core::ffi::c_uint
                && at_start.get() == 0
                && !(prev_at_start.get() != 0
                    && prevchr.get() == '^' as ::core::ffi::c_int - 256 as ::core::ffi::c_int)
                && (after_slash.get() != 0
                    || prevchr.get() != '(' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
                        && prevchr.get() != '&' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
                        && prevchr.get() != '|' as ::core::ffi::c_int - 256 as ::core::ffi::c_int)
            {
                curchr.set('*' as ::core::ffi::c_int - 256 as ::core::ffi::c_int);
            }
        }
        94 => {
            if reg_magic.get() as ::core::ffi::c_uint
                >= MAGIC_OFF as ::core::ffi::c_int as ::core::ffi::c_uint
                && (at_start.get() != 0
                    || reg_magic.get() as ::core::ffi::c_uint
                        == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                    || prevchr.get() == '(' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
                    || prevchr.get() == '|' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
                    || prevchr.get() == '&' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
                    || prevchr.get() == 'n' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
                    || no_Magic(prevchr.get()) == '(' as ::core::ffi::c_int
                        && prevprevchr.get()
                            == '%' as ::core::ffi::c_int - 256 as ::core::ffi::c_int)
            {
                curchr.set('^' as ::core::ffi::c_int - 256 as ::core::ffi::c_int);
                at_start.set(true_0);
                prev_at_start.set(false_0);
            }
        }
        36 => {
            if reg_magic.get() as ::core::ffi::c_uint
                >= MAGIC_OFF as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let mut p: *mut uint8_t =
                    (regparse.get() as *mut uint8_t).offset(1 as ::core::ffi::c_int as isize);
                let mut is_magic_all: bool = reg_magic.get() as ::core::ffi::c_uint
                    == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint;
                while *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int
                    && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'c' as ::core::ffi::c_int
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'C' as ::core::ffi::c_int
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'm' as ::core::ffi::c_int
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'M' as ::core::ffi::c_int
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'v' as ::core::ffi::c_int
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'V' as ::core::ffi::c_int
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'Z' as ::core::ffi::c_int)
                {
                    if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'v' as ::core::ffi::c_int
                    {
                        is_magic_all = true_0 != 0;
                    } else if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == 'm' as ::core::ffi::c_int
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'M' as ::core::ffi::c_int
                        || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == 'V' as ::core::ffi::c_int
                    {
                        is_magic_all = false_0 != 0;
                    }
                    p = p.offset(2 as ::core::ffi::c_int as isize);
                }
                if *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == NUL
                    || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int
                        && (*p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '|' as ::core::ffi::c_int
                            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '&' as ::core::ffi::c_int
                            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == ')' as ::core::ffi::c_int
                            || *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == 'n' as ::core::ffi::c_int)
                    || is_magic_all as ::core::ffi::c_int != 0
                        && (*p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            == '|' as ::core::ffi::c_int
                            || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == '&' as ::core::ffi::c_int
                            || *p.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                == ')' as ::core::ffi::c_int)
                    || reg_magic.get() as ::core::ffi::c_uint
                        == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    curchr.set('$' as ::core::ffi::c_int - 256 as ::core::ffi::c_int);
                }
            }
        }
        92 => {
            let mut c: ::core::ffi::c_int = *(*regparse.ptr())
                .offset(1 as ::core::ffi::c_int as isize)
                as uint8_t as ::core::ffi::c_int;
            if c == NUL {
                curchr.set('\\' as ::core::ffi::c_int);
            } else if c <= '~' as ::core::ffi::c_int
                && (*META_flags.ptr())[c as usize] as ::core::ffi::c_int != 0
            {
                curchr.set(-1 as ::core::ffi::c_int);
                prev_at_start.set(at_start.get());
                at_start.set(false_0);
                regparse.set((*regparse.ptr()).offset(1));
                (*after_slash.ptr()) += 1;
                peekchr();
                regparse.set((*regparse.ptr()).offset(-1));
                (*after_slash.ptr()) -= 1;
                curchr.set(toggle_Magic(curchr.get()));
            } else if !vim_strchr(REGEXP_ABBR.as_ptr(), c).is_null() {
                curchr.set(backslash_trans(c));
            } else if reg_magic.get() as ::core::ffi::c_uint
                == MAGIC_NONE as ::core::ffi::c_int as ::core::ffi::c_uint
                && (c == '$' as ::core::ffi::c_int || c == '^' as ::core::ffi::c_int)
            {
                curchr.set(toggle_Magic(c));
            } else {
                curchr.set(utf_ptr2char(
                    (*regparse.ptr()).offset(1 as ::core::ffi::c_int as isize),
                ));
            }
        }
        _ => {
            curchr.set(utf_ptr2char(regparse.get()));
        }
    }
    return curchr.get();
}
pub(crate) unsafe extern "C" fn skipchr() {
    if *regparse.get() as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
        prevchr_len.set(1 as ::core::ffi::c_int);
    } else {
        prevchr_len.set(0 as ::core::ffi::c_int);
    }
    if *(*regparse.ptr()).offset(prevchr_len.get() as isize) as ::core::ffi::c_int != NUL {
        (*prevchr_len.ptr()) += utf_ptr2len((*regparse.ptr()).offset(prevchr_len.get() as isize));
    }
    regparse.set((*regparse.ptr()).offset(prevchr_len.get() as isize));
    prev_at_start.set(at_start.get());
    at_start.set(false_0);
    prevprevchr.set(prevchr.get());
    prevchr.set(curchr.get());
    curchr.set(nextchr.get());
    nextchr.set(-1 as ::core::ffi::c_int);
}
pub(crate) unsafe extern "C" fn skipchr_keepstart() {
    let mut as_0: ::core::ffi::c_int = prev_at_start.get();
    let mut pr: ::core::ffi::c_int = prevchr.get();
    let mut prpr: ::core::ffi::c_int = prevprevchr.get();
    skipchr();
    at_start.set(as_0);
    prevchr.set(pr);
    prevprevchr.set(prpr);
}
pub(crate) unsafe extern "C" fn getchr() -> ::core::ffi::c_int {
    let mut chr: ::core::ffi::c_int = peekchr();
    skipchr();
    return chr;
}
pub(crate) unsafe extern "C" fn ungetchr() {
    nextchr.set(curchr.get());
    curchr.set(prevchr.get());
    prevchr.set(prevprevchr.get());
    at_start.set(prev_at_start.get());
    prev_at_start.set(false_0);
    regparse.set((*regparse.ptr()).offset(-(prevchr_len.get() as isize)));
}
pub(crate) unsafe extern "C" fn gethexchrs(mut maxinputlen: ::core::ffi::c_int) -> int64_t {
    let mut nr: int64_t = 0 as int64_t;
    let mut c: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < maxinputlen {
        c = *(*regparse.ptr()).offset(0 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int;
        if !ascii_isxdigit(c) {
            break;
        }
        nr <<= 4 as ::core::ffi::c_int;
        nr |= hex2nr(c) as int64_t;
        regparse.set((*regparse.ptr()).offset(1));
        i += 1;
    }
    if i == 0 as ::core::ffi::c_int {
        return -1 as int64_t;
    }
    return nr;
}
pub(crate) unsafe extern "C" fn getdecchrs() -> int64_t {
    let mut nr: int64_t = 0 as int64_t;
    let mut c: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    loop {
        c = *(*regparse.ptr()).offset(0 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int;
        if c < '0' as ::core::ffi::c_int || c > '9' as ::core::ffi::c_int {
            break;
        }
        nr *= 10 as int64_t;
        nr += (c - '0' as ::core::ffi::c_int) as int64_t;
        regparse.set((*regparse.ptr()).offset(1));
        curchr.set(-1 as ::core::ffi::c_int);
        i += 1;
    }
    if i == 0 as ::core::ffi::c_int {
        return -1 as int64_t;
    }
    return nr;
}
pub(crate) unsafe extern "C" fn getoctchrs() -> int64_t {
    let mut nr: int64_t = 0 as int64_t;
    let mut c: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    i = 0 as ::core::ffi::c_int;
    while i < 3 as ::core::ffi::c_int && nr < 0o40 as int64_t {
        c = *(*regparse.ptr()).offset(0 as ::core::ffi::c_int as isize) as uint8_t
            as ::core::ffi::c_int;
        if c < '0' as ::core::ffi::c_int || c > '7' as ::core::ffi::c_int {
            break;
        }
        nr <<= 3 as ::core::ffi::c_int;
        nr |= hex2nr(c) as int64_t;
        regparse.set((*regparse.ptr()).offset(1));
        i += 1;
    }
    if i == 0 as ::core::ffi::c_int {
        return -1 as int64_t;
    }
    return nr;
}
pub(crate) unsafe extern "C" fn read_limits(
    mut minval: *mut ::core::ffi::c_int,
    mut maxval: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut reverse: ::core::ffi::c_int = false_0;
    let mut first_char: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut tmp: ::core::ffi::c_int = 0;
    if *regparse.get() as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
        regparse.set((*regparse.ptr()).offset(1));
        reverse = true_0;
    }
    first_char = regparse.get();
    *minval = getdigits_int(regparse.ptr(), false_0 != 0, 0 as ::core::ffi::c_int);
    if *regparse.get() as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        regparse.set((*regparse.ptr()).offset(1));
        if ascii_isdigit(*regparse.get() as ::core::ffi::c_int) {
            *maxval = getdigits_int(regparse.ptr(), false_0 != 0, MAX_LIMIT);
        } else {
            *maxval = MAX_LIMIT;
        }
    } else if ascii_isdigit(*first_char as ::core::ffi::c_int) {
        *maxval = *minval;
    } else {
        *maxval = MAX_LIMIT;
    }
    if *regparse.get() as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
        regparse.set((*regparse.ptr()).offset(1));
    }
    if *regparse.get() as ::core::ffi::c_int != '}' as ::core::ffi::c_int {
        semsg(
            gettext(b"E554: Syntax error in %s{...}\0".as_ptr() as *const ::core::ffi::c_char),
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
    if reverse == 0 && *minval > *maxval || reverse != 0 && *minval < *maxval {
        tmp = *minval;
        *minval = *maxval;
        *maxval = tmp;
    }
    skipchr();
    return OK;
}
