use crate::src::nvim::global_cell::SharedCell;
use crate::src::nvim::os::libc::{
    __assert_fail, abort, fprintf, free, malloc, memcpy, snprintf, stderr, strchr, strtod,
};
pub use crate::src::nvim::types::{
    _IO_codecvt, _IO_lock_t, _IO_marker, _IO_wide_data, __off64_t, __off_t, size_t, FILE, _IO_FILE,
};
pub const __ASSERT_FUNCTION: [::core::ffi::c_char; 36] = unsafe {
    ::core::mem::transmute::<[u8; 36], [::core::ffi::c_char; 36]>(
        *b"void set_number_format(char *, int)\0",
    )
};
static locale_decimal_point: SharedCell<::core::ffi::c_char> =
    SharedCell::new('.' as ::core::ffi::c_char);
unsafe extern "C" fn fpconv_update_locale() {
    let mut buf: [::core::ffi::c_char; 8] = [0; 8];
    snprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        ::core::mem::size_of::<[::core::ffi::c_char; 8]>(),
        b"%g\0".as_ptr() as *const ::core::ffi::c_char,
        0.5f64,
    );
    if buf[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != '0' as ::core::ffi::c_int
        || buf[2 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != '5' as ::core::ffi::c_int
        || buf[3 as ::core::ffi::c_int as usize] as ::core::ffi::c_int != 0 as ::core::ffi::c_int
    {
        fprintf(
            stderr,
            b"Error: wide characters found or printf() bug.\0".as_ptr()
                as *const ::core::ffi::c_char,
        );
        abort();
    }
    locale_decimal_point.set(buf[1 as ::core::ffi::c_int as usize]);
}
#[inline]
unsafe extern "C" fn valid_number_character(mut ch: ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut lower_ch: ::core::ffi::c_char = 0;
    if '0' as ::core::ffi::c_int <= ch as ::core::ffi::c_int
        && ch as ::core::ffi::c_int <= '9' as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    if ch as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || ch as ::core::ffi::c_int == '+' as ::core::ffi::c_int
        || ch as ::core::ffi::c_int == '.' as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    lower_ch = (ch as ::core::ffi::c_int | 0x20 as ::core::ffi::c_int) as ::core::ffi::c_char;
    if 'a' as ::core::ffi::c_int <= lower_ch as ::core::ffi::c_int
        && lower_ch as ::core::ffi::c_int <= 'y' as ::core::ffi::c_int
    {
        return 1 as ::core::ffi::c_int;
    }
    return 0 as ::core::ffi::c_int;
}
unsafe extern "C" fn strtod_buffer_size(mut s: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let mut p: *const ::core::ffi::c_char = s;
    while valid_number_character(*p) != 0 {
        p = p.offset(1);
    }
    return p.offset_from(s) as ::core::ffi::c_int;
}
pub unsafe extern "C" fn fpconv_strtod(
    mut nptr: *const ::core::ffi::c_char,
    mut endptr: *mut *mut ::core::ffi::c_char,
) -> ::core::ffi::c_double {
    let mut localbuf: [::core::ffi::c_char; 32] = [0; 32];
    let mut buf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut endbuf: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut dp: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let mut buflen: ::core::ffi::c_int = 0;
    let mut value: ::core::ffi::c_double = 0.;
    if locale_decimal_point.get() as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
        return strtod(nptr, endptr);
    }
    buflen = strtod_buffer_size(nptr);
    if buflen == 0 {
        *endptr = nptr as *mut ::core::ffi::c_char;
        return 0 as ::core::ffi::c_int as ::core::ffi::c_double;
    }
    if buflen >= FPCONV_G_FMT_BUFSIZE {
        buf = malloc((buflen + 1 as ::core::ffi::c_int) as size_t) as *mut ::core::ffi::c_char;
        if buf.is_null() {
            fprintf(
                stderr,
                b"Out of memory\0".as_ptr() as *const ::core::ffi::c_char,
            );
            abort();
        }
    } else {
        buf = &raw mut localbuf as *mut ::core::ffi::c_char;
    }
    memcpy(
        buf as *mut ::core::ffi::c_void,
        nptr as *const ::core::ffi::c_void,
        buflen as size_t,
    );
    *buf.offset(buflen as isize) = 0 as ::core::ffi::c_char;
    dp = strchr(buf, '.' as ::core::ffi::c_int);
    if !dp.is_null() {
        *dp = locale_decimal_point.get();
    }
    value = strtod(buf, &raw mut endbuf);
    *endptr = nptr.offset(endbuf.offset_from(buf) as isize) as *mut ::core::ffi::c_char;
    if buflen >= FPCONV_G_FMT_BUFSIZE {
        free(buf as *mut ::core::ffi::c_void);
    }
    return value;
}
unsafe extern "C" fn set_number_format(
    mut fmt: *mut ::core::ffi::c_char,
    mut precision: ::core::ffi::c_int,
) {
    let mut d1: ::core::ffi::c_int = 0;
    let mut d2: ::core::ffi::c_int = 0;
    let mut i: ::core::ffi::c_int = 0;
    '_c2rust_label: {
        if 1 as ::core::ffi::c_int <= precision && precision <= 16 as ::core::ffi::c_int {
        } else {
            __assert_fail(
                b"1 <= precision && precision <= 16\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/cjson/fpconv.rs\0".as_ptr() as *const ::core::ffi::c_char,
                163 as ::core::ffi::c_uint,
                __ASSERT_FUNCTION.as_ptr(),
            );
        }
    };
    d1 = precision / 10 as ::core::ffi::c_int;
    d2 = precision % 10 as ::core::ffi::c_int;
    *fmt.offset(0 as ::core::ffi::c_int as isize) = '%' as ::core::ffi::c_char;
    *fmt.offset(1 as ::core::ffi::c_int as isize) = '.' as ::core::ffi::c_char;
    i = 2 as ::core::ffi::c_int;
    if d1 != 0 {
        let c2rust_fresh2 = i;
        i = i + 1;
        *fmt.offset(c2rust_fresh2 as isize) =
            ('0' as ::core::ffi::c_int + d1) as ::core::ffi::c_char;
    }
    let c2rust_fresh3 = i;
    i = i + 1;
    *fmt.offset(c2rust_fresh3 as isize) = ('0' as ::core::ffi::c_int + d2) as ::core::ffi::c_char;
    let c2rust_fresh4 = i;
    i = i + 1;
    *fmt.offset(c2rust_fresh4 as isize) = 'g' as ::core::ffi::c_char;
    *fmt.offset(i as isize) = 0 as ::core::ffi::c_char;
}
pub unsafe extern "C" fn fpconv_g_fmt(
    mut str: *mut ::core::ffi::c_char,
    mut num: ::core::ffi::c_double,
    mut precision: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let mut buf: [::core::ffi::c_char; 32] = [0; 32];
    let mut fmt: [::core::ffi::c_char; 6] = [0; 6];
    let mut len: ::core::ffi::c_int = 0;
    let mut b: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    set_number_format(&raw mut fmt as *mut ::core::ffi::c_char, precision);
    if locale_decimal_point.get() as ::core::ffi::c_int == '.' as ::core::ffi::c_int {
        return snprintf(
            str,
            FPCONV_G_FMT_BUFSIZE as size_t,
            &raw mut fmt as *mut ::core::ffi::c_char,
            num,
        );
    }
    len = snprintf(
        &raw mut buf as *mut ::core::ffi::c_char,
        FPCONV_G_FMT_BUFSIZE as size_t,
        &raw mut fmt as *mut ::core::ffi::c_char,
        num,
    );
    b = &raw mut buf as *mut ::core::ffi::c_char;
    loop {
        let c2rust_fresh0 = str;
        str = str.offset(1);
        *c2rust_fresh0 =
            (if *b as ::core::ffi::c_int == locale_decimal_point.get() as ::core::ffi::c_int {
                '.' as ::core::ffi::c_int
            } else {
                *b as ::core::ffi::c_int
            }) as ::core::ffi::c_char;
        let c2rust_fresh1 = b;
        b = b.offset(1);
        if *c2rust_fresh1 == 0 {
            break;
        }
    }
    return len;
}
pub unsafe extern "C" fn fpconv_init() {
    fpconv_update_locale();
}
pub const FPCONV_G_FMT_BUFSIZE: ::core::ffi::c_int = 32 as ::core::ffi::c_int;
