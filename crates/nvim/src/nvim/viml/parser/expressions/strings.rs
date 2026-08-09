use super::*;
use crate::src::nvim::ascii::ascii_isxdigit;

#[inline(always)]
pub(super) fn shifted_pos(pos: ParserPosition, shift: size_t) -> ParserPosition {
    return ParserPosition {
        line: pos.line,
        col: pos.col.wrapping_add(shift),
    };
}
#[inline(always)]
pub(super) fn recol_pos(pos: ParserPosition, new_col: size_t) -> ParserPosition {
    return ParserPosition {
        line: pos.line,
        col: new_col,
    };
}
pub(super) unsafe extern "C" fn parse_quoted_string(
    pstate: *mut ParserState,
    node: *mut ExprASTNode,
    token: LexExprToken,
    is_invalid: bool,
) {
    let pline: ParserLine = *(*pstate).reader.lines.items.add(token.start.line);
    let s: *const ::core::ffi::c_char = pline.data.add(token.start.col);
    let e: *const ::core::ffi::c_char = s
        .add(token.len)
        .offset(-(token.data.str.closed as ::core::ffi::c_int as isize));
    let mut p: *const ::core::ffi::c_char = s.offset(1 as ::core::ffi::c_int as isize);
    let is_double: bool = token.type_0 == kExprLexDoubleQuotedString;
    let mut size: size_t = token
        .len
        .wrapping_sub(token.data.str.closed as size_t)
        .wrapping_sub(1);
    let mut shifts: Vec<StringShift> = Vec::new();
    if !is_double {
        viml_parser_highlight(
            pstate,
            token.start,
            1,
            if is_invalid {
                b"NvimInvalidSingleQuote\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"NvimSingleQuote\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        while p < e {
            let chunk_e: *const ::core::ffi::c_char = memchr(
                p as *const ::core::ffi::c_void,
                '\'' as ::core::ffi::c_int,
                e.offset_from(p) as size_t,
            ) as *const ::core::ffi::c_char;
            if chunk_e.is_null() {
                break;
            }
            size = size.wrapping_sub(1);
            p = chunk_e.offset(2 as ::core::ffi::c_int as isize);
            if !(*pstate).colors.is_null() {
                shifts.push(StringShift {
                    start: token
                        .start
                        .col
                        .wrapping_add(chunk_e.offset_from(s) as size_t),
                    orig_len: 2,
                    act_len: 1,
                    escape_not_known: false,
                });
            }
        }
        (*node).data.str.size = size;
        if size == 0 {
            (*node).data.str.value = ::core::ptr::null_mut::<::core::ffi::c_char>();
        } else {
            let mut v_p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*node).data.str.value = xmallocz(size) as *mut ::core::ffi::c_char;
            v_p = (*node).data.str.value;
            p = s.offset(1 as ::core::ffi::c_int as isize);
            while p < e {
                let chunk_e_0: *const ::core::ffi::c_char = memchr(
                    p as *const ::core::ffi::c_void,
                    '\'' as ::core::ffi::c_int,
                    e.offset_from(p) as size_t,
                )
                    as *const ::core::ffi::c_char;
                if chunk_e_0.is_null() {
                    memcpy(
                        v_p as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        e.offset_from(p) as size_t,
                    );
                    break;
                } else {
                    memcpy(
                        v_p as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        chunk_e_0.offset_from(p) as size_t,
                    );
                    v_p = v_p.add((chunk_e_0.offset_from(p) as size_t).wrapping_add(1));
                    *v_p.offset(-1 as ::core::ffi::c_int as isize) = '\'' as ::core::ffi::c_char;
                    p = chunk_e_0.offset(2 as ::core::ffi::c_int as isize);
                }
            }
        }
    } else {
        viml_parser_highlight(
            pstate,
            token.start,
            1,
            if is_invalid {
                b"NvimInvalidDoubleQuote\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"NvimDoubleQuote\0".as_ptr() as *const ::core::ffi::c_char
            },
        );
        p = s.offset(1 as ::core::ffi::c_int as isize);
        while p < e {
            if *p as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && p.offset(1 as ::core::ffi::c_int as isize) < e
            {
                p = p.offset(1);
                if p.offset(1 as ::core::ffi::c_int as isize) == e {
                    size = size.wrapping_sub(1);
                    break;
                } else {
                    match *p as ::core::ffi::c_int {
                        60 => {
                            size = size.wrapping_add(5);
                        }
                        120 | 88 => {
                            size = size.wrapping_sub(1);
                            if ascii_isxdigit(
                                *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                            ) {
                                size = size.wrapping_sub(1);
                                if p.offset(2 as ::core::ffi::c_int as isize) < e
                                    && ascii_isxdigit(*p.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                {
                                    size = size.wrapping_sub(1);
                                }
                            }
                        }
                        117 | 85 => {
                            let esc_start: *const ::core::ffi::c_char = p;
                            let mut n: size_t =
                                (if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int {
                                    4 as ::core::ffi::c_int
                                } else {
                                    8 as ::core::ffi::c_int
                                }) as size_t;
                            let mut nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                            p = p.offset(1);
                            while p.offset(1 as ::core::ffi::c_int as isize) < e
                                && {
                                    let c2rust_fresh36 = n;
                                    n = n.wrapping_sub(1);
                                    c2rust_fresh36 != 0
                                }
                                && ascii_isxdigit(*p.offset(1 as ::core::ffi::c_int as isize)
                                    as ::core::ffi::c_int)
                            {
                                p = p.offset(1);
                                nr = (nr << 4 as ::core::ffi::c_int)
                                    + hex2nr(*p as ::core::ffi::c_int);
                            }
                            size = size.wrapping_sub(
                                (p.offset_from(
                                    esc_start.offset(-(1 as ::core::ffi::c_int as isize)),
                                ) - utf_char2len(nr) as isize)
                                    as size_t,
                            );
                            p = p.offset(-1);
                        }
                        48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 => {
                            size = size.wrapping_sub(1);
                            p = p.offset(1);
                            if *p as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                && *p as ::core::ffi::c_int <= '7' as ::core::ffi::c_int
                            {
                                size = size.wrapping_sub(1);
                                p = p.offset(1);
                                if p < e
                                    && *p as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                    && *p as ::core::ffi::c_int <= '7' as ::core::ffi::c_int
                                {
                                    size = size.wrapping_sub(1);
                                    p = p.offset(1);
                                }
                            }
                        }
                        _ => {
                            size = size.wrapping_sub(1);
                        }
                    }
                }
            }
            p = p.offset(1);
        }
        if size == 0 {
            (*node).data.str.value = ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*node).data.str.size = 0;
        } else {
            let mut v_p_0: *mut ::core::ffi::c_char =
                ::core::ptr::null_mut::<::core::ffi::c_char>();
            (*node).data.str.value = xmalloc(size) as *mut ::core::ffi::c_char;
            v_p_0 = (*node).data.str.value;
            p = s.offset(1 as ::core::ffi::c_int as isize);
            while p < e {
                let chunk_e_1: *const ::core::ffi::c_char = memchr(
                    p as *const ::core::ffi::c_void,
                    '\\' as ::core::ffi::c_int,
                    e.offset_from(p) as size_t,
                )
                    as *const ::core::ffi::c_char;
                if chunk_e_1.is_null() {
                    memcpy(
                        v_p_0 as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        e.offset_from(p) as size_t,
                    );
                    v_p_0 = v_p_0.offset(e.offset_from(p) as isize);
                    break;
                } else {
                    memcpy(
                        v_p_0 as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        chunk_e_1.offset_from(p) as size_t,
                    );
                    v_p_0 = v_p_0.add(chunk_e_1.offset_from(p) as size_t);
                    p = chunk_e_1.offset(1 as ::core::ffi::c_int as isize);
                    if p == e {
                        let c2rust_fresh37 = v_p_0;
                        v_p_0 = v_p_0.offset(1);
                        *c2rust_fresh37 = '\\' as ::core::ffi::c_char;
                        break;
                    } else {
                        let mut is_unknown: bool = false;
                        let v_p_start: *const ::core::ffi::c_char = v_p_0;
                        match *p as ::core::ffi::c_int {
                            98 => {
                                let c2rust_fresh38 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh38 = '\u{8}' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            101 => {
                                let c2rust_fresh39 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh39 = '\u{1b}' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            102 => {
                                let c2rust_fresh40 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh40 = '\u{c}' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            110 => {
                                let c2rust_fresh41 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh41 = '\n' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            114 => {
                                let c2rust_fresh42 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh42 = '\r' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            116 => {
                                let c2rust_fresh43 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh43 = '\t' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            34 => {
                                let c2rust_fresh44 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh44 = '"' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            92 => {
                                let c2rust_fresh45 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh45 = '\\' as ::core::ffi::c_char;
                                p = p.offset(1);
                            }
                            88 | 120 | 117 | 85 => {
                                if p.offset(1 as ::core::ffi::c_int as isize) < e
                                    && ascii_isxdigit(*p.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                {
                                    let mut n_0: size_t = 0;
                                    let mut nr_0: ::core::ffi::c_int = 0;
                                    let mut is_hex: bool = *p as ::core::ffi::c_int
                                        == 'x' as ::core::ffi::c_int
                                        || *p as ::core::ffi::c_int == 'X' as ::core::ffi::c_int;
                                    if is_hex {
                                        n_0 = 2;
                                    } else if *p as ::core::ffi::c_int == 'u' as ::core::ffi::c_int
                                    {
                                        n_0 = 4;
                                    } else {
                                        n_0 = 8;
                                    }
                                    nr_0 = 0 as ::core::ffi::c_int;
                                    while p.offset(1 as ::core::ffi::c_int as isize) < e
                                        && {
                                            let c2rust_fresh46 = n_0;
                                            n_0 = n_0.wrapping_sub(1);
                                            c2rust_fresh46 != 0
                                        }
                                        && ascii_isxdigit(
                                            *p.offset(1 as ::core::ffi::c_int as isize)
                                                as ::core::ffi::c_int,
                                        )
                                    {
                                        p = p.offset(1);
                                        nr_0 = (nr_0 << 4 as ::core::ffi::c_int)
                                            + hex2nr(*p as ::core::ffi::c_int);
                                    }
                                    p = p.offset(1);
                                    if is_hex {
                                        let c2rust_fresh47 = v_p_0;
                                        v_p_0 = v_p_0.offset(1);
                                        *c2rust_fresh47 = nr_0 as ::core::ffi::c_char;
                                    } else {
                                        v_p_0 = v_p_0.offset(utf_char2bytes(nr_0, v_p_0) as isize);
                                    }
                                } else {
                                    is_unknown = true;
                                    let c2rust_fresh48 = v_p_0;
                                    v_p_0 = v_p_0.offset(1);
                                    *c2rust_fresh48 = *p;
                                    p = p.offset(1);
                                }
                            }
                            48 | 49 | 50 | 51 | 52 | 53 | 54 | 55 => {
                                let c2rust_fresh49 = p;
                                p = p.offset(1);
                                let mut ch: uint8_t = (*c2rust_fresh49 as ::core::ffi::c_int
                                    - '0' as ::core::ffi::c_int)
                                    as uint8_t;
                                if p < e
                                    && *p as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                    && *p as ::core::ffi::c_int <= '7' as ::core::ffi::c_int
                                {
                                    let c2rust_fresh50 = p;
                                    p = p.offset(1);
                                    ch = (((ch as ::core::ffi::c_int) << 3 as ::core::ffi::c_int)
                                        + *c2rust_fresh50 as ::core::ffi::c_int
                                        - '0' as ::core::ffi::c_int)
                                        as uint8_t;
                                    if p < e
                                        && *p as ::core::ffi::c_int >= '0' as ::core::ffi::c_int
                                        && *p as ::core::ffi::c_int <= '7' as ::core::ffi::c_int
                                    {
                                        let c2rust_fresh51 = p;
                                        p = p.offset(1);
                                        ch = (((ch as ::core::ffi::c_int)
                                            << 3 as ::core::ffi::c_int)
                                            + *c2rust_fresh51 as ::core::ffi::c_int
                                            - '0' as ::core::ffi::c_int)
                                            as uint8_t;
                                    }
                                }
                                let c2rust_fresh52 = v_p_0;
                                v_p_0 = v_p_0.offset(1);
                                *c2rust_fresh52 = ch as ::core::ffi::c_char;
                            }
                            60 => {
                                let mut flags: ::core::ffi::c_int = FSK_KEYCODE
                                    as ::core::ffi::c_int
                                    | FSK_IN_STRING as ::core::ffi::c_int;
                                if *p.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                                    != '*' as ::core::ffi::c_int
                                {
                                    flags |= FSK_SIMPLIFY as ::core::ffi::c_int;
                                }
                                let special_len: size_t = trans_special(
                                    &raw mut p,
                                    e.offset_from(p) as size_t,
                                    v_p_0,
                                    flags,
                                    false,
                                    ::core::ptr::null_mut::<bool>(),
                                )
                                    as size_t;
                                if special_len != 0 {
                                    v_p_0 = v_p_0.add(special_len);
                                } else {
                                    is_unknown = true;
                                    mb_copy_char(&raw mut p, &raw mut v_p_0);
                                }
                            }
                            _ => {
                                is_unknown = true;
                                mb_copy_char(&raw mut p, &raw mut v_p_0);
                            }
                        }
                        if !(*pstate).colors.is_null() {
                            shifts.push(StringShift {
                                start: token
                                    .start
                                    .col
                                    .wrapping_add(chunk_e_1.offset_from(s) as size_t),
                                orig_len: p.offset_from(chunk_e_1) as size_t,
                                act_len: v_p_0.offset_from(v_p_start as *mut ::core::ffi::c_char)
                                    as size_t,
                                escape_not_known: is_unknown,
                            });
                        }
                    }
                }
            }
            (*node).data.str.size = v_p_0.offset_from((*node).data.str.value) as size_t;
        }
    }
    if !(*pstate).colors.is_null() {
        let mut next_col: size_t = token.start.col.wrapping_add(1);
        let body_str: *const ::core::ffi::c_char = if is_double {
            if is_invalid {
                b"NvimInvalidDoubleQuotedBody\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"NvimDoubleQuotedBody\0".as_ptr() as *const ::core::ffi::c_char
            }
        } else if is_invalid {
            b"NvimInvalidSingleQuotedBody\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"NvimSingleQuotedBody\0".as_ptr() as *const ::core::ffi::c_char
        };
        let esc_str: *const ::core::ffi::c_char = if is_double {
            if is_invalid {
                b"NvimInvalidDoubleQuotedEscape\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"NvimDoubleQuotedEscape\0".as_ptr() as *const ::core::ffi::c_char
            }
        } else if is_invalid {
            b"NvimInvalidSingleQuotedQuote\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"NvimSingleQuotedQuote\0".as_ptr() as *const ::core::ffi::c_char
        };
        let ukn_esc_str: *const ::core::ffi::c_char = if is_double {
            if is_invalid {
                b"NvimInvalidDoubleQuotedUnknownEscape\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"NvimDoubleQuotedUnknownEscape\0".as_ptr() as *const ::core::ffi::c_char
            }
        } else if is_invalid {
            b"NvimInvalidSingleQuotedUnknownEscape\0".as_ptr() as *const ::core::ffi::c_char
        } else {
            b"NvimSingleQuotedUnknownEscape\0".as_ptr() as *const ::core::ffi::c_char
        };
        let mut i: size_t = 0;
        while i < shifts.len() {
            let cur_shift: StringShift = shifts[i];
            if cur_shift.start > next_col {
                viml_parser_highlight(
                    pstate,
                    recol_pos(token.start, next_col),
                    cur_shift.start.wrapping_sub(next_col),
                    body_str,
                );
            }
            viml_parser_highlight(
                pstate,
                recol_pos(token.start, cur_shift.start),
                cur_shift.orig_len,
                if cur_shift.escape_not_known {
                    ukn_esc_str
                } else {
                    esc_str
                },
            );
            next_col = cur_shift.start.wrapping_add(cur_shift.orig_len);
            i = i.wrapping_add(1);
        }
        if next_col.wrapping_sub(token.start.col)
            < token.len.wrapping_sub(token.data.str.closed as size_t)
        {
            viml_parser_highlight(
                pstate,
                recol_pos(token.start, next_col),
                token
                    .start
                    .col
                    .wrapping_add(token.len)
                    .wrapping_sub(token.data.str.closed as size_t)
                    .wrapping_sub(next_col),
                body_str,
            );
        }
    }
    if token.data.str.closed {
        if is_double {
            viml_parser_highlight(
                pstate,
                shifted_pos(token.start, token.len.wrapping_sub(1)),
                1,
                if is_invalid {
                    b"NvimInvalidDoubleQuote\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"NvimDoubleQuote\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
        } else {
            viml_parser_highlight(
                pstate,
                shifted_pos(token.start, token.len.wrapping_sub(1)),
                1,
                if is_invalid {
                    b"NvimInvalidSingleQuote\0".as_ptr() as *const ::core::ffi::c_char
                } else {
                    b"NvimSingleQuote\0".as_ptr() as *const ::core::ffi::c_char
                },
            );
        }
    }
}
