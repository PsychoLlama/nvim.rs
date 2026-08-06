//! `:autocmd` itself -- parsing it, and installing what it says.
//!
//! `do_autocmd` splits the command into group, event list, pattern and
//! command, then loops `do_autocmd_event` over the events; `autocmd_register`
//! is the one place an `AutoPat`/`AutoCmd` pair is created, and the same
//! entry point the API's `nvim_create_autocmd` reaches.  The `arg_*`
//! helpers are the pieces of the parse the API shares, and
//! `autocmd_delete_id` is the deletion by id that `nvim_del_autocmd` is.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn do_autocmd(
    mut eap: *mut exarg_T,
    mut arg_in: *mut ::core::ffi::c_char,
    mut forceit: ::core::ffi::c_int,
) {
    unsafe {
        let mut arg: *mut ::core::ffi::c_char = arg_in;
        let mut envpat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut cmd: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut need_free: bool = false_0 != 0;
        let mut nested: bool = false_0 != 0;
        let mut once: bool = false_0 != 0;
        let mut group: ::core::ffi::c_int = 0;
        if *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
            (*eap).nextcmd = arg.offset(1 as ::core::ffi::c_int as isize);
            arg = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            group = AUGROUP_ALL as ::core::ffi::c_int;
        } else {
            group = arg_augroup_get(&raw mut arg);
        }
        let mut pat: *mut ::core::ffi::c_char =
            arg_event_skip(arg, group != AUGROUP_ALL as ::core::ffi::c_int);
        if pat.is_null() {
            return;
        }
        pat = skipwhite(pat);
        if *pat as ::core::ffi::c_int == '|' as ::core::ffi::c_int {
            (*eap).nextcmd = pat.offset(1 as ::core::ffi::c_int as isize);
            pat = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
            cmd = b"\0".as_ptr() as *const ::core::ffi::c_char as *mut ::core::ffi::c_char;
        } else {
            cmd = pat;
            while *cmd as ::core::ffi::c_int != 0
                && (!ascii_iswhite(*cmd as ::core::ffi::c_int)
                    || *cmd.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int)
            {
                cmd = cmd.offset(1);
            }
            if *cmd != 0 {
                let c2rust_fresh1 = cmd;
                cmd = cmd.offset(1);
                *c2rust_fresh1 = NUL as ::core::ffi::c_char;
            }
            if !vim_strchr(pat, '$' as ::core::ffi::c_int).is_null()
                || !vim_strchr(pat, '~' as ::core::ffi::c_int).is_null()
            {
                envpat = expand_env_save(pat);
                if !envpat.is_null() {
                    pat = envpat;
                }
            }
            cmd = skipwhite(cmd);
            let mut invalid_flags: bool = false_0 != 0;
            let mut i: size_t = 0 as size_t;
            while i < 2 as size_t {
                if *cmd as ::core::ffi::c_int != NUL {
                    invalid_flags = invalid_flags as ::core::ffi::c_int
                        | arg_autocmd_flag_get(
                            &raw mut once,
                            &raw mut cmd,
                            b"++once\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            6 as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                        != 0;
                    invalid_flags = invalid_flags as ::core::ffi::c_int
                        | arg_autocmd_flag_get(
                            &raw mut nested,
                            &raw mut cmd,
                            b"++nested\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            8 as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                        != 0;
                    invalid_flags = invalid_flags as ::core::ffi::c_int
                        | arg_autocmd_flag_get(
                            &raw mut nested,
                            &raw mut cmd,
                            b"nested\0".as_ptr() as *const ::core::ffi::c_char
                                as *mut ::core::ffi::c_char,
                            6 as ::core::ffi::c_int,
                        ) as ::core::ffi::c_int
                        != 0;
                }
                i = i.wrapping_add(1);
            }
            if invalid_flags {
                return;
            }
            if *cmd as ::core::ffi::c_int != NUL {
                cmd = expand_sfile(cmd);
                if cmd.is_null() {
                    return;
                }
                need_free = true_0 != 0;
            }
        }
        let is_showing: bool = forceit == 0 && *cmd as ::core::ffi::c_int == NUL;
        if is_showing {
            msg_ext_set_kind(b"list_cmd\0".as_ptr() as *const ::core::ffi::c_char);
            msg_puts_title(gettext(
                b"\n--- Autocommands ---\0".as_ptr() as *const ::core::ffi::c_char
            ));
            if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int
                || *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int
                || *arg as ::core::ffi::c_int == NUL
            {
                au_show_for_all_events(group, pat);
            } else {
                let mut event: event_T = event_name2nr(arg, &raw mut arg);
                '_c2rust_label: {
                    if (event as ::core::ffi::c_uint)
                        < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                    } else {
                        __assert_fail(
                            b"event < NUM_EVENTS\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            860 as ::core::ffi::c_uint,
                            b"void do_autocmd(exarg_T *, char *, int)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                au_show_for_event(group, event, pat);
            }
        } else if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int
            || *arg as ::core::ffi::c_int == NUL
            || *arg as ::core::ffi::c_int == '|' as ::core::ffi::c_int
        {
            if *cmd as ::core::ffi::c_int != NUL {
                emsg(gettext(
                    &raw const e_cannot_define_autocommands_for_all_events
                        as *const ::core::ffi::c_char,
                ));
            } else {
                do_all_autocmd_events(
                    pat,
                    once,
                    nested as ::core::ffi::c_int,
                    cmd,
                    forceit != 0,
                    group,
                );
            }
        } else {
            while *arg as ::core::ffi::c_int != 0
                && *arg as ::core::ffi::c_int != '|' as ::core::ffi::c_int
                && !ascii_iswhite(*arg as ::core::ffi::c_int)
            {
                let mut event_0: event_T = event_name2nr(arg, &raw mut arg);
                '_c2rust_label_0: {
                    if (event_0 as ::core::ffi::c_uint)
                        < NUM_EVENTS as ::core::ffi::c_int as ::core::ffi::c_uint
                    {
                    } else {
                        __assert_fail(
                            b"event < NUM_EVENTS\0".as_ptr() as *const ::core::ffi::c_char,
                            b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                            873 as ::core::ffi::c_uint,
                            b"void do_autocmd(exarg_T *, char *, int)\0".as_ptr()
                                as *const ::core::ffi::c_char,
                        );
                    }
                };
                if do_autocmd_event(
                    event_0,
                    pat,
                    once,
                    nested as ::core::ffi::c_int,
                    cmd,
                    forceit != 0,
                    group,
                ) == FAIL
                {
                    break;
                }
            }
        }
        if need_free {
            xfree(cmd as *mut ::core::ffi::c_void);
        }
        xfree(envpat as *mut ::core::ffi::c_void);
    }
}

pub unsafe extern "C" fn do_all_autocmd_events(
    mut pat: *const ::core::ffi::c_char,
    mut once: bool,
    mut nested: ::core::ffi::c_int,
    mut cmd: *mut ::core::ffi::c_char,
    mut del: bool,
    mut group: ::core::ffi::c_int,
) {
    unsafe {
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            if do_autocmd_event(event, pat, once, nested, cmd, del, group) == FAIL {
                return;
            }
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
    }
}

pub unsafe extern "C" fn do_autocmd_event(
    mut event: event_T,
    mut pat: *const ::core::ffi::c_char,
    mut once: bool,
    mut nested: ::core::ffi::c_int,
    mut cmd: *const ::core::ffi::c_char,
    mut del: bool,
    mut group: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        '_c2rust_label: {
            if *pat as ::core::ffi::c_int != '\0' as ::core::ffi::c_int
                || del as ::core::ffi::c_int != 0
            {
            } else {
                __assert_fail(
                b"*pat != NUL || del\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                908 as ::core::ffi::c_uint,
                b"int do_autocmd_event(event_T, const char *, _Bool, int, const char *, _Bool, int)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
            }
        };
        let mut buflocal_pat: [::core::ffi::c_char; 25] = [0; 25];
        let mut is_adding_cmd: bool = *cmd as ::core::ffi::c_int != NUL;
        let findgroup: ::core::ffi::c_int = if group == AUGROUP_ALL as ::core::ffi::c_int {
            current_augroup.get()
        } else {
            group
        };
        if *pat as ::core::ffi::c_int == NUL && del as ::core::ffi::c_int != 0 {
            aucmd_del_for_event_and_group(event, findgroup);
            return OK;
        }
        let mut patlen: ::core::ffi::c_int =
            aucmd_span_pattern(pat, &raw mut pat) as ::core::ffi::c_int;
        while patlen != 0 {
            let mut endpat: *const ::core::ffi::c_char = pat.offset(patlen as isize);
            let mut is_buflocal: bool = aupat_is_buflocal(pat, patlen);
            if is_buflocal {
                let buflocal_nr: ::core::ffi::c_int = aupat_get_buflocal_nr(pat, patlen);
                aupat_normalize_buflocal_pat(
                    &raw mut buflocal_pat as *mut ::core::ffi::c_char,
                    pat,
                    patlen,
                    buflocal_nr,
                );
                pat = &raw mut buflocal_pat as *mut ::core::ffi::c_char;
                patlen =
                    strlen(&raw mut buflocal_pat as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
            }
            if del {
                '_c2rust_label_0: {
                    if *pat as ::core::ffi::c_int != '\0' as ::core::ffi::c_int {
                    } else {
                        __assert_fail(
                        b"*pat != NUL\0".as_ptr() as *const ::core::ffi::c_char,
                        b"src/nvim/autocmd.rs\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                        939 as ::core::ffi::c_uint,
                        b"int do_autocmd_event(event_T, const char *, _Bool, int, const char *, _Bool, int)\0"
                            .as_ptr() as *const ::core::ffi::c_char,
                    );
                    }
                };
                let acs: *mut AutoCmdVec = (autocmds.ptr() as *mut AutoCmdVec)
                    .offset(event as ::core::ffi::c_int as isize);
                let mut i: size_t = 0 as size_t;
                while i < (*acs).size {
                    let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
                    let ap: *mut AutoPat = (*ac).pat;
                    if !ap.is_null()
                        && (*ap).group == findgroup
                        && (*ap).patlen == patlen
                        && strncmp(pat, (*ap).pat, patlen as size_t) == 0 as ::core::ffi::c_int
                    {
                        aucmd_del(ac);
                    }
                    i = i.wrapping_add(1);
                }
            }
            if is_adding_cmd {
                let mut handler_fn: Callback = Callback {
                    data: C2Rust_Unnamed_5 {
                        funcref: ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    },
                    type_0: kCallbackNone,
                };
                autocmd_register(
                    0 as int64_t,
                    event,
                    pat,
                    patlen,
                    group,
                    once,
                    nested != 0,
                    ::core::ptr::null_mut::<::core::ffi::c_char>(),
                    cmd,
                    &raw mut handler_fn,
                );
            }
            patlen = aucmd_span_pattern(endpat, &raw mut pat) as ::core::ffi::c_int;
        }
        au_cleanup();
        return OK;
    }
}

pub unsafe extern "C" fn autocmd_register(
    mut id: int64_t,
    mut event: event_T,
    mut pat: *const ::core::ffi::c_char,
    mut patlen: ::core::ffi::c_int,
    mut group: ::core::ffi::c_int,
    mut once: bool,
    mut nested: bool,
    mut desc: *mut ::core::ffi::c_char,
    mut handler_cmd: *const ::core::ffi::c_char,
    mut handler_fn: *mut Callback,
) -> ::core::ffi::c_int {
    unsafe {
        '_c2rust_label: {
            if group != 0 as ::core::ffi::c_int {
            } else {
                __assert_fail(
                b"group != 0\0".as_ptr() as *const ::core::ffi::c_char,
                b"src/nvim/autocmd.rs\0".as_ptr()
                    as *const ::core::ffi::c_char,
                984 as ::core::ffi::c_uint,
                b"int autocmd_register(int64_t, event_T, const char *, int, int, _Bool, _Bool, char *, const char *, Callback *)\0"
                    .as_ptr() as *const ::core::ffi::c_char,
            );
            }
        };
        if patlen > strlen(pat) as ::core::ffi::c_int {
            return FAIL;
        }
        let findgroup: ::core::ffi::c_int = if group == AUGROUP_ALL as ::core::ffi::c_int {
            current_augroup.get()
        } else {
            group
        };
        let is_buflocal: bool = aupat_is_buflocal(pat, patlen);
        let mut buflocal_nr: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        let mut buflocal_pat: [::core::ffi::c_char; 25] = [0; 25];
        if is_buflocal {
            buflocal_nr = aupat_get_buflocal_nr(pat, patlen);
            aupat_normalize_buflocal_pat(
                &raw mut buflocal_pat as *mut ::core::ffi::c_char,
                pat,
                patlen,
                buflocal_nr,
            );
            pat = &raw mut buflocal_pat as *mut ::core::ffi::c_char;
            patlen =
                strlen(&raw mut buflocal_pat as *mut ::core::ffi::c_char) as ::core::ffi::c_int;
        }
        let mut ap: *mut AutoPat = ::core::ptr::null_mut::<AutoPat>();
        let acs: *mut AutoCmdVec =
            (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
        let mut i: ptrdiff_t = (*acs).size as ptrdiff_t - 1 as ptrdiff_t;
        while i >= 0 as ptrdiff_t {
            ap = (*(*acs).items.offset(i as isize)).pat;
            if ap.is_null() {
                i -= 1;
            } else {
                if (*ap).group != findgroup
                    || (*ap).patlen != patlen
                    || strncmp(pat, (*ap).pat, patlen as size_t) != 0 as ::core::ffi::c_int
                {
                    ap = ::core::ptr::null_mut::<AutoPat>();
                }
                break;
            }
        }
        if ap.is_null() {
            if is_buflocal as ::core::ffi::c_int != 0
                && (buflocal_nr == 0 as ::core::ffi::c_int || buflist_findnr(buflocal_nr).is_null())
            {
                semsg(
                    gettext(b"E680: <buffer=%d>: invalid buffer number \0".as_ptr()
                        as *const ::core::ffi::c_char),
                    buflocal_nr,
                );
                return FAIL;
            }
            ap = xmalloc(::core::mem::size_of::<AutoPat>()) as *mut AutoPat;
            if is_buflocal {
                (*ap).buflocal_nr = buflocal_nr;
                (*ap).reg_prog = ::core::ptr::null_mut::<regprog_T>();
            } else {
                (*ap).buflocal_nr = 0 as ::core::ffi::c_int;
                let mut reg_pat: *mut ::core::ffi::c_char = file_pat_to_reg_pat(
                    pat,
                    pat.offset(patlen as isize),
                    &raw mut (*ap).allow_dirs,
                    true_0,
                );
                if !reg_pat.is_null() {
                    (*ap).reg_prog = vim_regcomp(reg_pat, RE_MAGIC);
                }
                xfree(reg_pat as *mut ::core::ffi::c_void);
                if reg_pat.is_null() || (*ap).reg_prog.is_null() {
                    xfree(ap as *mut ::core::ffi::c_void);
                    return FAIL;
                }
            }
            (*ap).refcount = 0 as size_t;
            (*ap).pat = xmemdupz(pat as *const ::core::ffi::c_void, patlen as size_t)
                as *mut ::core::ffi::c_char;
            (*ap).patlen = patlen;
            if event as ::core::ffi::c_uint
                == EVENT_MODECHANGED as ::core::ffi::c_int as ::core::ffi::c_uint
                && !has_event(EVENT_MODECHANGED)
            {
                get_mode(last_mode.ptr() as *mut ::core::ffi::c_char);
            }
            if event as ::core::ffi::c_uint
                == EVENT_CURSORMOVED as ::core::ffi::c_int as ::core::ffi::c_uint
                && !has_event(EVENT_CURSORMOVED)
                || event as ::core::ffi::c_uint
                    == EVENT_CURSORMOVEDI as ::core::ffi::c_int as ::core::ffi::c_uint
                    && !has_event(EVENT_CURSORMOVEDI)
            {
                last_cursormoved_win.set(curwin.get());
                last_cursormoved.set((*curwin.get()).w_cursor);
            }
            if (event as ::core::ffi::c_uint
                == EVENT_WINSCROLLED as ::core::ffi::c_int as ::core::ffi::c_uint
                || event as ::core::ffi::c_uint
                    == EVENT_WINRESIZED as ::core::ffi::c_int as ::core::ffi::c_uint)
                && !(has_event(EVENT_WINSCROLLED) as ::core::ffi::c_int != 0
                    || has_event(EVENT_WINRESIZED) as ::core::ffi::c_int != 0)
            {
                let mut save_curtab: *mut tabpage_T = curtab.get();
                let mut tp: *mut tabpage_T = first_tabpage.get() as *mut tabpage_T;
                while !tp.is_null() {
                    unuse_tabpage(curtab.get());
                    use_tabpage(tp as *mut tabpage_T);
                    snapshot_windows_scroll_size();
                    tp = (*tp).tp_next as *mut tabpage_T;
                }
                unuse_tabpage(curtab.get());
                use_tabpage(save_curtab);
            }
            (*ap).group = if group == AUGROUP_ALL as ::core::ffi::c_int {
                current_augroup.get()
            } else {
                group
            };
        }
        (*ap).refcount = (*ap).refcount.wrapping_add(1);
        if (*autocmds.ptr())[event as ::core::ffi::c_int as usize].size
            == (*autocmds.ptr())[event as ::core::ffi::c_int as usize].capacity
        {
            (*autocmds.ptr())[event as ::core::ffi::c_int as usize].capacity =
                if (*autocmds.ptr())[event as ::core::ffi::c_int as usize].capacity != 0 {
                    (*autocmds.ptr())[event as ::core::ffi::c_int as usize].capacity
                        << 1 as ::core::ffi::c_int
                } else {
                    8 as size_t
                };
            (*autocmds.ptr())[event as ::core::ffi::c_int as usize].items = xrealloc(
                (*autocmds.ptr())[event as ::core::ffi::c_int as usize].items
                    as *mut ::core::ffi::c_void,
                ::core::mem::size_of::<AutoCmd>()
                    .wrapping_mul((*autocmds.ptr())[event as ::core::ffi::c_int as usize].capacity),
            )
                as *mut AutoCmd;
        } else {
        };
        let c2rust_fresh2 = (*autocmds.ptr())[event as ::core::ffi::c_int as usize].size;
        (*autocmds.ptr())[event as ::core::ffi::c_int as usize].size = (*autocmds.ptr())
            [event as ::core::ffi::c_int as usize]
            .size
            .wrapping_add(1);
        let mut ac: *mut AutoCmd = (*autocmds.ptr())[event as ::core::ffi::c_int as usize]
            .items
            .offset(c2rust_fresh2 as isize);
        (*ac).pat = ap;
        (*ac).id = id;
        if !handler_cmd.is_null() {
            (*ac).handler_cmd = xstrdup(handler_cmd);
        } else {
            (*ac).handler_cmd = ::core::ptr::null_mut::<::core::ffi::c_char>();
            callback_copy(&raw mut (*ac).handler_fn, handler_fn);
        }
        (*ac).script_ctx = current_sctx.get();
        (*ac).script_ctx.sc_lnum += (*((*exestack.ptr()).ga_data as *mut estack_T)
            .offset(((*exestack.ptr()).ga_len - 1 as ::core::ffi::c_int) as isize))
        .es_lnum;
        nlua_set_sctx(&raw mut (*ac).script_ctx);
        (*ac).once = once;
        (*ac).nested = nested;
        (*ac).desc = if desc.is_null() {
            ::core::ptr::null_mut::<::core::ffi::c_char>()
        } else {
            xstrdup(desc)
        };
        return OK;
    }
}

pub unsafe extern "C" fn aucmd_span_pattern(
    mut pat: *const ::core::ffi::c_char,
    mut start: *mut *const ::core::ffi::c_char,
) -> size_t {
    unsafe {
        while *pat as ::core::ffi::c_int == ',' as ::core::ffi::c_int {
            pat = pat.offset(1);
        }
        let mut p: *const ::core::ffi::c_char = pat;
        let mut brace_level: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
        while *p as ::core::ffi::c_int != 0
            && (*p as ::core::ffi::c_int != ',' as ::core::ffi::c_int
                || brace_level != 0
                || p > pat
                    && *p.offset(-1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                        == '\\' as ::core::ffi::c_int)
        {
            if *p as ::core::ffi::c_int == '{' as ::core::ffi::c_int {
                brace_level += 1;
            } else if *p as ::core::ffi::c_int == '}' as ::core::ffi::c_int {
                brace_level -= 1;
            }
            p = p.offset(1);
        }
        *start = pat;
        return p.offset_from(pat) as size_t;
    }
}

pub unsafe extern "C" fn check_nomodeline(mut argp: *mut *mut ::core::ffi::c_char) -> bool {
    unsafe {
        if strncmp(
            *argp,
            b"<nomodeline>\0".as_ptr() as *const ::core::ffi::c_char,
            12 as size_t,
        ) == 0 as ::core::ffi::c_int
        {
            *argp = skipwhite((*argp).offset(12 as ::core::ffi::c_int as isize));
            return false_0 != 0;
        }
        return true_0 != 0;
    }
}

pub unsafe extern "C" fn autocmd_delete_id(mut id: int64_t) -> bool {
    unsafe {
        '_c2rust_label: {
            if id > 0 as int64_t {
            } else {
                __assert_fail(
                    b"id > 0\0".as_ptr() as *const ::core::ffi::c_char,
                    b"src/nvim/autocmd.rs\0".as_ptr() as *const ::core::ffi::c_char,
                    2560 as ::core::ffi::c_uint,
                    b"_Bool autocmd_delete_id(int64_t)\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
        };
        let mut success: bool = false_0 != 0;
        let mut event: event_T = EVENT_BUFADD;
        while (event as ::core::ffi::c_int) < NUM_EVENTS as ::core::ffi::c_int {
            let acs: *mut AutoCmdVec =
                (autocmds.ptr() as *mut AutoCmdVec).offset(event as ::core::ffi::c_int as isize);
            let mut i: size_t = 0 as size_t;
            while i < (*acs).size {
                let ac: *mut AutoCmd = (*acs).items.offset(i as isize);
                if (*ac).id == id {
                    aucmd_del(ac);
                    success = true_0 != 0;
                }
                i = i.wrapping_add(1);
            }
            event = (event as ::core::ffi::c_int + 1 as ::core::ffi::c_int) as event_T;
        }
        return success;
    }
}

pub unsafe extern "C" fn aucmd_handler_to_string(mut ac: *mut AutoCmd) -> *mut ::core::ffi::c_char {
    unsafe {
        if !(*ac).handler_cmd.is_null() {
            return xstrdup((*ac).handler_cmd);
        }
        return callback_to_string(&raw mut (*ac).handler_fn, ::core::ptr::null_mut::<Arena>());
    }
}

pub(crate) unsafe extern "C" fn arg_event_skip(
    mut arg: *mut ::core::ffi::c_char,
    mut have_group: bool,
) -> *mut ::core::ffi::c_char {
    unsafe {
        let mut pat: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        let mut p: *mut ::core::ffi::c_char = ::core::ptr::null_mut::<::core::ffi::c_char>();
        if *arg as ::core::ffi::c_int == '*' as ::core::ffi::c_int {
            if *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int != 0
                && !ascii_iswhite(
                    *arg.offset(1 as ::core::ffi::c_int as isize) as ::core::ffi::c_int
                )
            {
                semsg(
                    gettext(b"E215: Illegal character after *: %s\0".as_ptr()
                        as *const ::core::ffi::c_char),
                    arg,
                );
                return ::core::ptr::null_mut::<::core::ffi::c_char>();
            }
            pat = arg.offset(1 as ::core::ffi::c_int as isize);
        } else {
            pat = arg;
            while *pat as ::core::ffi::c_int != 0
                && *pat as ::core::ffi::c_int != '|' as ::core::ffi::c_int
                && !ascii_iswhite(*pat as ::core::ffi::c_int)
            {
                if event_name2nr(pat, &raw mut p) as ::core::ffi::c_int
                    >= NUM_EVENTS as ::core::ffi::c_int
                {
                    if have_group {
                        semsg(
                            gettext(
                                b"E216: No such event: %s\0".as_ptr() as *const ::core::ffi::c_char
                            ),
                            pat,
                        );
                    } else {
                        semsg(
                            gettext(b"E216: No such group or event: %s\0".as_ptr()
                                as *const ::core::ffi::c_char),
                            pat,
                        );
                    }
                    return ::core::ptr::null_mut::<::core::ffi::c_char>();
                }
                pat = p;
            }
        }
        return pat;
    }
}

unsafe extern "C" fn arg_autocmd_flag_get(
    mut flag: *mut bool,
    mut cmd_ptr: *mut *mut ::core::ffi::c_char,
    mut pattern: *mut ::core::ffi::c_char,
    mut len: ::core::ffi::c_int,
) -> bool {
    unsafe {
        if strncmp(*cmd_ptr, pattern, len as size_t) == 0 as ::core::ffi::c_int
            && ascii_iswhite(*(*cmd_ptr).offset(len as isize) as ::core::ffi::c_int)
                as ::core::ffi::c_int
                != 0
        {
            if *flag {
                semsg(
                    gettext(&raw const e_duparg2 as *const ::core::ffi::c_char),
                    pattern,
                );
                return true_0 != 0;
            }
            *flag = true_0 != 0;
            *cmd_ptr = skipwhite((*cmd_ptr).offset(len as isize));
        }
        return false_0 != 0;
    }
}
