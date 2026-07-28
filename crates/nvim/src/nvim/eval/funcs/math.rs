//! Numbers: arithmetic, the bitwise operators and the random-number
//! generator.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

use super::*;

pub unsafe extern "C" fn f_abs(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        float_op_wrapper(
            argvars,
            rettv,
            EvalFuncData {
                float_func: Some(
                    fabs as unsafe extern "C" fn(::core::ffi::c_double) -> ::core::ffi::c_double,
                ),
            },
        );
    } else {
        let mut error: bool = false_0 != 0;
        let mut n: varnumber_T = tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if error {
            (*rettv).vval.v_number = -1 as varnumber_T;
        } else if n > 0 as varnumber_T {
            (*rettv).vval.v_number = n;
        } else {
            (*rettv).vval.v_number = -n;
        }
    };
}
pub unsafe extern "C" fn f_and(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) & tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
}
pub unsafe extern "C" fn f_atan2(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fx: float_T = 0.;
    let mut fy: float_T = 0.;
    (*rettv).v_type = VAR_FLOAT;
    if tv_get_float_chk(argvars, &raw mut fx) as ::core::ffi::c_int != 0
        && tv_get_float_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut fy,
        ) as ::core::ffi::c_int
            != 0
    {
        (*rettv).vval.v_float =
            atan2(fx as ::core::ffi::c_double, fy as ::core::ffi::c_double) as float_T;
    } else {
        (*rettv).vval.v_float = 0.0f64 as float_T;
    };
}
pub unsafe extern "C" fn f_float2nr(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut f: float_T = 0.;
    if !tv_get_float_chk(argvars, &raw mut f) {
        return;
    }
    if f <= -VARNUMBER_MAX as ::core::ffi::c_double + DBL_EPSILON {
        (*rettv).vval.v_number = -VARNUMBER_MAX as varnumber_T;
    } else if f >= VARNUMBER_MAX as ::core::ffi::c_double - DBL_EPSILON {
        (*rettv).vval.v_number = VARNUMBER_MAX as varnumber_T;
    } else {
        (*rettv).vval.v_number = f as varnumber_T;
    };
}
pub unsafe extern "C" fn f_fmod(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fx: float_T = 0.;
    let mut fy: float_T = 0.;
    (*rettv).v_type = VAR_FLOAT;
    if tv_get_float_chk(argvars, &raw mut fx) as ::core::ffi::c_int != 0
        && tv_get_float_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut fy,
        ) as ::core::ffi::c_int
            != 0
    {
        (*rettv).vval.v_float =
            fmod(fx as ::core::ffi::c_double, fy as ::core::ffi::c_double) as float_T;
    } else {
        (*rettv).vval.v_float = 0.0f64 as float_T;
    };
}
pub unsafe extern "C" fn f_invert(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = !tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
}
pub unsafe extern "C" fn f_isinf(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        && ((*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_float as ::core::ffi::c_double)
            .is_infinite()
    {
        (*rettv).vval.v_number = (if (*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_float
            > 0.0f64
        {
            1 as ::core::ffi::c_int
        } else {
            -1 as ::core::ffi::c_int
        }) as varnumber_T;
    }
}
pub unsafe extern "C" fn f_isnan(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = ((*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type
        as ::core::ffi::c_uint
        == VAR_FLOAT as ::core::ffi::c_int as ::core::ffi::c_uint
        && ((*argvars.offset(0 as ::core::ffi::c_int as isize))
            .vval
            .v_float as ::core::ffi::c_double)
            .is_nan()) as ::core::ffi::c_int as varnumber_T;
}
pub unsafe extern "C" fn f_or(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) | tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
}
pub unsafe extern "C" fn f_pow(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut fx: float_T = 0.;
    let mut fy: float_T = 0.;
    (*rettv).v_type = VAR_FLOAT;
    if tv_get_float_chk(argvars, &raw mut fx) as ::core::ffi::c_int != 0
        && tv_get_float_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut fy,
        ) as ::core::ffi::c_int
            != 0
    {
        (*rettv).vval.v_float =
            pow(fx as ::core::ffi::c_double, fy as ::core::ffi::c_double) as float_T;
    } else {
        (*rettv).vval.v_float = 0.0f64 as float_T;
    };
}
unsafe extern "C" fn init_srand(x: *mut uint32_t) {
    let mut buf: C2Rust_Unnamed_52 = C2Rust_Unnamed_52 { number: 0 };
    if uv_random(
        ::core::ptr::null_mut::<uv_loop_t>(),
        ::core::ptr::null_mut::<uv_random_t>(),
        &raw mut buf.bytes as *mut uint8_t as *mut ::core::ffi::c_void,
        ::core::mem::size_of::<[uint8_t; 4]>(),
        0 as ::core::ffi::c_uint,
        None,
    ) == 0 as ::core::ffi::c_int
    {
        *x = buf.number;
        return;
    }
    *x = os_hrtime() as uint32_t;
    *x ^= os_get_pid() as uint32_t;
}
#[inline(always)]
unsafe extern "C" fn splitmix32(x: *mut uint32_t) -> uint32_t {
    *x = (*x as ::core::ffi::c_uint).wrapping_add(0x9e3779b9 as ::core::ffi::c_uint) as uint32_t;
    let mut z: uint32_t = *x;
    z = (z ^ z >> 16 as ::core::ffi::c_int).wrapping_mul(0x85ebca6b as uint32_t);
    z = (z ^ z >> 13 as ::core::ffi::c_int).wrapping_mul(0xc2b2ae35 as uint32_t);
    return z ^ z >> 16 as ::core::ffi::c_int;
}
#[inline(always)]
unsafe extern "C" fn shuffle_xoshiro128starstar(
    x: *mut uint32_t,
    y: *mut uint32_t,
    z: *mut uint32_t,
    w: *mut uint32_t,
) -> uint32_t {
    let result: uint32_t = ((*y).wrapping_mul(5 as uint32_t) << 7 as ::core::ffi::c_int
        | (*y).wrapping_mul(5 as uint32_t) >> 32 as ::core::ffi::c_int - 7 as ::core::ffi::c_int)
        .wrapping_mul(9 as uint32_t);
    let t: uint32_t = *y << 9 as ::core::ffi::c_int;
    *z ^= *x;
    *w ^= *y;
    *y ^= *z;
    *x ^= *w;
    *z ^= t;
    *w = *w << 11 as ::core::ffi::c_int | *w >> 32 as ::core::ffi::c_int - 11 as ::core::ffi::c_int;
    return result;
}
pub unsafe extern "C" fn f_rand(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut result: uint32_t = 0;
    's_126: {
        if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            static gx: GlobalCell<uint32_t> = GlobalCell::new(0);
            static gy: GlobalCell<uint32_t> = GlobalCell::new(0);
            static gz: GlobalCell<uint32_t> = GlobalCell::new(0);
            static gw: GlobalCell<uint32_t> = GlobalCell::new(0);
            static initialized: GlobalCell<bool> = GlobalCell::new(false_0 != 0);
            if !initialized.get() {
                let mut x: uint32_t = 0 as uint32_t;
                init_srand(&raw mut x);
                gx.set(splitmix32(&raw mut x));
                gy.set(splitmix32(&raw mut x));
                gz.set(splitmix32(&raw mut x));
                gw.set(splitmix32(&raw mut x));
                initialized.set(true_0 != 0);
            }
            result = shuffle_xoshiro128starstar(gx.ptr(), gy.ptr(), gz.ptr(), gw.ptr());
        } else {
            if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
                == VAR_LIST as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                let l: *mut list_T = (*argvars.offset(0 as ::core::ffi::c_int as isize))
                    .vval
                    .v_list;
                if tv_list_len(l) == 4 as ::core::ffi::c_int {
                    let tvx: *mut typval_T = &raw mut (*(tv_list_find
                        as unsafe extern "C" fn(
                            *mut list_T,
                            ::core::ffi::c_int,
                        ) -> *mut listitem_T)(
                        l, 0 as ::core::ffi::c_int
                    ))
                    .li_tv;
                    let tvy: *mut typval_T = &raw mut (*(tv_list_find
                        as unsafe extern "C" fn(
                            *mut list_T,
                            ::core::ffi::c_int,
                        ) -> *mut listitem_T)(
                        l, 1 as ::core::ffi::c_int
                    ))
                    .li_tv;
                    let tvz: *mut typval_T = &raw mut (*(tv_list_find
                        as unsafe extern "C" fn(
                            *mut list_T,
                            ::core::ffi::c_int,
                        ) -> *mut listitem_T)(
                        l, 2 as ::core::ffi::c_int
                    ))
                    .li_tv;
                    let tvw: *mut typval_T = &raw mut (*(tv_list_find
                        as unsafe extern "C" fn(
                            *mut list_T,
                            ::core::ffi::c_int,
                        ) -> *mut listitem_T)(
                        l, 3 as ::core::ffi::c_int
                    ))
                    .li_tv;
                    if (*tvx).v_type as ::core::ffi::c_uint
                        == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                        if (*tvy).v_type as ::core::ffi::c_uint
                            == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                        {
                            if (*tvz).v_type as ::core::ffi::c_uint
                                == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                            {
                                if (*tvw).v_type as ::core::ffi::c_uint
                                    == VAR_NUMBER as ::core::ffi::c_int as ::core::ffi::c_uint
                                {
                                    let mut x_0: uint32_t = (*tvx).vval.v_number as uint32_t;
                                    let mut y: uint32_t = (*tvy).vval.v_number as uint32_t;
                                    let mut z: uint32_t = (*tvz).vval.v_number as uint32_t;
                                    let mut w: uint32_t = (*tvw).vval.v_number as uint32_t;
                                    result = shuffle_xoshiro128starstar(
                                        &raw mut x_0,
                                        &raw mut y,
                                        &raw mut z,
                                        &raw mut w,
                                    );
                                    (*tvx).vval.v_number = x_0 as varnumber_T;
                                    (*tvy).vval.v_number = y as varnumber_T;
                                    (*tvz).vval.v_number = z as varnumber_T;
                                    (*tvw).vval.v_number = w as varnumber_T;
                                    break 's_126;
                                }
                            }
                        }
                    }
                }
            }
            semsg(
                gettext(&raw const e_invarg2 as *const ::core::ffi::c_char),
                tv_get_string(argvars.offset(0 as ::core::ffi::c_int as isize)),
            );
            (*rettv).v_type = VAR_NUMBER;
            (*rettv).vval.v_number = -1 as varnumber_T;
            return;
        }
    }
    (*rettv).v_type = VAR_NUMBER;
    (*rettv).vval.v_number = result as varnumber_T;
}
pub unsafe extern "C" fn f_srand(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut x: uint32_t = 0 as uint32_t;
    tv_list_alloc_ret(rettv, 4 as ptrdiff_t);
    if (*argvars.offset(0 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        init_srand(&raw mut x);
    } else {
        let mut error: bool = false_0 != 0;
        x = tv_get_number_chk(
            argvars.offset(0 as ::core::ffi::c_int as isize),
            &raw mut error,
        ) as uint32_t;
        if error {
            return;
        }
    }
    tv_list_append_number((*rettv).vval.v_list, splitmix32(&raw mut x) as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, splitmix32(&raw mut x) as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, splitmix32(&raw mut x) as varnumber_T);
    tv_list_append_number((*rettv).vval.v_list, splitmix32(&raw mut x) as varnumber_T);
}
pub unsafe extern "C" fn f_range(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut end: varnumber_T = 0;
    let mut stride: varnumber_T = 1 as varnumber_T;
    let mut error: bool = false_0 != 0;
    let mut start: varnumber_T = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        &raw mut error,
    );
    if (*argvars.offset(1 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
        == VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
    {
        end = start - 1 as varnumber_T;
        start = 0 as varnumber_T;
    } else {
        end = tv_get_number_chk(
            argvars.offset(1 as ::core::ffi::c_int as isize),
            &raw mut error,
        );
        if (*argvars.offset(2 as ::core::ffi::c_int as isize)).v_type as ::core::ffi::c_uint
            != VAR_UNKNOWN as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            stride = tv_get_number_chk(
                argvars.offset(2 as ::core::ffi::c_int as isize),
                &raw mut error,
            );
        }
    }
    if error {
        return;
    }
    if stride == 0 as varnumber_T {
        emsg(gettext(
            b"E726: Stride is zero\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    if if stride > 0 as varnumber_T {
        ((end + 1 as varnumber_T) < start) as ::core::ffi::c_int
    } else {
        (end - 1 as varnumber_T > start) as ::core::ffi::c_int
    } != 0
    {
        emsg(gettext(
            b"E727: Start past end\0".as_ptr() as *const ::core::ffi::c_char
        ));
        return;
    }
    tv_list_alloc_ret(
        rettv,
        (end as ptrdiff_t - start as ptrdiff_t) / stride as ptrdiff_t,
    );
    let mut i: varnumber_T = start;
    while if stride > 0 as varnumber_T {
        (i <= end) as ::core::ffi::c_int
    } else {
        (i >= end) as ::core::ffi::c_int
    } != 0
    {
        tv_list_append_number((*rettv).vval.v_list, i);
        i += stride;
    }
}
pub unsafe extern "C" fn f_str2float(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    let mut p: *mut ::core::ffi::c_char = skipwhite(tv_get_string(
        argvars.offset(0 as ::core::ffi::c_int as isize),
    ));
    let mut isneg: bool = *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int;
    if *p as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        || *p as ::core::ffi::c_int == '-' as ::core::ffi::c_int
    {
        p = skipwhite(p.offset(1 as ::core::ffi::c_int as isize));
    }
    string2float(p, &raw mut (*rettv).vval.v_float);
    if isneg {
        (*rettv).vval.v_float *= -1 as ::core::ffi::c_int as float_T;
    }
    (*rettv).v_type = VAR_FLOAT;
}
pub unsafe extern "C" fn f_xor(
    mut argvars: *mut typval_T,
    mut rettv: *mut typval_T,
    mut _fptr: EvalFuncData,
) {
    (*rettv).vval.v_number = tv_get_number_chk(
        argvars.offset(0 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    ) ^ tv_get_number_chk(
        argvars.offset(1 as ::core::ffi::c_int as isize),
        ::core::ptr::null_mut::<bool>(),
    );
}
