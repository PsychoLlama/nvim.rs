//! `nvim_paste`'s typeahead half.
//!
//! [`paste_store`] accumulates the pasted chunks into the redo buffer so that
//! `.` can repeat a paste, and [`paste_repeat`] is what `.` then runs.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn paste_store(
    channel_id: uint64_t,
    state: TriState,
    str: String_0,
    crlf: bool,
) {
    unsafe {
        if State.get() & MODE_CMDLINE != 0 {
            return;
        }
        let need_redo: bool = !block_redo.get();
        let need_record: bool =
            reg_recording.get() != 0 as ::core::ffi::c_int && !is_internal_call(channel_id);
        if !need_redo && !need_record {
            return;
        }
        if state as ::core::ffi::c_int != kNone as ::core::ffi::c_int {
            let c: ::core::ffi::c_int =
                if state as ::core::ffi::c_int == kFalse as ::core::ffi::c_int {
                    K_PASTE_START
                } else {
                    K_PASTE_END
                };
            if need_redo {
                if state as ::core::ffi::c_int == kFalse as ::core::ffi::c_int
                    && State.get() & MODE_INSERT == 0
                {
                    ResetRedobuff();
                }
                add_char_buff(redobuff.ptr(), c);
            }
            if need_record {
                add_char_buff(recordbuff.ptr(), c);
            }
            return;
        }
        let mut s: *const ::core::ffi::c_char = str.data;
        let str_end: *const ::core::ffi::c_char = str.data.offset(str.size as isize);
        while s < str_end {
            let mut start: *const ::core::ffi::c_char = s;
            while s < str_end
                && *s as uint8_t as ::core::ffi::c_int != K_SPECIAL
                && *s as ::core::ffi::c_int != NUL
                && *s as ::core::ffi::c_int != NL
                && !(crlf as ::core::ffi::c_int != 0 && *s as ::core::ffi::c_int == CAR)
            {
                s = s.offset(1);
            }
            if s > start {
                if need_redo {
                    add_buff(redobuff.ptr(), start, s.offset_from(start));
                }
                if need_record {
                    add_buff(recordbuff.ptr(), start, s.offset_from(start));
                }
            }
            if s < str_end {
                let c2rust_fresh17 = s;
                s = s.offset(1);
                let mut c_0: ::core::ffi::c_int = *c2rust_fresh17 as uint8_t as ::core::ffi::c_int;
                if crlf as ::core::ffi::c_int != 0 && c_0 == CAR {
                    if s < str_end && *s as ::core::ffi::c_int == NL {
                        s = s.offset(1);
                    }
                    c_0 = NL;
                }
                if need_redo {
                    add_byte_buff(redobuff.ptr(), c_0);
                }
                if need_record {
                    add_byte_buff(recordbuff.ptr(), c_0);
                }
            }
        }
    }
}

pub unsafe extern "C" fn paste_repeat(mut count: ::core::ffi::c_int) {
    unsafe {
        let mut ga: garray_T = garray_T {
            ga_len: 0 as ::core::ffi::c_int,
            ga_maxlen: 0 as ::core::ffi::c_int,
            ga_itemsize: 1 as ::core::ffi::c_int,
            ga_growsize: 32 as ::core::ffi::c_int,
            ga_data: NULL_0,
        };
        let mut aborted: bool = false_0 != 0;
        (*no_mapping.ptr()) += 1;
        got_int.set(false_0 != 0);
        while !aborted {
            ga_grow(&raw mut ga, 32 as ::core::ffi::c_int);
            let mut c1: uint8_t = vgetorpeek(true_0 != 0) as uint8_t;
            if c1 as ::core::ffi::c_int == K_SPECIAL {
                c1 = vgetorpeek(true_0 != 0) as uint8_t;
                let mut c2: uint8_t = vgetorpeek(true_0 != 0) as uint8_t;
                let mut c: ::core::ffi::c_int = if c1 as ::core::ffi::c_int == KS_SPECIAL {
                    K_SPECIAL
                } else if c1 as ::core::ffi::c_int == KS_ZERO {
                    K_ZERO
                } else {
                    -(c1 as ::core::ffi::c_int
                        + ((c2 as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                };
                if c == K_PASTE_END {
                    break;
                }
                if c == K_ZERO {
                    ga_append(&raw mut ga, NUL as uint8_t);
                } else if c == K_SPECIAL {
                    ga_append(&raw mut ga, K_SPECIAL as uint8_t);
                } else {
                    ga_append(&raw mut ga, K_SPECIAL as uint8_t);
                    ga_append(&raw mut ga, c1);
                    ga_append(&raw mut ga, c2);
                }
            } else {
                ga_append(&raw mut ga, c1);
            }
            aborted = got_int.get();
        }
        (*no_mapping.ptr()) -= 1;
        let mut str: String_0 = String_0 {
            data: ga.ga_data as *mut ::core::ffi::c_char,
            size: ga.ga_len as size_t,
        };
        let mut arena: Arena = ARENA_EMPTY;
        let mut err: Error = Error {
            type_0: kErrorTypeNone,
            msg: ::core::ptr::null_mut::<::core::ffi::c_char>(),
        };
        let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while !aborted && i < count {
            nvim_paste(
                LUA_INTERNAL_CALL,
                str,
                false_0 != 0,
                -1 as Integer,
                &raw mut arena,
                &raw mut err,
            );
            aborted = err.type_0 as ::core::ffi::c_int != kErrorTypeNone as ::core::ffi::c_int;
            i += 1;
        }
        api_clear_error(&raw mut err);
        arena_mem_free(arena_finish(&raw mut arena));
        ga_clear(&raw mut ga);
    }
}
