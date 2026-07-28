//! The `OptVal` tagged union: freeing, copying, comparing, and converting
//! to and from the option variable, an API `Object` and a C string.

#[allow(unused_imports)]
use super::*;

#[inline]
pub(crate) unsafe extern "C" fn is_power_of_two(mut x: uint64_t) -> bool {
    return x != 0 as uint64_t && x & x.wrapping_sub(1 as uint64_t) == 0 as uint64_t;
}

#[inline]
pub(crate) unsafe extern "C" fn optval_type_get_name(type_0: OptValType) -> *const c_char {
    match type_0 as c_int {
        -1 => return b"nil\0".as_ptr() as *const c_char,
        0 => return b"boolean\0".as_ptr() as *const c_char,
        1 => return b"number\0".as_ptr() as *const c_char,
        2 => return b"string\0".as_ptr() as *const c_char,
        _ => {}
    }
    unreachable!();
}

pub unsafe extern "C" fn optval_free(mut o: OptVal) {
    match o.type_0 as c_int {
        2 => {
            if o.data.string.data != empty_string_option.ptr() as *mut c_char {
                api_free_string(o.data.string);
            }
        }
        -1 | 0 | 1 | _ => {}
    };
}

pub unsafe extern "C" fn optval_copy(mut o: OptVal) -> OptVal {
    match o.type_0 as c_int {
        -1 | 0 | 1 => return o,
        2 => {
            return OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: copy_string(o.data.string, ::core::ptr::null_mut::<Arena>()),
                },
            };
        }
        _ => {}
    }
    unreachable!();
}

pub unsafe extern "C" fn optval_equal(mut o1: OptVal, mut o2: OptVal) -> bool {
    if o1.type_0 as c_int != o2.type_0 as c_int {
        return false_0 != 0;
    }
    match o1.type_0 as c_int {
        -1 => return true_0 != 0,
        0 => {
            return o1.data.boolean as c_int == o2.data.boolean as c_int;
        }
        1 => return o1.data.number == o2.data.number,
        2 => {
            return o1.data.string.size == o2.data.string.size
                && (o1.data.string.data == o2.data.string.data
                    || strnequal(
                        o1.data.string.data,
                        o2.data.string.data,
                        o1.data.string.size,
                    ) as c_int
                        != 0);
        }
        _ => {}
    }
    unreachable!();
}

pub(crate) unsafe extern "C" fn option_get_type(opt_idx: OptIndex) -> OptValType {
    return (*options.ptr())[opt_idx as usize].type_0;
}

pub unsafe extern "C" fn optval_from_varp(mut opt_idx: OptIndex, mut varp: *mut c_void) -> OptVal {
    if varp as *mut c_int == &raw mut (*curbuf.get()).b_changed {
        return OptVal {
            type_0: kOptValTypeBoolean,
            data: OptValData {
                boolean: curbufIsChanged() as TriState,
            },
        };
    }
    let mut type_0: OptValType = option_get_type(opt_idx);
    match type_0 as c_int {
        -1 => {
            return OptVal {
                type_0: kOptValTypeNil,
                data: OptValData { boolean: kFalse },
            };
        }
        0 => {
            return OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData {
                    boolean: (if *(varp as *mut c_int) == 0 as c_int {
                        kFalse as c_int
                    } else if *(varp as *mut c_int) >= 1 as c_int {
                        kTrue as c_int
                    } else {
                        kNone as c_int
                    }) as TriState,
                },
            };
        }
        1 => {
            return OptVal {
                type_0: kOptValTypeNumber,
                data: OptValData {
                    number: *(varp as *mut OptInt),
                },
            };
        }
        2 => {
            return OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string(*(varp as *mut *mut c_char)),
                },
            };
        }
        _ => {}
    }
    unreachable!();
}

pub(crate) unsafe extern "C" fn set_option_varp(
    mut opt_idx: OptIndex,
    mut varp: *mut c_void,
    mut value: OptVal,
    mut free_oldval: bool,
) {
    '_c2rust_label: {
        if option_has_type(opt_idx, value.type_0) {
        } else {
            __assert_fail(
                b"option_has_type(opt_idx, value.type)\0".as_ptr() as *const c_char,
                b"src/nvim/option.rs\0".as_ptr() as *const c_char,
                3401 as c_uint,
                b"void set_option_varp(OptIndex, void *, OptVal, _Bool)\0".as_ptr()
                    as *const c_char,
            );
        }
    };
    if free_oldval {
        optval_free(optval_from_varp(opt_idx, varp));
    }
    match value.type_0 as c_int {
        -1 => {
            abort();
        }
        0 => {
            *(varp as *mut c_int) = value.data.boolean as c_int;
            return;
        }
        1 => {
            *(varp as *mut OptInt) = value.data.number;
            return;
        }
        2 => {
            *(varp as *mut *mut c_char) = value.data.string.data;
            return;
        }
        _ => {}
    }
    unreachable!();
}

pub(crate) unsafe extern "C" fn optval_to_cstr(mut o: OptVal) -> *mut c_char {
    match o.type_0 as c_int {
        -1 => return xstrdup(b"\0".as_ptr() as *const c_char),
        0 => {
            return xstrdup(if o.data.boolean as c_int != 0 {
                b"true\0".as_ptr() as *const c_char
            } else {
                b"false\0".as_ptr() as *const c_char
            });
        }
        1 => {
            let mut buf: *mut c_char = xmalloc(NUMBUFLEN as c_int as size_t) as *mut c_char;
            snprintf(
                buf,
                NUMBUFLEN as c_int as size_t,
                b"%ld\0".as_ptr() as *const c_char,
                o.data.number,
            );
            return buf;
        }
        2 => {
            let mut buf_0: *mut c_char =
                xmalloc(o.data.string.size.wrapping_add(3 as size_t)) as *mut c_char;
            snprintf(
                buf_0,
                o.data.string.size.wrapping_add(3 as size_t),
                b"\"%s\"\0".as_ptr() as *const c_char,
                o.data.string.data,
            );
            return buf_0;
        }
        _ => {}
    }
    unreachable!();
}

pub unsafe extern "C" fn optval_as_object(mut o: OptVal) -> Object {
    match o.type_0 as c_int {
        -1 => {
            return object {
                type_0: kObjectTypeNil,
                data: object_data { boolean: false },
            };
        }
        0 => {
            match o.data.boolean as c_int {
                0 | 1 => {
                    return object {
                        type_0: kObjectTypeBoolean,
                        data: object_data {
                            boolean: o.data.boolean as u64 != 0,
                        },
                    };
                }
                -1 => {
                    return object {
                        type_0: kObjectTypeNil,
                        data: object_data { boolean: false },
                    };
                }
                _ => {}
            }
            unreachable!();
        }
        1 => {
            return object {
                type_0: kObjectTypeInteger,
                data: object_data {
                    integer: o.data.number,
                },
            };
        }
        2 => {
            return object {
                type_0: kObjectTypeString,
                data: object_data {
                    string: o.data.string,
                },
            };
        }
        _ => {}
    }
    unreachable!();
}

pub unsafe extern "C" fn object_as_optval(mut o: Object, mut error: *mut bool) -> OptVal {
    match o.type_0 as c_uint {
        0 => {
            return OptVal {
                type_0: kOptValTypeNil,
                data: OptValData { boolean: kFalse },
            };
        }
        1 => {
            return OptVal {
                type_0: kOptValTypeBoolean,
                data: OptValData {
                    boolean: o.data.boolean as TriState,
                },
            };
        }
        2 => {
            return OptVal {
                type_0: kOptValTypeNumber,
                data: OptValData {
                    number: o.data.integer,
                },
            };
        }
        4 => {
            return OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: o.data.string,
                },
            };
        }
        _ => {
            *error = true_0 != 0;
            return OptVal {
                type_0: kOptValTypeNil,
                data: OptValData { boolean: kFalse },
            };
        }
    };
}

pub(crate) unsafe extern "C" fn optval_default(
    mut opt_idx: OptIndex,
    mut varp: *mut c_void,
) -> c_int {
    let mut opt: *mut vimoption_T = (options.ptr() as *mut vimoption_T).offset(opt_idx as isize);
    if is_option_hidden(opt_idx) {
        return true_0;
    }
    let mut current_val: OptVal = optval_from_varp(opt_idx, varp);
    let mut default_val: OptVal = (*opt).def_val;
    return optval_equal(current_val, default_val) as c_int;
}
