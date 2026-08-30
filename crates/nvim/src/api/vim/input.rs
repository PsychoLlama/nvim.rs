//! Feeding the editor keys, and the mappings they may hit.
//!
//! `nvim_feedkeys` and `nvim_input` are the two ends of the input path: one
//! goes through the typeahead buffer with the caller's mode flags, the
//! other straight into it as if typed.  `nvim_input_mouse` synthesises a
//! mouse event from (button, action, grid, row, col).  The keymap
//! accessors sit here because they answer for the same table `nvim_input`
//! is resolved against.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api_error;
use crate::cstr;
use crate::getchar::typeahead;
use crate::guard::Depth;
use crate::keycodes::{
    KE_LEFTDRAG, KE_LEFTMOUSE, KE_LEFTRELEASE, KE_MIDDLEMOUSE, KE_MOUSEDOWN, KE_MOUSELEFT,
    KE_MOUSEMOVE, KE_MOUSERIGHT, KE_MOUSEUP, KE_RIGHTMOUSE, KE_X1MOUSE, KE_X2MOUSE,
};
use crate::message_fmt::msg_bytes;

pub unsafe fn nvim_feedkeys(keys: String_0, mode: String_0, escape_ks: Boolean) {
    let mut remap: bool = true;
    let mut insert: bool = false;
    let mut typed: bool = false;
    let mut execute: bool = false;
    let mut dangerous: bool = false;
    let mut lowlevel: bool = false;
    let mut i: size_t = 0 as size_t;
    while i < mode.len() {
        match unsafe { *mode.data().add(i) } as ::core::ffi::c_int {
            110 => {
                remap = false;
            }
            109 => {
                remap = true;
            }
            116 => {
                typed = true;
            }
            105 => {
                insert = true;
            }
            120 => {
                execute = true;
            }
            33 => {
                dangerous = true;
            }
            76 => {
                lowlevel = true;
            }
            _ => {}
        }
        i = i.wrapping_add(1);
    }
    if keys.len() == 0 as size_t && !execute {
        return;
    }
    let mut keys_esc: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    if escape_ks {
        keys_esc = unsafe { vim_strsave_escape_ks(keys.data()) };
    } else {
        keys_esc = keys.data();
    }
    if lowlevel {
        unsafe { input_enqueue_raw(keys_esc, cstr::bytes_at(keys_esc).len()) };
    } else {
        let remap_flag = if remap {
            REMAP_YES as ::core::ffi::c_int
        } else {
            REMAP_NONE as ::core::ffi::c_int
        };
        // `insert` puts the keys at the front of the typeahead; without it
        // they go after whatever is already queued.
        let offset = if insert { 0 } else { typeahead().len() };
        // SAFETY: `keys_esc` is the escaped copy, or the caller's own string.
        let _ = unsafe { ins_typebuf(keys_esc, remap_flag, offset, !typed, false) };
        if vgetc_busy.get() != 0 {
            typebuf_was_filled.set(true);
        }
    }
    if escape_ks {
        unsafe { xfree(keys_esc as *mut ::core::ffi::c_void) };
    }
    if execute {
        let mut save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
        msg_scroll.set(0);
        let busy = (!dangerous).then(|| Depth::of(&ex_normal_busy));
        unsafe { exec_normal(true, lowlevel) };
        drop(busy);
        msg_scroll.set(msg_scroll.get() | save_msg_scroll);
    }
}

pub unsafe fn nvim_input(channel_id: uint64_t, keys: String_0) -> Integer {
    may_trigger_vim_suspend_resume(false);
    unsafe { input_enqueue(channel_id, keys) as Integer }
}

pub unsafe fn nvim_input_mouse(
    button: String_0,
    action: String_0,
    modifier: String_0,
    grid: Integer,
    row: Integer,
    col: Integer,
) -> Result<(), Error> {
    let mut error = Error::none();
    let mut code: ::core::ffi::c_int = 0;
    let mut modmask: ::core::ffi::c_int = 0;
    may_trigger_vim_suspend_resume(false);
    '_error: {
        if !(button.data().is_null() || action.data().is_null()) {
            code = 0 as ::core::ffi::c_int;
            if unsafe { strequal(button.data(), c"left".as_ptr()) } {
                code = KE_LEFTMOUSE as ::core::ffi::c_int;
            } else if unsafe { strequal(button.data(), c"middle".as_ptr()) } {
                code = KE_MIDDLEMOUSE as ::core::ffi::c_int;
            } else if unsafe { strequal(button.data(), c"right".as_ptr()) } {
                code = KE_RIGHTMOUSE as ::core::ffi::c_int;
            } else if unsafe { strequal(button.data(), c"wheel".as_ptr()) } {
                code = KE_MOUSEDOWN as ::core::ffi::c_int;
            } else if unsafe { strequal(button.data(), c"x1".as_ptr()) } {
                code = KE_X1MOUSE as ::core::ffi::c_int;
            } else if unsafe { strequal(button.data(), c"x2".as_ptr()) } {
                code = KE_X2MOUSE as ::core::ffi::c_int;
            } else if unsafe { strequal(button.data(), c"move".as_ptr()) } {
                code = KE_MOUSEMOVE as ::core::ffi::c_int;
            } else {
                break '_error;
            }
            if code == KE_MOUSEDOWN as ::core::ffi::c_int {
                if unsafe { strequal(action.data(), c"down".as_ptr()) } {
                    code = KE_MOUSEUP as ::core::ffi::c_int;
                } else if !unsafe { strequal(action.data(), c"up".as_ptr()) } {
                    if unsafe { strequal(action.data(), c"left".as_ptr()) } {
                        code = KE_MOUSERIGHT as ::core::ffi::c_int;
                    } else if unsafe { strequal(action.data(), c"right".as_ptr()) } {
                        code = KE_MOUSELEFT as ::core::ffi::c_int;
                    } else {
                        break '_error;
                    }
                }
            } else if code != KE_MOUSEMOVE as ::core::ffi::c_int
                && !unsafe { strequal(action.data(), c"press".as_ptr()) }
            {
                if unsafe { strequal(action.data(), c"drag".as_ptr()) } {
                    code += KE_LEFTDRAG as ::core::ffi::c_int - KE_LEFTMOUSE as ::core::ffi::c_int;
                } else if unsafe { strequal(action.data(), c"release".as_ptr()) } {
                    code +=
                        KE_LEFTRELEASE as ::core::ffi::c_int - KE_LEFTMOUSE as ::core::ffi::c_int;
                } else {
                    break '_error;
                }
            }
            modmask = 0 as ::core::ffi::c_int;
            let mut i: size_t = 0 as size_t;
            while i < modifier.len() {
                let mut byte: ::core::ffi::c_char = unsafe { *modifier.data().add(i) };
                if byte as ::core::ffi::c_int != '-' as ::core::ffi::c_int {
                    let mut mod_0: ::core::ffi::c_int =
                        name_to_mod_mask(byte as ::core::ffi::c_int);
                    if !(mod_0 != 0 as ::core::ffi::c_int) {
                        // `%c` wrote the one byte, whatever it was; the
                        // adaptor keeps it rather than widening it to a char.
                        let raw = byte as u8;
                        let byte = msg_bytes(core::slice::from_ref(&raw));
                        error = api_error!(kErrorTypeValidation, "Invalid modifier: {byte}");
                        return ().reported(error);
                    }
                    modmask |= mod_0;
                }
                i = i.wrapping_add(1);
            }
            input_enqueue_mouse(
                code,
                modmask as uint8_t,
                grid as ::core::ffi::c_int,
                row as ::core::ffi::c_int,
                col as ::core::ffi::c_int,
            );
            return ().reported(error);
        }
    }
    error = Error::validation(c"invalid button or action");
    ().reported(error)
}

pub unsafe fn nvim_replace_termcodes(
    str: String_0,
    from_part: Boolean,
    do_lt: Boolean,
    special: Boolean,
) -> String_0 {
    if str.len() == 0 as size_t {
        return String_0::from_raw_parts(
            ::core::ptr::null_mut::<::core::ffi::c_char>(),
            0 as size_t,
        );
    }
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if from_part {
        flags |= REPTERM_FROM_PART as ::core::ffi::c_int;
    }
    if do_lt {
        flags |= REPTERM_DO_LT as ::core::ffi::c_int;
    }
    if !special {
        flags |= REPTERM_NO_SPECIAL as ::core::ffi::c_int;
    }
    let mut ptr: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
    let (text, len, out) = (str.data(), str.len(), &raw mut ptr);
    let (no_flag, cpo) = (::core::ptr::null_mut::<bool>(), p_cpo.get());
    // SAFETY: `str` is the caller's, `ptr` this frame's own out-parameter,
    // and `'cpoptions'` a live NUL-terminated string.
    unsafe { replace_termcodes(text, len, out, 0 as scid_T, flags, no_flag, cpo) };
    // SAFETY: `replace_termcodes` left an owned C string in `ptr`.
    unsafe { cstr_as_string(ptr) }
}

pub unsafe fn nvim_get_keymap(mode: String_0, arena: *mut Arena) -> Array {
    unsafe { keymap_array(mode, None, arena) }
}

pub unsafe fn nvim_set_keymap(
    channel_id: uint64_t,
    mode: String_0,
    lhs: String_0,
    rhs: String_0,
    opts: *mut KeyDict_keymap,
) -> Result<(), Error> {
    let mut error = Error::none();
    let slot = &mut error;
    unsafe { modify_keymap(channel_id, -1 as Buffer, false, mode, lhs, rhs, opts, slot) };
    ().reported(error)
}

pub unsafe fn nvim_del_keymap(
    channel_id: uint64_t,
    mode: String_0,
    lhs: String_0,
) -> Result<(), Error> {
    unsafe { nvim_buf_del_keymap(channel_id, -1 as Buffer, mode, lhs) }
}

pub unsafe fn nvim_select_popupmenu_item(
    item: Integer,
    mut insert: Boolean,
    finish: Boolean,
    _opts: *mut KeyDict_empty,
) {
    if finish {
        insert = true;
    }
    pum_ext_select_item(item as ::core::ffi::c_int, insert, finish);
}
