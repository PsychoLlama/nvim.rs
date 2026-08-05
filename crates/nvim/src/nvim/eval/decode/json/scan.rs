//! The two JSON values with a syntax of their own: strings and numbers.
//!
//! Both scan forward from the cursor, report their own errors and pop the
//! finished value themselves.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::super::*;
use super::stack::*;

#[inline(always)]
pub(crate) unsafe extern "C" fn parse_json_string(
    buf: *const ::core::ffi::c_char,
    buf_len: size_t,
    pp: *mut *const ::core::ffi::c_char,
    stack: *mut ValuesStack,
    container_stack: *mut ContainerStack,
    next_map_special: *mut bool,
    didcomma: *mut bool,
    didcolon: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut str: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut fst_in_pair: ::core::ffi::c_int = 0;
        let mut str_end: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut obj: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let e: *const ::core::ffi::c_char = buf.offset(buf_len as isize);
        let mut p: *const ::core::ffi::c_char = *pp;
        let mut len: size_t = 0 as size_t;
        p = p.offset(1);
        let s: *const ::core::ffi::c_char = p;
        let mut ret: ::core::ffi::c_int = OK;
        '_parse_json_string_ret: {
            '_parse_json_string_fail: {
                while p < e && *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int {
                    if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                        p = p.offset(1);
                        if p == e {
                            semsg(
                                gettext(b"E474: Unfinished escape sequence: %.*s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                buf_len as ::core::ffi::c_int,
                                buf,
                            );
                            break '_parse_json_string_fail;
                        } else {
                            match *p as ::core::ffi::c_int {
                                117 => {
                                    if p.offset(4 as ::core::ffi::c_int as isize) >= e {
                                        semsg(
                                            gettext(
                                                b"E474: Unfinished unicode escape sequence: %.*s\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            buf_len as ::core::ffi::c_int,
                                            buf,
                                        );
                                        break '_parse_json_string_fail;
                                    } else if !ascii_isxdigit(
                                        *p.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int,
                                    ) || !ascii_isxdigit(
                                        *p.offset(2 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int,
                                    ) || !ascii_isxdigit(
                                        *p.offset(3 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int,
                                    ) || !ascii_isxdigit(
                                        *p.offset(4 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int,
                                    ) {
                                        semsg(
                                            gettext(
                                                b"E474: Expected four hex digits after \\u: %.*s\0"
                                                    .as_ptr()
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            e.offset_from(
                                                p.offset(-(1 as ::core::ffi::c_int as isize)),
                                            )
                                                as ::core::ffi::c_int,
                                            p.offset(-(1 as ::core::ffi::c_int as isize)),
                                        );
                                        break '_parse_json_string_fail;
                                    } else {
                                        len = len.wrapping_add(3 as size_t);
                                        p = p.offset(5 as ::core::ffi::c_int as isize);
                                    }
                                }
                                92 | 47 | 34 | 116 | 98 | 110 | 114 | 102 => {
                                    len = len.wrapping_add(1);
                                    p = p.offset(1);
                                }
                                _ => {
                                    semsg(
                                        gettext(b"E474: Unknown escape sequence: %.*s\0".as_ptr()
                                            as *const ::core::ffi::c_char),
                                        e.offset_from(p.offset(-(1 as ::core::ffi::c_int as isize)))
                                            as ::core::ffi::c_int,
                                        p.offset(-(1 as ::core::ffi::c_int as isize)),
                                    );
                                    break '_parse_json_string_fail;
                                }
                            }
                        }
                    } else {
                        let mut p_byte: uint8_t = *p as uint8_t;
                        if (p_byte as ::core::ffi::c_int) < 0x20 as ::core::ffi::c_int {
                            semsg(
                            gettext(
                                b"E474: ASCII control characters cannot be present inside string: %.*s\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            e.offset_from(p) as ::core::ffi::c_int,
                            p,
                        );
                            break '_parse_json_string_fail;
                        } else {
                            let ch: ::core::ffi::c_int = utf_ptr2char(p);
                            if ch >= 0x80 as ::core::ffi::c_int
                                && p_byte as ::core::ffi::c_int == ch
                                && !(ch == 0xc3 as ::core::ffi::c_int
                                    && p.offset(1 as ::core::ffi::c_int as isize) < e
                                    && *p.offset(1 as ::core::ffi::c_int as isize) as uint8_t
                                        as ::core::ffi::c_int
                                        == 0x83 as ::core::ffi::c_int)
                            {
                                semsg(
                                    gettext(b"E474: Only UTF-8 strings allowed: %.*s\0".as_ptr()
                                        as *const ::core::ffi::c_char),
                                    e.offset_from(p) as ::core::ffi::c_int,
                                    p,
                                );
                                break '_parse_json_string_fail;
                            } else if ch > 0x10ffff as ::core::ffi::c_int {
                                semsg(
                                gettext(
                                    b"E474: Only UTF-8 code points up to U+10FFFF are allowed to appear unescaped: %.*s\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                ),
                                e.offset_from(p) as ::core::ffi::c_int,
                                p,
                            );
                                break '_parse_json_string_fail;
                            } else {
                                let ch_len: size_t = utf_char2len(ch) as size_t;
                                '_c2rust_label: {
                                    if ch_len
                                        == (if ch != 0 {
                                            utf_ptr2len(p)
                                        } else {
                                            1 as ::core::ffi::c_int
                                        }) as size_t
                                    {
                                    } else {
                                        __assert_fail(
                                        b"ch_len == (size_t)(ch ? utf_ptr2len(p) : 1)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/eval/decode.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        380 as ::core::ffi::c_uint,
                                        b"int parse_json_string(const char *const, const size_t, const char **const, ValuesStack *const, ContainerStack *const, _Bool *const, _Bool *const, _Bool *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                    }
                                };
                                len = len.wrapping_add(ch_len);
                                p = p.offset(ch_len as isize);
                            }
                        }
                    }
                }
                if p == e || *p as ::core::ffi::c_int != '"' as ::core::ffi::c_int {
                    semsg(
                        gettext(b"E474: Expected string end: %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        buf_len as ::core::ffi::c_int,
                        buf,
                    );
                } else {
                    str = xmalloc(len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
                    fst_in_pair = 0 as ::core::ffi::c_int;
                    str_end = str;
                    let mut t: *const ::core::ffi::c_char = s;
                    while t < p {
                        if *t.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            != '\\' as ::core::ffi::c_int
                            || *t.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                != 'u' as ::core::ffi::c_int
                        {
                            if fst_in_pair != 0 as ::core::ffi::c_int {
                                str_end =
                                    str_end.offset(utf_char2bytes(fst_in_pair, str_end) as isize);
                                fst_in_pair = 0 as ::core::ffi::c_int;
                            }
                        }
                        if *t as ::core::ffi::c_int == '\\' as ::core::ffi::c_int {
                            t = t.offset(1);
                            match *t as ::core::ffi::c_int {
                                117 => {
                                    let ubuf: [::core::ffi::c_char; 4] = [
                                        *t.offset(1 as ::core::ffi::c_int as isize),
                                        *t.offset(2 as ::core::ffi::c_int as isize),
                                        *t.offset(3 as ::core::ffi::c_int as isize),
                                        *t.offset(4 as ::core::ffi::c_int as isize),
                                    ];
                                    t = t.offset(4 as ::core::ffi::c_int as isize);
                                    let mut ch_0: uvarnumber_T = 0;
                                    vim_str2nr(
                                        &raw const ubuf as *const ::core::ffi::c_char,
                                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                                        STR2NR_HEX as ::core::ffi::c_int
                                            | STR2NR_FORCE as ::core::ffi::c_int,
                                        ::core::ptr::null_mut::<varnumber_T>(),
                                        &raw mut ch_0,
                                        4 as ::core::ffi::c_int,
                                        true_0 != 0,
                                        ::core::ptr::null_mut::<bool>(),
                                    );
                                    if SURROGATE_HI_START as uvarnumber_T <= ch_0
                                        && ch_0 <= SURROGATE_HI_END as uvarnumber_T
                                    {
                                        if fst_in_pair != 0 as ::core::ffi::c_int {
                                            str_end =
                                                str_end
                                                    .offset(utf_char2bytes(fst_in_pair, str_end)
                                                        as isize);
                                            fst_in_pair = 0 as ::core::ffi::c_int;
                                        }
                                        fst_in_pair = ch_0 as ::core::ffi::c_int;
                                    } else if SURROGATE_LO_START as uvarnumber_T <= ch_0
                                        && ch_0 <= SURROGATE_LO_END as uvarnumber_T
                                        && fst_in_pair != 0 as ::core::ffi::c_int
                                    {
                                        let full_char: ::core::ffi::c_int = ch_0
                                            .wrapping_sub(SURROGATE_LO_START as uvarnumber_T)
                                            as ::core::ffi::c_int
                                            + (fst_in_pair - SURROGATE_HI_START
                                                << 10 as ::core::ffi::c_int)
                                            + SURROGATE_FIRST_CHAR;
                                        str_end = str_end
                                            .offset(utf_char2bytes(full_char, str_end) as isize);
                                        fst_in_pair = 0 as ::core::ffi::c_int;
                                    } else {
                                        if fst_in_pair != 0 as ::core::ffi::c_int {
                                            str_end =
                                                str_end
                                                    .offset(utf_char2bytes(fst_in_pair, str_end)
                                                        as isize);
                                            fst_in_pair = 0 as ::core::ffi::c_int;
                                        }
                                        str_end = str_end.offset(utf_char2bytes(
                                            ch_0 as ::core::ffi::c_int,
                                            str_end,
                                        )
                                            as isize);
                                    }
                                }
                                92 | 47 | 34 | 116 | 98 | 110 | 114 | 102 => {
                                    static escapes: GlobalCell<[::core::ffi::c_char; 117]> =
                                        GlobalCell::new([
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            '"' as ::core::ffi::c_char,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            '/' as ::core::ffi::c_char,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            '\\' as ::core::ffi::c_char,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            BS as ::core::ffi::c_char,
                                            0,
                                            0,
                                            0,
                                            FF as ::core::ffi::c_char,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            0,
                                            NL as ::core::ffi::c_char,
                                            0,
                                            0,
                                            0,
                                            CAR as ::core::ffi::c_char,
                                            0,
                                            TAB as ::core::ffi::c_char,
                                        ]);
                                    let c2rust_fresh6 = str_end;
                                    str_end = str_end.offset(1);
                                    *c2rust_fresh6 =
                                        (*escapes.ptr())[*t as ::core::ffi::c_int as usize];
                                }
                                _ => {
                                    abort();
                                }
                            }
                        } else {
                            let c2rust_fresh7 = str_end;
                            str_end = str_end.offset(1);
                            *c2rust_fresh7 = *t;
                        }
                        t = t.offset(1);
                    }
                    if fst_in_pair != 0 as ::core::ffi::c_int {
                        str_end = str_end.offset(utf_char2bytes(fst_in_pair, str_end) as isize);
                        fst_in_pair = 0 as ::core::ffi::c_int;
                    }
                    *str_end = NUL as ::core::ffi::c_char;
                    obj = decode_string(
                        str,
                        str_end.offset_from(str) as size_t,
                        false_0 != 0,
                        true_0 != 0,
                    );
                    if json_decoder_pop(
                        ValuesStackItem {
                            is_special_string: obj.v_type as ::core::ffi::c_uint
                                != VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint,
                            didcomma: *didcomma,
                            didcolon: *didcolon,
                            val: obj,
                        },
                        stack,
                        container_stack,
                        &raw mut p,
                        next_map_special,
                        didcomma,
                        didcolon,
                    ) != FAIL
                    {
                        if *next_map_special {
                            break '_parse_json_string_ret;
                        } else {
                            break '_parse_json_string_ret;
                        }
                    }
                }
            }
            ret = FAIL;
        }
        *pp = p;
        return ret;
    }
}

#[inline(always)]
pub(crate) unsafe extern "C" fn parse_json_number(
    buf: *const ::core::ffi::c_char,
    buf_len: size_t,
    pp: *mut *const ::core::ffi::c_char,
    stack: *mut ValuesStack,
    container_stack: *mut ContainerStack,
    next_map_special: *mut bool,
    didcomma: *mut bool,
    didcolon: *mut bool,
) -> ::core::ffi::c_int {
    unsafe {
        let mut tv: typval_T = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        let mut exp_num_len: size_t = 0;
        let e: *const ::core::ffi::c_char = buf.offset(buf_len as isize);
        let mut p: *const ::core::ffi::c_char = *pp;
        let mut ret: ::core::ffi::c_int = OK;
        let s: *const ::core::ffi::c_char = p;
        let mut ints: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut fracs: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut exps: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut exps_s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
            p = p.offset(1);
        }
        ints = p;
        '_parse_json_number_ret: {
            '_parse_json_number_fail: {
                '_parse_json_number_check: {
                    if p < e {
                        while p < e
                            && ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
                        {
                            p = p.offset(1);
                        }
                        if p != ints.offset(1 as ::core::ffi::c_int as isize)
                            && *ints as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                        {
                            semsg(
                                gettext(b"E474: Leading zeroes are not allowed: %.*s\0".as_ptr()
                                    as *const ::core::ffi::c_char),
                                e.offset_from(s) as ::core::ffi::c_int,
                                s,
                            );
                            break '_parse_json_number_fail;
                        } else if !(p >= e || p == ints) {
                            if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                                p = p.offset(1);
                                fracs = p;
                                while p < e
                                    && ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int
                                        != 0
                                {
                                    p = p.offset(1);
                                }
                                if p >= e || p == fracs {
                                    break '_parse_json_number_check;
                                }
                            }
                            if *p as ::core::ffi::c_int == 'e' as ::core::ffi::c_int
                                || *p as ::core::ffi::c_int == 'E' as ::core::ffi::c_int
                            {
                                p = p.offset(1);
                                exps_s = p;
                                if p < e
                                    && (*p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                                        || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int)
                                {
                                    p = p.offset(1);
                                }
                                exps = p;
                                while p < e
                                    && ascii_isdigit(*p as ::core::ffi::c_int) as ::core::ffi::c_int
                                        != 0
                                {
                                    p = p.offset(1);
                                }
                            }
                        }
                    }
                }
                if p == ints {
                    semsg(
                        gettext(b"E474: Missing number after minus sign: %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        e.offset_from(s) as ::core::ffi::c_int,
                        s,
                    );
                } else if p == fracs
                    || !fracs.is_null() && exps_s == fracs.offset(1 as ::core::ffi::c_int as isize)
                {
                    semsg(
                        gettext(b"E474: Missing number after decimal dot: %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        e.offset_from(s) as ::core::ffi::c_int,
                        s,
                    );
                } else if p == exps {
                    semsg(
                        gettext(b"E474: Missing exponent: %.*s\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        e.offset_from(s) as ::core::ffi::c_int,
                        s,
                    );
                } else {
                    tv = typval_T {
                        v_type: VAR_NUMBER,
                        v_lock: VAR_UNLOCKED,
                        vval: typval_vval_union { v_number: 0 },
                    };
                    exp_num_len = p.offset_from(s) as size_t;
                    if !fracs.is_null() || !exps.is_null() {
                        let num_len: size_t = string2float(s, &raw mut tv.vval.v_float);
                        if exp_num_len != num_len {
                            semsg(
                            gettext(
                                b"E685: internal error: while converting number \"%.*s\" to float string2float consumed %zu bytes in place of %zu\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            exp_num_len as ::core::ffi::c_int,
                            s,
                            num_len,
                            exp_num_len,
                        );
                        }
                        tv.v_type = VAR_FLOAT;
                    } else {
                        let mut nr: varnumber_T = 0;
                        let mut num_len_0: ::core::ffi::c_int = 0;
                        vim_str2nr(
                            s,
                            ::core::ptr::null_mut::<::core::ffi::c_int>(),
                            &raw mut num_len_0,
                            0 as ::core::ffi::c_int,
                            &raw mut nr,
                            ::core::ptr::null_mut::<uvarnumber_T>(),
                            p.offset_from(s) as ::core::ffi::c_int,
                            true_0 != 0,
                            ::core::ptr::null_mut::<bool>(),
                        );
                        if exp_num_len as ::core::ffi::c_int != num_len_0 {
                            semsg(
                            gettext(
                                b"E685: internal error: while converting number \"%.*s\" to integer vim_str2nr consumed %i bytes in place of %zu\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            ),
                            exp_num_len as ::core::ffi::c_int,
                            s,
                            num_len_0,
                            exp_num_len,
                        );
                        }
                        tv.vval.v_number = nr;
                    }
                    if json_decoder_pop(
                        ValuesStackItem {
                            is_special_string: false,
                            didcomma: *didcomma,
                            didcolon: *didcolon,
                            val: tv,
                        },
                        stack,
                        container_stack,
                        &raw mut p,
                        next_map_special,
                        didcomma,
                        didcolon,
                    ) != FAIL
                    {
                        if *next_map_special {
                            break '_parse_json_number_ret;
                        } else {
                            p = p.offset(-1);
                            break '_parse_json_number_ret;
                        }
                    }
                }
            }
            ret = FAIL;
        }
        *pp = p;
        return ret;
    }
}
