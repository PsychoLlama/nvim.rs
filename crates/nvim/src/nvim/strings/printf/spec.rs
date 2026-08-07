//! Format specifiers: typing them, and the `%N$` positional pass.
//!
//! `format_typeof` reduces a conversion plus its length modifier to one of the
//! `TYPE_*` classes, and `format_typename` names that class for an error
//! message.  `parse_fmt_types` is the pre-pass positional arguments force: with
//! `%N$` the arguments are not consumed in order, so the whole format has to be
//! walked first to learn each position's type, `adjust_types` recording one and
//! rejecting a position used at two incompatible types.  `skip_to_arg` is the
//! lookup that pass exists to serve.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use super::*;

pub const TYPE_FLOAT: ::core::ffi::c_int = 12;

pub const TYPE_SIZET: ::core::ffi::c_int = 7;

pub const TYPE_UNSIGNEDLONGLONGINT: ::core::ffi::c_int = 6;

pub const TYPE_UNSIGNEDLONGINT: ::core::ffi::c_int = 5;

pub const TYPE_UNSIGNEDINT: ::core::ffi::c_int = 4;

pub const TYPE_SIGNEDSIZET: ::core::ffi::c_int = 3;

pub const TYPE_LONGLONGINT: ::core::ffi::c_int = 2;

pub const TYPE_LONGINT: ::core::ffi::c_int = 1;

pub const TYPE_INT: ::core::ffi::c_int = 0;

pub const TYPE_POINTER: ::core::ffi::c_int = 8;

pub const TYPE_STRING: ::core::ffi::c_int = 11;

pub const TYPE_CHAR: ::core::ffi::c_int = 10;

pub const TYPE_UNKNOWN: ::core::ffi::c_int = -1;

pub const TYPE_PERCENT: ::core::ffi::c_int = 9;

pub const MAX_ALLOWED_STRING_WIDTH: ::core::ffi::c_int = 1048576;

static e_cannot_mix_positional_and_non_positional_str: GlobalCell<[::core::ffi::c_char; 62]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 62], [::core::ffi::c_char; 62]>(
            *b"E1500: Cannot mix positional and non-positional arguments: %s\0",
        )
    });

static e_fmt_arg_nr_unused_str: GlobalCell<[::core::ffi::c_char; 55]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 55], [::core::ffi::c_char; 55]>(
        *b"E1501: format argument %d unused in $-style format: %s\0",
    )
});

static e_positional_num_field_spec_reused_str_str: GlobalCell<[::core::ffi::c_char; 82]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 82], [::core::ffi::c_char; 82]>(
            *b"E1502: Positional argument %d used as field width reused as different type: %s/%s\0",
        )
    });

static e_positional_nr_out_of_bounds_str: GlobalCell<[::core::ffi::c_char; 48]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 48], [::core::ffi::c_char; 48]>(
            *b"E1503: Positional argument %d out of bounds: %s\0",
        )
    });

static e_positional_arg_num_type_inconsistent_str_str: GlobalCell<[::core::ffi::c_char; 62]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 62], [::core::ffi::c_char; 62]>(
            *b"E1504: Positional argument %d type used inconsistently: %s/%s\0",
        )
    });

static e_invalid_format_specifier_str: GlobalCell<[::core::ffi::c_char; 36]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
            *b"E1505: Invalid format specifier: %s\0",
        )
    });

static e_aptypes_is_null_nr_str: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
        *b"E1507: Internal error: ap_types or ap_types[idx] is NULL: %d: %s\0",
    )
});

static typename_unknown: GlobalCell<[::core::ffi::c_char; 8]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"unknown\0")
});

static typename_int: GlobalCell<[::core::ffi::c_char; 4]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 4], [::core::ffi::c_char; 4]>(*b"int\0")
});

static typename_longint: GlobalCell<[::core::ffi::c_char; 9]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 9], [::core::ffi::c_char; 9]>(*b"long int\0")
});

static typename_longlongint: GlobalCell<[::core::ffi::c_char; 14]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"long long int\0")
});

static typename_signedsizet: GlobalCell<[::core::ffi::c_char; 14]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 14], [::core::ffi::c_char; 14]>(*b"signed size_t\0")
});

static typename_unsignedint: GlobalCell<[::core::ffi::c_char; 13]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 13], [::core::ffi::c_char; 13]>(*b"unsigned int\0")
});

static typename_unsignedlongint: GlobalCell<[::core::ffi::c_char; 18]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 18], [::core::ffi::c_char; 18]>(*b"unsigned long int\0")
});

static typename_unsignedlonglongint: GlobalCell<[::core::ffi::c_char; 23]> =
    GlobalCell::new(unsafe {
        ::core::mem::transmute::<[u8; 23], [::core::ffi::c_char; 23]>(*b"unsigned long long int\0")
    });

static typename_sizet: GlobalCell<[::core::ffi::c_char; 7]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"size_t\0")
});

static typename_pointer: GlobalCell<[::core::ffi::c_char; 8]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"pointer\0")
});

static typename_percent: GlobalCell<[::core::ffi::c_char; 8]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 8], [::core::ffi::c_char; 8]>(*b"percent\0")
});

static typename_char: GlobalCell<[::core::ffi::c_char; 5]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 5], [::core::ffi::c_char; 5]>(*b"char\0")
});

static typename_string: GlobalCell<[::core::ffi::c_char; 7]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 7], [::core::ffi::c_char; 7]>(*b"string\0")
});

static typename_float: GlobalCell<[::core::ffi::c_char; 6]> = GlobalCell::new(unsafe {
    ::core::mem::transmute::<[u8; 6], [::core::ffi::c_char; 6]>(*b"float\0")
});
// ── The vim_str* family: safe cores + C-ABI shims ─────────────────────────
//
// Byte-level logic (unquoting, ASCII case mapping, comparison, scanning)
// lives in safe functions; the shims confine the raw-pointer plumbing.
// Multibyte-aware functions (vim_strsave_escaped_ext, shellescape,
// strcase_save, vim_strchr) still call the transpiled mbyte/ex_docmd
// machinery through raw pointers and remain shims throughout. Results are
// allocated with the xmalloc family so ownership crosses the C ABI as
// before.

unsafe extern "C" fn format_typeof(mut type_0: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    unsafe {
        let mut length_modifier: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
        let mut fmt_spec: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
        if *type_0 as ::core::ffi::c_int == 'h' as ::core::ffi::c_int
            || *type_0 as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
            || *type_0 as ::core::ffi::c_int == 'z' as ::core::ffi::c_int
        {
            length_modifier = *type_0;
            type_0 = type_0.offset(1);
            if length_modifier as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
                && *type_0 as ::core::ffi::c_int == 'l' as ::core::ffi::c_int
            {
                length_modifier = 'L' as ::core::ffi::c_char;
                type_0 = type_0.offset(1);
            }
        }
        fmt_spec = *type_0;
        match fmt_spec as ::core::ffi::c_int {
            105 => {
                fmt_spec = 'd' as ::core::ffi::c_char;
            }
            42 => {
                fmt_spec = 'd' as ::core::ffi::c_char;
                length_modifier = 'h' as ::core::ffi::c_char;
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
            37 => return TYPE_PERCENT,
            99 => return TYPE_CHAR,
            115 | 83 => return TYPE_STRING,
            100 | 117 | 98 | 66 | 111 | 120 | 88 | 112 => {
                if fmt_spec as ::core::ffi::c_int == 'p' as ::core::ffi::c_int {
                    return TYPE_POINTER;
                } else if fmt_spec as ::core::ffi::c_int == 'b' as ::core::ffi::c_int
                    || fmt_spec as ::core::ffi::c_int == 'B' as ::core::ffi::c_int
                {
                    return TYPE_UNSIGNEDLONGLONGINT;
                } else if fmt_spec as ::core::ffi::c_int == 'd' as ::core::ffi::c_int {
                    match length_modifier as ::core::ffi::c_int {
                        NUL | 104 => return TYPE_INT,
                        108 => return TYPE_LONGINT,
                        76 => return TYPE_LONGLONGINT,
                        122 => return TYPE_SIGNEDSIZET,
                        _ => {}
                    }
                } else {
                    match length_modifier as ::core::ffi::c_int {
                        NUL | 104 => return TYPE_UNSIGNEDINT,
                        108 => return TYPE_UNSIGNEDLONGINT,
                        76 => return TYPE_UNSIGNEDLONGLONGINT,
                        122 => return TYPE_SIZET,
                        _ => {}
                    }
                }
            }
            102 | 70 | 101 | 69 | 103 | 71 => return TYPE_FLOAT,
            _ => {}
        }
        return TYPE_UNKNOWN;
    }
}

unsafe extern "C" fn format_typename(
    mut type_0: *const ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        match format_typeof(type_0) {
            0 => return gettext((typename_int.ptr() as *const _) as *const ::core::ffi::c_char),
            1 => {
                return gettext((typename_longint.ptr() as *const _) as *const ::core::ffi::c_char);
            }
            2 => {
                return gettext(
                    (typename_longlongint.ptr() as *const _) as *const ::core::ffi::c_char,
                );
            }
            4 => {
                return gettext(
                    (typename_unsignedint.ptr() as *const _) as *const ::core::ffi::c_char,
                );
            }
            3 => {
                return gettext(
                    (typename_signedsizet.ptr() as *const _) as *const ::core::ffi::c_char,
                );
            }
            5 => {
                return gettext(
                    (typename_unsignedlongint.ptr() as *const _) as *const ::core::ffi::c_char,
                );
            }
            6 => {
                return gettext(
                    (typename_unsignedlonglongint.ptr() as *const _) as *const ::core::ffi::c_char,
                );
            }
            7 => return gettext((typename_sizet.ptr() as *const _) as *const ::core::ffi::c_char),
            8 => {
                return gettext((typename_pointer.ptr() as *const _) as *const ::core::ffi::c_char);
            }
            9 => {
                return gettext((typename_percent.ptr() as *const _) as *const ::core::ffi::c_char);
            }
            10 => return gettext((typename_char.ptr() as *const _) as *const ::core::ffi::c_char),
            11 => {
                return gettext((typename_string.ptr() as *const _) as *const ::core::ffi::c_char);
            }
            12 => return gettext((typename_float.ptr() as *const _) as *const ::core::ffi::c_char),
            _ => {}
        }
        return gettext((typename_unknown.ptr() as *const _) as *const ::core::ffi::c_char);
    }
}

unsafe extern "C" fn adjust_types(
    mut ap_types: *mut *mut *const ::core::ffi::c_char,
    mut arg: ::core::ffi::c_int,
    mut num_posarg: *mut ::core::ffi::c_int,
    mut type_0: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        if arg <= 0 as ::core::ffi::c_int {
            semsg(
                gettext(
                    (e_invalid_format_specifier_str.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ),
                type_0,
            );
            return FAIL;
        }
        if (*ap_types).is_null() || *num_posarg < arg {
            let mut new_types: *mut *const ::core::ffi::c_char = (if (*ap_types).is_null() {
                xcalloc(
                    arg as size_t,
                    ::core::mem::size_of::<*const ::core::ffi::c_char>(),
                )
            } else {
                xrealloc(
                    *ap_types as *mut ::core::ffi::c_void,
                    (arg as size_t)
                        .wrapping_mul(::core::mem::size_of::<*const ::core::ffi::c_char>()),
                )
            })
                as *mut *const ::core::ffi::c_char;
            let mut idx: ::core::ffi::c_int = *num_posarg;
            while idx < arg {
                *new_types.offset(idx as isize) = ::core::ptr::null::<::core::ffi::c_char>();
                idx += 1;
            }
            *ap_types = new_types;
            *num_posarg = arg;
        }
        if !(*(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize)).is_null() {
            if *(*(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize))
                .offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '*' as ::core::ffi::c_int
                || *type_0.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
            {
                let mut pt: *const ::core::ffi::c_char = type_0;
                if *pt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '*' as ::core::ffi::c_int
                {
                    pt = *(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize);
                }
                if *pt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '*' as ::core::ffi::c_int
                {
                    match *pt.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int {
                        100 | 105 => {}
                        _ => {
                            semsg(
                                gettext(
                                    (e_positional_num_field_spec_reused_str_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                arg,
                                format_typename(
                                    *(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize),
                                ),
                                format_typename(type_0),
                            );
                            return FAIL;
                        }
                    }
                }
            } else if format_typeof(type_0)
                != format_typeof(*(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize))
            {
                semsg(
                    gettext(
                        (e_positional_arg_num_type_inconsistent_str_str.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ),
                    arg,
                    format_typename(type_0),
                    format_typename(*(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize)),
                );
                return FAIL;
            }
        }
        *(*ap_types).offset((arg - 1 as ::core::ffi::c_int) as isize) = type_0;
        return OK;
    }
}

pub(crate) unsafe extern "C" fn format_overflow_error(mut pstart: *const ::core::ffi::c_char) {
    unsafe {
        let mut p: *const ::core::ffi::c_char = pstart;
        while ascii_isdigit(*p as ::core::ffi::c_int) {
            p = p.offset(1);
        }
        semsg(
            gettext(&raw const e_val_too_large_len as *const ::core::ffi::c_char),
            p.offset_from(pstart) as ::core::ffi::c_int,
            pstart,
        );
    }
}

pub(crate) unsafe extern "C" fn get_unsigned_int(
    mut pstart: *const ::core::ffi::c_char,
    mut p: *mut *const ::core::ffi::c_char,
    mut uj: *mut ::core::ffi::c_uint,
    mut overflow_err: bool,
) -> ::core::ffi::c_int {
    unsafe {
        *uj = (**p as ::core::ffi::c_int - '0' as ::core::ffi::c_int) as ::core::ffi::c_uint;
        *p = (*p).offset(1);
        while ascii_isdigit(**p as ::core::ffi::c_int) as ::core::ffi::c_int != 0
            && *uj < MAX_ALLOWED_STRING_WIDTH as ::core::ffi::c_uint
        {
            *uj = (10 as ::core::ffi::c_uint).wrapping_mul(*uj).wrapping_add(
                (**p as ::core::ffi::c_int - '0' as ::core::ffi::c_int) as ::core::ffi::c_uint,
            );
            *p = (*p).offset(1);
        }
        if *uj > MAX_ALLOWED_STRING_WIDTH as ::core::ffi::c_uint {
            if overflow_err {
                format_overflow_error(pstart);
                return FAIL;
            } else {
                *uj = MAX_ALLOWED_STRING_WIDTH as ::core::ffi::c_uint;
            }
        }
        return OK;
    }
}

pub(crate) unsafe extern "C" fn parse_fmt_types(
    mut ap_types: *mut *mut *const ::core::ffi::c_char,
    mut num_posarg: *mut ::core::ffi::c_int,
    mut fmt: *const ::core::ffi::c_char,
    mut tvs: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let mut p: *const ::core::ffi::c_char = fmt;
        let mut arg: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut any_pos: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut any_arg: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if p.is_null() {
            return OK;
        }
        '_error: {
            while *p as ::core::ffi::c_int != NUL {
                if *p as ::core::ffi::c_int != '%' as ::core::ffi::c_int {
                    let mut n: size_t = xstrchrnul(
                        p.offset(1 as ::core::ffi::c_int as isize),
                        '%' as ::core::ffi::c_char,
                    )
                    .offset_from(p) as size_t;
                    p = p.offset(n as isize);
                } else {
                    let mut length_modifier: ::core::ffi::c_char = NUL as ::core::ffi::c_char;
                    let mut pos_arg: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
                    let mut pstart: *const ::core::ffi::c_char =
                        p.offset(1 as ::core::ffi::c_int as isize);
                    p = p.offset(1);
                    let mut ptype: *const ::core::ffi::c_char = p;
                    while ascii_isdigit(*ptype as ::core::ffi::c_int) {
                        ptype = ptype.offset(1);
                    }
                    if *ptype as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                        if *p as ::core::ffi::c_int == '0' as ::core::ffi::c_int {
                            semsg(
                                gettext(
                                    (e_invalid_format_specifier_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                fmt,
                            );
                            break '_error;
                        } else {
                            let mut uj: ::core::ffi::c_uint = 0;
                            if get_unsigned_int(pstart, &raw mut p, &raw mut uj, !tvs.is_null())
                                == FAIL
                            {
                                break '_error;
                            }
                            pos_arg = uj as ::core::ffi::c_int;
                            any_pos = 1 as ::core::ffi::c_int;
                            if any_pos != 0 && any_arg != 0 {
                                semsg(
                                    gettext(
                                        (e_cannot_mix_positional_and_non_positional_str.ptr()
                                            as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    fmt,
                                );
                                break '_error;
                            } else {
                                p = p.offset(1);
                            }
                        }
                    }
                    while *p as ::core::ffi::c_int == '0' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == ' ' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '#' as ::core::ffi::c_int
                        || *p as ::core::ffi::c_int == '\'' as ::core::ffi::c_int
                    {
                        match *p as ::core::ffi::c_int {
                            48 | 45 | 43 | 32 | 35 | 39 | _ => {}
                        }
                        p = p.offset(1);
                    }
                    arg = p;
                    if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                        p = p.offset(1);
                        if ascii_isdigit(*p as ::core::ffi::c_int) {
                            let mut uj_0: ::core::ffi::c_uint = 0;
                            if get_unsigned_int(
                                arg.offset(1 as ::core::ffi::c_int as isize),
                                &raw mut p,
                                &raw mut uj_0,
                                !tvs.is_null(),
                            ) == FAIL
                            {
                                break '_error;
                            }
                            if *p as ::core::ffi::c_int != '$' as ::core::ffi::c_int {
                                semsg(
                                    gettext(
                                        (e_invalid_format_specifier_str.ptr() as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    fmt,
                                );
                                break '_error;
                            } else {
                                p = p.offset(1);
                                any_pos = 1 as ::core::ffi::c_int;
                                if any_pos != 0 && any_arg != 0 {
                                    semsg(
                                        gettext(
                                            (e_cannot_mix_positional_and_non_positional_str.ptr()
                                                as *const _)
                                                as *const ::core::ffi::c_char,
                                        ),
                                        fmt,
                                    );
                                    break '_error;
                                } else if adjust_types(
                                    ap_types,
                                    uj_0 as ::core::ffi::c_int,
                                    num_posarg,
                                    arg,
                                ) == FAIL
                                {
                                    break '_error;
                                }
                            }
                        } else {
                            any_arg = 1 as ::core::ffi::c_int;
                            if any_pos != 0 && any_arg != 0 {
                                semsg(
                                    gettext(
                                        (e_cannot_mix_positional_and_non_positional_str.ptr()
                                            as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    fmt,
                                );
                                break '_error;
                            }
                        }
                    } else if ascii_isdigit(*p as ::core::ffi::c_int) {
                        let mut digstart: *const ::core::ffi::c_char = p;
                        let mut uj_1: ::core::ffi::c_uint = 0;
                        if get_unsigned_int(digstart, &raw mut p, &raw mut uj_1, !tvs.is_null())
                            == FAIL
                        {
                            break '_error;
                        }
                        if *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                            semsg(
                                gettext(
                                    (e_invalid_format_specifier_str.ptr() as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                fmt,
                            );
                            break '_error;
                        }
                    }
                    if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                        p = p.offset(1);
                        arg = p;
                        if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                            p = p.offset(1);
                            if ascii_isdigit(*p as ::core::ffi::c_int) {
                                let mut uj_2: ::core::ffi::c_uint = 0;
                                if get_unsigned_int(
                                    arg.offset(1 as ::core::ffi::c_int as isize),
                                    &raw mut p,
                                    &raw mut uj_2,
                                    !tvs.is_null(),
                                ) == FAIL
                                {
                                    break '_error;
                                }
                                if *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                                    any_pos = 1 as ::core::ffi::c_int;
                                    if any_pos != 0 && any_arg != 0 {
                                        semsg(
                                            gettext(
                                                (e_cannot_mix_positional_and_non_positional_str
                                                    .ptr()
                                                    as *const _)
                                                    as *const ::core::ffi::c_char,
                                            ),
                                            fmt,
                                        );
                                        break '_error;
                                    } else {
                                        p = p.offset(1);
                                        if adjust_types(
                                            ap_types,
                                            uj_2 as ::core::ffi::c_int,
                                            num_posarg,
                                            arg,
                                        ) == FAIL
                                        {
                                            break '_error;
                                        }
                                    }
                                } else {
                                    semsg(
                                        gettext(
                                            (e_invalid_format_specifier_str.ptr() as *const _)
                                                as *const ::core::ffi::c_char,
                                        ),
                                        fmt,
                                    );
                                    break '_error;
                                }
                            } else {
                                any_arg = 1 as ::core::ffi::c_int;
                                if any_pos != 0 && any_arg != 0 {
                                    semsg(
                                        gettext(
                                            (e_cannot_mix_positional_and_non_positional_str.ptr()
                                                as *const _)
                                                as *const ::core::ffi::c_char,
                                        ),
                                        fmt,
                                    );
                                    break '_error;
                                }
                            }
                        } else if ascii_isdigit(*p as ::core::ffi::c_int) {
                            let mut digstart_0: *const ::core::ffi::c_char = p;
                            let mut uj_3: ::core::ffi::c_uint = 0;
                            if get_unsigned_int(
                                digstart_0,
                                &raw mut p,
                                &raw mut uj_3,
                                !tvs.is_null(),
                            ) == FAIL
                            {
                                break '_error;
                            }
                            if *p as ::core::ffi::c_int == '$' as ::core::ffi::c_int {
                                semsg(
                                    gettext(
                                        (e_invalid_format_specifier_str.ptr() as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    fmt,
                                );
                                break '_error;
                            }
                        }
                    }
                    if pos_arg != -1 as ::core::ffi::c_int {
                        any_pos = 1 as ::core::ffi::c_int;
                        if any_pos != 0 && any_arg != 0 {
                            semsg(
                                gettext(
                                    (e_cannot_mix_positional_and_non_positional_str.ptr()
                                        as *const _)
                                        as *const ::core::ffi::c_char,
                                ),
                                fmt,
                            );
                            break '_error;
                        } else {
                            ptype = p;
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
                            p = p.offset(1);
                        }
                    }
                    match *p as ::core::ffi::c_int {
                        105 | 42 | 100 | 117 | 111 | 68 | 85 | 79 | 120 | 88 | 98 | 66 | 99
                        | 115 | 83 | 112 | 102 | 70 | 101 | 69 | 103 | 71 => {
                            if pos_arg != -1 as ::core::ffi::c_int {
                                if adjust_types(ap_types, pos_arg, num_posarg, ptype) == FAIL {
                                    break '_error;
                                }
                            } else {
                                any_arg = 1 as ::core::ffi::c_int;
                                if any_pos != 0 && any_arg != 0 {
                                    semsg(
                                        gettext(
                                            (e_cannot_mix_positional_and_non_positional_str.ptr()
                                                as *const _)
                                                as *const ::core::ffi::c_char,
                                        ),
                                        fmt,
                                    );
                                    break '_error;
                                }
                            }
                        }
                        _ => {
                            if pos_arg != -1 as ::core::ffi::c_int {
                                semsg(
                                    gettext(
                                        (e_cannot_mix_positional_and_non_positional_str.ptr()
                                            as *const _)
                                            as *const ::core::ffi::c_char,
                                    ),
                                    fmt,
                                );
                                break '_error;
                            }
                        }
                    }
                    if *p as ::core::ffi::c_int != NUL {
                        p = p.offset(1);
                    }
                }
            }
            let mut arg_idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while arg_idx < *num_posarg {
                if (*(*ap_types).offset(arg_idx as isize)).is_null() {
                    semsg(
                        gettext(
                            (e_fmt_arg_nr_unused_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        arg_idx + 1 as ::core::ffi::c_int,
                        fmt,
                    );
                    break '_error;
                } else if !tvs.is_null()
                    && (*tvs.offset(arg_idx as isize)).v_type as ::core::ffi::c_uint
                        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    semsg(
                        gettext(
                            (e_positional_nr_out_of_bounds_str.ptr() as *const _)
                                as *const ::core::ffi::c_char,
                        ),
                        arg_idx + 1 as ::core::ffi::c_int,
                        fmt,
                    );
                    break '_error;
                } else {
                    arg_idx += 1;
                }
            }
            return OK;
        }
        xfree(*ap_types as *mut ::core::ffi::c_void);
        *ap_types = ::core::ptr::null_mut::<*const ::core::ffi::c_char>();
        *num_posarg = 0 as ::core::ffi::c_int;
        return FAIL;
    }
}
// Hand-ported from neovim's static `skip_to_arg` in src/nvim/strings.c.
// c2rust drops this definition (it takes `va_list` by value, which its
// variadic support cannot translate) yet still emits the 17 call sites in
// `vim_vsnprintf_typval` below. This faithful port keeps the positional
// (`%N$`) printf path correct. The signature matches exactly what those call
// sites pass: `ap_start` is a fresh `va_copy` (`ap_start.clone()`) of the
// argument list's start, and `ap` is a pointer to the working `VaList`.

pub(crate) unsafe extern "C" fn skip_to_arg<'f>(
    ap_types: *mut *const ::core::ffi::c_char,
    ap_start: ::core::ffi::VaList<'f>,
    ap: *mut ::core::ffi::VaList<'f>,
    arg_idx: *mut ::core::ffi::c_int,
    arg_cur: *mut ::core::ffi::c_int,
    fmt: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut arg_min: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if *arg_cur + 1 as ::core::ffi::c_int == *arg_idx {
            *arg_cur += 1;
            *arg_idx += 1;
            return;
        }
        if *arg_cur >= *arg_idx {
            // Reset ap to ap_start and skip arg_idx - 1 types (va_end + va_copy).
            *ap = ap_start.clone();
        } else {
            // Skip over any we should skip.
            arg_min = *arg_cur;
        }
        *arg_cur = arg_min;
        while *arg_cur < *arg_idx - 1 as ::core::ffi::c_int {
            if ap_types.is_null() || (*ap_types.offset(*arg_cur as isize)).is_null() {
                siemsg(
                    (*e_aptypes_is_null_nr_str.ptr()).as_ptr() as *const ::core::ffi::c_char,
                    fmt,
                    *arg_cur,
                );
                return;
            }
            let p: *const ::core::ffi::c_char = *ap_types.offset(*arg_cur as isize);
            let fmt_type: ::core::ffi::c_int = format_typeof(p);
            // get parameter value, do initial processing (consume one va_arg)
            match fmt_type {
                TYPE_PERCENT | TYPE_UNKNOWN => {}
                TYPE_CHAR => {
                    (*ap).next_arg::<::core::ffi::c_int>();
                }
                TYPE_STRING => {
                    (*ap).next_arg::<*const ::core::ffi::c_char>();
                }
                TYPE_POINTER => {
                    (*ap).next_arg::<*mut ::core::ffi::c_void>();
                }
                TYPE_INT => {
                    (*ap).next_arg::<::core::ffi::c_int>();
                }
                TYPE_LONGINT => {
                    (*ap).next_arg::<::core::ffi::c_long>();
                }
                TYPE_LONGLONGINT => {
                    (*ap).next_arg::<::core::ffi::c_longlong>();
                }
                TYPE_SIGNEDSIZET => {
                    // implementation-defined, usually ptrdiff_t
                    (*ap).next_arg::<isize>();
                }
                TYPE_UNSIGNEDINT => {
                    (*ap).next_arg::<::core::ffi::c_uint>();
                }
                TYPE_UNSIGNEDLONGINT => {
                    (*ap).next_arg::<::core::ffi::c_ulong>();
                }
                TYPE_UNSIGNEDLONGLONGINT => {
                    (*ap).next_arg::<::core::ffi::c_ulonglong>();
                }
                TYPE_SIZET => {
                    (*ap).next_arg::<size_t>();
                }
                TYPE_FLOAT => {
                    (*ap).next_arg::<::core::ffi::c_double>();
                }
                _ => {}
            }
            *arg_cur += 1;
        }
        // Because we know that after we return from this call, a va_arg() call is
        // made, we can pre-emptively increment the current argument index.
        *arg_cur += 1;
        *arg_idx += 1;
    }
}
