//! `vgetc` and the peek variants: one whole key out of the typeahead.
//!
//! [`vgetc`] is what the rest of the editor calls.  It asks
//! [`crate::src::nvim::getchar::vgetorpeek`] for bytes and reassembles them
//! into a single key: a `K_SPECIAL` escape back into its key code, a modifier
//! prefix into `mod_mask`, a UTF-8 sequence into a codepoint.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn merge_modifiers(
    mut c_arg: ::core::ffi::c_int,
    mut modifiers: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = c_arg;
        if *modifiers & MOD_MASK_CTRL != 0 {
            if c >= '@' as ::core::ffi::c_int && c <= 0x7f as ::core::ffi::c_int {
                c &= 0x1f as ::core::ffi::c_int;
                if c == NUL {
                    c = K_ZERO;
                }
            } else if c == '6' as ::core::ffi::c_int {
                c = 0x1e as ::core::ffi::c_int;
            }
            if c != c_arg {
                *modifiers &= !MOD_MASK_CTRL;
            }
        }
        return c;
    }
}

pub unsafe extern "C" fn vgetc() -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = 0;
        let mut buf: [uint8_t; 22] = [0; 22];
        if may_garbage_collect.get() as ::core::ffi::c_int != 0
            && want_garbage_collect.get() as ::core::ffi::c_int != 0
        {
            garbage_collect(false_0 != 0);
        }
        if can_get_old_char() {
            c = old_char.get();
            old_char.set(-1 as ::core::ffi::c_int);
            mod_mask.set(old_mod_mask.get());
            mouse_grid.set(old_mouse_grid.get());
            mouse_row.set(old_mouse_row.get());
            mouse_col.set(old_mouse_col.get());
        } else {
            static last_vgetc_recorded_len: GlobalCell<size_t> = GlobalCell::new(0 as size_t);
            mod_mask.set(0 as ::core::ffi::c_int);
            vgetc_mod_mask.set(0 as ::core::ffi::c_int);
            vgetc_char.set(0 as ::core::ffi::c_int);
            last_recorded_len
                .set((*last_recorded_len.ptr()).wrapping_sub(last_vgetc_recorded_len.get()));
            loop {
                let mut did_inc: bool = false_0 != 0;
                if mod_mask.get() != 0 {
                    (*no_mapping.ptr()) += 1;
                    (*allow_keys.ptr()) += 1;
                    did_inc = true_0 != 0;
                }
                c = vgetorpeek(true_0 != 0);
                if did_inc {
                    (*no_mapping.ptr()) -= 1;
                    (*allow_keys.ptr()) -= 1;
                }
                if c == K_SPECIAL {
                    let mut save_allow_keys: ::core::ffi::c_int = allow_keys.get();
                    (*no_mapping.ptr()) += 1;
                    allow_keys.set(0 as ::core::ffi::c_int);
                    let mut c2: ::core::ffi::c_int = vgetorpeek(true_0 != 0);
                    c = vgetorpeek(true_0 != 0);
                    (*no_mapping.ptr()) -= 1;
                    allow_keys.set(save_allow_keys);
                    if c2 == KS_MODIFIER {
                        mod_mask.set(c);
                        continue;
                    } else {
                        c = if c2 == KS_SPECIAL {
                            K_SPECIAL
                        } else if c2 == KS_ZERO {
                            K_ZERO
                        } else {
                            -(c2 + (c << 8 as ::core::ffi::c_int))
                        };
                    }
                }
                let mut n: ::core::ffi::c_int = 0;
                n = if c < 0 as ::core::ffi::c_int || c > 255 as ::core::ffi::c_int {
                    1 as ::core::ffi::c_int
                } else {
                    (*utf8len_tab.ptr())[c as usize] as ::core::ffi::c_int
                };
                if n > 1 as ::core::ffi::c_int {
                    (*no_mapping.ptr()) += 1;
                    buf[0 as ::core::ffi::c_int as usize] = c as uint8_t;
                    let mut i: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
                    while i < n {
                        buf[i as usize] = vgetorpeek(true_0 != 0) as uint8_t;
                        if buf[i as usize] as ::core::ffi::c_int == K_SPECIAL {
                            vgetorpeek(true_0 != 0);
                            vgetorpeek(true_0 != 0);
                        }
                        i += 1;
                    }
                    (*no_mapping.ptr()) -= 1;
                    c = utf_ptr2char(&raw mut buf as *mut uint8_t as *mut ::core::ffi::c_char);
                }
                if no_mapping.get() == 0
                    && KeyTyped.get() as ::core::ffi::c_int != 0
                    && mod_mask.get() == MOD_MASK_ALT
                    && State.get() & MODE_TERMINAL == 0
                    && !is_mouse_key(c)
                {
                    mod_mask.set(0 as ::core::ffi::c_int);
                    let mut len: ::core::ffi::c_int =
                        ins_char_typebuf(c, 0 as ::core::ffi::c_int, false_0 != 0);
                    ins_char_typebuf(ESC, 0 as ::core::ffi::c_int, false_0 != 0);
                    let mut old_len: ::core::ffi::c_int = len + 3 as ::core::ffi::c_int;
                    ungetchars(old_len);
                    if (*on_key_buf.ptr()).size >= old_len as size_t {
                        (*on_key_buf.ptr()).size =
                            (*on_key_buf.ptr()).size.wrapping_sub(old_len as size_t);
                    }
                } else {
                    if vgetc_char.get() == 0 as ::core::ffi::c_int {
                        vgetc_mod_mask.set(mod_mask.get());
                        vgetc_char.set(c);
                    }
                    match c {
                        K_KPLUS => {
                            c = '+' as ::core::ffi::c_int;
                        }
                        K_KMINUS => {
                            c = '-' as ::core::ffi::c_int;
                        }
                        K_KDIVIDE => {
                            c = '/' as ::core::ffi::c_int;
                        }
                        K_KMULTIPLY => {
                            c = '*' as ::core::ffi::c_int;
                        }
                        K_KENTER => {
                            c = CAR;
                        }
                        K_KPOINT => {
                            c = '.' as ::core::ffi::c_int;
                        }
                        K_KCOMMA => {
                            c = ',' as ::core::ffi::c_int;
                        }
                        K_KEQUAL => {
                            c = '=' as ::core::ffi::c_int;
                        }
                        K_K0 => {
                            c = '0' as ::core::ffi::c_int;
                        }
                        K_K1 => {
                            c = '1' as ::core::ffi::c_int;
                        }
                        K_K2 => {
                            c = '2' as ::core::ffi::c_int;
                        }
                        K_K3 => {
                            c = '3' as ::core::ffi::c_int;
                        }
                        K_K4 => {
                            c = '4' as ::core::ffi::c_int;
                        }
                        K_K5 => {
                            c = '5' as ::core::ffi::c_int;
                        }
                        K_K6 => {
                            c = '6' as ::core::ffi::c_int;
                        }
                        K_K7 => {
                            c = '7' as ::core::ffi::c_int;
                        }
                        K_K8 => {
                            c = '8' as ::core::ffi::c_int;
                        }
                        K_K9 => {
                            c = '9' as ::core::ffi::c_int;
                        }
                        K_XHOME | K_ZHOME => {
                            if mod_mask.get() == MOD_MASK_SHIFT {
                                c = K_S_HOME;
                                mod_mask.set(0 as ::core::ffi::c_int);
                            } else if mod_mask.get() == MOD_MASK_CTRL {
                                c = -(253 as ::core::ffi::c_int
                                    + ((KE_C_HOME as ::core::ffi::c_int)
                                        << 8 as ::core::ffi::c_int));
                                mod_mask.set(0 as ::core::ffi::c_int);
                            } else {
                                c = K_HOME;
                            }
                        }
                        K_XEND | K_ZEND => {
                            if mod_mask.get() == MOD_MASK_SHIFT {
                                c = K_S_END;
                                mod_mask.set(0 as ::core::ffi::c_int);
                            } else if mod_mask.get() == MOD_MASK_CTRL {
                                c = -(253 as ::core::ffi::c_int
                                    + ((KE_C_END as ::core::ffi::c_int)
                                        << 8 as ::core::ffi::c_int));
                                mod_mask.set(0 as ::core::ffi::c_int);
                            } else {
                                c = K_END;
                            }
                        }
                        K_KUP | K_XUP => {
                            c = K_UP;
                        }
                        K_KDOWN | K_XDOWN => {
                            c = K_DOWN;
                        }
                        K_KLEFT | K_XLEFT => {
                            c = K_LEFT;
                        }
                        K_KRIGHT | K_XRIGHT => {
                            c = K_RIGHT;
                        }
                        _ => {}
                    }
                    break;
                }
            }
            last_vgetc_recorded_len.set(last_recorded_len.get());
        }
        may_garbage_collect.set(false_0 != 0);
        if (*on_key_buf.ptr()).size == (*on_key_buf.ptr()).capacity {
            (*on_key_buf.ptr()).capacity = if (*on_key_buf.ptr()).capacity
                << 1 as ::core::ffi::c_int
                > ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                            .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                (*on_key_buf.ptr()).capacity << 1 as ::core::ffi::c_int
            } else {
                ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                            .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                            == 0) as ::core::ffi::c_int as size_t,
                    )
            };
            (*on_key_buf.ptr()).items = (if (*on_key_buf.ptr()).capacity
                == ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                    .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
                    .wrapping_div(
                        (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                            .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                            == 0) as ::core::ffi::c_int as usize,
                    ) {
                if (*on_key_buf.ptr()).items
                    == &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
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
                    == &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
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
            }) as *mut ::core::ffi::c_char;
        } else {
        };
        let c2rust_fresh10 = (*on_key_buf.ptr()).size;
        (*on_key_buf.ptr()).size = (*on_key_buf.ptr()).size.wrapping_add(1);
        *(*on_key_buf.ptr()).items.offset(c2rust_fresh10 as isize) = '\0' as ::core::ffi::c_char;
        if nlua_execute_on_key(c, (*on_key_buf.ptr()).items) {
            if c == -(253 as ::core::ffi::c_int
                + ((KE_COMMAND as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                xfree(
                    getcmdkeycmd(NUL, NULL_0, 0 as ::core::ffi::c_int, false_0 != 0)
                        as *mut ::core::ffi::c_void,
                );
            } else if c
                == -(253 as ::core::ffi::c_int
                    + ((KE_LUA as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
            {
                map_execute_lua(false_0 != 0, true_0 != 0);
            } else if c == K_PASTE_START {
                paste_repeat(0 as ::core::ffi::c_int);
            }
            c = -(253 as ::core::ffi::c_int
                + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int));
        }
        if (*on_key_buf.ptr()).items
            != &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char
        {
            let mut ptr_: *mut *mut ::core::ffi::c_void =
                &raw mut (*on_key_buf.ptr()).items as *mut *mut ::core::ffi::c_void;
            xfree(*ptr_);
            *ptr_ = NULL_0;
            let _ = *ptr_;
        }
        (*on_key_buf.ptr()).capacity = ::core::mem::size_of::<[::core::ffi::c_char; 51]>()
            .wrapping_div(::core::mem::size_of::<::core::ffi::c_char>())
            .wrapping_div(
                (::core::mem::size_of::<[::core::ffi::c_char; 51]>()
                    .wrapping_rem(::core::mem::size_of::<::core::ffi::c_char>())
                    == 0) as ::core::ffi::c_int as usize,
            ) as size_t;
        (*on_key_buf.ptr()).size = 0 as size_t;
        (*on_key_buf.ptr()).items =
            &raw mut (*on_key_buf.ptr()).init_array as *mut ::core::ffi::c_char;
        if c != -(253 as ::core::ffi::c_int
            + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
        {
            state_no_longer_safe(b"key typed\0".as_ptr() as *const ::core::ffi::c_char);
        }
        return c;
    }
}

pub unsafe extern "C" fn safe_vgetc() -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = vgetc();
        if c == NUL {
            c = get_keystroke(::core::ptr::null_mut::<MultiQueue>());
        }
        return c;
    }
}

pub unsafe extern "C" fn plain_vgetc() -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = 0;
        loop {
            c = safe_vgetc();
            if !(c
                == -(253 as ::core::ffi::c_int
                    + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || c == K_VER_SCROLLBAR
                || c == K_HOR_SCROLLBAR
                || c == -(253 as ::core::ffi::c_int
                    + ((KE_MOUSEMOVE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
            {
                break;
            }
        }
        return c;
    }
}

pub unsafe extern "C" fn vpeekc() -> ::core::ffi::c_int {
    unsafe {
        if can_get_old_char() {
            return old_char.get();
        }
        return vgetorpeek(false_0 != 0);
    }
}

pub unsafe extern "C" fn vpeekc_any() -> ::core::ffi::c_int {
    unsafe {
        let mut c: ::core::ffi::c_int = vpeekc();
        if c == NUL && (*typebuf.ptr()).tb_len > 0 as ::core::ffi::c_int {
            c = ESC;
        }
        return c;
    }
}

pub unsafe extern "C" fn char_avail() -> bool {
    unsafe {
        if test_disable_char_avail.get() {
            return false_0 != 0;
        }
        (*no_mapping.ptr()) += 1;
        let mut retval: ::core::ffi::c_int = vpeekc();
        (*no_mapping.ptr()) -= 1;
        return retval != NUL;
    }
}

pub unsafe extern "C" fn vungetc(mut c: ::core::ffi::c_int) {
    old_char.set(c);
    old_mod_mask.set(mod_mask.get());
    old_mouse_grid.set(mouse_grid.get());
    old_mouse_row.set(mouse_row.get());
    old_mouse_col.set(mouse_col.get());
    old_KeyStuffed.set(KeyStuffed.get());
}

pub unsafe extern "C" fn check_end_reg_executing(mut advance: bool) {
    unsafe {
        if reg_executing.get() != 0 as ::core::ffi::c_int
            && ((*typebuf.ptr()).tb_maplen == 0 as ::core::ffi::c_int
                || pending_end_reg_executing.get() as ::core::ffi::c_int != 0)
        {
            if advance {
                reg_executing.set(0 as ::core::ffi::c_int);
                pending_end_reg_executing.set(false_0 != 0);
            } else {
                pending_end_reg_executing.set(true_0 != 0);
            }
        }
    }
}
