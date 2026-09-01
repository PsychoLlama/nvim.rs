//! Command-line history: `<Up>`, `<Down>` and the history commands.
//!
//! [`command_line_browse_history`] is the recall itself and
//! [`command_line_next_histidx`] the index walk it drives, matching the
//! typed prefix where `'wildoptions'` asks for it.  [`get_list_range`] parses
//! the `:history` and `:clist` style range arguments.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::keycodes::{Ctrl_N, Key};
use crate::types::{ExpandContext, Failed, NUL};

/// Step `s->hiscnt` one entry back (or forward, with `next_match`) through
/// the history, skipping entries that do not start with what was typed.
pub(crate) unsafe fn command_line_next_histidx(s: *mut CommandLineState, next_match: bool) {
    loop {
        if !next_match {
            // One step backwards.
            if unsafe { (*s).hiscnt } == get_hislen() {
                // first time
                unsafe { (*s).hiscnt = get_hisidx((*s).histype) };
            } else if unsafe { (*s).hiscnt } == 0
                && get_hisidx(unsafe { (*s).histype }) != get_hislen() - 1
            {
                unsafe { (*s).hiscnt = get_hislen() - 1 };
            } else if unsafe { (*s).hiscnt } != get_hisidx(unsafe { (*s).histype }) + 1 {
                unsafe { (*s).hiscnt -= 1 };
            } else {
                // at the top of the list
                unsafe { (*s).hiscnt = (*s).save_hiscnt };
                break;
            }
        } else if unsafe { (*s).hiscnt } == get_hisidx(unsafe { (*s).histype }) {
            // On the last entry: clear the line.
            unsafe { (*s).hiscnt = get_hislen() };
            break;
        } else if unsafe { (*s).hiscnt } == get_hislen() {
            // Not on a history line, nothing to do.
            break;
        } else if unsafe { (*s).hiscnt } == get_hislen() - 1 {
            unsafe { (*s).hiscnt = 0 }; // wrap around
        } else {
            unsafe { (*s).hiscnt += 1 };
        }

        let Some(entry) = hist_entry_ref(unsafe { (*s).histype }, unsafe { (*s).hiscnt }) else {
            unsafe { (*s).hiscnt = (*s).save_hiscnt };
            break;
        };
        if (unsafe { (*s).c } != Key::Up.code() && unsafe { (*s).c } != Key::Down.code())
            || unsafe { (*s).hiscnt } == unsafe { (*s).save_hiscnt }
            || unsafe { cstr::prefix_eq(entry.text, (*s).lookfor, (*s).lookforlen as size_t) }
        {
            break;
        }
    }
}

/// Handle Up, Down, PageUp, PageDown, CTRL-N and CTRL-P on the command line.
pub(crate) unsafe fn command_line_browse_history(s: *mut CommandLineState) -> KeyOutcome {
    let mut cc = Cc::current();
    if unsafe { (*s).histype } == HIST_INVALID || get_hislen() == 0 || unsafe { (*s).firstc } == NUL
    {
        return KeyOutcome::NotChanged; // no history
    }

    unsafe { (*s).save_hiscnt = (*s).hiscnt };

    // Save the current command string, so that it can be restored later.
    if unsafe { (*s).lookfor }.is_null() {
        unsafe { (*s).lookfor = xstrnsave(cc.text(), cc.len() as size_t) };
        unsafe { *(*s).lookfor.offset(cc.cmdpos as isize) = NUL as ::core::ffi::c_char };
        unsafe { (*s).lookforlen = cc.cmdpos };
    }

    let next_match = unsafe { (*s).c } == Key::Down.code()
        || unsafe { (*s).c } == Key::SDown.code()
        || unsafe { (*s).c } == Ctrl_N
        || unsafe { (*s).c } == Key::Pagedown.code()
        || unsafe { (*s).c } == Key::Kpagedown.code();
    unsafe { command_line_next_histidx(s, next_match) };

    if unsafe { (*s).hiscnt } == unsafe { (*s).save_hiscnt } {
        beep_flush();
        return KeyOutcome::NotChanged;
    }

    // Jumped to another entry.
    let p: *mut ::core::ffi::c_char;
    let plen: ::core::ffi::c_int;
    let mut hist_sep = NUL;

    dealloc_cmdbuff();
    unsafe { (*s).xpc.xp_context = ExpandContext::Nothing };
    if unsafe { (*s).hiscnt } == get_hislen() {
        p = unsafe { (*s).lookfor }; // back to the old one
        plen = unsafe { (*s).lookforlen };
    } else {
        let entry = hist_entry_ref(unsafe { (*s).histype }, unsafe { (*s).hiscnt })
            .expect("browsed slot is occupied");
        p = entry.text as *mut ::core::ffi::c_char;
        plen = entry.len as ::core::ffi::c_int;
        hist_sep = entry.sep as ::core::ffi::c_int;
    }

    let old_firstc = hist_sep;
    if unsafe { (*s).histype } == HIST_SEARCH
        && p != unsafe { (*s).lookfor }
        && old_firstc != unsafe { (*s).firstc }
    {
        // Correct for the separator character used when the history entry
        // was added versus the one used now. First pass counts the
        // length, second pass copies the characters, and the buffer is
        // allocated in between.
        // A closure rather than a `let`, because upstream only reads
        // `p[j - 1]` when the character before it matched -- keeping the
        // read lazy keeps the evaluation order.
        let unescaped = |j: isize| {
            j == 0
                || unsafe { *p.offset(j - 1) } as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
        };
        let mut len = 0;
        for pass in 0..2 {
            len = 0;
            let mut j = 0isize;
            while unsafe { *p.offset(j) } as ::core::ffi::c_int != NUL {
                if unsafe { *p.offset(j) } as ::core::ffi::c_int == old_firstc && unescaped(j) {
                    // Replace the old separator with the new one, unless
                    // it is escaped.
                    if pass > 0 {
                        unsafe { *cc.at(len) = (*s).firstc as ::core::ffi::c_char };
                    }
                } else {
                    // Escape the new separator, unless it is already
                    // escaped.
                    if unsafe { *p.offset(j) } as ::core::ffi::c_int == unsafe { (*s).firstc }
                        && unescaped(j)
                    {
                        if pass > 0 {
                            unsafe { *cc.at(len) = '\\' as ::core::ffi::c_char };
                        }
                        len += 1;
                    }
                    if pass > 0 {
                        unsafe { *cc.at(len) = *p.offset(j) };
                    }
                }
                len += 1;
                j += 1;
            }

            if pass == 0 {
                cc.open(len);
            }
        }
        unsafe { *cc.at(len) = NUL as ::core::ffi::c_char };
        cc.set_len(len);
        cc.cmdpos = len;
    } else {
        cc.open(plen);
        unsafe { strcpy(cc.text(), p) };
        cc.set_len(plen);
        cc.cmdpos = plen;
    }

    unsafe { redrawcmd() };
    KeyOutcome::Changed
}

/// Parse a `[N][,[M]]` range argument, as `:history` and `:clist` take.
///
/// `str` is advanced past what was parsed; `num1` and `num2` are only written
/// when the corresponding number was present.  Answers `Err` on a malformed
/// range or one whose numbers do not fit an `int`.
pub unsafe fn get_list_range(
    str: *mut *mut ::core::ffi::c_char,
    num1: *mut ::core::ffi::c_int,
    num2: *mut ::core::ffi::c_int,
) -> Result<(), Failed> {
    let mut len: ::core::ffi::c_int = 0;
    let mut num: varnumber_T = 0;
    let mut first = false;

    unsafe { *str = skipwhite(*str) };
    if unsafe { **str } as ::core::ffi::c_int == '-' as ::core::ffi::c_int
        || ascii_isdigit(unsafe { **str } as ::core::ffi::c_int)
    {
        unsafe {
            vim_str2nr(
                *str,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                &raw mut len,
                0,
                &raw mut num,
                ::core::ptr::null_mut::<uvarnumber_T>(),
                0,
                false,
                ::core::ptr::null_mut::<bool>(),
            )
        };
        unsafe { *str = (*str).offset(len as isize) };
        if num > INT_MAX as varnumber_T {
            return Err(Failed);
        }
        unsafe { *num1 = num as ::core::ffi::c_int };
        first = true;
    }

    unsafe { *str = skipwhite(*str) };
    if unsafe { **str } as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
        // parse "to" part of range
        unsafe { *str = skipwhite((*str).offset(1)) };
        unsafe {
            vim_str2nr(
                *str,
                ::core::ptr::null_mut::<::core::ffi::c_int>(),
                &raw mut len,
                0,
                &raw mut num,
                ::core::ptr::null_mut::<uvarnumber_T>(),
                0,
                false,
                ::core::ptr::null_mut::<bool>(),
            )
        };
        if len > 0 {
            unsafe { *str = skipwhite((*str).offset(len as isize)) };
            if num > INT_MAX as varnumber_T {
                return Err(Failed);
            }
            unsafe { *num2 = num as ::core::ffi::c_int };
        } else if !first {
            return Err(Failed);
        }
    } else if first {
        // only one number given
        unsafe { *num2 = *num1 };
    }
    Ok(())
}
