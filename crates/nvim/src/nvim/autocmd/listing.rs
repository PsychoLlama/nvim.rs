//! Printing autocommands back, and asking whether one exists.
//!
//! `au_show_for_event` is `:autocmd`'s listing for one event -- the group
//! header, the pattern column, the command, and the `Last set from` line a
//! `:verbose` listing adds.  `au_exists` answers `exists('#Group#Event#pat')`
//! in all four of its shapes, `has_autocmd` the pattern query behind it,
//! and `set_context_in_autocmd` is command-line completion.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub(crate) unsafe extern "C" fn au_show_for_all_events(
    mut group: ::core::ffi::c_int,
    mut pat: *const ::core::ffi::c_char,
) {
    unsafe {
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            au_show_for_event(group, event, pat);
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    }
}

pub(crate) unsafe extern "C" fn au_show_for_event(
    mut group: ::core::ffi::c_int,
    mut event: event_T,
    mut pat: *const ::core::ffi::c_char,
) {
    unsafe {
        let acs: *mut AutoCmdVec =
            (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
        if (*acs).size == 0 as size_t {
            return;
        }
        let mut patlen: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        if *pat as ::core::ffi::c_int != NUL {
            patlen = aucmd_span_pattern(pat, &raw mut pat) as ::core::ffi::c_int;
            if patlen == 0 as ::core::ffi::c_int {
                return;
            }
        }
        let mut buflocal_pat: [::core::ffi::c_char; 25] = [0; 25];
        let mut last_group: ::core::ffi::c_int = AUGROUP_ERROR as ::core::ffi::c_int;
        let mut last_group_name: *const ::core::ffi::c_char =
            ::core::ptr::null::<::core::ffi::c_char>();
        loop {
            let mut last_ap: *mut AutoPat = ::core::ptr::null_mut::<AutoPat>();
            let mut endpat: *const ::core::ffi::c_char = pat.offset(patlen as isize);
            if aupat_is_buflocal(pat, patlen) {
                aupat_normalize_buflocal_pat(
                    &raw mut buflocal_pat as *mut ::core::ffi::c_char,
                    pat,
                    patlen,
                    aupat_get_buflocal_nr(pat, patlen),
                );
                pat = &raw mut buflocal_pat as *mut ::core::ffi::c_char;
                patlen =
                    strlen(&raw mut buflocal_pat as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
            }
            let mut i: size_t = 0 as size_t;
            while i < (*acs).size {
                let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
                if !(*ac).pat.is_null() {
                    if !(group != AUGROUP_ALL as ::core::ffi::c_int && (*(*ac).pat).group != group
                        || patlen != 0
                            && ((*(*ac).pat).patlen != patlen
                                || strncmp(pat, (*(*ac).pat).pat, patlen as size_t)
                                    != 0 as ::core::ffi::c_int))
                    {
                        if (*(*ac).pat).group != last_group {
                            last_group = (*(*ac).pat).group;
                            last_group_name = augroup_name((*(*ac).pat).group);
                            if got_int.get() {
                                return;
                            }
                            msg_putchar('\n' as ::core::ffi::c_int);
                            if got_int.get() {
                                return;
                            }
                            if (*(*ac).pat).group != AUGROUP_DEFAULT as ::core::ffi::c_int {
                                if last_group_name.is_null() {
                                    msg_puts_hl(get_deleted_augroup(), HLF_E, false_0 != 0);
                                } else {
                                    msg_puts_hl(last_group_name, HLF_T, false_0 != 0);
                                }
                                msg_puts(b"  \0".as_ptr() as *const ::core::ffi::c_char);
                            }
                            msg_puts_hl(event_nr2name(event), HLF_T, false_0 != 0);
                        }
                        if last_ap != (*ac).pat {
                            last_ap = (*ac).pat;
                            msg_putchar('\n' as ::core::ffi::c_int);
                            if got_int.get() {
                                return;
                            }
                            msg_advance(4 as ::core::ffi::c_int);
                            msg_outtrans((*(*ac).pat).pat, 0 as ::core::ffi::c_int, false_0 != 0);
                        }
                        if got_int.get() {
                            return;
                        }
                        if msg_col.get() >= 14 as ::core::ffi::c_int {
                            msg_putchar('\n' as ::core::ffi::c_int);
                        }
                        msg_advance(14 as ::core::ffi::c_int);
                        if got_int.get() {
                            return;
                        }
                        let mut handler_str: *mut ::core::ffi::c_char = aucmd_handler_to_string(ac);
                        if !(*ac).desc.is_null() {
                            let mut msglen: size_t = 100 as size_t;
                            let mut msg: *mut ::core::ffi::c_char =
                                xmallocz(msglen) as *mut ::core::ffi::c_char;
                            if !(*ac).handler_cmd.is_null() {
                                snprintf(
                                    msg,
                                    msglen,
                                    b"%s [%s]\0".as_ptr() as *const ::core::ffi::c_char,
                                    handler_str,
                                    (*ac).desc,
                                );
                            } else {
                                msg_puts_hl(handler_str, HLF_8, false_0 != 0);
                                snprintf(
                                    msg,
                                    msglen,
                                    b" [%s]\0".as_ptr() as *const ::core::ffi::c_char,
                                    (*ac).desc,
                                );
                            }
                            msg_outtrans(msg, 0 as ::core::ffi::c_int, false_0 != 0);
                            let mut ptr_: *mut *mut ::core::ffi::c_void =
                                &raw mut msg as *mut *mut ::core::ffi::c_void;
                            xfree(*ptr_);
                            *ptr_ = NULL_0;
                            let _ = *ptr_;
                        } else if !(*ac).handler_cmd.is_null() {
                            msg_outtrans(handler_str, 0 as ::core::ffi::c_int, false_0 != 0);
                        } else {
                            msg_puts_hl(handler_str, HLF_8, false_0 != 0);
                        }
                        let mut ptr__0: *mut *mut ::core::ffi::c_void =
                            &raw mut handler_str as *mut *mut ::core::ffi::c_void;
                        xfree(*ptr__0);
                        *ptr__0 = NULL_0;
                        let _ = *ptr__0;
                        if p_verbose.get() > 0 as OptInt {
                            last_set_msg((*ac).script_ctx);
                        }
                        if got_int.get() {
                            return;
                        }
                    }
                }
                i = i.wrapping_add(1);
            }
            patlen = aucmd_span_pattern(endpat, &raw mut pat) as ::core::ffi::c_int;
            if patlen == 0 {
                break;
            }
        }
    }
}

pub unsafe extern "C" fn has_autocmd(
    mut event: event_T,
    mut sfname: *mut ::core::ffi::c_char,
    mut buf: *mut buf_T,
) -> bool {
    unsafe {
        let mut tail: *mut ::core::ffi::c_char = path_tail(sfname);
        let mut retval: bool = false_0 != 0;
        let mut fname: *mut ::core::ffi::c_char = FullName_save(sfname, false_0 != 0);
        if fname.is_null() {
            return false_0 != 0;
        }
        let acs: *mut AutoCmdVec =
            (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
        let mut i: size_t = 0 as size_t;
        while i < (*acs).size {
            let ap: *mut AutoPat = (*(*acs).items.offset(i as isize)).pat;
            if !ap.is_null()
                && (if (*ap).buflocal_nr == 0 as ::core::ffi::c_int {
                    match_file_pat(
                        ::core::ptr::null_mut::<::core::ffi::c_char>(),
                        &raw mut (*ap).reg_prog,
                        fname,
                        sfname,
                        tail,
                        (*ap).allow_dirs as ::core::ffi::c_int,
                    ) as ::core::ffi::c_int
                } else {
                    (!buf.is_null() && (*ap).buflocal_nr == (*buf).handle) as ::core::ffi::c_int
                }) != 0
            {
                retval = true_0 != 0;
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
        xfree(fname as *mut ::core::ffi::c_void);
        return retval;
    }
}

pub unsafe extern "C" fn set_context_in_autocmd(
    mut xp: *mut expand_T,
    mut arg: *mut ::core::ffi::c_char,
    mut doautocmd: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        autocmd_include_groups.set(false_0 != 0);
        let mut p: *mut ::core::ffi::c_char = arg;
        let mut group: ::core::ffi::c_int = arg_augroup_get(&raw mut arg);
        if *arg as ::core::ffi::c_int == NUL
            && group != AUGROUP_ALL as ::core::ffi::c_int
            && !ascii_iswhite(*arg.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int)
        {
            arg = p;
            group = AUGROUP_ALL as ::core::ffi::c_int;
        }
        p = arg;
        while *p as ::core::ffi::c_int != NUL && !ascii_iswhite(*p as ::core::ffi::c_int) {
            if *p as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
                arg = p.offset(1 as ::core::ffi::c_int as isize);
            }
            p = p.offset(1);
        }
        if *p as ::core::ffi::c_int == NUL {
            if group == AUGROUP_ALL as ::core::ffi::c_int {
                autocmd_include_groups.set(true_0 != 0);
            }
            (*xp).xp_context = EXPAND_EVENTS as ::core::ffi::c_int;
            (*xp).xp_pattern = arg;
            return ::core::ptr::null_mut::<::core::ffi::c_char>();
        }
        arg = skipwhite(p);
        while *arg as ::core::ffi::c_int != 0
            && (!ascii_iswhite(*arg as ::core::ffi::c_int)
                || *arg.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                    == '\\' as ::core::ffi::c_int)
        {
            arg = arg.offset(1);
        }
        if *arg != 0 {
            return arg;
        }
        if doautocmd {
            (*xp).xp_context = EXPAND_FILES as ::core::ffi::c_int;
        } else {
            (*xp).xp_context = EXPAND_NOTHING as ::core::ffi::c_int;
        }
        return ::core::ptr::null_mut::<::core::ffi::c_char>();
    }
}

pub unsafe extern "C" fn au_exists(arg: *const ::core::ffi::c_char) -> bool {
    unsafe {
        let mut pattern: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut event: event_T = EVENT_BUFADD;
        let mut acs: *mut AutoCmdVec = ::core::ptr::null_mut::<AutoCmdVec>();
        let mut buflocal_buf: *mut buf_T = ::core::ptr::null_mut::<buf_T>();
        let mut retval: bool = false_0 != 0;
        let arg_save: *mut ::core::ffi::c_char = xstrdup(arg);
        let mut p: *mut ::core::ffi::c_char = strchr(arg_save, '#' as ::core::ffi::c_int);
        if !p.is_null() {
            let c2rust_fresh13 = p;
            p = p.offset(1);
            *c2rust_fresh13 = NUL as ::core::ffi::c_char;
        }
        let mut group: ::core::ffi::c_int = augroup_find(arg_save);
        let mut event_name: *mut ::core::ffi::c_char =
            ::core::ptr::null_mut::<::core::ffi::c_char>();
        '_theend: {
            if group == AUGROUP_ERROR as ::core::ffi::c_int {
                group = AUGROUP_ALL as ::core::ffi::c_int;
                event_name = arg_save;
            } else if p.is_null() {
                retval = true_0 != 0;
                break '_theend;
            } else {
                event_name = p;
                p = strchr(event_name, '#' as ::core::ffi::c_int);
                if !p.is_null() {
                    let c2rust_fresh14 = p;
                    p = p.offset(1);
                    *c2rust_fresh14 = NUL as ::core::ffi::c_char;
                }
            }
            pattern = p;
            event = event_name2nr(event_name, &raw mut p);
            if event as ::core::ffi::c_uint
                != NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
            {
                acs = (autocmds.ptr() as *mut AutoCmdVec)
                    .offset(event as ::core::ffi::c_int as isize);
                if (*acs).size != 0 as size_t {
                    if !pattern.is_null()
                        && strcasecmp(
                            pattern,
                            b"<buffer>\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                        ) == 0 as ::core::ffi::c_int
                    {
                        buflocal_buf = curbuf.get();
                    }
                    let mut i: size_t = 0 as size_t;
                    while i < (*acs).size {
                        let ap: *mut AutoPat = (*(*acs).items.offset(i as isize)).pat;
                        if !ap.is_null()
                            && (group == AUGROUP_ALL as ::core::ffi::c_int || (*ap).group == group)
                            && (pattern.is_null()
                                || (if buflocal_buf.is_null() {
                                    (path_fnamecmp((*ap).pat, pattern) == 0 as ::core::ffi::c_int)
                                        as ::core::ffi::c_int
                                } else {
                                    ((*ap).buflocal_nr == (*buflocal_buf).handle)
                                        as ::core::ffi::c_int
                                }) != 0)
                        {
                            retval = true_0 != 0;
                            break;
                        } else {
                            i = i.wrapping_add(1);
                        }
                    }
                }
            }
        }
        xfree(arg_save as *mut ::core::ffi::c_void);
        return retval;
    }
}
