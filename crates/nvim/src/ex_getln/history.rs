//! Command-line history: `<Up>`, `<Down>` and the history commands.
//!
//! [`command_line_browse_history`] is the recall itself and
//! [`command_line_next_histidx`] the index walk it drives, matching the
//! typed prefix where `'wildoptions'` asks for it.  [`get_list_range`] parses
//! the `:history` and `:clist` style range arguments.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::keycodes::Ctrl_N;
use crate::types::{FAIL, OK};

/// Step `s->hiscnt` one entry back (or forward, with `next_match`) through
/// the history, skipping entries that do not start with what was typed.
pub(crate) unsafe fn command_line_next_histidx(s: *mut CommandLineState, next_match: bool) {
    unsafe {
        loop {
            if !next_match {
                // One step backwards.
                if (*s).hiscnt == get_hislen() {
                    // first time
                    (*s).hiscnt = get_hisidx((*s).histype);
                } else if (*s).hiscnt == 0 && get_hisidx((*s).histype) != get_hislen() - 1 {
                    (*s).hiscnt = get_hislen() - 1;
                } else if (*s).hiscnt != get_hisidx((*s).histype) + 1 {
                    (*s).hiscnt -= 1;
                } else {
                    // at the top of the list
                    (*s).hiscnt = (*s).save_hiscnt;
                    break;
                }
            } else if (*s).hiscnt == get_hisidx((*s).histype) {
                // On the last entry: clear the line.
                (*s).hiscnt = get_hislen();
                break;
            } else if (*s).hiscnt == get_hislen() {
                // Not on a history line, nothing to do.
                break;
            } else if (*s).hiscnt == get_hislen() - 1 {
                (*s).hiscnt = 0; // wrap around
            } else {
                (*s).hiscnt += 1;
            }

            let Some(entry) = hist_entry_ref((*s).histype, (*s).hiscnt) else {
                (*s).hiscnt = (*s).save_hiscnt;
                break;
            };
            if ((*s).c != K_UP && (*s).c != K_DOWN)
                || (*s).hiscnt == (*s).save_hiscnt
                || strncmp(entry.text, (*s).lookfor, (*s).lookforlen as size_t) == 0
            {
                break;
            }
        }
    }
}

/// Handle Up, Down, PageUp, PageDown, CTRL-N and CTRL-P on the command line.
pub(crate) unsafe fn command_line_browse_history(s: *mut CommandLineState) -> KeyOutcome {
    unsafe {
        let cc = ccline.ptr();
        if (*s).histype == HIST_INVALID || get_hislen() == 0 || (*s).firstc == NUL {
            return KeyOutcome::NotChanged; // no history
        }

        (*s).save_hiscnt = (*s).hiscnt;

        // Save the current command string, so that it can be restored later.
        if (*s).lookfor.is_null() {
            (*s).lookfor = xstrnsave((*cc).cmdbuff, (*cc).cmdlen as size_t);
            *(*s).lookfor.offset((*cc).cmdpos as isize) = NUL as ::core::ffi::c_char;
            (*s).lookforlen = (*cc).cmdpos;
        }

        let next_match = (*s).c == K_DOWN
            || (*s).c == K_S_DOWN
            || (*s).c == Ctrl_N
            || (*s).c == K_PAGEDOWN
            || (*s).c == K_KPAGEDOWN;
        command_line_next_histidx(s, next_match);

        if (*s).hiscnt == (*s).save_hiscnt {
            beep_flush();
            return KeyOutcome::NotChanged;
        }

        // Jumped to another entry.
        let p: *mut ::core::ffi::c_char;
        let plen: ::core::ffi::c_int;
        let mut hist_sep = NUL;

        dealloc_cmdbuff();
        (*s).xpc.xp_context = EXPAND_NOTHING;
        if (*s).hiscnt == get_hislen() {
            p = (*s).lookfor; // back to the old one
            plen = (*s).lookforlen;
        } else {
            let entry =
                hist_entry_ref((*s).histype, (*s).hiscnt).expect("browsed slot is occupied");
            p = entry.text as *mut ::core::ffi::c_char;
            plen = entry.len as ::core::ffi::c_int;
            hist_sep = entry.sep as ::core::ffi::c_int;
        }

        let old_firstc = hist_sep;
        if (*s).histype == HIST_SEARCH && p != (*s).lookfor && old_firstc != (*s).firstc {
            // Correct for the separator character used when the history entry
            // was added versus the one used now. First pass counts the
            // length, second pass copies the characters, and the buffer is
            // allocated in between.
            // A closure rather than a `let`, because upstream only reads
            // `p[j - 1]` when the character before it matched -- keeping the
            // read lazy keeps the evaluation order.
            let unescaped = |j: isize| {
                j == 0 || *p.offset(j - 1) as ::core::ffi::c_int != '\\' as ::core::ffi::c_int
            };
            let mut len = 0;
            for pass in 0..2 {
                len = 0;
                let mut j = 0isize;
                while *p.offset(j) as ::core::ffi::c_int != NUL {
                    if *p.offset(j) as ::core::ffi::c_int == old_firstc && unescaped(j) {
                        // Replace the old separator with the new one, unless
                        // it is escaped.
                        if pass > 0 {
                            *(*cc).cmdbuff.offset(len as isize) =
                                (*s).firstc as ::core::ffi::c_char;
                        }
                    } else {
                        // Escape the new separator, unless it is already
                        // escaped.
                        if *p.offset(j) as ::core::ffi::c_int == (*s).firstc && unescaped(j) {
                            if pass > 0 {
                                *(*cc).cmdbuff.offset(len as isize) = '\\' as ::core::ffi::c_char;
                            }
                            len += 1;
                        }
                        if pass > 0 {
                            *(*cc).cmdbuff.offset(len as isize) = *p.offset(j);
                        }
                    }
                    len += 1;
                    j += 1;
                }

                if pass == 0 {
                    alloc_cmdbuff(len);
                }
            }
            *(*cc).cmdbuff.offset(len as isize) = NUL as ::core::ffi::c_char;
            (*cc).cmdlen = len;
            (*cc).cmdpos = len;
        } else {
            alloc_cmdbuff(plen);
            strcpy((*cc).cmdbuff, p);
            (*cc).cmdlen = plen;
            (*cc).cmdpos = plen;
        }

        redrawcmd();
        KeyOutcome::Changed
    }
}

/// Parse a `[N][,[M]]` range argument, as `:history` and `:clist` take.
///
/// `str` is advanced past what was parsed; `num1` and `num2` are only written
/// when the corresponding number was present.  Answers `FAIL` on a malformed
/// range or one whose numbers do not fit an `int`.
pub unsafe fn get_list_range(
    str: *mut *mut ::core::ffi::c_char,
    num1: *mut ::core::ffi::c_int,
    num2: *mut ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let mut len: ::core::ffi::c_int = 0;
        let mut num: varnumber_T = 0;
        let mut first = false;

        *str = skipwhite(*str);
        if **str as ::core::ffi::c_int == '-' as ::core::ffi::c_int
            || ascii_isdigit(**str as ::core::ffi::c_int)
        {
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
            );
            *str = (*str).offset(len as isize);
            if num > INT_MAX as varnumber_T {
                return FAIL;
            }
            *num1 = num as ::core::ffi::c_int;
            first = true;
        }

        *str = skipwhite(*str);
        if **str as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            // parse "to" part of range
            *str = skipwhite((*str).offset(1));
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
            );
            if len > 0 {
                *str = skipwhite((*str).offset(len as isize));
                if num > INT_MAX as varnumber_T {
                    return FAIL;
                }
                *num2 = num as ::core::ffi::c_int;
            } else if !first {
                return FAIL;
            }
        } else if first {
            // only one number given
            *num2 = *num1;
        }
        OK
    }
}
