//! Compiling `'errorformat'` into regular expressions.
//!
//! The option is a comma-separated list of formats; [`parse_efm_option`]
//! splits it and turns each one into an `efm_T` holding a compiled pattern.
//! [`efm_to_regpat`] is the translator — `%f`, `%l`, `%m` and the rest
//! become capture groups from the `fmt_pat` table, `%*` takes a scanf
//! conversion ([`scanf_fmt_to_regpat`]), and everything else is escaped.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn efmpat_to_regpat(
    mut efmpat: *const ::core::ffi::c_char,
    mut regpat: *mut ::core::ffi::c_char,
    mut efminfo: *mut efm_T,
    mut idx: ::core::ffi::c_int,
    mut round: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        if (*efminfo).addr[idx as usize] != 0 {
            semsg(
                gettext(b"E372: Too many %%%c in format string\0".as_ptr()
                    as *const ::core::ffi::c_char),
                *efmpat as ::core::ffi::c_int,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        if idx != 0
            && idx < FMT_PATTERN_R
            && !vim_strchr(
                b"DXOPQ\0".as_ptr() as *const ::core::ffi::c_char,
                (*efminfo).prefix as uint8_t as ::core::ffi::c_int,
            )
            .is_null()
            || idx == FMT_PATTERN_R
                && vim_strchr(
                    b"OPQ\0".as_ptr() as *const ::core::ffi::c_char,
                    (*efminfo).prefix as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
        {
            semsg(
                gettext(b"E373: Unexpected %%%c in format string\0".as_ptr()
                    as *const ::core::ffi::c_char),
                *efmpat as ::core::ffi::c_int,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        round += 1;
        (*efminfo).addr[idx as usize] = round as ::core::ffi::c_char;
        let c2rust_fresh16 = regpat;
        regpat = regpat.offset(1);
        *c2rust_fresh16 = '\\' as ::core::ffi::c_char;
        let c2rust_fresh17 = regpat;
        regpat = regpat.offset(1);
        *c2rust_fresh17 = '(' as ::core::ffi::c_char;
        if *efmpat as ::core::ffi::c_int == 'f' as ::core::ffi::c_int
            && *efmpat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            if *efmpat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                != '\\' as ::core::ffi::c_int
                && *efmpat.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    != '%' as ::core::ffi::c_int
            {
                strcpy(
                    regpat,
                    b".\\{-1,}\0".as_ptr() as *const ::core::ffi::c_char
                        as *mut ::core::ffi::c_char,
                );
                regpat = regpat.offset(7 as ::core::ffi::c_int as isize);
            } else {
                strcpy(
                    regpat,
                    b"\\f\\+\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char,
                );
                regpat = regpat.offset(4 as ::core::ffi::c_int as isize);
            }
        } else {
            let mut srcptr: *mut ::core::ffi::c_char = (*fmt_pat.ptr())[idx as usize].pattern;
            loop {
                let c2rust_fresh18 = srcptr;
                srcptr = srcptr.offset(1);
                *regpat = *c2rust_fresh18;
                if *regpat as ::core::ffi::c_int == NUL {
                    break;
                }
                regpat = regpat.offset(1);
            }
        }
        let c2rust_fresh19 = regpat;
        regpat = regpat.offset(1);
        *c2rust_fresh19 = '\\' as ::core::ffi::c_char;
        let c2rust_fresh20 = regpat;
        regpat = regpat.offset(1);
        *c2rust_fresh20 = ')' as ::core::ffi::c_char;
        return regpat;
    }
}

pub(crate) unsafe extern "C" fn scanf_fmt_to_regpat(
    mut pefmp: *mut *const ::core::ffi::c_char,
    mut efm: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut regpat: *mut ::core::ffi::c_char,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut efmp: *const ::core::ffi::c_char = *pefmp;
        if *efmp as ::core::ffi::c_int == '[' as ::core::ffi::c_int
            || *efmp as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
        {
            let c2rust_fresh9 = regpat;
            regpat = regpat.offset(1);
            let c2rust_lvalue_ptr = &raw mut *c2rust_fresh9;
            *c2rust_lvalue_ptr = *efmp;
            if *c2rust_lvalue_ptr as ::core::ffi::c_int == '[' as ::core::ffi::c_int {
                if *efmp.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '^' as ::core::ffi::c_int
                {
                    efmp = efmp.offset(1);
                    let c2rust_fresh10 = regpat;
                    regpat = regpat.offset(1);
                    *c2rust_fresh10 = *efmp;
                }
                if efmp < efm.offset(len as isize) {
                    efmp = efmp.offset(1);
                    let c2rust_fresh11 = regpat;
                    regpat = regpat.offset(1);
                    *c2rust_fresh11 = *efmp;
                    while efmp < efm.offset(len as isize) && {
                        efmp = efmp.offset(1);
                        let c2rust_fresh12 = regpat;
                        regpat = regpat.offset(1);
                        let c2rust_lvalue_ptr_0 = &raw mut *c2rust_fresh12;
                        *c2rust_lvalue_ptr_0 = *efmp;
                        *c2rust_lvalue_ptr_0 as ::core::ffi::c_int != ']' as ::core::ffi::c_int
                    } {}
                    if efmp == efm.offset(len as isize) {
                        emsg(gettext(b"E374: Missing ] in format string\0".as_ptr()
                            as *const ::core::ffi::c_char));
                        return ::core::ptr::null_mut::<::core::ffi::c_char>();
                    }
                }
            } else if efmp < efm.offset(len as isize) {
                efmp = efmp.offset(1);
                let c2rust_fresh13 = regpat;
                regpat = regpat.offset(1);
                *c2rust_fresh13 = *efmp;
            }
            let c2rust_fresh14 = regpat;
            regpat = regpat.offset(1);
            *c2rust_fresh14 = '\\' as ::core::ffi::c_char;
            let c2rust_fresh15 = regpat;
            regpat = regpat.offset(1);
            *c2rust_fresh15 = '+' as ::core::ffi::c_char;
        } else {
            semsg(
                gettext(b"E375: Unsupported %%%c in format string\0".as_ptr()
                    as *const ::core::ffi::c_char),
                *efmp as ::core::ffi::c_int,
            );
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        *pefmp = efmp;
        return regpat;
    }
}

pub(crate) unsafe extern "C" fn efm_analyze_prefix(
    mut efmp: *const ::core::ffi::c_char,
    mut efminfo: *mut efm_T,
) -> *const ::core::ffi::c_char {
    unsafe {
        if !vim_strchr(
            b"+-\0".as_ptr() as *const ::core::ffi::c_char,
            *efmp as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            let c2rust_fresh8 = efmp;
            efmp = efmp.offset(1);
            (*efminfo).flags = *c2rust_fresh8;
        }
        if !vim_strchr(
            b"DXAEWINCZGOPQ\0".as_ptr() as *const ::core::ffi::c_char,
            *efmp as uint8_t as ::core::ffi::c_int,
        )
        .is_null()
        {
            (*efminfo).prefix = *efmp;
        } else {
            semsg(
                gettext(b"E376: Invalid %%%c in format string prefix\0".as_ptr()
                    as *const ::core::ffi::c_char),
                *efmp as ::core::ffi::c_int,
            );
            return ::core::ptr::null::<::core::ffi::c_char>();
        }
        return efmp;
    }
}

pub(crate) unsafe extern "C" fn efm_to_regpat(
    mut efm: *const ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
    mut fmt_ptr: *mut efm_T,
    mut regpat: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut ptr: *mut ::core::ffi::c_char = regpat;
        let c2rust_fresh2 = ptr;
        ptr = ptr.offset(1);
        *c2rust_fresh2 = '^' as ::core::ffi::c_char;
        let mut round: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut efmp: *const ::core::ffi::c_char = efm;
        while efmp < efm.offset(len as isize) {
            if *efmp as ::core::ffi::c_int == '%' as ::core::ffi::c_int {
                efmp = efmp.offset(1);
                let mut idx: ::core::ffi::c_int = 0;
                idx = 0 as ::core::ffi::c_int;
                while idx < FMT_PATTERNS {
                    if (*fmt_pat.ptr())[idx as usize].convchar as ::core::ffi::c_int
                        == *efmp as ::core::ffi::c_int
                    {
                        break;
                    }
                    idx += 1;
                }
                if idx < FMT_PATTERNS {
                    ptr = efmpat_to_regpat(efmp, ptr, fmt_ptr, idx, round);
                    if ptr.is_null() {
                        return FAIL;
                    }
                    round += 1;
                } else if *efmp as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
                    efmp = efmp.offset(1);
                    ptr = scanf_fmt_to_regpat(&raw mut efmp, efm, len, ptr);
                    if ptr.is_null() {
                        return FAIL;
                    }
                } else if !vim_strchr(
                    b"%\\.^$~[\0".as_ptr() as *const ::core::ffi::c_char,
                    *efmp as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
                {
                    let c2rust_fresh3 = ptr;
                    ptr = ptr.offset(1);
                    *c2rust_fresh3 = *efmp;
                } else if *efmp as ::core::ffi::c_int == '#' as ::core::ffi::c_int {
                    let c2rust_fresh4 = ptr;
                    ptr = ptr.offset(1);
                    *c2rust_fresh4 = '*' as ::core::ffi::c_char;
                } else if *efmp as ::core::ffi::c_int == '>' as ::core::ffi::c_int {
                    (*fmt_ptr).conthere = true_0;
                } else if efmp == efm.offset(1 as ::core::ffi::c_int as isize) {
                    efmp = efm_analyze_prefix(efmp, fmt_ptr);
                    if efmp.is_null() {
                        return FAIL;
                    }
                } else {
                    semsg(
                        gettext(b"E377: Invalid %%%c in format string\0".as_ptr()
                            as *const ::core::ffi::c_char),
                        *efmp as ::core::ffi::c_int,
                    );
                    return FAIL;
                }
            } else {
                if *efmp as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                    && efmp.offset(1 as ::core::ffi::c_int as isize) < efm.offset(len as isize)
                {
                    efmp = efmp.offset(1);
                } else if !vim_strchr(
                    b".*^$~[\0".as_ptr() as *const ::core::ffi::c_char,
                    *efmp as uint8_t as ::core::ffi::c_int,
                )
                .is_null()
                {
                    let c2rust_fresh5 = ptr;
                    ptr = ptr.offset(1);
                    *c2rust_fresh5 = '\\' as ::core::ffi::c_char;
                }
                if *efmp != 0 {
                    let c2rust_fresh6 = ptr;
                    ptr = ptr.offset(1);
                    *c2rust_fresh6 = *efmp;
                }
            }
            efmp = efmp.offset(1);
        }
        let c2rust_fresh7 = ptr;
        ptr = ptr.offset(1);
        *c2rust_fresh7 = '$' as ::core::ffi::c_char;
        *ptr = NUL as ::core::ffi::c_char;
        return OK;
    }
}

pub(crate) unsafe extern "C" fn free_efm_list(mut efm_first: *mut *mut efm_T) {
    unsafe {
        let mut efm_ptr: *mut efm_T = *efm_first;
        while !efm_ptr.is_null() {
            *efm_first = (*efm_ptr).next;
            vim_regfree((*efm_ptr).prog);
            xfree(efm_ptr as *mut ::core::ffi::c_void);
            efm_ptr = *efm_first;
        }
        fmt_start.set(::core::ptr::null_mut::<efm_T>());
    }
}

pub(crate) unsafe extern "C" fn efm_regpat_bufsz(mut efm: *mut ::core::ffi::c_char) -> size_t {
    unsafe {
        let mut sz: size_t = ((FMT_PATTERNS * 3 as ::core::ffi::c_int) as size_t)
            .wrapping_add(strlen(efm) << 2 as ::core::ffi::c_int);
        let mut i: ::core::ffi::c_int = FMT_PATTERNS - 1 as ::core::ffi::c_int;
        while i >= 0 as ::core::ffi::c_int {
            let c2rust_fresh1 = i;
            i = i - 1;
            sz = sz.wrapping_add(strlen((*fmt_pat.ptr())[c2rust_fresh1 as usize].pattern));
        }
        sz = sz.wrapping_add(2 as size_t);
        return sz;
    }
}

pub(crate) unsafe extern "C" fn efm_option_part_len(
    mut efm: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let mut len: ::core::ffi::c_int = 0;
        len = 0 as ::core::ffi::c_int;
        while *efm.offset(len as isize) as ::core::ffi::c_int != NUL
            && *efm.offset(len as isize) as ::core::ffi::c_int != ',' as ::core::ffi::c_int
        {
            if *efm.offset(len as isize) as ::core::ffi::c_int == '\\' as ::core::ffi::c_int
                && *efm.offset((len + 1 as ::core::ffi::c_int) as isize) as ::core::ffi::c_int
                    != NUL
            {
                len += 1;
            }
            len += 1;
        }
        return len;
    }
}

pub(crate) unsafe extern "C" fn parse_efm_option(mut efm: *mut ::core::ffi::c_char) -> *mut efm_T {
    unsafe {
        let mut fmt_first: *mut efm_T = ::core::ptr::null_mut::<efm_T>();
        let mut fmt_last: *mut efm_T = ::core::ptr::null_mut::<efm_T>();
        let mut sz: size_t = efm_regpat_bufsz(efm);
        let mut fmtstr: *mut ::core::ffi::c_char = xmalloc(sz) as *mut ::core::ffi::c_char;
        '_parse_efm_end: {
            '_parse_efm_error: {
                while *efm.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL {
                    let mut fmt_ptr: *mut efm_T =
                        xcalloc(1 as size_t, ::core::mem::size_of::<efm_T>()) as *mut efm_T;
                    if fmt_first.is_null() {
                        fmt_first = fmt_ptr;
                    } else {
                        (*fmt_last).next = fmt_ptr;
                    }
                    fmt_last = fmt_ptr;
                    let mut len: ::core::ffi::c_int = efm_option_part_len(efm);
                    if efm_to_regpat(efm, len, fmt_ptr, fmtstr) == FAIL {
                        break '_parse_efm_error;
                    }
                    (*fmt_ptr).prog = vim_regcomp(fmtstr, RE_MAGIC + RE_STRING);
                    if (*fmt_ptr).prog.is_null() {
                        break '_parse_efm_error;
                    }
                    efm = skip_to_option_part(efm.offset(len as isize));
                }
                if fmt_first.is_null() {
                    emsg(gettext(
                        b"E378: 'errorformat' contains no pattern\0".as_ptr()
                            as *const ::core::ffi::c_char,
                    ));
                }
                break '_parse_efm_end;
            }
            free_efm_list(&raw mut fmt_first);
        }
        xfree(fmtstr as *mut ::core::ffi::c_void);
        return fmt_first;
    }
}
