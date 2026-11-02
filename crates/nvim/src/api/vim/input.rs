//! Feeding the editor keys, and the mappings they may hit.
//!
//! `nvim_feedkeys` and `nvim_input` are the two ends of the input path: one
//! goes through the typeahead buffer with the caller's mode flags, the
//! other straight into it as if typed.  `nvim_input_mouse` synthesises a
//! mouse event from (button, action, grid, row, col).  The keymap
//! accessors sit here because they answer for the same table `nvim_input`
//! is resolved against.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::api_error;
use crate::getchar::typeahead;
use crate::guard::Depth;
use crate::keycodes::ModMask;
use crate::keycodes::{
    KE_LEFTDRAG, KE_LEFTMOUSE, KE_LEFTRELEASE, KE_MIDDLEMOUSE, KE_MOUSEDOWN, KE_MOUSELEFT,
    KE_MOUSEMOVE, KE_MOUSERIGHT, KE_MOUSEUP, KE_RIGHTMOUSE, KE_X1MOUSE, KE_X2MOUSE,
};
use crate::message_fmt::msg_bytes;
use crate::narrow::{len_as_integer, number_as_int};

pub fn nvim_feedkeys(keys: ApiString, mode: ApiString, escape_ks: Boolean) {
    let mut remap: bool = true;
    let mut insert: bool = false;
    let mut typed: bool = false;
    let mut execute: bool = false;
    let mut dangerous: bool = false;
    let mut lowlevel: bool = false;
    let mut i: size_t = 0 as size_t;
    while i < mode.len() {
        match ::core::ffi::c_int::from(mode.as_bytes()[i]) {
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

pub fn nvim_input(channel_id: uint64_t, keys: ApiString) -> Integer {
    may_trigger_vim_suspend_resume(false);
    len_as_integer(input_enqueue(channel_id, keys))
}

pub fn nvim_input_mouse(
    button: ApiString,
    action: ApiString,
    modifier: ApiString,
    grid: Integer,
    row: Integer,
    col: Integer,
) -> Result<(), Error> {
    let invalid = || Error::validation(c"invalid button or action");
    may_trigger_vim_suspend_resume(false);
    if button.data().is_null() || action.data().is_null() {
        return Err(invalid());
    }
    let mut code = match button.as_cstr().to_bytes() {
        b"left" => KE_LEFTMOUSE,
        b"middle" => KE_MIDDLEMOUSE,
        b"right" => KE_RIGHTMOUSE,
        b"wheel" => KE_MOUSEDOWN,
        b"x1" => KE_X1MOUSE,
        b"x2" => KE_X2MOUSE,
        b"move" => KE_MOUSEMOVE,
        _ => return Err(invalid()),
    }
    .cast_signed();
    let action = action.as_cstr().to_bytes();
    if code == KE_MOUSEDOWN.cast_signed() {
        // The wheel's "action" is its direction.
        code = match action {
            b"down" => KE_MOUSEUP,
            b"up" => KE_MOUSEDOWN,
            b"left" => KE_MOUSERIGHT,
            b"right" => KE_MOUSELEFT,
            _ => return Err(invalid()),
        }
        .cast_signed();
    } else if code != KE_MOUSEMOVE.cast_signed() && action != b"press" {
        code += match action {
            b"drag" => KE_LEFTDRAG.cast_signed() - KE_LEFTMOUSE.cast_signed(),
            b"release" => KE_LEFTRELEASE.cast_signed() - KE_LEFTMOUSE.cast_signed(),
            _ => return Err(invalid()),
        };
    }
    let mut modmask = ModMask::NONE;
    for &raw in modifier.as_bytes() {
        let byte = ::core::ffi::c_char::from_ne_bytes([raw]);
        if raw != b'-' {
            let mod_0 = name_to_mod_mask(::core::ffi::c_int::from(byte));
            if mod_0.is_empty() {
                // `%c` wrote the one byte, whatever it was; the adaptor keeps
                // it rather than widening it to a char.
                let byte = msg_bytes(core::slice::from_ref(&raw));
                return Err(api_error!(kErrorTypeValidation, "Invalid modifier: {byte}"));
            }
            modmask |= mod_0;
        }
    }
    input_enqueue_mouse(
        code,
        u8::try_from(modmask.bits()).expect("the modifiers fit a byte"),
        number_as_int(grid),
        number_as_int(row),
        number_as_int(col),
    );
    Ok(())
}

pub fn nvim_replace_termcodes(
    str: ApiString,
    from_part: Boolean,
    do_lt: Boolean,
    special: Boolean,
) -> ApiString {
    if str.len() == 0 as size_t {
        return ApiString::NULL;
    }
    let mut flags: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if from_part {
        flags |= REPTERM_FROM_PART.cast_signed();
    }
    if do_lt {
        flags |= REPTERM_DO_LT.cast_signed();
    }
    if !special {
        flags |= REPTERM_NO_SPECIAL.cast_signed();
    }
    let text = str.as_bytes();
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
    ApiString::from_xstring(crate::memory::XString::from_vec(replaced))
}

pub fn nvim_get_keymap(mode: ApiString) -> Array {
    keymap_array(mode, None)
}

pub fn nvim_set_keymap(
    channel_id: uint64_t,
    mode: ApiString,
    lhs: ApiString,
    rhs: ApiString,
    opts: &mut KeyDict_keymap,
) -> Result<(), Error> {
    // `-1` is the API's "every buffer" spelling of the buffer argument.
    let all_buffers: BufferHandle = Handle::new(-1);
    modify_keymap(channel_id, all_buffers, false, mode, lhs, rhs, Some(opts))
}

pub fn nvim_del_keymap(channel_id: uint64_t, mode: ApiString, lhs: ApiString) -> Result<(), Error> {
    nvim_buf_del_keymap(channel_id, Handle::new(-1), mode, lhs)
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
    pum_ext_select_item(number_as_int(item), insert, finish);
}
