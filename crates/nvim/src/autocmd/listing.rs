//! Printing autocommands back, and asking whether one exists.
//!
//! [`au_show_for_event`] is `:autocmd`'s listing for one event -- the group
//! header, the pattern column, the command, and the `Last set from` line a
//! `:verbose` listing adds.  [`au_exists`] answers
//! `exists('#Group#Event#pat')` in all four of its shapes, [`has_autocmd`]
//! the pattern query behind it, and [`set_context_in_autocmd`] is
//! command-line completion.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::cstr;
use crate::types::ExpandContext;
use crate::winlayer::Buf;

/// `:autocmd` with no event: list every event's autocommands.
pub(crate) unsafe fn au_show_for_all_events(
    group: ::core::ffi::c_int,
    pat: *const ::core::ffi::c_char,
) {
    for event in 0..NUM_EVENTS {
        // SAFETY: `group` and `pat` are the caller's, handed straight on.
        unsafe { au_show_for_event(group, event, pat) };
    }
}

/// List the autocommands defined for `event`, restricted to `group` and to
/// the comma-separated pattern list `pat` when either is given.
///
/// `got_int` is checked between every write: a listing is interruptible,
/// and each check is at the point where upstream put it.
pub(crate) unsafe fn au_show_for_event(
    group: ::core::ffi::c_int,
    event: event_T,
    mut pat: *const ::core::ffi::c_char,
) {
    let acs = au_event_vec(event);
    // SAFETY: `acs` is the event table's own row for `event`, which lives as
    // long as the editor does.  Its `size` and `items` are re-read at every
    // use because printing can run Lua, which can define an autocommand and
    // reallocate the vector.
    if unsafe { (*acs).size } == 0 {
        return;
    }

    // An empty pattern shows every autocommand for the event.
    let mut patlen: ::core::ffi::c_int = 0;
    // SAFETY: `pat` is the caller's NUL-terminated pattern list, and
    // `aucmd_span_pattern` only ever steps within it.
    if unsafe { *pat } != 0 {
        patlen = unsafe { aucmd_span_pattern(pat, &raw mut pat) } as ::core::ffi::c_int;
        // Only commas: nothing to show.
        if patlen == 0 {
            return;
        }
    }

    let mut buflocal_pat = [0 as ::core::ffi::c_char; BUFLOCAL_PAT_LEN as usize];
    let mut last_group = AUGROUP_ERROR;
    let mut last_group_name: *const ::core::ffi::c_char = ::core::ptr::null();

    // One pass per pattern in the list.
    loop {
        let mut last_ap: *mut AutoPat = ::core::ptr::null_mut();
        // SAFETY: `patlen` is the length just measured inside `pat`, so the
        // end of that item is in bounds of the same string.
        let endpat = unsafe { pat.offset(patlen as isize) };

        // `<buffer[=X]>` is normalised, as it was when it was defined,
        // so the comparison below can be a plain `strncmp`.
        //
        // SAFETY: `pat`/`patlen` name that many readable bytes, and
        // `buflocal_pat` is the `BUFLOCAL_PAT_LEN` buffer the normalised
        // form is sized for.
        if unsafe { aupat_is_buflocal(pat, patlen) } {
            unsafe {
                aupat_normalize_buflocal_pat(
                    buflocal_pat.as_mut_ptr(),
                    pat,
                    patlen,
                    aupat_get_buflocal_nr(pat, patlen),
                );
            };
            pat = buflocal_pat.as_ptr();
            // SAFETY: the normalised pattern is NUL-terminated.
            patlen = unsafe { strlen(buflocal_pat.as_ptr()) } as ::core::ffi::c_int;
        }

        let mut i: usize = 0;
        // SAFETY: the size is re-read every round, so `i` is below the
        // vector's current length and `items.add(i)` is one of its rows.
        while i < unsafe { (*acs).size } {
            let ac = unsafe { (*acs).items.add(i) };
            i = i.wrapping_add(1);

            // SAFETY: `ac` is a row of the vector.
            let ap = unsafe { (*ac).pat };
            // Skip a row `aucmd_del` has marked.
            if ap.is_null() {
                continue;
            }

            // Accept the row when the group matches (or none was asked
            // for) and the pattern matches (or none was asked for).
            //
            // SAFETY: `ap` is non-null, so it is a pattern the vector owns,
            // and `(*ap).pat`/`(*ap).patlen` are its text and that text's
            // length -- as `pat`/`patlen` are of the one being looked for.
            if group != AUGROUP_ALL && unsafe { (*ap).group } != group {
                continue;
            }
            if patlen != 0
                && (unsafe { (*ap).patlen } != patlen
                    || !unsafe { cstr::prefix_eq(pat, (*ap).pat, patlen as size_t) })
            {
                continue;
            }

            // The group and event headline only when the group changed.
            //
            // SAFETY: `ap` is live, as above; `augroup_name` and the `msg_*`
            // writers each take a NUL-terminated string, which is what every
            // name handed to them here is.
            if unsafe { (*ap).group } != last_group {
                last_group = unsafe { (*ap).group };
                last_group_name = unsafe { augroup_name((*ap).group) };

                if got_int.get() {
                    return;
                }
                unsafe { msg_putchar('\n' as ::core::ffi::c_int) };
                if got_int.get() {
                    return;
                }
                if unsafe { (*ap).group } != AUGROUP_DEFAULT {
                    // A group whose name is gone is one `:augroup!`
                    // renamed out from under its autocommands.
                    if last_group_name.is_null() {
                        unsafe { msg_puts_hl(get_deleted_augroup(), HLF_E, false) };
                    } else {
                        unsafe { msg_puts_hl(last_group_name, HLF_T, false) };
                    }
                    unsafe { msg_puts(c"  ".as_ptr()) };
                }
                unsafe { msg_puts_hl(event_nr2name(event), HLF_T, false) };
            }

            // The pattern only when it changed.
            if last_ap != ap {
                last_ap = ap;
                unsafe { msg_putchar('\n' as ::core::ffi::c_int) };
                if got_int.get() {
                    return;
                }
                unsafe { msg_advance(4) };
                // SAFETY: the pattern's own NUL-terminated text.
                unsafe { msg_outtrans((*ap).pat, 0, false) };
            }

            if got_int.get() {
                return;
            }
            if msg_col.get() >= 14 {
                unsafe { msg_putchar('\n' as ::core::ffi::c_int) };
            }
            unsafe { msg_advance(14) };
            if got_int.get() {
                return;
            }

            // SAFETY: `ac` is a live row, and `aucmd_handler_to_string`
            // answers an allocated NUL-terminated string this owns and frees
            // below.
            let handler_str = unsafe { aucmd_handler_to_string(ac) };
            if unsafe { (*ac).desc.is_null() } {
                // A command is transliterated, a callback is not.
                if unsafe { (*ac).handler_cmd.is_null() } {
                    unsafe { msg_puts_hl(handler_str, HLF_8, false) };
                } else {
                    unsafe { msg_outtrans(handler_str, 0, false) };
                }
            } else {
                let msglen: size_t = 100;
                // SAFETY: `msg` is `msglen` writable bytes, which is the
                // size `snprintf` is told it has; `desc` and `handler_str`
                // are NUL-terminated, as `%s` wants.
                let msg = unsafe { xmallocz(msglen) }.cast::<::core::ffi::c_char>();
                if unsafe { (*ac).handler_cmd.is_null() } {
                    unsafe { msg_puts_hl(handler_str, HLF_8, false) };
                    unsafe { snprintf(msg, msglen, c" [%s]".as_ptr(), (*ac).desc) };
                } else {
                    unsafe { snprintf(msg, msglen, c"%s [%s]".as_ptr(), handler_str, (*ac).desc) };
                }
                unsafe { msg_outtrans(msg, 0, false) };
                unsafe { xfree(msg.cast::<::core::ffi::c_void>()) };
            }
            unsafe { xfree(handler_str.cast::<::core::ffi::c_void>()) };

            if p_verbose.get() > 0 {
                // SAFETY: the row's own script context.
                unsafe { last_set_msg((*ac).script_ctx) };
            }
            if got_int.get() {
                return;
            }
        }

        // SAFETY: `endpat` is inside `pat`'s string, at the end of the item
        // just shown, so what follows is the rest of the list.
        patlen = unsafe { aucmd_span_pattern(endpat, &raw mut pat) } as ::core::ffi::c_int;
        if patlen == 0 {
            break;
        }
    }
}

/// Whether any autocommand for `event` would match the file `sfname`
/// opened in `buf`.
pub unsafe fn has_autocmd(
    event: event_T,
    sfname: *mut ::core::ffi::c_char,
    buf: Option<Buf>,
) -> bool {
    // SAFETY: `sfname` is the caller's NUL-terminated file name.  `path_tail`
    // answers a position inside it, and `full_name_save` an allocation this
    // owns and frees below (or null, when it could not make one).
    let tail = unsafe { path_tail(sfname) };
    let fname = unsafe { full_name_save(sfname, false) };
    if fname.is_null() {
        return false;
    }

    let acs = au_event_vec(event);
    let mut retval = false;
    let mut i: usize = 0;
    // SAFETY: the event table's own row for `event`, and `i` is below the
    // size just read, so `items.add(i)` is one of its rows.
    while i < unsafe { (*acs).size } {
        let ap = unsafe { (*(*acs).items.add(i)).pat };
        if !ap.is_null() {
            // A buffer-local pattern is matched by buffer number, every
            // other one against the file name.
            //
            // SAFETY: `ap` is non-null, so it is a pattern the vector owns;
            // its `reg_prog` is the cached program `match_file_pat` may
            // compile into, and `buf` is only read when there is one.
            let matched = if unsafe { (*ap).buflocal_nr } == 0 {
                unsafe {
                    match_file_pat(
                        ::core::ptr::null_mut(),
                        &raw mut (*ap).reg_prog,
                        fname,
                        sfname,
                        tail,
                        (*ap).allow_dirs as ::core::ffi::c_int,
                    )
                }
            } else {
                buf.is_some_and(|b| unsafe { (*ap).buflocal_nr } == b.handle)
            };
            if matched {
                retval = true;
                break;
            }
        }
        i = i.wrapping_add(1);
    }

    // SAFETY: `full_name_save` allocated it and nothing else took it.
    unsafe { xfree(fname.cast::<::core::ffi::c_void>()) };
    retval
}

/// Command-line completion for `:autocmd` (`doautocmd` false) and
/// `:doautocmd`/`:doautoall` (true).
///
/// Answers a pointer at the next command to expand instead, or null when
/// it has set `xp` itself.
pub unsafe fn set_context_in_autocmd(
    xp: *mut expand_T,
    mut arg: *mut ::core::ffi::c_char,
    doautocmd: bool,
) -> *mut ::core::ffi::c_char {
    // Skip a group name if there is one.
    autocmd_include_groups.set(false);
    let start = arg;
    // SAFETY: `arg` is the caller's NUL-terminated command line, and
    // `arg_augroup_get` only advances it over a name within that line.
    let mut group = unsafe { arg_augroup_get(&raw mut arg) };

    // A group name and nothing else is what is being completed, unless
    // it was already followed by a space.
    //
    // SAFETY: `arg` is inside the line, so it can be read.  `arg[-1]` is
    // only reached when a group name was skipped -- which is what the two
    // tests to its left say -- so it is that name's last byte.
    if unsafe { *arg } == 0
        && group != AUGROUP_ALL
        && !ascii_iswhite(unsafe { *arg.sub(1) } as ::core::ffi::c_int)
    {
        arg = start;
        group = AUGROUP_ALL;
    }

    // Skip over the event name, keeping the start of the last one in a
    // comma-separated list.
    let mut p = arg;
    // SAFETY: the walk stops at the line's NUL, so every step is in bounds.
    while unsafe { *p } != 0 && !ascii_iswhite(unsafe { *p } as ::core::ffi::c_int) {
        if unsafe { *p } == b',' as ::core::ffi::c_char {
            arg = unsafe { p.add(1) };
        }
        p = unsafe { p.add(1) };
    }
    if unsafe { *p } == 0 {
        if group == AUGROUP_ALL {
            autocmd_include_groups.set(true);
        }
        // SAFETY: `xp` is the caller's completion state, and `arg` points
        // into the line it is completing.
        unsafe { (*xp).xp_context = ExpandContext::Events };
        unsafe { (*xp).xp_pattern = arg };
        return ::core::ptr::null_mut();
    }

    // Skip over the pattern, whose whitespace may be backslash-escaped.
    //
    // SAFETY: `p` is at whitespace inside the line, so `skipwhite` answers a
    // position at or after it and `arg[-1]` is a byte of the same string.
    arg = unsafe { skipwhite(p) };
    while unsafe { *arg } != 0
        && (!ascii_iswhite(unsafe { *arg } as ::core::ffi::c_int)
            || unsafe { *arg.sub(1) } == b'\\' as ::core::ffi::c_char)
    {
        arg = unsafe { arg.add(1) };
    }
    if unsafe { *arg } != 0 {
        // What follows is the command, which the caller expands.
        return arg;
    }

    let context = if doautocmd {
        ExpandContext::Files
    } else {
        ExpandContext::Nothing
    };
    // SAFETY: `xp` is the caller's completion state.
    unsafe { (*xp).xp_context = context };
    ::core::ptr::null_mut()
}

/// `exists('#…')`, in all four shapes: `#Group`, `#Event`, `#Event#pat`
/// and `#Group#Event#pat`.
pub unsafe fn au_exists(arg: *const ::core::ffi::c_char) -> bool {
    // A copy, so the `#` separators can be overwritten with NULs.
    //
    // SAFETY: `arg` is the caller's NUL-terminated string, so `arg_save` is
    // a NUL-terminated copy of it this function owns and frees at the end.
    // Every pointer below is a position inside that copy.
    let arg_save = unsafe { xstrdup(arg) };
    let retval = 'theend: {
        let mut p = unsafe { strchr(arg_save, '#' as ::core::ffi::c_int) };
        if !p.is_null() {
            // SAFETY: `strchr` found a `#` here, so this byte is ours to
            // overwrite and there is at least a NUL after it.
            unsafe { *p = 0 };
            p = unsafe { p.add(1) };
        }

        // The first field is a group name if it names one, and an event
        // otherwise.
        let mut group = unsafe { augroup_find(arg_save) };
        let event_name = if group == AUGROUP_ERROR {
            group = AUGROUP_ALL;
            arg_save
        } else if p.is_null() {
            // Just "Group", and it exists.
            break 'theend true;
        } else {
            // "Group#Event" or "Group#Event#pat".
            let event_name = p;
            p = unsafe { strchr(event_name, '#' as ::core::ffi::c_int) };
            if !p.is_null() {
                // SAFETY: as above -- a `#` inside the copy.
                unsafe { *p = 0 };
                p = unsafe { p.add(1) };
            }
            event_name
        };

        // Null when no pattern was given.
        let pattern = p;
        // SAFETY: `event_name` is a NUL-terminated field of the copy, and
        // `p` is a local `event_name2nr` reports the name's end in.
        let event = unsafe { event_name2nr(event_name, &raw mut p) };
        if event == NUM_EVENTS {
            break 'theend false;
        }

        let acs = au_event_vec(event);
        // SAFETY: the event table's own row for `event`.
        if unsafe { (*acs).size } == 0 {
            break 'theend false;
        }

        // `<buffer>` means curbuf; `<buffer=N>` is already normalised,
        // so `path_fnamecmp` handles it.
        //
        // SAFETY: `pattern` is only read once it is known not to be null,
        // and it is NUL-terminated when it is not.
        let buflocal_buf =
            if !pattern.is_null() && unsafe { strcasecmp(pattern, c"<buffer>".as_ptr()) } == 0 {
                curbuf.get()
            } else {
                ::core::ptr::null_mut()
            };

        let mut i: usize = 0;
        // SAFETY: `i` is below the size just read, so `items.add(i)` is one
        // of the vector's rows; a non-null `pat` is a pattern it owns, and
        // `buflocal_buf` is either null or the current buffer.
        while i < unsafe { (*acs).size } {
            let ap = unsafe { (*(*acs).items.add(i)).pat };
            // Only a pattern that has not been removed counts.
            if !ap.is_null()
                && (group == AUGROUP_ALL || unsafe { (*ap).group } == group)
                && (pattern.is_null()
                    || if buflocal_buf.is_null() {
                        unsafe { path_fnamecmp((*ap).pat, pattern) == 0 }
                    } else {
                        unsafe { (*ap).buflocal_nr == (*buflocal_buf).handle }
                    })
            {
                break 'theend true;
            }
            i = i.wrapping_add(1);
        }
        false
    };

    // SAFETY: `xstrdup` allocated it and nothing else took it.
    unsafe { xfree(arg_save.cast::<::core::ffi::c_void>()) };
    retval
}
