//! The `<Cmd>` and `<Lua>` pseudo-keys.
//!
//! A `<Cmd>` mapping puts its command into the typeahead terminated by `<CR>`
//! and [`read_cmd_key_line`] reads it back out for `do_cmdline`; a `<Lua>` key
//! carries a `LuaRef` in decimal that [`map_execute_lua`] calls.

#![forbid(unsafe_code)]

use super::*;
use crate::charset::atoi;
use crate::guard::Keys;
use crate::keycodes::{Key, key_escape, key_unescape};
use crate::lua::executor::nlua_call_ref_reporting;
use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::semsg_multiline;
use crate::types::NUL;
use core::ffi::c_int;

/// Read the command a `<Cmd>` key introduced out of the typeahead.
///
/// The [`CmdKey`](crate::ex_docmd::LineSource::CmdKey) arm of `do_cmdline`'s
/// line source. The command runs to the `<CR>` that must terminate it;
/// `None` means it was aborted.
pub fn read_cmd_key_line() -> Option<XString> {
    let mut line = Vec::<u8>::new();
    let mut c1 = -1;
    let mut cmod = 0;
    let mut aborted = false;

    let unmapped = Keys::unmapped(); // no mapping for these characters
    got_int.set(false);
    while c1 != NUL && !aborted {
        if vgetorpeek(false) == NUL {
            // An incomplete <Cmd> is an error: there is not much the user
            // could do from this state.
            emsg(gettext(e_cmd_mapping_must_end_with_cr));
            aborted = true;
            break;
        }

        // One character at a time, three bytes for a special key.
        c1 = vgetorpeek(true);
        if c1 == K_SPECIAL {
            let second = vgetorpeek(true);
            let third = vgetorpeek(true);
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
        return None;
    }
    Some(XString::from_vec(line))
}

/// Read a `<Lua>` key's `LuaRef` out of the typeahead and call it.
///
/// With `may_repeat` the reference is remembered so that `.` can run it
/// again; with `discard` the keys are read but nothing is called. Answers
/// false only when reading the reference was aborted.
pub fn map_execute_lua(may_repeat: bool, discard: bool) -> bool {
    let mut line = Vec::<u8>::new();
    let mut c1 = -1;
    let mut aborted = false;

    let unmapped = Keys::unmapped();
    got_int.set(false);
    while c1 != NUL && !aborted {
        c1 = vgetorpeek(true);
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

    let luaref: LuaRef = atoi(&line);
    if may_repeat {
        repeat_luaref.set(luaref);
    }

    let called = nlua_call_ref_reporting(luaref, None, ARRAY_DICT_INIT, kRetNilBool);
    if let Err(e) = called {
        let msg = msg_cstr(e.message_or_empty());
        semsg_multiline!(c"emsg", "E5108: {msg}");
    }

    true
}
