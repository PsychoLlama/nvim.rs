use crate::src::nvim::ex_getln::getcmdline_prompt;
use crate::src::nvim::getchar::{fix_input_buffer, merge_modifiers};

use crate::src::nvim::eval::typval::kCallbackNone;
use crate::src::nvim::highlight_group::HLF_R;
use crate::src::nvim::keycodes::{Ctrl_C, K_SPECIAL, K_ZERO, KE_IGNORE, KE_LEFTMOUSE};
use crate::src::nvim::main::{
    IObuff, State, allow_keys, cmdline_row, keep_msg, keep_msg_hl_id, mapped_ctrl_c, mod_mask,
    msg_row, msg_scrolled, need_wait_return, no_mapping, no_wait_return,
};
use crate::src::nvim::mbyte::{utf_ptr2char, utf8len_tab};
use crate::src::nvim::memory::{xfree, xmalloc, xrealloc, xstrdup};
use crate::src::nvim::message::{msg_putchar, set_keep_msg};
use crate::src::nvim::mouse::{is_mouse_key, setmouse};
use crate::src::nvim::os::input::input_get;
use crate::src::nvim::os::libc::{atoi, gettext, memmove, snprintf};
use crate::src::nvim::types::ui::kUIMessages;
use crate::src::nvim::types::{
    Callback, Callback_data as C2Rust_Unnamed, MultiQueue, size_t, uint8_t,
};
use crate::src::nvim::ui::{ui_flush, ui_has};
pub type C2Rust_Unnamed_1 = ::core::ffi::c_int;
pub const EXPAND_FILES: C2Rust_Unnamed_1 = 2;
pub const EXPAND_NOTHING: C2Rust_Unnamed_1 = 0;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
pub const NUL: ::core::ffi::c_int = '\0' as ::core::ffi::c_int;
pub const ESC: ::core::ffi::c_int = '\u{1b}' as ::core::ffi::c_int;
pub const IOSIZE: ::core::ffi::c_int = 1024 as ::core::ffi::c_int + 1 as ::core::ffi::c_int;
pub unsafe extern "C" fn ask_yesno(str: *const ::core::ffi::c_char) -> ::core::ffi::c_int {
    let save_State: ::core::ffi::c_int = State.get();
    (*no_wait_return.ptr()) += 1;
    snprintf(
        IObuff.ptr() as *mut ::core::ffi::c_char,
        IOSIZE as size_t,
        gettext(c"%s (y/n)?".as_ptr()),
        str,
    );
    let mut prompt: *mut ::core::ffi::c_char = xstrdup(IObuff.ptr() as *mut ::core::ffi::c_char);
    let mut r: ::core::ffi::c_int = ' ' as ::core::ffi::c_int;
    while r != 'y' as ::core::ffi::c_int && r != 'n' as ::core::ffi::c_int {
        r = prompt_for_input(prompt, HLF_R, true_0 != 0, ::core::ptr::null_mut::<bool>());
        if r == Ctrl_C || r == ESC {
            r = 'n' as ::core::ffi::c_int;
            if !ui_has(kUIMessages) {
                msg_putchar(r);
            }
        }
    }
    need_wait_return.set(msg_scrolled.get() != 0);
    (*no_wait_return.ptr()) -= 1;
    State.set(save_State);
    setmouse();
    xfree(prompt as *mut ::core::ffi::c_void);
    return r;
}
pub unsafe extern "C" fn get_keystroke(mut events: *mut MultiQueue) -> ::core::ffi::c_int {
    let mut buf: *mut uint8_t = ::core::ptr::null_mut::<uint8_t>();
    let mut buflen: ::core::ffi::c_int = 150 as ::core::ffi::c_int;
    let mut len: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut n: ::core::ffi::c_int = 0;
    let mut save_mapped_ctrl_c: ::core::ffi::c_int = mapped_ctrl_c.get();
    mod_mask.set(0 as ::core::ffi::c_int);
    mapped_ctrl_c.set(0 as ::core::ffi::c_int);
    loop {
        ui_flush();
        let mut maxlen: ::core::ffi::c_int =
            (buflen - 6 as ::core::ffi::c_int - len) / 3 as ::core::ffi::c_int;
        if buf.is_null() {
            buf = xmalloc(buflen as size_t) as *mut uint8_t;
        } else if maxlen < 10 as ::core::ffi::c_int {
            buflen += 100 as ::core::ffi::c_int;
            buf = xrealloc(buf as *mut ::core::ffi::c_void, buflen as size_t) as *mut uint8_t;
            maxlen = (buflen - 6 as ::core::ffi::c_int - len) / 3 as ::core::ffi::c_int;
        }
        n = input_get(
            buf.offset(len as isize),
            maxlen,
            if len == 0 as ::core::ffi::c_int {
                -1 as ::core::ffi::c_int
            } else {
                100 as ::core::ffi::c_int
            },
            0 as ::core::ffi::c_int,
            events,
        );
        if n > 0 as ::core::ffi::c_int {
            n = fix_input_buffer(buf.offset(len as isize), n);
            len += n;
        }
        if n > 0 as ::core::ffi::c_int {
            len = n;
        }
        if len == 0 as ::core::ffi::c_int {
            continue;
        }
        n = *buf.offset(0 as ::core::ffi::c_int as isize) as ::core::ffi::c_int;
        if n == K_SPECIAL {
            n = if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_SPECIAL
            {
                K_SPECIAL
            } else if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_ZERO
            {
                K_ZERO
            } else {
                -(*buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    + ((*buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
                        << 8 as ::core::ffi::c_int))
            };
            if !(*buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER
                || n == -(253 as ::core::ffi::c_int
                    + ((KE_IGNORE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int))
                || is_mouse_key(n) as ::core::ffi::c_int != 0
                    && n != -(253 as ::core::ffi::c_int
                        + ((KE_LEFTMOUSE as ::core::ffi::c_int) << 8 as ::core::ffi::c_int)))
            {
                break;
            }
            if *buf.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int == KS_MODIFIER {
                mod_mask.set(*buf.offset(2 as ::core::ffi::c_int as isize) as ::core::ffi::c_int);
            }
            len -= 3 as ::core::ffi::c_int;
            if len > 0 as ::core::ffi::c_int {
                memmove(
                    buf as *mut ::core::ffi::c_void,
                    buf.offset(3 as ::core::ffi::c_int as isize) as *const ::core::ffi::c_void,
                    len as size_t,
                );
            }
        } else {
            if utf8len_tab[n as usize] as ::core::ffi::c_int > len {
                continue;
            }
            *buf.offset(
                (if len >= buflen {
                    buflen - 1 as ::core::ffi::c_int
                } else {
                    len
                }) as isize,
            ) = NUL as uint8_t;
            n = utf_ptr2char(buf as *mut ::core::ffi::c_char);
            break;
        }
    }
    xfree(buf as *mut ::core::ffi::c_void);
    mapped_ctrl_c.set(save_mapped_ctrl_c);
    return merge_modifiers(n, mod_mask.ptr());
}
pub unsafe extern "C" fn prompt_for_input(
    mut prompt: *mut ::core::ffi::c_char,
    mut hl_id: ::core::ffi::c_int,
    mut one_key: bool,
    mut mouse_used: *mut bool,
) -> ::core::ffi::c_int {
    let mut ret: ::core::ffi::c_int = if one_key as ::core::ffi::c_int != 0 {
        ESC
    } else {
        0 as ::core::ffi::c_int
    };
    let mut kmsg: *mut ::core::ffi::c_char = if !(*keep_msg.ptr()).is_null() {
        xstrdup(keep_msg.get())
    } else {
        ::core::ptr::null_mut::<::core::ffi::c_char>()
    };
    if prompt.is_null() {
        if !mouse_used.is_null() {
            prompt = gettext(
                c"Type number and <Enter> or click with the mouse (q or empty cancels): ".as_ptr(),
            );
        } else {
            prompt = gettext(c"Type number and <Enter> (q or empty cancels): ".as_ptr());
        }
    }
    cmdline_row.set(msg_row.get());
    ui_flush();
    (*no_mapping.ptr()) += 1;
    (*allow_keys.ptr()) += 1;
    let mut resp: *mut ::core::ffi::c_char = getcmdline_prompt(
        -1 as ::core::ffi::c_int,
        prompt,
        hl_id,
        EXPAND_NOTHING as ::core::ffi::c_int,
        ::core::ptr::null::<::core::ffi::c_char>(),
        Callback {
            data: C2Rust_Unnamed {
                funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
            },
            type_0: kCallbackNone,
        },
        one_key,
        mouse_used,
    );
    (*allow_keys.ptr()) -= 1;
    (*no_mapping.ptr()) -= 1;
    if !resp.is_null() {
        ret = if one_key as ::core::ffi::c_int != 0 {
            *resp as ::core::ffi::c_int
        } else {
            atoi(resp)
        };
        xfree(resp as *mut ::core::ffi::c_void);
    }
    if !kmsg.is_null() {
        set_keep_msg(kmsg, keep_msg_hl_id.get());
        xfree(kmsg as *mut ::core::ffi::c_void);
    }
    return ret;
}
pub const KS_ZERO: ::core::ffi::c_int = 255 as ::core::ffi::c_int;
pub const KS_SPECIAL: ::core::ffi::c_int = 254 as ::core::ffi::c_int;
pub const KS_MODIFIER: ::core::ffi::c_int = 252 as ::core::ffi::c_int;
pub const true_0: ::core::ffi::c_int = 1 as ::core::ffi::c_int;
