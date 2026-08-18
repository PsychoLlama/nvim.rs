//! Borders: the `border` key and the `'winborder'` option.
//!
//! `parse_border_style` accepts every spelling a border can take -- a named
//! style, a single character, an array of one, two, four or eight characters,
//! each optionally paired with a highlight group -- and fills the eight
//! `WinConfig` border slots from it.  `parse_winborder` is the option's own
//! parse, which shares the named styles and rejects the rest.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::array_add;

pub unsafe fn parse_border_style(
    mut style: Object,
    mut fconfig: *mut WinConfig,
    mut err: *mut Error,
) {
    unsafe {
        let mut defaults: [C2Rust_Unnamed_15; 7] = [
            C2Rust_Unnamed_15 {
                name: (*opt_winborder_values.ptr())[1 as ::core::ffi::c_int as usize]
                    as *const ::core::ffi::c_char,
                chars: [
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\x94\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\x91\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\x9D\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\x9A\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\x91\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                ],
                shadow_color: false,
            },
            C2Rust_Unnamed_15 {
                name: (*opt_winborder_values.ptr())[2 as ::core::ffi::c_int as usize]
                    as *const ::core::ffi::c_char,
                chars: [
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x8C\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x90\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x98\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x94\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                ],
                shadow_color: false,
            },
            C2Rust_Unnamed_15 {
                name: (*opt_winborder_values.ptr())[3 as ::core::ffi::c_int as usize]
                    as *const ::core::ffi::c_char,
                chars: [
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                ],
                shadow_color: true,
            },
            C2Rust_Unnamed_15 {
                name: (*opt_winborder_values.ptr())[4 as ::core::ffi::c_int as usize]
                    as *const ::core::ffi::c_char,
                chars: [
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\xAD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\xAE\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\xAF\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x80\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x95\xB0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x82\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                ],
                shadow_color: false,
            },
            C2Rust_Unnamed_15 {
                name: (*opt_winborder_values.ptr())[5 as ::core::ffi::c_int as usize]
                    as *const ::core::ffi::c_char,
                chars: [
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b" \0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                ],
                shadow_color: false,
            },
            C2Rust_Unnamed_15 {
                name: (*opt_winborder_values.ptr())[6 as ::core::ffi::c_int as usize]
                    as *const ::core::ffi::c_char,
                chars: [
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x8F\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x81\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x93\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x83\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x9B\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x81\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x97\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                    ::core::mem::transmute::<[u8; 32], [::core::ffi::c_char; 32]>(
                        *b"\xE2\x94\x83\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
                    ),
                ],
                shadow_color: false,
            },
            C2Rust_Unnamed_15 {
                name: ::core::ptr::null::<::core::ffi::c_char>(),
                chars: [
                    [0; 32], [0; 32], [0; 32], [0; 32], [0; 32], [0; 32], [0; 32], [0; 32],
                ],
                shadow_color: false,
            },
        ];
        let mut chars: *mut [::core::ffi::c_char; 32] =
            &raw mut (*fconfig).border_chars as *mut [::core::ffi::c_char; 32];
        let mut hl_ids: *mut ::core::ffi::c_int =
            &raw mut (*fconfig).border_hl_ids as *mut ::core::ffi::c_int;
        (*fconfig).border = true;
        if style.type_0 as ::core::ffi::c_uint
            == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut arr: Array = style.data.array;
            let mut size: size_t = arr.size;
            if size == 0 || size > 8 as size_t || size & size.wrapping_sub(1 as size_t) != 0 {
                api_err_exp(
                    err,
                    c"border".as_ptr(),
                    c"1, 2, 4, or 8 chars".as_ptr(),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
                return;
            }
            let mut i: size_t = 0 as size_t;
            while i < size {
                let mut iytem: Object = *arr.items.add(i);
                let mut string: String_0 = String_0 {
                    data: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    size: 0,
                };
                let mut hl_id: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
                if iytem.type_0 as ::core::ffi::c_uint
                    == kObjectTypeArray as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    let mut iarr: Array = iytem.data.array;
                    if iarr.size == 0 || iarr.size > 2 as size_t {
                        api_err_exp(
                            err,
                            c"border".as_ptr(),
                            c"1 or 2-item Array".as_ptr(),
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        );
                        return;
                    }
                    if !((*iarr.items.offset(0 as ::core::ffi::c_int as isize)).type_0
                        as ::core::ffi::c_uint
                        == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint)
                    {
                        api_err_exp(
                            err,
                            c"border".as_ptr(),
                            c"Array of Strings".as_ptr(),
                            ::core::ptr::null::<::core::ffi::c_char>(),
                        );
                        return;
                    }
                    string = (*iarr.items.offset(0 as ::core::ffi::c_int as isize))
                        .data
                        .string;
                    if iarr.size == 2 as size_t {
                        hl_id = object_to_hl_id(
                            *iarr.items.offset(1 as ::core::ffi::c_int as isize),
                            c"border char highlight".as_ptr(),
                            err,
                        );
                        if (*err).type_0 as ::core::ffi::c_int
                            != kErrorTypeNone as ::core::ffi::c_int
                        {
                            return;
                        }
                    }
                } else if iytem.type_0 as ::core::ffi::c_uint
                    == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
                {
                    string = iytem.data.string;
                } else if true {
                    api_err_exp(
                        err,
                        c"border".as_ptr(),
                        c"String or Array".as_ptr(),
                        api_typename(iytem.type_0),
                    );
                    return;
                }
                if string.size != 0 && mb_string2cells_len(string.data, string.size) > 1 as size_t {
                    api_err_exp(
                        err,
                        c"border".as_ptr(),
                        c"only one-cell chars".as_ptr(),
                        ::core::ptr::null::<::core::ffi::c_char>(),
                    );
                    return;
                }
                let mut len: size_t = if string.size
                    < ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_sub(1_usize)
                {
                    string.size
                } else {
                    ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_sub(1 as size_t)
                };
                if len != 0 {
                    memcpy(
                        &raw mut *chars.add(i) as *mut ::core::ffi::c_char
                            as *mut ::core::ffi::c_void,
                        string.data as *const ::core::ffi::c_void,
                        len,
                    );
                }
                (*chars.add(i))[len as usize] = NUL as ::core::ffi::c_char;
                *hl_ids.add(i) = hl_id;
                i = i.wrapping_add(1);
            }
            while size < 8 as size_t {
                memcpy(
                    chars.add(size) as *mut ::core::ffi::c_void,
                    chars as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<[::core::ffi::c_char; 32]>().wrapping_mul(size),
                );
                memcpy(
                    hl_ids.add(size) as *mut ::core::ffi::c_void,
                    hl_ids as *const ::core::ffi::c_void,
                    ::core::mem::size_of::<::core::ffi::c_int>().wrapping_mul(size),
                );
                size <<= 1 as ::core::ffi::c_int;
            }
            if (*chars.offset(7 as ::core::ffi::c_int as isize))[0 as ::core::ffi::c_int as usize]
                as ::core::ffi::c_int
                != 0
                && (*chars.offset(1 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    != 0
                && (*chars.offset(0 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize]
                    == 0
                || (*chars.offset(1 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    != 0
                    && (*chars.offset(3 as ::core::ffi::c_int as isize))
                        [0 as ::core::ffi::c_int as usize]
                        as ::core::ffi::c_int
                        != 0
                    && (*chars.offset(2 as ::core::ffi::c_int as isize))
                        [0 as ::core::ffi::c_int as usize]
                        == 0
                || (*chars.offset(3 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    != 0
                    && (*chars.offset(5 as ::core::ffi::c_int as isize))
                        [0 as ::core::ffi::c_int as usize]
                        as ::core::ffi::c_int
                        != 0
                    && (*chars.offset(4 as ::core::ffi::c_int as isize))
                        [0 as ::core::ffi::c_int as usize]
                        == 0
                || (*chars.offset(5 as ::core::ffi::c_int as isize))
                    [0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int
                    != 0
                    && (*chars.offset(7 as ::core::ffi::c_int as isize))
                        [0 as ::core::ffi::c_int as usize]
                        as ::core::ffi::c_int
                        != 0
                    && (*chars.offset(6 as ::core::ffi::c_int as isize))
                        [0 as ::core::ffi::c_int as usize]
                        == 0
            {
                api_err_exp(
                    err,
                    c"border".as_ptr(),
                    c"corner char between edge chars".as_ptr(),
                    ::core::ptr::null::<::core::ffi::c_char>(),
                );
                return;
            }
        } else if style.type_0 as ::core::ffi::c_uint
            == kObjectTypeString as ::core::ffi::c_int as ::core::ffi::c_uint
        {
            let mut str: String_0 = style.data.string;
            if str.size == 0 as size_t
                || strequal(str.data, c"none".as_ptr()) as ::core::ffi::c_int != 0
            {
                (*fconfig).border = false;
                (*fconfig).title = false;
                (*fconfig).footer = false;
                return;
            }
            let mut i_0: size_t = 0 as size_t;
            while !defaults[i_0 as usize].name.is_null() {
                if strequal(str.data, defaults[i_0 as usize].name) {
                    memcpy(
                        chars as *mut ::core::ffi::c_void,
                        &raw mut (*(&raw mut defaults as *mut C2Rust_Unnamed_15).add(i_0)).chars
                            as *mut [::core::ffi::c_char; 32]
                            as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<[[::core::ffi::c_char; 32]; 8]>(),
                    );
                    memset(
                        hl_ids as *mut ::core::ffi::c_void,
                        0 as ::core::ffi::c_int,
                        (8 as size_t).wrapping_mul(::core::mem::size_of::<::core::ffi::c_int>()),
                    );
                    if defaults[i_0 as usize].shadow_color {
                        let mut hl_blend: ::core::ffi::c_int = syn_check_group(
                            c"FloatShadow".as_ptr(),
                            ::core::mem::size_of::<[::core::ffi::c_char; 12]>()
                                .wrapping_sub(1 as size_t),
                        );
                        let mut hl_through: ::core::ffi::c_int = syn_check_group(
                            c"FloatShadowThrough".as_ptr(),
                            ::core::mem::size_of::<[::core::ffi::c_char; 19]>()
                                .wrapping_sub(1 as size_t),
                        );
                        *hl_ids.offset(2 as ::core::ffi::c_int as isize) = hl_through;
                        *hl_ids.offset(3 as ::core::ffi::c_int as isize) = hl_blend;
                        *hl_ids.offset(4 as ::core::ffi::c_int as isize) = hl_blend;
                        *hl_ids.offset(5 as ::core::ffi::c_int as isize) = hl_blend;
                        *hl_ids.offset(6 as ::core::ffi::c_int as isize) = hl_through;
                    }
                    return;
                }
                i_0 = i_0.wrapping_add(1);
            }
            if true {
                api_err_invalid(err, c"border".as_ptr(), str.data, 0 as int64_t, true);
                return;
            }
        }
    }
}

pub(crate) unsafe fn generate_api_error(
    mut wp: *mut win_T,
    mut attribute: *const ::core::ffi::c_char,
    mut err: *mut Error,
) {
    unsafe {
        if !wp.is_null() && (*wp).w_floating as ::core::ffi::c_int != 0 {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Required: 'relative' when reconfiguring floating window %d".as_ptr(),
                (*wp).handle,
            );
        } else if true {
            api_err_conflict(err, attribute, c"non-float window".as_ptr());
        }
    }
}

pub unsafe fn parse_winborder(
    mut fconfig: *mut WinConfig,
    mut border_opt: *mut ::core::ffi::c_char,
    mut err: *mut Error,
) -> bool {
    unsafe {
        if fconfig.is_null() {
            return false;
        }
        let mut style: Object = object {
            type_0: kObjectTypeNil,
            data: C2Rust_Unnamed { boolean: false },
        };
        if !strchr(border_opt, ',' as ::core::ffi::c_int).is_null() {
            let mut border_chars: Array = ARRAY_DICT_INIT;
            let mut p: *mut ::core::ffi::c_char = border_opt;
            let mut part: [::core::ffi::c_char; 32] = [0; 32];
            let mut count: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
            while *p as ::core::ffi::c_int != NUL {
                if count >= 8 as ::core::ffi::c_int {
                    api_free_array(border_chars);
                    return false;
                }
                let mut part_len: size_t = copy_option_part(
                    &raw mut p,
                    &raw mut part as *mut ::core::ffi::c_char,
                    ::core::mem::size_of::<[::core::ffi::c_char; 32]>(),
                    c",".as_ptr() as *mut ::core::ffi::c_char,
                );
                if part_len == 0 as size_t
                    || part[0 as ::core::ffi::c_int as usize] as ::core::ffi::c_int == NUL
                {
                    api_free_array(border_chars);
                    return false;
                }
                let mut str: String_0 = cstr_to_string(&raw mut part as *mut ::core::ffi::c_char);
                if border_chars.size == border_chars.capacity {
                    border_chars.capacity = if border_chars.capacity != 0 {
                        border_chars.capacity << 1 as ::core::ffi::c_int
                    } else {
                        8 as size_t
                    };
                    border_chars.items = xrealloc(
                        border_chars.items as *mut ::core::ffi::c_void,
                        ::core::mem::size_of::<Object>().wrapping_mul(border_chars.capacity),
                    ) as *mut Object;
                } else {
                };
                array_add(&mut border_chars, Object::string(str));
                count += 1;
            }
            if count != 8 as ::core::ffi::c_int {
                api_free_array(border_chars);
                return false;
            }
            style = object {
                type_0: kObjectTypeArray,
                data: C2Rust_Unnamed {
                    array: border_chars,
                },
            };
        } else {
            style = object {
                type_0: kObjectTypeString,
                data: C2Rust_Unnamed {
                    string: cstr_to_string(border_opt),
                },
            };
        }
        parse_border_style(style, fconfig, err);
        api_free_object(style);
        return !((*err).type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int);
    }
}
