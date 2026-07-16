extern "C" {
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
}
pub type int64_t = i64;
pub type uint64_t = u64;
pub type size_t = usize;
#[no_mangle]
pub unsafe extern "C" fn xfpclassify(mut d: ::core::ffi::c_double) -> ::core::ffi::c_int {
    let mut m: uint64_t = 0;
    memcpy(
        &raw mut m as *mut ::core::ffi::c_void,
        &raw mut d as *const ::core::ffi::c_void,
        ::core::mem::size_of::<uint64_t>(),
    );
    let mut e: ::core::ffi::c_int =
        (0x7ff as uint64_t & m >> 52 as ::core::ffi::c_int) as ::core::ffi::c_int;
    m = (0xfffffffffffff as ::core::ffi::c_ulonglong & m as ::core::ffi::c_ulonglong) as uint64_t;
    match e {
        0 => return if m != 0 { FP_SUBNORMAL } else { FP_ZERO },
        2047 => return if m != 0 { FP_NAN } else { FP_INFINITE },
        _ => return FP_NORMAL,
    };
}
#[no_mangle]
pub unsafe extern "C" fn xisinf(mut d: ::core::ffi::c_double) -> ::core::ffi::c_int {
    return (FP_INFINITE == xfpclassify(d)) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xisnan(mut d: ::core::ffi::c_double) -> ::core::ffi::c_int {
    return (FP_NAN == xfpclassify(d)) as ::core::ffi::c_int;
}
#[no_mangle]
pub unsafe extern "C" fn xctz(mut x: uint64_t) -> ::core::ffi::c_int {
    if x == 0 as uint64_t {
        return (8 as usize).wrapping_mul(::core::mem::size_of::<uint64_t>()) as ::core::ffi::c_int;
    }
    return (x as ::core::ffi::c_ulonglong).trailing_zeros() as i32;
}
#[no_mangle]
pub unsafe extern "C" fn xpopcount(mut x: uint64_t) -> ::core::ffi::c_uint {
    return (x as ::core::ffi::c_ulonglong).count_ones() as i32 as ::core::ffi::c_uint;
}
#[no_mangle]
pub unsafe extern "C" fn vim_append_digit_int(
    mut value: *mut ::core::ffi::c_int,
    mut digit: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut x: ::core::ffi::c_int = *value;
    if x > (INT_MAX - digit) / 10 as ::core::ffi::c_int {
        return FAIL;
    }
    *value = x * 10 as ::core::ffi::c_int + digit;
    return OK;
}
#[no_mangle]
pub unsafe extern "C" fn trim_to_int(mut x: int64_t) -> ::core::ffi::c_int {
    return if x > INT_MAX as int64_t {
        INT_MAX
    } else if x < INT_MIN as int64_t {
        INT_MIN
    } else {
        x as ::core::ffi::c_int
    };
}
pub const OK: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FAIL: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const INT_MIN: ::core::ffi::c_int = -INT_MAX - 1 as ::core::ffi::c_int;
pub const INT_MAX: ::core::ffi::c_int = __INT_MAX__;
pub const FP_NAN: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
pub const FP_INFINITE: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
pub const FP_ZERO: ::core::ffi::c_int = 2 as ::core::ffi::c_int;
pub const FP_SUBNORMAL: ::core::ffi::c_int = 3 as ::core::ffi::c_int;
pub const FP_NORMAL: ::core::ffi::c_int = 4 as ::core::ffi::c_int;
pub const __INT_MAX__: ::core::ffi::c_int = 2147483647 as ::core::ffi::c_int;
