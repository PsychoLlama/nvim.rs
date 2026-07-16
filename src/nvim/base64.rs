extern "C" {
    fn __assert_fail(
        __assertion: *const ::core::ffi::c_char,
        __file: *const ::core::ffi::c_char,
        __line: ::core::ffi::c_uint,
        __function: *const ::core::ffi::c_char,
    ) -> !;
    fn memcpy(
        __dest: *mut ::core::ffi::c_void,
        __src: *const ::core::ffi::c_void,
        __n: size_t,
    ) -> *mut ::core::ffi::c_void;
    fn xmalloc(size: size_t) -> *mut ::core::ffi::c_void;
    fn xfree(ptr: *mut ::core::ffi::c_void);
}
pub type size_t = usize;
pub type __uint32_t = u32;
pub type __uint64_t = u64;
pub type uint8_t = u8;
pub type uint32_t = u32;
pub type uint64_t = u64;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
#[inline]
unsafe extern "C" fn __bswap_32(mut __bsx: __uint32_t) -> __uint32_t {
    return (__bsx & 0xff000000 as __uint32_t) >> 24 as ::core::ffi::c_int
        | (__bsx & 0xff0000 as __uint32_t) >> 8 as ::core::ffi::c_int
        | (__bsx & 0xff00 as __uint32_t) << 8 as ::core::ffi::c_int
        | (__bsx & 0xff as __uint32_t) << 24 as ::core::ffi::c_int;
}
#[inline]
unsafe extern "C" fn __bswap_64(mut __bsx: __uint64_t) -> __uint64_t {
    return ((__bsx as ::core::ffi::c_ulonglong & 0xff00000000000000 as ::core::ffi::c_ulonglong)
        >> 56 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff000000000000 as ::core::ffi::c_ulonglong)
            >> 40 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff0000000000 as ::core::ffi::c_ulonglong)
            >> 24 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff00000000 as ::core::ffi::c_ulonglong)
            >> 8 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff000000 as ::core::ffi::c_ulonglong)
            << 8 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff0000 as ::core::ffi::c_ulonglong)
            << 24 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff00 as ::core::ffi::c_ulonglong)
            << 40 as ::core::ffi::c_int
        | (__bsx as ::core::ffi::c_ulonglong & 0xff as ::core::ffi::c_ulonglong)
            << 56 as ::core::ffi::c_int) as __uint64_t;
}
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
static mut alphabet: [::core::ffi::c_char; 65] = unsafe {
    ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
        *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/\0",
    )
};
static mut char_to_index: [uint8_t; 256] = [
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    63 as uint8_t,
    0,
    0,
    0,
    64 as uint8_t,
    53 as uint8_t,
    54 as uint8_t,
    55 as uint8_t,
    56 as uint8_t,
    57 as uint8_t,
    58 as uint8_t,
    59 as uint8_t,
    60 as uint8_t,
    61 as uint8_t,
    62 as uint8_t,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    1 as uint8_t,
    2 as uint8_t,
    3 as uint8_t,
    4 as uint8_t,
    5 as uint8_t,
    6 as uint8_t,
    7 as uint8_t,
    8 as uint8_t,
    9 as uint8_t,
    10 as uint8_t,
    11 as uint8_t,
    12 as uint8_t,
    13 as uint8_t,
    14 as uint8_t,
    15 as uint8_t,
    16 as uint8_t,
    17 as uint8_t,
    18 as uint8_t,
    19 as uint8_t,
    20 as uint8_t,
    21 as uint8_t,
    22 as uint8_t,
    23 as uint8_t,
    24 as uint8_t,
    25 as uint8_t,
    26 as uint8_t,
    0,
    0,
    0,
    0,
    0,
    0,
    27 as uint8_t,
    28 as uint8_t,
    29 as uint8_t,
    30 as uint8_t,
    31 as uint8_t,
    32 as uint8_t,
    33 as uint8_t,
    34 as uint8_t,
    35 as uint8_t,
    36 as uint8_t,
    37 as uint8_t,
    38 as uint8_t,
    39 as uint8_t,
    40 as uint8_t,
    41 as uint8_t,
    42 as uint8_t,
    43 as uint8_t,
    44 as uint8_t,
    45 as uint8_t,
    46 as uint8_t,
    47 as uint8_t,
    48 as uint8_t,
    49 as uint8_t,
    50 as uint8_t,
    51 as uint8_t,
    52 as uint8_t,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
    0,
];
#[no_mangle]
pub unsafe extern "C" fn base64_encode(
    mut src: *const ::core::ffi::c_char,
    mut src_len: size_t,
) -> *mut ::core::ffi::c_char {
    '_c2rust_label: {
        if !src.is_null() {
        } else {
            __assert_fail(
                b"src != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/base64.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                71 as ::core::ffi::c_uint,
                b"char *base64_encode(const char *, size_t)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let out_len: size_t = src_len
        .wrapping_add(2 as size_t)
        .wrapping_div(3 as size_t)
        .wrapping_mul(4 as size_t);
    let mut dest: *mut ::core::ffi::c_char =
        xmalloc(out_len.wrapping_add(1 as size_t)) as *mut ::core::ffi::c_char;
    let mut src_i: size_t = 0 as size_t;
    let mut out_i: size_t = 0 as size_t;
    let mut s: *const uint8_t = src as *const uint8_t;
    while src_i.wrapping_add(7 as size_t) < src_len {
        let mut bits_h: uint64_t = 0;
        memcpy(
            &raw mut bits_h as *mut ::core::ffi::c_void,
            s.offset(src_i as isize) as *const ::core::ffi::c_void,
            ::core::mem::size_of::<uint64_t>(),
        );
        let bits_be: uint64_t = __bswap_64(bits_h as __uint64_t);
        *dest.offset(out_i.wrapping_add(0 as size_t) as isize) =
            alphabet[(bits_be >> 58 as ::core::ffi::c_int & 0x3f as uint64_t) as usize];
        *dest.offset(out_i.wrapping_add(1 as size_t) as isize) =
            alphabet[(bits_be >> 52 as ::core::ffi::c_int & 0x3f as uint64_t) as usize];
        *dest.offset(out_i.wrapping_add(2 as size_t) as isize) =
            alphabet[(bits_be >> 46 as ::core::ffi::c_int & 0x3f as uint64_t) as usize];
        *dest.offset(out_i.wrapping_add(3 as size_t) as isize) =
            alphabet[(bits_be >> 40 as ::core::ffi::c_int & 0x3f as uint64_t) as usize];
        *dest.offset(out_i.wrapping_add(4 as size_t) as isize) =
            alphabet[(bits_be >> 34 as ::core::ffi::c_int & 0x3f as uint64_t) as usize];
        *dest.offset(out_i.wrapping_add(5 as size_t) as isize) =
            alphabet[(bits_be >> 28 as ::core::ffi::c_int & 0x3f as uint64_t) as usize];
        *dest.offset(out_i.wrapping_add(6 as size_t) as isize) =
            alphabet[(bits_be >> 22 as ::core::ffi::c_int & 0x3f as uint64_t) as usize];
        *dest.offset(out_i.wrapping_add(7 as size_t) as isize) =
            alphabet[(bits_be >> 16 as ::core::ffi::c_int & 0x3f as uint64_t) as usize];
        out_i = (out_i as ::core::ffi::c_ulong)
            .wrapping_add(::core::mem::size_of::<uint64_t>() as ::core::ffi::c_ulong)
            as size_t;
        src_i = src_i.wrapping_add(6 as size_t);
    }
    while src_i.wrapping_add(3 as size_t) < src_len {
        let mut bits_h_0: uint32_t = 0;
        memcpy(
            &raw mut bits_h_0 as *mut ::core::ffi::c_void,
            s.offset(src_i as isize) as *const ::core::ffi::c_void,
            ::core::mem::size_of::<uint32_t>(),
        );
        let bits_be_0: uint32_t = __bswap_32(bits_h_0 as __uint32_t);
        *dest.offset(out_i.wrapping_add(0 as size_t) as isize) =
            alphabet[(bits_be_0 >> 26 as ::core::ffi::c_int & 0x3f as uint32_t) as usize];
        *dest.offset(out_i.wrapping_add(1 as size_t) as isize) =
            alphabet[(bits_be_0 >> 20 as ::core::ffi::c_int & 0x3f as uint32_t) as usize];
        *dest.offset(out_i.wrapping_add(2 as size_t) as isize) =
            alphabet[(bits_be_0 >> 14 as ::core::ffi::c_int & 0x3f as uint32_t) as usize];
        *dest.offset(out_i.wrapping_add(3 as size_t) as isize) =
            alphabet[(bits_be_0 >> 8 as ::core::ffi::c_int & 0x3f as uint32_t) as usize];
        out_i = (out_i as ::core::ffi::c_ulong)
            .wrapping_add(::core::mem::size_of::<uint32_t>() as ::core::ffi::c_ulong)
            as size_t;
        src_i = src_i.wrapping_add(3 as size_t);
    }
    if src_i.wrapping_add(2 as size_t) < src_len {
        *dest.offset(out_i.wrapping_add(0 as size_t) as isize) = alphabet
            [(*s.offset(src_i as isize) as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as usize];
        *dest.offset(out_i.wrapping_add(1 as size_t) as isize) =
            alphabet[((*s.offset(src_i as isize) as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int)
                << 4 as ::core::ffi::c_int
                | *s.offset(src_i.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                    >> 4 as ::core::ffi::c_int) as usize];
        *dest.offset(out_i.wrapping_add(2 as size_t) as isize) =
            alphabet[((*s.offset(src_i.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int
                | *s.offset(src_i.wrapping_add(2 as size_t) as isize) as ::core::ffi::c_int
                    >> 6 as ::core::ffi::c_int) as usize];
        *dest.offset(out_i.wrapping_add(3 as size_t) as isize) =
            alphabet[(*s.offset(src_i.wrapping_add(2 as size_t) as isize) as ::core::ffi::c_int
                & 0x3f as ::core::ffi::c_int) as usize];
        out_i = out_i.wrapping_add(4 as size_t);
    } else if src_i.wrapping_add(1 as size_t) < src_len {
        *dest.offset(out_i.wrapping_add(0 as size_t) as isize) = alphabet
            [(*s.offset(src_i as isize) as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as usize];
        *dest.offset(out_i.wrapping_add(1 as size_t) as isize) =
            alphabet[((*s.offset(src_i as isize) as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int)
                << 4 as ::core::ffi::c_int
                | *s.offset(src_i.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                    >> 4 as ::core::ffi::c_int) as usize];
        *dest.offset(out_i.wrapping_add(2 as size_t) as isize) =
            alphabet[((*s.offset(src_i.wrapping_add(1 as size_t) as isize) as ::core::ffi::c_int
                & 0xf as ::core::ffi::c_int)
                << 2 as ::core::ffi::c_int) as usize];
        out_i = out_i.wrapping_add(3 as size_t);
    } else if src_i < src_len {
        *dest.offset(out_i.wrapping_add(0 as size_t) as isize) = alphabet
            [(*s.offset(src_i as isize) as ::core::ffi::c_int >> 2 as ::core::ffi::c_int) as usize];
        *dest.offset(out_i.wrapping_add(1 as size_t) as isize) =
            alphabet[((*s.offset(src_i as isize) as ::core::ffi::c_int & 0x3 as ::core::ffi::c_int)
                << 4 as ::core::ffi::c_int) as usize];
        out_i = out_i.wrapping_add(2 as size_t);
    }
    while out_i < out_len {
        *dest.offset(out_i as isize) = '=' as ::core::ffi::c_char;
        out_i = out_i.wrapping_add(1);
    }
    *dest.offset(out_len as isize) = NUL as ::core::ffi::c_char;
    return dest;
}
#[no_mangle]
pub unsafe extern "C" fn base64_decode(
    mut src: *const ::core::ffi::c_char,
    mut src_len: size_t,
    mut out_lenp: *mut size_t,
) -> *mut ::core::ffi::c_char {
    let mut out_len: size_t = 0;
    let mut s: *const uint8_t = ::core::ptr::null::<uint8_t>();
    let mut acc: ::core::ffi::c_int = 0;
    let mut acc_len: ::core::ffi::c_int = 0;
    let mut out_i: size_t = 0;
    let mut src_i: size_t = 0;
    let mut leftover_i: ::core::ffi::c_int = 0;
    '_c2rust_label: {
        if !src.is_null() {
        } else {
            __assert_fail(
                b"src != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/base64.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                147 as ::core::ffi::c_uint,
                b"char *base64_decode(const char *, size_t, size_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    '_c2rust_label_0: {
        if !out_lenp.is_null() {
        } else {
            __assert_fail(
                b"out_lenp != NULL\0".as_ptr() as *const ::core::ffi::c_char,
                b"/home/overlord/projects/neovim/neovim/src/nvim/base64.c\0".as_ptr()
                    as *const ::core::ffi::c_char,
                148 as ::core::ffi::c_uint,
                b"char *base64_decode(const char *, size_t, size_t *)\0".as_ptr()
                    as *const ::core::ffi::c_char,
            );
        }
    };
    let mut dest: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    '_invalid: {
        if src_len.wrapping_rem(4 as size_t) == 0 as size_t {
            out_len = src_len.wrapping_div(4 as size_t).wrapping_mul(3 as size_t);
            if src_len >= 1 as size_t
                && *src.offset(src_len.wrapping_sub(1 as size_t) as isize) as ::core::ffi::c_int
                    == '=' as ::core::ffi::c_int
            {
                out_len = out_len.wrapping_sub(1);
            }
            if src_len >= 2 as size_t
                && *src.offset(src_len.wrapping_sub(2 as size_t) as isize) as ::core::ffi::c_int
                    == '=' as ::core::ffi::c_int
            {
                out_len = out_len.wrapping_sub(1);
            }
            s = src as *const uint8_t;
            dest = xmalloc(out_len) as *mut ::core::ffi::c_char;
            acc = 0 as ::core::ffi::c_int;
            acc_len = 0 as ::core::ffi::c_int;
            out_i = 0 as size_t;
            src_i = 0 as size_t;
            leftover_i = -1 as ::core::ffi::c_int;
            while src_i < src_len {
                let c: uint8_t = *s.offset(src_i as isize);
                let d: uint8_t = char_to_index[c as usize];
                if d as ::core::ffi::c_int == 0 as ::core::ffi::c_int {
                    if c as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                        break '_invalid;
                    }
                    leftover_i = src_i as ::core::ffi::c_int;
                    break;
                } else {
                    acc = (acc << 6 as ::core::ffi::c_int & 0xfff as ::core::ffi::c_int)
                        + (d as ::core::ffi::c_int - 1 as ::core::ffi::c_int);
                    acc_len += 6 as ::core::ffi::c_int;
                    if acc_len >= 8 as ::core::ffi::c_int {
                        acc_len -= 8 as ::core::ffi::c_int;
                        *dest.offset(out_i as isize) = (acc >> acc_len) as ::core::ffi::c_char;
                        out_i = out_i.wrapping_add(1 as size_t);
                    }
                    src_i = src_i.wrapping_add(1);
                }
            }
            if !(acc_len > 4 as ::core::ffi::c_int
                || acc & ((1 as ::core::ffi::c_int) << acc_len) - 1 as ::core::ffi::c_int
                    != 0 as ::core::ffi::c_int)
            {
                if leftover_i > -1 as ::core::ffi::c_int {
                    let mut padding_len: ::core::ffi::c_int = acc_len / 2 as ::core::ffi::c_int;
                    let mut padding_chars: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                    while (leftover_i as size_t) < src_len {
                        let c_0: uint8_t = *s.offset(leftover_i as isize);
                        if c_0 as ::core::ffi::c_int != '=' as ::core::ffi::c_int {
                            break '_invalid;
                        }
                        padding_chars += 1 as ::core::ffi::c_int;
                        leftover_i += 1;
                    }
                    if padding_chars != padding_len {
                        break '_invalid;
                    }
                }
                *out_lenp = out_len;
                return dest;
            }
        }
    }
    if !dest.is_null() {
        xfree(dest as *mut ::core::ffi::c_void);
    }
    *out_lenp = 0 as size_t;
    return ::core::ptr::null_mut::<::core::ffi::c_char>();
}
