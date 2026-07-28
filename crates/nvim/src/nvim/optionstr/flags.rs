//! Options whose value is a set of flag letters or one word from a fixed
//! list.

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn did_set_opt_flags(
    mut val: *mut c_char,
    mut values: *mut *const c_char,
    mut flagp: *mut c_uint,
    mut list: bool,
) -> *const c_char {
    if opt_strings_flags(val, values, flagp, list) != OK {
        return &raw const e_invarg as *const c_char;
    }
    return ::core::ptr::null::<c_char>();
}

pub(crate) unsafe extern "C" fn opt_values(
    mut idx: OptIndex,
    mut values_len: *mut size_t,
) -> *mut *const c_char {
    let mut idx1: OptIndex = (if idx as c_int == kOptViewoptions as c_int {
        kOptSessionoptions as c_int
    } else if idx as c_int == kOptFileformats as c_int {
        kOptFileformat as c_int
    } else {
        idx as c_int
    }) as OptIndex;
    let mut opt: *mut vimoption_T = get_option(idx1);
    if !values_len.is_null() {
        *values_len = (*opt).values_len;
    }
    return (*opt).values;
}

pub unsafe extern "C" fn did_set_str_generic(mut args: *mut optset_T) -> *const c_char {
    return if check_str_opt((*args).os_idx, (*args).os_varp as *mut *mut c_char) != OK {
        &raw const e_invarg as *const c_char
    } else {
        ::core::ptr::null::<c_char>()
    };
}

pub(crate) unsafe extern "C" fn did_set_option_listflag(
    mut val: *mut c_char,
    mut flags: *mut c_char,
    mut errbuf: *mut c_char,
    mut errbuflen: size_t,
) -> *const c_char {
    let mut s: *mut c_char = val;
    while *s != 0 {
        if vim_strchr(flags, *s as uint8_t as c_int).is_null() {
            return illegal_char(errbuf, errbuflen, *s as uint8_t as c_int);
        }
        s = s.offset(1);
    }
    return ::core::ptr::null::<c_char>();
}

pub(crate) unsafe extern "C" fn opt_strings_flags(
    mut val: *const c_char,
    mut values: *mut *const c_char,
    mut flagp: *mut c_uint,
    mut list: bool,
) -> c_int {
    let mut new_flags: c_uint = 0 as c_uint;
    let mut iter_one: bool = *val as c_int == NUL && !list;
    while *val as c_int != 0 || iter_one as c_int != 0 {
        let mut i: c_uint = 0 as c_uint;
        loop {
            if (*values.offset(i as isize)).is_null() {
                return FAIL;
            }
            let mut len: size_t = strlen(*values.offset(i as isize));
            if strncmp(*values.offset(i as isize), val, len) == 0 as c_int
                && (list as c_int != 0 && *val.offset(len as isize) as c_int == ',' as c_int
                    || *val.offset(len as isize) as c_int == NUL)
            {
                val = val.offset(len.wrapping_add(
                    (*val.offset(len as isize) as c_int == ',' as c_int) as c_int as size_t,
                ) as isize);
                '_c2rust_label: {
                    if (i as usize) < ::core::mem::size_of::<c_uint>().wrapping_mul(8 as usize) {
                    } else {
                        __assert_fail(
                            b"i < sizeof(new_flags) * 8\0".as_ptr() as *const c_char,
                            b"src/nvim/optionstr.rs\0".as_ptr() as *const c_char,
                            2192 as c_uint,
                            __ASSERT_FUNCTION.as_ptr(),
                        );
                    }
                };
                new_flags |= (1 as c_uint) << i;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        if iter_one {
            break;
        }
    }
    if !flagp.is_null() {
        *flagp = new_flags;
    }
    return OK;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn check_ff_value(mut p: *mut c_char) -> c_int {
    return opt_strings_flags(
        p,
        opt_ff_values.ptr() as *mut *const c_char,
        ::core::ptr::null_mut::<c_uint>(),
        false_0 != 0,
    );
}
