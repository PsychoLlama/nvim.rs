//! The `<Cmd>` and `<Lua>` pseudo-keys.
//!
//! A `<Cmd>` mapping puts its command into the typeahead terminated by `<CR>`
//! and [`read_cmd_key_line`] reads it back out for `do_cmdline`; a `<Lua>` key
//! carries a `LuaRef` in decimal that [`map_execute_lua`] calls.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;
use crate::charset::atoi;
use crate::guard::Keys;
use crate::keycodes::{SpecialKey, key_escape, key_unescape};
use crate::lua::executor::nlua_call_ref_reporting;
use crate::memory::XString;
use crate::message_fmt::msg_cstr;
use crate::semsg_multiline;
use crate::types::Key;

/// Read the command a `<Cmd>` key introduced out of the typeahead.
///
/// The [`CmdKey`](crate::ex_docmd::LineSource::CmdKey) arm of `do_cmdline`'s
/// line source. The command runs to the `<CR>` that must terminate it;
/// `None` means it was aborted.
pub fn read_cmd_key_line() -> Option<XString> {
    let mut line = Vec::<u8>::new();
    let mut cmod = 0;
    let mut aborted = false;

    let unmapped = Keys::unmapped(); // no mapping for these characters
    got_int.set(false);
    while !aborted {
        if vgetorpeek(false) == 0 {
            // An incomplete <Cmd> is an error: there is not much the user
            // could do from this state.
            emsg(gettext(e_cmd_mapping_must_end_with_cr));
            aborted = true;
            break;
        }

        // One character at a time, three bytes for a special key.
        let lead = vgetorpeek(true);
        let c1 = if lead == K_SPECIAL {
            let second = vgetorpeek(true);
            let third = vgetorpeek(true);
            if second == KS_MODIFIER {
                cmod = third;
                continue;
            }
            key_unescape(second, third)
        } else {
            Key::byte(lead)
        };

        if got_int.get() {
            aborted = true;
        } else if c1 == Key::CAR || c1 == Key::NL {
            break; // end of the line
        } else if c1 == Key::ESC {
            aborted = true;
        } else if c1 == SpecialKey::Command {
            // A nicer error message for this special case.
            emsg(gettext(e_cmd_mapping_must_end_with_cr_before_second_cmd));
            aborted = true;
        } else if c1 == SpecialKey::Snr {
            line.extend_from_slice(b"<SNR>");
        } else {
            if cmod != 0 {
                line.extend_from_slice(&[K_SPECIAL, KS_MODIFIER, cmod]);
            }
            match c1.as_byte() {
                // A byte, or a K_SPECIAL its escape stood for.
                Some(byte) => line.push(byte),
                None => line.extend(key_escape(c1)),
            }
        }
        // A NUL went in like any byte, and ends the line after it.
        if c1.is_nul() {
            break;
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
    let mut aborted = false;

    let unmapped = Keys::unmapped();
    got_int.set(false);
    while !aborted {
        let c1 = vgetorpeek(true);
        if got_int.get() {
            aborted = true;
        } else if c1 == b'\r' || c1 == b'\n' {
            break; // end of the line
        } else {
            line.push(c1);
            // A NUL went in like any byte, and ends the line after it.
            if c1 == 0 {
                break;
            }
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

    let called = nlua_call_ref_reporting(luaref, None, Array::new(), kRetNilBool);
    if let Err(e) = called {
        let msg = msg_cstr(e.message_or_empty());
        semsg_multiline!(c"emsg", "E5108: {msg}");
    }

    true
}
