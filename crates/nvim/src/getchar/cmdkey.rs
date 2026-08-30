//! The `<Cmd>` and `<Lua>` pseudo-keys.
//!
//! A `<Cmd>` mapping puts its command into the typeahead terminated by `<CR>`
//! and [`getcmdkeycmd`] reads it back out for `do_cmdline`; a `<Lua>` key
//! carries a `LuaRef` in decimal that [`map_execute_lua`] calls.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::Keys;
use crate::keycodes::{K_COMMAND, K_SNR, key_escape, key_unescape};
use crate::message_fmt::c_str;
use crate::semsg_multiline;
use crate::types::NUL;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

/// A fresh byte-sized growable array, upstream's `GA_INIT(1, 32)`.
fn byte_garray() -> garray_T {
    garray_T {
        ga_len: 0,
        ga_maxlen: 0,
        ga_itemsize: 1,
        ga_growsize: 32,
        ga_data: ptr::null_mut(),
    }
}

/// Read the command a `<Cmd>` key introduced out of the typeahead.
///
/// This is a `LineGetter`, handed to `do_cmdline`, which is why it keeps that
/// signature and the two parameters it does not use. The command runs
/// to the `<CR>` that must terminate it; a null answer means it was aborted.
///
/// # Safety
/// Callable at any time; reads from the typeahead.
pub unsafe fn getcmdkeycmd(
    _promptc: c_int,
    _cookie: *mut c_void,
    _indent: c_int,
    _do_concat: bool,
) -> *mut c_char {
    let mut line_ga = byte_garray();
    let mut c1 = -1;
    let mut cmod = 0;
    let mut aborted = false;

    let unmapped = Keys::unmapped(); // no mapping for these characters
    got_int.set(false);
    while c1 != NUL && !aborted {
        // SAFETY (this body): `line_ga` is this frame's own growarray, and
        // every byte written into it comes from the typeahead, which
        // `vgetorpeek` has already validated.
        unsafe { ga_grow(&raw mut line_ga, 32) };

        if unsafe { vgetorpeek(false) } == NUL {
            // An incomplete <Cmd> is an error: there is not much the user
            // could do from this state.
            emsg(gettext(e_cmd_mapping_must_end_with_cr));
            aborted = true;
            break;
        }

        // One character at a time, three bytes for a special key.
        c1 = unsafe { vgetorpeek(true) };
        if c1 == K_SPECIAL {
            let second = unsafe { vgetorpeek(true) };
            let third = unsafe { vgetorpeek(true) };
            if second == KS_MODIFIER {
                cmod = third;
                continue;
            }
            c1 = key_unescape(second as u8, third as u8);
        }

        if got_int.get() {
            aborted = true;
        } else if c1 == '\r' as c_int || c1 == '\n' as c_int {
            c1 = NUL; // end of the line
        } else if c1 == ESC {
            aborted = true;
        } else if c1 == K_COMMAND {
            // A nicer error message for this special case.
            emsg(gettext(e_cmd_mapping_must_end_with_cr_before_second_cmd));
            aborted = true;
        } else if c1 == K_SNR {
            unsafe { ga_concat_len(&raw mut line_ga, c"<SNR>".as_ptr(), 5) };
        } else {
            if cmod != 0 {
                unsafe { ga_append(&raw mut line_ga, K_SPECIAL as u8) };
                unsafe { ga_append(&raw mut line_ga, KS_MODIFIER as u8) };
                unsafe { ga_append(&raw mut line_ga, cmod as u8) };
            }
            if c1 < 0 {
                for byte in key_escape(c1) {
                    unsafe { ga_append(&raw mut line_ga, byte) };
                }
            } else {
                unsafe { ga_append(&raw mut line_ga, c1 as u8) };
            }
        }

        cmod = 0;
    }
    drop(unmapped);

    if aborted {
        unsafe { ga_clear(&raw mut line_ga) };
    }
    line_ga.ga_data.cast()
}

/// Read a `<Lua>` key's `LuaRef` out of the typeahead and call it.
///
/// With `may_repeat` the reference is remembered so that `.` can run it
/// again; with `discard` the keys are read but nothing is called. Answers
/// false only when reading the reference was aborted.
///
/// # Safety
/// Callable at any time; reads from the typeahead.
pub unsafe fn map_execute_lua(may_repeat: bool, discard: bool) -> bool {
    let mut line_ga = byte_garray();
    let mut c1 = -1;
    let mut aborted = false;

    let unmapped = Keys::unmapped();
    got_int.set(false);
    while c1 != NUL && !aborted {
        // SAFETY (this body): the typeahead holds the `<Lua>` key's decimal
        // reference, NUL-terminated by construction, and `err` is this frame's
        // own slot.
        unsafe { ga_grow(&raw mut line_ga, 32) };
        c1 = unsafe { vgetorpeek(true) };
        if got_int.get() {
            aborted = true;
        } else if c1 == '\r' as c_int || c1 == '\n' as c_int {
            c1 = NUL; // end of the line
        } else {
            unsafe { ga_append(&raw mut line_ga, c1 as u8) };
        }
    }
    drop(unmapped);

    if aborted || discard {
        unsafe { ga_clear(&raw mut line_ga) };
        return !aborted;
    }

    let luaref: LuaRef = unsafe { atoi(line_ga.ga_data.cast()) };
    if may_repeat {
        repeat_luaref.set(luaref);
    }

    let mut err = Error::none();
    unsafe {
        nlua_call_ref(
            luaref,
            ptr::null(),
            ARRAY_DICT_INIT,
            kRetNilBool,
            ptr::null_mut(),
            &mut err,
        )
    };
    if err.is_set() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let msg = unsafe { c_str(err.message_or_empty().as_ptr()) };
        semsg_multiline!(c"emsg", "E5108: {msg}");
        err.clear();
    }

    unsafe { ga_clear(&raw mut line_ga) };
    true
}
