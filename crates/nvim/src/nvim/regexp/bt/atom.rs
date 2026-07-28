//! One atom of a pattern: the `[]` collections, the `\\%[]` sequence and
//! every escape that stands for a single node.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub(crate) unsafe extern "C" fn regatom(mut flagp: *mut ::core::ffi::c_int) -> *mut uint8_t {
    let mut ret: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut flags: ::core::ffi::c_int = 0;
    let mut c: ::core::ffi::c_int = 0;
    let mut p: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut extra: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut save_prev_at_start: ::core::ffi::c_int = prev_at_start.get();
    *flagp = WORST;
    c = getchr();
    let mut len_0: ::core::ffi::c_int = 0;
    's_2192: {
        '_do_multibyte: {
            's_2080: {
                'c_120706: {
                    match c {
                        -162 => {
                            ret = regnode(BOL);
                            break 's_2192;
                        }
                        -220 => {
                            ret = regnode(EOL);
                            had_eol.set(true_0);
                            break 's_2192;
                        }
                        -196 => {
                            ret = regnode(BOW);
                            break 's_2192;
                        }
                        -194 => {
                            ret = regnode(EOW);
                            break 's_2192;
                        }
                        -161 => {
                            c = no_Magic(getchr());
                            if c == '^' as ::core::ffi::c_int {
                                ret = regnode(BOL);
                                break 's_2192;
                            } else if c == '$' as ::core::ffi::c_int {
                                ret = regnode(EOL);
                                had_eol.set(true_0);
                                break 's_2192;
                            } else {
                                extra = ADD_NL;
                                *flagp |= HASNL;
                                if c != '[' as ::core::ffi::c_int {
                                    break 'c_120706;
                                }
                            }
                        }
                        -210 | -151 | -183 | -149 | -181 | -154 | -186 | -144 | -176 | -141
                        | -173 | -156 | -188 | -136 | -168 | -145 | -177 | -137 | -169 | -152
                        | -184 | -159 | -191 | -148 | -180 | -139 | -171 => {
                            break 'c_120706;
                        }
                        -146 => {
                            if reg_string.get() != 0 {
                                ret = regnode(EXACTLY);
                                regc(NL);
                                regc(NUL);
                                *flagp |= HASWIDTH | SIMPLE;
                            } else {
                                ret = regnode(NEWL);
                                *flagp |= HASWIDTH | HASNL;
                            }
                            break 's_2192;
                        }
                        -216 => {
                            if one_exactly.get() != 0 {
                                semsg(
                                    gettext(E_INVALID_ITEM_IN_STR_BRACKETS.as_ptr()),
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
                            ret = reg(REG_PAREN, &raw mut flags);
                            if ret.is_null() {
                                return ::core::ptr::null_mut::<uint8_t>();
                            }
                            *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
                            break 's_2192;
                        }
                        NUL | -132 | -218 | -215 => {
                            if one_exactly.get() != 0 {
                                semsg(
                                    gettext(E_INVALID_ITEM_IN_STR_BRACKETS.as_ptr()),
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
                            iemsg(gettext(
                                &raw const e_internal_error_in_regexp as *const ::core::ffi::c_char,
                            ));
                            rc_did_emsg.set(true_0 != 0);
                            return NULL_0 as *mut uint8_t;
                        }
                        -195 | -193 | -213 | -192 | -133 | -214 => {
                            c = no_Magic(c);
                            semsg(
                                gettext(b"E64: %s%c follows nothing\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                if (if c == '*' as ::core::ffi::c_int {
                                    (reg_magic.get() as ::core::ffi::c_uint
                                        >= MAGIC_ON as ::core::ffi::c_int as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                } else {
                                    (reg_magic.get() as ::core::ffi::c_uint
                                        == MAGIC_ALL as ::core::ffi::c_int as ::core::ffi::c_uint)
                                        as ::core::ffi::c_int
                                }) != 0
                                {
                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                } else {
                                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                                },
                                c,
                            );
                            rc_did_emsg.set(true_0 != 0);
                            return NULL_0 as *mut uint8_t;
                        }
                        -130 => {
                            if !(*reg_prev_sub.ptr()).is_null() {
                                let mut lp: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
                                ret = regnode(EXACTLY);
                                lp = reg_prev_sub.get() as *mut uint8_t;
                                while *lp as ::core::ffi::c_int != NUL {
                                    let c2rust_fresh1487 = lp;
                                    lp = lp.offset(1);
                                    regc(*c2rust_fresh1487 as ::core::ffi::c_int);
                                }
                                regc(NUL);
                                if *reg_prev_sub.get() as ::core::ffi::c_int != NUL {
                                    *flagp |= HASWIDTH;
                                    if lp.offset_from(reg_prev_sub.get() as *mut uint8_t)
                                        == 1 as isize
                                    {
                                        *flagp |= SIMPLE;
                                    }
                                }
                            } else {
                                emsg(gettext(&raw const e_nopresub as *const ::core::ffi::c_char));
                                rc_did_emsg.set(true_0 != 0);
                                return NULL_0 as *mut uint8_t;
                            }
                            break 's_2192;
                        }
                        -207 | -206 | -205 | -204 | -203 | -202 | -201 | -200 | -199 => {
                            let mut refnum: ::core::ffi::c_int = 0;
                            refnum = c - ('0' as ::core::ffi::c_int - 256 as ::core::ffi::c_int);
                            if seen_endbrace(refnum) == 0 {
                                return ::core::ptr::null_mut::<uint8_t>();
                            }
                            ret = regnode(BACKREF + refnum);
                            break 's_2192;
                        }
                        -134 => {
                            c = no_Magic(getchr());
                            match c {
                                40 => {
                                    if reg_do_extmatch.get() & REX_SET == 0 as ::core::ffi::c_int {
                                        emsg(gettext(E_Z_NOT_ALLOWED.as_ptr()));
                                        rc_did_emsg.set(true_0 != 0);
                                        return NULL_0 as *mut uint8_t;
                                    }
                                    if one_exactly.get() != 0 {
                                        semsg(
                                            gettext(E_INVALID_ITEM_IN_STR_BRACKETS.as_ptr()),
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
                                        return NULL_0 as *mut uint8_t;
                                    }
                                    ret = reg(REG_ZPAREN, &raw mut flags);
                                    if ret.is_null() {
                                        return ::core::ptr::null_mut::<uint8_t>();
                                    }
                                    *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
                                    re_has_z.set(REX_SET);
                                }
                                49 | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 => {
                                    if reg_do_extmatch.get() & REX_USE == 0 as ::core::ffi::c_int {
                                        emsg(gettext(E_Z1_NOT_ALLOWED.as_ptr()));
                                        rc_did_emsg.set(true_0 != 0);
                                        return NULL_0 as *mut uint8_t;
                                    }
                                    ret = regnode(ZREF + c - '0' as ::core::ffi::c_int);
                                    re_has_z.set(REX_USE);
                                }
                                115 => {
                                    ret = regnode(MOPEN + 0 as ::core::ffi::c_int);
                                    if !re_mult_next(b"\\zs\0".as_ptr()
                                        as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char)
                                    {
                                        return ::core::ptr::null_mut::<uint8_t>();
                                    }
                                }
                                101 => {
                                    ret = regnode(MCLOSE + 0 as ::core::ffi::c_int);
                                    if !re_mult_next(b"\\ze\0".as_ptr()
                                        as *const ::core::ffi::c_char
                                        as *mut ::core::ffi::c_char)
                                    {
                                        return ::core::ptr::null_mut::<uint8_t>();
                                    }
                                }
                                _ => {
                                    emsg(gettext(b"E68: Invalid character after \\z\0".as_ptr()
                                        as *const ::core::ffi::c_char));
                                    rc_did_emsg.set(true_0 != 0);
                                    return NULL_0 as *mut uint8_t;
                                }
                            }
                            break 's_2192;
                        }
                        -219 => {
                            c = no_Magic(getchr());
                            's_1154: {
                                match c {
                                    40 => {
                                        if one_exactly.get() != 0 {
                                            semsg(
                                                gettext(E_INVALID_ITEM_IN_STR_BRACKETS.as_ptr()),
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
                                            return NULL_0 as *mut uint8_t;
                                        }
                                        ret = reg(REG_NPAREN, &raw mut flags);
                                        if ret.is_null() {
                                            return ::core::ptr::null_mut::<uint8_t>();
                                        }
                                        *flagp |= flags & (HASWIDTH | SPSTART | HASNL | HASLOOKBH);
                                    }
                                    94 => {
                                        ret = regnode(RE_BOF);
                                    }
                                    36 => {
                                        ret = regnode(RE_EOF);
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
                                            return ::core::ptr::null_mut::<uint8_t>();
                                        }
                                        ret = regnode(CURSOR);
                                    }
                                    86 => {
                                        ret = regnode(RE_VISUAL);
                                    }
                                    67 => {
                                        ret = regnode(RE_COMPOSING);
                                    }
                                    91 => {
                                        if one_exactly.get() != 0 {
                                            semsg(
                                                gettext(E_INVALID_ITEM_IN_STR_BRACKETS.as_ptr()),
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
                                            return NULL_0 as *mut uint8_t;
                                        }
                                        let mut lastbranch: *mut uint8_t =
                                            ::core::ptr::null_mut::<uint8_t>();
                                        let mut lastnode: *mut uint8_t =
                                            ::core::ptr::null_mut::<uint8_t>();
                                        let mut br: *mut uint8_t =
                                            ::core::ptr::null_mut::<uint8_t>();
                                        ret = ::core::ptr::null_mut::<uint8_t>();
                                        loop {
                                            c = getchr();
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
                                                return NULL_0 as *mut uint8_t;
                                            }
                                            br = regnode(BRANCH);
                                            if ret.is_null() {
                                                ret = br;
                                            } else {
                                                regtail(lastnode, br);
                                                if reg_toolong.get() != 0 {
                                                    return ::core::ptr::null_mut::<uint8_t>();
                                                }
                                            }
                                            ungetchr();
                                            one_exactly.set(true_0);
                                            lastnode = regatom(flagp);
                                            one_exactly.set(false_0);
                                            if lastnode.is_null() {
                                                return ::core::ptr::null_mut::<uint8_t>();
                                            }
                                        }
                                        if ret.is_null() {
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
                                            return NULL_0 as *mut uint8_t;
                                        }
                                        lastbranch = regnode(BRANCH);
                                        br = regnode(NOTHING);
                                        if ret != JUST_CALC_SIZE {
                                            regtail(lastnode, br);
                                            regtail(lastbranch, br);
                                            br = ret;
                                            while br != lastnode {
                                                if *br as ::core::ffi::c_int == BRANCH {
                                                    regtail(br, lastbranch);
                                                    if reg_toolong.get() != 0 {
                                                        return ::core::ptr::null_mut::<uint8_t>();
                                                    }
                                                    br =
                                                        br.offset(3 as ::core::ffi::c_int as isize);
                                                } else {
                                                    br = regnext(br);
                                                }
                                            }
                                        }
                                        *flagp &= !(HASWIDTH | SIMPLE);
                                    }
                                    100 | 111 | 120 | 117 | 85 => {
                                        let mut i: int64_t = 0;
                                        match c {
                                            100 => {
                                                i = getdecchrs();
                                            }
                                            111 => {
                                                i = getoctchrs();
                                            }
                                            120 => {
                                                i = gethexchrs(2 as ::core::ffi::c_int);
                                            }
                                            117 => {
                                                i = gethexchrs(4 as ::core::ffi::c_int);
                                            }
                                            85 => {
                                                i = gethexchrs(8 as ::core::ffi::c_int);
                                            }
                                            _ => {
                                                i = -1 as int64_t;
                                            }
                                        }
                                        if i < 0 as int64_t || i > INT_MAX as int64_t {
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
                                            return NULL_0 as *mut uint8_t;
                                        }
                                        if use_multibytecode(i as ::core::ffi::c_int) {
                                            ret = regnode(MULTIBYTECODE);
                                        } else {
                                            ret = regnode(EXACTLY);
                                        }
                                        if i == 0 as int64_t {
                                            regc(0xa as ::core::ffi::c_int);
                                        } else {
                                            regmbc(i as ::core::ffi::c_int);
                                        }
                                        regc(NUL);
                                        *flagp |= HASWIDTH;
                                    }
                                    _ => {
                                        if ascii_isdigit(c) as ::core::ffi::c_int != 0
                                            || c == '<' as ::core::ffi::c_int
                                            || c == '>' as ::core::ffi::c_int
                                            || c == '\'' as ::core::ffi::c_int
                                            || c == '.' as ::core::ffi::c_int
                                        {
                                            let mut n: uint32_t = 0 as uint32_t;
                                            let mut cmp: ::core::ffi::c_int = 0;
                                            let mut cur: bool = false_0 != 0;
                                            let mut got_digit: bool = false_0 != 0;
                                            cmp = c;
                                            if cmp == '<' as ::core::ffi::c_int
                                                || cmp == '>' as ::core::ffi::c_int
                                            {
                                                c = getchr();
                                            }
                                            if no_Magic(c) == '.' as ::core::ffi::c_int {
                                                cur = true_0 != 0;
                                                c = getchr();
                                            }
                                            while ascii_isdigit(c) {
                                                got_digit = true_0 != 0;
                                                n = n.wrapping_mul(10 as uint32_t).wrapping_add(
                                                    (c - '0' as ::core::ffi::c_int) as uint32_t,
                                                );
                                                c = getchr();
                                            }
                                            if no_Magic(c) == '\'' as ::core::ffi::c_int
                                                && n == 0 as uint32_t
                                            {
                                                c = getchr();
                                                ret = regnode(RE_MARK);
                                                if ret == JUST_CALC_SIZE {
                                                    (*regsize.ptr()) += 2 as int64_t;
                                                } else {
                                                    let c2rust_fresh1488 = regcode.get();
                                                    regcode.set((*regcode.ptr()).offset(1));
                                                    *c2rust_fresh1488 = c as uint8_t;
                                                    let c2rust_fresh1489 = regcode.get();
                                                    regcode.set((*regcode.ptr()).offset(1));
                                                    *c2rust_fresh1489 = cmp as uint8_t;
                                                }
                                                break 's_1154;
                                            } else if (c == 'l' as ::core::ffi::c_int
                                                || c == 'c' as ::core::ffi::c_int
                                                || c == 'v' as ::core::ffi::c_int)
                                                && (cur as ::core::ffi::c_int != 0
                                                    || got_digit as ::core::ffi::c_int != 0)
                                            {
                                                if cur as ::core::ffi::c_int != 0 && n != 0 {
                                                    semsg(
                                                        gettext(
                                                            E_REGEXP_NUMBER_AFTER_DOT_POS_SEARCH_CHR.as_ptr(),
                                                        ),
                                                        no_Magic(c),
                                                    );
                                                    rc_did_emsg.set(true_0 != 0);
                                                    return ::core::ptr::null_mut::<uint8_t>();
                                                }
                                                if c == 'l' as ::core::ffi::c_int {
                                                    if cur {
                                                        n = (*curwin.get()).w_cursor.lnum
                                                            as uint32_t;
                                                    }
                                                    ret = regnode(RE_LNUM);
                                                    if save_prev_at_start != 0 {
                                                        at_start.set(true_0);
                                                    }
                                                } else if c == 'c' as ::core::ffi::c_int {
                                                    if cur {
                                                        n = (*curwin.get()).w_cursor.col
                                                            as uint32_t;
                                                        n = n.wrapping_add(1);
                                                    }
                                                    ret = regnode(RE_COL);
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
                                                        n = vcol as uint32_t;
                                                    }
                                                    ret = regnode(RE_VCOL);
                                                }
                                                if ret == JUST_CALC_SIZE {
                                                    (*regsize.ptr()) += 5 as int64_t;
                                                } else {
                                                    regcode.set(re_put_uint32(regcode.get(), n));
                                                    let c2rust_fresh1490 = regcode.get();
                                                    regcode.set((*regcode.ptr()).offset(1));
                                                    *c2rust_fresh1490 = cmp as uint8_t;
                                                }
                                                break 's_1154;
                                            }
                                        }
                                        semsg(
                                            gettext(
                                                b"E71: Invalid character after %s%%\0".as_ptr()
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
                                        return NULL_0 as *mut uint8_t;
                                    }
                                }
                            }
                            break 's_2192;
                        }
                        -165 => {}
                        _ => {
                            break 's_2080;
                        }
                    }
                    let mut lp_0: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
                    lp_0 = skip_anyof(regparse.get()) as *mut uint8_t;
                    if *lp_0 as ::core::ffi::c_int == ']' as ::core::ffi::c_int {
                        let mut startc: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                        let mut endc: ::core::ffi::c_int = 0;
                        if *regparse.get() as ::core::ffi::c_int == '^' as ::core::ffi::c_int {
                            ret = regnode(ANYBUT + extra);
                            regparse.set((*regparse.ptr()).offset(1));
                        } else {
                            ret = regnode(ANYOF + extra);
                        }
                        if *regparse.get() as ::core::ffi::c_int == ']' as ::core::ffi::c_int
                            || *regparse.get() as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                        {
                            startc = *regparse.get() as uint8_t as ::core::ffi::c_int;
                            let c2rust_fresh1491 = regparse.get();
                            regparse.set((*regparse.ptr()).offset(1));
                            regc(*c2rust_fresh1491 as ::core::ffi::c_int);
                        }
                        while *regparse.get() as ::core::ffi::c_int != NUL
                            && *regparse.get() as ::core::ffi::c_int != ']' as ::core::ffi::c_int
                        {
                            if *regparse.get() as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                                regparse.set((*regparse.ptr()).offset(1));
                                if *regparse.get() as ::core::ffi::c_int
                                    == ']' as ::core::ffi::c_int
                                    || *regparse.get() as ::core::ffi::c_int == NUL
                                    || startc == -1 as ::core::ffi::c_int
                                    || *(*regparse.ptr()).offset(0 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '\\' as ::core::ffi::c_int
                                        && *(*regparse.ptr())
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == 'n' as ::core::ffi::c_int
                                {
                                    regc('-' as ::core::ffi::c_int);
                                    startc = '-' as ::core::ffi::c_int;
                                } else {
                                    endc = 0 as ::core::ffi::c_int;
                                    if *regparse.get() as ::core::ffi::c_int
                                        == '[' as ::core::ffi::c_int
                                    {
                                        endc = get_coll_element(regparse.ptr());
                                    }
                                    if endc == 0 as ::core::ffi::c_int {
                                        endc = mb_ptr2char_adv(
                                            regparse.ptr() as *mut *const ::core::ffi::c_char
                                        );
                                    }
                                    if endc == '\\' as ::core::ffi::c_int && reg_cpo_lit.get() == 0
                                    {
                                        endc = coll_get_char();
                                    }
                                    if startc > endc {
                                        emsg(gettext(E_REVERSE_RANGE.as_ptr()));
                                        rc_did_emsg.set(true_0 != 0);
                                        return NULL_0 as *mut uint8_t;
                                    }
                                    if utf_char2len(startc) > 1 as ::core::ffi::c_int
                                        || utf_char2len(endc) > 1 as ::core::ffi::c_int
                                    {
                                        if endc > startc + 256 as ::core::ffi::c_int {
                                            emsg(gettext(E_LARGE_CLASS.as_ptr()));
                                            rc_did_emsg.set(true_0 != 0);
                                            return NULL_0 as *mut uint8_t;
                                        }
                                        loop {
                                            startc += 1;
                                            if startc > endc {
                                                break;
                                            }
                                            regmbc(startc);
                                        }
                                    } else {
                                        loop {
                                            startc += 1;
                                            if startc > endc {
                                                break;
                                            }
                                            regc(startc);
                                        }
                                    }
                                    startc = -1 as ::core::ffi::c_int;
                                }
                            } else if *regparse.get() as ::core::ffi::c_int
                                == '\\' as ::core::ffi::c_int
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
                                regparse.set((*regparse.ptr()).offset(1));
                                if *regparse.get() as ::core::ffi::c_int
                                    == 'n' as ::core::ffi::c_int
                                {
                                    if ret != JUST_CALC_SIZE {
                                        if *ret as ::core::ffi::c_int == ANYOF {
                                            *ret = (ANYOF + ADD_NL) as uint8_t;
                                            *flagp |= HASNL;
                                        }
                                    }
                                    regparse.set((*regparse.ptr()).offset(1));
                                    startc = -1 as ::core::ffi::c_int;
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
                                        return NULL_0 as *mut uint8_t;
                                    }
                                    if startc == 0 as ::core::ffi::c_int {
                                        regc(0xa as ::core::ffi::c_int);
                                    } else {
                                        regmbc(startc);
                                    }
                                } else {
                                    let c2rust_fresh1492 = regparse.get();
                                    regparse.set((*regparse.ptr()).offset(1));
                                    startc =
                                        backslash_trans(*c2rust_fresh1492 as ::core::ffi::c_int);
                                    regc(startc);
                                }
                            } else if *regparse.get() as ::core::ffi::c_int
                                == '[' as ::core::ffi::c_int
                            {
                                let mut c_class: ::core::ffi::c_int = 0;
                                let mut cu: ::core::ffi::c_int = 0;
                                c_class = get_char_class(regparse.ptr());
                                startc = -1 as ::core::ffi::c_int;
                                match c_class {
                                    99 => {
                                        c_class = get_equi_class(regparse.ptr());
                                        if c_class != 0 as ::core::ffi::c_int {
                                            reg_equi_class(c_class);
                                        } else {
                                            c_class = get_coll_element(regparse.ptr());
                                            if c_class != 0 as ::core::ffi::c_int {
                                                regmbc(c_class);
                                            } else {
                                                let c2rust_fresh1493 = regparse.get();
                                                regparse.set((*regparse.ptr()).offset(1));
                                                startc = *c2rust_fresh1493 as uint8_t
                                                    as ::core::ffi::c_int;
                                                regc(startc);
                                            }
                                        }
                                    }
                                    0 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu < 128 as ::core::ffi::c_int {
                                            if *(*__ctype_b_loc()).offset(cu as isize)
                                                as ::core::ffi::c_int
                                                & _ISalnum as ::core::ffi::c_int
                                                    as ::core::ffi::c_ushort
                                                    as ::core::ffi::c_int
                                                != 0
                                            {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    1 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu < 128 as ::core::ffi::c_int {
                                            if *(*__ctype_b_loc()).offset(cu as isize)
                                                as ::core::ffi::c_int
                                                & _ISalpha as ::core::ffi::c_int
                                                    as ::core::ffi::c_ushort
                                                    as ::core::ffi::c_int
                                                != 0
                                            {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    2 => {
                                        regc(' ' as ::core::ffi::c_int);
                                        regc('\t' as ::core::ffi::c_int);
                                    }
                                    3 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu <= 127 as ::core::ffi::c_int {
                                            if *(*__ctype_b_loc()).offset(cu as isize)
                                                as ::core::ffi::c_int
                                                & _IScntrl as ::core::ffi::c_int
                                                    as ::core::ffi::c_ushort
                                                    as ::core::ffi::c_int
                                                != 0
                                            {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    4 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu <= 127 as ::core::ffi::c_int {
                                            if ascii_isdigit(cu) {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    5 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu <= 127 as ::core::ffi::c_int {
                                            if *(*__ctype_b_loc()).offset(cu as isize)
                                                as ::core::ffi::c_int
                                                & _ISgraph as ::core::ffi::c_int
                                                    as ::core::ffi::c_ushort
                                                    as ::core::ffi::c_int
                                                != 0
                                            {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    6 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu <= 255 as ::core::ffi::c_int {
                                            if mb_islower(cu) as ::core::ffi::c_int != 0
                                                && cu != 170 as ::core::ffi::c_int
                                                && cu != 186 as ::core::ffi::c_int
                                            {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    7 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu <= 255 as ::core::ffi::c_int {
                                            if vim_isprintc(cu) {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    8 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu < 128 as ::core::ffi::c_int {
                                            if *(*__ctype_b_loc()).offset(cu as isize)
                                                as ::core::ffi::c_int
                                                & _ISpunct as ::core::ffi::c_int
                                                    as ::core::ffi::c_ushort
                                                    as ::core::ffi::c_int
                                                != 0
                                            {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    9 => {
                                        cu = 9 as ::core::ffi::c_int;
                                        while cu <= 13 as ::core::ffi::c_int {
                                            regc(cu);
                                            cu += 1;
                                        }
                                        regc(' ' as ::core::ffi::c_int);
                                    }
                                    10 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu <= 255 as ::core::ffi::c_int {
                                            if mb_isupper(cu) {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    11 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu <= 255 as ::core::ffi::c_int {
                                            if ascii_isxdigit(cu) {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    12 => {
                                        regc('\t' as ::core::ffi::c_int);
                                    }
                                    13 => {
                                        regc('\r' as ::core::ffi::c_int);
                                    }
                                    14 => {
                                        regc('\u{8}' as ::core::ffi::c_int);
                                    }
                                    15 => {
                                        regc(ESC);
                                    }
                                    16 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu <= 255 as ::core::ffi::c_int {
                                            if vim_isIDc(cu) {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    17 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu <= 255 as ::core::ffi::c_int {
                                            if reg_iswordc(cu) {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    18 => {
                                        cu = 1 as ::core::ffi::c_int;
                                        while cu <= 255 as ::core::ffi::c_int {
                                            if vim_isfilec(cu) {
                                                regmbc(cu);
                                            }
                                            cu += 1;
                                        }
                                    }
                                    _ => {}
                                }
                            } else {
                                startc = utf_ptr2char(regparse.get());
                                let mut len: ::core::ffi::c_int = utfc_ptr2len(regparse.get());
                                if utf_char2len(startc) != len {
                                    startc = -1 as ::core::ffi::c_int;
                                }
                                loop {
                                    len -= 1;
                                    if len < 0 as ::core::ffi::c_int {
                                        break;
                                    }
                                    let c2rust_fresh1494 = regparse.get();
                                    regparse.set((*regparse.ptr()).offset(1));
                                    regc(*c2rust_fresh1494 as ::core::ffi::c_int);
                                }
                            }
                        }
                        regc(NUL);
                        prevchr_len.set(1 as ::core::ffi::c_int);
                        if *regparse.get() as ::core::ffi::c_int != ']' as ::core::ffi::c_int {
                            emsg(gettext(&raw const e_toomsbra as *const ::core::ffi::c_char));
                            rc_did_emsg.set(true_0 != 0);
                            return NULL_0 as *mut uint8_t;
                        }
                        skipchr();
                        *flagp |= HASWIDTH | SIMPLE;
                        break 's_2192;
                    } else {
                        if reg_strict.get() != 0 {
                            semsg(
                                gettext(E_MISSINGBRACKET.as_ptr()),
                                if reg_magic.get() as ::core::ffi::c_uint
                                    > MAGIC_OFF as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    b"\0".as_ptr() as *const ::core::ffi::c_char
                                } else {
                                    b"\\\0".as_ptr() as *const ::core::ffi::c_char
                                },
                            );
                            rc_did_emsg.set(true_0 != 0);
                            return NULL_0 as *mut uint8_t;
                        }
                        break 's_2080;
                    }
                }
                p = vim_strchr(classchars.get() as *mut ::core::ffi::c_char, no_Magic(c))
                    as *mut uint8_t;
                if p.is_null() {
                    emsg(gettext(E_INVALID_USE_OF_UNDERSCORE.as_ptr()));
                    rc_did_emsg.set(true_0 != 0);
                    return NULL_0 as *mut uint8_t;
                }
                if c == '.' as ::core::ffi::c_int - 256 as ::core::ffi::c_int
                    && utf_iscomposing_legacy(peekchr()) as ::core::ffi::c_int != 0
                {
                    c = getchr();
                    break '_do_multibyte;
                } else {
                    ret = regnode(
                        (*classcodes.ptr())[p.offset_from(classchars.get()) as usize] + extra,
                    );
                    *flagp |= HASWIDTH | SIMPLE;
                    break 's_2192;
                }
            }
            len_0 = 0;
            if !use_multibytecode(c) {
                ret = regnode(EXACTLY);
                len_0 = 0 as ::core::ffi::c_int;
                while c != NUL
                    && (len_0 == 0 as ::core::ffi::c_int
                        || re_multi_type(peekchr()) == NOT_MULTI
                            && one_exactly.get() == 0
                            && !(c < 0 as ::core::ffi::c_int))
                {
                    c = no_Magic(c);
                    regmbc(c);
                    let mut l: ::core::ffi::c_int = 0;
                    let mut state: GraphemeState = GRAPHEME_STATE_INIT as GraphemeState;
                    loop {
                        l = utf_ptr2len(regparse.get());
                        if !utf_composinglike(
                            regparse.get(),
                            (*regparse.ptr()).offset(l as isize),
                            &raw mut state,
                        ) {
                            break;
                        }
                        regmbc(utf_ptr2char(regparse.get()));
                        skipchr();
                    }
                    c = getchr();
                    len_0 += 1;
                }
                ungetchr();
                regc(NUL);
                *flagp |= HASWIDTH;
                if len_0 == 1 as ::core::ffi::c_int {
                    *flagp |= SIMPLE;
                }
                break 's_2192;
            }
        }
        ret = regnode(MULTIBYTECODE);
        regmbc(c);
        *flagp |= HASWIDTH | SIMPLE;
    }
    return ret;
}
