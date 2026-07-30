//! The command modifiers (`:silent`, `:verbose`, `:tab`,
//! `:keeppatterns`, the split direction, `:filter`, …): recognising them,
//! putting them in force around the command, and taking them out again.

#[allow(unused_imports)]
use super::*;

pub unsafe extern "C" fn cmd_has_expr_args(mut cmdidx: cmdidx_T) -> bool {
    return cmdidx as c_int == CMD_execute as c_int
        || cmdidx as c_int == CMD_echo as c_int
        || cmdidx as c_int == CMD_echon as c_int
        || cmdidx as c_int == CMD_echomsg as c_int
        || cmdidx as c_int == CMD_echoerr as c_int;
}

pub unsafe extern "C" fn parse_command_modifiers(
    mut eap: *mut exarg_T,
    mut errormsg: *mut *const c_char,
    mut cmod: *mut cmdmod_T,
    mut skip_only: bool,
) -> c_int {
    let mut orig_cmd: *mut c_char = (*eap).cmd;
    let mut cmd_start: *mut c_char = ::core::ptr::null_mut::<c_char>();
    let mut use_plus_cmd: bool = false_0 != 0;
    let mut has_visual_range: bool = false_0 != 0;
    memset(
        cmod as *mut c_void,
        0 as c_int,
        ::core::mem::size_of::<cmdmod_T>(),
    );
    if strncmp(
        (*eap).cmd,
        b"'<,'>\0".as_ptr() as *const c_char,
        5 as size_t,
    ) == 0 as c_int
    {
        let mut p: *const c_char = skipwhite((*eap).cmd.offset(5 as c_int as isize));
        if *p as c_int != NUL && *p as c_int != '|' as c_int {
            (*eap).cmd = (*eap).cmd.offset(5 as c_int as isize);
            cmd_start = (*eap).cmd;
            has_visual_range = true_0 != 0;
        }
    }
    loop {
        while *(*eap).cmd as c_int == ' ' as c_int
            || *(*eap).cmd as c_int == '\t' as c_int
            || *(*eap).cmd as c_int == ':' as c_int
        {
            (*eap).cmd = (*eap).cmd.offset(1);
        }
        if *(*eap).cmd as c_int == NUL
            && exmode_active.get() as c_int != 0
            && getline_equal(
                (*eap).ea_getline,
                (*eap).cookie,
                Some(
                    getexline
                        as unsafe extern "C" fn(c_int, *mut c_void, c_int, bool) -> *mut c_char,
                ),
            ) as c_int
                != 0
            && (*curwin.get()).w_cursor.lnum < (*curbuf.get()).b_ml.ml_line_count
        {
            (*eap).cmd = exmode_plus.ptr() as *mut c_char;
            use_plus_cmd = true_0 != 0;
            if !skip_only {
                ex_pressedreturn.set(true_0 != 0);
            }
            break;
        } else {
            if *(*eap).cmd as c_int == '"' as c_int {
                (*eap).nextcmd = vim_strchr((*eap).cmd, '\n' as c_int);
                if !(*eap).nextcmd.is_null() {
                    (*eap).nextcmd = (*eap).nextcmd.offset(1);
                }
                return FAIL;
            }
            if *(*eap).cmd as c_int == '\n' as c_int {
                (*eap).nextcmd = (*eap).cmd.offset(1 as c_int as isize);
                return FAIL;
            }
            if *(*eap).cmd as c_int == NUL {
                if !skip_only {
                    ex_pressedreturn.set(true_0 != 0);
                }
                return FAIL;
            }
            let mut p_0: *mut c_char = skip_range((*eap).cmd, ::core::ptr::null_mut::<c_int>());
            match *p_0 as c_int {
                97 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"aboveleft\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_split |= WSP_ABOVE as c_int;
                }
                98 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"belowright\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_split |= WSP_BELOW as c_int;
                    } else if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"browse\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_BROWSE as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"botright\0".as_ptr() as *const c_char,
                            2 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_split |= WSP_BOT as c_int;
                    }
                }
                99 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"confirm\0".as_ptr() as *const c_char,
                        4 as c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_flags |= CMOD_CONFIRM as c_int;
                }
                107 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"keepmarks\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_KEEPMARKS as c_int;
                    } else if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"keepalt\0".as_ptr() as *const c_char,
                        5 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_KEEPALT as c_int;
                    } else if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"keeppatterns\0".as_ptr() as *const c_char,
                        5 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_KEEPPATTERNS as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"keepjumps\0".as_ptr() as *const c_char,
                            5 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_flags |= CMOD_KEEPJUMPS as c_int;
                    }
                }
                102 => {
                    let mut reg_pat: *mut c_char = ::core::ptr::null_mut::<c_char>();
                    if !checkforcmd(
                        &raw mut p_0,
                        b"filter\0".as_ptr() as *const c_char,
                        4 as c_int,
                    ) || *p_0 as c_int == NUL
                        || ends_excmd(*p_0 as c_int) != 0
                    {
                        break;
                    }
                    if *p_0 as c_int == '!' as c_int {
                        (*cmod).cmod_filter_force = true_0 != 0;
                        p_0 = skipwhite(p_0.offset(1 as c_int as isize));
                        if *p_0 as c_int == NUL || ends_excmd(*p_0 as c_int) != 0 {
                            break;
                        }
                    }
                    if skip_only {
                        p_0 = skip_vimgrep_pat(
                            p_0,
                            ::core::ptr::null_mut::<*mut c_char>(),
                            ::core::ptr::null_mut::<c_int>(),
                        );
                    } else {
                        p_0 = skip_vimgrep_pat(
                            p_0,
                            &raw mut reg_pat,
                            ::core::ptr::null_mut::<c_int>(),
                        );
                    }
                    if p_0.is_null() || *p_0 as c_int == NUL {
                        break;
                    }
                    if !skip_only {
                        (*cmod).cmod_filter_pat = xstrdup(reg_pat);
                        (*cmod).cmod_filter_regmatch.regprog = vim_regcomp(reg_pat, RE_MAGIC);
                        if (*cmod).cmod_filter_regmatch.regprog.is_null() {
                            break;
                        }
                    }
                    (*eap).cmd = p_0;
                }
                104 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"horizontal\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_split |= WSP_HOR as c_int;
                    } else {
                        if p_0 != (*eap).cmd
                            || !checkforcmd(
                                &raw mut p_0,
                                b"hide\0".as_ptr() as *const c_char,
                                3 as c_int,
                            )
                            || *p_0 as c_int == NUL
                            || ends_excmd(*p_0 as c_int) != 0
                        {
                            break;
                        }
                        (*eap).cmd = p_0;
                        (*cmod).cmod_flags |= CMOD_HIDE as c_int;
                    }
                }
                108 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"lockmarks\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_LOCKMARKS as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"leftabove\0".as_ptr() as *const c_char,
                            5 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_split |= WSP_ABOVE as c_int;
                    }
                }
                110 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"noautocmd\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_NOAUTOCMD as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"noswapfile\0".as_ptr() as *const c_char,
                            3 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_flags |= CMOD_NOSWAPFILE as c_int;
                    }
                }
                114 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"rightbelow\0".as_ptr() as *const c_char,
                        6 as c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_split |= WSP_BELOW as c_int;
                }
                115 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"sandbox\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        (*cmod).cmod_flags |= CMOD_SANDBOX as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"silent\0".as_ptr() as *const c_char,
                            3 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_flags |= CMOD_SILENT as c_int;
                        if *(*eap).cmd as c_int == '!' as c_int
                            && !ascii_iswhite(*(*eap).cmd.offset(-1 as c_int as isize) as c_int)
                        {
                            (*eap).cmd = skipwhite((*eap).cmd.offset(1 as c_int as isize));
                            (*cmod).cmod_flags |= CMOD_ERRSILENT as c_int;
                        }
                    }
                }
                116 => {
                    if checkforcmd(&raw mut p_0, b"tab\0".as_ptr() as *const c_char, 3 as c_int) {
                        if !skip_only {
                            let mut tabnr: c_int = get_address(
                                eap,
                                &raw mut (*eap).cmd,
                                ADDR_TABS,
                                (*eap).skip != 0,
                                skip_only,
                                false_0,
                                1 as c_int,
                                errormsg,
                            ) as c_int;
                            if (*eap).cmd.is_null() {
                                return false_0;
                            }
                            if tabnr == MAXLNUM as c_int {
                                (*cmod).cmod_tab = tabpage_index(curtab.get()) + 1 as c_int;
                            } else {
                                if tabnr < 0 as c_int
                                    || tabnr > current_tab_nr(::core::ptr::null_mut::<tabpage_T>())
                                {
                                    *errormsg = gettext(&raw const e_invrange as *const c_char);
                                    return false_0;
                                }
                                (*cmod).cmod_tab = tabnr + 1 as c_int;
                            }
                        }
                        (*eap).cmd = p_0;
                    } else {
                        if !checkforcmd(
                            &raw mut (*eap).cmd,
                            b"topleft\0".as_ptr() as *const c_char,
                            2 as c_int,
                        ) {
                            break;
                        }
                        (*cmod).cmod_split |= WSP_TOP as c_int;
                    }
                }
                117 => {
                    if !checkforcmd(
                        &raw mut (*eap).cmd,
                        b"unsilent\0".as_ptr() as *const c_char,
                        3 as c_int,
                    ) {
                        break;
                    }
                    (*cmod).cmod_flags |= CMOD_UNSILENT as c_int;
                }
                118 => {
                    if checkforcmd(
                        &raw mut (*eap).cmd,
                        b"vertical\0".as_ptr() as *const c_char,
                        4 as c_int,
                    ) {
                        (*cmod).cmod_split |= WSP_VERT as c_int;
                    } else {
                        if !checkforcmd(
                            &raw mut p_0,
                            b"verbose\0".as_ptr() as *const c_char,
                            4 as c_int,
                        ) {
                            break;
                        }
                        if ascii_isdigit(*(*eap).cmd as c_int) {
                            (*cmod).cmod_verbose = atoi((*eap).cmd) + 1 as c_int;
                        } else {
                            (*cmod).cmod_verbose = 2 as c_int;
                        }
                        (*eap).cmd = p_0;
                    }
                }
                _ => {
                    break;
                }
            }
        }
    }
    if has_visual_range {
        if (*eap).cmd > cmd_start {
            if use_plus_cmd {
                let mut len: size_t = strlen(cmd_start);
                memmove(orig_cmd as *mut c_void, cmd_start as *const c_void, len);
                xmemcpyz(
                    orig_cmd.offset(len as isize) as *mut c_void,
                    b" *+\0".as_ptr() as *const c_char as *const c_void,
                    ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as size_t),
                );
            } else {
                memmove(
                    cmd_start.offset(-(5 as c_int as isize)) as *mut c_void,
                    cmd_start as *const c_void,
                    (*eap).cmd.offset_from(cmd_start) as size_t,
                );
                (*eap).cmd = (*eap).cmd.offset(-(5 as c_int as isize));
                memmove(
                    (*eap).cmd.offset(-(1 as c_int as isize)) as *mut c_void,
                    b":'<,'>\0".as_ptr() as *const c_char as *const c_void,
                    6 as size_t,
                );
            }
        } else if use_plus_cmd {
            (*eap).cmd = b"'<,'>+\0".as_ptr() as *const c_char as *mut c_char;
        } else {
            (*eap).cmd = orig_cmd;
        }
    } else if use_plus_cmd {
        (*eap).cmd = exmode_plus.ptr() as *mut c_char;
    }
    return OK;
}

pub unsafe extern "C" fn apply_cmdmod(mut cmod: *mut cmdmod_T) {
    if (*cmod).cmod_flags & CMOD_SANDBOX as c_int != 0 && (*cmod).cmod_did_sandbox == 0 {
        (*sandbox.ptr()) += 1;
        (*cmod).cmod_did_sandbox = true_0;
    }
    if (*cmod).cmod_verbose > 0 as c_int {
        if (*cmod).cmod_verbose_save == 0 as OptInt {
            (*cmod).cmod_verbose_save = p_verbose.get() + 1 as OptInt;
        }
        p_verbose.set(((*cmod).cmod_verbose - 1 as c_int) as OptInt);
    }
    if (*cmod).cmod_flags & (CMOD_SILENT as c_int | CMOD_UNSILENT as c_int) != 0
        && (*cmod).cmod_save_msg_silent == 0 as c_int
    {
        (*cmod).cmod_save_msg_silent = msg_silent.get() + 1 as c_int;
        (*cmod).cmod_save_msg_scroll = msg_scroll.get();
    }
    if (*cmod).cmod_flags & CMOD_SILENT as c_int != 0 {
        (*msg_silent.ptr()) += 1;
    }
    if (*cmod).cmod_flags & CMOD_UNSILENT as c_int != 0 {
        msg_silent.set(0 as c_int);
    }
    if (*cmod).cmod_flags & CMOD_ERRSILENT as c_int != 0 {
        (*emsg_silent.ptr()) += 1;
        (*cmod).cmod_did_esilent += 1;
    }
    if (*cmod).cmod_flags & CMOD_NOAUTOCMD as c_int != 0 && (*cmod).cmod_save_ei.is_null() {
        (*cmod).cmod_save_ei = xstrdup(p_ei.get());
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: String_0 {
                        data: b"all\0".as_ptr() as *const c_char as *mut c_char,
                        size: ::core::mem::size_of::<[c_char; 4]>().wrapping_sub(1 as size_t),
                    },
                },
            },
            0 as c_int,
            SID_NONE,
        );
    }
}

pub unsafe extern "C" fn undo_cmdmod(mut cmod: *mut cmdmod_T) {
    if (*cmod).cmod_verbose_save > 0 as OptInt {
        p_verbose.set((*cmod).cmod_verbose_save - 1 as OptInt);
        (*cmod).cmod_verbose_save = 0 as OptInt;
    }
    if (*cmod).cmod_did_sandbox != 0 {
        (*sandbox.ptr()) -= 1;
        (*cmod).cmod_did_sandbox = false_0;
    }
    if !(*cmod).cmod_save_ei.is_null() {
        set_option_direct(
            kOptEventignore,
            OptVal {
                type_0: kOptValTypeString,
                data: OptValData {
                    string: cstr_as_string((*cmod).cmod_save_ei),
                },
            },
            0 as c_int,
            SID_NONE,
        );
        free_string_option((*cmod).cmod_save_ei);
        (*cmod).cmod_save_ei = ::core::ptr::null_mut::<c_char>();
    }
    xfree((*cmod).cmod_filter_pat as *mut c_void);
    vim_regfree((*cmod).cmod_filter_regmatch.regprog);
    if (*cmod).cmod_save_msg_silent > 0 as c_int {
        if did_emsg.get() == 0 || msg_silent.get() > (*cmod).cmod_save_msg_silent - 1 as c_int {
            msg_silent.set((*cmod).cmod_save_msg_silent - 1 as c_int);
        }
        (*emsg_silent.ptr()) -= (*cmod).cmod_did_esilent;
        emsg_silent.set(if emsg_silent.get() > 0 as c_int {
            emsg_silent.get()
        } else {
            0 as c_int
        });
        msg_scroll.set((*cmod).cmod_save_msg_scroll);
        if redirecting() != 0 {
            msg_col.set(0 as c_int);
        }
        (*cmod).cmod_save_msg_silent = 0 as c_int;
        (*cmod).cmod_did_esilent = 0 as c_int;
    }
}

pub unsafe extern "C" fn modifier_len(mut cmd: *mut c_char) -> c_int {
    let mut p: *mut c_char = cmd;
    if ascii_isdigit(*cmd as c_int) {
        p = skipwhite(skipdigits(cmd.offset(1 as c_int as isize)));
    }
    let mut i: c_int = 0 as c_int;
    while i < ::core::mem::size_of::<[cmdmod; 24]>()
        .wrapping_div(::core::mem::size_of::<cmdmod>())
        .wrapping_div(
            (::core::mem::size_of::<[cmdmod; 24]>().wrapping_rem(::core::mem::size_of::<cmdmod>())
                == 0) as c_int as usize,
        ) as c_int
    {
        let mut j: c_int = 0;
        j = 0 as c_int;
        while *p.offset(j as isize) as c_int != NUL {
            if *p.offset(j as isize) as c_int
                != *(*cmdmods.ptr())[i as usize].name.offset(j as isize) as c_int
            {
                break;
            }
            j += 1;
        }
        if j >= (*cmdmods.ptr())[i as usize].minlen
            && !(*p.offset(j as isize) as c_uint >= 'A' as c_uint
                && *p.offset(j as isize) as c_uint <= 'Z' as c_uint
                || *p.offset(j as isize) as c_uint >= 'a' as c_uint
                    && *p.offset(j as isize) as c_uint <= 'z' as c_uint)
            && (p == cmd || (*cmdmods.ptr())[i as usize].has_count != 0)
        {
            return j + p.offset_from(cmd) as c_int;
        }
        i += 1;
    }
    return 0 as c_int;
}

pub unsafe extern "C" fn expr_map_locked() -> bool {
    return expr_map_lock.get() > 0 as c_int && (*curbuf.get()).b_flags & BF_DUMMY == 0;
}

pub unsafe extern "C" fn is_loclist_cmd(mut cmdidx: c_int) -> bool {
    if cmdidx < 0 as c_int || cmdidx >= CMD_SIZE as c_int {
        return false_0 != 0;
    }
    return *(*cmdnames.ptr())[cmdidx as usize]
        .cmd_name
        .offset(0 as c_int as isize) as c_int
        == 'l' as c_int;
}

pub unsafe extern "C" fn is_map_cmd(mut cmdidx: cmdidx_T) -> bool {
    if (cmdidx as c_int) < 0 as c_int {
        return false_0 != 0;
    }
    let mut func: ex_func_T = (*cmdnames.ptr())[cmdidx as usize].cmd_func;
    return ex_func_is(func, ex_map)
        || ex_func_is(func, ex_unmap)
        || ex_func_is(func, ex_mapclear)
        || ex_func_is(func, ex_abbreviate)
        || ex_func_is(func, ex_abclear);
}
