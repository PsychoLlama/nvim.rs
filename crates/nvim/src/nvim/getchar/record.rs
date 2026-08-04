//! Recording what was typed: registers, `:redir`, `scriptout`, `on_key`.
//!
//! Every key `vgetorpeek` hands out passes through [`gotchars`], which writes
//! it to the recording register, the `'scriptout'` file, and the `on_key`
//! callbacks — and, along the way, feeds the `showcmd` area.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn gotchars_add_byte(
    mut state: *mut gotchars_state_T,
    mut byte: uint8_t,
) -> bool {
    unsafe {
        let c2rust_fresh4 = (*state).buflen;
        (*state).buflen = (*state).buflen.wrapping_add(1);
        let c2rust_lvalue_ptr = &raw mut (*state).buf[c2rust_fresh4 as usize];
        *c2rust_lvalue_ptr = byte;
        let mut c: ::core::ffi::c_int = *c2rust_lvalue_ptr as ::core::ffi::c_int;
        let mut retval: bool = false_0 != 0;
        let in_special: bool = (*state).pending_special > 0 as ::core::ffi::c_uint;
        let in_mbyte: bool = (*state).pending_mbyte > 0 as ::core::ffi::c_uint;
        if in_special {
            (*state).pending_special = (*state).pending_special.wrapping_sub(1);
        } else if c == K_SPECIAL {
            (*state).pending_special = 2 as ::core::ffi::c_uint;
        }
        '_ret_false: {
            if (*state).pending_special <= 0 as ::core::ffi::c_uint {
                if in_mbyte {
                    (*state).pending_mbyte = (*state).pending_mbyte.wrapping_sub(1);
                } else {
                    if in_special {
                        if (*state).prev_c == KS_MODIFIER {
                            break '_ret_false;
                        } else {
                            c = if (*state).prev_c == KS_SPECIAL {
                                K_SPECIAL
                            } else if (*state).prev_c == KS_ZERO {
                                K_ZERO
                            } else {
                                -((*state).prev_c + (c << 8 as ::core::ffi::c_int))
                            };
                        }
                    }
                    (*state).pending_mbyte =
                        ((if c < 0 as ::core::ffi::c_int || c > 255 as ::core::ffi::c_int {
                            1 as ::core::ffi::c_int
                        } else {
                            (*utf8len_tab.ptr())[c as usize] as ::core::ffi::c_int
                        }) - 1 as ::core::ffi::c_int)
                            as ::core::ffi::c_uint;
                }
                if (*state).pending_mbyte <= 0 as ::core::ffi::c_uint {
                    retval = true_0 != 0;
                }
            }
        }
        (*state).prev_c = c;
        return retval;
    }
}

pub(crate) unsafe extern "C" fn gotchars(mut chars: *const uint8_t, mut len: size_t) {
    unsafe {
        let mut s: *const uint8_t = chars;
        let mut todo: size_t = len;
        static state: GlobalCell<gotchars_state_T> = GlobalCell::new(gotchars_state_T {
            buf: [0; 67],
            prev_c: 0,
            buflen: 0,
            pending_special: 0,
            pending_mbyte: 0,
        });
        loop {
            let c2rust_fresh2 = todo;
            todo = todo.wrapping_sub(1);
            if c2rust_fresh2 <= 0 as size_t {
                break;
            }
            let c2rust_fresh3 = s;
            s = s.offset(1);
            if !gotchars_add_byte(state.ptr(), *c2rust_fresh3) {
                continue;
            }
            let mut i: size_t = 0 as size_t;
            while i < (*state.ptr()).buflen {
                updatescript((*state.ptr()).buf[i as usize] as ::core::ffi::c_int);
                i = i.wrapping_add(1);
            }
            if (*state.ptr()).buflen > on_key_ignore_len.get() {
                if (*state.ptr()).buflen.wrapping_sub(on_key_ignore_len.get()) > 0 as size_t {
                    if (*on_key_buf.ptr()).capacity
                        < (*on_key_buf.ptr())
                            .size
                            .wrapping_add((*state.ptr()).buflen)
                            .wrapping_sub(on_key_ignore_len.get())
                    {
                        (*on_key_buf.ptr()).capacity = (*on_key_buf.ptr())
                            .size
                            .wrapping_add((*state.ptr()).buflen)
                            .wrapping_sub(on_key_ignore_len.get());
                        (*on_key_buf.ptr()).capacity = (*on_key_buf.ptr()).capacity.wrapping_sub(1);
                        (*on_key_buf.ptr()).capacity |=
                            (*on_key_buf.ptr()).capacity >> 1 as ::core::ffi::c_int;
                        (*on_key_buf.ptr()).capacity |=
                            (*on_key_buf.ptr()).capacity >> 2 as ::core::ffi::c_int;
                        (*on_key_buf.ptr()).capacity |=
                            (*on_key_buf.ptr()).capacity >> 4 as ::core::ffi::c_int;
                        (*on_key_buf.ptr()).capacity |=
                            (*on_key_buf.ptr()).capacity >> 8 as ::core::ffi::c_int;
                        (*on_key_buf.ptr()).capacity |=
                            (*on_key_buf.ptr()).capacity >> 16 as ::core::ffi::c_int;
                        (*on_key_buf.ptr()).capacity = (*on_key_buf.ptr()).capacity.wrapping_add(1);
                        (*on_key_buf.ptr()).capacity = if (*on_key_buf.ptr()).capacity
                            > ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            (*on_key_buf.ptr()).capacity
                        } else {
                            ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as size_t,
                                )
                        };
                        (*on_key_buf.ptr()).items = (if (*on_key_buf.ptr()).capacity
                            == ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                                .wrapping_div(
                                    (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                                        .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                                        == 0)
                                        as ::core::ffi::c_int
                                        as usize,
                                ) {
                            if (*on_key_buf.ptr()).items
                                == &raw mut (*on_key_buf.ptr()).init_array
                                    as *mut ::core::ffi::c_char
                            {
                                (*on_key_buf.ptr()).items as *mut ::core::ffi::c_void
                            } else {
                                _memcpy_free(
                                &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
                                    as *mut ::core::ffi::c_void,
                                (*on_key_buf.ptr()).items as *mut ::core::ffi::c_void,
                                (*on_key_buf.ptr())
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                            )
                            }
                        } else {
                            if (*on_key_buf.ptr()).items
                                == &raw mut (*on_key_buf.ptr()).init_array
                                    as *mut ::core::ffi::c_char
                            {
                                memcpy(
                                xmalloc(
                                    (*on_key_buf.ptr())
                                        .capacity
                                        .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                                ),
                                (*on_key_buf.ptr()).items as *const ::core::ffi::c_void,
                                (*on_key_buf.ptr())
                                    .size
                                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                            )
                            } else {
                                xrealloc(
                                (*on_key_buf.ptr()).items as *mut ::core::ffi::c_void,
                                (*on_key_buf.ptr())
                                    .capacity
                                    .wrapping_mul(::core::mem::size_of::<::core::ffi::c_char>()),
                            )
                            }
                        })
                            as *mut ::core::ffi::c_char;
                    }
                    '_c2rust_label: {
                        if !(*on_key_buf.ptr()).items.is_null() {
                        } else {
                            __assert_fail(
                                b"(on_key_buf).items\0".as_ptr() as *const ::core::ffi::c_char,
                                b"src/nvim/getchar.rs\0".as_ptr() as *const ::core::ffi::c_char,
                                1230 as ::core::ffi::c_uint,
                                b"void gotchars(const uint8_t *, size_t)\0".as_ptr()
                                    as *const ::core::ffi::c_char,
                            );
                        }
                    };
                    memcpy(
                        (*on_key_buf.ptr())
                            .items
                            .offset((*on_key_buf.ptr()).size as isize)
                            as *mut ::core::ffi::c_void,
                        (&raw mut (*state.ptr()).buf as *mut uint8_t as *mut ::core::ffi::c_char)
                            .offset(on_key_ignore_len.get() as isize)
                            as *const ::core::ffi::c_void,
                        ::core::mem::size_of::<::core::ffi::c_char>()
                            .wrapping_mul((*state.ptr()).buflen)
                            .wrapping_sub(on_key_ignore_len.get()),
                    );
                    (*on_key_buf.ptr()).size = (*on_key_buf.ptr())
                        .size
                        .wrapping_add((*state.ptr()).buflen)
                        .wrapping_sub(on_key_ignore_len.get());
                }
                on_key_ignore_len.set(0 as size_t);
            } else {
                on_key_ignore_len
                    .set((*on_key_ignore_len.ptr()).wrapping_sub((*state.ptr()).buflen));
            }
            if reg_recording.get() != 0 as ::core::ffi::c_int {
                (*state.ptr()).buf[(*state.ptr()).buflen as usize] = NUL as uint8_t;
                add_buff(
                    recordbuff.ptr(),
                    &raw mut (*state.ptr()).buf as *mut uint8_t as *mut ::core::ffi::c_char,
                    (*state.ptr()).buflen as ptrdiff_t,
                );
                last_recorded_len
                    .set((*last_recorded_len.ptr()).wrapping_add((*state.ptr()).buflen));
            }
            (*state.ptr()).buflen = 0 as size_t;
        }
        may_sync_undo();
        debug_did_msg.set(false_0 != 0);
        (*maptick.ptr()) += 1;
    }
}

pub unsafe extern "C" fn gotchars_ignore() {
    unsafe {
        let mut nop_buf: [uint8_t; 3] = [
            K_SPECIAL as uint8_t,
            KS_EXTRA as uint8_t,
            KE_IGNORE as ::core::ffi::c_int as uint8_t,
        ];
        on_key_ignore_len.set((*on_key_ignore_len.ptr()).wrapping_add(3 as size_t));
        gotchars(&raw mut nop_buf as *mut uint8_t, 3 as size_t);
    }
}

pub(crate) unsafe extern "C" fn add_byte_to_showcmd(mut byte: uint8_t) {
    unsafe {
        static state: GlobalCell<gotchars_state_T> = GlobalCell::new(gotchars_state_T {
            buf: [0; 67],
            prev_c: 0,
            buflen: 0,
            pending_special: 0,
            pending_mbyte: 0,
        });
        if p_sc.get() == 0 || msg_silent.get() != 0 as ::core::ffi::c_int {
            return;
        }
        if !gotchars_add_byte(state.ptr(), byte) {
            return;
        }
        (*state.ptr()).buf[(*state.ptr()).buflen as usize] = NUL as uint8_t;
        (*state.ptr()).buflen = 0 as size_t;
        let mut modifiers: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut c: ::core::ffi::c_int = NUL;
        let mut ptr: *const uint8_t = &raw mut (*state.ptr()).buf as *mut uint8_t;
        if *ptr.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == K_SPECIAL
            && *ptr.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
            && *ptr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != NUL
        {
            modifiers = *ptr.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
            ptr = ptr.offset(3 as ::core::ffi::c_int as isize);
        }
        if *ptr as ::core::ffi::c_int != NUL {
            let mut mb_ptr: *const ::core::ffi::c_char =
                mb_unescape(&raw mut ptr as *mut *const ::core::ffi::c_char);
            c = if !mb_ptr.is_null() {
                utf_ptr2char(mb_ptr)
            } else {
                let c2rust_fresh7 = ptr;
                ptr = ptr.offset(1);
                *c2rust_fresh7 as ::core::ffi::c_int
            };
            if c <= 0x7f as ::core::ffi::c_int {
                let mut modifiers_after: ::core::ffi::c_int = modifiers;
                let mut mod_c: ::core::ffi::c_int = merge_modifiers(c, &raw mut modifiers_after);
                if modifiers_after == 0 as ::core::ffi::c_int {
                    modifiers = 0 as ::core::ffi::c_int;
                    c = mod_c;
                }
            }
        }
        if modifiers != 0 as ::core::ffi::c_int {
            add_to_showcmd(K_SPECIAL);
            add_to_showcmd(KS_MODIFIER);
            add_to_showcmd(modifiers);
        }
        if c != NUL {
            add_to_showcmd(c);
        }
        while *ptr as ::core::ffi::c_int != NUL {
            let c2rust_fresh8 = ptr;
            ptr = ptr.offset(1);
            add_to_showcmd(*c2rust_fresh8 as ::core::ffi::c_int);
        }
    }
}
