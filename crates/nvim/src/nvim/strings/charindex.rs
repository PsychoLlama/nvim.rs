//! Index conversion, and the substring builtins built on it.
//!
//! Vimscript addresses a string three ways -- by byte, by character and by
//! UTF-16 code unit -- and this is every conversion between them:
//! `byteidx()`/`byteidxcomp()`/`charidx()`/`utf16idx()` and the `strutf16len()`
//! that counts them.  `strgetchar()`, `strcharpart()` and `strpart()` are the
//! substring extractors that take their bounds in those units.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

unsafe extern "C" fn byteidx_common(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut comp: bool,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        let str: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut idx: varnumber_T = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        );
        if str.is_null() || idx < 0 as varnumber_T {
            return;
        }
        let mut utf16idx: varnumber_T = false_0 as varnumber_T;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut error: bool = false_0 != 0;
            utf16idx = tv_get_bool_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            );
            if error {
                return;
            }
            if utf16idx < 0 as varnumber_T || utf16idx > 1 as varnumber_T {
                semsg(
                    gettext(&raw const e_using_number_as_bool_nr as *const ::core::ffi::c_char),
                    utf16idx,
                );
                return;
            }
        }
        let mut ptr2len: Option<
            unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
        > = None;
        if comp {
            ptr2len = Some(
                utf_ptr2len
                    as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
            )
                as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
        } else {
            ptr2len = Some(
                utfc_ptr2len
                    as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
            )
                as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
        }
        let mut t: *const ::core::ffi::c_char = str;
        while idx > 0 as varnumber_T {
            if *t as ::core::ffi::c_int == NUL {
                return;
            }
            if utf16idx != 0 {
                let clen: ::core::ffi::c_int = ptr2len.expect("non-null function pointer")(t);
                let c: ::core::ffi::c_int = if clen > 1 as ::core::ffi::c_int {
                    utf_ptr2char(t)
                } else {
                    *t as ::core::ffi::c_int
                };
                if c > 0xffff as ::core::ffi::c_int {
                    idx -= 1;
                }
                if idx > 0 as varnumber_T {
                    t = t.offset(clen as isize);
                }
            } else if idx > 0 as varnumber_T {
                t = t.offset(ptr2len.expect("non-null function pointer")(t) as isize);
            }
            idx -= 1;
        }
        (*rettv).vval.v_number = t.offset_from(str) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_byteidx(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        byteidx_common(argvars, rettv, false_0 != 0);
    }
}

pub unsafe extern "C" fn f_byteidxcomp(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        byteidx_common(argvars, rettv, true_0 != 0);
    }
}

pub unsafe extern "C" fn f_charidx(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
            || tv_check_for_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
            || tv_check_for_opt_bool_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
            || (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && tv_check_for_opt_bool_arg(argvars, 3 as ::core::ffi::c_int) == FAIL
        {
            return;
        }
        let str: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut idx: varnumber_T = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        );
        if str.is_null() || idx < 0 as varnumber_T {
            return;
        }
        let mut countcc: varnumber_T = false_0 as varnumber_T;
        let mut utf16idx: varnumber_T = false_0 as varnumber_T;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            countcc = tv_get_bool(argvars.offset(2 as ::core::ffi::c_int as isize));
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                utf16idx = tv_get_bool(argvars.offset(3 as ::core::ffi::c_int as isize));
            }
        }
        let mut ptr2len: Option<
            unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
        > = None;
        if countcc != 0 {
            ptr2len = Some(
                utf_ptr2len
                    as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
            )
                as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
        } else {
            ptr2len = Some(
                utfc_ptr2len
                    as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
            )
                as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
        }
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut len: ::core::ffi::c_int = 0;
        p = str;
        len = 0 as ::core::ffi::c_int;
        while if utf16idx != 0 {
            (idx >= 0 as varnumber_T) as ::core::ffi::c_int
        } else {
            (p <= str.offset(idx as isize)) as ::core::ffi::c_int
        } != 0
        {
            if *p as ::core::ffi::c_int == NUL {
                if if utf16idx != 0 {
                    (idx == 0 as varnumber_T) as ::core::ffi::c_int
                } else {
                    (p == str.offset(idx as isize)) as ::core::ffi::c_int
                } != 0
                {
                    (*rettv).vval.v_number = len as varnumber_T;
                }
                return;
            }
            if utf16idx != 0 {
                idx -= 1;
                let clen: ::core::ffi::c_int = ptr2len.expect("non-null function pointer")(p);
                let c: ::core::ffi::c_int = if clen > 1 as ::core::ffi::c_int {
                    utf_ptr2char(p)
                } else {
                    *p as ::core::ffi::c_int
                };
                if c > 0xffff as ::core::ffi::c_int {
                    idx -= 1;
                }
            }
            p = p.offset(ptr2len.expect("non-null function pointer")(p) as isize);
            len += 1;
        }
        (*rettv).vval.v_number = (if len > 0 as ::core::ffi::c_int {
            len - 1 as ::core::ffi::c_int
        } else {
            0 as ::core::ffi::c_int
        }) as varnumber_T;
    }
}

pub unsafe extern "C" fn f_strgetchar(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        let str: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        if str.is_null() {
            return;
        }
        let mut error: bool = false_0 != 0;
        let mut charidx: varnumber_T = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if error {
            return;
        }
        let len: size_t = strlen(str);
        let mut byteidx: size_t = 0 as size_t;
        while charidx >= 0 as varnumber_T && byteidx < len {
            if charidx == 0 as varnumber_T {
                (*rettv).vval.v_number = utf_ptr2char(str.offset(byteidx as isize)) as varnumber_T;
                break;
            } else {
                charidx -= 1;
                byteidx = byteidx.wrapping_add(utf_ptr2len(str.offset(byteidx as isize)) as size_t);
            }
        }
    }
}

pub unsafe extern "C" fn f_strutf16len(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
            || tv_check_for_opt_bool_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
        {
            return;
        }
        let mut countcc: varnumber_T = false_0 as varnumber_T;
        if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            countcc = tv_get_bool(argvars.offset(1 as ::core::ffi::c_int as isize));
        }
        let mut s: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut len: varnumber_T = 0 as varnumber_T;
        let mut func_mb_ptr2char_adv: Option<
            unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
        > = None;
        func_mb_ptr2char_adv = (if countcc != 0 {
            Some(
                mb_cptr2char_adv
                    as unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
            )
        } else {
            Some(
                mb_ptr2char_adv
                    as unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int,
            )
        })
            as Option<unsafe extern "C" fn(*mut *const ::core::ffi::c_char) -> ::core::ffi::c_int>;
        while *s as ::core::ffi::c_int != NUL {
            let ch: ::core::ffi::c_int =
                func_mb_ptr2char_adv.expect("non-null function pointer")(&raw mut s);
            if ch > 0xffff as ::core::ffi::c_int {
                len += 1;
            }
            len += 1;
        }
        (*rettv).vval.v_number = len;
    }
}

pub unsafe extern "C" fn f_strcharpart(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let p: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let slen: size_t = strlen(p);
        let mut nbyte: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut skipcc: varnumber_T = false_0 as varnumber_T;
        let mut error: bool = false_0 != 0;
        let mut nchar: varnumber_T = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if !error {
            if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                    != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                skipcc = tv_get_bool_chk(
                    argvars.offset(3 as ::core::ffi::c_int as isize),
                    &raw mut error,
                );
                if error {
                    return;
                }
                if skipcc < 0 as varnumber_T || skipcc > 1 as varnumber_T {
                    semsg(
                        gettext(&raw const e_using_number_as_bool_nr as *const ::core::ffi::c_char),
                        skipcc,
                    );
                    return;
                }
            }
            if nchar > 0 as varnumber_T {
                while nchar > 0 as varnumber_T && (nbyte as size_t) < slen {
                    if skipcc != 0 {
                        nbyte += utfc_ptr2len(p.offset(nbyte as isize));
                    } else {
                        nbyte += utf_ptr2len(p.offset(nbyte as isize));
                    }
                    nchar -= 1;
                }
            } else {
                nbyte = nchar as ::core::ffi::c_int;
            }
        }
        let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut charlen: ::core::ffi::c_int =
                tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize))
                    as ::core::ffi::c_int;
            while charlen > 0 as ::core::ffi::c_int && nbyte + len < slen as ::core::ffi::c_int {
                let mut off: ::core::ffi::c_int = nbyte + len;
                if off < 0 as ::core::ffi::c_int {
                    len += 1 as ::core::ffi::c_int;
                } else if skipcc != 0 {
                    len += utfc_ptr2len(p.offset(off as isize));
                } else {
                    len += utf_ptr2len(p.offset(off as isize));
                }
                charlen -= 1;
            }
        } else {
            len = slen as ::core::ffi::c_int - nbyte;
        }
        if nbyte < 0 as ::core::ffi::c_int {
            len += nbyte;
            nbyte = 0 as ::core::ffi::c_int;
        } else if nbyte as size_t > slen {
            nbyte = slen as ::core::ffi::c_int;
        }
        if len < 0 as ::core::ffi::c_int {
            len = 0 as ::core::ffi::c_int;
        } else if nbyte + len > slen as ::core::ffi::c_int {
            len = slen as ::core::ffi::c_int - nbyte;
        }
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = xmemdupz(
            p.offset(nbyte as isize) as *const ::core::ffi::c_void,
            len as size_t,
        ) as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn f_strpart(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        let mut error: bool = false_0 != 0;
        let p: *const ::core::ffi::c_char =
            tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize));
        let slen: size_t = strlen(p);
        let mut n: varnumber_T = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        let mut len: varnumber_T = 0;
        if error {
            len = 0 as varnumber_T;
        } else if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            len = tv_get_number(argvars.offset(2 as ::core::ffi::c_int as isize));
        } else {
            len = slen as varnumber_T - n;
        }
        if n < 0 as varnumber_T {
            len += n;
            n = 0 as varnumber_T;
        } else if n > slen as varnumber_T {
            n = slen as varnumber_T;
        }
        if len < 0 as varnumber_T {
            len = 0 as varnumber_T;
        } else if n + len > slen as varnumber_T {
            len = slen as varnumber_T - n;
        }
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            && (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut off: int64_t = 0;
            off = n as int64_t;
            while off < slen as int64_t && len > 0 as varnumber_T {
                off += utfc_ptr2len(p.offset(off as isize)) as int64_t;
                len -= 1;
            }
            len = (off - n as int64_t) as varnumber_T;
        }
        (*rettv).v_type = VAR_STRING;
        (*rettv).vval.v_string = xmemdupz(
            p.offset(n as isize) as *const ::core::ffi::c_void,
            len as size_t,
        ) as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn f_utf16idx(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    unsafe {
        (*rettv).vval.v_number = -1 as varnumber_T;
        if tv_check_for_string_arg(argvars, 0 as ::core::ffi::c_int) == FAIL
            || tv_check_for_opt_number_arg(argvars, 1 as ::core::ffi::c_int) == FAIL
            || tv_check_for_opt_bool_arg(argvars, 2 as ::core::ffi::c_int) == FAIL
            || (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
                && tv_check_for_opt_bool_arg(argvars, 3 as ::core::ffi::c_int) == FAIL
        {
            return;
        }
        let str: *const ::core::ffi::c_char =
            tv_get_string_chk(argvars.offset(0 as ::core::ffi::c_int as isize));
        let mut idx: varnumber_T = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            ::core::ptr::null_mut::<bool>(),
        );
        if str.is_null() || idx < 0 as varnumber_T {
            return;
        }
        let mut countcc: varnumber_T = false_0 as varnumber_T;
        let mut charidx: varnumber_T = false_0 as varnumber_T;
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            countcc = tv_get_bool(argvars.offset(2 as ::core::ffi::c_int as isize));
            if (*argvars.offset(3 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                charidx = tv_get_bool(argvars.offset(3 as ::core::ffi::c_int as isize));
            }
        }
        let mut ptr2len: Option<
            unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
        > = None;
        if countcc != 0 {
            ptr2len = Some(
                utf_ptr2len
                    as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
            )
                as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
        } else {
            ptr2len = Some(
                utfc_ptr2len
                    as unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int,
            )
                as Option<unsafe extern "C" fn(*const ::core::ffi::c_char) -> ::core::ffi::c_int>;
        }
        let mut p: *const ::core::ffi::c_char = ::core::ptr::null::<::core::ffi::c_char>();
        let mut len: ::core::ffi::c_int = 0;
        let mut utf16idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        p = str;
        len = 0 as ::core::ffi::c_int;
        while if charidx != 0 {
            (idx >= 0 as varnumber_T) as ::core::ffi::c_int
        } else {
            (p <= str.offset(idx as isize)) as ::core::ffi::c_int
        } != 0
        {
            if *p as ::core::ffi::c_int == NUL {
                if if charidx != 0 {
                    (idx == 0 as varnumber_T) as ::core::ffi::c_int
                } else {
                    (p == str.offset(idx as isize)) as ::core::ffi::c_int
                } != 0
                {
                    (*rettv).vval.v_number = len as varnumber_T;
                }
                return;
            }
            utf16idx = len;
            let clen: ::core::ffi::c_int = ptr2len.expect("non-null function pointer")(p);
            let c: ::core::ffi::c_int = if clen > 1 as ::core::ffi::c_int {
                utf_ptr2char(p)
            } else {
                *p as ::core::ffi::c_int
            };
            if c > 0xffff as ::core::ffi::c_int {
                len += 1;
            }
            p = p.offset(clen as isize);
            if charidx != 0 {
                idx -= 1;
            }
            len += 1;
        }
        (*rettv).vval.v_number = utf16idx as varnumber_T;
    }
}
