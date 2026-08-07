//! The `vim_snprintf` family: entry points and argument fetchers.
//!
//! Every spelling funnels into `vim_vsnprintf_typval`.  The variadic ones take
//! a C `va_list`; `printf()` and friends pass a `typval_T` array instead, and
//! `tv_nr`/`tv_str`/`tv_ptr`/`tv_float` are the fetchers that read one
//! Vimscript argument out of it with the type checking C's varargs cannot do.
//! `kv_do_printf` and `arena_printf` are the two spellings that format into a
//! growable buffer rather than a fixed one.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

// The carve of the transpiled module; see each child's docs.
mod emit;
mod spec;

pub use self::emit::*;
pub use self::spec::*;

static e_printf: GlobalCell<*const ::core::ffi::c_char> = GlobalCell::new(
    b"E766: Insufficient arguments for printf()\0".as_ptr() as *const ::core::ffi::c_char,
);

pub(crate) unsafe extern "C" fn tv_nr(
    mut tvs: *mut typval_T,
    mut idxp: *mut ::core::ffi::c_int,
) -> varnumber_T {
    unsafe {
        let mut idx: ::core::ffi::c_int = *idxp - 1 as ::core::ffi::c_int;
        let mut n: varnumber_T = 0 as varnumber_T;
        if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(e_printf.get()));
        } else {
            *idxp += 1;
            let mut err: bool = false_0 != 0;
            n = tv_get_number_chk(tvs.offset(idx as isize), &raw mut err);
            if err {
                n = 0 as varnumber_T;
            }
        }
        return n;
    }
}

pub(crate) unsafe extern "C" fn tv_str(
    mut tvs: *mut typval_T,
    mut idxp: *mut ::core::ffi::c_int,
    tofree: *mut *mut ::core::ffi::c_char,
) -> *const ::core::ffi::c_char {
    unsafe {
        let mut idx: ::core::ffi::c_int = *idxp - 1 as ::core::ffi::c_int;
        let mut s: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(e_printf.get()));
        } else {
            *idxp += 1;
            if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
                == VAR_STRING as ::core::ffi::c_int as ::core::ffi::c_uint
                || (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
                    == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                s = tv_get_string_chk(tvs.offset(idx as isize));
                *tofree = ::core::ptr::null_mut::<::core::ffi::c_char>();
            } else {
                *tofree =
                    encode_tv2echo(tvs.offset(idx as isize), ::core::ptr::null_mut::<size_t>());
                s = *tofree;
            }
        }
        return s;
    }
}

pub(crate) unsafe extern "C" fn tv_ptr(
    tvs: *const typval_T,
    idxp: *mut ::core::ffi::c_int,
) -> *const ::core::ffi::c_void {
    unsafe {
        let idx: ::core::ffi::c_int = *idxp - 1 as ::core::ffi::c_int;
        if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(e_printf.get()));
            return ::core::ptr::null::<::core::ffi::c_void>();
        }
        *idxp += 1;
        return (*tvs.offset(idx as isize)).vval.v_string as *const ::core::ffi::c_void;
    }
}

pub(crate) unsafe extern "C" fn tv_float(
    tvs: *mut typval_T,
    idxp: *mut ::core::ffi::c_int,
) -> float_T {
    unsafe {
        let mut idx: ::core::ffi::c_int = *idxp - 1 as ::core::ffi::c_int;
        let mut f: float_T = 0 as ::core::ffi::c_int as float_T;
        if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            emsg(gettext(e_printf.get()));
        } else {
            *idxp += 1;
            if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
                == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                f = (*tvs.offset(idx as isize)).vval.v_float;
            } else if (*tvs.offset(idx as isize)).v_type as ::core::ffi::c_uint
                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                f = (*tvs.offset(idx as isize)).vval.v_number as float_T;
            } else {
                emsg(gettext(
                    b"E807: Expected Float argument for printf()\0".as_ptr()
                        as *const ::core::ffi::c_char,
                ));
            }
        }
        return f;
    }
}

pub unsafe extern "C" fn vim_snprintf_add(
    mut str: *mut ::core::ffi::c_char,
    mut str_m: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> ::core::ffi::c_int {
    unsafe {
        let len: size_t = strlen(str);
        let mut space: size_t = 0;
        if str_m <= len {
            space = 0 as size_t;
        } else {
            space = str_m.wrapping_sub(len);
        }
        let mut ap: ::core::ffi::VaList;
        ap = c2rust_args.clone();
        let str_l: ::core::ffi::c_int = vim_vsnprintf(str.offset(len as isize), space, fmt, ap);
        return str_l;
    }
}

pub unsafe extern "C" fn vim_snprintf(
    mut str: *mut ::core::ffi::c_char,
    mut str_m: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> ::core::ffi::c_int {
    unsafe {
        let mut ap: ::core::ffi::VaList;
        ap = c2rust_args.clone();
        let str_l: ::core::ffi::c_int = vim_vsnprintf(str, str_m, fmt, ap);
        return str_l;
    }
}

pub(crate) unsafe extern "C" fn infinity_str(
    mut positive: bool,
    mut fmt_spec: ::core::ffi::c_char,
    mut force_sign: ::core::ffi::c_int,
    mut space_for_positive: ::core::ffi::c_int,
) -> *const ::core::ffi::c_char {
    unsafe {
        static table: GlobalCell<[*const ::core::ffi::c_char; 8]> = GlobalCell::new([
            b"-inf\0".as_ptr() as *const ::core::ffi::c_char,
            b"inf\0".as_ptr() as *const ::core::ffi::c_char,
            b"+inf\0".as_ptr() as *const ::core::ffi::c_char,
            b" inf\0".as_ptr() as *const ::core::ffi::c_char,
            b"-INF\0".as_ptr() as *const ::core::ffi::c_char,
            b"INF\0".as_ptr() as *const ::core::ffi::c_char,
            b"+INF\0".as_ptr() as *const ::core::ffi::c_char,
            b" INF\0".as_ptr() as *const ::core::ffi::c_char,
        ]);
        let mut idx: ::core::ffi::c_int = positive as ::core::ffi::c_int
            * (1 as ::core::ffi::c_int + force_sign + force_sign * space_for_positive);
        if fmt_spec as ::core::ffi::c_uint >= 'A' as ::core::ffi::c_uint
            && fmt_spec as ::core::ffi::c_uint <= 'Z' as ::core::ffi::c_uint
        {
            idx += 4 as ::core::ffi::c_int;
        }
        return (*table.ptr())[idx as usize];
    }
}

pub unsafe extern "C" fn vim_snprintf_safelen(
    mut str: *mut ::core::ffi::c_char,
    mut str_m: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> size_t {
    unsafe {
        let mut ap: ::core::ffi::VaList;
        let mut str_l: ::core::ffi::c_int = 0;
        if str_m == 0 as size_t {
            return 0 as size_t;
        }
        ap = c2rust_args.clone();
        str_l = vim_vsnprintf_typval(str, str_m, fmt, ap, ::core::ptr::null_mut::<typval_T>());
        if str_l < 0 as ::core::ffi::c_int {
            *str = NUL as ::core::ffi::c_char;
            return 0 as size_t;
        }
        return if str_l as size_t >= str_m {
            str_m.wrapping_sub(1 as size_t)
        } else {
            str_l as size_t
        };
    }
}

pub unsafe extern "C" fn vim_vsnprintf(
    mut str: *mut ::core::ffi::c_char,
    mut str_m: size_t,
    mut fmt: *const ::core::ffi::c_char,
    mut ap: ::core::ffi::VaList,
) -> ::core::ffi::c_int {
    unsafe {
        return vim_vsnprintf_typval(str, str_m, fmt, ap, ::core::ptr::null_mut::<typval_T>());
    }
}

pub const TMP_LEN: ::core::ffi::c_int = 350 as ::core::ffi::c_int;

pub unsafe extern "C" fn kv_do_printf(
    mut str: *mut StringBuilder,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> ::core::ffi::c_int {
    unsafe {
        let mut remaining: size_t = (*str).capacity.wrapping_sub((*str).size);
        let mut ap: ::core::ffi::VaList;
        ap = c2rust_args.clone();
        let mut printed: ::core::ffi::c_int = vsnprintf(
            if !(*str).items.is_null() {
                (*str).items.offset((*str).size as isize)
            } else {
                ::core::ptr::null_mut::<::core::ffi::c_char>()
            },
            remaining,
            fmt,
            ap,
        );
        if printed < 0 as ::core::ffi::c_int {
            return -1 as ::core::ffi::c_int;
        }
        if printed as size_t >= remaining {
            if (*str).capacity
                < (*str)
                    .size
                    .wrapping_add(printed as size_t)
                    .wrapping_add(1 as size_t)
            {
                (*str).capacity = (*str)
                    .size
                    .wrapping_add(printed as size_t)
                    .wrapping_add(1 as size_t);
                (*str).capacity = (*str).capacity.wrapping_sub(1);
                (*str).capacity |= (*str).capacity >> 1 as ::core::ffi::c_int;
                (*str).capacity |= (*str).capacity >> 2 as ::core::ffi::c_int;
                (*str).capacity |= (*str).capacity >> 4 as ::core::ffi::c_int;
                (*str).capacity |= (*str).capacity >> 8 as ::core::ffi::c_int;
                (*str).capacity |= (*str).capacity >> 16 as ::core::ffi::c_int;
                (*str).capacity = (*str).capacity.wrapping_add(1);
                (*str).items = xrealloc(
                    (*str).items as *mut ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_char>().wrapping_mul((*str).capacity),
                ) as *mut ::core::ffi::c_char;
            }
            '_c2rust_label: {
                if !(*str).items.is_null() {
                } else {
                    __assert_fail(
                        b"str->items != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/strings.rs\0".as_ptr() as *const ::core::ffi::c_char,
                        2321 as ::core::ffi::c_uint,
                        b"int kv_do_printf(StringBuilder *, const char *, ...)\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    );
                }
            };
            ap = c2rust_args.clone();
            printed = vsnprintf(
                (*str).items.offset((*str).size as isize),
                (*str).capacity.wrapping_sub((*str).size),
                fmt,
                ap,
            );
            if printed < 0 as ::core::ffi::c_int {
                return -1 as ::core::ffi::c_int;
            }
        }
        (*str).size = (*str).size.wrapping_add(printed as size_t);
        return printed;
    }
}

pub unsafe extern "C" fn arena_printf(
    mut arena: *mut Arena,
    mut fmt: *const ::core::ffi::c_char,
    mut c2rust_args: ...
) -> String_0 {
    unsafe {
        let mut remaining: size_t = 0 as size_t;
        let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if !arena.is_null() {
            if (*arena).cur_blk.is_null() {
                arena_alloc_block(arena);
            }
            remaining = (*arena).size.wrapping_sub((*arena).pos);
            buf = (*arena).cur_blk.offset((*arena).pos as isize);
        }
        let mut ap: ::core::ffi::VaList;
        ap = c2rust_args.clone();
        let mut printed: ::core::ffi::c_int = vsnprintf(buf, remaining, fmt, ap);
        if printed < 0 as ::core::ffi::c_int {
            return String_0 {
                data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                size: 0 as size_t,
            };
        }
        if printed as size_t >= remaining {
            buf = arena_alloc(
                arena,
                (printed as size_t).wrapping_add(1 as size_t),
                false_0 != 0,
            ) as *mut ::core::ffi::c_char;
            ap = c2rust_args.clone();
            printed = vsnprintf(buf, (printed as size_t).wrapping_add(1 as size_t), fmt, ap);
            if printed < 0 as ::core::ffi::c_int {
                return String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0 as size_t,
                };
            }
        } else {
            (*arena).pos = (*arena)
                .pos
                .wrapping_add((printed as size_t).wrapping_add(1 as size_t));
        }
        return String_0 {
            data: buf,
            size: printed as size_t,
        };
    }
}
