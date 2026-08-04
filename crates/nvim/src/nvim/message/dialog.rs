//! `confirm()`, and the console dialog it falls back to.
//!
//! [`do_dialog`] renders the message plus the button list, works out the
//! hotkey letters ([`copy_confirm_hotkeys`]) and reads a keystroke until one
//! of them matches.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn do_dialog(
    mut _type_0: ::core::ffi::c_int,
    mut _title: *const ::core::ffi::c_char,
    mut message: *const ::core::ffi::c_char,
    mut buttons: *const ::core::ffi::c_char,
    mut dfltbutton: ::core::ffi::c_int,
    mut _textfield: *const ::core::ffi::c_char,
    mut ex_cmd: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut retval: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut i: ::core::ffi::c_int = 0;
        if silent_mode.get() {
            return dfltbutton;
        }
        let mut save_msg_silent: ::core::ffi::c_int = msg_silent.get();
        let mut oldState: ::core::ffi::c_int = State.get();
        msg_silent.set(0 as ::core::ffi::c_int);
        (*no_wait_return.ptr()) += 1;
        let mut hotkeys: *mut ::core::ffi::c_char =
            msg_show_console_dialog(message, buttons, dfltbutton);
        loop {
            if ui_active() == 0 && input_available() == 0 {
                retval = dfltbutton;
                break;
            } else {
                let mut c: ::core::ffi::c_int = prompt_for_input(
                    confirm_buttons.get(),
                    HLF_M,
                    true_0 != 0,
                    ::core::ptr::null_mut::<bool>(),
                );
                match c {
                    CAR | NUL => {
                        retval = dfltbutton;
                        break;
                    }
                    Ctrl_C | ESC => {
                        retval = 0 as ::core::ffi::c_int;
                        break;
                    }
                    _ => {
                        if c < 0 as ::core::ffi::c_int {
                            msg_didany.set(false_0 != 0);
                            msg_didout.set(msg_didany.get());
                        } else if c == ':' as ::core::ffi::c_int && ex_cmd != 0 {
                            retval = dfltbutton;
                            ins_char_typebuf(
                                ':' as ::core::ffi::c_int,
                                0 as ::core::ffi::c_int,
                                false_0 != 0,
                            );
                            break;
                        } else {
                            c = mb_tolower(c);
                            retval = 1 as ::core::ffi::c_int;
                            i = 0 as ::core::ffi::c_int;
                            while *hotkeys.offset(i as isize) != 0 {
                                if utf_ptr2char(hotkeys.offset(i as isize)) == c {
                                    break;
                                }
                                i += utfc_ptr2len(hotkeys.offset(i as isize))
                                    - 1 as ::core::ffi::c_int;
                                retval += 1;
                                i += 1;
                            }
                            if *hotkeys.offset(i as isize) != 0 {
                                break;
                            }
                            msg_didany.set(false_0 != 0);
                            msg_didout.set(msg_didany.get());
                        }
                    }
                }
            }
        }
        xfree(hotkeys as *mut ::core::ffi::c_void);
        xfree(confirm_msg.get() as *mut ::core::ffi::c_void);
        confirm_msg.set(::core::ptr::null_mut::<::core::ffi::c_char>());
        msg_silent.set(save_msg_silent);
        State.set(oldState);
        setmouse();
        (*no_wait_return.ptr()) -= 1;
        msg_end_prompt();
        return retval;
    }
}

pub(crate) unsafe extern "C" fn copy_char(
    mut from: *const ::core::ffi::c_char,
    mut to: *mut ::core::ffi::c_char,
    mut lowercase: bool,
) -> ::core::ffi::c_int {
    unsafe {
        if lowercase {
            let mut c: ::core::ffi::c_int = mb_tolower(utf_ptr2char(from));
            return utf_char2bytes(c, to);
        }
        let mut len: ::core::ffi::c_int = utfc_ptr2len(from);
        memmove(
            to as *mut ::core::ffi::c_void,
            from as *const ::core::ffi::c_void,
            len as size_t,
        );
        return len;
    }
}

pub(crate) unsafe extern "C" fn console_dialog_alloc(
    mut message: *const ::core::ffi::c_char,
    mut buttons: *const ::core::ffi::c_char,
    mut has_hotkey: *mut bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut lenhotkey: ::core::ffi::c_int = MB_MAXBYTES as ::core::ffi::c_int;
        *has_hotkey.offset(0 as ::core::ffi::c_int as isize) = false_0 != 0;
        let mut msg_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut button_len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut r: *const ::core::ffi::c_char = buttons;
        while *r != 0 {
            if *r as ::core::ffi::c_int == DLG_BUTTON_SEP as ::core::ffi::c_int {
                button_len += 3 as ::core::ffi::c_int;
                lenhotkey += MB_MAXBYTES as ::core::ffi::c_int;
                if idx < HAS_HOTKEY_LEN - 1 as ::core::ffi::c_int {
                    idx += 1;
                    *has_hotkey.offset(idx as isize) = false_0 != 0;
                }
            } else if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int {
                r = r.offset(1);
                button_len += 1;
                if idx < HAS_HOTKEY_LEN - 1 as ::core::ffi::c_int {
                    *has_hotkey.offset(idx as isize) = true_0 != 0;
                }
            }
            r = r.offset(utfc_ptr2len(r as *mut ::core::ffi::c_char) as isize);
        }
        msg_len += strlen(message) as ::core::ffi::c_int + 3 as ::core::ffi::c_int;
        button_len += strlen(buttons) as ::core::ffi::c_int + 3 as ::core::ffi::c_int;
        lenhotkey += 1;
        if !*has_hotkey.offset(0 as ::core::ffi::c_int as isize) {
            button_len += 2 as ::core::ffi::c_int;
        }
        confirm_msg.set(xmalloc(msg_len as size_t) as *mut ::core::ffi::c_char);
        snprintf(
            confirm_msg.get(),
            msg_len as size_t,
            if ui_has(kUIMessages) as ::core::ffi::c_int != 0 {
                b"%s\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                b"\n%s\n\0".as_ptr() as *const ::core::ffi::c_char
            },
            message,
        );
        xfree(confirm_buttons.get() as *mut ::core::ffi::c_void);
        confirm_buttons.set(xmalloc(button_len as size_t) as *mut ::core::ffi::c_char);
        return xmalloc(lenhotkey as size_t) as *mut ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn msg_show_console_dialog(
    mut message: *const ::core::ffi::c_char,
    mut buttons: *const ::core::ffi::c_char,
    mut dfltbutton: ::core::ffi::c_int,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut has_hotkey: [bool; 30] = [
            false_0 != 0,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        ];
        let mut hotk: *mut ::core::ffi::c_char =
            console_dialog_alloc(message, buttons, &raw mut has_hotkey as *mut bool);
        copy_confirm_hotkeys(
            buttons,
            dfltbutton,
            &raw mut has_hotkey as *mut bool as *const bool,
            hotk,
        );
        display_confirm_msg();
        return hotk;
    }
}

pub(crate) unsafe extern "C" fn copy_confirm_hotkeys(
    mut buttons: *const ::core::ffi::c_char,
    mut default_button_idx: ::core::ffi::c_int,
    mut has_hotkey: *const bool,
    mut hotkeys_ptr: *mut ::core::ffi::c_char,
) {
    unsafe {
        *hotkeys_ptr.offset(copy_char(buttons, hotkeys_ptr, true_0 != 0) as isize) =
            NUL as ::core::ffi::c_char;
        let mut first_hotkey: bool = false_0 != 0;
        if !*has_hotkey.offset(0 as ::core::ffi::c_int as isize) {
            first_hotkey = true_0 != 0;
        }
        let mut msgp: *mut ::core::ffi::c_char = confirm_buttons.get();
        let mut idx: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut r: *const ::core::ffi::c_char = buttons;
        while *r != 0 {
            if *r as ::core::ffi::c_int == DLG_BUTTON_SEP as ::core::ffi::c_int {
                let c2rust_fresh39 = msgp;
                msgp = msgp.offset(1);
                *c2rust_fresh39 = ',' as ::core::ffi::c_char;
                let c2rust_fresh40 = msgp;
                msgp = msgp.offset(1);
                *c2rust_fresh40 = ' ' as ::core::ffi::c_char;
                hotkeys_ptr = hotkeys_ptr.offset(strlen(hotkeys_ptr) as isize);
                *hotkeys_ptr.offset(copy_char(
                    r.offset(1 as ::core::ffi::c_int as isize),
                    hotkeys_ptr,
                    true_0 != 0,
                ) as isize) = NUL as ::core::ffi::c_char;
                if default_button_idx != 0 {
                    default_button_idx -= 1;
                }
                if idx < HAS_HOTKEY_LEN - 1 as ::core::ffi::c_int && {
                    idx += 1;
                    !*has_hotkey.offset(idx as isize)
                } {
                    first_hotkey = true_0 != 0;
                }
            } else if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int
                || first_hotkey as ::core::ffi::c_int != 0
            {
                if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int {
                    r = r.offset(1);
                }
                first_hotkey = false_0 != 0;
                if *r as ::core::ffi::c_int == DLG_HOTKEY_CHAR as ::core::ffi::c_int {
                    let c2rust_fresh41 = msgp;
                    msgp = msgp.offset(1);
                    *c2rust_fresh41 = *r;
                } else {
                    let c2rust_fresh42 = msgp;
                    msgp = msgp.offset(1);
                    *c2rust_fresh42 = (if default_button_idx == 1 as ::core::ffi::c_int {
                        '[' as ::core::ffi::c_int
                    } else {
                        '(' as ::core::ffi::c_int
                    }) as ::core::ffi::c_char;
                    msgp = msgp.offset(copy_char(r, msgp, false_0 != 0) as isize);
                    let c2rust_fresh43 = msgp;
                    msgp = msgp.offset(1);
                    *c2rust_fresh43 = (if default_button_idx == 1 as ::core::ffi::c_int {
                        ']' as ::core::ffi::c_int
                    } else {
                        ')' as ::core::ffi::c_int
                    }) as ::core::ffi::c_char;
                    *hotkeys_ptr.offset(copy_char(r, hotkeys_ptr, true_0 != 0) as isize) =
                        NUL as ::core::ffi::c_char;
                }
            } else {
                msgp = msgp.offset(copy_char(r, msgp, false_0 != 0) as isize);
            }
            r = r.offset(utfc_ptr2len(r as *mut ::core::ffi::c_char) as isize);
        }
        let c2rust_fresh44 = msgp;
        msgp = msgp.offset(1);
        *c2rust_fresh44 = ':' as ::core::ffi::c_char;
        let c2rust_fresh45 = msgp;
        msgp = msgp.offset(1);
        *c2rust_fresh45 = ' ' as ::core::ffi::c_char;
        *msgp = NUL as ::core::ffi::c_char;
    }
}

pub(crate) unsafe extern "C" fn display_confirm_msg() {
    unsafe {
        (*confirm_msg_used.ptr()) += 1;
        if !(*confirm_msg.ptr()).is_null() {
            msg_ext_set_kind(b"confirm\0".as_ptr() as *const ::core::ffi::c_char);
            msg_puts_hl(confirm_msg.get(), HLF_M, false_0 != 0);
        }
        (*confirm_msg_used.ptr()) -= 1;
    }
}

pub unsafe extern "C" fn vim_dialog_yesno(
    mut type_0: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut message: *mut ::core::ffi::c_char,
    mut dflt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        if do_dialog(
            type_0,
            if title.is_null() {
                gettext(b"Question\0".as_ptr() as *const ::core::ffi::c_char)
            } else {
                title
            },
            message,
            gettext(b"&Yes\n&No\0".as_ptr() as *const ::core::ffi::c_char),
            dflt,
            ::core::ptr::null::<::core::ffi::c_char>(),
            false_0,
        ) == 1 as ::core::ffi::c_int
        {
            return VIM_YES as ::core::ffi::c_int;
        }
        return VIM_NO as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn vim_dialog_yesnocancel(
    mut type_0: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut message: *mut ::core::ffi::c_char,
    mut dflt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        match do_dialog(
            type_0,
            if title.is_null() {
                gettext(b"Question\0".as_ptr() as *const ::core::ffi::c_char)
            } else {
                title
            },
            message,
            gettext(b"&Yes\n&No\n&Cancel\0".as_ptr() as *const ::core::ffi::c_char),
            dflt,
            ::core::ptr::null::<::core::ffi::c_char>(),
            false_0,
        ) {
            1 => return VIM_YES as ::core::ffi::c_int,
            2 => return VIM_NO as ::core::ffi::c_int,
            _ => {}
        }
        return VIM_CANCEL as ::core::ffi::c_int;
    }
}

pub unsafe extern "C" fn vim_dialog_yesnoallcancel(
    mut type_0: ::core::ffi::c_int,
    mut title: *mut ::core::ffi::c_char,
    mut message: *mut ::core::ffi::c_char,
    mut dflt: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        match do_dialog(
            type_0,
            if title.is_null() {
                b"Question\0".as_ptr() as *const ::core::ffi::c_char
            } else {
                title as *const ::core::ffi::c_char
            },
            message,
            gettext(b"&Yes\n&No\nSave &All\n&Discard All\n&Cancel\0".as_ptr()
                as *const ::core::ffi::c_char),
            dflt,
            ::core::ptr::null::<::core::ffi::c_char>(),
            false_0,
        ) {
            1 => return VIM_YES as ::core::ffi::c_int,
            2 => return VIM_NO as ::core::ffi::c_int,
            3 => return VIM_ALL as ::core::ffi::c_int,
            4 => return VIM_DISCARDALL as ::core::ffi::c_int,
            _ => {}
        }
        return VIM_CANCEL as ::core::ffi::c_int;
    }
}
