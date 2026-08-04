//! The `<Cmd>` and `<Lua>` pseudo-keys.
//!
//! A `<Cmd>` mapping puts its command into the typeahead terminated by `<CR>`
//! and [`getcmdkeycmd`] reads it back out for `do_cmdline`; a `<Lua>` key
//! carries a `LuaRef` that [`map_execute_lua`] calls.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn getcmdkeycmd(
    mut _promptc: ::core::ffi::c_int,
    mut _cookie: *mut ::core::ffi::c_void,
    mut _indent: ::core::ffi::c_int,
    mut _do_concat: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut line_ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut c1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut cmod: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut aborted: bool = false_0 != 0;
        ga_init(
            &raw mut line_ga,
            1 as ::core::ffi::c_int,
            32 as ::core::ffi::c_int,
        );
        (*no_mapping.ptr()) += 1;
        got_int.set(false_0 != 0);
        while c1 != NUL && !aborted {
            ga_grow(&raw mut line_ga, 32 as ::core::ffi::c_int);
            if vgetorpeek(false_0 != 0) == NUL {
                emsg(gettext(
                    (e_cmd_mapping_must_end_with_cr.ptr() as *const _)
                        as *const ::core::ffi::c_char,
                ));
                aborted = true_0 != 0;
                break;
            } else {
                c1 = vgetorpeek(true_0 != 0);
                if c1 == K_SPECIAL {
                    c1 = vgetorpeek(true_0 != 0);
                    let mut c2: ::core::ffi::c_int = vgetorpeek(true_0 != 0);
                    if c1 == KS_MODIFIER {
                        cmod = c2;
                        continue;
                    } else {
                        c1 = if c1 == KS_SPECIAL {
                            K_SPECIAL
                        } else if c1 == KS_ZERO {
                            K_ZERO
                        } else {
                            -(c1 + (c2 << 8 as ::core::ffi::c_int))
                        };
                    }
                }
                if got_int.get() {
                    aborted = true_0 != 0;
                } else if c1 == '\r' as ::core::ffi::c_int || c1 == '\n' as ::core::ffi::c_int {
                    c1 = NUL;
                } else if c1 == ESC {
                    aborted = true_0 != 0;
                } else if c1
                    == -(253 as ::core::ffi::c_int
                        + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    emsg(gettext(
                        (e_cmd_mapping_must_end_with_cr_before_second_cmd.ptr() as *const _)
                            as *const ::core::ffi::c_char,
                    ));
                    aborted = true_0 != 0;
                } else if c1
                    == -(253 as ::core::ffi::c_int
                        + ((KE_SNR as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                {
                    ga_concat_len(
                        &raw mut line_ga,
                        b"<SNR>\0".as_ptr() as *const ::core::ffi::c_char,
                        ::core::mem::size_of::<[::core::ffi::c_char; 6]>()
                            .wrapping_sub(1 as size_t),
                    );
                } else {
                    if cmod != 0 as ::core::ffi::c_int {
                        ga_append(&raw mut line_ga, K_SPECIAL as uint8_t);
                        ga_append(&raw mut line_ga, KS_MODIFIER as uint8_t);
                        ga_append(&raw mut line_ga, cmod as uint8_t);
                    }
                    if c1 < 0 as ::core::ffi::c_int {
                        ga_append(&raw mut line_ga, K_SPECIAL as uint8_t);
                        ga_append(
                            &raw mut line_ga,
                            (if c1 == K_SPECIAL {
                                KS_SPECIAL
                            } else if c1 == NUL {
                                KS_ZERO
                            } else {
                                -c1 & 0xff as ::core::ffi::c_int
                            }) as uint8_t,
                        );
                        ga_append(
                            &raw mut line_ga,
                            (if c1 == K_SPECIAL || c1 == NUL {
                                KE_FILLER as ::core::ffi::c_uint
                            } else {
                                -c1 as ::core::ffi::c_uint >> 8 as ::core::ffi::c_int
                                    & 0xff as ::core::ffi::c_uint
                            }) as uint8_t,
                        );
                    } else {
                        ga_append(&raw mut line_ga, c1 as uint8_t);
                    }
                }
                cmod = 0 as ::core::ffi::c_int;
            }
        }
        (*no_mapping.ptr()) -= 1;
        if aborted {
            ga_clear(&raw mut line_ga);
        }
        return line_ga.ga_data as *mut ::core::ffi::c_char;
    }
}

pub unsafe extern "C" fn map_execute_lua(mut may_repeat: bool, mut discard: bool) -> bool {
    unsafe {
        let mut line_ga: garray_T = garray_T {
            ga_len: 0,
            ga_maxlen: 0,
            ga_itemsize: 0,
            ga_growsize: 0,
            ga_data: ::core::ptr::null_mut::<::core::ffi::c_void>(),
        };
        let mut c1: ::core::ffi::c_int = -1 as ::core::ffi::c_int;
        let mut aborted: bool = false_0 != 0;
        ga_init(
            &raw mut line_ga,
            1 as ::core::ffi::c_int,
            32 as ::core::ffi::c_int,
        );
        (*no_mapping.ptr()) += 1;
        got_int.set(false_0 != 0);
        while c1 != NUL && !aborted {
            ga_grow(&raw mut line_ga, 32 as ::core::ffi::c_int);
            c1 = vgetorpeek(true_0 != 0);
            if got_int.get() {
                aborted = true_0 != 0;
            } else if c1 == '\r' as ::core::ffi::c_int || c1 == '\n' as ::core::ffi::c_int {
                c1 = NUL;
            } else {
                ga_append(&raw mut line_ga, c1 as uint8_t);
            }
        }
        (*no_mapping.ptr()) -= 1;
        if aborted as ::core::ffi::c_int != 0 || discard as ::core::ffi::c_int != 0 {
            ga_clear(&raw mut line_ga);
            return !aborted;
        }
        let mut ref_0: LuaRef = atoi(line_ga.ga_data as *const ::core::ffi::c_char);
        if may_repeat {
            repeat_luaref.set(ref_0);
        }
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut args: Array = ARRAY_DICT_INIT;
        nlua_call_ref(
            ref_0,
            ::core::ptr::null::<::core::ffi::c_char>(),
            args,
            kRetNilBool,
            ::core::ptr::null_mut::<Arena>(),
            &raw mut err,
        );
        if err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int {
            semsg_multiline(
                b"emsg\0".as_ptr() as *const ::core::ffi::c_char,
                b"E5108: %s\0".as_ptr() as *const ::core::ffi::c_char,
                err.msg,
            );
            api_clear_error(&raw mut err);
        }
        ga_clear(&raw mut line_ga);
        return true_0 != 0;
    }
}
