//! Coercions: reading a `typval_T` as a number, float, string or boolean.
//!
//! [`tv_get_number_chk`] is the arithmetic conversion, with the `_chk` half
//! reporting whether the value was convertible at all.
//! [`tv_get_string_buf_chk`] is the string one, which formats numbers into a
//! caller-supplied `NUMBUFLEN` buffer so the result never needs freeing.
//! [`tv2bool`] is the truthiness `if` and `while` ask for.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_get_number(tv: *const typval_T) -> varnumber_T {
    unsafe {
        let mut error: bool = false_0 != 0;
        return tv_get_number_chk(tv, &raw mut error);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_get_number_chk(
    tv: *const typval_T,
    ret_error: *mut bool,
) -> varnumber_T {
    unsafe {
        match (*tv).v_type as ::core::ffi::c_uint {
            3 | 9 | 4 | 5 | 10 | 6 => {
                emsg(gettext((*num_errors.ptr())[(*tv).v_type as usize]));
            }
            1 => return (*tv).vval.v_number,
            2 => {
                let mut n: varnumber_T = 0 as varnumber_T;
                if !(*tv).vval.v_string.is_null() {
                    vim_str2nr(
                        (*tv).vval.v_string,
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        ::core::ptr::null_mut::<::core::ffi::c_int>(),
                        STR2NR_ALL as ::core::ffi::c_int,
                        &raw mut n,
                        ::core::ptr::null_mut::<uvarnumber_T>(),
                        0 as ::core::ffi::c_int,
                        false_0 != 0,
                        ::core::ptr::null_mut::<bool>(),
                    );
                }
                return n;
            }
            7 => {
                return (if (*tv).vval.v_bool as ::core::ffi::c_uint
                    == kBoolVarTrue as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    1 as ::core::ffi::c_int
                } else {
                    0 as ::core::ffi::c_int
                }) as varnumber_T;
            }
            8 => return 0 as varnumber_T,
            0 => {
                semsg(
                    gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                    b"tv_get_number(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            _ => {}
        }
        if !ret_error.is_null() {
            *ret_error = true_0 != 0;
        }
        return (if ret_error.is_null() {
            -1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as varnumber_T;
    }
}

pub unsafe extern "C" fn tv_get_bool(tv: *const typval_T) -> varnumber_T {
    unsafe {
        return tv_get_number_chk(tv, ::core::ptr::null_mut::<bool>());
    }
}

pub unsafe extern "C" fn tv_get_bool_chk(tv: *const typval_T, ret_error: *mut bool) -> varnumber_T {
    unsafe {
        return tv_get_number_chk(tv, ret_error);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_get_lnum(tv: *const typval_T) -> linenr_T {
    unsafe {
        let did_emsg_before: ::core::ffi::c_int = did_emsg.get();
        let mut lnum: linenr_T = tv_get_number_chk(tv, ::core::ptr::null_mut::<bool>()) as linenr_T;
        if lnum <= 0 as linenr_T
            && did_emsg_before == did_emsg.get()
            && (*tv).v_type as ::core::ffi::c_uint
                != VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut fnum: ::core::ffi::c_int = 0;
            let fp: *mut pos_T =
                var2fpos(tv, true_0 != 0, &raw mut fnum, false_0 != 0, curwin.get());
            if !fp.is_null() {
                lnum = (*fp).lnum;
            }
        }
        return lnum;
    }
}

pub unsafe extern "C" fn tv_get_lnum_buf(tv: *const typval_T, buf: *const buf_T) -> linenr_T {
    unsafe {
        if (*tv).v_type as ::core::ffi::c_uint
            == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
            && !(*tv).vval.v_string.is_null()
            && *(*tv).vval.v_string.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '$' as ::core::ffi::c_int
            && *(*tv).vval.v_string.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == NUL
            && !buf.is_null()
        {
            return (*buf).b_ml.ml_line_count;
        }
        return tv_get_number_chk(tv, ::core::ptr::null_mut::<bool>()) as linenr_T;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_get_float(tv: *const typval_T) -> float_T {
    unsafe {
        match (*tv).v_type as ::core::ffi::c_uint {
            1 => return (*tv).vval.v_number as float_T,
            6 => return (*tv).vval.v_float,
            9 | 3 => {
                emsg(gettext(
                    b"E891: Using a Funcref as a Float\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            2 => {
                emsg(gettext(
                    b"E892: Using a String as a Float\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            4 => {
                emsg(gettext(
                    b"E893: Using a List as a Float\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            5 => {
                emsg(gettext(
                    b"E894: Using a Dictionary as a Float\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            7 => {
                emsg(gettext(
                    b"E362: Using a boolean value as a Float\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            }
            8 => {
                emsg(gettext(
                    b"E907: Using a special value as a Float\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            }
            10 => {
                emsg(gettext(
                    b"E975: Using a Blob as a Float\0".as_ptr() as *const ::core::ffi::c_char
                ));
            }
            0 => {
                semsg(
                    gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                    b"tv_get_float(UNKNOWN)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            _ => {}
        }
        return 0 as ::core::ffi::c_int as float_T;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_get_string_buf_chk(
    tv: *const typval_T,
    buf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        match (*tv).v_type as ::core::ffi::c_uint {
            1 => {
                snprintf(
                    buf,
                    NUMBUFLEN as ::core::ffi::c_int as size_t,
                    b"%ld\0".as_ptr() as *const ::core::ffi::c_char,
                    (*tv).vval.v_number,
                );
                return buf;
            }
            6 => {
                vim_snprintf(
                    buf,
                    NUMBUFLEN as ::core::ffi::c_int as size_t,
                    b"%g\0".as_ptr() as *const ::core::ffi::c_char,
                    (*tv).vval.v_float,
                );
                return buf;
            }
            2 => {
                if !(*tv).vval.v_string.is_null() {
                    return (*tv).vval.v_string;
                }
                return b"\0".as_ptr() as *const ::core::ffi::c_char;
            }
            7 => {
                strcpy(
                    buf,
                    *(&raw const encode_bool_var_names as *const *const ::core::ffi::c_char)
                        .offset((*tv).vval.v_bool as isize)
                        as *mut ::core::ffi::c_char,
                );
                return buf;
            }
            8 => {
                strcpy(
                    buf,
                    *(&raw const encode_special_var_names as *const *const ::core::ffi::c_char)
                        .offset((*tv).vval.v_special as isize)
                        as *mut ::core::ffi::c_char,
                );
                return buf;
            }
            9 | 3 | 4 | 5 | 10 | 0 => {
                emsg(gettext((*str_errors.ptr())[(*tv).v_type as usize]));
                return ::core::ptr::null::<::core::ffi::c_char>();
            }
            _ => {}
        }
        abort();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_get_string_chk(tv: *const typval_T) -> *const ::core::ffi::c_char {
    unsafe {
        static mybuf: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new([0; 65]);
        return tv_get_string_buf_chk(tv, mybuf.ptr() as *mut ::core::ffi::c_char);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_get_string(tv: *const typval_T) -> *const ::core::ffi::c_char {
    unsafe {
        static mybuf: GlobalCell<[::core::ffi::c_char; 65]> = GlobalCell::new([0; 65]);
        return tv_get_string_buf(tv as *mut typval_T, mybuf.ptr() as *mut ::core::ffi::c_char);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_get_string_buf(
    tv: *const typval_T,
    buf: *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let res: *const ::core::ffi::c_char = tv_get_string_buf_chk(tv, buf);
        return if !res.is_null() {
            res
        } else {
            b"\0".as_ptr() as *const ::core::ffi::c_char
        };
    }
}

pub unsafe extern "C" fn tv2bool(tv: *const typval_T) -> bool {
    unsafe {
        match (*tv).v_type as ::core::ffi::c_uint {
            1 => return (*tv).vval.v_number != 0 as varnumber_T,
            6 => return (*tv).vval.v_float != 0.0f64,
            9 => return !(*tv).vval.v_partial.is_null(),
            3 | 2 => {
                return !(*tv).vval.v_string.is_null()
                    && *(*tv).vval.v_string as ::core::ffi::c_int != NUL;
            }
            4 => {
                return !(*tv).vval.v_list.is_null()
                    && (*(*tv).vval.v_list).lv_len > 0 as ::core::ffi::c_int;
            }
            5 => {
                return !(*tv).vval.v_dict.is_null()
                    && (*(*tv).vval.v_dict).dv_hashtab.ht_used > 0 as size_t;
            }
            7 => {
                return (*tv).vval.v_bool as ::core::ffi::c_uint
                    == kBoolVarTrue as ::core::ffi::c_int as ::core::ffi::c_uint;
            }
            8 => {
                return (*tv).vval.v_special as ::core::ffi::c_uint
                    != kSpecialVarNull as ::core::ffi::c_int as ::core::ffi::c_uint;
            }
            10 => {
                return !(*tv).vval.v_blob.is_null()
                    && (*(*tv).vval.v_blob).bv_ga.ga_len > 0 as ::core::ffi::c_int;
            }
            0 | _ => {}
        }
        return false_0 != 0;
    }
}
