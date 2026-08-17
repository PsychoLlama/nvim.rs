//! The `<Cmd>` and `<Lua>` pseudo-keys.
//!
//! A `<Cmd>` mapping puts its command into the typeahead terminated by `<CR>`
//! and [`getcmdkeycmd`] reads it back out for `do_cmdline`; a `<Lua>` key
//! carries a `LuaRef` in decimal that [`map_execute_lua`] calls.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::{K_COMMAND, K_SNR, key_escape, key_unescape};
use crate::semsg_multiline_c;
use crate::types::kErrorTypeNone;
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
/// This is a `LineGetter`, handed to `do_cmdline`, which is why it keeps the
/// `extern "C"` ABI and the two parameters it does not use. The command runs
/// to the `<CR>` that must terminate it; a null answer means it was aborted.
///
/// # Safety
/// Callable at any time; reads from the typeahead.
pub unsafe extern "C" fn getcmdkeycmd(
    _promptc: c_int,
    _cookie: *mut c_void,
    _indent: c_int,
    _do_concat: bool,
) -> *mut c_char {
    unsafe {
        let mut line_ga = byte_garray();
        let mut c1 = -1;
        let mut cmod = 0;
        let mut aborted = false;

        *no_mapping.ptr() += 1; // no mapping for these characters
        got_int.set(false);
        while c1 != NUL && !aborted {
            ga_grow(&raw mut line_ga, 32);

            if vgetorpeek(false) == NUL {
                // An incomplete <Cmd> is an error: there is not much the user
                // could do from this state.
                emsg(gettext(e_cmd_mapping_must_end_with_cr.as_ptr()));
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
            } else if c1 == K_COMMAND {
                // A nicer error message for this special case.
                emsg(gettext(
                    e_cmd_mapping_must_end_with_cr_before_second_cmd
                        .as_ptr()
                        .cast(),
                ));
                aborted = true;
            } else if c1 == K_SNR {
                ga_concat_len(&raw mut line_ga, c"<SNR>".as_ptr(), 5);
            } else {
                if cmod != 0 {
                    ga_append(&raw mut line_ga, K_SPECIAL as u8);
                    ga_append(&raw mut line_ga, KS_MODIFIER as u8);
                    ga_append(&raw mut line_ga, cmod as u8);
                }
                if c1 < 0 {
                    for byte in key_escape(c1) {
                        ga_append(&raw mut line_ga, byte);
                    }
                } else {
                    ga_append(&raw mut line_ga, c1 as u8);
                }
            }

            cmod = 0;
        }
        *no_mapping.ptr() -= 1;

        if aborted {
            ga_clear(&raw mut line_ga);
        }
        line_ga.ga_data.cast()
    }
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
    unsafe {
        let mut line_ga = byte_garray();
        let mut c1 = -1;
        let mut aborted = false;

        *no_mapping.ptr() += 1;
        got_int.set(false);
        while c1 != NUL && !aborted {
            ga_grow(&raw mut line_ga, 32);
            c1 = vgetorpeek(true);
            if got_int.get() {
                aborted = true;
            } else if c1 == '\r' as c_int || c1 == '\n' as c_int {
                c1 = NUL; // end of the line
            } else {
                ga_append(&raw mut line_ga, c1 as u8);
            }
        }
        *no_mapping.ptr() -= 1;

        if aborted || discard {
            ga_clear(&raw mut line_ga);
            return !aborted;
        }

        let luaref: LuaRef = atoi(line_ga.ga_data.cast());
        if may_repeat {
            repeat_luaref.set(luaref);
        }

        let mut err = Error {
            type_0: kErrorTypeNone,
            msg: ptr::null_mut(),
        };
        nlua_call_ref(
            luaref,
            ptr::null(),
            ARRAY_DICT_INIT,
            kRetNilBool,
            ptr::null_mut(),
            &raw mut err,
        );
        if err.type_0 != kErrorTypeNone {
            semsg_multiline_c!(c"emsg".as_ptr(), c"E5108: %s".as_ptr(), err.msg);
            api_clear_error(&raw mut err);
        }

        ga_clear(&raw mut line_ga);
        true
    }
}
