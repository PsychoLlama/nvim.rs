//! Parsing 'cinoptions'.
//!
//! One pass over the option string; each item is a letter, optionally prefixed
//! by `>`/`-`/`+`, then a signed number optionally prefixed by `s` meaning
//! "multiples of 'shiftwidth'".  Every field it writes is a `b_ind_*` on the
//! buffer, and those are the only inputs `get_c_indent` has besides the text --
//! so this function is the whole option surface of C indenting.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn parse_cino(mut buf: *mut buf_T) {
    unsafe {
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut l: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut divider: ::core::ffi::c_int = 0;
        let mut fraction: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut sw: ::core::ffi::c_int = get_sw_value(buf);
        (*buf).b_ind_level = sw;
        (*buf).b_ind_open_imag = 0 as ::core::ffi::c_int;
        (*buf).b_ind_no_brace = 0 as ::core::ffi::c_int;
        (*buf).b_ind_first_open = 0 as ::core::ffi::c_int;
        (*buf).b_ind_open_extra = 0 as ::core::ffi::c_int;
        (*buf).b_ind_close_extra = 0 as ::core::ffi::c_int;
        (*buf).b_ind_open_left_imag = 0 as ::core::ffi::c_int;
        (*buf).b_ind_jump_label = -1 as ::core::ffi::c_int;
        (*buf).b_ind_case = sw;
        (*buf).b_ind_case_code = sw;
        (*buf).b_ind_case_break = 0 as ::core::ffi::c_int;
        (*buf).b_ind_scopedecl = sw;
        (*buf).b_ind_scopedecl_code = sw;
        (*buf).b_ind_param = sw;
        (*buf).b_ind_func_type = sw;
        (*buf).b_ind_cpp_baseclass = sw;
        (*buf).b_ind_continuation = sw;
        (*buf).b_ind_unclosed = sw * 2 as ::core::ffi::c_int;
        (*buf).b_ind_unclosed2 = sw;
        (*buf).b_ind_unclosed_noignore = 0 as ::core::ffi::c_int;
        (*buf).b_ind_unclosed_wrapped = 0 as ::core::ffi::c_int;
        (*buf).b_ind_unclosed_whiteok = 0 as ::core::ffi::c_int;
        (*buf).b_ind_matching_paren = 0 as ::core::ffi::c_int;
        (*buf).b_ind_paren_prev = 0 as ::core::ffi::c_int;
        (*buf).b_ind_comment = 0 as ::core::ffi::c_int;
        (*buf).b_ind_in_comment = 3 as ::core::ffi::c_int;
        (*buf).b_ind_in_comment2 = 0 as ::core::ffi::c_int;
        (*buf).b_ind_maxparen = 20 as ::core::ffi::c_int;
        (*buf).b_ind_maxcomment = 70 as ::core::ffi::c_int;
        (*buf).b_ind_java = 0 as ::core::ffi::c_int;
        (*buf).b_ind_js = 0 as ::core::ffi::c_int;
        (*buf).b_ind_keep_case_label = 0 as ::core::ffi::c_int;
        (*buf).b_ind_cpp_namespace = 0 as ::core::ffi::c_int;
        (*buf).b_ind_if_for_while = 0 as ::core::ffi::c_int;
        (*buf).b_ind_hash_comment = 0 as ::core::ffi::c_int;
        (*buf).b_ind_cpp_extern_c = 0 as ::core::ffi::c_int;
        (*buf).b_ind_pragma = 0 as ::core::ffi::c_int;
        p = (*buf).b_p_cino;
        while *p != 0 {
            let c2rust_fresh0 = p;
            p = p.offset(1);
            l = c2rust_fresh0;
            if *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int {
                p = p.offset(1);
            }
            let mut digits_start: *mut ::core::ffi::c_char = p;
            let mut n: int64_t =
                getdigits_int(&raw mut p, true_0 != 0, 0 as ::core::ffi::c_int) as int64_t;
            divider = 0 as ::core::ffi::c_int;
            if *p as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
                p = p.offset(1);
                fraction = atoi(p);
                while ascii_isdigit(*p as ::core::ffi::c_int) {
                    p = p.offset(1);
                    if divider != 0 {
                        divider *= 10 as ::core::ffi::c_int;
                    } else {
                        divider = 10 as ::core::ffi::c_int;
                    }
                }
            }
            if *p as ::core::ffi::c_int == 's' as ::core::ffi::c_int {
                if p == digits_start {
                    n = sw as int64_t;
                } else {
                    n *= sw as int64_t;
                    if divider != 0 {
                        n += (sw as int64_t * fraction as int64_t
                            + (divider / 2 as ::core::ffi::c_int) as int64_t)
                            / divider as int64_t;
                    }
                }
                p = p.offset(1);
            }
            if *l.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                == '-' as ::core::ffi::c_int
            {
                n = -n;
            }
            n = crate::src::nvim::math::trim_to_int(n) as int64_t;
            match *l as ::core::ffi::c_int {
                62 => {
                    (*buf).b_ind_level = n as ::core::ffi::c_int;
                }
                101 => {
                    (*buf).b_ind_open_imag = n as ::core::ffi::c_int;
                }
                110 => {
                    (*buf).b_ind_no_brace = n as ::core::ffi::c_int;
                }
                102 => {
                    (*buf).b_ind_first_open = n as ::core::ffi::c_int;
                }
                123 => {
                    (*buf).b_ind_open_extra = n as ::core::ffi::c_int;
                }
                125 => {
                    (*buf).b_ind_close_extra = n as ::core::ffi::c_int;
                }
                94 => {
                    (*buf).b_ind_open_left_imag = n as ::core::ffi::c_int;
                }
                76 => {
                    (*buf).b_ind_jump_label = n as ::core::ffi::c_int;
                }
                58 => {
                    (*buf).b_ind_case = n as ::core::ffi::c_int;
                }
                61 => {
                    (*buf).b_ind_case_code = n as ::core::ffi::c_int;
                }
                98 => {
                    (*buf).b_ind_case_break = n as ::core::ffi::c_int;
                }
                112 => {
                    (*buf).b_ind_param = n as ::core::ffi::c_int;
                }
                116 => {
                    (*buf).b_ind_func_type = n as ::core::ffi::c_int;
                }
                47 => {
                    (*buf).b_ind_comment = n as ::core::ffi::c_int;
                }
                99 => {
                    (*buf).b_ind_in_comment = n as ::core::ffi::c_int;
                }
                67 => {
                    (*buf).b_ind_in_comment2 = n as ::core::ffi::c_int;
                }
                105 => {
                    (*buf).b_ind_cpp_baseclass = n as ::core::ffi::c_int;
                }
                43 => {
                    (*buf).b_ind_continuation = n as ::core::ffi::c_int;
                }
                40 => {
                    (*buf).b_ind_unclosed = n as ::core::ffi::c_int;
                }
                117 => {
                    (*buf).b_ind_unclosed2 = n as ::core::ffi::c_int;
                }
                85 => {
                    (*buf).b_ind_unclosed_noignore = n as ::core::ffi::c_int;
                }
                87 => {
                    (*buf).b_ind_unclosed_wrapped = n as ::core::ffi::c_int;
                }
                119 => {
                    (*buf).b_ind_unclosed_whiteok = n as ::core::ffi::c_int;
                }
                109 => {
                    (*buf).b_ind_matching_paren = n as ::core::ffi::c_int;
                }
                77 => {
                    (*buf).b_ind_paren_prev = n as ::core::ffi::c_int;
                }
                41 => {
                    (*buf).b_ind_maxparen = n as ::core::ffi::c_int;
                }
                42 => {
                    (*buf).b_ind_maxcomment = n as ::core::ffi::c_int;
                }
                103 => {
                    (*buf).b_ind_scopedecl = n as ::core::ffi::c_int;
                }
                104 => {
                    (*buf).b_ind_scopedecl_code = n as ::core::ffi::c_int;
                }
                106 => {
                    (*buf).b_ind_java = n as ::core::ffi::c_int;
                }
                74 => {
                    (*buf).b_ind_js = n as ::core::ffi::c_int;
                }
                108 => {
                    (*buf).b_ind_keep_case_label = n as ::core::ffi::c_int;
                }
                35 => {
                    (*buf).b_ind_hash_comment = n as ::core::ffi::c_int;
                }
                78 => {
                    (*buf).b_ind_cpp_namespace = n as ::core::ffi::c_int;
                }
                107 => {
                    (*buf).b_ind_if_for_while = n as ::core::ffi::c_int;
                }
                69 => {
                    (*buf).b_ind_cpp_extern_c = n as ::core::ffi::c_int;
                }
                80 => {
                    (*buf).b_ind_pragma = n as ::core::ffi::c_int;
                }
                _ => {}
            }
            if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                p = p.offset(1);
            }
        }
    }
}
