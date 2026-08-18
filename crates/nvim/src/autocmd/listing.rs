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

/// `:autocmd` with no event: list every event's autocommands.
pub(crate) unsafe fn au_show_for_all_events(
    group: ::core::ffi::c_int,
    pat: *const ::core::ffi::c_char,
) {
    unsafe {
        for event in 0..NUM_EVENTS {
            au_show_for_event(group, event, pat);
        }
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
    unsafe {
        let acs = au_event_vec(event);
        if (*acs).size == 0 {
            return;
        }

        // An empty pattern shows every autocommand for the event.
        let mut patlen: ::core::ffi::c_int = 0;
        if *pat != 0 {
            patlen = aucmd_span_pattern(pat, &raw mut pat) as ::core::ffi::c_int;
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
            let endpat = pat.offset(patlen as isize);

            // `<buffer[=X]>` is normalised, as it was when it was defined,
            // so the comparison below can be a plain `strncmp`.
            if aupat_is_buflocal(pat, patlen) {
                aupat_normalize_buflocal_pat(
                    buflocal_pat.as_mut_ptr(),
                    pat,
                    patlen,
                    aupat_get_buflocal_nr(pat, patlen),
                );
                pat = buflocal_pat.as_ptr();
                patlen = strlen(buflocal_pat.as_ptr()) as ::core::ffi::c_int;
            }

            let mut i: usize = 0;
            while i < (*acs).size {
                let ac = (*acs).items.add(i);
                i = i.wrapping_add(1);

                let ap = (*ac).pat;
                // Skip a row `aucmd_del` has marked.
                if ap.is_null() {
                    continue;
                }
                // Accept the row when the group matches (or none was asked
                // for) and the pattern matches (or none was asked for).
                if (group != AUGROUP_ALL && (*ap).group != group)
                    || (patlen != 0
                        && ((*ap).patlen != patlen
                            || strncmp(pat, (*ap).pat, patlen as size_t) != 0))
                {
                    continue;
                }

                // The group and event headline only when the group changed.
                if (*ap).group != last_group {
                    last_group = (*ap).group;
                    last_group_name = augroup_name((*ap).group);

                    if got_int.get() {
                        return;
                    }
                    msg_putchar('\n' as ::core::ffi::c_int);
                    if got_int.get() {
                        return;
                    }
                    if (*ap).group != AUGROUP_DEFAULT {
                        // A group whose name is gone is one `:augroup!`
                        // renamed out from under its autocommands.
                        if last_group_name.is_null() {
                            msg_puts_hl(get_deleted_augroup(), HLF_E, false);
                        } else {
                            msg_puts_hl(last_group_name, HLF_T, false);
                        }
                        msg_puts(c"  ".as_ptr());
                    }
                    msg_puts_hl(event_nr2name(event), HLF_T, false);
                }

                // The pattern only when it changed.
                if last_ap != ap {
                    last_ap = ap;
                    msg_putchar('\n' as ::core::ffi::c_int);
                    if got_int.get() {
                        return;
                    }
                    msg_advance(4);
                    msg_outtrans((*ap).pat, 0, false);
                }

                if got_int.get() {
                    return;
                }
                if msg_col.get() >= 14 {
                    msg_putchar('\n' as ::core::ffi::c_int);
                }
                msg_advance(14);
                if got_int.get() {
                    return;
                }

                let handler_str = aucmd_handler_to_string(ac);
                if (*ac).desc.is_null() {
                    // A command is transliterated, a callback is not.
                    if (*ac).handler_cmd.is_null() {
                        msg_puts_hl(handler_str, HLF_8, false);
                    } else {
                        msg_outtrans(handler_str, 0, false);
                    }
                } else {
                    let msglen: size_t = 100;
                    let msg = xmallocz(msglen).cast::<::core::ffi::c_char>();
                    if (*ac).handler_cmd.is_null() {
                        msg_puts_hl(handler_str, HLF_8, false);
                        snprintf(msg, msglen, c" [%s]".as_ptr(), (*ac).desc);
                    } else {
                        snprintf(msg, msglen, c"%s [%s]".as_ptr(), handler_str, (*ac).desc);
                    }
                    msg_outtrans(msg, 0, false);
                    xfree(msg.cast::<::core::ffi::c_void>());
                }
                xfree(handler_str.cast::<::core::ffi::c_void>());

                if p_verbose.get() > 0 {
                    last_set_msg((*ac).script_ctx);
                }
                if got_int.get() {
                    return;
                }
            }

            patlen = aucmd_span_pattern(endpat, &raw mut pat) as ::core::ffi::c_int;
            if patlen == 0 {
                break;
            }
        }
    }
}

/// Whether any autocommand for `event` would match the file `sfname`
/// opened in `buf`.
pub unsafe fn has_autocmd(
    event: event_T,
    sfname: *mut ::core::ffi::c_char,
    buf: *mut buf_T,
) -> bool {
    unsafe {
        let tail = path_tail(sfname);
        let fname = FullName_save(sfname, false);
        if fname.is_null() {
            return false;
        }

        let acs = au_event_vec(event);
        let mut retval = false;
        let mut i: usize = 0;
        while i < (*acs).size {
            let ap = (*(*acs).items.add(i)).pat;
            if !ap.is_null() {
                // A buffer-local pattern is matched by buffer number, every
                // other one against the file name.
                let matched = if (*ap).buflocal_nr == 0 {
                    match_file_pat(
                        ::core::ptr::null_mut(),
                        &raw mut (*ap).reg_prog,
                        fname,
                        sfname,
                        tail,
                        (*ap).allow_dirs as ::core::ffi::c_int,
                    )
                } else {
                    !buf.is_null() && (*ap).buflocal_nr == (*buf).handle
                };
                if matched {
                    retval = true;
                    break;
                }
            }
            i = i.wrapping_add(1);
        }

        xfree(fname.cast::<::core::ffi::c_void>());
        retval
    }
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
    unsafe {
        // Skip a group name if there is one.
        autocmd_include_groups.set(false);
        let start = arg;
        let mut group = arg_augroup_get(&raw mut arg);

        // A group name and nothing else is what is being completed, unless
        // it was already followed by a space.
        if *arg == 0 && group != AUGROUP_ALL && !ascii_iswhite(*arg.sub(1) as ::core::ffi::c_int) {
            arg = start;
            group = AUGROUP_ALL;
        }

        // Skip over the event name, keeping the start of the last one in a
        // comma-separated list.
        let mut p = arg;
        while *p != 0 && !ascii_iswhite(*p as ::core::ffi::c_int) {
            if *p == b',' as ::core::ffi::c_char {
                arg = p.add(1);
            }
            p = p.add(1);
        }
        if *p == 0 {
            if group == AUGROUP_ALL {
                autocmd_include_groups.set(true);
            }
            (*xp).xp_context = EXPAND_EVENTS;
            (*xp).xp_pattern = arg;
            return ::core::ptr::null_mut();
        }

        // Skip over the pattern, whose whitespace may be backslash-escaped.
        arg = skipwhite(p);
        while *arg != 0
            && (!ascii_iswhite(*arg as ::core::ffi::c_int)
                || *arg.sub(1) == b'\\' as ::core::ffi::c_char)
        {
            arg = arg.add(1);
        }
        if *arg != 0 {
            // What follows is the command, which the caller expands.
            return arg;
        }

        (*xp).xp_context = if doautocmd {
            EXPAND_FILES
        } else {
            EXPAND_NOTHING
        };
        ::core::ptr::null_mut()
    }
}

/// `exists('#…')`, in all four shapes: `#Group`, `#Event`, `#Event#pat`
/// and `#Group#Event#pat`.
pub unsafe fn au_exists(arg: *const ::core::ffi::c_char) -> bool {
    unsafe {
        // A copy, so the `#` separators can be overwritten with NULs.
        let arg_save = xstrdup(arg);
        let retval = 'theend: {
            let mut p = strchr(arg_save, '#' as ::core::ffi::c_int);
            if !p.is_null() {
                *p = 0;
                p = p.add(1);
            }

            // The first field is a group name if it names one, and an event
            // otherwise.
            let mut group = augroup_find(arg_save);
            let event_name = if group == AUGROUP_ERROR {
                group = AUGROUP_ALL;
                arg_save
            } else if p.is_null() {
                // Just "Group", and it exists.
                break 'theend true;
            } else {
                // "Group#Event" or "Group#Event#pat".
                let event_name = p;
                p = strchr(event_name, '#' as ::core::ffi::c_int);
                if !p.is_null() {
                    *p = 0;
                    p = p.add(1);
                }
                event_name
            };

            // Null when no pattern was given.
            let pattern = p;
            let event = event_name2nr(event_name, &raw mut p);
            if event == NUM_EVENTS {
                break 'theend false;
            }

            let acs = au_event_vec(event);
            if (*acs).size == 0 {
                break 'theend false;
            }

            // `<buffer>` means curbuf; `<buffer=N>` is already normalised,
            // so `path_fnamecmp` handles it.
            let buflocal_buf =
                if !pattern.is_null() && strcasecmp(pattern, c"<buffer>".as_ptr()) == 0 {
                    curbuf.get()
                } else {
                    ::core::ptr::null_mut()
                };

            let mut i: usize = 0;
            while i < (*acs).size {
                let ap = (*(*acs).items.add(i)).pat;
                // Only a pattern that has not been removed counts.
                if !ap.is_null()
                    && (group == AUGROUP_ALL || (*ap).group == group)
                    && (pattern.is_null()
                        || if buflocal_buf.is_null() {
                            path_fnamecmp((*ap).pat, pattern) == 0
                        } else {
                            (*ap).buflocal_nr == (*buflocal_buf).handle
                        })
                {
                    break 'theend true;
                }
                i = i.wrapping_add(1);
            }
            false
        };

        xfree(arg_save.cast::<::core::ffi::c_void>());
        retval
    }
}
