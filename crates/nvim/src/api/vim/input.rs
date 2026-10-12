//! Feeding the editor keys, and the mappings they may hit.
//!
//! `nvim_feedkeys` and `nvim_input` are the two ends of the input path: one
//! goes through the typeahead buffer with the caller's mode flags, the
//! other straight into it as if typed.  `nvim_input_mouse` synthesises a
//! mouse event from (button, action, grid, row, col).  The keymap
//! accessors sit here because they answer for the same table `nvim_input`
//! is resolved against.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::api::private::helpers::Reported;
use crate::api_error;
use crate::cstr;
use crate::getchar::typeahead;
use crate::guard::Depth;
use crate::keycodes::ModMask;
use crate::keycodes::{
    KE_LEFTDRAG, KE_LEFTMOUSE, KE_LEFTRELEASE, KE_MIDDLEMOUSE, KE_MOUSEDOWN, KE_MOUSELEFT,
    KE_MOUSEMOVE, KE_MOUSERIGHT, KE_MOUSEUP, KE_RIGHTMOUSE, KE_X1MOUSE, KE_X2MOUSE,
};
use crate::message_fmt::msg_bytes;

pub fn nvim_feedkeys(keys: String_0, mode: String_0, escape_ks: Boolean) {
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
    // Up to the NUL, as upstream's `char *` is read.
    let keys = keys.as_bytes();
    let keys = &keys[..keys.iter().position(|&b| b == 0).unwrap_or(keys.len())];
    let escaped;
    let keys_esc: &[u8] = if escape_ks {
        escaped = crate::keycodes::escape_ks_bytes(keys);
        &escaped
    } else {
        keys
    };
    if lowlevel {
        input_enqueue_raw(keys_esc);
    } else {
        let remap_flag = if remap {
            REMAP_YES as ::core::ffi::c_int
        } else {
            REMAP_NONE as ::core::ffi::c_int
        };
        // `insert` puts the keys at the front of the typeahead; without it
        // they go after whatever is already queued.
        let offset = if insert { 0 } else { typeahead().len() };
        let _ = ins_typebuf(keys_esc, remap_flag, offset, !typed, false);
        if vgetc_busy.get() != 0 {
            typebuf_was_filled.set(true);
        }
    }
    if execute {
        let save_msg_scroll: ::core::ffi::c_int = msg_scroll.get();
        msg_scroll.set(0);
        let busy = (!dangerous).then(|| Depth::of(ex_normal_busy));
        exec_normal(true, lowlevel);
        drop(busy);
        msg_scroll.set(msg_scroll.get() | save_msg_scroll);
    }
}

pub fn nvim_input(channel_id: uint64_t, keys: String_0) -> Integer {
    may_trigger_vim_suspend_resume(false);
    unsafe { input_enqueue(channel_id, keys) as Integer }
}

pub fn nvim_input_mouse(
    button: String_0,
    action: String_0,
    modifier: String_0,
    grid: Integer,
    row: Integer,
    col: Integer,
) -> Result<(), Error> {
    let mut error = Error::none();
    let mut code: ::core::ffi::c_int;
    let mut modmask;
    may_trigger_vim_suspend_resume(false);
    '_error: {
        if !(button.data().is_null() || action.data().is_null()) {
            if button.as_cstr() == c"left" {
                code = KE_LEFTMOUSE as ::core::ffi::c_int;
            } else if button.as_cstr() == c"middle" {
                code = KE_MIDDLEMOUSE as ::core::ffi::c_int;
            } else if button.as_cstr() == c"right" {
                code = KE_RIGHTMOUSE as ::core::ffi::c_int;
            } else if button.as_cstr() == c"wheel" {
                code = KE_MOUSEDOWN as ::core::ffi::c_int;
            } else if button.as_cstr() == c"x1" {
                code = KE_X1MOUSE as ::core::ffi::c_int;
            } else if button.as_cstr() == c"x2" {
                code = KE_X2MOUSE as ::core::ffi::c_int;
            } else if button.as_cstr() == c"move" {
                code = KE_MOUSEMOVE as ::core::ffi::c_int;
            } else {
                break '_error;
            }
            if code == KE_MOUSEDOWN as ::core::ffi::c_int {
                if action.as_cstr() == c"down" {
                    code = KE_MOUSEUP as ::core::ffi::c_int;
                } else if action.as_cstr() != c"up" {
                    if action.as_cstr() == c"left" {
                        code = KE_MOUSERIGHT as ::core::ffi::c_int;
                    } else if action.as_cstr() == c"right" {
                        code = KE_MOUSELEFT as ::core::ffi::c_int;
                    } else {
                        break '_error;
                    }
                }
            } else if code != KE_MOUSEMOVE as ::core::ffi::c_int && action.as_cstr() != c"press" {
                if action.as_cstr() == c"drag" {
                    code += KE_LEFTDRAG as ::core::ffi::c_int - KE_LEFTMOUSE as ::core::ffi::c_int;
                } else if action.as_cstr() == c"release" {
                    code +=
                        KE_LEFTRELEASE as ::core::ffi::c_int - KE_LEFTMOUSE as ::core::ffi::c_int;
                } else {
                    break '_error;
                }
            }
            modmask = ModMask::NONE;
            let mut i: size_t = 0 as size_t;
            while i < modifier.len() {
                let byte: ::core::ffi::c_char = unsafe { *modifier.data().add(i) };
                if byte as ::core::ffi::c_int != '-' as ::core::ffi::c_int {
                    let mod_0 = name_to_mod_mask(byte as ::core::ffi::c_int);
                    if mod_0.is_empty() {
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
                modmask.bits() as uint8_t,
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

pub fn nvim_replace_termcodes(
    str: String_0,
    from_part: Boolean,
    do_lt: Boolean,
    special: Boolean,
) -> String_0 {
    if str.len() == 0 as size_t {
        return String_0::NULL;
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
    // SAFETY: `str` is the caller's, `len` bytes long.
    let text = unsafe { cstr::slice_at(str.data(), str.len()) };
    let replaced = p_cpo(|cpo| replace_termcodes(text, 0 as ScriptId, flags, None, cpo, None));
    let replaced = replaced.unwrap_or_default();
    // What a C reader of the answer saw: up to its first NUL.
    let mut replaced = replaced;
    replaced.truncate(
        replaced
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(replaced.len()),
    );
    String_0::from_xstring(crate::memory::XString::from_vec(replaced))
}

pub fn nvim_get_keymap(mode: String_0) -> Array {
    keymap_array(mode, None)
}

pub fn nvim_set_keymap(
    channel_id: uint64_t,
    mode: String_0,
    lhs: String_0,
    rhs: String_0,
    opts: &mut KeyDict_keymap,
) -> Result<(), Error> {
    // `-1` is the API's "every buffer" spelling of the buffer argument.
    let all_buffers: BufferHandle = -1;
    modify_keymap(channel_id, all_buffers, false, mode, lhs, rhs, Some(opts))
}

pub fn nvim_del_keymap(channel_id: uint64_t, mode: String_0, lhs: String_0) -> Result<(), Error> {
    nvim_buf_del_keymap(channel_id, -1 as BufferHandle, mode, lhs)
}

pub fn nvim_select_popupmenu_item(
    item: Integer,
    mut insert: Boolean,
    finish: Boolean,
    _opts: &mut KeyDict_empty,
) {
    if finish {
        insert = true;
    }
    pum_ext_select_item(item as ::core::ffi::c_int, insert, finish);
}
