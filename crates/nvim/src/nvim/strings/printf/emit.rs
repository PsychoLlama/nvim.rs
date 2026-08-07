//! `vim_vsnprintf_typval()`: the formatter itself.
//!
//! One function -- the whole of printf's output side.  It walks the format,
//! and for each conversion fetches the argument (from a `va_list` or a
//! `typval_T` array), renders it into a temporary, then applies the flags:
//! zero and space padding, a forced sign, the alternate form, the precision
//! and the minimum field width.  The `%s`/`%S` arms are the ones that measure
//! in display cells rather than bytes.

// One transpiled body of 1,297 lines -- over the 1,000-line cap on its
// own, so the four-space shift a wrapping block costs would only add to
// it.  Opt out until the rewrite decomposes the function.
#![allow(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn vim_vsnprintf_typval(
    mut str: *mut ::core::ffi::c_char,
    mut str_m: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut ap_start: ::core::ffi::VaList,
    tvs: *mut typval_T,
) -> ::core::ffi::c_int {
    let mut str_l: size_t = 0 as size_t;
    let mut str_avail: bool = str_l < str_m;
    let mut p: *const ::core::ffi::c_char = fmt;
    let mut arg_cur: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut num_posarg: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut arg_idx: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
    let mut ap: ::core::ffi::VaList;
    let mut ap_types: *mut *const ::core::ffi::c_char =
        ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
    if parse_fmt_types(&mut ap_types, &mut num_posarg, fmt, tvs).is_err() {
        return 0 as ::core::ffi::c_int;
    }
    ap = ap_start.clone();
    if p.is_null() {
        p = b"\0".as_ptr() as *const ::core::ffi::c_char;
    }
    '_error: {
        while *p != 0 {
            if *p as ::core::ffi::c_int != '%' as ::core::ffi::c_int {
                let mut n: size_t = xstrchrnul(
                    p.offset(1 as ::core::ffi::c_int as isize),
                    '%' as ::core::ffi::c_char,
                )
                .offset_from(p) as size_t;
                if str_avail {
                    let mut avail: size_t = str_m.wrapping_sub(str_l);
                    memmove(
                        str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                        p as *const ::core::ffi::c_void,
                        if n < avail { n } else { avail },
                    );
                    str_avail = n < avail;
                }
                p = p.offset(n as isize);
                '_c2rust_label: {
                    if n <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                    } else {
                        __assert_fail(
                            b"n <= SIZE_MAX - str_l\0".as_ptr()
                                as *const ::core::ffi::c_char,
                            b"src/nvim/strings.rs\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                            1486 as ::core::ffi::c_uint,
                            b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                .as_ptr() as *const ::core::ffi::c_char,
                        );
                    }
                };
                str_l = str_l.wrapping_add(n);
            } else {
                let mut min_field_width: size_t = 0 as size_t;
                let mut precision: size_t = 0 as size_t;
                let mut zero_padding: bool = false_0 != 0;
                let mut precision_specified: bool = false_0 != 0;
                let mut justify_left: bool = false_0 != 0;
                let mut alternate_form: bool = false_0 != 0;
                let mut force_sign: bool = false_0 != 0;
                let mut space_for_positive: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                let mut length_modifier: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                let mut tmp: [::core::ffi::c_char; 350] = [0; 350];
                let mut str_arg: *const ::core::ffi::c_char =
                    ::core::ptr::null::<::core::ffi::c_char>();
                let mut str_arg_l: size_t = 0;
                let mut uchar_arg: ::core::ffi::c_uchar = 0;
                let mut number_of_zeros_to_pad: size_t = 0 as size_t;
                let mut zero_padding_insertion_ind: size_t = 0 as size_t;
                let mut fmt_spec: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                let mut tofree: *mut ::core::ffi::c_char =
                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                let mut pos_arg: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                p = p.offset(1);
                let mut ptype: *const ::core::ffi::c_char = p;
                while ascii_isdigit(*ptype as ::core::ffi::c_int) {
                    ptype = ptype.offset(1);
                }
                if *ptype as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                    let mut digstart: *const ::core::ffi::c_char = p;
                    let Some(uj) = get_unsigned_int(digstart, &mut p, !tvs.is_null()) else {
                        break '_error;
                    };
                    pos_arg = uj as ::core::ffi::c_int;
                    p = p.offset(1);
                }
                loop {
                    match *p as ::core::ffi::c_int {
                        48 => {
                            zero_padding = true_0 != 0;
                            p = p.offset(1);
                        }
                        45 => {
                            justify_left = true_0 != 0;
                            p = p.offset(1);
                        }
                        43 => {
                            force_sign = true_0 != 0;
                            space_for_positive = 0 as ::core::ffi::c_int;
                            p = p.offset(1);
                        }
                        32 => {
                            force_sign = true_0 != 0;
                            p = p.offset(1);
                        }
                        35 => {
                            alternate_form = true_0 != 0;
                            p = p.offset(1);
                        }
                        39 => {
                            p = p.offset(1);
                        }
                        _ => {
                            break;
                        }
                    }
                }
                if *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                    let mut digstart_0: *const ::core::ffi::c_char =
                        p.offset(1 as ::core::ffi::c_int as isize);
                    p = p.offset(1);
                    if ascii_isdigit(*p as ::core::ffi::c_int) {
                        let Some(uj_0) = get_unsigned_int(digstart_0, &mut p, !tvs.is_null())
                        else {
                            break '_error;
                        };
                        arg_idx = uj_0 as ::core::ffi::c_int;
                        p = p.offset(1);
                    }
                    let mut j: ::core::ffi::c_int = if !tvs.is_null() {
                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_int
                    } else {
                        skip_to_arg(
                            ap_types,
                            ap_start.clone(),
                            &raw mut ap,
                            &raw mut arg_idx,
                            &raw mut arg_cur,
                            fmt,
                        );
                        ap.next_arg::<::core::ffi::c_int>()
                    };
                    if j > MAX_ALLOWED_STRING_WIDTH {
                        if !tvs.is_null() {
                            format_overflow_error(digstart_0);
                            break '_error;
                        } else {
                            j = MAX_ALLOWED_STRING_WIDTH;
                        }
                    }
                    if j >= 0 as ::core::ffi::c_int {
                        min_field_width = j as size_t;
                    } else {
                        min_field_width = -j as size_t;
                        justify_left = true_0 != 0;
                    }
                } else if ascii_isdigit(*p as ::core::ffi::c_int) {
                    let mut digstart_1: *const ::core::ffi::c_char = p;
                    let Some(uj_1) = get_unsigned_int(digstart_1, &mut p, !tvs.is_null()) else {
                        break '_error;
                    };
                    min_field_width = uj_1 as size_t;
                }
                if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                    p = p.offset(1);
                    precision_specified = true_0 != 0;
                    if ascii_isdigit(*p as ::core::ffi::c_int) {
                        let mut digstart_2: *const ::core::ffi::c_char = p;
                        let Some(uj_2) = get_unsigned_int(digstart_2, &mut p, !tvs.is_null())
                        else {
                            break '_error;
                        };
                        precision = uj_2 as size_t;
                    } else if *p as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                        let mut digstart_3: *const ::core::ffi::c_char = p;
                        p = p.offset(1);
                        if ascii_isdigit(*p as ::core::ffi::c_int) {
                            let Some(uj_3) = get_unsigned_int(digstart_3, &mut p, !tvs.is_null())
                            else {
                                break '_error;
                            };
                            arg_idx = uj_3 as ::core::ffi::c_int;
                            p = p.offset(1);
                        }
                        let mut j_0: ::core::ffi::c_int = if !tvs.is_null() {
                            tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_int
                        } else {
                            skip_to_arg(
                                ap_types,
                                ap_start.clone(),
                                &raw mut ap,
                                &raw mut arg_idx,
                                &raw mut arg_cur,
                                fmt,
                            );
                            ap.next_arg::<::core::ffi::c_int>()
                        };
                        if j_0 > MAX_ALLOWED_STRING_WIDTH {
                            if !tvs.is_null() {
                                format_overflow_error(digstart_3);
                                break '_error;
                            } else {
                                j_0 = MAX_ALLOWED_STRING_WIDTH;
                            }
                        }
                        if j_0 >= 0 as ::core::ffi::c_int {
                            precision = j_0 as size_t;
                        } else {
                            precision_specified = false_0 != 0;
                            precision = 0 as size_t;
                        }
                    }
                }
                if *p as ::core::ffi::c_int == 'h' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                    || *p as ::core::ffi::c_int == 'z' as ::core::ffi::c_int
                {
                    length_modifier = *p;
                    p = p.offset(1);
                    if length_modifier as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                        && *p as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                    {
                        length_modifier = 'L' as ::core::ffi::c_char;
                        p = p.offset(1);
                    }
                }
                fmt_spec = *p;
                match fmt_spec as ::core::ffi::c_int {
                    105 => {
                        fmt_spec = 'd' as ::core::ffi::c_char;
                    }
                    68 => {
                        fmt_spec = 'd' as ::core::ffi::c_char;
                        length_modifier = 'l' as ::core::ffi::c_char;
                    }
                    85 => {
                        fmt_spec = 'u' as ::core::ffi::c_char;
                        length_modifier = 'l' as ::core::ffi::c_char;
                    }
                    79 => {
                        fmt_spec = 'o' as ::core::ffi::c_char;
                        length_modifier = 'l' as ::core::ffi::c_char;
                    }
                    _ => {}
                }
                match fmt_spec as ::core::ffi::c_int {
                    100 | 117 | 111 | 120 | 88 => {
                        if !tvs.is_null() && length_modifier as ::core::ffi::c_int == NUL {
                            length_modifier = 'L' as ::core::ffi::c_char;
                        }
                    }
                    _ => {}
                }
                if pos_arg != -1 as ::core::ffi::c_int {
                    arg_idx = pos_arg;
                }
                match fmt_spec as ::core::ffi::c_int {
                    37 | 99 | 115 | 83 => {
                        str_arg_l = 1 as size_t;
                        match fmt_spec as ::core::ffi::c_int {
                            37 => {
                                str_arg = p;
                            }
                            99 => {
                                let j_1: ::core::ffi::c_int = if !tvs.is_null() {
                                    tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_int
                                } else {
                                    skip_to_arg(
                                        ap_types,
                                        ap_start.clone(),
                                        &raw mut ap,
                                        &raw mut arg_idx,
                                        &raw mut arg_cur,
                                        fmt,
                                    );
                                    ap.next_arg::<::core::ffi::c_int>()
                                };
                                uchar_arg = j_1 as ::core::ffi::c_uchar;
                                str_arg = &raw mut uchar_arg as *mut ::core::ffi::c_char;
                            }
                            115 | 83 => {
                                str_arg = if !tvs.is_null() {
                                    tv_str(tvs, &raw mut arg_idx, &raw mut tofree)
                                } else {
                                    skip_to_arg(
                                        ap_types,
                                        ap_start.clone(),
                                        &raw mut ap,
                                        &raw mut arg_idx,
                                        &raw mut arg_cur,
                                        fmt,
                                    );
                                    ap.next_arg::<*const ::core::ffi::c_char>()
                                };
                                if str_arg.is_null() {
                                    str_arg = b"[NULL]\0".as_ptr() as *const ::core::ffi::c_char;
                                    str_arg_l = 6 as size_t;
                                } else if !precision_specified {
                                    str_arg_l = strlen(str_arg);
                                } else if precision == 0 as size_t {
                                    str_arg_l = 0 as size_t;
                                } else {
                                    str_arg_l = (xmemscan(
                                        str_arg as *const ::core::ffi::c_void,
                                        NUL as ::core::ffi::c_char,
                                        if precision < 0x7fffffff as ::core::ffi::c_int as size_t {
                                            precision
                                        } else {
                                            0x7fffffff as ::core::ffi::c_int as size_t
                                        },
                                    )
                                        as *mut ::core::ffi::c_char)
                                        .offset_from(str_arg)
                                        as size_t;
                                }
                                if fmt_spec as ::core::ffi::c_int == 'S' as ::core::ffi::c_int {
                                    let mut p1: *const ::core::ffi::c_char =
                                        ::core::ptr::null::<::core::ffi::c_char>();
                                    let mut i: size_t = 0;
                                    i = 0 as size_t;
                                    p1 = str_arg;
                                    while *p1 != 0 {
                                        let mut cell: size_t = utf_ptr2cells(p1) as size_t;
                                        if precision_specified as ::core::ffi::c_int != 0
                                            && i.wrapping_add(cell) > precision
                                        {
                                            break;
                                        }
                                        i = i.wrapping_add(cell);
                                        p1 = p1.offset(utfc_ptr2len(p1) as isize);
                                    }
                                    str_arg_l = p1.offset_from(str_arg) as size_t;
                                    if min_field_width != 0 as size_t {
                                        min_field_width =
                                            min_field_width.wrapping_add(str_arg_l.wrapping_sub(i));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    100 | 117 | 98 | 66 | 111 | 120 | 88 | 112 => {
                        let mut arg_sign: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                        let mut arg: intmax_t = 0 as intmax_t;
                        let mut uarg: uintmax_t = 0 as uintmax_t;
                        let mut ptr_arg: *const ::core::ffi::c_void =
                            ::core::ptr::null::<::core::ffi::c_void>();
                        if fmt_spec as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
                            ptr_arg = if !tvs.is_null() {
                                tv_ptr(tvs, &raw mut arg_idx)
                            } else {
                                skip_to_arg(
                                    ap_types,
                                    ap_start.clone(),
                                    &raw mut ap,
                                    &raw mut arg_idx,
                                    &raw mut arg_cur,
                                    fmt,
                                );
                                ap.next_arg::<*mut ::core::ffi::c_void>()
                                    as *const ::core::ffi::c_void
                            };
                            if !ptr_arg.is_null() {
                                arg_sign = 1 as ::core::ffi::c_int;
                            }
                        } else if fmt_spec as ::core::ffi::c_int == 'b' as ::core::ffi::c_int
                            || fmt_spec as ::core::ffi::c_int == 'B' as ::core::ffi::c_int
                        {
                            uarg = (if !tvs.is_null() {
                                tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_ulonglong
                            } else {
                                skip_to_arg(
                                    ap_types,
                                    ap_start.clone(),
                                    &raw mut ap,
                                    &raw mut arg_idx,
                                    &raw mut arg_cur,
                                    fmt,
                                );
                                ap.next_arg::<::core::ffi::c_ulonglong>()
                            }) as uintmax_t;
                            arg_sign = (uarg != 0 as uintmax_t) as ::core::ffi::c_int;
                        } else if fmt_spec as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                            match length_modifier as ::core::ffi::c_int {
                                NUL => {
                                    arg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_int
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_int>()
                                    }) as intmax_t;
                                }
                                104 => {
                                    arg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_int
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_int>()
                                    }) as int16_t
                                        as intmax_t;
                                }
                                108 => {
                                    arg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_long
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_long>()
                                    }) as intmax_t;
                                }
                                76 => {
                                    arg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_longlong
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_longlong>()
                                    }) as intmax_t;
                                }
                                122 => {
                                    arg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ptrdiff_t
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<ptrdiff_t>()
                                    }) as intmax_t;
                                }
                                _ => {}
                            }
                            if arg > 0 as intmax_t {
                                arg_sign = 1 as ::core::ffi::c_int;
                            } else if arg < 0 as intmax_t {
                                arg_sign = -1 as ::core::ffi::c_int;
                            }
                        } else {
                            match length_modifier as ::core::ffi::c_int {
                                NUL => {
                                    uarg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_uint
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_uint>()
                                    }) as uintmax_t;
                                }
                                104 => {
                                    uarg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_uint
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_uint>()
                                    }) as uint16_t
                                        as uintmax_t;
                                }
                                108 => {
                                    uarg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_ulong
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_ulong>()
                                    }) as uintmax_t;
                                }
                                76 => {
                                    uarg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as ::core::ffi::c_ulonglong
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<::core::ffi::c_ulonglong>()
                                    }) as uintmax_t;
                                }
                                122 => {
                                    uarg = (if !tvs.is_null() {
                                        tv_nr(tvs, &raw mut arg_idx) as size_t
                                    } else {
                                        skip_to_arg(
                                            ap_types,
                                            ap_start.clone(),
                                            &raw mut ap,
                                            &raw mut arg_idx,
                                            &raw mut arg_cur,
                                            fmt,
                                        );
                                        ap.next_arg::<size_t>()
                                    }) as uintmax_t;
                                }
                                _ => {}
                            }
                            arg_sign = (uarg != 0 as uintmax_t) as ::core::ffi::c_int;
                        }
                        str_arg = &raw mut tmp as *mut ::core::ffi::c_char;
                        str_arg_l = 0 as size_t;
                        if precision_specified {
                            zero_padding = false_0 != 0;
                        }
                        if fmt_spec as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                            if force_sign as ::core::ffi::c_int != 0
                                && arg_sign >= 0 as ::core::ffi::c_int
                            {
                                let c2rust_fresh27 = str_arg_l;
                                str_arg_l = str_arg_l.wrapping_add(1);
                                tmp[c2rust_fresh27 as usize] = (if space_for_positive != 0 {
                                    ' ' as ::core::ffi::c_int
                                } else {
                                    '+' as ::core::ffi::c_int
                                })
                                    as ::core::ffi::c_char;
                            }
                        } else if alternate_form {
                            if arg_sign != 0 as ::core::ffi::c_int
                                && (fmt_spec as ::core::ffi::c_int == 'x' as ::core::ffi::c_int
                                    || fmt_spec as ::core::ffi::c_int == 'X' as ::core::ffi::c_int
                                    || fmt_spec as ::core::ffi::c_int == 'b' as ::core::ffi::c_int
                                    || fmt_spec as ::core::ffi::c_int == 'B' as ::core::ffi::c_int)
                            {
                                let c2rust_fresh28 = str_arg_l;
                                str_arg_l = str_arg_l.wrapping_add(1);
                                tmp[c2rust_fresh28 as usize] = '0' as ::core::ffi::c_char;
                                let c2rust_fresh29 = str_arg_l;
                                str_arg_l = str_arg_l.wrapping_add(1);
                                tmp[c2rust_fresh29 as usize] = fmt_spec;
                            }
                        }
                        zero_padding_insertion_ind = str_arg_l;
                        if !precision_specified {
                            precision = 1 as size_t;
                        }
                        if !(precision == 0 as size_t && arg_sign == 0 as ::core::ffi::c_int) {
                            match fmt_spec as ::core::ffi::c_int {
                                112 => {
                                    str_arg_l = str_arg_l.wrapping_add(snprintf(
                                        (&raw mut tmp as *mut ::core::ffi::c_char)
                                            .offset(str_arg_l as isize),
                                        ::core::mem::size_of::<[::core::ffi::c_char; 350]>()
                                            .wrapping_sub(str_arg_l),
                                        b"%p\0".as_ptr() as *const ::core::ffi::c_char,
                                        ptr_arg,
                                    )
                                        as size_t);
                                }
                                100 => {
                                    str_arg_l = str_arg_l.wrapping_add(snprintf(
                                        (&raw mut tmp as *mut ::core::ffi::c_char)
                                            .offset(str_arg_l as isize),
                                        ::core::mem::size_of::<[::core::ffi::c_char; 350]>()
                                            .wrapping_sub(str_arg_l),
                                        b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                                        arg,
                                    )
                                        as size_t);
                                }
                                98 | 66 => {
                                    let mut bits: size_t = 0 as size_t;
                                    bits = ::core::mem::size_of::<uintmax_t>()
                                        .wrapping_mul(8 as usize)
                                        as size_t;
                                    while bits > 0 as size_t {
                                        if uarg >> bits.wrapping_sub(1 as size_t) & 0x1 as uintmax_t
                                            != 0
                                        {
                                            break;
                                        }
                                        bits = bits.wrapping_sub(1);
                                    }
                                    while bits > 0 as size_t {
                                        bits = bits.wrapping_sub(1);
                                        let c2rust_fresh30 = str_arg_l;
                                        str_arg_l = str_arg_l.wrapping_add(1);
                                        tmp[c2rust_fresh30 as usize] =
                                            (if uarg >> bits & 0x1 as uintmax_t != 0 {
                                                '1' as ::core::ffi::c_int
                                            } else {
                                                '0' as ::core::ffi::c_int
                                            })
                                                as ::core::ffi::c_char;
                                    }
                                }
                                _ => {
                                    let mut f: [::core::ffi::c_char; 4] = ::core::mem::transmute::<
                                        [u8; 4],
                                        [::core::ffi::c_char; 4],
                                    >(
                                        *b"%lu\0"
                                    );
                                    f[::core::mem::size_of::<[::core::ffi::c_char; 4]>()
                                        .wrapping_sub(1 as usize)
                                        .wrapping_sub(1 as usize)
                                        as usize] = fmt_spec;
                                    '_c2rust_label_0: {
                                        if ::core::mem::transmute::<[u8; 3], [::core::ffi::c_char; 3]>(
                                            *b"lu\0",
                                        )
                                            [::core::mem::size_of::<[::core::ffi::c_char; 3]>()
                                                .wrapping_sub(1 as usize)
                                                .wrapping_sub(1 as usize)
                                                as usize]
                                            as ::core::ffi::c_int
                                            == 'u' as ::core::ffi::c_int
                                        {
                                        } else {
                                            __assert_fail(
                                                b"PRIuMAX[sizeof(PRIuMAX) - 1 - 1] == 'u'\0".as_ptr()
                                                    as *const ::core::ffi::c_char,
                                                b"src/nvim/strings.rs\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                                2001 as ::core::ffi::c_uint,
                                                b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                                    .as_ptr() as *const ::core::ffi::c_char,
                                            );
                                        }
                                    };
                                    str_arg_l = str_arg_l.wrapping_add(snprintf(
                                        (&raw mut tmp as *mut ::core::ffi::c_char)
                                            .offset(str_arg_l as isize),
                                        ::core::mem::size_of::<[::core::ffi::c_char; 350]>()
                                            .wrapping_sub(str_arg_l),
                                        &raw mut f as *mut ::core::ffi::c_char,
                                        uarg,
                                    )
                                        as size_t);
                                }
                            }
                            '_c2rust_label_1: {
                                if str_arg_l < ::core::mem::size_of::<[::core::ffi::c_char; 350]>()
                                {
                                } else {
                                    __assert_fail(
                                        b"str_arg_l < sizeof(tmp)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/strings.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2008 as ::core::ffi::c_uint,
                                        b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            // The direct `tmp[i] = ...` stores above created
                            // fresh borrows of `tmp` that invalidate the raw
                            // pointer taken before the match (Stacked Borrows);
                            // re-take it after the last store, as the float
                            // branch does. Everything below only reads `tmp`.
                            str_arg = &raw mut tmp as *mut ::core::ffi::c_char;
                            if zero_padding_insertion_ind < str_arg_l
                                && tmp[zero_padding_insertion_ind as usize] as ::core::ffi::c_int
                                    == '-' as ::core::ffi::c_int
                            {
                                zero_padding_insertion_ind =
                                    zero_padding_insertion_ind.wrapping_add(1);
                            }
                            if zero_padding_insertion_ind.wrapping_add(1 as size_t) < str_arg_l
                                && tmp[zero_padding_insertion_ind as usize] as ::core::ffi::c_int
                                    == '0' as ::core::ffi::c_int
                                && (tmp
                                    [zero_padding_insertion_ind.wrapping_add(1 as size_t) as usize]
                                    as ::core::ffi::c_int
                                    == 'x' as ::core::ffi::c_int
                                    || tmp[zero_padding_insertion_ind.wrapping_add(1 as size_t)
                                        as usize]
                                        as ::core::ffi::c_int
                                        == 'X' as ::core::ffi::c_int
                                    || tmp[zero_padding_insertion_ind.wrapping_add(1 as size_t)
                                        as usize]
                                        as ::core::ffi::c_int
                                        == 'b' as ::core::ffi::c_int
                                    || tmp[zero_padding_insertion_ind.wrapping_add(1 as size_t)
                                        as usize]
                                        as ::core::ffi::c_int
                                        == 'B' as ::core::ffi::c_int)
                            {
                                zero_padding_insertion_ind =
                                    zero_padding_insertion_ind.wrapping_add(2 as size_t);
                            }
                        }
                        let mut num_of_digits: size_t =
                            str_arg_l.wrapping_sub(zero_padding_insertion_ind);
                        if alternate_form as ::core::ffi::c_int != 0
                            && fmt_spec as ::core::ffi::c_int == 'o' as ::core::ffi::c_int
                            && !(zero_padding_insertion_ind < str_arg_l
                                && tmp[zero_padding_insertion_ind as usize] as ::core::ffi::c_int
                                    == '0' as ::core::ffi::c_int)
                        {
                            if !precision_specified
                                || precision < num_of_digits.wrapping_add(1 as size_t)
                            {
                                precision = num_of_digits.wrapping_add(1 as size_t);
                            }
                        }
                        if num_of_digits < precision {
                            number_of_zeros_to_pad = precision.wrapping_sub(num_of_digits);
                        }
                        if !justify_left && zero_padding as ::core::ffi::c_int != 0 {
                            let n_0: ::core::ffi::c_int = min_field_width
                                .wrapping_sub(str_arg_l.wrapping_add(number_of_zeros_to_pad))
                                as ::core::ffi::c_int;
                            if n_0 > 0 as ::core::ffi::c_int {
                                number_of_zeros_to_pad =
                                    number_of_zeros_to_pad.wrapping_add(n_0 as size_t);
                            }
                        }
                    }
                    102 | 70 | 101 | 69 | 103 | 71 => {
                        let mut format: [::core::ffi::c_char; 40] = [0; 40];
                        let mut remove_trailing_zeroes: bool = false_0 != 0;
                        let mut f_0: ::core::ffi::c_double = if !tvs.is_null() {
                            tv_float(tvs, &raw mut arg_idx)
                        } else {
                            skip_to_arg(
                                ap_types,
                                ap_start.clone(),
                                &raw mut ap,
                                &raw mut arg_idx,
                                &raw mut arg_cur,
                                fmt,
                            );
                            ap.next_arg::<::core::ffi::c_double>()
                        };
                        let mut abs_f: ::core::ffi::c_double =
                            if f_0 < 0 as ::core::ffi::c_int as ::core::ffi::c_double {
                                -f_0
                            } else {
                                f_0
                            };
                        if fmt_spec as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
                            || fmt_spec as ::core::ffi::c_int == 'G' as ::core::ffi::c_int
                        {
                            if abs_f >= 0.001f64 && abs_f < 10000000.0f64 || abs_f == 0.0f64 {
                                fmt_spec = (if fmt_spec as ::core::ffi::c_uint
                                    >= 'A' as ::core::ffi::c_uint
                                    && fmt_spec as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                                {
                                    'F' as ::core::ffi::c_int
                                } else {
                                    'f' as ::core::ffi::c_int
                                })
                                    as ::core::ffi::c_char;
                            } else {
                                fmt_spec =
                                    (if fmt_spec as ::core::ffi::c_int == 'g' as ::core::ffi::c_int
                                    {
                                        'e' as ::core::ffi::c_int
                                    } else {
                                        'E' as ::core::ffi::c_int
                                    }) as ::core::ffi::c_char;
                            }
                            remove_trailing_zeroes = true_0 != 0;
                        }
                        if f_0.is_infinite()
                            || !strchr(
                                b"fF\0".as_ptr() as *const ::core::ffi::c_char,
                                fmt_spec as ::core::ffi::c_int,
                            )
                            .is_null()
                                && abs_f > 1.0e307f64
                        {
                            xstrlcpy(
                                &raw mut tmp as *mut ::core::ffi::c_char,
                                infinity_str(
                                    f_0 > 0.0f64,
                                    fmt_spec,
                                    force_sign as ::core::ffi::c_int,
                                    space_for_positive,
                                ),
                                ::core::mem::size_of::<[::core::ffi::c_char; 350]>(),
                            );
                            str_arg_l = strlen(&raw mut tmp as *mut ::core::ffi::c_char);
                            zero_padding = false_0 != 0;
                        } else if f_0.is_nan() {
                            memmove(
                                &raw mut tmp as *mut ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                (if fmt_spec as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
                                    && fmt_spec as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
                                {
                                    b"NAN\0".as_ptr() as *const ::core::ffi::c_char
                                } else {
                                    b"nan\0".as_ptr() as *const ::core::ffi::c_char
                                }) as *const ::core::ffi::c_void,
                                4 as size_t,
                            );
                            str_arg_l = 3 as size_t;
                            zero_padding = false_0 != 0;
                        } else {
                            format[0 as ::core::ffi::c_int as usize] = '%' as ::core::ffi::c_char;
                            let mut l: size_t = 1 as size_t;
                            if force_sign {
                                let c2rust_fresh31 = l;
                                l = l.wrapping_add(1);
                                format[c2rust_fresh31 as usize] = (if space_for_positive != 0 {
                                    ' ' as ::core::ffi::c_int
                                } else {
                                    '+' as ::core::ffi::c_int
                                })
                                    as ::core::ffi::c_char;
                            }
                            if precision_specified {
                                let mut max_prec: size_t =
                                    (TMP_LEN - 10 as ::core::ffi::c_int) as size_t;
                                if (fmt_spec as ::core::ffi::c_int == 'f' as ::core::ffi::c_int
                                    || fmt_spec as ::core::ffi::c_int == 'F' as ::core::ffi::c_int)
                                    && abs_f > 1.0f64
                                {
                                    max_prec = max_prec.wrapping_sub(log10(abs_f) as size_t);
                                }
                                if precision > max_prec {
                                    precision = max_prec;
                                }
                                l = l.wrapping_add(snprintf(
                                    (&raw mut format as *mut ::core::ffi::c_char)
                                        .offset(l as isize),
                                    ::core::mem::size_of::<[::core::ffi::c_char; 40]>()
                                        .wrapping_sub(l),
                                    b".%d\0".as_ptr() as *const ::core::ffi::c_char,
                                    precision as ::core::ffi::c_int,
                                ) as size_t);
                            }
                            '_c2rust_label_2: {
                                if l.wrapping_add(1 as size_t)
                                    < ::core::mem::size_of::<[::core::ffi::c_char; 40]>()
                                {
                                } else {
                                    __assert_fail(
                                        b"l + 1 < sizeof(format)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/strings.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2119 as ::core::ffi::c_uint,
                                        b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            format[l as usize] =
                                (if fmt_spec as ::core::ffi::c_int == 'F' as ::core::ffi::c_int {
                                    'f' as ::core::ffi::c_int
                                } else {
                                    fmt_spec as ::core::ffi::c_int
                                }) as ::core::ffi::c_char;
                            format[l.wrapping_add(1 as size_t) as usize] =
                                NUL as ::core::ffi::c_char;
                            str_arg_l = snprintf(
                                &raw mut tmp as *mut ::core::ffi::c_char,
                                ::core::mem::size_of::<[::core::ffi::c_char; 350]>(),
                                &raw mut format as *mut ::core::ffi::c_char,
                                f_0,
                            ) as size_t;
                            '_c2rust_label_3: {
                                if str_arg_l < ::core::mem::size_of::<[::core::ffi::c_char; 350]>()
                                {
                                } else {
                                    __assert_fail(
                                        b"str_arg_l < sizeof(tmp)\0".as_ptr()
                                            as *const ::core::ffi::c_char,
                                        b"src/nvim/strings.rs\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                        2124 as ::core::ffi::c_uint,
                                        b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                            .as_ptr() as *const ::core::ffi::c_char,
                                    );
                                }
                            };
                            if remove_trailing_zeroes {
                                let mut tp: *mut ::core::ffi::c_char =
                                    ::core::ptr::null_mut::<::core::ffi::c_char>();
                                if fmt_spec as ::core::ffi::c_int == 'f' as ::core::ffi::c_int
                                    || fmt_spec as ::core::ffi::c_int == 'F' as ::core::ffi::c_int
                                {
                                    tp = (&raw mut tmp as *mut ::core::ffi::c_char)
                                        .offset(str_arg_l as isize)
                                        .offset(-(1 as ::core::ffi::c_int as isize));
                                } else {
                                    tp = vim_strchr(
                                        &raw mut tmp as *mut ::core::ffi::c_char,
                                        if fmt_spec as ::core::ffi::c_int
                                            == 'e' as ::core::ffi::c_int
                                        {
                                            'e' as ::core::ffi::c_int
                                        } else {
                                            'E' as ::core::ffi::c_int
                                        },
                                    );
                                    if !tp.is_null() {
                                        if *tp.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '+' as ::core::ffi::c_int
                                        {
                                            memmove(
                                                tp.offset(1 as ::core::ffi::c_int as isize)
                                                    as *mut ::core::ffi::c_void,
                                                tp.offset(2 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                strlen(tp.offset(2 as ::core::ffi::c_int as isize))
                                                    .wrapping_add(1 as size_t),
                                            );
                                            str_arg_l = str_arg_l.wrapping_sub(1);
                                        }
                                        let mut i_0: ::core::ffi::c_int = if *tp
                                            .offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '-' as ::core::ffi::c_int
                                        {
                                            2 as ::core::ffi::c_int
                                        } else {
                                            1 as ::core::ffi::c_int
                                        };
                                        while *tp.offset(i_0 as isize) as ::core::ffi::c_int
                                            == '0' as ::core::ffi::c_int
                                        {
                                            memmove(
                                                tp.offset(i_0 as isize) as *mut ::core::ffi::c_void,
                                                tp.offset(i_0 as isize)
                                                    .offset(1 as ::core::ffi::c_int as isize)
                                                    as *const ::core::ffi::c_void,
                                                strlen(
                                                    tp.offset(i_0 as isize)
                                                        .offset(1 as ::core::ffi::c_int as isize),
                                                )
                                                .wrapping_add(1 as size_t),
                                            );
                                            str_arg_l = str_arg_l.wrapping_sub(1);
                                        }
                                        tp = tp.offset(-1);
                                    }
                                }
                                if !tp.is_null() && !precision_specified {
                                    while tp
                                        > (&raw mut tmp as *mut ::core::ffi::c_char)
                                            .offset(2 as ::core::ffi::c_int as isize)
                                        && *tp as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                                        && *tp.offset(-1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            != '.' as ::core::ffi::c_int
                                    {
                                        memmove(
                                            tp as *mut ::core::ffi::c_void,
                                            tp.offset(1 as ::core::ffi::c_int as isize)
                                                as *const ::core::ffi::c_void,
                                            strlen(tp.offset(1 as ::core::ffi::c_int as isize))
                                                .wrapping_add(1 as size_t),
                                        );
                                        tp = tp.offset(-1);
                                        str_arg_l = str_arg_l.wrapping_sub(1);
                                    }
                                }
                            } else {
                                let mut tp_0: *mut ::core::ffi::c_char = vim_strchr(
                                    &raw mut tmp as *mut ::core::ffi::c_char,
                                    if fmt_spec as ::core::ffi::c_int == 'e' as ::core::ffi::c_int {
                                        'e' as ::core::ffi::c_int
                                    } else {
                                        'E' as ::core::ffi::c_int
                                    },
                                );
                                if !tp_0.is_null()
                                    && (*tp_0.offset(1 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '+' as ::core::ffi::c_int
                                        || *tp_0.offset(1 as ::core::ffi::c_int as isize)
                                            as ::core::ffi::c_int
                                            == '-' as ::core::ffi::c_int)
                                    && *tp_0.offset(2 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int
                                        == '0' as ::core::ffi::c_int
                                    && ascii_isdigit(*tp_0.offset(3 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0
                                    && ascii_isdigit(*tp_0.offset(4 as ::core::ffi::c_int as isize)
                                        as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                        != 0
                                {
                                    memmove(
                                        tp_0.offset(2 as ::core::ffi::c_int as isize)
                                            as *mut ::core::ffi::c_void,
                                        tp_0.offset(3 as ::core::ffi::c_int as isize)
                                            as *const ::core::ffi::c_void,
                                        strlen(tp_0.offset(3 as ::core::ffi::c_int as isize))
                                            .wrapping_add(1 as size_t),
                                    );
                                    str_arg_l = str_arg_l.wrapping_sub(1);
                                }
                            }
                        }
                        if zero_padding as ::core::ffi::c_int != 0
                            && min_field_width > str_arg_l
                            && (tmp[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                                == '-' as ::core::ffi::c_int
                                || force_sign as ::core::ffi::c_int != 0)
                        {
                            number_of_zeros_to_pad = min_field_width.wrapping_sub(str_arg_l);
                            zero_padding_insertion_ind = 1 as size_t;
                        }
                        str_arg = &raw mut tmp as *mut ::core::ffi::c_char;
                    }
                    _ => {
                        zero_padding = false_0 != 0;
                        justify_left = true_0 != 0;
                        min_field_width = 0 as size_t;
                        str_arg = p;
                        str_arg_l = 0 as size_t;
                        if *p != 0 {
                            str_arg_l = str_arg_l.wrapping_add(1);
                        }
                    }
                }
                if *p != 0 {
                    p = p.offset(1);
                }
                if !justify_left {
                    '_c2rust_label_4: {
                        if str_arg_l
                            <= (18446744073709551615 as size_t).wrapping_sub(number_of_zeros_to_pad)
                        {
                        } else {
                            __assert_fail(
                                b"str_arg_l <= SIZE_MAX - number_of_zeros_to_pad\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/strings.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2204 as ::core::ffi::c_uint,
                                b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if min_field_width > str_arg_l.wrapping_add(number_of_zeros_to_pad) {
                        let mut pn: size_t = min_field_width
                            .wrapping_sub(str_arg_l.wrapping_add(number_of_zeros_to_pad));
                        if str_avail {
                            let mut avail_0: size_t = str_m.wrapping_sub(str_l);
                            memset(
                                str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                                if zero_padding as ::core::ffi::c_int != 0 {
                                    '0' as ::core::ffi::c_int
                                } else {
                                    ' ' as ::core::ffi::c_int
                                },
                                if pn < avail_0 { pn } else { avail_0 },
                            );
                            str_avail = pn < avail_0;
                        }
                        '_c2rust_label_5: {
                            if pn <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                            } else {
                                __assert_fail(
                                    b"pn <= SIZE_MAX - str_l\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/strings.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2213 as ::core::ffi::c_uint,
                                    b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        str_l = str_l.wrapping_add(pn);
                    }
                }
                if number_of_zeros_to_pad == 0 as size_t {
                    zero_padding_insertion_ind = 0 as size_t;
                } else {
                    if zero_padding_insertion_ind > 0 as size_t {
                        let mut zn: size_t = zero_padding_insertion_ind;
                        if str_avail {
                            let mut avail_1: size_t = str_m.wrapping_sub(str_l);
                            memmove(
                                str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                                str_arg as *const ::core::ffi::c_void,
                                if zn < avail_1 { zn } else { avail_1 },
                            );
                            str_avail = zn < avail_1;
                        }
                        '_c2rust_label_6: {
                            if zn <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                            } else {
                                __assert_fail(
                                    b"zn <= SIZE_MAX - str_l\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/strings.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2233 as ::core::ffi::c_uint,
                                    b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        str_l = str_l.wrapping_add(zn);
                    }
                    let mut zn_0: size_t = number_of_zeros_to_pad;
                    if str_avail {
                        let mut avail_2: size_t = str_m.wrapping_sub(str_l);
                        memset(
                            str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                            '0' as ::core::ffi::c_int,
                            if zn_0 < avail_2 { zn_0 } else { avail_2 },
                        );
                        str_avail = zn_0 < avail_2;
                    }
                    '_c2rust_label_7: {
                        if zn_0 <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                        } else {
                            __assert_fail(
                                b"zn <= SIZE_MAX - str_l\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/strings.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2244 as ::core::ffi::c_uint,
                                b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    str_l = str_l.wrapping_add(zn_0);
                }
                if str_arg_l > zero_padding_insertion_ind {
                    let mut sn: size_t = str_arg_l.wrapping_sub(zero_padding_insertion_ind);
                    if str_avail {
                        let mut avail_3: size_t = str_m.wrapping_sub(str_l);
                        memmove(
                            str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                            str_arg.offset(zero_padding_insertion_ind as isize)
                                as *const ::core::ffi::c_void,
                            if sn < avail_3 { sn } else { avail_3 },
                        );
                        str_avail = sn < avail_3;
                    }
                    '_c2rust_label_8: {
                        if sn <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                        } else {
                            __assert_fail(
                                b"sn <= SIZE_MAX - str_l\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/strings.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2259 as ::core::ffi::c_uint,
                                b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    str_l = str_l.wrapping_add(sn);
                }
                if justify_left {
                    '_c2rust_label_9: {
                        if str_arg_l
                            <= (18446744073709551615 as size_t).wrapping_sub(number_of_zeros_to_pad)
                        {
                        } else {
                            __assert_fail(
                                b"str_arg_l <= SIZE_MAX - number_of_zeros_to_pad\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                                b"src/nvim/strings.rs\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                                2265 as ::core::ffi::c_uint,
                                b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                    .as_ptr() as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    if min_field_width > str_arg_l.wrapping_add(number_of_zeros_to_pad) {
                        let mut pn_0: size_t = min_field_width
                            .wrapping_sub(str_arg_l.wrapping_add(number_of_zeros_to_pad));
                        if str_avail {
                            let mut avail_4: size_t = str_m.wrapping_sub(str_l);
                            memset(
                                str.offset(str_l as isize) as *mut ::core::ffi::c_void,
                                ' ' as ::core::ffi::c_int,
                                if pn_0 < avail_4 { pn_0 } else { avail_4 },
                            );
                            str_avail = pn_0 < avail_4;
                        }
                        '_c2rust_label_10: {
                            if pn_0 <= (18446744073709551615 as size_t).wrapping_sub(str_l) {
                            } else {
                                __assert_fail(
                                    b"pn <= SIZE_MAX - str_l\0".as_ptr()
                                        as *const ::core::ffi::c_char,
                                    b"src/nvim/strings.rs\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                    2274 as ::core::ffi::c_uint,
                                    b"int vim_vsnprintf_typval(char *, size_t, const char *, struct __va_list_tag *, typval_T *const)\0"
                                        .as_ptr() as *const ::core::ffi::c_char,
                                );
                            }
                        };
                        str_l = str_l.wrapping_add(pn_0);
                    }
                }
                xfree(tofree as *mut ::core::ffi::c_void);
            }
        }
        if str_m > 0 as size_t {
            *str.offset(
                (if str_l <= str_m.wrapping_sub(1 as size_t) {
                    str_l
                } else {
                    str_m.wrapping_sub(1 as size_t)
                }) as isize,
            ) = NUL as ::core::ffi::c_char;
        }
        if !tvs.is_null()
            && (*tvs.offset(
                (if num_posarg != 0 as ::core::ffi::c_int {
                    num_posarg
                } else {
                    arg_idx - 1 as ::core::ffi::c_int
                }) as isize,
            ))
            .v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(
                b"E767: Too many arguments to printf()\0".as_ptr() as *const ::core::ffi::c_char
            ));
        }
    }
    xfree(ap_types as *mut ::core::ffi::c_void);
    return str_l as ::core::ffi::c_int;
}
