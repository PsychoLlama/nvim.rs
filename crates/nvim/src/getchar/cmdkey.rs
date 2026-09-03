//! The `<Cmd>` and `<Lua>` pseudo-keys.
//!
//! A `<Cmd>` mapping puts its command into the typeahead terminated by `<CR>`
//! and [`getcmdkeycmd`] reads it back out for `do_cmdline`; a `<Lua>` key
//! carries a `LuaRef` in decimal that [`map_execute_lua`] calls.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::guard::Keys;
use crate::keycodes::{Key, key_escape, key_unescape};
use crate::memory::handoff::owned_cstr;
use crate::message_fmt::c_str;
use crate::semsg_multiline;
use crate::types::NUL;
use core::ffi::{c_char, c_int, c_void};
use core::ptr;

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
    let mut line = Vec::<u8>::new();
    let mut c1 = -1;
    let mut cmod = 0;
    let mut aborted = false;

    let unmapped = Keys::unmapped(); // no mapping for these characters
    got_int.set(false);
    while c1 != NUL && !aborted {
        // SAFETY (this body): every byte appended comes from the typeahead,
        // which `vgetorpeek` has already validated.
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
        } else if c1 == Key::Command.code() {
            // A nicer error message for this special case.
            emsg(gettext(e_cmd_mapping_must_end_with_cr_before_second_cmd));
            aborted = true;
        } else if c1 == Key::Snr.code() {
            line.extend_from_slice(b"<SNR>");
        } else {
            if cmod != 0 {
                line.push(K_SPECIAL as u8);
                line.push(KS_MODIFIER as u8);
                line.push(cmod as u8);
            }
            if c1 < 0 {
                line.extend(key_escape(c1));
            } else {
                line.push(c1 as u8);
            }
        }

        cmod = 0;
    }
    drop(unmapped);

    if aborted {
        return ptr::null_mut();
    }
    owned_cstr(line)
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
    let mut line = Vec::<u8>::new();
    let mut c1 = -1;
    let mut aborted = false;

    let unmapped = Keys::unmapped();
    got_int.set(false);
    while c1 != NUL && !aborted {
        // SAFETY (this body): the typeahead holds the `<Lua>` key's decimal
        // reference, and `err` is this frame's own slot.
        c1 = unsafe { vgetorpeek(true) };
        if got_int.get() {
            aborted = true;
        } else if c1 == '\r' as c_int || c1 == '\n' as c_int {
            c1 = NUL; // end of the line
        } else {
            line.push(c1 as u8);
        }
    }
    drop(unmapped);

    if aborted || discard {
        return !aborted;
    }

    line.push(NUL as u8);
    // SAFETY: `line` is this frame's own buffer and now NUL-terminated.
    let luaref: LuaRef = unsafe { atoi(line.as_ptr().cast()) };
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

    true
}
